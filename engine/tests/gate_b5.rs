//! Gate B5 — bitmap mesh (roadmap B5):
//! 1. decode-on-demand textures render through the fill (bilinear,
//!    perspective-correct) — texel colors land on screen
//! 2. acceptance: a bitmap mesh (textured grid patches) deforms with the
//!    skeleton via bind — same texture, moved geometry
//! 3. damage parity holds for textured frames; round-trip byte-stable

mod common;

use mo2d::assets::AssetStore;
use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render_with_assets, RenderOpts};
use std::path::Path;

fn opts() -> RenderOpts {
    RenderOpts { width: 240, height: 160, camera: mo2d::render::Camera::Ortho2d, samples: 1 }
}

fn demo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn textured_doc() -> Document {
    let stream = r#"{"v":1,"cmds":[
      {"op":"doc.setRoot","width":240,"height":160},
      {"op":"line.create","id":"patch","closed":true,"strokeOn":false,"fillOn":true,
       "fillColor":[255,255,255,255],"texture":"assets/checker.png","nodes":[
        {"p":[40,40,0],"uv":[0.0,0.0]},
        {"p":[200,40,0],"uv":[1.0,0.0]},
        {"p":[200,120,0],"uv":[1.0,1.0]},
        {"p":[40,120,0],"uv":[0.0,1.0]}]}
    ]}"#;
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(stream).unwrap()).unwrap();
    doc
}

fn store() -> AssetStore {
    let mut s = AssetStore::new();
    s.root = demo_root().join("demo");
    s
}

fn px(img: &mo2d::render::Image, x: u32, y: u32) -> [u8; 4] {
    let o = ((y * img.w + x) * 4) as usize;
    [img.px[o], img.px[o + 1], img.px[o + 2], img.px[o + 3]]
}

#[test]
fn textured_fill_samples_texel_colors() {
    let doc = textured_doc();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let mut assets = store();
    let img = render_with_assets(&lines, 240.0, 160.0, &opts(), &mut assets);
    assert!(assets.errors.is_empty(), "checker.png must load: {:?}", assets.errors);
    // The patch maps world [40..200]×[40..120] onto the full texture (uv
    // 0..1). World y is up, screen y is down: v = (160 − sy − 0.5 − 40)/80.
    // Expected color at each screen-pixel center = bilinear over the decoded
    // texture at the linearly interpolated uv (ortho2d: uv-linear in screen).
    let tex = mo2d::png::decode_png(&std::fs::read(demo_root().join("demo/assets/checker.png")).unwrap()).unwrap();
    assert_eq!((tex.w, tex.h), (128, 128));
    let bilinear = |u: f64, v: f64| -> [f64; 4] {
        let x = u.clamp(0.0, 1.0) * (tex.w as f64 - 1.0);
        let y = v.clamp(0.0, 1.0) * (tex.h as f64 - 1.0);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let x0 = (x0 as u32).min(tex.w - 1);
        let y0 = (y0 as u32).min(tex.h - 1);
        let x1 = (x0 + 1).min(tex.w - 1);
        let y1 = (y0 + 1).min(tex.h - 1);
        let at = |xx: u32, yy: u32| -> [f64; 4] {
            let o = ((yy * tex.w + xx) * 4) as usize;
            [tex.px[o] as f64, tex.px[o + 1] as f64, tex.px[o + 2] as f64, tex.px[o + 3] as f64]
        };
        let mut out = [0.0f64; 4];
        for ch in 0..4 {
            let top = at(x0, y0)[ch] + (at(x1, y0)[ch] - at(x0, y0)[ch]) * fx;
            let bot = at(x0, y1)[ch] + (at(x1, y1)[ch] - at(x0, y1)[ch]) * fx;
            out[ch] = top + (bot - top) * fy;
        }
        out
    };
    for (sx, sy) in [(53u32, 46u32), (85u32, 78u32), (149u32, 102u32), (120u32, 80u32)] {
        let u = (sx as f64 + 0.5 - 40.0) / 160.0;
        let v = (160.0 - sy as f64 - 0.5 - 40.0) / 80.0;
        let want = bilinear(u, v);
        let got = px(&img, sx, sy);
        for ch in 0..4 {
            assert!(
                (f64::from(got[ch]) - want[ch]).abs() <= 3.0,
                "screen ({}, {}) channel {}: got {:?} want bilinear {:?}",
                sx,
                sy,
                ch,
                got,
                want
            );
        }
    }
    // Without the store, the textured line falls back to its flat fill
    // (white) — deterministic degradation.
    let plain = mo2d::render::render(&lines, 240.0, 160.0, &opts());
    let center = px(&plain, 120, 80);
    assert!(center.iter().all(|&b| b == 255), "flat fallback must be white: {:?}", center);
}

