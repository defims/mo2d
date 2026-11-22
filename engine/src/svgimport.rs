//! C3 SVG geometry import (roadmap C3 "几何导入"): parse an SVG document
//! (XML subset) into mo2d editable lines wrapped as one layer (group +
//! transform-box line). Zero dependencies: hand-rolled XML scanner and path
//! parser, deterministic by construction (document-order iteration, IEEE
//! f64 arithmetic; libm trig only inside arc→cubic conversion — same
//! precedent as Bézier handle evaluation).
//!
//! v1 scope: `path` (all commands M/L/H/V/C/S/Q/T/A/Z, absolute+relative,
//! implicit repeats), `rect`/`circle`/`ellipse`/`line`/`polygon`/`polyline`;
//! transforms flattened (`translate`/`scale`/`rotate`/`matrix`/`skew`,
//! composed left-to-right per SVG semantics); paint inheritance through
//! `g` (fill/stroke/fill-opacity/opacity/stroke-width + inline `style`
//! attr); y-down → y-up flip via the document height. Everything converts
//! to cubic-Bézier editable lines (arcs approximated, quadratics elevated)
//! — fully editable afterwards (nodes/handles are ordinary mo2d nodes).
//!
//! Skipped subtrees (v1): defs/symbol/use/clipPath/mask/marker/pattern/
//! linearGradient/radialGradient/filter/style/title/desc/metadata/text/
//! tspan/image/foreignObject/script. CSS classes are not resolved.

use crate::json::Value;
use crate::math::Vec3;
use crate::model::{Line, Node, NodePos, SegAttrs, SegType};

// ---------------------------------------------------------------------------
// XML scanner (minimal, SVG-sufficient)
// ---------------------------------------------------------------------------

struct Attr {
    name: String,
    value: String,
}

