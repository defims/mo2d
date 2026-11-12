//! Gate B2 — brush-stroke style library (roadmap B2):
//! 1. node-level pressure drives width (pressure-sensitive width)
//! 2. brush sizeAmp/sizeScale/tint consumed deterministically
//! 3. styles = attribute template library: the SAME line switches between
//!    brush strokes via style.apply; undo restores
//! 4. round-trip byte stability with brush/pressure/styles in the document

mod common;

use mo2d::command::{apply, apply_stream, parse_stream};
use mo2d::eval;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, Camera, RenderOpts};
use mo2d::sha::sha256_hex;

fn opts() -> RenderOpts {
    RenderOpts { width: 160, height: 90, camera: Camera::Ortho2d, samples: 1 }
}

fn base_doc() -> Document {
    let stream = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"stroke","width":10,"nodes":[{"p":[10,45,0]},{"p":[150,45,0]}]}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    doc
}

#[test]
fn pressure_shapes_width() {
    let mut doc = base_doc();
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"node.set","line":"stroke","index":0,"pr":0.5},
      {"op":"node.set","line":"stroke","index":1,"pr":2.0}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let s = lines.iter().find(|l| l.id == "stroke").unwrap();
    assert_eq!(s.nodes[0].width, 5.0, "pressure 0.5 × width 10");
    assert_eq!(s.nodes[1].width, 20.0, "pressure 2.0 × width 10");
    // Explicit node width still wins over pressure.
    let cmds = parse_stream(r#"{"v":1,"cmds":[{"op":"node.set","line":"stroke","index":0,"w":3.0}]}"#).unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let s = eval::evaluate(&doc, 0).unwrap().pop().unwrap();
    assert_eq!(s.nodes[0].width, 3.0, "explicit width overrides pressure");
}

#[test]
fn brush_size_amp_scale_tint() {
    let mut doc = base_doc();
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"node.set","line":"stroke","index":0,"pr":2.0},
      {"op":"line.setAttrs","id":"stroke","color":[255,100,50,255],
       "brush":{"spacing":0.5,"sizeAmp":0.5,"sizeScale":2.0,"tint":[128,255,0,255],"hueDrift":0.1}}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    // pressure 2 → amp: 1 + (2−1)×0.5 = 1.5 → 10×1.5 = 15 → ×sizeScale 2 = 30
    let s = &eval::evaluate(&doc, 0).unwrap()[0];
    assert_eq!(s.nodes[0].width, 30.0);
    // tint: [255,100,50] × [128/255, 1, 0] = [128, 100, 0]
    assert_eq!(s.nodes[0].color, [128, 100, 0, 255], "tint multiplies component-wise");
    // sizeAmp 0 → pressure has no effect (amp flattens the delta). Note brush
    // replacement semantics: the new brush object omits sizeScale → default 1.
    let cmds = parse_stream(r#"{"v":1,"cmds":[{"op":"line.setAttrs","id":"stroke","brush":{"sizeAmp":0.0}}]}"#).unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let s = &eval::evaluate(&doc, 0).unwrap()[0];
    assert_eq!(s.nodes[0].width, 10.0, "sizeAmp 0 flattens pressure; sizeScale back to default 1");
}

