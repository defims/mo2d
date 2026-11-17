---
type: Reference
title: "动画软件全景调研:传统与 AI 工具及概念"
description: 对 20+ 款传统与 AI 时代动画工具的第一性原理调研——范式、概念清单、本质/偶然而判定、新手学习成本证据、AI 工具的五维可控性判定;提炼动画创作的不可约内核,支撑生活概念层(vernacular.md)。
tags: [landscape, survey, first-principles, essential-incidental, ai-tools, learning-cost, interactivity]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-17T07:26:46Z }
sources:
  - id: adobe-animate
    resource: https://helpx.adobe.com/animate/desktop.html
    title: "Adobe Animate Help(Tweens 图解/动画 TOC)"
  - id: wikibooks-flash
    resource: https://en.wikibooks.org/wiki/Beginner%27s_Guide_to_Adobe_Flash/Symbols/Movie_Clip
    title: "Wikibooks:movie clip 与 graphic symbol 的混淆"
  - id: adobe-ae
    resource: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-basics/animation-basics.html
    title: "After Effects:Animation basics"
  - id: school-motion
    resource: https://schoolofmotion.com/blog/how-to-learn-after-effects
    title: "School of Motion:学习 AE 需要多久"
  - id: toonboom
    resource: https://docs.toonboom.com/help/harmony-22/premium/nodes/about-peg-node.html
    title: "Toon Boom Harmony 文档(peg/node view/deformers)"
  - id: reddit-harmony
    resource: https://www.reddit.com/r/ToonBoomHarmony/comments/1ky60pj/how_do_you_organize_your_nodes_node_hell
    title: "r/ToonBoomHarmony:node hell 与学习曲线"
  - id: opentoonz
    resource: https://opentoonz.readthedocs.io/en/latest/creating_movements.html
    title: "OpenToonz 文档(xsheet/columns/pegbars)"
  - id: tvpaint
    resource: https://doc.tvpaint.com/docs/animation-first-steps/instances/instance-concept
    title: "TVPaint 手册(instances/exposure cells)"
  - id: krita
    resource: https://docs.krita.org/en/user_manual/animation.html
    title: "Krita 手册:Animation"
  - id: dreams
    resource: https://help.procreate.com/dreams/handbook/interface-and-gestures/timeline
    title: "Procreate Dreams Handbook(timeline/Perform/flipbook)"
  - id: moho-features
    resource: https://moho.lostmarble.com/pages/features
    title: "Moho 官方特性页(bones/smart bones/switch/physics)"
  - id: pivot
    resource: https://pivotanimator.net/Pivot_Animator_Help_4-1.pdf
    title: "Pivot Animator Help(Add Frame)"
  - id: scratch
    resource: https://en.wikipedia.org/wiki/Scratch_(programming_language)
    title: "Wikipedia:Scratch(积木式编程)"
  - id: spine-guide
    resource: https://en.esotericsoftware.com/spine-user-guide
    title: "Spine User Guide(bones/slots/attachments)"
  - id: spine-constraints
    resource: https://en.esotericsoftware.com/spine-constraints
    title: "Spine:Constraints(IK/transform/path/physics)"
  - id: live2d-glossary
    resource: https://docs.live2d.com/en/cubism-editor-manual/glossary
    title: "Live2D Cubism 手册词汇表(parameter/keyform/deformer)"
  - id: live2d-auto
    resource: https://docs.live2d.com/en/cubism-editor-manual/auto-generation-of-deformer
    title: "Live2D:自动生成变形器(建议人工校验)"
  - id: rive-sm
    resource: https://rive.app/docs/editor/state-machine
    title: "Rive:State Machines(states/transitions/layers)"
  - id: rive-listeners
    resource: https://rive.app/docs/editor/state-machine/listeners
    title: "Rive:Listeners(指针/键盘条件→动作)"
  - id: rive-data
    resource: https://rive.app/docs/editor/data-binding
    title: "Rive:Data Binding(view models)"
  - id: lottie-spec
    resource: https://lottie.github.io/lottie-spec/latest/
    title: "Lottie 格式规范(只烘焙关键帧)"
  - id: unity-animator
    resource: https://docs.unity.com/en-us/engine/current/manual/animation-section/animation-mecanim/animation-animator-controller/class-animator-controller
    title: "Unity:Animator Controller(states/parameters/blend trees)"
  - id: godot-tree
    resource: https://docs.godotengine.org/en/stable/tutorials/animation/animation_tree.html
    title: "Godot:AnimationTree(blend spaces/state machine)"
  - id: dragonbones
    resource: https://dragonbones.github.io/en/animation.html
    title: "DragonBones 官网(armature/slot/skin)"
  - id: creature
    resource: https://www.kestrelmoon.com/creaturedocs/Tutorials_And_Walkthroughs/Quickstart.html
    title: "Creature 文档(auto-mesh/weights/motors)"
  - id: runway-gen4
    resource: https://runway.com/research/introducing-runway-gen-4
    title: "Runway Gen-4(prompt+参考图→视频)"
  - id: luma-ray32
    resource: https://lumalabs.ai/news/introducing-ray-3-2
    title: "Luma Ray3.2(单片段最多 16 个关键帧)"
  - id: kling-brush
    resource: https://kling.ai/quickstart/how-to-animate-image-parts
    title: "Kling:Motion Brush(涂选区域+画运动路径)"
  - id: sora-features
    resource: https://www.maginative.com/article/six-top-features-of-sora-openais-new-ai-video-creation-platform
    title: "Sora 特性综述(storyboard/remix/recut)"
  - id: gen-video-limits
    resource: https://morphic.com/resources/how-to/ai-video-troubleshooting
    title: "生成视频的失败模式:identity drift 等"
  - id: deepmotion
    resource: https://www.deepmotion.com/saymotion
    title: "DeepMotion SayMotion(文本→骨骼动画 FBX/GLB/BVH)"
  - id: mixamo
    resource: https://helpx.adobe.com/creative-cloud/help/mixamo-rigging-animation.html
    title: "Adobe Mixamo(自动绑定+动作库)"
  - id: cascadeur
    resource: https://cascadeur.com/blog/general/ai-assistance-in-3d-character-posing-how-it-works-in-cascadeur
    title: "Cascadeur(AutoPosing/AutoPhysics)"
  - id: sensei-lipsync
    resource: https://blog.adobe.com/en/publish/2019/11/07/animate-characters-with-auto-lip-sync-powered-by-adobe-sensei
    title: "Adobe Sensei:Animate 自动口型"
  - id: roto-brush
    resource: https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/roto-brush-refine-matte.html
    title: "AE Roto Brush(AI 传播+人工精修)"
  - id: ebsynth
    resource: https://ebsynth.com
    title: "EbSynth(例图传播,确定性)"
  - id: tooncrafter
    resource: https://arxiv.org/html/2405.17933v1
    title: "ToonCrafter:生成式卡通中间帧"
  - id: animated-drawings
    resource: https://ai.meta.com/blog/ai-dataset-animation-drawings
    title: "Meta Animated Drawings(涂鸦→自动绑定)"
  - id: spline-ai
    resource: https://spline.design/ai-generate
    title: "Spline AI(prompt→四选一→落入可编辑场景)"
  - id: figma-ai
    resource: https://www.figma.com/ai
    title: "Figma AI / Figma Make(prompt→可编辑设计稿)"
  - id: motiff
    resource: https://motiff.com
    title: "Motiff(文本/线框/截图→可编辑界面)"
  - id: v0-loop
    resource: https://lovable.dev/guides/lovable-vs-v0
    title: "v0/Lovable:prompt→制品精修循环与版本回退"
  - id: mdm
    resource: https://arxiv.org/abs/2209.14916
    title: "Motion Diffusion Models(ICLR 2023)"
  - id: line-inbetween
    resource: https://arxiv.org/abs/2309.16643
    title: "Deep Geometrized Cartoon Line Inbetweening(线条几何派)"
