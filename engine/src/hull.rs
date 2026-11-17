//! Visual-hull probe: one turnaround timeline → lifted solid → derived timelines.
//!
//! Probe demo for the "timeline generation" principle (docs/log.md 2026-11-17):
//! a turnaround (n closed silhouettes, each tagged with its pose angle about
//! the vertical lift axis) lifts into a solid — the visual hull, here carved
//! per scanline: at height y each frame contributes two half-planes
//! (screen-x span [a,b] rotated back into the object's horizontal plane),
//! intersected in frame order by Sutherland–Hodman (fixed order = fixed
//! bytes). The lifted `SliceStack` is then re-projected under ANY rigid pose
//! trajectory (`view`) to produce new 2D contour timelines — the three
//! `default_specs` trajectories below are the frozen demo's derived timelines.
//!
//! Properties the gate asserts (engine/tests/gate_hull.rs):
//! - containment: every lifted slice satisfies every drawn span exactly
//!   (the hull ⊇ true object, span-wise);
//! - monotone carving: lifting with more frames only shrinks sections
//!   (a patch frame never invalidates earlier work);
//! - byte determinism end-to-end (frozen hashes).
//!
//! Textbook algorithms only (half-plane clipping, marching squares,
//! Douglas–Peucker), written from their definitions — no external code.

use crate::math::{rotation_axis_angle, Mat4, Vec3};

/// One turnaround frame: a closed silhouette contour in screen coordinates
/// (x right, y up, z ignored) plus its pose angle about the lift (y) axis.
#[derive(Debug, Clone)]
pub struct Frame {
    pub angle_deg: f64,
    pub contour: Vec<Vec3>,
}

/// The lifted solid: one convex cross-section (in axis-centered object
/// coordinates x/z) per slice height. The lift axis is the vertical line
/// through the turnaround's global x-bbox center — contours are translated
/// by `center_x` before carving, so the stack lives centered on the axis.
/// An empty section means the hull is empty at that height.
#[derive(Debug, Clone)]
pub struct SliceStack {
    pub ys: Vec<f64>,
    /// Convex polygons as (x, z) pairs, one per `ys` entry (may be empty).
    pub sections: Vec<Vec<(f64, f64)>>,
    /// Document-space x of the lift axis (subtracted from the frames).
    pub center_x: f64,
}

/// A derived timeline: the lifted solid re-projected under one pose
/// trajectory. Loops are closed contours in view coordinates (x right,
/// y up), duplicated start point omitted.
#[derive(Debug, Clone)]
pub struct DerivedTimeline {
    pub name: String,
    /// Per frame: outline loops (x, y) in view coordinates.
    pub frames: Vec<Vec<Vec<(f64, f64)>>>,
}

/// Slices per lift, raster resolution and simplify epsilon per view — the
/// frozen demo constants (shared by the CLI and the gate tests).
pub const SLICES: usize = 128;
pub const GRID: usize = 256;
/// Douglas–Peucker epsilon in grid cells (cell = view extent / (GRID-1)).
pub const SIMPLIFY_EPS_CELLS: f64 = 2.0;

// ---------------------------------------------------------------------------
// lift
// ---------------------------------------------------------------------------

/// Screen-x span of a closed contour at height `y` (edge crossings, sorted,
/// outermost pair). None when the contour does not reach that height.
pub fn contour_span_x(contour: &[Vec3], y: f64) -> Option<(f64, f64)> {
    let n = contour.len();
    let mut xs: Vec<f64> = Vec::new();
    for i in 0..n {
        let p = &contour[i];
        let q = &contour[(i + 1) % n];
        // Half-open rule: exactly one of (p→q, q→p) crosses a given y,
        // so vertices on the scanline are handled deterministically.
        let crossing = (p.y <= y && q.y > y) || (q.y <= y && p.y > y);
        if crossing {
            let t = (y - p.y) / (q.y - p.y);
            xs.push(p.x + t * (q.x - p.x));
        }
    }
    if xs.len() < 2 {
        return None;
    }
    xs.sort_by(f64::total_cmp);
    Some((xs[0], xs[xs.len() - 1]))
}

