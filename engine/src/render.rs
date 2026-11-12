//! CPU rasterizer — the deterministic oracle (architecture.md §3: CPU oracle
//! is the source of truth; GPU comes later as an accelerator).
//!
//! The renderer **is** 3D from day one (mvp.md D6): perspective divide,
//! z-buffer, backface culling, flat unlit colors; 2D content is the same
//! pipeline with z ≡ 0 and an orthographic camera.
//!
//! Determinism: f64 arithmetic, fixed subdivision counts, no shortcuts that
//! vary by input magnitude. Same doc + same tick + same camera → same bytes.
//!
//! B1 damage rendering: `render_damaged` recomposites only the tiles touched
//! by changed lines (tile grid 64×32 + per-primitive screen bounding boxes).
//! The damaged path drives the *same* primitive list and the *same* rasterizer
//! as the full path — a dirty tile is cleared to the background and every
//! primitive intersecting it is redrawn in document order with a fresh
//! z-buffer — so damaged output is byte-identical to a full re-render by
//! construction (and verified by the B1 parity gate).

use crate::assets::AssetStore;
use crate::eval::EvalLine;
use crate::math::{self, Mat4, Vec3};
use crate::model::{SegType, TICKS_PER_SECOND};
use std::sync::Arc;

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
    pub fn matrices(&self, world_w: f64, world_h: f64, aspect: f64) -> (Mat4, Mat4) {
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

#[derive(Clone)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    /// RGBA8, row-major, top-down.
    pub px: Vec<u8>,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image").field("w", &self.w).field("h", &self.h).field("px_len", &self.px.len()).finish()
    }
}

/// RGBA8 background the framebuffer starts from (white, opaque).
pub const BACKGROUND: [u8; 4] = [255, 255, 255, 255];

