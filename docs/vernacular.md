---
type: Design Document
title: "The Everyday-Concept Layer: Zero-Jargon Authoring for Novices"
description: The novice vision made concrete — owner normative statement; first-principles definitions of learning cost / controllability / familiarity; the everyday-word → engine-compilation dictionary (pull/follow/bounce/appear/tap…); the six-step novice interaction loop; ten design rules; progressive disclosure without terminology; five acceptance gates and the concept budget.
tags: [vernacular, novice, everyday-concepts, ai-mediated, learning-cost, controllability, interaction]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-17T12:09:29Z }
sources:
  - id: landscape-doc
    resource: /landscape.md
    title: "本 bundle:动画软件全景调研(F1–F13 发现)"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线(九大映射)"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数(标准关系库)"
  - id: ai-interaction-doc
    resource: /ai-interaction.md
    title: "本 bundle:AI 原生交互(舞台即界面/控制棒即线)"
  - id: vision-doc
    resource: /vision.md
    title: "本 bundle:愿景(六部件/分工铁律)"
  - id: mixamo
    resource: https://helpx.adobe.com/creative-cloud/help/mixamo-rigging-animation.html
    title: "Adobe Mixamo(自动绑定+可校验标记)"
  - id: animated-drawings
    resource: https://ai.meta.com/blog/ai-dataset-animation-drawings
    title: "Meta Animated Drawings(涂鸦→确认关节→动作)"
  - id: spline-ai
    resource: https://spline.design/ai-generate
    title: "Spline AI(预览四选一→落入可编辑场景)"
  - id: dreams
    resource: https://help.procreate.com/dreams/handbook/interface-and-gestures/timeline
    title: "Procreate Dreams(Perform 录制模式)"
  - id: pivot
    resource: https://pivotanimator.net/Pivot_Animator_Help_4-1.pdf
    title: "Pivot Animator(<6 概念做出动画)"
  - id: rive-listeners
    resource: https://rive.app/docs/editor/state-machine/listeners
    title: "Rive:Listeners(指针条件→动作)"
---

**English** | [简体中文](vernacular.zh-CN.md)

# The Everyday-Concept Layer: Zero-Jargon Authoring for Novices

## 0. Normative Statements (owner-given, verbatim)

> 严格按照第一性原理，深入调研现有传统及ai动画软件各种工具、概念，从0规划mo2d，最终愿景是，小白用户不用理解各种专业术语，仅凭生活常见基础概念结合ai就能制作调整播放交互动画。且额外学习成本做到最少，可控性熟悉度最高。

*(Strictly by first principles, deeply survey the tools and concepts of existing traditional and AI
animation software, and plan mo2d from zero. The final vision: a novice user, without understanding any
professional terminology, using only everyday basic concepts combined with AI, can create, adjust, and
play interactive animations — with minimal extra learning cost, and maximal controllability and
familiarity.)*

Recorded 2026-11-17 as the session goal; this document operationalizes it. The survey half is
[landscape.md](landscape.md) (findings F1–F13, cited below as F*).

## 1. First-principles definitions

- **Novice (小白)**: a user who has never used animation/design software and does not intend to learn
  its vocabulary. Not a "beginner en route to professional" — the design target is someone who stays.
- **Learning cost (额外学习成本)** = the number of concepts the user must hold that are *not* part of
  everyday life (keyframe, easing, hierarchy, state machine…). **Minimizing it means the novice surface
  introduces zero new concepts** — every word in the interface is a word the user already owns
  (F2: terminology, not concepts, is the beginner tax).
- **Controllability (可控性)** = for every result the system produces, the fraction of outcomes
  reachable by *direct, local action* (grab this, drag here, say "slower") without redoing anything
  else. The negation is the generative-video dead end (landscape §3.6: fix one arm → regenerate all).
- **Familiarity (熟悉度)** = the fraction of interactions that behave the way everyday physical objects
  behave (things fall, springs wobble, heavy things are slower to start, things keep doing what they
  were doing until touched again). Everyday physics as the default behavior.

These three are **measurable**, and §6 turns them into acceptance gates — the same discipline the repo
already applies to determinism and throughput.

## 2. The everyday-concept dictionary (life words → engine compilation)

The dictionary is the product of this layer: **users speak life; AI compiles to the command protocol.**
Every compilation lands in existing kernel vocabulary — track lines/nodes/segment easing (line-model N4),
relations from the standard library ([relations.md](relations.md) §1), parameters, controls-as-lines
([ai-interaction.md](ai-interaction.md) §3). Items marked ⓓ were proposals at survey time —
**the owner confirmed them on 2026-11-17** (the respond family, `oscillate`, step interpolation,
perform-record; landing as roadmap C7, contract in [relations.md](relations.md) §1.2) — they extend,
never bend, the model.

