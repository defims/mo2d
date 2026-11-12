---
type: Task Brief
title: "MVP Final (v5): Tentacle + 3D Scene, Built End-to-End via the Command Protocol"
description: MVP v5 consolidated draft — six items in scope (3D-ready line kernel / hot-path relations / Tick time / command protocol + validator / format + export / minimal stage), six gated acceptance steps (A1 includes the 3D scene hash gate, A6 hand-built and AI command streams isomorphic), an eight-item Definition of Done, a nine-item frozen decision list, and the basic-3D hard boundary plus anti-scope-creep clauses.
tags: [mvp, scope, stage, mcp, ticks, 3d]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线(三级属性)"
  - id: moho-mapping-doc
    resource: /moho-mapping.md
    title: "本 bundle:Moho 概念逐项映射(属性/关系词表)"
  - id: file-format-doc
    resource: /file-format.md
    title: "本 bundle:文件格式(Tick/信封/字节确定性)"
  - id: vision-doc
    resource: /vision.md
    title: "本 bundle:愿景(§2 六部件、§6 分工铁律)"
  - id: 3d-doc
    resource: /3d-extension.md
    title: "本 bundle:3D 扩展(预防决策/旋转表示决议)"
---

**English** | [简体中文](mvp.zh-CN.md)

# MVP Final (v5): Tentacle + 3D Scene, Built End-to-End via the Command Protocol

## 0. Version history

| Version | Change |
|---|---|
| v2 | Initial version: tentacle vertical slice, stage included in scope |
| v3 | Re-scoped per the six-components engine definition: stage moved out, AI channel promoted to a main axis, validator made explicit, headless demo |
| v4 | Owner decided to pull B1 forward: the minimal stage returns as A6, and the Definition of Done gains direct-manipulation equivalence |
| v4.1 | Owner decided to "directly support basic 3D at low cost": renderer-is-3D, Vec3 nodes, 3D scene gate |
| **v5** | **Consolidated draft**: merges three rounds of increments into a self-consistent final — the stage enters the scope list, 3D enters the Definition of Done, the nine frozen decisions are summarized |

**MVP in one sentence**: one animated tentacle (2D) + one static z≠0 scene (3D), everything built via the command protocol,
AI and hand as two isomorphic channels, deterministic engine execution, playable offline.

## 1. What it proves

| Component / iron rule (vision §2/§6) | Form in the MVP |
|---|---|
| Semantic language | Line/node/segment three-level attributes + three kinds of hot-path relations (minimal vocabulary) |
| Deterministic runtime | Relation DAG evaluation + Tick sampling; same input, same output (byte-level) |
| Command protocol | JSON command stream (with protocol version) + three MCP commands |
| Validator | Schema validation + determinism gate (replay byte-equality) |
| Generatable view (first example) | A6 minimal stage: four operations, zero standard controls |
| AI compiles, engine executes | The AI builds the tentacle from scratch via MCP; the engine plays and exports offline |
| Capability as data | With the AI session closed, the document still plays/exports as usual, frame-identical |
| 2D = special case of 3D | A static z≠0 scene is built and rendered via the command stream (A1 gate) — the claim is nailed down by tests |

(Component 5, the performance machine, = CPU single-track to start, SoA keeps the path open, deferred to roadmap B1.)

## 2. Scope: six items in

1. **Line kernel (3D-ready, headless offscreen)**:
   - Structure: nodes/segments/closure; minimal subset of the three-level attributes ([moho-mapping.md](moho-mapping.md) §7.1–7.3):
     line-level `closed, width, color, strokeOn, fillOn`; node-level `position (Vec3, 2D always z=0),
     width, color, handle{type, offsetIn, offsetOut}`; segment-level `type (straight/curve), visible`;
   - Rendering: variable-width lines as triangle strips; closed fill non-zero (**3-node closed line = triangular face**);
   - **The renderer is 3D**: z-buffer + perspective divide + back-face culling + flat color, no lighting; perspective/orthographic cameras;
     2D always z=0 through the same pipeline ([3d-extension.md](3d-extension.md));
