---
type: Reference
title: "Animation Software Landscape Survey: Traditional & AI Tools and Concepts"
description: First-principles survey of 20+ traditional and AI-era animation tools — paradigms, concept inventories, essential-vs-incidental decomposition, novice learning-cost evidence, and a control-dimension verdict on AI tools; distills the irreducible core of animation authoring and feeds the everyday-concept layer (vernacular.md).
tags: [landscape, survey, first-principles, essential-incidental, ai-tools, learning-cost, interactivity]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-17T12:47:04Z }
sources:
  - id: adobe-animate
    resource: https://helpx.adobe.com/animate/desktop.html
    title: "Adobe Animate Help(Tweens 图解/动画 TOC)"
  - id: wikibooks-flash
    resource: https://en.wikibooks.org/wiki/Beginner%27s_Guide_to_Adobe_Flash/Symbols/Movie_Clip
    title: "Wikibooks:movie clip 与 graphic symbol 的混淆"
  - id: adobe-ae
    resource: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-basics/animation-basics.html
    title: "After Effects:Animation basics"
  - id: school-motion
    resource: https://schoolofmotion.com/blog/how-to-learn-after-effects
    title: "School of Motion:学习 AE 需要多久"
  - id: toonboom
    resource: https://docs.toonboom.com/help/harmony-22/premium/nodes/about-peg-node.html
    title: "Toon Boom Harmony 文档(peg/node view/deformers)"
  - id: reddit-harmony
    resource: https://www.reddit.com/r/ToonBoomHarmony/comments/1ky60pj/how_do_you_organize_your_nodes_node_hell
    title: "r/ToonBoomHarmony:node hell 与学习曲线"
  - id: opentoonz
    resource: https://opentoonz.readthedocs.io/en/latest/creating_movements.html
    title: "OpenToonz 文档(xsheet/columns/pegbars)"
  - id: tvpaint
    resource: https://doc.tvpaint.com/docs/animation-first-steps/instances/instance-concept
    title: "TVPaint 手册(instances/exposure cells)"
  - id: krita
    resource: https://docs.krita.org/en/user_manual/animation.html
    title: "Krita 手册:Animation"
  - id: dreams
    resource: https://help.procreate.com/dreams/handbook/interface-and-gestures/timeline
    title: "Procreate Dreams Handbook(timeline/Perform/flipbook)"
  - id: moho-features
    resource: https://moho.lostmarble.com/pages/features
    title: "Moho 官方特性页(bones/smart bones/switch/physics)"
  - id: pivot
    resource: https://pivotanimator.net/Pivot_Animator_Help_4-1.pdf
    title: "Pivot Animator Help(Add Frame)"
  - id: scratch
    resource: https://en.wikipedia.org/wiki/Scratch_(programming_language)
    title: "Wikipedia:Scratch(积木式编程)"
  - id: spine-guide
    resource: https://en.esotericsoftware.com/spine-user-guide
    title: "Spine User Guide(bones/slots/attachments)"
  - id: spine-constraints
    resource: https://en.esotericsoftware.com/spine-constraints
    title: "Spine:Constraints(IK/transform/path/physics)"
  - id: live2d-glossary
    resource: https://docs.live2d.com/en/cubism-editor-manual/glossary
    title: "Live2D Cubism 手册词汇表(parameter/keyform/deformer)"
  - id: live2d-auto
    resource: https://docs.live2d.com/en/cubism-editor-manual/auto-generation-of-deformer
    title: "Live2D:自动生成变形器(建议人工校验)"
  - id: rive-sm
    resource: https://rive.app/docs/editor/state-machine
    title: "Rive:State Machines(states/transitions/layers)"
  - id: rive-listeners
    resource: https://rive.app/docs/editor/state-machine/listeners
    title: "Rive:Listeners(指针/键盘条件→动作)"
  - id: rive-data
    resource: https://rive.app/docs/editor/data-binding
    title: "Rive:Data Binding(view models)"
  - id: lottie-spec
    resource: https://lottie.github.io/lottie-spec/latest/
    title: "Lottie 格式规范(只烘焙关键帧)"
  - id: unity-animator
    resource: https://docs.unity.com/en-us/engine/current/manual/animation-section/animation-mecanim/animation-animator-controller/class-animator-controller
    title: "Unity:Animator Controller(states/parameters/blend trees)"
  - id: godot-tree
    resource: https://docs.godotengine.org/en/stable/tutorials/animation/animation_tree.html
    title: "Godot:AnimationTree(blend spaces/state machine)"
  - id: dragonbones
    resource: https://dragonbones.github.io/en/animation.html
    title: "DragonBones 官网(armature/slot/skin)"
  - id: creature
    resource: https://www.kestrelmoon.com/creaturedocs/Tutorials_And_Walkthroughs/Quickstart.html
    title: "Creature 文档(auto-mesh/weights/motors)"
  - id: runway-gen4
    resource: https://runway.com/research/introducing-runway-gen-4
    title: "Runway Gen-4(prompt+参考图→视频)"
  - id: luma-ray32
    resource: https://lumalabs.ai/news/introducing-ray-3-2
    title: "Luma Ray3.2(单片段最多 16 个关键帧)"
  - id: kling-brush
    resource: https://kling.ai/quickstart/how-to-animate-image-parts
    title: "Kling:Motion Brush(涂选区域+画运动路径)"
  - id: sora-features
    resource: https://www.maginative.com/article/six-top-features-of-sora-openais-new-ai-video-creation-platform
    title: "Sora 特性综述(storyboard/remix/recut)"
  - id: gen-video-limits
    resource: https://morphic.com/resources/how-to/ai-video-troubleshooting
    title: "生成视频的失败模式:identity drift 等"
  - id: deepmotion
    resource: https://www.deepmotion.com/saymotion
    title: "DeepMotion SayMotion(文本→骨骼动画 FBX/GLB/BVH)"
  - id: mixamo
    resource: https://helpx.adobe.com/creative-cloud/help/mixamo-rigging-animation.html
    title: "Adobe Mixamo(自动绑定+动作库)"
  - id: cascadeur
    resource: https://cascadeur.com/blog/general/ai-assistance-in-3d-character-posing-how-it-works-in-cascadeur
    title: "Cascadeur(AutoPosing/AutoPhysics)"
  - id: sensei-lipsync
    resource: https://blog.adobe.com/en/publish/2019/11/07/animate-characters-with-auto-lip-sync-powered-by-adobe-sensei
    title: "Adobe Sensei:Animate 自动口型"
  - id: roto-brush
    resource: https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/roto-brush-refine-matte.html
    title: "AE Roto Brush(AI 传播+人工精修)"
  - id: ebsynth
    resource: https://ebsynth.com
    title: "EbSynth(例图传播,确定性)"
  - id: tooncrafter
    resource: https://arxiv.org/html/2405.17933v1
    title: "ToonCrafter:生成式卡通中间帧"
  - id: animated-drawings
    resource: https://ai.meta.com/blog/ai-dataset-animation-drawings
    title: "Meta Animated Drawings(涂鸦→自动绑定)"
  - id: spline-ai
    resource: https://spline.design/ai-generate
    title: "Spline AI(prompt→四选一→落入可编辑场景)"
  - id: figma-ai
    resource: https://www.figma.com/ai
    title: "Figma AI / Figma Make(prompt→可编辑设计稿)"
  - id: motiff
    resource: https://motiff.com
    title: "Motiff(文本/线框/截图→可编辑界面)"
  - id: v0-loop
    resource: https://lovable.dev/guides/lovable-vs-v0
    title: "v0/Lovable:prompt→制品精修循环与版本回退"
  - id: mdm
    resource: https://arxiv.org/abs/2209.14916
    title: "Motion Diffusion Models(ICLR 2023)"
  - id: line-inbetween
    resource: https://arxiv.org/abs/2309.16643
    title: "Deep Geometrized Cartoon Line Inbetweening(线条几何派)"
