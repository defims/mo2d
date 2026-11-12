//! Gate B6 — simulation as dynamic relations (roadmap B6):
//! 1. a trailing tail: Verlet chain anchored to a track-driven node follows
//!    the driver, keeps rest lengths, lags behind (the physics demo)
//! 2. replay-from-seed determinism: scrub back and re-evaluate — identical
//! 3. particles: seeded, deterministic, born and dying on schedule
//! 4. writer conflicts + round-trip byte stability

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::math::Vec3;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, Camera, RenderOpts};

fn opts() -> RenderOpts {
    RenderOpts { width: 320, height: 240, camera: Camera::Ortho2d, samples: 1 }
}

/// A chain of 5 nodes (rest: horizontal, head at the anchor) + a driver node
/// moving along a position track.
fn tail_doc() -> Document {
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"doc.setRoot","width":320,"height":240}},
      {{"op":"line.create","id":"tail","width":3,"nodes":[
        {{"p":[100,100,0]}},{{"p":[90,100,0]}},{{"p":[80,100,0]}},{{"p":[70,100,0]}},{{"p":[60,100,0]}}]}},
      {{"op":"line.create","id":"mover","width":2,"nodes":[{{"p":[100,100,0]}},{{"p":[100,100,0]}}]}},
      {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":[100,100,0]}},{{"p":[{}],"v":[160,40,0]}}]}},
      {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"mover","node":1,"attr":"position"}}}},
      {{"op":"relation.create","type":"sim","line":"tail","anchors":[{{"node":0,"driver":{{"line":"mover","node":1}}}}],"gravity":[0,-600,0],"damping":0.985}}
    ]}}"#,
        TICKS_PER_SECOND
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    doc
}

#[test]
fn trailing_tail_follows_and_lags() {
    let doc = tail_doc();
    let t_end = TICKS_PER_SECOND;
    let lines = eval::evaluate(&doc, t_end).unwrap();
    let tail = lines.iter().find(|l| l.id == "tail").unwrap();
    // The anchor rides the driver exactly (track-sampled per substep).
    assert_eq!(tail.nodes[0].p, Vec3::new(160.0, 40.0, 0.0), "anchored head = driver at tick");
    // Rest lengths survive the integration (PBD constraint, 4 iterations).
    for i in 0..4 {
        let d = tail.nodes[i + 1].p.sub(tail.nodes[i].p).len();
        assert!(
            (d - 10.0).abs() < 0.5,
            "segment {} length {} must stay ≈ rest length 10",
            i,
            d
        );
    }
    // The far end lags: it is NOT on the segment from rest-head to new-head.
    let moved = tail.nodes[4].p.sub(tail.nodes[0].p).len();
    assert!(moved > 10.0 && moved < 150.0, "tail must trail behind the anchor (far-end distance {:.1})", moved);
    // Gravity droop: the tail end sits below the straight rest line y=100
    // minus anchor motion — assert it left the rest position.
    assert_ne!(tail.nodes[4].p, Vec3::new(60.0, 100.0, 0.0), "far end must have moved");
    // It renders (draws the chain).
    let img = render(&lines, 320.0, 240.0, &opts());
    assert!(img.px.iter().any(|&b| b != 255), "tail must be visible");
}

#[test]
fn replay_from_seed_scrub_consistent() {
    let doc = tail_doc();
    // Evaluate out of order: T, T/2, T again — every evaluation re-integrates
    // from tick 0, so results are call-order independent.
    let t_end = TICKS_PER_SECOND;
    let a1 = eval::evaluate(&doc, t_end).unwrap();
    let _mid = eval::evaluate(&doc, t_end / 2).unwrap();
    let a2 = eval::evaluate(&doc, t_end).unwrap();
    assert_eq!(format!("{:?}", a1), format!("{:?}", a2), "scrub-back must reproduce byte-identical frames");
    // Half-way state is a partial integration (also deterministic).
    let m1 = eval::evaluate(&doc, t_end / 2).unwrap();
    let m2 = eval::evaluate(&doc, t_end / 2).unwrap();
    assert_eq!(format!("{:?}", m1), format!("{:?}", m2));
    let mid = m1.iter().find(|l| l.id == "tail").unwrap();
    assert_eq!(mid.nodes[0].p, Vec3::new(130.0, 70.0, 0.0), "anchor at mid-track (linear)");
}

#[test]
fn sim_writer_conflicts() {
    let mut doc = tail_doc();
    // A track claiming the sim line's positions conflicts with the sim claim.
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"line.create","id":"trk2","tags":["track"],"nodes":[{{"p":[0],"v":[0,0,0]}},{{"p":[{}],"v":[5,5,0]}}]}},
      {{"op":"relation.create","type":"track","trackLine":"trk2","target":{{"line":"tail","node":2,"attr":"position"}}}}
    ]}}"#,
        TICKS_PER_SECOND
    );
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let problems = mo2d::validate::validate(&doc);
    assert!(problems.iter().any(|p| p.message.contains("conflicting writers")), "sim + track on one line must conflict");
    // Two sims on one line conflict.
    let mut doc2 = tail_doc();
    let s2 = parse_stream(r#"{"v":1,"cmds":[{"op":"relation.create","type":"sim","line":"tail"}]}"#).unwrap();
    apply_stream(&mut doc2, &s2).unwrap();
    assert!(mo2d::validate::validate(&doc2).iter().any(|p| p.message.contains("conflicting writers")));
}

