//! Gate B3 — gradient control bars (roadmap B3):
//! 1. control-bar line + color-stop nodes → fill recolored by projection
//! 2. animatable: a color track on a stop node drives the gradient
//!    (acceptance: a color-changing tentacle)
//! 3. stroke role; validator conflicts; damage parity when only fills change

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::math::Vec3;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, render_damaged, Camera, RenderOpts};

fn opts() -> RenderOpts {
    RenderOpts { width: 200, height: 120, camera: Camera::Ortho2d, samples: 1 }
}

fn gradient_doc() -> Document {
    let stream = r#"{"v":1,"cmds":[
      {"op":"doc.setRoot","width":200,"height":120},
      {"op":"line.create","id":"bar","width":1,"strokeOn":true,"nodes":[
        {"p":[0,0,0],"c":[255,0,0,255]},
        {"p":[100,0,0],"c":[0,0,255,255]}]},
      {"op":"line.create","id":"blob","closed":true,"fillOn":true,"strokeOn":false,
       "fillColor":[200,200,200,255],"nodes":[
        {"p":[25,50,0]},{"p":[75,10,0]},{"p":[90,60,0]},{"p":[40,80,0]}]},
      {"op":"relation.create","type":"gradient","bar":"bar","target":"blob","role":"fill"}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    doc
}

#[test]
fn fill_recolored_by_bar_projection() {
    let doc = gradient_doc();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let blob = lines.iter().find(|l| l.id == "blob").unwrap();
    // Chord (0,0)→(100,0): node x=25 → u=0.25 → lerp(red,blue,0.25) = [191,0,64];
    // x=75 → u=0.75 → [64,0,191]; clamped ends for x beyond the bar.
    assert_eq!(blob.nodes[0].fill, [191, 0, 64, 255], "u=0.25 stop lerp");
    assert_eq!(blob.nodes[1].fill, [64, 0, 191, 255], "u=0.75 stop lerp");
    assert_eq!(blob.nodes[2].fill, [26, 0, 230, 255], "x=90 → u=0.9 near the last stop");
    assert_eq!(blob.nodes[3].fill, [153, 0, 102, 255], "x=40 → u=0.4 mid ramp");
    // Gradients are relations: removing the relation restores the fill.
    let mut doc2 = gradient_doc();
    let del = parse_stream(r#"{"v":1,"cmds":[{"op":"relation.delete","id":"r1"}]}"#).unwrap();
    apply_stream(&mut doc2, &del).unwrap();
    let lines2 = eval::evaluate(&doc2, 0).unwrap();
    let blob2 = lines2.iter().find(|l| l.id == "blob").unwrap();
    assert_eq!(blob2.nodes[0].fill, [200, 200, 200, 255], "no gradient → line fill color");
    let _ = Vec3::ZERO; // keep math import honest
}

#[test]
fn animatable_gradient_color_changing_tentacle() {
    let mut doc = gradient_doc();
    // Drive the second stop's color with a track: blue → green over 1 s.
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"track.create","line":"bar","node":1,"attr":"color","tick":0,"value":[0,0,255,255]}},
      {{"op":"node.set","line":"bar","index":1,"c":[0,255,0,255],"atTick":{}}}
    ]}}"#,
        TICKS_PER_SECOND
    );
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();

    let t0 = eval::evaluate(&doc, 0).unwrap();
    let t1 = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let b0 = t0.iter().find(|l| l.id == "blob").unwrap();
    let b1 = t1.iter().find(|l| l.id == "blob").unwrap();
    assert_eq!(b0.nodes[0].fill, [191, 0, 64, 255], "start: red→blue ramp");
    // At t1 the second stop is green: u=0.25 → lerp(red, green, 0.25) = [191,64,0].
    assert_eq!(b1.nodes[0].fill, [191, 64, 0, 255], "end: red→green ramp — the gradient is animated");
    assert_ne!(b0.nodes[0].fill, b1.nodes[0].fill);
}

