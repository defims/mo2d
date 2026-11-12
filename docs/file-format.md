---
type: Design Document
title: "文件格式:设计建议与参照"
description: mo2d 原生格式设计——三层格式架构(.mo2d 单文件交付包=交付即工程/可选工程目录形态/SVG 帧视图)、尾部 BLOB 区、Tick 整数时间、字节级确定性、扩展命名空间、JSON Schema 校验与版本迁移纪律;参照表覆盖 glTF 2.0/vectorcraft/maop/Moho/Lottie/OpenRaster/AniCraft/USD/Collada,各注明取什么与避什么。
tags: [file-format, storage, reference, gltf, ticks, svg-view]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T15:56:02Z }
sources:
  - id: gltf
    resource: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html
    title: "glTF 2.0 规范(Khronos)——asset.version/扩展机制/JSON+BIN 分离/官方校验器"
  - id: vc-format
    resource: file:///Users/wei/vectorcraft/crates/format/src/lib.rs
    title: "vectorcraft .vectorcraft 格式实现(信封/尾部 BLOB 区/原子写/未知键透传;MIT/Apache 可合法参照代码)"
  - id: maop-spec
    resource: file:///Users/wei/maohou/plan/moho/maop-spec.md
    title: ".maop 规范 v1(Tick 时间模型/键升序与读端修复/时间钳制)"
  - id: mohoproj-format
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 格式规范(双形态/保存剥外来键的教训/documentviewstate 混入文档的反例)"
  - id: lottie
    resource: https://lottie.github.io/lottie-spec/
    title: "Lottie(JSON 动画互换格式)——关键帧 JSON 编码的工业对照"
  - id: ora
    resource: https://www.openraster.org/
    title: "OpenRaster(ORA)——ZIP 容器纪律(mimetype 首条未压缩/合成预览/层文件)"
  - id: anicraft
    resource: file:///Users/wei/maohou/README.md
    title: "AniCraft .anicraft FORMAT_VERSION v4→v7 逐版读端迁移的实践样本"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: arch-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§4 存储格式原则)"
---

# 文件格式:设计建议与参照

owner 问:文件格式有什么推荐与建议,哪些项目和格式规范可以参考。
本文给出推荐设计(信封草案)+ 参照表(每个参照注明**取什么、避什么**),
并把 [architecture.md](architecture.md) §4 的五条原则落成可执行细则。

## 1. 需求(由 mo2d 模型决定)

格式要承载:线集合(节点/段/三级属性)、关系图(预定义+自定义,含函数体载荷)、
通道与轨道、根参数、媒体/逻辑载荷、外部资产引用、命令历史。约束:

1. **AI 可读可写**:AI 原生引擎——格式是智能体的"源代码",须语义命名、可 diff、可手改;
2. **确定性**:同一文档 → 同一字节(便于 git diff、命令回放哈希、双渲染器对照);
3. **版本演进**:加字段不改读端,升版本有迁移路;
4. **大载荷**:十万节点级 SoA 数组与位图不能被 base64 撑爆;
5. **git 友好**:文本优先,二进制后置或外置。

## 2. 推荐设计(mo2d 信封草案)

**主格式 = `.mo2d` 单文件交付包**(UTF-8 JSON 头 + 可选尾部 BLOB 区,媒体内嵌不走 base64)
——**交付即工程**:一个文件既自包含可双击交付,JSON 头又可 git diff、AI 直读。
**可选工程目录形态**("解包"):`project/` 目录 = `document.mo2d.json` + `assets/`(refs 外置),
供重 git 协作/大体量工程;引擎打开两种形态**等价**(同一文档,两种容器)。
对应通行惯例:主扩展名=交付包(Moho .moho=ZIP、.kra/.ora/.docx 同;
mo2d 因尾部 BLOB 设计,包比 ZIP 更 git/AI 友好)[^mohoproj-format]。
~~.mo2dz~~ 不设(多场景打包需求出现再议)。

