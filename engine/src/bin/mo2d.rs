//! mo2d CLI — every capability is reachable headless:
//! apply / replay / render / export / verify / bench / stage / serve-mcp.

use mo2d::command;
use mo2d::eval;
use mo2d::eval::{EvalLine, EvalNode};
use mo2d::export::{export_frames, export_gif, ExportOpts};
use mo2d::format;
use mo2d::hull;
use mo2d::json::{self, Value};
use mo2d::mcp::{parse_camera, McpSession};
use mo2d::math::Vec3;
use mo2d::model::{Document, SegAttrs, TICKS_PER_SECOND};
use mo2d::png::encode_png;
use mo2d::render::{render, render_with_assets, Camera, RenderOpts};
use mo2d::sha::sha256_hex;
use mo2d::stage::{serve, StageState};
use mo2d::validate;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let cmd = args[0].as_str();
    let rest = &args[1..];
    let result = match cmd {
        "apply" => cmd_apply(rest),
        "replay" => cmd_replay(rest),
        "render" => cmd_render(rest),
        "export" => cmd_export(rest),
        "verify" => cmd_verify(rest),
        "bench" => cmd_bench(rest),
        "stage" => cmd_stage(rest),
        "serve-mcp" => cmd_serve_mcp(rest),
        "registry" => cmd_registry(),
        "hash" => cmd_hash(rest),
        "pack" => cmd_pack(rest),
        "unpack" => cmd_unpack(rest),
        "hull" => cmd_hull(rest),
        "schema" => cmd_schema(),
        "help" | "--help" | "-h" => {
            usage();
        }
        other => {
            eprintln!("unknown command '{}'", other);
            usage();
        }
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        exit(1);
    }
}

/// B7 pack: project directory (document.mo2d.json + assets/*) → package.
fn cmd_pack(args: &[String]) -> Result<(), String> {
    let dir = Path::new(require(args, "--dir"));
    let out = require(args, "--out");
    let json_path = dir.join("document.mo2d.json");
    let text = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("read {}: {} (a project directory carries document.mo2d.json + assets/)", json_path.display(), e))?;
    let doc = format::parse_doc(&text)?;
    let mut blobs: Vec<(String, Vec<u8>)> = Vec::new();
    let assets_dir = dir.join("assets");
    if assets_dir.is_dir() {
        let mut names: Vec<String> = std::fs::read_dir(&assets_dir)
            .map_err(|e| format!("read {}: {}", assets_dir.display(), e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        for name in names {
            let data = std::fs::read(assets_dir.join(&name)).map_err(|e| format!("read assets/{}: {}", name, e))?;
            blobs.push((format!("assets/{}", name), data));
        }
    }
    let bytes = format::doc_to_package_bytes(&doc, &blobs)?;
    format::write_atomic(Path::new(out), &bytes)?;
    println!("packed {} ({} blobs) -> {} (sha256 {})", dir.display(), blobs.len(), out, sha256_hex(&bytes));
    Ok(())
}

/// B7 unpack: package → project directory (document.mo2d.json + assets/*).
fn cmd_unpack(args: &[String]) -> Result<(), String> {
    let file = Path::new(require(args, "--file"));
    let dir = Path::new(require(args, "--dir"));
    let loaded = format::load_doc_full(file)?;
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {}", dir.display(), e))?;
    let header = format::doc_to_bytes(&loaded.doc)?;
    std::fs::write(dir.join("document.mo2d.json"), &header).map_err(|e| format!("write document.mo2d.json: {}", e))?;
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir).map_err(|e| format!("create {}: {}", assets_dir.display(), e))?;
    for (name, data) in &loaded.blobs {
        let rel = name.strip_prefix("assets/").unwrap_or(name);
        std::fs::write(assets_dir.join(rel), data).map_err(|e| format!("write assets/{}: {}", rel, e))?;
    }
    println!("unpacked {} -> {} ({} blobs)", file.display(), dir.display(), loaded.blobs.len());
    Ok(())
}

/// B7 schema: print the v1 JSON Schema.
fn cmd_schema() -> Result<(), String> {
    println!("{}", mo2d::schema::schema_v1());
    Ok(())
}