fn particles_doc(seed: i64) -> Document {
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"doc.setRoot","width":320,"height":240}},
      {{"op":"line.create","id":"tip","width":2,"nodes":[{{"p":[80,120,0]}},{{"p":[80,120,0]}}]}},
      {{"op":"relation.create","type":"particles","emitter":"tip","node":1,"rate":20,"speed":100,"dir":[1,0,0],"spread":30,"life":0.5,"gravity":[0,-200,0],"damping":0.98,"size":2,"color":[255,120,0,255],"len":6,"seed":{}}}
    ]}}"#,
        seed
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    doc
}

#[test]
fn particles_seed_deterministic() {
    let t = TICKS_PER_SECOND / 2;
    let a = eval::evaluate(&particles_doc(42), t).unwrap();
    let b = eval::evaluate(&particles_doc(42), t).unwrap();
    let c = eval::evaluate(&particles_doc(43), t).unwrap();
    let fa = format!("{:?}", a);
    let fb = format!("{:?}", b);
    let fc = format!("{:?}", c);
    assert_eq!(fa, fb, "same seed → identical particles");
    assert_ne!(fa, fc, "different seed → different scatter");
    let spawned = a.iter().filter(|l| l.tags.contains(&"particle".to_string())).count();
    // rate 20/s × life 0.5s → ≈10 alive at any tick (±1 boundary).
    assert!((9..=11).contains(&spawned), "expected ≈10 alive particles, got {}", spawned);
    // Particles are short directed segments named after their emitter.
    // (At exactly t = life the spawn-0 particle has just died — check #p1.)
    assert!(a.iter().any(|l| l.id == "tip#p1"), "particle ids are <emitter>#p<k>");
    // They draw.
    let img = render(&a, 320.0, 240.0, &opts());
    assert!(img.px.iter().any(|&b| b != 255), "particles must be visible");
}

#[test]
fn particles_life_cycle_and_ballistics() {
    let doc = particles_doc(7);
    // Before the first spawn window closes: t = life → ~10 alive.
    let t1 = (0.5 * TICKS_PER_SECOND as f64).round() as i64;
    let at_life = eval::evaluate(&doc, t1).unwrap();
    let n1 = at_life.iter().filter(|l| l.tags.contains(&"particle".to_string())).count();
    assert!(n1 > 0, "particles alive at life boundary");
    // After 2× life, the FIRST particle is dead; the set shifted.
    let t2 = TICKS_PER_SECOND;
    let at_2 = eval::evaluate(&doc, t2).unwrap();
    assert!(!at_2.iter().any(|l| l.id == "tip#p0"), "particle #0 dies after its lifetime");
    let n2 = at_2.iter().filter(|l| l.tags.contains(&"particle".to_string())).count();
    assert!(n2 > 0);
    // Ballistics: the same particle later in its life has fallen (gravity −200).
    // Particle #5 spawns at t = 0.25 s, lives to 0.75 s — sample 0.30 s vs 0.45 s.
    let young = eval::evaluate(&doc, (0.30 * TICKS_PER_SECOND as f64).round() as i64).unwrap();
    let older = eval::evaluate(&doc, (0.45 * TICKS_PER_SECOND as f64).round() as i64).unwrap();
    let head = |ls: &Vec<mo2d::eval::EvalLine>, k: i64| {
        ls.iter().find(|l| l.id == format!("tip#p{}", k)).map(|l| l.nodes[0].p)
    };
    let p_young = head(&young, 5).expect("particle #5 alive at 0.30 s");
    let p_old = head(&older, 5).expect("particle #5 alive at 0.45 s");
    assert!(p_old.y < p_young.y, "gravity pulls particles down over their lifetime");
    assert!(p_old.x > p_young.x, "particles keep drifting along their velocity");
}

#[test]
fn sim_particles_round_trip_byte_stable() {
    let mut doc = tail_doc();
    let add = r#"{"v":1,"cmds":[
      {"op":"relation.create","type":"particles","emitter":"mover","node":1,"seed":9,"rate":30}
    ]}"#;
    apply_stream(&mut doc, &parse_stream(add).unwrap()).unwrap();
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "sim + particles relations round-trip byte-stably");
    let text = std::str::from_utf8(&b2).unwrap();
    assert!(text.contains("\"sim\"") && text.contains("\"particles\""));
    assert!(text.contains("\"seed\": 9"), "explicit seed carried in the document");
}
