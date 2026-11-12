**English** | [简体中文](README.zh-CN.md)

# mo2d

**An AI-native 2D vector animation engine — everything is lines.**

A bone is a 2-node line segment; a skeleton is a polyline. A timeline is a set of
straight lines whose nodes are keyframes. Gradients are control bars (line segments).
Layers and groups are just collections of lines (predefined relations), and simulation
is relations between lines. With AI, standard toolbars and menus are no longer needed —
users define lines and the relations between them directly on the stage.

## Core ideas

1. **Everything is lines** — all structure and control reduce to lines;
2. **Relations are functions** — all behavior is relations between lines;
3. **The stage is the interface** — no standard widgets; direct manipulation + natural language;
4. **Concepts fit the GPU** — conceptual unification ⇒ uniform data ⇒ maximum GPU throughput.

## Repository layout

- [`engine/`](engine/README.md) — the Rust headless engine (zero third-party dependencies),
  full MVP: line kernel (3D-ready) / hot-path relations (track · share · bind) /
  Tick time / command protocol v1 + validator + determinism gate / `.mo2d` format +
  PNG export / minimal stage (four operations, zero widgets).

  ```bash
  cd engine && cargo test                # 113 tests: unit + gates A1–A6 + gates B1–B7 + collisions + GPU prototype + demos + throughput
  cargo run --release -- stage --doc ../demo/out/tentacle.mo2d   # open the minimal stage
  ```

- [`demo/`](demo/) — tentacle + z≠0 3D-scene command streams, 48-frame export + GIF,
  frozen hashes ([demo/frozen.json](demo/frozen.json)).

## Documentation

Docs are maintained in **English and 简体中文** (English is primary) and organized per
[Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format)
under [`docs/`](docs/index.md). Maintenance rules: [docs/AGENTS.md](docs/AGENTS.md);
after any edit run `python3 docs/check-okf.py` (must pass with 0 errors).
Change journal: [docs/log.md](docs/log.md) (kept in its original language).

Suggested reading order: [Vision](docs/vision.md) → **[Everything is Lines](docs/line-model.md)** →
[Moho Concept Mapping](docs/moho-mapping.md) → [Relations are Functions](docs/relations.md) →
[AI-Native Interaction](docs/ai-interaction.md) → [Architecture](docs/architecture.md) →
[File Format](docs/file-format.md) → [MVP](docs/mvp.md) → [Roadmap](docs/roadmap.md) →
[Glossary](docs/glossary.md)

## License

Dual-licensed under **MIT OR Apache-2.0** at your option — see
[LICENSE-MIT](LICENSE-MIT) / [LICENSE-APACHE](LICENSE-APACHE).
**Both require attribution**: distributing this software (in whole or substantial part,
with modifications) MUST retain the copyright and license notices, and
Apache-2.0 §4.1(d) additionally requires the [NOTICE](NOTICE) file to be carried verbatim.
Copyright © 2026 defims and the mo2d contributors.
Contributions are accepted under this dual license (see [AGENTS.md](AGENTS.md)).
