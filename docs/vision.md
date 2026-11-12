---
type: Design Document
title: "Vision: An AI-Native 2D Vector Animation Engine"
description: mo2d's positioning and the definition of what an AI-era engine is (six components — semantic language / deterministic runtime / command protocol / validator / performance machine / generatable views); four core ideas; division-of-labor principles; success criteria; open decision items.
tags: [vision, positioning, ai-native, engine-definition]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
  - id: ai-interaction-doc
    resource: /ai-interaction.md
    title: "本 bundle:AI 原生交互"
---

**English** | [简体中文](vision.zh-CN.md)

# Vision: An AI-Native 2D Vector Animation Engine

mo2d is an **AI-native 2D vector animation engine**: skeletal animation, vector graphics, bitmap mesh deformation,
timelines — the heavyweight concepts of traditional software are reduced to **lines and relations between lines**; and with AI,
standard GUI controls (toolbars/menus/property panels) are no longer necessities — users simply define lines
and the linkage relations between them directly on the stage.

## 1. Problem Diagnosis

Two structural dilemmas of traditional 2D animation tools (Flash/Animate, Moho, AE, Toon Boom):

**The GUI paradigm dilemma**
- Every domain concept gets its own dedicated controls: toolbars, menus, a forest of panels — a combinatorial explosion of features and a steep learning cost;
- Rigid layer trees/group trees are container type systems, conceptually redundant (layers, groups, symbols, and comps each have their own);
- AI can only operate them by "simulated clicks" — a GUI is an interface for hands, not an interface for agents.

**The heterogeneous-data dilemma**
- Bitmaps, vectors, text, and bones each have their own data structures and rendering paths;
- Many branches on the CPU side, many state switches on the GPU side — throughput cannot scale.

## 2. The Shape of an AI-Era Engine (Definition)

**An AI-era engine is not a feature set, but a minimal semantic language + a deterministic runtime.
Features are AI's output, not the engine's building blocks.** Six components, none dispensable — and no seventh needed:

1. **Semantic language** (the kernel is the language): minimal, orthogonal, composable primitives (mo2d: lines / nodes / three-level attributes / relations / channels).
   AI capability = **coverage of composition**, not the enumeration surface of an API — the smaller and more stable the language, the stronger the AI
   (thousands of APIs are an AI's graveyard; six primitives are an AI's LEGO);
2. **Deterministic runtime**: AI compiles, the engine executes; the runtime never calls AI; same input, same output (byte-identical).
   Determinism is the **foundation of trust** for AI output — every AI generation may differ; every engine execution must be identical;
3. **Command protocol**: the single interface shared by humans, AI, and the engine; recordable, replayable, diffable.
   The protocol is the product — GUIs go obsolete; protocols are assets;
4. **Validator**: schema + determinism gate (byte-identical replay) + sandbox. Exploits the **generation-verification asymmetry**:
   generation is expensive (AI's job), verification is cheap (the engine's job) — the cornerstone of engineering safety in the AI era;
5. **Performance machine**: uniform data layout (SoA/instancing/damage rendering). The **double dividend** of conceptual unification:
   a small language (easy for AI to compile to) + isomorphic data (easy for the GPU to saturate);
6. **Generatable views**: the GUI degrades from "product" to "view"; controls are lines, discarded after use.

| Dimension | Traditional engine | AI-era engine |
|---|---|---|
| Essence | Feature set (millions of lines of code) | Semantic language + deterministic runtime |
| Source of capability | Humans write features across version iterations | AI-compiled artifacts; capability is data |
| Interface | GUI for humans, API for programmers | Command protocol for agents; the GUI is a generated view |
| Learning cost | Humans learn controls and concepts | AI learns the language (once); humans only express intent |
| State | Opaque binary | Declarative, diffable, hand-editable |
| Quality assurance | Test teams | Generation-verification asymmetry |
| Performance | Heterogeneous features each take their own path | One data shape ⇒ GPU throughput |
| Evolution | Releases add features | The language barely changes; capability grows wildly at the AI layer |

**The underlying economics**: in the AI era, features/interfaces/knowledge become **abundant goods** (AI generates them at will);
determinism/truth/performance/safety remain **scarce goods** (AI cannot supply them). **The engine holds the scarce; AI produces the abundant.**
"Everything is Lines" is the enabling decision: conceptual unification feeds both ends at once — AI wants a small, stable compilation target,
the GPU wants an isomorphic data shape; one unification, dividends collected at both ends.

## 3. Four Core Ideas

1. **Everything is Lines** — bones, timelines, vector graphics, bitmap meshes, gradients, layers are, in essence, all
   representation and control of lines ([line-model.md](line-model.md));
