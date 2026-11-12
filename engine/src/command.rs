//! Command protocol v1: the single entry point into the engine
//! (docs/ai-interaction.md §4 — every operation is a command).
//!
//! Stream envelope (D2: protocol version field; old streams replay forever):
//! `{"v": 1, "cmds": [{"op": "line.create", ...}, ...]}`
//!
//! Session-class commands (`export.*`, `screenshot`, `play`) are handled by the
//! host (CLI/MCP/stage); this module owns **document** commands, which are
//! recorded, replayable and undoable via generated inverses.

use crate::format;
use crate::json::{self, Value};
use crate::math::Vec3;
use crate::model::*;

pub const PROTOCOL_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub op: String,
    /// Argument fields in original order (for faithful stream echo).
    pub args: Vec<(String, Value)>,
}

impl Command {
    pub fn arg(&self, key: &str) -> Option<&Value> {
        self.args.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    pub fn to_value(&self) -> Value {
        let mut o = Value::obj();
        o.set("op", Value::Str(self.op.clone()));
        for (k, v) in &self.args {
            o.set(k, v.clone());
        }
        o
    }
}

// ---------------------------------------------------------------------------
// Stream parse
// ---------------------------------------------------------------------------

pub fn parse_stream(text: &str) -> Result<Vec<Command>, String> {
    let v = json::parse(text)?;
    let ver = v.get("v").and_then(|x| x.as_i64()).ok_or("command stream must carry an integer protocol version field 'v'")?;
    if ver != PROTOCOL_VERSION {
        return Err(format!("protocol version {} not supported (this engine speaks v{}); replaying old streams requires the engine that wrote them or a newer protocol addition", ver, PROTOCOL_VERSION));
    }
    let cmds = v.get("cmds").and_then(|x| x.as_arr()).ok_or("command stream must have a 'cmds' array")?;
    let mut out = Vec::with_capacity(cmds.len());
    for (i, cv) in cmds.iter().enumerate() {
        let op = cv.get("op").and_then(|x| x.as_str()).ok_or_else(|| format!("cmds[{}]: missing 'op'", i))?.to_string();
        let args = match cv {
            Value::Obj(fields) => fields.iter().filter(|(k, _)| k != "op").cloned().collect(),
            _ => return Err(format!("cmds[{}] must be an object", i)),
        };
        out.push(Command { op, args });
    }
    Ok(out)
}

pub fn stream_to_string(cmds: &[Command]) -> Result<String, String> {
    let mut o = Value::obj();
    o.set("v", Value::Int(PROTOCOL_VERSION));
    o.set("cmds", Value::Arr(cmds.iter().map(|c| c.to_value()).collect()));
    let mut s = json::to_string(&o)?;
    s.push('\n');
    Ok(s)
}

// ---------------------------------------------------------------------------
// Registry (for `list` / MCP introspection)
// ---------------------------------------------------------------------------

pub struct CmdSpec {
    pub name: &'static str,
    pub args: &'static str,
    pub doc: &'static str,
}

pub fn registry() -> Vec<CmdSpec> {
    vec![
        CmdSpec { name: "doc.setRoot", args: "{width?, height?, fps?, ticksPerSecond?}", doc: "Set root parameters (canvas size, fps is display-only)." },
        CmdSpec { name: "line.create", args: "{id?, tags?, closed?, width?, color?, fillColor?, strokeOn?, fillOn?, brush?, texture?, nodes: [{p, w?, pr?, uv?, c?, hi?, ho?, v?}], segs?: [{type?, visible?, interp?}]}", doc: "Create a line. 'p' = [x,y,z] geometric or [tick] for track lines; 'pr' = node pressure (B2); 'uv' = node texture anchor (B5); 'brush' = line-level brush object (B2); 'texture' = external image path (B5 textured fill). Auto id l<N> when omitted." },
        CmdSpec { name: "line.setAttrs", args: "{id, closed?, width?, color?, fillColor?, strokeOn?, fillOn?, brush?, texture?}", doc: "Merge line-level attributes. brush: object sets/replaces the line brush; null clears it. texture: path or null." },
        CmdSpec { name: "line.delete", args: "{id}", doc: "Delete a line (relations referencing it are deleted too)." },
        CmdSpec { name: "node.set", args: "{line, index, p?, w?, pr?, c?, hi?, ho?, atTick?}", doc: "Set node fields ('pr' = pressure, B2). With atTick and an existing track on this node+attr, upserts a keyframe instead (auto-key)." },
        CmdSpec { name: "node.append", args: "{line, p, w?, pr?, c?, hi?, ho?, v?}", doc: "Append a node to a line." },
        CmdSpec { name: "node.delete", args: "{line, index}", doc: "Delete a node (and its keyframe if it lives on a track line)." },
        CmdSpec { name: "seg.set", args: "{line, index, type?, visible?, interp?}", doc: "Merge segment attributes (straight/curve, visibility, interp for track segments)." },
        CmdSpec { name: "relation.create", args: "{id?, type: bind|share|track|gradient|group|sim|particles, ...}", doc: "bind: {target, skeleton, falloff?, stretch?}; share: {driver: {line,node}, driven: {line,node}}; track: {trackLine, target: {line, node?, attr: position|width|color}}; gradient (B3): {bar, target, role?: fill|stroke}; group (B4): {box, members, visible?} — transform-box line; sim (B6): {line, anchors: [{node, driver: {line,node}}], gravity?, damping?, iterations?, collideWith?, radius?, selfCollide?} — deterministic trailing rope with optional collisions; particles (B6): {emitter, node?, rate?, speed?, dir?, spread?, life?, gravity?, damping?, size?, color?, len?, seed?, collideWith?, bounce?, radius?}. Auto id r<N>." },
        CmdSpec { name: "relation.delete", args: "{id}", doc: "Delete a relation." },
        CmdSpec { name: "track.create", args: "{line, node, attr?: position|width, tick?, value?}", doc: "Create a track line (auto id trk_<line>_<node>_<attr>) + its track relation + the first keyframe (defaults: tick 0, value = the node's current value). This is the gesture command for node→timeline connect; AI uses it too, so both channels produce identical documents." },
        CmdSpec { name: "style.create", args: "{id?, width?, color?, fillColor?, strokeOn?, fillOn?, brush?}", doc: "Define a named line-attribute template (B2 styles). Auto id st<N>." },
        CmdSpec { name: "style.delete", args: "{id}", doc: "Delete a style template." },
        CmdSpec { name: "style.apply", args: "{id, line}", doc: "Copy the style's defined attributes onto a line (undefined fields leave the line untouched). Switching a line between brush strokes = applying different styles." },
    ]
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Effect {
    /// Command that undoes this one (already carries pre-state).
    pub inverse: Command,
    /// Human-readable description (for logs).
    pub summary: String,
}

/// Generate the next auto id: `prefix` + (max existing numeric suffix + 1).
pub fn next_auto_id(doc: &Document, prefix: &str) -> String {
    let mut max = 0;
    let probe = |id: &str, max: &mut usize| {
        if let Some(rest) = id.strip_prefix(prefix) {
            if let Ok(n) = rest.parse::<usize>() {
                if n > *max {
                    *max = n;
                }
            }
        }
    };
    for l in &doc.lines {
        probe(&l.id, &mut max);
    }
    for r in &doc.relations {
        probe(&r.id, &mut max);
    }
    format!("{}{}", prefix, max + 1)
}

fn need_str<'a>(cmd: &'a Command, key: &str) -> Result<&'a str, String> {
    cmd.arg(key).and_then(|v| v.as_str()).ok_or_else(|| format!("{}: '{}' must be a string", cmd.op, key))
}

