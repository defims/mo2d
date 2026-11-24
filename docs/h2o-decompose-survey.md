---
type: Reference
title: "Survey: h2o-video-decompose — measuring a 2D animation for 1:1 canvas2d+js recreation"
description: Source-level survey of the h2o-video-decompose skill — a measurement + dispatch + acceptance system (no canvas template; the AI authors JS from measured data). Pipeline, classical-CV measurement layer, the canvas contract, four acceptance gates with anti-gaming mechanics, and the curve-drawing vision/measurement division of labor.
tags: [h2o, survey, measurement, canvas2d, acceptance, reference]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-25T00:12:00Z }
sources:
  - id: h2o-repo
    resource: https://github.com/Poseima/h2o-video-decompose
    title: "h2o-video-decompose (no LICENSE file found — treat as all-rights-reserved: reference ideas/public documentation only, zero code)"
  - id: h2o-tree
    resource: file:///Users/wei/skill-study/h2o-video-decompose/
    title: "Local clone (tarball; distribution/ = SKILL.md 174 lines + references ×5 + scripts ×14 (12.7k lines) + tests ×6)"
---

# Survey: h2o-video-decompose

A Claude skill that decomposes a provided video (shots, cuts, per-frame motion, fonts,
colors, audio-visual alignment) so an AI can **rebuild it 1:1 as an editable project**
(Remotion / AE / CSS / **Canvas** / Blender / 3DGS). Source-level survey: local clone +
three parallel deep-reads of all 12.7k lines.

## 1. What it is — and the pivotal finding

It is a **measurement + dispatch + acceptance** skill. **It ships no canvas2d/JS
template, no scaffolding, no code-gen conventions** ("Canvas" appears only as a routing
label — the JS is authored from scratch by segment subagents). What makes 1:1 possible
is not a template but **measured data + hard contracts + quantified acceptance gates**:

> measure the video into per-frame numbers → subagents rebuild from the numbers under
> contracts (coordinate chain / timing model / no-source-pixels) → four gates judge the
> export → rework until byte-level-consistent evidence says pass.

This is "AI compiles, gates adjudicate" — the same philosophy as mo2d's division of
labor, and an industry-grade proof of R5 (agent writes declarative source + deterministic
renderer) from the ai-era audit.

## 2. The pipeline (nine stages)

1. **Install/verify**: Python 3.12 venv + ffmpeg (libx264); smoke test renders + scans a
   synthetic clip;
2. **scan**: ffprobe per-frame PTS index (first frame = zero; ranges left-closed/right-open)
   → 180 px decode → adjacent-luma-MAE cut candidates → storyboard + lookdev keyframes;
