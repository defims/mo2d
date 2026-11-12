---
type: Design Document
title: "Moho Concept-by-Concept Mapping: Complete Table of Point/Line Attributes and Relations"
description: Every concept of Moho Pro 14.4 understood one by one and classified into mo2d's lines/nodes/three attribute levels (line-level/node-level/segment-level)/nine relation classes/channels/payloads — covering 31 top-level keys, 25 project_data keys, 10 layer types and 49 common keys, the mesh three-layer structure, 61 Bone keys, 34 style keys, and 135 channel codes, producing a complete mo2d-side list of attribute values and relation parameters.
tags: [moho, mapping, attributes, segment, relations, reference-table]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: fmt
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 文件格式规范(31 顶层键/层型键数/mesh/Bone/Style 全表)"
  - id: chan
    resource: file:///Users/wei/maohou/plan/moho/channel-codes.md
    title: "通道码表 14.4(135 项 10000-10137)+ 13.5.3"
  - id: interp
    resource: file:///Users/wei/maohou/plan/moho/interp-modes.md
    title: "InterpSetting 与 12 插值常量"
  - id: mbone
    resource: file:///Users/wei/maohou/plan/moho/m-bone.md
    title: "M_Bone 类归档(AnimChannel 全表)"
  - id: mskel
    resource: file:///Users/wei/maohou/plan/moho/m-skeleton.md
    title: "M_Skeleton 类归档(平表寻址/骨骼组/IK)"
  - id: mlayer
    resource: file:///Users/wei/maohou/plan/moho/moholayer.md
    title: "MohoLayer 类归档(层父子/绑骨/跟随/引用族)"
  - id: achan
    resource: file:///Users/wei/maohou/plan/moho/animchannel.md
    title: "AnimChannel 类归档(六通道类型/键寻址/Action)"
  - id: parity
    resource: file:///Users/wei/maohou/plan/moho/anicraft-moho-parity-plan.md
    title: "能力域清单与 Image/Audio/Particle 处置参照"
---

**English** | [简体中文](moho-mapping.zh-CN.md)

# Moho Concept-by-Concept Mapping: Complete Table of Point/Line Attributes and Relations

This document takes **every single concept** of Moho Pro 14.4, understands it, and classifies it into a mo2d
model slot, then compiles a **complete list of attribute values and relation parameters** on the mo2d side.
All facts come from the empirical archives of the maohou knowledge base
(8 corpus fixtures saved natively by Moho 14.4, high confidence[^fmt]);
the mapping judgments are mo2d design content (annotations: absorb = model directly / simplify = swap to a simpler form / degrade = reduce to a tag or parameter / boundary = stays inside the non-line boundary).

## 0. Slot decision rules

For each Moho key/concept, ask the six questions in order and land it in one of the ten slots:

1. Is it the **geometric path itself**? → line; points on the path → nodes; the portions between nodes → segments;
2. Is it direct state **held per segment** (not interpolated between segments)? → segment-level attribute;
3. Is it a quantity that **varies point by point**? → node-level attribute (interpolated between segments); otherwise uniform along the whole line → line-level attribute;
4. Does it describe behavior **between two objects**? → relation (one of the nine predefined classes) + relation parameters;
5. Does it vary over time? → any attribute can get a track (channel);
6. None of the above → root parameters / payloads (media/logic) / session state / envelope metadata.

| Slot | mo2d definition |
|---|---|
| Line | ordered node sequence + line-level attributes |
| Node | a point on a line: position + node-level attributes |
| Line-level attribute | whole-line defaults (brush/width/color/switches...) |
| Node-level attribute | same-name local override, interpolated between segments, falls back to line-level |
| Segment-level attribute | the direct state value of each segment, not interpolated between segments (owner: "每条线段也有属性值" — "every segment also has attribute values") |
| Relation + parameters | the nine predefined relation classes ([relations.md](relations.md) §1) |
| Track | a time line + keyframe nodes (value/interpolation/action) |
| Root parameters | parameters of the root relation (document-level) |
| Payloads | media fields / logic |
| Session state | editor state outside the document |
| Envelope | file-format metadata |

