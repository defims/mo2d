//! Gate hull — the visual-hull probe demo (docs/log.md 2026-11-17):
//! one turnaround timeline lifts into a slice stack (per-scanline half-plane
//! carving, deterministic order), then re-projects under three pose
//! trajectories into three derived 2D contour timelines. Assertions:
//! - containment: every lifted slice satisfies every drawn span exactly
//!   (hull ⊇ true object, span-wise), with a small carve-away bound;
//! - monotone carving: more frames only shrink sections (patch frames
//!   never invalidate earlier work);
//! - derived timelines: correct frame counts, non-degenerate outlines,
//!   byte-identical artifacts vs the frozen hashes;
//! - the frozen turnaround document and its render (purely additive to
//!   demo/frozen.json — existing demos untouched).

mod common;

use common::{demo_dir, doc_bytes, load_frozen, replay_stream, render_sha};
use mo2d::hull::{self, Frame, SliceStack};
use mo2d::model::*;
use mo2d::render::Camera;
use mo2d::sha::sha256_hex;
use std::sync::OnceLock;

fn hull_frames() -> &'static Vec<Frame> {
    static FRAMES: OnceLock<Vec<Frame>> = OnceLock::new();
    FRAMES.get_or_init(|| {
        let doc = replay_stream(&demo_dir().join("cmd/hull_turnaround.json"));
        let mut frames: Vec<(usize, Frame)> = Vec::new();
        for line in &doc.lines {
            if !line.tags.iter().any(|t| t == "hull-frame") {
                continue;
            }
            let mut idx = None;
            let mut angle = None;
            for tag in &line.tags {
                if let Some(v) = tag.strip_prefix("frame=") {
                    idx = Some(v.parse::<usize>().unwrap());
                }
                if let Some(v) = tag.strip_prefix("angle=") {
                    angle = Some(v.parse::<f64>().unwrap());
                }
            }
            let contour = line
                .nodes
                .iter()
                .map(|n| match &n.pos {
                    NodePos::Geom(p) => *p,
                    NodePos::Tick(_) => panic!("hull frames must be static geometry"),
                })
                .collect();
            frames.push((idx.unwrap(), Frame { angle_deg: angle.unwrap(), contour }));
        }
        frames.sort_by_key(|(i, _)| *i);
        frames.into_iter().map(|(_, f)| f).collect()
    })
}

fn lifted() -> &'static SliceStack {
    static STACK: OnceLock<SliceStack> = OnceLock::new();
    STACK.get_or_init(|| hull::lift(hull_frames(), hull::SLICES))
}

fn derived() -> &'static Vec<hull::DerivedTimeline> {
    static TIMELINES: OnceLock<Vec<hull::DerivedTimeline>> = OnceLock::new();
    TIMELINES.get_or_init(|| {
        let stack = lifted();
        let specs = hull::default_specs(stack);
        hull::derive_timelines(stack, &specs)
    })
}

#[test]
fn turnaround_doc_and_render_frozen() {
    let frozen = load_frozen();
    let section = frozen.raw.get("hull").expect("frozen.json: hull section");
    let want_doc = section.get("docSha256").and_then(|v| v.as_str()).unwrap();
    let want_frame = section.get("frameSha256").and_then(|v| v.as_str()).unwrap();
    let tick = section.get("midTick").and_then(|v| v.as_i64()).unwrap();

    let doc = replay_stream(&demo_dir().join("cmd/hull_turnaround.json"));
    assert_eq!(
        sha256_hex(&doc_bytes(&doc)),
        want_doc,
        "turnaround document bytes drifted"
    );
    assert_eq!(render_sha(&doc, tick, Camera::Ortho2d), want_frame, "turnaround render drifted");
}

/// Round trip: at every drawn angle the lifted slice's screen-x span must
/// sit inside the drawn span (exact, by construction) and lose at most a
/// small carve-away to the other frames' constraints (empirical bound:
/// adjacent 30° frames carve a little; gross thinning would mean a bug).
#[test]
fn hull_lift_containment_and_round_trip() {
    let stack = lifted();
    let frames = hull_frames();
    assert_eq!(stack.ys.len(), hull::SLICES, "slice count");
    let dy = stack.ys[1] - stack.ys[0];
    for f in frames {
        for (j, sec) in stack.sections.iter().enumerate() {
            let y = stack.ys[j];
            let Some((a, b)) = hull::contour_span_x(&f.contour, y) else {
                panic!("frame must span every slice height");
            };
            let (a, b) = (a - stack.center_x, b - stack.center_x);
            let Some((lo, hi)) = hull::section_span_x(sec, f.angle_deg) else {
                panic!("slice {} lifted to an empty section", j);
            };
            assert!(
                lo >= a - 1e-6 && hi <= b + 1e-6,
                "containment violated at frame angle {} slice {}: [{},{}] vs drawn [{},{}]",
                f.angle_deg, j, lo, hi, a, b
            );
            let drawn = b - a;
            let deficit = (b - hi) + (lo - a);
            let bound = 0.10 * drawn + 4.0 * dy;
            assert!(
                deficit <= bound,
                "carve-away too large at frame angle {} slice {}: deficit {} > {}",
                f.angle_deg, j, deficit, bound
            );
        }
    }
}

