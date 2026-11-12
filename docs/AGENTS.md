---
type: Playbook
title: "docs/ Knowledge Base Maintenance Playbook (OKF v0.2)"
description: This directory is organized per Open Knowledge Format v0.2; any session (human or agent) that adds, deletes, or modifies .md files under docs/ must follow this playbook and validate with check-okf.py.
tags: [okf, governance, workflow]
status: stable
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
sources:
  - id: okf-spec
    resource: https://github.com/GoogleCloudPlatform/open-knowledge-format
    title: Open Knowledge Format v0.2(官方 SPEC.md)
  - id: maohou-okf
    resource: file:///Users/wei/maohou/plan/AGENTS.md
    title: maohou 知识库 OKF 维护规程(本规程的范本)
---

**English** | [简体中文](AGENTS.zh-CN.md)

# Scope

The entire `docs/` directory (the project's official documentation library; committed to the repository once git is initialized). **Any new session** that creates or modifies any `.md` file in this directory **must** follow this playbook. If the mo2d repository later creates a root AGENTS.md that conflicts with this document, the root file takes precedence.

> If the upstream OKF specification publishes a version newer than v0.2, review this playbook and check-okf.py before upgrading `okf_version` (when the bundle was created on 2026-11-11 the network window was not open, so v0.2 as practiced in the maohou knowledge base is authoritative[^maohou-okf]).

# Hard rules (OKF v0.2 compliance)

1. **Concept documents** (every `.md` other than `index.md`/`log.md`/`AGENTS.md`) **must** carry YAML frontmatter
   (starting with `---` on the file's first line, closed by a matching `---`), and **must** contain a non-empty `type`. `type` is self-chosen and descriptive;
   current values: `Design Document`/`Reference`/`Task Brief`/`Playbook`.
2. **Recommended** fields: `title`, `description` (one sentence, for the index and retrieval), `tags` (YAML list).
3. **Provenance**: sources go into the frontmatter `sources` (`resource` is required on each entry; relative paths inside the bundle,
   `file://` absolute paths or URLs outside it; each factual claim in the body is footnoted `[^id]` back to `sources[].id`).
   Normative statements dictated by the owner are recorded verbatim in blockquotes in the body, and collected in a leading "Normative Statements" section.
4. **Trust/lifecycle**: the session that writes/modifies content **must** update `generated: { by, at }`; actor conventions —
   agent `zcode-agent/<model>`, human `human:<id>`, automated process `process:<id>`; `at` is the real UTC time
   (`date -u +"%Y-%m-%dT%H:%M:%SZ"`, fabricating it is forbidden). Content explicitly verified by the owner **should** append
   `verified: { by: human:<id>, at: … }`; if unverified, do not write it (absence means unverified — an honesty signal).
   Drafts use `status: draft`, finalized ones `stable` (default is stable), superseded-but-kept ones `deprecated`.
5. **Reserved filenames**: `index.md` (directory listing; the bundle root may only carry `okf_version`) and
   `log.md` (update history, first line `# Directory Update Log`, grouped by ISO `YYYY-MM-DD` dates,
   newest on top, entries starting with a bolded verb) are **forbidden** from use as concept documents.

## Bilingual documents (2026-11-12)

- Every concept document exists as `X.md` (English, primary) and `X.zh-CN.md` (Chinese); both carry a language-switch line at the top of the body;
- **English is primary/authoritative**; the Chinese twin is a faithful translation. Owner-given normative quotes (规范表述) stay verbatim in Chinese in BOTH versions, each followed by an English translation in the English version;
- **Any substantive change must be applied to both language versions** (same edit, or the immediately following edit recorded in `log.md`);
- `log.md` is a journal and is NOT translated (entries keep the language they were written in);
- `check-okf.py` enforces twin completeness (every non-reserved .md must have its language twin).

# Change workflow (run for every .md change in docs/)

1. Modify a document body → update its frontmatter `generated` (for substantive content changes; pure formatting tweaks may skip it but must be recorded in the log).
2. Create/delete/rename a document → update `index.md` in sync (the description is taken from that document's `description`).
3. Append an entry for the day to `log.md` (merge into the day's existing date group if one already exists).
4. Run `python3 docs/check-okf.py` — it must report 0 errors before the change counts as done.
5. Documents that pass owner review flip `status: draft` to `stable` and gain `verified`.

# Structure conventions

- Bundle root = `docs/` (the root `index.md` declares `okf_version: "0.2"`).
- The directory stays flat; open subdirectories only when topics grow, each with its own subdirectory `index.md`.
- Concept evolution path: a new normative statement from the owner → update the "Normative Statements" section of the corresponding concept document and sync the body → record in the log;
  superseded old statements are not deleted from history, but kept with a "revised" annotation.
