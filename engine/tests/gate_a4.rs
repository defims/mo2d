//! Gate A4 — command protocol + MCP + validator (mvp.md §3).
//!
//! A1–A3 recordings (the demo streams) replay to frozen byte hashes; the same
//! animation is built from zero through a REAL MCP session over stdio
//! (pseudo-AI lower bound, completion definition 1); the schema rejects bad
//! documents with pointed errors; the determinism gate catches divergence.

mod common;

use mo2d::command::{self, parse_stream};
use mo2d::json::Value;
use mo2d::model::Document;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn a4_recorded_streams_replay_to_frozen_bytes() {
    let frozen = common::load_frozen();
    for (name, root) in [("tentacle-ai.json", "tentacle"), ("scene3d.json", "scene3d")] {
        let doc = common::replay_stream(&common::demo_dir().join("cmd").join(name));
        let got = mo2d::sha::sha256_hex(&common::doc_bytes(&doc));
        let want = frozen.raw.get(root).unwrap().get("docSha256").unwrap().as_str().unwrap();
        assert_eq!(&got, want, "{} replay must hit the frozen document hash", name);

        let doc2 = common::replay_stream(&common::demo_dir().join("cmd").join(name));
        assert_eq!(common::doc_bytes(&doc), common::doc_bytes(&doc2), "replay must be byte-stable");
    }
}

