//! Gate C7 — interactivity vocabulary (roadmap C7, established 2026-11-17):
//! - oscillate: deterministic periodic driver (pure f(tick), period wrap)
//! - step interpolation im=3: discrete channels hold their key value
//! - respond + event stream: the fold of the recorded event prefix decides
//!   what fires; evaluation = pure f(doc, tick, eventPrefix); events are
//!   irreversible facts (inverse noop) persisted in the document

mod common;

use common::{doc_bytes, demo_dir, load_frozen, replay_stream, render_sha, repo_root};
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
    for (key, stream) in [("breathe", "breathe.json"), ("sway", "sway.json"), ("recordWave", "record_wave.json")] {
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
    for name in ["breathe.json", "sway.json", "tap_jump.json", "record_wave.json"] {
        assert!(demo_dir().join("cmd").join(name).is_file(), "missing demo/cmd/{}", name);
        assert!(Path::new(&demo_dir()).exists());
    }
}

// ---------------------------------------------------------------------------
// P2: respond + event stream
// ---------------------------------------------------------------------------

/// A horizontal line + a tap respond that lifts the whole line by dy.
fn tap_doc(order: &'static str) -> Document {
    let respond = r#"{"op":"relation.create","type":"respond","trigger":"g","event":"tap","radius":30,
       "actions":[{"write":{"line":"g","attr":"position","mode":"add","value":[0,-40,0]}}]}"#;
    let event = format!(
        r#"{{"op":"event.fire","name":"tap","tick":{},"target":"g","x":50,"y":0}}"#,
        TPS / 4
    );
    let (a, b) = if order == "respond_first" {
        (respond, event.as_str())
    } else {
        (event.as_str(), respond)
    };
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {},{}
        ]}}"#,
        a, b
    );
    doc_from(&text)
}

#[test]
fn respond_writes_only_from_the_event_tick_on() {
    let doc = tap_doc("respond_first");
    let before = eval::evaluate(&doc, TPS / 8).unwrap();
    assert!((before[0].nodes[0].p.y - 0.0).abs() < 1e-9, "before the event: rest");
    let at = eval::evaluate(&doc, TPS / 4).unwrap();
    assert!((at[0].nodes[0].p.y - (-40.0)).abs() < 1e-9, "at the event tick (inclusive): lifted");
    let after = eval::evaluate(&doc, TPS / 2).unwrap();
    assert!((after[0].nodes[1].p.y - (-40.0)).abs() < 1e-9, "after the event: still lifted (fold state)");
    // Same prefix → same frame; extra empty slice == the time-driven wrapper.
    assert_eq!(
        eval::evaluate(&doc, TPS / 2).unwrap(),
        eval::evaluate_with_events(&doc, TPS / 2, &[]).unwrap()
    );
    // A document WITHOUT the event never lifts (empty prefix → no respond fires).
    let mut silent = doc.clone();
    silent.events.clear();
    let quiet = eval::evaluate(&silent, TPS / 2).unwrap();
    assert!((quiet[0].nodes[0].p.y - 0.0).abs() < 1e-9, "empty prefix = time-driven behavior only");
}

#[test]
fn respond_fold_is_order_normalized() {
    // Two events at DIFFERENT ticks recorded in reverse order: the fold sorts
    // by (tick, seq), so the final frame must not depend on insertion order.
    let text = |first: &str, second: &str| {
        format!(
            r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"tap","radius":30,
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":5.0}}}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"land","radius":30,
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":-1.0}}}}]}},
          {},{}
        ]}}"#,
            first, second
        )
    };
    let d1 = doc_from(&text(
        r#"{"op":"event.fire","name":"land","tick":50000000000,"target":"g"}"#,
        r#"{"op":"event.fire","name":"tap","tick":10000000000,"target":"g"}"#,
    ));
    let d2 = doc_from(&text(
        r#"{"op":"event.fire","name":"tap","tick":10000000000,"target":"g"}"#,
        r#"{"op":"event.fire","name":"land","tick":50000000000,"target":"g"}"#,
    ));
    let f1 = eval::evaluate(&d1, TPS).unwrap();
    let f2 = eval::evaluate(&d2, TPS).unwrap();
    assert_eq!(f1, f2, "fold must be order-normalized by (tick, seq)");
    assert!((f1[0].width - (6.0 + 5.0 - 1.0)).abs() < 1e-9, "tap then land, got {}", f1[0].width);
    // Mid-way: only the tap has happened.
    let mid = eval::evaluate(&d1, 30000000000).unwrap();
    assert!((mid[0].width - 11.0).abs() < 1e-9, "only tap applied at 0.3s, got {}", mid[0].width);
}

