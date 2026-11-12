---
type: Design Document
title: "Architecture Draft: Layering, Document Model, GPU Strategy, and Format Principles"
description: The mo2d architecture draft — document model = a flat line collection + relation graph (no rigid tree); L0-L4 layering; a dedicated GPU-strategy section (SoA / unified vertex format / DAG parallelism / PBD-compute / CPU oracle + damage rendering / instancing, referencing BUI); storage-format principles; tech stack leaning toward Rust + wgpu/wasm (to be decided).
tags: [architecture, gpu, damage-rendering, storage-format, layering]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
  - id: maohou-readme
    resource: file:///Users/wei/maohou/README.md
    title: "maohou/AniCraft 工程总览(CPU oracle + wgpu 双轨、命令/MCP 范式参照)"
  - id: vectorcraft-arch
    resource: file:///Users/wei/vectorcraft/docs/architecture.md
    title: "vectorcraft 架构文档(L0-L7 分层、命令注册表、版本化 JSON 格式参照)"
  - id: bui
    resource: https://github.com/ppmpretham/BUI
    title: "BUI(Blender UI 解耦库)——损伤渲染/实例化 uber-shader/双渲染器奇偶校验的工程参照(GPL,净室借鉴)"
---

**English** | [简体中文](architecture.zh-CN.md)

# Architecture Draft

> Status: **draft**. This document fixes only layering and principles, not crate-level design;
> the tech stack is a **lean**, not a decision (see §5).

## 1. Document model

**A flat line collection + relation graph + channel table + payload library + external-asset reference table.**

- No vectorcraft-style rigid Node tree[^vectorcraft-arch]: layers/groups are merely **organizational predefined
  relations**, and nesting is a mapping of relations ([relations.md](relations.md));
- Document-level parameters (frame rate, color space, stage size) = parameters of the **root relation**;
- For the data skeleton see [line-model.md](line-model.md) §5.

Key differences from the existing projects:

| | vectorcraft / AniCraft | mo2d |
|---|---|---|
| Organization | rigid Node tree (Layer/Group container types) | flat line collection + relation graph |
| Domain concepts | separate data types per concept (Path/Bone/Text/…) | everything projected to lines + semantic tags + relations |
| Behavior | feature code written separately per feature | all behavior = relations (functions) |

## 2. Layering (L0–L4)

| Layer | Responsibility | Notes |
|---|---|---|
| L0 geometry | line/node evaluation, Bézier flattening, hit testing | pure functions, no document concepts |
| L1 document | line collection, relation graph, channel table (three-level attributes), payload library, external references | the document model proper |
| L2 evaluation engine | topological-order evaluation of the relation DAG, time sampling, dynamics integration | deterministic core; layered parallelism |
| L3 rendering and compute | CPU oracle reference renderer + GPU (wgpu/WebGPU) | variable-width-line triangle strips, closed-line fill tessellation, binding vertex blending, compute simulation / parallel evaluation |
| L4 interaction | command registry, JSON command stream, MCP server, AI host | every operation is a command ([ai-interaction.md](ai-interaction.md)) |

### 2.1 The division of labor made concrete: hot-path kernel + relation IR (the implementation face of vision §5)

The boundary of "more capability to AI, engine focused on performance" lands in engineering as three items:

1. **A slimmer predefined-relation library**: the engine keeps built-in implementations only for the
   **per-frame hot loop** — skinning (binding), simulation solving (springs/particles/collisions),
   track sampling (interpolation), rendering; no other "features" are written into the engine — they are
   carried by **AI-generated custom relations** instead (auto-rigging / lip sync / follow etc. = relation
   networks the AI compiles at creation time, not engine features);
2. **Relation IR**: the function body of a custom relation is not an interpreted script but a restricted
   intermediate representation (sandboxed WASM or an expression IR): deterministic and verifiable, statically
   analyzable by the engine (dependency declarations/purity), and able to degrade to built-in implementations
   on hot paths; **AI generates → passes the determinism gate (byte-equal command replay) → only then stored**;
   once stored it is frozen as data (re-evaluation never calls the AI again);
3. The runtime never calls AI (a veto, see [vision.md](vision.md) §6); offline playback = pure engine.

## 3. GPU strategy (dedicated section)

owner: "线和线关系其实就可以最大化地利用 GPU,概念契合,提高性能。"

*("Lines and line relations can in fact maximize GPU utilization — the concepts fit, and performance improves.")*

**Core argument: conceptual unification ⇒ isomorphic data ⇒ maximized GPU throughput.**

1. **Unified data layout**: node-level attributes = SoA arrays (one array each for positions/widths/colors/values),
   directly mapped to vertex attributes/storage buffers, uploading the whole scene at once;
2. **Rendering**: variable-width lines → unified triangle-strip expansion; closed lines → fill tessellation;
   isomorphic data = very few draw calls, zero pipeline-state switching (versus traditional software toggling
   back and forth among heterogeneous bitmap/vector/text objects);