2. **Relations are Functions** — sets/layers/groups are predefined relations, simulation is dynamic relations, all behavior logic
   is relations between lines ([relations.md](relations.md));
3. **The Stage is the Interface** — no standard toolbars/menus; users define lines and linkages directly on the stage,
   and AI performs every operation through the command protocol ([ai-interaction.md](ai-interaction.md));
4. **Concepts fit the GPU** — the unified model of lines + relations is isomorphic to the GPU execution model:
   conceptual unification ⇒ data isomorphism ⇒ maximal GPU throughput (owner: "Lines and line relations can in fact maximize GPU utilization; the concepts fit, improving performance").

## 4. Concepts Fit the Hardware (Four Arguments)

| Stage | Argument |
|---|---|
| Rendering | Node channels = SoA arrays → vertex attributes/storage buffers, the whole scene uploaded at once; variable-width lines → unified triangle strips, closed lines → fill tessellation; isomorphic data = very few draw calls, zero state switches |
| Evaluation | The relation DAG evaluates in parallel by layers: independent relations within a layer run in parallel; node-level channel sampling is a parallel map |
| Simulation | Particles = directed segments = instanced rendering + compute integration; springs/constraints = PBD-style parallel solving |
| Skinning/deformation | Binding = vertex blending, a natural workload for the GPU vertex stage |

## 5. Minimal Kernel

**Lines (with nodes and three-level attributes) + relations (functions) + channels (parameters) + payloads** — four primitives carry all domain concepts;
whatever is not lines honestly stays outside the boundary (sampling fields / symbolic logic / out-of-document state / external references, see
[line-model.md](line-model.md) §4).

## 6. Division of Labor Between AI and the Engine (direction set by the owner on 2026-11-11)

owner: "Since we have AI, can we hand more of the capability over to AI and have the engine handle only the performance part?"
The direction holds, made precise: **the engine holds "performance + determinism + semantic language"; every other capability goes to AI.**

**Three iron rules**:

1. **AI compiles, the engine executes** — AI compiles intent into deterministic artifacts (lines/relations/channels/commands), and the engine executes the artifacts;
   AI is the compiler, not the runtime;
2. **AI at authoring time, pure engine at playback** — rigging/animating/writing relation functions go to AI (seconds of latency is tolerable);
   the 60fps evaluation/rendering/simulation of every frame is pure engine math — **the runtime never calls AI**;
3. **Capability is data** — AI-generated capabilities (auto-rigging/lip sync/cursor following) are solidified as relations and channels in the document;
   the file still opens, plays, and exports without AI (the product is not hostage to AI availability).

| Kept in the engine (more than performance) | Handed to AI (the capability layer) |
|---|---|
| Deterministic evaluation (CPU oracle, byte-identical command replay) | Auto-rigging, in-between, lip sync |
| Semantic language (lines/nodes/segments/relations/channels = AI's compilation target) | Motion synthesis, retargeting, stylized brushes |
| Hot paths built in (skinning/simulation solving/track sampling/rendering) | Control generation (controls are lines), format understanding and conversion |
| Truth and validation (document state/schema/acceptance gates) | Natural language → command translation (existing) |
| Sandbox (safe execution of AI-generated code) | All "how to do it" knowledge |

**One-vote veto**: calling AI per frame at runtime — determinism/performance/cost/reproducibility all collapse.
**Architecture landing** (see [architecture.md](architecture.md) §2.1): the predefined relation library shrinks to
a "hot-path kernel + semantic anchors"; AI-generated relation function bodies run through a restricted IR and pass the determinism gate before admission.

## 7. Success Criteria

1. Users complete skeletal rigging and animation by intuition (draw lines/drag nodes/connect lines) plus natural language, with zero menu operations;
2. AI completes **all** operations through the command protocol (create lines, create relations, set keyframes, export), and the command stream replays reproducibly;
3. Large-scale scenes (on the order of a hundred thousand nodes) evaluate + render smoothly on the GPU; the CPU oracle agrees with the GPU output (determinism);
4. Once AI-generated capabilities are solidified, **offline (no-AI) playback/export** results match the online ones.

## 8. Open Decision Items (not to be written as settled facts)

| Item | Current status |
|---|---|
| Relation between mo2d and maohou | Leaning: an independent new-engine mainline that absorbs maohou's Moho format research but is not bound to its code; "mo2d" is the engine codename, the product brand is TBD |
| Tech stack | Leaning: Rust + wgpu/wasm (WebGPU); concept documents are stack-agnostic, see [architecture.md](architecture.md) |
| Moho compatibility | A fully independent line-native format; .moho import listed only as a long-term option (the 84 tutorial projects can serve as an acceptance corpus) |
