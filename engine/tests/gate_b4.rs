//! Gate B4 — multi-layer/groups + transform-box lines (roadmap B4):
//! 1. a box line's rest→current similarity scales and rotates member lines
//!    (the transform box animates via tracks — rest-relative, same rule as
//!    bind: the document rest state IS the untransformed frame)
//! 2. layer visibility parameter hides members
//! 3. groups nest (the inner group's box is a member of the outer group)
//! 4. one-writer rule: group members conflict with bind/other groups
//! 5. damage parity + round-trip byte stability

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::math::Vec3;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, render_damaged, Camera, RenderOpts};

fn opts() -> RenderOpts {
    RenderOpts { width: 320, height: 240, camera: Camera::Ortho2d, samples: 1 }
}

fn stage_doc() -> Document {
    let stream = r#"{"v":1,"cmds":[
      {"op":"doc.setRoot","width":320,"height":240},
      {"op":"line.create","id":"box","closed":true,"width":1,"nodes":[
        {"p":[0,0,0]},{"p":[100,0,0]},{"p":[100,100,0]},{"p":[0,100,0]}]},
      {"op":"line.create","id":"char1","width":4,"nodes":[{"p":[30,40,0]},{"p":[60,40,0]}]},
      {"op":"line.create","id":"char2","width":4,"nodes":[{"p":[70,70,0]},{"p":[90,70,0]}]},
      {"op":"relation.create","type":"group","box":"box","members":["char1","char2"]}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    doc
}

/// Track the box's diagonal node (node 3, the transform reference) from its
/// rest position to `to` over 1 s (keyframe at tick 0 keeps the rest).
fn drive_box_diagonal(doc: &mut Document, to: [f64; 3]) {
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"track.create","line":"box","node":3,"attr":"position","tick":0,"value":[0,100,0]}},
      {{"op":"node.set","line":"box","index":3,"p":[{}, {}, {}],"atTick":{}}}
    ]}}"#,
        to[0], to[1], to[2], TICKS_PER_SECOND
    );
    apply_stream(doc, &parse_stream(&stream).unwrap()).unwrap();
}

#[test]
fn rest_box_is_identity() {
    let doc = stage_doc();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let c1 = lines.iter().find(|l| l.id == "char1").unwrap();
    assert_eq!(c1.nodes[0].p, Vec3::new(30.0, 40.0, 0.0), "rest box → identity transform");
    assert_eq!(c1.nodes[0].width, 4.0);
    assert!(c1.visible);
}

#[test]
fn box_scaling_scales_members() {
    let mut doc = stage_doc();
    drive_box_diagonal(&mut doc, [0.0, 200.0, 0.0]);
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let c1 = lines.iter().find(|l| l.id == "char1").unwrap();
    // Anchor (0,0), diagonal (0,100)→(0,200): a=2, b=0 → (x,y) → (2x, 2y).
    assert_eq!(c1.nodes[0].p, Vec3::new(60.0, 80.0, 0.0), "scale 2× about the anchor");
    assert_eq!(c1.nodes[1].p, Vec3::new(120.0, 80.0, 0.0));
    assert_eq!(c1.nodes[0].width, 8.0, "stroke widths scale with the box");
}

#[test]
fn box_rotation_rotates_members() {
    let mut doc = stage_doc();
    drive_box_diagonal(&mut doc, [100.0, 0.0, 0.0]);
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let c1 = lines.iter().find(|l| l.id == "char1").unwrap();
    // Rotate 90°: a=0, b=−1 → (x,y) → (y, −x).
    assert_eq!(c1.nodes[0].p, Vec3::new(40.0, -30.0, 0.0), "90° rotation about the anchor");
    // Mid-animation: diagonal linear (0,100)→(100,0) midpoint = (50,50).
    // a = 0.5, b = −0.5 → (x,y) → (0.5x+0.5y, −0.5x+0.5y): (30,40) → (35,5).
    let mid = eval::evaluate(&doc, TICKS_PER_SECOND / 2).unwrap();
    let c1m = mid.iter().find(|l| l.id == "char1").unwrap();
    assert_eq!(c1m.nodes[0].p, Vec3::new(35.0, 5.0, 0.0), "mid-track drives a partial rotate+scale (deterministic)");
}

