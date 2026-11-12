//! Validator: structural schema checks + determinism gate
//! (mvp.md §2.4 — a document only passes to disk after both pass).

use crate::eval;
use crate::format;
use crate::json::Value;
use crate::model::*;

/// One validation problem with a JSON-path-ish location.
#[derive(Debug, Clone)]
pub struct Problem {
    pub path: String,
    pub message: String,
}

impl Problem {
    fn new(path: &str, message: impl Into<String>) -> Problem {
        Problem { path: path.to_string(), message: message.into() }
    }
}

pub fn format_problems(problems: &[Problem]) -> String {
    problems.iter().map(|p| format!("{}: {}", p.path, p.message)).collect::<Vec<_>>().join("\n")
}

/// Full document validation. Empty result = valid.
pub fn validate(doc: &Document) -> Vec<Problem> {
    let mut p = Vec::new();
    validate_root(doc, &mut p);
    validate_lines(doc, &mut p);
    validate_relations(doc, &mut p);
    validate_writers(doc, &mut p);
    if let Err(e) = eval::topo_order(doc) {
        p.push(Problem::new("relations", e));
    }
    p
}

fn validate_root(doc: &Document, p: &mut Vec<Problem>) {
    if doc.root.width == 0 || doc.root.width > 16384 {
        p.push(Problem::new("root.width", format!("must be in 1..=16384, got {}", doc.root.width)));
    }
    if doc.root.height == 0 || doc.root.height > 16384 {
        p.push(Problem::new("root.height", format!("must be in 1..=16384, got {}", doc.root.height)));
    }
    if doc.root.ticks_per_second != TICKS_PER_SECOND {
        p.push(Problem::new(
            "root.ticksPerSecond",
            format!("must be {} (the Tick time model constant), got {}", TICKS_PER_SECOND, doc.root.ticks_per_second),
        ));
    }
    if !(0.0..=1000.0).contains(&doc.root.fps) || doc.root.fps <= 0.0 {
        p.push(Problem::new("root.fps", format!("must be in 0..1000 and > 0, got {}", doc.root.fps)));
    }
}