---

**English** | [简体中文](landscape.zh-CN.md)

# Animation Software Landscape Survey: Traditional & AI Tools and Concepts

Session goal (owner, 2026-11-17): strictly by first principles, deeply survey the tools and concepts of
existing traditional and AI animation software, and plan mo2d from zero — the final vision being that a
novice user, without understanding professional terminology, can create, adjust, play **interactive**
animations using only everyday basic concepts combined with AI, with minimal extra learning cost and
maximal controllability and familiarity. The normative statement is recorded in
[vernacular.md](vernacular.md) §0; this document is the survey half of that goal.

## 0. Purpose and method

- **Purpose**: extract, across the whole industry, (a) the complete inventory of user-facing concepts,
  (b) which are **essential** (any system that "makes things change over time / respond to input" needs
  them) vs **incidental** (historical/GUI/architecture artifacts), (c) the documented learning cost, and
  (d) what AI-era tools prove about novice authoring. The output feeds [vernacular.md](vernacular.md)
  (the everyday-concept layer) and the roadmap;
- **Method**: per tool — paradigm in one sentence; concept inventory; essential/incidental judgment;
  learning-cost evidence. AI tools are additionally scored on five **control dimensions**:
  **E**ditable (fix one part without redoing everything), **D**eterministic (same input → same output),
  **I**ncremental (adjust without re-rolling), **C**omposable (mix with other content), e**X**portable
  to an editable format;
- **Scope**: 3 families — traditional timeline/frame tools (§1), rig-based & interactive tools (§2),
  AI-era tools (§3). Moho's deep coverage lives in [moho-mapping.md](moho-mapping.md) (§1.5 here is a stub).

## 1. Traditional timeline/frame tools

### 1.1 Adobe Flash / Animate[^adobe-animate]

**Paradigm**: layer-based vector timeline; reusable **symbols** on the stage animated by **tweens**
between keyframes; historically coupled to an interactive runtime (Flash Player → HTML5).

