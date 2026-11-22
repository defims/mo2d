---
type: Reference
title: "渲染器调研:Rive 管线与 Grease Pencil 对照"
description: Rive 运行时与渲染器的源码级深调研(加载/advance/draw 流水线、互锁模式、覆盖编码、羽化、ore 绑定层)与 Grease Pencil v3 数据模型——给出 mo2d 的参照系分工(变形→Rive;素材/时序→GPv3;语义→Moho)与可执行借鉴清单。
tags: [renderer, rive, grease-pencil, reference, survey]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-22T17:29:52Z }
sources:
  - id: rive-runtime
    resource: https://github.com/rive-app/rive-runtime
    title: "rive-runtime(MIT © 2023 Rive——许可证已在克隆树核验)"
  - id: rive-renderer
    resource: https://github.com/rive-app/rive-renderer
    title: "rive-renderer(MIT——PLS/深度模板后端、着色器、羽化、ore)"
  - id: rive-rs
    resource: https://github.com/rive-app/rive-rs
    title: "rive-rs(MIT——rive-cpp 的 FFI 壳,2605 行,Vello 后端模块)"
  - id: gp3-design
    resource: https://code.blender.org/2023/05/the-next-big-step-grease-pencil-3-0
    title: "The Next Big Step: Grease Pencil 3.0(code.blender.org)"
  - id: gp-arch
    resource: https://developer.blender.org/docs/features/grease_pencil/architecture/
    title: "Grease Pencil Architecture——Blender 开发者文档"
  - id: gp-migration
    resource: https://developer.blender.org/docs/release_notes/4.3/grease_pencil_migration/
    title: "GP Migration to Blender 4.3(属性化存储映射)"
  - id: gp51
    resource: https://developer.blender.org/docs/release_notes/5.1/grease_pencil
    title: "Grease Pencil 5.1 说明(fill_id 分组 CDT、偶奇带洞)"
---

[English](renderer-survey.md) | **简体中文**


# 渲染器调研:Rive 管线与 Grease Pencil 对照

源码级调研(克隆位于 `/Users/wei/rive-study/`;两者均 MIT——已在树核验——依
[AGENTS.zh-CN.md](../AGENTS.zh-CN.md) 可合法参照)。Blender/GPv3 材料仅公开文档(GPL:只取设计思想,零代码)。

## 1. 为何是这两个参照——参照系分工

| 议题 | 参照 | 理由 |
|---|---|---|
| **曲线变形动画**(连续、逐节点通道、骨骼/模拟逐帧拉曲线) | **Rive** | advance→变形→渲染是它的主管线;逐帧新曲线的解析渲染是它的核心能力 |
| **素材、时间轴语义、逐帧工作流**(drawing 池、实例化、显式 hold) | **GPv3** | 它整个重构就是这个范式 |
| **变形的语义**(点通道、智能骨骼、绑定) | **Moho**,经 [moho-mapping.zh-CN.md](moho-mapping.zh-CN.md) | 最深的先源,已全量映射 |
| 逐帧手绘序列(若将来支持) | GPv3 形态 | 原生可表达:一条轨道,其关键帧节点各引用一条 drawing 线——无需换范式 |

## 2. Rive 运行时管线(源码:file.cpp / artboard.cpp / scene.rs)

```
.riv(运行时优先紧凑二进制,比同效果 Lottie JSON 小 10–15×)
  ↓ RuntimeHeader(major/minor)→ ImportStack 单遍流式解码
    (~40 个类型化 importer:artboard/状态机/keyed property/viewmodel/…)
  ↓ 文件内记录 artboard 字节区间 → 单板延迟解码
    (deferArtboardBody / decodeAnimationBody;属性表保留复用)
  ↓ WITH_RIVE_SCRIPTING_LUAU:Luau 脚本 + ShaderAsset 导入后注册
    数据绑定全家:ViewModel 实例、转换器、公式、枚举
Artboard → StateMachineInstance(输入)→ advance_and_apply(elapsed) -> bool
  → ComponentDirt 脏位 + dependencyOrderRecipe(缓存拓扑序,
    valid/unbuilt/unusable 三态;重建零分配)→ m_dirtyLayout 队列
  → draw(renderer)   // 布尔="有变化"——帧级变化检测烙在 API 合同里
```

对照 mo2d 的亮点:**artboard 字节区间表是格式内建的按场景对象懒解码**(我们的
`.mo2d` BLOB 布局兼容同一想法);脏位+配方系统与 `inval.rs`+缓存拓扑序同形;
`advance → bool` 是 API 合同级的时间增量纪律。

## 3. Rive 渲染器(源码:renderer/src + shaders)

**每帧五阶段**:

1. **命令生成**:runtime 把场景转成 `Draw` 对象——只有四种:`path | imageRect |
   imageMesh | stencilClipReset`(draw.hpp);
2. **批处理**:多条路径合并进**一次 flush**——三张 GPU 缓冲累积(`PathBuffer`:
   4×u32x4/路径——2×2 矩阵、平移、标志、描边/羽化半径;`ContourBuffer`:中点、
   pathID、首顶点、标志;`TessVertexTexture`:RGBA32UI 细分顶点);
3. **几何展开**:每轮廓一个"楔形"实例;顶点着色器做三级间接寻址(实例→细分顶点→
   轮廓→路径),见 `unpack_tessellated_path_vertex`(draw_path_common.glsl:316),
   顶点带 `outset`/`fillCoverage`,带自修正 padding 顶点逻辑,羽化带用镜像轮廓
   (`MIRRORED_CONTOUR_CONTOUR_FLAG`);
