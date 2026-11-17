//! Gate C7 — interactivity vocabulary (roadmap C7, established 2026-11-17):
//! - oscillate: deterministic periodic driver (pure f(tick), period wrap)
//! - step interpolation im=3: discrete channels hold their key value
//! - (P2 adds: respond relations + event stream, tap_jump frozen demo)

mod common;

use common::{doc_bytes, load_frozen, replay_stream, render_sha, repo_root, demo_dir};
use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::model::*;
use mo2d::render::Camera;
use mo2d::sha::sha256_hex;
use std::path::Path;

const TPS: i64 = 254_016_000_000;

fn doc_from(text: &str) -> Document {
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(text).unwrap()).unwrap();
    doc
}

/// A 2-node line + a sine position oscillation on node 1 along x.
fn osc_doc(wave: i64, period: i64) -> Document {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"relation.create","type":"oscillate","target":"g","node":1,"attr":"position",
           "axis":[1,0,0],"wave":{},"period":{},"amplitude":50.0}}
        ]}}"#,
        wave, period
    );
    doc_from(&text)
}

#[test]
fn oscillate_is_a_pure_function_of_tick() {
    let doc = osc_doc(0, TPS);
    let a = eval::evaluate(&doc, TPS / 4).unwrap();
    let b = eval::evaluate(&doc, TPS / 4).unwrap();
    assert_eq!(a, b, "double eval must be identical");
    // sine at quarter period = +amplitude.
    assert!((a[0].nodes[1].p.x - 150.0).abs() < 1e-9, "sine quarter period should peak at +amplitude, got {}", a[0].nodes[1].p.x);
    assert!((a[0].nodes[0].p.x - 0.0).abs() < 1e-9, "node 0 must stay at rest");
}

#[test]
fn oscillate_wraps_exactly_after_one_period() {
    let doc = osc_doc(0, TPS);
    // tick and tick + period must produce identical frames (loop = wrap).
    for t in [0i64, TPS / 8, TPS / 3, TPS / 2] {
        let a = eval::evaluate(&doc, t).unwrap();
        let b = eval::evaluate(&doc, t + TPS).unwrap();
        assert_eq!(a, b, "period wrap must be exact at tick {}", t);
    }
    // Start node of the period = rest position (sine zero crossing).
    let rest = eval::evaluate(&doc, 0).unwrap();
    assert!((rest[0].nodes[1].p.x - 100.0).abs() < 1e-9);
}

#[test]
fn oscillate_triangle_and_saw_shapes() {
    let tri = osc_doc(1, TPS);
    let at_start = eval::evaluate(&tri, 0).unwrap();
    assert!((at_start[0].nodes[1].p.x - 150.0).abs() < 1e-9, "triangle starts at +amplitude");
    let at_mid = eval::evaluate(&tri, TPS / 2).unwrap();
    assert!((at_mid[0].nodes[1].p.x - 50.0).abs() < 1e-9, "triangle mid = -amplitude");
    let saw = osc_doc(2, TPS);
    let saw_quarter = eval::evaluate(&saw, TPS / 4).unwrap();
    assert!((saw_quarter[0].nodes[1].p.x - 75.0).abs() < 1e-9, "saw quarter = -0.5 amplitude");
    let saw_zero = eval::evaluate(&saw, 0).unwrap();
    assert!((saw_zero[0].nodes[1].p.x - 50.0).abs() < 1e-9, "saw starts at -amplitude");
}

#[test]
fn oscillate_width_targets_node_and_line() {
    let text = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"oscillate","target":"g","attr":"width","wave":0,"period":1000000,"amplitude":2.0}
    ]}"#;
    let doc = doc_from(text);
    // Line-level width oscillation writes the line AND every node width.
    let quarter = eval::evaluate(&doc, 250_000).unwrap();
    assert!((quarter[0].width - 8.0).abs() < 1e-9, "line width = 6 + 2 at quarter period, got {}", quarter[0].width);
    for n in &quarter[0].nodes {
        assert!((n.width - 8.0).abs() < 1e-9);
    }
}