---

**English** | [landscape.md](landscape.md)

# 动画软件全景调研:传统与 AI 工具及概念

会话目标(owner,2026-11-17):严格按照第一性原理,深入调研现有传统及 AI 动画软件各种工具、概念,从 0 规划 mo2d——最终愿景是,小白用户不用理解各种专业术语,仅凭生活常见基础概念结合 AI 就能制作、调整、播放**交互**动画,且额外学习成本做到最少、可控性熟悉度最高。规范表述收录于 [vernacular.zh-CN.md](vernacular.zh-CN.md) §0;本文档是该目标的调研半边。

## 0. 目的与方法

- **目的**:在全行业范围提取 (a) 完整的用户可见概念清单;(b) 其中哪些是**本质的**(任何"让东西随时间变化 / 响应输入"的系统都需要)vs **偶然的**(历史/GUI/架构残留);(c) 有据可查的学习成本;(d) AI 时代工具对小白创作的证明。产出支撑 [vernacular.zh-CN.md](vernacular.zh-CN.md)(生活概念层)与 roadmap;
- **方法**:逐工具——一句话范式;概念清单;本质/偶然判定;学习成本证据。AI 工具另按五个**可控性维度**打分:**E** 可编辑(能改局部而不推翻重来)、**D** 确定性(同输入→同输出)、**I** 增量调整(不掷骰子)、**C** 可组合(与其他内容混合)、**X** 可导出为可编辑格式;
- **范围**:三族——传统时间轴/逐帧工具(§1)、绑定与交互工具(§2)、AI 时代工具(§3)。Moho 深度覆盖在 [moho-mapping.zh-CN.md](moho-mapping.zh-CN.md)(此处 §1.5 仅存根)。

## 1. 传统时间轴/逐帧工具

### 1.1 Adobe Flash / Animate[^adobe-animate]

**范式**:图层式矢量时间轴;舞台上摆可复用的**元件**,用**补间**在关键帧之间动画;历史上与交互运行时(Flash Player → HTML5)耦合。

