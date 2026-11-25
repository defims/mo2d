---
type: Design Document
title: "Reference objects & derivation chains — video layer, white-model pipeline, fill reconciliation"
description: mo2d design resolution — "reference" is a role, not a type (any entity may serve as reference and continue deriving); the video layer (frame box + payload + time-remap track, own timeline); the white-model pipeline (video → parts/joints/skeleton/tracks as an AI-in-the-loop derivation relation); derivation chains with live-link vs snapshot; fill kinds reconciled as components (query-modify unified, ownership split).
tags: [reference, derivation, video-layer, white-model, fill, reconciliation]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-25T16:18:23Z }
sources:
  - id: h2o-survey
    resource: /h2o-decompose-survey.md
    title: "Survey: h2o-video-decompose (the measurement stage of the white-model pipeline)"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
  - id: arch-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§2.1.1 表示边界、§2.2 组件化存储)"
---

# Reference objects & derivation chains

Resolutions from the owner's sessions of 2026-11-22/23 (video-as-reference, white-model
rig pipeline, the zip/json-family question, fill-kind derivation, and the ECS follow-up).
All of it lands inside the existing primitives — **zero new concepts**: lines, relations,
tracks, media payloads.

## 0. Normative statements (owner, verbatim)

> **视频和参考球(人物头部转面参考用)类似,都是可参考对象之一;视频层,有自己的时间线。**
>
> **可参考的不止视频和 3d 球体,任意矢量图形本身也可以作为参考继续派生。**
>
> **画条封闭线,可以创建图形;图形填充可以纯色,可以渐变,也可以视频,以后可以扩展其他——类似 html 的 video、img 标签。填充渐变就会派生出渐变控制线,视频会派生出时间线。**
>
> **由于 mo2d 是查改一体,派生实际就是 ecs 里处理 component 的方法处理的。**

(The reference ball belongs to the 3D session per D11 — the 3D sphere as a head-turn
reference, represented as 2D silhouette lines / turn tables. The video layer joins that
family as its temporal-media member.)

## 1. Reference is a role, not a type

In the data layer, content and reference are the same thing — lines + relations. The
difference is a **role annotation**: any entity may serve as

- final content (rendered),
- reference (underlay/hidden, consulted by humans and by AI vision),
- derivation source (input to derive relations, producing new content),

and derivation chains continue: video → white model → styled character → outfit variants.
Each link is a derive relation; each intermediate may serve as the next reference. The
chain is a DAG (already enforced); a **lineage view** over the relation graph makes
"what derived from what" visible (no new storage — inputs/outputs already record it).

**Family members so far**: 3D reference sphere (D11: represented as silhouette lines /
turn tables), video layer (temporal media), any vector graphic (derivation source),
white model (derived reference), imported SVG (derivation source).

## 2. The video layer — a reference object with its own timeline

```
video layer = frame-box line (4-node closed line; placement/scale)
            + video payload (media field; external ref / BLOB)
            + time-remap track (document Tick → video Tick; animatable)
```

- **Own timeline**: the mapping is a track relation over time itself — slow-motion /
  reverse / loop / offset are keyframes on the mapping (Moho ImageLayer, AE time-remap,
  Remotion `<Sequence>` precedents; bitmap-mesh single-quad is the same anchoring shape
  as B5, with a moving texture);
- **Reference-live workflow**: the underlay's current frame follows the document timeline
  — AI vision sampling and user tracing always see "the reference frame at this moment";
  the mapping track can be slowed 2× for rigging fast motion (the standard animator
  trick), or looped;
- The payload is a media field (non-line boundary class 1); semantics (parts/joints) are
  never parsed into the format — they are derived relations on top (§3).

## 3. The white-model pipeline — an AI-in-the-loop derivation relation

The reference video → white-model rig conversion is expressed as **one derivation
relation** whose input is the video layer (AI vision inside the measurement stage,
scripts adjudicating, following the h2o methodology — see
[h2o-decompose-survey.md](h2o-decompose-survey.md)):

1. **Part segmentation**: sliding-window affine fits; pixels sharing consistent affine
   motion cluster into parts (head / torso / upper arm / forearm / …) — cutout-style
   rigid parts;
2. **Part reconstruction**: one clean frame per part → silhouette → traced into a mo2d
   closed line with flat fill (the white model; satisfies the no-source-pixels rule);
