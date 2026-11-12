---
okf_version: "0.2"
---

**English** | [简体中文](index.zh-CN.md)

# mo2d Knowledge Base

The official documentation library of mo2d — an AI-native 2D vector animation
engine (everything is lines). Organized per
[Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format);
see [AGENTS.md](AGENTS.md) for maintenance rules (any session must read it before
editing any .md here) and [log.md](log.md) for the change journal.

# Governance & tooling

* [AGENTS.md](AGENTS.md) - OKF maintenance rules (frontmatter/index/log/validation requirements)
* [check-okf.py](check-okf.py) - compliance validator (`python3 docs/check-okf.py`, must run after every edit)

# Concept documents (suggested reading order)

* [vision.md](vision.md) - Vision: the AI-native engine — six-component definition, division of labor
* [line-model.md](line-model.md) - **Everything is lines** (core): lines, nodes, three-level attributes, nine mappings, non-line boundary
* [moho-mapping.md](moho-mapping.md) - Moho concept mapping: complete attribute/relation value inventories
* [relations.md](relations.md) - Relations are functions: the standard relation library and custom relations
* [ai-interaction.md](ai-interaction.md) - AI-native interaction: the stage is the interface; controls are lines
* [architecture.md](architecture.md) - Architecture draft: layering, document model, GPU strategy
* [file-format.md](file-format.md) - File format: `.mo2d` single-file deliverable, Tick time, reference table
* [mvp.md](mvp.md) - MVP (v5): tentacle + 3D scene via the command protocol
* [roadmap.md](roadmap.md) - Roadmap (v4): phase A/B/C + Phase D (3D)
* [glossary.md](glossary.md) - Glossary (EN/中文)
* [3d-extension.md](3d-extension.md) - 3D extension path + preemptive decisions (Vec3 from day one)
* [legal-risks.md](legal-risks.md) - Legal risk register (GPL isolation, trademark, corpus redlines)

# Related resources (outside the bundle)

* `file:///Users/wei/maohou/plan/` - maohou knowledge base (first-hand Moho format research)
* `file:///Users/wei/vectorcraft/docs/` - vectorcraft engineering docs (command/MCP/format patterns)
* `file:///Users/wei/moho user content/` - Moho Pro 14.4 tutorial projects (84 .moho; never enters this repo)