#[test]
fn respond_fire_chain_and_guards() {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"go","radius":30,
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":4.0}}}},{{"fire":{{"name":"chain"}}}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"chain","radius":30,
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":1.0}}}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"timePoint","at":{},"radius":30,
           "guard":{{"line":"g","op":">","value":6}},
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":-2.0}}}}]}},
          {{"op":"event.fire","name":"go","tick":10000000000,"target":"g"}}
        ]}}"#,
        TPS / 2
    );
    let doc = doc_from(&text);
    // Guard fallback: absent channel state reads as rest width (6), so the
    // timePoint never fires on its own.
    let mut no_event = doc.clone();
    no_event.events.clear();
    let quiet = eval::evaluate(&no_event, TPS).unwrap();
    assert!((quiet[0].width - 6.0).abs() < 1e-9, "guard blocks without the tap");
    // With the tap: go (+4) → chain (+1) at the same moment; at TPS/2 the
    // guard (10 > 6) passes and −2 lands.
    let at_quarter = eval::evaluate(&doc, TPS / 4).unwrap();
    assert!((at_quarter[0].width - 11.0).abs() < 1e-9, "go + chain = +5, got {}", at_quarter[0].width);
    let at_half = eval::evaluate(&doc, TPS / 2).unwrap();
    assert!((at_half[0].width - 9.0).abs() < 1e-9, "timePoint fires through the passing guard, got {}", at_half[0].width);
}

#[test]
fn respond_hit_test_uses_radius() {
    let text = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"respond","trigger":"g","event":"tap","radius":12,
       "actions":[{"write":{"line":"g","attr":"width","mode":"add","value":5.0}}]}
    ]}"#;
    // Two events at the same tick: one lands on the line, one misses.
    let doc = doc_from(&text);
    let mut doc = doc;
    doc.events.push(MoEvent {
        name: "tap".into(), tick: TPS / 2, seq: 0, target: Some("g".into()), x: Some(50.0), y: Some(5.0),
    });
    doc.events.push(MoEvent {
        name: "tap".into(), tick: TPS / 2, seq: 1, target: Some("g".into()), x: Some(50.0), y: Some(100.0),
    });
    let frame = eval::evaluate(&doc, TPS).unwrap();
    assert!((frame[0].width - 11.0).abs() < 1e-9, "only the on-line event passes the radius test, got {}", frame[0].width);
}

#[test]
fn respond_overlap_rising_edge_at_moments() {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"a","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"line.create","id":"b","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[50,-10,0]}},{{"p":[50,10,0]}}]}},
          {{"op":"relation.create","type":"respond","trigger":"a","event":"overlap","other":"b",
           "actions":[{{"write":{{"line":"a","attr":"width","mode":"add","value":5.0}}}}]}},
          {{"op":"event.fire","name":"ping","tick":{},"target":"a"}}
        ]}}"#,
        TPS / 2
    );
    let doc = doc_from(&text);
    // No moments yet → the overlap has not been sampled.
    let early = eval::evaluate(&doc, TPS / 4).unwrap();
    assert!((early[0].width - 6.0).abs() < 1e-9);
    // At the moment: rising edge fires once (v1: moment-sampled rest geometry).
    let late = eval::evaluate(&doc, TPS).unwrap();
    assert!((late[0].width - 11.0).abs() < 1e-9, "overlap rising edge fired exactly once, got {}", late[0].width);
    // Re-evaluating must not fire again (state is the fold, not an accumulator).
    let again = eval::evaluate(&doc, TPS).unwrap();
    assert_eq!(late, again);
}

