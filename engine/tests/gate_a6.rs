//! Gate A6 — the minimal stage (mvp.md §3, D8 hard boundary).
//!
//! Four operations, zero standard controls; every interaction is a protocol
//! command; the hand-built tentacle's command stream and the AI path are
//! isomorphic: same protocol vocabulary, same relation structure, and the
//! recorded hand stream replays byte-identically (completion definition 7).

mod common;

use mo2d::mcp::McpSession;
use mo2d::stage::{serve, StageState};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn spawn_stage() -> (u16, std::path::PathBuf) {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("mo2d-a6-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    let doc_path = dir.join("stage.mo2d");
    let session = McpSession::new(&doc_path, true).unwrap();
    let state = Arc::new(Mutex::new(StageState::new(session)));
    // Grab a free port.
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    std::thread::spawn(move || serve(state, port).unwrap());
    // Wait for the server to come up.
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return (port, doc_path);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("stage server did not start");
}

fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, Vec<u8>) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect stage");
    stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    let body_bytes = body.map(|b| b.as_bytes().to_vec()).unwrap_or_default();
    let head = format!(
        "{} {} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        method,
        path,
        body_bytes.len()
    );
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(&body_bytes).unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    // Split headers/body.
    let split = buf.windows(4).position(|w| w == b"\r\n\r\n").expect("http header end");
    let status: u16 = std::str::from_utf8(&buf[..split]).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, buf[split + 4..].to_vec())
}

fn post_cmd(port: u16, cmd_json: &str) -> bool {
    let (status, body) = http(port, "POST", "/cmd", Some(cmd_json));
    assert_eq!(status, 200);
    let text = String::from_utf8_lossy(&body).to_string();
    let reply = mo2d::json::parse(&text).unwrap();
    if reply.get("ok").and_then(mo2d::json::Value::as_bool) != Some(true) {
        panic!("stage command rejected: {}", reply.get("error").and_then(mo2d::json::Value::as_str).unwrap_or("?"));
    }
    true
}

/// The registry op names (from the engine itself — the protocol definition).
fn registry_ops() -> Vec<String> {
    mo2d::command::registry().iter().map(|s| s.name.to_string()).collect()
}

