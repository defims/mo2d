---
type: Risk Register
title: "法务风险登记册(2026-11-12 审计)"
description: mo2d 项目法务风险十项——GPL 隔离(BUI/Blender)、Moho 商标与"mo"命名、EULA/净室纪律、教程语料红线、格式事实表版权、参照许可证(已盘面验证 MIT/Apache)、专利未检索声明、自身许可证待定、AI 隐私远期;各附等级/缓解/红线与待办清单。
tags: [legal, risk-register, license, gpl, trademark, clean-room]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T06:50:12Z }
sources:
  - id: vc-license
    resource: file:///Users/wei/vectorcraft/LICENSE-MIT
    title: "vectorcraft 许可证(MIT OR Apache-2.0,盘面验证 2026-11-12)"
  - id: maohou-license
    resource: file:///Users/wei/maohou/LICENSE-MIT
    title: "maohou/AniCraft 许可证(MIT OR Apache-2.0,盘面验证 2026-11-12)"
  - id: bui-repo
    resource: https://github.com/ppmpreetham/BUI
    title: "BUI(GPL-3.0,含 Blender GPL-2+ 衍生代码——仅设计参照)"
  - id: naming
    resource: file:///Users/wei/maohou/plan/moho/naming-domains.md
    title: "maohou 命名决策备忘(mo 系词根封区/mo2d.com 仅辅助域名裁定)"
  - id: arch-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§3.1 BUI 借鉴红线)"
---

[English](legal-risks.md) | **简体中文**

# 法务风险登记册(2026-11-12 审计)

owner 要求仔细检查本项目法务风险。方法:逐项识别→盘面取证→等级/缓解/红线。
**取证结论先行**:vectorcraft 与 maohou(AniCraft fork)均在盘面持有
`LICENSE-MIT` + `LICENSE-APACHE`(MIT OR Apache-2.0)[^vc-license][^maohou-license]——
既有文档中"vectorcraft 代码可合法参照"的声明**成立**;mo2d 自身**无 LICENSE 文件、未建 git 仓库**。

## 1. 风险总表

| # | 风险 | 等级 | 状态 |
|---|---|---|---|
| R1 | GPL 污染(BUI/Blender 衍生代码混入) | **高**(一旦发生) | 规则已立,需入库检查项 |
| R2 | Moho 商标与 "mo" 命名邻接 | 中 | 缓解中(代号定位);产品化前必须处理 |
| R3 | Moho EULA/逆向边界(经 maohou 研究链) | 低-中 | 依赖链事实已记录;EULA 条款未审 |
| R4 | 教程语料(84 个 .moho)外流 | 低(内部) | 红线待写入工程规范 |
| R5 | 格式事实表(通道码等)的汇编版权 | 低 | 已溯源引用;公开时保留引注 |
| R6 | 参照项目许可证误读 | 已消除 | 盘面验证(2026-11-12) |
| R7 | 专利(骨骼动画/蒙皮/插值机制) | 未检索 | 诚实声明;产品化前 FTO |
| R8 | mo2d 自身许可证未定 + 无 LICENSE | 已解决 | 2026-11-12 建库落地(git+双许可+NOTICE),见 §2 R8 |
| R9 | AI 通道隐私(用户文档送第三方模型) | 远期 | 产品化时隐私政策 |
| R10 | 文档内嵌 maohou 成果 | 无 | 同一 owner 自有成果 |

## 2. 逐项分析

### R1 GPL 污染(BUI / Blender)——最高优先

- **事实**:BUI 为 GPL-3.0,且自述含 Blender GPL-2+ 衍生代码[^bui-repo];
  本项目曾参照其性能设计(损伤渲染/实例化 uber-shader/奇偶校验门,
  [architecture.md](architecture.zh-CN.md) §3.1)。
- **现状**:文档仅以自己的语言描述其设计思想并引用来源——思想/事实不受版权保护,
  风险低;**红线已在 §3.1 写明"只借设计,不引实现"**。
- **残余风险**:未来写代码时"顺手"参考 BUI 源码或逐段翻译其实现——GPL 传染,
  mo2d 若走 MIT/Apache 即破裂。
- **缓解(待办)**:①建库时在 AGENTS/CONTRIBUTING 写入 **GPL 隔离条款**
  (禁止复制/翻译 BUI、Blender、任何 GPL 代码;BUI 参照仅限公开文档);
  ②代码评审检查项:发现与 BUI/Blender 同构的非常规实现即溯源质询。

### R2 Moho 商标与 "mo" 命名——产品化前必须处理

- **事实**:maohou 命名备忘已裁定:"mo 系词根封区(mo2d/homo/mhp/morph/morpho)";
  mo2d.com 仅可作中性辅助域名、绝不做品牌——商标弱(2D 通用词 + Moho 邻接)[^naming]。
  该裁定针对 maohou 品牌,但**商标弱性分析对 mo2d 作为产品名同样成立**。
