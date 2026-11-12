# mo2d engine (MVP + Phase B: performance & vocabulary rollout)

Headless implementation of the mo2d MVP — docs/mvp.md v5 — plus Phase B
(docs/roadmap.md): **performance machinery (B1 CPU side), the vocabulary
rollout (B2 brush / B3 gradient / B4 groups / B5 bitmap mesh / B6
simulation), format formalization (B7 packages, pack/unpack, schema,
migration chain), and the B1 GPU half on the **owner-approved wgpu path**
(`gpuwgpu.rs`): one instanced draw call through a WGSL uber-shader, the
**CPU/GPU pixel parity gate ACTIVE at ≥95% within tol 40 (measured 0.98,
p50/p90 = 0)**, and the 100k-node frame at **4.17 ms median in release**
(60 fps budget 16 ms) — the CPU oracle stays the byte-level source of
truth; the GPU is the parity-verified accelerator.
One sentence: **a moving tentacle (2D) + a z≠0 static scene (3D), built
entirely through the command protocol, AI and hand channels isomorphic,
deterministic execution, offline playable — now with brush styles, animatable
gradients, transform-box groups, and seeded simulation.**

Zero external dependencies by design: byte-level determinism must not depend
on third-party serializer behavior, and the engine must build offline.

## Build & test

```bash
cargo build --release          # binary: target/release/mo2d
cargo test                     # 34 unit tests + gates A1–A6 + gates B1–B7 + collisions + GPU parity/60fps + demos + throughput (113 total)
python3 ../demo/freeze.py      # re-freeze acceptance hashes (after intentional changes only)
```

The gate tests compare against `demo/frozen.json` (hash-freeze + human
eyeball policy, mvp.md P1-7). `demo/cmd/*.json` are the frozen command
streams — `tentacle-ai`, `scene3d`, plus the Phase B demos `brush-styles`,
`gradient-tentacle`, `stage-groups`, `sim-tail`, and `mesh-bitmap`; `demo/out/`
holds the regenerated artifacts (48-frame PNG sequence + manifest, GIF, 3D
scene render, one PNG per B demo, and the mesh media project in BOTH
container forms: `mesh-project/` (document.mo2d.json + assets/) and the
single-file `mesh-package.mo2d` — the freeze gate asserts the package renders
byte-identically to the directory form).

## CLI

```
mo2d replay   --stream FILE --out FILE       replay a command stream from scratch
mo2d apply    --doc FILE --stream FILE       apply onto an existing document
mo2d render   --doc FILE [--tick N] [--out PNG] [--camera JSON]
mo2d export   --doc FILE --dir DIR [--gif FILE] [--seconds N] [--from --to --step]
mo2d verify   --doc FILE                     schema + determinism gate
mo2d bench    [--nodes N]                    throughput (10k nodes < 16ms/frame; 100k eval ≈ 5ms)
mo2d stage    --doc FILE [--port N]          A6 minimal stage (four operations)
mo2d serve-mcp --doc FILE                    MCP server (apply/list/screenshot) on stdio
mo2d registry                                print the command registry
```

Camera JSON: `{"type":"ortho2d"}` or
`{"type":"perspective","eye":[x,y,z],"target":[x,y,z],"fovY":deg}`.

## The pieces (mvp.md §2 mapping + Phase B)

