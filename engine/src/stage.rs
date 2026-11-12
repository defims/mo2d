//! A6 minimal stage: a thin viewport around the headless kernel.
//!
//! Four operations only (mvp.md D8 hard boundary): draw line / drag node /
//! connect-line-to-build-relation / play. Zero standard controls — the time
//! baseline and playhead are themselves lines (ai-interaction.md §3).
//! Every interaction is a protocol command: document commands mutate the doc
//! (validated, saved); session commands (session.play / session.seek) drive
//! playback without touching document history.

use crate::command::{self, Command};
use crate::eval;
use crate::json::{self, Value};
use crate::mcp::McpSession;
use crate::png::encode_png;
use crate::render::{render, RenderOpts};
use crate::sha::sha256_hex;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub struct StageState {
    pub session: McpSession,
    pub tick: i64,
}

impl StageState {
    pub fn new(session: McpSession) -> StageState {
        StageState { session, tick: 0 }
    }

    /// Apply a batch of document commands; roll back if the result is invalid.
    pub fn apply_commands(&mut self, cmds: &[Command]) -> Result<Value, String> {
        let backup = self.session.doc.clone();
        let effects = command::apply_stream(&mut self.session.doc, cmds)
            .map_err(|e| format!("apply failed: {}", e))?;
        let problems = crate::validate::validate(&self.session.doc);
        if !problems.is_empty() {
            self.session.doc = backup;
            return Err(format!("validator rejected the result:\n{}", crate::validate::format_problems(&problems)));
        }
        self.session.log.extend(cmds.iter().cloned());
        self.session.save().map_err(|e| format!("save failed: {}", e))?;
        let mut summaries = Vec::new();
        for e in &effects {
            summaries.push(Value::Str(e.summary.clone()));
        }
        let doc_bytes = crate::format::doc_to_bytes(&self.session.doc)?;
        let mut out = Value::obj();
        out.set("ok", Value::Bool(true));
        out.set("docSha256", Value::Str(sha256_hex(&doc_bytes)));
        out.set("applied", Value::Int(effects.len() as i64));
        out.set("summaries", Value::Arr(summaries));
        Ok(out)
    }

    fn scene_json(&self) -> Result<Value, String> {
        let lines = eval::evaluate(&self.session.doc, self.tick)?;
        let mut out = Value::obj();
        let doc = &self.session.doc;
        out.set("tick", Value::Int(self.tick));
        out.set("playing", Value::Bool(false)); // playback runs client-side
        out.set("tps", Value::Int(doc.root.ticks_per_second));
        out.set("fps", Value::Num(doc.root.fps));
        out.set("width", Value::Int(doc.root.width as i64));
        out.set("height", Value::Int(doc.root.height as i64));
        let mut arr = Vec::new();
        for l in &lines {
            if l.tags.iter().any(|t| t == "track") {
                continue;
            }
            let mut lo = Value::obj();
            lo.set("id", Value::Str(l.id.clone()));
            lo.set("closed", Value::Bool(l.closed));
            lo.set("tags", Value::Arr(l.tags.iter().map(|t| Value::Str(t.clone())).collect()));
            let mut nodes = Vec::new();
            for n in &l.nodes {
                nodes.push(Value::Arr(vec![Value::Num(n.p.x), Value::Num(n.p.y), Value::Num(n.p.z)]));
            }
            lo.set("nodes", Value::Arr(nodes));
            arr.push(lo);
        }
        out.set("lines", Value::Arr(arr));
        let mut rels = Vec::new();
        for r in &doc.relations {
            let mut ro = Value::obj();
            ro.set("id", Value::Str(r.id.clone()));
            ro.set("type", Value::Str(r.kind.name().into()));
            match r.kind {
                crate::model::RelationKind::Bind => {
                    ro.set("target", Value::Str(r.bind_target.clone()));
                    ro.set("skeleton", Value::Str(r.bind_skeleton.clone()));
                }
                crate::model::RelationKind::Share => {
                    let mut d = Value::obj();
                    d.set("line", Value::Str(r.share_driver.0.clone()));
                    d.set("node", Value::Int(r.share_driver.1 as i64));
                    ro.set("driver", d);
                    let mut dr = Value::obj();
                    dr.set("line", Value::Str(r.share_driven.0.clone()));
                    dr.set("node", Value::Int(r.share_driven.1 as i64));
                    ro.set("driven", dr);
                }
                crate::model::RelationKind::Track => {
                    ro.set("trackLine", Value::Str(r.track_line.clone()));
                    let mut t = Value::obj();
                    t.set("line", Value::Str(r.track_target_line.clone()));
                    if let Some(n) = r.track_target_node {
                        t.set("node", Value::Int(n as i64));
                    }
                    ro.set("target", t);
                }
            }
            rels.push(ro);
        }
        out.set("relations", Value::Arr(rels));
        Ok(out)
    }

    fn frame_png(&self) -> Result<Vec<u8>, String> {
        let lines = eval::evaluate(&self.session.doc, self.tick)?;
        let doc = &self.session.doc;
        let img = render(
            &lines,
            doc.root.width as f64,
            doc.root.height as f64,
            &RenderOpts { width: doc.root.width, height: doc.root.height, camera: Default::default(), samples: 1 },
        );
        Ok(encode_png(&img))
    }
}