**Concepts**: stage / timeline / layers+folders / frame-keyframe-blank keyframe / frame-by-frame /
onion skin / symbols (graphic·button·movie clip) / library+instances / nested timelines / classic tween /
motion tween (tween span, property keyframes) / shape tween / motion path / Motion Editor / easing
presets+custom / motion presets / bone tool (armature, IK) / mask layers / motion guide layers / scenes /
camera / filters+blends / 3D rotation / text / sound-video / ActionScript+snippets / components /
publish targets.

**Judgment**: essential — timeline, keyframes, tweens, easing, onion skin, layers, reuse, motion path,
bone-as-optional-depth. Incidental — the graphic/button/movie-clip taxonomy (a Flash-Player-runtime
artifact: movie clips carry an independent timeline and do not animate while scrubbing — a documented
beginner trap[^wikibooks-flash]), scenes, ActionScript/publish machinery, and the *coexistence of two
tween systems* (classic + motion), which Adobe itself calls transitional.

**Learning cost**: the manual's animation section alone teaches ~20 topics (three tween types, motion
editor, motion paths, bone tool, masks…) before any interactivity[^adobe-animate].

### 1.2 Adobe After Effects[^adobe-ae]

**Paradigm**: layer-and-composition compositing; every animatable property is a function of time
(keyframes or expressions), evaluated through a render-order DAG of nested compositions.

**Concepts**: project+footage / composition / layers (footage·solid·shape·text·null·camera·light·adjustment)
/ stopwatch (the universal "this changes over time" switch) / keyframes / interpolation (linear·Bézier·hold)
/ layer bar vs Graph Editor (value graph+speed graph) / expressions (JavaScript)+pick-whip / parenting+nulls /
precomposing / render order / masks+track mattes / shape layers+path ops / text animators / puppet pins /
time-remapping / effects+animation presets / tracking+keying+Roto / 3D layers / render queue.

**Judgment**: essential — composition-as-scene, layers, per-property keyframes, interpolation, graph
editor, easing, parenting, masks, presets. Incidental — precomps-as-render-order-patch, expressions-as-JS,
the *two separate transparency systems* (masks vs track mattes), time-remapping, footage-production tools,
the layer-bar/graph dual mental model.

**Learning cost**: community consensus ≈ weeks for basics, 6–8 months to competence, years to mastery;
beginner threads call it "the steepest learning curve of any editing software"[^school-motion].

### 1.3 Toon Boom Harmony[^toonboom]

**Paradigm**: professional cut-out rigging with **two UIs over one model** — a timeline/xsheet for
animating and a **node graph** defining rig structure and compositing.

**Concepts**: drawing layers (line-art/color-art) / timeline·xsheet / drawing substitution / **peg**
(transform container; pegs stack into rig hierarchies, master pegs root characters) / parenting network /
node view (Drawing·Peg·Composite·Cutter·effects·Camera) / cutter node / deformers (Curve·Envelope·Bone,
as deformer chains) / IK / function curves / onion skin / templates+library / camera / game-rig export.

**Judgment**: essential — drawing layers, substitution, pegs-as-transform-hierarchy (≈ AE parenting),
deformation, keyframes/easing, onion skin. Incidental — the node view as primary rig surface (compositor
heritage), cutter-as-distinct-node (it is just a mask), the timeline-vs-node split itself, deformer
grouping conventions.

**Learning cost**: rigging competency is structured into 3+ sequential official courses; community
threads document "node hell" and "winning the fight against Harmony"[^reddit-harmony].

### 1.4 Frame-by-frame family — OpenToonz · TVPaint · Krita · Procreate Dreams

- **OpenToonz**[^opentoonz]: digital emulation of the paper pipeline — xsheet (cell grid), columns,
  levels, drawings+exposure, **pegbars** (hierarchy, from physical animation stands), camera+table,
  sub-xsheet, inbetweening, Plastic mesh deformation, FX schematic (node graph), function editor,
  scan/cleanup/ink&paint. Essential: xsheet/columns/exposure (per-drawing timing), pegbars, inbetweening,
  onion skin. Incidental: paper-era names (pegbar/table), scan pipeline, FX node graph. The manual is
  557 pages; community often steers beginners to the Tahoma2D fork "focused on ease of use".
- **TVPaint**[^tvpaint]: raster frame-by-frame as **instances** (image container) + head cell + exposure
  cells (extend a hold by dragging), auto-break instance, light table, FX stack, puppet layer, timesheet.
  Essential: layer/image/instance/exposure (the classical "hold N frames"), light table. Incidental:
  head-vs-exposure editing semantics leaking into UX, FX stack ordering, George scripting.
- **Krita**[^krita]: paint program + frame-by-frame mode — timeline docker, create-duplicate vs
  create-blank frames, hold frames, frame color labels, onion skin docker, animation curves docker
  (scalar keyframes on e.g. a Transform Mask — bolt-on tweening), storyboard docker, RAM-management
  guidance (a raster-architecture cost the manual spends pages on). Essential: frames, onion skin,
  frame rate. Incidental: mask-based transform tweening workaround, memory management itself.
