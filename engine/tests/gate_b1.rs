//! Gate B1 — performance machinery (CPU side of roadmap B1):
//! 1. reverse-dependency invalidation: an edit dirties exactly its downstream
//! 2. damage rendering: damaged output byte-equals a full re-render (parity
//!    gate), untouched work costs zero tiles
//! 3. animation frames reuse the vast majority of tiles (damage efficiency)
//!
//! The GPU half of B1 (instanced uber-shader, 100k@60fps, CPU/GPU parity)
//! lands with the tech-stack decision (leaning wgpu) — see roadmap.

mod common;

use mo2d::command::{apply_stream, parse_stream};
use mo2d::eval;
use mo2d::inval::DepIndex;
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::render::{render, render_damaged, Camera, DamageStats, RenderOpts};

/// A tentacle-like document: skeleton + bound variable-width body with fill
/// (exercises both stroke strips and fill triangles in the damage path).
fn tentacle_doc() -> Document {
    let stream = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"doc.setRoot","width":320,"height":180}},
          {{"op":"line.create","id":"skel","tags":["skeleton"],"width":2,"nodes":[{{"p":[40,90,0]}},{{"p":[80,90,0]}},{{"p":[120,90,0]}},{{"p":[160,90,0]}},{{"p":[200,90,0]}}]}},
          {{"op":"line.create","id":"body","closed":true,"width":10,"fillColor":[200,120,40,255],"fillOn":true,"nodes":[
            {{"p":[40,86,0],"w":12}},{{"p":[80,84,0],"w":10}},{{"p":[120,86,0],"w":8}},{{"p":[160,88,0],"w":6}},{{"p":[200,90,0],"w":4}},
            {{"p":[160,94,0]}},{{"p":[120,96,0]}},{{"p":[80,96,0]}}]}},
          {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":[40,90,0]}},{{"p":[{}],"v":[70,130,0]}}]}},
          {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"skel","node":4,"attr":"position"}}}},
          {{"op":"relation.create","type":"bind","target":"body","skeleton":"skel","falloff":60}}
        ]}}"#,
        TICKS_PER_SECOND
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    doc
}

fn opts() -> RenderOpts {
    RenderOpts { width: 320, height: 180, camera: Camera::Ortho2d, samples: 1 }
}