### 2.1 Motion words (make it move)

| Life word | User intent | mo2d compilation | Replaces (jargon) |
|---|---|---|---|
| 拉 / 拽 pull | move this by hand | `node.move` command (direct manipulation) | transform |
| 走到 X walk to X | travel to a place | straight line drawn on stage → track on position; AI picks ticks + easing | motion path + tween |
| 转圈 spin | keep turning | cyclic track on rotation ⓓ(R3 oscillate/cycle) | loop expression |
| 变大 / 变小 grow / shrink | scale change | track on the box line's scale (N8) | scale keyframes |
| 弹 bounce | hit and rebound | sim relation (gravity + floor collision) or bounce-eased track (N9) | physics preset |
| 摆 / 晃 sway | swing around a point | sim relation (spring/pendulum), seed fixed (N9) | spring constraint |
| 飘 float | drift slowly | sim (buoyancy/wind) or cyclic position track ⓓ | noise modifier |
| 呼吸 breathe | gentle repeating swell | scale track ×2 with soft easing, loop; or `oscillate` relation ⓓ | expression/loop |
| 跟着 follow | stay attached to a moving thing | bind / propagate relation | parenting / IK target |
| 转向 face / turn to | orient toward a point | constraint (look-at) relation | IK aim constraint |
| 回来 return | undo the motion | reverse track / spring back to anchor (sim) | — |
| 停住 hold | stay still N beats | track node spacing (exposure) | hold frames / exposure |
| 慢一点 / 快一点 slower / faster | rate change | stretch segment spans on the track (one command) | time-remap |
| 优雅一点 / 更弹 smoother / bouncier | change the feel | swap segment easing (AI picks from a small named feel set) | easing curves |
| 一边…一边… while… do… | two motions overlap | two track lines on two attributes — nothing special is needed | timeline layers |

### 2.2 Appearance words (make it look different)

| Life word | User intent | mo2d compilation | Replaces |
|---|---|---|---|
| 出现 / 消失 appear / vanish | visibility at a moment | visibility/opacity parameter key (N7/N4) | opacity keyframe |
| 变色 turn color X | recolor | color track on style / gradient stops (B3/N6) | color keyframes |
| 变成 Y 形状 morph into Y | shape change | track on node positions (AI places correspondences) | shape tween |
| 换装 / 换表情 change look | discrete swap | discrete parameter over shared curves (moho switch reduction) | switch layers / skins |
| 盖在…上面 / 藏到…后面 in front of / behind | stacking | zOrder parameter (animatable) | layer ordering |
| 只露出这部分 show only this part | clip | shape mask relation | mask/matte/cutter |

### 2.3 Interaction words (make it respond)

The perceive→decide→respond minimum model (F11). Owner-confirmed mapping (2026-11-17;
[relations.md](relations.md) §1.2):

| Life word | User intent | mo2d compilation | Replaces |
|---|---|---|---|
| 点它 / 戳它 tap / poke it | react to click | respond relation: event `tap(target line)` → action | listener + trigger input |
| 悬停 / 移开 hover / move away | the pointer is over it | gesture-line `hoverIn`/`hoverOut` edges (phase 1/0) + respond on hover events | mouseenter/mouseleave listener |
| 碰到 / 撞上 touch / bump into | react to contact | respond relation: event `overlap(line A, line B)` | collision event |
| 按住 / 松开 press / let go | hold states | gesture line: press/release edges materialize a per-target gesture timeline (im=3); respond relations on `press` / `release` write a switch parameter | boolean input |
| 长按 / 按久一点 hold it / press longer | keep pressing for a while | release respond with `minHold` (hold = press→release pair measurement = the gesture line's pressed-segment length) | long-press event |
| 一碰就跳 / 害羞地躲开 jump when touched / shy away | event → motion | respond → switch/start a track (AI also writes the track) | state machine + transitions |
| 等一下再… after a moment… | delayed response | respond on `time-point(t)` / `track-end(track)` event | exit time / delay |
| 动一下就停下 stop after a while | one-shot behavior | trigger semantics: fires once, resets | trigger input |

**Where it lives**: an event is a single-tick irreversible fact; a **gesture is a timeline** — its
phase edges materialize as a **read-only reference lane on the time axis** (`gesture/<target>`,
step im=3, roadmap C8), and the pressed segment's length *is* how long it was held. A respond
relation is a function from (event × guard, incl. hold length) to parameter writes / fires —
squarely "relations are functions" ([relations.md](relations.md) §1.2). The plain player evaluates
respond relations deterministically (same events → same behavior) — offline playback with
interactivity intact, capability-as-data (vision rule 3).

### 2.4 Flow words (put it together)

| Life word | User intent | mo2d compilation | Replaces |
|---|---|---|---|
| 一遍又一遍 repeat forever | loop | track cycle flag / sim steady state | loop playback range |
| 然后接着 then next | sequence | AI lays successive spans on the same tracks | scenes/sequence |
| 先…再… first… then… | order | track span placement (+respond on track-end ⓓ) | timeline ordering |
| 照我做的做 do what I do | demonstrate | ⓓ perform-record: capture gestures → track keys (F6, Dreams Perform precedent[^dreams]) | motion sketch / recording |

### 2.5 Ask-for-help words (talk to the AI)

| Life word | User intent | mo2d compilation |
|---|---|---|
| 让它挥手 make it wave | intent | AI builds rig + track + commands, all inspectable on stage |
| 照这个样子 like this picture | reference input | AI derives lines from the image (derive relation / external-ref) |
| 它自己动它会动的部分 let it move the parts that would move | auto-rig | AI proposes skeleton polyline + binds, shows one confirm checkpoint (F7)[^mixamo][^animated-drawings] |
| 这样不对 that's not right | feedback | AI edits the *same* lines/keys — or the user drags them; both channels land on one truth |

**Dictionary discipline**: the novice surface is allowed exactly these words (≤ ~25 roots). Every new
verb must either compile to existing vocabulary or justify a new predefined relation — the dictionary is
a **budget**, reviewed like any other gate.

## 3. The novice interaction loop (six steps)

Distilled from landscape §3.4/§3.6/F13 — every trusted AI tool shares this shape:

1. **Anything goes in** — a sketch, a photo, a child's drawing, or just a sentence (Motiff/Animated
   Drawings precedent). Nothing to learn: input is whatever the user already has.
