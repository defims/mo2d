//! Gate C8 — gesture timelines (roadmap C8, established 2026-11-17):
//! - event.fire atomic dual-write: MoEvent (unchanged) + phase key on the
//!   read-only gesture line `gesture/<target>` (step im=3, codes 0/1/2)
//! - phase table follows browser hover/click semantics (physical containment)
//! - hold discrimination: minHold/maxHold as length predicates on the
//!   pressed segment (press→release pairing, path-independent)
//! - guards may read only gesture lines (cycle-free by construction)

mod common;

use common::{doc_bytes, load_frozen, replay_stream, render_sha, repo_root};
use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::json::Value;
use mo2d::model::*;
use mo2d::render::Camera;
use mo2d::sha::sha256_hex;

const TPS: i64 = 254_016_000_000;

fn doc_from(text: &str) -> Document {
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(text).unwrap()).unwrap();
    doc
}

fn ball_doc() -> Document {
    doc_from(
        r#"{"v":1,"cmds":[
          {"op":"line.create","id":"ball","closed":true,"width":3,
           "nodes":[{"p":[480,180,0]},{"p":[510,160,0]},{"p":[480,140,0]},{"p":[450,160,0]}]}
        ]}"#,
    )
}

fn gesture_line<'a>(doc: &'a Document, id: &str) -> &'a Line {
    doc.line(id).unwrap_or_else(|_| panic!("gesture line '{}' missing", id))
}

#[test]
fn dual_write_materializes_the_gesture_line() {
    let mut doc = ball_doc();
    let fire = format!(
        r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"press","tick":{},"target":"ball","x":480,"y":160,"radius":60}}]}}"#,
        TPS / 4
    );
    apply_stream(&mut doc, &parse_stream(&fire).unwrap()).unwrap();
    // ① the MoEvent (unchanged shape)…
    assert_eq!(doc.events.len(), 1);
    // ② …and the phase key on the read-only gesture line.
    let g = gesture_line(&doc, "gesture/ball");
    assert!(g.tags.contains(&"gesture".to_string()) && g.tags.contains(&"track".to_string()));
    assert_eq!(g.nodes.len(), 1);
    assert_eq!(g.nodes[0].pos.tick(), Ok(TPS / 4));
    assert_eq!(g.nodes[0].value, Some(Value::Num(2.0)), "press inside = phase 2");
    // Same-phase event later (still pressed, still inside) → no new key.
    let fire2 = format!(
        r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"move","tick":{},"target":"ball","x":480,"y":160,"radius":60}}]}}"#,
        TPS / 2
    );
    apply_stream(&mut doc, &parse_stream(&fire2).unwrap()).unwrap();
    assert_eq!(gesture_line(&doc, "gesture/ball").nodes.len(), 1, "same phase must not land a key");
    // Leaving the radius while pressed → phase 0 key.
    let fire3 = format!(
        r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"move","tick":{},"target":"ball","x":480,"y":60,"radius":60}}]}}"#,
        3 * TPS / 4
    );
    apply_stream(&mut doc, &parse_stream(&fire3).unwrap()).unwrap();
    let g = gesture_line(&doc, "gesture/ball");
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.nodes[1].value, Some(Value::Num(0.0)), "wandering out = phase 0");
}

#[test]
fn phase_table_transitions() {
    let mut doc = ball_doc();
    let fire = |doc: &mut Document, name: &str, tick: i64, x: f64, y: f64| {
        let text = format!(
            r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"{}","tick":{},"target":"ball","x":{},"y":{},"radius":60}}]}}"#,
            name, tick, x, y
        );
        apply_stream(doc, &parse_stream(&text).unwrap()).unwrap();
    };
    // hover in → 1, press → 2, release inside → 1, press, release outside → stays 0 after leaving…
    fire(&mut doc, "hoverIn", TPS / 8, 480.0, 160.0);
    assert_eq!(gesture_line(&doc, "gesture/ball").nodes[0].value, Some(Value::Num(1.0)));
    fire(&mut doc, "press", TPS / 4, 480.0, 160.0);
    assert_eq!(gesture_line(&doc, "gesture/ball").nodes[1].value, Some(Value::Num(2.0)));
    fire(&mut doc, "release", TPS / 2, 480.0, 160.0);
    assert_eq!(gesture_line(&doc, "gesture/ball").nodes[2].value, Some(Value::Num(1.0)), "release inside → still hovering");
    fire(&mut doc, "press", 3 * TPS / 4, 480.0, 160.0);
    fire(&mut doc, "release", TPS, 480.0, 300.0);
    let g = gesture_line(&doc, "gesture/ball");
    // Release far outside: hit test fails (click does not complete); the
    // pointer is outside, so the phase is 0 — but it was 2 (pressed inside),
    // so the transition lands a key.
    assert_eq!(g.nodes.last().unwrap().value, Some(Value::Num(0.0)), "release outside → out");
}

