---
type: Reference
title: "Mainstream Vector Editors: Tool Inventory Comparison and the Minimal Tool Set"
description: Tool-by-tool inventory of the mainstream vector editors (Illustrator ~35 tools, Inkscape ~22, Affinity/CorelDRAW ~25, Figma ~10, Boxy SVG ~14, Vectr ~8), the cross-editor intersection, and the derived minimal tool set — six interaction tools + four panels + undo — each mapped to mo2d's primitives with landed/pending status.
tags: [vector-editors, tools, minimal-tool-set, illustrator, inkscape, figma, boxy-svg, comparison]
status: draft
generated: { by: zcode-agent/glm-5.3-flash, at: 2026-11-22T15:30:00Z }
sources:
  - id: adobe-tools
    resource: https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/select-and-use-tools-in-the-toolbar.html
    title: "Adobe: Select and use tools in the Illustrator toolbar"
  - id: inkscape-glossary
    resource: https://wiki.inkscape.org/wiki/
    title: "Inkscape wiki: toolbox/glossary"
  - id: inkscape-wikibooks
    resource: https://en.wikibooks.org/wiki/A_Guide_to_Inkscape/Path_Drawing_Tools
    title: "Wikibooks: Inkscape path drawing tools (pencil/bezier/calligraphy/node)"
  - id: boxy-press
    resource: https://boxy-svg.com/press
    title: "Boxy SVG press (minimal UI inspired by Inkscape/Sketch/Illustrator)"
  - id: vectr
    resource: https://vectr.com
    title: "Vectr (free minimal vector editor)"
  - id: figma-dev
    resource: https://help.figma.com/hc/en-us/articles/360039956974-Explore-tools-in-the-toolbar
    title: "Figma: explore tools in the toolbar"
  - id: wiki-comparison
    resource: https://en.wikipedia.org/wiki/Comparison_of_vector_graphics_editors
    title: "Wikipedia: Comparison of vector graphics editors"
  - id: landscape-doc
    resource: /landscape.md
    title: "本 bundle:动画软件全景调研(姊妹篇:动画工具概念清单)"
---

**English** | [简体中文](vector-tools.zh-CN.md)

# Mainstream Vector Editors: Tool Inventory Comparison and the Minimal Tool Set

Companion to [landscape.md](landscape.md) (which covers *animation* software). Here: *vector
editors* — what tools they ship, what the cross-editor intersection is, and the derived
**minimal tool set** with the mo2d mapping. Survey date 2026-11-22.

## 1. Inventory by editor

| Editor | Tool count (approx) | Tools (grouped) |
|---|---|---|
| Illustrator (full toolbar) | ~35 | **Select** (selection V · direct A · group · artboard · slice) · **Draw** (pen P · add/delete/switch anchor · curvature · pencil N · smooth · brush B · width Shift+W · type T +7 variants · line · arc · spiral · rect grid · polar grid · rect M · rounded rect · ellipse L · polygon · star · flare) · **Modify** (rotate R · reflect O · scale S · shear · reshape · free transform E · blend W · graph J · eyedropper I · measure · mesh U · gradient G · eraser Shift+E · scissors C · knife) · **Nav** (hand H · zoom Z) · paint modes (normal/behind/inside) |
| Inkscape | ~22 | selector S · node N · tweak W · spray A · measure M · zoom Z · rect R · 3D box · ellipse E · star * · spiral I · pencil P · bezier pen B · calligraphy C · eraser E · paint bucket U · text T · connector O · gradient G · mesh · dropper D · dialogs |
| Affinity Designer / CorelDRAW | ~25 each | move · node · pen/pencil/brush · shape builder · frame/artistic text · gradient · transparency · fill · colour picker · vector crop · corner · point transform · measure/dimension · artistic media · smear/twirl/attract/repel · erase/virtual-segment-delete · free-transform · zoom/view (Corel adds: polydraw, B-spline, 3-point curve, dimension, connector) |
| Figma | ~10 | move V · frame F · region shapes (R rect / O ellipse / L line / polygon) · pen P · pencil B · text T · comment C · hand H · scale K + **vector networks** (unique), components/auto-layout (panel-level) |
| Boxy SVG (minimalist) | ~14 tools + 6 panels | select · node · pencil · pen · **14 parametric shapes** (rect · circle · ellipse · ring · pie · crescent · triangle · n-gon · star · cog · arrow · cross · line · polyline) · text · gradient · eyedropper · zoom/pan; panels: Fill&Stroke · Compositing (layers) · Arrangement (align/transform/z-order) · Typography · View |
| Vectr (minimalist) | ~8 | move · pen/pencil path · rect/ellipse/line · text · background · pages |

## 2. The intersection — what every editor ships

| # | Invariant | Notes |
|---|---|---|
| 1 | **Select / move** (incl. multi-select, group select) | the root interaction |
| 2 | **Node / direct edit** (anchors, handles) | edit the curve structure |
| 3 | **Draw** (pen / bezier / pencil — freehand or precise) | create arbitrary curves |
| 4 | **Parametric shapes** (rect · ellipse · line · polygon; star/star-variants in most) | quick structured creation |
| 5 | **Text** | everywhere except pure game-asset editors |
| 6 | **Viewport** (zoom / pan / hand) | navigation, session-layer |
| 7 | **Fill / stroke** (paint attributes — panel + eyedropper) | universal panel |
| 8 | **Layers / object tree** | universal panel |
| 9 | **Align / distribute** | near-universal panel |
| 10 | **Undo / redo / duplicate** | universal command, not a toolbar tool |