/// Clip polygon by the half-plane `n·p ≤ off` (Sutherland–Hodman, fixed
/// traversal order — deterministic by construction).
fn clip_half_plane(poly: &[(f64, f64)], nx: f64, nz: f64, off: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(poly.len() + 1);
    let n = poly.len();
    if n == 0 {
        return out;
    }
    let side = |p: &(f64, f64)| nx * p.0 + nz * p.1 - off;
    for i in 0..n {
        let cur = &poly[i];
        let nxt = &poly[(i + 1) % n];
        let (sc, sn) = (side(cur), side(nxt));
        if sc <= 0.0 {
            out.push((cur.0, cur.1));
        }
        if (sc > 0.0) != (sn > 0.0) {
            let t = sc / (sc - sn);
            out.push((cur.0 + t * (nxt.0 - cur.0), cur.1 + t * (nxt.1 - cur.1)));
        }
    }
    out
}

/// Lift the turnaround into a visual hull. `slices` horizontal cuts sample
/// the contour height range at cell centers; each drawn frame contributes
/// two half-plane clips per slice, applied in frame order.
/// Lift with the axis chosen from the frames' global x-bbox center.
pub fn lift(frames: &[Frame], slices: usize) -> SliceStack {
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    for f in frames {
        for p in &f.contour {
            xmin = xmin.min(p.x);
            xmax = xmax.max(p.x);
        }
    }
    if !xmin.is_finite() {
        return SliceStack { ys: Vec::new(), sections: Vec::new(), center_x: 0.0 };
    }
    lift_centered(frames, slices, (xmin + xmax) / 2.0)
}

/// Lift about an explicit axis (document-space x). `lift()` picks the
/// turnaround's horizontal center; explicit callers (the carve-monotonicity
/// gate) pin the axis so two lifts of overlapping frame subsets live in the
/// same coordinate system.
pub fn lift_centered(frames: &[Frame], slices: usize, center_x: f64) -> SliceStack {
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    for f in frames {
        for p in &f.contour {
            ymin = ymin.min(p.y);
            ymax = ymax.max(p.y);
            xmin = xmin.min(p.x);
            xmax = xmax.max(p.x);
        }
    }
    if !ymin.is_finite() || slices == 0 || ymax <= ymin {
        return SliceStack { ys: Vec::new(), sections: Vec::new(), center_x };
    }
    let dy = (ymax - ymin) / slices as f64;
    // The constraint plane is axis-centered object-space (x, z); the z
    // extent under any yaw is bounded by the half screen width, so a
    // generous fixed box suffices.
    let e = xmax - xmin;
    let box_poly = vec![(-e, -e), (e, -e), (e, e), (-e, e)];
    let mut stack = SliceStack {
        ys: Vec::with_capacity(slices),
        sections: Vec::with_capacity(slices),
        center_x,
    };
    for j in 0..slices {
        let y = ymin + (j as f64 + 0.5) * dy;
        let mut poly = box_poly.clone();
        for f in frames {
            let Some((a, b)) = contour_span_x(&f.contour, y) else { continue };
            // Axis-centered span (the carve rotates about the axis).
            let (a, b) = (a - center_x, b - center_x);
            let th = f.angle_deg.to_radians();
            let (c, s) = th.sin_cos();
            // Rotating the object by θ, screen-x of a point p is
            // c·p.x + s·p.z → span constraint a ≤ c·x + s·z ≤ b.
            poly = clip_half_plane(&poly, c, s, b);
            if poly.is_empty() {
                break;
            }
            poly = clip_half_plane(&poly, -c, -s, -a);
            if poly.is_empty() {
                break;
            }
        }
        stack.ys.push(y);
        stack.sections.push(poly);
    }
    stack
}
/// Shoelace area of a simple polygon (positive = counter-clockwise in a
/// y-up plane).
pub fn shoelace_area(poly: &[(f64, f64)]) -> f64 {
    let n = poly.len();
    let mut s = 0.0;
    for i in 0..n {
        let p = &poly[i];
        let q = &poly[(i + 1) % n];
        s += p.0 * q.1 - q.0 * p.1;
    }
    0.5 * s
}

/// Span of a lifted section projected under pose `angle_deg` (screen-x
/// extent after rotating the slice by the pose). The gate's round-trip
/// primitive: containment against the drawn span is exact by construction.
pub fn section_span_x(section: &[(f64, f64)], angle_deg: f64) -> Option<(f64, f64)> {
    if section.is_empty() {
        return None;
    }
    let th = angle_deg.to_radians();
    let (c, s) = th.sin_cos();
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for (x, z) in section {
        let x2 = c * *x + s * *z;
        lo = lo.min(x2);
        hi = hi.max(x2);
    }
    Some((lo, hi))
}