- **现状**:本项目文档已定位"mo2d 为引擎**代号**,产品品牌另议"(vision 待定项)——正确。
- **残余风险**:①若以 mo2d 上市销售,存在与 Moho 商标(Lost Marble/Smith Micro)
  混淆主张的风险;②文档大量引用 Moho 属**指称性合理使用**(描述互操作事实),
  但公开材料应避免"复刻/兼容 Moho"式营销措辞(mo2d 也不是复刻,如实即可);
  ③禁止任何暗示 Moho 背书/关联的表述。
- **待办**:产品化前做商标检索(France/US/China 视市场);品牌不落在 mo 系词根上。

### R3 Moho EULA / 逆向边界(依赖链事实,如实记录)

- **事实**:mo2d 的 Moho 知识全部来自 maohou 知识库(同 owner)的**净室研究**:
  字段表来自 8 个自造 fixture 的 JSON 实证 + mohoscripting.com 社区文档;
  但 maohou 流程含"**实机门**"(本机正版 Moho 14.4,GUI 自动化只开不存、Lua 生成
  fixture、FileRender 出图对照)——属**黑盒使用**,与反编译有本质区别;
  **Moho EULA 的逆向条款未经审阅**(本登记册不假设其内容)。
- **mo2d 侧的净室纪律(应写入工程规范)**:①消费 maohou 文档中的**格式事实**
  (字段名/默认值/通道码——事实不受版权),不引入任何 Moho 二进制分析产物;
  ②远期 .moho 导入器(C3)只依公开文档事实实现;③mo2d 自身不运行/不自动化 Moho。
- **待办(低优先)**:若 .moho 导入转为正式特性,请律师审 Moho EULA 互操作条款
  (多数法域互操作目的的格式事实使用可辩护,但应留法律意见)。

### R4 教程语料红线(84 个 .moho)

- **事实**:`moho user content/` 为 Moho 官方教程工程(受版权保护的样例内容);
  roadmap C3 计划以其作导入验收语料。
- **判定**:内部测试使用=低风险;**红线**:①永不进入任何公开仓库
  (maohou 侧同样有"corpus 入库未决"的教训);②CI fixtures 必须自造,
  不派生自教程文件;③不随 mo2d 分发。
- **待办**:写入建库后的 .gitignore 与贡献规范。

### R5 格式事实表(汇编薄版权)

- 通道码表/字段表为**事实**(数字与名称,不受版权);其**选择与编排**可能有薄版权
  (maohou 的汇编)。mo2d 文档已逐处溯源(脚注→ maohou/mohoscripting);
  本项目与 maohou 同 owner(R10),**当前无第三方风险**;若文档公开,
  保留引注即可;moho-mapping 的表格为 mo2d 自己的重新编排。

### R6 参照许可证误读——已消除

vectorcraft、maohou 均 MIT OR Apache-2.0(盘面双许可证文件,2026-11-12 取证)
[^vc-license][^maohou-license]——"vectorcraft 格式实现代码可合法参照"成立;
即使直接移植其 BLOB 区/原子写实现,与 mo2d 拟采用的 MIT/Apache 兼容(保留版权声明即可)。

### R7 专利——未检索(诚实声明)

骨骼动画/LBS 蒙皮/贝塞尔插值等基础技术多源于 1980-90 年代,核心专利大概率过期;
但**应用层专利(智能骨骼、特定变形算法、PBD 变体)未做任何检索**。
本登记册不提供专利意见;**待办**:产品化/融资节点做 FTO(freedom-to-operate)检索。

### R8 mo2d 自身许可证——**已解决(2026-11-12,owner 提供远端后落地)**

`git init`(main 分支)+ 远端 `github.com/defims/mo2d`;**LICENSE-MIT + LICENSE-APACHE
双许可**(版权署名 `defims and the mo2d contributors`,与 vectorcraft/maohou 生态一致);
根 `AGENTS.md` 写入 GPL 隔离条款、Moho 语料红线、SPDX 规则与署名政策;
`.gitignore` 排除会话产物与构建产物。行动清单 1–3 全部关闭。

### R9 AI 通道隐私——远期

产品化后,自然语言→命令的 AI 通道若将用户文档内容送第三方模型,
需隐私政策与数据流向声明;本地模型/MCP 架构可最小化。
远期待办,不阻塞工程。

### R10 文档内嵌 maohou 成果——无风险

maohou 知识库为本机同 owner 自有成果(其上游 AniCraft 为 MIT/Apache);
跨项目复用无第三方权利问题,记录在案。

## 3. 立即行动清单

| # | 行动 | 时机 |
|---|---|---|
| 1 | ~~GPL 隔离条款写入仓库 AGENTS~~ | ✅ 2026-11-12 根 AGENTS.md |
| 2 | ~~加 LICENSE~~ | ✅ 2026-11-12 MIT OR Apache-2.0 双许可 |
| 3 | ~~教程语料红线~~ | ✅ 2026-11-12 AGENTS.md §Moho 语料红线 + .gitignore |
| 4 | 公开材料措辞审查:无"复刻/兼容 Moho"、无背书暗示 | 首次公开前 |
| 5 | 商标检索 + 品牌决策(避开 mo 系词根) | 产品化前 |
| 6 | Moho EULA 互操作条款法律意见 | .moho 导入立项时 |
| 7 | 专利 FTO 检索 | 产品化/融资节点 |
| 8 | AI 隐私政策 | 产品化时 |
