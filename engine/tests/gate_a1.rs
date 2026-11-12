//! Gate A1 — line kernel + offscreen render, 3D from day one (mvp.md §3).
//!
//! 1. Command stream builds the variable-width tentacle line; rendering is
//!    compared against the frozen reference hash (triangle strip + nonzero
//!    fill; hash-freeze per P1-7 after human eyeballing).
//! 2. 3D static scene gate: command stream builds a z≠0 scene of triangle
//!    faces + wireframe; perspective render hash frozen — "2D = 3D special
//!    case" pinned by test (completion definition 8).
//! 3. Render determinism: same doc + tick + camera → identical PNG bytes.

mod common;

use common::serde_less::Value;
use mo2d::render::Camera;

fn frozen_get(root: &str, key: &str) -> String {
    let frozen = common::load_frozen();
    frozen
        .raw
        .get(root)
        .and_then(|v| v.get(key))
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("frozen.json missing {}.{}", root, key))
        .to_string()
}

#[test]
fn a1_tentacle_render_frozen() {
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let doc_sha = mo2d::sha::sha256_hex(&common::doc_bytes(&doc));
    assert_eq!(&doc_sha, &frozen_get("tentacle", "docSha256"), "replaying the frozen stream must reproduce the frozen document hash");

    // Mid-wave frame (the visually inspected reference frame).
    let tps = doc.root.ticks_per_second;
    assert_eq!(
        common::render_sha(&doc, tps / 2, Camera::Ortho2d),
        frozen_get("tentacle", "midFrameSha256"),
        "tentacle mid-wave render must match the frozen reference (regression in eval or renderer)"
    );
}

#[test]
fn a1_scene3d_static_frozen() {
    let doc = common::replay_stream(&common::demo_dir().join("cmd/scene3d.json"));

    // z ≠ 0 is the point of the gate: assert the geometry actually uses z.
    let z_nonzero = doc
        .lines
        .iter()
        .any(|l| l.nodes.iter().any(|n| matches!(n.pos, mo2d::model::NodePos::Geom(v) if v.z.abs() > 1e-9)));
    assert!(z_nonzero, "the 3D scene must contain z≠0 nodes (this is the 2D=3D-special-case gate)");

    assert_eq!(
        common::render_sha(&doc, 0, common::scene3d_camera()),
        frozen_get("scene3d", "frameSha256"),
        "3D static scene render must match the frozen perspective-camera hash (backface culling + z-buffer evidence)"
    );
}

#[test]
fn a1_render_deterministic() {
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let tps = doc.root.ticks_per_second;
    for tick in [0, tps / 3, tps / 2, tps] {
        let a = common::render_sha(&doc, tick, Camera::Ortho2d);
        let b = common::render_sha(&doc, tick, Camera::Ortho2d);
        assert_eq!(a, b, "render at tick {} must be byte-reproducible", tick);
    }
    let doc3 = common::replay_stream(&common::demo_dir().join("cmd/scene3d.json"));
    let a = common::render_sha(&doc3, 0, common::scene3d_camera());
    let b = common::render_sha(&doc3, 0, common::scene3d_camera());
    assert_eq!(a, b, "3D render must be byte-reproducible");
}

#[test]
fn a1_variable_width_triangle_strip_present() {
    // The tentacle body must actually be variable-width: node widths span a
    // range, and rendering with widths vs a uniform width must differ.
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let body = doc.line("body").unwrap();
    let widths: Vec<f64> = body.nodes.iter().map(|n| n.eff_width(body.width)).collect();
    let min = widths.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = widths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(max / min > 3.0, "tentacle must taper (widths {}..{})", min, max);

    let sha_tapered = common::render_sha(&doc, 0, Camera::Ortho2d);
    // Same doc but uniform widths on the body → different image.
    let mut doc2 = doc.clone();
    let body2 = doc2.line_mut("body").unwrap();
    for n in &mut body2.nodes {
        n.width = Some(12.0);
    }
    let sha_uniform = common::render_sha(&doc2, 0, Camera::Ortho2d);
    assert_ne!(sha_tapered, sha_uniform, "tapered and uniform-width renders must differ");
}