## 1. Master table: concept → slot

| Moho concept | mo2d slot | Disposition |
|---|---|---|
| Curve | **line** | absorb |
| Point | **node** | absorb |
| Segment state (segments_on / edges flag / interpolation out-segment) | **segment-level attribute** | absorb (Moho stores it scattered across points/keys; mo2d promotes it to the segment) |
| Shape | line + rendering interpretation (stroke/fill switches) | simplify |
| Style (34 keys) | **line-level attribute template** (library) | absorb |
| Bone | **line** (2-node segment, semantic tag bone) | absorb (length = distance between the two points, not a separate field) |
| Skeleton | **polyline / multiple polylines + node sharing** | absorb |
| Bone groups (bones_groups) | named subset of a relation (ownership) | degrade |
| Binding (Point.parent/parent_bone/strength) | **bind relation** + weight parameters | absorb |
| Angle constraints (constraints/min/max/fixed_angle) | **constraint relation** | absorb |
| Control bones (angle/pos/scale_control_*) | **propagate relation** (with delay/scale parameters) | absorb |
| Smart-bone Action (actions/pose) | **propagate relation** (named lookup table) | absorb and generalize |
| IK (ik_lock/target_bone/ik_*) | constraint relation (endpoint target) + node parameters | absorb |
| Bone dynamics (bone_dynamics + force families) | **simulate relation** | absorb |
| Layer transform (translation/scale/rot/flip/shear, 10 channels) | parameters of the layer relation (channels) + **transform-box line** | absorb |
| Layer effects (6 groups, 40 sub-channels) | parameters of the layer relation (channels) | absorb |
| Layer physics (physics, 14 keys) | simulate-relation parameters | absorb |
| Follow curve (follow_curve/following) | **derive relation** (layer along a line) | absorb |
| Distortion reference layer (distortion/warp) | **derive relation** (mesh deformation) | absorb |
| Layer references (layer_ref_* family) | **external-reference relation** | absorb |
| Layer nesting (Group/Bone/Switch containers) | **ownership relation** (organizational class) | absorb |
| SwitchLayer (switch_keys) | track (String / discrete-value nodes) | absorb |
| Camera (camera_track/zoom/roll/pan_tilt) | **transform-box line** (node channels) | absorb |
| Timeline markers (timeline_markers) | annotation nodes on a track | absorb |
| Channel objects (all ⟨ch:T⟩) | **track lines** | absorb |
| Interpolation (interp field family) | **segment attribute** (between keyframe nodes) | absorb |
| Style color keyframes | line-level attribute channels | absorb |
| TextLayer | payload (text) + **derive relation** (→ outline lines) | simplify |
| ImageLayer | payload (bitmap) + bind (anchoring to mesh lines) | absorb |
| AudioLayer/soundtrack | payload (audio) + tracks (level/jump) | boundary (media field) |
| Particle layer (ParticleLayer) | simulate relation (emitter lines) | boundary, pending verification (zero instances in corpus) |
| NoteLayer/PatchLayer | semantic tags + payloads | boundary, pending verification (zero instances in corpus) |
| project_data project settings | **root parameters** | absorb |
| documentviewstate/onion skin | **session state** | boundary (outside the document) |
| _SEL companion channels | session state (selection) | boundary |
| mime/version/uuid/dates | envelope | absorb |
| layercomps (layer comps) | named views of the ownership relation | degrade |
| tracking point / poser | node semantic tags + relations | undecided (no sample files) |

## 2. Vector mesh: complete attribute values for lines and nodes

Moho's vector layer = the `mesh` substructure (three layers: points/curves/shapes)[^fmt]. mo2d drops the
three-layer container and lands directly on lines/nodes — a **factual precedent for two-level attributes**.

### 2.1 Node-level attributes (source: Point's 9 keys[^fmt] + POINT-family channel codes[^chan])

