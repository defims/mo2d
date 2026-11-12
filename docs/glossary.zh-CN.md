---
type: Reference
title: "术语表(中英对照)"
description: mo2d 全部术语的一句话定义——线/节点/段/三级属性(线级/节点级/段级)/笔触/通道/关系/函数/预定义与自定义关系/九类关系/轨道/关键帧/控制棒/变换方框/四类非线边界/GPU 相关/命令与会话状态。
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

[English](glossary.md) | **简体中文**

# 术语表(中英对照)

## 结构原语

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 线 | Line | 有序节点序列 + 线级属性;mo2d 唯一的结构原语 |
| 节点 | Node | 线上的点:位置 + 节点级属性 + 语义标签;关节/关键帧/锚点/色标的统一抽象 |
| 段 | Segment | 相邻节点之间的部分,可直可曲;自身持有段级属性 |
| 线级属性 | Line Attribute | 整条线的默认属性(笔触/基准宽度/颜色/不透明度) |
| 节点级属性 | Node Attribute | 与线级同名的局部覆盖,段间插值,缺省回落线级 |
| 段级属性 | Segment Attribute | 每条线段的直接状态值(段类型/可见/插值/填充参与),不在段间插值 |
| 笔触 | Brush Stroke | 线级样式 + 节点级压力/宽度变化 |
| 压力 | Pressure | 节点级宽度通道(笔尖输入):宽度 = 线宽 × 压力;显式节点宽度优先(B2 落地) |
| 样式模板 | Style Template | 具名的线级属性模板;`style.apply` 把已定义字段复制到线上——切换笔触 = 应用另一个模板(B2) |
| 回落 | Fallback | 节点属性缺省时取线级默认值的规则 |
| 通道 | Channel | 可采样、可动画的标量参数序列 |
| 语义标签 | Semantic Tag | 标在线上的角色标注(bone/skeleton/track/mesh-warp…) |

## 关系原语

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 关系 | Relation | 线与线(含节点)之间的有向函数:输入线与参数 → 输出派生结构与值 |
| 函数 | Function | 关系的本质(owner:关系就是函数) |
| 预定义关系 | Predefined Relation | 标准函数库内的关系(层/群组/绑定/约束/…);集合即预定义关系 |
| 标准关系库 | Standard Relation Library | 预定义关系的全集 |
| 自定义关系 | Custom Relation | 用户/AI 定义的函数,函数体为逻辑载荷 |
| 根关系 | Root Relation | 文档级根;帧率/色彩空间等文档参数挂在它上面 |

