---
type: Design Document
title: "Moho 概念逐项映射:点线属性关系全表"
description: Moho Pro 14.4 全部概念逐个理解并归入 mo2d 的线/节点/三级属性(线级/节点级/段级)/九类关系/通道/载荷——覆盖 31 顶层键、project_data 25 键、10 层型 49 公共键、mesh 三层结构、Bone 61 键、styles 34 键、通道码 135 项,产出 mo2d 侧完整属性值与关系参数清单。
tags: [moho, mapping, attributes, segment, relations, reference-table]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T09:53:55Z }
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

# Moho 概念逐项映射:点线属性关系全表

本文把 Moho Pro 14.4 的**每一个概念**理解后归入 mo2d 的模型槽位,并整理出
mo2d 侧**完整的属性值与关系参数清单**。事实全部来自 maohou 知识库的实证归档
(8 个 corpus fixtures 为 Moho 14.4 原生保存,置信度高[^fmt]);
映射判断是 mo2d 的设计内容(标注:吸收=直接建模 / 简化=换更简形态 / 退化=转为标签或参数 / 边界=留在非线边界内)。

## 0. 槽位判定规则

每个 Moho 键/概念按顺序问四个问题,落到十个槽位之一:

1. 是**几何路径本身**吗?→ 线;路径上的点 → 节点;节点之间的段 → 段;
2. 是**逐段持有**的直接状态(不在段间插值)吗?→ 段级属性;
3. 是**逐点变化**的量吗?→ 节点级属性(段间插值);否则整条一致 → 线级属性;
4. 描述**两个对象之间**的行为吗?→ 关系(九类预定义之一)+ 关系参数;
5. 随时间变化吗?→ 任何属性都可加轨道(通道);
6. 都不是 → 根参数 / 载荷(媒体/逻辑) / 会话状态 / 信封元数据。

| 槽位 | mo2d 定义 |
|---|---|
| 线 | 有序节点序列 + 线级属性 |
| 节点 | 线上的点:位置 + 节点级属性 |
| 线级属性 | 整线默认值(笔触/宽度/颜色/开关…) |
| 节点级属性 | 同名局部覆盖,段间插值,回落线级 |
| 段级属性 | 每条线段的直接状态值,不在段间插值(owner:"每条线段也有属性值") |
| 关系+参数 | 九类预定义关系([relations.md](relations.md) §1) |
| 轨道 | 时间直线 + 关键帧节点(值/插值/动作) |
| 根参数 | 根关系(文档级)参数 |
| 载荷 | 媒体场 / 逻辑 |
| 会话状态 | 文档外编辑器状态 |
| 信封 | 文件格式元数据 |

## 1. 总表:概念 → 槽位

