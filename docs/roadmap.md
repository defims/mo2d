---
type: Design Document
title: "Roadmap: Three-Phase Advance (2026-11-11 v4, stage pulled forward into the MVP)"
description: Phase A = MVP tentacle v4 (AI compiles, engine executes + A6 minimal stage = first example of a generatable view); Phase B = performance and vocabulary rollout (rendering & invalidation formalization / brush strokes / gradients / layers & groups / bitmap mesh / simulation / format formalization); Phase C = AI ecosystem (relation IR / controls-as-lines / import / audio / collaboration / wasm); six-components → phase mapping and performance metrics.
tags: [roadmap, phases, mvp, long-term, interactivity]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-22T15:10:00Z }
sources:
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:MVP v5 定稿(触手+3D 场景)"
  - id: vision-doc
    resource: /vision.md
    title: "本 bundle:愿景(§2 六部件、§6 分工)"
  - id: architecture-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§2.1 热路径内核+关系 IR、§3.1 BUI 借鉴)"
  - id: file-format-doc
    resource: /file-format.md
    title: "本 bundle:文件格式(八条细则)"
---

**English** | [简体中文](roadmap.zh-CN.md)

# Roadmap: Three-Phase Advance (v4)

Principle: **every step ships a visible deliverable; every step is accepted via the command stream**
(all operations are commands, a demo = command stream + output, automatically re-verifiable);
no passing the gate, no next step. v4 is resequenced per the six-components engine definition of
[vision.md](vision.md) §2: **A builds the language and runtime, B builds the views and performance, C grows the ecosystem.**

## Six components → phase mapping

| Component (vision §2) | Landing phase |
|---|---|
| 1 Semantic language | A (minimal vocabulary) → B4–B7 (vocabulary rollout) → C7 (interaction vocabulary: respond/oscillate/step) → C8 (gesture timelines) |
| 2 Deterministic runtime | A (core) → B2 (invalidation & parallelization) |
| 3 Command protocol | A (three commands + command stream) → C (ecosystem scale-up) |
| 4 Validator | A (schema + determinism gate) → C1 (gate for the relation IR) |
| 5 Performance machine | B1 (GPU workhorse + damage rendering) |
| 6 Generatable view | **A6 (MVP minimal stage, owner decided on 2026-11-11 to pull it forward)** → C2 (controls as lines) |

## Phase A: MVP — the tentacle ("AI compiles, engine executes" + first example of a generatable view)

For the task brief and acceptance gates see [mvp.md](mvp.md) v4: A1 line kernel (offscreen) →
A2 skeleton binding → A3 tracks/Tick/playback → A4 command protocol + MCP + validator →
A5 format + export + offline closed loop → **A6 minimal stage (formerly Phase B1, pulled forward:
four operations, hand-built and AI command streams isomorphic)**.
Definition of Done, seven items: the AI builds the animation end-to-end via MCP, replay byte-equality,
**offline playback consistency (capability as data)**, round-trip byte-equality, evaluation throughput,
validator on duty, **direct-manipulation equivalence**.

## Phase B: Performance and vocabulary rollout

> **Status (2026-11-12)**: Phase B is landed with one exception. B1's CPU
> side — reverse-dependency invalidation (`inval.rs`), damage rendering
> (64×32 tiles, byte-parity with the full render), stage render cache,
> 100k-node eval gate; B2 brush + pressure + styles; B3 gradient + color
> tracks; B4 group/transform-box; B5 bitmap mesh (zero-dep PNG decode,
> texture refs + uv anchors, budgeted cache, textured fills); B6 sim
> (replay-from-seed) + seeded particles + collisions (clearance, self-separation,
> particle restitution); B7 format formalization (single-file
> packages with a tail BLOB area, pack/unpack project directories, schema
> emission, the version-migration chain) — all implemented with gate tests
> and frozen demos (see log). **The one remaining item is B1's GPU half**
> (instanced uber-shader, 100k@60fps render, CPU/GPU pixel parity), which
> waits on the tech-stack decision in "Not yet scheduled".

