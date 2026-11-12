---
type: Design Document
title: "3D Extension Path: What to Adjust and What to Do Now"
description: Conclusions of the 3D extension — all primitives survive (3D bone = line segment / skeleton = polyline / surface = line net); only four semantic rules are adjusted (closed-line fill generalized to triangular faces / surface = line net / occlusion becomes depth testing / camera = frustum wireframe) plus three new relations (projection/aim/pole vector); six preemptive decisions (Vec3 from day one, rotation representation, etc.) are made now; 3D does not enter the current three phases and is recorded as long-term Phase D.
tags: [3d, extension, vec3, line-net, roadmap-d]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
  - id: mohoproj-format
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 字段全表(translation/scale 本为 Vec3 通道、rotation 三通道、3d_mode 先例)"
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:MVP v4"
---

**English** | [简体中文](3d-extension.zh-CN.md)

# 3D Extension Path

The owner asked: what needs adjusting to extend to 3D later. Conclusion: **the paradigm does not change — lines live even better in 3D**
(3D bones are line segments to begin with, and Maya/Blender bones are wireframes; skeletons = polyline chains; NURBS control nets = line nets).
2D is a special case of 3D (z≡0), not two engines. What changes is four **semantic rules**, not the primitives.

## 1. What stays unchanged (the 3D dividend of everything-is-lines)

| Primitive/mechanism | Status in 3D |
|---|---|
| Lines, nodes, three-level attributes | Survive as-is |
| Relations as functions, DAG evaluation | Unchanged; bind/constraint/propagate/track lift directly to higher dimensions |
| Command protocol, validator, MCP, Tick | Unchanged |
| SoA/instancing/damage rendering | The dividend is bigger (triangle counts explode in 3D; isomorphic data is worth more) |

## 2. Four semantic rule adjustments (not new primitives)

1. **Generalization of closed-line fill**: 2D = closed line + fill rule (non-zero/even-odd); in 3D an arbitrary closed line does not uniquely determine
   a surface — generalized to **3-node closed line = triangular face** (three points are uniquely coplanar, so it holds naturally); n>3 closed lines
   need a coplanarity constraint or triangulation (the fill rule gains "planarity");
2. **Surface = line net**: a generalization of the 2D bitmap-mesh (two sets of warp/weft lines) concept — surface = line net,
   bilinear/Coons interpolation, isomorphic to a NURBS control net; normals/lighting = derived attributes of the line net;
3. **Occlusion**: from the ownership relation's ordering parameter (zOrder) to **depth testing** (a rendering attribute);
4. **Camera/transform box**: box lines → frustum wireframes/wireframe boxes — still lines. Transform channels follow the Moho facts
   (translation/scale were already ⟨ch:Vec3⟩, rotation was already three xyz channels [^mohoproj-format])
   — **Moho has already validated this upgrade path**.

**New relation types (three)**: projection (deriving 2D graphics from a 3D line net; the Moho 3d_mode precedent [^mohoproj-format]),
aim, and pole vector (a 3D IK supplement).

### 2.1 Representation resolution for position and rotation (2026-11-11, owner asked "quaternion or Vec3 for points")

- **Position = Vec3, no dispute**: quaternions are a representation of rotation and do not encode position; dual quaternions
  (rotation + translation in one) serve only as a **runtime intermediate** for skinning blending (usable internally when the bind
  implementation upgrades LBS→DQS) and do not enter the format;
- **Rotation: the format stores Euler xyz, the runtime converts to quaternion/matrix** — the storage representation serves
  AI readability / per-component tracks / byte determinism (three Euler channels, the Moho rotation_x/y/z precedent [^mohoproj-format]; angles
  normalized to (−π,π] on write); the runtime representation serves the math (quaternions/matrices used inside the transform chain, an
  implementation detail); glTF chose quaternion storage because its consumers are engines, while mo2d's consumers are the AI + the channel
  model, hence it sides with Moho;
- **Gimbal-lock fallback**: mo2d bone orientation is **derived** from the positions of the two end nodes rather than stored, so hinge-style
  animation naturally dodges most of the problem; if a free-tumbling requirement truly appears (Phase D), add an **optional quaternion channel
  value type** (Vec4 + unit constraint + q/−q canonicalization with w≥0) as a channel-type extension, without overturning Euler.

## 3. Preventive decisions (do them now, nearly zero cost, no migration later)

| # | Decision | Where it lands |
|---|---|---|
| 1 | **Node position stored as Vec3 from day one** (z≡0) | [file-format.md](file-format.md) envelope, SoA, command protocol |
| 2 | The fill-region rule states the "3-node closed line = triangular face" generalization path | [line-model.md](line-model.md) §1 note |
| 3 | The renderer abstraction keeps a **depth switch** (the CPU oracle takes the planar path during the 2D period) | architecture L3 |
| 4 | The bind weight formula = **projected distance** from point to segment (same formula in 2D/3D) | A2 implementation choice |
| 5 | Camera-line nodes carry z (placeholders for perspective parameters) | [line-model.md](line-model.md) §3.1 |
| 6 | **Rotation: format = Euler xyz per-component channels (normalized on write); runtime = quaternion/matrix** | This document §2.1 |

## 4. Honest difficulties

- **3D modeling interaction** (depth picking/multiple views) is the traditional disaster area — "AI-native + command protocol" happens to be
  the solution: the AI doing command-style modeling in 3D beats dragging (an unexpected advantage of the mo2d paradigm in 3D);
- Lighting/materials/shadows: split per the division-of-labor iron rules — material parameters = channels; the lighting hot path stays in the
  engine; advanced effects are AI-generated;
- **Depth-precision consistency** between the CPU oracle and the GPU is harder than in 2D; the parity gate needs to be finer.

## 5. Scheduling (revised v4.1: basic 3D has entered the MVP)

After the owner decided to "directly support basic 3D since the cost is low," 3D entered the map in two layers:

| Layer | Status |
|---|---|
| **Engine-side basic 3D** | **In the MVP (v4.1, [mvp.md](mvp.md) §2.1)**: the renderer is 3D (z-buffer/perspective/back-face culling/flat color with no lighting, 2D always z=0 through the same pipeline), Vec3 nodes, triangular faces, perspective/orthographic cameras, the 3D static-scene hash gate; the AI can build z≠0 scenes via the command stream alone (headless) |
| 3D interaction & deepening | **Phase D (long-term, not chartered)**: orbit navigation, drawing on work planes, depth picking, lights/materials/shadows, glTF — the cost is in the interaction, not the math; trigger conditions to be set once 2D is on solid ground |

> The very existence of this document is a signal: the "2d" in mo2d's naming is honesty about the present, not a ceiling;
> if Phase D is chartered, the renaming question will be handled together with the branding decision.