3. **Joint inference (the mathematical core)**: for adjacent parts B over A, the
   relative transforms `T_rel(t) = A(t)⁻¹·B(t)` are rotations around the joint — **the
   joint is the fixed point**, solved sub-pixel from two time samples; part hierarchy
   from relative-motion complexity;
4. **Skeleton = the polyline through joints** (shared nodes — the owner's normative
   form); **bind relations** attach part outline nodes to their bone;
5. **Motion migration**: per-part affine sequences decompose into the owning bone's
   angle/position channels → track lines with Tick keyframes (the h2o measurement table
   is exactly a keyframe table).

**Boundary honesty** (h2o's L1/L2/L3 adopted): rigid cutout parts = 🟢 full pipeline;
perspective flips (3D head turns) = 🟡 — the white-model assumption breaks → **turn
tables**: one white model per facing, switched by a String track (switch-track semantics);
soft deformation (cloth/hair) = 🔴 for the rigid pipeline → B6 simulation tails or
switch-postures; occluded shapes = AI completion highlighted as inferred + user
confirmation.

## 4. Derivation chains — live-link vs snapshot

Deriving from a reference creates a chain (video → white model → styled → variants).
One new design point, a **parameter of the derive relation**:

| mode | behavior | use |
|---|---|---|
| **live-link** | reference updates propagate into the derivation | white model follows the reference video (re-time the reference, the rigging alignment follows) |
| **snapshot** | frozen at derivation time, independent afterwards | outfit variants (editing the variant must not pollute the reference) |

Precedents: GPv3 drawing instancing, Rive artboard references. Cycles stay forbidden
(DAG rule). The lineage view (§4 of the relation graph) surfaces the chain.

Every link on the chain is an **intermediate value** — a relation output as a first-class entity, with whole/member/snapshot modes (see [relations.md](relations.md) §1.0.2).

## 5. Fill kinds = component reconciliation (query-modify unified)

**Draw a closed line → a shape; the shape's fill declares its kind — the machinery
derives itself**, like HTML's `video`/`img` tags:

| fill kind | derives | HTML analogue | status |
|---|---|---|---|
| `solid` | fill color attribute only | `background-color` | landed |
| `gradient` | **gradient control-bar line** (nodes = color stops) + gradient relation | `linear-gradient()` | bar+relation landed; the UX command is new |
| `video` | **video layer** (§2) + fill-sampling relation (shape clips the video) | `<video>` | components exist (B5 + video layer); assembly pending |
| `image` | image asset + UV anchoring (bitmap-mesh single quad) | `<img>` | B5 |
| `pattern` / `shader` / `audio-reactive` | pattern line arrays / custom-relation body / input-line driven | — | future registry entries |

**Because mo2d is query-modify unified, derivation is not a special pipeline — it is a
reconciliation system processing components** (the owner's formulation):

```
Shape entity:  FillComponent { kind, ref: derived-entity id, params }
DeriveSystem(kind): on FillComponent change → ensure the derived entities
                    exist and are linked (create if missing, repair if broken)
```

**The ownership split (the one big pit, closed by one rule)**:

- the derive system owns **existence and linkage** only (idempotent reconciliation);
- the **user owns the values** of derived entities (control-bar nodes, video src, time
  mapping keyframes) — reconciliation never overwrites them.

This is the React props/state distinction. Benefits: `fill.set` shrinks to "set the
FillComponent" (everything else is system reaction); undo/redo unchanged (component-set
flows); determinism preserved (reconciliation is deterministic given component state);
extensibility = register one system per kind.

## 6. Roadmap placement

- **C9 Reference layers & white-model derivation** (roadmap): video layer + the §3
  derivation relation + live-link/snapshot; depends on B5 (media payload) and the landed
  bind/track relations;
- fill-kind registry: `gradient` assembly is a thin UX command over landed B3;
  `video`/`image` fills land with C9/B5.

## 7. Open questions

- [ ] The expr/body language minimal grammar (C1) — what the FillComponent params may
      call;
- [ ] Host input-line interface for external variables (cursor/audio/gamepad) — payload
      or track?
- [ ] Multi-video-layer reference mixing (two references, different time mappings) —
      does the white-model derivation take a primary/secondary?
