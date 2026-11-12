---
type: Design Document
title: "Relations are Functions: The Predefined Relation Library and Custom Relations"
description: mo2d's behavior model — relation = function (takes lines and parameters as input, outputs derived structure and values); sets/layers/groups = predefined relations, relation parameters = set parameters; the nine-category predefined relation standard library, custom relations, DAG evaluation and conflict handling.
tags: [relation, function, linkage, evaluation]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T10:55:00Z }
sources:
  - id: m-bone
    resource: file:///Users/wei/maohou/plan/moho/m-bone.md
    title: "mohoscripting · M_Bone 类归档(maohou 知识库)"
  - id: mohoproj-format
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 字段全表(maohou 知识库)"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
---

**English** | [简体中文](relations.zh-CN.md)

# Relations are Functions

Lines are data; relations are behavior. **Relation = function**: takes several lines (and parameters) as input, and outputs derived structure and values.
Document = the relation graph formed by all lines + all relations; animation = frame-by-frame evaluation of the relation graph over time.

## 0. Normative Statements (owner-given, verbatim)

> **集合本质是线和线的关系,只不过是预定义关系;关系就是函数;关系的参数就是集合的参数。**
> *A set is, in essence, a relation between lines — merely a predefined one; a relation is a function; a relation's parameters are the set's parameters.*
>
> **模拟过程本质是线和线的关系。**
> *A simulation process is, in essence, a relation between lines.*
>
> **嵌套层、父子层本质是线与线的关系映射。**
> *Nested layers and parent-child layers are essentially relation mappings between lines.*

Hence: layers/groups are not container types but **organizational predefined relations**; "a layer's opacity/blend mode"
is a parameter of that relation function; document-level parameters such as "frame rate/color space" are parameters of the **root relation**.

## 1. Predefined Relations (the Standard Relation Library)

| Class | Relation | Semantics (function-signature sketch) | Parameters (i.e. "the set's properties") — examples |
|---|---|---|---|
| Organization | **layer** | (set of lines) → an ordered renderable unit | Visibility, opacity, blend mode, transform (box line) |
| Organization | **group** | (set of lines) → a named package | Name, expanded state |
| Organization | **ownership / hierarchy mapping** | (parent relation, child line/child relation) → nesting | Depth/occlusion ordering parameters |
| Connection | **node sharing** | (node of line A ↔ node of line B) → the same node | Sharing mode (skeleton polylines, warp/weft mesh crossings) |
| Binding | **bind** | (node, bone line/mesh line/control bar) → the node moves with the line | Weights, falloff |
| Constraint | **constraint** | (line A, line B) → geometric restrictions | Angle min/max, equal length, tangency |
| Drive | **propagate** | (control line, target line set) → targets follow the control | Drive mapping, value range |
| Generate | **derive** | (input line set) → an output line | Generation parameters for medial axis/envelope/loft/text→outline |
| Time | **track relation** | (track line, target attribute) → sampled and written every frame; **the track itself = a line** (nodes = keyframes: position = tick, value = node-level attribute) — the track is data, the track relation is the wiring (decision in [mvp.md](mvp.md) §2.2) | Interpolation/easing = the track line's segment-level attribute (per-component handles) |
| Dynamic | **simulate** | (line set, force field) → the line set after frame-by-frame integration | Spring/damping/gravity/wind/collision/particle emission rate; random seed |
| Reference | **external asset reference external-ref** | (out-of-document entity) → an anchorable payload | file:// path, relative base |

Grounding in Moho semantics:
- Angle constraints: Bone `fConstraints`/`fMinConstraint`/`fMaxConstraint`[^m-bone];
- Smart bones (Action dials driving multi-channel lookup tables[^m-bone]) → a special case of the propagate relation, generalized;
- Bone dynamics (`fBoneDynamics` + the spring/damping/force parameter families[^m-bone]) → parameters of the simulate relation;
- Point's `parent` bone index[^mohoproj-format] → the bind relation;
- `fileref{path,relativeTo}`[^mohoproj-format] → the external asset reference.

### 1.1 Landed forms in the engine (Phase B, 2026-11-12)

The first engine slice of the library is live (command `relation.create`):
`bind {target, skeleton, falloff?, stretch?}`, `share {driver, driven}`,
`track {trackLine, target{line, node?, attr: position|width|color}}` — plus
the Phase B vocabulary:

