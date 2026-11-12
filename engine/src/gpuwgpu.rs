//! B1 GPU half (production path): an instanced uber-shader renderer over
//! **wgpu** — the owner-approved GPU dependency (Apache-2.0 OR MIT).
//!
//! This module supersedes the Metal FFI prototype (`gpumetal.rs`, removed):
//! the prototype proved the pipeline end to end and surfaced the Metal 4
//! dialect defects (multi-instance draws drop instances, sRGB-store surprises,
//! removed accessors); wgpu drives the same OS Metal backend through a
//! maintained API, so the single instanced draw call works at 100k nodes.
//!
//! Design (identical data model to the CPU oracle path):
//! - `build_instances` (shared via `render::gpu_common`) flattens/annotates
//!   the evaluated lines into segment-patch instances — the exact ribbon
//!   offsets the CPU `width_strip` uses (miter parity).
//! - One instanced draw (6 verts per instance) through the WGSL uber-shader.
//! - Offscreen Rgba8Unorm render + blit into a MAP_READ buffer (row-padded
//!   to 256 as Metal/wgpu require) → de-padded RGBA8 `Image`.
//! - No depth buffer, no MSAA, paint order = document order — matching the
//!   CPU oracle's 2D pipeline.
//!
//! The CPU oracle remains the source of truth; this path is verified by the
//! B1 GPU parity gate (`gate_b1gpu`).

use crate::render::Image;
use pollster::block_on;

/// GPU uniform block — layout must match the WGSL `Uniforms`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    vp: [[f32; 4]; 4],  // column-major VP (same matrix as the CPU oracle)
    viewport: [f32; 4], // width, height, 0, 0
}

/// GPU per-instance data — layout must match the WGSL `Instance`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Instance {
    pub a: [f32; 4],  // endpoint A world xyz + half stroke width px
    pub b: [f32; 4],  // endpoint B world xyz + half stroke width px
    pub ca: [f32; 4], // premultiplied rgba at A
    pub cb: [f32; 4], // premultiplied rgba at B
    pub kind_c: [f32; 4], // kind (0 stroke quad / 1 fill tri) + fill corner C xyz
    pub off: [f32; 4], // stroke: per-endpoint ribbon offset (dir×miter) px per half-width unit
}

/// The uber-shader: stroke quads expand in screen pixels around projected
/// segments using the CPU ribbon offsets; fill triangles project their three
/// world corners. Both go through the CPU oracle's own VP matrix.
const KIND_STROKE: f32 = 0.0;
const KIND_FILL: f32 = 1.0;

const SHADER_SOURCE: &str = r##"
struct Uniforms {
    vp: mat4x4<f32>,
    viewport: vec4<f32>,
};

struct Instance {
    a: vec4<f32>,
    b: vec4<f32>,
    ca: vec4<f32>,
    cb: vec4<f32>,
    kind_c: vec4<f32>,
    off: vec4<f32>,
};

struct RasterOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> instances: array<Instance>;
@group(0) @binding(1) var<uniform> u: Uniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) iid: u32,
) -> RasterOut {
    let inst = instances[iid];
    var out: RasterOut;
    let kind = inst.kind_c.x;
    if (kind < 0.5) {
        // Stroke quad: screen-space expansion around the projected segment,
        // using the CPU ribbon's per-endpoint offsets (miter parity).
        let ca4 = u.vp * vec4<f32>(inst.a.xyz, 1.0);
        let cb4 = u.vp * vec4<f32>(inst.b.xyz, 1.0);
        var a_px = (ca4.xy / max(ca4.w, 1e-6) * 0.5 + 0.5) * u.viewport.xy;
        var b_px = (cb4.xy / max(cb4.w, 1e-6) * 0.5 + 0.5) * u.viewport.xy;
        a_px.y = u.viewport.y - a_px.y; // screen y down (CPU oracle mapping)
        b_px.y = u.viewport.y - b_px.y;
        let dir = b_px - a_px;
        let len = length(dir);
        var n = vec2<f32>(0.0, 1.0);
        if (len > 1e-9) {
            n = vec2<f32>(-dir.y, dir.x) / len;
        }
        let c = vid % 6u;
        var t = 1.0;
        if (c == 0u || c == 1u || c == 3u) {
            t = 0.0;
        }
        var side = -1.0;
        if (c == 0u || c == 2u || c == 5u) {
            side = 1.0;
        }
        let o = mix(inst.off.xy, inst.off.zw, t);
        let px = mix(a_px, b_px, t) + o * side * mix(inst.a.w, inst.b.w, t);
        let ndc = px / u.viewport.xy * 2.0 - 1.0;
        out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
        out.color = mix(inst.ca, inst.cb, t);
    } else {
        // Fill triangle: three world corners, flat premultiplied color.
        let fc = vid % 3u;
        var p = inst.a.xyz;
        if (fc == 1u) {
            p = inst.b.xyz;
        }
        if (fc == 2u) {
            p = inst.kind_c.yzw;
        }
        out.position = u.vp * vec4<f32>(p, 1.0);
        out.color = inst.ca;
    }
    return out;
}