#[test]
fn bitmap_mesh_deforms_with_skeleton() {
    let mut doc = textured_doc();
    // Skeleton + bind: the patch rides the skeleton rigidly (falloff 0).
    let stream = format!(
        r#"{{"v":1,"cmds":[
      {{"op":"line.create","id":"skel","tags":["skeleton"],"width":2,"strokeOn":false,"nodes":[
        {{"p":[40,80,0]}},{{"p":[100,80,0]}},{{"p":[160,80,0]}},{{"p":[220,80,0]}}]}},
      {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":[220,80,0]}},{{"p":[{}],"v":[220,20,0]}}]}},
      {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"skel","node":3,"attr":"position"}}}},
      {{"op":"relation.create","type":"bind","target":"patch","skeleton":"skel"}}
    ]}}"#,
        TICKS_PER_SECOND
    );
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let mut assets = store();
    let rest = eval::evaluate(&doc, 0).unwrap();
    let moved = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    // Geometry moved: the corner riding skeleton segment 2 (nearest to rest
    // (200,120)) is dragged by the animated bone end (220,80)→(220,20).
    let p0 = rest.iter().find(|l| l.id == "patch").unwrap();
    let p1 = moved.iter().find(|l| l.id == "patch").unwrap();
    assert_ne!(p0.nodes[2].p, p1.nodes[2].p, "the bitmap mesh must deform with the skeleton");
    // The far corner (rest (40,120), riding the unmoved segment 0) stays.
    assert_eq!(p1.nodes[3].p, mo2d::math::Vec3::new(40.0, 120.0, 0.0), "corners on unmoved bones stay put");
    // The deformed frame still samples the texture (interior not flat).
    let img = render_with_assets(&moved, 240.0, 160.0, &opts(), &mut assets);
    let center = px(&img, 120, 60);
    assert!(
        !(center[0] == 255 && center[1] == 255 && center[2] == 255 && center[3] == 255),
        "deformed mesh interior must show texture, got {:?}",
        center
    );
    // Damage parity with textures across the animation.
    let o = opts();
    let mut prev = None;
    let mut prev_lines = rest;
    for s in 0..=4 {
        let tick = TICKS_PER_SECOND * s as i64 / 4;
        let lines = eval::evaluate(&doc, tick).unwrap();
        let full = render_with_assets(&lines, 240.0, 160.0, &o, &mut assets);
        if let Some(ref p) = prev {
            let (dmg, _) = mo2d::render::render_damaged(p, &prev_lines, &lines, 240.0, 160.0, &o, &mut assets);
            assert_eq!(dmg.px, full.px, "textured frames must damage-render byte-identically (step {})", s);
        }
        prev = Some(full);
        prev_lines = lines;
    }
}

#[test]
fn textured_docs_round_trip_byte_stable() {
    let doc = textured_doc();
    let b1 = mo2d::format::doc_to_bytes(&doc).unwrap();
    let doc2 = mo2d::format::parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
    let b2 = mo2d::format::doc_to_bytes(&doc2).unwrap();
    assert_eq!(b1, b2, "texture + uv attrs round-trip byte-stably");
    let text = std::str::from_utf8(&b2).unwrap();
    assert!(text.contains("\"texture\": \"assets/checker.png\""), "texture ref carried verbatim");
    assert!(text.contains("\"uv\""));
    // The referenced file exists where the document says it does.
    assert!(Path::exists(&demo_root().join("demo/assets/checker.png")));
}