| Relation | Signature as implemented | Notes |
|---|---|---|
| **gradient** | (bar line, target line, role `fill`\|`stroke`) → per-node colors | The bar's nodes are the color stops (node color = stop color, projection on the bar chord = stop position); target nodes sample the ramp by the same projection. Stops animate through **color tracks** (track attr `color`) — this is the "color-changing tentacle" acceptance (roadmap B3) |
| **group** | (box line, members[], `visible`) → transformed member lines | The box's rest→current similarity (anchor = first node, reference = last; 2D rotation + uniform scale, z scaled, widths scaled) applies to every member node; `visible` is the layer-visibility parameter; nested groups compose (an inner group's box may be an outer group's member) — roadmap B4 |
| **sim** | (line, anchors[{node, driver}], gravity, damping, iterations, collideWith?, radius?, selfCollide?) → integrated line | Verlet + distance constraints at fixed 240 Hz substeps; anchors ride their driving position tracks exactly per substep; **replay-from-seed** determinism (§4); collisions: free nodes keep clearance from collider segments and (optionally) non-adjacent chain pairs keep separation — roadmap B6 |
| **particles** | (emitter line/node, rate, speed, dir, spread, life, gravity, damping, size, color, len, **seed**, collideWith?, bounce?, radius?) → generated segment lines | Each particle = a short directed segment (`<emitter>#p<k>`); scatter angles hash through splitmix64 on the explicit seed — same doc + same seed → byte-identical output; collider segments reflect the inward velocity with restitution `bounce` — roadmap B6 |

One writer per output still governs: gradient claims a recolor channel of a
line (fill and stroke roles compose), group/sim claim their members'/line's
positions; a second writer on the same claim is rejected by the validator.

## 2. Dynamic Relations (Simulation)

Owner normative statement: **a simulation process is, in essence, a relation between lines** — not a concept outside lines, just relations
**computed frame by frame by a solver**, rather than laid out directly by the author:

- **Springs/damping**: act between bone lines and bone lines — "dynamic constraints";
- **Gravity/wind**: fields acting on lines, expressed as relations between lines and (virtual) lines;
- **Collision** *(landed, B6)*: distance relations between lines — sim nodes keep a clearance radius from collider segments, chains can self-separate, and particles bounce off colliders with explicit restitution;
- **Particles**: the emitter is a line; each particle is a short **directed segment** (position + velocity vector);
  attraction/repulsion/attachment are also relations between lines.

The solver code itself is **logical payload** (non-line boundary category 2, see [line-model.md](line-model.md) §4),
but the **structure** of the simulation lives entirely in relations between lines. Determinism requirement: the random seed must be made an explicit parameter;
same document, same input → same output (verifiable by the CPU oracle).

## 3. Custom Relations

Functions defined by users/AI: the function body is logical payload (expressions/scripts), the parameters are channels.
This is mo2d's single outlet for extensibility — **add no new kernel primitives, only new relations**.

- AI generates from natural language: "make the tentacle tip follow the cursor" → generates a custom propagate/simulate relation;
- Reverse explanation: with any relation selected, AI can translate its parameters and behavior back into natural language;
- Custom relations can be promoted into the library as predefined relations (the extension mechanism the standard library opens).

## 4. Evaluation Model

- Relations form a **DAG**: topologically sorted, then evaluated layer by layer;
- **Layered parallelism**: independent relations within the same layer run in parallel; node-level channel sampling is a parallel map —
  isomorphic to the GPU execution model (owner: lines and line relations maximize GPU utilization; the concepts fit);
- **Time dimension**: track relations first sample the current frame's parameters, then geometric/dynamic evaluation proceeds;
  dynamic-relation state integrates over time — **the playback/scrub consistency strategy is resolved (B6): replay-from-seed**.
  Every evaluation re-integrates dynamic relations from tick 0 in fixed 240 Hz substeps (254,016,000,000 ticks / 240 = 1,058,400,000
  exactly), with anchors sampled per substep from their driving position tracks. Cost is O(tick/240 × nodes) per evaluation — bounded,
  and scrub-back/replay/offline export are byte-identical by construction;
- **Cycles and conflicts**: cycles are in principle forbidden in the relation graph (detected → error); multiple writing relations on the same node
  are composed by relation priority + a deterministic order (details during engineering; the principles recorded now: **determinism, explainability, parallelizability**);
- **Reverse-dependency invalidation (landed, B1)**: the DAG doubles as the change-propagation index — an edit to a line dirties exactly its
  transitive downstream (channels = signals; `engine/src/inval.rs`).

## 5. Open Questions

- [ ] Refining the parameter tables of each predefined relation (the first version takes the minimal set used by [mvp.md](mvp.md) as authoritative); — the landed Phase B forms are recorded in §1.1, fuller tables still pending;
- [ ] Whether the propagate relation supports a "lookup table" form (the smart-bone equivalent) in the first version, or is deferred;
- [x] ~~The playback/scrub-back consistency strategy for dynamic relations~~ — **resolved 2026-11-12 (B6): replay-from-seed** (§4); if interactive cost ever demands incremental state, the replacement strategy must reproduce byte-identical results against this baseline.