- **Procreate Dreams**[^dreams]: tablet movie editor with **three modes over one timeline** — Compose
  (arrange/trim), **Perform (record gestures in real time → keyframe tracks)**, Keyframe (manual), plus
  flipbook drawing tracks with onion skin, easing, audio, groups/masks. Essential: timeline, keyframes,
  flipbook, and especially **perform-recording** (capture-then-edit — a genuinely alternative authoring
  model). Incidental: the three-mode split as touch adaptation, "theater" naming.

### 1.5 Moho (stub)[^moho-features]

Bone-based cut-out over vectors + point-level animation; **Smart Bones** (dial a bone → arbitrary
correction animation), **Switch Layers** (swap artwork states — phonemes/poses), physics/particles for
secondary motion. Full mapping: [moho-mapping.md](moho-mapping.md).

### 1.6 Mass-market beginner tools — Pivot · Scratch

- **Pivot Animator**[^pivot]: pose jointed stick figures, "Add Frame", repeat. Total concept count ≈ 5
  (figure, joint, frame, add-frame, play); interpolation exists but is implicit — the user never learns
  the word. Proof that **< 6 concepts suffice to produce animation**.
- **Scratch**[^scratch]: block-based programming; animation = looping scripts switching **costumes**
  ("next costume" block) with `wait`/`repeat`/`forever` as the clock and `glide N secs` as a built-in
  eased tween; events/broadcast sequence scenes; cloning spawns copies. Time is authored as program
  control flow, not a timeline. Proof that **everyday mechanics (picture + patience) can replace the
  keyframe machinery** when syntax disappears.

## 2. Rig-based and interactive tools

### 2.1 Spine[^spine-guide][^spine-constraints]

**Paradigm**: keyframe a hierarchy of **bones**; images attach through **slots** (slot controls which
attachment shows + draw order); one rig plays back cheaply at runtime.

**Concepts**: skeleton / bone / slot / attachments (region·mesh·bounding box·clipping·path·point) /
mesh+weights (smooth multi-bone deformation) / skins (named attachment sets — outfit swap) / constraints
(IK·transform·path·physics·sliders; each with mix% and apply order) / animations+keys+curves (dopesheet,
graph) / events (named time markers → host) / runtimes (Unity/Unreal/Cocos/PixiJS…).

**Interactivity**: none in-file — the host picks and crossfades animations; events are the only contract.
**Judgment**: essential — bone hierarchy, the slot/attachment indirection, keys+curves, events;
derivable — IK, mesh/weights; incidental — skins, path/point attachments, physics constraints.

### 2.2 Live2D Cubism[^live2d-glossary][^live2d-auto]

**Paradigm**: rig a layered illustration into a **parameter space** — every parameter value maps to
stored **keyforms** (deformations); a pose *is* a parameter vector; "2.5D" without 3D models.

**Concepts**: ArtMesh+mesh / deformers (warp·rotation; nested hierarchies) / parts / **parameter**
(range+default+slider) / **key**·keyform / XY parameter chaining (3×3 keyform grid) / draw order (0–1000)
/ skinning / **physics** (pendulum input→output chains, computed at runtime, not keyframed) / motion
files (animate *parameters* over time, not shapes) / motion-sync (lip sync) / templates / hit areas.

**The defining move**: **pose is decoupled from time.** Rigging builds `parameter → deformation`;
motions are thin scripts animating parameter values; face trackers or physics can drive the same rig
live — the timeline is optional, the parameter space is not. This is why VTubers exist on Live2D.

**Judgment**: essential — parameter+keyform, ArtMesh/mesh, deformers, draw order. Incidental — parts,
physics, skinning, motion files as such. Note: its own docs say auto-generated meshes/deformers need
manual checking[^live2d-auto] — **AI proposes, human validates** as an official workflow.

### 2.3 Rive[^rive-sm][^rive-listeners][^rive-data]

**Paradigm**: state machines authored *inside the design file*, so timelines, rigging, and interaction
logic ship together and respond to inputs with no playback code.

**Concepts**: artboard (nestable, components, layout) / bones+joints (3 binding modes: parent transform /
path-vertex weights / mesh weights) / constraints (IK·distance·translation·rotation·scale·transform·
follow path) / timelines / **state machine** — states (animation·blend) + Entry/Any State/Exit +
transitions (conditions·exit time) + parallel layers / **inputs** (boolean·number·trigger — legacy,
deprecated) / **listeners** (pointer enter·exit·move·down·up·drag·click, keyboard, gamepad, focus;
actions: set input/view-model property, move object to pointer, report event) / events (open URL·audio·
general; fired from timelines/states/listeners) / **data binding** (view models + instances; bindings
connect data to properties bidirectionally — "once bound, the hierarchy no longer matters").

