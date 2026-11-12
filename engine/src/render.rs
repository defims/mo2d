//! CPU rasterizer — the deterministic oracle (architecture.md §3: CPU oracle
//! is the source of truth; GPU comes later as an accelerator).
//!
//! The renderer **is** 3D from day one (mvp.md D6): perspective divide,
//! z-buffer, backface culling, flat unlit colors; 2D content is the same
//! pipeline with z ≡ 0 and an orthographic camera.
//!
//! Determinism: f64 arithmetic, fixed subdivision counts, no shortcuts that
//! vary by input magnitude. Same doc + same tick + same camera → same bytes.

use crate::eval::EvalLine;
use crate::math::{self, Mat4, Vec3};
use crate::model::{SegType, TICKS_PER_SECOND};

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Camera {
    /// 2D: world rect [0..root.width]×[0..root.height] fills the canvas.
    /// Depth range ±ORTHO_DEPTH covers z ≠ 0 content in 2D scenes.
    Ortho2d,
    /// 3D perspective: eye/target/up in world space, vertical FOV in degrees.
    Perspective {
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y_deg: f64,
        near: f64,
        far: f64,
    },
}

pub const ORTHO_DEPTH: f64 = 2000.0;

impl Default for Camera {
    fn default() -> Self {
        Camera::Ortho2d
    }
}