impl Image {
    pub fn new(w: u32, h: u32) -> Image {
        let mut px = vec![0u8; (w * h * 4) as usize];
        for cell in px.chunks_exact_mut(4) {
            cell.copy_from_slice(&BACKGROUND);
        }
        Image { w, h, px }
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

/// Render one evaluated frame. Textured lines (B5) render as their flat fill
/// color here — use [`render_with_assets`] to supply the texture store.
pub fn render(lines: &[EvalLine], world_w: f64, world_h: f64, opts: &RenderOpts) -> Image {
    let mut empty = AssetStore::new();
    render_with_assets(lines, world_w, world_h, opts, &mut empty)
}

/// Render one evaluated frame with texture support (B5): lines carrying a
/// `texture` reference sample it through `assets` (decode-on-demand,
/// document-root-relative paths). Missing textures fall back to the flat
/// fill color (deterministic; the store records the error for the host).
pub fn render_with_assets(
    lines: &[EvalLine],
    world_w: f64,
    world_h: f64,
    opts: &RenderOpts,
    assets: &mut AssetStore,
) -> Image {
    let mut img = Image::new(opts.width, opts.height);
    let mut zbuf = vec![f32::INFINITY; (opts.width * opts.height) as usize];
    let aspect = opts.width as f64 / opts.height as f64;
    let (view, proj) = opts.camera.matrices(world_w, world_h, aspect);
    let vp = proj.mul(&view); // clip = P·V·p (projection composes ONTO the view)

    let mut ctx = Ctx { img: &mut img, zbuf: &mut zbuf, w: opts.width, h: opts.height, scissor: None };
    for prim in build_prims(lines, &vp, opts.width, opts.height, assets) {
        ctx.raster_prim(&prim);
    }
    img
}

// ---------------------------------------------------------------------------
// B1 damage rendering: tile grid + dirty rectangles
// ---------------------------------------------------------------------------

/// Damage tile size (roadmap B1: tile 64×32 + dirty rectangles).
pub const TILE_W: u32 = 64;
pub const TILE_H: u32 = 32;

/// Miter compensation clamp used by `width_strip` (bbox padding must match it).
const MITER_CLAMP: f64 = 4.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DamageStats {
    /// Tiles recomposited this frame.
    pub dirty_tiles: u32,
    /// Tiles on the canvas.
    pub total_tiles: u32,
    /// Evaluated lines whose content changed (or appeared/vanished).
    pub changed_lines: usize,
}

/// Recomposite `next` over `prev` touching only dirty tiles.
///
/// Dirty tiles = screen bounding boxes of evaluated lines that changed
/// (position/width/color/attrs), appeared, or vanished — each box padded by
/// the stroke's maximum miter-extended half width. Every dirty tile is reset
/// to the background and redrawn from the **full** primitive list of `next`
/// (scissored), so occlusion and alpha behave exactly as in a full render.
pub fn render_damaged(
    prev: &Image,
    prev_lines: &[EvalLine],
    next: &[EvalLine],
    world_w: f64,
    world_h: f64,
    opts: &RenderOpts,
    assets: &mut AssetStore,
) -> (Image, DamageStats) {
    let mut img = prev.clone();
    let stats = DamageStats { dirty_tiles: 0, total_tiles: tiles_x(opts.width) * tiles_y(opts.height), changed_lines: 0 };
    let aspect = opts.width as f64 / opts.height as f64;
    let (view, proj) = opts.camera.matrices(world_w, world_h, aspect);
    let vp = proj.mul(&view);

    // --- diff prev → next: which lines changed / appeared / vanished? ---
    let mut prev_at: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (i, l) in prev_lines.iter().enumerate() {
        prev_at.entry(l.id.as_str()).or_insert(i);
    }
    let mut next_at: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (i, l) in next.iter().enumerate() {
        next_at.entry(l.id.as_str()).or_insert(i);
    }

    let mut dirty: Option<(f64, f64, f64, f64)> = None;
    let mut changed_lines = 0usize;
    let mark = |b: Option<(f64, f64, f64, f64)>, acc: &mut Option<(f64, f64, f64, f64)>| {
        if let Some(b) = b {
            match acc {
                None => *acc = Some(b),
                Some(a) => {
                    a.0 = a.0.min(b.0);
                    a.1 = a.1.min(b.1);
                    a.2 = a.2.max(b.2);
                    a.3 = a.3.max(b.3);
                }
            }
        }
    };
    for nl in next.iter() {
        let unchanged = matches!(prev_at.get(nl.id.as_str()), Some(&pi) if prev_lines[pi] == *nl);
        if !unchanged {
            changed_lines += 1;
            mark(line_screen_bbox(nl, &vp, opts.width, opts.height), &mut dirty);
        }
    }
    for pl in prev_lines {
        if !next_at.contains_key(pl.id.as_str()) {
            changed_lines += 1;
            mark(line_screen_bbox(pl, &vp, opts.width, opts.height), &mut dirty);
        }
    }

    let tiles_x_n = tiles_x(opts.width);
    let tiles_y_n = tiles_y(opts.height);
    // Collect dirty tiles row-major (deterministic), clamped to the canvas.
    let mut tiles: Vec<(u32, u32)> = Vec::new();
    if let Some((x0, y0, x1, y1)) = dirty {
        let tx0 = ((x0.max(0.0) as u32) / TILE_W).min(tiles_x_n.saturating_sub(1));
        let tx1 = (((x1.max(0.0) as u32).min(opts.width.saturating_sub(1))) / TILE_W).min(tiles_x_n.saturating_sub(1));
        let ty0 = ((y0.max(0.0) as u32) / TILE_H).min(tiles_y_n.saturating_sub(1));
        let ty1 = (((y1.max(0.0) as u32).min(opts.height.saturating_sub(1))) / TILE_H).min(tiles_y_n.saturating_sub(1));
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                tiles.push((tx, ty));
            }
        }
    }

    if tiles.is_empty() {
        return (img, DamageStats { dirty_tiles: 0, total_tiles: stats.total_tiles, changed_lines });
    }

