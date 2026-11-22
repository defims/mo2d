---
type: Reference
title: "Renderer survey: Rive pipeline & Grease Pencil comparison"
description: Deep source-level survey of Rive's runtime and renderer (load/advance/draw pipeline, interlock modes, coverage encoding, feathering, ore binding layer) and Grease Pencil v3's data model — with the reference division of labor for mo2d (deformation → Rive; assets/timeline → GPv3; semantics → Moho) and the executable borrow list.
tags: [renderer, rive, grease-pencil, reference, survey]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-22T17:29:52Z }
sources:
  - id: rive-runtime
    resource: https://github.com/rive-app/rive-runtime
    title: "rive-runtime (MIT © 2023 Rive — license verified on the cloned tree)"
  - id: rive-renderer
    resource: https://github.com/rive-app/rive-renderer
    title: "rive-renderer (MIT — PLS/depth-stencil backends, shaders, feather, ore)"
  - id: rive-rs
    resource: https://github.com/rive-app/rive-rs
    title: "rive-rs (MIT — FFI shell over rive-cpp, 2,605 lines, Vello backend module)"
  - id: gp3-design
    resource: https://code.blender.org/2023/05/the-next-big-step-grease-pencil-3-0
    title: "The Next Big Step: Grease Pencil 3.0 (code.blender.org)"
  - id: gp-arch
    resource: https://developer.blender.org/docs/features/grease_pencil/architecture/
    title: "Grease Pencil Architecture — Blender Developer Docs"
  - id: gp-migration
    resource: https://developer.blender.org/docs/release_notes/4.3/grease_pencil_migration/
    title: "GP Migration to Blender 4.3 (attribute storage mapping)"
  - id: gp51
    resource: https://developer.blender.org/docs/release_notes/5.1/grease_pencil
    title: "Grease Pencil 5.1 notes (fill_id-grouped CDT, even-odd with holes)"
---

**English** | [简体中文](renderer-survey.zh-CN.md)


# Renderer survey: Rive pipeline & Grease Pencil comparison

Source-level survey (clones under `/Users/wei/rive-study/`; both MIT — verified on the
trees — legally reference-able per [AGENTS.md](../AGENTS.md)). Blender/GPv3 material is
public documentation only (GPL: design ideas, zero code).

## 1. Why these two references — the division of labor

| Concern | Reference | Why |
|---|---|---|
| **Curve deformation animation** (continuous, per-node channels, bones/sim deforming curves every frame) | **Rive** | advance-then-deform-then-render is its main pipeline; analytic rendering of per-frame-new curves is its core competence |
| **Assets, timeline semantics, frame-by-frame workflow** (drawing pool, instancing, explicit holds) | **GPv3** | its whole rewrite is that paradigm |
| **Semantics of deformation** (point channels, smart bones, binding) | **Moho** via [moho-mapping.md](moho-mapping.md) | the deepest prior; already fully mapped |
| Hand-drawn frame sequences, if ever supported | GPv3 form | expressible natively: a track whose keyframe nodes each reference a drawing line — no paradigm change needed |

## 2. Rive runtime pipeline (source: file.cpp / artboard.cpp / scene.rs)

```
.riv (runtime-first compact binary, 10–15× smaller than Lottie JSON)
  ↓ RuntimeHeader(major/minor) → ImportStack single-pass streaming decode
    (~40 typed importers: artboard/state machine/keyed property/viewmodel/…)
  ↓ artboard byte ranges recorded in-file → deferred per-artboard decode
    (deferArtboardBody / decodeAnimationBody; property table kept for reuse)
  ↓ WITH_RIVE_SCRIPTING_LUAU: Luau scripts + ShaderAsset registered post-import
    data binding family: viewmodel instances, converters, formulas, enums
Artboard → StateMachineInstance(inputs) → advance_and_apply(elapsed) -> bool
  → ComponentDirt bits + dependencyOrderRecipe (cached topological order,
    valid/unbuilt/unusable; allocation-free rebuild) → m_dirtyLayout queue
  → draw(renderer)   // the bool means "changed" — frame-level change
                     // detection baked into the API contract
```

Highlights vs mo2d: the deferred-artboard byte-range table is **lazy per-scene-object
decode built into the format** (our `.mo2d` BLOB layout is compatible with the same
idea); the dirt+recipe system is the same shape as our `inval.rs` + cached topological
order; `advance → bool` is the temporal-increment discipline at the API contract level.

## 3. Rive renderer (source: renderer/src + shaders)

**Five stages per frame**:

1. **Command generation**: the runtime converts the scene into `Draw` objects — four
   kinds only: `path | imageRect | imageMesh | stencilClipReset` (draw.hpp);
2. **Batching**: many paths merge into ONE flush — three GPU buffers accumulate
   (`PathBuffer`: 4×u32x4/path — 2×2 matrix, translation, flags, stroke/feather radii;
   `ContourBuffer`: midpoint, pathID, first-vertex, flags; `TessVertexTexture`:
   RGBA32UI tessellation vertices);
3. **Geometry expansion**: per-contour "wedge" instances; the vertex shader does
   three-level indirection (instance → tess vertex → contour → path) in
   `unpack_tessellated_path_vertex` (draw_path_common.glsl:316), with per-vertex
   `outset`/`fillCoverage`, self-correcting padding-vertex logic, and mirrored
   contours for feather bands (`MIRRORED_CONTOUR_CONTOUR_FLAG`);