impl Camera {
    fn matrices(&self, world_w: f64, world_h: f64, aspect: f64) -> (Mat4, Mat4) {
        match self {
            Camera::Ortho2d => {
                let v = Mat4::identity();
                let p = math::ortho(0.0, world_w, 0.0, world_h, -ORTHO_DEPTH, ORTHO_DEPTH);
                (v, p)
            }
            Camera::Perspective { eye, target, up, fov_y_deg, near, far } => {
                let v = math::look_at(*eye, *target, *up);
                let p = math::perspective(*fov_y_deg, aspect, *near, *far);
                (v, p)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Framebuffer
// ---------------------------------------------------------------------------

pub struct Image {
    pub w: u32,
    pub h: u32,
    /// RGBA8, row-major, top-down.
    pub px: Vec<u8>,
}

impl Image {
    pub fn new(w: u32, h: u32) -> Image {
        Image { w, h, px: vec![255u8; (w * h * 4) as usize] } // white background
    }
    fn blend(&mut self, x: u32, y: u32, r: f64, g: f64, b: f64, a: f64) {
        if a <= 0.0 {
            return;
        }
        let idx = ((y * self.w + x) * 4) as usize;
        let sa = a.clamp(0.0, 1.0);
        let da = self.px[idx + 3] as f64 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            return;
        }
        for (c, ch) in [(r, 0usize), (g, 1), (b, 2)] {
            let dst = self.px[idx + ch] as f64;
            let src = (c.clamp(0.0, 255.0)) * sa + dst * da * (1.0 - sa);
            self.px[idx + ch] = (src / out_a).round().clamp(0.0, 255.0) as u8;
        }
        self.px[idx + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }
}

// ---------------------------------------------------------------------------
// Render options
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RenderOpts {
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    /// Supersampling factor (1 = aliased oracle; deterministic either way).
    pub samples: u32,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts { width: 640, height: 360, camera: Camera::Ortho2d, samples: 1 }
    }
}

/// Render one evaluated frame.
pub fn render(lines: &[EvalLine], world_w: f64, world_h: f64, opts: &RenderOpts) -> Image {
    let img = Image::new(opts.width, opts.height);
    let zbuf = vec![f32::INFINITY; (opts.width * opts.height) as usize];
    let aspect = opts.width as f64 / opts.height as f64;
    let (view, proj) = opts.camera.matrices(world_w, world_h, aspect);
    let vp = proj.mul(&view); // clip = P·V·p (projection composes ONTO the view)

    let mut ctx = Ctx { img, zbuf, vp, w: opts.width, h: opts.height };

    for line in lines {
        if line.tags.iter().any(|t| t == "track") {
            continue; // track lines are data, not drawable geometry
        }
        if line.nodes.len() < 2 {
            continue;
        }
        // Flatten (curve segments → fixed-step polylines).
        let mut flat = flatten(line);
        // Closed contours from flatten() end with a duplicate of the first
        // point — drop it so fill/stroke see a clean loop.
        if line.closed && flat.len() > 1 && (flat[0].p.sub(flat[flat.len() - 1].p).len()) < 1e-9 {
            flat.pop();
        }

        // Fill first, stroke on top (closed line = fill domain + outline).
        if line.closed && line.fill_on {
            let tris = triangulate(&flat, line.closed);
            let c = line.fill_color;
            for tri in tris {
                ctx.draw_tri(&tri, c, true);
            }
        }
        if line.stroke_on {
            let strip = width_strip(&ctx, &flat, line.closed);
            let mut i = 0;
            while i + 3 < strip.len() {
                let quad = [strip[i].clone(), strip[i + 1].clone(), strip[i + 2].clone(), strip[i + 3].clone()];
                // Two triangles per quad step; colors ride on strip vertices.
                ctx.draw_tri_raw(&[quad[0].clone(), quad[1].clone(), quad[2].clone()], false);
                ctx.draw_tri_raw(&[quad[1].clone(), quad[3].clone(), quad[2].clone()], false);
                i += 2;
            }
        }
    }
    ctx.img
}

// ---------------------------------------------------------------------------
// Geometry: flatten + strip + triangulate (all in world space)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FlatPt {
    p: Vec3,
    width: f64,
    color: [u8; 4],
}

/// Flatten curve segments with fixed 16-step subdivision (deterministic).
fn flatten(line: &EvalLine) -> Vec<FlatPt> {
    let n = line.nodes.len();
    let seg_count = if line.closed { n } else { n - 1 };
    let mut out: Vec<FlatPt> = Vec::with_capacity(seg_count * 17);
    if seg_count == 0 {
        return out;
    }
    for k in 0..seg_count {
        let a = &line.nodes[k];
        let b = &line.nodes[(k + 1) % n];
        let sa = line.seg(k);
        if !sa.visible {
            // Invisible segment: skip geometry but keep endpoints chain intact
            // by not emitting its interior (strokes will connect across visually).
            continue;
        }
        let curve = sa.seg_type == SegType::Curve && (a.handle_out.is_some() || b.handle_in.is_some());
        if k == 0 {
            out.push(FlatPt { p: a.p, width: a.width, color: a.color });
        }
        if curve {
            let p0 = a.p;
            let p1 = a.p.add(a.handle_out.unwrap_or(Vec3::ZERO));
            let p3 = b.p;
            let p2 = b.p.add(b.handle_in.unwrap_or(Vec3::ZERO));
            const STEPS: usize = 16;
            for s in 1..=STEPS {
                let t = s as f64 / STEPS as f64;
                let m = 1.0 - t;
                let pos = p0
                    .scale(m * m * m)
                    .add(p1.scale(3.0 * m * m * t))
                    .add(p2.scale(3.0 * m * t * t))
                    .add(p3.scale(t * t * t));
                out.push(FlatPt {
                    p: pos,
                    width: a.width + (b.width - a.width) * t,
                    color: lerp_color(a.color, b.color, t),
                });
            }
        } else {
            out.push(FlatPt { p: b.p, width: b.width, color: b.color });
        }
    }
    out
}

fn lerp_color(a: [u8; 4], b: [u8; 4], t: f64) -> [u8; 4] {
    let mut c = [0u8; 4];
    for i in 0..4 {
        c[i] = (a[i] as f64 + (b[i] as f64 - a[i] as f64) * t).round().clamp(0.0, 255.0) as u8;
    }
    c
}

/// Triangulate a flattened contour (ear clipping). Simple polygons only —
/// the nonzero fill rule for self-intersecting contours is a later-phase item.
/// 3-node closed lines are the triangle-face special case (D5) and may have
/// arbitrary orientation (true 3D triangle).
fn triangulate(pts: &[FlatPt], _closed: bool) -> Vec<[Vec3; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[pts[0].p, pts[1].p, pts[2].p]];
    }
    // Ear clip on the xy projection, then emit world-space triangles.
    let mut idx: Vec<usize> = (0..n).collect();
    let area2 = signed_area_xy(pts);
    // Work with consistent CCW (positive area, y-up world).
    if area2 < 0.0 {
        idx.reverse();
    }
    let mut tris = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < 10_000 {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let ia = idx[(i + m - 1) % m];
            let ib = idx[i];
            let ic = idx[(i + 1) % m];
            let (a, b, c) = (pts[ia].p, pts[ib].p, pts[ic].p);
            if cross_xy(a, b, c) <= 1e-12 {
                continue; // reflex or degenerate ear tip
            }
            let mut contains = false;
            for &j in &idx {
                if j == ia || j == ib || j == ic {
                    continue;
                }
                if point_in_tri_xy(pts[j].p, a, b, c) {
                    contains = true;
                    break;
                }
            }
            if contains {
                continue;
            }
            tris.push([a, b, c]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break; // degenerate polygon; emit what we have as a fan
        }
    }
    if idx.len() == 3 {
        tris.push([pts[idx[0]].p, pts[idx[1]].p, pts[idx[2]].p]);
    }
    tris
}

