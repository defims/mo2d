//! Shared helpers for the A1–A6 gate tests.

use mo2d::command;
use mo2d::format;
use mo2d::model::Document;
use mo2d::render::{render, Camera, RenderOpts};
use mo2d::sha::sha256_hex;
use std::path::{Path, PathBuf};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

pub fn demo_dir() -> PathBuf {
    repo_root().join("demo")
}

/// A frozen acceptance reference (mvp.md P1-7: hash freeze + human eyeball;
/// the true oracle gate is roadmap B1).
#[derive(Debug, Clone)]
pub struct Frozen {
    pub raw: serde_less::Value,
}

/// Minimal read-only JSON access for frozen.json (no external crates).
pub mod serde_less {
    #[derive(Debug, Clone)]
    pub enum Value {
        Str(String),
        Int(i64),
        Arr(Vec<Value>),
        Obj(Vec<(String, Value)>),
    }
    impl Value {
        pub fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::Str(s) => Some(s),
                _ => None,
            }
        }
        pub fn as_i64(&self) -> Option<i64> {
            match self {
                Value::Int(i) => Some(*i),
                _ => None,
            }
        }
        pub fn as_arr(&self) -> Option<&[Value]> {
            match self {
                Value::Arr(a) => Some(a),
                _ => None,
            }
        }
    }

    /// Parse the frozen.json subset: objects, arrays, strings, integers.
    pub fn parse(text: &str) -> Result<Value, String> {
        let mut p = P { b: text.as_bytes(), i: 0 };
        p.ws();
        let v = p.value()?;
        Ok(v)
    }
    struct P<'a> {
        b: &'a [u8],
        i: usize,
    }
    impl<'a> P<'a> {
        fn ws(&mut self) {
            while self.i < self.b.len() && (self.b[self.i] as char).is_whitespace() {
                self.i += 1;
            }
        }
        fn value(&mut self) -> Result<Value, String> {
            self.ws();
            match self.b.get(self.i) {
                Some(b'{') => self.obj(),
                Some(b'[') => self.arr(),
                Some(b'"') => Ok(Value::Str(self.string()?)),
                Some(c) if c.is_ascii_digit() || *c == b'-' => {
                    let start = self.i;
                    if self.b[self.i] == b'-' {
                        self.i += 1;
                    }
                    while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                        self.i += 1;
                    }
                    std::str::from_utf8(&self.b[start..self.i])
                        .unwrap()
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|e| e.to_string())
                }
                other => Err(format!("unexpected {:?} in frozen.json", other)),
            }
        }
        fn obj(&mut self) -> Result<Value, String> {
            self.i += 1; // {
            let mut fields = Vec::new();
            loop {
                self.ws();
                if self.b.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Value::Obj(fields));
                }
                let key = self.string()?;
                self.ws();
                if self.b.get(self.i) != Some(&b':') {
                    return Err("expected :".into());
                }
                self.i += 1;
                let v = self.value()?;
                fields.push((key, v));
                self.ws();
                match self.b.get(self.i) {
                    Some(b',') => self.i += 1,
                    Some(b'}') => {
                        self.i += 1;
                        return Ok(Value::Obj(fields));
                    }
                    _ => return Err("expected , or }".into()),
                }
            }
        }
        fn arr(&mut self) -> Result<Value, String> {
            self.i += 1; // [
            let mut items = Vec::new();
            loop {
                self.ws();
                if self.b.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Value::Arr(items));
                }
                items.push(self.value()?);
                self.ws();
                match self.b.get(self.i) {
                    Some(b',') => self.i += 1,
                    Some(b']') => {
                        self.i += 1;
                        return Ok(Value::Arr(items));
                    }
                    _ => return Err("expected , or ]".into()),
                }
            }
        }
        fn string(&mut self) -> Result<String, String> {
            if self.b.get(self.i) != Some(&b'"') {
                return Err("expected string".into());
            }
            self.i += 1;
            let mut s = String::new();
            while self.i < self.b.len() && self.b[self.i] != b'"' {
                if self.b[self.i] == b'\\' {
                    self.i += 1;
                    match self.b.get(self.i) {
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(c) => s.push(*c as char),
                        None => return Err("bad escape".into()),
                    }
                } else {
                    s.push(self.b[self.i] as char);
                }
                self.i += 1;
            }
            self.i += 1; // closing quote
            Ok(s)
        }
    }
}

pub fn load_frozen() -> Frozen {
    let path = demo_dir().join("frozen.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {} (run the freeze step first)", path.display(), e));
    Frozen { raw: serde_less::parse(&text).expect("parse frozen.json") }
}

pub fn replay_stream(path: &Path) -> Document {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| format!("read {}: {}", path.display(), e));
    let cmds = command::parse_stream(&text).expect("parse command stream");
    let mut doc = Document::default();
    command::apply_stream(&mut doc, &cmds).expect("apply stream");
    doc
}

pub fn doc_bytes(doc: &Document) -> Vec<u8> {
    format::doc_to_bytes(doc).expect("serialize doc")
}

pub fn render_sha(doc: &Document, tick: i64, camera: Camera) -> String {
    let lines = mo2d::eval::evaluate(doc, tick).expect("evaluate");
    let img = render(
        &lines,
        doc.root.width as f64,
        doc.root.height as f64,
        &RenderOpts { width: doc.root.width, height: doc.root.height, camera, samples: 1 },
    );
    let png = mo2d::png::encode_png(&img);
    sha256_hex(&png)
}

pub fn scene3d_camera() -> Camera {
    // The frozen 3D scene camera (mvp.md A1 3D static scene gate).
    let v = mo2d::json::parse(r#"{"type":"perspective","eye":[320,200,600],"target":[320,190,120],"fovY":42}"#).unwrap();
    mo2d::mcp::parse_camera(Some(&v)).unwrap()
}