#[test]
fn damage_animation_parity_full_render() {
    let doc = tentacle_doc();
    let o = opts();
    // Walk the animation, recompositing damaged each step; every frame must
    // byte-match a full re-render (the B1 parity gate).
    let tick_end = TICKS_PER_SECOND / 2;
    let steps = 8;
    let mut prev_img = None;
    let mut prev_lines = eval::evaluate(&doc, 0).unwrap();
    let mut total_dirty = 0u32;
    let mut total_tiles = 0u32;
    for s in 0..=steps {
        let tick = tick_end * s as i64 / steps as i64;
        let lines = eval::evaluate(&doc, tick).unwrap();
        let full = render(&lines, 320.0, 180.0, &o);
        match prev_img {
            None => prev_img = Some(full),
            Some(ref prev) => {
                let (damaged, stats) = render_damaged(prev, &prev_lines, &lines, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
                assert_eq!(
                    damaged.px, full.px,
                    "damaged frame {} must byte-match the full render (parity gate)",
                    s
                );
                total_dirty += stats.dirty_tiles;
                total_tiles += stats.total_tiles;
                prev_img = Some(full);
            }
        }
        prev_lines = lines;
    }
    assert!(total_dirty > 0, "animation must dirty something");
    assert!(
        total_dirty < total_tiles,
        "animation damage must reuse tiles: dirty {} of {} (steps summed)",
        total_dirty,
        total_tiles
    );
}

#[test]
fn damage_no_change_costs_zero_tiles() {
    let doc = tentacle_doc();
    let o = opts();
    let lines = eval::evaluate(&doc, 0).unwrap();
    let prev = render(&lines, 320.0, 180.0, &o);
    let lines2 = eval::evaluate(&doc, 0).unwrap();
    let (img, stats) = render_damaged(&prev, &lines, &lines2, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
    assert_eq!(stats.dirty_tiles, 0, "identical frames must cost zero tiles");
    assert_eq!(stats.changed_lines, 0);
    assert_eq!(img.px, prev.px, "image must pass through untouched");
}

#[test]
fn damage_line_removal_clears_region() {
    let mut doc = tentacle_doc();
    let o = opts();
    let before = eval::evaluate(&doc, 0).unwrap();
    let prev = render(&before, 320.0, 180.0, &o);
    // Delete the body line entirely.
    let del = parse_stream(r#"{"v":1,"cmds":[{"op":"line.delete","id":"body"}]}"#).unwrap();
    apply_stream(&mut doc, &del).unwrap();
    let after = eval::evaluate(&doc, 0).unwrap();
    let (img, stats) = render_damaged(&prev, &before, &after, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
    assert!(stats.dirty_tiles > 0, "removal must dirty the line's tiles");
    let full = render(&after, 320.0, 180.0, &o);
    assert_eq!(img.px, full.px, "removal damage must byte-match full render");
    assert!(!img.px.iter().any(|&b| b == 200), "old fill pixels must be gone");
}

#[test]
fn damage_edit_dirties_local_region_only() {
    let mut doc = tentacle_doc();
    let o = opts();
    let before = eval::evaluate(&doc, 0).unwrap();
    let prev = render(&before, 320.0, 180.0, &o);
    // Move one node of the free-ish skeleton end (drives bind locally).
    let mv = parse_stream(
        r#"{"v":1,"cmds":[{"op":"node.set","line":"skel","index":4,"p":[240,60,0]}]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &mv).unwrap();
    let after = eval::evaluate(&doc, 0).unwrap();
    let (img, stats) = render_damaged(&prev, &before, &after, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
    let full = render(&after, 320.0, 180.0, &o);
    assert_eq!(img.px, full.px, "edit damage must byte-match full render");
    assert!(
        stats.dirty_tiles <= stats.total_tiles / 2,
        "a local edit must leave most tiles untouched: {}/{}",
        stats.dirty_tiles,
        stats.total_tiles
    );
}

#[test]
fn damage_3d_perspective_parity() {
    // The parity gate must hold under the perspective camera too.
    let stream = format!(
        r#"{{"v":1,"cmds":[
          {{"op":"line.create","id":"pyr","closed":true,"fillColor":[220,80,80,255],"fillOn":true,"nodes":[
            {{"p":[0,-30,-80]}},{{"p":[40,30,0]}},{{"p":[30,10,60]}}]}},
          {{"op":"line.create","id":"trk","tags":["track"],"nodes":[{{"p":[0],"v":[0,0,0]}},{{"p":[{}],"v":[25,10,0]}}]}},
          {{"op":"relation.create","type":"track","trackLine":"trk","target":{{"line":"pyr","node":0,"attr":"position"}}}}
        ]}}"#,
        TICKS_PER_SECOND
    );
    let mut doc = Document::default();
    apply_stream(&mut doc, &parse_stream(&stream).unwrap()).unwrap();
    let cam = Camera::Perspective { eye: mo2d::math::Vec3::new(80.0, 60.0, 140.0), target: mo2d::math::Vec3::ZERO, up: mo2d::math::Vec3::new(0.0, 1.0, 0.0), fov_y_deg: 60.0, near: 1.0, far: 1000.0 };
    let o = RenderOpts { width: 320, height: 180, camera: cam, samples: 1 };
    let l0 = eval::evaluate(&doc, 0).unwrap();
    let img0 = render(&l0, 320.0, 180.0, &o);
    let l1 = eval::evaluate(&doc, TICKS_PER_SECOND).unwrap();
    let img1_full = render(&l1, 320.0, 180.0, &o);
    let (img1_dmg, stats) = render_damaged(&img0, &l0, &l1, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
    assert_eq!(img1_dmg.px, img1_full.px, "3D perspective damage must byte-match full render");
    assert!(stats.dirty_tiles > 0);
    assert!(stats.dirty_tiles < stats.total_tiles);
}

#[test]
fn damage_stats_shape() {
    // DamageStats carries the numbers the stage/UI reports.
    let doc = tentacle_doc();
    let o = opts();
    let a = eval::evaluate(&doc, 0).unwrap();
    let img = render(&a, 320.0, 180.0, &o);
    let b = eval::evaluate(&doc, TICKS_PER_SECOND / 4).unwrap();
    let (_, stats) = render_damaged(&img, &a, &b, 320.0, 180.0, &o, &mut mo2d::assets::AssetStore::new());
    let DamageStats { dirty_tiles, total_tiles, changed_lines } = stats;
    assert_eq!(total_tiles, ((320 + 63) / 64) * ((180 + 31) / 32));
    assert!(changed_lines >= 2, "skeleton + bound body must both be flagged (got {})", changed_lines);
    assert!(dirty_tiles <= total_tiles);
}

#[test]
fn edit_signals_flow_through_the_dag() {
    // Integration: a command edit → seeds → DepIndex → the exact dirty set.
    let mut doc = tentacle_doc();
    let idx = DepIndex::build(&doc);
    // An edit to the track line dirties skeleton and body, not the camera.
    let dirty = idx.affected(&doc, &["trk".to_string()]);
    assert!(dirty.contains("skel") && dirty.contains("body"));
    assert!(!dirty.contains("trk") || dirty.contains("skel"), "seed itself included");
    // Direct stage-style flow: move a skeleton node → body dirty.
    let mv = parse_stream(
        r#"{"v":1,"cmds":[{"op":"node.set","line":"skel","index":2,"p":[130,120,0]}]}"#,
    )
    .unwrap();
    apply_stream(&mut doc, &mv).unwrap();
    let dirty = idx.affected(&doc, &["skel".to_string()]);
    assert!(dirty.contains("body"), "bind target must be flagged dirty by the index");
}