fn signed_area_xy(pts: &[FlatPt]) -> f64 {
    let n = pts.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = pts[i].p;
        let b = pts[(i + 1) % n].p;
        s += a.x * b.y - b.x * a.y;
    }
    s / 2.0
}

fn cross_xy(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_in_tri_xy(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    let d1 = cross_xy(a, b, p);
    let d2 = cross_xy(b, c, p);
    let d3 = cross_xy(c, a, p);
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

// ---------------------------------------------------------------------------
// Raster
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ScreenVtx {
    x: f64,
    y: f64, // pixels, y down
    z: f32, // NDC depth [0,1]
    iw: f64, // 1/w for perspective-correct attributes
    c: [u8; 4],
}

struct Ctx {
    img: Image,
    zbuf: Vec<f32>,
    vp: Mat4,
    w: u32,
    h: u32,
}

impl Ctx {
    fn project(&self, p: Vec3, c: [u8; 4]) -> Option<ScreenVtx> {
        let (clip, w) = self.vp.project(p);
        if w <= 1e-9 {
            return None; // behind the camera / degenerate
        }
        let ndc_x = clip.x / w;
        let ndc_y = clip.y / w;
        let ndc_z = (clip.z / w) as f32;
        if !(-1.0..=1.0).contains(&ndc_x) || !(-1.0..=1.0).contains(&ndc_y) {
            // MVP scissor: no guard-band clipping; triangles touching the canvas
            // edge must be built fully on-screen (demo streams comply). Points
            // outside are marked off-canvas and their triangles are dropped.
            return None;
        }
        Some(ScreenVtx {
            x: (ndc_x + 1.0) * 0.5 * self.w as f64,
            y: (1.0 - ndc_y) * 0.5 * self.h as f64,
            z: ndc_z,
            iw: 1.0 / w,
            c,
        })
    }

    /// Draw a world-space triangle with a flat color.
    fn draw_tri(&mut self, tri: &[Vec3; 3], color: [u8; 4], cull_back: bool) {
        let verts = [
            self.project(tri[0], color),
            self.project(tri[1], color),
            self.project(tri[2], color),
        ];
        if verts.iter().any(|v| v.is_none()) {
            return;
        }
        let v = [verts[0].clone().unwrap(), verts[1].clone().unwrap(), verts[2].clone().unwrap()];
        self.raster(&v, cull_back);
    }

    /// Draw a screen-space triangle (stroke strips) with per-vertex colors.
    fn draw_tri_raw(&mut self, v: &[ScreenVtx; 3], cull_back: bool) {
        self.raster(v, cull_back);
    }

    fn raster(&mut self, v: &[ScreenVtx; 3], cull_back: bool) {
        // Signed area in screen space (y-down pixels, NDC y was flipped).
        // Content whose world winding faces the camera (right-hand normal
        // toward the eye) lands NEGATIVE here — that is the front face.
        let area2 = (v[1].x - v[0].x) * (v[2].y - v[0].y) - (v[1].y - v[0].y) * (v[2].x - v[0].x);
        if area2.abs() < 1e-12 {
            return;
        }
        if area2 > 0.0 && cull_back {
            return; // backface culled (D6)
        }
        self.raster_pos(v);
    }

    fn raster_pos(&mut self, v: &[ScreenVtx; 3]) {
        let mut area2 = (v[1].x - v[0].x) * (v[2].y - v[0].y) - (v[1].y - v[0].y) * (v[2].x - v[0].x);
        // Normalize orientation so the fill rule below is sign-stable.
        let flipped;
        let v: &[ScreenVtx; 3] = if area2 < 0.0 {
            flipped = [v[2].clone(), v[1].clone(), v[0].clone()];
            area2 = -area2;
            &flipped
        } else {
            v
        };
        if area2 <= 1e-12 {
            return;
        }
        // Top-left fill rule (no cracks, no double-draw on shared edges).
        let top_left = |a: &ScreenVtx, b: &ScreenVtx| -> bool {
            (a.y == b.y && b.x > a.x) || (a.x == b.x && b.y < a.y)
        };
        let min_x = v.iter().map(|p| p.x).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
        let max_x = v.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max).ceil().min(self.w as f64 - 1.0) as i64;
        let min_y = v.iter().map(|p| p.y).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
        let max_y = v.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max).ceil().min(self.h as f64 - 1.0) as i64;

        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let p = Vec3::new(px as f64 + 0.5, py as f64 + 0.5, 0.0);
                let e0 = (v[1].x - v[0].x) * (p.y - v[0].y) - (v[1].y - v[0].y) * (p.x - v[0].x);
                let e1 = (v[2].x - v[1].x) * (p.y - v[1].y) - (v[2].y - v[1].y) * (p.x - v[1].x);
                let e2 = (v[0].x - v[2].x) * (p.y - v[2].y) - (v[0].y - v[2].y) * (p.x - v[2].x);
                let inside = |e: f64, on: bool| e > 0.0 || (e == 0.0 && on);
                if !(inside(e0, top_left(&v[0], &v[1])) && inside(e1, top_left(&v[1], &v[2])) && inside(e2, top_left(&v[2], &v[0]))) {
                    continue;
                }
                let w0 = e1 / area2;
                let w1 = e2 / area2;
                let w2 = e0 / area2;
                // Depth: NDC z is screen-affine (that's why z-buffering works).
                let z = (w0 * v[0].z as f64 + w1 * v[1].z as f64 + w2 * v[2].z as f64) as f32;
                if z < 0.0 || z > 1.0 {
                    continue;
                }
                let idx = (py as u32 * self.w + px as u32) as usize;
                // `<=` so coplanar 2D content (all z equal) follows paint order.
                if z > self.zbuf[idx] {
                    continue;
                }
                // Perspective-correct attributes.
                let iw = w0 * v[0].iw + w1 * v[1].iw + w2 * v[2].iw;
                if iw <= 0.0 {
                    continue;
                }
                let mut col = [0.0f64; 4];
                for (vi, sv) in v.iter().enumerate() {
                    let wgt = [w0, w1, w2][vi];
                    for ch in 0..4 {
                        col[ch] += wgt * sv.iw * sv.c[ch] as f64;
                    }
                }
                let (r, g, b, a) = (col[0] / iw, col[1] / iw, col[2] / iw, col[3] / iw);
                self.img.blend(px as u32, py as u32, r, g, b, a);
                self.zbuf[idx] = z;
            }
        }
    }
}

