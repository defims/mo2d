//! Evaluation: relation DAG → topological order → per-frame evaluation.
//!
//! Pipeline per tick (docs/relations.md §4):
//!   1. working state = rest document state (positions/widths/colors)
//!   2. relations in topological order: track (sample at tick) → share → bind
//!   3. result feeds the renderer
//!
//! Determinism: Kahn's algorithm with stable (document-order) tie-breaking,
//! fixed iteration counts in the bezier solver, no hash-ordered iteration.

use crate::json::Value;
use crate::math::Vec3;
use crate::model::*;

/// An evaluated node as seen by the renderer.
#[derive(Debug, Clone)]
pub struct EvalNode {
    pub p: Vec3,
    pub width: f64,
    pub color: [u8; 4],
    pub handle_in: Option<Vec3>,
    pub handle_out: Option<Vec3>,
}

#[derive(Debug, Clone)]
pub struct EvalLine {
    pub id: String,
    pub tags: Vec<String>,
    pub closed: bool,
    pub stroke_on: bool,
    pub fill_on: bool,
    pub fill_color: [u8; 4],
    pub width: f64,
    pub nodes: Vec<EvalNode>,
    pub segs: Vec<SegAttrs>,
}

impl EvalLine {
    pub fn seg(&self, i: usize) -> &SegAttrs {
        self.segs.get(i).unwrap_or(&STATIC_SEG)
    }
}

static STATIC_SEG: SegAttrs = SegAttrs {
    seg_type: SegType::Straight,
    visible: true,
    interp: None,
    unknown: Vec::new(),
};

/// Topologically sort relations. Edge R1→R2 when R2 reads a line R1 writes.
/// Returns the evaluation order (indices into doc.relations) or a cycle error.
pub fn topo_order(doc: &Document) -> Result<Vec<usize>, String> {
    let n = doc.relations.len();
    let mut order = Vec::with_capacity(n);
    let mut placed = vec![false; n];
    let mut remaining = n;

    // Deterministic multi-pass: each pass places every relation whose inputs
    // are already satisfied, in document order. O(n²) worst case — MVP scale.
    while remaining > 0 {
        let mut progress = false;
        for i in 0..n {
            if placed[i] {
                continue;
            }
            let r = &doc.relations[i];
            let inputs_clear = r.reads().iter().copied().all(|read| {
                doc.relations
                    .iter()
                    .enumerate()
                    .all(|(j, other)| placed[j] || j == i || other.writes() != read)
            });
            if inputs_clear {
                order.push(i);
                placed[i] = true;
                remaining -= 1;
                progress = true;
            }
        }
        if !progress {
            let stuck: Vec<String> = doc
                .relations
                .iter()
                .enumerate()
                .filter(|(i, _)| !placed[*i])
                .map(|(_, r)| r.id.clone())
                .collect();
            return Err(format!("relation cycle detected among: {}", stuck.join(", ")));
        }
    }
    Ok(order)
}

