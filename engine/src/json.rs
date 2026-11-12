//! Minimal JSON value + parser + canonical serializer.
//!
//! Design goals (driven by the byte-determinism requirement, file-format.md §2.4):
//! - Object key order is **insertion order** (`Vec<(String, Value)>`), never sorted:
//!   the document writer constructs objects in canonical field order and the parser
//!   preserves what it read, so round-trips are stable.
//! - Integers and floats are distinct variants: ticks are `i64` and must never
//!   drift through `f64`.
//! - `Display for f64` in Rust is the shortest round-trip form and never uses
//!   exponents, which makes the canonical textual form deterministic.

use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn obj() -> Value {
        Value::Obj(Vec::new())
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
            Value::Num(n) if n.fract() == 0.0 && n.is_finite() && *n >= -(2f64.powi(53)) && *n <= 2f64.powi(53) => Some(*n as i64),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Num(n) if n.is_finite() => Some(*n),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_obj(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Obj(o) => Some(o),
            _ => None,
        }
    }
    /// Object field lookup (first match).
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    /// Insert or replace a key, preserving position when replacing.
    pub fn set(&mut self, key: &str, val: Value) {
        if let Value::Obj(o) = self {
            for (k, v) in o.iter_mut() {
                if k == key {
                    *v = val;
                    return;
                }
            }
            o.push((key.to_string(), val));
        }
    }
    /// Type name for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "integer",
            Value::Num(_) => "number",
            Value::Str(_) => "string",
            Value::Arr(_) => "array",
            Value::Obj(_) => "object",
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical serializer
// ---------------------------------------------------------------------------

/// Canonical JSON text: 2-space indent, one array element per line,
/// floats always carry a decimal point, string escapes are the JSON minimum.
/// NaN/Infinity are rejected (they would break determinism guarantees).
pub fn to_string(v: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_value(&mut out, v, 0)?;
    Ok(out)
}

fn write_value(out: &mut String, v: &Value, indent: usize) -> Result<(), String> {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => {
            let _ = write!(out, "{}", i);
        }
        Value::Num(n) => {
            if !n.is_finite() {
                return Err(format!("non-finite number {} cannot be serialized", n));
            }
            let s = format!("{}", n);
            out.push_str(&s);
            // Keep the float-ness visible so a round-trip stays a float.
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                out.push_str(".0");
            }
        }
        Value::Str(s) => write_string(out, s),
        Value::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                push_indent(out, indent + 1);
                write_value(out, item, indent + 1)?;
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent);
            out.push(']');
        }
        Value::Obj(fields) => {
            if fields.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            out.push_str("{\n");
            for (i, (k, val)) in fields.iter().enumerate() {
                push_indent(out, indent + 1);
                write_string(out, k);
                out.push_str(": ");
                write_value(out, val, indent + 1)?;
                if i + 1 < fields.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent);
            out.push('}');
        }
    }
    Ok(())
}

/// Compact form for wire messages (MCP stdio is newline-delimited — one
/// message per line, so pretty-printed multi-line JSON must not be used).
pub fn to_string_compact(v: &Value) -> Result<String, String> {
    let mut out = String::new();
    write_compact(&mut out, v)?;
    Ok(out)
}

fn write_compact(out: &mut String, v: &Value) -> Result<(), String> {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => {
            let _ = write!(out, "{}", i);
        }
        Value::Num(n) => {
            if !n.is_finite() {
                return Err(format!("non-finite number {} cannot be serialized", n));
            }
            let s = format!("{}", n);
            out.push_str(&s);
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                out.push_str(".0");
            }
        }
        Value::Str(s) => write_string(out, s),
        Value::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact(out, item)?;
            }
            out.push(']');
        }
        Value::Obj(fields) => {
            out.push('{');
            for (i, (k, val)) in fields.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, k);
                out.push(':');
                write_compact(out, val)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn push_indent(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push_str("  ");
    }
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