```jsonc
{
  "format": "mo2d",
  "version": 1,               // 文档格式版本(整数递增;语义化标签另议)
  "minReader": 1,             // 低于此版本的读端应拒绝(glTF asset.minVersion 式)[^gltf]
  "generator": "mo2d 0.1.0",
  "root": {                   // 根关系参数(画布/ticksPerSecond/色彩/导出)
    "width": 1920, "height": 1080,
    "ticksPerSecond": 254016000000,   // maop 实证值[^maop-spec]
    "fps": 24.0                       // 仅显示/导出参数,时间真值是 Tick
  },
  "lines":    [ { "id": "l1", "tags": ["bone"], "attrs": {...},
                 "nodes": [ {"p": [x,y,z], "attrs": {...}} ],   // 2D 恒 z=0(Vec3 从第一天,3D 扩展决议)
                 "segAttrs": [ {...} ] } ],
  "relations":[ { "id": "r1", "kind": "predefined", "type": "bind",
                 "inputs": [...], "params": {...}, "body": {...}? } ],
  "payloads": { "media": {...}, "logic": {...} },
  "refs":     [ { "id": "img1", "path": "assets/a.png", "relativeTo": "file" } ],
  "history":  [ /* 命令流,可选;可拆 sidecar */ ],
  "extensions": { "vendor.feature": { ... } }   // glTF 式扩展命名空间[^gltf]
}
```

八条细则:

1. **时间 = Tick 裸整数**(每秒 254,016,000,000 = 504,000²,一 tick ≈ 3.9 皮秒;
   ≈±105 天,越界钳制并记修复;键必须时间升序,读端对乱序排序并记修复)
   ——全盘照 maop 实证[^maop-spec],避免浮点帧漂移;fps 只是根参数。
   **选此数的理由**(2026-11-11 计算验证):24/25/30/48/50/60/120/240 fps、
   毫秒/微秒、NTSC 30000/1001(=8,475,667,200 精确)、音频采样率 44.1k/48k/96k/192k
   全部整除——帧↔tick↔音频采样换算皆为精确整数运算;值域钳在 i64/4(±105 天)
   留 2 bit 余量,时间差/插值中间运算不溢出;
2. **尾部 BLOB 区**(vectorcraft 式):JSON 头之后接魔数与字节区,载荷条目写
   `"blob": [offset, length]` 而非 base64;小额(阈值 INLINE_MAX)仍内嵌——
   vectorcraft 是 MIT/Apache,**连代码都可合法参照**[^vc-format];
3. **媒体默认外置**(refs + 相对基准),内嵌可选;preview 缩略图可选、
   保存时再生、**不计入规范哈希**(Moho 的 thumbnail/preview.jpg 同此)[^mohoproj-format];
4. **字节级确定性**:稳定字段序(写入按规范序,不排序不压缩)、正文无时间戳
   (modified_date 之类只进信封)、原子写(temp + rename)[^vc-format];
5. **未知键透传 + 扩展命名空间**:读端不认识的键原样保留写回;
   第三方扩展一律进 `extensions.{vendor.feature}`,并像 glTF 一样在
   `extensionsUsed` 声明[^gltf]——吸取 Moho **保存时剥离外来键**的教训:
   mo2d 明文承诺**不剥**[^mohoproj-format];
6. **JSON Schema + 校验器**:格式规范先行(OKF 文档,照 maop-spec 的写法),
   仓库带 schema 与校验命令(glTF validator 文化)[^gltf];
7. **版本迁移读端**:FORMAT_VERSION 逐版升,旧版照读——AniCraft v4→v7 是
   活样本(每升一版,旧读端保留)[^anicraft];
8. **history 与 document 分离**:document 是**真值**,history 只是审计/回放;
   两者分叉时以 document 为准(避免"回放 vs 状态"两套真相)。

## 3. 参照表:项目与规范

