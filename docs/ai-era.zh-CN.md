---
type: Reference
title: "AI 时代替代审计:动画软件全部概念与环节,按 SVML 模式逐项判定"
description: 对动画软件全部概念与环节(14 个流水线阶段+16 个机制)的细致排查,以 Hypit 的 SVML 模式为参照(词锚定声明式源、时间轴降为投影、确定性编译器、agent 写源);每项给出 AI 时代替代候选或保留判定,附 mo2d 状态与路线图提案——全部待 owner 确认。
tags: [ai-era, svml, hypit, replacement-audit, timeline, authoring, intent, anchors]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-22T16:30:00Z }
sources:
  - id: hypit-components
    resource: https://hypit.ai/developers/custom-components
    title: "Hypit:Custom Components(SVML Source/Script/Selections/Moments/Recipe)"
  - id: hypit-docs
    resource: https://docs.magnific.com
    title: "(navi) Hypit/AI video docs hub"
  - id: opentrend-hypit
    resource: https://opentrend.org/project/hypit
    title: "Open Trend:Hypit 开源项目(SVML 词锚定)"
  - id: remotion
    resource: https://www.remotion.dev
    title: "Remotion:Make videos programmatically (React)"
  - id: vision-doc
    resource: /vision.zh-CN.md
    title: "本 bundle:vision §6.1 精度与表现力两层+AI 三无"
  - id: landscape-doc
    resource: /landscape.zh-CN.md
    title: "本 bundle:动画软件全景调研(概念清单 F1–F13)"
  - id: line-model-doc
    resource: /line-model.zh-CN.md
    title: "本 bundle:万物皆线(十大映射)"
  - id: relations-doc
    resource: /relations.zh-CN.md
    title: "本 bundle:关系即函数(C7/C8 事件与手势)"
  - id: mvp-doc
    resource: /mvp.zh-CN.md
    title: "本 bundle:D7 tick 时间真值/D10 手势时间线"
---

[English](ai-era.md) | **简体中文**

# AI 时代替代审计:动画软件全部概念与环节,按 SVML 模式逐项判定

owner 指令(2026-11-22):细致排查动画软件所有概念和环节，参考 hypit 用 svml 替代时间轴的方式，
看哪个环节可以用 ai 时代的概念代替。本文档就是这次排查。以下所有替代提案均为
**[推导,待 owner 确认]**——尚未实现任何一项。

## 1. 参照案例:Hypit 的 SVML(已从文档核实)

Hypit 是开源的 AI 视频创作系统。其源文档是 **SVML 文件(Structured Video Markup Language)**——
"它指明剧本(script)、媒体(media)、轨道(Tracks)和影片(Film)",分层为
Script / Timeline / Canvas / Frame / Style。已核实的机制:

1. **词锚定**:内容绑定到转写**词**,而不是秒。剧本词上的内联标签兼作绑定
   (`@hair-gel more hair gel than @trophy … @/trophy`),别处引用为
   `during={script.selection.hair-gel}`。两类锚原语:**Selections**(一段词)与
   **Moments**(一个瞬间);
2. **时间轴降级**:一条程序时间轴,只装 `{id, frameRate, frameCount}`——无片段、无轨道、无中央事件登记表(转述;见 Timing & Assembly 文档);
3. **由演出(performance)投影**:「一旦演出存在,投影即给出精确窗口,重写剧本也不会让图形失步」——Take(实际配音/表演时序)提供秒数;编译器把锚投影为窗口;
4. **重写安全**:"台词变长或变短,图形揭示跟着变";
5. **agent 写源文件**:"你的 agent 写 SVML 源并组合组件";`hypit check` 编译导入/类型/图边**不调任何外部服务**;组件发布 `.schedule`/`.program`/`.visual`;HyperFrames 逐帧绘制;
6. **可溯源**:Recipe "把每个决定追溯到拥有它的 Source、Recipe 或包"。

### 1.1 第二个参照:Remotion(2026-11-22 核对)