/// Build a variable-width ribbon in screen space, as left/right vertex pairs
/// (l0, r0, l1, r1, …). Joints use averaged normals with miter compensation
/// (clamped to 4× half-width) so the ribbon is continuous without notches.
/// Widths are world units converted to pixels via the projection scale at
/// each point (exact for ortho2d; per-point depth for perspective).
fn width_strip(ctx: &Ctx, flat: &[FlatPt], closed: bool) -> Vec<ScreenVtx> {
    let n = flat.len();
    if n < 2 {
        return vec![];
    }

    // Screen positions + half-widths in px.
    let mut scr: Vec<(f64, f64, f64, Option<ScreenVtx>)> = Vec::with_capacity(n);
    for pt in flat {
        if let Some(base) = project_keep(ctx, pt.p, pt.color) {
            let scale = px_per_world(ctx, pt.p);
            scr.push((base.x, base.y, scale * pt.width * 0.5, Some(base)));
        } else {
            scr.push((0.0, 0.0, 0.0, None));
        }
    }

    // Per-segment normals in screen space.
    let seg_count = if closed { n } else { n - 1 };
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

    let mut out: Vec<ScreenVtx> = Vec::with_capacity(n * 2 + 2);
    for i in 0..n {
        let Some(base) = scr[i].3.clone() else { continue };
        if !closed && i == 0 {
            let (nx, ny) = seg_norm[0];
            push_pair(&mut out, &base, scr[i].2, nx, ny);
            continue;
        }
        if !closed && i == n - 1 {
            let (nx, ny) = seg_norm[seg_count - 1];
            push_pair(&mut out, &base, scr[i].2, nx, ny);
            continue;
        }
        // Interior joint: average of adjacent segment normals, miter-scaled.
        let prev = if i == 0 { seg_count - 1 } else { i - 1 };
        let next = if closed { i % seg_count } else { i.min(seg_count - 1) };
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
        // Miter scale = 1 / cos(half-angle) via dot with a segment normal.
        let cos_half = (nx * pa.0 + ny * pa.1).abs().max(1e-6);
        let scale = (1.0 / cos_half).min(4.0);
        push_pair(&mut out, &base, scr[i].2 * scale, nx, ny);
    }
    if closed && out.len() >= 2 {
        // Wrap: repeat the first pair so the closing quad is generated.
        let first2 = [out[0].clone(), out[1].clone()];
        out.push(first2[0].clone());
        out.push(first2[1].clone());
    }
    out
}

