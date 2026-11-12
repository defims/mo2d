**English** | [简体中文](AGENTS.zh-CN.md)

# mo2d Engineering Rules (read this first — every session / contributor)

mo2d — an AI-native 2D vector animation engine (everything is lines).
**The documentation library lives in [`docs/`](docs/index.md)**, organized per
Open Knowledge Format v0.2. Read [docs/AGENTS.md](docs/AGENTS.md) before editing
any .md under docs/; after every edit run `python3 docs/check-okf.py`
(the change is not done until it passes with 0 errors). Docs are bilingual
(English primary + 简体中文 twin) — keep both in sync.

## License & attribution

- This project is dual-licensed **MIT OR Apache-2.0** (see `LICENSE-MIT` / `LICENSE-APACHE`),
  copyright `defims and the mo2d contributors`;
- **Both licenses enforce attribution**: redistribution must retain the copyright and
  license notices; the root **NOTICE** file must be carried verbatim per Apache-2.0 §4.1(d).
  NOTICE changes require owner approval; contributors must never remove or alter attribution;
- New source files should carry the SPDX header: `SPDX-License-Identifier: MIT OR Apache-2.0`;
- Document authorship goes through OKF `generated: { by, at }` trust stamps
  (agents `zcode-agent/<model>`, humans `human:<id>`); git commits use the repo identity
  (`Long, Wei <lostaim@gmail.com>`); session work is recorded in docs/log.md.

## GPL isolation (hard redlines)

1. **Never copy, translate, or derive from GPL-licensed code** — in particular
   [BUI](https://github.com/ppmpreetham/BUI) (GPL-3.0, contains Blender GPL-2+ derivatives)
   and Blender sources: BUI may only be referenced for the **design ideas** described in its
   public documentation (damage rendering / instanced uber-shader etc., see
   [docs/architecture.md](docs/architecture.md) §3.1);
2. Review checkpoint: any unusual implementation structurally identical to BUI/Blender code
   must be traced to its origin; without a clean explanation, rewrite it;
3. Dependency rule: third-party crates/packages must be permissively licensed
   (MIT/Apache-2.0/BSD/MPL); GPL/AGPL/LGPL dependencies are forbidden
   (the CPU oracle and byte-level determinism stay dependency-free; the
   owner-approved GPU accelerator path added **wgpu** (Apache-2.0 OR MIT) and
   **pollster** (Apache-2.0 OR MIT) on 2026-11-12 — new GPU-adjacent
   dependencies still need owner sign-off).

## Moho corpus redlines

1. `/Users/wei/moho user content/` (84 official tutorial projects) and any
   Moho-owned fixtures: **never enter this repository**; CI/test fixtures are always self-made;
2. mo2d consumes **format facts** from the maohou knowledge base (field names/defaults/
   channel codes — facts are not copyrightable), never any Moho binary-analysis artifacts;
   mo2d itself never runs or automates Moho;
3. Public-material wording: never claim "a replica of / compatible with Moho", never imply
   Moho endorsement ([docs/legal-risks.md](docs/legal-risks.md) R2/R3/R4).

## Quality gates

- `cd engine && cargo test` must stay 47+ green; `python3 docs/check-okf.py` must report 0 errors;
- Anything that breaks byte-level determinism (hashes/timestamps/iteration order) will be
  caught by the frozen gate ([demo/frozen.json](demo/frozen.json)) — never bypass it; fix the cause.
