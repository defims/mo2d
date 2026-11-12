//! Gate A2 — node sharing + skeleton + binding (mvp.md §3).
//!
//! Deformation comes from RELATIONS (tracks animating the skeleton), because
//! the rest state *is* the document: a polyline skeleton driven by tracks
//! deforms a bound body; an unbound negative control stays frozen; shared
//! nodes stay welded.

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::math::Vec3;
use mo2d::model::Document;

const TPS: i64 = 254_016_000_000;

/// skel (4 nodes) + body bound to it + unbound control, and a track that
/// lifts skeleton joint 3 from y=0 to y=`lift` at tick TPS.
fn base_doc(lift: f64, falloff: f64) -> Document {
    let text = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"skel","tags":["skeleton"],"width":2,"color":[255,170,60,255],
           "nodes":[{{"p":[0,0,0]}},{{"p":[50,0,0]}},{{"p":[100,0,0]}},{{"p":[150,0,0]}}]}},
          {{"op":"line.create","id":"body","color":[16,84,96,255],"width":6,
           "nodes":[{{"p":[0,10,0],"w":20}},{{"p":[50,10,0],"w":16}},{{"p":[100,10,0],"w":10}},{{"p":[150,10,0],"w":6}}]}},
          {{"op":"line.create","id":"control","color":[120,120,120,255],"width":4,
           "nodes":[{{"p":[0,40,0]}},{{"p":[50,40,0]}},{{"p":[100,40,0]}},{{"p":[150,40,0]}}]}},
          {{"op":"relation.create","id":"bind_body","type":"bind","target":"body","skeleton":"skel","falloff":{},"stretch":true}},
          {{"op":"line.create","id":"trk_skel_3_position","tags":["track"],
           "nodes":[{{"p":[0],"v":[150,0,0]}},{{"p":[{}],"v":[150,{},0]}}]}},
          {{"op":"relation.create","id":"rel","type":"track","trackLine":"trk_skel_3_position",
           "target":{{"line":"skel","node":3,"attr":"position"}}}}
        ]}}"#,
        falloff, TPS, lift
    );
    let cmds = parse_stream(&text).unwrap();
    let mut doc = Document::default();
    apply_stream(&mut doc, &cmds).unwrap();
    doc
}

#[test]
fn a2_skeleton_drives_bound_body() {
    let doc = base_doc(60.0, 0.0);
    let at0 = mo2d::eval::evaluate(&doc, 0).unwrap();
    let at1 = mo2d::eval::evaluate(&doc, TPS).unwrap();

    let body0 = at0.iter().find(|l| l.id == "body").unwrap();
    let body1 = at1.iter().find(|l| l.id == "body").unwrap();
    let control0 = at0.iter().find(|l| l.id == "control").unwrap();
    let control1 = at1.iter().find(|l| l.id == "control").unwrap();

    // Rest: body sits at y=10 above the skeleton.
    assert_eq!(body0.nodes[3].p.y, 10.0);
    // Track lifts the bone end to y=60 → the bound body node rides along.
    assert!(
        body1.nodes[3].p.y > 40.0,
        "bound body must follow the animated bone end (y={})",
        body1.nodes[3].p.y
    );
    // Unbound negative control: bit-identical between ticks.
    for (a, b) in control0.nodes.iter().zip(control1.nodes.iter()) {
        assert_eq!(a.p, b.p, "unbound control must not deform");
    }
    assert_eq!(control1.nodes[3].p, Vec3::new(150.0, 40.0, 0.0));
}

#[test]
fn a2_unbound_negative_control_frozen() {
    let doc = base_doc(120.0, 0.0);
    // Whatever the skeleton does, the control line's evaluated positions are
    // exactly its rest positions at every tick.
    for frac in [0i64, TPS / 3, TPS / 2, TPS] {
        let lines = mo2d::eval::evaluate(&doc, frac).unwrap();
        let control = lines.iter().find(|l| l.id == "control").unwrap();
        let rest_x: Vec<f64> = vec![0.0, 50.0, 100.0, 150.0];
        for (i, n) in control.nodes.iter().enumerate() {
            assert_eq!((n.p.x, n.p.y), (rest_x[i], 40.0), "control node {} at tick {}", i, frac);
        }
    }
}