**概念**:舞台 / 时间轴 / 图层+文件夹 / 帧-关键帧-空白关键帧 / 逐帧 / 洋葱皮 / 元件(图形·按钮·影片剪辑)/ 库+实例 / 嵌套时间轴 / 传统补间 / 补间动画(tween span、属性关键帧)/ 形状补间 / 运动路径 / Motion Editor / 缓动预设+自定义 / 动画预设 / 骨骼工具(armature、IK)/ 遮罩层 / 运动引导层 / 场景 / 摄像机 / 滤镜+混合 / 3D 旋转 / 文本 / 音视频 / ActionScript+代码片段 / 组件 / 发布目标。

**判定**:本质——时间轴、关键帧、补间、缓动、洋葱皮、图层、复用、运动路径、骨骼(选学深度)。偶然——图形/按钮/影片剪辑分类法(Flash Player 运行时残留:影片剪辑自带独立时间轴,在主时间轴上拖动不播放——有据可查的小白陷阱[^wikibooks-flash])、场景、ActionScript/发布机器,以及**两套补间系统并存**(传统+新),Adobe 自己都称其为过渡性质。

**学习成本**:仅手册的动画部分就要教约 20 个主题(三种补间、Motion Editor、运动路径、骨骼工具、遮罩……)才触及交互[^adobe-animate]。

### 1.2 Adobe After Effects[^adobe-ae]

**范式**:图层+合成(synthetic composition)的合成/动画:每个可动画属性都是时间的函数(关键帧或表达式),经嵌套合成的渲染顺序 DAG 求值。

**概念**:项目+素材 / 合成 / 图层(素材·纯色·形状·文本·空对象·摄像机·灯光·调整)/ 秒表(通用的"此属性随时间变化"开关)/ 关键帧 / 插值(线性·贝塞尔·保持)/ 图层条 vs 图表编辑器(数值图+速度图)/ 表达式(JavaScript)+Pick Whip / 父子级+空对象 / 预合成 / 渲染顺序 / 遮罩+轨道遮罩 / 形状图层+路径操作 / 文本动画师 / Puppet 图钉 / 时间重映射 / 效果+动画预设 / 跟踪+抠像+Roto / 3D 图层 / 渲染队列。

**判定**:本质——合成即场景、图层、逐属性关键帧、插值、图表编辑器、缓动、父子级、遮罩、预设。偶然——预合成作为渲染顺序补丁、表达式即 JS、**两套独立透明系统**(遮罩 vs 轨道遮罩)、时间重映射、素材制作工具、图层条/图表的双重心智模型。

**学习成本**:社区共识≈基础数周、胜任 6–8 个月、精通数年;新手帖称之为"所有剪辑软件里最陡的学习曲线"[^school-motion]。

### 1.3 Toon Boom Harmony[^toonboom]

**范式**:专业剪影(-cut-out)绑定,**一个模型两套 UI**——动画用时间轴/xsheet,结构与合成用**节点图**。

**概念**:绘图层(线稿/色稿)/ 时间轴·xsheet / 绘图替换 / **peg**(变换容器;peg 层叠成绑定层级,master peg 根节点)/ 父子网络 / 节点视图(Drawing·Peg·Composite·Cutter·效果·Camera)/ Cutter 节点 / 变形器(Curve·Envelope·Bone,成链成组)/ IK / 函数曲线 / 洋葱皮 / 模板+库 / 摄像机 / 游戏绑定导出。

**判定**:本质——绘图层、替换、peg 即变换层级(≈ AE 父子级)、变形、关键帧/缓动、洋葱皮。偶然——节点视图作为绑定主界面(合成器血统)、Cutter 作为独立节点(它只是遮罩)、时间轴 vs 节点双视图本身、变形器分组规约。

**学习成本**:绑定能力被编排为 3 门以上官方进阶课程;社区帖记录"node hell"与"打赢与 Harmony 的战斗"[^reddit-harmony]。

### 1.4 逐帧族——OpenToonz · TVPaint · Krita · Procreate Dreams

- **OpenToonz**[^opentoonz]:纸质管线的数字化仿真——xsheet(单元格网格)、列、level、绘图+曝光、**pegsbar**(层级,来自实体动画台)、摄像机+台面、sub-xsheet、中间帧生成、Plastic 网格变形、FX 示意图(节点图)、函数编辑器、扫描/清理/上色。本质:xsheet/列/曝光(逐张绘图的节奏)、pegsbar、中间帧、洋葱皮。偶然:纸时代命名(pegbar/台面)、扫描管线、FX 节点图。手册 557 页;社区常把新手引向"更易用"的 Tahoma2D 分支。
- **TVPaint**[^tvpaint]:栅格逐帧组织为**实例**(图像容器)+头单元格+曝光单元格(拖拽延长停留)、auto-break 实例、透光台、FX 堆栈、木偶层、时间表。本质:层/图像/实例/曝光(经典的"停 N 帧")+透光台。偶然:头 vs 曝光的编辑语义泄漏进 UX、FX 堆栈顺序、George 脚本。
- **Krita**[^krita]:绘画程序+逐帧模式——时间轴 dock、复制帧 vs 空白帧、停留帧、帧颜色标签、洋葱皮 dock、动画曲线 dock(标量关键帧,比如挂在 Transform Mask 上——补缀式补间)、故事板 dock、内存管理指引(栅格架构把成本泄漏给用户,手册花数页讲)。本质:帧、洋葱皮、帧率。偶然:遮罩变形补间 workaround、内存管理本身。
- **Procreate Dreams**[^dreams]:平板视频编辑器,**同一时间轴三种模式**——Compose(编排/修剪)、**Perform(实时录制手势→关键帧轨道)**、Keyframe(手动),外加 flipbook 绘图轨道+洋葱皮、缓动、音频、分组/遮罩。本质:时间轴、关键帧、flipbook,尤其**表演录制**(先捕获后编辑——真正另类的创作模型)。偶然:三模式拆分只是触屏适配、"theater"命名。

