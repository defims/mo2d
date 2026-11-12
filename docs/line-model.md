---
type: Design Document
title: "Everything is Lines: The Unified Model of Lines, Nodes, and Relations"
description: mo2d core concept document — a line = an ordered sequence of nodes, with three-level attributes (line-level defaults / node-level overrides with interpolation / segment-level direct storage); bones/skeletons/timelines/gradients/layers/meshes/simulation are all projections of lines; four categories of non-line boundary with criteria; plus a Moho concept mapping and absorption table.
tags: [line-model, node, segment, attribute, core-concepts]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: m-bone
    resource: file:///Users/wei/maohou/plan/moho/m-bone.md
    title: "mohoscripting · M_Bone 类归档(maohou 知识库)"
  - id: mohoproj-format
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 字段全表(maohou 知识库)"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
---

**English** | [简体中文](line-model.zh-CN.md)

# Everything is Lines

mo2d's core claim: **all structure and control can be reduced to lines** (Line). Bones, timelines, vector graphics,
bitmap meshes, gradients, layers — concepts that are mutually heterogeneous in traditional software are, in mo2d, all the same thing:
**lines composed of nodes, plus relations between lines** (for relations see [relations.md](relations.md)).

## 0. Normative Statements (owner-given, verbatim)

> **骨骼是 2 个节点的线段,骨架是一条折线。**
> *A bone is a segment of 2 nodes; a skeleton is a polyline.*
>
> **时间轴是多条有多个节点的直线,每个节点是一个关键帧。**
> *A timeline is multiple straight lines with multiple nodes, and each node is a keyframe.*
>
> **渐变是控制棒决定的,控制棒就是线段。**
> *Gradients are determined by control bars, and a control bar is a segment.*
>
> **层和群组本质相同,都是线的集合;嵌套层、父子层本质是线与线的关系映射;层的缩放旋转用方框表示。**
> *Layers and groups are essentially the same — both are sets of lines; nested layers and parent-child layers are essentially relation mappings between lines; a layer's scaling and rotation are represented with a box.*
>
> **模拟过程本质是线和线的关系。**
> *A simulation process is, in essence, a relation between lines.*
>
> **集合本质是线和线的关系,只不过是预定义关系;关系就是函数;关系的参数就是集合的参数。**
> *A set is, in essence, a relation between lines — merely a predefined one; a relation is a function; a relation's parameters are the set's parameters.*
>
> **线有属性(比如笔触),节点也有相应的属性。**
> *Lines have attributes (such as brush), and nodes also have corresponding attributes.*
>
> **一条线,每个节点有属性,每条线段也有属性值。**
> *One line: every node has attributes, and every segment also has attribute values.*
>
> **线和线关系可以最大化地利用 GPU,概念契合,提高性能。**
> *Lines and line relations can maximize GPU utilization; the concepts fit, improving performance.*

## 1. The Line Primitive and Three-Level Attributes

**Line = an ordered sequence of nodes + line-level attributes.**

- **The node (Node) is a first-class citizen**: control points of vector graphics, joints of bones, keyframes of timelines,
  texture anchors of bitmap meshes, color stops of gradients — all unified as "nodes on a line".
- **Node = position + node-level attributes + semantic tag**.
- The **segment (Segment)** between adjacent nodes is also a first-class attribute carrier: **each segment has its own attribute values**;
  a segment can be straight or curved, its curvature determined by the handles of the nodes at its two ends (handles are stored on nodes and act on segments).
- A line can be **open** or **closed**; a closed line encloses a **fill region** (the fill rule belongs to logical payload, see §4);
  **Generalization note**: a 3-node closed line = a triangular face — the natural entry into 3D surface representation ([3d-extension.md](3d-extension.md) §2).

**Three-level attribute model** (owner: "Lines have attributes, e.g. brush; nodes also have corresponding attributes" + "One line: every node has attributes, and every segment also has attribute values"):