/// Run the stage HTTP server (blocks).
pub fn serve(state: Arc<Mutex<StageState>>, port: u16) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("bind 127.0.0.1:{}: {}", port, e))?;
    eprintln!("mo2d stage listening on http://127.0.0.1:{}", port);
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            let _ = handle_connection(stream, state);
        });
    }
    Ok(())
}

fn handle_connection(stream: TcpStream, state: Arc<Mutex<StageState>>) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    // Headers.
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(v) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };

    match (method.as_str(), path.as_str()) {
        ("GET", "/") => respond_html(stream, include_str!("stage.html")),
        ("GET", "/scene") => {
            let body = {
                let st = state.lock().unwrap();
                json::to_string(&st.scene_json()?)?
            };
            respond_json(stream, &body)
        }
        ("GET", "/frame") => {
            // /frame?tick=N
            let png = {
                let mut st = state.lock().unwrap();
                if let Some(t) = query_param(&query, "tick").and_then(|v| v.parse::<i64>().ok()) {
                    st.tick = t;
                }
                st.frame_png()?
            };
            respond_bytes(stream, "image/png", &png)
        }
        ("GET", "/doc") => {
            let body = {
                let st = state.lock().unwrap();
                json::to_string(&crate::format::doc_to_value(&st.session.doc))?
            };
            respond_bytes(stream, "application/json", body.as_bytes())
        }
        ("GET", "/history") => {
            let body = {
                let st = state.lock().unwrap();
                let cmds: Vec<Value> = st.session.log.iter().map(|c| c.to_value()).collect();
                let mut o = Value::obj();
                o.set("v", Value::Int(command::PROTOCOL_VERSION));
                o.set("cmds", Value::Arr(cmds));
                json::to_string(&o)?
            };
            respond_json(stream, &body)
        }
        ("POST", "/cmd") => {
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&body).to_string();
            let result = handle_cmd(&state, &text);
            let body = match result {
                Ok(mut v) => {
                    v.set("ok", Value::Bool(true));
                    json::to_string(&v)?
                }
                Err(e) => {
                    let mut v = Value::obj();
                    v.set("ok", Value::Bool(false));
                    v.set("error", Value::Str(e));
                    json::to_string(&v)?
                }
            };
            respond_json(stream, &body)
        }
        _ => respond_status(stream, 404, "not found"),
    }
}

/// Apply document commands and session.* commands arriving from the page.
fn handle_cmd(state: &Arc<Mutex<StageState>>, text: &str) -> Result<Value, String> {
    // Accept {"op":...} single, or {"v":1,"cmds":[...]}.
    let parsed = json::parse(text)?;
    let cmds: Vec<Command> = if parsed.get("cmds").is_some() {
        command::parse_stream(text)?
    } else {
        vec![value_to_command(&parsed)?]
    };

    let mut st = state.lock().unwrap();
    let mut response = Value::obj();
    // Split: session.* vs document commands.
    let mut doc_cmds = Vec::new();
    for c in &cmds {
        if c.op == "session.play" || c.op == "session.pause" || c.op == "session.seek" {
            match c.op.as_str() {
                "session.seek" => {
                    let t = c.arg("tick").and_then(|v| v.as_i64()).ok_or("session.seek: tick required")?;
                    st.tick = t;
                }
                _ => {} // play/pause run client-side; recorded for the log only
            }
            let mut ack = Value::obj();
            ack.set("tick", Value::Int(st.tick));
            response.set("session", ack);
        } else {
            doc_cmds.push(c.clone());
        }
    }
    if !doc_cmds.is_empty() {
        let apply_result = st.apply_commands(&doc_cmds)?;
        if let (Value::Obj(_), Value::Obj(_)) = (&apply_result, &response) {
            for (k, v) in apply_result.as_obj().unwrap_or(&[]).to_vec() {
                response.set(&k, v);
            }
        }
    }
    if response.as_obj().map(|o| o.is_empty()).unwrap_or(true) {
        response.set("applied", Value::Int(0));
    }
    Ok(response)
}

fn value_to_command(v: &Value) -> Result<Command, String> {
    let op = v.get("op").and_then(|x| x.as_str()).ok_or("command object needs 'op'")?.to_string();
    let args = match v {
        Value::Obj(fields) => fields.iter().filter(|(k, _)| k != "op").cloned().collect(),
        _ => Vec::new(),
    };
    Ok(Command { op, args })
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        if it.next() == Some(key) {
            return it.next().map(|v| v.to_string());
        }
    }
    None
}

fn respond_html(stream: TcpStream, body: &str) -> Result<(), String> {
    respond_bytes(stream, "text/html; charset=utf-8", body.as_bytes())
}

fn respond_json(stream: TcpStream, body: &str) -> Result<(), String> {
    respond_bytes(stream, "application/json", body.as_bytes())
}

fn respond_bytes(mut stream: TcpStream, content_type: &str, body: &[u8]) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        content_type,
        body.len()
    );
    stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn respond_status(mut stream: TcpStream, code: u16, reason: &str) -> Result<(), String> {
    let head = format!("HTTP/1.1 {} {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", code, reason);
    stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}
