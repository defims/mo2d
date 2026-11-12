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
#[derive(Debug, Clone, PartialEq)]
pub struct EvalNode {
    pub p: Vec3,
    pub width: f64,
    pub color: [u8; 4],
    /// Per-node fill color (B3): defaults to the line fill color; a gradient
    /// relation bound to the fill overwrites it per node.
    pub fill: [u8; 4],
    /// Texture anchor (B5).
    pub uv: Option<(f64, f64)>,
    pub handle_in: Option<Vec3>,
    pub handle_out: Option<Vec3>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalLine {
    pub id: String,
    pub tags: Vec<String>,
    pub closed: bool,
    pub stroke_on: bool,
    pub fill_on: bool,
    pub fill_color: [u8; 4],
    /// External texture path (B5); None = plain fill.
    pub texture: Option<String>,
    pub width: f64,
    /// False when a group/layer relation hides the line (B4).
    pub visible: bool,
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
                    .all(|(j, other)| placed[j] || j == i || !other.writes_all().contains(&read))
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
            texture: l.texture.clone(),
            width: l.width,
            visible: true,
            nodes: l
                .nodes
                .iter()
                .enumerate()
                .map(|(idx, n)| EvalNode {
                    p: match n.pos {
                        NodePos::Geom(v) => v,
                        NodePos::Tick(t) => Vec3::new(t as f64, 0.0, 0.0), // placeholder, never rendered
                    },
                    // B2: width = node override | line width × pressure, brush-scaled;
                    // color = line/node level, then brush tint.
                    width: l.stroke_width(idx),
                    color: l.stroke_color(idx),
                    fill: l.fill_color,
                    uv: n.uv.map(|uv| (uv[0], uv[1])),
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
                    TrackAttr::Color => {
                        if let Some(node_idx) = r.track_target_node {
                            if let Some(ti) = find_line(&lines, &target_line) {
                                if node_idx < lines[ti].nodes.len() {
                                    let prev_alpha = lines[ti].nodes[node_idx].color[3];
                                    if let Some(c) = color_from_track_values(&values, prev_alpha) {
                                        lines[ti].nodes[node_idx].color = c;
                                    }
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
            RelationKind::Gradient => {
                apply_gradient(&mut lines, &r.gradient_bar, &r.gradient_target, r.gradient_role);
            }
            RelationKind::Group => {
                apply_group(doc, &mut lines, &r.group_box, &r.group_members, r.group_visible)?;
            }
            RelationKind::Sim => {
                apply_sim(doc, &mut lines, r, tick)?;
            }
            RelationKind::Particles => {
                let generated = emit_particles(doc, &lines, r, tick)?;
                lines.extend(generated);
            }
        }
    }
    Ok(lines)
}

// ---------------------------------------------------------------------------
// B6 simulation — deterministic dynamic relations
// ---------------------------------------------------------------------------

/// Simulation substep rate: 240 steps/second. Divides TICKS_PER_SECOND
/// exactly (254,016,000,000 / 240 = 1,058,400,000), so tick → substep is
/// exact integer math and every platform integrates the same discrete steps.
pub const SIM_HZ: i64 = 240;
pub const SIM_SUBSTEP_TICKS: i64 = TICKS_PER_SECOND / SIM_HZ;
pub const SIM_DT: f64 = 1.0 / SIM_HZ as f64;

/// B6 sim: Verlet-integrate the sim line's free nodes from tick 0 to `tick`
/// and write the result into the working copy.
///
/// Determinism strategy ("replay-from-seed"): every evaluation re-integrates
/// from the rest state at tick 0 — no hidden state. Scrub-back, replay and
/// offline export therefore all match by construction (relations.md §4). The
/// cost is O(tick/240 × nodes) per evaluation — bounded and predictable.
///
/// - Free nodes: velocity-Verlet with damping + gravity (dt = 1/240 s).
/// - Anchored nodes ride their drivers: if the driving relation is a
///   position track, the anchor position is sampled **exactly per substep**
///   (tracks are pure functions of tick); otherwise the working-copy
///   position is held constant across the integration.
/// - Chain constraints: each consecutive node pair keeps its rest length
///   (from the document), solved with a fixed number of PBD iterations —
///   this is the trailing tail / rope behavior.
fn apply_sim(doc: &Document, lines: &mut [EvalLine], r: &Relation, tick: i64) -> Result<(), String> {
    let sim_line = doc.line(&r.sim_line)?;
    let n = sim_line.nodes.len();
    if n < 2 {
        return Err(format!("sim: line '{}' needs ≥ 2 nodes", r.sim_line));
    }
    let rest: Vec<Vec3> = sim_line
        .nodes
        .iter()
        .map(|nd| nd.pos.geom().map_err(|_| format!("sim: line '{}' must be a geometric line", r.sim_line)))
        .collect::<Result<_, _>>()?;
    let rest_len: Vec<f64> = (0..n - 1).map(|i| rest[i + 1].sub(rest[i]).len()).collect();

    let mut anchored: std::collections::BTreeMap<usize, (&str, usize)> = std::collections::BTreeMap::new();
    for (node, dl, dn) in &r.sim_anchors {
        if *node >= n {
            return Err(format!("sim: anchor node {} out of range ({} nodes)", node, n));
        }
        anchored.insert(*node, (dl.as_str(), *dn));
    }
    // B6 collisions: collider segments from the working copy (held at the
    // frame's positions — the same rule as untracked anchors; documented).
    let collider_segs = resolve_colliders(lines, &r.sim_collide_with);
    let collide = r.sim_radius > 0.0;

    let mut p: Vec<Vec3> = rest.clone();
    let mut prev: Vec<Vec3> = rest.clone();
    let steps = if tick <= 0 { 0usize } else { (tick / SIM_SUBSTEP_TICKS) as usize };
    let damp = r.sim_damping;
    let dt2 = SIM_DT * SIM_DT;

    for k in 1..=steps {
        let tick_k = (k as i64) * SIM_SUBSTEP_TICKS;
        for i in 0..n {
            if anchored.contains_key(&i) {
                continue;
            }
            let v = p[i].sub(prev[i]).scale(damp);
            prev[i] = p[i];
            p[i] = p[i].add(v).add(r.sim_gravity.scale(dt2));
        }
        for (&i, (dl, dn)) in anchored.iter() {
            let a = anchor_pos_at(doc, lines, dl, *dn, tick_k);
            p[i] = a;
            prev[i] = a;
        }
        for _ in 0..r.sim_iterations {
            for i in 0..n - 1 {
                let d = p[i + 1].sub(p[i]);
                let len = d.len();
                if len <= 1e-12 {
                    continue;
                }
                let diff = (len - rest_len[i]) / len;
                let a_anch = anchored.contains_key(&i);
                let b_anch = anchored.contains_key(&(i + 1));
                if a_anch && !b_anch {
                    p[i + 1] = p[i + 1].sub(d.scale(diff));
                } else if b_anch && !a_anch {
                    p[i] = p[i].add(d.scale(diff));
                } else if !a_anch && !b_anch {
                    let half = d.scale(diff * 0.5);
                    p[i] = p[i].add(half);
                    p[i + 1] = p[i + 1].sub(half);
                }
            }
        }
        // B6 collisions: push free nodes out of collider segments, then
        // (optionally) separate non-adjacent node pairs of the chain itself.
        // Applied AFTER the constraint passes so clearance holds at rest.
        if collide {
            for i in 0..n {
                if anchored.contains_key(&i) {
                    continue;
                }
                p[i] = push_out_of_segments(p[i], &collider_segs, r.sim_radius);
            }
            if r.sim_self_collide {
                for i in 0..n {
                    for j in (i + 2)..n {
                        let a_anch = anchored.contains_key(&i);
                        let b_anch = anchored.contains_key(&j);
                        if a_anch && b_anch {
                            continue;
                        }
                        let d = p[j].sub(p[i]);
                        let len = d.len();
                        if len >= r.sim_radius {
                            continue;
                        }
                        if len <= 1e-12 {
                            // Exactly coincident: separate deterministically
                            // along world-up (lower index sinks, higher rises).
                            let half = Vec3::new(0.0, r.sim_radius * 0.5, 0.0);
                            if a_anch {
                                p[j] = p[j].add(half.scale(2.0));
                            } else if b_anch {
                                p[i] = p[i].sub(half.scale(2.0));
                            } else {
                                p[i] = p[i].sub(half);
                                p[j] = p[j].add(half);
                            }
                            continue;
                        }
                        let push = d.scale((r.sim_radius - len) / len);
                        if a_anch {
                            p[j] = p[j].add(push);
                        } else if b_anch {
                            p[i] = p[i].sub(push);
                        } else {
                            let half = push.scale(0.5);
                            p[i] = p[i].sub(half);
                            p[j] = p[j].add(half);
                        }
                    }
                }
            }
        }
    }

    let si = lines.iter().position(|l| &l.id == &r.sim_line).ok_or_else(|| format!("sim: line '{}' missing", r.sim_line))?;
    for (i, pos) in p.iter().enumerate() {
        lines[si].nodes[i].p = *pos;
    }
    Ok(())
}

/// B6 collisions: collider segments from the working copy, in the order the
/// relation lists them (deterministic). Colliders are held at the frame's
/// positions during integration — the same rule as untracked anchors.
fn resolve_colliders(lines: &[EvalLine], ids: &[String]) -> Vec<(Vec3, Vec3)> {
    let mut segs = Vec::new();
    for id in ids {
        let Some(li) = lines.iter().position(|l| &l.id == id) else { continue };
        let l = &lines[li];
        for k in 0..l.nodes.len().saturating_sub(1) {
            segs.push((l.nodes[k].p, l.nodes[k + 1].p));
        }
        if l.closed && l.nodes.len() >= 2 {
            segs.push((l.nodes[l.nodes.len() - 1].p, l.nodes[0].p));
        }
    }
    segs
}

/// Project a point out of every collider segment's clearance radius (single
/// deterministic pass in segment order; closest point from the shared
/// projection helper; degenerate direction falls back to world-up).
fn push_out_of_segments(p: Vec3, segs: &[(Vec3, Vec3)], radius: f64) -> Vec3 {
    let mut out = p;
    for (a, b) in segs {
        let (_, d, closest) = point_segment(out, *a, *b);
        if d < radius {
            let n = if d > 1e-9 { out.sub(closest).scale(1.0 / d) } else { Vec3::new(0.0, 1.0, 0.0) };
            out = closest.add(n.scale(radius));
        }
    }
    out
}

/// Anchor (emitter) position at an arbitrary tick: sampled exactly from the
/// driving position track when one exists (tracks are pure functions of
/// tick), else held at the current working-copy position.
fn anchor_pos_at(doc: &Document, lines: &[EvalLine], line: &str, node: usize, tick: i64) -> Vec3 {
    if let Some(rel) = doc.relations.iter().find(|r| {
        r.kind == RelationKind::Track
            && r.track_target_line == line
            && r.track_target_node == Some(node)
            && r.track_attr == TrackAttr::Position
    }) {
        if let Ok(tl) = doc.line(&rel.track_line) {
            if let Ok(vals) = sample_track(tl, tick) {
                if vals.len() == 3 {
                    return Vec3::new(vals[0], vals[1], vals[2]);
                }
            }
        }
    }
    if let Some(li) = lines.iter().position(|l| l.id == line) {
        if node < lines[li].nodes.len() {
            return lines[li].nodes[node].p;
        }
    }
    Vec3::ZERO
}

/// splitmix64 — the explicit, documented deterministic RNG for seeds
/// (relations.md §2: the random seed must be made an explicit parameter).
fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// B6 particles: generate the particle EvalLines alive at `tick`.
///
/// Particle k spawns at tick k·interval (interval = TPS/rate, ≥ 1, rounded
/// deterministically) at the emitter node's position (track-sampled per
/// spawn tick when driven), with initial velocity = seed-scattered direction
/// (splitmix64 on the explicit `seed` ⊕ k, uniform in ±spread around `dir`)
/// × speed, then ballistic integration (gravity + damping) up to `tick`.
/// Each particle is a short directed segment (head at the position, tail one
/// `len` back along the velocity) — line-model mapping 9. Same document +
/// same seed → byte-identical particles.
fn emit_particles(doc: &Document, lines: &[EvalLine], r: &Relation, tick: i64) -> Result<Vec<EvalLine>, String> {
    let emitter = doc.line(&r.part_emitter)?;
    if emitter.is_track() {
        return Err(format!("particles: emitter '{}' must be a geometric line", r.part_emitter));
    }
    if r.part_node >= emitter.nodes.len() {
        return Err(format!("particles: emitter node {} out of range ({} nodes)", r.part_node, emitter.nodes.len()));
    }
    if tick <= 0 || r.part_rate <= 0.0 || r.part_life_s <= 0.0 {
        return Ok(vec![]);
    }
    let interval = ((TICKS_PER_SECOND as f64 / r.part_rate).round() as i64).max(1);
    let life_ticks = (r.part_life_s * TICKS_PER_SECOND as f64).round() as i64;
    if life_ticks <= 0 {
        return Ok(vec![]);
    }
    let dir_len = r.part_dir.len();
    let base_dir = if dir_len > 1e-12 { r.part_dir.normalize() } else { Vec3::new(0.0, 1.0, 0.0) };
    let spread_rad = r.part_spread_deg.to_radians();
    let collider_segs = resolve_colliders(lines, &r.part_collide_with);
    let collide = r.part_radius > 0.0 && !collider_segs.is_empty();

    let mut out = Vec::new();
    let total_spawns = tick / interval;
    for k in 0..=total_spawns {
        let t_spawn = k * interval;
        if t_spawn > tick {
            break;
        }
        if tick >= t_spawn + life_ticks {
            continue; // already dead
        }
        let spawn = anchor_pos_at(doc, lines, &r.part_emitter, r.part_node, t_spawn);
        // Deterministic scatter: uniform angle in ±spread around the base direction.
        let rng = splitmix64((r.part_seed as u64) ^ (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let frac = ((rng >> 11) as f64) / ((1u64 << 53) as f64); // [0, 1)
        let theta = (frac * 2.0 - 1.0) * spread_rad;
        let (s, c) = theta.sin_cos();
        let v0 = Vec3::new(base_dir.x * c - base_dir.y * s, base_dir.x * s + base_dir.y * c, 0.0).scale(r.part_speed);
        // Ballistic integration from spawn to tick (fixed substeps).
        let steps = ((tick - t_spawn) / SIM_SUBSTEP_TICKS) as usize;
        let mut pp = spawn;
        let mut pv = v0;
        for _ in 0..steps {
            pv = pv.add(r.part_gravity.scale(SIM_DT));
            pp = pp.add(pv.scale(SIM_DT));
            pv = pv.scale(r.part_damping);
            // B6 collisions: project out of colliders and reflect the inward
            // velocity component with restitution `bounce` (only when moving
            // inward — vn < 0 — so resting contact never gains energy).
            if collide {
                for (a, b) in &collider_segs {
                    let (_, d, closest) = point_segment(pp, *a, *b);
                    if d < r.part_radius {
                        let n = if d > 1e-9 { pp.sub(closest).scale(1.0 / d) } else { Vec3::new(0.0, 1.0, 0.0) };
                        pp = closest.add(n.scale(r.part_radius));
                        let vn = pv.dot(n);
                        if vn < 0.0 {
                            pv = pv.sub(n.scale((1.0 + r.part_bounce) * vn));
                        }
                    }
                }
            }
        }
        let vdir = if pv.len() > 1e-9 { pv.normalize() } else { base_dir };
        // A particle = a short directed segment: head at pp, tail back along v.
        let tail = pp.sub(vdir.scale(r.part_len));
        out.push(EvalLine {
            id: format!("{}#p{}", r.part_emitter, k),
            tags: vec!["particle".to_string()],
            closed: false,
            stroke_on: true,
            fill_on: false,
            fill_color: r.part_color,
            texture: None,
            width: r.part_size,
            visible: true,
            nodes: vec![
                EvalNode { p: pp, width: r.part_size, color: r.part_color, fill: r.part_color, uv: None, handle_in: None, handle_out: None },
                EvalNode { p: tail, width: r.part_size, color: r.part_color, fill: r.part_color, uv: None, handle_in: None, handle_out: None },
            ],
            segs: vec![SegAttrs::default()],
        });
    }
    Ok(out)
}

/// B4 group: carry member lines through the transform box's rest→current
/// similarity. The box's first node is the anchor, its last node the
/// diagonal reference; the 2D similarity (rotation + uniform scale, complex
/// quotient of the two diagonals) plus the anchor translation maps every
/// member node; z scales about the anchor's z; member stroke widths scale.
/// Members' rest state comes from the document (like bind), the box's
/// current state from the working copy (so tracks driving the box compose).
fn apply_group(
    doc: &Document,
    lines: &mut [EvalLine],
    box_id: &str,
    members: &[String],
    visible: bool,
) -> Result<(), String> {
    let box_line = doc.line(box_id)?;
    let rest_box: Vec<Vec3> = box_line
        .nodes
        .iter()
        .map(|n| n.pos.geom().map_err(|_| format!("group: box '{}' must be a geometric line", box_id)))
        .collect::<Result<_, _>>()?;
    if rest_box.len() < 2 {
        return Err(format!("group: box '{}' needs ≥ 2 nodes", box_id));
    }
    let bi = lines.iter().position(|l| l.id == box_id).ok_or_else(|| format!("group: box '{}' missing", box_id))?;
    let cur_box: Vec<Vec3> = lines[bi].nodes.iter().map(|n| n.p).collect();

    let (r0, r1) = (rest_box[0], rest_box[rest_box.len() - 1]);
    let (c0, c1) = (cur_box[0], cur_box[box_len_min(cur_box.len())]);
    // Similarity coefficients (2D complex quotient d_cur / d_rest):
    //   a = s·cosθ = (dc·d)/|d|²,  b = s·sinθ = (dc.y·d.x − dc.x·d.y)/|d|²
    let d = r1.sub(r0);
    let dd = d.x * d.x + d.y * d.y;
    let (a, b, s) = if dd > 1e-12 {
        let dc = c1.sub(c0);
        let a = (dc.x * d.x + dc.y * d.y) / dd;
        let b = (dc.y * d.x - dc.x * d.y) / dd;
        (a, b, (a * a + b * b).sqrt())
    } else {
        (1.0, 0.0, 1.0)
    };
    let z0 = r0.z;
    for m in members {
        let mline = doc.line(m)?;
        let mi = lines.iter().position(|l| &l.id == m).ok_or_else(|| format!("group: member '{}' missing", m))?;
        for (i, n) in mline.nodes.iter().enumerate() {
            let p = n
                .pos
                .geom()
                .map_err(|_| format!("group: member '{}' has track nodes — cannot transform", m))?;
            let q = p.sub(r0);
            let qx = a * q.x - b * q.y;
            let qy = b * q.x + a * q.y;
            let np = Vec3::new(c0.x + qx, c0.y + qy, z0 + (p.z - z0) * s);
            if i < lines[mi].nodes.len() {
                lines[mi].nodes[i].p = np;
                lines[mi].nodes[i].width *= s;
            }
        }
        if !visible {
            lines[mi].visible = false;
        }
    }
    Ok(())
}

fn box_len_min(n: usize) -> usize {
    if n == 0 {
        0
    } else {
        n - 1
    }
}

/// B3 gradient: recolor the target's nodes by sampling the bar's stop ramp.
///
/// The bar is a control-bar line; each of its nodes is a color stop:
/// stop position = the node's projection parameter along the bar chord
/// (p_first → p_last), stop color = the node color override (else the bar
/// line color). Every target node is projected onto the same chord and its
/// fill (or stroke color, per role) becomes the interpolated stop color.
/// Deterministic: f64 projection + the shared color lerp, endpoints clamped.
fn apply_gradient(lines: &mut [EvalLine], bar_id: &str, target_id: &str, role: GradientRole) {
    let Some(bi) = lines.iter().position(|l| l.id == bar_id) else { return };
    let Some(ti) = lines.iter().position(|l| l.id == target_id) else { return };
    let bar = &lines[bi];
    if bar.nodes.len() < 2 {
        return;
    }
    let p0 = bar.nodes[0].p;
    let pn = bar.nodes[bar.nodes.len() - 1].p;
    let dir = pn.sub(p0);
    let len2 = dir.dot(dir);
    if len2 <= 1e-12 {
        return; // zero-length bar: no gradient
    }
    let param = |q: Vec3| -> f64 { q.sub(p0).dot(dir) / len2 };
    // Stops: (chord parameter, color) in node order.
    let mut stops: Vec<(f64, [u8; 4])> = bar
        .nodes
        .iter()
        .map(|n| (param(n.p), n.color))
        .collect();
    // Guard against a folded bar (non-monotone params): sort by parameter,
    // stable (node order preserved for equal params) → deterministic.
    stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for node in &mut lines[ti].nodes {
        let u = param(node.p).clamp(stops[0].0, stops[stops.len() - 1].0);
        let c = sample_stops(&stops, u);
        match role {
            GradientRole::Fill => node.fill = c,
            GradientRole::Stroke => node.color = c,
        }
    }
}

/// Interpolate the stop ramp at parameter `u` (stops sorted by parameter).
fn sample_stops(stops: &[(f64, [u8; 4])], u: f64) -> [u8; 4] {
    if u <= stops[0].0 {
        return stops[0].1;
    }
    if u >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }
    for w in stops.windows(2) {
        let (u0, c0) = w[0];
        let (u1, c1) = w[1];
        if u >= u0 && u <= u1 {
            let t = if u1 > u0 { (u - u0) / (u1 - u0) } else { 0.0 };
            return lerp_color(c0, c1, t);
        }
    }
    stops[stops.len() - 1].1
}

/// Track values → color: [r,g,b] keeps the previous alpha, [r,g,b,a] is full.
/// Components are f64, rounded once on write (deterministic).
fn color_from_track_values(values: &[f64], prev_alpha: u8) -> Option<[u8; 4]> {
    let round = |x: f64| -> u8 { x.round().clamp(0.0, 255.0) as u8 };
    match values.len() {
        3 => Some([round(values[0]), round(values[1]), round(values[2]), prev_alpha]),
        4 => Some([round(values[0]), round(values[1]), round(values[2]), round(values[3])]),
        _ => None,
    }
}

/// Shared deterministic color lerp (eval-side gradient sampling and the
/// renderer's attribute interpolation must agree exactly).
pub(crate) fn lerp_color(a: [u8; 4], b: [u8; 4], t: f64) -> [u8; 4] {
    let mut c = [0u8; 4];
    for i in 0..4 {
        c[i] = (a[i] as f64 + (b[i] as f64 - a[i] as f64) * t).round().clamp(0.0, 255.0) as u8;
    }
    c
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