#[test]
fn nested_groups_compose() {
    let mut doc = stage_doc();
    let nested = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"boxB","closed":true,"width":1,"nodes":[
        {"p":[0,0,0]},{"p":[10,0,0]},{"p":[10,10,0]},{"p":[0,10,0]}]},
      {"op":"line.create","id":"char3","width":2,"nodes":[{"p":[1,2,0]},{"p":[5,2,0]}]},
      {"op":"relation.create","id":"inner","type":"group","box":"boxB","members":["char3"]},
      {"op":"relation.create","id":"outer2","type":"group","box":"box","members":["boxB"]}
    ]}"#;
    apply_stream(&mut doc, &parse_stream(nested).unwrap()).unwrap();
    assert!(
        mo2d::validate::validate(&doc).is_empty(),
        "nested groups compose: each line still has exactly one writer"
    );
    drive_box_diagonal(&mut doc, [0.0, 200.0, 0.0]);
    let lines = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let c3 = lines.iter().find(|l| l.id == "char3").unwrap();
    // Outer 2× scales boxB (diag 10→20); the inner group reads the scaled
    // boxB → char3 rides the inner transform only: (1,2) × 2 = (2,4).
    assert_eq!(c3.nodes[0].p, Vec3::new(2.0, 4.0, 0.0), "nested transforms compose transitively");
    assert_eq!(c3.nodes[0].width, 4.0, "width scales through the chain");
}

#[test]
fn layer_visibility_hides_members() {
    let mut doc = stage_doc();
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"line.setAttrs","id":"box","strokeOn":false},
      {"op":"relation.delete","id":"r1"},
      {"op":"relation.create","type":"group","box":"box","members":["char1","char2"],"visible":false}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let c1 = lines.iter().find(|l| l.id == "char1").unwrap();
    let c2 = lines.iter().find(|l| l.id == "char2").unwrap();
    assert!(!c1.visible && !c2.visible, "hidden group hides members");
    let img = render(&lines, 320.0, 240.0, &opts());
    let blank = render(&[], 320.0, 240.0, &opts());
    assert_eq!(img.px, blank.px, "hidden members must not draw");
    // Toggling visible back on restores rendering.
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"relation.delete","id":"r1"},
      {"op":"relation.create","type":"group","box":"box","members":["char1","char2"],"visible":true}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let img2 = render(&lines, 320.0, 240.0, &opts());
    assert_ne!(img2.px, blank.px, "visible group draws again");
}

#[test]
fn group_writer_conflicts() {
    let mut doc = stage_doc();
    // bind + group on the same member line → conflicting writers.
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"line.create","id":"skel","tags":["skeleton"],"nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]},
      {"op":"relation.create","type":"bind","target":"char1","skeleton":"skel"}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let problems = mo2d::validate::validate(&doc);
    assert!(
        problems.iter().any(|p| p.message.contains("conflicting writers")),
        "bind on a group member must conflict with the group claim: {:?}",
        problems
    );
    // Second group claiming the same member → conflict.
    let mut doc2 = stage_doc();
    let g2 = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"line.create","id":"box2","closed":true,"nodes":[{"p":[0,0,0]},{"p":[50,0,0]},{"p":[50,50,0]},{"p":[0,50,0]}]},
      {"op":"relation.create","type":"group","box":"box2","members":["char1"]}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc2, &g2).unwrap();
    let problems2 = mo2d::validate::validate(&doc2);
    assert!(problems2.iter().any(|p| p.message.contains("conflicting writers")));
    // Missing member + box-as-member rejected at command time.
    let mut doc3 = stage_doc();
    let bad = parse_stream(r#"{"v":1,"cmds":[
      {"op":"relation.create","type":"group","box":"box","members":["ghost"]}
    ]}"#).unwrap();
    assert!(apply_stream(&mut doc3, &bad).is_err());
    let bad2 = parse_stream(r#"{"v":1,"cmds":[
      {"op":"relation.create","type":"group","box":"box","members":["box"]}
    ]}"#).unwrap();
    assert!(apply_stream(&mut doc3, &bad2).is_err(), "box cannot be its own member");
}

#[test]
fn group_damage_parity_and_round_trip() {
    let mut doc = stage_doc();
    drive_box_diagonal(&mut doc, [0.0, 260.0, 0.0]);
    let o = opts();
    let mut prev = None;
    let mut prev_lines = eval::evaluate(&doc, 0).unwrap();
    for s in 0..=4 {
        let tick = TICKS_PER_SECOND * s as i64 / 4;
        let lines = eval::evaluate(&doc, tick).unwrap();
        let full = render(&lines, 320.0, 240.0, &o);
        if let Some(ref p) = prev {
            let (dmg, _) = render_damaged(p, &prev_lines, &lines, 320.0, 240.0, &o, &mut mo2d::assets::AssetStore::new());
            assert_eq!(dmg.px, full.px, "group-animated frames must damage-render byte-identically (step {})", s);
        }
        prev = Some(full);
        prev_lines = lines;
    }
    // Round-trip byte stability with a group relation in the document.
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "group relations round-trip byte-stably");
    assert!(std::str::from_utf8(&b2).unwrap().contains("\"group\""));
}