#[test]
fn a4_stream_rejection() {
    assert!(parse_stream(r#"{"v":2,"cmds":[]}"#).is_err(), "future protocol versions must be rejected");
    assert!(parse_stream(r#"{"cmds":[]}"#).is_err(), "missing version must be rejected");
    let mut doc = Document::default();
    let bad = r#"{"v":1,"cmds":[{"op":"line.create","id":"a","nodes":[{"p":[0,0,0]},{"p":[1,0,0]}]},{"op":"wat"}]}"#;
    assert!(command::apply_stream(&mut doc, &parse_stream(bad).unwrap()).is_err());
    assert!(doc.lines.is_empty(), "rejected batch must leave no partial state");
}

#[test]
fn a4_schema_rejects_bad_documents() {
    let base = r#"{"v":1,"cmds":[
      {"op":"line.create","id":"g","nodes":[{"p":[0,0,0]},{"p":[10,0,0]},{"p":[10,10,0]}]},
      {"op":"line.create","id":"trk","tags":["track"],"nodes":[{"p":[0],"v":1},{"p":[1000000000],"v":2}]},
      {"op":"relation.create","type":"track","trackLine":"trk","target":{"line":"g","node":0,"attr":"width"}}
    ]}"#;

    let cases: Vec<(&str, Box<dyn Fn(&mut Document)>)> = vec![
        ("dangling relation reference", Box::new(|d: &mut Document| {
            d.relations.push(mo2d::model::Relation::new_bind("bad", "g", "nope"));
        })),
        ("node index out of range", Box::new(|d: &mut Document| {
            d.relations.push(mo2d::model::Relation::new_share("bad2", ("g", 0), ("g", 99)));
        })),
        ("conflicting writers", Box::new(|d: &mut Document| {
            d.relations.push(mo2d::model::Relation::new_track("bad3", "trk", "g", Some(0), mo2d::model::TrackAttr::Width));
        })),
        ("mixed node kinds on one line", Box::new(|d: &mut Document| {
            d.line_mut("g").unwrap().nodes.push(mo2d::model::Node::track(5, Value::Num(1.0)));
        })),
    ];
    for (name, corrupt) in cases {
        let mut doc = Document::default();
        command::apply_stream(&mut doc, &parse_stream(base).unwrap()).unwrap();
        assert!(mo2d::validate::validate(&doc).is_empty(), "pristine doc must validate");
        corrupt(&mut doc);
        let problems = mo2d::validate::validate(&doc);
        assert!(!problems.is_empty(), "{} must be rejected", name);
        assert!(problems.iter().all(|p| !p.message.is_empty() && !p.path.is_empty()), "rejections carry path + message");
    }
}

#[test]
fn a4_determinism_gate() {
    let stream = std::fs::read_to_string(common::demo_dir().join("cmd/tentacle-ai.json")).unwrap();
    let doc = common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json"));
    let report = mo2d::validate::determinism_gate(&stream, &doc);
    assert!(report.problems.is_empty(), "determinism gate problems: {:?}", report.problems);
    assert!(report.double_apply_equal && report.round_trip_equal && report.double_eval_equal);
}

// ---------------------------------------------------------------------------
// MCP: a real server session over stdio (the pseudo-AI lower bound).
// ---------------------------------------------------------------------------

struct Session {
    child: std::process::Child,
    reader: BufReader<std::process::ChildStdout>,
    next_id: usize,
}

impl Session {
    fn start(doc_path: &std::path::Path) -> Session {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mo2d"))
            .args(["serve-mcp", "--doc"])
            .arg(doc_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mo2d serve-mcp");
        let stdout = child.stdout.take().unwrap();
        Session { child, reader: BufReader::new(stdout), next_id: 0 }
    }

    fn send(&mut self, v: &str) {
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(v.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
    }

    fn recv(&mut self) -> Value {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).expect("read mcp line");
            assert!(n > 0, "MCP server closed stdin early");
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            return mo2d::json::parse(t).expect("valid JSON-RPC reply");
        }
    }

    fn call_tool(&mut self, name: &str, args: &str) -> Value {
        self.next_id += 1;
        let msg = format!(
            r#"{{"jsonrpc":"2.0","id":"c{}","method":"tools/call","params":{{"name":"{}","arguments":{}}}}}"#,
            self.next_id, name, args
        );
        self.send(&msg);
        let reply = self.recv();
        assert!(reply.get("error").is_none(), "tool {} failed: {:?}", name, reply.get("error"));
        let result = reply.get("result").expect("result");
        assert!(result.get("isError").and_then(Value::as_bool) != Some(true), "tool {} returned isError", name);
        let text = result
            .get("content")
            .and_then(Value::as_arr)
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(Value::as_str)
            .expect("text content")
            .to_string();
        mo2d::json::parse(&text).unwrap_or_else(|_| Value::Str(text.clone()))
    }
}

#[test]
fn a4_mcp_builds_tentacle_from_zero() {
    let dir = std::env::temp_dir().join(format!("mo2d-mcp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let doc_path = dir.join("tentacle.mo2d");

    let mut s = Session::start(&doc_path);

    // MCP handshake.
    s.send(r#"{"jsonrpc":"2.0","id":"init","method":"initialize","params":{}}"#);
    let init = s.recv();
    assert_eq!(init.get("result").unwrap().get("protocolVersion").unwrap().as_str(), Some("2024-11-05"));
    s.send(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);

    // list: the command registry (AI introspection).
    let list = s.call_tool("list", "{}");
    let cmds = list.get("commands").and_then(Value::as_arr).expect("commands");
    assert!(cmds.len() >= 10, "registry must list the full protocol");

    // apply: feed the recorded tentacle stream (pseudo-AI).
    let stream_text = std::fs::read_to_string(common::demo_dir().join("cmd/tentacle-ai.json")).unwrap();
    let stream = mo2d::json::parse(&stream_text).unwrap();
    let apply_arg = mo2d::json::to_string_compact(&stream).unwrap();
    let applied = s.call_tool("apply", &apply_arg);
    assert_eq!(applied.get("applied").and_then(Value::as_i64), Some(18), "18 commands applied");

    // screenshot: render through MCP; hash must equal the frozen CLI render.
    let shot = s.call_tool("screenshot", r#"{"tick":127008000000}"#);
    let got_sha = shot.get("sha256").and_then(Value::as_str).unwrap().to_string();
    let frozen = common::load_frozen();
    let want = frozen.raw.get("tentacle").unwrap().get("midFrameSha256").unwrap().as_str().unwrap();
    assert_eq!(&got_sha, want, "MCP screenshot must equal the frozen CLI render of the same frame");

    // The MCP-saved document must be byte-identical to the CLI replay
    // (one protocol; every channel converges to the same bytes).
    let mcp_doc = common::doc_bytes(&mo2d::format::load_doc(&doc_path).unwrap());
    let replay_doc = common::doc_bytes(&common::replay_stream(&common::demo_dir().join("cmd/tentacle-ai.json")));
    assert_eq!(mcp_doc, replay_doc);

    s.child.kill().unwrap();
    std::fs::remove_dir_all(&dir).ok();
}