@fragment
fn fs_main(in: RasterOut) -> @location(0) vec4<f32> {
    return in.color;
}
"##;

/// A headless wgpu renderer (instance/adapter/device/queue + pipeline,
/// created once; textures and readback buffers are sized per frame).
pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_capacity: u64, // bytes
    /// Adapter name (diagnostics).
    pub device_name: String,
    bind_group: wgpu::BindGroup,
}

impl WgpuRenderer {
    /// Create the renderer on the best available adapter (Metal on macOS).
    /// Err when no compatible adapter exists — callers must skip cleanly.
    pub fn new() -> Result<WgpuRenderer, String> {
        let instance = wgpu::Instance::default();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| format!("no compatible GPU adapter: {e}"))?;
        let info = adapter.get_info();
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("mo2d"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
                ..Default::default()
            },
        ))
        .map_err(|e| format!("device request failed: {e}"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mo2d uber-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mo2d bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mo2d pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mo2d uber-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mo2d uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_capacity = std::mem::size_of::<Instance>() as u64 * 1024; // start at 1024 instances
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mo2d instances"),
            size: instance_capacity,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = Self::make_bind_group(&device, &bind_group_layout, &uniform_buf, &instance_buf, instance_capacity);

        let device_name = adapter.get_info().name;
        let _ = info;
        Ok(WgpuRenderer {
            device,
            queue,
            pipeline,
            bind_group_layout,
            uniform_buf,
            instance_buf,
            instance_capacity,
            bind_group,
            device_name,
        })
    }

    fn make_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform_buf: &wgpu::Buffer,
        instance_buf: &wgpu::Buffer,
        instance_size: u64,
    ) -> wgpu::BindGroup {
        let _ = instance_size;
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mo2d bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
        })
    }

    /// Render one evaluated frame on the GPU (offscreen RGBA8, read back).
    /// One instanced draw call covers every instance of the frame.
    pub fn render_frame(
        &mut self,
        lines: &[crate::eval::EvalLine],
        world_w: f64,
        world_h: f64,
        opts: &crate::render::RenderOpts,
    ) -> Result<Image, String> {
        let (view, proj) = opts.camera.matrices(world_w, world_h, opts.width as f64 / opts.height as f64);
        let vp = proj.mul(&view);
        let uniforms = Uniforms {
            vp: std::array::from_fn(|c| std::array::from_fn(|r| vp.0[c][r] as f32)),
            viewport: [opts.width as f32, opts.height as f32, 0.0, 0.0],
        };
        let instances = build_instances(lines, &vp, opts.width);

        // Grow the instance storage buffer if needed.
        let need = instances.len() as u64 * std::mem::size_of::<Instance>() as u64;
        if need > self.instance_capacity {
            self.instance_capacity = (need * 2).max(1024 * 96);
            self.instance_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("mo2d instances"),
                size: self.instance_capacity,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.bind_group = Self::make_bind_group(
                &self.device,
                &self.bind_group_layout,
                &self.uniform_buf,
                &self.instance_buf,
                self.instance_capacity,
            );
        }
        if !instances.is_empty() {
            let bytes = instances_as_bytes(&instances);
            self.queue.write_buffer(&self.instance_buf, 0, &bytes);
        }
        self.queue.write_buffer(&self.uniform_buf, 0, bytes_of(&uniforms));

        // Offscreen color texture.
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mo2d frame"),
            size: wgpu::Extent3d { width: opts.width, height: opts.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Readback buffer (rows padded to wgpu::COPY_BYTES_PER_ROW_ALIGNMENT 256).
        let bytes_per_row = ((opts.width * 4) as usize).div_ceil(256) * 256;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mo2d readback"),
            size: (bytes_per_row * opts.height as usize) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mo2d frame"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mo2d frame pass"),
                multiview_mask: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..6, 0..instances.len() as u32);
        }
        {
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row as u32),
                        rows_per_image: Some(opts.height),
                    },
                },
                wgpu::Extent3d { width: opts.width, height: opts.height, depth_or_array_layers: 1 },
            );
        }
        self.queue.submit(Some(encoder.finish()));

        // Map the readback buffer (wait for completion).
        let (tx, rx) = std::sync::mpsc::channel();
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().map_err(|e| format!("map channel closed: {e}"))?.map_err(|e| format!("map failed: {e}"))?;

        // De-pad rows into the Image.
        let mut px = vec![0u8; (opts.width * opts.height * 4) as usize];
        {
            let data = slice
                .get_mapped_range()
                .map_err(|e| format!("mapped range failed: {e}"))?;
            for y in 0..opts.height as usize {
                let src = y * bytes_per_row;
                let dst = y * (opts.width as usize * 4);
                px[dst..dst + opts.width as usize * 4].copy_from_slice(&data[src..src + opts.width as usize * 4]);
            }
        }
        readback.unmap();

        Ok(Image { w: opts.width, h: opts.height, px })
    }
}