#[test]
fn a6_stage_four_operations_all_commands() {
    let (port, doc_path) = spawn_stage();

    // ── Operation 1: DRAW a line (pointer drag → one line.create command,
    //    exactly what the page's pointerup handler posts).
    post_cmd(
        port,
        r#"{"v":1,"cmds":[{"op":"line.create","nodes":[{"p":[110,184,0]},{"p":[185,192,0]},{"p":[260,196,0]},{"p":[335,192,0]},{"p":[410,184,0]}]}]}"#,
    );
    // Draw the body (the page rounds coordinates to 0.01 — same here).
    post_cmd(
        port,
        r#"{"v":1,"cmds":[{"op":"line.create","nodes":[{"p":[110,184,0],"w":34},{"p":[190,192.3,0],"w":26},{"p":[270,195.5,0],"w":18.5},{"p":[350,190.4,0],"w":12},{"p":[425,184,0],"w":7}]}]}"#,
    );
    // Draw the sucker as a closed stroke (the page auto-closes when the
    // endpoint lands back on the start point).
    post_cmd(
        port,
        r#"{"v":1,"cmds":[{"op":"line.create","closed":true,"fillOn":true,"nodes":[{"p":[423,184,0]},{"p":[462,168,0]},{"p":[462,200,0]},{"p":[425,186,0]}]}]}"#,
    );

    // ── Operation 2: DRAG a node (auto-key once a track exists; plain set before).
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"node.set","line":"l1","index":2,"p":[260,210,0],"atTick":0}]}"#);

    // ── Operation 3: CONNECT —
    //   line → line = bind (drag from body onto skeleton);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"relation.create","type":"bind","target":"l2","skeleton":"l1"}]}"#);
    //   node dropped on the timeline = track.create for that node.
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"track.create","line":"l1","node":2,"tick":0}]}"#);
    //   then dragging that node at t = 0.5s auto-keys a keyframe.
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"node.set","line":"l1","index":2,"p":[260,286,0],"atTick":127008000000}]}"#);

    // ── Operation 4: PLAY (session commands through the same /cmd channel).
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"session.seek","tick":127008000000}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"session.play","tick":127008000000}]}"#);

    // The rendered frame must exist and be non-trivial.
    let (status, png) = http(port, "GET", "/frame?tick=127008000000", None);
    assert_eq!(status, 200);
    assert_eq!(&png[..4], b"\x89PNG");
    assert!(png.len() > 10_000, "rendered frame must have content");

    // The scene must show the bound body following the keyed pose.
    let (status, scene) = http(port, "GET", "/scene?tick=127008000000", None);
    assert_eq!(status, 200);
    let scene = mo2d::json::parse(&String::from_utf8_lossy(&scene)).unwrap();
    let lines = scene.get("lines").and_then(mo2d::json::Value::as_arr).unwrap();
    let body = lines.iter().find(|l| l.get("id").and_then(mo2d::json::Value::as_str) == Some("l2")).unwrap();
    let nodes = body.get("nodes").and_then(mo2d::json::Value::as_arr).unwrap();
    let mid_y = nodes[2].as_arr().unwrap()[1].as_f64().unwrap();
    assert!(mid_y > 200.0, "bound body must follow the keyed skeleton pose (y={})", mid_y);

    // ── Every interaction is a command: the recorded history replays to the
    //    staged document, byte-identically.
    let (status, history) = http(port, "GET", "/history", None);
    assert_eq!(status, 200);
    let history_text = String::from_utf8_lossy(&history).to_string();
    let hist = mo2d::command::parse_stream(&history_text).unwrap();
    let ops = registry_ops();
    let session_ops = ["session.play", "session.pause", "session.seek"];
    for c in &hist {
        assert!(
            ops.contains(&c.op) || session_ops.contains(&c.op.as_str()),
            "history op '{}' must be a protocol command",
            c.op
        );
    }
    // Replay the recorded history from scratch → the staged document, byte-equal.
    let mut doc = mo2d::model::Document::default();
    let document_cmds: Vec<_> = hist.iter().filter(|c| !c.op.starts_with("session.")).cloned().collect();
    mo2d::command::apply_stream(&mut doc, &document_cmds).unwrap();
    let staged = mo2d::format::load_doc(&doc_path).unwrap();
    assert_eq!(
        mo2d::format::doc_to_bytes(&doc).unwrap(),
        mo2d::format::doc_to_bytes(&staged).unwrap(),
        "replaying the hand-recorded stream must reproduce the staged document byte-for-byte"
    );

    // ── Zero standard controls: the page ships no <button>/<input>/<select>.
    let (status, html) = http(port, "GET", "/", None);
    assert_eq!(status, 200);
    let html = String::from_utf8_lossy(&html).to_lowercase();
    assert!(!html.contains("<button"), "no button controls");
    assert!(!html.contains("<input"), "no input controls");
    assert!(!html.contains("<select"), "no select controls");
    assert!(!html.contains("<menu"), "no menu controls");
}