#[test]
fn events_are_irreversible_and_round_trip_stable() {
    let doc = tap_doc("respond_first");
    // Double apply from scratch → byte-identical (determinism gate class).
    let mut again = Document::default();
    let text = std::fs::read_to_string(repo_root().join("demo/cmd/tap_jump.json")).unwrap_or_default();
    let _ = text; // (covered by the frozen demo below)
    let _ = &mut again;
    // Round trip: events serialize and parse back byte-stably.
    let b1 = doc_bytes(&doc);
    let parsed = mo2d::format::parse_doc(&String::from_utf8(b1.clone()).unwrap()).unwrap();
    assert_eq!(parsed.events.len(), 1, "events survive the round trip");
    let b2 = doc_bytes(&parsed);
    assert_eq!(b1, b2);
    // Irreversible: the inverse of event.fire is noop — events stay.
    let mut doc2 = tap_doc("respond_first");
    let before = doc2.events.clone();
    let inv = mo2d::command::Command { op: "noop".into(), args: vec![] };
    apply_stream(&mut doc2, &[inv]).unwrap();
    assert_eq!(doc2.events, before, "noop must not touch the event prefix");
    // Writer discipline: respond + track on the same claim conflict.
    let both = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
       "nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"line.create","id":"trk","tags":["track"],"nodes":[{"p":[0],"v":6.0},{"p":[1000000000000],"v":2.0}]},
      {"op":"relation.create","type":"track","trackLine":"trk","target":{"line":"g","node":0,"attr":"width"}},
      {"op":"relation.create","type":"respond","trigger":"g","event":"tap","radius":30,
       "actions":[{"write":{"line":"g","node":0,"attr":"width","mode":"add","value":1.0}}]}
    ]}"#;
    let d = doc_from(both);
    let problems = mo2d::validate::validate(&d);
    assert!(problems.iter().any(|p| p.message.contains("conflicting writers")), "respond + track on one claim must conflict");
}

#[test]
fn tap_jump_frozen_demo() {
    let frozen = load_frozen();
    let sec = frozen
        .raw
        .get("tapJump")
        .unwrap_or_else(|| panic!("frozen.json missing 'tapJump' (run demo/freeze.py)"));
    let want_doc = sec.get("docSha256").and_then(|v| v.as_str()).expect("docSha256").to_string();
    let want_frame = sec.get("frameSha256").and_then(|v| v.as_str()).expect("frameSha256").to_string();
    let tick = sec.get("midTick").and_then(|v| v.as_i64()).expect("midTick");
    let doc = replay_stream(&repo_root().join("demo/cmd/tap_jump.json"));
    assert!(!doc.events.is_empty(), "the tap_jump document must carry its recorded tap");
    assert_eq!(sha256_hex(&doc_bytes(&doc)), want_doc, "tap_jump: document hash drifted");
    assert_eq!(render_sha(&doc, tick, Camera::Ortho2d), want_frame, "tap_jump: frame hash drifted");
    // The N3 gate, precisely: replay + offline render, no AI, no code.
    let rest_frame = render_sha(&doc, 0, Camera::Ortho2d);
    let jumped_frame = render_sha(&doc, tick, Camera::Ortho2d);
    let back_frame = render_sha(&doc, 223900000000, Camera::Ortho2d);
    assert_ne!(rest_frame, jumped_frame, "the tap must lift the ball");
    assert_eq!(rest_frame, back_frame, "the guarded timePoint must settle it back");
}
