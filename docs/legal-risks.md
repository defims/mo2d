---
type: Risk Register
title: "Legal Risk Register (2026-11-12 audit)"
description: Ten legal risks for the mo2d project — GPL isolation (BUI/Blender), the Moho trademark and "mo" naming, EULA/clean-room discipline, the tutorial-corpus redline, copyright in format fact tables, reference-project licenses (MIT/Apache verified on disk), a no-patent-search disclosure, own license pending, AI privacy as a long-term item; each with level/mitigation/redline and a to-do list.
tags: [legal, risk-register, license, gpl, trademark, clean-room]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T07:10:24Z }
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

**English** | [简体中文](legal-risks.zh-CN.md)

# Legal Risk Register (2026-11-12 audit)

The owner asked for a careful review of this project's legal risks. Method: identify item by item → gather on-disk evidence → level/mitigation/redline.
**Evidence conclusion first**: vectorcraft and maohou (an AniCraft fork) both hold
`LICENSE-MIT` + `LICENSE-APACHE` on disk (MIT OR Apache-2.0) [^vc-license][^maohou-license] —
the existing documents' claim that "vectorcraft code may legally be used as a reference" **holds**;
mo2d itself **has no LICENSE file and no git repository**.

## 1. Risk summary table

| # | Risk | Level | Status |
|---|---|---|---|
| R1 | GPL contamination (BUI/Blender derivative code mixed in) | **High** (if it ever happens) | Rules established; needs checklist items in the repository |
| R2 | Moho trademark and "mo" naming adjacency | Medium | Mitigation in progress (codename positioning); must be handled before productization |
| R3 | Moho EULA/reverse-engineering boundary (via the maohou research chain) | Low-Medium | Dependency-chain facts recorded; EULA clauses unreviewed |
| R4 | Tutorial corpus (84 .moho files) leaking out | Low (internal) | Redline to be written into engineering standards |
| R5 | Compilation copyright in format fact tables (channel codes etc.) | Low | Sourced citations in place; keep attribution when made public |
| R6 | Misreading of reference-project licenses | Eliminated | Verified on disk (2026-11-12) |
| R7 | Patents (skeletal animation/skinning/interpolation mechanisms) | Not searched | Honest disclosure; FTO before productization |
| R8 | mo2d's own license undecided + no LICENSE | Resolved | 2026-11-12 repository established (git + dual license + NOTICE), see §2 R8 |
| R9 | AI-channel privacy (user documents sent to third-party models) | Long-term | Privacy policy at productization |
| R10 | maohou results embedded in the documents | None | The same owner's own work |

## 2. Item-by-item analysis

### R1 GPL contamination (BUI / Blender) — highest priority

- **Facts**: BUI is GPL-3.0 and self-describes as containing Blender GPL-2+ derivative code [^bui-repo];
  this project once referenced its performance design (damage rendering/instanced uber-shader/parity gate,
  [architecture.md](architecture.md) §3.1).
- **Current state**: the documents only describe its design ideas in our own words and cite the source — ideas/facts are not
  copyright-protected, so the risk is low; **the redline already written in §3.1 is "borrow the design only, never the implementation"**.
- **Residual risk**: "casually" consulting BUI source code, or translating its implementation section by section, when writing code in the
  future — GPL contamination; if mo2d goes MIT/Apache, it breaks.
- **Mitigation (to-do)**: ① when creating the repository, write a **GPL isolation clause** into AGENTS/CONTRIBUTING
  (no copying/translating BUI, Blender, or any GPL code; BUI references are limited to public documents);
  ② a code-review checklist item: any unconventional implementation found isomorphic to BUI/Blender triggers a provenance inquiry.

### R2 Moho trademark and "mo" naming — must be handled before productization

- **Facts**: the maohou naming memo has already ruled: "the mo- family of roots is fenced off (mo2d/homo/mhp/morph/morpho)";
  mo2d.com may serve only as a neutral auxiliary domain, never as a brand — the trademark is weak (a generic 2D word + Moho adjacency) [^naming].
  That ruling targets the maohou brand, but **the trademark-weakness analysis applies equally to mo2d as a product name**.
- **Current state**: this project's documents already position "mo2d as the engine **codename**, product branding to be decided separately"
  (a vision open item) — correct.
- **Residual risks**: ① if mo2d is brought to market for sale, there is a risk of confusion claims against the Moho trademark
  (Lost Marble/Smith Micro); ② the documents' extensive references to Moho are **nominative fair use**
  (describing interoperability facts), but public materials should avoid "replica/Moho-compatible" marketing wording
  (mo2d is not a replica anyway; just state the truth); ③ any wording implying Moho endorsement/affiliation is forbidden.
- **To-do**: run a trademark search before productization (France/US/China depending on the market); the brand must not land on a mo- family root.

### R3 Moho EULA / reverse-engineering boundary (dependency-chain facts, recorded as they are)

