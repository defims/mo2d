---
type: Design Document
title: "万物皆线:线、节点与关系的统一模型"
description: mo2d 核心概念文档——线=有序节点序列,三级属性(线级默认/节点级覆盖插值/段级直存);骨骼/骨架/时间轴/渐变/层/网格/模拟皆为线的投影;四类非线边界与判定标准;附 Moho 概念对照吸收表。
tags: [line-model, node, segment, attribute, core-concepts]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T13:58:22Z }
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

[English](line-model.md) | **简体中文**

# 万物皆线

mo2d 的核心主张:**一切结构与控制皆可归约为线**(Line)。骨骼、时间轴、矢量图形、
位图网格、渐变、层——这些传统软件里彼此异构的概念,在 mo2d 里都是同一种东西:
**由节点组成的线,以及线与线之间的关系**(关系见 [relations.md](relations.zh-CN.md))。

## 0. 规范表述(owner 给定,逐字收录)

> **骨骼是 2 个节点的线段,骨架是一条折线。**
>
> **时间轴是多条有多个节点的直线,每个节点是一个关键帧。**
>
> **渐变是控制棒决定的,控制棒就是线段。**
>
> **层和群组本质相同,都是线的集合;嵌套层、父子层本质是线与线的关系映射;层的缩放旋转用方框表示。**
>
> **模拟过程本质是线和线的关系。**
>
> **集合本质是线和线的关系,只不过是预定义关系;关系就是函数;关系的参数就是集合的参数。**
>
> **线有属性(比如笔触),节点也有相应的属性。**
>
> **一条线,每个节点有属性,每条线段也有属性值。**
>
> **线和线关系可以最大化地利用 GPU,概念契合,提高性能。**

## 1. 线原语与三级属性

**线 = 有序节点序列 + 线级属性。**

- **节点(Node)是一等公民**:矢量图形的控制点、骨骼的关节、时间轴的关键帧、
  位图网格的纹理锚点、渐变的色标——统一为"线上的节点"。
- **节点 = 位置 + 节点级属性 + 语义标签**。
- 相邻节点之间的**段(Segment)**也是一等属性载体:**每条线段有自己的属性值**;
  段可直可曲,其弯曲由两端节点的手柄决定(手柄存于节点、作用于段)。
- 线可**开放**可**闭合**;闭合线围合出**填充域**(填充规则属逻辑载荷,见 §4);
  **推广注记**:3 节点闭合线=三角形面——3D 曲面表示的天然入口([3d-extension.md](3d-extension.zh-CN.md) §2)。

**三级属性模型**(owner:"线有属性,比如笔触;节点也有相应的属性"+"一条线,每个节点有属性,每条线段也有属性值"):

| 层级 | 角色 | 例子 |
|---|---|---|
| 线级属性 | 整条线的默认值 | 笔触(brush style)、基准宽度、颜色、不透明度 |
| 节点级属性 | 同名属性的逐点覆盖,在段之间**插值**;缺省回落线级值 | 节点宽度、节点颜色、压力 |
| 段级属性 | 每条线段的**直接状态值**,不在段间插值(它就是段自身的值) | 段类型(直/曲)、段可见、段插值模式(轨道)、段填充参与 |

- 解析顺序:**段级 → 节点级(两端插值)→ 线级**;下级缺省向上回落。
- 属性本身就是**通道(Channel)**:可采样、可随时间动画(见 [relations.md](relations.zh-CN.md) §轨道)。
- 笔触 = 线级样式 + 节点级压力/宽度变化——一条均匀笔触的线,在节点上压出粗细节奏。
- 佐证:Moho 的 Shape 级 Style(fill_color/line_color)与 Point 级 color/width/color_strength
  通道覆盖 = 线/节点两级先例;`segments_on`(段可见)与插值设置"每键一份、作用于出段"
  ([moho-mapping.md](moho-mapping.zh-CN.md) §5.2)= 段级属性的工程先例。

**GPU 契合**(owner:"线和线关系可以最大化地利用 GPU,概念契合"):
节点级属性天然是 SoA(Structure-of-Arrays)数组——位置数组、宽数组、颜色数组——
直接映射 GPU 顶点属性/存储缓冲,一次上传全场景;详见 [architecture.md](architecture.zh-CN.md) §GPU 策略。

## 2. 关系原语

**关系 = 函数**:输入若干线与参数,输出派生的结构与值。**集合 = 预定义关系**,
关系的参数就是集合的参数(owner 规范表述)。文档 = 全部线 + 全部关系构成的关系图。
完整的关系分类学见 [relations.md](relations.zh-CN.md),本文只保留一句话:层、群组是组织类预定义关系,
绑定、约束、传导、派生、轨道、模拟是其余各类。

