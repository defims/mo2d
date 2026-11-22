---
type: Reference
title: "主流矢量编辑器工具清单对比与最小工具集"
description: 主流矢量编辑器逐个工具清单(Illustrator ~35、Inkscape ~22、Affinity/CorelDRAW ~25、Figma ~10、Boxy SVG ~14、Vectr ~8),跨编辑器交集,以及推导出的最小工具集——六个交互工具+四个面板+撤销,每个都映射到 mo2d 原语并标注落地状态。
tags: [vector-editors, tools, minimal-tool-set, illustrator, inkscape, figma, boxy-svg, comparison]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-22T15:30:00Z }
sources:
  - id: adobe-tools
    resource: https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/select-and-use-tools-in-the-toolbar.html
    title: "Adobe:Select and use tools in the Illustrator toolbar"
  - id: inkscape-glossary
    resource: https://wiki.inkscape.org/wiki/
    title: "Inkscape wiki:toolbox/glossary"
  - id: inkscape-wikibooks
    resource: https://en.wikibooks.org/wiki/A_Guide_to_Inkscape/Path_Drawing_Tools
    title: "Wikibooks:Inkscape path drawing tools (pencil/bezier/calligraphy/node)"
  - id: boxy-press
    resource: https://boxy-svg.com/press
    title: "Boxy SVG press(极简 UI,灵感来自 Inkscape/Sketch/Illustrator)"
  - id: vectr
    resource: https://vectr.com
    title: "Vectr(免费极简矢量编辑器)"
  - id: figma-dev
    resource: https://help.figma.com/hc/en-us/articles/360039956974-Explore-tools-in-the-toolbar
    title: "Figma:explore tools in the toolbar"
  - id: wiki-comparison
    resource: https://en.wikipedia.org/wiki/Comparison_of_vector_graphics_editors
    title: "Wikipedia:Comparison of vector graphics editors"
  - id: landscape-doc
    resource: /landscape.zh-CN.md
    title: "本 bundle:动画软件全景调研(姊妹篇:动画工具概念清单)"
---

[English](vector-tools.md) | **简体中文**

# 主流矢量编辑器工具清单对比与最小工具集

[landscape.md](landscape.zh-CN.md)(动画软件)的姊妹篇:这里是**矢量编辑器**——各家装了什么工具、跨编辑器交集是什么、推导出的**最小工具集**与 mo2d 映射。调研日期 2026-11-22。

## 1. 逐编辑器清单

| 编辑器 | 工具数(约) | 工具(分组) |
|---|---|---|
| Illustrator(完整工具栏) | ~35 | **选择**(选择 V · 直接选择 A · 组选择 · 画板 · 切片) · **绘制**(钢笔 P · 添加/删除/切换锚点 · 曲率 · 铅笔 N · 平滑 · 画笔 B · 宽度 Shift+W · 文字 T+7 变体 · 直线 · 弧线 · 螺旋 · 矩形网格 · 极坐标网格 · 矩形 M · 圆角矩形 · 椭圆 L · 多边形 · 星形 · 光晕) · **修改**(旋转 R · 镜像 O · 缩放 S · 倾斜 · 重塑 · 自由变换 E · 混合 W · 图表 J · 吸管 I · 测量 · 网格 U · 渐变 G · 橡皮 Shift+E · 剪刀 C · 刻刀) · **导航**(抓手 H · 缩放 Z) · 绘制模式(正常/后方/内部) |
| Inkscape | ~22 | 选择器 S · 节点 N · 微调 W · 喷涂 A · 测量 M · 缩放 Z · 矩形 R · 3D 盒 · 椭圆 E · 星形 * · 螺旋 I · 铅笔 P · 贝塞尔笔 B · 书法 C · 橡皮 E · 油漆桶 U · 文字 T · 连接器 O · 渐变 G · 网格 · 取色 D · 对话框 |
| Affinity Designer / CorelDRAW | 各 ~25 | 移动 · 节点 · 钢笔/铅笔/画笔 · 形状生成器 · 框架/美术文本 · 渐变 · 透明度 · 填充 · 取色器 · 矢量裁剪 · 圆角 · 点变换 · 测量/标注 · 艺术笔 · 涂抹/扭曲/吸引/排斥 · 擦除/虚拟段删除 · 自由变换 · 缩放/视图(Corel 另有:多段绘制、B 样条、三点曲线、标注、连接器) |
| Figma | ~10 | 移动 V · 框架 F · 区域形状(矩形 R/椭圆 O/直线 L/多边形) · 钢笔 P · 铅笔 B · 文字 T · 评论 C · 抓手 H · 缩放 K + **矢量网络**(独有)、组件/自动布局(面板层) |
| Boxy SVG(极简派) | ~14 工具+6 面板 | 选择 · 节点 · 铅笔 · 钢笔 · **14 种参数形状**(矩形 · 圆 · 椭圆 · 环 · 扇形 · 弓形 · 三角 · n 边形 · 星 · 齿轮 · 箭头 · 十字 · 直线 · 折线) · 文字 · 渐变 · 取色 · 缩放/平移;面板:填充描边 · 合成(层) · 排列(对齐/变换/次序) · 排版 · 视图 |
| Vectr(极简派) | ~8 | 移动 · 路径(钢笔/铅笔) · 矩形/椭圆/直线 · 文字 · 背景 · 页面 |

## 2. 交集——每个编辑器都装的

