---
type: Reference
title: "The AI-era Replacement Audit: Every Animation Concept & Stage, Judged Against the SVML Pattern"
description: Careful sweep of all animation software concepts and stages (14 pipeline stages + 16 mechanisms), judged against Hypit's SVML pattern (word-anchored declarative source, timeline demoted to a projection, deterministic compiler, agent-writes-source); each entry gets an AI-era replacement candidate or a keep verdict, with the mo2d status and roadmap proposals — all pending owner confirmation.
tags: [ai-era, svml, hypit, replacement-audit, timeline, authoring, intent, anchors]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-22T16:30:00Z }
sources:
  - id: hypit-components
    resource: https://hypit.ai/developers/custom-components
    title: "Hypit:Custom Components(SVML Source/Script/Selections/Moments/Recipe)"
  - id: hypit-docs
    resource: https://docs.magnific.com
    title: "(navi) Hypit/AI video docs hub"
  - id: opentrend-hypit
    resource: https://opentrend.org/project/hypit
    title: "Open Trend:Hypit 开源项目(SVML 词锚定)"
  - id: remotion
    resource: https://www.remotion.dev
    title: "Remotion:Make videos programmatically (React)"
  - id: vision-doc
    resource: /vision.md
    title: "本 bundle:vision §6.1 精度与表现力两层+AI 三无"
  - id: landscape-doc
    resource: /landscape.md
    title: "本 bundle:动画软件全景调研(概念清单 F1–F13)"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线(十大映射)"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数(C7/C8 事件与手势)"
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:D7 tick 时间真值/D10 手势时间线"
---

**English** | [简体中文](ai-era.zh-CN.md)

# The AI-era Replacement Audit: Every Animation Concept & Stage, Judged Against the SVML Pattern

Owner directive (2026-11-22): 细致排查动画软件所有概念和环节，参考 hypit 用 svml 替代时间轴的方式，
看哪个环节可以用 ai 时代的概念代替。This document is the sweep. All replacement proposals below are
**[derived, pending owner confirmation]** — none is implemented.

## 1. The reference case: Hypit's SVML (verified from the docs)

Hypit is an open-source AI video authoring system. Its source document is an **SVML file
(Structured Video Markup Language)** — "It names the script, the media, the Tracks and the Film,"
with layers Script / Timeline / Canvas / Frame / Style[^hypit-components]. Verified mechanics:

1. **Word anchoring**: content binds to transcript **words**, not seconds. Inline tags around
   script words double as bindings (`@hair-gel more hair gel than @trophy … @/trophy`), referenced
   elsewhere as `during={script.selection.hair-gel}`. Two anchor primitives: **Selections**
   (a span of words) and **Moments** (an instant);
2. **The timeline is demoted**: one program-time axis carrying only `{id, frameRate, frameCount}` —
   no clips, no tracks, no central event registry (reported; the Timing & Assembly docs);
3. **Projection from the performance**: "Their projection gives exact windows once the performance
   exists, so a rewrite never leaves a graphic out of sync" — the Take (actual narration/acting
   timing) supplies the seconds; the compiler projects anchors → windows;
4. **Rewrite safety**: "when a phrase gets longer or shorter, the reveal follows it";
5. **The agent writes the source**: "your agent writes the SVML Source and composes components";
   `hypit check` compiles imports/types/graph edges **without calling any external service**;
   components publish `.schedule`/`.program`/`.visual`; HyperFrames draws per frame;
6. **Traceability**: a Recipe "traces each decision to the Source, Recipe or package that owns it."

### 1.1 Second reference: Remotion (checked 2026-11-22)

Remotion is the older "programmatic video" framework: **video as React/TypeScript code** —
components evaluate per frame, headless Chromium screenshots each frame, MP4 out. Its 2026 state:
an official **agent skill** for Claude Code/Codex ("use agents, render in bulk"), an interactive
Studio preview, and a paid company license on top of the open-source core.

| Axis | Remotion | Hypit |
|---|---|---|
| Source language | general-purpose code (React/TS — Turing-complete) | constrained declarative grammar (SVML) |
| Anchor unit | frame numbers / props | **words** (Selections/Moments) |
| First author | developers (agents via the official skill) | **AI agents** (the agent is the first author) |
| Ceiling | near-unlimited (whole web stack) | narrow composition templates (captions/B-roll/口播) |
| Editing model | edit source → re-render (Studio preview) | edit source → re-compile (preview) |
| Render stack | headless Chromium per frame | HyperFrames per frame (Chromium) |
| License | open core + paid company license | Apache-2.0 + BYOK |