## 3. 九大映射

每个传统概念 = 线(或关系)的投影。前四条与第 6/7 条为 owner 规范表述;标 [推导] 者为本文档延伸,待 owner 确认。

| # | 传统概念 | mo2d 表述 | 依据 |
|---|---|---|---|
| 1 | 矢量图形 | **线**:开放线→描边,闭合线→填充域;变宽=节点宽度通道;笔触=线级样式+节点级压力 | 规范表述 |
| 2 | 骨骼 | **2 个节点的线段**;关节即节点 | 规范表述 |
| 3 | 骨架 | **一条折线**;骨骼链首尾相接;分叉(如双臂)= 多条折线**共享节点** | 规范表述 |
| 4 | 时间轴 | **多条含多节点的直线**;每条直线=一条轨道,**节点=关键帧**(position=Tick、值=节点级属性);**插值/缓动=段级属性**(每分量句柄);track 预定义关系=把轨道线接到被动画属性的引用/采样;(t,v) 缓动曲线=轨道的派生视图 | 规范表述 + 已决议([mvp.md](mvp.zh-CN.md) §2.2 P0-1) |
| 5 | 位图网格 | **经纬两组交织的线**;节点即纹理锚点;[推导] 交叉点为经纬线共享节点,变形=线动纹理随动 | 推导 |
| 6 | 渐变 | **控制棒=线段**(2 节点);色标=棒上节点(位置通道=色标位置,颜色通道=色标颜色);[推导] 节点可动画 ⇒ 渐变自然可动画 | 规范表述 + 推导 |
| 7 | 层/群组 | **线的集合**(预定义关系);嵌套/父子=关系映射;深度/遮挡=关系上的排序参数 | 规范表述 |
| 8 | 层变换 | **方框=4 节点闭合线**;[推导] 拖方框的节点/边即施加缩放旋转,变换是对方框这条线的操作 | 规范表述 + 推导 |
| 9 | 模拟 | **线与线的动态关系**(弹簧/力/碰撞/粒子);粒子=发射线发出的有向短线段(位置+速度向量) | 规范表述 |

### 3.1 看似不是线、其实归约自然

| 概念 | 归约 |
|---|---|
| 摄像机 | 一个方框线(4 节点闭合线),推拉摇移=方框节点的动画 |
| 舞台边界 | 一条方框线 |
| 播放头 | 时间基准线上的一条垂直短线 |
| 运动路径 | 舞台上一条线(物体沿它走) |
| 音量包络 | 一条线 |
| 笔刷散布 | 沿线按间距通道盖章 |
| 口型/切换(Switch) | 轨道直线上携带离散值的节点 |

## 4. 非线边界(四类)

经压力测试(owner 问"还有哪些概念不能和线关联/和线没关系"两轮)收敛:**结构与关系上万物皆线;
以下四类不可再收编,再收编即硬扯**(如把一段 MP3 说成"线"没有意义)。

| # | 类别 | 内容 | 收不进来的原因 |
|---|---|---|---|
| 1 | **采样场** | 位图像素、音频采样、视频帧 | 非几何——是数据。线只负责锚定(网格)、变形(经纬线)、驱动(包络线);图像轮廓可**派生**为线,但那是输出,不是数据本身 |
| 2 | **符号与逻辑** | 代码(表达式/自定义关系函数体/求解器/事件脚本)、文本串与字体(字形轮廓可派生为线,语义源不是线)、命名/注释/语义标签(依附于线的元数据) | 非几何——是语义/规则。填充规则(非零/奇偶)归此类:它是线集合的解释规则 |
| 3 | **文档外的流程与状态** | 会话状态(视图相机/选择集/撤销栈/剪贴板/吸附设置)、协作与版本、导出与发布配置 | 非文档——是环境 |
| 4 | **外部资产引用** | 位图/音频/字体文件等文档外实体 | 不在文档内——关系的一端可以指向非线实体 |

判定标准:**非几何 / 非文档 / 文档外端点**。
文档级参数(帧率、色彩空间)不是独立类别——它们是**根关系的参数**(见 [relations.md](relations.zh-CN.md))。

## 5. 数据模型骨架(栈无关伪类型)