### 1.5 Moho(存根)[^moho-features]

矢量骨骼剪影绑定+点级动画;**Smart Bones**(转动骨骼→任意修正动画)、**Switch Layers**(切换图形状态——口型/姿势)、物理/粒子次级运动。完整映射:[moho-mapping.zh-CN.md](moho-mapping.zh-CN.md)。

### 1.6 大众级入门工具——Pivot · Scratch

- **Pivot Animator**[^pivot]:摆弄关节火柴人→"Add Frame"→重复。概念总数≈5(角色、关节、帧、加帧、播放);插值存在但是隐式的——用户永远不用学这个词。证明**不到 6 个概念就够做出动画**。
- **Scratch**[^scratch]:积木式编程;动画=循环脚本切换**造型**("next costume" 积木),`wait`/`repeat`/`forever` 当时钟,`glide N secs` 内置缓动补间;事件/广播编排场景;克隆生成副本。时间被编排为程序控制流而非时间轴。证明**语法消失时,生活机制(画面+耐心)可以顶替关键帧机器**。

## 2. 绑定与交互工具

### 2.1 Spine[^spine-guide][^spine-constraints]

**范式**:给**骨骼**层级打关键帧;图像经**槽(slot)**附着(槽控制显示哪个附件+绘制顺序);一套绑定运行时廉价回放。

**概念**:骨架 / 骨骼 / 槽 / 附件(region·mesh·包围盒·裁剪·路径·点)/ 网格+权重(多骨骼平滑变形)/ 皮肤(skins,命名附件集——换装)/ 约束(IK·transform·path·physics·sliders;各带 mix% 与应用顺序)/ 动画+关键帧+曲线(dopesheet、图表)/ 事件(命名时间标记→宿主)/ 运行时(Unity/Unreal/Cocos/PixiJS…)。

**交互**:文件内没有——宿主挑选并交叉淡化动画;事件是唯一契约。**判定**:本质——骨骼层级、槽/附件间接层、关键帧+曲线、事件;可推导——IK、网格/权重;偶然——皮肤、path/point 附件、物理约束。

### 2.2 Live2D Cubism[^live2d-glossary][^live2d-auto]

**范式**:把分层插画绑定进**参数空间**——每个参数值映射到存好的**关键形(keyform)**(变形);姿势*就是*参数向量;无 3D 模型的"2.5D"。

**概念**:ArtMesh+网格 / 变形器(弯曲·旋转;嵌套层级)/ 部件 / **参数**(范围+默认+滑杆)/ **键**·关键形 / XY 参数链接(3×3 关键形网格)/ 绘制顺序(0–1000)/ 蒙皮 / **物理**(钟摆输入→输出链,运行时计算,不打关键帧)/ motion 文件(随时间驱动*参数*,而非形状)/ motion-sync(口型)/ 模板 / hit areas。

**决定性一步**:**姿势与时间解耦。** 绑定建立 `参数→变形`;motion 只是把参数值随时间动画化的薄脚本;面部跟踪或物理可以实时驱动同一套绑定——时间轴是可选的,参数空间不是。VTuber 生态因此长在 Live2D 上。

**判定**:本质——参数+关键形、ArtMesh/网格、变形器、绘制顺序。偶然——部件、物理、蒙皮、motion 文件本身。注:官方文档明言自动生成的网格/变形器需人工检查[^live2d-auto]——**AI 提案、人工校验**是官方工作流。

### 2.3 Rive[^rive-sm][^rive-listeners][^rive-data]

**范式**:状态机直接编*进设计文件*,时间轴、绑定与交互逻辑一起交付,运行时对输入作出响应而无需播放代码。

**概念**:画板(可嵌套、组件、布局)/ 骨骼+关节(3 种绑定:父变换 / 路径顶点权重 / 网格权重)/ 约束(IK·距离·平移·旋转·缩放·变换·路径跟随)/ 时间轴 / **状态机**——状态(动画·混合)+Entry/Any State/Exit+过渡(条件·退出时间)+并行层 / **输入**(布尔·数值·触发——旧版,已弃用)/ **监听器**(指针 enter·exit·move·down·up·drag·click、键盘、手柄、焦点;动作:设置输入/视图模型属性、对象移向指针、上报事件)/ 事件(打开 URL·音频·通用;从时间轴/状态/监听器发出)/ **数据绑定**(视图模型+实例;绑定把数据与属性双向相连——"绑定后层级不再重要")。

