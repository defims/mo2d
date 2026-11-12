//! Gate B-demos — the Phase B acceptance demos pinned by demo/frozen.json
//! (hash freeze + human eyeball policy, same as the A1 gates):
//! - brush:    B2 — one line family, two styles; pressure-sensitive width
//! - gradient: B3 — color-changing tentacle (gradient stroke + color track)
//! - groups:   B4 — two characters on one stage driven by a transform box
//! - sim:      B6 — physically trailing tail + seeded particles

mod common;

use common::{demo_dir, doc_bytes, load_frozen, replay_stream, repo_root};
use mo2d::render::{Camera, RenderOpts};
use mo2d::sha::sha256_hex;

fn frozen_section(key: &str) -> (String, String, i64) {
    let frozen = load_frozen();
    let sec = frozen.raw.get(key).unwrap_or_else(|| panic!("frozen.json missing '{}' (run demo/freeze.py)", key));
    let doc = sec.get("docSha256").and_then(|v| v.as_str()).expect("docSha256").to_string();
    let frame = sec.get("frameSha256").and_then(|v| v.as_str()).expect("frameSha256").to_string();
    let tick = sec.get("midTick").and_then(|v| v.as_i64()).expect("midTick");
    (doc, frame, tick)
}

fn check_demo(key: &str, stream: &str) {
    let (want_doc, want_frame, tick) = frozen_section(key);
    let doc = replay_stream(&repo_root().join("demo/cmd").join(stream));
    let got_doc = sha256_hex(&doc_bytes(&doc));
    assert_eq!(got_doc, want_doc, "{}: replayed document hash drifted from frozen.json", key);
    let lines = mo2d::eval::evaluate(&doc, tick).expect("evaluate");
    // The frozen documents live in demo/out — B5 texture references resolve
    // against that directory (project-directory form: assets/ alongside).
    let mut assets = mo2d::assets::AssetStore::new();
    assets.root = repo_root().join("demo/out");
    let img = mo2d::render::render_with_assets(
        &lines,
        doc.root.width as f64,
        doc.root.height as f64,
        &RenderOpts { width: doc.root.width, height: doc.root.height, camera: Camera::Ortho2d, samples: 1 },
        &mut assets,
    );
    assert!(assets.errors.is_empty(), "{}: texture load errors: {:?}", key, assets.errors);
    let got_frame = sha256_hex(&mo2d::png::encode_png(&img));
    assert_eq!(got_frame, want_frame, "{}: rendered frame hash drifted from frozen.json", key);
}

#[test]
fn brush_demo_frozen() {
    check_demo("brush", "brush-styles.json");
}

#[test]
fn gradient_demo_frozen() {
    check_demo("gradient", "gradient-tentacle.json");
}

#[test]
fn groups_demo_frozen() {
    check_demo("groups", "stage-groups.json");
}

#[test]
fn sim_demo_frozen() {
    check_demo("sim", "sim-tail.json");
}

#[test]
fn mesh_demo_frozen_both_container_forms() {
    // B5+B7: the media project is frozen in BOTH container forms —
    // demo/out/mesh-project/ (document.mo2d.json + assets/) and the
    // single-file package demo/out/mesh-package.mo2d. Both must render the
    // identical frame, and the package hash must match frozen.json.
    let (want_doc, want_frame, tick) = frozen_section("mesh");
    let frozen = load_frozen();
    let want_pkg = frozen
        .raw
        .get("mesh")
        .and_then(|m| m.get("packageSha256"))
        .and_then(|v| v.as_str())
        .expect("packageSha256")
        .to_string();

    let doc = mo2d::format::load_doc(&demo_dir().join("out/mesh-project/document.mo2d.json"))
        .expect("project-directory document loads");
    assert_eq!(sha256_hex(&doc_bytes(&doc)), want_doc, "project-directory doc hash");
    let mut assets = mo2d::assets::AssetStore::new();
    assets.root = repo_root().join("demo/out/mesh-project");
    let lines = mo2d::eval::evaluate(&doc, tick).expect("evaluate");
    let img = mo2d::render::render_with_assets(
        &lines,
        doc.root.width as f64,
        doc.root.height as f64,
        &RenderOpts { width: doc.root.width, height: doc.root.height, camera: Camera::Ortho2d, samples: 1 },
        &mut assets,
    );
    assert!(assets.errors.is_empty(), "texture load errors: {:?}", assets.errors);
    assert_eq!(sha256_hex(&mo2d::png::encode_png(&img)), want_frame, "directory-form frame hash");

    // The single-file package: hash pinned, and its embedded blob renders the
    // same frame with NO filesystem assets present.
    let pkg_path = demo_dir().join("out/mesh-package.mo2d");
    let pkg_bytes = std::fs::read(&pkg_path).expect("package present");
    assert_eq!(sha256_hex(&pkg_bytes), want_pkg, "package hash");
    let loaded = mo2d::format::load_doc_full(&pkg_path).expect("package loads");
    assert_eq!(loaded.blobs.len(), 1, "the texture rides as a blob");
    let mut pkg_assets = mo2d::assets::AssetStore::new();
    pkg_assets.root = repo_root().join("demo/definitely-not-a-real-dir");
    pkg_assets.attach_blobs(&loaded.blobs);
    let lines2 = mo2d::eval::evaluate(&loaded.doc, tick).expect("evaluate package doc");
    let img2 = mo2d::render::render_with_assets(
        &lines2,
        loaded.doc.root.width as f64,
        loaded.doc.root.height as f64,
        &RenderOpts { width: loaded.doc.root.width, height: loaded.doc.root.height, camera: Camera::Ortho2d, samples: 1 },
        &mut pkg_assets,
    );
    assert!(pkg_assets.errors.is_empty(), "blob texture must resolve: {:?}", pkg_assets.errors);
    assert_eq!(
        sha256_hex(&mo2d::png::encode_png(&img2)),
        want_frame,
        "the single-file package renders byte-identically to the directory form"
    );
}

#[test]
fn frozen_file_carries_all_b_sections() {
    let frozen = load_frozen();
    for key in ["tentacle", "scene3d", "brush", "gradient", "groups", "sim", "mesh"] {
        assert!(frozen.raw.get(key).is_some(), "frozen.json missing section '{}'", key);
    }
    let _ = demo_dir();
}
