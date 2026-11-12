---
type: Reference
title: "Glossary (English–Chinese)"
description: One-sentence definitions of every mo2d term — line/node/segment/three-level attributes (line-level/node-level/segment-level)/brush stroke/channel/relation/function/predefined & custom relations/nine kinds of relations/track/keyframe/control bar/transform box/four kinds of non-line boundaries/GPU-related/command & session state.
tags: [glossary, terminology]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T10:55:00Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
---

**English** | [简体中文](glossary.zh-CN.md)

# Glossary (English–Chinese)

## Structural primitives

| Term | 中文 | Definition |
|---|---|---|
| Line | 线 | An ordered sequence of nodes + line-level attributes; the only structural primitive of mo2d |
| Node | 节点 | A point on a line: position + node-level attributes + semantic tag; the unified abstraction of joints/keyframes/anchors/color stops |
| Segment | 段 | The portion between adjacent nodes, straight or curved; holds its own segment-level attributes |
| Line Attribute | 线级属性 | Default attributes of the whole line (brush/base width/color/opacity) |
| Node Attribute | 节点级属性 | A local override with the same name as its line-level counterpart; interpolated across segments; falls back to line level when absent |
| Segment Attribute | 段级属性 | The direct state value held by each segment (segment type/visibility/interpolation/fill participation); not interpolated across segments |
| Brush Stroke | 笔触 | Line-level style + node-level pressure/width variation |
| Pressure | 压力 | Node-level width channel (stylus input): width = line width × pressure; an explicit node width wins (landed in B2) |
| Style Template | 样式模板 | A named line-attribute template; `style.apply` copies its defined fields onto a line — switching brush strokes = applying another template (B2) |
| Fallback | 回落 | The rule that a missing node attribute takes the line-level default value |
| Channel | 通道 | A sequence of scalar parameters that can be sampled and animated |
| Semantic Tag | 语义标签 | A role annotation attached to a line (bone/skeleton/track/mesh-warp…) |

## Relation primitives

| Term | 中文 | Definition |
|---|---|---|
| Relation | 关系 | A directed function between lines (nodes included): input lines and parameters → output derived structure and values |
| Function | 函数 | The essence of a relation (owner: a relation is just a function) |
| Predefined Relation | 预定义关系 | A relation from the standard function library (layer/group/bind/constraint/…); a set is itself a predefined relation |
| Standard Relation Library | 标准关系库 | The full set of predefined relations |
| Custom Relation | 自定义关系 | A function defined by the user/AI, with a logic payload as its body |
| Root Relation | 根关系 | The document-level root; document parameters such as frame rate and color space hang on it |

## Predefined relations (nine kinds)