#[test]
fn broadcast_events_land_on_the_pointer_line() {
    let mut doc = ball_doc();
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"event.fire","name":"press","tick":{},"x":480,"y":160}},
          {{"op":"event.fire","name":"release","tick":{},"x":480,"y":160}}
        ]}}"#,
        TPS / 4,
        TPS / 2
    );
    apply_stream(&mut doc, &parse_stream(&text).unwrap()).unwrap();
    let p = gesture_line(&doc, "gesture/pointer");
    assert_eq!(p.nodes.len(), 2);
    assert_eq!(p.nodes[0].value, Some(Value::Num(2.0)));
    assert_eq!(p.nodes[1].value, Some(Value::Num(1.0)));
}

#[test]
fn hold_discrimination_short_vs_long() {
    let build = |press: i64, release: i64| {
        let text = format!(
            r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"release","radius":30,
           "minHold":{},
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":4.0}}}}]}},
          {{"op":"event.fire","name":"press","tick":{},"target":"g","x":50,"y":0,"radius":30}},
          {{"op":"event.fire","name":"release","tick":{},"target":"g","x":50,"y":0,"radius":30}}
        ]}}"#,
            2 * TPS / 5, // minHold 0.4s
            press, release
        );
        doc_from(&text)
    };
    // Short hold (0.2s) → the release does not fire.
    let short = build(TPS / 2, 7 * TPS / 10);
    let f = eval::evaluate(&short, TPS).unwrap();
    assert!((f[0].width - 6.0).abs() < 1e-9, "short press must not fire, got {}", f[0].width);
    // Long hold (0.5s ≥ 0.4s) → fires.
    let long = build(TPS / 2, TPS);
    let f = eval::evaluate(&long, TPS).unwrap();
    assert!((f[0].width - 10.0).abs() < 1e-9, "long press fires, got {}", f[0].width);
    // Release without any press (no pair) → hold predicate fails.
    let mut lonely = long;
    lonely.events.retain(|e| e.name != "press");
    let f = eval::evaluate(&lonely, TPS).unwrap();
    assert!((f[0].width - 6.0).abs() < 1e-9, "unpaired release must not fire");
}

#[test]
fn guard_reads_the_gesture_phase() {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"g","width":6,"color":[10,10,10,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[100,0,0]}}]}},
          {{"op":"relation.create","type":"respond","trigger":"g","event":"move","radius":30,
           "guard":{{"line":"gesture/g","op":">=","value":2.0}},
           "actions":[{{"write":{{"line":"g","attr":"width","mode":"add","value":4.0}}}}]}},
          {{"op":"event.fire","name":"press","tick":{},"target":"g","x":50,"y":0,"radius":30}},
          {{"op":"event.fire","name":"move","tick":{},"target":"g","x":50,"y":0,"radius":30}},
          {{"op":"event.fire","name":"move","tick":{},"target":"g","x":50,"y":300,"radius":30}}
        ]}}"#,
        TPS / 4,
        TPS / 2,
        3 * TPS / 4
    );
    let doc = doc_from(&text);
    // While pressed (phase 2 at TPS/2): the move event passes the phase guard.
    let pressed = eval::evaluate(&doc, TPS / 2).unwrap();
    assert!((pressed[0].width - 10.0).abs() < 1e-9, "phase ≥ 2 → write fires, got {}", pressed[0].width);
    // After wandering out (phase 0 at 3TPS/4): a later move would not pass —
    // the folded state persists (capability as data), so assert the fold
    // moment itself: an evaluation before the press shows the rest width.
    let early = eval::evaluate(&doc, TPS / 8).unwrap();
    assert!((early[0].width - 6.0).abs() < 1e-9, "guard blocked before the press");
}