fn validate_lines(doc: &Document, p: &mut Vec<Problem>) {
    let mut seen = std::collections::BTreeSet::new();
    for (li, l) in doc.lines.iter().enumerate() {
        let base = format!("lines[{}] ('{}')", li, l.id);
        if l.id.is_empty() {
            p.push(Problem::new(&base, "empty line id"));
        }
        if !seen.insert(l.id.clone()) {
            p.push(Problem::new(&base, "duplicate line id"));
        }        // Geometric lines need ≥ 2 nodes; a track line is a keyframe series and
        // a single key is a valid constant track.
        let is_track = kinds_contains_track(l);
        let min_nodes = if is_track { 1 } else { 2 };
        if l.nodes.len() < min_nodes {
            p.push(Problem::new(&base, format!("needs ≥ {} nodes, got {}", min_nodes, l.nodes.len())));
        }
        let want_segs = if l.closed { l.nodes.len() } else { l.nodes.len().saturating_sub(1) };
        if l.segs.len() != want_segs {
            p.push(Problem::new(&format!("{}.segs", base), format!("expected {} segment attrs, got {}", want_segs, l.segs.len())));
        }
        // Uniform line kind: all geometric or all track.
        let mut kinds = std::collections::BTreeSet::new();
        for (ni, n) in l.nodes.iter().enumerate() {
            kinds.insert(match n.pos {
                NodePos::Geom(_) => "geom",
                NodePos::Tick(_) => "track",
            });
            if let NodePos::Tick(t) = n.pos {
                if t.abs() > TICK_LIMIT {
                    p.push(Problem::new(&format!("{}.nodes[{}]", base, ni), format!("tick {} beyond ±{} limit", t, TICK_LIMIT)));
                }
            }
            if let Some(w) = n.width {
                if !(0.0..=10000.0).contains(&w) {
                    p.push(Problem::new(&format!("{}.nodes[{}].w", base, ni), format!("width {} out of range 0..10000", w)));
                }
            }
        }
        if kinds.len() > 1 {
            p.push(Problem::new(&base, "mixed node kinds: a line is either geometric (Vec3) or a track (Tick), never both"));
        }
        // Track lines: ticks strictly ascending; values same arity; interp sane.
        if l.is_track() {
            for w in l.nodes.windows(2) {
                let (t0, t1) = (w[0].pos.tick().unwrap(), w[1].pos.tick().unwrap());
                if t1 <= t0 {
                    p.push(Problem::new(&base, format!("track ticks must be strictly ascending ({} then {})", t0, t1)));
                    break;
                }
            }
            let arity = |v: &Value| -> Option<usize> {
                match v {
                    Value::Num(_) | Value::Int(_) => Some(1),
                    Value::Arr(a) => Some(a.len()),
                    _ => None,
                }
            };
            let first = l.nodes.first().and_then(|n| n.value.as_ref()).and_then(arity);
            for (ni, n) in l.nodes.iter().enumerate() {
                if n.value.is_none() {
                    p.push(Problem::new(&format!("{}.nodes[{}]", base, ni), "track node missing keyframe value 'v'"));
                } else if let (Some(want), Some(got)) = (first, n.value.as_ref().and_then(arity)) {
                    if want != got {
                        p.push(Problem::new(&format!("{}.nodes[{}].v", base, ni), format!("keyframe arity mismatch: {} vs {}", got, want)));
                    }
                } else if n.value.as_ref().and_then(arity).is_none() {
                    p.push(Problem::new(&format!("{}.nodes[{}].v", base, ni), "keyframe value must be number or array"));
                }
            }
            for (si, s) in l.segs.iter().enumerate() {
                if let Some(interp) = &s.interp {
                    if interp.im != 0 && interp.im != 9 {
                        p.push(Problem::new(&format!("{}.segs[{}].interp.im", base, si), format!("MVP interp modes are 0=Linear and 9=Bezier, got {}", interp.im)));
                    }
                    if let Some(b) = &interp.b {
                        let comps = first.unwrap_or(1);
                        if b.len() != comps {
                            p.push(Problem::new(&format!("{}.segs[{}].interp.b", base, si), format!("expected {} per-component handles, got {}", comps, b.len())));
                        }
                        for (hi, h) in b.iter().enumerate() {
                            if !(0.0..=1.0).contains(&h.po) || !(0.0..=1.0).contains(&h.pi) {
                                p.push(Problem::new(&format!("{}.segs[{}].interp.b[{}]", base, si, hi), "handle lengths po/pi must be in 0..=1"));
                            }
                        }
                    }
                }
            }
        } else {
            // Geometric lines must not carry keyframe values.
            for (ni, n) in l.nodes.iter().enumerate() {
                if n.value.is_some() {
                    p.push(Problem::new(&format!("{}.nodes[{}].v", base, ni), "geometric nodes cannot carry track values"));
                }
            }
        }
        if l.width < 0.0 {
            p.push(Problem::new(&format!("{}.width", base), "negative width"));
        }
        // B2: brush + pressure sanity.
        for (ni, n) in l.nodes.iter().enumerate() {
            if let Some(pr) = n.pressure {
                if !(0.0..=10.0).contains(&pr) {
                    p.push(Problem::new(&format!("{}.nodes[{}].pr", base, ni), format!("pressure {} out of range 0..10", pr)));
                }
            }
            if let Some(uv) = n.uv {
                if !(0.0..=1.0).contains(&uv[0]) || !(0.0..=1.0).contains(&uv[1]) {
                    p.push(Problem::new(&format!("{}.nodes[{}].uv", base, ni), format!("uv ({}, {}) out of range 0..1", uv[0], uv[1])));
                }
            }
        }
        // B5: a textured line must be closed with fill on.
        if l.texture.is_some() {
            if !l.closed || !l.fill_on {
                p.push(Problem::new(&format!("{}.texture", base), "textured fill requires closed + fillOn"));
            }
        }
        if let Some(b) = &l.brush {
            let bpath = format!("{}.brush", base);
            if let Some(s) = b.spacing {
                if !(0.0..=1.0).contains(&s) {
                    p.push(Problem::new(&bpath, format!("spacing {} out of range 0..1", s)));
                }
            }
            for (key, v) in [("sizeScale", b.size_scale), ("sizeAmp", b.size_amp), ("align", b.align)] {
                if let Some(x) = v {
                    if !(0.0..=100.0).contains(&x) {
                        p.push(Problem::new(&bpath, format!("{} {} out of range 0..100", key, x)));
                    }
                }
            }
        }
    }
    // B2: style ids unique.
    let mut seen_styles = std::collections::BTreeSet::new();
    for (si, s) in doc.styles.iter().enumerate() {
        if !seen_styles.insert(s.id.clone()) {
            p.push(Problem::new(&format!("styles[{}] ('{}')", si, s.id), "duplicate style id"));
        }
    }
}

