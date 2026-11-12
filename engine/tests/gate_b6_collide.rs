//! Gate B6-collide — the `collisions` clause of roadmap B6:
//! 1. sim nodes keep clearance from collider segments (floor push-out)
//! 2. chain self-collision: non-adjacent nodes keep separation
//! 3. particles bounce off collider segments with restitution (and never
//!    tunnel below the surface), deterministically
//! 4. round-trip byte stability with the collision params

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::math::Vec3;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, Camera, RenderOpts};

fn opts() -> RenderOpts {
    RenderOpts { width: 200, height: 200, camera: Camera::Ortho2d, samples: 1 }
}

/// A chain hanging from an anchor that drops onto a horizontal floor over
/// the second (the drag pulls the chain down into the collider).
fn fall_doc(self_collide: bool) -> Document {
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"doc.setRoot","width":200,"height":200}},
      {{"op":"line.create","id":"floor","closed":true,"width":2,"strokeOn":true,"nodes":[
        {{"p":[10,40,0]}},{{"p":[190,40,0]}},{{"p":[190,30,0]}},{{"p":[10,30,0]}}]}},
      {{"op":"line.create","id":"chain","width":3,"nodes":[
        {{"p":[100,190,0]}},{{"p":[100,180,0]}},{{"p":[100,170,0]}},{{"p":[100,160,0]}},{{"p":[100,150,0]}}]}},
      {{"op":"line.create","id":"anchor","width":2,"nodes":[{{"p":[100,190,0]}},{{"p":[100,190,0]}}]}},
      {{"op":"line.create","id":"trk_a","tags":["track"],"nodes":[{{"p":[0],"v":[100,190,0]}},{{"p":[{}],"v":[100,100,0]}}]}},
      {{"op":"relation.create","type":"track","trackLine":"trk_a","target":{{"line":"anchor","node":0,"attr":"position"}}}},
      {{"op":"relation.create","type":"sim","line":"chain",
       "anchors":[{{"node":0,"driver":{{"line":"anchor","node":0}}}}],
       "gravity":[0,-500,0],"damping":0.99,"iterations":4,
       "collideWith":["floor"],"radius":8.0,"selfCollide":{}}}
    ]}}"#,
        TICKS_PER_SECOND,
        self_collide
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    doc
}

#[test]
fn sim_nodes_keep_clearance_from_colliders() {
    let doc = fall_doc(false);
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let chain = lines.iter().find(|l| l.id == "chain").unwrap();
    // Every free node must sit at least `radius` above the floor's top edge
    // (y=40): node y >= 40 + 8 = 48 (minus PBD slack).
    for (i, n) in chain.nodes.iter().enumerate() {
        assert!(
            n.p.y >= 47.0,
            "node {} must stay above the floor clearance (y = {:.2})",
            i,
            n.p.y
        );
    }
    // The chain fell well below its rest span (rest tail y = 150) and pooled
    // on the floor instead of sinking through.
    let tail = chain.nodes[n_chain_len(&chain) - 1].p.y;
    assert!(tail < 120.0, "the tail must have fallen (y = {:.2})", tail);
    assert!(tail >= 47.0, "…while keeping the collider clearance (y = {:.2})", tail);
}

fn n_chain_len(chain: &mo2d::eval::EvalLine) -> usize {
    chain.nodes.len()
}

#[test]
fn self_collision_keeps_separation() {
    let doc = fall_doc(true);
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let chain = lines.iter().find(|l| l.id == "chain").unwrap();
    // Non-adjacent pairs must keep ~radius apart (PBD slack tolerated).
    let n = chain.nodes.len();
    for i in 0..n {
        for j in (i + 2)..n {
            let d = chain.nodes[j].p.sub(chain.nodes[i].p).len();
            assert!(
                d >= 7.0,
                "non-adjacent nodes {}–{} must stay separated (d = {:.2})",
                i,
                j,
                d
            );
        }
    }
}