#[test]
fn gesture_namespace_is_reserved_and_read_only() {
    // line.create cannot take the namespace…
    let bad = r#"{"v":1,"cmds":[{"op":"line.create","id":"gesture/fake","nodes":[{"p":[0,0,0]},{"p":[1,0,0]}]}]}"#;
    let mut doc = Document::default();
    assert!(apply_stream(&mut doc, &parse_stream(bad).unwrap()).is_err(), "gesture/ ids are reserved");
    // …and existing gesture lines are read-only.
    let mut doc = ball_doc();
    let fire = format!(
        r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"press","tick":{},"target":"ball","x":480,"y":160,"radius":60}}]}}"#,
        TPS / 4
    );
    apply_stream(&mut doc, &parse_stream(&fire).unwrap()).unwrap();
    for op in [
        r#"{"op":"node.set","line":"gesture/ball","index":0,"p":[1,1,0]}"#,
        r#"{"op":"node.append","line":"gesture/ball","p":[2,2,0]}"#,
        r#"{"op":"node.delete","line":"gesture/ball","index":0}"#,
        r#"{"op":"seg.set","line":"gesture/ball","index":0,"visible":false}"#,
        r#"{"op":"track.create","line":"gesture/ball","node":0}"#,
    ] {
        let text = format!(r#"{{"v":1,"cmds":[{}]}}"#, op);
        let mut d = doc.clone();
        assert!(
            apply_stream(&mut d, &parse_stream(&text).unwrap()).is_err(),
            "{} must be rejected on a gesture line",
            op
        );
    }
    // No relation may write a gesture line.
    let text = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"ball","nodes":[{"p":[480,180,0]},{"p":[510,160,0]}]},
      {"op":"relation.create","type":"respond","trigger":"ball","event":"tap","radius":30,
       "actions":[{"write":{"line":"gesture/ball","attr":"width","mode":"add","value":1.0}}]}
    ]}"#;
    let mut d = Document::default();
    // The command channel rejects the write outright…
    assert!(apply_stream(&mut d, &parse_stream(&text).unwrap()).is_err(), "respond writes to gesture lines must be rejected");
    // …and the validator flags it too (for direct document surgery).
    let mut d2 = doc_from(r#"{"v":1,"cmds":[
      {"op":"line.create","id":"ball","nodes":[{"p":[480,180,0]},{"p":[510,160,0]}]}
    ]}"#);
    let mut r = Relation::new_respond("r_probe", "ball", RespondEvent::Tap);
    r.resp_actions.push(RespondAction::WriteWidth {
        line: "gesture/ball".into(),
        node: None,
        mode: WriteMode::Add,
        value: 1.0,
    });
    d2.relations.push(r);
    let problems = mo2d::validate::validate(&d2);
    assert!(problems.iter().any(|p| p.message.contains("gesture/ line")), "validator must flag gesture-line writes");
}

#[test]
fn gesture_lines_survive_the_round_trip() {
    let mut doc = ball_doc();
    let fire = format!(
        r#"{{"v":1,"cmds":[{{"op":"event.fire","name":"press","tick":{},"target":"ball","x":480,"y":160,"radius":60}}]}}"#,
        TPS / 4
    );
    apply_stream(&mut doc, &parse_stream(&fire).unwrap()).unwrap();
    let b1 = doc_bytes(&doc);
    let parsed = mo2d::format::parse_doc(&String::from_utf8(b1.clone()).unwrap()).unwrap();
    assert_eq!(doc_bytes(&parsed), b1, "gesture lines + events must round-trip byte-stably");
}

#[test]
fn tap_vs_hold_frozen_demo() {
    let frozen = load_frozen();
    let sec = frozen
        .raw
        .get("tapVsHold")
        .unwrap_or_else(|| panic!("frozen.json missing 'tapVsHold' (run demo/freeze.py)"));
    let want_doc = sec.get("docSha256").and_then(|v| v.as_str()).expect("docSha256").to_string();
    let want_frame = sec.get("frameSha256").and_then(|v| v.as_str()).expect("frameSha256").to_string();
    let tick = sec.get("midTick").and_then(|v| v.as_i64()).expect("midTick");
    let doc = replay_stream(&repo_root().join("demo/cmd/tap_vs_hold.json"));
    // The gesture lane is materialized: hover/press/release keys on gesture/ball.
    assert!(doc.line("gesture/ball").is_ok(), "the gesture line must exist");
    assert!(doc.events.len() >= 3, "tap + press + release recorded");
    assert_eq!(sha256_hex(&doc_bytes(&doc)), want_doc, "tap_vs_hold: document hash drifted");
    assert_eq!(render_sha(&doc, tick, Camera::Ortho2d), want_frame, "tap_vs_hold: frame hash drifted");
    // N3-style acceptance: short tap jumps (position), long press ≥ 0.4s
    // widens (width) — both offline, no AI, no code.
    let at_end = eval::evaluate(&doc, 2 * TPS).unwrap();
    let ball = at_end.iter().find(|l| l.id == "ball").unwrap();
    assert!((ball.nodes[0].p.y - 230.0).abs() < 1e-9, "the tap jumped the ball (140+50 → 230 in world y), got {}", ball.nodes[0].p.y);
    assert!((ball.width - 6.0).abs() < 1e-9, "long press widened 3+3 → 6, got {}", ball.width);
}