/// Patch monotonicity: lifting with MORE frames only shrinks sections.
/// The partial lift keeps the first three frames only (normals leave wide
/// unconstrained sectors), so the remaining nine genuinely carve — every
/// full-lift slice area is ≤ the partial's, and the total carve is real.
#[test]
fn hull_carve_is_monotone() {
    let full = lifted();
    // Same axis as the full lift, so the two stacks live in one coordinate
    // system and slice areas are comparable.
    let partial = hull::lift_centered(&hull_frames()[..3], hull::SLICES, full.center_x);
    let mut total_carve = 0.0f64;
    for (sf, sp) in full.sections.iter().zip(partial.sections.iter()) {
        let (af, ap) = (hull::shoelace_area(sf), hull::shoelace_area(sp));
        assert!(
            af <= ap + 1e-6,
            "adding frames grew a slice: {} > {}",
            af, ap
        );
        total_carve += ap - af;
    }
    // Slice areas run ~10^4; the nine extra frames must carve a visible
    // fraction, not float noise.
    assert!(total_carve > 100.0, "patch frames carved almost nothing: {}", total_carve);
}

#[test]
fn hull_derived_timelines_frozen() {
    let frozen = load_frozen();
    let section = frozen.raw.get("hull").expect("frozen.json: hull section");
    let derived_section = section.get("derived").expect("frozen.json: hull.derived");
    let timelines = derived();
    let mut names: Vec<&str> = timelines.iter().map(|t| t.name.as_str()).collect();
    names.sort();
    assert_eq!(names, vec!["orbitDiagonal", "pitchNod", "yawSwing"]);

    for t in timelines {
        let want = derived_section
            .get(&t.name)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("frozen.json: hull.derived.{} missing", t.name));
        let bytes = hull::timeline_json(t);
        // Determinism: identical bytes on a second derive.
        let stack = lifted();
        let specs = hull::default_specs(stack);
        let again = hull::derive_timelines(stack, &specs)
            .into_iter()
            .find(|x| x.name == t.name)
            .unwrap();
        assert_eq!(hull::timeline_json(&again), bytes, "{} not reproducible", t.name);
        assert_eq!(sha256_hex(bytes.as_bytes()), want, "{} artifact drifted", t.name);

        // Sanity: frame count matches its spec, every frame non-degenerate.
        let specs = hull::default_specs(lifted());
        let spec = specs.iter().find(|s| s.name == t.name).unwrap();
        assert_eq!(t.frames.len(), spec.angles_deg.len(), "{} frame count", t.name);
        for (fi, f) in t.frames.iter().enumerate() {
            assert!(!f.is_empty(), "{} frame {}: no loops", t.name, fi);
            for lp in f {
                assert!(lp.len() >= 3, "{} frame {}: degenerate loop", t.name, fi);
                let area = hull::shoelace_area(lp).abs();
                assert!(area > 1.0, "{} frame {}: outline area {}", t.name, fi, area);
            }
        }
    }
}

/// The derived timelines are genuinely different 3D rotations, not one
/// trajectory relabeled: their mid frames must differ pairwise.
#[test]
fn hull_derived_timelines_differ() {
    let timelines = derived();
    let mid = |t: &hull::DerivedTimeline| -> String { hull::timeline_json(&hull::DerivedTimeline {
        name: String::new(),
        frames: vec![t.frames[t.frames.len() / 2].clone()],
    }) };
    let (a, b, c) = (&timelines[0], &timelines[1], &timelines[2]);
    assert_ne!(mid(a), mid(b));
    assert_ne!(mid(b), mid(c));
    assert_ne!(mid(a), mid(c));
}

/// The frozen.json hull section exists with the expected shape (guards the
/// freeze step: section keys, slice/grid constants recorded).
#[test]
fn frozen_hull_section_shape() {
    let frozen = load_frozen();
    let section = frozen.raw.get("hull").expect("frozen.json: hull section");
    for key in ["docSha256", "frameSha256", "midTick", "derived", "slices", "grid"] {
        assert!(section.get(key).is_some(), "frozen.json: hull.{} missing", key);
    }
    assert_eq!(section.get("slices").and_then(|v| v.as_i64()), Some(hull::SLICES as i64));
    assert_eq!(section.get("grid").and_then(|v| v.as_i64()), Some(hull::GRID as i64));
}