**Interactivity**: the full loop in-file — listener (perceive) → condition on input/view-model (decide) →
state switch + crossfade (respond) → events back to the host. **Judgment**: essential — artboard,
timeline+keys, state machine, one symbolic input parameter, listeners; incidental — bones/constraints
(optional depth), events-as-URL/audio, nested artboards. Rive deprecated bespoke inputs in favor of
typed data binding — hierarchy-free named data is easier for code *and AI agents* to target.

### 2.4 Lottie / Bodymovin[^lottie-spec]

**Paradigm**: a **baked export format** — AE compositions compiled to JSON (Bézier path data, hierarchy,
keyframes with easing tangents) re-rendered by player libraries.

**Cannot carry**: rigs, expressions, state machines, logic — anything beyond interpolation. Interactivity
reduced to external code scrubbing play ranges. The lesson: **bake nothing you can evaluate** — exporting
keys-only amputates exactly what makes animation alive.

### 2.5 Game engines + the rest (brief)

- **Unity Mecanim**[^unity-animator]: animation clips; Animator Controller = flow-chart of states +
  guarded transitions on **parameters (int·float·bool·trigger)**; blend trees mix clips continuously
  (1D thresholds / 2D freeform).
- **Godot**[^godot-tree]: AnimationPlayer tracks (keyframed properties on any node); AnimationTree with
  Blend2/3·OneShot·TimeSeek·Transition nodes, BlendSpace1D/2D (points in parameter space, Delaunay
  triangulation), StateMachine (advance conditions/expressions), RESET animation for deterministic
  initial state.
- **DragonBones**[^dragonbones]: open-source Spine-class — armature/bone/slot/skin/mesh+weights/IK/events.
- **Creature**[^creature]: procedural rigging — auto-meshing, auto-weighting, **motors** (bone/deform/
  physics motors with keyable attributes, phase offsets).

## 3. AI-era tools

### 3.1 Generative video (prompt → pixels)

- **Runway Gen-4**[^runway-gen4]: prompt + reference images → video ("world consistency" across shots);
  the old Motion Brush (paint a region, drive it with direction/speed sliders) was dropped.
- **Pika**: Pikaframes (start/end images → generated transition), Pikaffects (**verb presets**: inflate,
  melt, explode), Pikadditions.
- **Luma Ray3.2**[^luma-ray32]: up to **16 keyframes** per clip ("frame-level directorial control") +
  composable named camera moves — the industry converging on "sparse conditions + generated fill".
- **Kling**[^kling-brush]: Motion Brush (paint area → draw a motion path → mark static areas); Motion
  Control (transfer full-body motion from a reference video).
- **Sora**[^sora-features]: **Storyboard** (descriptions on a timeline), Remix (replace elements with a
  strength slider), Re-cut/Loop/Blend — reviewers note its value is "editorial rather than visual".
- **Shared limits**[^gen-video-limits]: identity drift, temporal drift, non-determinism (seeds don't
  guarantee semantics), no selective editing — fixing one element = regenerate everything.

**Verdict**: E✗ D✗ I(re-prompt only) C✗ X✗. **Zero terminology, near-zero control** — the artifact is a
dead end. Natural language removes the terminology barrier; the flat artifact removes everything after.

### 3.2 Motion capture & synthesis (AI → editable rig data)

- **DeepMotion SayMotion**[^deepmotion]: text→3D motion; exports **FBX/GLB/BVH** — fully editable
  downstream. E✓ D✓ I✓(layered prompts) C✓(retarget) X✓.
- **Plask**: video→skeletal motion→clean-up tools→retarget→FBX/BVH/GLB. AI as *preprocessing into a
  deterministic pipeline* — an honest contract novices understand.
- **Move AI**: professional markerless multi-camera mocap; even pro-grade capture needs cleanup —
  "AI gets 90%, deterministic editing gets the last 10%" is the norm.
- **Adobe Mixamo**[^mixamo]: the canonical auto-rig — upload mesh → automatic joint placement (user can
  nudge markers) → motion library → FBX. **The highest-leverage AI move for novices in the last decade:
  it deleted the most terminology-heavy step (rigging) by inferring structure from geometry, with a
  visible confirmation checkpoint.**
- **Cascadeur**[^cascadeur]: the user keeps controllers; AutoPosing (neural) infers natural poses,
  AutoPhysics corrects trajectories for balance/momentum — **AI as constraint solver inside the user's
  document, not artifact factory**. Closest existing model to what mo2d needs.

### 3.3 2D-specific AI

- **Adobe Sensei in-product**[^sensei-lipsync][^roto-brush]: Auto Lip-Sync (visemes mapped onto the
  timeline), Roto Brush 2/3 (brush once, AI propagates, human refines) — AI performs one tedious step
  and produces **vanilla editable data**. This "invisible AI" contract is why professionals trust it.
- **Live2D auto features**[^live2d-auto]: auto mesh generation, auto deformer generation (v5.1) — with
  official guidance to check results manually. AI proposes the rig, user validates, downstream is
  deterministic *and interactive*.
