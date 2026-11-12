---
type: Playbook
title: "docs/ 知识库维护规程(OKF v0.2)"
description: 本目录按 Open Knowledge Format v0.2 组织;任何会话(人或代理)对 docs/ 下 .md 的增删改必须遵守本规程并以 check-okf.py 校验。
tags: [okf, governance, workflow]
status: stable
generated: { by: zcode-agent/glm-5.3, at: 2026-11-11T08:04:06Z }
sources:
  - id: okf-spec
    resource: https://github.com/GoogleCloudPlatform/open-knowledge-format
    title: Open Knowledge Format v0.2(官方 SPEC.md)
  - id: maohou-okf
    resource: file:///Users/wei/maohou/plan/AGENTS.md
    title: maohou 知识库 OKF 维护规程(本规程的范本)
---

# 适用范围

`docs/` 整个目录(项目正式文档库,git 建库后随仓库入库)。**任何新会话**在本目录
新建或修改任何 `.md` 文件时,本规程**必须**遵守。mo2d 仓库若建立根 AGENTS.md 并与本文冲突,以根文件优先。

> 上游 OKF 规范若发布比 v0.2 更新的版本,应复核本规程与 check-okf.py 后升级
> `okf_version`(2026-11-11 建立 bundle 时网络窗口未开,以 maohou 知识库执行的 v0.2 为准[^maohou-okf])。

# 硬性规则(OKF v0.2 符合性)

1. **概念文档**(`index.md`/`log.md`/`AGENTS.md` 以外的一切 `.md`)**必须**有 YAML frontmatter
   (文件首行 `---` 起始、配对 `---` 结束),且**必须**含非空 `type`。`type` 自选、描述性强;
   现有取值:`Design Document`/`Reference`/`Task Brief`/`Playbook`。
2. **推荐**字段:`title`、`description`(一句话,index 与检索用)、`tags`(YAML 列表)。
3. **Provenance**:来源进 frontmatter `sources`(每条 `resource` 必填;bundle 内相对路径、
   bundle 外给 `file://` 绝对路径或 URL;正文逐条事实断言用脚注 `[^id]` 对到 `sources[].id`)。
   owner 口述给定的规范表述在正文以引用块逐字收录,并在文首"规范表述"一节集中列出。
4. **Trust/lifecycle**:写/改内容的会话**必须**更新 `generated: { by, at }`;actor 惯例——
   代理 `zcode-agent/<model>`、人 `human:<id>`、自动过程 `process:<id>`;`at` 为真实 UTC 时间
   (`date -u +"%Y-%m-%dT%H:%M:%SZ"`,禁止编造)。owner 明确核验过的内容**应当**追加
   `verified: { by: human:<id>, at: … }`;未核验就不写(缺省即 unverified,是诚实信号)。
   草案用 `status: draft`,定稿 `stable`(缺省即 stable),被取代但留档 `deprecated`。
5. **保留文件名**:`index.md`(目录清单;bundle 根仅可带 `okf_version`)与
   `log.md`(更新历史,首行为 `# Directory Update Log`,ISO `YYYY-MM-DD` 日期分组、
   最新在上、条目粗体动词开头)**禁止**用作概念文档。

# 变更流程(每次改 docs/ 的 .md 都走)

1. 改文档正文 → 更新其 frontmatter `generated`(实质性内容变更时;纯格式微调可不更新但要在 log 记录)。
2. 新建/删除/更名文档 → 同步更新 `index.md`(描述取自该文档 `description`)。
3. 在 `log.md` 追加当日条目(当日已有日期组就并入该组)。
4. 跑 `python3 docs/check-okf.py` —— 必须 0 error 才算完成。
5. owner 评审通过的文档把 `status: draft` 翻成 `stable` 并补 `verified`。

# 结构约定

- Bundle 根 = `docs/`(根 `index.md` 声明 `okf_version: "0.2"`)。
- 目录维持扁平;主题增多再开子目录并配该子目录 `index.md`。
- 概念演进方式:owner 给出的新规范表述 → 更新对应概念文档的"规范表述"节并同步正文 → log 记录;
  被推翻的旧表述不删除历史,以"已修订"注记保留。