    let prims = build_prims(next, &vp, opts.width, opts.height, assets);
    // Screen bbox per primitive (floor/ceil exactly as the rasterizer bounds).
    let prim_bb: Vec<Option<(i64, i64, i64, i64)>> = prims
        .iter()
        .map(|p| {
            let min_x = p.v.iter().map(|v| v.x).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
            let max_x = p.v.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max).ceil().min(opts.width as f64 - 1.0) as i64;
            let min_y = p.v.iter().map(|v| v.y).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
            let max_y = p.v.iter().map(|v| v.y).fold(f64::NEG_INFINITY, f64::max).ceil().min(opts.height as f64 - 1.0) as i64;
            if min_x > max_x || min_y > max_y {
                None
            } else {
                Some((min_x, min_y, max_x, max_y))
            }
        })
        .collect();

    let mut zbuf = vec![f32::INFINITY; (opts.width * opts.height) as usize];
    for &(tx, ty) in &tiles {
        let x0 = tx * TILE_W;
        let y0 = ty * TILE_H;
        let x1 = ((x0 + TILE_W) as f64).min(opts.width as f64) as u32;
        let y1 = ((y0 + TILE_H) as f64).min(opts.height as f64) as u32;
        // Reset the tile: background pixels + fresh depth.
        for py in y0..y1 {
            for px in x0..x1 {
                let idx = ((py * opts.width + px) * 4) as usize;
                img.px[idx..idx + 4].copy_from_slice(&BACKGROUND);
                zbuf[(py * opts.width + px) as usize] = f32::INFINITY;
            }
        }
        let scissor = (x0 as i64, y0 as i64, (x1 as i64) - 1, (y1 as i64) - 1);
        let mut ctx = Ctx { img: &mut img, zbuf: &mut zbuf, w: opts.width, h: opts.height, scissor: Some(scissor) };
        for (prim, bb) in prims.iter().zip(prim_bb.iter()) {
            let Some((px0, py0, px1, py1)) = bb else { continue };
            if *px0 > scissor.2 || *px1 < scissor.0 || *py0 > scissor.3 || *py1 < scissor.1 {
                continue; // primitive cannot touch this tile
            }
            ctx.raster_prim(prim);
        }
    }

    (
        img,
        DamageStats { dirty_tiles: tiles.len() as u32, total_tiles: stats.total_tiles, changed_lines },
    )
}

fn tiles_x(w: u32) -> u32 {
    (w + TILE_W - 1) / TILE_W
}
fn tiles_y(h: u32) -> u32 {
    (h + TILE_H - 1) / TILE_H
}

// ---------------------------------------------------------------------------
// Geometry: flatten + strip + triangulate (all in world space)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FlatPt {
    p: Vec3,
    width: f64,
    color: [u8; 4],
    /// Per-point fill color (B3 gradients): line default or gradient-sampled.
    fill: [u8; 4],
    /// Texture anchor (B5): None for plain lines.
    uv: Option<(f64, f64)>,
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
            out.push(FlatPt { p: a.p, width: a.width, color: a.color, fill: a.fill, uv: a.uv });
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
                let uv = match (a.uv, b.uv) {
                    (Some((u0, v0)), Some((u1, v1))) => Some((u0 + (u1 - u0) * t, v0 + (v1 - v0) * t)),
                    _ => None,
                };
                out.push(FlatPt {
                    p: pos,
                    width: a.width + (b.width - a.width) * t,
                    color: lerp_color(a.color, b.color, t),
                    fill: lerp_color(a.fill, b.fill, t),
                    uv,
                });
            }
        } else {
            out.push(FlatPt { p: b.p, width: b.width, color: b.color, fill: b.fill, uv: b.uv });
        }
    }
    out
}

fn lerp_color(a: [u8; 4], b: [u8; 4], t: f64) -> [u8; 4] {
    crate::eval::lerp_color(a, b, t)
}