enum Scan {
    Open { tag: String, attrs: Vec<Attr>, self_close: bool },
    Close { tag: String },
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'&' {
            if let Some(semi) = s[i..].find(';') {
                let ent = &s[i + 1..i + semi];
                let repl = match ent {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16).ok().and_then(char::from_u32),
                    e if e.starts_with('#') => e[1..].parse::<u32>().ok().and_then(char::from_u32),
                    _ => None,
                };
                if let Some(c) = repl {
                    out.push(c);
                    i += semi + 1;
                    continue;
                }
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

/// Walk the document yielding Open/Close events (an empty element yields
/// Open{self_close:true} immediately followed by Close). Malformed XML → Err.
fn scan_elements(xml: &str, mut f: impl FnMut(Scan) -> Result<(), String>) -> Result<(), String> {
    let b = xml.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<String> = Vec::new();
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        if xml[i..].starts_with("<!--") {
            i = xml[i + 4..].find("-->").map(|p| i + 4 + p + 3).ok_or("unterminated comment")?;
            continue;
        }
        if xml[i..].starts_with("<![CDATA[") {
            i = xml[i + 9..].find("]]>").map(|p| i + 9 + p + 3).ok_or("unterminated CDATA")?;
            continue;
        }
        if xml[i..].starts_with("<?") || xml[i..].starts_with("<!") {
            i = xml[i + 2..].find('>').map(|p| i + 2 + p + 1).ok_or("unterminated markup declaration")?;
            continue;
        }
        if xml[i..].starts_with("</") {
            let end = b[i + 2..].iter().position(|c| *c == b'>').ok_or("unterminated close tag")?;
            let tag = xml[i + 2..i + 2 + end].trim().to_string();
            i += 2 + end + 1;
            stack.pop();
            f(Scan::Close { tag });
            continue;
        }
        // open tag: name + attributes
        let mut j = i + 1;
        while j < b.len() && !(b[j] as char).is_whitespace() && b[j] != b'>' && b[j] != b'/' {
            j += 1;
        }
        let tag = xml[i + 1..j].to_string();
        let mut attrs: Vec<Attr> = Vec::new();
        let mut self_close = false;
        loop {
            while j < b.len() && (b[j] as char).is_whitespace() {
                j += 1;
            }
            if j >= b.len() {
                return Err(format!("unterminated tag <{}", tag));
            }
            if b[j] == b'>' {
                j += 1;
                break;
            }
            if b[j] == b'/' {
                if b.get(j + 1) == Some(&b'>') {
                    self_close = true;
                    j += 2;
                    break;
                }
                j += 1;
                continue;
            }
            let ns = j;
            while j < b.len() && !(b[j] as char).is_whitespace() && b[j] != b'=' && b[j] != b'>' && b[j] != b'/' {
                j += 1;
            }
            let aname = xml[ns..j].to_string();
            while j < b.len() && (b[j] as char).is_whitespace() {
                j += 1;
            }
            let aval = if j < b.len() && b[j] == b'=' {
                j += 1;
                while j < b.len() && (b[j] as char).is_whitespace() {
                    j += 1;
                }
                let v = if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    j += 1;
                    let vs = j;
                    while j < b.len() && b[j] != q {
                        j += 1;
                    }
                    let val = xml[vs..j].to_string();
                    j += 1;
                    val
                } else {
                    let vs = j;
                    while j < b.len() && !(b[j] as char).is_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    xml[vs..j].to_string()
                };
                decode_entities(&v)
            } else {
                String::new()
            };
            attrs.push(Attr { name: aname, value: aval });
        }
        i = j;
        // NOTE: self-closing elements emit Open only — the Close arm pops
        // style/transform frames, and an empty shape/g must not pop theirs.
        f(Scan::Open { tag: tag.clone(), attrs, self_close })?;
        if !self_close {
            stack.push(tag);
        }
    }
    if let Some(unclosed) = stack.last() {
        return Err(format!("unclosed element <{}>", unclosed));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Affine transforms (SVG 2×3, `matrix(a,b,c,d,e,f)` semantics)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Affine {
    const IDENTITY: Affine = Affine { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    fn apply_point(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
            p.z,
        )
    }

    /// Apply only the linear part (offsets — Bézier handles are relative).
    fn apply_offset(&self, h: Vec3) -> Vec3 {
        Vec3::new(self.a * h.x + self.c * h.y, self.b * h.x + self.d * h.y, h.z)
    }

    /// `self` applied after `child` (SVG composition: transform="A B" means
    /// B first, then A — so total = self ∘ child).
    fn then(parent: Affine, child: Affine) -> Affine {
        Affine {
            a: parent.a * child.a + parent.c * child.b,
            b: parent.b * child.a + parent.d * child.b,
            c: parent.a * child.c + parent.c * child.d,
            d: parent.b * child.c + parent.d * child.d,
            e: parent.a * child.e + parent.c * child.f + parent.e,
            f: parent.b * child.e + parent.d * child.f + parent.f,
        }
    }

    fn translate(tx: f64, ty: f64) -> Affine {
        Affine { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: tx, f: ty }
    }

    fn scale(sx: f64, sy: f64) -> Affine {
        Affine { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
    }

    fn rotate(deg: f64) -> Affine {
        let r = deg.to_radians();
        Affine { a: r.cos(), b: r.sin(), c: -r.sin(), d: r.cos(), e: 0.0, f: 0.0 }
    }

    /// Uniform stroke scale factor: SVG scales stroke-width by the transform.
    fn stroke_scale(&self) -> f64 {
        (self.a * self.d - self.b * self.c).abs().sqrt().max(1e-9)
    }
}

fn parse_transform(s: &str) -> Result<Affine, String> {
    let mut total = Affine::IDENTITY;
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        while i < b.len() && !b[i].is_ascii_alphabetic() {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let ns = i;
        while i < b.len() && b[i].is_ascii_alphabetic() {
            i += 1;
        }
        let op = &s[ns..i];
        while i < b.len() && b[i] != b'(' {
            i += 1;
        }
        if i >= b.len() {
            return Err(format!("transform: missing ( after '{}'", op));
        }
        i += 1;
        let vs = i;
        while i < b.len() && b[i] != b')' {
            i += 1;
        }
        let args: Vec<f64> = s[vs..i]
            .split([',', ' ', '\t', '\n'])
            .filter(|t| !t.is_empty())
            .map(|t| t.parse::<f64>().map_err(|e| format!("transform '{}': bad number '{}': {}", op, t, e)))
            .collect::<Result<_, _>>()?;
        if i < b.len() && b[i] == b')' {
            i += 1;
        }
        let one = |want: &str, n: usize, min: usize| -> Result<(), String> {
            if op != want || (args.len() != n && args.len() < min) {
                Err(format!("transform: bad argument count for '{}'", op))
            } else {
                Ok(())
            }
        };
        let aff = match op {
            "translate" => {
                one("translate", 2, 1)?;
                Affine::translate(args[0], *args.get(1).unwrap_or(&0.0))
            }
            "scale" => {
                one("scale", 2, 1)?;
                let sy = *args.get(1).unwrap_or(&args[0]);
                Affine::scale(args[0], sy)
            }
            "rotate" => {
                one("rotate", 3, 1)?;
                let mut a = Affine::rotate(args[0]);
                if args.len() >= 3 {
                    a = Affine::then(Affine::translate(args[1], args[2]), Affine::then(a, Affine::translate(-args[1], -args[2])));
                }
                a
            }
            "matrix" => {
                one("matrix", 6, 6)?;
                Affine { a: args[0], b: args[1], c: args[2], d: args[3], e: args[4], f: args[5] }
            }
            "skewX" => {
                one("skewX", 1, 1)?;
                Affine { a: 1.0, b: 0.0, c: args[0].to_radians().tan(), d: 1.0, e: 0.0, f: 0.0 }
            }
            "skewY" => {
                one("skewY", 1, 1)?;
                Affine { a: 1.0, b: args[0].to_radians().tan(), c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
            }
            other => return Err(format!("transform: unknown op '{}'", other)),
        };
        total = Affine::then(total, aff);
    }
    Ok(total)
}

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Paint {
    Inherit,
    None,
    Color([u8; 4]),
}

fn parse_color(s: &str) -> Option<[u8; 4]> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(hex) = t.strip_prefix('#') {
        let v = |h: &str| -> Option<f64> { u32::from_str_radix(h, 16).ok().map(|n| n as f64) };
        return match hex.len() {
            3 => {
                let r = (v(&hex[0..1])? / 15.0 * 255.0) as u8;
                let g = (v(&hex[1..2])? / 15.0 * 255.0) as u8;
                let b = (v(&hex[2..3])? / 15.0 * 255.0) as u8;
                Some([r, g, b, 255])
            }
            6 => Some([v(&hex[0..2])? as u8, v(&hex[2..4])? as u8, v(&hex[4..6])? as u8, 255]),
            8 => Some([v(&hex[0..2])? as u8, v(&hex[2..4])? as u8, v(&hex[4..6])? as u8, v(&hex[6..8])? as u8]),
            _ => None,
        };
    }
    if let Some(rest) = t.strip_prefix("rgb(").or_else(|| t.strip_prefix("rgb (")) {
        let parts: Vec<f64> = rest
            .trim_end_matches(')')
            .split([',', ' ', '\t'])
            .filter(|p| !p.is_empty())
            .map(|p| p.trim().parse::<f64>().unwrap_or(0.0))
            .collect();
        if parts.len() >= 3 {
            let conv = |x: f64| -> f64 { if x > 1.0 { x } else { x * 255.0 } };
            return Some([conv(parts[0]) as u8, conv(parts[1]) as u8, conv(parts[2]) as u8, 255]);
        }
        return None;
    }
    let named: &[(&str, [u8; 4])] = &[
        ("black", [0, 0, 0, 255]),
        ("white", [255, 255, 255, 255]),
        ("red", [255, 0, 0, 255]),
        ("green", [0, 128, 0, 255]),
        ("lime", [0, 255, 0, 255]),
        ("blue", [0, 0, 255, 255]),
        ("yellow", [255, 255, 0, 255]),
        ("gray", [128, 128, 128, 255]),
        ("grey", [128, 128, 128, 255]),
        ("orange", [255, 165, 0, 255]),
        ("pink", [255, 192, 203, 255]),
        ("purple", [128, 0, 128, 255]),
        ("cyan", [0, 255, 255, 255]),
        ("aqua", [0, 255, 255, 255]),
        ("magenta", [255, 0, 255, 255]),
        ("brown", [165, 42, 42, 255]),
        ("silver", [192, 192, 192, 255]),
        ("navy", [0, 0, 128, 255]),
        ("teal", [0, 128, 128, 255]),
        ("olive", [128, 128, 0, 255]),
        ("maroon", [128, 0, 0, 255]),
        ("gold", [255, 215, 0, 255]),
    ];
    for (n, c) in named {
        if t.eq_ignore_ascii_case(n) {
            return Some(*c);
        }
    }
    None
}

fn attr_num(attrs: &[Attr], name: &str) -> Option<f64> {
    attrs.iter().find(|a| a.name == name).and_then(|a| a.value.trim().parse::<f64>().ok())
}

fn attr_str<'a>(attrs: &'a [Attr], name: &str) -> Option<&'a str> {
    attrs.iter().find(|a| a.name == name).map(|a| a.value.trim())
}

fn parse_paint(v: &str) -> Paint {
    let t = v.trim();
    if t == "none" {
        Paint::None
    } else {
        parse_color(t).map(Paint::Color).unwrap_or(Paint::Inherit)
    }
}

/// Parse the inline `style` attribute ("fill:red;stroke-width:2") into
/// attribute pairs (element presentation attrs are read later and win by
/// order — callers read the LAST occurrence).
fn push_style_attr(attrs: &mut Vec<Attr>, style: &str) {
    for part in style.split(';') {
        if let Some((k, v)) = part.split_once(':') {
            attrs.push(Attr { name: k.trim().to_string(), value: v.trim().to_string() });
        }
    }
}

// ---------------------------------------------------------------------------
// Path data → subpaths of cubic nodes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SubPath {
    /// (position, handle_out, handle_in) — handles are offsets from the node
    /// (mo2d convention: the renderer adds them to the node position).
    pts: Vec<(Vec3, Option<Vec3>, Option<Vec3>)>,
    closed: bool,
}