- **Facts**: all of mo2d's Moho knowledge comes from the **clean-room research** of the maohou knowledge base (same owner):
  the field tables come from JSON empirical work on 8 self-made fixtures + the mohoscripting.com community docs;
  but the maohou process includes a "**live-app gate**" (a legitimately licensed Moho 14.4 on this machine; GUI automation only opens and
  never saves; Lua generates fixtures; FileRender produces comparison images) — this is **black-box use**, essentially different from
  decompilation; **the reverse-engineering clauses of the Moho EULA have not been reviewed** (this register assumes nothing about their content).
- **Clean-room discipline on the mo2d side (should be written into engineering standards)**: ① consume the **format facts**
  in maohou documents (field names/defaults/channel codes — facts are not copyrightable) and bring in no products of Moho binary analysis;
  ② the long-term .moho importer (C3) will be implemented only from public-document facts; ③ mo2d itself does not run/automate Moho.
- **To-do (low priority)**: if .moho import becomes an official feature, have a lawyer review the Moho EULA's interoperability clauses
  (in most jurisdictions, using format facts for interoperability purposes is defensible, but a legal opinion should be kept on file).

### R4 Tutorial-corpus redline (84 .moho files)

- **Facts**: `moho user content/` consists of official Moho tutorial projects (copyrighted sample content);
  roadmap C3 plans to use them as the import-acceptance corpus.
- **Verdict**: internal test use = low risk; **redlines**: ① they never enter any public repository
  (maohou had the same lesson, "corpus ingestion undecided"); ② CI fixtures must be self-made,
  not derived from tutorial files; ③ they are never distributed with mo2d.
- **To-do**: write into the .gitignore and contribution guidelines once the repository exists.

### R5 Format fact tables (thin compilation copyright)

- The channel-code tables/field tables are **facts** (numbers and names, not copyrightable); their **selection and arrangement** may carry
  thin copyright (maohou's compilation). The mo2d documents already cite sources everywhere (footnotes → maohou/mohoscripting);
  this project and maohou share the same owner (R10), so there is **no third-party risk at present**; if the documents go public,
  keeping the attributions suffices; the tables in moho-mapping are mo2d's own re-arrangement.

### R6 Misreading of reference licenses — eliminated

vectorcraft and maohou are both MIT OR Apache-2.0 (dual-license files verified on disk, evidence gathered 2026-11-12)
[^vc-license][^maohou-license] — "vectorcraft format-implementation code may legally be used as a reference" holds;
even directly porting its BLOB area/atomic-write implementation is compatible with the MIT/Apache that mo2d plans to adopt
(retaining the copyright notices is enough).

### R7 Patents — not searched (honest disclosure)

Foundational techniques such as skeletal animation/LBS skinning/Bezier interpolation mostly date from the 1980s–90s, and the core patents
have most likely expired; but **no search at all has been done for application-layer patents (smart bones, specific deformation
algorithms, PBD variants)**. This register offers no patent opinion; **to-do**: perform an FTO (freedom-to-operate) search at the
productization/fundraising milestone.

### R8 mo2d's own license — **resolved (2026-11-12, landed once the owner provided the remote)**

`git init` (main branch) + remote `github.com/defims/mo2d`; **LICENSE-MIT + LICENSE-APACHE dual licensing**
(copyright attribution `defims and the mo2d contributors`, consistent with the vectorcraft/maohou ecosystem);
the root `AGENTS.md` was written with the GPL isolation clause, the Moho corpus redline, SPDX rules, and the attribution policy;
`.gitignore` excludes session artifacts and build artifacts. Action items 1–3 are all closed.

### R9 AI-channel privacy — long-term

After productization, if the natural-language→command AI channel sends user-document content to third-party models,
a privacy policy and a data-flow statement will be needed; a local-model/MCP architecture can minimize this.
A long-term to-do; it does not block engineering.

### R10 maohou results embedded in the documents — no risk

The maohou knowledge base is the same owner's own work on this machine (its upstream AniCraft is MIT/Apache);
cross-project reuse raises no third-party rights issues — recorded on file.

## 3. Immediate action list

| # | Action | Timing |
|---|---|---|
| 1 | ~~Write the GPL isolation clause into the repository AGENTS~~ | ✅ 2026-11-12 root AGENTS.md |
| 2 | ~~Add LICENSE~~ | ✅ 2026-11-12 MIT OR Apache-2.0 dual license |
| 3 | ~~Tutorial-corpus redline~~ | ✅ 2026-11-12 AGENTS.md §Moho corpus redline + .gitignore |
| 4 | Wording review of public materials: no "replica/Moho-compatible", no implied endorsement | Before first publication |
| 5 | Trademark search + branding decision (avoid the mo- family of roots) | Before productization |
| 6 | Legal opinion on the Moho EULA interoperability clauses | When .moho import is chartered |
| 7 | Patent FTO search | Productization/fundraising milestone |
| 8 | AI privacy policy | At productization |
