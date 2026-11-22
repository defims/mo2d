//! Gate SVG (roadmap C3 "几何导入", landed 2026-11-22): drag-in / command
//! import of an SVG document → editable mo2d cubic lines wrapped as one
//! layer (invisible transform-box line + group relation). Everything is
//! ordinary mo2d data afterwards: draggable nodes, group transform, the
//! works. Third-party assets never enter the repo — committed fixtures are
//! hand-authored minimal SVGs (repo redline: fixtures are always self-made).

mod common;

use common::{doc_bytes, load_frozen, repo_root};
use mo2d::render::Camera;
use mo2d::sha::sha256_hex;
use common::render_sha;
use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::model::*;

fn doc_from(xml: &str, name: &str) -> Document {
    let text = format!(r#"{{"v":1,"cmds":[{{"op":"svg.import","xml":{:?},"name":"{}"}}]}}"#, xml, name);
    let mut d = Document::default();
    apply_stream(&mut d, &parse_stream(&text).unwrap()).unwrap();
    d
}

const H: f64 = 100.0; // fixture viewBox height (y-flip reference)

fn import_doc(xml: &str, name: &str) -> Document {
    // {:?} on a str produces JSON-compatible escaping for these ASCII fixtures
    let text = format!(
        r#"{{"v":1,"cmds":[{{"op":"svg.import","xml":{:?},"name":"{}"}}]}}"#,
        xml, name
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&text).unwrap()).unwrap();
    doc
}

#[test]
fn svg_path_imports_as_editable_curves() {
    // Hand-authored fixture: S-curve path, closed, with fill+stroke.
    let xml = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
      <path d="M10 20 L 100 20 C 130 0 160 40 180 20 Z" fill="#cc4422" stroke="blue" stroke-width="3"/>
    </svg>"##;
    let doc = import_doc(xml, "t");
    let l = doc.line("t_l1").expect("path line");
    assert!(l.closed, "Z closes the subpath");
    assert_eq!(l.nodes.len(), 3, "M+L+C = 3 nodes");
    // y-flip: SVG y=20 → world y = 100−20 = 80.
    assert!((l.nodes[0].pos.geom().unwrap().y - 80.0).abs() < 1e-9, "y-flip at import");
    // The C segment: node1 h_out = control1 − node1 = (130,0)−(100,80) — wait,
    // the y-flip mirrors the CONTROL too: SVG control1 (130,0) → world (130,100).
    // So h_out = (130−110, 100−80) = (20, 20).
    let ho = l.nodes[1].handle_out.unwrap();
    assert!((ho.x - 30.0).abs() < 1e-9 && (ho.y - 20.0).abs() < 1e-9, "cubic handle offset, got {:?}", ho);
    // Paint mapping.
    assert!(l.fill_on && l.fill_color[0] == 0xcc && l.fill_color[1] == 0x44, "hex fill parsed");
    assert_eq!(l.color[2], 255, "named color: blue");
    assert_eq!(l.width, 3.0, "stroke-width");
    // Curve segments typed.
    assert!(l.segs.iter().any(|s| s.seg_type == SegType::Curve), "C segment is Curve");
    assert!(l.segs[0].seg_type == SegType::Straight, "L segment is Straight");
}

#[test]
fn svg_shapes_import_with_flattened_transforms() {
    let xml = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
      <g transform="translate(10,10)">
        <rect x="0" y="0" width="40" height="20" fill="red"/>
        <circle cx="60" cy="30" r="10" fill="gold"/>
      </g>
    </svg>"#;
    let doc = import_doc(xml, "t");
    let r = doc.line("t_l1").expect("rect");
    // rect local (0,0) + translate(10,10) → SVG (10,10) → world (10, 90).
    let p = r.nodes[0].pos.geom().unwrap();
    assert!((p.x - 10.0).abs() < 1e-9 && (p.y - 90.0).abs() < 1e-9, "translate flattened + y-flip, got {:?}", p);
    assert!(r.fill_on && r.fill_color[0] == 255, "named fill");
    let c = doc.line("t_l2").expect("circle");
    // circle center (60,30)+translate → SVG (70,40) → world (70,60); right node = (80,60).
    let cp = c.nodes[0].pos.geom().unwrap();
    assert!((cp.x - 80.0).abs() < 1e-9 && (cp.y - 60.0).abs() < 1e-9, "circle right node, got {:?}", cp);
    assert!(c.segs.iter().all(|s| s.seg_type == SegType::Curve), "kappa circle is all curves");
}