/// Triangulate a flattened contour (ear clipping). Simple polygons only —
/// the nonzero fill rule for self-intersecting contours is a later-phase item.
/// 3-node closed lines are the triangle-face special case (D5) and may have
/// arbitrary orientation (true 3D triangle). Emits the corner FlatPts so the
/// fill carries per-point colors (B3 gradients ride the existing
/// perspective-correct attribute interpolation).
fn triangulate(pts: &[FlatPt], _closed: bool) -> Vec<[FlatPt; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[pts[0].clone(), pts[1].clone(), pts[2].clone()]];
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
            tris.push([pts[ia].clone(), pts[ib].clone(), pts[ic].clone()]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break; // degenerate polygon; emit what we have as a fan
        }
    }
    if idx.len() == 3 {
        tris.push([pts[idx[0]].clone(), pts[idx[1]].clone(), pts[idx[2]].clone()]);
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
// Primitives: the shared draw list of full and damaged rendering
// ---------------------------------------------------------------------------

/// One drawable screen-space triangle. The primitive list is the *only* thing
/// both render paths consume — that is what makes damaged output byte-equal
/// to full output (B1 parity gate). `tex` carries the B5 texture for
/// textured fills (all three vertices carry UVs).
#[derive(Debug, Clone)]
struct Prim {
    v: [ScreenVtx; 3],
    cull: bool,
    tex: Option<Arc<Image>>,
}

fn project_vp(vp: &Mat4, w: u32, h: u32, p: Vec3, c: [u8; 4], uv: Option<(f64, f64)>) -> Option<ScreenVtx> {
    let (clip, wc) = vp.project(p);
    if wc <= 1e-9 {
        return None; // behind the camera / degenerate
    }
    let ndc_x = clip.x / wc;
    let ndc_y = clip.y / wc;
    let ndc_z = (clip.z / wc) as f32;
    if !(-1.0..=1.0).contains(&ndc_x) || !(-1.0..=1.0).contains(&ndc_y) {
        // MVP scissor: no guard-band clipping; triangles touching the canvas
        // edge must be built fully on-screen (demo streams comply). Points
        // outside are marked off-canvas and their triangles are dropped.
        return None;
    }
    Some(ScreenVtx {
        x: (ndc_x + 1.0) * 0.5 * w as f64,
        y: (1.0 - ndc_y) * 0.5 * h as f64,
        z: ndc_z,
        iw: 1.0 / wc,
        c,
        uv,
    })
}

/// Project without the on-canvas guard (strip building needs off-canvas verts;
/// raster clips by bounding box anyway).
fn project_keep_vp(vp: &Mat4, w: u32, h: u32, p: Vec3, c: [u8; 4]) -> Option<ScreenVtx> {
    let (clip, wc) = vp.project(p);
    if wc <= 1e-9 {
        return None;
    }
    Some(ScreenVtx {
        x: (clip.x / wc + 1.0) * 0.5 * w as f64,
        y: (1.0 - clip.y / wc) * 0.5 * h as f64,
        z: (clip.z / wc) as f32,
        iw: 1.0 / wc,
        c,
        uv: None,
    })
}

/// World→pixel scale at a point (for stroke width conversion).
fn px_per_world_vp(vp: &Mat4, w: u32, p: Vec3) -> f64 {
    let (c, wc) = vp.project(p);
    if wc <= 1e-9 {
        return 1.0;
    }
    let (c2, w2) = vp.project(Vec3::new(p.x + 1.0, p.y, p.z));
    if w2 <= 1e-9 {
        return 1.0;
    }
    let x1 = (c.x / wc + 1.0) * 0.5 * w as f64;
    let x2 = (c2.x / w2 + 1.0) * 0.5 * w as f64;
    (x2 - x1).abs().max(1e-9)
}

/// The frame's drawable primitives, in document order (fill before stroke per
/// line). Full render and damage render both consume exactly this list.
fn build_prims(lines: &[EvalLine], vp: &Mat4, w: u32, h: u32, assets: &mut AssetStore) -> Vec<Prim> {
    let mut prims = Vec::new();
    for line in lines {
        if line.tags.iter().any(|t| t == "track") {
            continue; // track lines are data, not drawable geometry
        }
        if line.visible != true {
            continue; // hidden (e.g. via a group/layer relation)
        }
        if line.nodes.len() < 2 {
            continue;
        }
        // B5: resolve the texture once per line (decode-on-demand; missing
        // textures fall back to flat fill — assets records the error).
        let tex: Option<Arc<Image>> = line.texture.as_ref().and_then(|t| assets.get_ref(t));
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
            for tri in tris {
                let verts = [
                    project_vp(vp, w, h, tri[0].p, tri[0].fill, tri[0].uv),
                    project_vp(vp, w, h, tri[1].p, tri[1].fill, tri[1].uv),
                    project_vp(vp, w, h, tri[2].p, tri[2].fill, tri[2].uv),
                ];
                if verts.iter().any(|v| v.is_none()) {
                    continue;
                }
                prims.push(Prim {
                    v: [verts[0].clone().unwrap(), verts[1].clone().unwrap(), verts[2].clone().unwrap()],
                    cull: true,
                    tex: tex.clone(),
                });
            }
        }
        if line.stroke_on {
            let strip = width_strip(vp, w, h, &flat, line.closed);
            let mut i = 0;
            while i + 3 < strip.len() {
                let (a, b, c, d) = (strip[i].clone(), strip[i + 1].clone(), strip[i + 2].clone(), strip[i + 3].clone());
                prims.push(Prim { v: [a.clone(), b.clone(), c.clone()], cull: false, tex: None });
                prims.push(Prim { v: [b, d, c], cull: false, tex: None });
                i += 2;
            }
        }
    }
    prims
}

