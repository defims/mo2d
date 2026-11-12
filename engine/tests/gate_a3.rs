//! Gate A3 — tracks + Tick + playback (mvp.md §3).
//!
//! Keyframe ticks ascend (validator enforces); Linear/Bezier interpolation
//! switchable on the same track; loop playback is frame-exact reproducible.

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::json::Value;
use mo2d::model::*;

const TPS: i64 = 254_016_000_000;

fn track_doc(im: i64) -> Document {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"line.create","id":"trk","tags":["track"],
           "nodes":[{{"p":[0],"v":0.0}},{{"p":[{}],"v":100.0}},{{"p":[{}],"v":0.0}}],
           "segs":[{{"interp":{{"im":{}}}}},{{"interp":{{"im":{}}}}}]}},
          {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"g","node":1,"attr":"width"}}}}
        ]}}"#,
        TPS / 2, TPS, im, im
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&text).unwrap()).unwrap();
    doc
}

#[test]
fn a3_ticks_must_ascend() {
    let text = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"trk","tags":["track"],
       "nodes":[{"p":[0],"v":1.0},{"p":[50000000000],"v":2.0},{"p":[10000000000],"v":3.0}]}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(text).unwrap()).unwrap();
    let problems = mo2d::validate::validate(&doc);
    assert!(
        problems.iter().any(|p| p.message.contains("ascending")),
        "descending keyframe ticks must be rejected: {:?}",
        problems
    );
}

#[test]
fn a3_linear_bezier_switchable() {
    let linear = track_doc(0);
    let bezier = track_doc(9);
    let t_quarter = TPS / 8; // u = 0.25 on the first segment

    let l = eval::evaluate(&linear, t_quarter).unwrap()[0].nodes[1].width;
    let b = eval::evaluate(&bezier, t_quarter).unwrap()[0].nodes[1].width;
    // Linear at u=0.25 → 25.0. Default-handle Bezier eases: ≈ 0.156·100 ≈ 15.6.
    assert!((l - 25.0).abs() < 1e-9, "linear interpolation at u=0.25, got {}", l);
    assert!(b < l - 5.0, "bezier must ease differently from linear ({} vs {})", b, l);
    // Both hit the exact keyframe values at the keys themselves.
    for doc in [&linear, &bezier] {
        let at = |t: i64| eval::evaluate(doc, t).unwrap()[0].nodes[1].width;
        assert_eq!(at(0), 0.0);
        assert_eq!(at(TPS / 2), 100.0);
        assert_eq!(at(TPS), 0.0);
    }
}

#[test]
fn a3_tick_time_model() {
    // Time truth is Ticks; fps is display-only. Frame→tick is exact integer math.
    let doc = track_doc(0);
    assert_eq!(doc.root.ticks_per_second, TPS, "the Tick constant is frozen (file-format.md §2.1)");
    // 24 fps → 10_584_000_000 ticks per frame exactly.
    assert_eq!(doc.frame_tick(1), TPS / 24);
    assert_eq!(doc.frame_tick(24), TPS);
    // Common rates divide exactly (the reason this constant was chosen).
    for fps in [24.0, 25.0, 30.0, 48.0, 50.0, 60.0, 120.0, 240.0] {
        let mut d = track_doc(0);
        d.root.fps = fps;
        let tpf = d.frame_tick(1);
        assert_eq!(tpf * fps as i64, TPS, "fps {} must divide the tick constant exactly", fps);
    }
}

#[test]
fn a3_loop_playback_frame_reproducible() {
    // Loop playback = the playhead wraps (eff(t) = t mod period, exactly what
    // the stage's player does); evaluation itself is pure per tick. The gate:
    // 1. the loop point joins exactly (frame(0) == frame(TPS): no pop),
    // 2. a wrapped second pass reproduces every frame hash of the first,
    // 3. evaluation order does not matter (purity).
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let tps = doc.root.ticks_per_second;
    let step = tps / 24;
    let frame = |t: i64| common::render_sha(&doc, t, mo2d::render::Camera::Ortho2d);

    // 1. Loop join: first key == last key value → identical frames.
    assert_eq!(frame(0), frame(tps), "the loop point must join exactly");

    // 2. Two passes over one period (second one wrapped from t+period).
    let mut first = Vec::new();
    let mut second = Vec::new();
    for k in 0..24i64 {
        first.push(frame(k * step));
        second.push(frame((k * step + tps) % tps)); // playhead wrapped one period later
    }
    assert_eq!(first, second, "loop playback must be frame-exact reproducible");

    // 3. Purity: evaluate the same ticks in reverse order — same hashes.
    let mut reversed = Vec::new();
    for k in (0..24i64).rev() {
        reversed.push(frame(k * step));
    }
    reversed.reverse();
    assert_eq!(first, reversed, "evaluation must be order-independent");

    // And the animation actually moves within the period (no frozen loop).
    let distinct: std::collections::BTreeSet<&String> = first.iter().collect();
    assert!(distinct.len() >= 12, "a looping tentacle must animate ({} distinct frames)", distinct.len());
}

#[test]
fn a3_track_is_a_line() {
    // D1: a track IS a line — nodes at Tick positions carrying values,
    // interpolation is a SEGMENT attribute on that line.
    let doc = track_doc(9);
    let trk = doc.line("trk").unwrap();
    assert!(trk.is_track(), "track line nodes are Tick-positioned");
    let ticks: Vec<i64> = trk.nodes.iter().map(|n| n.pos.tick().unwrap()).collect();
    assert_eq!(ticks, vec![0, TPS / 2, TPS]);
    assert!(trk.segs[0].interp.is_some(), "interp lives on the segment (D1)");
    // The track node value is a node-level attribute (v).
    assert!(matches!(trk.nodes[0].value, Some(Value::Num(0.0))));
}
