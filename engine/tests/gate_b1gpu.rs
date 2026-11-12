//! Gate B1-GPU — the GPU half of roadmap B1, now on the owner-approved wgpu
//! path (Apache-2.0 OR MIT; approved 2026-11-12, replacing the zero-crate
//! Metal FFI prototype whose findings live in log.md).
//!
//! 1. construct + encode: the full pipeline runs (instanced draw, blit
//!    readback, map) with no validation errors
//! 2. CPU/GPU pixel parity gate: identical scene through both pipelines,
//!    sampled on a fixed grid — the gate asserts ≥ 0.95 within tol 40
//!    (p50/p90 = 0 expected: interiors pixel-match; only miter-joint and
//!    rasterization-edge pixels legitimately differ)
//! 3. 100k-node class at 60 fps: one instanced draw call, median < 16 ms

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::gpuwgpu::{parity_report, WgpuRenderer};
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render_with_assets, Camera, RenderOpts};
use std::time::Instant;

fn opts(w: u32, h: u32) -> RenderOpts {
    RenderOpts { width: w, height: h, camera: Camera::Ortho2d, samples: 1 }
}

/// A content-rich frame: variable-width stroke + filled closed line + colors
/// (exercises stroke quads, fill triangles, paint order, blending).
fn scene_doc() -> Document {
    let stream = r#"{"v":1,"cmds":[
      {"op":"doc.setRoot","width":320,"height":180},
      {"op":"line.create","id":"blob","closed":true,"width":4,"strokeOn":true,"fillOn":true,
       "fillColor":[220,120,40,255],"color":[40,40,40,255],"nodes":[
        {"p":[40,40,0],"w":6},{"p":[140,30,0],"w":4},{"p":[180,90,0],"w":3},{"p":[120,130,0],"w":5},{"p":[50,110,0],"w":4}]},
      {"op":"line.create","id":"ribbon","width":8,"color":[30,90,200,255],"nodes":[
        {"p":[200,150,0],"w":12},{"p":[240,90,0],"w":4},{"p":[280,60,0],"w":9},{"p":[310,30,0],"w":3}]},
      {"op":"line.create","id":"dot","closed":true,"strokeOn":false,"fillOn":true,
       "fillColor":[200,30,30,255],"nodes":[
        {"p":[100,60,0]},{"p":[112,60,0]},{"p":[112,72,0]},{"p":[100,72,0]}]}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    doc
}

fn skip_if_no_gpu() -> Option<WgpuRenderer> {
    match WgpuRenderer::new() {
        Ok(g) => {
            eprintln!("B1 GPU backend: {}", g.device_name);
            Some(g)
        }
        Err(e) => {
            eprintln!("SKIP: no GPU backend on this platform ({}), CPU oracle gates still apply", e);
            None
        }
    }
}

#[test]
fn gpu_backend_constructs_and_encodes() {
    let Some(mut gpu) = skip_if_no_gpu() else { return };
    let doc = scene_doc();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let img = gpu.render_frame(&lines, 320.0, 180.0, &opts(320, 180)).expect("GPU frame encodes + completes");
    assert_eq!((img.w, img.h), (320, 180), "frame dimensions match the render target");
    // Background corners stay white (the clear); content pixels exist.
    let g = |x: u32, y: u32| {
        let o = ((y * img.w + x) * 4) as usize;
        [img.px[o], img.px[o + 1], img.px[o + 2]]
    };
    assert_eq!(g(0, 0), [255, 255, 255], "background stays white");
    assert_eq!(g(319, 179), [255, 255, 255]);
    let mut non_white = 0;
    for y in 0..img.h {
        for x in 0..img.w {
            if g(x, y) != [255, 255, 255] {
                non_white += 1;
            }
        }
    }
    assert!(non_white > 2000, "content must be visible ({} non-white px)", non_white);
}

#[test]
fn gpu_cpu_pixel_parity_gate() {
    let Some(mut gpu) = skip_if_no_gpu() else { return };
    let doc = scene_doc();
    let o = opts(320, 180);
    let mut assets = mo2d::assets::AssetStore::new();
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND / 2).unwrap();
    let cpu = render_with_assets(&lines, 320.0, 180.0, &o, &mut assets);
    let gp = gpu.render_frame(&lines, 320.0, 180.0, &o).expect("GPU frame encodes");
    let report = parity_report(&cpu, &gp, 4, 40);
    let rate = report.rate_within(40);
    eprintln!(
        "B1 GPU parity: {}/{} within tol 40 (rate {:.4}); p50 {} / p90 {} / p99 {} / worst {}",
        report.within_tol,
        report.checked,
        rate,
        report.percentile(50),
        report.percentile(90),
        report.percentile(99),
        report.worst
    );
    // THE PARITY GATE — active: the GPU must pixel-match the CPU oracle on
    // ≥95% of sampled pixels (interiors are exact: p50/p90 = 0 expected; the
    // residual tail is stroke-joint and rasterization-edge divergence).
    assert!(
        rate >= 0.95,
        "parity gate: pass rate must hold at ≥ 0.95 (got {:.4}, worst {})",
        rate,
        report.worst
    );
}

#[test]
fn gpu_100k_nodes_at_60fps() {
    let Some(mut gpu) = skip_if_no_gpu() else { return };
    // 100k-node stroke (the roadmap's 100k-node class): ONE instanced draw.
    let mut node_json = Vec::with_capacity(100_000);
    for i in 0..100_000 {
        let x = (i as f64) * 0.006;
        let y = ((i as f64) * 0.013).sin() * 80.0 + 90.0;
        node_json.push(format!("{{\"p\":[{:.4},{:.4},0],\"w\":3}}", x, y));
    }
    let stream = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"doc.setRoot","width":640,"height":360}},
          {{"op":"line.create","id":"stress","width":3,"nodes":[{}]}}
        ]}}"#,
        node_json.join(",")
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let lines = eval::evaluate(&doc, 0).unwrap();

    let mut samples = Vec::new();
    for _ in 0..9 {
        let t = Instant::now();
        let img = gpu.render_frame(&lines, 640.0, 360.0, &opts(640, 360)).expect("GPU frame");
        std::hint::black_box(&img);
        samples.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    eprintln!("B1 GPU 100k-node frame: median {:.2} ms (60 fps budget 16 ms)", median);
    // Debug Rust is several× slower; the roadmap metric is measured on the
    // optimized build (release) — assert strictly there, relaxed in debug.
    if cfg!(debug_assertions) {
        assert!(
            median < 48.0,
            "100k-node GPU frame (debug) must stay under 3× budget, got {:.2} ms",
            median
        );
    } else {
        assert!(
            median < 16.0,
            "100k-node GPU frame must fit the 16 ms (60 fps) budget, got {:.2} ms",
            median
        );
    }
}