**Read-across**: both independently validate R5 (agent writes a declarative source, deterministic
render); Remotion proves general-purpose code attracts a developer ecosystem, Hypit proves a
constrained grammar anchored to semantic units is safer for agents and rewrite-tolerant; both are
**video composition, not character animation** — no editable geometry, no rigs, no curves, no
physics, no direct manipulation (edit = edit the source and re-render). Their shared blind spot is
exactly mo2d's position: a deterministic **document** (lines/relations/parameters) + stage
display=edit + the agent command stream.

## 2. The pattern, abstracted (five rules)

| Rule | Statement |
|---|---|
| **R1 — primitive moves up** | the authoring primitive is the **intent layer** (script words, beats, named events), not the mechanical apparatus (clips/tracks/keyframes) |
| **R2 — apparatus demoted** | the mechanical representation becomes a **minimal projection** (a timeline that is only `{frameRate, frameCount}`; keys, curves, cuts are compiled output) |
| **R3 — performance supplies timing** | exact windows come from a **timing source at projection time** (a recorded take, a beat sheet, a gesture) — anchors are resolved then, and re-resolved on rewrite |
| **R4 — semantic anchors make rewrites safe** | bind to **words/beats/named events**, never to raw seconds — editing the source never desynchronizes dependents |
| **R5 — AI writes the source, the compiler is deterministic** | the agent authors the declarative file; a local deterministic compiler lowers it; every decision is traceable to its source |

Note the split this forces: **authoring-time** (agent + compiler, may be empirical) vs
**runtime** (pure functions, may never call AI — vision §6 one-vote veto). SVML keeps the same
split: `hypit check` is local and deterministic; only the *authorship* is agentic.

## 3. The sweep — all animation concepts & stages

Verdict key: **替换** = the legacy authoring apparatus can be replaced by an AI-era concept;
**半替** = the *interface* is replaced, the *mechanism* stays (compiled into lines); **保持** =
must stay in lines/laws (vision §6.1: AI 三无 — no precision, no physics, no reproducibility);
**已替** = mo2d already did it.

### 3.1 Authoring pipeline stages

| Stage | Legacy form | AI-era replacement (ⓓ proposals) | Anchor unit | Verdict |
|---|---|---|---|---|
| 剧本 script | a separate document; timings by hand | **the source of truth** — the Script layer (SVML); animation becomes a projection of it | 词/segment | 替换(升格)ⓓ |
| 分镜 storyboard | static boards, manual timing | AI-drafted boards bound to script beats | 镜头号/beat | 替换 ⓓ |
| 设计 design (角色/场景/道具) | hand-drawn assets | AI-generated drafts **compiled into lines** (R5: text-to-vector, sketch→lines) | 物件名 | 半替(产物必须编译)ⓓ |
| Layout/构图 | manual placement | declarative composition intent ("角色在左，夕阳背景") → layout relations | 物件名/方位词 | 替换(接口)ⓓ |
| 绑定 rig | manual bones/weights/IK | **auto-rig with a confirm checkpoint** (Mixamo/Animated Drawings pattern; vision §6) | 部位名 | 半替(接口)——已立项 |
| 动画 keyframing/timing | **the timeline**: scrub, place keys, tune curves | **beat anchoring + feel words**: "在他说到 X 时跳一下，更弹" — the compiler lowers anchors → tick keys (D7 truth unchanged); the timeline UI becomes a *projection* for inspection | 节拍/词/命名事件 | **替换(最大单点)** ⓓ |
| 中间帧 inbetween | manual drawings | stroke-geometric in-between generation **between exact keys** (landscape R5: curves, not pixels) | 两端关键帧 | 半替 ⓓ |
| 缓动 easing | curve editors | feel words ("更弹") → segment easing from a named feel-set | 感觉词 | 替换(接口)ⓓ |
| 特效 FX | effect stacks, manual params | named intent ("湍流的水") → sim/oscillate relation parameters (AI picks law+params, §6.1 follow-up) | 效果名 | 半替 ⓓ |
| 灯光/材质 | light rigs, material editors | parameters = channels; AI sets parameters (3d-extension split) | 灯光名/材质名 | 半替 |
| 渲染 render | render farm (deterministic) | **保持** — runtime rendering stays pure algebra (one-vote veto); offline neural rendering only as a seeded operator (EbSynth precedent) | — | 保持 |
| 合成/剪辑 compositing/editing | track stacking, cut points | declarative layer intent anchored to script beats (`during=` 的动画版) | 节拍/selection | 替换(接口)ⓓ |
| 音频/口型 audio/lipsync | manual visemes | audio payload → auto viseme track (C4; Adobe Sensei precedent) | 音素/词 | 替换 ⓓ(C4) |
| 审阅 QA | human eyeball | agent replay-review + validator (drift reports) | 镜头号 | 增强 ⓓ |
| 发布 publish | export dialogs | declarative export config (file-format §4 already) | 产物名 | 已替(格式层) |