/// Evaluate the document at `tick`.
pub fn evaluate(doc: &Document, tick: i64) -> Result<Vec<EvalLine>, String> {
    // Working copy: positions/widths/colors start from rest state.
    let mut lines: Vec<EvalLine> = doc
        .lines
        .iter()
        .map(|l| EvalLine {
            id: l.id.clone(),
            tags: l.tags.clone(),
            closed: l.closed,
            stroke_on: l.stroke_on,
            fill_on: l.fill_on,
            fill_color: l.fill_color,
            width: l.width,
            nodes: l
                .nodes
                .iter()
                .map(|n| EvalNode {
                    p: match n.pos {
                        NodePos::Geom(v) => v,
                        NodePos::Tick(t) => Vec3::new(t as f64, 0.0, 0.0), // placeholder, never rendered
                    },
                    width: n.eff_width(l.width),
                    color: n.eff_color(l.color),
                    handle_in: n.handle_in,
                    handle_out: n.handle_out,
                })
                .collect(),
            segs: l.segs.clone(),
        })
        .collect();

    let find_line = |lines: &[EvalLine], id: &str| -> Option<usize> { lines.iter().position(|l| l.id == id) };

    let order = topo_order(doc)?;
    for ri in order {
        let r = &doc.relations[ri];
        match r.kind {
            RelationKind::Track => {
                let Some(tli) = find_line(&lines, &r.track_line) else { continue };
                // Sample the track at `tick` for each component.
                let values = sample_track(&doc.lines[tli], tick)?;
                let target_line = r.track_target_line.clone();
                match r.track_attr {
                    TrackAttr::Position => {
                        if let Some(node_idx) = r.track_target_node {
                            if let Some(ti) = find_line(&lines, &target_line) {
                                if node_idx < lines[ti].nodes.len() {
                                    if values.len() == 3 {
                                        let v = Vec3::new(values[0], values[1], values[2]);
                                        lines[ti].nodes[node_idx].p = v;
                                    }
                                }
                            }
                        }
                    }
                    TrackAttr::Width => {
                        let v = values[0];
                        if let Some(node_idx) = r.track_target_node {
                            if let Some(ti) = find_line(&lines, &target_line) {
                                if node_idx < lines[ti].nodes.len() {
                                    lines[ti].nodes[node_idx].width = v;
                                }
                            }
                        } else {
                            // Line-level width track.
                            if let Some(ti) = find_line(&lines, &target_line) {
                                lines[ti].width = v;
                                for node in &mut lines[ti].nodes {
                                    node.width = v;
                                }
                            }
                        }
                    }
                }
            }
            RelationKind::Share => {
                let (dl, dn) = r.share_driver.clone();
                let (rl, rn) = r.share_driven.clone();
                if let Some(di) = find_line(&lines, &dl) {
                    if dn < lines[di].nodes.len() {
                        let p = lines[di].nodes[dn].p;
                        if let Some(ri2) = find_line(&lines, &rl) {
                            if rn < lines[ri2].nodes.len() {
                                lines[ri2].nodes[rn].p = p;
                            }
                        }
                    }
                }
            }
            RelationKind::Bind => {
                apply_bind(doc, &mut lines, &r.bind_target, &r.bind_skeleton, r.bind_falloff, r.bind_stretch)?;
            }
        }
    }
    Ok(lines)
}

/// Sample a track line at `tick`: returns per-component f64 values.
/// Before the first key → first value; after the last → last value (clamped).
pub fn sample_track(track: &Line, tick: i64) -> Result<Vec<f64>, String> {
    if track.nodes.is_empty() {
        return Err(format!("track '{}' is empty", track.id));
    }
    let ticks: Vec<i64> = track.nodes.iter().map(|n| n.pos.tick().unwrap_or(0)).collect();
    let values: Vec<Vec<f64>> = track
        .nodes
        .iter()
        .map(|n| match &n.value {
            Some(Value::Num(x)) => Ok(vec![*x]),
            Some(Value::Int(x)) => Ok(vec![*x as f64]),
            Some(Value::Arr(a)) => Ok(a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect()),
            other => Err(format!("track '{}': keyframe value must be number or array, got {}", track.id, other.as_ref().map(|v| v.type_name()).unwrap_or("none"))),
        })
        .collect::<Result<_, _>>()?;

    if tick <= ticks[0] {
        return Ok(values[0].clone());
    }
    if tick >= *ticks.last().unwrap() {
        return Ok(values.last().unwrap().clone());
    }
    // Exact key hits return the keyframe value verbatim (no ulp drift).
    if let Some(i) = ticks.iter().position(|&t| t == tick) {
        return Ok(values[i].clone());
    }
    // Find segment [i, i+1] containing tick.
    let mut i = 0;
    for k in 0..ticks.len() - 1 {
        if ticks[k] <= tick && tick <= ticks[k + 1] {
            i = k;
            break;
        }
    }
    let t0 = ticks[i];
    let t1 = ticks[i + 1];
    let span = (t1 - t0) as f64;
    let u = if span > 0.0 { (tick - t0) as f64 / span } else { 0.0 };

    let interp = track.segs.get(i).and_then(|s| s.interp.clone());
    let comps = values[i].len();
    let mut out = Vec::with_capacity(comps);
    match interp {
        Some(Interp { im: 9, b }) => {
            for c in 0..comps {
                let h = b.as_ref().map(|bb| bb.get(c).copied()).flatten().unwrap_or_else(BezierHandle::default_handle);
                let v0 = values[i][c];
                let v1 = values[i + 1][c];
                out.push(bezier_eval(v0, v1, u, &h));
            }
        }
        _ => {
            // Linear (im=0) and anything unset.
            for c in 0..comps {
                out.push(values[i][c] + (values[i + 1][c] - values[i][c]) * u);
            }
        }
    }
    Ok(out)
}