#[test]
fn a2_node_sharing_welds_joints() {
    // A fork polyline shares the skeleton tip node: wherever the tip goes
    // (track-driven), the fork's first node is welded to it.
    let mut doc = base_doc(60.0, 0.0);
    let text = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"fork","width":2,"color":[255,170,60,255],
       "nodes":[{"p":[150,0,0]},{"p":[200,-30,0]}]},
      {"op":"relation.create","id":"share","type":"share",
       "driver":{"line":"skel","node":3},"driven":{"line":"fork","node":0}}
    ]}"#;
    apply_stream(&mut doc, &parse_stream(text).unwrap()).unwrap();

    let lines = mo2d::eval::evaluate(&doc, TPS).unwrap();
    let skel = lines.iter().find(|l| l.id == "skel").unwrap();
    let fork = lines.iter().find(|l| l.id == "fork").unwrap();
    // The track drives the skeleton tip to (150,60)…
    assert_eq!(skel.nodes[3].p, Vec3::new(150.0, 60.0, 0.0));
    // …and the shared fork node is welded to it.
    assert_eq!(fork.nodes[0].p, skel.nodes[3].p, "shared node must be welded");
    // The fork's far node is independent (no relation touches it).
    assert_eq!(fork.nodes[1].p, Vec3::new(200.0, -30.0, 0.0));
}

#[test]
fn a2_bind_falloff_linear_weights() {
    // Body node 0 rests 10 units from bone 0 (skel nodes 0–1). Animate the
    // FAR END of that bone (joint 1) so the node's attach segment actually
    // moves. With falloff F=40 the follow weight is w = 1 − d/F = 0.75:
    // soft = rest + (rigid − rest) · 0.75, component-exact.
    let build = |falloff: f64| {
        let text = format!(
            r#"{{"v":1,"cmds":[
              {{"op":"line.create","id":"skel","tags":["skeleton"],"width":2,
               "nodes":[{{"p":[0,0,0]}},{{"p":[50,0,0]}},{{"p":[100,0,0]}}]}},
              {{"op":"line.create","id":"body","width":6,
               "nodes":[{{"p":[0,10,0],"w":20}},{{"p":[50,10,0],"w":10}},{{"p":[100,10,0],"w":6}}]}},
              {{"op":"relation.create","type":"bind","target":"body","skeleton":"skel","falloff":{},"stretch":false}},
              {{"op":"line.create","id":"trk_skel_1_position","tags":["track"],
               "nodes":[{{"p":[0],"v":[50,0,0]}},{{"p":[{}],"v":[50,100,0]}}]}},
              {{"op":"relation.create","type":"track","trackLine":"trk_skel_1_position",
               "target":{{"line":"skel","node":1,"attr":"position"}}}}
            ]}}"#,
            falloff, TPS
        );
        let mut doc = Document::default();
        apply_stream(&mut doc, &parse_stream(&text).unwrap()).unwrap();
        doc
    };
    let rigid = build(0.0);
    let soft = build(40.0);
    let r = mo2d::eval::evaluate(&rigid, TPS).unwrap();
    let s = mo2d::eval::evaluate(&soft, TPS).unwrap();
    let rigid_node = r.iter().find(|l| l.id == "body").unwrap().nodes[0].p;
    let soft_node = s.iter().find(|l| l.id == "body").unwrap().nodes[0].p;
    let rest = Vec3::new(0.0, 10.0, 0.0);

    // The rigid follow must actually move the node, or the test is vacuous.
    assert!(
        (rigid_node.x - rest.x).abs() > 5.0 || (rigid_node.y - rest.y).abs() > 5.0,
        "rigid follow must move the node (got {:?})",
        rigid_node
    );
    let expected = Vec3::lerp(rest, rigid_node, 0.75);
    assert!(
        (soft_node.x - expected.x).abs() < 1e-9 && (soft_node.y - expected.y).abs() < 1e-9,
        "linear falloff weight: expected {:?} (w=0.75), got {:?}",
        expected,
        soft_node
    );
}