enum Tok {
    Cmd(char),
    Num(f64),
}

struct PathLexer {
    b: Vec<u8>,
    i: usize,
    peeked: Option<Tok>,
}

impl PathLexer {
    fn new(d: &str) -> Self {
        PathLexer { b: d.as_bytes().to_vec(), i: 0, peeked: None }
    }
    fn skip_sep(&mut self) {
        while self.i < self.b.len()
            && ((self.b[self.i] as char).is_whitespace() || self.b[self.i] == b',')
        {
            self.i += 1;
        }
    }
    fn next(&mut self) -> Option<Tok> {
        if let Some(t) = self.peeked.take() {
            return Some(t);
        }
        self.skip_sep();
        if self.i >= self.b.len() {
            return None;
        }
        let c = self.b[self.i] as char;
        if c.is_ascii_alphabetic() {
            self.i += 1;
            return Some(Tok::Cmd(c));
        }
        // number: starts with digit/'.' or a sign followed by digit/'.'; the
        // sign rule splits "1-2.5+3" into 1, −2.5, +3.
        let start = self.i;
        if c == '-' || c == '+' {
            self.i += 1;
        }
        while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if self.i < self.b.len() && self.b[self.i] == b'.' {
            self.i += 1;
            while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                self.i += 1;
            }
        }
        if self.i < self.b.len() && (self.b[self.i] == b'e' || self.b[self.i] == b'E') {
            let save = self.i;
            self.i += 1;
            if self.i < self.b.len() && (self.b[self.i] == b'-' || self.b[self.i] == b'+') {
                self.i += 1;
            }
            if self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                    self.i += 1;
                }
            } else {
                self.i = save;
            }
        }
        if self.i == start {
            return None; // stray character — stop
        }
        let text = std::str::from_utf8(&self.b[start..self.i]).ok()?;
        text.parse::<f64>().ok().map(Tok::Num)
    }
    fn next_num(&mut self, cmd: char) -> Result<f64, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(n),
            Some(Tok::Cmd(_)) => Err(format!("path '{}': missing numbers", cmd)),
            None => Err(format!("path '{}': unexpected end of data", cmd)),
        }
    }
    fn next_cmd(&mut self) -> Option<char> {
        match self.next() {
            Some(Tok::Cmd(c)) => Some(c),
            Some(Tok::Num(_)) => None, // numbers handled by the caller
            None => None,
        }
    }
}