/// Visual-hull probe (docs/log.md 2026-11-17): lift the turnaround document
/// (closed lines tagged `hull-frame`, tags `frame=N angle=DEG`) into a
/// slice stack, derive the three default pose trajectories as new 2D
/// contour timelines, and write per-timeline JSON artifacts + filmstrip
/// PNGs. Prints a single-line JSON summary of artifact hashes.
fn cmd_hull(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let out_dir = Path::new(require(args, "--out"));
    let doc = format::load_doc(Path::new(doc_path))?;

    // Collect the turnaround frames from tagged lines.
    let mut frames: Vec<(usize, hull::Frame)> = Vec::new();
    for line in &doc.lines {
        if !line.tags.iter().any(|t| t == "hull-frame") {
            continue;
        }
        let mut idx = None;
        let mut angle = None;
        for tag in &line.tags {
            if let Some(v) = tag.strip_prefix("frame=") {
                idx = Some(v.parse::<usize>().map_err(|_| format!("line {}: bad frame tag", line.id))?);
            }
            if let Some(v) = tag.strip_prefix("angle=") {
                angle = Some(v.parse::<f64>().map_err(|_| format!("line {}: bad angle tag", line.id))?);
            }
        }
        let (Some(idx), Some(angle)) = (idx, angle) else {
            return Err(format!("line {}: hull-frame needs frame= and angle= tags", line.id));
        };
        if !line.closed {
            return Err(format!("line {}: hull-frame contour must be closed", line.id));
        }
        let contour = line
            .nodes
            .iter()
            .map(|n| match &n.pos {
                mo2d::model::NodePos::Geom(p) => Ok(*p),
                mo2d::model::NodePos::Tick(_) => {
                    Err(format!("line {}: hull-frame contour must be static geometry", line.id))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        frames.push((idx, hull::Frame { angle_deg: angle, contour }));
    }
    frames.sort_by_key(|(i, _)| *i);
    if frames.is_empty() {
        return Err("no hull-frame lines in document".into());
    }
    let frames: Vec<hull::Frame> = frames.into_iter().map(|(_, f)| f).collect();

    std::fs::create_dir_all(out_dir).map_err(|e| format!("create {}: {}", out_dir.display(), e))?;

    // Lift + derive.
    let stack = hull::lift(&frames, hull::SLICES);
    let specs = hull::default_specs(&stack);
    let timelines = hull::derive_timelines(&stack, &specs);

    // Artifacts: canonical JSON per timeline + a filmstrip PNG per timeline
    // + a contact sheet of the input turnaround (eyeball policy, P1-7).
    let mut summary = mo2d::json::Value::obj();
    summary.set("slices", mo2d::json::Value::Int(hull::SLICES as i64));
    summary.set("grid", mo2d::json::Value::Int(hull::GRID as i64));
    for t in &timelines {
        let json_bytes = hull::timeline_json(t);
        let artifact = out_dir.join(format!("{}.json", t.name));
        format::write_atomic(&artifact, json_bytes.as_bytes())?;
        let sha = sha256_hex(json_bytes.as_bytes());
        summary.set(&t.name, mo2d::json::Value::Str(sha));
        let png = filmstrip_png(t);
        let png_path = out_dir.join(format!("{}.png", t.name));
        std::fs::write(&png_path, &png).map_err(|e| format!("write {}: {}", png_path.display(), e))?;
    }
    let contact = contact_sheet_png(&frames);
    let contact_path = out_dir.join("turnaround_contact.png");
    std::fs::write(&contact_path, &contact).map_err(|e| format!("write {}: {}", contact_path.display(), e))?;

    println!("{}", mo2d::json::to_string(&summary)?);
    Ok(())
}

/// Deterministic world→cell mapping for the preview sheets.
fn sheet_transform(
    bbox: (f64, f64, f64, f64), // xmin, xmax, ymin, ymax
    cell: (f64, f64),
) -> impl Fn(f64, f64) -> Vec3 + Copy {
    let (xmin, xmax, ymin, ymax) = bbox;
    let (cw, ch) = cell;
    let m = 12.0;
    let s = (((cw - 2.0 * m) / (xmax - xmin).max(1e-9)).min((ch - 2.0 * m) / (ymax - ymin).max(1e-9)) * 1000.0).round() / 1000.0;
    let ox = m + ((cw - 2.0 * m) - s * (xmax - xmin)) / 2.0;
    let oy = m + ((ch - 2.0 * m) - s * (ymax - ymin)) / 2.0;
    move |x: f64, y: f64| Vec3::new(ox + (x - xmin) * s, oy + (y - ymin) * s, 0.0)
}

fn poly_lines(
    id: &str,
    loops: &[Vec<(f64, f64)>],
    xf: impl Fn(f64, f64) -> mo2d::math::Vec3 + Copy,
    color: [u8; 4],
    width: f64,
) -> Vec<EvalLine> {
    loops
        .iter()
        .enumerate()
        .map(|(li, pts)| EvalLine {
            id: format!("{}_{}", id, li),
            tags: Vec::new(),
            closed: true,
            stroke_on: true,
            fill_on: false,
            fill_color: [0, 0, 0, 255],
            texture: None,
            width,
            visible: true,
            nodes: pts
                .iter()
                .map(|(x, y)| EvalNode {
                    p: xf(*x, *y),
                    width,
                    color,
                    fill: [0, 0, 0, 255],
                    uv: None,
                    handle_in: None,
                    handle_out: None,
                })
                .collect(),
            segs: vec![SegAttrs::default(); pts.len()],
        })
        .collect()
}

/// One PNG per derived timeline: all frames side by side, left to right.
fn filmstrip_png(t: &mo2d::hull::DerivedTimeline) -> Vec<u8> {
    let (cw, ch) = (170.0f64, 230.0f64);
    let mut bbox = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for f in &t.frames {
        for lp in f {
            for (x, y) in lp {
                bbox.0 = bbox.0.min(*x);
                bbox.1 = bbox.1.max(*x);
                bbox.2 = bbox.2.min(*y);
                bbox.3 = bbox.3.max(*y);
            }
        }
    }
    if !bbox.0.is_finite() {
        bbox = (0.0, 1.0, 0.0, 1.0);
    }
    let n = t.frames.len();
    let mut lines = Vec::new();
    for (fi, f) in t.frames.iter().enumerate() {
        let tx = |x: f64, y: f64| {
            let cell = (cw, ch);
            let f = sheet_transform(bbox, cell);
            let mut p = f(x, y);
            p.x += fi as f64 * cw;
            p.y += 30.0;
            p
        };
        lines.extend(poly_lines(&format!("f{}", fi), f, tx, [25, 25, 35, 255], 1.6));
    }
    let img = render(
        &lines,
        n as f64 * cw,
        ch + 60.0,
        &RenderOpts { width: (n as f64 * cw) as u32, height: (ch + 60.0) as u32, camera: Camera::Ortho2d, samples: 1 },
    );
    encode_png(&img)
}

/// Contact sheet of the input turnaround frames (4 per row).
fn contact_sheet_png(frames: &[mo2d::hull::Frame]) -> Vec<u8> {
    let (cw, ch) = (190.0f64, 250.0f64);
    let mut bbox = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for f in frames {
        for p in &f.contour {
            bbox.0 = bbox.0.min(p.x);
            bbox.1 = bbox.1.max(p.x);
            bbox.2 = bbox.2.min(p.y);
            bbox.3 = bbox.3.max(p.y);
        }
    }
    let cols = 4usize;
    let rows = (frames.len() + cols - 1) / cols;
    let mut lines = Vec::new();
    for (fi, f) in frames.iter().enumerate() {
        let loops = vec![f.contour.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>()];
        let tx = |x: f64, y: f64| {
            let f2 = sheet_transform(bbox, (cw, ch));
            let mut p = f2(x, y);
            p.x += (fi % cols) as f64 * cw;
            p.y += (fi / cols) as f64 * ch;
            p
        };
        lines.extend(poly_lines(&format!("t{}", fi), &loops, tx, [30, 30, 45, 255], 1.5));
    }
    let img = render(
        &lines,
        cols as f64 * cw,
        rows as f64 * ch,
        &RenderOpts {
            width: (cols as f64 * cw) as u32,
            height: (rows as f64 * ch) as u32,
            camera: Camera::Ortho2d,
            samples: 1,
        },
    );
    encode_png(&img)
}

fn usage() -> ! {
    eprintln!(
        "mo2d — everything is a line (headless engine)

USAGE:
  mo2d apply    --doc FILE --stream FILE          apply a command stream, validate + save
  mo2d replay   --stream FILE --out FILE          replay from scratch into a document
  mo2d render   --doc FILE [--tick N] [--out PNG] [--camera JSON] render one frame
  mo2d export   --doc FILE --dir DIR [--gif FILE] [--from T] [--to T] [--step T] [--camera JSON]
                                                          PNG sequence + manifest (+ GIF)
  mo2d verify   --doc FILE                        schema validation + determinism gate
  mo2d bench    [--nodes N]                       throughput: single-frame eval of a synthetic doc
  mo2d stage    --doc FILE [--port N]             A6 minimal stage (four operations)
  mo2d serve-mcp --doc FILE [--out-dir DIR]       MCP server (apply/list/screenshot) on stdio
  mo2d registry                                  print the command registry
  mo2d hash     FILE                              sha256 of a file
  mo2d pack     --dir DIR --out FILE              project directory → single-file package (B7)
  mo2d unpack   --file FILE --dir DIR             package → project directory (B7)
  mo2d schema                                     print the v1 JSON Schema (B7)

Ticks: 254,016,000,000 per second. Time truth is ticks; fps is display-only."
    );
    exit(2)
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().map(|s| s.as_str());
        }
    }
    None
}

fn require<'a>(args: &'a [String], name: &str) -> &'a str {
    flag(args, name).unwrap_or_else(|| {
        eprintln!("missing required flag {}", name);
        usage();
    })
}

