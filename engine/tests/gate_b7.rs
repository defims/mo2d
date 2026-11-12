//! Gate B7 — format formalization (roadmap B7):
//! 1. a media-bearing project delivered as a SINGLE .mo2d file: the package
//!    embeds its texture as a blob; render resolves it with no assets dir
//! 2. unpack/pack round-trip lossless: package → directory → package is
//!    byte-identical, and both containers load to the same document
//! 3. old versions still readable: v1 documents written before the Phase B
//!    vocabulary (no styles/texture keys) load unchanged; the migration
//!    chain is wired for future bumps
//! 4. the v1 JSON Schema is emitted deterministically

mod common;

use mo2d::assets::AssetStore;
use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::format::{doc_to_package_bytes, load_doc_full, BLOB_MARKER};
use mo2d::render::{render_with_assets, RenderOpts};
use std::path::Path;

fn demo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// The mesh demo document + its texture as a blob: a media-bearing project.
fn media_project() -> (mo2d::model::Document, Vec<(String, Vec<u8>)>) {
    let stream = std::fs::read_to_string(demo_root().join("demo/cmd/mesh-bitmap.json")).unwrap();
    let mut doc = mo2d::model::Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let tex = std::fs::read(demo_root().join("demo/assets/checker.png")).unwrap();
    (doc, vec![("assets/checker.png".to_string(), tex)])
}