const KAPPA: f64 = 0.552_284_749_8;

/// Elliptical arc → cubic Béziers (SVG spec appendix: endpoint→center
/// parameterization, ≤90° segments, kappa control offsets). Appends to the
/// subpath: sets the previous node's out-handle and pushes the end node.
fn arc_to_cubics(sub: &mut SubPath, from: Vec3, rx: f64, ry: f64, x_axis_rotation_deg: f64, large_arc: bool, sweep: bool, to: Vec3) {
    if rx.abs() < 1e-9 || ry.abs() < 1e-9 {
        sub.pts.push((to, None, None));
        return;
    }
    let (x1, y1) = (from.x, from.y);
    let (x2, y2) = (to.x, to.y);
    let phi = x_axis_rotation_deg.to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    let dx2 = (x1 - x2) / 2.0;
    let dy2 = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;
    let lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
    let (rx, ry) = if lambda > 1.0 {
        let s = lambda.sqrt();
        (rx * s, ry * s)
    } else {
        (rx, ry)
    };
    let num = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p).max(0.0);
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut co = (num / den).sqrt();
    if large_arc == sweep {
        co = -co;
    }
    let cxp = co * rx * y1p / ry;
    let cyp = -co * ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

    let ang = |ax: f64, ay: f64, bx: f64, by: f64| -> f64 {
        let dot = ax * bx + ay * by;
        let len = (ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt();
        let mut a = (if len > 0.0 { dot / len } else { 0.0 }).clamp(-1.0, 1.0).acos();
        if ax * by - ay * bx < 0.0 {
            a = -a;
        }
        a
    };
    let theta1 = ang(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dtheta = ang((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry);
    if !sweep && dtheta > 0.0 {
        dtheta -= 2.0 * std::f64::consts::PI;
    } else if sweep && dtheta < 0.0 {
        dtheta += 2.0 * std::f64::consts::PI;
    }

    let n = ((dtheta.abs() / std::f64::consts::FRAC_PI_2).ceil() as usize).max(1);
    let delta = dtheta / n as f64;
    let t = 4.0 / 3.0 * (delta / 4.0).tan();
    let pt = |th: f64| -> (f64, f64) {
        (
            cx + rx * th.cos() * cos_phi - ry * th.sin() * sin_phi,
            cy + rx * th.cos() * sin_phi + ry * th.sin() * cos_phi,
        )
    };
    for k in 0..n {
        let a1 = theta1 + delta * k as f64;
        let a2 = theta1 + delta * (k + 1) as f64;
        let (mut ex, mut ey) = pt(a2);
        if k == n - 1 {
            ex = x2;
            ey = y2;
        }
        let (s1x, s1y) = pt(a1);
        let (s2x, s2y) = pt(a2);
        let c1 = (s1x + t * -(s1y - cy), s1y + t * (s1x - cx));
        let c2 = (s2x + t * (s2y - cy), s2y + t * -(s2x - cx));
        let last = sub.pts.last().expect("arc continues an existing subpath").clone();
        sub.pts.last_mut().unwrap().1 = Some(Vec3::new(c1.0 - last.0.x, c1.1 - last.0.y, 0.0));
        sub.pts.push((Vec3::new(ex, ey, 0.0), None, Some(Vec3::new(c2.0 - ex, c2.1 - ey, 0.0))));
    }
}

/// One imported layer: the converted lines + the invisible transform-box
/// line the caller wraps in a `group` relation (the mo2d "层").
pub struct ImportedLayer {
    pub lines: Vec<Line>,
    pub box_line: Line,
}

#[derive(Debug, Clone)]
struct Frame {
    affine: Affine,
    fill: Paint,
    stroke: Paint,
    stroke_width: Option<f64>,
    opacity: f64,
}

fn parse_path_data(d: &str) -> Result<Vec<SubPath>, String> {
    let mut lx = PathLexer::new(d);
    let mut out: Vec<SubPath> = Vec::new();
    let mut sp: Option<SubPath> = None;
    let mut cur = Vec3::ZERO;
    let mut start = Vec3::ZERO;
    let mut prev_c2: Option<Vec3> = None;
    let mut prev_q: Option<Vec3> = None;
    let mut cmd: Option<char> = None;
    let mut nums: Vec<f64> = Vec::new();

    macro_rules! ensure_sub {
        () => {
            if sp.is_none() {
                sp = Some(SubPath { pts: vec![(cur, None, None)], closed: false });
                start = cur;
            }
        };
    }
    macro_rules! finish {
        () => {
            if let Some(s) = sp.take() {
                if s.pts.len() >= 2 {
                    out.push(s);
                }
            }
        };
    }
    let rel_pt = |cur: Vec3, rel: bool, x: f64, y: f64| -> Vec3 {
        if rel {
            Vec3::new(cur.x + x, cur.y + y, 0.0)
        } else {
            Vec3::new(x, y, 0.0)
        }
    };

    loop {
        let Some(tok) = lx.next() else { break };
        let c: char = match tok {
            Tok::Cmd(ch) => {
                nums.clear();
                ch
            }
            Tok::Num(n) => {
                // implicit repeat: an M continues as L, m as l (spec)
                let base = cmd.ok_or("path data starts with a number (no command)")?;
                nums.push(n);
                match base {
                    'M' => 'L',
                    'm' => 'l',
                    other => other,
                }
            }
            _ => return Err("path: unexpected token".into()),
        };
        let upper = c.to_ascii_uppercase();
        let rel = c.is_ascii_lowercase();
        let arity: usize = match upper {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'C' | 'S' => 6,
            'Q' => 4,
            'A' => 7,
            'Z' => 0,
            other => return Err(format!("path: unknown command '{}'", other)),
        };
        while nums.len() < arity {
            match lx.next() {
                Some(Tok::Num(n)) => nums.push(n),
                Some(Tok::Cmd(_)) => return Err(format!("path '{}': missing numbers", c)),
                None => return Err(format!("path '{}': unexpected end of data", c)),
            }
        }
        let inst: Vec<f64> = nums.drain(..arity).collect();
        match upper {
            'M' => {
                finish!();
                let p = rel_pt(cur, rel, inst[0], inst[1]);
                sp = Some(SubPath { pts: vec![(p, None, None)], closed: false });
                cur = p;
                start = p;
                prev_c2 = None;
                prev_q = None;
            }
            'L' => {
                ensure_sub!();
                let p = rel_pt(cur, rel, inst[0], inst[1]);
                sp.as_mut().unwrap().pts.push((p, None, None));
                cur = p;
            }
            'H' => {
                ensure_sub!();
                let x = if rel { cur.x + inst[0] } else { inst[0] };
                let p = Vec3::new(x, cur.y, 0.0);
                sp.as_mut().unwrap().pts.push((p, None, None));
                cur = p;
            }
            'V' => {
                ensure_sub!();
                let y = if rel { cur.y + inst[0] } else { inst[0] };
                let p = Vec3::new(cur.x, y, 0.0);
                sp.as_mut().unwrap().pts.push((p, None, None));
                cur = p;
            }
            'C' => {
                ensure_sub!();
                let c1 = rel_pt(cur, rel, inst[0], inst[1]);
                let c2 = rel_pt(cur, rel, inst[2], inst[3]);
                let p = rel_pt(cur, rel, inst[4], inst[5]);
                let last = sp.as_mut().unwrap().pts.last_mut().unwrap();
                last.1 = Some(Vec3::new(c1.x - cur.x, c1.y - cur.y, 0.0));
                sp.as_mut().unwrap().pts.push((p, None, Some(Vec3::new(c2.x - p.x, c2.y - p.y, 0.0))));
                cur = p;
                prev_c2 = Some(Vec3::new(c2.x, c2.y, 0.0));
            }
            'S' => {
                ensure_sub!();
                let c1 = match prev_c2 {
                    Some(c) => Vec3::new(2.0 * cur.x - c.x, 2.0 * cur.y - c.y, 0.0),
                    None => cur,
                };
                let c2 = rel_pt(cur, rel, inst[0], inst[1]);
                let p = rel_pt(cur, rel, inst[2], inst[3]);
                let last = sp.as_mut().unwrap().pts.last_mut().unwrap();
                last.1 = Some(Vec3::new(c1.x - cur.x, c1.y - cur.y, 0.0));
                sp.as_mut().unwrap().pts.push((p, None, Some(Vec3::new(c2.x - p.x, c2.y - p.y, 0.0))));
                cur = p;
                prev_c2 = Some(Vec3::new(c2.x, c2.y, 0.0));
            }
            'Q' => {
                ensure_sub!();
                let q = rel_pt(cur, rel, inst[0], inst[1]);
                let p = rel_pt(cur, rel, inst[2], inst[3]);
                // elevate the quadratic to a cubic
                let c1 = Vec3::new(cur.x + 2.0 / 3.0 * (q.x - cur.x), cur.y + 2.0 / 3.0 * (q.y - cur.y), 0.0);
                let c2 = Vec3::new(p.x + 2.0 / 3.0 * (q.x - p.x), p.y + 2.0 / 3.0 * (q.y - p.y), 0.0);
                let last = sp.as_mut().unwrap().pts.last_mut().unwrap();
                last.1 = Some(Vec3::new(c1.x - cur.x, c1.y - cur.y, 0.0));
                sp.as_mut().unwrap().pts.push((p, None, Some(Vec3::new(c2.x - p.x, c2.y - p.y, 0.0))));
                cur = p;
                prev_q = Some(q);
            }
            'T' => {
                ensure_sub!();
                let q = match prev_q {
                    Some(q) => Vec3::new(2.0 * cur.x - q.x, 2.0 * cur.y - q.y, 0.0),
                    None => cur,
                };
                let p = rel_pt(cur, rel, inst[0], inst[1]);
                let c1 = Vec3::new(cur.x + 2.0 / 3.0 * (q.x - cur.x), cur.y + 2.0 / 3.0 * (q.y - cur.y), 0.0);
                let c2 = Vec3::new(p.x + 2.0 / 3.0 * (q.x - p.x), p.y + 2.0 / 3.0 * (q.y - p.y), 0.0);
                let last = sp.as_mut().unwrap().pts.last_mut().unwrap();
                last.1 = Some(Vec3::new(c1.x - cur.x, c1.y - cur.y, 0.0));
                sp.as_mut().unwrap().pts.push((p, None, Some(Vec3::new(c2.x - p.x, c2.y - p.y, 0.0))));
                cur = p;
                prev_q = Some(q);
            }
            'A' => {
                ensure_sub!();
                let p = rel_pt(cur, rel, inst[5], inst[6]);
                let sub = sp.as_mut().unwrap();
                arc_to_cubics(sub, cur, inst[0], inst[1], inst[2], inst[3] != 0.0, inst[4] != 0.0, p);
                cur = p;
                prev_c2 = None;
                prev_q = None;
            }
            'Z' => {
                ensure_sub!();
                sp.as_mut().unwrap().closed = true;
                finish!();
                cur = start;
                prev_c2 = None;
                prev_q = None;
            }
            _ => unreachable!(),
        }
    }
    finish!();
    Ok(out)
}

/// Build one mo2d `Line` from a subpath: nodes with handles (cubic
/// convention: seg-type Curve where either end carries a handle), paint from
/// the resolved style, geometry already transformed to world space.
fn subpath_to_line(sub: &SubPath, id: &str, fill_on: bool, fill_color: [u8; 4], stroke_on: bool, stroke_color: [u8; 4], width: f64) -> Line {
    let mut l = Line::new(id);
    l.closed = sub.closed;
    l.fill_on = fill_on;
    l.fill_color = fill_color;
    l.stroke_on = stroke_on;
    l.color = stroke_color;
    l.width = width.max(0.1);
    for (p, h_out, h_in) in &sub.pts {
        let mut n = Node::geom(*p);
        n.handle_out = *h_out;
        n.handle_in = *h_in;
        l.nodes.push(n);
    }
    l.sync_segs();
    for (i, sg) in l.segs.iter_mut().enumerate() {
        let a = &l.nodes[i];
        let b = &l.nodes[(i + 1) % l.nodes.len()];
        if a.handle_out.is_some() || b.handle_in.is_some() {
            sg.seg_type = SegType::Curve;
        }
    }
    l
}

/// Rounded rect → one closed subpath (corner arcs as cubics when rx/ry > 0).
fn rect_subpath(x: f64, y: f64, w: f64, h: f64, rx: f64, ry: f64) -> SubPath {
    let k = KAPPA;
    let (mut rx, mut ry) = (rx.abs().min(w / 2.0), ry.abs().min(h / 2.0));
    if rx == 0.0 || ry == 0.0 {
        rx = 0.0;
        ry = 0.0;
    }
    let mut pts: Vec<(Vec3, Option<Vec3>, Option<Vec3>)> = Vec::new();
    if rx == 0.0 {
        pts.push((Vec3::new(x, y, 0.0), None, None));
        pts.push((Vec3::new(x + w, y, 0.0), None, None));
        pts.push((Vec3::new(x + w, y + h, 0.0), None, None));
        pts.push((Vec3::new(x, y + h, 0.0), None, None));
    } else {
        // clockwise from the top-left corner arc end — handles are
        // node-relative offsets (mo2d convention): corner arcs carry
        // kappa-ry on the vertical axis and kappa-rx on the horizontal axis.
        pts.push((Vec3::new(x + rx, y, 0.0), Some(Vec3::new(k * rx, 0.0, 0.0)), Some(Vec3::new(-k * rx, 0.0, 0.0))));
        pts.push((Vec3::new(x + w, y + ry, 0.0), None, Some(Vec3::new(0.0, -k * ry, 0.0))));
        pts.push((Vec3::new(x + w, y + h - ry, 0.0), Some(Vec3::new(0.0, k * ry, 0.0)), None));
        pts.push((Vec3::new(x + w - rx, y + h, 0.0), None, Some(Vec3::new(-(-k * rx), 0.0, 0.0))));
        pts.push((Vec3::new(x, y + h - ry, 0.0), Some(Vec3::new(0.0, k * ry, 0.0)), None));
        pts.push((Vec3::new(x, y + ry, 0.0), None, Some(Vec3::new(0.0, -k * ry, 0.0))));
        pts.push((Vec3::new(x + rx, y, 0.0), Some(Vec3::new(k * rx, 0.0, 0.0)), None));
        // last node = first node position (closed line joins ends)
        pts[0].2 = Some(Vec3::new(0.0, k * ry, 0.0));
        pts.pop();
    }
    SubPath { pts, closed: true }
}

/// Circle/ellipse → one closed subpath, 4 cubics (kappa handles).
fn ellipse_subpath(cx: f64, cy: f64, rx: f64, ry: f64) -> SubPath {
    let k = KAPPA;
    let kx = rx * k;
    let ky = ry * k;
    // Handles are node-relative offsets (mo2d convention). Traversal is
    // clockwise in SVG's y-down space: right → svg-bottom → left → svg-top.
    // Quarter-circle kappa: right h_out=(0,+ky) toward svg-bottom;
    // svg-bottom h_in=(+kx,0) h_out=(-kx,0); left h_out=(0,-ky);
    // svg-top h_in=(-kx,0) h_out=(+kx,0).
    SubPath {
        pts: vec![
            (Vec3::new(cx + rx, cy, 0.0), Some(Vec3::new(0.0, ky, 0.0)), Some(Vec3::new(0.0, -ky, 0.0))),
            (Vec3::new(cx, cy + ry, 0.0), Some(Vec3::new(-kx, 0.0, 0.0)), Some(Vec3::new(kx, 0.0, 0.0))),
            (Vec3::new(cx - rx, cy, 0.0), Some(Vec3::new(0.0, -ky, 0.0)), Some(Vec3::new(0.0, ky, 0.0))),
            (Vec3::new(cx, cy - ry, 0.0), Some(Vec3::new(kx, 0.0, 0.0)), Some(Vec3::new(-kx, 0.0, 0.0))),
        ],
        closed: true,
    }
}

fn points_list(s: &str) -> Vec<Vec3> {
    s.split([',', ' ', '\t', '\n'])
        .filter(|t| !t.is_empty())
        .map(|t| t.trim().parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>()
        .chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| Vec3::new(c[0], c[1], 0.0))
        .collect()
}

const SHAPE_TAGS: &[&str] = &["path", "rect", "circle", "ellipse", "line", "polygon", "polyline"];

const SKIP_SUBTREES: &[&str] = &[
    "defs", "symbol", "use", "clipPath", "mask", "marker", "pattern", "linearGradient", "radialGradient", "filter", "style",
    "title", "desc", "metadata", "text", "tspan", "image", "foreignObject", "script", "switch",
];

/// Import an SVG document: geometry → editable mo2d lines wrapped as one
/// layer. `scale` scales the artwork (1.0 = SVG user units). The layer box
/// line is returned separately for the caller to wrap in a `group` relation.
pub fn import_svg(xml: &str, name: &str, scale: f64) -> Result<ImportedLayer, String> {
    if !xml.contains('<') {
        return Err("svg.import: not an SVG document".into());
    }
    // Pass 1: document height (root <svg> viewBox/height) for the y-flip.
    let mut h = 100.0f64;
    let mut got_h = false;
    scan_elements(xml, |sc| {
        if !got_h {
            if let Scan::Open { tag, attrs, .. } = sc {
                if tag == "svg" {
                    if let Some(vb) = attr_str(&attrs, "viewBox") {
                        let p: Vec<f64> = vb.split_whitespace().filter_map(|t| t.parse::<f64>().ok()).collect();
                        if p.len() == 4 {
                            h = p[3];
                            got_h = true;
                        }
                    } else if let Some(ht) = attr_num(&attrs, "height") {
                        h = ht;
                        got_h = true;
                    }
                }
            }
        }
        Ok(())
    })?;

    // Root affine: y-down → y-up flip, scaled by `scale`.
    let root = Affine { a: scale, b: 0.0, c: 0.0, d: -scale, e: 0.0, f: h * scale };
    let mut frames = vec![Frame { affine: root, fill: Paint::Inherit, stroke: Paint::Inherit, stroke_width: None, opacity: 1.0 }];

    let mut lines: Vec<Line> = Vec::new();
    let mut counter = 0usize;
    let mut skipping = 0usize;
    let mut bbox: Option<(f64, f64, f64, f64)> = None;

    scan_elements(xml, |sc| match sc {
        Scan::Open { tag, mut attrs, self_close } => {
            if skipping > 0 {
                if !self_close {
                    skipping += 1;
                }
                return Ok(());
            }
            if SKIP_SUBTREES.contains(&tag.as_str()) {
                if !self_close {
                    skipping = 1;
                }
                return Ok(());
            }
            // inline style attr joins the attribute list (attrs win by order)
            if let Some(style) = attr_str(&attrs, "style").map(|s| s.to_string()) {
                push_style_attr(&mut attrs, &style);
            }
            let opacity = attr_num(&attrs, "opacity").unwrap_or(1.0).clamp(0.0, 1.0)
                * attr_num(&attrs, "fill-opacity").unwrap_or(1.0).clamp(0.0, 1.0)
                * attr_num(&attrs, "stroke-opacity").unwrap_or(1.0).clamp(0.0, 1.0);
            let frame = frames.last().unwrap().clone();
            match tag.as_str() {
                "g" => {
                    let aff = match attr_str(&attrs, "transform") {
                        Some(t) => Affine::then(frame.affine, parse_transform(t)?),
                        None => frame.affine,
                    };
                    if !self_close {
                        frames.push(Frame {
                            affine: aff,
                            fill: resolve_paint_val(&frames, &attrs, "fill"),
                            stroke: resolve_paint_val(&frames, &attrs, "stroke"),
                            stroke_width: attr_num(&attrs, "stroke-width").or(frame.stroke_width),
                            opacity: frame.opacity * opacity,
                        });
                    }
                    Ok(())
                }
                "path" | "rect" | "circle" | "ellipse" | "line" | "polygon" | "polyline" => {
                    let aff = match attr_str(&attrs, "transform") {
                        Some(t) => Affine::then(frame.affine, parse_transform(t)?),
                        None => frame.affine,
                    };
                    let (fill_on, mut fill_color) = resolve_paint_from(&frames, &attrs, "fill");
                    let (stroke_on, mut stroke_color) = resolve_paint_from(&frames, &attrs, "stroke");
                    fill_color[3] = (fill_color[3] as f64 * frame.opacity * opacity).round() as u8;
                    stroke_color[3] = (stroke_color[3] as f64 * frame.opacity * opacity).round() as u8;
                    let width = attr_num(&attrs, "stroke-width")
                        .or(frame.stroke_width)
                        .unwrap_or(1.0)
                        * aff.stroke_scale();
                    let mut subpaths: Vec<SubPath> = match tag.as_str() {
                        "path" => {
                            let d = attr_str(&attrs, "d").ok_or("svg.import: <path> missing 'd'")?;
                            parse_path_data(d)?
                        }
                        "rect" => {
                            let (x, y) = (attr_num(&attrs, "x").unwrap_or(0.0), attr_num(&attrs, "y").unwrap_or(0.0));
                            let w = attr_num(&attrs, "width").unwrap_or(0.0);
                            let hh = attr_num(&attrs, "height").unwrap_or(0.0);
                            if w <= 0.0 || hh <= 0.0 {
                                return Ok(());
                            }
                            let rx = attr_num(&attrs, "rx").unwrap_or(0.0);
                            let ry = attr_num(&attrs, "ry").unwrap_or(rx);
                            vec![rect_subpath(x, y, w, hh, rx, if ry == 0.0 && rx != 0.0 { rx } else { ry })]
                        }
                        "circle" => {
                            let (cx, cy) = (attr_num(&attrs, "cx").unwrap_or(0.0), attr_num(&attrs, "cy").unwrap_or(0.0));
                            let r = attr_num(&attrs, "r").unwrap_or(0.0);
                            if r <= 0.0 {
                                return Ok(());
                            }
                            vec![ellipse_subpath(cx, cy, r, r)]
                        }
                        "ellipse" => {
                            let (cx, cy) = (attr_num(&attrs, "cx").unwrap_or(0.0), attr_num(&attrs, "cy").unwrap_or(0.0));
                            let rx = attr_num(&attrs, "rx").unwrap_or(0.0);
                            let ry = attr_num(&attrs, "ry").unwrap_or(0.0);
                            if rx <= 0.0 || ry <= 0.0 {
                                return Ok(());
                            }
                            vec![ellipse_subpath(cx, cy, rx, ry)]
                        }
                        "line" => {
                            let pts = vec![
                                (
                                    Vec3::new(attr_num(&attrs, "x1").unwrap_or(0.0), attr_num(&attrs, "y1").unwrap_or(0.0), 0.0),
                                    None,
                                    None,
                                ),
                                (
                                    Vec3::new(attr_num(&attrs, "x2").unwrap_or(0.0), attr_num(&attrs, "y2").unwrap_or(0.0), 0.0),
                                    None,
                                    None,
                                ),
                            ];
                            vec![SubPath { pts, closed: false }]
                        }
                        "polygon" | "polyline" => {
                            let pts = points_list(attr_str(&attrs, "points").unwrap_or(""))
                                .into_iter()
                                .map(|p| (p, None, None))
                                .collect::<Vec<_>>();
                            if pts.len() < 2 {
                                return Ok(());
                            }
                            vec![SubPath { pts, closed: tag == "polygon" }]
                        }
                        _ => return Ok(()),
                    };
                    for mut sp in subpaths {
                        if sp.pts.len() < 2 {
                            continue;
                        }
                        // transform to world space (points full affine, handles linear-only)
                        for (p, h_out, h_in) in &mut sp.pts {
                            *p = aff.apply_point(*p);
                            *h_out = h_out.map(|h| aff.apply_offset(h));
                            *h_in = h_in.map(|h| aff.apply_offset(h));
                        }
                        counter += 1;
                        let l = subpath_to_line(&sp, &format!("{}_l{}", name, counter), fill_on, fill_color, stroke_on, stroke_color, width);
                        // bbox
                        for n in &l.nodes {
                            let p = match n.pos {
                                NodePos::Geom(g) => g,
                                NodePos::Tick(_) => continue,
                            };
                            bbox = Some(match bbox {
                                None => (p.x, p.y, p.x, p.y),
                                Some((x0, y0, x1, y1)) => (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y)),
                            });
                        }
                        lines.push(l);
                    }
                    Ok(())
                }
                other => {
                    // unknown element: track its subtree for balance, keep
                    // inheritance (children may still carry shapes)
                    if !self_close {
                        frames.push(frames.last().unwrap().clone());
                    }
                    Ok(())
                }
                #[allow(unreachable_patterns)]
                _ => Ok(()),
            }
        }
        Scan::Close { tag } => {
            if skipping > 0 {
                skipping -= 1;
                return Ok(());
            }
            if SKIP_SUBTREES.contains(&tag.as_str()) {
                return Ok(()); // started mid-subtree — already balanced
            }
            // A shape's close tag is a no-op: the shape was fully processed
            // on its Open event (and empty elements emit no Close at all).
            if SHAPE_TAGS.contains(&tag.as_str()) {
                return Ok(());
            }
            if frames.len() > 1 {
                frames.pop();
            }
            Ok(())
        }
    })?;

    if lines.is_empty() {
        return Err("svg.import: no vector geometry found (v1 imports path/shape elements; text/image/gradients are skipped)".into());
    }
    let (x0, y0, x1, y1) = bbox.unwrap_or((0.0, 0.0, 10.0, 10.0));
    let mut box_line = Line::new(&format!("{}_box", name));
    box_line.closed = true;
    box_line.stroke_on = false;
    box_line.fill_on = false;
    box_line.width = 1.0;
    for p in [
        Vec3::new(x0 - 2.0, y0 - 2.0, 0.0),
        Vec3::new(x1 + 2.0, y0 - 2.0, 0.0),
        Vec3::new(x1 + 2.0, y1 + 2.0, 0.0),
        Vec3::new(x0 - 2.0, y1 + 2.0, 0.0),
    ] {
        box_line.nodes.push(Node::geom(p));
    }
    box_line.sync_segs();
    Ok(ImportedLayer { lines, box_line })
}

fn resolve_paint_val(frames: &[Frame], attrs: &[Attr], which: &str) -> Paint {
    // own attribute wins (last occurrence), then frames outward, then inherit
    if let Some(v) = attr_str(attrs, which) {
        return parse_paint(v);
    }
    for f in frames.iter().rev() {
        let p = if which == "fill" { f.fill } else { f.stroke };
        match p {
            Paint::Color(_) | Paint::None => return p,
            Paint::Inherit => {}
        }
    }
    Paint::Inherit
}

fn resolve_paint_from(frames: &[Frame], attrs: &[Attr], which: &str) -> (bool, [u8; 4]) {
    let initial_black = [0, 0, 0, 255];
    match resolve_paint_val(frames, attrs, which) {
        Paint::Color(mut c) => (true, c),
        Paint::None => (false, initial_black),
        // SVG initial values: fill = black, stroke = none
        Paint::Inherit => {
            if which == "fill" {
                (true, initial_black)
            } else {
                (false, initial_black)
            }
        }
    }
}