/// Cubic bezier between (0, v0) and (1, v1) with Moho-style handles.
/// Control points:
///   C1 = (po·cos(ao), v0 + po·sin(ao)·Δv)   (out of key i)
///   C2 = (1 − pi·cos(ai), v1 − pi·sin(ai)·Δv) (into key i+1)
/// x stays monotonic for po,pi ∈ [0,1]; solve x(s)=u by 48 bisection steps
/// (fixed count → deterministic on every platform).
fn bezier_eval(v0: f64, v1: f64, u: f64, h: &BezierHandle) -> f64 {
    let dv = v1 - v0;
    let c1x = h.po * h.ao.cos();
    let c1y = v0 + h.po * h.ao.sin() * dv;
    let c2x = 1.0 - h.pi * h.ai.cos();
    let c2y = v1 - h.pi * h.ai.sin() * dv;

    let bezier_x = |s: f64| -> f64 {
        let m = 1.0 - s;
        3.0 * m * m * s * c1x + 3.0 * m * s * s * c2x + s * s * s
    };
    let bezier_y = |s: f64| -> f64 {
        let m = 1.0 - s;
        3.0 * m * m * s * c1y + 3.0 * m * s * s * c2y + s * s * s * v1
    };

    let mut lo = 0.0f64;
    let mut hi = 1.0f64;
    for _ in 0..48 {
        let mid = (lo + hi) / 2.0;
        if bezier_x(mid) < u {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bezier_y((lo + hi) / 2.0)
}

/// Bind: body line follows skeleton polyline (3d-extension.md decision #4:
/// weights from point-to-segment **projection distance**, same formula in 2D/3D).
///
/// Rest state (the un-evaluated document) fixes, per body node:
///   - nearest skeleton segment k (by projection distance), parameter u ∈ [0,1]
///   - rest offset o = p_rest − lerp(A_rest, B_rest, u), rest distance d = |o|
/// At eval time, the offset rides the bone's local frame (rotate rest-dir →
/// cur-dir, Rodrigues; same formula in 2D and 3D):
///   follow = lerp(A', B', u) + R(θ)·o · (|A'B'|/|AB| if stretch else 1)
///   weight w = 1 − clamp(d / falloff, 0, 1)   (falloff ≤ 0 → rigid, w = 1)
///   p' = lerp(p_rest, follow, w)
fn apply_bind(
    doc: &Document,
    lines: &mut [EvalLine],
    target_id: &str,
    skeleton_id: &str,
    falloff: f64,
    stretch: bool,
) -> Result<(), String> {
    // Rest state comes from the document, never from the working copy
    // (bind offsets are constant per document edit — deterministic).
    let skel_line = doc.line(skeleton_id)?;
    let body_line = doc.line(target_id)?;

    let rest_skel: Vec<Vec3> = skel_line
        .nodes
        .iter()
        .map(|n| n.pos.geom().unwrap_or(Vec3::ZERO))
        .collect();
    if rest_skel.len() < 2 {
        return Err(format!("bind: skeleton '{}' needs ≥ 2 nodes", skeleton_id));
    }

    // Precompute per body node: (segment k, u, offset, dist).
    let mut binds: Vec<(usize, f64, Vec3, f64)> = Vec::with_capacity(body_line.nodes.len());
    for n in &body_line.nodes {
        let p = n.pos.geom().map_err(|_| format!("bind: target line '{}' has track nodes — cannot bind", target_id))?;
        binds.push(nearest_on_polyline(p, &rest_skel));
    }

    // Eval-time skeleton positions come from the working copy (post-track/share).
    let si = lines.iter().position(|l| l.id == skeleton_id).ok_or_else(|| format!("bind: skeleton '{}' missing", skeleton_id))?;
    let cur_skel: Vec<Vec3> = lines[si].nodes.iter().map(|n| n.p).collect();

    let ti = lines.iter().position(|l| l.id == target_id).ok_or_else(|| format!("bind: target '{}' missing", target_id))?;
    for (idx, (k, u, off, d)) in binds.iter().enumerate() {
        let a = cur_skel[*k];
        let b = cur_skel[*k + 1];
        let rest_a = rest_skel[*k];
        let rest_b = rest_skel[*k + 1];
        let rest_len = rest_b.sub(rest_a).len();
        let cur_len = b.sub(a).len();
        let scale = if stretch && rest_len > 1e-12 { cur_len / rest_len } else { 1.0 };
        let rotated = rotate_offset(rest_a, rest_b, a, b, *off);
        let follow = Vec3::lerp(a, b, *u).add(rotated.scale(scale));
        let w = if falloff > 0.0 { (1.0 - (d / falloff)).clamp(0.0, 1.0) } else { 1.0 };
        let rest_p = body_line.nodes[idx].pos.geom().unwrap_or(Vec3::ZERO);
        let new_p = Vec3::lerp(rest_p, follow, w);
        if idx < lines[ti].nodes.len() {
            lines[ti].nodes[idx].p = new_p;
        }
    }
    Ok(())
}

/// Rotate a rest-frame offset into the bone's current frame: the minimal
/// rotation taking the rest segment direction to the current one (Rodrigues).
/// Collinear directions → identity; anti-parallel → rotate 180° around any
/// perpendicular axis (deterministic pick: the rest-direction × world-up,
/// falling back to world-x when parallel to up).
fn rotate_offset(rest_a: Vec3, rest_b: Vec3, a: Vec3, b: Vec3, off: Vec3) -> Vec3 {
    let rest_dir = rest_b.sub(rest_a);
    let cur_dir = b.sub(a);
    if rest_dir.len() < 1e-12 || cur_dir.len() < 1e-12 || off.len() < 1e-12 {
        return off;
    }
    let rd = rest_dir.normalize();
    let cd = cur_dir.normalize();
    let c = rd.cross(cd);
    let sin_a = c.len();
    let cos_a = rd.dot(cd);
    if sin_a < 1e-12 {
        if cos_a > 0.0 {
            return off; // same direction — identity
        }
        // Anti-parallel: pick a stable perpendicular axis.
        let axis = {
            let up = Vec3::new(0.0, 0.0, 1.0);
            let candidate = if rd.cross(up).len() > 1e-9 { rd.cross(up) } else { rd.cross(Vec3::new(1.0, 0.0, 0.0)) };
            candidate.normalize()
        };
        // 180° rotation around `axis`: v' = 2(k·v)k − v.
        return axis.scale(off.dot(axis)).scale(2.0).sub(off);
    }
    let angle = sin_a.atan2(cos_a);
    let axis = c.scale(1.0 / sin_a);
    // Rodrigues: v·cosθ + (k×v)·sinθ + k·(k·v)·(1−cosθ)
    let k = axis;
    let term1 = off.scale(angle.cos());
    let term2 = k.cross(off).scale(angle.sin());
    let term3 = k.scale(k.dot(off)).scale(1.0 - angle.cos());
    term1.add(term2).add(term3)
}

/// Nearest point on a polyline: returns (segment index, u, offset vector, distance).
fn nearest_on_polyline(p: Vec3, pts: &[Vec3]) -> (usize, f64, Vec3, f64) {
    let mut best = (0usize, 0.0f64, p, f64::INFINITY);
    for k in 0..pts.len() - 1 {
        let (u, dist, closest) = point_segment(p, pts[k], pts[k + 1]);
        if dist < best.3 {
            best = (k, u, p.sub(closest), dist);
        }
    }
    best
}

/// Point-to-segment projection: returns (u, distance, closest point).
/// Works identically in 2D (z=0) and 3D — the frozen bind formula.
pub fn point_segment(p: Vec3, a: Vec3, b: Vec3) -> (f64, f64, Vec3) {
    let ab = b.sub(a);
    let len2 = ab.dot(ab);
    let u = if len2 <= 1e-12 { 0.0 } else { (p.sub(a).dot(ab) / len2).clamp(0.0, 1.0) };
    let closest = a.add(ab.scale(u));
    let d = p.sub(closest).len();
    (u, d, closest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_doc() -> Document {
        let mut doc = Document::default();
        // skeleton: 3 nodes along x
        let mut skel = Line::new("skel");
        skel.tags = vec!["skeleton".into()];
        skel.nodes = vec![Node::geom(Vec3::new(0.0, 0.0, 0.0)), Node::geom(Vec3::new(50.0, 0.0, 0.0)), Node::geom(Vec3::new(100.0, 0.0, 0.0))];
        skel.sync_segs();
        // body: node above the middle of segment 1
        let mut body = Line::new("body");
        body.nodes = vec![Node::geom(Vec3::new(25.0, 10.0, 0.0)), Node::geom(Vec3::new(75.0, 10.0, 0.0))];
        body.sync_segs();
        // free line (unbound negative control)
        let mut free = Line::new("free");
        free.nodes = vec![Node::geom(Vec3::new(25.0, 10.0, 0.0)), Node::geom(Vec3::new(75.0, 10.0, 0.0))];
        free.sync_segs();
        doc.lines = vec![skel, body, free];
        doc.relations.push(Relation::new_bind("b", "body", "skel"));
        doc
    }

    #[test]
    fn bind_follows_skeleton() {
        // Deformation via a track on the skeleton (the rest state IS the
        // document — animation comes from relations, not rest edits).
        let mut doc = build_doc();
        let trk = r#"{"v":1,"cmds":[
          {"op":"line.create","id":"trk_skel_2_position","tags":["track"],
           "nodes":[{"p":[0],"v":[100,0,0]},{"p":[254016000000],"v":[100,50,0]}]},
          {"op":"relation.create","type":"track","trackLine":"trk_skel_2_position",
           "target":{"line":"skel","node":2,"attr":"position"}}
        ]}"#;
        let cmds = crate::command::parse_stream(trk).unwrap();
        crate::command::apply_stream(&mut doc, &cmds).unwrap();

        let at_rest = evaluate(&doc, 0).unwrap();
        let at_end = evaluate(&doc, 254016000000).unwrap();
        let body0 = at_rest.iter().find(|l| l.id == "body").unwrap();
        let body1 = at_end.iter().find(|l| l.id == "body").unwrap();
        let free = at_end.iter().find(|l| l.id == "free").unwrap();
        // Bound body deforms with the animated bone end.
        assert!(
            (body1.nodes[1].p.y - body0.nodes[1].p.y).abs() > 10.0,
            "bound body should deform, Dy={}",
            body1.nodes[1].p.y - body0.nodes[1].p.y
        );
        // Unbound control stays at rest.
        assert_eq!(free.nodes[1].p, Vec3::new(75.0, 10.0, 0.0), "unbound line must stay put");
    }

    #[test]
    fn track_sampling_linear_and_bezier() {
        let mut trk = Line::new("t");
        trk.nodes = vec![Node::track(0, Value::Num(0.0)), Node::track(100, Value::Num(10.0))];
        trk.sync_segs();
        let v = sample_track(&trk, 50).unwrap();
        assert_eq!(v[0], 5.0);
        // Out-of-range clamps.
        assert_eq!(sample_track(&trk, -10).unwrap()[0], 0.0);
        assert_eq!(sample_track(&trk, 1000).unwrap()[0], 10.0);

        // Bezier with horizontal handles (ao=ai=0, po=pi=1/3) eases in the middle.
        trk.segs[0].interp = Some(Interp { im: 9, b: Some(vec![BezierHandle { ao: 0.0, ai: 0.0, po: 1.0 / 3.0, pi: 1.0 / 3.0 }]) });
        let mid = sample_track(&trk, 50).unwrap()[0];
        // Symmetric ease: value at midpoint = 5 exactly for symmetric handles.
        assert!((mid - 5.0).abs() < 1e-9, "symmetric bezier midpoint = 5, got {}", mid);
        let q1 = sample_track(&trk, 25).unwrap()[0];
        assert!(q1 < 2.5, "eased start: expected < 2.5 at u=0.25, got {}", q1);
    }

    #[test]
    fn topo_detects_cycle_and_orders() {
        let mut doc = build_doc();
        // share driven body node ← driver skel: must come after nothing; bind reads skel writes body.
        // Make a cycle: track writing skel, with skel also... simplest cycle: share A→B and share B→A.
        doc.lines.push(Line::new("a2"));
        doc.relations.push(Relation::new_share("s1", ("body", 0), ("a2", 0)));
        doc.relations.push(Relation::new_share("s2", ("a2", 0), ("body", 0)));
        assert!(topo_order(&doc).is_err(), "cycle must be detected");
    }

    #[test]
    fn deterministic_evaluation() {
        let doc = build_doc();
        let a = evaluate(&doc, 0).unwrap();
        let b = evaluate(&doc, 0).unwrap();
        assert_eq!(format!("{:?}", a), format!("{:?}", b));
    }
}