/// Screen-space bounding box of a drawable line (for damage tiles): flatten
/// bbox padded by the miter-clamped stroke half width. None = not drawable or
/// entirely off-canvas.
fn line_screen_bbox(line: &EvalLine, vp: &Mat4, w: u32, h: u32) -> Option<(f64, f64, f64, f64)> {
    if line.tags.iter().any(|t| t == "track") || line.visible != true || line.nodes.len() < 2 {
        return None;
    }
    let mut flat = flatten(line);
    if line.closed && flat.len() > 1 && (flat[0].p.sub(flat[flat.len() - 1].p).len()) < 1e-9 {
        flat.pop();
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut max_half = 0.0f64;
    let mut any = false;
    for pt in &flat {
        if let Some(s) = project_keep_vp(vp, w, h, pt.p, pt.color) {
            any = true;
            min_x = min_x.min(s.x);
            min_y = min_y.min(s.y);
            max_x = max_x.max(s.x);
            max_y = max_y.max(s.y);
            max_half = max_half.max(px_per_world_vp(vp, w, pt.p) * pt.width * 0.5);
        }
    }
    if !any {
        return None;
    }
    let pad = max_half * MITER_CLAMP + 2.0;
    Some((min_x - pad, min_y - pad, max_x + pad, max_y + pad))
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
    /// Texture anchor (B5); None on stroke strip vertices.
    uv: Option<(f64, f64)>,
}

/// Pixel scissor (inclusive min/max) — None for full-frame rendering.
type Scissor = (i64, i64, i64, i64);

struct Ctx<'a> {
    img: &'a mut Image,
    zbuf: &'a mut [f32],
    w: u32,
    h: u32,
    scissor: Option<Scissor>,
}

impl<'a> Ctx<'a> {
    /// Draw one primitive triangle.
    fn raster_prim(&mut self, prim: &Prim) {
        let v = &prim.v;
        // Signed area in screen space (y-down pixels, NDC y was flipped).
        // Content whose world winding faces the camera (right-hand normal
        // toward the eye) lands NEGATIVE here — that is the front face.
        let area2 = (v[1].x - v[0].x) * (v[2].y - v[0].y) - (v[1].y - v[0].y) * (v[2].x - v[0].x);
        if area2.abs() < 1e-12 {
            return;
        }
        if area2 > 0.0 && prim.cull {
            return; // backface culled (D6)
        }
        self.raster_pos(v, prim.tex.as_deref());
    }

