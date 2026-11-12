//! MCP server mode: the engine as an MCP server over stdio (newline-delimited
//! JSON-RPC 2.0). Three tools (mvp.md §2.4): apply / list / screenshot.
//!
//! This is the AI channel. The hand channel (stage) speaks the very same
//! command protocol — dual-channel isomorphism is the A6 gate.

use crate::command::{self, Command};
use crate::eval;
use crate::json::{self, Value};
use crate::model::Document;
use crate::png::encode_png;
use crate::assets::AssetStore;
use crate::render::{render_with_assets, Camera, RenderOpts};
use crate::sha::sha256_hex;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub struct McpSession {
    pub doc: Document,
    pub doc_path: PathBuf,
    /// Session command log (document commands applied this session).
    pub log: Vec<Command>,
    /// Auto-save after each apply (CLI decides).
    pub autosave: bool,
}

impl McpSession {
    pub fn new(doc_path: &Path, autosave: bool) -> Result<McpSession, String> {
        let doc = if doc_path.exists() {
            crate::format::load_doc(doc_path)?
        } else {
            Document::default()
        };
        Ok(McpSession { doc, doc_path: doc_path.to_path_buf(), log: Vec::new(), autosave })
    }

    fn save_if_needed(&mut self) -> Result<(), String> {
        if self.autosave {
            self.save()
        } else {
            Ok(())
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        crate::validate::verify_before_save(&self.doc)?;
        crate::format::save_doc(&self.doc_path, &self.doc)
    }

    /// Tool: apply a command or command list (the protocol is identical to the
    /// file/CLI stream: {"v":1,"cmds":[...]} — either the envelope or a bare
    /// array or a single command object is accepted here for convenience).
    pub fn tool_apply(&mut self, args: &Value) -> Result<Value, String> {
        let cmds: Vec<Command> = if let Some(arr) = args.get("cmds").and_then(|v| v.as_arr()) {
            let mut envelope = Value::obj();
            envelope.set("v", Value::Int(command::PROTOCOL_VERSION));
            envelope.set("cmds", Value::Arr(arr.to_vec()));
            command::parse_stream(&json::to_string(&envelope)?)?
        } else if let Some(arr) = args.as_arr() {
            let mut envelope = Value::obj();
            envelope.set("v", Value::Int(command::PROTOCOL_VERSION));
            envelope.set("cmds", Value::Arr(arr.to_vec()));
            command::parse_stream(&json::to_string(&envelope)?)?
        } else {
            // Single command object (or envelope with "cmds").
            let mut obj = args.clone();
            if obj.get("v").is_none() {
                obj.set("v", Value::Int(command::PROTOCOL_VERSION));
            }
            if obj.get("cmds").is_none() {
                // Wrap a lone command into a one-command stream.
                let mut envelope = Value::obj();
                envelope.set("v", Value::Int(command::PROTOCOL_VERSION));
                envelope.set("cmds", Value::Arr(vec![obj.clone()]));
                obj = envelope;
            }
            command::parse_stream(&json::to_string(&obj)?)?
        };

        let effects = command::apply_stream(&mut self.doc, &cmds)?;
        self.log.extend(cmds.iter().cloned());
        let applied = effects.len();
        self.save_if_needed()?;
        let doc_bytes = crate::format::doc_to_bytes(&self.doc)?;
        let mut out = Value::obj();
        out.set("applied", Value::Int(applied as i64));
        out.set("docSha256", Value::Str(sha256_hex(&doc_bytes)));
        Ok(out)
    }

    /// Tool: list the command registry.
    pub fn tool_list(&self) -> Result<Value, String> {
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
        Ok(out)
    }

    /// Tool: render the document at a tick to a PNG file.
    pub fn tool_screenshot(&self, args: &Value, out_dir: &Path) -> Result<Value, String> {
        let tick = args.get("tick").and_then(|v| v.as_i64()).unwrap_or(0);
        let width = args.get("width").and_then(|v| v.as_i64()).unwrap_or(self.doc.root.width as i64) as u32;
        let height = args.get("height").and_then(|v| v.as_i64()).unwrap_or(self.doc.root.height as i64) as u32;
        let camera = parse_camera(args.get("camera"))?;
        let lines = eval::evaluate(&self.doc, tick)?;
        let mut assets = AssetStore::new();
        assets.root = self.doc_path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        // B7: packaged documents resolve textures from their blob area.
        if let Ok(loaded) = crate::format::load_doc_full(&self.doc_path) {
            assets.attach_blobs(&loaded.blobs);
        }
        let img = render_with_assets(
            &lines,
            self.doc.root.width as f64,
            self.doc.root.height as f64,
            &RenderOpts { width, height, camera, samples: 1 },
            &mut assets,
        );
        let png = encode_png(&img);
        let name = format!("screenshot_{}.png", tick);
        let path = out_dir.join(&name);
        std::fs::create_dir_all(out_dir).map_err(|e| format!("create {}: {}", out_dir.display(), e))?;
        std::fs::write(&path, &png).map_err(|e| format!("write {}: {}", path.display(), e))?;
        let mut out = Value::obj();
        out.set("tick", Value::Int(tick));
        out.set("path", Value::Str(path.to_string_lossy().into_owned()));
        out.set("sha256", Value::Str(sha256_hex(&png)));
        Ok(out)
    }
}

/// Parse a camera spec: {"type":"ortho2d"} or
/// {"type":"perspective","eye":[..],"target":[..],"up"?:[..],"fovY":deg}.
pub fn parse_camera(v: Option<&Value>) -> Result<Camera, String> {
    let Some(v) = v else { return Ok(Camera::Ortho2d) };
    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("ortho2d");
    match ty {
        "ortho2d" | "ortho" => Ok(Camera::Ortho2d),
        "perspective" => {
            let vec = |key: &str, default: [f64; 3]| -> Result<[f64; 3], String> {
                match v.get(key) {
                    Some(a) => {
                        let arr = a.as_arr().ok_or(format!("camera.{} must be [x,y,z]", key))?;
                        if arr.len() != 3 {
                            return Err(format!("camera.{} must have 3 components", key));
                        }
                        Ok([
                            arr[0].as_f64().ok_or(format!("camera.{} component", key))?,
                            arr[1].as_f64().ok_or(format!("camera.{} component", key))?,
                            arr[2].as_f64().ok_or(format!("camera.{} component", key))?,
                        ])
                    }
                    None => Ok(default),
                }
            };
            let eye = vec("eye", [0.0, 0.0, 400.0])?;
            let target = vec("target", [0.0, 0.0, 0.0])?;
            let up = vec("up", [0.0, 1.0, 0.0])?;
            let fov = v.get("fovY").and_then(|f| f.as_f64()).unwrap_or(45.0);
            Ok(Camera::Perspective {
                eye: crate::math::Vec3::new(eye[0], eye[1], eye[2]),
                target: crate::math::Vec3::new(target[0], target[1], target[2]),
                up: crate::math::Vec3::new(up[0], up[1], up[2]),
                fov_y_deg: fov,
                near: v.get("near").and_then(|f| f.as_f64()).unwrap_or(1.0),
                far: v.get("far").and_then(|f| f.as_f64()).unwrap_or(5000.0),
            })
        }
        other => Err(format!("unknown camera type '{}'", other)),
    }
}

/// Run the stdio server loop. Returns when stdin closes.
pub fn serve(session: &mut McpSession, out_dir: &Path) -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut line = String::new();
    loop {
        line.clear();
        let n = lock.read_line(&mut line).map_err(|e| format!("stdin: {}", e))?;
        if n == 0 {
            return Ok(()); // EOF — session over
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg = match json::parse(trimmed) {
            Ok(m) => m,
            Err(e) => {
                respond_err(null_id(), -32700, &format!("parse error: {}", e))?;
                continue;
            }
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        match method {
            "initialize" => {
                let mut result = Value::obj();
                result.set("protocolVersion", Value::Str("2024-11-05".into()));
                let mut caps = Value::obj();
                let mut tools = Value::obj();
                tools.set("listChanged", Value::Bool(false));
                caps.set("tools", tools);
                result.set("capabilities", caps);
                let mut info = Value::obj();
                info.set("name", Value::Str("mo2d".into()));
                info.set("version", Value::Str("0.1.0".into()));
                result.set("serverInfo", info);
                respond_ok(id, result)?;
            }
            "notifications/initialized" | "initialized" => {
                // notification — no response
            }
            "ping" => respond_ok(id, Value::obj())?,
            "tools/list" => {
                let mut result = Value::obj();
                let tools = vec![
                    tool_spec(
                        "apply",
                        "Apply document commands (the mo2d command protocol: an envelope {v:1, cmds:[...]}, a bare array, or a single command object). Document is validated and saved after applying.",
                        vec![
                            ("type", "object"),
                            ("description", "Command(s) to apply"),
                        ],
                    ),
                    tool_spec("list", "List the mo2d command registry (protocol version + every command with its argument shape).", vec![]),
                    tool_spec(
                        "screenshot",
                        "Render the current document to a PNG and return its path + sha256 hash.",
                        vec![
                            ("tick", "integer"),
                            ("width", "integer"),
                            ("height", "integer"),
                            ("camera", "object: {type: ortho2d|perspective, eye, target, up, fovY}"),
                        ],
                    ),
                ];
                result.set("tools", Value::Arr(tools));
                respond_ok(id, result)?;
            }
            "tools/call" => {
                let name = msg.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string();
                let args = msg.get("params").and_then(|p| p.get("arguments")).cloned().unwrap_or(Value::obj());
                let result = match name.as_str() {
                    "apply" => session.tool_apply(&args).map(|v| (v, false)),
                    "list" => session.tool_list().map(|v| (v, false)),
                    "screenshot" => session.tool_screenshot(&args, out_dir).map(|v| (v, false)),
                    other => Err(format!("unknown tool '{}'", other)),
                };
                match result {
                    Ok((content, _is_error)) => {
                        let mut r = Value::obj();
                        r.set("content", Value::Arr(vec![text_content(&content)]));
                        respond_ok(id, r)?;
                    }
                    Err(e) => {
                        // Tool errors are results with isError, not JSON-RPC errors.
                        let mut r = Value::obj();
                        let mut c = Value::obj();
                        c.set("type", Value::Str("text".into()));
                        c.set("text", Value::Str(format!("error: {}", e)));
                        r.set("content", Value::Arr(vec![c]));
                        r.set("isError", Value::Bool(true));
                        respond_ok(id, r)?;
                    }
                }
            }
            other => {
                respond_err(id, -32601, &format!("method not found: {}", other))?;
            }
        }
    }
}

fn tool_spec(name: &str, desc: &str, props: Vec<(&str, &str)>) -> Value {
    let mut t = Value::obj();
    t.set("name", Value::Str(name.into()));
    t.set("description", Value::Str(desc.into()));
    let mut schema = Value::obj();
    schema.set("type", Value::Str("object".into()));
    let mut properties = Value::obj();
    for (k, ty) in props {
        let mut p = Value::obj();
        p.set("type", Value::Str(ty.into()));
        properties.set(k, p);
    }
    schema.set("properties", properties);
    t.set("inputSchema", schema);
    t
}

fn text_content(v: &Value) -> Value {
    let mut c = Value::obj();
    c.set("type", Value::Str("text".into()));
    c.set("text", Value::Str(json::to_string(v).unwrap_or_default()));
    c
}

fn null_id() -> Value {
    Value::Null
}

fn respond_ok(id: Value, result: Value) -> Result<(), String> {
    let mut resp = Value::obj();
    resp.set("jsonrpc", Value::Str("2.0".into()));
    resp.set("id", id);
    resp.set("result", result);
    write_line(&resp)
}

fn respond_err(id: Value, code: i64, message: &str) -> Result<(), String> {
    let mut resp = Value::obj();
    resp.set("jsonrpc", Value::Str("2.0".into()));
    resp.set("id", id);
    let mut err = Value::obj();
    err.set("code", Value::Int(code));
    err.set("message", Value::Str(message.into()));
    resp.set("error", err);
    write_line(&resp)
}

fn write_line(v: &Value) -> Result<(), String> {
    let text = json::to_string_compact(v)?;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(text.as_bytes()).map_err(|e| format!("stdout: {}", e))?;
    lock.write_all(b"\n").map_err(|e| format!("stdout: {}", e))?;
    lock.flush().map_err(|e| format!("stdout: {}", e))?;
    Ok(())
}