fn push_pair(out: &mut Vec<ScreenVtx>, base: &ScreenVtx, half: f64, nx: f64, ny: f64) {
    out.push(ScreenVtx { x: base.x + nx * half, y: base.y + ny * half, ..base.clone() });
    out.push(ScreenVtx { x: base.x - nx * half, y: base.y - ny * half, ..base.clone() });
}

/// Project without the on-canvas guard (strip building needs off-canvas verts;
/// raster clips by bounding box anyway).
fn project_keep(ctx: &Ctx, p: Vec3, c: [u8; 4]) -> Option<ScreenVtx> {
    let (clip, w) = ctx.vp.project(p);
    if w <= 1e-9 {
        return None;
    }
    Some(ScreenVtx {
        x: (clip.x / w + 1.0) * 0.5 * ctx.w as f64,
        y: (1.0 - clip.y / w) * 0.5 * ctx.h as f64,
        z: (clip.z / w) as f32,
        iw: 1.0 / w,
        c,
    })
}

/// World→pixel scale at a point (for stroke width conversion).
fn px_per_world(ctx: &Ctx, p: Vec3) -> f64 {
    let (c, w) = ctx.vp.project(p);
    if w <= 1e-9 {
        return 1.0;
    }
    let (c2, w2) = ctx.vp.project(Vec3::new(p.x + 1.0, p.y, p.z));
    if w2 <= 1e-9 {
        return 1.0;
    }
    let x1 = (c.x / w + 1.0) * 0.5 * ctx.w as f64;
    let x2 = (c2.x / w2 + 1.0) * 0.5 * ctx.w as f64;
    (x2 - x1).abs().max(1e-9)
}

/// Convenience: render at root size with the default camera.
pub fn render_frame(lines: &[EvalLine], world_w: f64, world_h: f64, camera: &Camera) -> Image {
    render(
        lines,
        world_w,
        world_h,
        &RenderOpts { width: world_w as u32, height: world_h as u32, camera: camera.clone(), samples: 1 },
    )
}

/// Ticks per second constant re-export for consumers.
pub const TPS: i64 = TICKS_PER_SECOND;
