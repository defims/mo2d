---
type: Design Document
title: "架构草案:分层、文档模型、GPU 策略与格式原则"
description: mo2d 架构草案——文档模型=扁平线集合+关系图(无刚性树);L0-L4 分层;GPU 策略专节(SoA/统一顶点格式/DAG 并行/PBD-compute/CPU oracle+损伤渲染/实例化参照 BUI);存储格式原则;技术栈倾向 Rust+wgpu/wasm(待定)。
tags: [architecture, gpu, damage-rendering, storage-format, layering]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T12:58:58Z }
sources:
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
  - id: maohou-readme
    resource: file:///Users/wei/maohou/README.md
    title: "maohou/AniCraft 工程总览(CPU oracle + wgpu 双轨、命令/MCP 范式参照)"
  - id: vectorcraft-arch
    resource: file:///Users/wei/vectorcraft/docs/architecture.md
    title: "vectorcraft 架构文档(L0-L7 分层、命令注册表、版本化 JSON 格式参照)"
  - id: bui
    resource: https://github.com/ppmpreetham/BUI
    title: "BUI(Blender UI 解耦库)——损伤渲染/实例化 uber-shader/双渲染器奇偶校验的工程参照(GPL,净室借鉴)"
---

# 架构草案

> 状态:**草案**。本文只定分层与原则,不做 crate 级设计;
> 技术栈为**倾向**而非决定(见 §5)。

## 1. 文档模型

**扁平线集合 + 关系图 + 通道表 + 载荷库 + 外部资产引用表。**

- 没有 vectorcraft 式的刚性 Node 树[^vectorcraft-arch]:层/群组只是**组织类预定义关系**,
  嵌套是关系的映射([relations.md](relations.md));
- 文档级参数(帧率、色彩空间、舞台尺寸)= **根关系**的参数;
- 数据骨架见 [line-model.md](line-model.md) §5。

与既有工程的关键差异:

| | vectorcraft / AniCraft | mo2d |
|---|---|---|
| 组织 | 刚性 Node 树(Layer/Group 容器类型) | 扁平线集合 + 关系图 |
| 领域概念 | 各自数据类型(Path/Bone/Text/…) | 全部投影为线 + 语义标签 + 关系 |
| 行为 | 特性代码各写各的 | 一切行为 = 关系(函数) |

## 2. 分层(L0–L4)

| 层 | 职责 | 说明 |
|---|---|---|
| L0 几何 | 线/节点求值、贝塞尔展平、命中测试 | 纯函数,无文档概念 |
| L1 文档 | 线集合、关系图、通道表(三级属性)、载荷库、外部引用 | 文档模型本体 |
| L2 求值引擎 | 关系 DAG 拓扑序求值、时间采样、动态积分 | 确定性核心;分层并行 |
| L3 渲染与计算 | CPU oracle 参考渲染器 + GPU(wgpu/WebGPU) | 变宽线三角带、闭合线填充细分、绑定顶点混合、compute 模拟/并行求值 |
| L4 交互 | 命令注册表、JSON 命令流、MCP server、AI 宿主 | 一切操作皆命令([ai-interaction.md](ai-interaction.md)) |

### 2.1 分工落地:热路径内核 + 关系 IR(vision §5 的实现面)

"更多能力交给 AI、引擎专注性能"的边界在工程上落地为两条:

1. **预定义关系库收缩**:引擎只保留**每帧热循环**的内置实现——蒙皮(绑定)、
   模拟求解(弹簧/粒子/碰撞)、轨道采样(插值)、渲染;其余"特性"不再写入引擎,
   改由 **AI 生成的自定义关系**承载(自动绑骨/口型同步/跟随等 = AI 在创作时
   编译出的关系网,不是引擎特性);
2. **关系 IR**:自定义关系的函数体不是解释执行的脚本,而是受限中间表示
   (沙箱 WASM 或表达式 IR):确定性可校验、可被引擎静态分析(依赖声明/纯度)、
   热路径可退化到内置实现;**AI 生成 → 过确定性门(命令重放字节相等)→ 才入库**,
   入库后即冻结为数据(重求值不再调 AI);
3. 运行时永不调 AI(一票否决,见 [vision.md](vision.md) §6);离线播放 = 纯引擎。

## 3. GPU 策略(专节)

owner:"线和线关系其实就可以最大化地利用 GPU,概念契合,提高性能。"

**核心论证:概念统一 ⇒ 数据同构 ⇒ GPU 吞吐最大化。**

1. **统一数据布局**:节点级属性 = SoA 数组(位置/宽度/颜色/值各一数组),
   直接映射顶点属性/存储缓冲,一次上传全场景;
2. **渲染**:变宽线 → 统一三角带展开;闭合线 → 填充细分;同构数据 =
   极少 draw call、零管线状态切换(对比传统软件位图/矢量/文本异构对象来回切换);