4. **Coverage solve** — five-level `CoverageType`, chosen by `selectInterlockMode`
   (render_context.cpp:404): MSAA → `depthStencil` (universal fallback:
   `DS_POLAR_STROKE` + `DS_HAIRLINE_STROKE`); clockwise override → `clockwise`
   (raster-ordering hardware) or `clockwiseAtomic` (barrier-free atomics — the
   Clockwise fill rule makes atomics barrier-free); standard → `rasterOrdering`
   (PLS); last → `atomics`. Coverage accumulates into an R32UI atlas, resolved to R8
   by `resolve_atlas.glsl` (which also provides PLS's missing "clear");
5. **Composite**: gradients/images via `PaintAuxBuffer`, blend per Skia numbering
   (SrcOver=3 … Luminosity=28 — ecosystem interop).

**Passes are draws**: `renderPassInitialize` / `renderPassResolve` share the DrawType
table with ordinary draws (render_context.cpp:1505/2398/4105+) — the whole frame is one
flat GPU command list with zero branches (structurally identical to mo2d's unified
DrawType command table).

**Vector feathering** (the showcase): not a blur — a **Gaussian-integral lookup table**
(`g_gaussianIntegralTableF16`, F16 R16F 1D array texture; regenerable via
`RIVE_GENERATE_FEATHER_LUT`) sampled along mirrored contour bands; geometric and feather
coverage **share one float4 varying** via the `-2` bias encoding (`FEATHER_COVERAGE_THRESHOLD
-1.5` separates them) — zero extra passes, any feather radius; feather patches are
bin-packed into a skyline-rectanizer atlas (frame-local reuse, auto-grow). Large feathers
may opt into `featherAtlas` coverage; depthStencil mode must.

**ore** (2026): a unified bind-group/resource abstraction with per-backend contexts —
`ore_context_{metal,vulkan,d3d11,d3d12,gl,wgpu}` — **WebGPU as a first-class citizen**;
`gpu_census.hpp` adds a render-replay/census layer. Rive is migrating its backends onto
ore.

## 4. Why it is fast — the pipeline-level multiplication

| Layer | Mechanism | Evidence |
|---|---|---|
| Format | runtime-first binary; property-key streams; per-artboard lazy decode | file.cpp |
| Decode | ImportStack single pass, typed importers | file.cpp |
| Evaluation | dirt bits + cached topological recipe + advance/draw split + change-bool | artboard.cpp / scene.rs |
| Render | analytic coverage on tile GPUs (no tessellation storage), one flush, feather LUT | shaders/ |
| Engineering | thin bindings, one renderer everywhere, tiny C++ footprint | rive-rs 2,605 lines |

## 5. Grease Pencil v3 data model (public docs)

```
GreasePencil (ID)
├── layer tree root (LayerGroup; only groups contain nodes; layers are leaves)
│   └── Layer → own timeline → keyframes REFERENCE drawings by index
│       (frames carry start/end = explicit holds; instanced keyframes dark-shaded)
├── drawings: [Drawing]  ← "a static set of strokes with no layers; a vector image"
│   = CurvesGeometry: flat point array + curve_offsets
│   POINT domain: position, radius (thickness×pressure baked in), opacity,
│                vertex_color, rotation, delta_time, selection
│   CURVE domain: material_index, cyclic, caps, softness, curve_type
│                (POLY/BEZIER/CATMULL_ROM/NURBS), aspect_ratio,
│                fill_color/fill_opacity, init_time
└── materials (deliberately simple: fill+stroke)
```

Deleted in the rewrite: `triangles` (baked triangulation), `bounds_*`, `line_width`,
`edit_curve` — all pure functions of stored data now computed at runtime (the same rule
as mo2d's derived-data principle). 15 years of per-type rigid structs were the debt;
the rewrite converged onto references + attributes + flat arrays.

## 6. mo2d takeaways (executable, legally clean)

1. `advance_and_apply -> bool` — bake the change-bool into the evaluation API contract
   (A6 stage / B2 invalidation);
2. `MAPPED_ONCE_AT_INITIALIZATION` buffer flag — zero-copy resource semantics for the
   future GPU resource interface;
3. `Path::extend(from, transform)` — instanced path reuse, the render-side express lane
   for shape relations sharing one curve;
4. Blend-mode numbering aligned to Skia (Lottie/SVG interop for free);
5. GPv3 hold-boxes + dark-shaded instanced keyframes — the stage timeline UI semantics;
6. GPv5.1 fill: strokes grouped by `fill_id`, one CDT with holes (even-odd) per group —
   batch fill evaluation by color for our shape relations;
7. GPv2→v3 confirms: per-type rigid structs are debt; references + attributes + flat
   arrays are the direction mo2d starts at;
8. B1 far-term option: PLS/clockwise coverage path (recorded; WebGPU lacks a standard
   PLS extension — the wgpu instanced route stays primary).

## 7. Legal note

Rive runtimes: **MIT** (verified on the cloned trees) — implementation-level reference is
permitted with attribution if code is ever carried. Blender/GPv3: **GPL** — public
documentation only, design ideas, zero code (AGENTS redlines).
