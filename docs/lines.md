---
type: Design Document
title: "Lines All the Way: Space, Time, Generation, Stream"
description: The four-clause ontology that unifies the whole engine on one noun and two verbs — everything is lines (space), motion is walking (timelines), generation is slicing (lift/view), streams are the trace (frame/event streams); fold and slice are the only two universal operators, and determinism is walkability. Descriptive synthesis, not a protocol change; grounds a future generation-relation slice (numbering open — C8 is taken by gesture timelines).
tags: [lines, ontology, timeline, fold, slice, streaming, events, generation]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-17T14:30:41Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线(空间线:线、节点、三级属性、九大映射)"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数(§1.2 事件也是命令,§4 事件维度)"
  - id: architecture-doc
    resource: /architecture.md
    title: "本 bundle:架构草稿(§3.2 流式渲染:惰性帧流=把 tick→frame 纯函数变现)"
  - id: vernacular-doc
    resource: /vernacular.md
    title: "本 bundle:生活概念层(§6 验收门 N1–N5,N5=概念预算 ≤25 词根)"
  - id: roadmap-doc
    resource: /roadmap.md
    title: "本 bundle:路线图(C7 交互、C8 手势时间线;生成关系切片编号未定,本文件铺其地基)"
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:MVP v5(D7 Tick 时间:时间真值是原始 tick 整数)"
  - id: log-doc
    resource: /log.md
    title: "本 bundle:变更日志(2026-11-17 hull 探针:转面→视觉外壳→三轴派生时间线)"
---

**English** | [简体中文](lines.zh-CN.md)

# Lines All the Way: Space, Time, Generation, Stream

This document is a **descriptive synthesis** (postdictive unification, not a new
mechanism): it states what the engine already is, in one noun and two verbs, and
records where each clause is already grounded in shipped code and gates. It
proposes **no protocol change**; its only forward commitment is conceptual
groundwork for a candidate generation-relation slice (C8)[^roadmap-doc].

## 1. Owner's normative statements

Recorded verbatim (Chinese original; English translation follows each):

> 一切皆线

("Everything is lines." — the project motto, foundational for the line model[^line-model-doc].)

> 一条时间线可以是另外时间线的生成线

("One timeline can be the generating line of other timelines." — stated 2026-11-17,
while reviewing the visual-hull probe; grounds the generation clause.)

> 流也是线

("A stream is also a line." — stated 2026-11-17, completing the four-clause
ontology; grounds the stream clause.)

From these the session synthesized a four-clause slogan (agent wording, not an
owner quote): **万物皆线,动即行走,生即切片,流即行迹** — everything is lines,
motion is walking, generation is slicing, streaming is the trace.

## 2. Four clauses, four levels of lines

| Level | The line | Where it already exists |
|---|---|---|
| Space | Drawn lines: strokes, contours, skeletons | L0 model — Line/Node, the nine mappings[^line-model-doc] |
| Time | Timelines: `t ↦ lines`, a sweep through shape space | Parameters + track sampling; Tick truth is raw integers (D7)[^mvp-doc] |
| Generation | Generating lines: operators mapping timelines to timelines | `lift`/`view` in the hull probe (turnaround → visual hull → three derived timelines)[^log-doc] |
| Stream | Traversed lines: frame stream = sampled walk of a timeline; event stream = fact line ordered by (tick, seq) | §3.2 streaming renders the lazy frame stream[^architecture-doc]; C7 folds the event prefix[^relations-doc]; C8 P1 already instantiates the clause — the event stream is dual-written into read-only step-track lines (`gesture/<target>`), the fact line becomes an evaluable line[^roadmap-doc] |

The difference between a line and a stream is only discretization and traversal:
a timeline is the continuous sweep, a frame stream is its per-tick walk, an
event stream is the discrete line of irreversible facts. **Determinism is
walkability**: the walk is a pure function, so the same position always yields
the same frame — replay is re-walking from the origin, seek is repositioning on
the line, and frozen hashes are fixed points pinned along the walk (A1 gate,
P1-7 policy[^architecture-doc]).

## 3. The two universal operators

Every mechanism above decomposes into two operations on lines:

**fold** — integrate along a line; raises dimension (line → state/artifact/envelope):

| fold instance | Already shipped |
|---|---|
| Event prefix fold → parameter state | C7: evaluation is a pure `f(doc, tick, eventPrefix)`[^relations-doc] |
| Whole-timeline fold → hull artifact | Hull probe: `lift` carves 12 silhouettes into a slice stack[^log-doc] |
| Sim replay from tick 0 | B6: re-walk with fixed substeps |

**slice** — cut across a family; lowers dimension (envelope → line):

| slice instance | Already shipped |
|---|---|
| Timeline at tick → frame | `evaluate(doc, tick)` — any frame re-evaluable independently |
| Solid at pose → derived timeline | Hull probe: `view` under three pose trajectories[^log-doc] |
| Three-view drawings | Special-case slices (n=3 frames of the same turn) |

The probe and the event system are the same shape twice: **slice ∘ fold**.
The economics follow from the composition — fold once (expensive, baked into an
artifact, like a sim checkpoint or a frozen reference), slice endlessly (cheap,
lazy per demand). This is the same cache structure as §3.2's lazy frame stream
and the damage-rendering cache layer[^architecture-doc]: derived timelines are
lazy views of the solid, never materialized wholesale.

## 4. Concept budget

The ontology costs **one noun and two verbs**: line, fold, slice. Everything
else — animation, streaming, events, seek, replay, generation — is a walked,
sliced, or folded line, not a new primitive. This serves the novice-gate
budget directly (N5: ≤ 25 word roots[^vernacular-doc]); the novice phrasing
belong to vernacular.md and are only sketched here: an animation is a line
walked to the end; dragging the playhead is jumping on the line; recording is
drawing the fact line; an export is the walk written down.

## 5. What this grounds, and what it does not

Grounds: a candidate **generation-relation slice** (numbering open — C8 is
taken by the gesture timelines, established the same day) — a
timeline-to-timeline relation with `lift` (timeline → baked artifact,
deterministic, provenance-carrying) and `view` (artifact × pose trajectory →
parameters), fitting the existing gate culture byte-for-byte (the probe is the
existence proof[^log-doc]).

Does **not** do: expose fold/slice as user-facing primitives now; touch the
command protocol, the format, or the validator; nor claim that any two
timelines are generable — generation requires a shared underlying model
(same solid, two trajectories), which is exactly what the probe's containment
and carve-monotonicity gates assert. Known hard boundaries carry over from the
probe: silhouette-visibility (hull ⊇ true object), partial-turn
under-constraint, and non-rigid authoring cheats stay out of scope until a
dedicated slice document decides them.

[^line-model-doc]: The spatial line model: lines, nodes, three attribute levels, nine mappings.
[^mvp-doc]: Frozen decision D7 — time truth is raw tick integers; fps is display-only.
[^relations-doc]: C7 event dimension — events are irreversible facts; the prefix folds by (tick, seq); empty prefix reduces to the previous behavior.
[^architecture-doc]: §3.2 streaming rendering — the lazy frame stream; A1/P1-7 hash-freeze + eyeball policy; cache-as-cache-layer.
[^vernacular-doc]: N1–N5 novice gates; N5 caps the concept budget at ≤ 25 word roots.
[^log-doc]: log.md 2026-11-17 — the hull probe: turnaround → visual hull lift → three derived timelines, frozen as `hull`, gate_hull six assertions.
[^roadmap-doc]: C7 interactivity shipped; C8 (interactive gesture timelines) established by the owner and landed as C8 P1 the same day; the generation-relation slice has no number yet — this document only prepares its ground.
