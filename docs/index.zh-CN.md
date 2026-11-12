---
type: Reference
title: "mo2d 知识库(中文目录)"
description: mo2d 文档库中文版目录——英语为主语言,本页为 docs/index.md 的中文孪生。
tags: [index, zh-CN]
status: stable
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
---

[English](index.md) | **简体中文**

# mo2d 知识库

mo2d——AI 原生的 2D 矢量动画引擎(万物皆线)——的正式文档库。
按 [Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format) 组织;
维护规程见 [AGENTS.md](AGENTS.zh-CN.md)(任何会话改本目录下的 .md 前必读),变更历史见 [log.md](log.md)。

# 规程与工具

* [AGENTS.md](AGENTS.zh-CN.md) - OKF 维护规程(frontmatter/index/log/校验的强制要求)
* [check-okf.py](check-okf.py) - 符合性校验器(`python3 docs/check-okf.py`,改完必须跑)

# 概念文档(建议阅读顺序)

* [vision.md](vision.zh-CN.md) - 愿景:AI 原生的 2D 矢量动画引擎,四条核心理念
* [line-model.md](line-model.zh-CN.md) - **万物皆线**(核心):线、节点、三级属性(线/节点/段)、九大映射与非线边界
* [moho-mapping.md](moho-mapping.zh-CN.md) - Moho 概念逐项映射:点线属性关系全表(完整属性值/关系参数词表)
* [relations.md](relations.zh-CN.md) - 关系即函数:预定义关系标准库与自定义关系
* [ai-interaction.md](ai-interaction.zh-CN.md) - AI 原生交互:舞台即界面,控件即线,一切操作皆命令
* [architecture.md](architecture.zh-CN.md) - 架构草案:分层、文档模型、GPU 策略、格式原则(status: draft)
* [file-format.md](file-format.zh-CN.md) - 文件格式:设计建议与参照(信封草案/Tick 时间/双形态/glTF·vectorcraft·maop 参照表)
* [mvp.md](mvp.zh-CN.md) - MVP 定稿 v5:触手+3D 场景,六件范围/六步门/八条完成定义/九项冻结决策
* [roadmap.md](roadmap.zh-CN.md) - 路线图 v4:六部件→阶段映射;A MVP(含舞台)/B 性能与词表/C AI 生态
* [3d-extension.md](3d-extension.zh-CN.md) - 3D 扩展路径:四条语义规则+六项预防决策(Vec3 从第一天);Phase D 远期
* [legal-risks.md](legal-risks.zh-CN.md) - 法务风险登记册:GPL 隔离/商标/净室/语料红线/许可证待办(八项行动清单)
* [glossary.md](glossary.zh-CN.md) - 术语表(中英对照)

# 相关资源(bundle 之外)

* `file:///Users/wei/maohou/plan/` - maohou 知识库(Moho 格式一手研究,本文档集的 Moho 事实来源)
* `file:///Users/wei/vectorcraft/docs/` - vectorcraft 工程文档(命令/MCP/格式工程范式参照)
* `file:///Users/wei/moho user content/` - Moho Pro 14.4 教程工程语料(84 个 .moho,远期导入验收用)
