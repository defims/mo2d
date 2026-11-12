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
// B7 package form: JSON header + tail BLOB area (single-file .mo2d)
// ---------------------------------------------------------------------------
//
// Layout (byte-deterministic; file-format.md §2):
//
//   <canonical JSON header>\n            (doc_to_bytes — ends with a newline)
//   --BLOBS--\n                          (marker line; cannot occur inside a
//                                         JSON string — raw newlines are
//                                         escaped in JSON)
//   <name>\t<length>\n<length bytes>     (records in sorted-name order)
//
// The same document loads from the package OR the unpacked directory
// (document.mo2d.json + assets/): texture references resolve against the
// blob area first, then the filesystem — two containers, one document.

pub const BLOB_MARKER: &str = "--BLOBS--";

/// A loaded document plus its blob area (empty for plain JSON documents).
pub struct LoadedDoc {
    pub doc: Document,
    /// Blob entries (name → raw bytes), sorted by name.
    pub blobs: Vec<(String, Vec<u8>)>,
}

/// Validate a blob name (must be a single line without tabs).
fn check_blob_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("blob name must not be empty".into());
    }
    if name.contains('\t') || name.contains('\n') || name.contains('\r') {
        return Err(format!("blob name '{}' contains line breaks or tabs", name));
    }
    Ok(())
}

/// Serialize the document as a single-file package: canonical JSON header +
/// marker + blob records (sorted by name).
pub fn doc_to_package_bytes(doc: &Document, blobs: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut out = doc_to_bytes(doc)?;
    if !blobs.is_empty() {
        out.extend_from_slice(BLOB_MARKER.as_bytes());
        out.push(b'\n');
        let mut sorted: Vec<&(String, Vec<u8>)> = blobs.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, data) in sorted {
            check_blob_name(name)?;
            out.extend_from_slice(name.as_bytes());
            out.push(b'\t');
            out.extend_from_slice(data.len().to_string().as_bytes());
            out.push(b'\n');
            out.extend_from_slice(data);
        }
    }
    Ok(out)
}