| Item | Content | Acceptance demo |
|---|---|---|
| B1 Rendering & invalidation formalization (component 5) | GPU workhorse: instanced uber-shader (segment/node patches = instances), damage rendering (tile 64×32 + dirty rectangles), **DAG reverse-dependency invalidation** (parameters = signals), CPU/GPU pixel parity gate | 100k-node class at 60fps; edit viewport idle at 0% CPU |
| B2 Brush-stroke style library | Line-level brush 15-parameter family + node-level pressure; styles = attribute template library | The same line switching between brush strokes; pressure-sensitive width |
| B3 Gradient control bars | Control-bar line segments + color-stop nodes; bound to fill | A color-changing tentacle (animatable gradient) |
| B4 Multi-layer/groups | Layer/group predefined relations + transform-box lines | Multiple characters on one stage; box scaling and rotation |
| B5 Bitmap mesh | Warp/weft lines + texture-anchor binding = media payload; decode-on-demand + budgeted cache (following BUI) | A bitmap character deforming with the skeleton |
| B6 Simulation (dynamic relations) | Springs/forces/collisions/particles; PBD/compute; deterministic seeds | Physically trailing tentacle tail + particles |
| B7 Format formalization | Tail BLOB area (embedded media) + project-directory layout (unpacked, equivalent to the package) + external refs + JSON Schema/validator + FORMAT_VERSION migration; the spec draft follows the maop-spec conventions | A media-bearing project delivered as a single .mo2d file; unpack/pack round-trip lossless; old versions still readable |

Dependencies: B1 depends on A's SoA layout; B2–B6 are mutually independent and can proceed in parallel; B7 (format formalization) depends on B5 (bitmap mesh — only media payloads create the BLOB need).

## Phase C: AI ecosystem (capability as data)

| Item | Content | Acceptance demo |
|---|---|---|
| C1 **Relation IR + custom relation generation** | Natural language → relation IR (restricted intermediate representation, sandboxed); **generate → pass the determinism gate → freeze into the library** (architecture §2.1) | "Make the tentacle follow the cursor" — one sentence becomes an animation; playable offline once frozen into the library |
| C2 Controls as lines | Ephemeral control system (slider = track line + cursor line; color picker strip = control bar; transform box = box) | The AI generates controls on demand, discarded after use |
| C3 Importers/interchange | **Bidirectional SVG**: frame-level export (a natural byproduct of display-edit unification) + geometry import ([file-format.md](file-format.md) §4 three-layer format); .moho as a long-term option ([moho-mapping.md](moho-mapping.md) is a ready-made mapping table; acceptance via the 84-project tutorial corpus) — **geometry import landed (2026-11-22)**: `svg.import` command + stage drag-drop → editable cubic lines as one layer (gate_svg + the svg_face frozen demo); frame-level export still pending | Exported frame SVG viewable in any tool; import real SVG/tutorial projects |
| C4 Audio | Audio-track layer: media payload + level/jump tracks; Tick dividing 44.1k/48k evenly exists exactly for this | Lip-sync/beat-driven animation |
| C5 Collaboration & versioning | Command-stream merging, change records, reference layers | Multi-session collaboration without conflicts |
| C6 WebAssembly + frame streaming | wasm release + MCP over WebSocket + **streaming rendering** (architecture §3.2: frame-stream protocol; pixel streams for export, display-list streams for remote stages) | tentacle demo in the browser, zero install; remote stage renders progressively |
| C7 **Interactivity** (established 2026-11-17 by the owner; [relations.md](relations.md) §1.2, architecture §3.3) | **respond** relation family (event × guard → write parameter/fire; state machine = discrete parameter + guarded respond), **oscillate** periodic driver, **step interpolation im=3** (discrete parameters — the Switch basis), events enter the command stream (**`event.fire`**, irreversible facts), stage tap (D8 amended: five operations), minimal **perform-record** (gesture → track keys via auto-key); evaluation = pure `f(doc, tick, event-prefix)`, empty prefix = existing behavior | "Tap it and it jumps": **tap_jump demo replays byte-identically offline, no AI, no code** (the novice gate N3); breathe/sway oscillate demos; same stream + same events → same bytes |
| C8 **Interactive gesture timelines** (established 2026-11-17 by the owner; line-model §0 statement, [relations.md](relations.md) §1.2, mvp §8 D10) | **gesture lines materialized from the event stream**: `event.fire` atomic dual-write → read-only step (im=3) track lines `gesture/<target>` / `gesture/pointer` (keys on phase changes only; `gesture/` namespace reserved); recorder phase edges (press/release/hoverIn/hoverOut detectors join tap); **hold-aware responds** — `minHold`/`maxHold` = press→release pair measurement (fold lookback, browser click semantics; the line itself tracks physical containment, browser hover semantics); read-only reference-lane rendering on the stage's time axis (offline renderer skips `gesture`-tagged lines like `track` ones — `MoEvent` unchanged, events stay single-tick facts) | "tap_vs_hold" demo: a short tap jumps, a long press (hold ≥ 0.4 s) grows — replays byte-identically offline, no AI, no code (N3-style); gesture-line keys byte-stable from the same script — **landed (C8 P1, 2026-11-17)**: gate_c8 + the tap_vs_hold frozen demo |