| 参照 | 许可 | 取什么 | 避什么/教训 |
|---|---|---|---|
| **glTF 2.0**(Khronos)[^gltf] | 规范开放 | `asset.version/minVersion` 必填纪律;extensionsUsed/Required 机制;JSON 头 + BIN 数据分离;官方 JSON Schema + validator 文化 | 它的 3D 概念体系(accessor/bufferView 间接层)对 mo2d 过重——用简化的尾部 BLOB 区即可 |
| **vectorcraft `.vectorcraft`**[^vc-format] | MIT/Apache(**代码可合法参照**) | 信封形状;尾部 BLOB 区(魔数+offset/length+INLINE_MAX 阈值);原子写;`document.unknown` 外来数据位;v1→v4 逐版演进 | — |
| **maohou `.maop` 规范**[^maop-spec] | 本机知识库 | **Tick 时间模型**(裸整数/每秒 254,016,000,000/钳制修复);键时间升序;规范文档先行、逐字段必写/不写标注法 | 它的双写扩展键(标准+扩展)是为迁就 Moho 剥键——mo2d 无此包袱,直接单写 |
| **Moho `.moho/.mohoproj`**[^mohoproj-format] | 事实参照(不引代码) | 双形态(ZIP 容器 + 单文件 JSON)并存;preview 外置/内嵌两可;读端宽松(未知键容忍) | ①保存**剥离外来键**;②编辑器状态(documentviewstate 48 键/洋葱皮)混进文档——mo2d 已划会话状态,格式只带文档;③空层列表被拒收之类的容器怪癖 |
| **Lottie**(JSON 动画)[^lottie] | 规范开放 | 关键帧/插值/预合成的 JSON 编码工业对照;若将来要 Web 互通,是最现实的交换目标 | 无严格 schema、概念历史堆积;其"形状=图层树"与线模型不合,只借鉴编码不借鉴模型 |
| **OpenRaster(ORA)**[^ora] | 规范开放 | ZIP 容器纪律:mimetype 为首条目且不压缩、mergedimage.png 预览、层文件分立——将来若设打包形态照此办 | — |
| **AniCraft `.anicraft`**[^anicraft] | 本机(fork) | FORMAT_VERSION 逐版读端迁移 + 旁存(sidecar)承载兼容数据 | — |
| **USD** | 规范开放 | 分层组合/引用(若远期做协作组合可再看) | 过重——mo2d 的引用需求用 refs 关系即可,现在不引入 |
| **Collada / .blend DNA** | — | (.blend 的自描述版本化是好想法) | Collada=XML 过度工程的教训;.blend 二进制自描述与"AI 可读"冲突——都是反面参照 |
| **BUI**(GPL)[^arch-doc] | **GPL-3.0** | (性能设计另见 architecture §3.1) | 格式上无参照价值;GPL 不可引代码 |

## 4. 三层格式架构(2026-11-11,owner 问"能否直接用 SVG 表示工程"后确立)

**展示编辑一体成立**(同一渲染管线的多个客户端:舞台/无头导出/AI 会话皆渲染同一文档,
叠加物也是线),但**工程真值不能是 SVG**——SVG 是"某一帧的图像",工程是"全部帧的成因":

```
① 主格式(交付即工程) .mo2d           单文件:JSON 头+尾部 BLOB 区(媒体内嵌);日常保存/分享/交付
② 工程目录形态(可选)  project/        document.mo2d.json + assets/(refs 外置);重 git 协作;两种容器等价
③ 帧视图               SVG            任一帧的无损展示;模型即线 ⇒ 原则上可重导入为几何线(关系/轨道不随行)
```

SVG 作工程格式的五个硬伤:①stroke-width 每 path 单值,**无逐节点宽度通道**(变宽线需
展开为填充轮廓,结构丢失),更无逐点颜色/段级属性;②path `d` 是指令串非节点表——
节点 ID/手柄/段边界不可往返,而绑定/轨道/命令靠**节点 ID 寻址**;③无骨骼/绑定/约束/
轨道——文档模型三分之二无处安放;④无时间模型(SMIL 已废弃);⑤XML 无规范字节形态,
字节级确定性做不到,大场景解析慢。
**SVG 的正确定位**:帧级视图/交换格式(导出=展示一体的自然产物;LLM 极熟悉 SVG,
亦可作 AI 的几何交换通道)——roadmap C3 的 SVG 导入即此角色,另加帧导出。

## 5. 与既有文档的关系

- [architecture.md](architecture.md) §4 的五条原则(信封/语义版本/未知键透传/命令历史内嵌/资产外置)本文落成八条细则,若有出入以本文为准并回改 §4;
- 轨道/关键帧/段的编码语义见 [moho-mapping.md](moho-mapping.md) §5(通道对象与 interp 字段族);
- 格式规范正式稿(逐字段"必写/默认/不写"标注,照 maop-spec 体例)在 roadmap Phase A5
  动笔,MVP 只需实现本文信封的最小子集(lines/relations/root/refs,不带 BLOB 区)。

## 6. 待确认事项

- [ ] `version` 用整数递增(AniCraft 式)还是语义化字符串(glTF 式)——倾向整数+变更日志;
- [ ] BLOB 区魔数与 INLINE_MAX 阈值的具体值(可直接采纳 vectorcraft 常数);
- [ ] ~~`.mo2dz` 是否首版就要~~(已决:**不设**,2026-11-12 owner 按通行惯例修正——主扩展名即交付包);
- [ ] 自定义关系的函数体(logic 载荷)在格式中的位置:document 内 vs refs 外置脚本。