```ts
Line {
  id: LineId
  nodes: [Node]                 // 有序;闭合线首尾相接
  segAttrs: [SegmentAttrs]      // 段级属性——与"节点间段"一一平行(n-1 条;闭合线 n 条)
  attrs: LineAttrs              // 线级属性(笔触/基准宽度/颜色/不透明度…)
  tags: [SemanticTag]           // 语义标签(如 "bone"/"skeleton"/"track"/"mesh-warp")
}

Node {
  id: NodeId
  position: Vec3 | Tick         // 几何线节点=Vec3(2D 恒 z=0,3D 扩展决议);轨道线节点=Tick(1D 时间)
  attrs: NodeAttrs              // 节点级属性——与线级同名者覆盖,段间插值,缺省回落线级
  handleIn/handleOut: Offset?   // 手柄存于节点、作用于相邻段(出/入)
}

SegmentAttrs {                  // 段的直接状态值,不在段间插值
  type: straight | curve        // 段类型(直/曲;弯曲程度由两端节点手柄决定)
  visible: bool                 // 段可见(源:Moho segments_on)
  fillRole?: fillFlag           // 段参与填充边界的方式(源:Moho Shape edges flag)
  interp?: InterpSpec           // 轨道线的段:插值/缓动模式(源:Moho InterpSetting 出段语义)
}

Relation {                       // 关系 = 函数
  id: RelationId
  kind: "predefined" | "custom"
  type: RelationType             // layer/group/bind/constraint/propagate/derive/track/simulate/…
  inputs: [LineRef | NodeRef | RelationRef]
  params: Params                 // 关系的参数 = 集合的参数;参数即通道,可动画
  body?: LogicPayload            // 自定义关系的函数体(逻辑载荷)
}

Document {                       // 文档 = 全部线 + 全部关系
  lines: [Line]
  relations: [Relation]          // 含根关系(文档级参数挂在它上面)
}
```

## 6. Moho 概念对照吸收表

Moho 是骨骼动画领域概念最完整的参照系。**逐概念完整映射(31 顶层键/10 层型/Bone 61 键/
mesh 三层/styles 34 键/通道码 135 项,含 mo2d 侧完整属性与关系参数词表)见
[moho-mapping.md](moho-mapping.zh-CN.md)**——本节只保留骨架级对照。
下表事实取自 maohou 知识库的一手归档(Moho Pro 14.4 / mohoscripting.com)[^mohoproj-format][^m-bone],映射为吸收/简化决策:

| Moho 概念 | Moho 事实 | mo2d 映射 | 决策 |
|---|---|---|---|
| Curve(曲线) | 每点 5 通道(smoothness/offset_in/out/weight_in/out)+ trim 起止百分比(可动画)+ 变宽轮廓 profile_curve_id[^mohoproj-format] | 线 + 节点手柄/宽度通道;trim=轨道上的起止通道 | 吸收(几乎同构) |
| Point(点) | position/width/color/color_strength 通道(均可动画)+ parent 绑骨索引[^mohoproj-format] | 节点 + 节点级属性 + 绑定关系的输入端 | 吸收(两级属性同构) |
| Bone(骨骼) | fLength/fAngle/fPos(静态)+ AnimChannel:angle/pos/scale;角约束 fConstraints/fMin/fMax;动力学 fBoneDynamics+弹簧/阻尼/力族;智能骨骼 Action 权重[^m-bone] | 2 节点线段;角度/位置/缩放=轨道;角约束→约束关系;动力学→动态关系;智能骨骼→传导关系 | 吸收(语义),简化(长度=两点距离,非独立字段) |
| mesh(矢量网格) | points/curves/shapes 三层;Shape 的 edges 引用 curve 段,has_fill/has_outline 决定同一曲线的线/面角色[^mohoproj-format] | 线集合;闭合线围合填充;描边/填充为线的渲染解释 | 吸收+简化(去三层容器,直接线+关系) |
| 10 种图层类型(Vector/Image/Audio/Group/Switch/Bone/Particle/Note/Text/Patch) | 各带专有键族[^mohoproj-format] | 预定义关系的类型模板(层的参数表) | 参照(哪些进标准关系库、哪些退化为语义标签,工程期定) |
| fileref{path,relativeTo} | 外部资产引用机制[^mohoproj-format] | 引用类关系(端点为文档外实体) | 吸收 |
| 智能骨骼(Action) | 拨盘骨骼角度驱动多通道查找表[^m-bone] | 传导关系(控制线驱动多条目标线) | 吸收并推广 |

**工程红线**:本表仅作概念对照;mo2d 为净室自主实现,不承诺 .moho 兼容
(导入器仅列远期可选项,见 [roadmap.md](roadmap.zh-CN.md))。

## 7. 待确认事项

- [ ] 段级属性已入规范;轨道段"插值/缓动=段属性"的取值语义(im 12 模式、h/in、
      贝塞尔句柄与段几何的关系)沿 [moho-mapping.md](moho-mapping.zh-CN.md) §5.2 待 owner 确认;
- [ ] [推导] 位图网格交叉点=经纬线共享节点,是否采纳;
- [ ] [推导] 变换=拖方框线,交互细节工程期定;
- [ ] 语义标签的初始词表(bone/skeleton/track/mesh-warp/…)工程期定。