#[test]
fn same_line_switches_between_brush_strokes() {
    let mut doc = base_doc();
    let setup = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"style.create","id":"marker","width":12,"color":[40,90,200,255],"fillColor":[40,90,200,255]},
      {"op":"style.create","id":"pen","width":2,"color":[20,20,20,255],"brush":{"sizeAmp":1.5,"sizeScale":1.0}},
      {"op":"node.set","line":"stroke","index":0,"pr":0.3},
      {"op":"node.set","line":"stroke","index":1,"pr":1.8}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &setup).unwrap();

    let apply_a = parse_stream(r#"{"v":1,"cmds":[{"op":"style.apply","id":"marker","line":"stroke"}]}"#).unwrap();
    let eff_a = apply_stream(&mut doc, &apply_a).unwrap();
    let a = eval::evaluate(&doc, 0).unwrap();
    let img_a = render(&a, 160.0, 90.0, &opts());
    let hash_a = sha256_hex(&mo2d::png::encode_png(&img_a));
    assert_eq!(a[0].width, 12.0);
    assert_eq!(a[0].nodes[0].color, [40, 90, 200, 255], "marker style recolors");

    let apply_b = parse_stream(r#"{"v":1,"cmds":[{"op":"style.apply","id":"pen","line":"stroke"}]}"#).unwrap();
    let eff_b = apply_stream(&mut doc, &apply_b).unwrap();
    let b = eval::evaluate(&doc, 0).unwrap();
    let img_b = render(&b, 160.0, 90.0, &opts());
    let hash_b = sha256_hex(&mo2d::png::encode_png(&img_b));
    assert_eq!(b[0].width, 2.0);
    assert_ne!(hash_a, hash_b, "the same line must render differently under different brush styles");

    // Undo pen → back to marker state.
    apply(&mut doc, &eff_b[0].inverse).unwrap();
    let back = eval::evaluate(&doc, 0).unwrap();
    assert_eq!(back[0].width, 12.0, "undo restores the previous style state");
    assert_eq!(back[0].nodes[0].color, [40, 90, 200, 255]);
    // Undo marker → the pristine pre-style line.
    apply(&mut doc, &eff_a[0].inverse).unwrap();
    let pristine = eval::evaluate(&doc, 0).unwrap();
    assert_eq!(pristine[0].width, 10.0, "double undo returns to the untouched line");
    assert_eq!(pristine[0].nodes[0].color, [30, 30, 30, 255]);
}

#[test]
fn brush_docs_round_trip_byte_stable() {
    let mut doc = base_doc();
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"style.create","id":"brush.marker","width":8,"brush":{"name":"chunky","jitter":0.2,"vendor.custom":{"x":1}}},
      {"op":"line.setAttrs","id":"stroke","brush":{"sizeAmp":1.2,"sizeScale":0.9,"tint":[200,200,200,255]}},
      {"op":"node.set","line":"stroke","index":1,"pr":1.5}
    ]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &cmds).unwrap();
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "brush/pressure/styles must round-trip byte-stably");
    let text = std::str::from_utf8(&b2).unwrap();
    assert!(text.contains("vendor.custom"), "unknown brush keys pass through");
    assert!(text.contains("chunky"), "brush name (external ref) carried verbatim");
    // Legacy documents (no styles) keep their exact canonical bytes: styles
    // omitted when empty.
    let legacy = base_doc();
    let lb = mo2d::format::doc_to_bytes(&legacy).unwrap();
    assert!(!std::str::from_utf8(&lb).unwrap().contains("styles"));
}

#[test]
fn validator_rejects_bad_brush_values() {
    let mut doc = base_doc();
    let cmds = parse_stream(
        r#"{"v":1,"cmds":[
      {"op":"node.set","line":"stroke","index":0,"pr":50.0},
      {"op":"style.create","id":"a","width":1},
      {"op":"style.create","id":"a","width":2}
    ]}"#,
    );
    // Second style.create with the same id must fail at command level…
    let cmds = cmds.unwrap();
    assert!(apply_stream(&mut doc, &cmds).is_err(), "duplicate style id rejected");
    // …and a bad pressure rides through commands but is flagged by the validator.
    let mut doc2 = base_doc();
    let bad = parse_stream(r#"{"v":1,"cmds":[{"op":"node.set","line":"stroke","index":0,"pr":50.0}]}"#).unwrap();
    apply_stream(&mut doc2, &bad).unwrap();
    let problems = mo2d::validate::validate(&doc2);
    assert!(problems.iter().any(|p| p.message.contains("pressure")), "validator must flag pressure 50");
    assert!(problems.is_empty() == false);
    // Time model untouched: sanity anchor.
    assert_eq!(TICKS_PER_SECOND, 254_016_000_000);
}
