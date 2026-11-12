---
type: Design Document
title: "路线图:三阶段推进(2026-11-11 v4,舞台提前入 MVP)"
description: Phase A=MVP 触手 v4(AI 编译引擎执行 + A6 最小舞台=生成的视图第一例);Phase B=性能与词表铺开(渲染与失效正式化/笔触/渐变/层群组/位图网格/模拟/格式正式化);Phase C=AI 生态(关系 IR/控件即线/导入/音频/协作/wasm);六部件→阶段映射与性能指标。
tags: [roadmap, phases, mvp, long-term]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T14:41:31Z }
sources:
  - id: mvp-doc
    resource: /mvp.md
    title: "本 bundle:MVP v3(无头触手)"
  - id: vision-doc
    resource: /vision.md
    title: "本 bundle:愿景(§2 六部件、§6 分工)"
  - id: architecture-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§2.1 热路径内核+关系 IR、§3.1 BUI 借鉴)"
  - id: file-format-doc
    resource: /file-format.md
    title: "本 bundle:文件格式(八条细则)"
---

# 路线图:三阶段推进(v3)

原则:**每一步都有可视产出;每一步都用命令流验收**(一切操作皆命令,演示=命令流+输出,
可自动复验);门不过不进下一步。v3 按 [vision.md](vision.md) §2 六部件引擎定义重排:
**A 造语言与运行时,B 造视图与性能,C 长生态。**

## 六部件 → 阶段映射

| 部件(vision §2) | 落地阶段 |
|---|---|
| 1 语义语言 | A(最小词表)→ B4–B7(词表铺开) |
| 2 确定性运行时 | A(核心)→ B2(失效与并行化) |
| 3 命令协议 | A(三命令+命令流)→ C(生态扩容) |
| 4 校验器 | A(schema+确定性门)→ C1(关系 IR 的门) |
| 5 性能机器 | B1(GPU 主力+损伤渲染) |
| 6 可生成的视图 | **A6(MVP 最小舞台,owner 2026-11-11 决定提前)** → C2(控件即线) |

## Phase A:MVP——触手("AI 编译,引擎执行"+ 生成的视图第一例)

任务书与验收门见 [mvp.md](mvp.md) v4:A1 线内核(离屏)→ A2 骨架绑定 →
A3 轨道/Tick/播放 → A4 命令协议+MCP+校验器 → A5 格式+导出+离线闭环 →
**A6 最小舞台(原 Phase B1 提前:四操作,手搓与 AI 命令流同构)**。
完成定义七条:AI 经 MCP 全程建动画、重放字节相等、**离线播放一致(能力即数据)**、
round-trip 字节相等、求值吞吐、校验器在岗、**直接操纵等价**。

## Phase B:性能与词表铺开

| 项 | 内容 | 验收演示 |
|---|---|---|
| B1 渲染与失效正式化(部件 5) | GPU 主力:实例化 uber-shader(段/节点补丁=instance)、损伤渲染(tile 64×32+脏矩形)、**DAG 反向依赖失效**(通道=信号)、CPU/GPU 像素奇偶校验门 | 十万节点级 60fps;编辑视口空闲 0% CPU |
| B2 笔触样式库 | 线级 brush 15 参数族 + 节点级压力;样式=属性模板库 | 同一线切不同笔触;压感变宽 |
| B3 渐变控制棒 | 控制棒线段 + 色标节点;绑到填充 | 会变色的触手(渐变可动画) |
| B4 多层/群组 | 层/群组预定义关系 + 变换方框线 | 多角色同台;方框缩放旋转 |
| B5 位图网格 | 经纬线+纹理锚点绑定=媒体载荷;按需解码+预算缓存(参照 BUI) | 位图角色随骨架变形 |
| B6 模拟(动态关系) | 弹簧/力/碰撞/粒子;PBD/compute;确定性种子 | 触手尾巴物理拖尾+粒子 |
| B7 格式正式化 | 尾部 BLOB 区(媒体内嵌)+ 工程目录形态(解包,与包等价)+ refs 外置 + JSON Schema/校验器 + FORMAT_VERSION 迁移;规范稿照 maop-spec 体例 | 带媒体工程单文件 .mo2d 交付;解包/打包往返无损;旧版照读 |

依赖:B1 依赖 A 的 SoA 布局;B2–B6 相互独立可并行;B7(格式正式化)依赖 B5(位图网格,媒体载荷才有 BLOB 需求)。

## Phase C:AI 生态(能力即数据)

| 项 | 内容 | 验收演示 |
|---|---|---|
| C1 **关系 IR + 自定义关系生成** | 自然语言 → 关系 IR(受限中间表示,沙箱);**生成 → 过确定性门 → 入库冻结**(architecture §2.1) | "让触手跟着光标"一句话成动画,入库后离线可播 |
| C2 控件即线 | 临时控件系统(滑杆=轨道线+游标线;取色条=控制棒;变换框=方框) | AI 按需生成控件,用后即弃 |
| C3 导入器/交换 | **SVG 双向**:帧级导出(展示编辑一体的自然产物)+ 几何导入([file-format.md](file-format.md) §4 三层格式);.moho 远期选项([moho-mapping.md](moho-mapping.md) 即现成映射表,84 教程语料验收) | 导出帧 SVG 任意工具可看;导入真实 SVG/教程工程 |
| C4 音频 | 音轨层:媒体载荷+level/jump 轨道;Tick 整除 44.1k/48k 正为此 | 口型/节拍驱动动画 |
| C5 协作与版本 | 命令流合并、变更记录、引用层 | 多会话协作无冲突 |
| C6 WebAssembly | wasm 发布 + MCP over WebSocket | 网页里跑触手 demo,零安装 |

依赖:C1 依赖 A4 校验器与 B1(热路径退化);C3 的 .moho 依赖 B4(层/群组)+B5(位图网格);C4 依赖 B7(格式正式化)。

## 里程碑与性能指标

| 阶段 | 指标 |
|---|---|
| Phase A | 1 万节点 60fps(CPU);命令重放字节相等;离线播放一致 |
| Phase A6 | 手搓与 AI 命令流同构(直接操纵等价) |
| Phase B1 | 十万节点级 60fps(GPU);奇偶校验门通过;空闲 0% CPU |
| Phase C | AI 端到端(自然语言 → 可导出动画)单次成功率 = 北极星指标(目标待定) |

## 节奏条款

- 每阶段结束**回看本路线图**并修订(走 log);
- 任何新特性先问:"这是**热路径**吗?"——是,进引擎;否,交给 AI(C1 承载);
  语言本身的变更(新原语/新属性族)走 line-model/relations 的规范表述流程。

## 暂不排期

技术栈最终决策(Rust+wgpu 倾向,[architecture.md](architecture.md) §5)、产品品牌、
mo2d 与 maohou 的工程合并策略——待 owner 拍板后再入图。

**Phase D(3D 深化,远期未立项)**:扩展路径见 [3d-extension.md](3d-extension.md)——
**引擎侧基础 3D 已入 MVP v4.1**(渲染器即 3D/Vec3/三角形面/3D 场景门,AI 可命令式搭 z≠0 场景);
Phase D 只剩深化项:3D 交互(导航/工作平面/深度拾取)、灯光/材质/阴影、glTF;
触发条件待 2D 站稳后定。