**交互**:完整闭环在文件内——监听器(感知)→输入/视图模型条件(决策)→状态切换+交叉淡化(响应)→事件回宿主。**判定**:本质——画板、时间轴+关键帧、状态机、一个符号化输入通道、监听器;偶然——骨骼/约束(选学深度)、URL/音频事件、嵌套画板。Rive 已弃用专用输入、改推类型化数据绑定——无层级、有名字的类型化数据对代码*和 AI 代理*都更好瞄准。

### 2.4 Lottie / Bodymovin[^lottie-spec]

**范式**:**烘焙导出格式**——AE 合成被编译为 JSON(贝塞尔路径数据、层级、带缓动切线的关键帧),由播放器库重渲染。

**带不走**:绑定、表达式、状态机、逻辑——插值以外的一切。交互退化为外部代码拖动播放区间。教训:**能求值的就不要烘焙**——只导关键帧,恰好切掉让动画活着的部分。

### 2.5 游戏引擎与其余(简)

- **Unity Mecanim**[^unity-animator]:动画剪辑;Animator Controller=状态+带守卫过渡的流程图,跑在**参数(int·float·bool·trigger)**上;blend tree 连续混合剪辑(1D 阈值 / 2D freeform)。
- **Godot**[^godot-tree]:AnimationPlayer 轨道(任意节点属性打关键帧);AnimationTree 有 Blend2/3·OneShot·TimeSeek·Transition 节点、BlendSpace1D/2D(参数空间布点,Delaunay 三角化)、StateMachine(advance 条件/表达式)、RESET 动画定确定性初值。
- **DragonBones**[^dragonbones]:开源 Spine 级——armature/骨骼/槽/皮肤/网格+权重/IK/事件。
- **Creature**[^creature]:程序化绑定——自动网格、自动权重、**motors**(骨骼/变形/物理马达,可打关键帧属性、相位错开)。

## 3. AI 时代工具

### 3.1 生成式视频(prompt→像素)

- **Runway Gen-4**[^runway-gen4]:prompt+参考图→视频(跨镜头"世界一致性");旧 Motion Brush(涂选区域+方向/速度滑杆驱动)未保留。
- **Pika**:Pikaframes(首尾图→生成过渡)、Pikaffects(**动词预设**:充气、融化、爆炸)、Pikadditions。
- **Luma Ray3.2**[^luma-ray32]:单片段最多 **16 个关键帧**("帧级导演控制")+可组合的命名运镜——业界正收敛到"稀疏条件+生成填充"。
- **Kling**[^kling-brush]:Motion Brush(涂区域→画运动路径→标静止区);Motion Control(从参考视频迁移全身动作)。
- **Sora**[^sora-features]:**Storyboard**(把事件描述放到时间轴上)、Remix(带强度滑杆替换元素)、Re-cut/Loop/Blend——评测普遍认为其价值"在剪辑而非画面"。
- **共同极限**[^gen-video-limits]:identity drift、时间漂移、非确定(种子保证不了语义)、无法局部修改——改一个元素=全部重生成。

**判定**:E✗ D✗ I(只能重写 prompt)C✗ X✗。**零术语、近零控制**——产物是死胡同。自然语言拆掉了术语门槛;扁平产物又拆掉之后的一切。

### 3.2 动作捕捉与动作合成(AI→可编辑绑定数据)

- **DeepMotion SayMotion**[^deepmotion]:文本→3D 动作;导出 **FBX/GLB/BVH**——下游完全可编辑。E✓ D✓ I✓(分层 prompt)C✓(重定向)X✓。
- **Plask**:视频→骨骼动作→清理工具→重定向→FBX/BVH/GLB。AI 作为*确定性流水线的前处理*——小白能理解的诚实契约。
- **Move AI**:专业级无标记多机位动捕;专业级捕获仍需清理——"AI 拿 90%,确定性编辑拿最后 10%"是行业常态。
- **Adobe Mixamo**[^mixamo]:自动绑定的范本——上传网格→自动关节放置(用户可微调标记)→动作库→FBX。**十年来对小白杠杆最大的 AI 之举:从几何推断结构,删掉了术语最重的一步(绑定),还带一个人眼可见的确认检查点。**
- **Cascadeur**[^cascadeur]:用户保留控制器;AutoPosing(神经网络)推断自然姿势、AutoPhysics 按平衡/动量修正轨迹——**AI 作为用户文档内的约束求解器,而非产物工厂**。现存工具中最接近 mo2d 需要的模型。

### 3.3 2D 专用 AI

