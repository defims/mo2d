---
type: Design Document
title: "Relations are Functions: The Predefined Relation Library and Custom Relations"
description: mo2d's behavior model — relation = function (takes lines and parameters as input, outputs derived structure and values); sets/layers/groups = predefined relations, relation parameters = set parameters; the predefined relation standard library (now incl. interaction: respond + oscillate, established 2026-11-17), custom relations, DAG evaluation, event-prefix folding and conflict handling.
tags: [relation, function, linkage, evaluation, interaction, events]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-25T16:18:23Z }
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
>
> **就像ae有合成层一样,视频、图形、群组其实是中间结果当作整体来处理的,相当于中间变量。**
> *Like AE's precompositions: video, shapes and groups are intermediate results treated as wholes — intermediate variables.*
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
| Time | **track relation** | (track line, target attribute) → sampled and written every frame; **the track itself = a line** (nodes = keyframes: position = tick, value = node-level attribute) — the track is data, the track relation is the wiring (decision in [mvp.md](mvp.md) §2.2) | Interpolation/easing = the track line's segment-level attribute (per-component handles); interpolation codes follow Moho (moho-mapping §5.2): **0 linear / 3 step / 9 Bézier** — step (im=3) is the discrete parameter, the basis of Switch/state machines (established 2026-11-17, roadmap C7) |
| Dynamic | **simulate** | (line set, force field) → the line set after frame-by-frame integration | Spring/damping/gravity/wind/collision/particle emission rate; random seed |
| Drive | **oscillate** | (target attribute) → a periodic value of tick: base + amplitude × wave(tick − phase); the deterministic cyclic driver (breathe/sway/float/spin) — physical sway stays with simulate | Waveform (sine/triangle/saw), period (ticks), phase, base, amplitude (established 2026-11-17) |
| Interaction | **respond** | (event × guard) → ordered actions: **write** a parameter {target line+attr, value \| add} / **fire** an event; events ∈ {tap, press, release, hoverIn, hoverOut, overlap(lineB), timePoint(t)}; **a state machine = a discrete parameter + guarded respond relations**; a listener = a respond relation on a target line — no new concept (established 2026-11-17, see §1.2) | Event vocabulary, guard {channel, op, value}, actions list, hit-test radius, `minHold`/`maxHold` (release-class hold discrimination — C8, see §1.2) |
| Generate | **shape (fill region)** | ([curve-ref | segment-ref (with direction flag)], …) → a renderable region; **one curve may be shared by N shape relations** (relations only read lines; curve reuse = several relations with the same input) | fillOn/strokeOn, fill rule, style, **zOrder (a parameter — stacking is animatable)** |
| Reference | **external asset reference external-ref** | (out-of-document entity) → an anchorable payload | file:// path, relative base |

### 1.0.1 Line/shape split — the revised shape relation (established 2026-11-13; option A decided by the owner 2026-11-24)

**The essence of "one closed curve, multiple shapes": one curve + a group of shape
entities — structurally identical to layers. Line and shape are split apart.**

- **Line = pure geometry**: ordered nodes + segments/handles, carrying **no fill/stroke
  semantics**;
- **Shape = an independent entity**: boundary reference (whole curve / a segment with
  direction / multi-curve compound with holes — Moho's `edges {curve, segment, flag}`
  [^mohoproj-format]) + **paint (fill + stroke) + zOrder**;