fn premul(c: [u8; 4]) -> [f32; 4] {
    let a = c[3] as f32 / 255.0;
    [c[0] as f32 / 255.0 * a, c[1] as f32 / 255.0 * a, c[2] as f32 / 255.0 * a, a]
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>()) }
}

fn instances_as_bytes(instances: &[Instance]) -> Vec<u8> {
    let mut out = Vec::with_capacity(instances.len() * std::mem::size_of::<Instance>());
    for i in instances {
        out.extend_from_slice(bytes_of(i));
    }
    out
}

/// Build the frame's instance list from evaluated lines: stroke quads (one
/// per flattened segment — "segment patches = instances", roadmap B1) and
/// fill triangles, in the CPU oracle's paint order (per line: fill, then
/// stroke; lines in document order). The stroke quads replicate the CPU
/// oracle's width_strip ribbon offsets (per-endpoint averaged normals with
/// the miter clamp) so the GPU draws the same quad geometry per segment.
pub fn build_instances(lines: &[crate::eval::EvalLine], vp: &crate::math::Mat4, w: u32) -> Vec<Instance> {
    let mut out = Vec::new();
    for line in lines {
        if line.tags.iter().any(|t| t == "track") || line.visible != true || line.nodes.len() < 2 {
            continue;
        }
        let mut flat = crate::render::gpu_common::flatten(line);
        if line.closed && flat.len() > 1 && (flat[0].p.sub(flat[flat.len() - 1].p).len()) < 1e-9 {
            flat.pop();
        }
        if line.closed && line.fill_on {
            for tri in crate::render::gpu_common::triangulate(&flat) {
                out.push(Instance {
                    a: [tri[0].0.x as f32, tri[0].0.y as f32, tri[0].0.z as f32, 0.0],
                    b: [tri[1].0.x as f32, tri[1].0.y as f32, tri[1].0.z as f32, 0.0],
                    ca: premul(tri[0].1),
                    cb: premul(tri[0].1),
                    kind_c: [KIND_FILL, tri[2].0.x as f32, tri[2].0.y as f32, tri[2].0.z as f32],
                    off: [0.0; 4],
                });
            }
        }
        if !line.stroke_on || flat.len() < 2 {
            continue;
        }
        let n = flat.len();
        let seg_count = if line.closed { n } else { n - 1 };

        // Screen positions + per-point half-width px + per-segment normals —
        // the exact computation of the CPU oracle's width_strip.
        let mut scr: Vec<(f64, f64, f64)> = Vec::with_capacity(n);
        for pt in &flat {
            if let Some((bx, by)) = crate::render::gpu_common::project_keep(vp, w, pt.p) {
                let scale = crate::render::gpu_common::px_per_world(vp, w, pt.p);
                scr.push((bx, by, scale * pt.width * 0.5));
            } else {
                scr.push((0.0, 0.0, 0.0));
            }
        }
        let mut seg_norm: Vec<(f64, f64)> = vec![(0.0, 0.0); seg_count];
        for i in 0..seg_count {
            let (ia, ib) = (i, (i + 1) % n);
            let dx = scr[ib].0 - scr[ia].0;
            let dy = scr[ib].1 - scr[ia].1;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-9 {
                seg_norm[i] = (0.0, 0.0);
            } else {
                seg_norm[i] = (-dy / len, dx / len);
            }
        }
        // Per-point ribbon offset direction×miter-scale (width_strip parity).
        let mut off: Vec<(f64, f64)> = vec![(0.0, 0.0); n];
        for i in 0..n {
            if !line.closed && i == 0 {
                off[0] = seg_norm[0];
                continue;
            }
            if !line.closed && i == n - 1 {
                off[n - 1] = seg_norm[seg_count - 1];
                continue;
            }
            let prev = if i == 0 { seg_count - 1 } else { i - 1 };
            let next = if line.closed { i % seg_count } else { i.min(seg_count - 1) };
            let (pa, pb) = (seg_norm[prev], seg_norm[next]);
            let mut nx = pa.0 + pb.0;
            let mut ny = pa.1 + pb.1;
            let nl = (nx * nx + ny * ny).sqrt();
            if nl < 1e-9 {
                nx = pa.0;
                ny = pa.1;
            } else {
                nx /= nl;
                ny /= nl;
            }
            let cos_half = (nx * pa.0 + ny * pa.1).abs().max(1e-6);
            let scale = (1.0 / cos_half).min(4.0);
            off[i] = (nx * scale, ny * scale);
        }

        for k in 0..seg_count {
            let a = &flat[k];
            let b = &flat[(k + 1) % n];
            let ha = scr[k].2 as f32;
            let hb = scr[(k + 1) % n].2 as f32;
            out.push(Instance {
                a: [a.p.x as f32, a.p.y as f32, a.p.z as f32, ha],
                b: [b.p.x as f32, b.p.y as f32, b.p.z as f32, hb],
                ca: premul(a.color),
                cb: premul(b.color),
                kind_c: [KIND_STROKE, 0.0, 0.0, 0.0],
                off: [off[k].0 as f32, off[k].1 as f32, off[(k + 1) % n].0 as f32, off[(k + 1) % n].1 as f32],
            });
        }
    }
    out
}