3. **Scene-paradigm classification** (per shot): histogram top-two-bin occupancy > 0.68 +
   single-plane homography holdout p95 ≤ 2.75 px → `2D_PLANAR_UI_MOTION` (→
   Remotion/AE/**Canvas**); parallax P85/P20 > 1.35 → `3D_BLENDER_SCENE`; both scores high
   → `HYBRID_2D_OVER_3D`;
4. **Capability grading**: 🟢 L1 (2D UI/motion graphics — full recreation) / 🟡 L2 (needs
   verified assets) / 🔴 L3 (out of bounds — notably **traditional hand-drawn frame-by-frame
   animation is honestly 🔴**); per-shot grading, "never reject a whole video";
5. **Key-effect inventory**: per shot, the changes that most affect perception, each with
   an observable pass criterion (becomes a mandatory acceptance item);
6. **Kickoff questionnaire** (AskUserQuestion batched: route / tier / subagent model …)
   + dispatch confirmation table;
7. **measure** (the core): per-shot plan (reference frame, bridges, stable/exclude zones,
   anchors, keyframed zone boxes) → per-frame homography + tracks + edge widths
   (see §3);
8. **Segment rebuild + rework**: subagents author the editable segment (for 2D: the
   canvas2d/JS project) — **no source pixels rule** ("禁止把原片片段、静帧、截图…作为底图、
   纹理或其他可见层"); version 0 free, ≥ 1 rework round per segment;
9. **Integration + whole-film acceptance**: merge, PTS sync check, editability proof
   (modify one key element → re-render → verify), `effects` + `continuity` on the final
   export; segment passes never substitute the whole-film verdict.

Master–servant agent structure is **mandatory for recreation** (master only scans/grades/
dispatches/integrates; each segment owner covers ≤ 2 s in the precise tier; if the host
cannot spawn subagents → pause and explain, never degrade to single-agent).

## 3. The measurement layer (all classical CV — zero deep learning)

- **Sparse LK optical flow** (winSize 29×29, 3 levels) with **bidirectional gating** —
  the reverse track must return within 1.15 px, else the point is discarded;
- **NCC template matching**: DoG-normalized 25×25 templates searched in 53×53 windows,
  score ≥ 0.64 with a 0.045 uniqueness margin, parabolic sub-pixel refinement;
- **Hierarchical model fit with spatial holdout**: similarity → affine → homography
  (USAC_MAGSAC, 2.8 px), complexity penalty prefers simpler models; 34×31 lattice holdout
  cells; status `validated` only when holdout_p95_px ≤ 2.75;
- **ESF edge-width bank**: 288 erf templates (32 σ × 9 shifts × 57 samples); width =
  2.563σ (the 10–90% transition width), validity gates on amplitude/plateau/residual;
- **Scene classifier**: 16-bin histogram top-two > 0.68 + flat-gradient > 0.58 (+2.5
  toward 2D) vs shading gradients / vanishing points / parallax (+1.8–2.0 toward 3D);
- Outputs: `measurements.json` per-frame `{geometry.H, status, holdout_p95_px, metrics
  (anchor/spans/angle), zones (edge-width stats), independent_checks}` + CSV mirrors +
  annotated overlays + trajectory curves.

The data contract is the deliverable: **per-frame H (camera/plane motion), per-layer
trajectories (endpoints, contour points, part angles), per-zone edge widths, palette
median colors, font IoU scores, audio-visual event table** — everything a canvas/JS
author needs to redraw without seeing the source as pixels.

## 4. The canvas2d+js contract (three layer kinds + one transform chain)

- **Layer taxonomy** (decided by observation): (a) static plane carried by the camera —
  authored once on the reference frame, per-frame H applied; (b) independently moving
  elements — camera motion removed, local trajectory fitted; (c) canvas-fixed elements
  (HUD/typing/subtitles) — authored directly in canvas coordinates. Decision method: track
  the element in both canvas and page-reference coordinates and see which is more stable;
- **The transform chain** (verbatim): "若 A 把工程图层坐标映射到测量参考帧,H_f 把测量
  参考帧映射到当前分析帧,S 把分析像素映射到输出画幅,则复刻投影为 S · H_f · A,最后
  除以齐次坐标";
- **Timing model**: constant frame rate → align by source frame number; variable frame
  rate → first-frame PTS as zero, bind render time via PTS deltas; 2 s measurement windows
  with overlapping verified anchors;
- **No named easing**: trajectories are stored as measured per-frame values aligned to
  source frames/PTS, with recorded phases (start / peak / pause / inter-layer delay /
  rebound / settle); unreliable magnitudes are left empty, order-only;
- **No source pixels**: measured composition/colors/trajectories/timings may be written
  into independently rebuilt layers — nothing else;
- Fonts: binary-contour **IoU across multiple contrast thresholds** against candidates
  (ink-box aligned/normalized per glyph); colors: median colors per stable region with
  source frame + rect + color space recorded; soft edges: edge-normal brightness profiles.

## 5. The acceptance loop — four orthogonal gates

All gates share the exit-code contract **0 = pass / 2 = rework / 3 = pending review**,
and anti-gaming mechanics: **review_id** = SHA-256 over (script + reference + replica +
plan); `--previous` must share reference+plan but have a *different* replica hash
("相同导出不能证明已修复"); human decisions require attached evidence; "缺测不能通过
人工结论覆盖"; only the **current export's** report counts.

1. **measure (position/timing)**: re-run measurement on the replica; compare H/trajectories
   to the reference table;
2. **lookdiff (appearance)**: 20 local-appearance metrics with per-metric resolution
   floors (`edge_width` 0.72 px, `period` 1.1 px, Lab `color_a/b` 7, `attachment` 0.24 …)
   + fractional deltas + noise floors; findings need **sign agreement ≥ 0.75 across
   frames**; **codec abstention** — if a re-encode control experiment reproduces the
   residual, the verdict is `insufficient` rather than blaming materials; blind test:
   top-3 localization 16/16, false positives 3/16 on held-out projects;
3. **effects (per planned item)**: 7 modes (motion / perspective / trail / focus /
   visibility / **impact (audio-visual: 5 ms RMS onset vs picture impulse)** / manual);
   per-mode algorithms (SIFT+homography for perspective with differential-foreshortening
   vectors; 4-axis Gaussian energy ratios for trails; layer-wise 10–90% widths for DOF…);
4. **continuity (whole export)**: **bidirectional DIS dense optical flow** — round-trip
   error > 2 px = mismatch; 12 defect kinds (scene/layer jumps, brightness steps, camera
   abrupt stops, round-trip jitter, duplicated frames, missing original events…), each
   with exact thresholds; evidence = side-by-side clips at 1× and 4× slow motion.

3D route additionally: `compare --strict` (19 lookdev feature gates + photometry/material
thresholds + SSIM ≥ 0.72 / Edge-IoU ≥ 0.55) and `audit-blend --strict` (headless .blend
audit enforcing the no-speed-compromise rendering policy).

## 6. The curve-drawing question — vision/measurement division of labor

**How does a curve get its parameters?** Not by vectorization — there is no tracing
anywhere. The parameter pipeline is: **the AI picks discriminating quantities** (line
endpoints, contour points, part angles) → writes them into the measurement plan
(`independent_points`) → **the scripts track them frame by frame** (optical flow + NCC,
sub-pixel) → **the canvas author draws constrained by the measured trajectories**
(any curve representation is allowed, as long as it passes the pixel gates).

Consequences:
- A static frame's vectors are **authored by the AI looking at the reference frame**
  (semantic judgment: "this is a rounded rect / circle / free curve"), with measured
  boxes/colors/widths as hard constraints;
- A free-form curve = **sample points on the curve → fit a path through them
  (Catmull-Rom → bezier conversion is mechanical) → pixel gates verify** (add samples
  where deviation exceeds tolerance, refit);
- The vision model's precision demand is deliberately low ("describe this screenshot"
  level): **vision proposes, measurement disposes, pixel gates adjudicate** — model
  quality affects rework rounds, not the quality ceiling (up to the 🔴 boundary where
  classical CV itself fails: non-rigid chaos, hand-drawn frames).

## 7. mo2d cross-insights

1. The measured-data JSON (per-frame H + trajectories) is **almost exactly the input
   form of mo2d track lines** — each `independent_point` trajectory = a track line with
   Tick keyframe nodes; "video → white-model rig" becomes a derivation relation with the
   h2o pipeline as its measurement stage (see [reference-derive.md](reference-derive.md));
2. **The missing piece this skill proves exists**: every segment re-authors canvas code
   from scratch — no engine semantics (no Tick truth, no relation reuse, no deterministic
   cache). mo2d is that missing engine: h2o's measurement data compiles into mo2d
   lines + relations + tracks;
3. **Adopt into mo2d's roadmap C acceptance design**: the four-gate structure
   (position/timing + appearance + per-effect + whole-export), the exit-code contract,
   `review_id` anti-gaming, codec abstention, and the blind-test protocol (held-out
   synthetic cases never used for tuning);
4. **Adopt the L1/L2/L3 honest-boundary wording** for mo2d capability claims
   (per-shot grading; "hand-drawn frame-by-frame = out of bounds" is stated, not hidden);
5. Master–subagent dispatch discipline (segment ≤ 2 s per owner, rework to the original
   owner) maps onto mo2d's own agent workflows.

## 8. Legal note

**No LICENSE file** in the repository → default all-rights-reserved: reference **ideas,
workflow design, and thresholds from public documentation only**; zero code from the
skill may be copied or translated into mo2d (stricter than the MIT-licensed Rive
references).