| Moho 概念 | mo2d 槽位 | 处置 |
|---|---|---|
| Curve(曲线) | **线** | 吸收 |
| Point(点) | **节点** | 吸收 |
| 线段状态(segments_on/edges flag/插值出段) | **段级属性** | 吸收(Moho 散存于点/键,mo2d 提升为段) |
| Shape(形状) | 线 + 渲染解释(描边/填充开关) | 简化 |
| Style(样式 34 键) | **线级属性模板**(库) | 吸收 |
| Bone(骨骼) | **线**(2 节点线段,语义标签 bone) | 吸收(长度=两点距离,非独立字段) |
| Skeleton(骨架) | **折线/多折线 + 节点共享** | 吸收 |
| 骨骼组(bones_groups) | 关系(归属)的命名子集 | 退化 |
| 绑定(Point.parent/parent_bone/strength) | **绑定关系** + 权重参数 | 吸收 |
| 角约束(constraints/min/max/fixed_angle) | **约束关系** | 吸收 |
| 控制骨(angle/pos/scale_control_*) | **传导关系**(带 delay/scale 参数) | 吸收 |
| 智能骨骼 Action(actions/pose) | **传导关系**(命名查找表) | 吸收并推广 |
| IK(ik_lock/target_bone/ik_*) | 约束关系(端点目标)+ 节点参数 | 吸收 |
| 骨骼动力学(bone_dynamics + 力族) | **模拟关系** | 吸收 |
| 层变换(translation/scale/rot/flip/shear 10 通道) | 层关系的参数(通道)+ **方框线** | 吸收 |
| 层效果(6 组 40 子通道) | 层关系的参数(通道) | 吸收 |
| 层物理(physics 14 键) | 模拟关系参数 | 吸收 |
| 跟随曲线(follow_curve/following) | **派生关系**(层沿线) | 吸收 |
| 扭曲参考层(distortion/warp) | **派生关系**(网格变形) | 吸收 |
| 层引用(layer_ref_* 族) | **外部引用关系** | 吸收 |
| 层嵌套(Group/Bone/Switch 容器) | **归属关系**(组织类) | 吸收 |
| SwitchLayer(switch_keys) | 轨道(String/离散值节点) | 吸收 |
| 摄像机(camera_track/zoom/roll/pan_tilt) | **方框线**(节点通道) | 吸收 |
| 时间轴标记(timeline_markers) | 轨道上的标注节点 | 吸收 |
| 通道对象(⟨ch:T⟩ 全部) | **轨道线** | 吸收 |
| 插值(interp 字段族) | **段属性**(关键帧节点间) | 吸收 |
| 样式颜色关键帧 | 线级属性通道 | 吸收 |
| TextLayer | 载荷(文本)+ **派生关系**(→轮廓线) | 简化 |
| ImageLayer | 载荷(位图)+ 绑定(网格线锚定) | 吸收 |
| AudioLayer/soundtrack | 载荷(音频)+ 轨道(level/jump) | 边界(媒体场) |
| 粒子层 ParticleLayer | 模拟关系(发射线) | 边界待证(corpus 零实例) |
| NoteLayer/PatchLayer | 语义标签 + 载荷 | 边界待证(corpus 零实例) |
| project_data 工程设置 | **根参数** | 吸收 |
| documentviewstate/洋葱皮 | **会话状态** | 边界(文档外) |
| _SEL 伴随通道 | 会话状态(选择) | 边界 |
| mime/version/uuid/dates | 信封 | 吸收 |
| layercomps(层组合) | 归属关系的命名视图 | 退化 |
| tracking point / poser | 节点语义标签 + 关系 | 待定(无样张) |

## 2. 矢量 mesh:线与节点的完整属性值

Moho 的矢量层 = `mesh` 子结构(points/curves/shapes 三层)[^fmt]。mo2d 去掉三层容器,
直接落在线/节点上——**两级属性的事实先例**。

### 2.1 节点级属性(源:Point 9 键[^fmt] + 通道码 POINT 族[^chan])

| Moho 键 | 通道码 | mo2d 节点属性 | 值域/说明 |
|---|---|---|---|
| `position` ⟨ch:Vec2⟩ | 10030 | **position 通道** | Vec2;轨道可动画 |
| `width` ⟨ch:Val⟩ | 10038(线宽)/10054(THICKNESS) | **width 通道** | f64;线级 line_width 为回落值 |
| `color` ⟨ch:Color⟩ | 10056 POINTCOL | **color 通道** | {r,g,b,a};沿线插值=渐变 |
| `color_strength` ⟨ch:Val⟩ | 10058 POINTSTRENGTH | **color_strength 通道** | 0..1;colored=true 时生效 |
| `colored` bool | — | color 开关(节点级) | 缺省 false=回落线级色 |
| `smoothness` ⟨ch:Val⟩ | 10032 CURVE(每点) | **handle 类型**(直/曲) | 0=直线段,>0=平滑度 |
| `offset_in`/`offset_out` ⟨ch:Val⟩ | 10032 | **手柄偏移** | 控制段曲率(贝塞尔手柄) |
| `weight_in`/`weight_out` ⟨ch:Val⟩ | 10032 | **手柄权重** | 手柄影响力 |
| `segments_on` bool | — | 段可见开关 | 该点与下一点之间的段 |
| `parent` int(绑骨) | — | **绑定关系端点**(非属性) | -2=无;见 §4.1 |
| `selected` bool | 10031 POINT_SEL | 会话状态 | 不入文档模型 |
| `curves`(引用) | — | 结构性反向索引 | 引擎内部 |