// ---------------------------------------------------------------------------
// view
// ---------------------------------------------------------------------------

/// Boundary loops of the lifted solid under `model_view`, extracted on a
/// fixed GRID×GRID raster (even-odd fill of all projected sections →
/// marching squares → chained loops → Douglas–Peucker). Output loops are in
/// view coordinates (the model_view result's x/y), counter-clockwise,
/// start vertex = lexicographic minimum — all fixed orders, fixed bytes.
pub fn view(stack: &SliceStack, model_view: &Mat4, grid: usize, eps_cells: f64) -> Vec<Vec<(f64, f64)>> {
    // 1. Project every section vertex into view space, then build one convex
    //    hull per slab (ring j ∪ ring j+1): a slab is the convex hull of its
    //    two parallel rings, and projection commutes with convex hull, so the
    //    slab hulls tile the projected silhouette. (Rasterizing bare rings
    //    would give zero coverage under yaw poses — flat rings never cross a
    //    horizontal scanline.)
    let mut rings: Vec<Option<Vec<(f64, f64)>>> = Vec::with_capacity(stack.sections.len());
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for (j, sec) in stack.sections.iter().enumerate() {
        if sec.is_empty() {
            rings.push(None);
            continue;
        }
        let y = stack.ys[j];
        let mut vp = Vec::with_capacity(sec.len());
        for (x, z) in sec {
            let (p, w) = model_view.project(Vec3::new(*x, y, *z));
            debug_assert!(w != 0.0);
            vp.push((p.x, p.y));
            xmin = xmin.min(p.x);
            xmax = xmax.max(p.x);
            ymin = ymin.min(p.y);
            ymax = ymax.max(p.y);
        }
        rings.push(Some(vp));
    }
    let mut polys: Vec<Vec<(f64, f64)>> = Vec::new();
    let n = rings.len();
    for j in 0..n {
        let Some(cur) = &rings[j] else { continue };
        match rings.get(j + 1) {
            Some(Some(nxt)) => {
                let mut pts = cur.clone();
                pts.extend(nxt.iter().copied());
                polys.push(convex_hull(pts));
            }
            _ => polys.push(convex_hull(cur.clone())), // cap ring (boundary slab)
        }
    }
    if polys.is_empty() || grid < 2 {
        return Vec::new();
    }
    // Pad so boundary samples are strictly outside (loops stay closed).
    let (mut wx, mut wy) = (xmax - xmin, ymax - ymin);
    if wx <= 0.0 {
        wx = 1.0;
    }
    if wy <= 0.0 {
        wy = 1.0;
    }
    xmin -= wx * 0.05;
    xmax += wx * 0.05;
    ymin -= wy * 0.05;
    ymax += wy * 0.05;
    let (wx, wy) = (xmax - xmin, ymax - ymin);
    let cell_x = wx / (grid - 1) as f64;
    let cell_y = wy / (grid - 1) as f64;

    // 2. Union raster: per sample row, DIRECTIONAL crossings of all polygons
    //    (+1 upward edge / −1 downward edge), swept in x order — coverage > 0
    //    is inside the union. (Plain even-odd parity breaks under overlap:
    //    two overlapping convex hulls pair each other's crossings into an
    //    even count, punching holes in the interior.)
    let mut mask = vec![false; grid * grid];
    let mut xs: Vec<(f64, i32)> = Vec::new();
    for j in 0..grid {
        let y = ymin + j as f64 * cell_y;
        xs.clear();
        for poly in &polys {
            let n = poly.len();
            for i in 0..n {
                let p = &poly[i];
                let q = &poly[(i + 1) % n];
                if p.1 <= y && q.1 > y {
                    let t = (y - p.1) / (q.1 - p.1);
                    xs.push((p.0 + t * (q.0 - p.0), 1));
                } else if q.1 <= y && p.1 > y {
                    let t = (y - q.1) / (p.1 - q.1);
                    xs.push((q.0 + t * (p.0 - q.0), -1));
                }
            }
        }
        if xs.is_empty() {
            continue;
        }
        xs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut k = 0usize;
        let mut cov = 0i32;
        for i in 0..grid {
            let x = xmin + i as f64 * cell_x;
            while k < xs.len() && xs[k].0 <= x {
                cov += xs[k].1;
                k += 1;
            }
            // Non-zero winding: the slab hulls all share one orientation, so
            // overlap adds up and never cancels; sign is convention-agnostic.
            if cov != 0 {
                mask[j * grid + i] = true;
            }
        }
    }
    // 3. Marching squares on cell-center samples; segment endpoints are
    //    edge midpoints identified by edge id so adjacent cells chain exactly.
    //    Edge id: (1<<62 | i<<32 | j) horizontal h(i,j) between samples
    //    (i,j)-(i+1,j); (1<<61 | ...) vertical v(i,j) between (i,j)-(i,j+1).
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    struct EdgeId(u64);
    let h = |i: usize, j: usize| EdgeId((1u64 << 62) | ((i as u64) << 32) | j as u64);
    let v = |i: usize, j: usize| EdgeId((1u64 << 61) | ((i as u64) << 32) | j as u64);
    let mut segs: Vec<(EdgeId, EdgeId)> = Vec::new();
    for j in 0..grid - 1 {
        for i in 0..grid - 1 {
            let a = mask[j * grid + i] as u8;
            let b = mask[j * grid + i + 1] as u8;
            let c = mask[(j + 1) * grid + i + 1] as u8;
            let d = mask[(j + 1) * grid + i] as u8;
            let case = a | (b << 1) | (c << 2) | (d << 3);
            let mut push = |s: Vec<(EdgeId, EdgeId)>| segs.extend(s);
            match case {
                0 | 15 => {}
                1 => push(vec![(v(i, j), h(i, j))]),
                2 => push(vec![(h(i, j), v(i + 1, j))]),
                3 => push(vec![(v(i, j), v(i + 1, j))]),
                4 => push(vec![(v(i + 1, j), h(i, j + 1))]),
                // Ambiguous saddles: fixed pairing (deterministic bytes).
                5 => push(vec![(v(i, j), h(i, j)), (v(i + 1, j), h(i, j + 1))]),
                6 => push(vec![(h(i, j), h(i, j + 1))]),
                7 => push(vec![(v(i, j), h(i, j + 1))]),
                8 => push(vec![(h(i, j + 1), v(i, j))]),
                9 => push(vec![(h(i, j), h(i, j + 1))]),
                10 => push(vec![(v(i, j), h(i, j + 1)), (h(i, j), v(i + 1, j))]),
                11 => push(vec![(h(i, j + 1), v(i + 1, j))]),
                12 => push(vec![(v(i + 1, j), v(i, j))]),
                13 => push(vec![(h(i, j), v(i + 1, j))]),
                14 => push(vec![(h(i, j + 1), v(i, j))]),
                _ => unreachable!(),
            }
        }
    }
    if segs.is_empty() {
        return Vec::new();
    }

    // 4. Chain segments into loops via shared edge ids.
    use std::collections::HashMap;
    let mut adj: HashMap<EdgeId, Vec<usize>> = HashMap::new();
    for (idx, (a, b)) in segs.iter().enumerate() {
        adj.entry(*a).or_default().push(idx);
        adj.entry(*b).or_default().push(idx);
    }
    let mut used = vec![false; segs.len()];
    let mid = |e: EdgeId| -> (f64, f64) {
        // Mask excludes the marker bits (61/62) after the shift.
        let i = ((e.0 >> 32) & 0x1FFF_FFFF) as usize;
        let j = (e.0 & 0xFFFF_FFFF) as usize;
        if e.0 >> 62 == 1 {
            (xmin + (i as f64 + 0.5) * cell_x, ymin + j as f64 * cell_y)
        } else {
            (xmin + i as f64 * cell_x, ymin + (j as f64 + 0.5) * cell_y)
        }
    };
    let mut loops: Vec<Vec<(f64, f64)>> = Vec::new();
    for s0 in 0..segs.len() {
        if used[s0] {
            continue;
        }
        used[s0] = true;
        let mut pts = vec![mid(segs[s0].0), mid(segs[s0].1)];
        let mut open_edge = segs[s0].1;
        loop {
            let cands = adj.get(&open_edge).cloned().unwrap_or_default();
            let mut next: Option<usize> = None;
            for &c in &cands {
                if !used[c] {
                    next = Some(c);
                    break;
                }
            }
            match next {
                Some(ci) => {
                    used[ci] = true;
                    let (a, b) = segs[ci];
                    let other = if a == open_edge { b } else { a };
                    pts.push(mid(other));
                    open_edge = other;
                }
                None => break,
            }
        }
        if pts.len() >= 3 {
            loops.push(pts);
        }
    }

    // 5. Simplify + canonical orientation/start.
    let eps = eps_cells * cell_x.max(cell_y);
    let mut out = Vec::with_capacity(loops.len());
    for pts in loops {
        // Drop the duplicated closing point; canonical start = min vertex;
        // counter-clockwise orientation.
        let mut ring = pts;
        if ring.len() > 1 && ring[0] == ring[ring.len() - 1] {
            ring.pop();
        }
        if ring.len() < 3 {
            continue;
        }
        if shoelace_area(&ring) < 0.0 {
            ring.reverse();
        }
        let start = (0..ring.len())
            .min_by(|&a, &b| cmp_pt(ring[a], ring[b]).then(a.cmp(&b)))
            .unwrap();
        ring.rotate_left(start);
        let simplified = dp_simplify(&ring, eps);
        if simplified.len() >= 3 {
            out.push(simplified);
        }
    }
    // Drop raster specks: under glancing poses a thin solid bridge can break
    // into a tiny disconnected component at grid resolution. A SliceStack is
    // one connected solid, so keep loops within a relative area band of the
    // largest (deterministic; single-outline timelines keep their bytes).
    if let Some(max_area) = out.iter().map(|l| shoelace_area(l).abs()).max_by(f64::total_cmp) {
        let floor = (max_area * 0.02).max(4.0);
        out.retain(|l| shoelace_area(l).abs() >= floor);
    }
    out.sort_by(|a, b| cmp_poly(a, b));
    out
}