| # | 不变量 | 说明 |
|---|---|---|
| 1 | **选择/移动**(含多选、组选) | 根交互 |
| 2 | **节点/直接编辑**(锚点、句柄) | 编辑曲线结构 |
| 3 | **画线**(钢笔/贝塞尔/铅笔——自由或精确) | 创建任意曲线 |
| 4 | **参数形状**(矩形 · 椭圆 · 直线 · 多边形;多数含星形变体) | 快速结构化创建 |
| 5 | **文字** | 除纯游戏素材编辑器外人人都有 |
| 6 | **视口**(缩放/平移/抓手) | 导航,会话层 |
| 7 | **填充/描边**(绘制属性——面板+吸管) | 通用面板 |
| 8 | **层/对象树** | 通用面板 |
| 9 | **对齐/分布** | 近通用面板 |
| 10 | **撤销/重做/复制** | 通用命令,不算工具栏工具 |

其余全是分层,不是核心:
- **第二层(多数编辑器)**:专用变换工具(旋转/缩放/镜像/倾斜——别处是面板操作)、布尔/路径运算(并/差/交/排除)、渐变工具、平滑/雕刻(微调/涂抹/扭曲)、橡皮/剪刀/刻刀/裁剪、测量/标注、油漆桶、混合/插值、喷涂、连接器、书法。
- **专业/小众层**:网格(渐变网格)、图表工具、光晕、画板/切片管理、符号系统、透视网格、封套扭曲、3D 盒、宽度工具、重塑、绘制模式。

## 3. 最小工具集(推导)

**六个交互工具 + 四个面板 + 撤销。** 这就是能复现上述所有编辑器能力的最小集合——其余一切都是这些之上的宏或参数面板:

1. **选·移** ——选择、多选、移动、组选;
2. **节点编辑** ——加/移/删节点,拖句柄(曲线结构);
3. **画线** ——自由铅笔 + 精确钢笔(一个手势面,两档容差);
4. **形状** ——矩形 · 椭圆 · 直线 · 多边形(参数化,可调);
5. **文字** ——文本输入(转轮廓获得完全控制);
6. **视口** ——缩放/平移/抓手(会话层,非文档);
7. **填充/描边面板** ——填充、描边、宽度、不透明度(面板+吸管=属性复制);
8. **层面板** ——对象树、排序、成组/解组;
9. **对齐面板** ——对齐/分布(+吸附);
10. **撤销(历史)** ——每个操作可逆。

为什么这是"最小"(证据而非口味):Vectr 恰好只装这些并被宣传为最简编辑器;Boxy 只在其上加了参数形状广度;Illustrator 的 35 个工具全部可分解为这套之上的宏/参数手柄(宽度工具=可拖的描边宽手柄;旋转/缩放/镜像=变换手势;混合=插值实例;橡皮/剪刀=路径分割修剪;网格=结构化参数网格;吸管=样式复制;油漆桶=填充指派;符号喷枪=实例副本;画板/切片=文档导出管理)。

## 4. mo2d 映射——最小集住在哪里

| 最小工具 | mo2d 形态 | 状态 |
|---|---|---|
| 选·移 | 舞台直接操纵+`node.set`/群组变换盒 | ✅ A6/D8 |
| 节点编辑 | `node.set`(p/hi/ho)+段可见性 | ✅ |
| 画线 | 画线手势→`line.create`;舞台五操作 | ✅ A6 |
| 形状 | `line.create` 参数节点序列(rect/circle 构造器已在 svgimport;参数形状命令是薄 AI/宏层) | ✅(逐形状糖待) |
| 文字 | **derive** 关系(文字→轮廓)——库 §1,引擎待 | 📋 C-队列 |
| 视口 | 会话状态(永不入文档) | ✅ |
| 填充/描边 | 线三级属性+`style.apply` 模板(吸管=样式复制) | ✅ B2 |
| 层 | layer/group 关系+变换盒 | ✅ B4 |
| 对齐 | 确定性几何运算(AI/命令层);吸附=会话状态 | 部分(命令层) |
| 撤销 | 命令流逆命令(每个操作可逆) | ✅ A4 |
| **布尔运算**(第二层,唯一真缺口) | 多边形并/差/交作为**关系**——确定性几何已在 `hull.rs`(Sutherland–Hodman 交集+marching squares);候选:`carve`/`union` 关系族 | 📋 候选(可复用 hull 先例) |
| 渐变工具 | gradient 关系+颜色参数(B3) | ✅ |
| 平滑/雕刻 | sim 关系+微调即 sim(有机变形=动态关系) | ✅ 机制(B6)/糖待 |
| 橡皮/剪刀 | 节点/段删除+路径分割=命令宏 | ✅ 原语级 |
| 混合 | derive/插值实例序列 | 📋 |
| 测量/标注 | 测量=线几何查询(命令),标注=文字+线关系 | 📋 |
| 画笔/书法 | brush(B2)+压力(节点属性) | ✅ B2 |

**读表**:最小集约 90% 映射到已落地原语。真缺口是**布尔/路径运算关系族**(可复用 `hull.rs` 的确定性求交代码)、**文字→轮廓 derive**、**混合**——全是"加一个关系,不加原语"(relations §3)。

## 5. 来源

合并自 2026-11-22 调研。Adobe 工具栏参考、Inkscape glossary/Wikibooks、Boxy SVG press、Vectr、Figma 工具栏帮助、Wikipedia 对比——见 frontmatter `sources`。动画软件姊妹调研在 [landscape.md](landscape.zh-CN.md)。
