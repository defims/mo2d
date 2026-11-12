//! `.mo2d` envelope: minimal MVP subset of docs/file-format.md §2.
//!
//! Envelope: `{format, version, minReader, generator, root{width,height,ticksPerSecond,fps},
//! lines, relations}` — no BLOB area, no refs, no history (mvp.md §2.5).
//!
//! Byte-level determinism (file-format.md §2.4): stable field order (the writer
//! emits fields in the canonical order below; unknown keys pass through after
//! the known ones, in their original relative order), no timestamps in the
//! body, atomic write (tmp file + rename).

use crate::json::{self, Value};
use crate::math::Vec3;
use crate::model::*;
use std::fs;
use std::io::Write;
use std::path::Path;

pub const FORMAT_NAME: &str = "mo2d";
pub const FORMAT_VERSION: i64 = 1;
pub const MIN_READER: i64 = 1;
pub const GENERATOR: &str = "mo2d 0.1.0";

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

pub fn doc_to_value(doc: &Document) -> Value {
    let mut root = Value::obj();
    root.set("width", Value::Int(doc.root.width as i64));
    root.set("height", Value::Int(doc.root.height as i64));
    root.set("ticksPerSecond", Value::Int(doc.root.ticks_per_second));
    root.set("fps", Value::Num(doc.root.fps));
    append_unknown(&mut root, &doc.root.unknown);

    let mut v = Value::obj();
    v.set("format", Value::Str(FORMAT_NAME.into()));
    v.set("version", Value::Int(FORMAT_VERSION));
    v.set("minReader", Value::Int(MIN_READER));
    v.set("generator", Value::Str(GENERATOR.into()));
    v.set("root", root);
    v.set("lines", Value::Arr(doc.lines.iter().map(line_to_value).collect()));
    v.set("relations", Value::Arr(doc.relations.iter().map(relation_to_value).collect()));
    append_unknown(&mut v, &doc.unknown);
    v
}

/// Canonical document bytes (trailing newline).
pub fn doc_to_bytes(doc: &Document) -> Result<Vec<u8>, String> {
    let text = json::to_string(&doc_to_value(doc))?;
    let mut bytes = text.into_bytes();
    bytes.push(b'\n');
    Ok(bytes)
}

fn append_unknown(v: &mut Value, unknown: &[(String, Value)]) {
    for (k, val) in unknown {
        if v.get(k).is_none() {
            v.set(k, val.clone());
        }
    }
}

fn color_value(c: [u8; 4]) -> Value {
    Value::Arr(c.iter().map(|b| Value::Int(*b as i64)).collect())
}

fn pos_value(p: &NodePos) -> Value {
    match p {
        NodePos::Geom(v) => Value::Arr(vec![Value::Num(v.x), Value::Num(v.y), Value::Num(v.z)]),
        NodePos::Tick(t) => Value::Arr(vec![Value::Int(*t)]),
    }
}

fn vec_value(v: Vec3) -> Value {
    Value::Arr(vec![Value::Num(v.x), Value::Num(v.y), Value::Num(v.z)])
}

fn line_to_value(l: &Line) -> Value {
    let mut o = Value::obj();
    o.set("id", Value::Str(l.id.clone()));
    if !l.tags.is_empty() {
        o.set("tags", Value::Arr(l.tags.iter().map(|t| Value::Str(t.clone())).collect()));
    }
    o.set("closed", Value::Bool(l.closed));
    o.set("width", Value::Num(l.width));
    o.set("color", color_value(l.color));
    o.set("fillColor", color_value(l.fill_color));
    o.set("strokeOn", Value::Bool(l.stroke_on));
    o.set("fillOn", Value::Bool(l.fill_on));
    o.set(
        "nodes",
        Value::Arr(
            l.nodes
                .iter()
                .map(|n| {
                    let mut no = Value::obj();
                    no.set("p", pos_value(&n.pos));
                    if let Some(w) = n.width {
                        no.set("w", Value::Num(w));
                    }
                    if let Some(c) = n.color {
                        no.set("c", color_value(c));
                    }
                    if let Some(h) = n.handle_in {
                        no.set("hi", vec_value(h));
                    }
                    if let Some(h) = n.handle_out {
                        no.set("ho", vec_value(h));
                    }
                    if let Some(val) = &n.value {
                        no.set("v", val.clone());
                    }
                    append_unknown(&mut no, &n.unknown);
                    no
                })
                .collect(),
        ),
    );
    o.set(
        "segs",
        Value::Arr(
            l.segs
                .iter()
                .map(|s| {
                    let mut so = Value::obj();
                    so.set("type", Value::Str(match s.seg_type { SegType::Straight => "straight", SegType::Curve => "curve" }.into()));
                    so.set("visible", Value::Bool(s.visible));
                    if let Some(interp) = &s.interp {
                        let mut io = Value::obj();
                        io.set("im", Value::Int(interp.im));
                        if let Some(b) = &interp.b {
                            io.set(
                                "b",
                                Value::Arr(
                                    b.iter()
                                        .map(|h| {
                                            let mut ho = Value::obj();
                                            ho.set("ao", Value::Num(h.ao));
                                            ho.set("ai", Value::Num(h.ai));
                                            ho.set("po", Value::Num(h.po));
                                            ho.set("pi", Value::Num(h.pi));
                                            ho
                                        })
                                        .collect(),
                                ),
                            );
                        }
                        so.set("interp", io);
                    }
                    append_unknown(&mut so, &s.unknown);
                    so
                })
                .collect(),
        ),
    );
    append_unknown(&mut o, &l.unknown);
    o
}