| Piece | Code |
|---|---|
| Line kernel, 3D-ready (D3 Vec3 nodes, D5 triangle faces, D6 renderer-is-3D) | `src/model.rs`, `src/render.rs` |
| Relation hot paths: bind / share / track (D1) + gradient, group, sim, particles (B3/B4/B6) | `src/eval.rs` |
| Reverse-dependency invalidation (B1: channels = signals) | `src/inval.rs` |
| Damage rendering — tile 64×32 + dirty rects, byte-parity with full render (B1) | `src/render.rs` |
| Brush 15-param family, node pressure, style templates (B2) | `src/model.rs`, `src/command.rs` |
| Bitmap mesh: PNG decode (zero-dep inflate), texture refs + uv anchors, budgeted asset cache (B5) | `src/inflate.rs`, `src/png.rs`, `src/assets.rs`, `src/render.rs` |
| Package form (header + `--BLOBS--`), pack/unpack, migration chain, JSON Schema (B7) | `src/format.rs`, `src/schema.rs` |
| Time = raw Ticks, 254,016,000,000/s (D7) | `src/model.rs` |
| Command protocol v1 (D2 versioned) + validator + determinism gate | `src/command.rs`, `src/validate.rs` |
| `.mo2d` envelope, byte-stable, unknown-key passthrough, atomic write | `src/format.rs` |
| Export: PNG sequence primary + GIF bonus | `src/export.rs`, `src/png.rs`, `src/gif.rs` |
| A6 minimal stage: four operations, zero controls; render cache (doc epoch+tick) | `src/stage.rs`, `src/stage.html` |
| AI channel: MCP server over stdio | `src/mcp.rs` |

## Command protocol v1

Stream envelope: `{"v":1,"cmds":[{...}, ...]}`. Every op is in
`mo2d registry`; highlights:

- `line.create` — `p:[x,y,z]` geometric or `p:[tick]` for track lines
  (a track IS a line: node = keyframe, `v` = value, segment `interp.im`
  0=Linear / 9=Bezier with per-component `{ao,ai,po,pi}` handles);
  `pr` = node pressure (B2), `brush` = line-level brush object (B2).
- `node.set {…, atTick}` — auto-key: upserts a keyframe when a track already
  drives that node for `p` (position), `w` (width) or `c` (color, B3).
- `track.create {line,node,attr?,tick?}` — the node→timeline connect gesture;
  creates track line + relation + first key. Attrs: `position` | `width` |
  `color` (B3 — what makes gradient stops animatable).
- `relation.create` —
  `bind` (projection-distance linear weights, offset rides the bone frame,
  optional stretch), `share` (welded nodes), `track`,
  **`gradient`** (B3: `{bar, target, role: fill|stroke}` — the bar line's
  nodes are the color stops; target nodes are recolored by projection onto
  the bar chord),
  **`group`** (B4: `{box, members, visible?}` — the transform-box line's
  rest→current similarity scales/rotates all members; nesting composes),
  **`sim`** (B6: `{line, anchors, gravity?, damping?, iterations?,
  collideWith?, radius?, selfCollide?}` — deterministic trailing rope with
  optional collisions, see below),
  **`particles`** (B6: `{emitter, node?, rate?, speed?, dir?, spread?,
  life?, gravity?, damping?, size?, color?, len?, seed?, collideWith?,
  bounce?, radius?}`).
- `texture` (line-level) + `uv` (node-level) — B5 bitmap mesh: an external
  image ref (resolved against the package blob area, then the document
  directory) with per-node texture anchors; textured fills sample
  perspective-correct bilinearly and modulate the fill color.
- `style.create / style.apply / style.delete` (B2) — named line-attribute
  templates; switching a line between brush strokes = applying different
  styles; undefined template fields leave the line untouched.
- Session class (`session.play/pause/seek`) drives playback without touching
  document history; document commands are all-or-nothing, undoable
  (inverse-carrying effects) and validated before save.

## Performance (B1 CPU side)

- **Reverse-dependency invalidation** (`inval.rs`): an edit dirties exactly
  its transitive downstream (channels = signals); hosts ask "what changed"
  instead of re-evaluating the world.
- **Damage rendering**: frames recomposite through a 64×32 tile grid — only
  tiles touched by changed lines are cleared and redrawn (scissored, fresh
  depth) from the same primitive list the full path uses, so damaged output
  is **byte-identical** to a full re-render (gate B1 parity, incl. 3D
  perspective cameras and fill-only changes).
- **Stage render cache**: `/frame` responses are keyed on (document epoch,
  tick) — idle polling costs no evaluation/render/encode (edit viewport
  idle at 0% CPU; the HTTP stage is pull-driven, it renders nothing unless
  asked).
- **Throughput** (debug build, Apple Silicon): 10k-node single-frame eval
  ≈ 0.5 ms; 100k-node ≈ 5 ms — both far inside the 16 ms budget (gates).