2. **Relation hot-path kernel** ([architecture.md](architecture.md) §2.1):
   - Node sharing (skeleton polylines); **bind** (linear weights, formula = **projected distance** from point to segment, same formula in 2D/3D);
   - **track (track reference)** (a track = a line: node position = Tick, keyframe value = a node-level attribute,
     interpolation Linear/Bezier = im 0/9 = a segment-level attribute; the track relation = wiring the track line to the animated attribute;
     resolution in §8 D1 of this file);
3. **Time**: keyframe time position = **raw Tick integers** ([file-format.md](file-format.md) §2 rule 1:
   ascending, fps is a display-only parameter); the demo artifact includes **one closed-line element**
   (the sucker at the tentacle's tip) — fill has an acceptance artifact;
4. **Command protocol + validator**: command registry, JSON command stream (**protocol version field**, old streams remain forever replayable),
   undo/replay, three MCP commands (apply/list/screenshot); schema validation + determinism gate (persist to disk only after passing the gate);
5. **Format + export**: minimal envelope subset `{format, version, minReader, generator, root{width,height,ticksPerSecond,fps}, lines, relations}` (no BLOB/refs/history);
   byte-level determinism (stable field order, atomic writes); **PNG sequence as primary** (verifiable frame-by-frame hashes), GIF as a bonus;
6. **Minimal stage** (A6 deliverable, first example of a generatable view): a thin viewport wrapping the headless kernel —
   draw lines / drag nodes / connect lines to build relations / play, **four operations**, zero standard controls, every interaction = a command.

**All out**: bitmap mesh, simulation, gradients, custom relations, brush-stroke style library, importers, multi-layer/groups,
BLOB area, project-directory layout, collaboration; **3D side**: lights, materials, 3D navigation, work planes, depth picking, glTF
(all in Phase D — the cost is in the interaction, not the math).

## 3. Six gated steps (no passing the gate, no moving on; driven by the command stream end-to-end from A1 on)

| Step | Content | Acceptance gate |
|---|---|---|
| A1 | Line kernel + offscreen rendering (3D-ready) | The command stream builds the variable-width tentacle line; rendering compared against reference images (triangle strip/non-zero fill); **3D static scene gate**: the command stream builds z≠0 triangular faces/wireframes, render hash frozen |
| A2 | Node sharing + skeleton + bind | Polyline skeletons drive deformation; the unbound negative control does not deform |
| A3 | Tracks + Tick + playback | Keyframe Ticks ascending; Linear/Bezier switchable; loop playback reproducible frame by frame |
| A4 | Command protocol + MCP + validator | Record the entire A1–A3 process; replay reproduces it; the AI completes the same animation from scratch via the three MCP commands; schema rejects bad documents with errors |
| A5 | Format + export + offline closed loop | round-trip byte-equality; **close the AI session and reopen, playback/export frame-identical** |
| A6 | Minimal stage | Four operations, zero controls; every interaction = a command; **the hand-built tentacle's command stream is isomorphic to the AI path's** |

## 4. Definition of Done (eight items)

1. **AI compiles**: the AI builds and exports the tentacle animation from scratch via MCP; the floor = a prerecorded command stream (pseudo-AI) must pass,
   a real AI session is a stretch bonus;
2. **Byte-level reproduction**: replay command stream → save → byte-identical to the document from the original path (the protocol version field guarantees old streams remain forever replayable);
3. **Offline consistency** (capability as data): with no AI session, open/playback/export is frame-identical to online;
4. **Persistence**: round-trip byte-equality; unknown keys passed through;
5. **Evaluation throughput**: synthetic stress document (command stream generates 10k nodes), single-frame evaluation < 16ms (CPU, reference hardware TBD);
6. **Validator on duty**: schema rejects bad documents; the determinism gate blocks non-deterministic output;
7. **Direct-manipulation equivalence** (A6): the hand-built tentacle's command stream is isomorphic to the AI path's;
8. **3D special-case verification** (A1): a static z≠0 scene can be built and rendered, hash frozen — "2D = special case of 3D" holds.

## 5. Key trade-offs

- **Headless first, A6 wraps a thin viewport**: strictly the four operations; **whatever is missing, have the AI emit commands — add no controls to the viewport**;
- **No GPU in the MVP**: CPU oracle single-track; SoA layout keeps the path open (roadmap B1);
- **Minimal interpolation set**: Linear/Bezier (im 0/9); Smooth and the rest deferred (moho-mapping §5.2);
- **Simplest linear bind**: single bone + projected-distance weights; multi-bone blending / weight painting deferred;
- **Invalidation strategy deferred**: full recomputation; damage rendering / DAG reverse invalidation goes into roadmap B1;
- **Command normalization across different paths is not in the MVP**: only same-path replay is verified (known boundary);
- **Basic-3D hard boundary**: flat color, no lighting + perspective camera + z≠0 buildable and renderable; lights/materials/navigation/work planes/
  depth picking/glTF are all Phase D;
- **Start with three MCP commands**: if Definition of Done item 1 fails, supplement (supplement = add a new version, never change an old one).

## 6. Known Issues & Mitigations (status overview)

| Level | Issue | Status |
|---|---|---|
| P0-1 | Track representation gap | ✅ Resolved (§8 D1) |
| P0-2 | Protocol versioning | ✅ Fixed (§2.4) |
| P0-3 | Fill had no acceptance artifact | ✅ Fixed (closed sucker, §2.3) |
| P1-4 | Different-paths-same-result normalization | Declared as a boundary (§5) |
| P1-5 | Vanity metric | ✅ Fixed (evaluation throughput) |
| P1-6 | AI acceptance external dependency | ✅ Fixed (pseudo-AI floor) |
| P1-7 | Render reference images self-certifying | Downgraded: hash freeze + human eyeballing; a true oracle gate is in roadmap B1 |
| P2-8 | Headless, nothing playable | ✅ Solved (A6, v4) |
| P2-9 | GIF banding | ✅ Fixed (PNG primary) |

## 7. Risks & mitigations

| Risk | Mitigation |
|---|---|
| No MCP host environment | A1–A3 driven by local command-stream files (same protocol); real host from A4 on |
| Bezier handle semantics | Segment shape = in/out handles at both ends (empirically verified in moho-mapping §5.2); a 1D angle channel first, Vec2 later |
| Byte determinism broken by timestamps/hashes | No timestamps in the envelope; preview not in the MVP; atomic writes + canonical field order |
| A4 (protocol + validator + MCP) most likely to slip | Split into order: command-stream file → schema → determinism gate → MCP shell last |
| A6/3D scope creep | Two hard boundaries in §5 (four operations / 3D list); whatever is missing, have the AI emit commands |

## 8. Frozen decision list (basis for starting A1, nine items)

| # | Decision | Source |
|---|---|---|
| D1 | **Track = line** (nodes = keyframes: position = Tick, value = node-level attributes; interp = segment-level attributes); the track relation = a reference/sampling wire | Owner asked "track as line or as relation — which is better" |
| D2 | The command stream carries a **protocol version field**; supplement = add a new version, never change an old one | P0-2 |
| D3 | Node position = **Vec3** (2D always z=0) | 3D-extension resolution |
| D4 | Rotation: **format = Euler xyz per-component channels** (normalized on write to (−π,π]); runtime = quaternion/matrix | Owner asked "quaternion or Vec3" |
| D5 | **3-node closed line = triangular face** (fill generalization) | 3D-extension resolution |
| D6 | The renderer **is 3D**: z-buffer/perspective/back-face culling/flat color with no lighting, 2D through the same pipeline | Owner decided to directly support basic 3D |
| D7 | Time = **raw Tick integers** (254,016,000,000/s) | file-format §2 |
| D8 | Minimal stage pulled forward as A6, four-operation hard boundary | Owner decided to pull B1 forward |
| D9 | Demo artifact includes a closed sucker (fill acceptance artifact) + a static z≠0 scene (3D acceptance artifact) | P0-3 / D6 |

**Once this document is frozen, A1 can start.**