### 3.2 Software mechanisms (the apparatus)

| Mechanism | Legacy form | AI-era replacement | mo2d status |
|---|---|---|---|
| 时间轴/轨道/律表 timeline | clips+tracks+keys in seconds | **minimal projection** (SVML: `{frameRate, frameCount}`) + beat anchors above; mo2d already made tracks *lines* (mapping 4) — the remaining step is the anchor layer above ticks ⓓ | 半替(界面) |
| 关键帧 keyframes | manual placement | AI writes keys (vision §6; landed as commands) | 已替 |
| 缓动曲线 easing curves | manual curve editors | feel-set named parameters + AI selection | 接口ⓓ |
| 图层/混合 layers/blend | stack + blend modes | group relation (landed B4) + blend 待 | 已/📋 blend |
| 符号/复用 symbols | symbol systems | curve reuse (§1.0.1) + group packages | 已替 |
| 相机 camera | keyframed camera | camera = box line + AI-written track (mapping §3.1) | 机制✅/接口ⓓ |
| 状态机/交互 state machines | visual SM editors | **parameters + respond relations** (landed C7); intent compiler ("点了就跳") = C1 territory | 机制✅/编译ⓓ |
| 物理 physics | baked curves or realtime sims | **laws live in lines**: sim (replay-from-seed) / oscillate / constraints — AI picks law+params (vision §6.1) | ✅ 保持(定律在参数) |
| 约束/IK constraints | solver editors | constraint relation (library §1) — engine pending | 📋 |
| 权重/蒙皮 weights/skinning | weight painting | linear bind landed; multi-bone blending pending | 部分 |
| 洋葱皮 onion skin | ghost frames | overlay lines (session layer) | ✅ 机制 |
| 素材库/模板 libraries | asset folders | style templates (B2) + group packages + svg.import layers | ✅ |
| 导出/发布 export | dialogs/presets | declarative config (file-format §4) | ✅ |
| 预览 preview | play button | display=edit (§1.5) — preview is continuous | ✅ |

### 3.3 What cannot be replaced (the keep set)

Precision, physical laws, determinism, runtime evaluation — vision §6.1's AI 三无 against the
lines' 三有. In stage terms: **render stays algebra** (per-frame AI = one-vote veto), **physics
stays in relations**, **runtime interaction stays respond/parameters**, and the **tick truth stays
raw integers (D7)** — every anchor layer above compiles *down to* ticks and never replaces them.

## 4. The mo2d consequence (proposals ⓓ, pending owner confirmation)

- **P1 — beat-anchor authoring layer**: a Script/Beat layer above ticks (Selections/Moments
  analog): named anchors bound to words/beats/events; tracks and responds bind to anchors; the
  compiler lowers anchors → tick keys given a timing source (perform-record, narration audio,
  or a beat sheet). D7 tick truth unchanged — anchors are authoring-level, compiled down;
- **P2 — the timeline becomes a projection view**: the timeline UI renders the compiled keys
  (already true in data: tracks are lines) plus the anchor lanes — inspection, not authoring;
- **P3 — feel-words compiler**: "更弹/更顺" → segment easing from the named feel-set (C7
  vocabulary extension);
- **P4 — script layer**: the Script as an importable document layer (SVML's script), with
  selections as respond/guard addresses — "在他的台词说到 X 时" becomes an address;
- **P5 — agent-writes-source discipline**: every proposal keeps R5 — the agent authors the
  declarative layer, `check` stays local/deterministic, traceability per decision (Recipe analog:
  the command stream already is the Recipe).

Note the precedent already landed: **C8 gesture lines are the mechanism-level Selections/Moments**
— phase edges anchored to events instead of seconds; P1 generalizes the anchor vocabulary from
pointer events to script words/beats.

## 5. Sources

Hypit custom-components + docs (fetched 2026-11-22); Open Trend listing; the mo2d bundle docs
(vision §6.1, landscape, line-model, relations, mvp D7/D10). See frontmatter `sources`.