#[test]
fn media_project_renders_from_a_single_file() {
    let (doc, blobs) = media_project();
    let pkg = doc_to_package_bytes(&doc, &blobs).unwrap();
    // Package structure sanity: header ends with the marker line.
    let text_end = pkg.windows(BLOB_MARKER.len() + 2).position(|w| &w[..] == format!("\n{}\n", BLOB_MARKER).into_bytes().as_slice());
    assert!(text_end.is_some(), "package must carry the marker line");

    // Reload from bytes (simulating a host loading the single file) and
    // render — the texture must come from the blob area, no filesystem.
    let tmp = std::env::temp_dir().join("mo2d-b7-mesh.mo2d");
    mo2d::format::write_atomic(&tmp, &pkg).unwrap();
    let loaded = load_doc_full(&tmp).unwrap();
    assert_eq!(loaded.blobs.len(), 1);
    assert_eq!(loaded.blobs[0].0, "assets/checker.png");

    let lines = eval::evaluate(&loaded.doc, mo2d::model::TICKS_PER_SECOND / 2).unwrap();
    let mut assets = AssetStore::new();
    assets.root = std::env::temp_dir().join("mo2d-no-such-assets-dir");
    assets.attach_blobs(&loaded.blobs);
    let img = render_with_assets(
        &lines,
        loaded.doc.root.width as f64,
        loaded.doc.root.height as f64,
        &RenderOpts { width: loaded.doc.root.width, height: loaded.doc.root.height, camera: Default::default(), samples: 1 },
        &mut assets,
    );
    assert!(assets.errors.is_empty(), "texture must resolve from the blob area: {:?}", assets.errors);
    // The checker colors actually landed (non-white, multi-color render).
    let mut colors = std::collections::BTreeSet::new();
    for y in (0..img.h).step_by(7) {
        for x in (0..img.w).step_by(7) {
            let o = ((y * img.w + x) * 4) as usize;
            colors.insert([img.px[o], img.px[o + 1], img.px[o + 2]]);
        }
    }
    assert!(colors.len() > 8, "textured mesh must show many colors, got {:?}", colors);
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn unpack_pack_round_trip_is_lossless() {
    let (doc, blobs) = media_project();
    let pkg = doc_to_package_bytes(&doc, &blobs).unwrap();
    let tmp_pkg = std::env::temp_dir().join("mo2d-b7-roundtrip.mo2d");
    let dir = std::env::temp_dir().join("mo2d-b7-dir");
    let dir2 = std::env::temp_dir().join("mo2d-b7-dir2");
    mo2d::format::write_atomic(&tmp_pkg, &pkg).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);

    // unpack via the same code path as the CLI (library-level mirror of
    // `mo2d unpack`): header → document.mo2d.json, blobs → assets/.
    let loaded = load_doc_full(&tmp_pkg).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("document.mo2d.json"), mo2d::format::doc_to_bytes(&loaded.doc).unwrap()).unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    for (name, data) in &loaded.blobs {
        let rel = name.strip_prefix("assets/").unwrap_or(name);
        std::fs::write(dir.join("assets").join(rel), data).unwrap();
    }

    // pack again (mirror of `mo2d pack`): canonical header + sorted assets.
    let header = mo2d::format::doc_to_bytes(&loaded.doc).unwrap();
    let mut names: Vec<String> = std::fs::read_dir(dir.join("assets"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut blobs2: Vec<(String, Vec<u8>)> = names
        .into_iter()
        .map(|n| (format!("assets/{}", n), std::fs::read(dir.join("assets").join(n)).unwrap()))
        .collect();
    blobs2.sort_by(|a, b| a.0.cmp(&b.0));
    let pkg2 = doc_to_package_bytes(&loaded.doc, &blobs2).unwrap();

    assert_eq!(pkg, pkg2, "pack ∘ unpack ∘ pack must be byte-identical (lossless)");
    // Both containers load to the same document bytes.
    let from_dir = mo2d::format::parse_doc(&std::fs::read_to_string(dir.join("document.mo2d.json")).unwrap()).unwrap();
    assert_eq!(
        mo2d::format::doc_to_bytes(&from_dir).unwrap(),
        mo2d::format::doc_to_bytes(&loaded.doc).unwrap(),
        "directory form and package form are the same document"
    );
    let _ = std::fs::remove_file(&tmp_pkg);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn package_bytes_and_blob_errors() {
    let (doc, blobs) = media_project();
    // Deterministic: same inputs → identical package bytes.
    let a = doc_to_package_bytes(&doc, &blobs).unwrap();
    let b = doc_to_package_bytes(&doc, &blobs).unwrap();
    assert_eq!(a, b);
    // A package without blobs is just the canonical document bytes.
    let plain = doc_to_package_bytes(&doc, &[]).unwrap();
    assert_eq!(plain, mo2d::format::doc_to_bytes(&doc).unwrap());
    // Truncated blob payloads are rejected.
    let mut truncated = a.clone();
    truncated.truncate(truncated.len() - 10);
    let tmp = std::env::temp_dir().join("mo2d-b7-trunc.mo2d");
    mo2d::format::write_atomic(&tmp, &truncated).unwrap();
    assert!(load_doc_full(&tmp).is_err(), "truncated blob must be rejected");
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn old_versions_still_readable_and_migration_chain_wired() {
    // (a) A pre-Phase-B document (no styles/brush/texture keys) — the frozen
    // MVP tentacle document — loads byte-stably today.
    let tentacle = demo_root().join("demo/out/tentacle.mo2d");
    let loaded = mo2d::format::load_doc(&tentacle).expect("pre-B documents must stay readable");
    let b1 = mo2d::format::doc_to_bytes(&loaded).unwrap();
    let re = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    assert_eq!(b1, mo2d::format::doc_to_bytes(&re).unwrap());

    // (b) Future versions are refused, never guessed (existing gate, kept here
    // so the migration contract stays pinned next to the chain).
    let future = r#"{"format":"mo2d","version":99,"root":{},"lines":[],"relations":[]}"#;
    assert!(mo2d::format::parse_doc(future).is_err());

    // (c) v1 documents pass through the (currently empty) migration chain
    // unchanged — wire is exercised, semantics preserved.
    let current = r#"{"format":"mo2d","version":1,"root":{"width":10,"height":10,"ticksPerSecond":254016000000,"fps":24.0},"lines":[],"relations":[]}"#;
    let doc = mo2d::format::parse_doc(current).unwrap();
    assert_eq!(doc.root.width, 10);
}

#[test]
fn schema_emission_deterministic_and_complete() {
    let s1 = mo2d::schema::schema_v1();
    let s2 = mo2d::schema::schema_v1();
    assert_eq!(s1, s2, "schema emission must be deterministic");
    for needle in ["gradient", "group", "sim", "particles", "texture", "pr", "uv", "styles", "brush"] {
        assert!(s1.contains(needle), "schema must cover Phase B vocabulary key '{}'", needle);
    }
    assert!(Path::new(&demo_root()).exists());
}