Remotion 是更早的"程序化视频"框架:**视频即 React/TypeScript 代码**——组件逐帧求值,headless
Chromium 逐帧截图出 MP4。2026 年现状:官方 **agent skill** 给 Claude Code/Codex("use agents,
render in bulk"),交互式 Studio 预览,开源核心+公司付费授权。

| 轴 | Remotion | Hypit |
|---|---|---|
| 源语言 | 通用代码(React/TS——图灵完备) | 受限声明文法(SVML) |
| 锚单位 | 帧号/props | **词**(Selections/Moments) |
| 第一作者 | 开发者(官方 skill 让 agent 写) | **AI 代理**(agent 就是第一作者) |
| 上限 | 近乎无限(整个 web 栈) | 窄的合成模板(字幕/B-roll/口播) |
| 编辑模型 | 改源→重渲染(Studio 预览) | 改源→重编译(预览) |
| 渲染栈 | headless Chromium 逐帧 | HyperFrames 逐帧(Chromium) |
| 授权 | 开源核心+公司付费授权 | Apache-2.0 + BYOK |

**互读**:两者独立验证了 R5(agent 写声明式源,确定性渲染);Remotion 证明通用代码能吸引开发者
生态,Hypit 证明锚到语义单元的受限文法对 agent 更安全、重写更容忍;两者都是**视频合成而非角色
动画**——无可编辑几何、无绑定、无曲线、无物理、无直接操纵(编辑=改源重渲染)。它们的共同盲区
恰是 mo2d 的生态位:确定性**文档**(线/关系/参数)+舞台展示=编辑+agent 命令流。

## 2. 模式抽象(五条规则)

| 规则 | 内容 |
|---|---|
| **R1——原语上移** | 创作原语是**意图层**(剧本词、节拍、命名事件),不是机械装置(片段/轨道/关键帧) |
| **R2——装置降级** | 机械表示变成**最小投影**(一条只剩 `{frameRate, frameCount}` 的时间轴;键/曲线/剪切点是编译产物) |
| **R3——演出供时序** | 精确窗口来自**投影时刻的时序源**(录制的 take、节拍表、一次手势)——锚在那时求解,重写时重新求解 |
| **R4——语义锚使重写安全** | 绑定到**词/节拍/命名事件**,绝不绑定裸秒——改源永不失步 |
| **R5——AI 写源,编译器确定** | agent 写声明式文件;本地确定性编译器降级之;每个决定可溯源 |

注意这强制出的切分:**创作时**(agent+编译器,可以是经验型) vs **运行时**(纯函数,永不调 AI——vision §6 一票否决)。SVML 保持同一划分:`hypit check` 本地且确定;只有"写作"是代理的。

## 3. 排查——动画软件全部概念与环节

判定键:**替换** = 旧的创作装置可以被 AI 时代概念取代;**半替** = *接口*被替代,*机制*保留(编译进线);**保持** = 必须留在线/定律里(vision §6.1:AI 三无——无精确、无物理定律、无可复现性);**已替** = mo2d 已做。

### 3.1 创作流水线阶段

| 阶段 | 旧形态 | AI 时代替代(ⓓ 提案) | 锚单位 | 判定 |
|---|---|---|---|---|
| 剧本 script | 独立文档;时序靠手 | **真值源**——Script 层(SVML 式);动画成为它的投影 | 词/segment | 替换(升格)ⓓ |
| 分镜 storyboard | 静态画板,手排时序 | AI 起草的画板,绑到剧本节拍 | 镜头号/beat | 替换 ⓓ |
| 设计 design(角色/场景/道具) | 手绘资产 | AI 起草的草稿**编译为线**(R5:文字转矢量、涂鸦转线) | 物件名 | 半替(产物必须编译)ⓓ |
| Layout/构图 | 手动摆位 | 声明式构图意图("角色在左,夕阳背景")→布局关系 | 物件名/方位词 | 替换(接口)ⓓ |
| 绑定 rig | 手动骨骼/权重/IK | **带确认检查点的自动绑定**(Mixamo/Animated Drawings 模式;vision §6) | 部位名 | 半替(接口)——已立项 |
| 动画 K帧/时序 | **时间轴**:拖播放头、摆键、调曲线 | **节拍锚+感觉词**:"在他说到 X 时跳一下,更弹"——编译器把锚降为 tick 键(D7 真值不变);时间轴 UI 变成*检查用投影* | 节拍/词/命名事件 | **替换(最大单点)** ⓓ |
| 中间帧 inbetween | 手绘 | **两个精确关键帧之间**的线条几何生成(landscape R5:曲线,不是像素) | 两端关键帧 | 半替 ⓓ |
| 缓动 easing | 曲线编辑器 | 感觉词("更弹")→命名手感集的段缓动 | 感觉词 | 替换(接口)ⓓ |
| 特效 FX | 效果堆栈,手调参数 | 命名意图("湍流的水")→sim/oscillate 关系参数(AI 选定律+参数,§6.1 后续) | 效果名 | 半替 ⓓ |
| 灯光/材质 | 灯光架,材质编辑器 | 参数=通道;AI 设参数(3d-extension 分工) | 灯光名/材质名 | 半替 |
| 渲染 render | 渲染农场(确定性) | **保持**——运行时渲染留纯代数(一票否决);离线神经渲染只能作为带种子的算子(EbSynth 先例) | — | 保持 |
| 合成/剪辑 | 轨道堆叠,剪切点 | 声明式层意图锚到剧本节拍(`during=` 的动画版) | 节拍/selection | 替换(接口)ⓓ |
| 音频/口型 | 手动音素 | 音频载荷→自动口型轨道(C4;Adobe Sensei 先例) | 音素/词 | 替换 ⓓ(C4) |
| 审阅 QA | 人眼 | 代理回放审阅+校验器(漂移报告) | 镜头号 | 增强 ⓓ |
| 发布 publish | 导出对话框 | 声明式导出配置(file-format §4 已是) | 产物名 | 已替(格式层) |

### 3.2 软件机制(创作装置)

| 机制 | 旧形态 | AI 时代替代 | mo2d 状态 |
|---|---|---|---|
| 时间轴/轨道/律表 | 秒锚定的片段+轨道+键 | **最小投影**(SVML:`{frameRate, frameCount}`)+其上的节拍锚;mo2d 已把轨道变成*线*(映射 4)——剩余一步是 tick 之上的锚层 ⓓ | 半替(界面) |
| 关键帧 keyframes | 手动摆放 | AI 写键(vision §6;已以命令落地) | 已替 |
| 缓动曲线 | 手动曲线编辑器 | 命名手感集参数+AI 选择 | 接口ⓓ |
| 图层/混合 | 堆叠+混合模式 | group 关系(B4 落地)+混合待 | 已/📋 blend |
| 符号/复用 | 符号系统 | 曲线复用(§1.0.1)+group 包 | 已替 |
| 相机 | 打关键帧的相机 | 相机=盒线+AI 写的轨道(映射 §3.1) | 机制✅/接口ⓓ |
| 状态机/交互 | 可视状态机编辑器 | **参数+respond 关系**(C7 落地);意图编译器("点了就跳")=C1 领域 | 机制✅/编译ⓓ |
| 物理 physics | 烘焙曲线或实时模拟 | **定律住在线里**:sim(种子重放)/oscillate/约束——AI 选定律+参数(vision §6.1) | ✅ 保持(定律在参数) |
| 约束/IK | 求解器编辑器 | constraint 关系(库 §1)——引擎待 | 📋 |
| 权重/蒙皮 | 权重绘制 | 线性 bind 落地;多骨骼混合待 | 部分 |
| 洋葱皮 | 鬼影帧 | 叠加线(会话层) | ✅ 机制 |
| 素材库/模板 | 素材文件夹 | 样式模板(B2)+group 包+svg.import 层 | ✅ |
| 导出/发布 | 对话框/预设 | 声明式配置(file-format §4) | ✅ |
| 预览 | 播放键 | 展示=编辑(§1.5)——预览是连续的 | ✅ |

### 3.3 不可替代的(保留集)

精确、物理定律、确定性、运行时求值——vision §6.1 的 AI 三无对参考线三有。落到环节:**渲染保持代数**(逐帧 AI=一票否决),**物理留在关系**,**运行时交互留在 respond/参数**,**tick 真值保持裸整数(D7)**——其上的每一层锚都*向下编译*为 tick,绝不取代之。

## 4. 对 mo2d 的推论(提案 ⓓ,待 owner 确认)

- **P1——节拍锚创作层**:tick 之上的 Script/Beat 层(Selections/Moments 的同类):命名锚绑定到词/节拍/事件;轨道与 respond 绑到锚;编译器在时序源(表演录制、旁白音频或节拍表)给定时把锚降为 tick 键。D7 tick 真值不变——锚是创作层的,向下编译;
- **P2——时间轴变成投影视图**:时间轴 UI 渲染编译出的键(数据上已是:轨道即线)外加锚泳道——用于检查,不用于创作;
- **P3——感觉词编译器**:"更弹/更顺"→命名手感集的段缓动(C7 词表扩展);
- **P4——剧本层**:Script 作为可导入的文档层(SVML 的 script 层),selection 作为 respond/guard 的地址——"在他的台词说到 X 时"成为一个地址;
- **P5——agent 写源纪律**:每个提案保持 R5——agent 写声明式层,`check` 本地确定,每个决定可溯源(Recipe 的类比:命令流本来就是)。

注意已落地的先例:**C8 手势线就是机制层的 Selections/Moments**——相位边沿锚到事件而非秒;P1 把锚词表从指针事件推广到剧本词/节拍。

## 5. 来源

Hypit custom-components + docs(2026-11-22 抓取);Open Trend 条目;mo2d bundle 文档(vision §6.1、landscape、line-model、relations、mvp D7/D10)。见 frontmatter `sources`。