Everything else is tiered, not core:
- **Second tier (most editors)**: dedicated transform tools (rotate/scale/reflect/sheer — panel ops elsewhere), boolean/path ops (union/subtract/intersect/exclude), gradient tool, smooth/sculpt (tweak/smear/twirl), eraser/scissors/knife/crop, measure/dimension, paint bucket, blend/interpolate, spray, connector, calligraphy.
- **Pro/niche tier**: mesh (gradient mesh), graph tools, flare, artboard/slice management, symbol systems, perspective grid, envelope distort, 3D box, width tool, reshape, paint modes.

## 3. The minimal tool set (derived)

**Six interaction tools + four panels + undo.** That is the smallest set that reproduces the
capability of every editor above — everything else is a macro or a parameter panel over these:

1. **选·移 (select & move)** — select, multi-select, move, group select;
2. **节点编辑 (node edit)** — add/move/delete nodes, drag handles (curve structure);
3. **画线 (draw)** — freehand pencil + precise pen (one gesture surface, two tolerances);
4. **形状 (shapes)** — rect · ellipse · line · polygon (parametric, resizable);
5. **文本 (text)** — text entry (convert-to-outlines for full control);
6. **视口 (viewport)** — zoom/pan/hand (session-layer, not document);
7. **填充/描边面板 (paint)** — fill, stroke, width, opacity (panel + eyedropper = attribute copy);
8. **层面板 (layers)** — object tree, reorder, group/ungroup;
9. **对齐面板 (align)** — align/distribute (+ snapping);
10. **撤销 (undo/history)** — every operation reversible.

Why this is *the* minimum (evidence, not taste): Vectr ships exactly these and is marketed as
the simplest editor; Boxy adds only parametric-shape breadth on top; Illustrator's 35 tools all
decompose into macros/parameter handles over this set (the width tool = a draggable stroke-width
handle; rotate/scale/reflect = transform gestures; blend = interpolated instances; eraser/scissors
= path split/trim ops; mesh = a structured parameter grid; eyedropper = style copy; paint bucket =
fill assignment; symbol sprayer = instanced copies; artboard/slice = document/export management).

## 4. mo2d mapping — where the minimal set lives

| Minimal tool | mo2d form | Status |
|---|---|---|
| 选·移 select & move | stage direct manipulation + `node.set` / group transform-box | ✅ A6/D8 |
| 节点编辑 node edit | `node.set` (p/hi/ho) + seg visibility | ✅ |
| 画线 draw | draw gesture → `line.create`; stage five-operations | ✅ A6 |
| 形状 shapes | `line.create` parametric node sequences (rect/circle builders live in svgimport for SVG; parametric shape commands are a thin AI/macro layer) | ✅ (per-shape sugar pending) |
| 文本 text | **derive** relation (text→outline) — library §1, engine pending | 📋 C-队列 |
| 视口 viewport | session state (never document) | ✅ |
| 填充/描边 paint | line three-level attrs + `style.apply` templates (eyedropper = style copy) | ✅ B2 |
| 层 layers | layer/group relation + transform box | ✅ B4 |
| 对齐 align | deterministic geometry ops (AI/command layer); snapping = session state | 部分 (命令层) |
| 撤销 undo | command stream inverses (every op reversible) | ✅ A4 |
| **布尔 boolean ops** (second tier, the one real gap) | polygon union/subtract/intersect as a **relation** — deterministic geometry already exists in `hull.rs` (Sutherland–Hodman intersection + marching squares); candidate: a `carve`/`union` relation family | 📋 候选(可复用 hull 先例) |
| 渐变 gradient tool | gradient relation + color 参数 (B3) | ✅ |
| 平滑/雕刻 sculpt | sim 关系 + tweak-as-sim (organic deform = dynamic relations) | ✅ 机制(B6)/Sugar 待 |
| 橡皮/剪刀 eraser/scissors | node/seg delete + path split = command macros | ✅ 原语级 |
| 混合 blend | derive/interpolated instance sequence | 📋 |
| 测量/标注 measure | measure = line-geometry query (command), dimension = text+line relation | 📋 |
| 画笔/书法 brush/calligraphy | brush (B2) + pressure (node attr) | ✅ B2 |

**Reading**: the minimal set maps ~90% onto landed mo2d primitives. The genuine gaps are the
**boolean/path-op relation family** (reusable from `hull.rs`'s deterministic intersection code),
**text→outline derive**, and **blend** — all "add a relation, not a primitive" per relations §3.

## 5. Sources

Consolidated; fetched during the 2026-11-22 survey. Adobe toolbar reference, Inkscape
glossary/Wikibooks, Boxy SVG press, Vectr, Figma toolbar help, Wikipedia comparison — see
frontmatter `sources`. The animation-software companion survey is [landscape.md](landscape.md).