fn validate_relations(doc: &Document, p: &mut Vec<Problem>) {
    let mut seen = std::collections::BTreeSet::new();
    for (ri, r) in doc.relations.iter().enumerate() {
        let base = format!("relations[{}] ('{}')", ri, r.id);
        if !seen.insert(r.id.clone()) {
            p.push(Problem::new(&base, "duplicate relation id"));
        }
        match r.kind {
            RelationKind::Bind => {
                check_line(doc, &r.bind_target, &format!("{}.target", base), p);
                check_line(doc, &r.bind_skeleton, &format!("{}.skeleton", base), p);
                if let (Ok(t), Ok(s)) = (doc.line(&r.bind_target), doc.line(&r.bind_skeleton)) {
                    if t.is_track() {
                        p.push(Problem::new(&format!("{}.target", base), "cannot bind a track line"));
                    }
                    if s.is_track() {
                        p.push(Problem::new(&format!("{}.skeleton", base), "skeleton must be a geometric line"));
                    }
                    if s.nodes.len() < 2 {
                        p.push(Problem::new(&format!("{}.skeleton", base), "skeleton needs ≥ 2 nodes"));
                    }
                }
                if r.bind_falloff < 0.0 {
                    p.push(Problem::new(&format!("{}.falloff", base), "negative falloff"));
                }
            }
            RelationKind::Share => {
                let (dl, dn) = &r.share_driver;
                let (rl, rn) = &r.share_driven;
                check_line(doc, dl, &format!("{}.driver.line", base), p);
                check_line(doc, rl, &format!("{}.driven.line", base), p);
                if let Ok(l) = doc.line(dl) {
                    if dn >= &l.nodes.len() {
                        p.push(Problem::new(&format!("{}.driver.node", base), format!("index {} out of range ({} nodes)", dn, l.nodes.len())));
                    }
                    if l.is_track() {
                        p.push(Problem::new(&format!("{}.driver", base), "driver must be a geometric line"));
                    }
                }
                if let Ok(l) = doc.line(rl) {
                    if rn >= &l.nodes.len() {
                        p.push(Problem::new(&format!("{}.driven.node", base), format!("index {} out of range ({} nodes)", rn, l.nodes.len())));
                    }
                    if l.is_track() {
                        p.push(Problem::new(&format!("{}.driven", base), "driven must be a geometric line"));
                    }
                }
                if dl == rl && dn == rn {
                    p.push(Problem::new(&base, "driver and driven are the same node"));
                }
            }
            RelationKind::Track => {
                check_line(doc, &r.track_line, &format!("{}.trackLine", base), p);
                check_line(doc, &r.track_target_line, &format!("{}.target.line", base), p);
                if let Ok(t) = doc.line(&r.track_line) {
                    if !t.is_track() {
                        p.push(Problem::new(&format!("{}.trackLine", base), "track line must have Tick nodes"));
                    }
                }
                if let Ok(t) = doc.line(&r.track_target_line) {
                    if t.is_track() {
                        p.push(Problem::new(&format!("{}.target", base), "cannot target a track line"));
                    }
                    if let Some(n) = r.track_target_node {
                        if n >= t.nodes.len() {
                            p.push(Problem::new(&format!("{}.target.node", base), format!("index {} out of range ({} nodes)", n, t.nodes.len())));
                        }
                    }
                }
                // Track value arity must match the target attribute.
                if let (Ok(t), Ok(tl)) = (doc.line(&r.track_line), doc.line(&r.track_target_line)) {
                    let _ = tl;
                    if let Some(first) = t.nodes.first() {
                        let arity = match first.value.as_ref() {
                            Some(Value::Num(_)) | Some(Value::Int(_)) => 1,
                            Some(Value::Arr(a)) => a.len(),
                            _ => 0,
                        };
                        let want: &[usize] = match r.track_attr {
                            TrackAttr::Position => &[3],
                            TrackAttr::Width => &[1],
                            TrackAttr::Color => &[3, 4],
                        };
                        if arity != 0 && !want.contains(&arity) {
                            p.push(Problem::new(
                                &format!("{}.trackLine", base),
                                format!("track value arity {} does not match attr '{}' (needs {})", arity, r.track_attr.name(), want.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" or ")),
                            ));
                        }
                    }
                }
            }
            RelationKind::Gradient => {
                check_line(doc, &r.gradient_bar, &format!("{}.bar", base), p);
                check_line(doc, &r.gradient_target, &format!("{}.target", base), p);
                if let (Ok(b), Ok(t)) = (doc.line(&r.gradient_bar), doc.line(&r.gradient_target)) {
                    if b.is_track() {
                        p.push(Problem::new(&format!("{}.bar", base), "bar must be a geometric line"));
                    }
                    if b.nodes.len() < 2 {
                        p.push(Problem::new(&format!("{}.bar", base), "bar needs ≥ 2 nodes (color stops)"));
                    }
                    if t.is_track() {
                        p.push(Problem::new(&format!("{}.target", base), "cannot target a track line"));
                    }
                    if r.gradient_bar == r.gradient_target {
                        p.push(Problem::new(&base, "bar and target must differ"));
                    }
                }
            }
            RelationKind::Group => {
                check_line(doc, &r.group_box, &format!("{}.box", base), p);
                if r.group_members.is_empty() {
                    p.push(Problem::new(&base, "group needs at least one member"));
                }
                if let Ok(b) = doc.line(&r.group_box) {
                    if b.is_track() {
                        p.push(Problem::new(&format!("{}.box", base), "box must be a geometric line"));
                    }
                    if b.nodes.len() < 2 {
                        p.push(Problem::new(&format!("{}.box", base), "box needs ≥ 2 nodes"));
                    }
                }
                for m in &r.group_members {
                    check_line(doc, m, &format!("{}.members", base), p);
                    if let Ok(l) = doc.line(m) {
                        if l.is_track() {
                            p.push(Problem::new(&format!("{}.members", base), format!("member '{}' must be a geometric line", m)));
                        }
                    }
                    if m == &r.group_box {
                        p.push(Problem::new(&base, "the box cannot be its own member"));
                    }
                }
            }
            RelationKind::Sim => {
                check_line(doc, &r.sim_line, &format!("{}.line", base), p);
                if let Ok(l) = doc.line(&r.sim_line) {
                    if l.is_track() {
                        p.push(Problem::new(&format!("{}.line", base), "the sim line must be a geometric line"));
                    }
                    if l.nodes.len() < 2 {
                        p.push(Problem::new(&format!("{}.line", base), "the sim line needs ≥ 2 nodes"));
                    }
                }
                for (node, dl, dn) in &r.sim_anchors {
                    check_line(doc, dl, &format!("{}.anchors", base), p);
                    if let (Ok(sl), Ok(al)) = (doc.line(&r.sim_line), doc.line(dl)) {
                        if *node >= sl.nodes.len() {
                            p.push(Problem::new(&format!("{}.anchors", base), format!("anchor node {} out of range", node)));
                        }
                        if al.is_track() {
                            p.push(Problem::new(&format!("{}.anchors", base), "anchor driver must be a geometric line"));
                        }
                        if *dn >= al.nodes.len() {
                            p.push(Problem::new(&format!("{}.anchors", base), format!("anchor driver node {} out of range", dn)));
                        }
                    }
                }
                if r.sim_damping < 0.0 || r.sim_damping > 1.0 {
                    p.push(Problem::new(&format!("{}.damping", base), "damping must be in 0..1"));
                }
                for c in &r.sim_collide_with {
                    check_line(doc, c, &format!("{}.collideWith", base), p);
                    if let Ok(cl) = doc.line(c) {
                        if cl.is_track() {
                            p.push(Problem::new(&format!("{}.collideWith", base), "collider must be a geometric line"));
                        }
                    }
                    if c == &r.sim_line {
                        p.push(Problem::new(&format!("{}.collideWith", base), "the sim line cannot list itself (use selfCollide)"));
                    }
                }
                if r.sim_radius < 0.0 {
                    p.push(Problem::new(&format!("{}.radius", base), "radius must be >= 0"));
                }
            }
            RelationKind::Particles => {
                check_line(doc, &r.part_emitter, &format!("{}.emitter", base), p);
                if let Ok(l) = doc.line(&r.part_emitter) {
                    if l.is_track() {
                        p.push(Problem::new(&format!("{}.emitter", base), "emitter must be a geometric line"));
                    }
                    if r.part_node >= l.nodes.len() {
                        p.push(Problem::new(&format!("{}.node", base), format!("emitter node {} out of range", r.part_node)));
                    }
                }
                if r.part_rate <= 0.0 {
                    p.push(Problem::new(&format!("{}.rate", base), "rate must be > 0"));
                }
                if r.part_life_s <= 0.0 {
                    p.push(Problem::new(&format!("{}.life", base), "life must be > 0"));
                }
                if !(0.0..=360.0).contains(&r.part_spread_deg) {
                    p.push(Problem::new(&format!("{}.spread", base), "spread must be in 0..360 degrees"));
                }
                for c in &r.part_collide_with {
                    check_line(doc, c, &format!("{}.collideWith", base), p);
                    if let Ok(cl) = doc.line(c) {
                        if cl.is_track() {
                            p.push(Problem::new(&format!("{}.collideWith", base), "collider must be a geometric line"));
                        }
                    }
                }
                if !(0.0..=1.0).contains(&r.part_bounce) {
                    p.push(Problem::new(&format!("{}.bounce", base), "bounce must be in 0..1"));
                }
                if r.part_radius < 0.0 {
                    p.push(Problem::new(&format!("{}.radius", base), "radius must be >= 0"));
                }
            }
        }
    }
}