- **Adobe Sensei 产品内功能**[^sensei-lipsync][^roto-brush]:自动口型(音素映射到时间轴)、Roto Brush 2/3(刷首帧,AI 传播,人工精修)——AI 只做一步枯燥工作,产出**纯常规可编辑数据**。这种"隐形 AI"契约正是专业者信任它的原因。
- **Live2D 自动功能**[^live2d-auto]:自动网格、自动变形器(v5.1)——官方仍建议人工检查。AI 提案绑定,用户校验,下游确定*且可交互*。
- **Meta Animated Drawings**[^animated-drawings]:上传任意单角色**涂鸦**→CV 检测、用户确认关节→自动绑定 2D 骨架→动作库。零术语,儿童画也能用(约 18 万张涂鸦数据集),已开源。证明**涂鸦→推断绑定→动作**是小白第一天就该有的"魔法时刻"。
- **EbSynth**[^ebsynth]:画一帧,确定性按例传播到所有帧——赢得画师信任;遮挡/大幅运动处崩坏(合成像素而非变形结构的代价)。
- **ToonCrafter**[^tooncrafter]:生成式卡通中间帧(两个关键帧+可选草图指导)——能想象的最小白友好的 AI 任务("把这两个姿势连起来"),毁在栅格输出上。研究分两派:栅格生成派 vs **线条几何派**[^line-inbetween]——几何派输出矢量编辑器能表达的结构。

### 3.4 AI 原生设计工具(交互模式)

- **Spline AI**[^spline-ai]:prompt→**四张预览图**→选一张→生成物*落入可编辑场景*;新编辑器让 AI"读场景、写场景"。预览即菜单吸收非确定(用选择代替种子);编辑发生在正常工具界面,不是聊天窗。
- **Figma AI / Make**[^figma-ai]:prompt→以标准表示(图层、自动布局、组件)落入→直接操纵→再 prompt。AI 写的就是专业者编辑的同一批原语——没有二等公民。
- **Motiff**[^motiff]:接受小白手头**已有的任何产物**(文本、线框、截图、PRD),归一化为标准可编辑原语。
- **v0 / Lovable**[^v0-loop]:prompt→制品→精修循环;制品始终可见可查;**每次 AI 动作都有版本化可回退历史**——这就是信任机制。

### 3.5 学术前沿(简)

Motion Diffusion Models(文本→动作,transformer 骨干)[^mdm];涂鸦→绑定(Animated Drawings 管线)[^animated-drawings];LLM→SVG 矢量生成(StarVector 等——输出天然可编辑);卡通中间帧的栅格生成派 vs 线条几何派[^tooncrafter][^line-inbetween]。

### 3.6 可控性维度判定

| 族 | E 可编辑 | D 确定 | I 增量 | C 可组合 | X 导出可编辑 | 模式 |
|---|---|---|---|---|---|---|
| 生成视频(Runway/Pika/Luma/Kling/Sora) | ✗ | ✗ | 只能重写 prompt | ✗ | ✗ | 零术语、零控制 |
| 动作 AI(SayMotion/Plask/Move/Mixamo) | ✓ | ✓ | ✓ | ✓ | ✓ | AI→可编辑绑定数据 |
| Cascadeur | ✓ | ✓ | ✓ | ✓ | ✓ | AI 在用户文档内补合理性 |
| Sensei 产品内 / Live2D 自动 | ✓ | ✓ | ✓ | ✓ | ✓ | 隐形 AI 产出纯常规数据 |
| Krikey(prompt+模板→短片) | ✗ | ✗ | 模板 | ✗ | ✗ | 规模化不成创作 |
| AI 设计工具(Spline/Figma/Motiff/v0) | ✓ | 部分 | ✓ | ✓ | ✓ | 生成*落入*场景;预览吸收非确定 |

**每个被信任的工具共享的那个倒置**:AI 产出被放进**用户随后亲手操纵的同一个可编辑、确定性表示**里——绝不是旁路产物。

## 4. 概念总清单(本质 vs 偶然→mo2d 归约)

判定键:**E** 本质,**D** 可推导/选学深度,**I** 偶然。"mo2d"列引用 [line-model.zh-CN.md](line-model.zh-CN.md) §3 的九大映射(N1–N9)与 [relations.zh-CN.md](relations.zh-CN.md) §1 的关系库。