fn relation_to_value(r: &Relation) -> Value {
    let mut o = Value::obj();
    o.set("id", Value::Str(r.id.clone()));
    o.set("type", Value::Str(r.kind.name().into()));
    match r.kind {
        RelationKind::Bind => {
            o.set("target", Value::Str(r.bind_target.clone()));
            o.set("skeleton", Value::Str(r.bind_skeleton.clone()));
            o.set("falloff", Value::Num(r.bind_falloff));
            o.set("stretch", Value::Bool(r.bind_stretch));
        }
        RelationKind::Share => {
            let mut d = Value::obj();
            d.set("line", Value::Str(r.share_driver.0.clone()));
            d.set("node", Value::Int(r.share_driver.1 as i64));
            o.set("driver", d);
            let mut dr = Value::obj();
            dr.set("line", Value::Str(r.share_driven.0.clone()));
            dr.set("node", Value::Int(r.share_driven.1 as i64));
            o.set("driven", dr);
        }
        RelationKind::Track => {
            o.set("trackLine", Value::Str(r.track_line.clone()));
            let mut t = Value::obj();
            t.set("line", Value::Str(r.track_target_line.clone()));
            if let Some(n) = r.track_target_node {
                t.set("node", Value::Int(n as i64));
            }
            t.set("attr", Value::Str(r.track_attr.name().into()));
            o.set("target", t);
        }
    }
    append_unknown(&mut o, &r.unknown);
    o
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

/// Unknown keys of `obj` not present in `known`.
pub fn unknown_of(obj: &Value, known: &[&str]) -> Vec<(String, Value)> {
    match obj {
        Value::Obj(fields) => fields
            .iter()
            .filter(|(k, _)| !known.contains(&k.as_str()))
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
}

pub fn parse_doc(text: &str) -> Result<Document, String> {
    let v = json::parse(text)?;
    doc_from_value(&v)
}

pub fn doc_from_value(v: &Value) -> Result<Document, String> {
    let obj = v.as_obj().ok_or("document root must be a JSON object")?;
    let mut doc = Document::default();

    let fmt = v.get("format").and_then(|x| x.as_str()).unwrap_or("");
    if fmt != FORMAT_NAME {
        return Err(format!("not a mo2d document (format = '{}')", fmt));
    }
    if let Some(ver) = v.get("version") {
        let ver = ver.as_i64().ok_or("version must be an integer")?;
        if ver > FORMAT_VERSION {
            return Err(format!("document version {} is newer than this reader ({}); upgrade the engine", ver, FORMAT_VERSION));
        }
    }
    if let Some(minr) = v.get("minReader") {
        let minr = minr.as_i64().ok_or("minReader must be an integer")?;
        if minr > FORMAT_VERSION {
            return Err(format!("document requires reader >= {}, this reader is {}", minr, FORMAT_VERSION));
        }
    }

    if let Some(root) = v.get("root") {
        if let Some(w) = root.get("width") {
            doc.root.width = w.as_i64().ok_or("root.width must be an integer")? as u32;
        }
        if let Some(h) = root.get("height") {
            doc.root.height = h.as_i64().ok_or("root.height must be an integer")? as u32;
        }
        if let Some(t) = root.get("ticksPerSecond") {
            doc.root.ticks_per_second = t.as_i64().ok_or("root.ticksPerSecond must be an integer")?;
        }
        if let Some(f) = root.get("fps") {
            doc.root.fps = f.as_f64().ok_or("root.fps must be a number")?;
        }
        doc.root.unknown = unknown_of(root, &["width", "height", "ticksPerSecond", "fps"]);
    }

    if let Some(lines) = v.get("lines").and_then(|x| x.as_arr()) {
        for lv in lines {
            doc.lines.push(line_from_value(lv)?);
        }
    }
    if let Some(rels) = v.get("relations").and_then(|x| x.as_arr()) {
        for rv in rels {
            doc.relations.push(relation_from_value(rv)?);
        }
    }

    let _ = obj;
    doc.unknown = unknown_of(v, &["format", "version", "minReader", "generator", "root", "lines", "relations"]);
    Ok(doc)
}

fn parse_pos(v: &Value, ctx: &str) -> Result<NodePos, String> {
    let arr = v.as_arr().ok_or_else(|| format!("{}: 'p' must be an array", ctx))?;
    match arr.len() {
        1 => {
            let t = arr[0].as_i64().ok_or_else(|| format!("{}: track tick must be an integer", ctx))?;
            if t.abs() > TICK_LIMIT {
                return Err(format!("{}: tick {} out of range (±{}); clamping is a writer job, reader rejects", ctx, t, TICK_LIMIT));
            }
            Ok(NodePos::Tick(t))
        }
        3 => {
            let x = arr[0].as_f64().ok_or_else(|| format!("{}: p[0] must be a number", ctx))?;
            let y = arr[1].as_f64().ok_or_else(|| format!("{}: p[1] must be a number", ctx))?;
            let z = arr[2].as_f64().ok_or_else(|| format!("{}: p[2] must be a number", ctx))?;
            Ok(NodePos::Geom(Vec3::new(x, y, z)))
        }
        n => Err(format!("{}: 'p' must have 1 (tick) or 3 (vec3) components, got {}", ctx, n)),
    }
}

fn parse_color(v: &Value, ctx: &str) -> Result<[u8; 4], String> {
    let arr = v.as_arr().ok_or_else(|| format!("{}: color must be an array [r,g,b,a]", ctx))?;
    if arr.len() != 4 {
        return Err(format!("{}: color must have 4 components", ctx));
    }
    let mut out = [0u8; 4];
    for (i, c) in arr.iter().enumerate() {
        let n = c.as_i64().ok_or_else(|| format!("{}: color component {} must be an integer 0-255", ctx, i))?;
        if !(0..=255).contains(&n) {
            return Err(format!("{}: color component {} out of range 0-255: {}", ctx, i, n));
        }
        out[i] = n as u8;
    }
    Ok(out)
}

fn parse_vec3(v: &Value, ctx: &str) -> Result<Vec3, String> {
    let arr = v.as_arr().ok_or_else(|| format!("{}: expected [x,y,z]", ctx))?;
    if arr.len() != 3 {
        return Err(format!("{}: expected 3 components", ctx));
    }
    Ok(Vec3::new(
        arr[0].as_f64().ok_or_else(|| format!("{}: x must be a number", ctx))?,
        arr[1].as_f64().ok_or_else(|| format!("{}: y must be a number", ctx))?,
        arr[2].as_f64().ok_or_else(|| format!("{}: z must be a number", ctx))?,
    ))
}

fn line_from_value(v: &Value) -> Result<Line, String> {
    let id = v.get("id").and_then(|x| x.as_str()).ok_or("line.id must be a string")?.to_string();
    let ctx = format!("line '{}'", id);
    let mut l = Line::new(&id);
    if let Some(t) = v.get("tags").and_then(|x| x.as_arr()) {
        for tag in t {
            l.tags.push(tag.as_str().ok_or_else(|| format!("{}: tags must be strings", ctx))?.to_string());
        }
    }
    if let Some(b) = v.get("closed").and_then(|x| x.as_bool()) {
        l.closed = b;
    }
    if let Some(w) = v.get("width").and_then(|x| x.as_f64()) {
        l.width = w;
    }
    if let Some(c) = v.get("color") {
        l.color = parse_color(c, &ctx)?;
    }
    if let Some(c) = v.get("fillColor") {
        l.fill_color = parse_color(c, &ctx)?;
    }
    if let Some(b) = v.get("strokeOn").and_then(|x| x.as_bool()) {
        l.stroke_on = b;
    }
    if let Some(b) = v.get("fillOn").and_then(|x| x.as_bool()) {
        l.fill_on = b;
    }
    if let Some(nodes) = v.get("nodes").and_then(|x| x.as_arr()) {
        for (i, nv) in nodes.iter().enumerate() {
            let nctx = format!("{} node {}", ctx, i);
            let p = parse_pos(nv.get("p").ok_or_else(|| format!("{}: missing 'p'", nctx))?, &nctx)?;
            let mut n = match p {
                NodePos::Geom(g) => Node::geom(g),
                NodePos::Tick(t) => {
                    let val = nv.get("v").cloned().ok_or_else(|| format!("{}: track node needs a value 'v'", nctx))?;
                    Node::track(t, val)
                }
            };
            if let Some(w) = nv.get("w") {
                n.width = Some(w.as_f64().ok_or_else(|| format!("{}: 'w' must be a number", nctx))?);
            }
            if let Some(c) = nv.get("c") {
                n.color = Some(parse_color(c, &nctx)?);
            }
            if let Some(h) = nv.get("hi") {
                n.handle_in = Some(parse_vec3(h, &nctx)?);
            }
            if let Some(h) = nv.get("ho") {
                n.handle_out = Some(parse_vec3(h, &nctx)?);
            }
            n.unknown = unknown_of(nv, &["p", "w", "c", "hi", "ho", "v"]);
            l.nodes.push(n);
        }
    }
    if let Some(segs) = v.get("segs").and_then(|x| x.as_arr()) {
        for (i, sv) in segs.iter().enumerate() {
            let sctx = format!("{} seg {}", ctx, i);
            let mut s = SegAttrs::default();
            if let Some(t) = sv.get("type").and_then(|x| x.as_str()) {
                s.seg_type = match t {
                    "straight" => SegType::Straight,
                    "curve" => SegType::Curve,
                    other => return Err(format!("{}: unknown segment type '{}'", sctx, other)),
                };
            }
            if let Some(b) = sv.get("visible").and_then(|x| x.as_bool()) {
                s.visible = b;
            }
            if let Some(iv) = sv.get("interp") {
                let im = iv.get("im").and_then(|x| x.as_i64()).ok_or_else(|| format!("{}: interp.im must be an integer", sctx))?;
                let mut b = None;
                if let Some(bv) = iv.get("b").and_then(|x| x.as_arr()) {
                    let mut handles = Vec::new();
                    for (j, hv) in bv.iter().enumerate() {
                        let hctx = format!("{} interp handle {}", sctx, j);
                        let mut h = BezierHandle::default_handle();
                        if let Some(x) = hv.get("ao").and_then(|x| x.as_f64()) { h.ao = x; }
                        if let Some(x) = hv.get("ai").and_then(|x| x.as_f64()) { h.ai = x; }
                        if let Some(x) = hv.get("po").and_then(|x| x.as_f64()) { h.po = x; }
                        if let Some(x) = hv.get("pi").and_then(|x| x.as_f64()) { h.pi = x; }
                        let _ = hctx;
                        handles.push(h);
                    }
                    b = Some(handles);
                }
                s.interp = Some(Interp { im, b });
            }
            s.unknown = unknown_of(sv, &["type", "visible", "interp"]);
            l.segs.push(s);
        }
    }
    l.sync_segs();
    l.unknown = unknown_of(v, &["id", "tags", "closed", "width", "color", "fillColor", "strokeOn", "fillOn", "nodes", "segs"]);
    Ok(l)
}

fn relation_from_value(v: &Value) -> Result<Relation, String> {
    let id = v.get("id").and_then(|x| x.as_str()).ok_or("relation.id must be a string")?.to_string();
    let ty = v.get("type").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': missing type", id))?;
    let mut r = match ty {
        "bind" => {
            let target = v.get("target").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': bind needs target", id))?;
            let skeleton = v.get("skeleton").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': bind needs skeleton", id))?;
            Relation::new_bind(&id, target, skeleton)
        }
        "share" => {
            let d = v.get("driver").ok_or_else(|| format!("relation '{}': share needs driver", id))?;
            let dr = v.get("driven").ok_or_else(|| format!("relation '{}': share needs driven", id))?;
            let dl = d.get("line").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': driver.line missing", id))?;
            let dn = d.get("node").and_then(|x| x.as_i64()).ok_or_else(|| format!("relation '{}': driver.node missing", id))?;
            let rl = dr.get("line").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': driven.line missing", id))?;
            let rn = dr.get("node").and_then(|x| x.as_i64()).ok_or_else(|| format!("relation '{}': driven.node missing", id))?;
            Relation::new_share(&id, (dl, dn as usize), (rl, rn as usize))
        }
        "track" => {
            let tl = v.get("trackLine").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': track needs trackLine", id))?;
            let t = v.get("target").ok_or_else(|| format!("relation '{}': track needs target", id))?;
            let line = t.get("line").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': target.line missing", id))?;
            let node = t.get("node").and_then(|x| x.as_i64()).map(|n| n as usize);
            let attr = match t.get("attr").and_then(|x| x.as_str()) {
                Some("position") => TrackAttr::Position,
                Some("width") => TrackAttr::Width,
                other => return Err(format!("relation '{}': target.attr must be 'position' or 'width', got {:?}", id, other)),
            };
            Relation::new_track(&id, tl, line, node, attr)
        }
        other => return Err(format!("relation '{}': unknown type '{}'", id, other)),
    };
    if let Some(f) = v.get("falloff").and_then(|x| x.as_f64()) {
        r.bind_falloff = f;
    }
    if let Some(b) = v.get("stretch").and_then(|x| x.as_bool()) {
        r.bind_stretch = b;
    }
    r.unknown = unknown_of(v, &["id", "type", "target", "skeleton", "falloff", "stretch", "driver", "driven", "trackLine"]);
    Ok(r)
}

// ---------------------------------------------------------------------------
// Atomic write
// ---------------------------------------------------------------------------

/// Atomic write: write to `path.tmp-<pid>` then rename over `path`
/// (file-format.md §2.4, vectorcraft practice).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("invalid path (no parent)")?;
    let tmp = dir.join(format!(
        ".{}.tmp-{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("mo2d"),
        std::process::id()
    ));
    {
        let mut f = fs::File::create(&tmp).map_err(|e| format!("create {}: {}", tmp.display(), e))?;
        f.write_all(bytes).map_err(|e| format!("write {}: {}", tmp.display(), e))?;
        f.sync_all().map_err(|e| format!("sync {}: {}", tmp.display(), e))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("rename {} -> {}: {}", tmp.display(), path.display(), e))?;
    Ok(())
}

pub fn save_doc(path: &Path, doc: &Document) -> Result<(), String> {
    let bytes = doc_to_bytes(doc)?;
    write_atomic(path, &bytes)
}

pub fn load_doc(path: &Path) -> Result<Document, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    parse_doc(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_byte_equal() {
        let mut doc = Document::default();
        let mut l = Line::new("body");
        l.nodes.push(Node::geom(Vec3::new(10.0, 20.0, 0.0)));
        l.nodes.push(Node::geom(Vec3::new(30.0, 40.0, 0.0)));
        l.segs.push(SegAttrs { seg_type: SegType::Curve, ..Default::default() });
        doc.lines.push(l);
        doc.relations.push(Relation::new_bind("b1", "body", "skel"));

        let b1 = doc_to_bytes(&doc).unwrap();
        let doc2 = parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
        let b2 = doc_to_bytes(&doc2).unwrap();
        assert_eq!(b1, b2, "save→load→save must be byte-identical");
    }

    #[test]
    fn unknown_keys_pass_through() {
        let src = r#"{
  "format": "mo2d",
  "version": 1,
  "minReader": 1,
  "generator": "mo2d 0.1.0",
  "root": {"width": 640, "height": 360, "ticksPerSecond": 254016000000, "fps": 24.0, "vendor.cool": {"a": 1}},
  "lines": [],
  "relations": [],
  "vendor.extra": true
}"#;
        let doc = parse_doc(src).unwrap();
        assert_eq!(doc.unknown.len(), 1);
        assert_eq!(doc.root.unknown.len(), 1);
        let b1 = doc_to_bytes(&doc).unwrap();
        let doc2 = parse_doc(std::str::from_utf8(&b1).unwrap()).unwrap();
        let b2 = doc_to_bytes(&doc2).unwrap();
        assert_eq!(b1, b2, "unknown keys must survive round-trips stably");
        assert!(std::str::from_utf8(&b2).unwrap().contains("vendor.cool"));
    }
}
