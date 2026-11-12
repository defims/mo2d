//! mo2d CLI — every capability is reachable headless:
//! apply / replay / render / export / verify / bench / stage / serve-mcp.

use mo2d::command;
use mo2d::eval;
use mo2d::export::{export_frames, export_gif, ExportOpts};
use mo2d::format;
use mo2d::json::{self, Value};
use mo2d::mcp::{parse_camera, McpSession};
use mo2d::model::{Document, TICKS_PER_SECOND};
use mo2d::png::encode_png;
use mo2d::render::{render, RenderOpts};
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

fn cmd_render(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let tick: i64 = flag(args, "--tick").and_then(|v| v.parse().ok()).unwrap_or(0);
    let out = flag(args, "--out").unwrap_or("frame.png");
    let camera = parse_camera_arg(args)?;
    let doc = format::load_doc(Path::new(doc_path))?;
    let lines = eval::evaluate(&doc, tick)?;
    let img = render(
        &lines,
        doc.root.width as f64,
        doc.root.height as f64,
        &RenderOpts { width: doc.root.width, height: doc.root.height, camera, samples: 1 },
    );
    let png = encode_png(&img);
    std::fs::write(out, &png).map_err(|e| format!("write {}: {}", out, e))?;
    println!("{} (tick {}, sha256 {})", out, tick, sha256_hex(&png));
    Ok(())
}

fn cmd_export(args: &[String]) -> Result<(), String> {
    let doc_path = require(args, "--doc");
    let dir = require(args, "--dir");
    let doc = format::load_doc(Path::new(doc_path))?;
    let tps = doc.root.ticks_per_second;
    let seconds: i64 = flag(args, "--seconds").and_then(|v| v.parse().ok()).unwrap_or(1);
    let opts = ExportOpts {
        from_tick: flag(args, "--from").and_then(|v| v.parse().ok()).unwrap_or(0),
        to_tick: flag(args, "--to").and_then(|v| v.parse().ok()).unwrap_or(seconds * tps - 1),
        step: flag(args, "--step").and_then(|v| v.parse().ok()).unwrap_or((tps as f64 / doc.root.fps).round() as i64),
        camera: parse_camera_arg(args)?,
        width: None,
        height: None,
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
    let text = std::fs::read_to_string(doc_path).map_err(|e| format!("read {}: {}", doc_path, e))?;
    let doc = format::parse_doc(&text)?;

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
            let img = render(
                &lines,
                doc.root.width as f64,
                doc.root.height as f64,
                &RenderOpts { width: doc.root.width, height: doc.root.height, camera: camera.clone(), samples: 1 },
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