| 概念 | 使用工具 | 根本作用 | 判定 | mo2d 归约 |
|---|---|---|---|---|
| 时间轴(时间轴/帧条) | 全部 | "何时"的居所 | E | N4 轨道线 |
| 帧 / 单元格 / 造型 | 除 Scratch 类 | 离散视觉状态 | E | N4 轨道线上的节点(离散值) |
| 关键帧(时刻的值) | Animate·AE·Harmony·Krita·Dreams·Moho | 稀疏的授权状态 | E | N4 节点(位置=tick,值=节点属性) |
| 插值 / 补间 / 中间帧 | 全部 | 在状态之间合成 | E | N4 段属性 |
| 缓动 / 曲线编辑器 | Animate·AE·Harmony·Krita·Dreams | 变化速率 | E | N4 段属性(逐分量手柄) |
| 图层 / 列 / 轨道 | 全部 | 平行的 z 序内容流 | E | N7 集合关系+zOrder 通道 |
| 洋葱皮 | 逐帧族 | 看见相邻状态 | D | 会话层叠加物(线) |
| 复用(元件/库/实例) | Animate·TVPaint·Harmony·Scratch | 定义一次,放置多次 | E | 一曲线 N shape 关系(relations §1.0.1) |
| 变换层级(父/peg/pegsbar) | AE·Harmony·OpenToonz·Moho | 运动继承 | E | bind+节点共享(N2/N3) |
| 变形器 / 骨骼 / 木偶 | Harmony·Animate·AE·Moho·OpenToonz·Spine·Live2D·Rive | 非刚性重塑 | E(角色) | N2/N3(骨=段,骨架=折线) |
| 网格+权重 | Spine·Live2D·Rive·DragonBones·Creature | 多骨骼平滑弯曲 | D | N5 经纬线+bind 权重 |
| 图形替换 / switch | Harmony·Moho·Scratch | 离散状态换装 | E | 轨道上的离散通道(line-model §3.1) |
| 遮罩 / matte / cutter | Animate·AE·Harmony | 被另一形状裁剪 | E | shape 关系+填充规则(非线第 2 类) |
| 绘制顺序(z) | Live2D·一切图层 | 遮挡 | E | zOrder 通道(moho-mapping §7.1) |
| 参数空间(参数→姿势) | Live2D·Unity·Godot·Rive | 姿势与时间解耦 | E | **通道**(早已是一等公民) |
| 混合空间(N 姿势↔参数) | Unity·Godot·Rive·Live2D | 连续姿势混合 | E | 通道+多轨道采样关系 |
| 物理 / 钟摆 | Moho·Live2D·Spine·Creature | 自主次级运动 | D | N9 sim 关系(B6 已落地) |
| 事件(时间标记→宿主) | Spine·Rive·Unity·Godot | 与应用逻辑同步 | E | **respond** 关系族(路线图 C7,2026-11-17 确认) |
| 状态机(带守卫的过渡) | Rive·Unity·Godot | 离散行为选择 | E | **离散通道+带守卫的 respond**(C7,2026-11-17 确认) |
| 监听器(输入→动作) | Rive | 文件内交互接线 | E | 目标线上的 **respond** 关系(C7,2026-11-17 确认) |
| 节点图(绑定/FX/合成) | Harmony·OpenToonz | 声明式接线 | I | 关系(但作为*函数*,而非 GUI) |
| 预合成 / 嵌套 | AE·OpenToonz | 封装一组 | E(功能) | group 关系(N7,B4 已落地) |
| 表达式 / 脚本 | AE·Animate·TVPaint·OpenToonz | 程序化关系 | I | 自定义关系(沙箱 IR) |
| 曝光 / 停留 | OpenToonz·TVPaint·Krita | 停 N 帧 | E | 轨道节点间距 |
| 预设(动作/缓动) | Animate·AE | 打包的命名意图 | E(模式) | 样式模板(B2)+AI |
| 表演 / 录制手势 | Dreams·AE(Motion Sketch) | 先捕获后编辑 | E(入门) | **表演录制**(C7:手势→轨道关键帧,走 auto-key,2026-11-17 确认) |
| 摄像机 / 舞台 | Animate·OpenToonz·Harmony | 视口变换 | E | 盒线(line-model §3.1) |
| 场景 / 合成套合成 | Animate·AE·Dreams | 文档分段 | I | 文档=一个舞台;复用靠 group |
| 纸质血统(xsheet/pegbar/台面/秒表) | OpenToonz·TVPaint | 数字化经典工作流 | I | 留功能,绝不留名字 |

**读表**:§4 每一个本质行都能经九大映射归约——内核**无需格式改动**即可表达业界认为本质的一切。关系库里尚未一等公民化的能力簇只有两团:**交互簇**(事件/状态机/监听器)与**捕获簇**(表演录制)。§6 处理两者。

## 5. 第一性原理发现