2. **Intent in life words** — the user says what they want (dictionary §2); no feature is named, none
   needs to be.
3. **AI proposes; one glance confirms** — structure inference (parts/joints/deformers) lands as ordinary
   lines + bind relations, shown on stage for one-glance correction (Mixamo markers precedent[^mixamo]).
   The checkpoint is where trust is established — and the only pause in the whole loop.
4. **It lands as the real thing** — generated motion = track lines + keys; generated response = respond
   relations; generated controls = lines (ai-interaction §3). Nothing is baked, nothing is second-class
   (Figma Make precedent). The stage now *plays* — display equals edit, no mode switch.
5. **Adjust by hand or by word — same truth** — drag a node / stretch a span on the stage, or say
   "bouncier"; both edit the same keys. Direct manipulation and language are one channel pair
   (ai-interaction §2), never rivals (Kling's motion-path gesture kept, its pixels refused).
6. **Deterministic play + rewindable history** — play back anywhere: byte-identical, no AI needed
   (capability-as-data); every AI action is a checkpoint in the command stream — undo is inverse
   commands (v0/Lovable precedent; see landscape sources).

## 4. Design rules (learning cost ↓, controllability & familiarity ↑)

1. **No invisible state** (F3): anything affecting the picture is visible where the user is looking —
   overlays are lines, controls are lines; nothing hides in collapsed panels (there are no panels).
2. **One primitive per function** (F10): one mask, one reuse mechanism, one hierarchy mechanism — the
   kernel already is that; the *novice surface* must not re-multiply synonyms.
3. **Everything obeys the same rule** (familiarity): lines + relations, drawn/connected like physical
   things; no special cases the user must memorize.
4. **Everyday physics is the default** (familiarity): unattached things hang or fall (sim), interrupted
   motions spring back (damped spring settle), heavy-feeling things ease in slower — the feel set is
   named in life words (弹性/飘/沉/跟手).
5. **Life words compile to editable curves** (controllability): "bounce" resolves to a track with
   bounce easing or a sim seed — never to a baked clip (Pika's verb-presets kept as *interface*, refused
   as *output*).
6. **AI lands native, never bakes** (controllability, F13): generated = ordinary lines/keys/relations;
   every generated thing hand-fixable (the "fix one arm" test must always pass).
7. **Non-determinism lives only at choice points** (trust): when AI must pick among plausible results,
   it offers stage previews to point at (Spline four-preview pattern[^spline-ai]); after the pick,
   playback is deterministic forever.
8. **Undo everything, checkpoint per AI action** (trust): the command stream already is the history
   (ai-interaction §4); the novice never fears saying "redo it differently".
9. **The professional name is optional, never required** (zero-jargon): the engine may reveal "这是
   关键帧 (keyframe)" as a secondary label for curiosity — no gate, button, or tutorial requires it.
   The user can go a lifetime without meeting the word.
10. **Progressive depth by need, not by curriculum** (F4/F12): first visible result is pose-without-time
    (drag the puppet / slider — Live2D lesson); time enters when the user says a time word; interaction
    words come last. Each stage ships a working result.

## 5. Progressive disclosure without terminology

| Level | User sees | New concepts needed | Kernel in play |
|---|---|---|---|
| 0 — stage + talk | the picture, a chat box, play | 0 | commands |
| 1 — hands | drag parts; it follows the hand; "回到原位" anchor | 0 | bind/propagate + parameters (pose w/o time, F12) |
| 2 — motion | "走到那边""弹一下""晃一晃" | 0 | tracks + easing + sim |
| 3 — response | "点它就跳""按住不动就眨眼" | 0 | respond relations ⓓ |
| 4 — power (optional) | handles, tracks visible as lines, professional labels | opt-in | the whole kernel |

Levels are **not modes** — one stage, one truth; the level only decides how much structure is *shown*.
Expert views are generated views (vision §2 component 6): the "timeline" a professional would recognize
is AI-drawn as lines on demand (line-model §3.1: playhead/track views).

## 6. Acceptance gates (the novice test battery)

Concrete, runnable gates; a session goal done = gates scripted as demo command streams (repo
convention: every step ships a visible deliverable accepted via the command stream).

- **N1 — First animation, 10 minutes, zero jargon** (goal: 制作): a user who has never used animation
  software turns a sketch into a waving character: input → confirm rig (one glance) → "让它挥挥手" →
  plays. Instrument: no professional term appears at any point; total user concepts used ≤ dictionary
  §2.5. Precedent: Animated Drawings works on children's drawings[^animated-drawings].
- **N2 — Fix one arm** (goal: 调整, the controllability probe): with a finished animation, the user
  changes exactly one limb's motion — by dragging, or by saying which part; the other 95% does not move.
  Fails if the only path is re-generating everything (the generative-video dead end).
- **N3 — Interactive without code or AI** (goal: 播放交互): "点它就跳一下" plays in the plain offline
  player — respond relations evaluated deterministically, no AI, no scripts (capability-as-data,
  vision rule 3).
- **N4 — Trust**: same document + same command stream (including AI actions) replays byte-identically;
  every AI action undoable via inverse commands. Covered by the existing determinism gate, extended to
  respond relations.
- **N5 — Concept budget**: the entire novice surface (all UI strings, all AI replies, all docs aimed at
  novices) uses ≤ ~25 everyday roots (dictionary §2), zero professional terms required. Audited like
  the frozen hash gate: a jargon word leaking into the novice surface is a regression.

## 7. Impact on existing docs & roadmap (owner-confirmed 2026-11-17; landing = roadmap C7)

- **relations.md §1**: add the **respond** relation family (interaction: event × condition → parameter
  write / track switch / fire) and optionally `oscillate` (cyclic driver: period/amplitude/phase) —
  from landscape §6 R2/R3;
- **roadmap.md Phase C**: interactivity vocabulary + plain-player deterministic event handling becomes
  an explicit C-item (with N3 as its gate); perform-record (F6/R6) as a capture channel;
- **ai-interaction.md §2**: the dual channel gains its novice form — the two channels are *equal* also
  for a user who has zero vocabulary (the language side must accept dictionary words only);
- **vision.md §7**: success criteria gain the novice gates N1–N5 (the vision already demands
  "by intuition, zero menu operations" — N1–N5 make it testable);
- **C8 (2026-11-17)**: the interaction words gain 悬停 (hover) and 长按 (long-press) — both compile
  to the gesture-timeline vocabulary ([relations.md](relations.md) §1.2); the dictionary budget
  absorbs them (+2 roots, still within ~25);
- **mvp.md**: unchanged — the MVP tentacle is untouched; the dictionary compiles onto it
  (让触手摆动 = the demo's own commands).

## 8. Open Questions

- [x] The respond relation family + event vocabulary (§2.3) — **confirmed by the owner 2026-11-17**
  (relations.md §1.2; events enter the command stream as irreversible `event.fire` facts);
- [x] `oscillate` / cyclic-track mode vs pure sim — **confirmed 2026-11-17**: both exist — `oscillate`
  is the deterministic cyclic driver (breathe/float/spin), sim keeps physical sway;
- [x] Perform-record placement — **confirmed 2026-11-17**: the minimal version (gesture → track keys
  via auto-key) lands in roadmap C7;
- [ ] The dictionary roots (~25) and the Chinese/English word lists as a normative budget —
  provisionally adopted; per-word review continues as the AI surface grows;
- [ ] The confirm-checkpoint UX for auto-rig (one-glance handles: what exactly is shown, N1);
- [ ] Localization policy for the dictionary in other languages (the life words must stay everyday in
  each locale — terminology-free is a per-language property).