pub fn parse(text: &str) -> Result<Value, String> {
    let bytes = text.as_bytes();
    let mut p = Parser { bytes, pos: 0 };
    p.skip_ws();
    let v = p.value(0)?;
    p.skip_ws();
    if p.pos != bytes.len() {
        return Err(p.err("trailing characters after JSON value"));
    }
    Ok(v)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

const MAX_DEPTH: usize = 128;

impl<'a> Parser<'a> {
    fn err(&self, msg: &str) -> String {
        let mut line = 1usize;
        let mut col = 1usize;
        for &b in &self.bytes[..self.pos.min(self.bytes.len())] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        format!("JSON parse error at line {} col {}: {}", line, col, msg)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, b: u8) -> Result<(), String> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(&format!("expected '{}'", b as char)))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err(self.err("nesting too deep"));
        }
        match self.peek() {
            None => Err(self.err("unexpected end of input")),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Value::Str(self.string()?)),
            Some(b't') => self.lit("true", Value::Bool(true)),
            Some(b'f') => self.lit("false", Value::Bool(false)),
            Some(b'n') => self.lit("null", Value::Null),
            Some(b'-') | Some(b'0'..=b'9') => self.number(),
            Some(c) => Err(self.err(&format!("unexpected character '{}'", c as char))),
        }
    }

    fn lit(&mut self, word: &str, v: Value) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(self.err(&format!("invalid literal, expected '{}'", word)))
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, String> {
        self.expect(b'{')?;
        let mut fields = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Obj(fields));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let val = self.value(depth + 1)?;
            fields.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Obj(fields));
                }
                _ => return Err(self.err("expected ',' or '}' in object")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Arr(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value(depth + 1)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Arr(items));
                }
                _ => return Err(self.err("expected ',' or ']' in array")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut s = String::new();
        loop {
            let start = self.pos;
            // Fast path over plain bytes.
            while let Some(b) = self.peek() {
                if b == b'"' || b == b'\\' || b < 0x20 {
                    break;
                }
                self.pos += 1;
            }
            if self.pos > start {
                s.push_str(std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| self.err("invalid UTF-8 in string"))?);
            }
            match self.peek() {
                None => return Err(self.err("unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(s);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => { s.push('"'); self.pos += 1; }
                        Some(b'\\') => { s.push('\\'); self.pos += 1; }
                        Some(b'/') => { s.push('/'); self.pos += 1; }
                        Some(b'n') => { s.push('\n'); self.pos += 1; }
                        Some(b't') => { s.push('\t'); self.pos += 1; }
                        Some(b'r') => { s.push('\r'); self.pos += 1; }
                        Some(b'b') => { s.push('\u{08}'); self.pos += 1; }
                        Some(b'f') => { s.push('\u{0C}'); self.pos += 1; }
                        Some(b'u') => {
                            self.pos += 1;
                            let cp = self.hex4()?;
                            let ch = if (0xD800..0xDC00).contains(&cp) {
                                // Surrogate pair.
                                if self.peek() != Some(b'\\') {
                                    return Err(self.err("unpaired surrogate"));
                                }
                                self.pos += 1;
                                if self.peek() != Some(b'u') {
                                    return Err(self.err("unpaired surrogate"));
                                }
                                self.pos += 1;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(self.err("invalid low surrogate"));
                                }
                                let c = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                char::from_u32(c).ok_or_else(|| self.err("invalid surrogate pair"))?
                            } else {
                                char::from_u32(cp).ok_or_else(|| self.err("invalid unicode escape"))?
                            };
                            s.push(ch);
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                }
                Some(_) => return Err(self.err("control character in string")),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        for _ in 0..4 {
            let b = self.peek().ok_or_else(|| self.err("truncated \\u escape"))?;
            let d = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                _ => return Err(self.err("invalid hex digit in \\u escape")),
            };
            v = v * 16 + d;
            self.pos += 1;
        }
        Ok(v)
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| self.err("invalid number"))?;
        if text.is_empty() || text == "-" {
            return Err(self.err("invalid number"));
        }
        if !is_float {
            if let Ok(i) = text.parse::<i64>() {
                return Ok(Value::Int(i));
            }
        }
        let f: f64 = text.parse::<f64>().map_err(|_| self.err("invalid number"))?;
        if !f.is_finite() {
            return Err(self.err("number out of range"));
        }
        Ok(Value::Num(f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let src = r#"{"a":1,"b":[1.5,2],"c":{"d":true,"e":null},"f":"x\ny"}"#;
        let v = parse(src).unwrap();
        let text = to_string(&v).unwrap();
        let v2 = parse(&text).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn float_keeps_point() {
        let v = Value::Arr(vec![Value::Num(1.0), Value::Num(0.5), Value::Num(-3.0)]);
        let s = to_string(&v).unwrap();
        assert_eq!(s, "[\n  1.0,\n  0.5,\n  -3.0\n]");
    }

    #[test]
    fn int_stays_int() {
        let v = parse("254016000000").unwrap();
        assert_eq!(v, Value::Int(254016000000));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("{").is_err());
        assert!(parse("[1,]").is_err());
        assert!(parse("01a").is_err());
        assert!(parse("\"\\q\"").is_err());
    }
}