Dependencies: C1 depends on the A4 validator and B1 (hot-path fallback); C3's .moho depends on B4 (layers/groups) + B5 (bitmap mesh); C4 depends on B7 (format formalization); C7 depends on the A4 protocol/validator (events are commands) and the D8 revision (owner approved 2026-11-17); C8 depends on C7 (event stream + step im=3) and the D10 decision (owner approved 2026-11-17).

## Milestones and performance metrics

| Phase | Metric |
|---|---|
| Phase A | 10k nodes at 60fps (CPU); command replay byte-equality; offline playback consistency |
| Phase A6 | Hand-built and AI command streams isomorphic (direct-manipulation equivalence) |
| Phase B1 | 100k-node class at 60fps (GPU); parity gate passes; idle 0% CPU |
| Phase C | AI end-to-end (natural language → exportable animation) single-shot success rate = north-star metric (target TBD) |

## Cadence clauses

- At the end of every phase, **look back at this roadmap** and revise it (via log);
- For any new feature, ask first: "Is this a **hot path**?" — if yes, it goes into the engine; if no, hand it to the AI (carried by C1);
  changes to the language itself (new primitives/new attribute families) go through the Normative Statements process of line-model/relations.

## Not yet scheduled

Final tech-stack decision (leaning Rust+wgpu, [architecture.md](architecture.md) §5), product branding,
and the mo2d–maohou engineering merge strategy — they enter the map only after the owner makes the call.

**Phase D (~~3D deepening, long-term, not yet chartered~~ dissolved 2026-11-18 by D11)**: owner decision —
"既然3d可以由2d表示和操控，那3d就不用单独支持，而是转为2d支持" (since 3D can be represented and manipulated
through 2D, 3D is never supported separately; it converts into 2D support, [mvp.md](mvp.md) §8 D11).
The engine side keeps its basic 3D dividend (renderer-is-3D/Vec3/triangular faces/3D scene gate — D3/D6 unchanged,
occlusion and cameras stay correct for free). What was the Phase D deepening list converts into small **2D-native slices**:
① generation-relation lift/view (the hull probe is the existence proof, log 2026-11-17); ② surface stamp
(internal lines anchored onto surfaces); ③ camera rig as lines (orbit/zoom = 2D-driven parameters — no 3D navigation);
④ ray-snap manipulation (drag a projection point → ray → surface, no work planes/depth picking as 3D concepts);
lights/materials/shadows are deleted as engine features (replacement candidate: tone lines stamped on surfaces —
pending owner confirmation); glTF fate pending owner confirmation (delete vs demote-to-baked-artifact).
The extension math remains recorded in [3d-extension.md](3d-extension.md).