3. **Evaluation**: the relation DAG is evaluated in parallel by layers — independent relations within the same
   layer run in parallel; node-level channel sampling is a parallel map;
4. **Simulation**: particles = directed segments = instanced rendering + compute integration;
   springs/constraints = PBD-style parallel solving;
5. **Skinning/mesh deformation**: binding = vertex blending, which the GPU vertex stage is naturally good at.

**Dual-track discipline** (following maohou's CPU oracle + wgpu dual-track practice[^maohou-readme]):
the CPU oracle renderer/evaluator is the **deterministic source of truth**, and the GPU implementation =
an accelerator; the two outputs must be identical (a per-pixel/per-value gate). In the MVP phase we start on
CPU while the SoA layout keeps the GPU path open (see [mvp.md](mvp.md) § trade-offs).

### 3.1 External reference: BUI's performance design (assessed 2026-11-11 after the owner asked)

The high-performance design of [BUI](https://github.com/ppmpretham/BUI) (a C library decoupled from Blender's
UI, GPL-3.0) is highly isomorphic to mo2d's; five points of **clean-room borrowing**[^bui]:

1. **Damage rendering**: a change marks only its own rectangle dirty (merge cap of 24);
   sleeps when nothing is dirty (0% CPU when idle); the CPU rasterizer compares 64×32 tiles against the
   previous frame, and only damaged tiles go to screen — mo2d's editing-viewport presentation layer follows
   this; **the evaluation-side analogue = reverse-dependency invalidation of the relation DAG**:
   a channel change recomputes only the dependent subgraph (BUI's single-node signal update is the same idea,
   "channel = signal");
2. **Instanced uber-shader**: each primitive = one 128-byte instance of a single shader, and a whole frame
   needs only a few instanced draw calls — the **concrete implementation shape** of "everything is lines ⇒
   isomorphic data ⇒ few draw calls": segment/node patches = instances, fitting the SoA layout exactly;
3. **Same pixels from two renderers**: the CPU rasterizer and the GPU shader evaluate the **same formula**;
   parity gate (out of 172,800 pixels only 10 differ by >3/255) — proof of feasibility for the dual-track
   discipline and the acceptance-gate form (a pixel-threshold parity gate);
4. **Zero rendering when idle + animations run only while something moves (paced by the display) + a headless
   virtual clock stepping animation precisely** — the primitive for mo2d's deterministic playback / regression
   testing (headless, reproducible frame by frame);
5. **Images decoded at drawn size, decoded off-thread, cached under a memory budget (default 256 MiB), decoding
   cancelled when off-screen** — a management reference for mo2d's media payloads (bitmap mesh textures).

**Limitations**: GPL-3.0 (Blender-derived code) — borrow the design ideas only, **do not reference the
implementation** (avoiding license contagion, keeping the MIT/Apache lean); GPU backends are only D3D11/GL3.3
(no Metal/WebGPU) — the instancing pattern can be carried over to wgpu/WebGPU but the code cannot; BUI is a
widget-level UI toolkit, and mo2d's "stage as interface" has no use for its widget tree/layout/CSS bulk.

## 4. Storage-format principles

**For the details and envelope draft see [file-format.md](file-format.md)** (established 2026-11-11; this
section keeps a summary of the principles):

- JSON envelope + optional trailing BLOB area (large payloads not base64); **a single .mo2d file is itself the
  deliverable package** (delivery equals project); an optional project-directory (unpacked) form
  (document.mo2d.json + assets/) is equivalent to the package;
- **Time = bare Tick integers** (254,016,000,000 per second, empirically verified by maop); fps is only a
  display parameter;
- **Byte-level determinism**: stable field order, no timestamps in the body, atomic writes;
- **Unknown-key passthrough + extension namespaces** (glTF-style; a promise not to strip foreign keys — the
  Moho lesson);
- **Command history embedded but separate**: document is the truth, history is only for audit/replay;
- **Assets as external references**: file:// + a relative base; preview optional and excluded from the
  canonical hash;
- JSON Schema + validator; FORMAT_VERSION with per-version reader migration (vectorcraft/AniCraft practice);
- Extension: `.mo2d` (to be decided).

## 5. Tech-stack lean (to be decided)

**Leaning toward Rust + wgpu/wasm (WebGPU)**, because:
- engineering experience from the existing ecosystem (vectorcraft/maohou) is directly reusable: layering
  discipline, command registry, MCP protocol, deterministic-evaluation
  culture[^maohou-readme][^vectorcraft-arch];
- the wasm target = runs directly in the browser; cheapest for distribution and AI-host integration;
- `unsafe` discipline and clean-room processes can continue.

The concept documents ([line-model.md](line-model.md)/[relations.md](relations.md)) stay stack-agnostic;
if the stack changes, only §2-§4 of this document need rewriting.

## 6. Open Questions

- [ ] Crate division (to be decided when MVP work starts; the first version may begin as a single crate);
- [ ] The evaluation order of track lines (timelines) and where they converge with geometric evaluation;
- [ ] A deterministic integration scheme for dynamic relations (simulation) on the GPU (compute floating-point consistency).