- **F1 — 不可约内核极小且处处相同。** 时间轴 · 分层内容流 · 状态(帧/关键帧)· 插值 · 速率控制 · 层级 · 复用 · 变形 · 离散换装 · 遮挡 · 参数 ·(交互作品还有)输入→决策→响应。调查过的每种范式都是这约 12 个观念的某种打包;其余全是表皮。
- **F2 — 术语而非概念,才是新手税。** 同一概念 Many 名(补间/中间帧/曝光;peg/父级/空对象/pegsbar;元件/实例;遮罩/matte/cutter;合成/场景/sub-xsheet)。卸掉命名负担,大多数有据可查的摩擦即消失,模型无需改动。
- **F3 — 最昂贵的失败是"状态不可见"问题。** 影片剪辑拖动时不播放[^wikibooks-flash]、AE 关键帧藏在折叠属性里、Harmony"时间轴会骗人"的双视图[^reddit-harmony]、TVPaint 头 vs 曝光的传播规则[^tvpaint]。规则:**凡影响输出的状态,必须显形在用户正在看的地方。**
- **F4 — 双表示是被证明的渐进披露**:先简单视图,专家视图按需出现(Harmony 时间轴↔节点图;AE 图层条↔图表;Dreams Compose↔Keyframe)——但必须是**一个模型两张皮**,绝不是两个模型。
- **F5 — 预设就是业界既有的"AI 替你命名"**:动作预设、缓动画廊——打包的、命名的、官方推荐的入口。AI 助手是它的推广。
- **F6 — 表演/捕获是找到的最强新手坡道**(Dreams Perform、AE Motion Sketch):先录下变化,再编辑录下的结果——关键帧词汇被推迟,而非被要求。
- **F7 — 绑定是所有学习曲线变陡的地方**(Harmony node hell、Smart Bones、骨骼工具)。新手安全的绑定=系统提案绑定(自动绑定),只露手柄——用户明显能操作自己建不出的绑定(Mixamo、Animated Drawings)。
- **F8 — 极简主义有效但不成规模。** Pivot(<6 概念)做得出动画;Krita 证明栅格架构把成本泄漏给用户。插值+复用仍是新手系统必须暗中内置的杠杆概念。
- **F9 — 血统隐喻(xsheet、pegbar、秒表、台面)服务老手,不服务新手。** 留下功能,按用户生成名字——正是 AI 能动态做的事。
- **F10 — 每个本质功能在工具间存在 2–3 个冗余变体**(遮罩/matte/cutter;图形/影片剪辑;时间轴 vs 节点图)。每功能一个干净原语就能避免大多数有据可查的困惑——mo2d 内核正是那个"每功能一个原语"的赌注。
- **F11 — 交互动画的最小第一性原理模型**(Rive/Unity/Godot/Live2D 独立趋同):`姿势 = f(参数)`,`参数 = g(输入, 时间, 物理)`;交互=事件→改输入→重求值→输出收敛。一个工具"多交互",取决于 (b)–(d) 活在**文件里**还是宿主代码里。输入类型三件套**布尔·数值·触发**在 Rive、Unity、Godot 独立复现——趋同演化,故近乎基本。
- **F12 — Live2D 的解耦:先有"无时间的姿势"。** 参数空间(滑杆→姿势)比时间轴更根本;时间只是参数的驱动器之一。新手应在遇见时间之前先拿到可见结果(一张跟随滑杆的脸)。
- **F13 — AI 信任模式**(§3.6 那个倒置):AI 提案→一个人眼可见的检查点由人校验(Mixamo 标记、Animated Drawings 关节、Spline 预览选图)→下游一切确定且手改。非确定性被关在**选择点**里,绝不漏进播放。生成必须**以原生对象落地**(关键帧/曲线/约束)——绝不烤成像素。

## 6. 这份调研对 mo2d 意味着什么

- **R1 — 内核覆盖确认。** §4 每个本质概念都能经九大映射归约;调研不暗示任何格式改动(与 architecture §3.2 流式渲染"零格式改造"结论一致)。
- **R2 — 缺口:文件内交互。** 交互簇(事件/状态机/监听器)对愿景("交互动画")是本质的,但缺席于标准关系库与 roadmap。**owner 2026-11-17 确认:respond 关系族**——(事件源:点击·触摸·松开·悬停·拖拽·按键·时间点·动画播完)×(条件)→(动作:写通道 / 切离散轨道 / 触发另一事件);"状态机"即离散通道+带守卫的 respond 关系;"监听器"即目标线上的 respond 关系。完全契合"关系即函数",无需改模型;**落路线图 C7**(词表在 relations.zh-CN.md §1.2,事件驱动求值在 architecture §3.3,纯播放器确定性事件处理)。
- **R3 — 缺口:周期驱动器。** 生活动词(摆/呼吸/飘/浮沉)要循环轨道模式或 `oscillate` 热路径关系(周期/振幅/相位通道)——**2026-11-17 确认**:`oscillate` 关系(路线图 C7)。
- **R4 — 带可见确认检查点的自动绑定**(Mixamo/Animated Drawings 模式):AI 能力(vision §6 已划给 AI)增加要求:提案绑定以普通线+bind 关系落地,播放前有一眼可校的确认[推导]。
- **R5 — 生成的动作以轨道/关键帧落地,绝不落像素**:文本/草图/表演输入编译为轨道线+段缓动(能力即数据,vision 规则 3);ToonCrafter 式中间帧只走线条几何路线[推导]。
- **R6 — 表演录制作为捕获通道**(Dreams Perform 模式):录手势→轨道关键帧——**2026-11-17 确认**:最小表演录制落路线图 C7(手势→轨道关键帧,走 auto-key)。
- **R7 — 小白界面是独立于内核的关注点**:生活概念层——AI 接受哪些词、什么永不出现、验收门——在 [vernacular.zh-CN.md](vernacular.zh-CN.md) 规定。

## 7. 来源

合并自正文脚注;抓取于 2026-11-17 调研。id→URL 全映射见 frontmatter `sources`。关键簇:Adobe Animate/AE helpx 手册;Toon Boom 文档+r/ToonBoomHarmony;OpenToonz readthedocs(557 页手册);TVPaint/Krita/Procreate Dreams 手册;Pivot help;Scratch(Wikipedia);Spine 用户指南;Live2D Cubism 手册;Rive 文档(状态机/监听器/数据绑定);Lottie 规范;Unity/Godot 手册;Runway/Luma/Kling/Sora 官方页+生成视频失败分析;DeepMotion/Plask/Move AI/Mixamo/Cascadeur;Adobe Sensei 博客+帮助;Meta Animated Drawings;EbSynth;ToonCrafter+线条中间帧论文;Spline/Figma/Motiff/v0 页面;MDM(ICLR 2023)。