    fn raster_pos(&mut self, v: &[ScreenVtx; 3], tex: Option<&Image>) {
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
        let mut min_x = v.iter().map(|p| p.x).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
        let mut max_x = v.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max).ceil().min(self.w as f64 - 1.0) as i64;
        let mut min_y = v.iter().map(|p| p.y).fold(f64::INFINITY, f64::min).floor().max(0.0) as i64;
        let mut max_y = v.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max).ceil().min(self.h as f64 - 1.0) as i64;
        if let Some(s) = self.scissor {
            min_x = min_x.max(s.0);
            min_y = min_y.max(s.1);
            max_x = max_x.min(s.2);
            max_y = max_y.min(s.3);
        }

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
                // B5: textured fills sample the texture perspective-correctly
                // and modulate the fill color (deterministic bilinear).
                let (mut r, mut g, mut b, mut a) = (r, g, b, a);
                if let Some(tex) = tex {
                    if v[0].uv.is_some() && v[1].uv.is_some() && v[2].uv.is_some() {
                        let mut u = 0.0f64;
                        let mut vv = 0.0f64;
                        for (sv, wgt) in v.iter().zip([w0, w1, w2]) {
                            let (su, svv) = sv.uv.unwrap_or((0.0, 0.0));
                            u += wgt * sv.iw * su;
                            vv += wgt * sv.iw * svv;
                        }
                        let (tr, tg, tb, ta) = sample_bilinear(tex, u / iw, vv / iw);
                        r *= tr / 255.0;
                        g *= tg / 255.0;
                        b *= tb / 255.0;
                        a *= ta / 255.0;
                    }
                }
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
fn width_strip(vp: &Mat4, w: u32, _h: u32, flat: &[FlatPt], closed: bool) -> Vec<ScreenVtx> {
    let n = flat.len();
    if n < 2 {
        return vec![];
    }

    // Screen positions + half-widths in px.
    let mut scr: Vec<(f64, f64, f64, Option<ScreenVtx>)> = Vec::with_capacity(n);
    for pt in flat {
        if let Some(base) = project_keep_vp(vp, w, _h, pt.p, pt.color) {
            let scale = px_per_world_vp(vp, w, pt.p);
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
        let scale = (1.0 / cos_half).min(MITER_CLAMP);
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

/// Deterministic bilinear texture sample: u,v clamped to [0,1], texel centers
/// spanned edge-to-edge (u=0 hits the first texel column, u=1 the last).
fn sample_bilinear(tex: &Image, u: f64, v: f64) -> (f64, f64, f64, f64) {
    let x = u.clamp(0.0, 1.0) * (tex.w as f64 - 1.0);
    let y = v.clamp(0.0, 1.0) * (tex.h as f64 - 1.0);
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let x0 = (x0 as u32).min(tex.w - 1);
    let y0 = (y0 as u32).min(tex.h - 1);
    let x1 = (x0 + 1).min(tex.w - 1);
    let y1 = (y0 + 1).min(tex.h - 1);
    let at = |xx: u32, yy: u32| -> [f64; 4] {
        let o = ((yy * tex.w + xx) * 4) as usize;
        [tex.px[o] as f64, tex.px[o + 1] as f64, tex.px[o + 2] as f64, tex.px[o + 3] as f64]
    };
    let c00 = at(x0, y0);
    let c10 = at(x1, y0);
    let c01 = at(x0, y1);
    let c11 = at(x1, y1);
    let mut out = [0.0f64; 4];
    for ch in 0..4 {
        let top = c00[ch] + (c10[ch] - c00[ch]) * fx;
        let bot = c01[ch] + (c11[ch] - c01[ch]) * fx;
        out[ch] = top + (bot - top) * fy;
    }
    (out[0], out[1], out[2], out[3])
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

/// Public geometry helpers for the GPU backend (B1): the GPU draws the CPU
/// oracle's own flattened segments and triangles, in the same paint order —
/// that is what makes the parity gate meaningful.
pub mod gpu_common {
    use super::{flatten as flatten_inner, project_keep_vp, px_per_world_vp, triangulate as triangulate_inner, EvalLine, FlatPt};
    use crate::math::{Mat4, Vec3};

    pub struct GpuPt {
        pub p: Vec3,
        pub width: f64,
        pub color: [u8; 4],
        pub fill: [u8; 4],
    }

    impl GpuPt {
        fn from(f: &FlatPt) -> GpuPt {
            GpuPt { p: f.p, width: f.width, color: f.color, fill: f.fill }
        }
    }

    /// The line's flattened polyline (identical to the CPU path, including
    /// curve subdivision and invisible-segment gaps).
    pub fn flatten(line: &EvalLine) -> Vec<GpuPt> {
        flatten_inner(line).iter().map(GpuPt::from).collect()
    }

    /// The line's fill triangles as (corner, fill color) triples — corner
    /// fill colors ride FlatPt so gradients carry into the GPU path.
    pub fn triangulate(pts: &[GpuPt]) -> Vec<[(Vec3, [u8; 4]); 3]> {
        let owned: Vec<FlatPt> = pts
            .iter()
            .map(|g| FlatPt { p: g.p, width: g.width, color: g.color, fill: g.fill, uv: None })
            .collect();
        triangulate_inner(&owned, true)
            .into_iter()
            .map(|tri| [(tri[0].p, tri[0].fill), (tri[1].p, tri[1].fill), (tri[2].p, tri[2].fill)])
            .collect()
    }

    /// Screen projection without the canvas guard — returns (x_px, y_px).
    /// Ribbon offsets live in screen px (width_strip parity for the GPU
    /// uber-shader).
    pub fn project_keep(vp: &Mat4, w: u32, p: Vec3) -> Option<(f64, f64)> {
        project_keep_vp(vp, w, 0, p, [0, 0, 0, 0]).map(|s| (s.x, s.y))
    }

    /// World→pixel scale at a point.
    pub fn px_per_world(vp: &Mat4, w: u32, p: Vec3) -> f64 {
        px_per_world_vp(vp, w, p)
    }
}