- **Meta Animated Drawings**[^animated-drawings]: upload any single-character **drawing** → CV detects,
  user confirms joints → auto-rigged 2D skeleton → motion library. Zero terminology, works on children's
  drawings (~180k-drawing dataset), open-sourced. Proof that **sketch → inferred rig → motion** is a
  day-one "magic moment" novices can have.
- **EbSynth**[^ebsynth]: paint one keyframe, deterministic example-based propagation across frames —
  earns artist trust; fails at occlusions/large motion (the price of synthesizing pixels instead of
  deforming structure).
- **ToonCrafter**[^tooncrafter]: generative cartoon in-betweening (two keyframes + optional sketch) —
  the most novice-friendly AI task imaginable ("connect these two poses"), destroyed by its raster
  output. Research split: raster-generative vs **stroke-geometric**[^line-inbetween] — the geometric
  branch emits structures a vector editor can represent.

### 3.4 AI-native design tools (interaction patterns)

- **Spline AI**[^spline-ai]: prompt → **four preview images** → pick one → generated *into the editable
  scene*; newer editor lets AI "read the scene, write to it". Preview-as-menu absorbs non-determinism
  (choice replaces seed); editing happens in the normal tool surface, not a chat window.
- **Figma AI / Make**[^figma-ai]: prompt → artifact in the standard representation (layers, auto-layout,
  components) → direct manipulation → re-prompt. AI writes the same primitives professionals edit —
  nothing second-class.
- **Motiff**[^motiff]: accepts **whatever artifact the novice already has** (text, wireframe, screenshot,
  PRD) and normalizes it into standard editable primitives.
- **v0 / Lovable**[^v0-loop]: prompt→artifact→refine loop; always-visible inspectable artifact;
  **versioned rewindable history** around every AI action — the trust mechanism.

### 3.5 Academic frontier (brief)

Motion Diffusion Models (text→motion, transformer backbone)[^mdm]; sketch-to-rig (Animated Drawings
pipeline)[^animated-drawings]; LLM→SVG vector generation (StarVector etc. — output editable by
construction); cartoon in-betweening split into raster-generative vs stroke-geometric
[^tooncrafter][^line-inbetween].

### 3.6 The control-dimension verdict

| Family | E editable | D deterministic | I incremental | C composable | X export editable | Pattern |
|---|---|---|---|---|---|---|
| Generative video (Runway/Pika/Luma/Kling/Sora) | ✗ | ✗ | re-prompt only | ✗ | ✗ | zero terminology, zero control |
| Motion AI (SayMotion/Plask/Move/Mixamo) | ✓ | ✓ | ✓ | ✓ | ✓ | AI → editable rig data |
| Cascadeur | ✓ | ✓ | ✓ | ✓ | ✓ | AI fills plausibility inside the user's document |
| Sensei in-product / Live2D auto | ✓ | ✓ | ✓ | ✓ | ✓ | invisible AI producing vanilla data |
| Krikey (prompt+template→clip) | ✗ | ✗ | templates | ✗ | ✗ | doesn't scale into authoring |
| AI design tools (Spline/Figma/Motiff/v0) | ✓ | partial | ✓ | ✓ | ✓ | generate *into* the scene; previews absorb non-determinism |

**The inversion every trusted tool shares**: AI output is deposited into **the same editable, deterministic
representation the user then manipulates by hand** — never a side artifact.

## 4. Consolidated concept inventory (essential vs incidental → mo2d reduction)

Judgment keys: **E** = essential, **D** = derivable/optional depth, **I** = incidental. "mo2d" column
references the nine mappings of [line-model.md](line-model.md) §3 (N1–N9) and the relation library of
[relations.md](relations.md) §1.