- **B1 GPU on wgpu** (`gpuwgpu.rs`, owner-approved dependency): ONE
  instanced draw call covers every stroke quad and fill triangle of the
  frame through a WGSL uber-shader; offscreen RGBA8 + blit readback;
  parity verified sample-wise (0.98 within tol 40, interiors exact). The
  zero-crate Metal FFI prototype that preceded it (device/pipeline/
  readback proof + the Metal 4 dialect findings — sRGB-store pixel format,
  Line-mode outlines, removed accessors, force-loaded Foundation) is
  documented in log.md; its findings transfer directly to any future
  raw-Metal work.

## Determinism

- Same doc + tick + camera → identical PNG bytes (gate A1, the B1 damage
  path is byte-identical to the full path, and the B7 package form renders
  byte-identically to the project-directory form).
- Texture cache state never affects output pixels (B5): decoded bytes are a
  pure function of the image file; eviction only touches memory.
- Same stream replayed → byte-identical document (gate A4/A5).
- **Simulation is replay-from-seed** (B6): every evaluation re-integrates
  from tick 0 in fixed 240 Hz substeps (254,016,000,000 / 240 = 1,058,400,000
  exactly) — scrub-back, replay and offline export match by construction.
  Anchored nodes are sampled per substep from their driving position tracks;
  particles hash their scatter angles through splitmix64 on the explicit
  `seed` parameter.
- Fixed iteration counts (bezier bisection ×48, curve flattening ×16, PBD
  constraint iterations = parameter), f64 math, stable field order, no
  timestamps, temp+rename atomic writes, exact keyframe hits return key
  values verbatim.

## Try the stage

```bash
target/release/mo2d stage --doc ../demo/out/tentacle.mo2d --port 7788
# open http://127.0.0.1:7788
```

Four operations (zero standard controls — the time baseline and playhead are
themselves lines):

1. **draw** — drag on empty canvas → `line.create` (drop the end on the start
   to close+fill);
2. **drag node** — drop on empty space → move/auto-key (`node.set` + atTick);
3. **connect** — line→line = `bind`; node dropped on another node = `share`;
   node dropped on the timeline = `track.create`;
4. **play** — tap the time baseline; drag it to scrub.

Every interaction is POSTed as a protocol command; `/history` returns the
recorded stream, which replays to the staged document byte-for-byte (gate A6).

## Known boundaries (by design)

- B1 GPU binding layer (see the prototype status above); 3D depth parity
  on the GPU likewise lands with it. The CPU oracle remains the source of
  truth; `render.rs` builds a primitive list that maps 1:1 onto the
  instanced uber-shader.
- Brush texture-stamp params (jitter/scattering/angle-drift/hue-drift …) are
  stored faithfully but only `sizeAmp`/`sizeScale`/`tint` are consumed by the
  CPU oracle today.
- Gradient fill colors are sampled at fill vertices (perspective-correct
  barycentric in between) — per-pixel gradient evaluation arrives with the
  uber-shader; straight control bars assumed (folded bars sort deterministically).
- Group transforms are 2D similarities (xy rotation + uniform scale, z scaled
  about the anchor); a static box drag edits the rest frame (members follow as
  layout) — animated/static transforms ride box tracks, same rest-relative
  rule as bind.
- Sim anchors resolve through first-level position tracks (deeper chains hold
  at the frame's position); colliders are held at the frame's positions during
  integration (push-out is a single deterministic pass in segment order, so
  heavily over-constrained contact configs may retain a residual violation).
- Linear/Bezier interpolation only; simple polygons for fills (nonzero-rule
  tessellation of self-intersecting contours is later); no guard-band
  clipping; GIF uses a fixed 3-3-2 palette (banding acknowledged, PNG primary).
- B5 textures decode PNGs with bit depth 8 (color types 0/2/6, non-interlaced);
  other PNGs are rejected with a clear error (fixtures are self-made).
- B7 blob names are single-line, tab-free; blob records are
  length-delimited and sorted, so package bytes are canonical.