fn parse_node_spec(cmd: &Command, v: &Value, ctx: &str) -> Result<Node, String> {
    let pv = v.get("p").ok_or_else(|| format!("{}: node spec needs 'p'", ctx))?;
    let pos = match pv.as_arr() {
        Some([t]) => {
            let t = t.as_i64().ok_or_else(|| format!("{}: track tick must be an integer", ctx))?;
            if t.abs() > TICK_LIMIT {
                return Err(format!("{}: tick {} beyond ±{} limit", ctx, t, TICK_LIMIT));
            }
            NodePos::Tick(t)
        }
        Some([x, y, z]) => NodePos::Geom(Vec3::new(
            x.as_f64().ok_or_else(|| format!("{}: p[0] must be a number", ctx))?,
            y.as_f64().ok_or_else(|| format!("{}: p[1] must be a number", ctx))?,
            z.as_f64().ok_or_else(|| format!("{}: p[2] must be a number", ctx))?,
        )),
        _ => return Err(format!("{}: 'p' must be [x,y,z] or [tick]", ctx)),
    };
    let mut n = match pos {
        NodePos::Geom(g) => Node::geom(g),
        NodePos::Tick(t) => {
            let val = v.get("v").cloned().ok_or_else(|| format!("{}: track node needs 'v'", ctx))?;
            Node::track(t, val)
        }
    };
    if let Some(w) = v.get("w") {
        n.width = Some(w.as_f64().ok_or_else(|| format!("{}: 'w' must be a number", ctx))?);
    }
    if let Some(pr) = v.get("pr") {
        n.pressure = Some(pr.as_f64().ok_or_else(|| format!("{}: 'pr' must be a number", ctx))?);
    }
    if let Some(uv) = v.get("uv") {
        let arr = uv.as_arr().ok_or_else(|| format!("{}: 'uv' must be [u,v]", ctx))?;
        if arr.len() != 2 {
            return Err(format!("{}: 'uv' must have 2 components", ctx));
        }
        n.uv = Some([
            arr[0].as_f64().ok_or_else(|| format!("{}: uv[0] must be a number", ctx))?,
            arr[1].as_f64().ok_or_else(|| format!("{}: uv[1] must be a number", ctx))?,
        ]);
    }
    if let Some(c) = v.get("c") {
        n.color = Some(parse_json_color(c, ctx)?);
    }
    if let Some(h) = v.get("hi") {
        n.handle_in = Some(parse_json_vec3(h, ctx)?);
    }
    if let Some(h) = v.get("ho") {
        n.handle_out = Some(parse_json_vec3(h, ctx)?);
    }
    let _ = cmd;
    Ok(n)
}

fn parse_brush_arg(v: &Value, ctx: &str) -> Result<Brush, String> {
    if matches!(v, Value::Null) {
        return Ok(Brush::default());
    }
    format::brush_from_value(v, ctx)
}

fn parse_json_color(v: &Value, ctx: &str) -> Result<[u8; 4], String> {
    let arr = v.as_arr().ok_or_else(|| format!("{}: color must be [r,g,b,a]", ctx))?;
    if arr.len() != 4 {
        return Err(format!("{}: color must have 4 components", ctx));
    }
    let mut c = [0u8; 4];
    for (i, x) in arr.iter().enumerate() {
        let n = x.as_i64().ok_or_else(|| format!("{}: color[{}] must be integer 0-255", ctx, i))?;
        if !(0..=255).contains(&n) {
            return Err(format!("{}: color[{}] out of range: {}", ctx, i, n));
        }
        c[i] = n as u8;
    }
    Ok(c)
}