/// Load a document from a plain JSON file OR a package file (marker
/// detected → blob area parsed). Blob records are length-delimited, so
/// binary payload bytes never confound the parser.
pub fn load_doc_full(path: &Path) -> Result<LoadedDoc, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    let marker = format!("\n{}\n", BLOB_MARKER).into_bytes();
    let split = bytes
        .windows(marker.len())
        .position(|w| w == marker.as_slice());
    match split {
        None => {
            let text = String::from_utf8(bytes).map_err(|_| "document is not UTF-8".to_string())?;
            Ok(LoadedDoc { doc: parse_doc(&text)?, blobs: Vec::new() })
        }
        Some(pos) => {
            // Header = everything up to and including the newline before the marker.
            let header = &bytes[..pos + 1];
            let text = String::from_utf8(header.to_vec()).map_err(|_| "document header is not UTF-8".to_string())?;
            let doc = parse_doc(&text)?;
            let mut rest = &bytes[pos + marker.len()..];
            let mut blobs: Vec<(String, Vec<u8>)> = Vec::new();
            while !rest.is_empty() {
                let line_end = rest
                    .iter()
                    .position(|&b| b == b'\n')
                    .ok_or_else(|| "package: truncated blob record header".to_string())?;
                let line = std::str::from_utf8(&rest[..line_end]).map_err(|_| "package: blob record header not UTF-8")?;
                let (name, len_str) = line.split_once('\t').ok_or_else(|| format!("package: blob record '{}' lacks name\\tlength", line))?;
                check_blob_name(name)?;
                let len: usize = len_str.parse().map_err(|_| format!("package: blob '{}' has bad length '{}'", name, len_str))?;
                rest = &rest[line_end + 1..];
                if rest.len() < len {
                    return Err(format!("package: blob '{}' truncated (want {} bytes, have {})", name, len, rest.len()));
                }
                blobs.push((name.to_string(), rest[..len].to_vec()));
                rest = &rest[len..];
            }
            blobs.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(LoadedDoc { doc, blobs })
        }
    }
}

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
    // Styles (B2) emitted only when present — documents without styles keep
    // their exact legacy canonical bytes.
    if !doc.styles.is_empty() {
        v.set("styles", Value::Arr(doc.styles.iter().map(style_to_value).collect()));
    }
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
    if let Some(b) = &l.brush {
        o.set("brush", brush_to_value(b));
    }
    if let Some(t) = &l.texture {
        o.set("texture", Value::Str(t.clone()));
    }
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
                    if let Some(pr) = n.pressure {
                        no.set("pr", Value::Num(pr));
                    }
                    if let Some(uv) = n.uv {
                        no.set("uv", Value::Arr(vec![Value::Num(uv[0]), Value::Num(uv[1])]));
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

// ---------------------------------------------------------------------------
// Brush + style (B2)
// ---------------------------------------------------------------------------

pub const BRUSH_KEYS: &[&str] = &[
    "name", "align", "jitter", "spacing", "angleDrift", "randomize", "mergedAlpha", "tint", "randOrder",
    "sizeAmp", "sizeScale", "randomInterval", "hueDrift", "satDrift", "valDrift",
];

pub fn brush_to_value(b: &Brush) -> Value {
    let mut o = Value::obj();
    if let Some(x) = &b.name {
        o.set("name", Value::Str(x.clone()));
    }
    if let Some(x) = b.align {
        o.set("align", Value::Num(x));
    }
    if let Some(x) = b.jitter {
        o.set("jitter", Value::Num(x));
    }
    if let Some(x) = b.spacing {
        o.set("spacing", Value::Num(x));
    }
    if let Some(x) = b.angle_drift {
        o.set("angleDrift", Value::Num(x));
    }
    if let Some(x) = b.randomize {
        o.set("randomize", Value::Bool(x));
    }
    if let Some(x) = b.merged_alpha {
        o.set("mergedAlpha", Value::Bool(x));
    }
    if let Some(x) = b.tint {
        o.set("tint", color_value(x));
    }
    if let Some(x) = b.rand_order {
        o.set("randOrder", Value::Bool(x));
    }
    if let Some(x) = b.size_amp {
        o.set("sizeAmp", Value::Num(x));
    }
    if let Some(x) = b.size_scale {
        o.set("sizeScale", Value::Num(x));
    }
    if let Some(x) = b.random_interval {
        o.set("randomInterval", Value::Num(x));
    }
    if let Some(x) = b.hue_drift {
        o.set("hueDrift", Value::Num(x));
    }
    if let Some(x) = b.sat_drift {
        o.set("satDrift", Value::Num(x));
    }
    if let Some(x) = b.val_drift {
        o.set("valDrift", Value::Num(x));
    }
    append_unknown(&mut o, &b.unknown);
    o
}

pub fn brush_from_value(v: &Value, ctx: &str) -> Result<Brush, String> {
    let mut b = Brush::default();
    if let Some(x) = v.get("name") {
        b.name = Some(x.as_str().ok_or_else(|| format!("{}: brush.name must be a string", ctx))?.to_string());
    }
    for (key, slot) in [
        ("align", 0usize),
        ("jitter", 1),
        ("spacing", 2),
        ("angleDrift", 3),
        ("sizeAmp", 4),
        ("sizeScale", 5),
        ("randomInterval", 6),
        ("hueDrift", 7),
        ("satDrift", 8),
        ("valDrift", 9),
    ] {
        if let Some(x) = v.get(key) {
            let n = x.as_f64().ok_or_else(|| format!("{}: brush.{} must be a number", ctx, key))?;
            match slot {
                0 => b.align = Some(n),
                1 => b.jitter = Some(n),
                2 => b.spacing = Some(n),
                3 => b.angle_drift = Some(n),
                4 => b.size_amp = Some(n),
                5 => b.size_scale = Some(n),
                6 => b.random_interval = Some(n),
                7 => b.hue_drift = Some(n),
                8 => b.sat_drift = Some(n),
                _ => b.val_drift = Some(n),
            }
        }
    }
    if let Some(x) = v.get("randomize") {
        b.randomize = Some(x.as_bool().ok_or_else(|| format!("{}: brush.randomize must be a bool", ctx))?);
    }
    if let Some(x) = v.get("mergedAlpha") {
        b.merged_alpha = Some(x.as_bool().ok_or_else(|| format!("{}: brush.mergedAlpha must be a bool", ctx))?);
    }
    if let Some(x) = v.get("randOrder") {
        b.rand_order = Some(x.as_bool().ok_or_else(|| format!("{}: brush.randOrder must be a bool", ctx))?);
    }
    if let Some(x) = v.get("tint") {
        b.tint = Some(parse_color(x, &format!("{}: brush.tint", ctx))?);
    }
    b.unknown = unknown_of(v, BRUSH_KEYS);
    Ok(b)
}

fn style_to_value(s: &Style) -> Value {
    let mut o = Value::obj();
    o.set("id", Value::Str(s.id.clone()));
    if let Some(x) = s.width {
        o.set("width", Value::Num(x));
    }
    if let Some(x) = s.color {
        o.set("color", color_value(x));
    }
    if let Some(x) = s.fill_color {
        o.set("fillColor", color_value(x));
    }
    if let Some(x) = s.stroke_on {
        o.set("strokeOn", Value::Bool(x));
    }
    if let Some(x) = s.fill_on {
        o.set("fillOn", Value::Bool(x));
    }
    if let Some(b) = &s.brush {
        o.set("brush", brush_to_value(b));
    }
    append_unknown(&mut o, &s.unknown);
    o
}

pub fn style_from_value(v: &Value) -> Result<Style, String> {
    let id = v.get("id").and_then(|x| x.as_str()).ok_or("style.id must be a string")?.to_string();
    let mut s = Style::new(&id);
    if let Some(x) = v.get("width").and_then(|x| x.as_f64()) {
        s.width = Some(x);
    }
    if let Some(x) = v.get("color") {
        s.color = Some(parse_color(x, &format!("style '{}'", id))?);
    }
    if let Some(x) = v.get("fillColor") {
        s.fill_color = Some(parse_color(x, &format!("style '{}'", id))?);
    }
    if let Some(x) = v.get("strokeOn").and_then(|x| x.as_bool()) {
        s.stroke_on = Some(x);
    }
    if let Some(x) = v.get("fillOn").and_then(|x| x.as_bool()) {
        s.fill_on = Some(x);
    }
    if let Some(bv) = v.get("brush") {
        s.brush = Some(brush_from_value(bv, &format!("style '{}'", id))?);
    }
    s.unknown = unknown_of(v, &["id", "width", "color", "fillColor", "strokeOn", "fillOn", "brush"]);
    Ok(s)
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
        RelationKind::Gradient => {
            o.set("bar", Value::Str(r.gradient_bar.clone()));
            o.set("target", Value::Str(r.gradient_target.clone()));
            o.set("role", Value::Str(r.gradient_role.name().into()));
        }
        RelationKind::Group => {
            o.set("box", Value::Str(r.group_box.clone()));
            o.set("members", Value::Arr(r.group_members.iter().map(|m| Value::Str(m.clone())).collect()));
            o.set("visible", Value::Bool(r.group_visible));
        }
        RelationKind::Sim => {
            o.set("line", Value::Str(r.sim_line.clone()));
            o.set(
                "anchors",
                Value::Arr(
                    r.sim_anchors
                        .iter()
                        .map(|(node, dl, dn)| {
                            let mut a = Value::obj();
                            a.set("node", Value::Int(*node as i64));
                            let mut d = Value::obj();
                            d.set("line", Value::Str(dl.clone()));
                            d.set("node", Value::Int(*dn as i64));
                            a.set("driver", d);
                            a
                        })
                        .collect(),
                ),
            );
            o.set("gravity", vec_value(r.sim_gravity));
            o.set("damping", Value::Num(r.sim_damping));
            o.set("iterations", Value::Int(r.sim_iterations as i64));
            if !r.sim_collide_with.is_empty() {
                o.set("collideWith", Value::Arr(r.sim_collide_with.iter().map(|c| Value::Str(c.clone())).collect()));
                o.set("radius", Value::Num(r.sim_radius));
                o.set("selfCollide", Value::Bool(r.sim_self_collide));
            }
        }
        RelationKind::Particles => {
            o.set("emitter", Value::Str(r.part_emitter.clone()));
            o.set("node", Value::Int(r.part_node as i64));
            o.set("rate", Value::Num(r.part_rate));
            o.set("speed", Value::Num(r.part_speed));
            o.set("dir", vec_value(r.part_dir));
            o.set("spread", Value::Num(r.part_spread_deg));
            o.set("life", Value::Num(r.part_life_s));
            o.set("gravity", vec_value(r.part_gravity));
            o.set("damping", Value::Num(r.part_damping));
            o.set("size", Value::Num(r.part_size));
            o.set("color", color_value(r.part_color));
            o.set("len", Value::Num(r.part_len));
            o.set("seed", Value::Int(r.part_seed));
            if !r.part_collide_with.is_empty() {
                o.set("collideWith", Value::Arr(r.part_collide_with.iter().map(|c| Value::Str(c.clone())).collect()));
                o.set("bounce", Value::Num(r.part_bounce));
                o.set("radius", Value::Num(r.part_radius));
            }
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

/// Version migrations (file-format.md rule 7: FORMAT_VERSION rises one step
/// at a time; old versions stay readable). `MIGRATIONS[i]` upgrades a
/// version-(i+1) document's JSON value to version i+2 — bump FORMAT_VERSION
/// and push exactly one function here per release. v1 is the first version,
/// so the table is empty today; the loop is wired and exercised by the gate.
type Migration = fn(&mut Value) -> Result<(), String>;
pub const MIGRATIONS: &[Migration] = &[];

fn apply_migrations(v: &mut Value, ver: i64) -> Result<(), String> {
    let mut ver = ver;
    while ver < FORMAT_VERSION {
        let idx = (ver - 1).max(0) as usize;
        let m = *MIGRATIONS.get(idx).ok_or_else(|| {
            format!("document version {} has no migration path to {} (reader bug)", ver, FORMAT_VERSION)
        })?;
        m(v)?;
        ver += 1;
    }
    Ok(())
}

pub fn doc_from_value(v: &Value) -> Result<Document, String> {
    let obj = v.as_obj().ok_or("document root must be a JSON object")?;
    let mut doc = Document::default();

    // Version gate + migration: older documents upgrade in place through the
    // MIGRATIONS chain (B7); newer versions are refused (never guessed).
    let ver = v.get("version").and_then(|x| x.as_i64());
    let mut migrated;
    let v = match ver {
        Some(ver) if ver < FORMAT_VERSION => {
            migrated = v.clone();
            apply_migrations(&mut migrated, ver)?;
            &migrated
        }
        _ => v,
    };
    if let Some(ver) = ver {
        if ver > FORMAT_VERSION {
            return Err(format!("document version {} is newer than this reader ({}); upgrade the engine", ver, FORMAT_VERSION));
        }
        if ver < 1 {
            return Err(format!("document version {} is invalid", ver));
        }
    }
    if let Some(minr) = v.get("minReader") {
        let minr = minr.as_i64().ok_or("minReader must be an integer")?;
        if minr > FORMAT_VERSION {
            return Err(format!("document requires reader >= {}, this reader is {}", minr, FORMAT_VERSION));
        }
    }

    let fmt = v.get("format").and_then(|x| x.as_str()).unwrap_or("");
    if fmt != FORMAT_NAME {
        return Err(format!("not a mo2d document (format = '{}')", fmt));
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
    if let Some(styles) = v.get("styles").and_then(|x| x.as_arr()) {
        for sv in styles {
            doc.styles.push(style_from_value(sv)?);
        }
    }

    let _ = obj;
    doc.unknown = unknown_of(v, &["format", "version", "minReader", "generator", "root", "lines", "relations", "styles"]);
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
    if let Some(bv) = v.get("brush") {
        l.brush = Some(brush_from_value(bv, &ctx)?);
    }
    if let Some(t) = v.get("texture").and_then(|x| x.as_str()) {
        l.texture = Some(t.to_string());
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
            if let Some(pr) = nv.get("pr") {
                n.pressure = Some(pr.as_f64().ok_or_else(|| format!("{}: 'pr' must be a number", nctx))?);
            }
            if let Some(uv) = nv.get("uv") {
                let arr = uv.as_arr().ok_or_else(|| format!("{}: 'uv' must be [u,v]", nctx))?;
                if arr.len() != 2 {
                    return Err(format!("{}: 'uv' must have 2 components", nctx));
                }
                let u = arr[0].as_f64().ok_or_else(|| format!("{}: uv[0] must be a number", nctx))?;
                let v = arr[1].as_f64().ok_or_else(|| format!("{}: uv[1] must be a number", nctx))?;
                n.uv = Some([u, v]);
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
            n.unknown = unknown_of(nv, &["p", "w", "pr", "uv", "c", "hi", "ho", "v"]);
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
    l.unknown = unknown_of(v, &["id", "tags", "closed", "width", "color", "fillColor", "strokeOn", "fillOn", "brush", "texture", "nodes", "segs"]);
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
                Some("color") => TrackAttr::Color,
                other => return Err(format!("relation '{}': target.attr must be 'position'|'width'|'color', got {:?}", id, other)),
            };
            Relation::new_track(&id, tl, line, node, attr)
        }
        "gradient" => {
            let bar = v.get("bar").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': gradient needs bar", id))?;
            let target = v.get("target").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': gradient needs target", id))?;
            let role = match v.get("role").and_then(|x| x.as_str()) {
                Some("fill") | None => GradientRole::Fill,
                Some("stroke") => GradientRole::Stroke,
                other => return Err(format!("relation '{}': role must be 'fill'|'stroke', got {:?}", id, other)),
            };
            Relation::new_gradient(&id, bar, target, role)
        }
        "group" => {
            let box_id = v.get("box").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': group needs box", id))?;
            let mut members = Vec::new();
            if let Some(ms) = v.get("members").and_then(|x| x.as_arr()) {
                for m in ms {
                    members.push(m.as_str().ok_or_else(|| format!("relation '{}': members must be strings", id))?.to_string());
                }
            }
            let visible = v.get("visible").and_then(|x| x.as_bool()).unwrap_or(true);
            Relation::new_group(&id, box_id, &members.iter().map(|s| s.as_str()).collect::<Vec<_>>(), visible)
        }
        "sim" => {
            let line = v.get("line").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': sim needs line", id))?;
            let mut r = Relation::new_sim(&id, line);
            if let Some(av) = v.get("anchors").and_then(|x| x.as_arr()) {
                for a in av {
                    let node = a.get("node").and_then(|x| x.as_i64()).ok_or_else(|| format!("relation '{}': anchors[].node must be an integer", id))?;
                    let d = a.get("driver").ok_or_else(|| format!("relation '{}': anchors[].driver missing", id))?;
                    let dl = d.get("line").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': driver.line missing", id))?;
                    let dn = d.get("node").and_then(|x| x.as_i64()).ok_or_else(|| format!("relation '{}': driver.node missing", id))?;
                    r.sim_anchors.push((node as usize, dl.to_string(), dn as usize));
                }
            }
            if let Some(gv) = v.get("gravity") {
                r.sim_gravity = parse_vec3(gv, &format!("relation '{}': gravity", id))?;
            }
            if let Some(d) = v.get("damping").and_then(|x| x.as_f64()) {
                r.sim_damping = d;
            }
            if let Some(i) = v.get("iterations").and_then(|x| x.as_i64()) {
                r.sim_iterations = i.max(0) as usize;
            }
            if let Some(cv) = v.get("collideWith").and_then(|x| x.as_arr()) {
                for c in cv {
                    r.sim_collide_with.push(c.as_str().ok_or_else(|| format!("relation '{}': collideWith must be strings", id))?.to_string());
                }
            }
            if let Some(x) = v.get("radius").and_then(|x| x.as_f64()) {
                r.sim_radius = x;
            }
            if let Some(x) = v.get("selfCollide").and_then(|x| x.as_bool()) {
                r.sim_self_collide = x;
            }
            r
        }
        "particles" => {
            let emitter = v.get("emitter").and_then(|x| x.as_str()).ok_or_else(|| format!("relation '{}': particles needs emitter", id))?;
            let mut r = Relation::new_particles(&id, emitter, v.get("node").and_then(|x| x.as_i64()).unwrap_or(0) as usize);
            if let Some(x) = v.get("rate").and_then(|x| x.as_f64()) {
                r.part_rate = x;
            }
            if let Some(x) = v.get("speed").and_then(|x| x.as_f64()) {
                r.part_speed = x;
            }
            if let Some(x) = v.get("dir") {
                r.part_dir = parse_vec3(x, &format!("relation '{}': dir", id))?;
            }
            if let Some(x) = v.get("spread").and_then(|x| x.as_f64()) {
                r.part_spread_deg = x;
            }
            if let Some(x) = v.get("life").and_then(|x| x.as_f64()) {
                r.part_life_s = x;
            }
            if let Some(x) = v.get("gravity") {
                r.part_gravity = parse_vec3(x, &format!("relation '{}': gravity", id))?;
            }
            if let Some(x) = v.get("damping").and_then(|x| x.as_f64()) {
                r.part_damping = x;
            }
            if let Some(x) = v.get("size").and_then(|x| x.as_f64()) {
                r.part_size = x;
            }
            if let Some(x) = v.get("color") {
                r.part_color = parse_color(x, &format!("relation '{}': color", id))?;
            }
            if let Some(x) = v.get("len").and_then(|x| x.as_f64()) {
                r.part_len = x;
            }
            if let Some(x) = v.get("seed").and_then(|x| x.as_i64()) {
                r.part_seed = x;
            }
            if let Some(cv) = v.get("collideWith").and_then(|x| x.as_arr()) {
                for c in cv {
                    r.part_collide_with.push(c.as_str().ok_or_else(|| format!("relation '{}': collideWith must be strings", id))?.to_string());
                }
            }
            if let Some(x) = v.get("bounce").and_then(|x| x.as_f64()) {
                r.part_bounce = x;
            }
            if let Some(x) = v.get("radius").and_then(|x| x.as_f64()) {
                r.part_radius = x;
            }
            r
        }
        other => return Err(format!("relation '{}': unknown type '{}'", id, other)),
    };
    if let Some(f) = v.get("falloff").and_then(|x| x.as_f64()) {
        r.bind_falloff = f;
    }
    if let Some(b) = v.get("stretch").and_then(|x| x.as_bool()) {
        r.bind_stretch = b;
    }
    // Per-type consumed keys: foreign keys of ANY name pass through untouched
    // (file-format.md §2.5), so the filter must reflect what this relation
    // type actually consumed — never a global superset.
    let consumed: &[&str] = match r.kind {
        RelationKind::Bind => &["id", "type", "target", "skeleton", "falloff", "stretch"],
        RelationKind::Share => &["id", "type", "driver", "driven"],
        RelationKind::Track => &["id", "type", "trackLine", "target"],
        RelationKind::Gradient => &["id", "type", "bar", "target", "role"],
        RelationKind::Group => &["id", "type", "box", "members", "visible"],
        RelationKind::Sim => &["id", "type", "line", "anchors", "gravity", "damping", "iterations", "collideWith", "radius", "selfCollide"],
        RelationKind::Particles => &["id", "type", "emitter", "node", "rate", "speed", "dir", "spread", "life", "gravity", "damping", "size", "color", "len", "seed", "collideWith", "bounce", "radius"],
    };
    r.unknown = unknown_of(v, consumed);
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