/// Deterministic CPU/GPU parity metric (the B1 gate): sample a fixed grid of
/// pixels; a sample passes when every RGB channel differs by ≤ `tol`. GPU
/// rasterization edge rules differ from software rasterization by nature —
/// parity is a tolerance contract, not byte equality.
pub struct ParityReport {
    pub checked: usize,
    pub within_tol: usize,
    pub worst: i32,
    /// Max channel diff per sample, sorted ascending (percentile source).
    pub diffs: Vec<i32>,
}

impl ParityReport {
    /// Fraction of samples whose worst channel diff is ≤ `tol`.
    pub fn rate_within(&self, tol: i32) -> f64 {
        if self.checked == 0 {
            return 0.0;
        }
        let n = self.diffs.partition_point(|d| *d <= tol);
        n as f64 / self.checked as f64
    }

    /// Per-sample worst diff at percentile `p` (0..=100).
    pub fn percentile(&self, p: usize) -> i32 {
        if self.diffs.is_empty() {
            return 0;
        }
        self.diffs[(p.min(100) * (self.diffs.len() - 1)) / 100.max(1)]
    }
}

pub fn parity_report(cpu: &Image, gpu: &Image, grid_step: u32, tol: i32) -> ParityReport {
    let mut checked = 0usize;
    let mut within = 0usize;
    let mut worst = 0i32;
    let mut diffs: Vec<i32> = Vec::new();
    for y in (0..cpu.h.min(gpu.h)).step_by(grid_step.max(1) as usize) {
        for x in (0..cpu.w.min(gpu.w)).step_by(grid_step.max(1) as usize) {
            let idx = ((y * cpu.w + x) * 4) as usize;
            let mut sample_worst = 0i32;
            for ch in 0..3 {
                let d = (cpu.px[idx + ch] as i32 - gpu.px[idx + ch] as i32).abs();
                sample_worst = sample_worst.max(d);
            }
            worst = worst.max(sample_worst);
            diffs.push(sample_worst);
            checked += 1;
            if sample_worst <= tol {
                within += 1;
            }
        }
    }
    diffs.sort_unstable();
    ParityReport { checked, within_tol: within, worst, diffs }
}