3. **求值**:关系 DAG 分层并行——同层独立关系并行,节点级通道采样是并行 map;
4. **模拟**:粒子 = 有向线段 = 实例化渲染 + compute 积分;弹簧/约束 = PBD 式并行求解;
5. **蒙皮/网格变形**:绑定 = 顶点混合,GPU 顶点阶段天然擅长。

**双轨纪律**(参照 maohou 的 CPU oracle + wgpu 双轨实践[^maohou-readme]):
CPU oracle 渲染器/求值器为**确定性真值源**,GPU 实现 = 加速器;两者输出必须一致
(逐像素/逐数值门)。MVP 期 CPU 起步、SoA 布局保 GPU 路(见 [mvp.md](mvp.md) §取舍)。

### 3.1 外部参照:BUI 的性能设计(2026-11-11 owner 询问后评估)

[BUI](https://github.com/ppmpreetham/BUI)(Blender UI 解耦出的 C 库,GPL-3.0)的
高性能设计与 mo2d 高度同构,五个**净室借鉴**点[^bui]:

1. **损伤渲染(damage rendering)**:变更只标脏自己的矩形(合并上限 24 个);
   无脏即睡(空闲 0% CPU);CPU 光栅按 64×32 tile 与上帧比较,只有损伤 tile 上屏——
   mo2d 编辑视口的呈现层照此;**求值侧的同构物 = 关系 DAG 的反向依赖失效**:
   通道变更只重算依赖子图(BUI 的 signal 单节点更新同理,"通道=信号");
2. **实例化 uber-shader**:每图元 = 单一 shader 的一个 128 字节 instance,
   整帧只有几个 instanced draw call——"万物皆线 ⇒ 数据同构 ⇒ 少 draw call"的
   **具体实现形态**:段/节点补丁 = instance,与 SoA 布局严丝合缝;
3. **双渲染器同像素**:CPU 光栅与 GPU shader 求值**同一公式**,奇偶校验门
   (172,800 像素仅 10 个差 >3/255)——双轨纪律的可行性证明与验收门形式(像素阈值奇偶校验);
4. **空闲零渲染 + 动画只在动时运行(按显示节拍)+ headless 虚拟时钟精确步进动画**——
   mo2d 确定性播放/回退测试的原语(headless 逐帧可复现);
5. **图像按绘制尺寸解码、线程外解码、内存预算缓存(默认 256 MiB)、离屏取消**——
   mo2d 媒体载荷(位图网格纹理)的管理参照。

**限制**:GPL-3.0(Blender 衍生代码)——只借鉴设计思想,**不引用实现**(避免许可证传染,
维持 MIT/Apache 倾向);GPU 后端仅 D3D11/GL3.3(无 Metal/WebGPU),实例化模式可平移到
wgpu/WebGPU 但代码不可;BUI 是控件级 UI 工具包,mo2d"舞台即界面"用不上其控件树/布局/CSS 主体。

## 4. 存储格式原则

**细则与信封草案见 [file-format.md](file-format.md)**(2026-11-11 立;本节保留原则摘要):

- JSON 信封 + 可选尾部 BLOB 区(大载荷不 base64);**.mo2d 单文件即交付包**(交付即工程);
  可选工程目录形态(解包:document.mo2d.json + assets/)与包等价;
- **时间 = Tick 裸整数**(每秒 254,016,000,000,maop 实证),fps 只是显示参数;
- **字节级确定性**:稳定字段序、正文无时间戳、原子写;
- **未知键透传 + 扩展命名空间**(glTF 式;承诺不剥外来键——Moho 教训);
- **命令历史内嵌但分离**:document 为真值,history 仅审计/回放;
- **资产外置引用**:file:// + 相对基准;preview 可选、不计入规范哈希;
- JSON Schema + 校验器;FORMAT_VERSION 逐版读端迁移(vectorcraft/AniCraft 实践);
- 扩展名:`.mo2d`(待定)。

## 5. 技术栈倾向(待定)

**倾向 Rust + wgpu/wasm(WebGPU)**,理由:
- 与既有生态(vectorcraft/maohou)的工程经验直接复用:分层纪律、命令注册表、
  MCP 协议、确定性求值文化[^maohou-readme][^vectorcraft-arch];
- wasm 目标 = 浏览器直跑,分发与 AI 宿主接入最省;
- `unsafe` 纪律与净室流程可延续。

概念文档([line-model.md](line-model.md)/[relations.md](relations.md))保持栈无关;
若换栈,只有本文 §2-§4 需要重写。

## 6. 待确认事项

- [ ] crate 划分(待 MVP 动工时定,首版可能单 crate 起步);
- [ ] 轨道线(时间轴)的求值顺序与几何求值的合流点;
- [ ] 动态关系(模拟)在 GPU 上的确定性积分方案(compute 浮点一致性)。