| Term | 中文 | Definition |
|---|---|---|
| Layer | 层 | Organizational relation: a set of lines → an ordered renderable unit (parameters = visibility/opacity/blend mode/transform) |
| Group | 群组 | Organizational relation: a set of lines → a named package; essentially the same as a layer |
| Ownership | 归属/层级映射 | Parent-child nesting = relation mapping; depth/occlusion = its ordering parameters |
| Node Sharing | 节点共享 | Multiple lines share the same node (skeleton polylines, warp/weft mesh intersections) |
| Bind | 绑定 | Nodes move with a bone line/mesh line/control bar (skinning/texture anchoring/gradient mounting) |
| Constraint | 约束 | Geometric restrictions such as angle/length/tangency (a static relation) |
| Propagate | 传导 | One control line drives multiple target lines (a generalization of smart bones) |
| Derive | 派生 | The output line is computed from input lines (medial axis/envelope/loft/text→outline) |
| Track | 轨道 | Temporal relation: a straight line + keyframe nodes, carrying the time series of one channel |
| Keyframe | 关键帧 | A node on the track line: time position + animation value |
| Simulate | 模拟 | Dynamic relation: springs/damping/gravity/wind/collisions/particles, integrated frame by frame by a solver |
| Spring | 弹簧 | A dynamic constraint acting between two lines |
| Particle | 粒子 | A directed short line segment emitted by an emitter line (position + velocity vector) |
| Solver | 求解器 | The program that integrates dynamic relations (a logic payload) |
| External Asset Reference | 外部资产引用 | Reference relation: an endpoint that is an entity outside the document (a file:// path) |

## Domain projections (line-model mapping)

| Term | 中文 | Definition |
|---|---|---|
| Bone | 骨骼 | A line segment of 2 nodes (owner normative statement) |
| Skeleton | 骨架 | One polyline; branching = multiple polylines sharing nodes |
| Timeline | 时间轴 | Multiple straight lines containing multiple nodes (= a family of tracks) |
| Bitmap Mesh | 位图网格 | Two interwoven sets of warp/weft lines; the nodes are the texture anchors |
| Gradient | 渐变 | A control bar (2-node line segment) + color-stop nodes |
| Control Bar | 控制棒 | The line segment that defines a gradient's direction and extent |
| Color Stop | 色标 | A node on the control bar: position channel = stop position, color channel = stop color |
| Transform Box | 变换方框 | The operation box for layer scaling/rotation = a 4-node closed line |
| Fill Region | 填充域 | The region enclosed by a closed line; interpretation rules (non-zero/even-odd) belong to the logic payload |
| Camera | 摄像机 | A box line; push/pull/pan/truck = its node animation |

## Non-line boundaries (four kinds)

| Term | 中文 | Definition |
|---|---|---|
| Payload | 载荷 | Non-geometric content attached to structure (media/logic) |
| Media Field | 媒体场 | The sampled field data itself (bitmap pixels/audio samples/video frames); lines anchor, deform, and drive it |
| Logic | 逻辑 | Symbols and rules such as expressions/scripts/function bodies/solvers/fill rules |
| Session State | 会话状态 | Editor state outside the document (view/selection/undo stack/clipboard) |

## Evaluation & rendering

| Term | 中文 | Definition |
|---|---|---|
| Relation Graph | 关系图 | The directed acyclic graph formed by all relations |
| Evaluation Graph | 求值图 | The executable form of the relation graph after topological layering; parallelizable within each layer |
| CPU Oracle Renderer/Evaluator | CPU oracle | The deterministic ground-truth source; GPU implementations must match its output |
| Structure-of-Arrays | SoA 布局 | A memory layout with one array per attribute kind; GPU-friendly |
| Damage Rendering | 损伤渲染 | Recomposite only the 64×32 tiles touched by changed lines (dirty rectangles); byte-identical to a full re-render (landed in B1) |
| Reverse-Dependency Invalidation | 反向依赖失效 | An edit dirties exactly its transitive downstream through the relation DAG — channels are signals (landed in B1) |
| Gradient Relation | 渐变关系 | Recolors a target line by projecting its nodes onto the control bar and sampling the color stops (fill or stroke role); stops are animatable via color tracks (landed in B3) |
| Group Relation (Transform Box) | 群组关系（变换框） | The box line's rest→current similarity (anchor + diagonal) scales/rotates its member lines; `visible` = the layer visibility parameter; nesting composes (landed in B4) |
| Sim Relation | 模拟关系 | A Verlet chain with distance constraints, integrated in fixed 240 Hz substeps; anchored nodes ride their driving tracks (landed in B6) |
| Replay-from-Seed | 种子重放 | The dynamic-relation determinism strategy: every evaluation re-integrates from tick 0, so scrub-back/replay/offline export match by construction (landed in B6) |
| Explicit Seed | 显式种子 | Particle scatter angles hash through splitmix64 on a seed parameter carried in the document — same doc + same seed → same output (B6) |
| Texture Anchor | 纹理锚点 | A node's `uv` (0..1) within its line's texture; the textured fill interpolates anchors across the surface (landed in B5) |
| Decode-on-Demand | 按需解码 | Textures decode the first time a render asks, through a budgeted cache (256 MiB default); cache state never affects output pixels (B5) |
| Package (.mo2d) | 包(.mo2d) | The single-file deliverable: canonical JSON header + a `--BLOBS--` marker + sorted, length-delimited blob records (landed in B7) |
| Blob | 二进制区条目 | One named byte region in a package's tail area (e.g. an embedded texture); references resolve against blobs first, then the filesystem (B7) |
| Project Directory | 工程目录 | The unpacked container form: `document.mo2d.json` + `assets/`; equivalent to the package — pack ∘ unpack ∘ pack is byte-identical (B7) |
| Migration Chain | 迁移链 | The reader's per-version upgrade table: bumping FORMAT_VERSION pushes exactly one migration; newer documents are refused, never guessed (B7) |
| Collision Clearance | 碰撞间隙 | A sim/particles parameter: points keep a radius from collider segments (sim chains may also self-separate); restitution `bounce` reflects particle velocity (landed in B6) |

## Interaction & protocol

| Term | 中文 | Definition |
|---|---|---|
| Stage | 舞台 | The world of lines; the only permanent interface |
| Command | 命令 | The operation unit at the engine's single entry point (JSON); every operation is a command |
| Command Stream | 命令流 | The time-ordered record of commands; replayability = undo/collaboration/AI context |
| Controls-as-Lines | 控件即线 | Ephemeral controls (sliders/gradient bars/boxes) generated by the AI as lines, discarded after use |
| Model Context Protocol | MCP | The server-mode protocol by which an AI host connects to the engine |