#[test]
fn oscillate_rejects_bad_params() {
    // period 0
    let bad = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"oscillate","target":"g","attr":"position","axis":[1,0,0],"period":0}
    ]}"#;
    let mut doc = Document::default();
    assert!(apply_stream(&mut doc, &parse_stream(bad).unwrap()).is_err(), "period 0 must be rejected");
    // zero axis
    let bad2 = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"oscillate","target":"g","attr":"position","axis":[0,0,0]}
    ]}"#;
    let mut doc2 = Document::default();
    assert!(apply_stream(&mut doc2, &parse_stream(bad2).unwrap()).is_err(), "zero axis must be rejected");
    // wave 7
    let bad3 = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"oscillate","target":"g","attr":"width","wave":7}
    ]}"#;
    let mut doc3 = Document::default();
    assert!(apply_stream(&mut doc3, &parse_stream(bad3).unwrap()).is_err(), "unknown wave code must be rejected");
    // validator: oscillate vs track on the same node+attr conflicts.
    let both = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"line.create","id":"trk","tags":["track"],"nodes":[{"p":[0],"v":6.0},{"p":[1000000000000],"v":2.0}]},
      {"op":"relation.create","type":"track","trackLine":"trk","target":{"line":"g","node":0,"attr":"width"}},
      {"op":"relation.create","type":"oscillate","target":"g","node":0,"attr":"width","wave":0}
    ]}"#;
    let doc4 = doc_from(both);
    let problems = mo2d::validate::validate(&doc4);
    assert!(problems.iter().any(|p| p.message.contains("conflicting writers")), "oscillate + track on the same claim must conflict");
}

#[test]
fn oscillate_round_trip_byte_stable() {
    let doc = osc_doc(0, TPS);
    let b1 = doc_bytes(&doc);
    let parsed = mo2d::format::parse_doc(&String::from_utf8(b1.clone()).unwrap()).unwrap();
    let b2 = doc_bytes(&parsed);
    assert_eq!(b1, b2, "oscillate relation must round-trip byte-stably");
}

#[test]
fn step_interp_holds_the_key_value() {
    // 3 keys, im=3 on both segments: mid-segment samples must hold the LEFT
    // key's value exactly (discrete channels — Switch/state-machine basis).
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"line.create","id":"trk","tags":["track"],
           "nodes":[{{"p":[0],"v":0.0}},{{"p":[{}],"v":100.0}},{{"p":[{}],"v":30.0}}],
           "segs":[{{"interp":{{"im":3}}}},{{"interp":{{"im":3}}}}]}},
          {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"g","node":1,"attr":"width"}}}}
        ]}}"#,
        TPS / 2,
        TPS
    );
    let doc = doc_from(&text);
    let q = eval::evaluate(&doc, TPS / 4).unwrap();
    assert!((q[0].nodes[1].width - 0.0).abs() < 1e-9, "step holds key0's value through segment 0, got {}", q[0].nodes[1].width);
    let three_quarter = eval::evaluate(&doc, 3 * TPS / 4).unwrap();
    assert!((three_quarter[0].nodes[1].width - 100.0).abs() < 1e-9, "step holds key1's value through segment 1, got {}", three_quarter[0].nodes[1].width);
    let at_key = eval::evaluate(&doc, TPS / 2).unwrap();
    assert!((at_key[0].nodes[1].width - 100.0).abs() < 1e-9, "exact key hits return verbatim");
}

#[test]
fn c7_frozen_demos() {
    for (key, stream) in [("breathe", "breathe.json"), ("sway", "sway.json")] {
        let frozen = load_frozen();
        let sec = frozen
            .raw
            .get(key)
            .unwrap_or_else(|| panic!("frozen.json missing '{}' (run demo/freeze.py)", key));
        let want_doc = sec.get("docSha256").and_then(|v| v.as_str()).expect("docSha256").to_string();
        let want_frame = sec.get("frameSha256").and_then(|v| v.as_str()).expect("frameSha256").to_string();
        let tick = sec.get("midTick").and_then(|v| v.as_i64()).expect("midTick");
        let doc = replay_stream(&repo_root().join("demo/cmd").join(stream));
        assert_eq!(sha256_hex(&doc_bytes(&doc)), want_doc, "{}: document hash drifted", key);
        assert_eq!(render_sha(&doc, tick, Camera::Ortho2d), want_frame, "{}: frame hash drifted", key);
    }
}

#[test]
fn c7_frozen_streams_live_in_the_demo_dir() {
    for name in ["breathe.json", "sway.json"] {
        assert!(demo_dir().join("cmd").join(name).is_file(), "missing demo/cmd/{}", name);
        assert!(Path::new(&demo_dir()).exists());
    }
}
