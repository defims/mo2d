---
type: Design Document
title: "File Format: Design Recommendations and References"
description: Design of the mo2d native format — three-layer format architecture (.mo2d single-file deliverable package = delivery equals project / optional project-directory (unpacked) form / SVG frame view), trailing BLOB area, Tick integer time, byte-level determinism, extension namespaces, JSON Schema validation and version-migration discipline; the reference table covers glTF 2.0/vectorcraft/maop/Moho/Lottie/OpenRaster/AniCraft/USD/Collada, each annotated with what to take and what to avoid.
tags: [file-format, storage, reference, gltf, ticks, svg-view]
status: draft
generated: { by: zcode-agent/glm-5.3, at: 2026-11-12T10:34:00Z }
sources:
  - id: gltf
    resource: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html
    title: "glTF 2.0 规范(Khronos)——asset.version/扩展机制/JSON+BIN 分离/官方校验器"
  - id: vc-format
    resource: file:///Users/wei/vectorcraft/crates/format/src/lib.rs
    title: "vectorcraft .vectorcraft 格式实现(信封/尾部 BLOB 区/原子写/未知键透传;MIT/Apache 可合法参照代码)"
  - id: maop-spec
    resource: file:///Users/wei/maohou/plan/moho/maop-spec.md
    title: ".maop 规范 v1(Tick 时间模型/键升序与读端修复/时间钳制)"
  - id: mohoproj-format
    resource: file:///Users/wei/maohou/plan/moho/mohoproj-format.md
    title: ".mohoproj 格式规范(双形态/保存剥外来键的教训/documentviewstate 混入文档的反例)"
  - id: lottie
    resource: https://lottie.github.io/lottie-spec/
    title: "Lottie(JSON 动画互换格式)——关键帧 JSON 编码的工业对照"
  - id: ora
    resource: https://www.openraster.org/
    title: "OpenRaster(ORA)——ZIP 容器纪律(mimetype 首条未压缩/合成预览/层文件)"
  - id: anicraft
    resource: file:///Users/wei/maohou/README.md
    title: "AniCraft .anicraft FORMAT_VERSION v4→v7 逐版读端迁移的实践样本"
  - id: line-model-doc
    resource: /line-model.md
    title: "本 bundle:万物皆线"
  - id: arch-doc
    resource: /architecture.md
    title: "本 bundle:架构草案(§4 存储格式原则)"
---

**English** | [简体中文](file-format.zh-CN.md)

# File Format: Design Recommendations and References

The owner asked: any recommendations and suggestions for the file format, and which projects and format
specifications are worth referencing.
This document gives a recommended design (an envelope draft) + a reference table (each reference annotated
with **what to take and what to avoid**),
and turns the five principles of [architecture.md](architecture.md) §4 into executable details.

## 1. Requirements (dictated by the mo2d model)

The format must carry: a line collection (nodes/segments/three-level attributes), the relation graph
(predefined + custom, including function-body payloads), channels and tracks, root parameters, media/logic
payloads, external-asset references, and command history. Constraints:

1. **AI-readable and AI-writable**: an AI-native engine — the format is the agent's "source code"; it must be
   semantically named, diffable, and hand-editable;
2. **Determinism**: same document → same bytes (good for git diff, command-replay hashes, two-renderer comparison);
3. **Version evolution**: adding fields does not change readers; upgrading versions has a migration path;
4. **Large payloads**: SoA arrays on the order of a hundred thousand nodes and bitmaps must not be bloated by base64;
5. **git-friendly**: text first; binary appended at the end or kept external.

## 2. Recommended design (mo2d envelope draft)

**Primary format = the `.mo2d` single-file deliverable package** (UTF-8 JSON header + optional trailing BLOB
area; embedded media without base64) — **delivery equals project**: one file is both self-contained for
double-click delivery, and its JSON header is git-diffable and directly AI-readable.
**Optional project-directory (unpacked) form**: a `project/` directory = `document.mo2d.json` + `assets/`
(refs kept external), for heavy git collaboration / large projects; the engine treats the two forms as
**equivalent** (one document, two containers). Matching common practice: the primary extension = the
deliverable package (Moho's .moho is a ZIP; likewise .kra/.ora/.docx; with its trailing-BLOB design, mo2d's
package is more git/AI-friendly than a ZIP)[^mohoproj-format].
~~.mo2dz~~ is not defined (revisit if a need for multi-scene packaging arises).

```jsonc
{
  "format": "mo2d",
  "version": 1,               // Document format version (monotonically increasing integer; a semantic-version tag to be discussed separately)
  "minReader": 1,             // Readers older than this version should refuse (glTF asset.minVersion style)[^gltf]
  "generator": "mo2d 0.1.0",
  "root": {                   // Root-relation parameters (canvas/ticksPerSecond/color/export)
    "width": 1920, "height": 1080,
    "ticksPerSecond": 254016000000,   // empirically verified maop value[^maop-spec]
    "fps": 24.0                       // display/export parameter only; the source of truth for time is Ticks
  },
  "lines":    [ { "id": "l1", "tags": ["bone"], "attrs": {...},
                 "nodes": [ {"p": [x,y,z], "attrs": {...}} ],   // in 2D z is always 0 (Vec3 from day one, per the 3D-extension decision)
                 "segAttrs": [ {...} ] } ],
  "relations":[ { "id": "r1", "kind": "predefined", "type": "bind",
                 "inputs": [...], "params": {...}, "body": {...}? } ],
  "payloads": { "media": {...}, "logic": {...} },
  "refs":     [ { "id": "img1", "path": "assets/a.png", "relativeTo": "file" } ],
  "history":  [ /* command stream, optional; can be split into a sidecar */ ],
  "extensions": { "vendor.feature": { ... } }   // glTF-style extension namespace[^gltf]
}
```