| Moho key | Channel code | mo2d node attribute | Value range / notes |
|---|---|---|---|
| `position` ⟨ch:Vec2⟩ | 10030 | **position channel** | Vec2; animatable via a track |
| `width` ⟨ch:Val⟩ | 10038 (line width) / 10054 (THICKNESS) | **width channel** | f64; line-level line_width is the fallback |
| `color` ⟨ch:Color⟩ | 10056 POINTCOL | **color channel** | {r,g,b,a}; interpolating along the line = gradient |
| `color_strength` ⟨ch:Val⟩ | 10058 POINTSTRENGTH | **color_strength channel** | 0..1; effective when colored=true |
| `colored` bool | — | color switch (node-level) | default false = fall back to line-level color |
| `smoothness` ⟨ch:Val⟩ | 10032 CURVE (per point) | **handle type** (straight/curved) | 0 = straight segment, >0 = smoothness |
| `offset_in`/`offset_out` ⟨ch:Val⟩ | 10032 | **handle offset** | controls segment curvature (Bézier handles) |
| `weight_in`/`weight_out` ⟨ch:Val⟩ | 10032 | **handle weight** | handle influence |
| `segments_on` bool | — | segment visibility switch | the segment between this point and the next |
| `parent` int (bound bone) | — | **bind-relation endpoint** (not an attribute) | -2 = none; see §4.1 |
| `selected` bool | 10031 POINT_SEL | session state | not part of the document model |
| `curves` (reference) | — | structural reverse index | engine-internal |