#[test]
fn particles_bounce_off_colliders_deterministically() {
    // Particles rain down onto the floor: with bounce 0.9 they end above it;
    // with bounce 0 they rest on it; determinism: same seed → identical.
    // Fall time to the floor (y=180→44, v0=30, g=−200) is ≈1.03 s, so sample
    // at 1.2 s (within the 1.5 s lifetime).
    let mk = |bounce: f64| -> Document {
        let stream = format!(
            r#"{{"v":1,"cmds":[
      {{"op":"doc.setRoot","width":200,"height":200}},
      {{"op":"line.create","id":"floor","closed":true,"width":2,"strokeOn":true,"nodes":[
        {{"p":[10,40,0]}},{{"p":[190,40,0]}},{{"p":[190,30,0]}},{{"p":[10,30,0]}}]}},
      {{"op":"line.create","id":"tip","width":2,"nodes":[{{"p":[100,180,0]}},{{"p":[100,180,0]}}]}},
      {{"op":"relation.create","type":"particles","emitter":"tip","node":1,"rate":20,"speed":30,
       "dir":[0,-1,0],"spread":10,"life":1.5,"gravity":[0,-200,0],"damping":1.0,
       "size":2,"color":[255,120,0,255],"len":4,"seed":7,
       "collideWith":["floor"],"bounce":{},"radius":4.0}}
    ]}}"#,
            bounce
        );
        let mut doc = Document::default();
        apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
        doc
    };
    let t = (1.2 * TICKS_PER_SECOND as f64).round() as i64;
    let bouncing = eval::evaluate(&mk(0.9), t).unwrap();
    let sticking = eval::evaluate(&mk(0.0), t).unwrap();
    let heads = |ls: &Vec<mo2d::eval::EvalLine>| -> Vec<Vec3> {
        let mut v: Vec<Vec3> = ls
            .iter()
            .filter(|l| l.tags.contains(&"particle".to_string()))
            .map(|l| l.nodes[0].p)
            .collect();
        v.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap().then(a.y.partial_cmp(&b.y).unwrap()));
        v
    };
    assert!(!heads(&bouncing).is_empty(), "particles alive at 1.2 s (life 1.5 s)");
    for p in heads(&bouncing) {
        assert!(p.y >= 43.0, "bouncing particles must rest above the floor (y = {:.2})", p.y);
    }
    for p in heads(&sticking) {
        assert!(p.y >= 43.0, "sticking particles also stay above (y = {:.2})", p.y);
    }
    // Different restitution → different final states (the reflection matters).
    assert_ne!(heads(&bouncing), heads(&sticking), "bounce must change the outcome");
    // Determinism: same seed twice → identical.
    let again = eval::evaluate(&mk(0.9), t).unwrap();
    assert_eq!(format!("{:?}", heads(&bouncing)), format!("{:?}", heads(&again)));
    // Renders (particles visible above the floor).
    let img = render(&bouncing, 200.0, 200.0, &opts());
    assert!(img.px.iter().any(|&b| b != 255));
}

#[test]
fn collision_params_round_trip_and_validation() {
    let mut doc = fall_doc(true);
    let add = r#"{"v":1,"cmds":[
      {"op":"relation.create","type":"particles","emitter":"anchor","node":0,"seed":3,
       "collideWith":["floor"],"bounce":0.7,"radius":5.0}
    ]}"#;
    apply_stream(&mut doc, &parse_stream(add).unwrap()).unwrap();
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "collision params round-trip byte-stably");
    let text = std::str::from_utf8(&b2).unwrap();
    assert!(text.contains("collideWith") && text.contains("selfCollide") && text.contains("bounce"));

    // A sim line listing itself as collider is rejected at command time…
    let mut doc3 = fall_doc(false);
    let bad = parse_stream(r#"{"v":1,"cmds":[
      {"op":"relation.create","type":"sim","line":"chain","collideWith":["chain"],"radius":5.0}
    ]}"#).unwrap();
    assert!(apply_stream(&mut doc3, &bad).is_err(), "self-listing rejected at command level");
    // …and flagged by the validator too (defense in depth for documents
    // assembled without the command protocol).
    let mut doc4 = fall_doc(false);
    let mut direct = mo2d::model::Relation::new_sim("rself", "chain");
    direct.sim_collide_with = vec!["chain".to_string()];
    direct.sim_radius = 5.0;
    doc4.relations.push(direct);
    let problems = mo2d::validate::validate(&doc4);
    assert!(
        problems.iter().any(|p| p.message.contains("cannot list itself")),
        "self-listing must be flagged: {:?}",
        problems
    );
}