## 预定义关系(九类)

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 层 | Layer | 组织类关系:线集合 → 有序可渲染单元(参数=可见性/不透明度/混合模式/变换) |
| 群组 | Group | 组织类关系:线集合 → 命名包;与层本质相同 |
| 归属/层级映射 | Ownership | 父子嵌套=关系映射;深度/遮挡=其排序参数 |
| 节点共享 | Node Sharing | 多条线共用同一节点(骨架折线、经纬网格交点) |
| 绑定 | Bind | 节点随骨骼线/网格线/控制棒动(蒙皮/纹理锚定/渐变挂载) |
| 约束 | Constraint | 角度/长度/相切等几何限制(静态关系) |
| 传导 | Propagate | 一条控制线驱动多条目标线(智能骨骼的推广) |
| 派生 | Derive | 输出线由输入线计算得出(中轴/包络/放样/文本→轮廓) |
| 轨道 | Track | 时间类关系:一条直线 + 关键帧节点,承载一个通道的时间序列 |
| 关键帧 | Keyframe | 轨道线上的节点:时间位置 + 动画值 |
| 模拟 | Simulate | 动态类关系:弹簧/阻尼/重力/风/碰撞/粒子,由求解器逐帧积分 |
| 弹簧 | Spring | 作用于两线之间的动态约束 |
| 粒子 | Particle | 发射线发出的有向短线段(位置+速度向量) |
| 求解器 | Solver | 执行动态关系积分的程序(逻辑载荷) |
| 外部资产引用 | External Asset Reference | 引用类关系:端点为文档外实体(file:// 路径) |

## 领域投影(线模型映射)

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 骨骼 | Bone | 2 个节点的线段(owner 规范表述) |
| 骨架 | Skeleton | 一条折线;分叉=多折线共享节点 |
| 时间轴 | Timeline | 多条含多节点的直线(=轨道族) |
| 位图网格 | Bitmap Mesh | 经纬两组交织的线,节点即纹理锚点 |
| 渐变 | Gradient | 控制棒(2 节点线段)+ 色标节点 |
| 控制棒 | Control Bar | 定义渐变方向与范围的线段 |
| 色标 | Color Stop | 控制棒上的节点:位置通道=色标位置,颜色通道=色标颜色 |
| 变换方框 | Transform Box | 层缩放旋转的操作框=4 节点闭合线 |
| 填充域 | Fill Region | 闭合线围合的区域;解释规则(非零/奇偶)属逻辑载荷 |
| 摄像机 | Camera | 一个方框线,推拉摇移=其节点动画 |

## 非线边界(四类)

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 载荷 | Payload | 挂在结构上的非几何内容(媒体/逻辑) |
| 媒体场 | Media Field | 采样场数据本身(位图像素/音频采样/视频帧);线锚定、变形、驱动它 |
| 逻辑 | Logic | 表达式/脚本/函数体/求解器/填充规则等符号与规则 |
| 会话状态 | Session State | 文档之外的编辑器状态(视图/选择集/撤销栈/剪贴板) |

## 求值与渲染

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 关系图 | Relation Graph | 全部关系构成的有向无环图 |
| 求值图 | Evaluation Graph | 关系图按拓扑序分层后的执行形态;层内可并行 |
| CPU oracle | CPU Oracle Renderer/Evaluator | 确定性真值源;GPU 实现必须与其输出一致 |
| SoA 布局 | Structure-of-Arrays | 每类属性一个数组的内存布局,GPU 友好 |
| 损伤渲染 | Damage Rendering | 只重绘被变化线条触及的 64×32 瓦片(脏矩形);与整帧重绘逐字节一致(B1 落地) |
| 反向依赖失效 | Reverse-Dependency Invalidation | 一次编辑沿关系 DAG 恰好标脏其传递下游——通道即信号(B1 落地) |
| 渐变关系 | Gradient Relation | 把目标线的节点投影到控制棒上采样色标,重着色填充或描边;色标经颜色通道可动画(B3 落地) |
| 群组关系(变换框) | Group Relation (Transform Box) | 方框线的"静止→当前"相似变换(锚点+对角)缩放/旋转其成员线;`visible` 即层可见性参数;嵌套可组合(B4 落地) |
| 模拟关系 | Sim Relation | Verlet 链 + 距离约束,按固定 240Hz 子步积分;锚点节点精确跟随其驱动轨道(B6 落地) |
| 种子重放 | Replay-from-Seed | 动态关系的确定性策略:每次求值都从 tick 0 重新积分,回擦/重放/离线导出天然一致(B6 落地) |
| 显式种子 | Explicit Seed | 粒子散布角经 splitmix64 哈希于文档携带的 seed 参数——同文档+同种子 → 同输出(B6) |
| 纹理锚点 | Texture Anchor | 节点在其所属线纹理内的 `uv`(0..1);纹理化填充沿表面插值锚点(B5 落地) |
| 按需解码 | Decode-on-Demand | 纹理在渲染首次请求时才解码,走预算缓存(默认 256 MiB);缓存状态永不影响输出像素(B5) |
| 包(.mo2d) | Package (.mo2d) | 单文件交付形态:规范 JSON 头 + `--BLOBS--` 标记行 + 按名排序、按长度定界的 blob 记录(B7 落地) |
| 二进制区条目 | Blob | 包尾部区中的一段具名字节区(如内嵌纹理);引用先解析 blob、再文件系统(B7) |
| 工程目录 | Project Directory | 解包容器形态:`document.mo2d.json` + `assets/`;与包等价——pack ∘ unpack ∘ pack 逐字节一致(B7) |
| 迁移链 | Migration Chain | 读端按版本的升级表:升 FORMAT_VERSION 恰好压入一条迁移;更新的文档被拒绝、绝不猜测(B7) |
| 碰撞间隙 | Collision Clearance | sim/粒子的参数:质点与碰撞体线段保持半径间隙(sim 链可自分离);恢复系数 `bounce` 反射粒子速度(B6 落地) |

## 交互与协议

| 术语 | 英文 | 一句话定义 |
|---|---|---|
| 舞台 | Stage | 线的世界;唯一常驻界面 |
| 命令 | Command | 引擎唯一入口的操作单元(JSON);一切操作皆命令 |
| 命令流 | Command Stream | 命令的时序记录;可回放=撤销/协作/AI 上下文 |
| 控件即线 | Controls-as-Lines | 临时控件(滑杆/渐变条/方框)由 AI 生成为线,用后即弃 |
| MCP | Model Context Protocol | AI 宿主接入引擎的 server 模式协议 |