fn parse_json_vec3(v: &Value, ctx: &str) -> Result<Vec3, String> {
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

fn parse_pos_arg(v: &Value, ctx: &str) -> Result<NodePos, String> {
    match v.as_arr() {
        Some([t]) => {
            let t = t.as_i64().ok_or_else(|| format!("{}: tick must be an integer", ctx))?;
            Ok(NodePos::Tick(t))
        }
        Some([x, y, z]) => Ok(NodePos::Geom(Vec3::new(
            x.as_f64().ok_or_else(|| format!("{}: p[0] must be a number", ctx))?,
            y.as_f64().ok_or_else(|| format!("{}: p[1] must be a number", ctx))?,
            z.as_f64().ok_or_else(|| format!("{}: p[2] must be a number", ctx))?,
        ))),
        _ => Err(format!("{}: 'p' must be [x,y,z] or [tick]", ctx)),
    }
}

fn value_of_pos(p: &NodePos) -> Value {
    match p {
        NodePos::Geom(g) => Value::Arr(vec![Value::Num(g.x), Value::Num(g.y), Value::Num(g.z)]),
        NodePos::Tick(t) => Value::Arr(vec![Value::Int(*t)]),
    }
}

/// Value triple for a track keyframe (position) or scalar (width).
/// Color keyframes are built at the call site (from the node's color).
fn track_value_for_attr(attr: TrackAttr, p: Vec3) -> Value {
    match attr {
        TrackAttr::Position => Value::Arr(vec![Value::Num(p.x), Value::Num(p.y), Value::Num(p.z)]),
        TrackAttr::Width | TrackAttr::Color => Value::Num(p.x),
    }
}


/// Deterministic auto-key track id for a target (used by both the hand path and
/// the pseudo-AI path so the two documents come out byte-identical).
pub fn auto_track_id(target_line: &str, node: usize, attr: &TrackAttr) -> String {
    format!("trk_{}_{}_{}", target_line, node, attr.name())
}

/// Find the track relation driving (line, node, attr), if any.
fn track_relation_for(doc: &Document, line: &str, node: usize, attr: &TrackAttr) -> Option<Relation> {
    doc.relations
        .iter()
        .find(|r| {
            r.kind == RelationKind::Track
                && r.track_target_line == line
                && r.track_target_node == Some(node)
                && r.track_attr == *attr
        })
        .cloned()
}

/// Upsert a keyframe at `tick` with value `val` on the track line.
/// Creates the track line + relation when `create` is set (auto-key path);
/// the relation must already reference `track_line` otherwise.
fn upsert_keyframe(doc: &mut Document, rel: &Relation, tick: i64, val: Value, create_rel: bool) -> Result<(), String> {
    let tl_id = rel.track_line.clone();
    if doc.line(&tl_id).is_err() {
        if !create_rel {
            return Err(format!("track line '{}' not found", tl_id));
        }
        let mut tl = Line::new(&tl_id);
        tl.tags = vec!["track".to_string()];
        doc.lines.push(tl);
    }
    let tl = doc.line_mut(&tl_id)?;
    // Keyframes must stay tick-ascending (file-format.md §2.1); upsert keeps order.
    if let Some(idx) = tl.nodes.iter().position(|n| matches!(n.pos, NodePos::Tick(t) if t == tick)) {
        tl.nodes[idx].value = Some(val);
    } else {
        let idx = tl
            .nodes
            .iter()
            .position(|n| matches!(n.pos, NodePos::Tick(t) if t > tick))
            .unwrap_or(tl.nodes.len());
        tl.nodes.insert(idx, Node::track(tick, val));
    }
    tl.sync_segs();
    // Keep interp defaults sensible for fresh Bezier-less segments (Linear).
    Ok(())
}

/// Apply one command to the document. Returns the effect carrying the inverse.
pub fn apply(doc: &mut Document, cmd: &Command) -> Result<Effect, String> {
    match cmd.op.as_str() {
        "doc.setRoot" => {
            let mut inv = Value::obj();
            inv.set("op", Value::Str("doc.setRoot".into()));
            let mut changed = false;
            let set_field = |doc: &mut Document, key: &str, val: &Value, inv: &mut Value, changed: &mut bool| -> Result<(), String> {
                match key {
                    "width" => {
                        inv.set("width", Value::Int(doc.root.width as i64));
                        doc.root.width = val.as_i64().ok_or("width must be an integer")? as u32;
                        *changed = true;
                    }
                    "height" => {
                        inv.set("height", Value::Int(doc.root.height as i64));
                        doc.root.height = val.as_i64().ok_or("height must be an integer")? as u32;
                        *changed = true;
                    }
                    "fps" => {
                        inv.set("fps", Value::Num(doc.root.fps));
                        doc.root.fps = val.as_f64().ok_or("fps must be a number")?;
                        *changed = true;
                    }
                    "ticksPerSecond" => {
                        inv.set("ticksPerSecond", Value::Int(doc.root.ticks_per_second));
                        doc.root.ticks_per_second = val.as_i64().ok_or("ticksPerSecond must be an integer")?;
                        *changed = true;
                    }
                    _ => return Err(format!("doc.setRoot: unknown field '{}'", key)),
                }
                Ok(())
            };
            for (k, v) in &cmd.args {
                set_field(doc, k, v, &mut inv, &mut changed)?;
            }
            if !changed {
                return Err("doc.setRoot: nothing to set".into());
            }
            Ok(Effect { inverse: value_to_command(&inv)?, summary: "set root".into() })
        }

        "line.create" => {
            let id = match cmd.arg("id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => next_auto_id(doc, "l"),
            };
            if doc.line(&id).is_ok() {
                return Err(format!("line '{}' already exists", id));
            }
            let mut l = Line::new(&id);
            if let Some(tags) = cmd.arg("tags").and_then(|v| v.as_arr()) {
                for t in tags {
                    l.tags.push(t.as_str().ok_or("tags must be strings")?.to_string());
                }
            }
            if let Some(b) = cmd.arg("closed").and_then(|v| v.as_bool()) {
                l.closed = b;
            }
            if let Some(w) = cmd.arg("width").and_then(|v| v.as_f64()) {
                l.width = w;
            }
            if let Some(c) = cmd.arg("color") {
                l.color = parse_json_color(c, "line.create color")?;
            }
            if let Some(c) = cmd.arg("fillColor") {
                l.fill_color = parse_json_color(c, "line.create fillColor")?;
            }
            if let Some(bv) = cmd.arg("brush") {
                l.brush = Some(parse_brush_arg(bv, "line.create brush")?);
            }
            if let Some(t) = cmd.arg("texture").and_then(|v| v.as_str()) {
                l.texture = Some(t.to_string());
            }
            if let Some(b) = cmd.arg("strokeOn").and_then(|v| v.as_bool()) {
                l.stroke_on = b;
            }
            if let Some(b) = cmd.arg("fillOn").and_then(|v| v.as_bool()) {
                l.fill_on = b;
            }
            let nodes = cmd.arg("nodes").and_then(|v| v.as_arr()).ok_or("line.create: 'nodes' array required")?;
            for (i, nv) in nodes.iter().enumerate() {
                let ctx = format!("line.create {} node {}", id, i);
                l.nodes.push(parse_node_spec(cmd, nv, &ctx)?);
            }
            if let Some(segs) = cmd.arg("segs").and_then(|v| v.as_arr()) {
                for (i, sv) in segs.iter().enumerate() {
                    let ctx = format!("line.create {} seg {}", id, i);
                    let mut s = SegAttrs::default();
                    if let Some(t) = sv.get("type").and_then(|x| x.as_str()) {
                        s.seg_type = match t {
                            "straight" => SegType::Straight,
                            "curve" => SegType::Curve,
                            o => return Err(format!("{}: unknown segment type '{}'", ctx, o)),
                        };
                    }
                    if let Some(b) = sv.get("visible").and_then(|x| x.as_bool()) {
                        s.visible = b;
                    }
                    if let Some(iv) = sv.get("interp") {
                        let im = iv.get("im").and_then(|x| x.as_i64()).ok_or_else(|| format!("{}: interp.im required", ctx))?;
                        let mut b = None;
                        if let Some(bv) = iv.get("b").and_then(|x| x.as_arr()) {
                            let mut handles = Vec::new();
                            for hv in bv {
                                let mut h = BezierHandle::default_handle();
                                if let Some(x) = hv.get("ao").and_then(|x| x.as_f64()) { h.ao = x; }
                                if let Some(x) = hv.get("ai").and_then(|x| x.as_f64()) { h.ai = x; }
                                if let Some(x) = hv.get("po").and_then(|x| x.as_f64()) { h.po = x; }
                                if let Some(x) = hv.get("pi").and_then(|x| x.as_f64()) { h.pi = x; }
                                handles.push(h);
                            }
                            b = Some(handles);
                        }
                        s.interp = Some(Interp { im, b });
                    }
                    l.segs.push(s);
                }
            }
            l.sync_segs();
            let min_nodes = if l.is_track() { 1 } else { 2 };
            if l.nodes.len() < min_nodes {
                return Err(format!("line.create {}: a line needs at least {} nodes, got {}", id, min_nodes, l.nodes.len()));
            }
            doc.lines.push(l);
            let inv = obj_command("line.delete", &[("id", Value::Str(id.clone()))]);
            Ok(Effect { inverse: inv, summary: format!("create line {}", id) })
        }

        "line.setAttrs" => {
            let id = need_str(cmd, "id")?.to_string();
            let l = doc.line_mut(&id)?;
            let mut inv = Value::obj();
            inv.set("op", Value::Str("line.setAttrs".into()));
            inv.set("id", Value::Str(id.clone()));
            for (k, v) in &cmd.args {
                if k == "id" {
                    continue;
                }
                match k.as_str() {
                    "closed" => {
                        inv.set("closed", Value::Bool(l.closed));
                        l.closed = v.as_bool().ok_or("closed must be a bool")?;
                        l.sync_segs();
                    }
                    "width" => {
                        inv.set("width", Value::Num(l.width));
                        l.width = v.as_f64().ok_or("width must be a number")?;
                    }
                    "color" => {
                        inv.set("color", color_arr(l.color));
                        l.color = parse_json_color(v, "line.setAttrs color")?;
                    }
                    "fillColor" => {
                        inv.set("fillColor", color_arr(l.fill_color));
                        l.fill_color = parse_json_color(v, "line.setAttrs fillColor")?;
                    }
                    "strokeOn" => {
                        inv.set("strokeOn", Value::Bool(l.stroke_on));
                        l.stroke_on = v.as_bool().ok_or("strokeOn must be a bool")?;
                    }
                    "fillOn" => {
                        inv.set("fillOn", Value::Bool(l.fill_on));
                        l.fill_on = v.as_bool().ok_or("fillOn must be a bool")?;
                    }
                    "brush" => {
                        inv.set("brush", l.brush.as_ref().map(|b| format::brush_to_value(b)).unwrap_or(Value::Null));
                        if matches!(v, Value::Null) {
                            l.brush = None;
                        } else {
                            l.brush = Some(parse_brush_arg(v, "line.setAttrs brush")?);
                        }
                    }
                    "texture" => {
                        inv.set("texture", l.texture.clone().map(Value::Str).unwrap_or(Value::Null));
                        if matches!(v, Value::Null) {
                            l.texture = None;
                        } else {
                            l.texture = Some(v.as_str().ok_or("texture must be a string")?.to_string());
                        }
                    }
                    "tags" => {
                        inv.set("tags", Value::Arr(l.tags.iter().map(|t| Value::Str(t.clone())).collect()));
                        let mut tags = Vec::new();
                        for t in v.as_arr().ok_or("tags must be an array")? {
                            tags.push(t.as_str().ok_or("tags must be strings")?.to_string());
                        }
                        l.tags = tags;
                    }
                    other => return Err(format!("line.setAttrs: unknown field '{}'", other)),
                }
            }
            Ok(Effect { inverse: value_to_command(&inv)?, summary: format!("set attrs {}", id) })
        }

        "line.delete" => {
            let id = need_str(cmd, "id")?.to_string();
            let idx = doc.lines.iter().position(|l| l.id == id).ok_or_else(|| format!("line '{}' not found", id))?;
            let removed = doc.lines.remove(idx);
            // Relations referencing the line go with it (deterministic order).
            let mut i = 0;
            while i < doc.relations.len() {
                let r = &doc.relations[i];
                let touches = match r.kind {
                    RelationKind::Bind => r.bind_target == id || r.bind_skeleton == id,
                    RelationKind::Share => r.share_driver.0 == id || r.share_driven.0 == id,
                    RelationKind::Track => r.track_line == id || r.track_target_line == id,
                    RelationKind::Gradient => r.gradient_bar == id || r.gradient_target == id,
                    RelationKind::Group => r.group_box == id || r.group_members.iter().any(|m| m == &id),
                    RelationKind::Sim => {
                        r.sim_line == id
                            || r.sim_anchors.iter().any(|(_, l, _)| l == &id)
                            || r.sim_collide_with.iter().any(|l| l == &id)
                    }
                    RelationKind::Particles => r.part_emitter == id || r.part_collide_with.iter().any(|l| l == &id),
                };
                if touches {
                    doc.relations.remove(i);
                } else {
                    i += 1;
                }
            }
            let inv = obj_command("line.create@restore", &[("restore", line_restore_value(&removed))]);
            Ok(Effect { inverse: inv, summary: format!("delete line {}", id) })
        }

        "node.set" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let index = cmd.arg("index").and_then(|v| v.as_i64()).ok_or("node.set: 'index' must be an integer")? as usize;
            let l = doc.line(&line_id)?;
            if index >= l.nodes.len() {
                return Err(format!("node.set: index {} out of range (line '{}' has {} nodes)", index, line_id, l.nodes.len()));
            }
            let old = l.nodes[index].clone();

            // Auto-key: atTick + existing track on (line, node, attr) → keyframe upsert.
            let at_tick = cmd.arg("atTick").and_then(|v| v.as_i64());
            if let Some(tick) = at_tick {
                if tick.abs() > TICK_LIMIT {
                    return Err(format!("node.set: atTick {} beyond ±{} limit", tick, TICK_LIMIT));
                }
                let attr = if cmd.arg("p").is_some() {
                    TrackAttr::Position
                } else if cmd.arg("w").is_some() {
                    TrackAttr::Width
                } else {
                    TrackAttr::Color
                };
                if let Some(rel) = track_relation_for(doc, &line_id, index, &attr) {
                    let val = match (attr.clone(), cmd.arg("p"), cmd.arg("w"), cmd.arg("c")) {
                        (TrackAttr::Position, Some(pv), _, _) => {
                            let pos = parse_pos_arg(pv, "node.set p")?;
                            let v = pos.geom().map_err(|e| format!("node.set: auto-key needs a Vec3 position ({})", e))?;
                            track_value_for_attr(attr, v)
                        }
                        (TrackAttr::Width, _, Some(wv), _) => Value::Num(wv.as_f64().ok_or("node.set: 'w' must be a number")?),
                        (TrackAttr::Color, _, _, Some(cv)) => {
                            let c = parse_json_color(cv, "node.set c")?;
                            Value::Arr(vec![
                                Value::Num(c[0] as f64),
                                Value::Num(c[1] as f64),
                                Value::Num(c[2] as f64),
                                Value::Num(c[3] as f64),
                            ])
                        }
                        _ => return Err("node.set: auto-key needs 'p', 'w' or 'c' alongside atTick".into()),
                    };
                    upsert_keyframe(doc, &rel, tick, val, false)?;
                    // Inverse: remove the keyframe again. (Documented limitation:
                    // undoing the re-key of an existing tick cannot resurrect the
                    // previous value — record-level undo restores it; see history.)
                    let inv = obj_command(
                        "node.set@dropKey",
                        &[("line", Value::Str(line_id.clone())), ("index", Value::Int(index as i64)), ("atTick", Value::Int(tick))],
                    );
                    return Ok(Effect { inverse: inv, summary: format!("auto-key {} node {} @tick {}", line_id, index, tick) });
                }
            }

            // Plain (static) set.
            let l = doc.line_mut(&line_id)?;
            let mut inv = Value::obj();
            inv.set("op", Value::Str("node.set".into()));
            inv.set("line", Value::Str(line_id.clone()));
            inv.set("index", Value::Int(index as i64));
            let mut any = false;
            if let Some(pv) = cmd.arg("p") {
                let pos = parse_pos_arg(pv, "node.set p")?;
                // Geometric ↔ track switch is not allowed (keeps line kind stable).
                match (&old.pos, &pos) {
                    (NodePos::Geom(_), NodePos::Geom(_)) | (NodePos::Tick(_), NodePos::Tick(_)) => {}
                    _ => return Err("node.set: cannot switch node kind (geom ↔ track) — lines are uniformly geometric or track lines".into()),
                }
                inv.set("p", value_of_pos(&old.pos));
                l.nodes[index].pos = pos;
                any = true;
            }
            if let Some(wv) = cmd.arg("w") {
                inv.set("w", old.width.map(Value::Num).unwrap_or(Value::Null));
                // null clears the override (falls back to line level).
                if matches!(wv, Value::Null) {
                    l.nodes[index].width = None;
                } else {
                    l.nodes[index].width = Some(wv.as_f64().ok_or("node.set: 'w' must be a number")?);
                }
                any = true;
            }
            if let Some(prv) = cmd.arg("pr") {
                inv.set("pr", old.pressure.map(Value::Num).unwrap_or(Value::Null));
                if matches!(prv, Value::Null) {
                    l.nodes[index].pressure = None;
                } else {
                    l.nodes[index].pressure = Some(prv.as_f64().ok_or("node.set: 'pr' must be a number")?);
                }
                any = true;
            }
            if let Some(uvv) = cmd.arg("uv") {
                inv.set("uv", old.uv.map(|uv| Value::Arr(vec![Value::Num(uv[0]), Value::Num(uv[1])])).unwrap_or(Value::Null));
                if matches!(uvv, Value::Null) {
                    l.nodes[index].uv = None;
                } else {
                    let arr = uvv.as_arr().ok_or("node.set: 'uv' must be [u,v]")?;
                    if arr.len() != 2 {
                        return Err("node.set: 'uv' must have 2 components".into());
                    }
                    l.nodes[index].uv = Some([
                        arr[0].as_f64().ok_or("node.set: uv[0] must be a number")?,
                        arr[1].as_f64().ok_or("node.set: uv[1] must be a number")?,
                    ]);
                }
                any = true;
            }
            if let Some(cv) = cmd.arg("c") {
                inv.set("c", old.color.map(color_arr).unwrap_or(Value::Null));
                if matches!(cv, Value::Null) {
                    l.nodes[index].color = None;
                } else {
                    l.nodes[index].color = Some(parse_json_color(cv, "node.set c")?);
                }
                any = true;
            }
            if let Some(hv) = cmd.arg("hi") {
                inv.set("hi", old.handle_in.map(vec3_value).unwrap_or(Value::Null));
                if matches!(hv, Value::Null) {
                    l.nodes[index].handle_in = None;
                } else {
                    l.nodes[index].handle_in = Some(parse_json_vec3(hv, "node.set hi")?);
                }
                any = true;
            }
            if let Some(hv) = cmd.arg("ho") {
                inv.set("ho", old.handle_out.map(vec3_value).unwrap_or(Value::Null));
                if matches!(hv, Value::Null) {
                    l.nodes[index].handle_out = None;
                } else {
                    l.nodes[index].handle_out = Some(parse_json_vec3(hv, "node.set ho")?);
                }
                any = true;
            }
            if cmd.arg("atTick").is_some() {
                // atTick given but no track matched: plain set above (static value).
            }
            if !any {
                return Err("node.set: nothing to set (p/w/pr/uv/c/hi/ho)".into());
            }
            let inv = value_to_command(&inv)?;
            Ok(Effect { inverse: inv, summary: format!("set node {}#{}", line_id, index) })
        }

        "node.append" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let pv = cmd.arg("p").ok_or("node.append: 'p' required")?;
            let pos = parse_pos_arg(pv, "node.append p")?;
            let mut n = match pos {
                NodePos::Geom(g) => Node::geom(g),
                NodePos::Tick(t) => {
                    let val = cmd.arg("v").cloned().ok_or("node.append: track node needs 'v'")?;
                    Node::track(t, val)
                }
            };
            if let Some(w) = cmd.arg("w").and_then(|v| v.as_f64()) {
                n.width = Some(w);
            }
            if let Some(c) = cmd.arg("c") {
                n.color = Some(parse_json_color(c, "node.append c")?);
            }
            if let Some(h) = cmd.arg("hi") {
                n.handle_in = Some(parse_json_vec3(h, "node.append hi")?);
            }
            if let Some(h) = cmd.arg("ho") {
                n.handle_out = Some(parse_json_vec3(h, "node.append ho")?);
            }
            let l = doc.line_mut(&line_id)?;
            // Track lines keep ticks ascending on insert.
            if let NodePos::Tick(t) = n.pos {
                if let Some(last) = l.nodes.last() {
                    if let NodePos::Tick(lt) = last.pos {
                        if t <= lt {
                            return Err(format!("node.append: track ticks must be strictly ascending ({} <= {})", t, lt));
                        }
                    }
                }
            }
            l.nodes.push(n);
            l.sync_segs();
            let inv = obj_command("node.delete", &[("line", Value::Str(line_id.clone())), ("index", Value::Int(l.nodes.len() as i64 - 1))]);
            Ok(Effect { inverse: inv, summary: format!("append node to {}", line_id) })
        }

        "node.delete" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let index = cmd.arg("index").and_then(|v| v.as_i64()).ok_or("node.delete: 'index' must be an integer")? as usize;
            let l = doc.line_mut(&line_id)?;
            if index >= l.nodes.len() {
                return Err(format!("node.delete: index {} out of range", index));
            }
            let removed = l.nodes.remove(index);
            if l.nodes.len() < 2 && !l.nodes.is_empty() {
                // Lines need ≥ 2 nodes; allow dropping to 0 only for track lines being emptied by undo.
                if !l.is_track() {
                    l.nodes.insert(index, removed.clone());
                    return Err("node.delete: a geometric line must keep at least 2 nodes".into());
                }
            }
            l.sync_segs();
            let inv = obj_command("node.insert", &[("line", Value::Str(line_id.clone())), ("index", Value::Int(index as i64)), ("node", node_value(&removed))]);
            Ok(Effect { inverse: inv, summary: format!("delete node {}#{}", line_id, index) })
        }

        "node.insert" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let index = cmd.arg("index").and_then(|v| v.as_i64()).ok_or("node.insert: 'index' must be an integer")? as usize;
            let nv = cmd.arg("node").ok_or("node.insert: 'node' required")?;
            let n = parse_node_spec(cmd, nv, "node.insert")?;
            let l = doc.line_mut(&line_id)?;
            if index > l.nodes.len() {
                return Err(format!("node.insert: index {} out of range", index));
            }
            l.nodes.insert(index, n);
            l.sync_segs();
            let inv = obj_command("node.delete", &[("line", Value::Str(line_id.clone())), ("index", Value::Int(index as i64))]);
            Ok(Effect { inverse: inv, summary: format!("insert node {}#{}", line_id, index) })
        }

        "seg.set" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let index = cmd.arg("index").and_then(|v| v.as_i64()).ok_or("seg.set: 'index' must be an integer")? as usize;
            let l = doc.line_mut(&line_id)?;
            if index >= l.segs.len() {
                return Err(format!("seg.set: index {} out of range ({} segs)", index, l.segs.len()));
            }
            let old = l.segs[index].clone();
            let mut inv = Value::obj();
            inv.set("op", Value::Str("seg.set".into()));
            inv.set("line", Value::Str(line_id.clone()));
            inv.set("index", Value::Int(index as i64));
            for (k, v) in &cmd.args {
                if k == "line" || k == "index" {
                    continue;
                }
                match k.as_str() {
                    "type" => {
                        inv.set("type", Value::Str(match old.seg_type { SegType::Straight => "straight", SegType::Curve => "curve" }.into()));
                        l.segs[index].seg_type = match v.as_str() {
                            Some("straight") => SegType::Straight,
                            Some("curve") => SegType::Curve,
                            o => return Err(format!("seg.set: bad type {:?}", o)),
                        };
                    }
                    "visible" => {
                        inv.set("visible", Value::Bool(old.visible));
                        l.segs[index].visible = v.as_bool().ok_or("seg.set: visible must be a bool")?;
                    }
                    "interp" => {
                        inv.set("interp", old.interp.as_ref().map(interp_value).unwrap_or(Value::Null));
                        if matches!(v, Value::Null) {
                            l.segs[index].interp = None;
                        } else {
                            let im = v.get("im").and_then(|x| x.as_i64()).ok_or("seg.set: interp.im required")?;
                            let mut b = None;
                            if let Some(bv) = v.get("b").and_then(|x| x.as_arr()) {
                                let mut handles = Vec::new();
                                for hv in bv {
                                    let mut h = BezierHandle::default_handle();
                                    if let Some(x) = hv.get("ao").and_then(|x| x.as_f64()) { h.ao = x; }
                                    if let Some(x) = hv.get("ai").and_then(|x| x.as_f64()) { h.ai = x; }
                                    if let Some(x) = hv.get("po").and_then(|x| x.as_f64()) { h.po = x; }
                                    if let Some(x) = hv.get("pi").and_then(|x| x.as_f64()) { h.pi = x; }
                                    handles.push(h);
                                }
                                b = Some(handles);
                            }
                            l.segs[index].interp = Some(Interp { im, b });
                        }
                    }
                    other => return Err(format!("seg.set: unknown field '{}'", other)),
                }
            }
            let inv = value_to_command(&inv)?;
            Ok(Effect { inverse: inv, summary: format!("set seg {}#{}", line_id, index) })
        }

        "relation.create" => {
            let ty = need_str(cmd, "type")?;
            let mut r = match ty {
                "bind" => {
                    let target = need_str(cmd, "target")?.to_string();
                    let skeleton = need_str(cmd, "skeleton")?.to_string();
                    Relation::new_bind("", &target, &skeleton)
                }
                "share" => {
                    let d = cmd.arg("driver").ok_or("relation.create: share needs driver {line,node}")?;
                    let dr = cmd.arg("driven").ok_or("relation.create: share needs driven {line,node}")?;
                    let dl = d.get("line").and_then(|x| x.as_str()).ok_or("driver.line missing")?;
                    let dn = d.get("node").and_then(|x| x.as_i64()).ok_or("driver.node missing")?;
                    let rl = dr.get("line").and_then(|x| x.as_str()).ok_or("driven.line missing")?;
                    let rn = dr.get("node").and_then(|x| x.as_i64()).ok_or("driven.node missing")?;
                    Relation::new_share("", (dl, dn as usize), (rl, rn as usize))
                }
                "track" => {
                    let tl = need_str(cmd, "trackLine")?.to_string();
                    let t = cmd.arg("target").ok_or("relation.create: track needs target {line,node?,attr}")?;
                    let line = t.get("line").and_then(|x| x.as_str()).ok_or("target.line missing")?;
                    let node = t.get("node").and_then(|x| x.as_i64()).map(|n| n as usize);
                    let attr = match t.get("attr").and_then(|x| x.as_str()) {
                        Some("position") => TrackAttr::Position,
                        Some("width") => TrackAttr::Width,
                        Some("color") => TrackAttr::Color,
                        o => return Err(format!("relation.create: target.attr must be position|width|color, got {:?}", o)),
                    };
                    Relation::new_track("", &tl, line, node, attr)
                }
                "gradient" => {
                    let bar = need_str(cmd, "bar")?.to_string();
                    let target = need_str(cmd, "target")?.to_string();
                    let role = match cmd.arg("role").and_then(|x| x.as_str()) {
                        Some("fill") | None => GradientRole::Fill,
                        Some("stroke") => GradientRole::Stroke,
                        o => return Err(format!("relation.create: role must be fill|stroke, got {:?}", o)),
                    };
                    Relation::new_gradient("", &bar, &target, role)
                }
                "group" => {
                    let box_id = need_str(cmd, "box")?.to_string();
                    let mut members = Vec::new();
                    if let Some(ms) = cmd.arg("members").and_then(|x| x.as_arr()) {
                        for m in ms {
                            members.push(m.as_str().ok_or("relation.create: members must be strings")?.to_string());
                        }
                    }
                    if members.is_empty() {
                        return Err("relation.create: group needs at least one member".into());
                    }
                    let visible = cmd.arg("visible").and_then(|x| x.as_bool()).unwrap_or(true);
                    Relation::new_group("", &box_id, &members.iter().map(|s| s.as_str()).collect::<Vec<_>>(), visible)
                }
                "sim" => {
                    let line_id = need_str(cmd, "line")?.to_string();
                    let mut r = Relation::new_sim("", &line_id);
                    if let Some(av) = cmd.arg("anchors").and_then(|x| x.as_arr()) {
                        for a in av {
                            let node = a.get("node").and_then(|x| x.as_i64()).ok_or("relation.create: sim anchors[].node must be an integer")?;
                            let d = a.get("driver").ok_or("relation.create: sim anchors[].driver {line,node} required")?;
                            let dl = d.get("line").and_then(|x| x.as_str()).ok_or("driver.line missing")?;
                            let dn = d.get("node").and_then(|x| x.as_i64()).ok_or("driver.node missing")?;
                            r.sim_anchors.push((node as usize, dl.to_string(), dn as usize));
                        }
                    }
                    if let Some(gv) = cmd.arg("gravity") {
                        r.sim_gravity = parse_json_vec3(gv, "relation.create: sim gravity")?;
                    }
                    if let Some(x) = cmd.arg("damping").and_then(|x| x.as_f64()) {
                        r.sim_damping = x;
                    }
                    if let Some(x) = cmd.arg("iterations").and_then(|x| x.as_i64()) {
                        r.sim_iterations = x.max(0) as usize;
                    }
                    if let Some(cv) = cmd.arg("collideWith").and_then(|x| x.as_arr()) {
                        for c in cv {
                            r.sim_collide_with.push(c.as_str().ok_or("relation.create: sim collideWith must be strings")?.to_string());
                        }
                    }
                    if let Some(x) = cmd.arg("radius").and_then(|x| x.as_f64()) {
                        r.sim_radius = x;
                    }
                    if let Some(x) = cmd.arg("selfCollide").and_then(|x| x.as_bool()) {
                        r.sim_self_collide = x;
                    }
                    r
                }
                "particles" => {
                    let emitter = need_str(cmd, "emitter")?.to_string();
                    // node defaults to the emitter's LAST node (tail-tip emission).
                    let node = match cmd.arg("node").and_then(|x| x.as_i64()) {
                        Some(n) => n as usize,
                        None => doc.line(&emitter).map(|l| l.nodes.len().saturating_sub(1)).unwrap_or(0),
                    };
                    let mut r = Relation::new_particles("", &emitter, node);
                    if let Some(x) = cmd.arg("rate").and_then(|x| x.as_f64()) {
                        r.part_rate = x;
                    }
                    if let Some(x) = cmd.arg("speed").and_then(|x| x.as_f64()) {
                        r.part_speed = x;
                    }
                    if let Some(dv) = cmd.arg("dir") {
                        r.part_dir = parse_json_vec3(dv, "relation.create: particles dir")?;
                    }
                    if let Some(x) = cmd.arg("spread").and_then(|x| x.as_f64()) {
                        r.part_spread_deg = x;
                    }
                    if let Some(x) = cmd.arg("life").and_then(|x| x.as_f64()) {
                        r.part_life_s = x;
                    }
                    if let Some(gv) = cmd.arg("gravity") {
                        r.part_gravity = parse_json_vec3(gv, "relation.create: particles gravity")?;
                    }
                    if let Some(x) = cmd.arg("damping").and_then(|x| x.as_f64()) {
                        r.part_damping = x;
                    }
                    if let Some(x) = cmd.arg("size").and_then(|x| x.as_f64()) {
                        r.part_size = x;
                    }
                    if let Some(cv) = cmd.arg("color") {
                        r.part_color = parse_json_color(cv, "relation.create: particles color")?;
                    }
                    if let Some(x) = cmd.arg("len").and_then(|x| x.as_f64()) {
                        r.part_len = x;
                    }
                    if let Some(x) = cmd.arg("seed").and_then(|x| x.as_i64()) {
                        r.part_seed = x;
                    }
                    if let Some(cv) = cmd.arg("collideWith").and_then(|x| x.as_arr()) {
                        for c in cv {
                            r.part_collide_with.push(c.as_str().ok_or("relation.create: particles collideWith must be strings")?.to_string());
                        }
                    }
                    if let Some(x) = cmd.arg("bounce").and_then(|x| x.as_f64()) {
                        r.part_bounce = x;
                    }
                    if let Some(x) = cmd.arg("radius").and_then(|x| x.as_f64()) {
                        r.part_radius = x;
                    }
                    r
                }
                o => return Err(format!("relation.create: unknown type '{}'", o)),
            };
            r.id = match cmd.arg("id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => next_auto_id(doc, "r"),
            };
            if doc.relation(&r.id).is_ok() {
                return Err(format!("relation '{}' already exists", r.id));
            }
            if let Some(f) = cmd.arg("falloff").and_then(|v| v.as_f64()) {
                r.bind_falloff = f;
            }
            if let Some(b) = cmd.arg("stretch").and_then(|v| v.as_bool()) {
                r.bind_stretch = b;
            }
            // Referential checks.
            match r.kind {
                RelationKind::Bind => {
                    doc.line(&r.bind_target).map_err(|e| format!("relation.create bind: {}", e))?;
                    doc.line(&r.bind_skeleton).map_err(|e| format!("relation.create bind: {}", e))?;
                    if r.bind_target == r.bind_skeleton {
                        return Err("relation.create bind: target and skeleton must differ".into());
                    }
                }
                RelationKind::Share => {
                    let dl = doc.line(&r.share_driver.0).map_err(|e| format!("relation.create share: {}", e))?;
                    let dr = doc.line(&r.share_driven.0).map_err(|e| format!("relation.create share: {}", e))?;
                    if r.share_driver.1 >= dl.nodes.len() {
                        return Err(format!("relation.create share: driver node {} out of range", r.share_driver.1));
                    }
                    if r.share_driven.1 >= dr.nodes.len() {
                        return Err(format!("relation.create share: driven node {} out of range", r.share_driven.1));
                    }
                }
                RelationKind::Track => {
                    doc.line(&r.track_line).map_err(|e| format!("relation.create track: {}", e))?;
                    let t = doc.line(&r.track_target_line).map_err(|e| format!("relation.create track: {}", e))?;
                    if let Some(n) = r.track_target_node {
                        if n >= t.nodes.len() {
                            return Err(format!("relation.create track: target node {} out of range", n));
                        }
                    }
                    if r.track_line == r.track_target_line {
                        return Err("relation.create track: track line and target line must differ".into());
                    }
                }
                RelationKind::Gradient => {
                    doc.line(&r.gradient_bar).map_err(|e| format!("relation.create gradient: {}", e))?;
                    doc.line(&r.gradient_target).map_err(|e| format!("relation.create gradient: {}", e))?;
                    let b = doc.line(&r.gradient_bar)?;
                    if b.is_track() {
                        return Err("relation.create gradient: bar must be a geometric line".into());
                    }
                    if b.nodes.len() < 2 {
                        return Err("relation.create gradient: bar needs ≥ 2 nodes (color stops)".into());
                    }
                    if doc.line(&r.gradient_target)?.is_track() {
                        return Err("relation.create gradient: cannot target a track line".into());
                    }
                    if r.gradient_bar == r.gradient_target {
                        return Err("relation.create gradient: bar and target must differ".into());
                    }
                }
                RelationKind::Group => {
                    let b = doc.line(&r.group_box).map_err(|e| format!("relation.create group: {}", e))?;
                    if b.is_track() {
                        return Err("relation.create group: box must be a geometric line".into());
                    }
                    if b.nodes.len() < 2 {
                        return Err("relation.create group: box needs ≥ 2 nodes".into());
                    }
                    for m in &r.group_members {
                        let l = doc.line(m).map_err(|e| format!("relation.create group: {}", e))?;
                        if l.is_track() {
                            return Err(format!("relation.create group: member '{}' must be a geometric line", m));
                        }
                        if m == &r.group_box {
                            return Err("relation.create group: the box cannot be its own member".into());
                        }
                    }
                }
                RelationKind::Sim => {
                    let l = doc.line(&r.sim_line).map_err(|e| format!("relation.create sim: {}", e))?;
                    if l.is_track() {
                        return Err("relation.create sim: the sim line must be a geometric line".into());
                    }
                    if l.nodes.len() < 2 {
                        return Err("relation.create sim: the sim line needs ≥ 2 nodes".into());
                    }
                    for (node, dl, dn) in &r.sim_anchors {
                        if *node >= l.nodes.len() {
                            return Err(format!("relation.create sim: anchor node {} out of range", node));
                        }
                        let al = doc.line(dl).map_err(|e| format!("relation.create sim: {}", e))?;
                        if al.is_track() {
                            return Err(format!("relation.create sim: anchor driver '{}' must be a geometric line", dl));
                        }
                        if *dn >= al.nodes.len() {
                            return Err(format!("relation.create sim: anchor driver node {} out of range", dn));
                        }
                    }
                    if r.sim_iterations > 100 {
                        return Err("relation.create sim: iterations cap at 100".into());
                    }
                    for c in &r.sim_collide_with {
                        let cl = doc.line(c).map_err(|e| format!("relation.create sim: {}", e))?;
                        if cl.is_track() {
                            return Err(format!("relation.create sim: collider '{}' must be a geometric line", c));
                        }
                        if c == &r.sim_line {
                            return Err("relation.create sim: the sim line cannot collide with itself here (use selfCollide)".into());
                        }
                    }
                    if r.sim_radius < 0.0 {
                        return Err("relation.create sim: radius must be >= 0".into());
                    }
                }
                RelationKind::Particles => {
                    let l = doc.line(&r.part_emitter).map_err(|e| format!("relation.create particles: {}", e))?;
                    if l.is_track() {
                        return Err("relation.create particles: emitter must be a geometric line".into());
                    }
                    if r.part_node >= l.nodes.len() {
                        return Err(format!("relation.create particles: emitter node {} out of range", r.part_node));
                    }
                    if r.part_rate <= 0.0 {
                        return Err("relation.create particles: rate must be > 0".into());
                    }
                    if r.part_life_s <= 0.0 {
                        return Err("relation.create particles: life must be > 0".into());
                    }
                    if !(0.0..=360.0).contains(&r.part_spread_deg) {
                        return Err("relation.create particles: spread must be in 0..360 degrees".into());
                    }
                    for c in &r.part_collide_with {
                        let cl = doc.line(c).map_err(|e| format!("relation.create particles: {}", e))?;
                        if cl.is_track() {
                            return Err(format!("relation.create particles: collider '{}' must be a geometric line", c));
                        }
                    }
                    if !(0.0..=1.0).contains(&r.part_bounce) {
                        return Err("relation.create particles: bounce must be in 0..1".into());
                    }
                    if r.part_radius < 0.0 {
                        return Err("relation.create particles: radius must be >= 0".into());
                    }
                }
            }
            doc.relations.push(r.clone());
            let inv = obj_command("relation.delete", &[("id", Value::Str(r.id.clone()))]);
            Ok(Effect { inverse: inv, summary: format!("create relation {} ({})", r.id, r.kind.name()) })
        }

        "track.create" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let node = cmd.arg("node").and_then(|v| v.as_i64()).ok_or("track.create: 'node' must be an integer")? as usize;
            let attr = match cmd.arg("attr").and_then(|v| v.as_str()) {
                Some("position") | None => TrackAttr::Position,
                Some("width") => TrackAttr::Width,
                Some("color") => TrackAttr::Color,
                Some(o) => return Err(format!("track.create: attr must be position|width|color, got '{}'", o)),
            };
            let tick = cmd.arg("tick").and_then(|v| v.as_i64()).unwrap_or(0);
            if tick.abs() > TICK_LIMIT {
                return Err(format!("track.create: tick {} beyond ±{} limit", tick, TICK_LIMIT));
            }
            let l = doc.line(&line_id)?;
            if l.is_track() {
                return Err(format!("track.create: cannot track a track line '{}'", line_id));
            }
            if node >= l.nodes.len() {
                return Err(format!("track.create: node {} out of range ({} nodes)", node, l.nodes.len()));
            }
            // Existing track on this target? Reject (one writer per output).
            if track_relation_for(doc, &line_id, node, &attr).is_some() {
                return Err(format!("track.create: a track already drives {}#{} {}", line_id, node, attr.name()));
            }
            // First keyframe value: explicit, else the node's current value.
            let value = match cmd.arg("value") {
                Some(v) => v.clone(),
                None => match (&attr, &l.nodes[node]) {
                    (TrackAttr::Position, n) => match n.pos {
                        NodePos::Geom(g) => track_value_for_attr(*&attr, g),
                        NodePos::Tick(_) => unreachable!("checked above"),
                    },
                    (TrackAttr::Width, n) => Value::Num(n.eff_width(l.width)),
                    (TrackAttr::Color, n) => {
                        let c = n.color.unwrap_or(l.color);
                        Value::Arr(vec![
                            Value::Num(c[0] as f64),
                            Value::Num(c[1] as f64),
                            Value::Num(c[2] as f64),
                            Value::Num(c[3] as f64),
                        ])
                    }
                },
            };
            let track_id = auto_track_id(&line_id, node, &attr);
            if doc.line(&track_id).is_ok() {
                return Err(format!("track.create: track line '{}' already exists", track_id));
            }
            let mut tl = Line::new(&track_id);
            tl.tags = vec!["track".to_string()];
            tl.nodes.push(Node::track(tick, value));
            tl.sync_segs();
            doc.lines.push(tl);
            let rel_id = next_auto_id(doc, "r");
            let rel = Relation::new_track(&rel_id, &track_id, &line_id, Some(node), attr.clone());
            doc.relations.push(rel);
            // Undo: delete relation + track line (one internal multi-op inverse).
            let inv = obj_command(
                "track.create@undo",
                &[("relation", Value::Str(rel_id)), ("trackLine", Value::Str(track_id.clone()))],
            );
            Ok(Effect { inverse: inv, summary: format!("create track {} for {}#{} {}", track_id, line_id, node, attr.name()) })
        }

        "track.create@undo" => {
            let rel_id = need_str(cmd, "relation")?.to_string();
            let tl_id = need_str(cmd, "trackLine")?.to_string();
            doc.relations.retain(|r| r.id != rel_id);
            doc.lines.retain(|l| l.id != tl_id);
            Ok(Effect { inverse: obj_command("noop", &[]), summary: format!("undo track.create {}", tl_id) })
        }

        "relation.delete" => {
            let id = need_str(cmd, "id")?.to_string();
            let idx = doc.relations.iter().position(|r| r.id == id).ok_or_else(|| format!("relation '{}' not found", id))?;
            let removed = doc.relations.remove(idx);
            let inv = obj_command("relation.create@restore", &[("restore", relation_restore_value(&removed))]);
            Ok(Effect { inverse: inv, summary: format!("delete relation {}", id) })
        }

        // Internal inverse ops (not in the public registry): restore forms.
        "line.create@restore" => {
            let rv = cmd.arg("restore").ok_or("restore payload missing")?;
            let l = line_from_restore(rv)?;
            doc.lines.push(l.clone());
            let inv = obj_command("line.delete", &[("id", Value::Str(l.id.clone()))]);
            Ok(Effect { inverse: inv, summary: format!("restore line {}", l.id) })
        }
        "relation.create@restore" => {
            let rv = cmd.arg("restore").ok_or("restore payload missing")?;
            let r = relation_from_restore(rv)?;
            doc.relations.push(r.clone());
            let inv = obj_command("relation.delete", &[("id", Value::Str(r.id.clone()))]);
            Ok(Effect { inverse: inv, summary: format!("restore relation {}", r.id) })
        }
        "node.set@dropKey" => {
            // Undo of an auto-key: remove the keyframe at atTick.
            let line_id = need_str(cmd, "line")?.to_string();
            let index = cmd.arg("index").and_then(|v| v.as_i64()).ok_or("index required")? as usize;
            let tick = cmd.arg("atTick").and_then(|v| v.as_i64()).ok_or("atTick required")?;
            let attr = TrackAttr::Position;
            if let Some(rel) = track_relation_for(doc, &line_id, index, &attr) {
                if let Ok(tl) = doc.line_mut(&rel.track_line) {
                    tl.nodes.retain(|n| !matches!(n.pos, NodePos::Tick(t) if t == tick));
                    tl.sync_segs();
                }
            } else if let Some(rel) = track_relation_for(doc, &line_id, index, &TrackAttr::Width) {
                if let Ok(tl) = doc.line_mut(&rel.track_line) {
                    tl.nodes.retain(|n| !matches!(n.pos, NodePos::Tick(t) if t == tick));
                    tl.sync_segs();
                }
            } else if let Some(rel) = track_relation_for(doc, &line_id, index, &TrackAttr::Color) {
                if let Ok(tl) = doc.line_mut(&rel.track_line) {
                    tl.nodes.retain(|n| !matches!(n.pos, NodePos::Tick(t) if t == tick));
                    tl.sync_segs();
                }
            }
            Ok(Effect { inverse: obj_command("noop", &[]), summary: format!("drop key {}#{} @{}", line_id, index, tick) })
        }

        "noop" => Ok(Effect { inverse: obj_command("noop", &[]), summary: "noop".into() }),

        // -------------------------------------------------------------------
        // B2 styles: named line-attribute templates
        // -------------------------------------------------------------------
        "style.create" => {
            let id = match cmd.arg("id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => next_auto_id(doc, "st"),
            };
            if doc.style(&id).is_ok() {
                return Err(format!("style '{}' already exists", id));
            }
            let mut s = Style::new(&id);
            if let Some(x) = cmd.arg("width").and_then(|v| v.as_f64()) {
                s.width = Some(x);
            }
            if let Some(cv) = cmd.arg("color") {
                s.color = Some(parse_json_color(cv, "style.create color")?);
            }
            if let Some(cv) = cmd.arg("fillColor") {
                s.fill_color = Some(parse_json_color(cv, "style.create fillColor")?);
            }
            if let Some(b) = cmd.arg("strokeOn").and_then(|v| v.as_bool()) {
                s.stroke_on = Some(b);
            }
            if let Some(b) = cmd.arg("fillOn").and_then(|v| v.as_bool()) {
                s.fill_on = Some(b);
            }
            if let Some(bv) = cmd.arg("brush") {
                s.brush = Some(parse_brush_arg(bv, "style.create brush")?);
            }
            let defined = s.width.is_some()
                || s.color.is_some()
                || s.fill_color.is_some()
                || s.stroke_on.is_some()
                || s.fill_on.is_some()
                || s.brush.is_some();
            if !defined {
                return Err("style.create: nothing defined (width/color/fillColor/strokeOn/fillOn/brush)".into());
            }
            let inv = obj_command("style.delete@restore", &[("restore", style_restore_value(&s))]);
            doc.styles.push(s);
            Ok(Effect { inverse: inv, summary: format!("create style {}", id) })
        }

        "style.delete" => {
            let id = need_str(cmd, "id")?.to_string();
            let idx = doc.styles.iter().position(|s| s.id == id).ok_or_else(|| format!("style '{}' not found", id))?;
            let removed = doc.styles.remove(idx);
            let inv = obj_command("style.create@restore", &[("restore", style_restore_value(&removed))]);
            Ok(Effect { inverse: inv, summary: format!("delete style {}", id) })
        }

        "style.apply" => {
            let sid = need_str(cmd, "id")?.to_string();
            let line_id = need_str(cmd, "line")?.to_string();
            let s = doc.style(&sid)?.clone();
            doc.line(&line_id)?;
            // Undo carries the complete pre-apply line-level state.
            let mut inv = Value::obj();
            inv.set("op", Value::Str("style.apply@undo".into()));
            inv.set("line", Value::Str(line_id.clone()));
            {
                let l = doc.line(&line_id)?;
                inv.set("width", Value::Num(l.width));
                inv.set("color", color_arr(l.color));
                inv.set("fillColor", color_arr(l.fill_color));
                inv.set("strokeOn", Value::Bool(l.stroke_on));
                inv.set("fillOn", Value::Bool(l.fill_on));
                inv.set("brush", l.brush.as_ref().map(|b| format::brush_to_value(b)).unwrap_or(Value::Null));
            }
            // Defined template fields copy over; undefined fields leave the line alone.
            let l = doc.line_mut(&line_id)?;
            if let Some(x) = s.width {
                l.width = x;
            }
            if let Some(x) = s.color {
                l.color = x;
            }
            if let Some(x) = s.fill_color {
                l.fill_color = x;
            }
            if let Some(x) = s.stroke_on {
                l.stroke_on = x;
            }
            if let Some(x) = s.fill_on {
                l.fill_on = x;
            }
            if s.brush.is_some() {
                l.brush = s.brush.clone();
            }
            Ok(Effect { inverse: value_to_command(&inv)?, summary: format!("apply style {} to {}", sid, line_id) })
        }

        "style.apply@undo" => {
            let line_id = need_str(cmd, "line")?.to_string();
            let l = doc.line_mut(&line_id)?;
            for (k, v) in &cmd.args {
                match k.as_str() {
                    "line" => {}
                    "width" => l.width = v.as_f64().ok_or("bad undo width")?,
                    "color" => l.color = parse_json_color(v, "undo color")?,
                    "fillColor" => l.fill_color = parse_json_color(v, "undo fillColor")?,
                    "strokeOn" => l.stroke_on = v.as_bool().ok_or("bad undo strokeOn")?,
                    "fillOn" => l.fill_on = v.as_bool().ok_or("bad undo fillOn")?,
                    "brush" => {
                        l.brush = if matches!(v, Value::Null) {
                            None
                        } else {
                            Some(parse_brush_arg(v, "undo brush")?)
                        };
                    }
                    other => return Err(format!("style.apply@undo: unknown field '{}'", other)),
                }
            }
            Ok(Effect { inverse: obj_command("noop", &[]), summary: format!("undo style apply on {}", line_id) })
        }

        "style.create@restore" => {
            let rv = cmd.arg("restore").ok_or("restore payload missing")?;
            let s = format::style_from_value(rv)?;
            doc.styles.push(s.clone());
            Ok(Effect { inverse: obj_command("style.delete", &[("id", Value::Str(s.id.clone()))]), summary: format!("restore style {}", s.id) })
        }

        "style.delete@restore" => {
            let rv = cmd.arg("restore").ok_or("restore payload missing")?;
            let s = format::style_from_value(rv)?;
            doc.styles.push(s);
            Ok(Effect { inverse: obj_command("noop", &[]), summary: "undo style delete".into() })
        }

        other => Err(format!("unknown command '{}' (protocol v{}; see registry)", other, PROTOCOL_VERSION)),
    }
}