- **One curve, N shapes** = the curve referenced by N shape entities (edit the curve
  once, all N shapes change); the curve no longer carries fillOn/strokeOn — strokes live
  on shapes too (Moho's "one curve, two roles" returns in a more thorough form);
- **Bare-line sugar**: an open line's stroke is auto-materialized by the engine as an
  **implicit shape** (explicit once edited) — lines are pure geometry, all paint lives on
  shapes (option A, decided by the owner 2026-11-24);
- Isomorphism principle: **geometry (lines/nodes) and semantic organization
  (layers/shapes/skeletons) are orthogonal** — layers organize line sets, shapes
  organize "boundary + paint", skeletons organize nodes.

**Stacking order**:
- shape entities carry a **zOrder parameter** (Moho's `shape_order` is itself
  animatable[^mohoproj-format]) ⇒ **stacking can be keyframed** (card flips, occlusion
  reversals);
- three layers of convenience: the command `rel.set zOrder`; AI natural language; direct
  stage manipulation (the AI generates a temporary depth slider on the spot — a control
  bar is a line); semantic commands bringForward/sendBackward/toFront/toBack (=
  deterministic zOrder arithmetic);
- **two-level ordering composed lexicographically**: between layers = the ownership
  relation's ordering parameter; within a layer = the zOrder channel; global render
  order = `(layerOrder, zOrder)`;
- two overlapping fill regions over the same curve both render, ordered by zOrder
  (well-defined); the fill rule is a parameter of the shape relation (logic payload).

**Knock-on revisions (incremental)**: (1) line-level vocabulary shrinks —
fillOn/strokeOn/fillColor/strokeColor/lineWidth migrate to shape paint
([moho-mapping.md](moho-mapping.md) §7.1 note); (2) the gradient relation's target moves
to shapes (the landed engine version targets line fill/stroke roles — one refactor);
(3) deleting a curve leaves referencing shapes boundary-less → validator reports
"missing boundary".

Grounding in Moho semantics:
- Angle constraints: Bone `fConstraints`/`fMinConstraint`/`fMaxConstraint`[^m-bone];
- Smart bones (Action dials driving multi-channel lookup tables[^m-bone]) → a special case of the propagate relation, generalized;
- Bone dynamics (`fBoneDynamics` + the spring/damping/force parameter families[^m-bone]) → parameters of the simulate relation;
- Point's `parent` bone index[^mohoproj-format] → the bind relation;
- `fileref{path,relativeTo}`[^mohoproj-format] → the external asset reference.

### 1.0.2 Intermediate values: relation outputs as first-class entities (established 2026-11-25 by the owner)

**Owner normative statement**: "就像ae有合成层一样,视频、图形、群组其实是中间结果当作整体来处理的,相当于中间变量。"
*Like AE's precompositions: video, shapes and groups are intermediate results treated as wholes — intermediate variables.*

Every relation's output is a first-class addressable entity — video layers, shapes,
groups, white models, gradient compositions are intermediate results treated as wholes
(AE precomps):

- **Whole-level operations**: transform/effects/time-mapping act on the intermediate
  itself; composition transforms **never write into members** (live);
- **Member-level operations**: query-modify unified — members remain their own entities,
  openable and editable; the intermediate is a function of its members' current state;
- **Snapshot** (a parameter of the derivation relation, see
  [reference-derive.md](reference-derive.md) §4): freeze the composed result into
  independent content, decoupled afterwards;
- **Naming**: intermediates can be named — AI natural-language reference, debugging and
  export all use the name as the interface (h2o's named semantic points, same technique);
- **Cache boundaries**: invalidation is cut at intermediates (inputs unchanged →
  whole-block cache hit) — the natural cache hierarchy for B1's change detection;
- **Composition freedom**: intermediates keep participating in any relation
  (re-transform / re-clip / re-nest) — the relation graph IS the dataflow; AE
  precomp-in-precomp freedom with zero special mechanisms.

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

One writer per output still governs: gradient claims a recolor parameter of a
line (fill and stroke roles compose), group/sim claim their members'/line's
positions; a second writer on the same claim is rejected by the validator.
The interaction slice (**oscillate**, **step interpolation im=3**, **respond** +
the event stream) lands at roadmap **C7** — the contract is §1.2 below.

### 1.2 Interaction relations (respond; established 2026-11-17 by the owner)

The landscape survey (F11: perceive → decide → respond is the minimal interactive
model; the bool·number·trigger input triad recurs across Rive/Unity/Godot) plus the
owner's novice vision ("交互动画" is in the goal sentence) make in-file interactivity
an essential capability. It lands entirely inside "relations are functions":

- **Events are commands**: an interaction enters the document history as
  `event.fire {name, tick, target?, x?, y?}` — the replayed command stream
  includes behavior, so determinism (same doc + same stream + same events →
  byte-identical output) extends to event handling unchanged. Events are
  **irreversible facts**: the inverse of `event.fire` is `noop` (deleting an
  event would change everything downstream of it);
- **Event vocabulary v1**: `tap` / `press` / `release` / `hoverIn` / `hoverOut`
  (pointer against a target line, engine-side deterministic hit test = segment
  distance, same criterion as the stage), `overlap(lineB)` (segment intersection
  at the current tick), `timePoint(t)` (a moment arrived);
- **Guards**: a respond relation may carry a condition on a parameter
  `{channel, op, value}` — guarded transitions, exactly the Rive/Unity/Godot
  convergence (F11);
- **Actions** (ordered list): `write {target line+attr, value | add}` (a state
  machine = a **discrete parameter** written by guarded respond relations — the
  discrete values ride **step interpolation, im=3**, and are the Switch-layer
  equivalent) and `fire {name}` (chain reactions: one event triggering another);
- **Single-writer discipline holds**: a respond is a *conditional* writer — the
  validator rejects an action target (line+attr) that any unconditional writer
  (track/oscillate/bind/…) already claims. Composition pattern: respond writes
  a dedicated control parameter, time-driven relations consume it;
- **Plain-player interactivity (N3)**: respond relations evaluate
  deterministically in any offline player, no AI, no scripts (capability as
  data, vision rule 3) — evaluation semantics in §4;
- **v1 evaluation notes (landed with C7)**: hit tests and overlap checks use
  **rest geometry** (the document's un-animated positions — deterministic and
  cycle-free, which is what makes "tap on X writes X" legal); overlap edges
  are sampled at **fold moments** (recorded event ticks and declared
  timePoints), not every tick — a continuous overlap watch is a v2 refinement;
  multiple guarded responds may compose **one** parameter (the state-machine
  pattern): the fold sequences their writes by (moment, document order), and
  the first claimant applies the composed state exactly once per frame.

#### Gesture timelines (established 2026-11-17 by the owner — line-model §0 statement; **landed (C8), 2026-11-17**)

**Interactive events are timelines**: a gesture's phase edges (`hoverIn` / `press` /
`release` / `hoverOut`) are keyframes on a **gesture line** — a **step track line
(im=3)** named `gesture/<target>` (targetless broadcast events → `gesture/pointer`),
node value = the phase code (0 out / 1 hover / 2 pressed):

- **Recording is materialization**: `event.fire` becomes an **atomic dual-write** —
  append the irreversible `MoEvent` (**shape unchanged**: single tick, no duration,
  no phase field) + upsert the phase key on the gesture line. Keys land **only on
  phase changes** (same-value upserts are skipped). The line grows only with events:
  the validator reserves the `gesture/` namespace (user lines cannot take it, and no
  relation may write it), the stage refuses to drag/delete its nodes — an event is
  irreversible, so its materialized timeline is **read-only**;
- **Phase semantics follow the browser's hover/click split** — the line tracks
  *physical containment* (hover semantics), hold discrimination uses the *event pair*
  (click semantics: down + up on the same target completes regardless of path; the
  release hit test stays radius-based):

  | Physical state (inside radius? × button?) | Browser counterpart | Phase | Key lands when |
  |---|---|---|---|
  | outside · up | `pointerout` / no hover | 0 | leaving the radius |
  | inside · up | `pointerover` / `:hover` | 1 | entering the radius |
  | inside · down | `pointerdown` | 2 | pressing inside |
  | outside · down (wandered off mid-hold) | physically out — `pointerout` already fired | 0 | leaving the radius (re-entry restores 2) |
  | inside · release | click completes, still hovering | 1 | releasing inside |
  | outside · release | click does not complete | 0 | (already 0 — no key) |

  In the common no-wander case the pressed segment's length **is**
  `release.tick − press.tick` — the duration is a length on the timeline, not a
  metadata field; when the pointer wanders the two quantities split, exactly as the
  browser splits hover from click;
- **Hold-aware responds**: a release-class respond may carry `minHold` / `maxHold`
  (ticks) — "long-press vs short-tap" is a **length predicate on the gesture
  timeline**. The fold evaluates it by a deterministic lookback: pair the release
  with the latest unpaired press on the same target in the prefix (sorted by
  `(tick, seq)`; per-target pairing stack) — hold = `release.tick − press.tick`,
  pure arithmetic on recorded facts. `tap` stays as-is (recorder-synthesized
  "press + release without drag", `tap_jump` frozen demo untouched); conceptually
  it is the shorthand for a short, unmoved press→release pair;
- **Reads stay cycle-free**: v1 guards may read **only** gesture lines — they derive
  from events, not geometry, so "respond reads no lines" survives (event → gesture
  line → guard is one-way data flow). Generalizing track-guards to arbitrary track
  lines would break that argument and is deferred (§5);
- **Rendering = read-only reference lane**: the stage draws gesture lines as faint
  lanes on the time axis (session overlay; tags `track` + `gesture` — the offline
  renderer and the scene snapshot skip them exactly like `track` lines), the pressed
  segment's drawn length being the duration. Offline frames and frozen PNG hashes
  are untouched; picking (hit tests) ignores gesture lines.

## 2. Dynamic Relations (Simulation)

Owner normative statement: **a simulation process is, in essence, a relation between lines** — not a concept outside lines, just relations
**computed frame by frame by a solver**, rather than laid out directly by the author:

- **Springs/damping**: act between bone lines and bone lines — "dynamic constraints";
- **Gravity/wind**: fields acting on lines, expressed as relations between lines and (virtual) lines;
- **Collision** *(landed, B6)*: distance relations between lines — sim nodes keep a clearance radius from collider segments, chains can self-separate, and particles bounce off colliders with explicit restitution;
- **Particles**: the emitter is a line; each particle is a short **directed segment** (position + velocity vector);
  attraction/repulsion/attachment are also relations between lines.

The solver code is the **function body of the simulate relation** — inside the relation
(owner, 2026-11-17: "代码是关系里的,关系即函数"; the non-line boundary no longer lists code,
see the revised [line-model.md](line-model.md) §4), and the **structure** of the simulation
lives entirely in relations between lines. Determinism requirement: the random seed must be made an explicit parameter;
same document, same input → same output (verifiable by the CPU oracle).

## 3. Custom Relations

Functions defined by users/AI: the function body is logical payload (expressions/scripts), the parameters are drivable parameters (legacy alias: channels).
This is mo2d's single outlet for extensibility — **add no new kernel primitives, only new relations**.
The body lives *inside* the relation concept (owner, 2026-11-17: "代码是关系里的,关系即函数" —
the non-line boundary no longer lists code; see the revised line-model §4).

- AI generates from natural language: "make the tentacle tip follow the cursor" → generates a custom propagate/simulate relation;
- Reverse explanation: with any relation selected, AI can translate its parameters and behavior back into natural language;
- Custom relations can be promoted into the library as predefined relations (the extension mechanism the standard library opens).
- **Representation boundary**: custom relation bodies compile via the relation IR into a wasm execution cache
  (sandboxed, derived data, excluded from the canonical hash); external variables arrive as **input lines** the host
  writes per frame; determinism = document + recorded-input replay. Full resolution:
  [architecture.md](architecture.md) §2.1.1; component-column storage & LineCore: §2.2.

## 4. Evaluation Model

- Relations form a **DAG**: topologically sorted, then evaluated layer by layer;
- **Layered parallelism**: independent relations within the same layer run in parallel; node-level parameter sampling is a parallel map —
  isomorphic to the GPU execution model (owner: lines and line relations maximize GPU utilization; the concepts fit);
- **Time dimension**: track relations first sample the current frame's parameters, then geometric/dynamic evaluation proceeds;
  dynamic-relation state integrates over time — **the playback/scrub consistency strategy is resolved (B6): replay-from-seed**.
  Every evaluation re-integrates dynamic relations from tick 0 in fixed 240 Hz substeps (254,016,000,000 ticks / 240 = 1,058,400,000
  exactly), with anchors sampled per substep from their driving position tracks. Cost is O(tick/240 × nodes) per evaluation — bounded,
  and scrub-back/replay/offline export are byte-identical by construction;
- **Event dimension (established 2026-11-17, C7)**: evaluation is a pure function
  **`f(doc, tick, eventPrefix)`** — the recorded `event.fire` prefix up to `tick`, sorted by
  `(tick, seq)`, is **folded** into parameter state (guarded respond relations applying their writes),
  and the folded state feeds the topological evaluation. The fold is deterministic (same prefix →
  same state) and cheap (O(events)); the empty prefix reduces exactly to the time-driven
  `f(doc, tick)` — every existing gate and frozen hash is untouched. Seek = replay the fold
  (same philosophy as replay-from-seed; an optional per-tick checkpoint cache of the same class
  as the sim one may be added later if long event histories ever measure slow);
  gesture-line phase keys (C8) ride event ticks — they add **no** fold moments; a release
  respond with `minHold`/`maxHold` adds one deterministic lookback to the fold (per-target
  press pairing, §1.2);
- **Cycles and conflicts**: cycles are in principle forbidden in the relation graph (detected → error); multiple writing relations on the same node
  are composed by relation priority + a deterministic order (details during engineering; the principles recorded now: **determinism, explainability, parallelizability**);
- **Reverse-dependency invalidation (landed, B1)**: the DAG doubles as the change-propagation index — an edit to a line dirties exactly its
  transitive downstream (parameters = signals; `engine/src/inval.rs`).

## 5. Open Questions

- [ ] Refining the parameter tables of each predefined relation (the first version takes the minimal set used by [mvp.md](mvp.md) as authoritative); — the landed Phase B forms are recorded in §1.1, fuller tables still pending;
- [ ] Whether the propagate relation supports a "lookup table" form (the smart-bone equivalent) in the first version, or is deferred;
- [x] ~~The playback/scrub-back consistency strategy for dynamic relations~~ — **resolved 2026-11-12 (B6): replay-from-seed** (§4); if interactive cost ever demands incremental state, the replacement strategy must reproduce byte-identical results against this baseline.
- [ ] Whether track-guards generalize beyond gesture lines (v1 restricts guard reads to
  `gesture/*` to keep respond geometry-free, §1.2), and whether touch-style implicit capture
  (hold preserved when the pointer is dragged off) joins the desktop physical semantics —
  both await owner confirmation.