fn parse_camera_arg(args: &[String]) -> Result<mo2d::render::Camera, String> {
    match flag(args, "--camera") {
        None => Ok(Default::default()),
        Some(text) => {
            let v = json::parse(text)?;
            parse_camera(Some(&v))
        }
    }
}

fn cmd_apply(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let stream_path = require(args, "--stream");
    let mut doc = if Path::new(doc_path).exists() {
        format::load_doc(Path::new(doc_path))?
    } else {
        Document::default()
    };
    let stream_text = std::fs::read_to_string(stream_path).map_err(|e| format!("read {}: {}", stream_path, e))?;
    let cmds = command::parse_stream(&stream_text)?;
    command::apply_stream(&mut doc, &cmds)?;
    let problems = validate::validate(&doc);
    if !problems.is_empty() {
        return Err(format!("validator rejected the result:\n{}", validate::format_problems(&problems)));
    }
    format::save_doc(Path::new(doc_path), &doc)?;
    let bytes = format::doc_to_bytes(&doc)?;
    println!("applied {} commands -> {} (sha256 {})", cmds.len(), doc_path, sha256_hex(&bytes));
    Ok(())
}

fn cmd_replay(args: &[String]) -> Result<(), String> {
    let stream_path = require(args, "--stream");
    let out_path = require(args, "--out");
    let stream_text = std::fs::read_to_string(stream_path).map_err(|e| format!("read {}: {}", stream_path, e))?;
    let cmds = command::parse_stream(&stream_text)?;
    let mut doc = Document::default();
    command::apply_stream(&mut doc, &cmds)?;
    let problems = validate::validate(&doc);
    if !problems.is_empty() {
        return Err(format!("validator rejected the replay:\n{}", validate::format_problems(&problems)));
    }
    format::save_doc(Path::new(out_path), &doc)?;
    let bytes = format::doc_to_bytes(&doc)?;
    println!("replayed {} commands -> {} (sha256 {})", cmds.len(), out_path, sha256_hex(&bytes));
    Ok(())
}