#[test]
fn stroke_role_recolors_stroke_not_fill() {
    let mut doc = gradient_doc();
    // Replace the fill gradient with a stroke gradient.
    let stream = r#"{"v":1,"cmds":[
      {"op":"relation.delete","id":"r1"},
      {"op":"relation.create","type":"gradient","bar":"bar","target":"blob","role":"stroke"}
    ]}"#;
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let blob = lines.iter().find(|l| l.id == "blob").unwrap();
    assert_eq!(blob.nodes[0].fill, [200, 200, 200, 255], "fill untouched under stroke role");
    assert_eq!(blob.nodes[0].color, [191, 0, 64, 255], "stroke color sampled from the bar");
}

#[test]
fn gradient_writer_conflicts_and_validation() {
    let mut doc = gradient_doc();
    // Same target+role twice → conflicting writers.
    let dup = parse_stream(r#"{"v":1,"cmds":[{"op":"relation.create","type":"gradient","bar":"bar","target":"blob","role":"fill"}]}"#).unwrap();
    apply_stream(&mut doc, &dup).unwrap();
    let problems = mo2d::validate::validate(&doc);
    assert!(problems.iter().any(|p| p.message.contains("conflicting writers")), "same role twice must conflict");
    // Different roles compose: fill + stroke both allowed.
    let mut doc2 = gradient_doc();
    let stroke = parse_stream(r#"{"v":1,"cmds":[{"op":"relation.create","type":"gradient","bar":"bar","target":"blob","role":"stroke"}]}"#).unwrap();
    apply_stream(&mut doc2, &stroke).unwrap();
    assert!(mo2d::validate::validate(&doc2).is_empty(), "fill+stroke gradients must coexist: {:?}",
        mo2d::validate::validate(&doc2));
    // Bad bar (1 node) rejected at command time.
    let mut doc3 = gradient_doc();
    let bad = parse_stream(r#"{"v":1,"cmds":[
      {"op":"line.create","id":"tiny","nodes":[{"p":[0,0,0]}]},
      {"op":"relation.create","type":"gradient","bar":"tiny","target":"blob"}
    ]}"#).unwrap();
    assert!(apply_stream(&mut doc3, &bad).is_err(), "1-node bar rejected");
}

#[test]
fn damage_parity_when_only_fills_change() {
    // The hardest damage case: geometry static, per-node fills animate.
    let mut doc = gradient_doc();
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"track.create","line":"bar","node":1,"attr":"color","tick":0,"value":[0,0,255,255]}},
      {{"op":"node.set","line":"bar","index":1,"c":[255,255,0,255],"atTick":{}}}
    ]}}"#,
        TICKS_PER_SECOND
    );
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let o = opts();
    let mut prev = None;
    let mut prev_lines = eval::evaluate(&doc, 0).unwrap();
    for s in 0..=4 {
        let tick = TICKS_PER_SECOND * s as i64 / 4;
        let lines = eval::evaluate(&doc, tick).unwrap();
        let full = render(&lines, 200.0, 120.0, &o);
        if let Some(ref p) = prev {
            let (dmg, stats) = render_damaged(p, &prev_lines, &lines, 200.0, 120.0, &o, &mut mo2d::assets::AssetStore::new());
            assert_eq!(dmg.px, full.px, "gradient-only change must damage-render byte-identically (step {})", s);
            assert!(stats.dirty_tiles > 0 && stats.dirty_tiles < stats.total_tiles);
        }
        prev = Some(full);
        prev_lines = lines;
    }
}

#[test]
fn gradient_docs_round_trip_byte_stable() {
    let doc = gradient_doc();
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "gradient relations round-trip byte-stably");
    let text = std::str::from_utf8(&b2).unwrap();
    assert!(text.contains("\"gradient\""));
    assert!(text.contains("\"fill\""));
}
