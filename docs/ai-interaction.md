---
type: Design Document
title: "AI-Native Interaction: The Stage Is the Interface"
description: The mo2d interaction model — no standard toolbar/menus; display equals edit (multiple clients of one rendering core); dual channels (direct manipulation + natural language); controls are lines (temporary controls generated on demand by AI); every operation is a command (JSON command stream + MCP); unified undo/history.
tags: [ai-native, interaction, command-protocol, mcp, display-equals-edit]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: vectorcraft-mcp
    resource: file:///Users/wei/vectorcraft/docs/mcp.md
    title: "vectorcraft MCP 文档(命令协议工程范式参照)"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: relations-doc
    resource: /relations.md
    title: "本 bundle:关系即函数"
---

**English** | [简体中文](ai-interaction.zh-CN.md)

# AI-Native Interaction: The Stage Is the Interface

The owner's starting point: **有了 AI,不仅 GUI 控件等几乎就不怎么需要了,可以交给用户自定义,也就不大需要标准的工具栏、菜单栏等控件,用户直接在舞台定义线及线间的联动关系即可。**

*(With AI, GUI controls and the like become almost unnecessary — they can be left for users to customize — so there is little need for a standard toolbar, menu bar, or other such controls; users simply define lines and the linkage relations between them directly on the stage.)*

## 1. Position

- **No built-in standard toolbar/menu/properties panel**. The interface = the stage (the world of lines) + the command protocol (the agent's interface);
- GUI is demoted from "product" to "**generatable view**": any control can be generated on demand by AI and discarded after use;
- Users do not learn controls; they learn **the domain itself** — drawing a line, connecting a relation; this is closer
  to the heart of animation than memorizing which panel hides under which menu.

## 1.5 Display equals edit (established by the owner, 2026-11-11)

The technical implication of "GUI no longer matters": **there is no editor/player distinction** —

- The stage, headless export, AI sessions, and future web pages are all **clients of the same rendering core**,
  rendering the same document;
- Editing = commands change the document; display = the same pipeline renders the same document — there is no pair of
  representations, "edit-mode model" vs "playback-mode model";
- **Overlays** (node handles, selection highlights, connection indicators) do not belong to the document but to the
  session layer, and **overlays are lines too** (the same family as the §3 controls-are-lines);
- Corollary: **the view of any frame is faithful** — SVG can serve as a per-frame view/exchange format
  (the engineering source of truth remains .mo2d; see [file-format.md](file-format.md) §4 Three-layer format architecture).

## 2. Dual-channel interaction

| Channel | Operations | Examples |
|---|---|---|
| **Direct manipulation** | draw lines, drag nodes, connect lines to build relations, drag the box to transform | draw a polyline beside a tentacle's body, then drag the body's nodes onto the polyline to complete the binding |
| **Natural language** | state intent → AI translates it into a command sequence | "make the tentacle sway following the cursor" → AI builds propagate/simulate relations and sets keyframes |

The two channels are equivalent: every action done by hand is also a command (see §4), and every sentence said to the AI also lands on the same command stream.

## 3. Controls are lines

Temporary controls **are themselves lines**, generated on demand by AI and discarded after use — no control framework needed:

| Control | Line form |
|---|---|
| Slider | a track line + a short cursor line (the cursor's position = the value; the cursor is a node on the track line) |
| Color-picker gradient bar | a gradient control bar (color stops = nodes) |
| Transform box | a 4-node closed box line |
| Playback controls | a time base line + a short playhead line |

[Inference] Controls and content are isomorphic — the natural extension of "everything is lines" to the interaction layer:
if you can draw a line, you can make a control; AI generating a control = AI drawing a line.

## 4. Every operation is a command

- **Command protocol**: the engine's single entry point is commands (a JSON command stream); create/delete lines,
  modify nodes, build relations, set keyframes, play, export — all are commands;
- **Replayable**: the command stream is the history — undo = inverse commands, collaboration = command-stream merging,
  AI context = command-stream summaries;
- **MCP server mode**: the engine exposes commands as an MCP server (following the engineering practice of
  vectorcraft/maohou[^vectorcraft-mcp]); AI hosts (zcode etc.) perform all operations over MCP;
- **Determinism**: same document + same command stream → same result (verified by the CPU oracle).

## 5. Drawing the boundary between document state and session state

Based on [line-model.md](line-model.md) §4, the third class of the non-line boundary:

- **Document state** (persisted): lines, relations, channels, payloads;
- **Session state** (not persisted): view camera, selection set, undo stack, clipboard, AI conversation context;
- The command stream records **document changes**; session state is managed by the host itself and is not carried by save/load.

## 6. Security and permissions

[Inference, to be refined during engineering] The function body of a custom relation is a logic payload (code) —
execution must be sandboxed (a WASM interpreter etc., following existing engineering practice); AI-generated
command streams should be previewable (diff) before being applied.

## 7. Open Questions

- [ ] The first-version command table of the command protocol (with [mvp.md](mvp.md) § Scope 5 as the minimal set);
- [ ] The control-to-command binding syntax for controls-are-lines;
- [ ] The preferred form of the AI host (local MCP / cloud).