/// Load a document that may be a single-file package; texture blobs ride
/// along in the store (B7: the package is self-contained).
fn load_doc_and_assets(doc_path: &str) -> Result<(Document, mo2d::assets::AssetStore), String> {
    let loaded = format::load_doc_full(Path::new(doc_path))?;
    let mut assets = mo2d::assets::AssetStore::new();
    assets.root = Path::new(doc_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    assets.attach_blobs(&loaded.blobs);
    Ok((loaded.doc, assets))
}

fn cmd_render(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let tick: i64 = flag(args, "--tick").and_then(|v| v.parse().ok()).unwrap_or(0);
    let out = flag(args, "--out").unwrap_or("frame.png");
    let camera = parse_camera_arg(args)?;
    let (doc, mut assets) = load_doc_and_assets(doc_path)?;
    let lines = eval::evaluate(&doc, tick)?;
    let img = render_with_assets(
        &lines,
        doc.root.width as f64,
        doc.root.height as f64,
        &RenderOpts { width: doc.root.width, height: doc.root.height, camera, samples: 1 },
        &mut assets,
    );
    for e in &assets.errors {
        eprintln!("warning: {}", e);
    }
    let png = encode_png(&img);
    std::fs::write(out, &png).map_err(|e| format!("write {}: {}", out, e))?;
    println!("{} (tick {}, sha256 {})", out, tick, sha256_hex(&png));
    Ok(())
}

fn cmd_export(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let dir = require(args, "--dir");
    let (doc, _assets) = load_doc_and_assets(doc_path)?;
    let tps = doc.root.ticks_per_second;
    let seconds: i64 = flag(args, "--seconds").and_then(|v| v.parse().ok()).unwrap_or(1);
    let opts = ExportOpts {
        from_tick: flag(args, "--from").and_then(|v| v.parse().ok()).unwrap_or(0),
        to_tick: flag(args, "--to").and_then(|v| v.parse().ok()).unwrap_or(seconds * tps - 1),
        step: flag(args, "--step").and_then(|v| v.parse().ok()).unwrap_or((tps as f64 / doc.root.fps).round() as i64),
        camera: parse_camera_arg(args)?,
        width: None,
        height: None,
        asset_root: Path::new(doc_path).parent().map(|p| p.to_path_buf()).unwrap_or_default(),
    };
    let result = export_frames(&doc, &opts, Path::new(dir))?;
    println!("exported {} frames -> {}/manifest.json", result.frames.len(), dir);
    if let Some(gif) = flag(args, "--gif") {
        export_gif(&doc, &opts, Path::new(gif))?;
        println!("gif -> {}", gif);
    }
    Ok(())
}

fn cmd_verify(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let loaded = format::load_doc_full(Path::new(doc_path))?;
    let doc = loaded.doc;

    let problems = validate::validate(&doc);
    if !problems.is_empty() {
        return Err(format!("schema validation failed:\n{}", validate::format_problems(&problems)));
    }

    // Round-trip determinism.
    let b1 = format::doc_to_bytes(&doc)?;
    let d2 = format::parse_doc(std::str::from_utf8(&b1).map_err(|_| "utf8")?)?;
    let b2 = format::doc_to_bytes(&d2)?;
    if b1 != b2 {
        return Err("determinism gate failed: save→load→save differs".into());
    }

    // Evaluation + render determinism at sample ticks.
    let camera = parse_camera_arg(args)?;
    let tps = doc.root.ticks_per_second;
    for frac in [0i64, tps / 3, tps / 2, tps] {
        let mut hashes = Vec::new();
        for _ in 0..2 {
            let lines = eval::evaluate(&doc, frac)?;
            let mut assets = mo2d::assets::AssetStore::new();
            assets.root = Path::new(doc_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
            assets.attach_blobs(&loaded.blobs);
            let img = render_with_assets(
                &lines,
                doc.root.width as f64,
                doc.root.height as f64,
                &RenderOpts { width: doc.root.width, height: doc.root.height, camera: camera.clone(), samples: 1 },
                &mut assets,
            );
            hashes.push(sha256_hex(&encode_png(&img)));
        }
        if hashes[0] != hashes[1] {
            return Err(format!("determinism gate failed: render at tick {} is not reproducible", frac));
        }
    }
    println!(
        "ok: schema clean, round-trip byte-stable, evaluation + render deterministic (doc sha256 {})",
        sha256_hex(&b1)
    );
    Ok(())
}

fn cmd_bench(args: &[String]) -> Result<(), String> {
    let nodes: usize = flag(args, "--nodes").and_then(|v| v.parse().ok()).unwrap_or(10_000);
    // Synthetic stress document built through the command protocol
    // (completion definition 5: command-stream-generated 10k nodes).
    let tps = TICKS_PER_SECOND;
    let mut node_json = Vec::new();
    for i in 0..nodes {
        let x = (i as f64) * 0.1;
        let y = ((i as f64) * 0.37).sin() * 80.0 + 180.0;
        node_json.push(format!("{{\"p\":[{:.3},{:.3},0],\"w\":{}}}", x, y, 2.0 + (i % 20) as f64 * 0.5));
    }
    let stream = format!(
        "{{\"v\":1,\"cmds\":[\
          {{\"op\":\"line.create\",\"id\":\"stress\",\"width\":3,\"nodes\":[{}]}},\
          {{\"op\":\"line.create\",\"id\":\"trk\",\"tags\":[\"track\"],\"nodes\":[{{\"p\":[0],\"v\":2.0}},{{\"p\":[{}],\"v\":8.0}}]}},\
          {{\"op\":\"relation.create\",\"type\":\"track\",\"trackLine\":\"trk\",\"target\":{{\"line\":\"stress\",\"attr\":\"width\"}}}}\
        ]}}",
        node_json.join(","),
        tps
    );
    let cmds = command::parse_stream(&stream)?;
    let mut doc = Document::default();
    let t0 = Instant::now();
    command::apply_stream(&mut doc, &cmds)?;
    let build = t0.elapsed();

    // Warm up, then time single-frame evaluation (median of 31 runs).
    eval::evaluate(&doc, 0)?;
    let mut samples = Vec::new();
    for run in 0..31 {
        let tick = tps / 3 + run as i64; // different ticks, same cost profile
        let t = Instant::now();
        let lines = eval::evaluate(&doc, tick)?;
        let took = t.elapsed();
        std::hint::black_box(&lines);
        samples.push(took.as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    println!(
        "bench: {} nodes, build {:?}, single-frame eval median {:.3} ms (min {:.3}, max {:.3}) — {}",
        nodes,
        build,
        median,
        samples[0],
        samples[samples.len() - 1],
        if median < 16.0 { "PASS (<16ms)" } else { "FAIL (>=16ms)" }
    );
    if median >= 16.0 {
        return Err("throughput gate failed: median single-frame evaluation >= 16ms".into());
    }
    Ok(())
}

fn cmd_stage(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let port: u16 = flag(args, "--port").and_then(|v| v.parse().ok()).unwrap_or(7788);
    let session = McpSession::new(Path::new(doc_path), true)?;
    let state = Arc::new(Mutex::new(StageState::new(session)));
    serve(state, port)
}

fn cmd_serve_mcp(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let out_dir: PathBuf = flag(args, "--out-dir").map(PathBuf::from).unwrap_or_else(|| {
        let p = Path::new(doc_path);
        p.parent().unwrap_or(Path::new(".")).to_path_buf()
    });
    let mut session = McpSession::new(Path::new(doc_path), true)?;
    mo2d::mcp::serve(&mut session, &out_dir)
}

fn cmd_registry() -> Result<(), String> {
    let mut arr = Vec::new();
    for spec in command::registry() {
        let mut o = Value::obj();
        o.set("name", Value::Str(spec.name.into()));
        o.set("args", Value::Str(spec.args.into()));
        o.set("doc", Value::Str(spec.doc.into()));
        arr.push(o);
    }
    let mut out = Value::obj();
    out.set("protocolVersion", Value::Int(command::PROTOCOL_VERSION));
    out.set("commands", Value::Arr(arr));
    println!("{}", json::to_string(&out)?);
    Ok(())
}

fn cmd_hash(args: &[String]) -> Result<(), String> {
    let path = args.first().map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("usage: mo2d hash FILE");
        usage();
    });
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {}", path, e))?;
    println!("{}", sha256_hex(&bytes));
    Ok(())
}