| Concept | Tools | Fundamental role | Jdg | mo2d reduction |
|---|---|---|---|---|
| Time axis (timeline/frame strip) | all | where "when" lives | E | N4 track lines |
| Frame / cell / costume | all but Scratch-likes | discrete visual state | E | N4 node on track line (discrete value) |
| Keyframe (value at time) | Animate·AE·Harmony·Krita·Dreams·Moho | sparse authored states | E | N4 node (position=tick, value=node attribute) |
| Interpolation / tween / inbetween | all | synthesize between states | E | N4 segment attribute |
| Easing / curve editor | Animate·AE·Harmony·Krita·Dreams | rate of change | E | N4 segment attribute (per-component handles) |
| Layer / column / track | all | parallel z-ordered streams | E | N7 set relation + zOrder parameter |
| Onion skin | frame-by-frame family | see neighboring states | D | session-layer overlay (lines) |
| Reuse (symbol/library/instance) | Animate·TVPaint·Harmony·Scratch | define once, place many | E | one curve, N shape relations (relations.md §1.0.1) |
| Transform hierarchy (parent/peg/pegsbar) | AE·Harmony·OpenToonz·Moho | motion inheritance | E | bind + node sharing (N2/N3) |
| Deformer / bone / puppet | Harmony·Animate·AE·Moho·OpenToonz·Spine·Live2D·Rive | non-rigid reshaping | E(char) | N2/N3 (bone=segment, skeleton=polyline) |
| Mesh + weights | Spine·Live2D·Rive·DragonBones·Creature | smooth multi-bone bend | D | N5 warp/weft lines + bind weights |
| Artwork substitution / switch | Harmony·Moho·Scratch | discrete state swap | E | discrete parameter on track (§3.1 line-model) |
| Mask / matte / cutter | Animate·AE·Harmony | clip by another's shape | E | shape relation + fill rule (a relation parameter) |
| Draw order (z) | Live2D·all layers | occlusion | E | zOrder parameter (moho-mapping §7.1) |
| Parameter space (param→pose) | Live2D·Unity·Godot·Rive | pose decoupled from time | E | **parameters** (first-class already) |
| Blend space (N poses ↔ params) | Unity·Godot·Rive·Live2D | continuous pose blending | E | parameters + relations sampling multiple tracks |
| Physics / pendulum | Moho·Live2D·Spine·Creature | autonomous secondary motion | D | N9 sim relation (landed B6) |
| Events (time markers → host) | Spine·Rive·Unity·Godot | sync with app logic | E | **respond** relation family (roadmap C7, confirmed 2026-11-17) |
| State machine (guarded transitions) | Rive·Unity·Godot | discrete behavior selection | E | **discrete parameter + guarded respond** (C7, confirmed 2026-11-17) |
| Listeners (input → action) | Rive | in-file interaction binding | E | **respond** relation on a target line (C7, confirmed 2026-11-17) |
| Node graph (rig/FX/composite) | Harmony·OpenToonz | declarative wiring | I | relations (but as *functions*, not a GUI) |
| Precomp / nesting | AE·OpenToonz | encapsulate a group | E(fn) | group relation (N7, landed B4) |
| Expressions / scripting | AE·Animate·TVPaint·OpenToonz | programmatic relations | I | custom relation (sandboxed IR) |
| Exposure / hold | OpenToonz·TVPaint·Krita | keep N frames | E | track node spacing |
| Presets (motion/easing) | Animate·AE | packaged named intent | E(pattern) | style templates (B2) + AI |
| Perform / record gestures | Dreams·AE(Motion Sketch) | capture-then-edit | E(onramp) | **perform-record** (C7: gesture → track keys via auto-key, confirmed 2026-11-17) |
| Camera / stage | Animate·OpenToonz·Harmony | viewport transform | E | box line (line-model §3.1) |
| Scenes / comps-of-comps | Animate·AE·Dreams | document segmentation | I | document = one stage; reuse via groups |
| Paper heritage (xsheet/pegbar/table/stopwatch) | OpenToonz·TVPaint | digitize classical workflow | I | keep function, never the name |

**Reading of the table**: every essential row already reduces to the mo2d kernel (lines/nodes/three-level
attributes/relations/parameters) — the kernel needs **no format change** to express anything the industry
considers essential. Two capability clusters are not yet first-class in the relation library: the
**interaction cluster** (events, state machines, listeners) and the **capture cluster** (perform-record).
Both are addressed in §6.

## 5. First-principles findings

- **F1 — The irreducible core is tiny and identical everywhere.** Time axis · layered content streams ·
  states (frames/keyframes) · interpolation · rate control · hierarchy · reuse · deformation · discrete
  swap · occlusion · parameters · (for interactive work) input→decision→response. Every paradigm surveyed
  is a packaging of those ~12 ideas; everything else is surface.
- **F2 — Terminology, not concepts, is the beginner tax.** The same concept carries many names
  (tween/inbetween/exposure; peg/parent/null/pegsbar; symbol/instance; mask/matte/cutter; composition/
  scene/sub-xsheet). Remove the naming burden and most documented friction disappears without changing
  the model.
- **F3 — The costliest failures are invisible-state problems.** Movie clips not animating while
  scrubbing[^wikibooks-flash], AE keyframes hidden in collapsed properties, Harmony's "timeline can be
  misleading" duality[^reddit-harmony], TVPaint head-vs-exposure propagation[^tvpaint]. Rule: **every
  state that affects output must be visible where the user is looking.**
- **F4 — Dual representation is proven progressive disclosure**: simple view first, expert view on
  demand (Harmony timeline↔node view; AE layer bar↔graph; Dreams Compose↔Keyframe) — but it must be
  **one model with two skins**, never two models.
- **F5 — Presets are the industry's existing "AI does the naming"**: motion presets, easing galleries —
  packaged, named, officially recommended entry points. An AI assistant is the generalization.
- **F6 — Perform/capture is the strongest novice onramp found** (Dreams Perform, AE Motion Sketch):
  record change first, edit the recording after — keyframe vocabulary deferred, not required.
- **F7 — Rigging is where every learning curve steepens** (Harmony node hell, Smart Bones, bone tool).
  Novice-safe rigging = the system proposes the rig (auto-rig) and exposes only handles — users
  demonstrably operate rigs they could not build (Mixamo, Animated Drawings).