#[test]
fn a6_hand_and_ai_paths_isomorphic() {
    // The hand path (above, through the stage HTTP channel) and the AI path
    // (the recorded tentacle stream) build the SAME tentacle structure through
    // the SAME protocol: identical relation wiring by line order, identical
    // command vocabulary.
    let (port, _doc_path) = spawn_stage();

    // Hand build: skeleton, body, sucker (closed), bind, track + keys.
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"line.create","nodes":[{"p":[110,184,0]},{"p":[185,192,0]},{"p":[260,196,0]},{"p":[335,192,0]},{"p":[410,184,0]}]}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"line.create","nodes":[{"p":[110,184,0],"w":34},{"p":[190,192.3,0],"w":26},{"p":[270,195.5,0],"w":18.5},{"p":[350,190.4,0],"w":12},{"p":[425,184,0],"w":7}]}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"relation.create","type":"bind","target":"l2","skeleton":"l1"}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"track.create","line":"l1","node":2,"tick":0}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"node.set","line":"l1","index":2,"p":[260,286,0],"atTick":127008000000}]}"#);
    post_cmd(port, r#"{"v":1,"cmds":[{"op":"node.set","line":"l1","index":2,"p":[260,196,0],"atTick":254016000000}]}"#);
    // AI build: the recorded stream, applied through the SAME stage channel.
    let (port2, _doc2) = spawn_stage();
    let ai = std::fs::read_to_string(common::demo_dir().join("cmd/tentacle-ai.json")).unwrap();
    let (status, body) = http(port2, "POST", "/cmd", Some(&ai));
    assert_eq!(status, 200);
    let reply = mo2d::json::parse(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(reply.get("ok").and_then(mo2d::json::Value::as_bool), Some(true), "AI stream through the stage channel must apply");

    // Structural comparison: the hand path's relations are wired to the same
    // line-order positions as the AI path's corresponding relations (the hand
    // build is a subset — a human keys fewer joints — but the wiring of what
    // it builds is identical; ids are auto-generated, structure is the
    // isomorphism).
    let hand = stage_scene(port);
    let ai_scene = stage_scene(port2);
    let hrels = relation_shapes(&hand);
    let arels = relation_shapes(&ai_scene);

    // Hand: bind(2nd line → 1st line); AI: bind(body → skel) = bind(2nd → 1st) ✓.
    assert!(
        hrels.iter().any(|r| r.starts_with("bind:1<-0")),
        "hand path wires bind body→skeleton by line order: {:?}",
        hrels
    );
    assert!(
        arels.iter().any(|r| r.starts_with("bind:1<-0")),
        "AI path wires the same bind: {:?}",
        arels
    );
    // Both have a position track on the skeleton (hand: track.create; AI: track).
    assert!(hrels.iter().any(|r| r.starts_with("track:")), "hand path has a track relation");
    assert!(arels.iter().any(|r| r.starts_with("track:")), "AI path has track relations");

    // The AI stream through the STAGE channel produced the same doc hash as
    // through the CLI replay path (one protocol, two channels).
    let (_, h1) = http(port2, "GET", "/doc", None);
    let ai_doc = mo2d::format::parse_doc(&String::from_utf8_lossy(&h1)).unwrap();
    let cli_doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    assert_eq!(
        mo2d::format::doc_to_bytes(&ai_doc).unwrap(),
        mo2d::format::doc_to_bytes(&cli_doc).unwrap(),
        "stage channel and CLI replay must converge to identical bytes"
    );
}

fn stage_scene(port: u16) -> mo2d::json::Value {
    let (status, body) = http(port, "GET", "/scene", None);
    assert_eq!(status, 200);
    mo2d::json::parse(&String::from_utf8_lossy(&body)).unwrap()
}

/// Relation shapes as "type:targetIdx<-sourceIdx" strings (line order indices).
fn relation_shapes(scene: &mo2d::json::Value) -> Vec<String> {
    let lines = scene.get("lines").and_then(mo2d::json::Value::as_arr).unwrap();
    let rels = scene.get("relations").and_then(mo2d::json::Value::as_arr).unwrap();
    let line_index = |id: &str| -> usize {
        lines
            .iter()
            .position(|l| l.get("id").and_then(mo2d::json::Value::as_str) == Some(id))
            .unwrap_or(usize::MAX)
    };
    rels.iter()
        .map(|r| {
            let ty = r.get("type").and_then(mo2d::json::Value::as_str).unwrap_or("?");
            match ty {
                "bind" => format!(
                    "bind:{}<-{}",
                    line_index(r.get("target").and_then(mo2d::json::Value::as_str).unwrap_or("")),
                    line_index(r.get("skeleton").and_then(mo2d::json::Value::as_str).unwrap_or(""))
                ),
                "share" => format!(
                    "share:{}<-{}",
                    r.get("driven").and_then(|d| d.get("line")).and_then(mo2d::json::Value::as_str).map(line_index).unwrap_or(usize::MAX),
                    r.get("driver").and_then(|d| d.get("line")).and_then(mo2d::json::Value::as_str).map(line_index).unwrap_or(usize::MAX)
                ),
                "track" => format!(
                    "track:{}",
                    r.get("target").and_then(|d| d.get("line")).and_then(mo2d::json::Value::as_str).map(|id| line_index(id)).unwrap_or(usize::MAX)
                ),
                other => other.to_string(),
            }
        })
        .collect()
}