**Conclusion: the first-version node-level attribute vocabulary = {position, width, color, color_strength, colored, handle{type, offsetIn, offsetOut, weightIn, weightOut}}**;
of these, position/width/color/color_strength and the handle family can all carry tracks.
(`segments_on` originally lives on the point, with semantics pointing to "the segment between this point and the next" — per the owner's normative statement it is **classified as a segment-level attribute**, see §2.2.)

### 2.2 Segment-level attributes (owner: "每条线段也有属性值" — "every segment also has attribute values")

Moho stores segment state scattered across points/keys; mo2d promotes it to **attributes of the segment itself**
(not interpolated between segments — it is the segment's own value):

| Moho fact (storage location) | mo2d segment-level attribute | Value range / notes |
|---|---|---|
| `segments_on` (on Point) | **visible** | visibility switch for the segment |
| `smoothness`=0 marks a straight segment (per point on Curve) | **type** | straight / curve; segment bending is determined by the handles of the two end nodes (handles still stored on nodes, acting on the segment) |
| Shape `edges {curve, segment, flag}` | **fillRole** | how the segment participates in the fill boundary (direction/role) |
| InterpSetting out-segment semantics (stored on key i, acting on segment i→i+1, see §5.2) | **interp** (track line) | im (12 modes) / h (hold frames) / in (on-N's) / s (stagger) / v1 / v2 / b handles |

### 2.3 Line-level attributes (source: Curve's 9 keys[^fmt] + Shape's 16 keys[^fmt] + Style's 34 keys[^fmt])

| Moho key | mo2d line-level attribute | Value range / notes |
|---|---|---|
| `closed` bool | **closed** | a closed line encloses the fill region |
| `start_percent`/`end_percent` ⟨ch:Val⟩ | **trimStart/trimEnd channels** | trim start/end (corresponding families CURVEEXP 10046 / PROFILEOFFSET 10048[^chan]) |
| `profile_curve_id`/`profile_layer_uuid`/`profile_repeat` | **widthProfile derive relation** (profile line → width) | variable-width profile: another line serves as this line's width source |
| `has_fill`/`has_outline` (Shape) | **fillOn/strokeOn** | two rendering interpretations of the same line |
| `fill_color`/`line_color` ⟨ch:Color⟩ (Style) | **fillColor/strokeColor channels** | style-level can also be animated (style = attribute template) |
| `line_width` (Style) | **width (line-level fallback)** | default 0.022 |
| `line_caps` (Style) | **cap** | line cap |
| Style `define_*` three switches | enable bits of the attribute template | the template only overrides checked attributes |
| `effect_offset`/`effect_scale`/`effect_rotation` (Shape ⟨ch⟩) | **offset/scale/rotation channels** (line-level transform) | FXXFORM 10052 family[^chan] |
| `3d_thickness` ⟨ch:Val⟩ | thickness3d | 3D thickness |
| `shape_order` ⟨ch:String⟩ / SHAPE_ORDER 10060 | **zOrder channel** | depth/occlusion = ordering parameter (channelized) |
| `name`/`id` | semantic tag + id | — |
| `groups` (mesh.groups) | ownership relation (named subset) | — |

### 2.4 Brush (line-level; source: Style's brush_* 13 keys[^fmt])

**brush attribute group (entirely line-level, usable as an attribute template)**:
`brush_name` (brush file → external reference), `align`, `jitter`, `spacing` (default 0.38),
`angle_drift`, `randomize`, `merged_alpha`, `tint`, `rand_order`, `size_amp` (0.14),
`size_scale` (0.03), `random_interval` (1), `hue_drift` (-0.02), `sat_drift` (0.33), `val_drift` (0.12).
Additionally the COLOR_DRIFT 10042 / OPACITY 10040 / BLEND 10044 channel families[^chan] → line-level colorDrift/opacity/blend channels.

## 3. Layers: organizational relations and their complete parameters

A Moho layer = 49 common keys + layer-type-specific keys[^fmt]. In mo2d: a layer is a **predefined relation**
(owner's normative statement), and every key below becomes a **parameter of the layer relation**
(those marked ⟨ch⟩ can carry tracks).

### 3.1 Layer-relation parameters (source: the 49 common keys[^fmt])

| Parameter group | Keys → mo2d parameter names |
|---|---|
| Identity/display (9) | name, label_col→labelColor, visible, shownInTimeline, expanded (session preference), origin, blendMode; uuid→envelope |
| Render switches (7) | renderOnly, editOnly, cameraImmune, dofImmune, consolidatedChannels, qualityFlags, masking |
| Transform (10 channels) | translation⟨Vec3⟩, scale⟨Vec3⟩ (ratio-based), rotationX/Y/Z⟨Val⟩ (radians), flipH/flipV⟨Bool⟩, shear⟨Vec3⟩, following⟨Val⟩, physicsNudge⟨Vec2⟩ — **the operating surface of the layer transform = the transform-box line** (a 4-node closed line) |
| Effect — opacity/visibility | alpha⟨Val⟩ (10014), visibility⟨Bool⟩ (10012) |
| Effect — blur (10013) | blur⟨Val⟩ |
| Effect — AO (10015) | ambientOcclusion⟨Val⟩ |
| Effect — noise (10017) | noise⟨Val⟩ |
| Effect — pixelation (10018) | pixelation⟨Val⟩ |
| Effect — threshold (10016) | threshold⟨Val⟩ |
| Effect — outline (10023) | outline{color⟨Color⟩, on⟨Bool⟩, width⟨Val⟩} |
| Effect — shading (10020) | shading{angle,blur,color,contraction,noiseAmp,noiseScale,offset,on,threshold} |
| Effect — shadow (10019) | shadow{angle,blur,clipToGroup,color,expansion,noiseAmp,noiseScale,offset,on,threshold} |
| Effect — perspective shadow (10022) | perspectiveShadow{blur,color,on,scale,shear,threshold} |
| Effect — motion blur (10021) | motionBlur{alphaEnd,alphaStart,extendedFrames,framePercentage,frames,on,radius,skip,subFrames} |
| Layer color / annotation | layerColor{color⟨Color⟩,on⟨Bool⟩} (10024), userComments, userTags, timelineMarkers⟨String⟩ (10025), layerOrdering⟨String⟩ (10124 LAYER_ORDER) |
| Timing / misc | timingOffset, randomNum, rotateToFollow, scaleCompensation, scaleNormalization, faceCamera, faceCameraMode, animatedLayerEffects |
| Masks | maskExpansion, groupMask |

### 3.2 Each layer type → mo2d relation / line form

| Layer type (key count) | mo2d destination | Key points |
|---|---|---|
| MeshLayer (70) | line collection (semantic tag vector) | all of §2 |
| GroupLayer (61) | **ownership relation** (group type) | parameters: enablePhysics⟨Bool⟩ (10125), gravity{x,y}, wind{x,y}, useBakedPhysics; child layers = nested layers |
| BoneLayer (59) | **skeleton polyline family** (semantic tag skeleton) | skeleton parameters: bindingMode (int=1), bonesGroups (bone groups), wind/gravity (14.x, wind/gravity fields the layer provides to bones), grandpaBone; in-bone keys in §4 |
| SwitchLayer (57) | ownership relation (switch type) + **String track** | switchKeys⟨ch:String⟩ = current child-layer name (10123; nodes carry discrete values); switchInterpolation (false = step), frameByFrame, previewAlignment; SwitchLayer also has a simplified skeleton |
| TextLayer (74) | payload (text) + **derive relation** (text → outline lines) | text/textsize/font/justification/kerning/leading/fill/stroke/fillcolor/linecolor/linewidth + the bubble balloon* 12 keys; **the embedded full mesh_layer (70 keys) = the landing point of the derived output** — when the text source changes, the outline lines are re-derived |
| ImageLayer | payload (bitmap) + bind (anchoring to mesh lines) | image_fileref{path,relativeTo} → external reference; deformation = distortion reference layer (§3.4) |
| AudioLayer | payload (audio) + tracks | audioLevel (10129)/audioJump (10130); media field = non-line boundary |
| ParticleLayer | **simulate relation** (emitter lines) | 10126; zero instances in the corpus, parameter table deferred until sample files[^parity] |
| NoteLayer | payload (text note) + anchoring | zero instances in the corpus[^parity] |
| PatchLayer | derive relation (patch reshaping) | depends on image deformation; zero instances in the corpus[^parity] |

**Inter-layer relations (MohoLayer API facts[^mlayer])**: layer parent-child = container nesting (Parent() returns
the Group type; transform inheritance via GetFullTransform) → mo2d ownership relation + propagate (parent-layer
transform → child layers); **layer bound to bone** parent_bone key (a layer can be bound to a bone line) → bind
relation; **layer follow curve** follow_curve/follow_layer_uuid/follow_bending (10008 FOLLOW) → derive relation;
**flexi-bone follow** flexi_bone_elbow/flexi_bone_subset → a derive + bind hybrid.

### 3.3 Smart-bone Action (propagate relation)

Storage form[^fmt][^achan]: inside the channel object, `actions:[{name, pose:{when:[frames], val:[values]}}]`;
pose.when holds **integer frame numbers** (= a response table keyed by the triggering bone-dial angle); the layer
object separately holds an `actions:[{name,pose:int}]` reference table.
mo2d = **propagate relation**: parameters {name, source line (the dial), lookup table (a track), feathering
outside the domain}; Action weights angle_weight/pos_weight/scale_weight[^fmt] → mixing-weight parameters of
the propagate relation.

### 3.4 References and deformation

| Moho | mo2d relation |
|---|---|
| `layer_ref_uuid/same_doc/mod_date/path/fileref` | **external-reference relation** (cross-document layer reference; links can break / be updated[^mlayer]) |
| `distortion_layer_uuid` (warp, 14.0) | **derive relation**: the target line collection is deformed by the reference mesh lines |
| `frame_zero_deformer`/`squashable_deformer`/`triangulated` | mode switches of the derive relation |
| layercomps[{name,uuid,layer_ids}] | named view of the ownership relation (layer comps) |

## 4. Bones: segments, polylines, and complete relation parameters

### 4.1 Structure

- **Bone = a 2-node segment** (owner's normative statement); `length` (default 0.021, not pixels) + `offset` + `anim_pos`/`anim_angle` determine the two endpoints — mo2d stores the two end nodes directly; length is derived;
- **Skeleton = a polyline**: `parent` (int index, -1 = root) + end-to-end connection; branching = multiple polylines **sharing nodes**;
  Moho fact: bones form a flat table addressed by integer ids, hierarchy queried via parent[^mskel] — isomorphic to "polylines sharing nodes";
- `anim_parent` ⟨ch:Val⟩ (parent bone channelized, 11.0) → **the ownership relation is animatable** (re-parenting);
- bone groups `bones_groups`[^fmt] / the AddGroup API[^mskel] → named subsets of the ownership relation (smart bones are often referenced by group).

### 4.2 Bone channels (tracks) and static parameters (full destination of Bone's 61 keys[^fmt][^mbone])

| Moho key | mo2d destination | Notes |
|---|---|---|
| `anim_angle`⟨Val⟩/`anim_pos`⟨Vec2⟩/`anim_scale`⟨Val ratio-based, uniform scaling⟩ | **node/line channels** (tracks) | angles in radians; BONE_T/S slots 10074-10108[^chan] |
| `flip_h`/`flip_v`⟨Bool⟩ | line channel | 10100-10103 |
| `ik_lock`⟨Bool⟩/`ik_global_angle`/`ik_parent_target` | **constraint-relation parameters** (IK, animatable) | Lock is per-frame[^mskel] — supporting evidence that lock is animatable |
| `target_bone`⟨Val val=-1⟩ | constraint relation (endpoint target bone) | 10106/10107 |
| `constraints`/`min_constraint` (-1.222 rad)/`max_constraint` (1.5708 rad)/`fixed_angle` | **constraint-relation parameters** (angle constraints) | static |
| `strength` (default 1.0) | **bind-relation parameter** (influence weight) | Moho has no per-point weight editing; strength = binding-radius semantics |
| `scaling_mode` (0 manual / 1 2D / 2 squash & stretch)/`max_auto_scaling`/`squash_stretch_scaling` | bind-relation parameters (scale-propagation mode) | static |
| `bone_dynamics`⟨Bool⟩ (10110) | **simulate-relation switch** | animatable |
| `damping_force` (1.0)/`spring_force` (2.0)/`torque_force` (2.0) | simulate-relation parameters (angular) | static |
| `angle_dynamics`/`pos_dynamics`/`scale_dynamics` (bool) + their own damping/spring/torque | simulate parameters (per-channel families) | static |
| `wind_dynamics` + WIND_DIR/STRENGTH/TURBULENCE (10118-10120) + GRAVITY_DIR/STRENGTH (10121-10122, 14.x)[^chan] | simulate parameters (fields) + layer-level wind/gravity fields | partially channelized |
| `physics_radius` (-1.0)/`physics_torque`/`physics_lock_tip`/`physics_return_to_zero` | simulate parameters (collision / return-to-zero) | static |
| `physics_motor_speed`⟨Val⟩ (10112) | simulate parameter (motor) | animatable |
| `angle/pos/scale_control_parent` (-1)/`_scale`/`_delay` | **propagate-relation parameters** (control bones: parent/scale/delay) | static; delay = propagation lag |
| `angle_weight`/`pos_weight`/`scale_weight` (14.0) | propagation mixing weights | static |
| `bone_enable_arc_solver` | constraint-solver switch (arc IK) | static |
| `hidden`/`shy`/`selected`/`bone_label_showing` | session state / display switch | shy/selected = session |
| `bone_tags` (int32 color tags) | semantic tags (color) | — |
| `offset`{x,y} | node attribute (draw offset) | — |
| `ignored_by_ik` | constraint parameter (IK bypass) | static |
| `grandpa_bone` (layer) | ownership parameter | — |

### 4.3 Bone-dynamics supplement (layer physics, 14 keys[^fmt])

`physics.enabled/static/density/friction/restitution/respawn/sleeping/pivot/velocity/force_field/force_field_vector⟨Vec2⟩ (10011)/enable_motor/motor_speed⟨Val⟩ (10010)/motor_torque`
→ simulate-relation parameters (rigid-body family); `physics_nudge`⟨Vec2⟩ (10009) → a layer-transform channel.

## 5. The channel system: track lines, keyframe nodes, segment attributes

### 5.1 Tracks (lines) and keyframes (nodes)

- Six channel types: Val (f64) / Vec2 / Vec3 / Bool / Color / String[^achan] → track value types;
- channel object {type,ref,mute,when[],val[],interp[],actions[]}[^fmt]:
  - `mute` → track mute parameter;
  - `when[]` (integer frames) → **the node's position in time**;
  - `val[]` → **node value**;
  - `actions[]` → the §3.3 propagate lookup table;
- nearly every channel has a `_SEL` companion channel[^chan] → **session state** (selection does not enter the document);
- `expr` (a maop extension) → logic payloads attached to tracks.

### 5.2 Segment attributes (interpolation, InterpSetting[^interp])

**A segment = the part between adjacent keyframe nodes; the interpolation setting is an attribute of the
segment** — [inference] confirms "timeline segment shape = easing":

| interp field | mo2d segment attribute | Value range |
|---|---|---|
| `im` | **interpolation mode** | 0 Linear / 1 Smooth (default) / 2 Ease / 3 Step / 4 Noisy / 5 Cycle / 6 Pose / 7 EaseIn / 8 EaseOut / 9 Bezier / 10 Bounce / 11 Elastic |
| `h` | hold frames | how many frames the key holds before interpolating |
| `in` | interval | 1 = every frame, 2 = on 2's, 3 = on 3's... |
| `s` | stagger | bool (semantics not detailed) |
| `t` | key color | int32 (shares the code space with bone color tags) |
| `v1`/`v2` | mode parameters | Noisy = amplitude/frequency; Cycle = wrap frames; Pose = action index |
| `b[]` (im=9) | **Bézier handles (per component)** | {ao,ai,po,pi}: out/in angles (radians) + out/in percentages (0..1); segment shape = key i's out-handle + key i+1's in-handle |

**Interpolation-model facts**: one interpolation setting per key, acting on the out-segment; String/Bool
channels are inherently stepped (Step)[^interp].
**mo2d promotion**: per the owner's normative statement "每条线段也有属性值" ("every segment also has
attribute values"), the out-segment semantics are **promoted off the keys into a segment-level attribute**
(interp attaches directly to the segments of the track line, no longer translated through nodes) — §2.2.

### 5.3 Document-level tracks (animated_values, 5 channels[^fmt])

| Moho | mo2d |
|---|---|
| camera_track⟨Vec3⟩ (z=3.732)/camera_zoom⟨Val, val=2.0 i.e. 1:1>/camera_roll/camera_pan_tilt⟨Vec2⟩ | node channels of the **camera transform-box line** (10134-10137) |
| timeline_markers⟨String⟩ (when's first key = -1000000, values carry a trailing \n) | annotation track (10133 DOC_MARKERS) |

## 6. Root parameters and session state

### 6.1 Root parameters (source: project_data's 25 keys[^fmt])

Canvas: width/height/startFrame (default 1)/endFrame/fps (24.0)/backColor (0-255);
Rendering: antialiasing/depthSort/distanceSort/noiseGrain/pixelation/displayQuality/globalRenderStyle{fill,line,layer}/minimizeRandomness;
Depth of field: depthOfField/focusDistance (2.0)/focusRange (1.0)/focusBlur (0.05);
Stereo: stereoMode/stereoSeparation;
Misc: extraSwfFrame (Flash legacy)/colorPalette (→ external reference)/soundtrack (→ external reference, audio payload).
The maop extension maop_color (color management) → root parameter color. **Envelope metadata**:
mime/version/major/rev/doc_uuid/created/modified/comment (cleared on save)/thumbnail (regenerated on save).

### 6.2 Session state (outside the document; source: documentviewstate's 48 keys + onion skin[^fmt])

Grid (gridOn/gridSize/gridStyle/gridSnappingOn), safe area / rule-of-thirds lines / output-only area,
playback range (playStart/playEnd), four viewports (zoom/viewOffset/rotation/stereo/outsideView*),
onion skin (onions_enabled/sellayer/filled/colored/relative/behind + frame0..7 offsets),
layer panel expanded, selection family (_SEL), metadata editor state, `shown_in_timeline`.
→ In mo2d all of these land in **session state** (the third class of the non-line boundary); they are not
carried by save/load (or are saved to a separate session file).

## 7. Complete mo2d-side inventories (merged result)

### 7.1 Line-level attribute vocabulary (with channel marker ⟨ch⟩)

closed, width, cap, fillOn, strokeOn, fillColor⟨ch⟩, strokeColor⟨ch⟩, opacity⟨ch⟩, blend⟨ch⟩,
trimStart⟨ch⟩, trimEnd⟨ch⟩, colorDrift⟨ch⟩, zOrder⟨ch⟩, offset⟨ch⟩, scale⟨ch⟩, rotation⟨ch⟩, thickness3d⟨ch⟩,
brush{name (→ external reference), align, jitter, spacing, angleDrift, randomize, mergedAlpha, tint, randOrder, sizeAmp, sizeScale, randomInterval, hueDrift, satDrift, valDrift}

### 7.2 Node-level attribute vocabulary (all can take ⟨ch⟩)

position, width, color, colorStrength, colored,
handle{type (smoothness), offsetIn, offsetOut, weightIn, weightOut}

### 7.3 Segment-level attribute vocabulary (interp can take ⟨ch⟩)

visible, type (straight/curve), fillRole,
interp{im (0-11, twelve modes), h (hold-frame count), in (on-N's), s (stagger), v1, v2, b[{ao,ai,po,pi}] per-component handles}

### 7.4 Vocabulary of relation types and parameters

| Relation | Parameters (⟨ch⟩ = animatable) |
|---|---|
| layer (organizational) | the full §3.1 table (visible/blendMode/transforms⟨ch⟩/effects 6 groups⟨ch⟩/masking/timingOffset...) |
| group (organizational) | enablePhysics⟨ch⟩, gravity, wind, useBakedPhysics |
| ownership (organizational) | nesting references, zOrder/layerOrdering⟨ch⟩, bone groups, layercomps views |
| node-sharing (connective) | sharing mode (skeleton polylines / mesh intersections) |
| bind | boneRef (parent/parent_bone), strength, flexiBone{elbow,subset}, scalingMode, maxAutoScaling, squashStretch |
| constraint | angleMin/angleMax (-1.222/1.5708 rad), fixedAngle, ik{lock⟨ch⟩,targetBone⟨ch⟩,globalAngle⟨ch⟩,parentTarget⟨ch⟩,arcSolver,ignoredByIK} |
| propagate | controlParent{angle,pos,scale}+scale+delay, weights{angle,pos,scale}, action{name, lookup-table track} |
| derive | widthProfile{sourceLine,repeat}, followCurve{layer,curve,bending}, warp{refLine,frameZero,squashable,triangulated}, text{→ outline} |
| track (temporal) | value types (6), mute, keyframe nodes{when,val}, segment attributes{im,h,in,s,t,v1,v2,b[]}, actions |
| simulate | bone family (dynamics⟨ch⟩ + damping/spring/torque + three-component families + wind/gravity fields⟨ch⟩ + motor⟨ch⟩ + collision radius), layer rigid-body family (physics.* 14), particle family (pending sample files) |
| external-ref | file{path,relativeTo}, layerRef{uuid,sameDoc,modDate}, palette, brush, soundtrack |

### 7.5 Root-parameter vocabulary

The full §6.1 table (canvas 6 + rendering 9 + depth of field 4 + stereo 2 + misc 3 + the color extension).

### 7.6 Payload vocabulary (non-line boundary)

Media: bitmaps (ImageLayer), audio (AudioLayer/soundtrack/level/jump), text (TextLayer text/font families + the bubble 12 keys), notes (NoteLayer);
Logic: channel expressions (expr), POSE interpolation references, fill rules;
Brush files / palette files (external assets).

## 8. Gaps and open items

- [ ] ParticleLayer/NoteLayer/PatchLayer/TrackingPoint/Poser: zero instances in the corpus[^parity]; this document maps them conceptually by layer-type enumeration, parameter tables to be completed once sample files exist;
- [ ] The equivalent formula in mo2d's bind relation for the precise radius semantics of Moho `strength` binding (the flexible weighting of Moho smart binding), to be settled during engineering;
- [ ] Channel-code rearrangement between 13.5.3 and 14.4[^chan]: mo2d does not use numeric channel codes (tracks attach directly to attributes); tables would be selected by version only for a future .moho import;
- [ ] The [inference] items (segment shape = easing / camera = transform-box line / TextLayer derived form) await owner confirmation together with [line-model.md](line-model.md) §7.