#[test]
fn svg_import_wraps_as_one_layer() {
    let xml = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
      <path d="M10 10 L 90 10 L 50 90 Z" fill="teal"/>
      <circle cx="50" cy="50" r="5" fill="none" stroke="black"/>
    </svg>"#;
    let doc = import_doc(xml, "art");
    // The 层: one group relation, box = the invisible transform-box line.
    let rel = doc.relations.iter().find(|r| r.kind == RelationKind::Group).expect("group relation (the layer)");
    assert_eq!(rel.group_box, "art_box", "box line drives the layer");
    assert_eq!(rel.group_members, vec!["art_l1", "art_l2"], "both imported lines are members");
    let box_line = doc.line("art_box").expect("box line");
    assert!(!box_line.stroke_on && !box_line.fill_on, "transform box is invisible");
    // Moving/scaling the box transforms the layer (B4 group machinery).
    // Editability proof: node.set on an imported node works.
    let text = r#"{"v":1,"cmds":[{"op":"node.set","line":"art_l1","index":0,"p":[10,20,0]}]}"#;
    apply_stream(&mut { doc.clone() }, &parse_stream(text).unwrap()).unwrap();
}

#[test]
fn svg_arcs_become_cubics_and_text_is_skipped() {
    let xml = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
      <path d="M10 50 A 40 40 0 0 1 90 50" fill="none" stroke="red"/>
      <defs><circle cx="999" cy="999" r="5"/></defs>
      <text x="10" y="10">skip me</text>
    </svg>"#;
    let doc = import_doc(xml, "t");
    let l = doc.line("t_l1").expect("arc path line");
    // Semicircle arc (180°) → 2 cubic segments (≤90° each) → 3 nodes.
    assert_eq!(l.nodes.len(), 3);
    // Endpoints exact.
    assert!((l.nodes[0].pos.geom().unwrap().x - 10.0).abs() < 1e-9);
    assert!((l.nodes[2].pos.geom().unwrap().x - 90.0).abs() < 1e-9);
    // defs/text subtrees skipped: only 1 imported line (+ box).
    assert_eq!(doc.lines.len(), 2, "only the path line + box line");
}

#[test]
fn svg_import_is_deterministic_and_byte_stable() {
    let xml = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
      <path d="M5 5 Q 50 95 95 5" stroke="purple" fill="none" stroke-width="2"/>
      <rect x="10" y="10" width="30" height="30" fill="lime"/>
    </svg>"#;
    let d1 = import_doc(xml, "t");
    let d2 = import_doc(xml, "t");
    assert_eq!(doc_bytes(&d1), doc_bytes(&d2), "same svg → byte-identical document");
    let parsed = mo2d::format::parse_doc(&String::from_utf8(doc_bytes(&d1).clone()).unwrap()).unwrap();
    assert_eq!(doc_bytes(&parsed), doc_bytes(&d1), "round trip byte-stable");
    // elevation check: the quadratic became a cubic (2-node line).
    let q = d1.line("t_l1").expect("quadratic path");
    assert_eq!(q.nodes.len(), 2, "Q with 2 endpoints = 2 nodes");
    assert!(q.nodes[0].handle_out.is_some() && q.nodes[1].handle_in.is_some(), "quadratic elevated to cubic handles");
}

#[test]
fn svg_face_frozen_demo() {
    // The drag-in acceptance artifact: a hand-authored SVG (self-made, no
    // third-party assets) imported as one layer — editable lines + group.
    let frozen = load_frozen();
    let sec = frozen
        .raw
        .get("svgFace")
        .unwrap_or_else(|| panic!("frozen.json missing 'svgFace' (run demo/freeze.py)"));
    let want_doc = sec.get("docSha256").and_then(|v| v.as_str()).expect("docSha256").to_string();
    let want_frame = sec.get("frameSha256").and_then(|v| v.as_str()).expect("frameSha256").to_string();
    let tick = sec.get("midTick").and_then(|v| v.as_i64()).expect("midTick");
    let doc = common::replay_stream(&repo_root().join("demo/cmd/svg_face.json"));
    assert_eq!(sha256_hex(&doc_bytes(&doc)), want_doc, "svg_face: document hash drifted");
    assert_eq!(common::render_sha(&doc, tick, Camera::Ortho2d), want_frame, "svg_face: frame hash drifted");
    // The layer: group relation + box + imported lines, all editable.
    let rel = doc.relations.iter().find(|r| r.kind == RelationKind::Group).expect("layer group");
    assert_eq!(rel.group_members.len(), 5, "head + 2 eyes + smile + hair = 5 lines");
    let face = doc.line(&rel.group_members[0]).expect("head line");
    assert!(face.nodes.iter().all(|n| n.handle_out.is_some() || n.handle_in.is_some()), "head is a kappa ellipse (editable handles)");
}

#[test]
fn svg_import_rejects_garbage() {
    for xml in ["not an svg at all", "<svg viewBox='0 0 10 10'><text>no geometry</text></svg>"] {
        let text = format!(r#"{{"v":1,"cmds":[{{"op":"svg.import","xml":{:?},"name":"g"}}]}}"#, xml);
        let mut doc = Document::default();
        assert!(
            apply_stream(&mut doc, &parse_stream(&text).unwrap()).is_err(),
            "must reject: {}",
            xml
        );
    }
}
