//! Minimal JSON support: just enough for scenario and answer documents.
//!
//! Values are parsed into a tree of `Json` nodes. Serialization of answer
//! maps keeps insertion order via a `Vec<(String, Json)>`, so output is
//! byte-stable across runs.

use std::collections::BTreeMap;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(v) => Some(v),
            _ => None,
        }
    }
}

pub fn parse(input: &str) -> Result<Json, String> {
    let bytes: Vec<char> = input.chars().collect();
    let mut pos = 0usize;
    let value = parse_value(&bytes, &mut pos)?;
    skip_ws(&bytes, &mut pos);
    if pos != bytes.len() {
        return Err(format!("trailing characters at offset {pos}"));
    }
    Ok(value)
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn parse_value(chars: &[char], pos: &mut usize) -> Result<Json, String> {
    skip_ws(chars, pos);
    if *pos >= chars.len() {
        return Err("unexpected end of input".into());
    }
    match chars[*pos] {
        '{' => parse_obj(chars, pos),
        '[' => parse_arr(chars, pos),
        '"' => parse_string(chars, pos).map(Json::Str),
        't' if chars[*pos..].starts_with(&['t', 'r', 'u', 'e']) => {
            *pos += 4;
            Ok(Json::Bool(true))
        }
        'f' if chars[*pos..].starts_with(&['f', 'a', 'l', 's', 'e']) => {
            *pos += 5;
            Ok(Json::Bool(false))
        }
        'n' if chars[*pos..].starts_with(&['n', 'u', 'l', 'l']) => {
            *pos += 4;
            Ok(Json::Null)
        }
        _ => parse_number(chars, pos),
    }
}

fn parse_obj(chars: &[char], pos: &mut usize) -> Result<Json, String> {
    *pos += 1; // '{'
    let mut map = BTreeMap::new();
    skip_ws(chars, pos);
    if *pos < chars.len() && chars[*pos] == '}' {
        *pos += 1;
        return Ok(Json::Obj(map));
    }
    loop {
        skip_ws(chars, pos);
        let key = parse_string(chars, pos)?;
        skip_ws(chars, pos);
        if *pos >= chars.len() || chars[*pos] != ':' {
            return Err(format!("expected ':' at offset {pos}"));
        }
        *pos += 1;
        let value = parse_value(chars, pos)?;
        map.insert(key, value);
        skip_ws(chars, pos);
        match chars.get(*pos) {
            Some(',') => *pos += 1,
            Some('}') => {
                *pos += 1;
                return Ok(Json::Obj(map));
            }
            _ => return Err(format!("expected ',' or '}}' at offset {pos}")),
        }
    }
}

fn parse_arr(chars: &[char], pos: &mut usize) -> Result<Json, String> {
    *pos += 1; // '['
    let mut items = Vec::new();
    skip_ws(chars, pos);
    if *pos < chars.len() && chars[*pos] == ']' {
        *pos += 1;
        return Ok(Json::Arr(items));
    }
    loop {
        let value = parse_value(chars, pos)?;
        items.push(value);
        skip_ws(chars, pos);
        match chars.get(*pos) {
            Some(',') => *pos += 1,
            Some(']') => {
                *pos += 1;
                return Ok(Json::Arr(items));
            }
            _ => return Err(format!("expected ',' or ']' at offset {pos}")),
        }
    }
}

fn parse_string(chars: &[char], pos: &mut usize) -> Result<String, String> {
    if *pos >= chars.len() || chars[*pos] != '"' {
        return Err(format!("expected string at offset {pos}"));
    }
    *pos += 1;
    let mut out = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        *pos += 1;
        match c {
            '"' => return Ok(out),
            '\\' => {
                let esc = chars.get(*pos).copied().unwrap_or('?');
                *pos += 1;
                match esc {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    other => out.push(other),
                }
            }
            _ => out.push(c),
        }
    }
    Err("unterminated string".into())
}

fn parse_number(chars: &[char], pos: &mut usize) -> Result<Json, String> {
    let start = *pos;
    if chars.get(*pos) == Some(&'-') {
        *pos += 1;
    }
    while *pos < chars.len()
        && (chars[*pos].is_ascii_digit()
            || chars[*pos] == '.'
            || chars[*pos] == 'e'
            || chars[*pos] == 'E'
            || chars[*pos] == '+')
    {
        *pos += 1;
    }
    let text: String = chars[start..*pos].iter().collect();
    text.parse::<f64>()
        .map(Json::Num)
        .map_err(|_| format!("invalid number {text:?}"))
}

/// Format an f64 the way the scenario corpus stores numbers: plain decimal,
/// shortest round-trip representation produced by `serde`-style formatting.
pub fn fmt_f64(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        let s = format!("{}", v as i64);
        return s;
    }
    let mut s = format!("{v}");
    if s.contains('e') {
        s = format!("{v:.10}");
    }
    s.trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// Serialize a scalar/array/object tree with 2-space indent.
pub fn dump(root: &Json) -> String {
    let mut out = String::new();
    write_node(&mut out, root, 0);
    out.push('\n');
    out
}

fn write_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_node(out: &mut String, node: &Json, depth: usize) {
    match node {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => {
            let _ = write!(out, "{b}");
        }
        Json::Num(v) => out.push_str(&fmt_f64(*v)),
        Json::Str(s) => write_json_string(out, s),
        Json::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                write_indent(out, depth + 1);
                write_node(out, item, depth + 1);
                if i + 1 != items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            write_indent(out, depth);
            out.push(']');
        }
        Json::Obj(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            let last = map.len() - 1;
            for (i, (k, v)) in map.iter().enumerate() {
                write_indent(out, depth + 1);
                write_json_string(out, k);
                out.push_str(": ");
                write_node(out, v, depth + 1);
                if i != last {
                    out.push(',');
                }
                out.push('\n');
            }
            write_indent(out, depth);
            out.push('}');
        }
    }
}

fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
}