fn cmp_pt(a: (f64, f64), b: (f64, f64)) -> std::cmp::Ordering {
    a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1))
}

/// Convex hull, Andrew monotone chain (counter-clockwise, collinear points
/// dropped). Deterministic: total-order sort, fixed traversal.
fn convex_hull(mut pts: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    pts.sort_by(|a, b| cmp_pt(*a, *b));
    pts.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    let n = pts.len();
    if n < 3 {
        return pts;
    }
    let cross = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let mut lower: Vec<(f64, f64)> = Vec::with_capacity(n);
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<(f64, f64)> = Vec::with_capacity(n);
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn cmp_poly(a: &[(f64, f64)], b: &[(f64, f64)]) -> std::cmp::Ordering {
    let n = a.len().min(b.len());
    for k in 0..n {
        let o = cmp_pt(a[k], b[k]);
        if o != std::cmp::Ordering::Equal {
            return o;
        }
    }
    a.len().cmp(&b.len())
}

/// Iterative Douglas–Peucker on an open polyline (first != last kept).
fn dp_simplify(pts: &[(f64, f64)], eps: f64) -> Vec<(f64, f64)> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let mut stack = vec![(0usize, n - 1usize)];
    while let Some((i0, i1)) = stack.pop() {
        if i1 <= i0 + 1 {
            continue;
        }
        let (a, b) = (pts[i0], pts[i1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = dx * dx + dy * dy;
        let mut best = (0usize, -1.0f64);
        for k in i0 + 1..i1 {
            let p = &pts[k];
            let d2 = if len2 > 0.0 {
                let t = ((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2;
                let (cx, cy) = (a.0 + t * dx, a.1 + t * dy);
                (p.0 - cx).powi(2) + (p.1 - cy).powi(2)
            } else {
                (p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)
            };
            if d2 > best.1 {
                best = (k, d2);
            }
        }
        if best.1 > eps * eps {
            keep[best.0] = true;
            stack.push((i0, best.0));
            stack.push((best.0, i1));
        }
    }
    (0..n).filter(|&k| keep[k]).map(|k| pts[k]).collect()
}

// ---------------------------------------------------------------------------
// derived timelines
// ---------------------------------------------------------------------------

/// One pose trajectory: rotate by `angles_deg[k]` about `axis` through
/// `pivot`, view from +z (screen = world x/y).
#[derive(Debug, Clone)]
pub struct DerivedSpec {
    pub name: &'static str,
    pub axis: [f64; 3],
    pub pivot: [f64; 3],
    pub angles_deg: Vec<f64>,
}

/// Bounding-box center of the stack (the default rotation pivot).
pub fn stack_pivot(stack: &SliceStack) -> [f64; 3] {
    let (mut x0, mut x1, mut z0, mut z1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    let mut y0 = f64::INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for (j, sec) in stack.sections.iter().enumerate() {
        for (x, z) in sec {
            x0 = x0.min(*x);
            x1 = x1.max(*x);
            z0 = z0.min(*z);
            z1 = z1.max(*z);
        }
        if !sec.is_empty() {
            y0 = y0.min(stack.ys[j]);
            y1 = y1.max(stack.ys[j]);
        }
    }
    [
        (x0 + x1) / 2.0,
        (y0 + y1) / 2.0,
        (z0 + z1) / 2.0,
    ]
}

/// The frozen demo's three trajectories: same-axis re-timing (includes
/// undrawn in-between angles), a nod about the horizontal axis through the
/// pivot, and a full revolution about the diagonal — three different 3D
/// rotations from one authored turnaround.
pub fn default_specs(stack: &SliceStack) -> Vec<DerivedSpec> {
    let p = stack_pivot(stack);
    let mut swing = Vec::new();
    for k in 0..16 {
        swing.push(-30.0 + 60.0 * k as f64 / 15.0);
    }
    let mut nod = Vec::new();
    for k in 0..16 {
        nod.push(-25.0 + 50.0 * k as f64 / 15.0);
    }
    let mut orbit = Vec::new();
    for k in 0..24 {
        orbit.push(360.0 * k as f64 / 24.0);
    }
    vec![
        DerivedSpec { name: "yawSwing", axis: [0.0, 1.0, 0.0], pivot: p, angles_deg: swing },
        DerivedSpec { name: "pitchNod", axis: [1.0, 0.0, 0.0], pivot: p, angles_deg: nod },
        DerivedSpec { name: "orbitDiagonal", axis: [1.0, 1.0, 1.0], pivot: p, angles_deg: orbit },
    ]
}

/// Derive all timelines for the given specs. The camera sits on +z looking
/// at −z, so screen (x, y) = pose-matrix (x, y) — no separate view matrix.
pub fn derive_timelines(stack: &SliceStack, specs: &[DerivedSpec]) -> Vec<DerivedTimeline> {
    let mut out = Vec::new();
    for spec in specs {
        let mut frames = Vec::with_capacity(spec.angles_deg.len());
        for &ang in &spec.angles_deg {
            let rot = rotation_axis_angle(Vec3::new(spec.axis[0], spec.axis[1], spec.axis[2]), ang.to_radians());
            // Model-view = T(pivot) · R · T(−pivot).
            let mv = pose_matrix(&rot, &spec.pivot);
            frames.push(view(stack, &mv, GRID, SIMPLIFY_EPS_CELLS));
        }
        out.push(DerivedTimeline { name: spec.name.to_string(), frames });
    }
    out
}

/// Model-view = T(pivot) · R · T(−pivot) (affine, w stays 1).
fn pose_matrix(rot: &Mat4, pivot: &[f64; 3]) -> Mat4 {
    let mut m = *rot;
    // Last column of the product: R·(−p) + p.
    let rp = rot.project(Vec3::new(-pivot[0], -pivot[1], -pivot[2])).0;
    m.0[3][0] = rp.x + pivot[0];
    m.0[3][1] = rp.y + pivot[1];
    m.0[3][2] = rp.z + pivot[2];
    m
}

/// Canonical artifact bytes for a derived timeline (shortest-round-trip f64
/// via serde-less Display; fixed field order).
pub fn timeline_json(t: &DerivedTimeline) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, r#"{{"name":"{}","frameCount":{},"frames":["#, t.name, t.frames.len());
    for (fi, loops) in t.frames.iter().enumerate() {
        let _ = write!(s, r#"  {{"loops":["#);
        for (li, loop_pts) in loops.iter().enumerate() {
            let _ = write!(s, "[");
            for (pi, (x, y)) in loop_pts.iter().enumerate() {
                if pi > 0 {
                    let _ = write!(s, ",");
                }
                let _ = write!(s, "[{},{}]", x, y);
            }
            let _ = write!(s, "]{}", if li + 1 < loops.len() { "," } else { "" });
        }
        let _ = writeln!(s, "]}}{}", if fi + 1 < t.frames.len() { "," } else { "" });
    }
    let _ = write!(s, "]}}\n");
    s
}
