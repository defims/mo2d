//! Gate A5 — format + export + offline loop (mvp.md §3).
//!
//! Round-trip is byte-stable; unknown keys pass through untouched; and the
//! "capability is data" gate: with NO session anywhere (fresh process, no AI,
//! no stage), open → play → export produces per-frame hashes identical to the
//! online path (completion definitions 3 and 4).

mod common;

use mo2d::format;
use mo2d::render::Camera;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn a5_round_trip_byte_equal() {
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let b1 = common::doc_bytes(&doc);
    let doc2 = format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = common::doc_bytes(&doc2);
    assert_eq!(b1, b2, "save→load→save must be byte-identical");

    // Same via disk + atomic write.
    let dir = std::env::temp_dir().join(format!("mo2d-a5-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rt.mo2d");
    format::save_doc(&path, &doc).unwrap();
    let from_disk = std::fs::read(&path).unwrap();
    assert_eq!(from_disk, b1);
    let loaded = format::load_doc(&path).unwrap();
    format::save_doc(&path, &loaded).unwrap();
    let again = std::fs::read(&path).unwrap();
    assert_eq!(again, b1, "disk round-trip must be byte-stable too");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a5_unknown_keys_pass_through() {
    let text = r#"{
  "format": "mo2d", "version": 1, "minReader": 1, "generator": "mo2d 0.1.0",
  "root": {"width": 640, "height": 360, "ticksPerSecond": 254016000000, "fps": 24.0, "vendor.thing": [1, 2, {"deep": true}]},
  "lines": [{"id": "g", "closed": false, "width": 4.0, "color": [0,0,0,255], "fillColor": [255,255,255,255], "strokeOn": true, "fillOn": false,
             "nodes": [{"p": [0,0,0]}, {"p": [9,9,0], "vendor.nodeExtra": "keep-me"}],
             "segs": [{"type": "straight", "visible": true, "vendor.segExtra": 7}]}],
  "relations": [],
  "vendor.top": {"anything": "goes"}
}"#;
    let doc = format::parse_doc(text).unwrap();
    assert!(!doc.unknown.is_empty(), "top-level unknown key preserved");
    assert!(!doc.root.unknown.is_empty(), "root unknown key preserved");
    assert!(!doc.lines[0].nodes[1].unknown.is_empty(), "node unknown key preserved");
    assert!(!doc.lines[0].segs[0].unknown.is_empty(), "segment unknown key preserved");
    let b1 = common::doc_bytes(&doc);
    let doc2 = format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = common::doc_bytes(&doc2);
    assert_eq!(b1, b2, "unknown keys must survive round-trips stably");
    let text2 = std::str::from_utf8(&b2).unwrap();
    for needle in ["vendor.thing", "vendor.nodeExtra", "vendor.segExtra", "vendor.top"] {
        assert!(text2.contains(needle), "{} must survive the round-trip", needle);
    }
}

#[test]
fn a5_export_manifest_matches_online() {
    // In-process export (the "online" path — built through commands this session).
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let dir = std::env::temp_dir().join(format!("mo2d-a5-online-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let tps = doc.root.ticks_per_second;
    let opts = mo2d::export::ExportOpts {
        from_tick: 0,
        to_tick: tps - 1,
        step: tps / 24,
        camera: Camera::Ortho2d,
        width: None,
        height: None,
    };
    let online = mo2d::export::export_frames(&doc, &opts, &dir).unwrap();

    // Fresh PROCESS, no session state at all: CLI reads the saved file and
    // exports the same range — "close every session; the document still plays".
    let dir2 = std::env::temp_dir().join(format!("mo2d-a5-offline-{}", std::process::id()));
    let saved = dir.join("saved.mo2d");
    format::save_doc(&saved, &doc).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mo2d"))
        .args(["export", "--doc"])
        .arg(&saved)
        .arg("--dir")
        .arg(&dir2)
        .arg("--from")
        .arg("0")
        .arg("--to")
        .arg(&(tps - 1).to_string())
        .arg("--step")
        .arg(&(tps / 24).to_string())
        .output()
        .expect("run offline export");
    assert!(out.status.success(), "offline export failed: {}", String::from_utf8_lossy(&out.stderr));

    let online_manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let offline_manifest = std::fs::read_to_string(dir2.join("manifest.json")).unwrap();
    assert_eq!(online_manifest, offline_manifest, "online and offline manifests must be byte-identical");

    // And every frame file matches.
    assert_eq!(online.frames.len(), 24);
    for (name, sha) in &online.frames {
        let bytes = std::fs::read(dir2.join(name)).unwrap();
        assert_eq!(&mo2d::sha::sha256_hex(&bytes), sha, "frame {} must match offline", name);
    }
    std::fs::remove_dir_all(&dir).ok();
    std::fs::remove_dir_all(&dir2).ok();
}

#[test]
fn a5_gif_export_deterministic() {
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let dir: PathBuf = std::env::temp_dir().join(format!("mo2d-a5-gif-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let tps = doc.root.ticks_per_second;
    let opts = mo2d::export::ExportOpts {
        from_tick: 0,
        to_tick: tps / 2,
        step: tps / 24,
        camera: Camera::Ortho2d,
        width: None,
        height: None,
    };
    let a = dir.join("a.gif");
    let b = dir.join("b.gif");
    mo2d::export::export_gif(&doc, &opts, &a).unwrap();
    mo2d::export::export_gif(&doc, &opts, &b).unwrap();
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap(), "GIF encoding must be deterministic");
    assert!(std::fs::metadata(&a).unwrap().len() > 1000);
    std::fs::remove_dir_all(&dir).ok();
}