4. **覆盖求解**——五级 `CoverageType`,由 `selectInterlockMode` 选择
   (render_context.cpp:404):要 MSAA→`depthStencil`(万能兜底:`DS_POLAR_STROKE`+
   `DS_HAIRLINE_STROKE`);clockwise 覆盖→`clockwise`(光栅序硬件)或
   `clockwiseAtomic`(免屏障 atomics——Clockwise 填充规则让原子免屏障);标准→
   `rasterOrdering`(PLS);最后→`atomics`。覆盖累积进 R32UI atlas,由
   `resolve_atlas.glsl` 解析为 R8(顺带补 PLS 缺失的"clear");
5. **合成**:渐变/图像经 `PaintAuxBuffer`,混合按 Skia 编号(SrcOver=3 …
   Luminosity=28——生态互认)。

**pass 即 draw**:`renderPassInitialize`/`renderPassResolve` 与普通 draw 共用
DrawType 表(render_context.cpp:1505/2398/4105+)——整帧是一条平铺的 GPU 命令列表,
零分支(与 mo2d 的统一 DrawType 命令表结构同构)。

**矢量羽化**(招牌):不是模糊——**高斯积分查找表**(`g_gaussianIntegralTableF16`,
F16 R16F 1D array 纹理;`RIVE_GENERATE_FEATHER_LUT` 可再生),沿镜像轮廓带采样;
几何覆盖与羽化覆盖**共用一个 float4 varying**(`-2` 偏移编码,
`FEATHER_COVERAGE_THRESHOLD -1.5` 区分)——零额外 pass,羽化半径任意;羽化贴图经
skyline 矩形装箱进 feather atlas(帧内复用、自动扩容)。大羽化可选用
`featherAtlas` 覆盖;depthStencil 模式必须。

**ore**(2026):统一绑组/资源抽象,带各后端 context——`ore_context_{metal,vulkan,
d3d11,d3d12,gl,wgpu}`——**WebGPU 是一等公民**;`gpu_census.hpp` 增加渲染回放/统计
层。Rive 正把后端迁移到 ore 上。

## 4. 为什么快——管线级乘法

| 层 | 机制 | 证据 |
|---|---|---|
| 格式 | 运行时优先二进制;属性键流;板级懒解码 | file.cpp |
| 解码 | ImportStack 单遍,类型化 importer | file.cpp |
| 求值 | 脏位+缓存拓扑配方+advance/draw 分离+变化布尔 | artboard.cpp / scene.rs |
| 渲染 | tile GPU 解析覆盖(不存细分)、单 flush、羽化 LUT | shaders/ |
| 工程 | 薄绑定、全平台单一渲染器、C++ 小足迹 | rive-rs 2605 行 |

## 5. Grease Pencil v3 数据模型(公开文档)

```
GreasePencil(ID)
├── 层树根(LayerGroup;只有组含节点;层是叶子)
│   └── Layer → 独立时间线 → 关键帧按索引引用 drawing
│       (帧带 start/end=显式 hold;实例关键帧深色)
├── drawings: [Drawing] ← "无层的静态笔迹集;一张矢量图像"
│   = CurvesGeometry:扁平点数组 + curve_offsets
│   POINT 域:position、radius(粗细×压感烘入)、opacity、vertex_color、
│            rotation、delta_time、selection
│   CURVE 域:material_index、cyclic、caps、softness、curve_type
│            (POLY/BEZIER/CATMULL_ROM/NURBS)、aspect_ratio、
│            fill_color/fill_opacity、init_time
└── materials(刻意简单:填充+描边)
```

重构中删除:`triangles`(烘焙三角化)、`bounds_*`、`line_width`、`edit_curve`
——都是已存数据的纯函数,改为运行时计算(与 mo2d 的派生数据原则同一条)。
15 年的逐类型刚性结构体是债;重写收敛到"引用+属性+扁平数组"。

## 6. mo2d 借鉴清单(可执行、法务干净)

1. `advance_and_apply -> bool`——变化布尔烙进求值 API 合同(A6 舞台/B2 失效);
2. `MAPPED_ONCE_AT_INITIALIZATION` 缓冲标志——未来 GPU 资源接口的零拷贝语义;
3. `Path::extend(from, transform)`——实例化路径复用,shape 关系同曲线共享的
   渲染侧直通车;
4. 混合模式编号对齐 Skia(Lottie/SVG 互换免费);
5. GPv3 hold 盒子+实例深色关键帧——舞台时间轴 UI 语义;
6. GP 5.1 填充:按 `fill_id` 分组同色笔迹,一组一次带洞 CDT(偶奇)——我们的
   shape 关系按色批量求值;
7. GPv2→v3 实证:逐类型刚性结构体是债;引用+属性+扁平数组是 mo2d 的起点方向;
8. B1 远期可选:PLS/clockwise 覆盖路线(已记录;WebGPU 无标准 PLS 扩展——
   wgpu 实例化路线保持主选)。

## 7. 法务注记

Rive 运行时:**MIT**(克隆树核验)——实现级参照合法,搬运代码须带署名。
Blender/GPv3:**GPL**——仅公开文档、设计思想,零代码(AGENTS 红线)。