fn style_restore_value(s: &Style) -> Value {
    let mut doc = Document::default();
    doc.styles.push(s.clone());
    let v = format::doc_to_value(&doc);
    v.get("styles").and_then(|x| x.as_arr()).and_then(|a| a.first()).cloned().unwrap_or(Value::obj())
}

fn color_arr(c: [u8; 4]) -> Value {
    Value::Arr(c.iter().map(|b| Value::Int(*b as i64)).collect())
}
fn vec3_value(v: Vec3) -> Value {
    Value::Arr(vec![Value::Num(v.x), Value::Num(v.y), Value::Num(v.z)])
}
fn interp_value(i: &Interp) -> Value {
    let mut o = Value::obj();
    o.set("im", Value::Int(i.im));
    if let Some(b) = &i.b {
        o.set(
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
    o
}
fn node_value(n: &Node) -> Value {
    let mut o = Value::obj();
    o.set("p", value_of_pos(&n.pos));
    if let Some(w) = n.width {
        o.set("w", Value::Num(w));
    }
    if let Some(pr) = n.pressure {
        o.set("pr", Value::Num(pr));
    }
    if let Some(uv) = n.uv {
        o.set("uv", Value::Arr(vec![Value::Num(uv[0]), Value::Num(uv[1])]));
    }
    if let Some(c) = n.color {
        o.set("c", color_arr(c));
    }
    if let Some(h) = n.handle_in {
        o.set("hi", vec3_value(h));
    }
    if let Some(h) = n.handle_out {
        o.set("ho", vec3_value(h));
    }
    if let Some(v) = &n.value {
        o.set("v", v.clone());
    }
    o
}

fn line_restore_value(l: &Line) -> Value {
    // Reuse the format serializer by converting the line through a doc value.
    let mut doc = Document::default();
    doc.lines.push(l.clone());
    let v = format::doc_to_value(&doc);
    v.get("lines").and_then(|x| x.as_arr()).and_then(|a| a.first()).cloned().unwrap_or(Value::obj())
}

fn line_from_restore(v: &Value) -> Result<Line, String> {
    // Route through the document parser for full fidelity (unknown keys ride along).
    let full = {
        let mut full = Value::obj();
        full.set("format", Value::Str("mo2d".into()));
        full.set("version", Value::Int(1));
        full.set("minReader", Value::Int(1));
        full.set("generator", Value::Str("mo2d 0.1.0".into()));
        full.set("root", Value::obj());
        full.set("lines", Value::Arr(vec![v.clone()]));
        full.set("relations", Value::Arr(vec![]));
        full
    };
    let doc = format::doc_from_value(&full)?;
    doc.lines.into_iter().next().ok_or_else(|| "restore: no line parsed".to_string())
}

fn relation_restore_value(r: &Relation) -> Value {
    let mut doc = Document::default();
    doc.relations.push(r.clone());
    let v = format::doc_to_value(&doc);
    v.get("relations").and_then(|x| x.as_arr()).and_then(|a| a.first()).cloned().unwrap_or(Value::obj())
}

fn relation_from_restore(v: &Value) -> Result<Relation, String> {
    let mut full = Value::obj();
    full.set("format", Value::Str("mo2d".into()));
    full.set("version", Value::Int(1));
    full.set("minReader", Value::Int(1));
    full.set("generator", Value::Str("mo2d 0.1.0".into()));
    full.set("root", Value::obj());
    full.set("lines", Value::Arr(vec![]));
    full.set("relations", Value::Arr(vec![v.clone()]));
    let doc = format::doc_from_value(&full)?;
    doc.relations.into_iter().next().ok_or_else(|| "restore: no relation parsed".to_string())
}

fn obj_command(op: &str, fields: &[(&str, Value)]) -> Command {
    Command {
        op: op.to_string(),
        args: fields.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
    }
}

fn value_to_command(v: &Value) -> Result<Command, String> {
    let op = v.get("op").and_then(|x| x.as_str()).ok_or("inverse: missing op")?.to_string();
    let args = match v {
        Value::Obj(fields) => fields.iter().filter(|(k, _)| k != "op").cloned().collect(),
        _ => Vec::new(),
    };
    Ok(Command { op, args })
}

/// Apply a stream of commands; on the first error the document is untouched
/// (all-or-nothing batch, checked by the determinism gate).
pub fn apply_stream(doc: &mut Document, cmds: &[Command]) -> Result<Vec<Effect>, String> {
    let backup = doc.clone();
    let mut effects = Vec::with_capacity(cmds.len());
    for (i, cmd) in cmds.iter().enumerate() {
        match apply(doc, cmd) {
            Ok(e) => effects.push(e),
            Err(e) => {
                *doc = backup;
                return Err(format!("cmds[{}] ({}): {}", i, cmd.op, e));
            }
        }
    }
    Ok(effects)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(text: &str) -> Vec<Command> {
        parse_stream(text).unwrap()
    }

    #[test]
    fn version_gate() {
        assert!(parse_stream(r#"{"v":2,"cmds":[]}"#).is_err(), "future protocol versions must be rejected, not guessed");
        assert!(parse_stream(r#"{"cmds":[]}"#).is_err(), "missing version must be rejected");
    }

    #[test]
    fn create_and_inverse() {
        let mut doc = Document::default();
        let cmds = op(r#"{"v":1,"cmds":[{"op":"line.create","id":"body","nodes":[{"p":[0,0,0]},{"p":[100,0,0]}]}]}"#);
        let effects = apply_stream(&mut doc, &cmds).unwrap();
        assert_eq!(doc.lines.len(), 1);
        let undo = apply(&mut doc, &effects[0].inverse).unwrap();
        assert_eq!(doc.lines.len(), 0);
        let _ = apply(&mut doc, &undo.inverse).unwrap();
        assert_eq!(doc.lines.len(), 1);
    }

    #[test]
    fn all_or_nothing() {
        let mut doc = Document::default();
        let cmds = op(r#"{"v":1,"cmds":[{"op":"line.create","id":"a","nodes":[{"p":[0,0,0]},{"p":[1,0,0]}]},{"op":"bogus.op"}]}"#);
        let err = apply_stream(&mut doc, &cmds).unwrap_err();
        assert!(err.contains("bogus.op"));
        assert_eq!(doc.lines.len(), 0, "failed batch must leave the document untouched");
    }

    #[test]
    fn autokey_upserts_sorted() {
        let mut doc = Document::default();
        let cmds = op(
            r#"{"v":1,"cmds":[
              {"op":"line.create","id":"skel","tags":["skeleton"],"nodes":[{"p":[0,0,0]},{"p":[50,0,0]},{"p":[100,0,0]}]},
              {"op":"line.create","id":"trk_skel_2_position","tags":["track"],"nodes":[{"p":[0],"v":[0,0,0]},{"p":[254016000000],"v":[10,20,0]}]},
              {"op":"relation.create","id":"r1","type":"track","trackLine":"trk_skel_2_position","target":{"line":"skel","node":2,"attr":"position"}}
            ]}"#,
        );
        apply_stream(&mut doc, &cmds).unwrap();
        // auto-key in the middle
        let key = op(r#"{"v":1,"cmds":[{"op":"node.set","line":"skel","index":2,"p":[5,5,0],"atTick":127008000000}]}"#);
        apply_stream(&mut doc, &key).unwrap();
        let tl = doc.line("trk_skel_2_position").unwrap();
        let ticks: Vec<i64> = tl.nodes.iter().map(|n| n.pos.tick().unwrap()).collect();
        assert_eq!(ticks, vec![0, 127008000000, 254016000000], "keyframes stay tick-ascending after upsert");
    }
}
