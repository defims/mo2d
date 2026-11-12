# mo2d engine (MVP: a moving tentacle + a z≠0 static scene)

Headless implementation of the mo2d MVP — docs/mvp.md v5. One sentence:
**a moving tentacle (2D) + a z≠0 static scene (3D), built entirely through the
command protocol, AI and hand channels isomorphic, deterministic execution,
offline playable.**

Zero external dependencies by design: byte-level determinism must not depend
on third-party serializer behavior, and the engine must build offline.

## Build & test

```bash
cargo build --release          # binary: target/release/mo2d
cargo test                     # 22 unit tests + gates A1–A6 + throughput
python3 ../demo/freeze.py      # re-freeze acceptance hashes (after intentional changes only)
```

The gate tests compare against `demo/frozen.json` (hash-freeze + human
eyeball policy, mvp.md P1-7). `demo/cmd/*.json` are the frozen command
streams; `demo/out/` holds the regenerated artifacts (48-frame PNG sequence +
manifest, GIF, 3D scene render).

## CLI

```
mo2d replay   --stream FILE --out FILE       replay a command stream from scratch
mo2d apply    --doc FILE --stream FILE       apply onto an existing document
mo2d render   --doc FILE [--tick N] [--out PNG] [--camera JSON]
mo2d export   --doc FILE --dir DIR [--gif FILE] [--seconds N] [--from --to --step]
mo2d verify   --doc FILE                     schema + determinism gate
mo2d bench    [--nodes N]                    throughput (10k nodes < 16ms/frame)
mo2d stage    --doc FILE [--port N]          A6 minimal stage (four operations)
mo2d serve-mcp --doc FILE                    MCP server (apply/list/screenshot) on stdio
mo2d registry                                print the command registry
```

Camera JSON: `{"type":"ortho2d"}` or
`{"type":"perspective","eye":[x,y,z],"target":[x,y,z],"fovY":deg}`.

## The pieces (mvp.md §2 mapping)

| Piece | Code |
|---|---|
| Line kernel, 3D-ready (D3 Vec3 nodes, D5 triangle faces, D6 renderer-is-3D) | `src/model.rs`, `src/render.rs` |
| Relation hot paths: node sharing / bind / track (D1 track=line) | `src/eval.rs` |
| Time = raw Ticks, 254,016,000,000/s (D7) | `src/model.rs` |
| Command protocol v1 (D2 versioned) + validator + determinism gate | `src/command.rs`, `src/validate.rs` |
| `.mo2d` envelope, byte-stable, unknown-key passthrough, atomic write | `src/format.rs` |
| Export: PNG sequence primary + GIF bonus | `src/export.rs`, `src/png.rs`, `src/gif.rs` |
| A6 minimal stage: four operations, zero controls | `src/stage.rs`, `src/stage.html` |
| AI channel: MCP server over stdio | `src/mcp.rs` |

## Command protocol v1

Stream envelope: `{"v":1,"cmds":[{...}, ...]}`. Every op is in
`mo2d registry`; highlights:

- `line.create` — `p:[x,y,z]` geometric or `p:[tick]` for track lines
  (a track IS a line: node = keyframe, `v` = value, segment `interp.im`
  0=Linear / 9=Bezier with per-component `{ao,ai,po,pi}` handles).
- `node.set {…, atTick}` — auto-key: upserts a keyframe when a track already
  drives that node (this is what the stage's drag emits; AI uses the same op).
- `track.create {line,node,attr?,tick?}` — the node→timeline connect gesture;
  creates track line + relation + first key. Deterministic auto id
  `trk_<line>_<node>_<attr>`.
- `relation.create` — `bind` (projection-distance linear weights, offset rides
  the bone frame, optional stretch), `share` (welded nodes), `track`.
- Session class (`session.play/pause/seek`) drives playback without touching
  document history; document commands are all-or-nothing, undoable
  (inverse-carrying effects) and validated before save.

## Determinism

- Same doc + tick + camera → identical PNG bytes (gate A1).
- Same stream replayed → byte-identical document (gate A4/A5).
- Fixed iteration counts (bezier bisection ×48, curve flattening ×16), f64
  math, stable field order, no timestamps in the body, temp+rename atomic
  writes, exact keyframe hits return key values verbatim.

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

## Known MVP boundaries (by design, see docs/mvp.md §5)

CPU oracle only (GPU is B1); Linear/Bezier only; single-bone rigid/linear
bind; full re-evaluation (no invalidation); simple polygons for fills
(nonzero-rule tessellation of self-intersecting contours is later); no
guard-band clipping (build on-canvas or in front of the camera); GIF uses a
fixed 3-3-2 palette (banding acknowledged, PNG is primary).