Eight rules:

1. **Time = bare Tick integers** (254,016,000,000 per second = 504,000², one tick ≈ 3.9 picoseconds;
   range ≈ ±105 days, out-of-range values clamped and the repair recorded; keys must be in ascending time
   order, the reader sorts out-of-order keys and records the repair) — taken wholesale from maop's empirical
   results[^maop-spec], avoiding floating-point frame drift; fps is only a root parameter.
   **Why this number** (verified by calculation, 2026-11-11): 24/25/30/48/50/60/120/240 fps,
   milliseconds/microseconds, NTSC 30000/1001 (= 8,475,667,200 exactly), and audio sample rates
   44.1k/48k/96k/192k all divide it evenly — frame ↔ tick ↔ audio-sample conversions are all exact integer
   arithmetic; the value range is clamped to i64/4 (±105 days), leaving 2 bits of headroom so time
   differences / intermediate interpolation calculations do not overflow;
2. **Trailing BLOB area** (vectorcraft-style): after the JSON header come a magic number and byte areas;
   payload entries write `"blob": [offset, length]` instead of base64; small items (below the INLINE_MAX
   threshold) stay inline — vectorcraft is MIT/Apache, so **even its code can legally be referenced**[^vc-format];
3. **Media external by default** (refs + a relative base), embedding optional; preview thumbnails optional,
   regenerated on save, and **excluded from the canonical hash** (the same as Moho's
   thumbnail/preview.jpg)[^mohoproj-format];
4. **Byte-level determinism**: stable field order (written in canonical order; no sorting, no compression),
   no timestamps in the body (modified_date and the like go into the envelope only), atomic writes
   (temp + rename)[^vc-format];
5. **Unknown-key passthrough + extension namespaces**: keys the reader does not recognize are preserved and
   written back as-is; third-party extensions always go into `extensions.{vendor.feature}` and, like glTF,
   are declared in `extensionsUsed`[^gltf] — learning from Moho's lesson of **stripping foreign keys on
   save**: mo2d explicitly promises **not to strip**[^mohoproj-format];
6. **JSON Schema + validator**: the format specification comes first (an OKF document, written in the style
   of maop-spec), and the repository carries the schema and a validation command (glTF validator culture)[^gltf];
7. **Version migration on the reader**: FORMAT_VERSION rises one version at a time, and old versions are
   still read — AniCraft v4→v7 is a living sample (with each version bump, the old readers are kept)[^anicraft];
8. **history separated from document**: document is the **truth**; history is only for audit/replay;
   when the two diverge, document wins (avoiding two competing truths, "replay vs state").