- **F8 — Minimalism works but doesn't scale.** Pivot (<6 concepts) produces animation; Krita proves the
  raster architecture leaks its costs to users. Interpolation + reuse are the leverage concepts a novice
  system must still include under the hood.
- **F9 — Legacy metaphors (xsheet, pegbar, stopwatch, table) serve veterans, not novices.** Keep the
  function, generate the name per user — exactly the work AI can do dynamically.
- **F10 — Each essential function exists in 2–3 redundant variants across tools** (mask/matte/cutter;
  graphic/movie-clip; timeline vs node view). A single clean primitive per function would have prevented
  most documented confusion — mo2d's kernel is exactly that single-primitive bet.
- **F11 — The minimal first-principles model of interactive animation** (convergent across Rive/Unity/
  Godot/Live2D): `pose = f(params)`, `params = g(inputs, time, physics)`; interaction = event → mutate
  inputs → re-evaluate → output converges. A tool is interactive to the degree that (b)–(d) live **in
  the file**, not in host code. The input-type triad **bool · number · trigger** recurs independently in
  Rive, Unity, Godot — convergent evolution, hence near-fundamental.
- **F12 — Live2D's decoupling: pose-without-time first.** The parameter space (slider → pose) is more
  fundamental than the timeline; time is just one driver of parameters. Novices should get a visible
  result (a face following a slider) before ever meeting time.
- **F13 — The AI trust pattern** (the inversion of §3.6): AI proposes → human validates at one visible
  checkpoint (Mixamo markers, Animated Drawings joints, Spline preview-pick) → everything downstream is
  deterministic and hand-editable. Non-determinism is contained at **choice points**, never leaked into
  playback. Generation must **land as native objects** (keys/curves/constraints) — never baked pixels.

## 6. What this survey means for mo2d

- **R1 — Kernel coverage confirmed.** Every essential concept of §4 reduces via the nine mappings; no
  format change is implied by the survey (consistent with the streaming conclusion of architecture §3.2:
  zero format changes).
- **R2 — Gap: in-file interactivity.** The interaction cluster (events / state machines / listeners) is
  essential for the vision ("交互动画") but absent from the standard relation library and roadmap.
  **Confirmed by the owner 2026-11-17: the respond relation family** —
  (event source: tap·touch·release·hover·drag·key·time-point·animation-end) × (condition) →
  (action: write a parameter / switch a discrete track / fire another event); a "state machine" is then a
  discrete parameter + guarded respond relations; a "listener" is a respond relation on a target line.
  Fits "relations are functions" unchanged; **lands as roadmap C7** (vocabulary in
  relations.md §1.2, event-driven evaluation in architecture §3.3, plain-player deterministic
  event handling).
- **R3 — Gap: periodic drivers.** Everyday verbs (sway/breathe/float/bob) want a cyclic track mode or an
  `oscillate` hot-path relation (period/amplitude/phase parameters) — **confirmed 2026-11-17**: the
  `oscillate` relation (roadmap C7).
- **R4 — Auto-rig with a visible confirm checkpoint** (Mixamo/Animated Drawings pattern): AI capability
  (already assigned to AI in vision §6) gains the requirement: the proposed rig lands as ordinary lines +
  bind relations, shown for one-glance correction before play [derived].
- **R5 — Generated motion lands as tracks/keys, never pixels**: text/sketch/perform inputs compile to
  track lines + segment easing (capability-as-data, vision rule 3); ToonCrafter-style in-betweening only
  via the stroke-geometric branch [derived].
- **R6 — Perform-record as a capture parameter** (Dreams Perform pattern): record gestures → track keys
  — **confirmed 2026-11-17**: minimal perform-record in roadmap C7 (gesture → track keys via
  auto-key).
- **R7 — The novice surface is a separate concern from the kernel**: the everyday-concept layer — which
  words AI accepts, what is never shown, the acceptance gates — is specified in
  [vernacular.md](vernacular.md).

## 7. Sources

Consolidated from the per-claim footnotes; fetched during the 2026-11-17 survey. See `sources` in the
frontmatter for the full id → URL mapping. Key clusters: Adobe Animate/AE helpx manuals; Toon Boom docs +
r/ToonBoomHarmony; OpenToonz readthedocs (557-page manual); TVPaint/Krita/Procreate Dreams handbooks;
Pivot help; Scratch (Wikipedia); Spine user guide; Live2D Cubism manual; Rive docs (state machines/
listeners/data binding); Lottie spec; Unity/Godot manuals; Runway/Luma/Kling/Sora official pages +
generative-video failure analyses; DeepMotion/Plask/Move AI/Mixamo/Cascadeur; Adobe Sensei blog+help;
Meta Animated Drawings; EbSynth; ToonCrafter + line-inbetweening papers; Spline/Figma/Motiff/v0 pages;
MDM (ICLR 2023).