**结论:节点级属性首版词表 = {position, width, color, color_strength, colored, handle{type, offsetIn, offsetOut, weightIn, weightOut}}**;
其中 position/width/color/color_strength/handle 族全部可挂轨道。
(`segments_on` 原在点上、语义指向"该点与下一点之间的段"——按 owner 规范表述**归入段级属性**,见 §2.2。)

### 2.2 段级属性(owner:"每条线段也有属性值")

Moho 把段的状态散存在点/键上;mo2d 提升为**段自身的属性**(不在段间插值——它就是段的值):

| Moho 事实(存储位置) | mo2d 段级属性 | 值域/说明 |
|---|---|---|
| `segments_on`(Point 上) | **visible** | 该段可见开关 |
| `smoothness`=0 判直段(Curve 每点) | **type** | straight / curve;段弯曲由两端节点手柄决定(手柄仍存节点、作用于段) |
| Shape `edges {curve, segment, flag}` | **fillRole** | 段参与填充边界的方式(方向/角色) |
| InterpSetting 出段语义(存于键 i,作用于段 i→i+1,见 §5.2) | **interp**(轨道线) | im(12 模式)/h(保持帧)/in(on-N's)/s(错帧)/v1/v2/b 句柄 |

### 2.3 线级属性(源:Curve 9 键[^fmt] + Shape 16 键[^fmt] + Style 34 键[^fmt])

| Moho 键 | mo2d 线级属性 | 值域/说明 |
|---|---|---|
| `closed` bool | **closed** | 闭合线围合填充域 |
| `start_percent`/`end_percent` ⟨ch:Val⟩ | **trimStart/trimEnd 通道** | 修剪起止(CURVEEXP 10046/PROFILEOFFSET 10048 对应族[^chan]) |
| `profile_curve_id`/`profile_layer_uuid`/`profile_repeat` | **widthProfile 派生关系**(轮廓线→宽度) | 变宽轮廓:另一条线作为本线宽度源 |
| `has_fill`/`has_outline`(Shape) | **fillOn/strokeOn** | 同一条线的两种渲染解释 |
| `fill_color`/`line_color` ⟨ch:Color⟩(Style) | **fillColor/strokeColor 通道** | 样式级也可动画(样式=属性模板) |
| `line_width`(Style) | **width(线级回落值)** | 默认 0.022 |
| `line_caps`(Style) | **cap** | 线帽 |
| Style `define_*` 三开关 | 属性模板的启用位 | 模板只覆盖勾选的属性 |
| `effect_offset`/`effect_scale`/`effect_rotation`(Shape ⟨ch⟩) | **offset/scale/rotation 通道**(线级变换) | FXXFORM 10052 族[^chan] |
| `3d_thickness` ⟨ch:Val⟩ | thickness3d | 3D 厚度 |
| `shape_order` ⟨ch:String⟩/SHAPE_ORDER 10060 | **zOrder 通道** | 深度/遮挡=排序参数(通道化) |
| `name`/`id` | 语义标签 + id | — |
| `groups`(mesh.groups) | 归属关系(命名子集) | — |

### 2.4 笔触(线级,源:Style brush_* 13 键[^fmt])

**brush 属性组(整组线级,可作属性模板)**:
`brush_name`(笔刷文件→外部引用)、`align`、`jitter`、`spacing`(默认 0.38)、
`angle_drift`、`randomize`、`merged_alpha`、`tint`、`rand_order`、`size_amp`(0.14)、
`size_scale`(0.03)、`random_interval`(1)、`hue_drift`(-0.02)、`sat_drift`(0.33)、`val_drift`(0.12)。
另 COLOR_DRIFT 10042/OPACITY 10040/BLEND 10044 通道族[^chan] → 线级 colorDrift/opacity/blend 通道。

## 3. 层:组织关系及其完整参数

Moho 层 = 49 公共键 + 层型专有键[^fmt]。mo2d:层是**预定义关系**(owner 规范表述),
下列键全部成为**层关系的参数**(标 ⟨ch⟩ 者可挂轨道)。

### 3.1 层关系参数(源:49 公共键[^fmt])

| 参数组 | 键 → mo2d 参数名 |
|---|---|
| 标识/显示(9) | name、label_col→labelColor、visible、shownInTimeline、expanded(会话倾向)、origin、blendMode;uuid→信封 |
| 渲染开关(7) | renderOnly、editOnly、cameraImmune、dofImmune、consolidatedChannels、qualityFlags、masking |
| 变换(10 通道) | translation⟨Vec3⟩、scale⟨Vec3⟩(比例制)、rotationX/Y/Z⟨Val⟩(弧度)、flipH/flipV⟨Bool⟩、shear⟨Vec3⟩、following⟨Val⟩、physicsNudge⟨Vec2⟩ —— **层变换的操作面=方框线**(4 节点闭合线) |
| 效果-透明度/可见 | alpha⟨Val⟩(10014)、visibility⟨Bool⟩(10012) |
| 效果-模糊(10013) | blur⟨Val⟩ |
| 效果-AO(10015) | ambientOcclusion⟨Val⟩ |
| 效果-噪点(10017) | noise⟨Val⟩ |
| 效果-像素化(10018) | pixelation⟨Val⟩ |
| 效果-阈值(10016) | threshold⟨Val⟩ |
| 效果-描边(10023) | outline{color⟨Color⟩, on⟨Bool⟩, width⟨Val⟩} |
| 效果-着色(10020) | shading{angle,blur,color,contraction,noiseAmp,noiseScale,offset,on,threshold} |
| 效果-阴影(10019) | shadow{angle,blur,clipToGroup,color,expansion,noiseAmp,noiseScale,offset,on,threshold} |
| 效果-透视影(10022) | perspectiveShadow{blur,color,on,scale,shear,threshold} |
| 效果-运动模糊(10021) | motionBlur{alphaEnd,alphaStart,extendedFrames,framePercentage,frames,on,radius,skip,subFrames} |
| 层色/标注 | layerColor{color⟨Color⟩,on⟨Bool⟩}(10024)、userComments、userTags、timelineMarkers⟨String⟩(10025)、layerOrdering⟨String⟩(10124 LAYER_ORDER) |
| 时序/杂项 | timingOffset、randomNum、rotateToFollow、scaleCompensation、scaleNormalization、faceCamera、faceCameraMode、animatedLayerEffects |
| 蒙版 | maskExpansion、groupMask |

### 3.2 各层型 → mo2d 关系/线形态

| 层型(键数) | mo2d 归宿 | 要点 |
|---|---|---|
| MeshLayer(70) | 线集合(语义标签 vector) | §2 全部 |
| GroupLayer(61) | **归属关系**(group 型) | 参数:enablePhysics⟨Bool⟩(10125)、gravity{x,y}、wind{x,y}、useBakedPhysics;子层=layers 嵌套 |
| BoneLayer(59) | **骨架折线族**(语义标签 skeleton) | 骨架参数:bindingMode(int=1)、bonesGroups(骨骼组)、wind/gravity(14.x,层给骨的风/重力场)、grandpaBone;骨内键见 §4 |
| SwitchLayer(57) | 归属关系(switch 型)+ **String 轨道** | switchKeys⟨ch:String⟩=当前子层名(10123,节点携带离散值);switchInterpolation(false=阶跃)、frameByFrame、previewAlignment;SwitchLayer 也有简化 skeleton |
| TextLayer(74) | 载荷(文本)+ **派生关系**(text→轮廓线) | text/textsize/font/justification/kerning/leading/fill/stroke/fillcolor/linecolor/linewidth + 气泡 balloon* 12 键;**内嵌完整 mesh_layer(70 键)= 派生输出的落点**——文本源变,轮廓线重派生 |
| ImageLayer | 载荷(位图)+ 绑定(网格线锚定) | image_fileref{path,relativeTo}→外部引用;变形=distortion 参考层(§3.4) |
| AudioLayer | 载荷(音频)+ 轨道 | audioLevel(10129)/audioJump(10130);媒体场=非线边界 |
| ParticleLayer | **模拟关系**(发射线) | 10126;corpus 零实例,参数表留待样张[^parity] |
| NoteLayer | 载荷(文本注释)+ 锚定 | corpus 零实例[^parity] |
| PatchLayer | 派生关系(补丁修形) | 依赖图像变形;corpus 零实例[^parity] |

**层间关系(MohoLayer API 事实[^mlayer])**:层父子=容器嵌套(Parent() 返回 Group 型;变换继承 GetFullTransform)→ mo2d 归属关系+传导(父层变换→子层);**层绑骨** parent_bone 键(层可绑到骨骼线)→ 绑定关系;**层跟随曲线** follow_curve/follow_layer_uuid/follow_bending(10008 FOLLOW)→ 派生关系;**弯曲骨骼跟随** flexi_bone_elbow/flexi_bone_subset → 派生+绑定混合。

### 3.3 智能骨骼 Action(传导关系)

存储形态[^fmt][^achan]:通道对象内 `actions:[{name, pose:{when:[帧],val:[值]}}]`,
pose.when 为**整数帧号**(=触发骨骼拨盘角度的应答表);层对象另持 `actions:[{name,pose:int}]` 引用表。
mo2d = **传导关系**:参数 {name、源线(拨盘)、查找表(轨道)、域外羽化};
Action 权重 angle_weight/pos_weight/scale_weight[^fmt] → 传导关系的混合权重参数。

### 3.4 引用与变形

| Moho | mo2d 关系 |
|---|---|
| `layer_ref_uuid/same_doc/mod_date/path/fileref` | **外部引用关系**(跨文档层引用,可断链/更新[^mlayer]) |
| `distortion_layer_uuid`(warp,14.0) | **派生关系**:目标线集合由参考网格线变形 |
| `frame_zero_deformer`/`squashable_deformer`/`triangulated` | 派生关系的模式开关 |
| layercomps[{name,uuid,layer_ids}] | 归属关系的命名视图(层组合) |

## 4. 骨骼:线段、折线与关系的完整参数

### 4.1 结构

- **Bone = 2 节点线段**(owner 规范表述);`length`(默认 0.021,非像素)+`offset`+`anim_pos`/`anim_angle` 决定两端点——mo2d 直接存两端节点,length 派生;
- **骨架 = 折线**:`parent`(int 索引,-1=根)+ 首尾相接;分叉=多折线**共享节点**;
  Moho 事实:骨骼平表+整型 id 寻址,层级经 parent 查询[^mskel]——与"折线共享节点"同构;
- `anim_parent` ⟨ch:Val⟩(父骨通道化,11.0)→ **归属关系可动画**(重挂父);
- 骨骼组 `bones_groups`[^fmt] / AddGroup API[^mskel] → 归属关系命名子集(智能骨骼常按组引用)。

### 4.2 骨骼通道(轨道)与静态参数(Bone 61 键全归宿[^fmt][^mbone])

| Moho 键 | mo2d 归宿 | 说明 |
|---|---|---|
| `anim_angle`⟨Val⟩/`anim_pos`⟨Vec2⟩/`anim_scale`⟨Val 比例制,均匀缩放> | **节点/线通道**(轨道) | 角度弧度制;BONE_T/S 槽位 10074-10108[^chan] |
| `flip_h`/`flip_v`⟨Bool⟩ | 线通道 | 10100-10103 |
| `ik_lock`⟨Bool⟩/`ik_global_angle`/`ik_parent_target` | **约束关系参数**(IK,可动画) | Lock 按帧[^mskel]——lock 可动画的旁证 |
| `target_bone`⟨Val val=-1⟩ | 约束关系(端点目标骨) | 10106/10107 |
| `constraints`/`min_constraint`(-1.222rad)/`max_constraint`(1.5708rad)/`fixed_angle` | **约束关系参数**(角约束) | 静态 |
| `strength`(默认 1.0) | **绑定关系参数**(影响权重) | Moho 无逐点权重编辑,强度=绑定半径语义 |
| `scaling_mode`(0 手动/1 2D/2 挤压拉伸)/`max_auto_scaling`/`squash_stretch_scaling` | 绑定关系参数(缩放传导模式) | 静态 |
| `bone_dynamics`⟨Bool⟩(10110) | **模拟关系开关** | 可动画 |
| `damping_force`(1.0)/`spring_force`(2.0)/`torque_force`(2.0) | 模拟关系参数(角) | 静态 |
| `angle_dynamics`/`pos_dynamics`/`scale_dynamics`(bool)+ 各自 damping/spring/torque | 模拟参数(分通道族) | 静态 |
| `wind_dynamics` + WIND_DIR/STRENGTH/TURBULENCE(10118-10120)+ GRAVITY_DIR/STRENGTH(10121-10122,14.x)[^chan] | 模拟参数(场)+ 层级 wind/gravity 场 | 部分通道化 |
| `physics_radius`(-1.0)/`physics_torque`/`physics_lock_tip`/`physics_return_to_zero` | 模拟参数(碰撞/回零) | 静态 |
| `physics_motor_speed`⟨Val⟩(10112) | 模拟参数(马达) | 可动画 |
| `angle/pos/scale_control_parent`(-1)/`_scale`/`_delay` | **传导关系参数**(控制骨:父/缩放/延迟) | 静态;delay=传导时滞 |
| `angle_weight`/`pos_weight`/`scale_weight`(14.0) | 传导混合权重 | 静态 |
| `bone_enable_arc_solver` | 约束求解器开关(弧线 IK) | 静态 |
| `hidden`/`shy`/`selected`/`bone_label_showing` | 会话状态/显示开关 | shy/selected=会话 |
| `bone_tags`(int32 色标签) | 语义标签(色) | — |
| `offset`{x,y} | 节点属性(绘制偏移) | — |
| `ignored_by_ik` | 约束参数(IK 旁路) | 静态 |
| `grandpa_bone`(层) | 归属参数 | — |

### 4.3 骨骼动力学补充(层物理 14 键[^fmt])

`physics.enabled/static/density/friction/restitution/respawn/sleeping/pivot/velocity/force_field/force_field_vector⟨Vec2⟩(10011)/enable_motor/motor_speed⟨Val⟩(10010)/motor_torque`
→ 模拟关系参数(刚体族);`physics_nudge`⟨Vec2⟩(10009)→ 层变换通道。

## 5. 通道系统:轨道线、关键帧节点、段属性

### 5.1 轨道(线)与关键帧(节点)

- 通道六类型:Val(f64)/Vec2/Vec3/Bool/Color/String[^achan] → 轨道值类型;
- 通道对象 {type,ref,mute,when[],val[],interp[],actions[]}[^fmt]:
  - `mute` → 轨道静音参数;
  - `when[]`(整数帧) → **节点的时间位置**;
  - `val[]` → **节点值**;
  - `actions[]` → §3.3 传导查找表;
- 几乎每个通道有 `_SEL` 伴随通道[^chan] → **会话状态**(选择不入文档);
- `expr`(maop 扩展)→ 逻辑载荷挂轨道。

### 5.2 段属性(插值,InterpSetting[^interp])

**段 = 相邻关键帧节点之间的部分;插值设置是段的属性**——[推导] 印证"时间轴段形状=缓动":

| interp 字段 | mo2d 段属性 | 值域 |
|---|---|---|
| `im` | **interpolation 模式** | 0 Linear/1 Smooth(默认)/2 Ease/3 Step/4 Noisy/5 Cycle/6 Pose/7 EaseIn/8 EaseOut/9 Bezier/10 Bounce/11 Elastic |
| `h` | hold 帧数 | 该键保持多少帧后再插值 |
| `in` | interval | 1=逐帧,2=on 2's,3=on 3's… |
| `s` | stagger 错帧 | bool(语义未详) |
| `t` | 键颜色 | int32(与骨骼色标签共用码) |
| `v1`/`v2` | 模式参数 | Noisy=幅度/频率;Cycle=回绕帧;Pose=动作索引 |
| `b[]`(im=9) | **贝塞尔句柄(每分量)** | {ao,ai,po,pi}:出/入角(弧度)+出/入百分比(0..1);段形状=键 i 出句柄+键 i+1 入句柄 |

**插值模型事实**:插值设置每键一份、作用于出段;String/Bool 通道本就阶跃(Step)[^interp]。
**mo2d 提升**:按 owner 规范表述"每条线段也有属性值",出段语义从键上**提升为段级属性**
(interp 直接挂在轨道线的段上,不再经由节点转译)——§2.2。

### 5.3 文档级轨道(animated_values 5 通道[^fmt])

| Moho | mo2d |
|---|---|
| camera_track⟨Vec3⟩(z=3.732)/camera_zoom⟨Val,val=2.0 即 1:1>/camera_roll/camera_pan_tilt⟨Vec2⟩ | **摄像机方框线**的节点通道(10134-10137) |
| timeline_markers⟨String⟩(when 首键=-1000000,值带尾随\n) | 标注轨道(10133 DOC_MARKERS) |

## 6. 根参数与会话状态

### 6.1 根参数(源:project_data 25 键[^fmt])

画布:width/height/startFrame(默认 1)/endFrame/fps(24.0)/backColor(0-255);
渲染:antialiasing/depthSort/distanceSort/noiseGrain/pixelation/displayQuality/globalRenderStyle{fill,line,layer}/minimizeRandomness;
景深:depthOfField/focusDistance(2.0)/focusRange(1.0)/focusBlur(0.05);
立体:stereoMode/stereoSeparation;
杂项:extraSwfFrame(Flash 遗留)/colorPalette(→外部引用)/soundtrack(→外部引用,音频载荷)。
maop 扩展 maop_color(色彩管理)→ 根参数 color。**信封元数据**:mime/version/major/rev/doc_uuid/created/modified/comment(保存时清空)/thumbnail(保存时再生)。

### 6.2 会话状态(文档外,源:documentviewstate 48 键 + 洋葱皮[^fmt])

网格(gridOn/gridSize/gridStyle/gridSnappingOn)、安全区/三分线/仅输出区、播放范围(playStart/playEnd)、
四视口(zoom/viewOffset/rotation/stereo/outsideView*)、洋葱皮(onions_enabled/sellayer/filled/colored/relative/behind + frame0..7 偏移)、
层面板 expanded、选中族(_SEL)、metadata 编辑器状态、`shown_in_timeline`。
→ mo2d 全部落**会话状态**(非线边界第三类);保存/载出不携带(或另存会话文件)。

## 7. mo2d 侧完整清单(归并结果)

### 7.1 线级属性词表(含通道标记⟨ch⟩)

closed、width、cap、fillOn、strokeOn、fillColor⟨ch⟩、strokeColor⟨ch⟩、opacity⟨ch⟩、blend⟨ch⟩、
trimStart⟨ch⟩、trimEnd⟨ch⟩、colorDrift⟨ch⟩、zOrder⟨ch⟩、offset⟨ch⟩、scale⟨ch⟩、rotation⟨ch⟩、thickness3d⟨ch⟩、
brush{name(→外部引用),align,jitter,spacing,angleDrift,randomize,mergedAlpha,tint,randOrder,sizeAmp,sizeScale,randomInterval,hueDrift,satDrift,valDrift}

### 7.2 节点级属性词表(全部可⟨ch⟩)

position、width、color、colorStrength、colored、
handle{type(smoothness)、offsetIn、offsetOut、weightIn、weightOut}

### 7.3 段级属性词表(interp 可⟨ch⟩)

visible、type(straight/curve)、fillRole、
interp{im(0-11 十二模式)、h(保持帧数)、in(on-N's)、s(错帧)、v1、v2、b[{ao,ai,po,pi}] 每分量句柄}

### 7.4 关系类型与参数词表

| 关系 | 参数(⟨ch⟩=可动画) |
|---|---|
| layer(组织) | §3.1 全表(visible/blendMode/transforms⟨ch⟩/效果 6 组⟨ch⟩/masking/timingOffset…) |
| group(组织) | enablePhysics⟨ch⟩、gravity、wind、useBakedPhysics |
| ownership(组织) | 嵌套引用、zOrder/layerOrdering⟨ch⟩、骨骼组、layercomps 视图 |
| node-sharing(连接) | 共享模式(骨架折线/网格交点) |
| bind(绑定) | boneRef(parent/parent_bone)、strength、flexiBone{elbow,subset}、scalingMode、maxAutoScaling、squashStretch |
| constraint(约束) | angleMin/angleMax(-1.222/1.5708 rad)、fixedAngle、ik{lock⟨ch⟩,targetBone⟨ch⟩,globalAngle⟨ch⟩,parentTarget⟨ch⟩,arcSolver,ignoredByIK} |
| propagate(传导) | controlParent{angle,pos,scale}+scale+delay、weights{angle,pos,scale}、action{name,查找表轨道} |
| derive(派生) | widthProfile{sourceLine,repeat}、followCurve{layer,curve,bending}、warp{refLine,frameZero,squashable,triangulated}、text{→轮廓} |
| track(时间) | 值类型(6)、mute、关键帧节点{when,val}、段属性{im,h,in,s,t,v1,v2,b[]}、actions |
| simulate(模拟) | bone 族(dynamics⟨ch⟩+damping/spring/torque+三分量族+wind/gravity 场⟨ch⟩+motor⟨ch⟩+collision 半径)、layer 刚体族(physics.* 14)、粒子族(待样张) |
| external-ref(引用) | file{path,relativeTo}、layerRef{uuid,sameDoc,modDate}、palette、brush、soundtrack |

### 7.5 根参数词表

§6.1 全表(画布 6 + 渲染 9 + 景深 4 + 立体 2 + 杂项 3 + color 扩展)。

### 7.6 载荷词表(非线边界)

媒体:位图(ImageLayer)、音频(AudioLayer/soundtrack/level/jump)、文本(TextLayer text/font 族+气泡 12 键)、注释(NoteLayer);
逻辑:通道表达式(expr)、POSE 插值引用、填充规则;
笔刷文件/调色板文件(外部资产)。

## 8. 缺口与待定

- [ ] ParticleLayer/NoteLayer/PatchLayer/TrackingPoint/Poser:corpus 零实例[^parity],本文按层型枚举做概念级映射,参数表待样张补全;
- [ ] Moho `strength` 绑定的精确半径语义(Moho 智能绑定的柔性权重)在 mo2d 绑定关系中的等价公式,工程期定;
- [ ] 通道码 13.5.3 与 14.4 重排[^chan]:mo2d 不使用数字通道码(轨道直接挂属性),仅在远期 .moho 导入时按版本选表;
- [ ] [推导] 项(段形状=缓动/摄像机=方框线/TextLayer 派生形态)沿 [line-model.md](line-model.md) §7 一并待 owner 确认。