**Engine status vs this draft (2026-11-12, Phase B)**: the MVP envelope plus the B7 package form are
live. Single-file `.mo2d` packages = canonical JSON header + `--BLOBS--` marker line + length-delimited
blob records (sorted names; a real marker line cannot occur inside a JSON string — raw newlines are
escaped); `mo2d pack`/`mo2d unpack` convert between the package and the project directory
(`document.mo2d.json` + `assets/`), losslessly (pack ∘ unpack ∘ pack is byte-identical), and texture
references resolve against the blob area first, then the filesystem — **two containers, one document**
(the frozen mesh demo is pinned in both forms and renders byte-identically). Phase B vocabulary rides
version 1 additively: top-level `styles` array (B2 — **emitted only when non-empty** so legacy bytes
are unchanged), line `brush{…}`, node `pr`/`uv`, line `texture`, relation types
`gradient`/`group`/`sim`/`particles`, track attr `color` — all consumed by the per-type unknown-key
filter (rule 5: foreign keys of any name pass through). The reader carries an empty **MIGRATIONS**
chain indexed by version (rule 7's mechanism, first bump will push v1→v2); newer versions are refused,
never guessed. `mo2d schema` emits the v1 JSON Schema; history-in-file remains future scope.

## 3. Reference table: projects and specifications

| Reference | License | What to take | What to avoid / lessons |
|---|---|---|---|
| **glTF 2.0** (Khronos)[^gltf] | open specification | the `asset.version/minVersion` mandatory discipline; the extensionsUsed/Required mechanism; JSON header + BIN data separation; official JSON Schema + validator culture | its 3D concept system (accessor/bufferView indirection layers) is too heavy for mo2d — a simplified trailing BLOB area is enough |
| **vectorcraft `.vectorcraft`**[^vc-format] | MIT/Apache (**code can legally be referenced**) | envelope shape; trailing BLOB area (magic number + offset/length + INLINE_MAX threshold); atomic writes; the `document.unknown` slot for foreign data; step-by-step v1→v4 evolution | — |
| **maohou `.maop` spec**[^maop-spec] | local knowledge base | the **Tick time model** (bare integers / 254,016,000,000 per second / clamp-and-repair); keys in ascending time order; specification-first documents with a per-field must-write/never-write annotation style | its double-written extension keys (standard + extension) exist to accommodate Moho's key stripping — mo2d has no such baggage and writes once |
| **Moho `.moho/.mohoproj`**[^mohoproj-format] | factual reference (no code referenced) | two forms coexisting (ZIP container + single-file JSON); preview either external or embedded; lenient readers (unknown-key tolerance) | ① **strips foreign keys** on save; ② editor state (documentviewstate 48 keys / onion skin) leaks into the document — mo2d already separates session state, the format carries document only; ③ container quirks such as empty layer lists being rejected |
| **Lottie** (JSON animation)[^lottie] | open specification | an industrial comparison for the JSON encoding of keyframes/interpolation/precomps; the most realistic exchange target if Web interoperability is wanted later | no strict schema, accretion of historical concepts; its "shape = layer tree" does not fit the line model — borrow the encoding, not the model |
| **OpenRaster (ORA)**[^ora] | open specification | ZIP container discipline: mimetype as the first, uncompressed entry; mergedimage.png preview; separate layer files — follow this if a packaged form is ever defined | — |
| **AniCraft `.anicraft`**[^anicraft] | local (fork) | FORMAT_VERSION per-version reader migration + sidecar files carrying compatibility data | — |
| **USD** | open specification | layered composition/references (worth another look if collaborative composition is done in the far future) | too heavy — mo2d's referencing needs are met by the refs relation; not introducing it now |
| **Collada / .blend DNA** | — | (.blend's self-describing versioning is a good idea) | Collada = a lesson in XML over-engineering; .blend's binary self-description conflicts with "AI-readable" — both are negative references |
| **BUI** (GPL)[^arch-doc] | **GPL-3.0** | (for the performance design see architecture §3.1) | no reference value for the format; GPL, code cannot be referenced |

## 4. Three-layer format architecture (established 2026-11-11 after the owner asked "can the project be represented directly in SVG")

**Display equals edit holds** (multiple clients of the same rendering pipeline: the stage / headless export /
AI sessions all render the same document, and overlays are lines too), but **the engineering source of truth
cannot be SVG** — SVG is "the image of one frame", while the project is "the cause of all frames":

```
① Primary format (delivery equals project)  .mo2d       single file: JSON header + trailing BLOB area (media embedded); everyday save/share/delivery
② Project-directory form (optional)         project/    document.mo2d.json + assets/ (refs external); heavy git collaboration; the two containers are equivalent
③ Frame view                                SVG         lossless display of any single frame; the model is lines ⇒ in principle re-importable as geometric lines (relations/tracks do not come along)
```

Five hard flaws of SVG as a project format: ① stroke-width is a single value per path — **no per-node width
channel** (variable-width lines must be expanded into filled outlines, losing the structure), let alone
per-point colors / segment-level attributes; ② path `d` is an instruction string, not a node table — node
IDs/handles/segment boundaries cannot round-trip, while binding/tracks/commands rely on **addressing by node
ID**; ③ no bones/binding/constraints/tracks — two-thirds of the document model has nowhere to live; ④ no time
model (SMIL is deprecated); ⑤ XML has no canonical byte form — byte-level determinism is impossible, and
large scenes parse slowly.
**SVG's correct role**: a per-frame view/exchange format (export = the natural byproduct of display equals
edit; LLMs know SVG extremely well, so it can also serve as the AI's geometry exchange channel) — the
roadmap C3 SVG import plays exactly this role, plus frame export.

## 5. Relation to the existing documents

- The five principles of [architecture.md](architecture.md) §4 (envelope / semantic versioning / unknown-key
  passthrough / embedded command history / external assets) are turned into the eight rules in this document;
  if they disagree, this document prevails and §4 will be amended accordingly;
- The encoding semantics of tracks/keyframes/segments are in [moho-mapping.md](moho-mapping.md) §5 (channel
  objects and the interp field family);
- The formal format specification (per-field "must-write/default/never-write" annotations, in the maop-spec
  style) starts in roadmap Phase A5; the MVP only needs the minimal subset of this document's envelope
  (lines/relations/root/refs, without the BLOB area).

## 6. Open Questions

- [ ] Whether `version` uses a monotonically increasing integer (AniCraft-style) or a semantic-version string (glTF-style) — leaning toward integer + changelog;
- [ ] The concrete values of the BLOB-area magic number and the INLINE_MAX threshold (vectorcraft's constants can be adopted directly);
- [ ] ~~Whether `.mo2dz` is needed in the first version~~ (decided: **not defined**; corrected by the owner per common practice, 2026-11-12 — the primary extension is itself the deliverable package);
- [ ] Where the function body of custom relations (the logic payload) lives in the format: inside document vs. an external script in refs.