| Level | Role | Examples |
|---|---|---|
| Line-level attributes | Default values for the whole line | Brush (brush style), base width, color, opacity |
| Node-level attributes | Per-point overrides of same-named attributes, **interpolated** between segments; fall back to the line-level value when absent | Node width, node color, pressure |
| Segment-level attributes | **Direct state values** for each segment, not interpolated between segments (they are the segment's own values) | Segment type (straight/curved), segment visibility, segment interpolation mode (track), segment fill participation |

- Resolution order: **segment-level → node-level (interpolated between both ends) → line-level**; lower levels fall back upward when absent.
- Attributes are themselves **channels (Channel)**: samplable, animatable over time (see [relations.md](relations.md) § Track).
- Brush = line-level style + node-level pressure/width variation — a line of uniform brush is pressed into a thick-thin rhythm at its nodes.
- Corroboration: Moho's Shape-level Style (fill_color/line_color) and Point-level color/width/color_strength
  channel overrides = precedent for the line/node two levels; `segments_on` (segment visibility) and interpolation settings "one per key, acting on the outgoing segment"
  ([moho-mapping.md](moho-mapping.md) §5.2) = engineering precedent for segment-level attributes.

**GPU fit** (owner: "Lines and line relations can maximize GPU utilization; the concepts fit"):
node-level attributes are naturally SoA (Structure-of-Arrays) arrays — a position array, a width array, a color array —
mapping directly onto GPU vertex attributes/storage buffers and uploading the whole scene at once; see [architecture.md](architecture.md) § GPU strategy.

## 2. The Relation Primitive

**Relation = function**: takes several lines and parameters as input, and outputs derived structure and values. **Set = predefined relation**,
and a relation's parameters are the set's parameters (owner normative statement). Document = the relation graph formed by all lines + all relations.
The full taxonomy of relations is in [relations.md](relations.md); this document keeps only one sentence: layers and groups are organizational predefined relations,
and bind, constraint, propagate, derive, track, and simulate make up the remaining categories.

## 3. The Nine Mappings

Each traditional concept = a projection of lines (or relations). The first four items and items 6/7 are owner normative statements; those marked [derived] are extensions of this document, pending owner confirmation.

| # | Traditional concept | mo2d formulation | Basis |
|---|---|---|---|
| 1 | Vector graphics | **Line**: open line → stroke, closed line → fill region; variable width = node width channel; brush = line-level style + node-level pressure | Normative statement |
| 2 | Bone | **A segment of 2 nodes**; joints are nodes | Normative statement |
| 3 | Skeleton | **A polyline**; bone chains connect end to end; branching (e.g. two arms) = multiple polylines **sharing nodes** | Normative statement |
| 4 | Timeline | **Multiple straight lines containing multiple nodes**; each straight line = a track, **node = keyframe** (position = tick, value = node-level attribute); **interpolation/easing = segment-level attribute** (per-component handles); the track predefined relation = wiring the track line to the attribute being animated for reference/sampling; the (t,v) easing curve = a derived view of the track | Normative statement + resolved ([mvp.md](mvp.md) §2.2 P0-1) |
| 5 | Bitmap mesh | **Two interwoven sets of warp and weft lines**; nodes are texture anchors; [derived] crossing points are nodes shared by the warp and weft lines; deformation = the lines move and the texture follows | Derived |
| 6 | Gradient | **Control bar = segment** (2 nodes); color stops = nodes on the bar (position channel = stop position, color channel = stop color); [derived] nodes are animatable ⇒ gradients are naturally animatable | Normative statement + derived |
| 7 | Layer/group | **A set of lines** (predefined relation); nesting/parent-child = relation mapping; depth/occlusion = ordering parameters on the relation | Normative statement |
| 8 | Layer transform | **Box = a 4-node closed line**; [derived] dragging the box's nodes/edges applies scaling and rotation — the transform is an operation on the box line | Normative statement + derived |
| 9 | Simulation | **Dynamic relations between lines** (springs/forces/collisions/particles); a particle = a short directed segment emitted by an emitter line (position + velocity vector) | Normative statement |

### 3.1 Things That Don't Look Like Lines but Reduce Naturally

| Concept | Reduction |
|---|---|
| Camera | A box line (4-node closed line); dolly in/out, pan, truck = animation of the box nodes |
| Stage boundary | A box line |
| Playhead | A short vertical line on the time baseline |
| Motion path | A line on the stage (objects travel along it) |
| Volume envelope | A line |
| Brush scattering | Stamping along a line at intervals of a spacing channel |
| Lipsync/Switch | Nodes carrying discrete values on a track's straight line |

## 4. The Non-Line Boundary (Four Categories)

Converged through stress testing (two rounds of the owner asking "which further concepts cannot be associated with lines / have nothing to do with lines"): **in structure and in relations, everything is lines;
the following four categories cannot be annexed any further — any further annexation is a stretch** (e.g., calling an MP3 clip a "line" is meaningless).

| # | Category | Contents | Why it cannot be absorbed |
|---|---|---|---|
| 1 | **Sampling fields** | Bitmap pixels, audio samples, video frames | Not geometry — data. Lines only anchor (the mesh), deform (warp/weft lines), drive (envelope lines); image outlines can be **derived** as lines, but that is an output, not the data itself |
| 2 | **Symbols and logic** | Code (expressions/custom relation function bodies/solvers/event scripts), text strings and fonts (glyph outlines can be derived as lines; the semantic source is not lines), naming/annotations/semantic tags (metadata attached to lines) | Not geometry — semantics/rules. Fill rules (nonzero/even-odd) belong here: they are interpretation rules over a set of lines |
| 3 | **Out-of-document processes and state** | Session state (view camera/selection set/undo stack/clipboard/snapping settings), collaboration and versioning, export and publishing configuration | Not document — environment |
| 4 | **External asset references** | Out-of-document entities such as bitmap/audio/font files | Not inside the document — one end of a relation may point to a non-line entity |

Criteria: **non-geometric / non-document / out-of-document endpoint**.
Document-level parameters (frame rate, color space) are not an independent category — they are **parameters of the root relation** (see [relations.md](relations.md)).

## 5. Data Model Skeleton (stack-agnostic pseudo-types)

```ts
Line {
  id: LineId
  nodes: [Node]                 // ordered; a closed line's ends join
  segAttrs: [SegmentAttrs]      // segment-level attributes — parallel one-to-one with the "segments between nodes" (n-1 of them; n for a closed line)
  attrs: LineAttrs              // line-level attributes (brush/base width/color/opacity…)
  tags: [SemanticTag]           // semantic tags (e.g. "bone"/"skeleton"/"track"/"mesh-warp")
}

Node {
  id: NodeId
  position: Vec3 | Tick         // geometric-line node = Vec3 (2D always z=0, per the 3D extension decision); track-line node = Tick (1D time)
  attrs: NodeAttrs              // node-level attributes — same-named ones override the line level, are interpolated between segments, and fall back to line level when absent
  handleIn/handleOut: Offset?   // handles are stored on the node and act on the adjacent segments (out/in)
}

SegmentAttrs {                  // the segment's direct state values, not interpolated between segments
  type: straight | curve        // segment type (straight/curved; the curvature is determined by the handles of the two end nodes)
  visible: bool                 // segment visibility (source: Moho segments_on)
  fillRole?: fillFlag           // how the segment participates in the fill boundary (source: Moho Shape edges flag)
  interp?: InterpSpec           // segments of track lines: interpolation/easing mode (source: Moho InterpSetting outgoing-segment semantics)
}

Relation {                       // relation = function
  id: RelationId
  kind: "predefined" | "custom"
  type: RelationType             // layer/group/bind/constraint/propagate/derive/track/simulate/…
  inputs: [LineRef | NodeRef | RelationRef]
  params: Params                 // the relation's parameters = the set's parameters; parameters are channels, animatable
  body?: LogicPayload            // function body of a custom relation (logical payload)
}

Document {                       // document = all lines + all relations
  lines: [Line]
  relations: [Relation]          // includes the root relation (document-level parameters hang on it)
}
```

## 6. Moho Concept Mapping and Absorption Table

Moho is the most conceptually complete reference frame in the bone-animation domain. **The complete per-concept mapping (31 top-level keys / 10 layer types / Bone's 61 keys /
mesh's three layers / styles' 34 keys / 135 channel-code entries, including the complete mo2d-side vocabulary of attributes and relation parameters) is in
[moho-mapping.md](moho-mapping.md)** — this section keeps only the skeleton-level comparison.
Facts in the table below are taken from first-hand archives of the maohou knowledge base (Moho Pro 14.4 / mohoscripting.com)[^mohoproj-format][^m-bone], with the mappings recorded as absorption/simplification decisions:

| Moho concept | Moho facts | mo2d mapping | Decision |
|---|---|---|---|
| Curve | 5 channels per point (smoothness/offset_in/out/weight_in/out) + trim start/end percentages (animatable) + variable-width profile profile_curve_id[^mohoproj-format] | Line + node handles/width channels; trim = start/end channels on a track | Absorb (nearly isomorphic) |
| Point | position/width/color/color_strength channels (all animatable) + parent bone index[^mohoproj-format] | Node + node-level attributes + the input end of a bind relation | Absorb (two-level attributes isomorphic) |
| Bone | fLength/fAngle/fPos (static) + AnimChannel: angle/pos/scale; angle constraints fConstraints/fMin/fMax; dynamics fBoneDynamics + spring/damping/force families; smart bone Action weights[^m-bone] | A 2-node segment; angle/position/scale = tracks; angle constraints → constraint relations; dynamics → dynamic relations; smart bones → propagate relations | Absorb (semantics), simplify (length = distance between the two points, not an independent field) |
| mesh (vector mesh) | Three layers: points/curves/shapes; a Shape's edges reference curve segments, has_fill/has_outline decide the same curve's line/face role[^mohoproj-format] | A set of lines; closed lines enclose fills; stroke/fill are rendering interpretations of a line | Absorb + simplify (drop the three-layer containers; lines + relations directly) |
| 10 layer types (Vector/Image/Audio/Group/Switch/Bone/Particle/Note/Text/Patch) | Each carries its own key family[^mohoproj-format] | Type templates of predefined relations (a layer's parameter table) | Reference (which ones enter the standard relation library and which degrade into semantic tags — decided during engineering) |
| fileref{path,relativeTo} | External asset reference mechanism[^mohoproj-format] | A reference-type relation (the endpoint is an out-of-document entity) | Absorb |
| Smart bones (Action) | Dial bone angle drives multi-channel lookup tables[^m-bone] | A propagate relation (one control line drives multiple target lines) | Absorb and generalize |

**Engineering red line**: this table is a concept comparison only; mo2d is a clean-room independent implementation and promises no .moho compatibility
(the importer is listed only as a long-term option, see [roadmap.md](roadmap.md)).

## 7. Open Questions

- [ ] Segment-level attributes are now normative; the value semantics of track-segment "interpolation/easing = segment attribute" (the 12 im modes, h/in,
      the relation between Bezier handles and segment geometry) awaits owner confirmation along [moho-mapping.md](moho-mapping.md) §5.2;
- [ ] [derived] bitmap-mesh crossing points = nodes shared by warp and weft lines — whether to adopt;
- [ ] [derived] transform = dragging the box line; interaction details decided during engineering;
- [ ] The initial vocabulary of semantic tags (bone/skeleton/track/mesh-warp/…) to be decided during engineering.