fn kinds_contains_track(l: &Line) -> bool {
    l.nodes.iter().any(|n| matches!(n.pos, NodePos::Tick(_)))
}

fn check_line(doc: &Document, id: &str, path: &str, p: &mut Vec<Problem>) {
    if doc.line(id).is_err() {
        p.push(Problem::new(path, format!("line '{}' not found", id)));
    }
}

/// One writer per output: bind claims a whole line's positions; share claims
/// one node's position; track claims one node's attribute (or the line's width).
/// Overlapping claims conflict and are rejected.
fn validate_writers(doc: &Document, p: &mut Vec<Problem>) {
    enum Claim {
        Bind(String),
        ShareNode(String, usize),
        TrackNode(String, Option<usize>, TrackAttr),
        /// Gradient claims one recolor channel of a whole line (B3).
        Gradient(String, GradientRole),
        /// Group claims every member's positions (B4).
        GroupMember(String),
        /// Sim claims the whole simulated line's positions (B6).
        SimLine(String),
    }
    let claims: Vec<(&Relation, Claim)> = doc
        .relations
        .iter()
        .flat_map(|r| {
            let base = match r.kind {
                RelationKind::Bind => vec![Claim::Bind(r.bind_target.clone())],
                RelationKind::Share => vec![Claim::ShareNode(r.share_driven.0.clone(), r.share_driven.1)],
                RelationKind::Track => vec![Claim::TrackNode(r.track_target_line.clone(), r.track_target_node, r.track_attr)],
                RelationKind::Gradient => vec![Claim::Gradient(r.gradient_target.clone(), r.gradient_role)],
                RelationKind::Group => r.group_members.iter().map(|m| Claim::GroupMember(m.clone())).collect(),
                RelationKind::Sim => vec![Claim::SimLine(r.sim_line.clone())],
                // Particles claim no document line (B6).
                RelationKind::Particles => vec![],
            };
            base.into_iter().map(move |c| (r, c))
        })
        .collect();

    for (i, (ra, ca)) in claims.iter().enumerate() {
        for (rb, cb) in claims.iter().skip(i + 1) {
            if ra.id == rb.id {
                continue;
            }
            let overlaps = match (ca, cb) {
                (Claim::Bind(la), Claim::Bind(lb)) => la == lb,
                (Claim::Bind(la), Claim::ShareNode(lb, _)) | (Claim::ShareNode(lb, _), Claim::Bind(la)) => la == lb,
                (Claim::Bind(la), Claim::TrackNode(lb, _, TrackAttr::Position)) | (Claim::TrackNode(lb, _, TrackAttr::Position), Claim::Bind(la)) => la == lb,
                (Claim::Bind(la), Claim::GroupMember(lb)) | (Claim::GroupMember(lb), Claim::Bind(la)) => la == lb,
                (Claim::Bind(la), Claim::SimLine(lb)) | (Claim::SimLine(lb), Claim::Bind(la)) => la == lb,
                (Claim::ShareNode(la, na), Claim::ShareNode(lb, nb)) => la == lb && na == nb,
                (Claim::ShareNode(la, na), Claim::TrackNode(lb, Some(nb), TrackAttr::Position))
                | (Claim::TrackNode(lb, Some(nb), TrackAttr::Position), Claim::ShareNode(la, na)) => la == lb && na == nb,
                (Claim::ShareNode(la, _), Claim::GroupMember(lb)) | (Claim::GroupMember(lb), Claim::ShareNode(la, _)) => la == lb,
                (Claim::ShareNode(la, _), Claim::SimLine(lb)) | (Claim::SimLine(lb), Claim::ShareNode(la, _)) => la == lb,
                (Claim::TrackNode(la, _, TrackAttr::Position), Claim::GroupMember(lb))
                | (Claim::GroupMember(lb), Claim::TrackNode(la, _, TrackAttr::Position)) => la == lb,
                (Claim::TrackNode(la, _, TrackAttr::Position), Claim::SimLine(lb))
                | (Claim::SimLine(lb), Claim::TrackNode(la, _, TrackAttr::Position)) => la == lb,
                (Claim::GroupMember(la), Claim::GroupMember(lb)) => la == lb,
                (Claim::GroupMember(la), Claim::SimLine(lb)) | (Claim::SimLine(lb), Claim::GroupMember(la)) => la == lb,
                (Claim::SimLine(la), Claim::SimLine(lb)) => la == lb,
                (Claim::TrackNode(la, na, attr_a), Claim::TrackNode(lb, nb, attr_b)) => {
                    la == lb && attr_a == attr_b && (na == nb || na.is_none() || nb.is_none())
                }
                // Gradients write one color channel of a line — two gradients on
                // the same line+role conflict (different roles compose: fill+stroke).
                (Claim::Gradient(la, ra_), Claim::Gradient(lb, rb_)) => la == lb && ra_ == rb_,
                _ => false,
            };
            if overlaps {
                p.push(Problem::new(
                    &format!("relations ('{}' / '{}')", ra.id, rb.id),
                    "conflicting writers: two relations animate the same node attribute",
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Determinism gate
// ---------------------------------------------------------------------------

pub struct GateReport {
    pub double_apply_equal: bool,
    pub round_trip_equal: bool,
    pub double_eval_equal: bool,
    pub problems: Vec<String>,
}

/// The determinism gate (mvp.md A4/A5):
/// 1. applying the same stream twice from scratch → byte-identical docs
/// 2. doc → bytes → parse → bytes is stable
/// 3. evaluating the same doc+tick twice → identical debug output
/// 4. (render determinism is checked by the caller via PNG hashes)
pub fn determinism_gate(cmds_text: &str, doc: &Document) -> GateReport {
    let mut report = GateReport {
        double_apply_equal: true,
        round_trip_equal: true,
        double_eval_equal: true,
        problems: Vec::new(),
    };

    // 1. Replay equality.
    match crate::command::parse_stream(cmds_text) {
        Ok(cmds) => {
            let mut a = Document::default();
            let mut b = Document::default();
            match (crate::command::apply_stream(&mut a, &cmds), crate::command::apply_stream(&mut b, &cmds)) {
                (Ok(_), Ok(_)) => {
                    let ba = format::doc_to_bytes(&a);
                    let bb = format::doc_to_bytes(&b);
                    if ba != bb {
                        report.double_apply_equal = false;
                        report.problems.push("applying the same stream twice produced different documents".into());
                    }
                }
                (Err(e1), _) => report.problems.push(format!("stream apply failed: {}", e1)),
                (_, Err(e2)) => report.problems.push(format!("second apply failed: {}", e2)),
            }
        }
        Err(e) => report.problems.push(format!("stream parse failed: {}", e)),
    }

    // 2. Round-trip stability.
    if let Ok(b1) = format::doc_to_bytes(doc) {
        if let Ok(d2) = format::parse_doc(std::str::from_utf8(&b1).unwrap_or("")) {
            if let Ok(b2) = format::doc_to_bytes(&d2) {
                if b1 != b2 {
                    report.round_trip_equal = false;
                    report.problems.push("save→load→save is not byte-stable".into());
                }
            }
        }
    }

    // 3. Evaluation stability at a few ticks.
    let tps = doc.root.ticks_per_second;
    for frac in [0i64, tps / 4, tps / 2, tps] {
        match (eval::evaluate(doc, frac), eval::evaluate(doc, frac)) {
            (Ok(a), Ok(b)) => {
                if format!("{:?}", a) != format!("{:?}", b) {
                    report.double_eval_equal = false;
                    report.problems.push(format!("evaluation at tick {} is not reproducible", frac));
                }
            }
            (Err(e), _) | (_, Err(e)) => report.problems.push(format!("evaluation failed at tick {}: {}", frac, e)),
        }
    }

    report
}

/// Validate + gate in one: the pre-save check the CLI and MCP use.
pub fn verify_before_save(doc: &Document) -> Result<(), String> {
    let problems = validate(doc);
    if !problems.is_empty() {
        return Err(format!("schema validation failed:\n{}", format_problems(&problems)));
    }
    // Round-trip stability is part of the gate.
    let b1 = format::doc_to_bytes(doc)?;
    let d2 = format::parse_doc(std::str::from_utf8(&b1).map_err(|_| "UTF-8")?)?;
    let b2 = format::doc_to_bytes(&d2)?;
    if b1 != b2 {
        return Err("determinism gate failed: save→load→save is not byte-stable".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    #[test]
    fn rejects_bad_docs() {
        let mut doc = Document::default();
        doc.root.ticks_per_second = 1000; // wrong constant
        let mut l = Line::new("a");
        l.nodes = vec![Node::geom(Vec3::new(0.0, 0.0, 0.0)), Node::geom(Vec3::new(1.0, 1.0, 0.0))];
        doc.lines.push(l);
        doc.relations.push(Relation::new_bind("r", "a", "missing")); // dangling ref
        let problems = validate(&doc);
        assert!(problems.iter().any(|p| p.path.starts_with("root.ticksPerSecond")), "must flag wrong TPS");
        assert!(problems.iter().any(|p| p.message.contains("missing")), "must flag dangling relation ref: {:?}", problems);
    }

    #[test]
    fn rejects_descending_ticks_and_double_writers() {
        let mut doc = Document::default();
        let mut t = Line::new("trk");
        t.nodes = vec![Node::track(100, Value::Num(1.0)), Node::track(50, Value::Num(2.0))];
        doc.lines.push(t);
        let mut g = Line::new("g");
        g.nodes = vec![Node::geom(Vec3::ZERO), Node::geom(Vec3::new(1.0, 0.0, 0.0))];
        doc.lines.push(g);
        doc.relations.push(Relation::new_track("t1", "trk", "g", Some(0), TrackAttr::Width));
        doc.relations.push(Relation::new_track("t2", "trk", "g", Some(0), TrackAttr::Width)); // same node → conflict
        let problems = validate(&doc);
        assert!(problems.iter().any(|p| p.message.contains("ascending")), "must flag descending ticks");
        assert!(problems.iter().any(|p| p.message.contains("conflicting writers")), "must flag double writers");
    }
}
