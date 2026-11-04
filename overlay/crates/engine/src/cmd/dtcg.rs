//! FigCraft Plus: design tokens in the W3C Design Tokens Community Group format (DTCG JSON).
//!
//! - `variable.exportDtcg {collections?, path?}` writes this file's variables as DTCG tokens:
//!   each collection a top-level group, each `/` in a name a nested group, `$type` and `$value`
//!   per token. Colours are hex (`#RRGGBB`, `#RRGGBBAA`); lengths (sizes, spacing, radii, type
//!   sizes, strokes) `dimension` in px (`{"value": 16, "unit": "px"}`); weights and opacities
//!   `number`; text `fontFamily` when it is a font, else plain strings; aliases `{group.token}`.
//!   The first mode is `$value`; every mode is kept in `$extensions["figcraft.modes"]`, and a
//!   Plus formula in `$extensions["figcraft.formula"]`.
//! - `variable.importDtcg {json (text or a path), collection?}` reads DTCG tokens (and the older
//!   `value` / `type` form): groups become `/` in names, top-level groups collections (or all go to
//!   `collection`), `$type` is inherited from groups, `{a.b}` references become aliases,
//!   `$extensions["figcraft.modes"]` modes. Colours (hex, CSS colours, `{colorSpace, components}`),
//!   dimensions (px, rem of 16), numbers, font weights, durations and font families are read; other
//!   composite types (shadow, typography, gradient…) are skipped with a warning.

use std::collections::{BTreeMap, BTreeSet};

use figcraft_color::Color;
use figcraft_doc::{Collection, Document, Mode, ModeId, Variable, VariableId, VariableType, VariableValue};
use serde_json::{Map, Value, json};

use super::CommandSpec;
use crate::library::{is_remote_collection, local_collections};
use crate::params::{get, string};
use crate::token_roles::{NumberRole, Roles};
use crate::{EngineError, Result, Session, bad, spec};

const MODES: &str = "figcraft.modes";
const FORMULA: &str = "figcraft.formula";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "variable.exportDtcg",
            "Export design tokens (DTCG)",
            "{collections? [ids or names; default every collection of this file], path? (write it there)} → {text} (DTCG JSON: collections as top-level groups, $type / $value, aliases as {a.b}, every mode in $extensions[\"figcraft.modes\"]), or {path}",
            false,
            export
        ),
        spec!(
            "variable.importDtcg",
            "Import design tokens (DTCG)",
            "{json (DTCG JSON text, or a path to a .json file), collection? (name: every token goes there; default: each top-level group is a collection)} → {variables (added), updated, modes, warnings}",
            true,
            import
        ),
    ]
}

// ---- export ---------------------------------------------------------------------------------------

fn hex(c: Color) -> String {
    let [r, g, b, a] = c.sanitized().to_rgba8();
    if a == 255 { format!("#{r:02X}{g:02X}{b:02X}") } else { format!("#{r:02X}{g:02X}{b:02X}{a:02X}") }
}

/// A token path segment: a name part with the characters DTCG reserves (`.{}$`) replaced.
fn segment(s: &str) -> String {
    let t: String = s.trim().chars().map(|c| if matches!(c, '.' | '{' | '}') { '-' } else { c }).collect();
    t.trim_start_matches('$').to_string()
}

/// `{Collection.group.token}` for variable `v`.
fn reference(doc: &Document, v: &Variable) -> String {
    let coll = doc.variables.collections.iter().find(|c| c.id == v.collection).map(|c| c.name.as_str()).unwrap_or("Tokens");
    let mut parts = vec![segment(coll)];
    parts.extend(v.name.split('/').map(segment));
    format!("{{{}}}", parts.join("."))
}

fn is_font(v: &Variable) -> bool {
    v.scopes.as_ref().is_some_and(|s| s.iter().any(|x| x == "FONT_FAMILY")) || {
        let n = v.name.to_ascii_lowercase();
        n.contains("font") || n.contains("family") || n.contains("typeface")
    }
}

/// The DTCG `$type` of `v`, None for a token with no type of its own (text, booleans).
fn dtcg_type(v: &Variable, roles: &Roles) -> Option<&'static str> {
    match v.kind {
        VariableType::Color => Some("color"),
        VariableType::Number => {
            let n = v.name.to_ascii_lowercase();
            if v.scopes.as_ref().is_some_and(|s| s.iter().any(|x| x == "FONT_WEIGHT")) || n.contains("weight") {
                Some("fontWeight")
            } else if roles.role(v) == NumberRole::Unitless {
                Some("number")
            } else {
                Some("dimension")
            }
        }
        VariableType::String if is_font(v) => Some("fontFamily"),
        _ => None,
    }
}

/// A value as DTCG writes it for a token of type `ty`.
fn value_json(doc: &Document, ty: Option<&str>, value: &VariableValue) -> Option<Value> {
    Some(match value {
        VariableValue::Alias(id) => json!(reference(doc, doc.variables.get_by_id(id)?)),
        VariableValue::Color(c) => json!(hex(*c)),
        VariableValue::Number(x) if ty == Some("dimension") => json!({"value": round(*x), "unit": "px"}),
        VariableValue::Number(x) => json!(round(*x)),
        VariableValue::String(s) => json!(s),
        VariableValue::Boolean(b) => json!(b),
        _ => return None,
    })
}

fn round(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

/// The tokens of `colls` as a DTCG document.
pub fn dtcg(doc: &Document, colls: &[&Collection]) -> Value {
    let roles = Roles::new(doc);
    let mut root = Map::new();
    for c in colls {
        let mut group = Map::new();
        let default = c.modes.first().map(|m| m.id.clone());
        for v in doc.variables.variables.iter().filter(|v| v.collection == c.id) {
            let ty = dtcg_type(v, &roles);
            let modes_map: BTreeMap<_, _> = default.iter().map(|m| (c.id.clone(), m.clone())).collect();
            // The first mode's value: an alias as a reference, a calculation as what it gives.
            let first = default.as_ref().and_then(|m| v.values.get(m));
            let value = match first {
                Some(VariableValue::Alias(_)) => first.and_then(|x| value_json(doc, ty, x)),
                _ => doc.variables.resolve(&v.id, &modes_map).and_then(|x| value_json(doc, ty, &x)),
            };
            let Some(value) = value else { continue };
            let mut token = Map::new();
            if let Some(t) = ty {
                token.insert("$type".into(), json!(t));
            }
            token.insert("$value".into(), value);
            let description: String = v.description.lines().filter(|l| !l.contains('{') && !l.contains("oklch(") && !l.contains("calc(")).collect::<Vec<_>>().join("\n");
            if !description.trim().is_empty() {
                token.insert("$description".into(), json!(description.trim()));
            }
            let mut ext = Map::new();
            if c.modes.len() > 1 {
                let mut per = Map::new();
                for m in &c.modes {
                    let modes: BTreeMap<_, _> = [(c.id.clone(), m.id.clone())].into_iter().collect();
                    let x = match v.values.get(&m.id) {
                        Some(a @ VariableValue::Alias(_)) => value_json(doc, ty, a),
                        _ => doc.variables.resolve(&v.id, &modes).and_then(|x| value_json(doc, ty, &x)),
                    };
                    if let Some(x) = x {
                        per.insert(m.name.clone(), x);
                    }
                }
                ext.insert(MODES.into(), Value::Object(per));
            }
            if let Some(f) = v.description.lines().find(|l| l.contains("calc(") || l.contains("oklch(") || l.contains("color-mix(")) {
                ext.insert(FORMULA.into(), json!(f.trim()));
            }
            if !ext.is_empty() {
                token.insert("$extensions".into(), Value::Object(ext));
            }
            // Into its groups.
            let parts: Vec<String> = v.name.split('/').map(segment).filter(|s| !s.is_empty()).collect();
            let Some((last, path)) = parts.split_last() else { continue };
            let mut at = &mut group;
            for p in path {
                let next = at.entry(p.clone()).or_insert_with(|| Value::Object(Map::new()));
                if !next.is_object() || next.get("$value").is_some() {
                    // A token and a group of one name: the group wins the name, the token keeps `_`.
                    *next = Value::Object(Map::new());
                }
                let Value::Object(m) = next else { break };
                at = m;
            }
            at.insert(last.clone(), Value::Object(token));
        }
        if c.modes.len() > 1 {
            group.insert("$extensions".into(), json!({MODES: c.modes.iter().map(|m| m.name.clone()).collect::<Vec<_>>()}));
        }
        root.insert(segment(&c.name), Value::Object(group));
    }
    Value::Object(root)
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "variable.exportDtcg";
    let doc = s.doc()?;
    let colls: Vec<&Collection> = match get(p, "collections") {
        None | Some(Value::Null) => local_collections(doc).collect(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| {
                let k = x.as_str().unwrap_or_default();
                doc.variables.collections.iter().find(|c| c.id == k || c.name == k).ok_or_else(|| bad(cmd, format!("no collection {k}")))
            })
            .collect::<Result<_>>()?,
        Some(_) => return Err(bad(cmd, "collections: [collection ids or names]")),
    };
    let text = serde_json::to_string_pretty(&dtcg(doc, &colls)).map_err(|e| EngineError::Other(e.to_string()))?;
    match string(p, "path") {
        Some(path) => {
            super::file::write(&path, text.as_bytes())?;
            Ok(json!({"path": path}))
        }
        None => Ok(json!({"text": text})),
    }
}

// ---- import ---------------------------------------------------------------------------------------

/// A token read: its path (collection first), type, value per mode name (None: the first mode).
struct Token {
    path: Vec<String>,
    ty: Option<String>,
    values: Vec<(Option<String>, Value)>,
    description: String,
}

fn walk(v: &Map<String, Value>, path: &mut Vec<String>, inherited: Option<String>, out: &mut Vec<Token>) {
    let ty = v.get("$type").or_else(|| v.get("type")).and_then(Value::as_str).map(str::to_string).or(inherited);
    let value = v.get("$value").or_else(|| v.get("value").filter(|_| !v.contains_key("$value")));
    if let Some(value) = value {
        // A token.
        let mut values = vec![(None, value.clone())];
        if let Some(Value::Object(modes)) = v.get("$extensions").and_then(|e| e.get(MODES)) {
            values.extend(modes.iter().map(|(k, x)| (Some(k.clone()), x.clone())));
        }
        let description = v.get("$description").or_else(|| v.get("description")).and_then(Value::as_str).unwrap_or_default().to_string();
        out.push(Token { path: path.clone(), ty, values, description });
        return;
    }
    for (k, child) in v {
        if k.starts_with('$') {
            continue;
        }
        if let Value::Object(m) = child {
            path.push(k.clone());
            walk(m, path, ty.clone(), out);
            path.pop();
        }
    }
}

/// What a raw value becomes.
enum Read {
    Value(VariableType, VariableValue),
    Ref(Vec<String>),
}

fn reference_path(s: &str) -> Option<Vec<String>> {
    let inner = s.trim().strip_prefix('{')?.strip_suffix('}')?;
    Some(inner.split('.').map(|p| p.trim().to_string()).collect())
}

fn dimension_px(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::Object(o) => {
            let x = o.get("value")?.as_f64()?;
            match o.get("unit").and_then(Value::as_str).unwrap_or("px") {
                "px" => Some(x),
                "rem" | "em" => Some(x * 16.0),
                _ => None,
            }
        }
        Value::String(s) => {
            let t = s.trim();
            if let Some(n) = t.strip_suffix("px") {
                n.trim().parse().ok()
            } else if let Some(n) = t.strip_suffix("rem").or_else(|| t.strip_suffix("em")) {
                n.trim().parse::<f64>().ok().map(|x| x * 16.0)
            } else {
                t.parse().ok()
            }
        }
        _ => None,
    }
}

fn color_of(v: &Value) -> Option<Color> {
    match v {
        Value::String(s) => Color::parse_css(s),
        Value::Object(o) => {
            if let Some(h) = o.get("hex").and_then(Value::as_str) {
                let mut c = Color::from_hex(h)?;
                if let Some(a) = o.get("alpha").and_then(Value::as_f64) {
                    c.a = a as f32;
                }
                return Some(c);
            }
            let space = o.get("colorSpace").and_then(Value::as_str).unwrap_or("srgb");
            let comps: Vec<f64> = o.get("components")?.as_array()?.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect();
            let a = o.get("alpha").and_then(Value::as_f64).unwrap_or(1.0) as f32;
            match (space, comps.as_slice()) {
                ("srgb", [r, g, b]) => Some(Color::new(*r as f32, *g as f32, *b as f32, a)),
                ("oklch", [l, c, h]) => Some(figcraft_color::oklch_profile::Oklch::new(*l, *c, *h).to_color(figcraft_color::ColorProfile::Srgb, a)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn read(ty: Option<&str>, v: &Value) -> Option<Read> {
    if let Some(r) = v.as_str().and_then(reference_path) {
        return Some(Read::Ref(r));
    }
    let n = |x: Option<f64>| x.filter(|x| x.is_finite()).map(|x| Read::Value(VariableType::Number, VariableValue::Number(x)));
    match ty {
        Some("color") => color_of(v).map(|c| Read::Value(VariableType::Color, VariableValue::Color(c))),
        Some("dimension") => n(dimension_px(v)),
        Some("number") => n(v.as_f64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))),
        Some("fontWeight") => n(v.as_f64().or_else(|| {
            let s = v.as_str()?.to_ascii_lowercase();
            Some(match s.as_str() {
                "thin" | "hairline" => 100.0,
                "extra-light" | "ultra-light" => 200.0,
                "light" => 300.0,
                "normal" | "regular" | "book" => 400.0,
                "medium" => 500.0,
                "semi-bold" | "demi-bold" => 600.0,
                "bold" => 700.0,
                "extra-bold" | "ultra-bold" => 800.0,
                "black" | "heavy" => 900.0,
                _ => return None,
            })
        })),
        Some("duration") => n(dimension_ms(v)),
        Some("fontFamily") => match v {
            Value::String(s) => Some(Read::Value(VariableType::String, VariableValue::String(s.clone()))),
            Value::Array(a) => a.first().and_then(Value::as_str).map(|s| Read::Value(VariableType::String, VariableValue::String(s.to_string()))),
            _ => None,
        },
        Some(_) => None,
        // No type: what the value says.
        None => match v {
            Value::Bool(b) => Some(Read::Value(VariableType::Boolean, VariableValue::Boolean(*b))),
            Value::Number(x) => n(x.as_f64()),
            Value::String(s) => match Color::parse_css(s) {
                Some(c) if s.trim().starts_with('#') || s.contains('(') => Some(Read::Value(VariableType::Color, VariableValue::Color(c))),
                _ => match dimension_px(v) {
                    Some(x) if s.trim().ends_with("px") || s.trim().ends_with("rem") => n(Some(x)),
                    _ => Some(Read::Value(VariableType::String, VariableValue::String(s.clone()))),
                },
            },
            Value::Object(_) => color_of(v).map(|c| Read::Value(VariableType::Color, VariableValue::Color(c))).or_else(|| n(dimension_px(v))),
            _ => None,
        },
    }
}

fn dimension_ms(v: &Value) -> Option<f64> {
    match v {
        Value::Object(o) => {
            let x = o.get("value")?.as_f64()?;
            Some(if o.get("unit").and_then(Value::as_str) == Some("s") { x * 1000.0 } else { x })
        }
        Value::String(s) => {
            let t = s.trim();
            t.strip_suffix("ms").and_then(|n| n.trim().parse().ok()).or_else(|| t.strip_suffix('s').and_then(|n| n.trim().parse::<f64>().ok()).map(|x| x * 1000.0))
        }
        Value::Number(x) => x.as_f64(),
        _ => None,
    }
}

fn read_json(cmd: &str, p: &Value) -> Result<Value> {
    let raw = match get(p, "json") {
        Some(Value::String(t)) => t.clone(),
        Some(v @ Value::Object(_)) => return Ok(v.clone()),
        _ => return Err(bad(cmd, "json: DTCG JSON text, or a path")),
    };
    let text = if raw.trim_start().starts_with('{') {
        raw
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let meta = std::fs::metadata(&raw).map_err(|e| bad(cmd, format!("{raw}: {e}")))?;
            if meta.len() > 32 * 1024 * 1024 {
                return Err(bad(cmd, "the file is over 32 MiB"));
            }
            std::fs::read_to_string(&raw).map_err(|e| EngineError::Other(format!("{raw}: {e}")))?
        }
        #[cfg(target_arch = "wasm32")]
        return Err(bad(cmd, "json: the tokens' text"));
    };
    serde_json::from_str(&text).map_err(|e| bad(cmd, format!("not JSON: {e}")))
}

fn import(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "variable.importDtcg";
    let json = read_json(cmd, p)?;
    let Value::Object(root) = &json else { return Err(bad(cmd, "a DTCG document is a JSON object")) };
    let into = string(p, "collection").map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    let mut tokens = Vec::new();
    walk(root, &mut Vec::new(), None, &mut tokens);
    let doc = s.doc_mut()?;
    let mut warnings: Vec<String> = Vec::new();
    // Which collection and which name each token goes to.
    let place = |path: &[String]| -> (String, String) {
        match (&into, path) {
            (Some(c), _) => (c.clone(), path.join("/")),
            (None, [one]) => ("Tokens".into(), one.clone()),
            (None, [first, rest @ ..]) => (first.clone(), rest.join("/")),
            (None, []) => ("Tokens".into(), String::new()),
        }
    };
    // Mode names per collection, in first-seen order (the first mode is "Default" unless named).
    let mut coll_modes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in &tokens {
        let (c, _) = place(&t.path);
        let e = coll_modes.entry(c).or_default();
        for (m, _) in &t.values {
            if let Some(m) = m
                && !e.contains(m)
            {
                e.push(m.clone());
            }
        }
    }
    // Collections and modes.
    let mut coll_ids: BTreeMap<String, figcraft_doc::CollectionId> = BTreeMap::new();
    let mut mode_ids: BTreeMap<(String, Option<String>), ModeId> = BTreeMap::new();
    let mut added_modes = 0usize;
    for (cname, modes) in &coll_modes {
        let cid = match doc.variables.collections.iter().find(|c| c.name == *cname && c.extends.is_none() && !is_remote_collection(doc, &c.id)) {
            Some(c) => c.id.clone(),
            None => {
                let id = doc.new_collection_id();
                let name = super::variables::free_collection_name(doc, cname);
                doc.variables.collections.push(Collection { id: id.clone(), name, modes: Vec::new(), extends: None });
                let first = doc.new_mode_id(&id).ok_or_else(|| EngineError::Other("no mode id".into()))?;
                let first_name = modes.first().cloned().unwrap_or_else(|| "Default".into());
                if let Some(c) = doc.variables.collections.iter_mut().find(|c| c.id == id) {
                    c.modes.push(Mode { id: first, name: first_name });
                }
                id
            }
        };
        let first = doc.variables.collections.iter().find(|c| c.id == cid).and_then(|c| c.modes.first()).map(|m| m.id.clone());
        if let Some(f) = &first {
            mode_ids.insert((cname.clone(), None), f.clone());
        }
        for m in modes {
            let found = doc.variables.collections.iter().find(|c| c.id == cid).and_then(|c| c.modes.iter().find(|x| x.name.eq_ignore_ascii_case(m))).map(|x| x.id.clone());
            let id = match found {
                Some(id) => id,
                None => {
                    let Some(id) = doc.new_mode_id(&cid) else { continue };
                    if let Some(c) = doc.variables.collections.iter_mut().find(|c| c.id == cid) {
                        c.modes.push(Mode { id: id.clone(), name: m.clone() });
                    }
                    for var in doc.variables.variables.iter_mut().filter(|v| v.collection == cid) {
                        if let Some(val) = first.as_ref().and_then(|f| var.values.get(f)).cloned() {
                            var.values.insert(id.clone(), val);
                        }
                    }
                    added_modes += 1;
                    id
                }
            };
            mode_ids.insert((cname.clone(), Some(m.clone())), id);
        }
        coll_ids.insert(cname.clone(), cid);
    }
    // Read every value; types come from the first that tells (references take their target's).
    let reads: Vec<Vec<(Option<String>, Option<Read>)>> = tokens.iter().map(|t| t.values.iter().map(|(m, v)| (m.clone(), read(t.ty.as_deref(), v))).collect()).collect();
    let key = |path: &[String]| path.join(".");
    let mut kinds: BTreeMap<String, VariableType> = BTreeMap::new();
    for (t, rs) in tokens.iter().zip(&reads) {
        if let Some(k) = rs.iter().find_map(|(_, r)| match r {
            Some(Read::Value(k, _)) => Some(*k),
            _ => None,
        }) {
            kinds.insert(key(&t.path), k);
        }
    }
    // A reference may leave out the collection: find it either way.
    let resolve_path = |r: &[String], kinds: &BTreeMap<String, VariableType>| -> Option<String> {
        let full = key(r);
        if kinds.contains_key(&full) {
            return Some(full);
        }
        kinds.keys().find(|k| k.ends_with(&format!(".{full}"))).cloned()
    };
    for _ in 0..8 {
        for (t, rs) in tokens.iter().zip(&reads) {
            if kinds.contains_key(&key(&t.path)) {
                continue;
            }
            if let Some(k) = rs.iter().find_map(|(_, r)| match r {
                Some(Read::Ref(p)) => resolve_path(p, &kinds).and_then(|k| kinds.get(&k).copied()),
                _ => None,
            }) {
                kinds.insert(key(&t.path), k);
            }
        }
    }
    // Create or find the variables.
    let mut ids: BTreeMap<String, VariableId> = BTreeMap::new();
    let (mut added, mut updated) = (0usize, 0usize);
    for t in &tokens {
        let k = key(&t.path);
        let (cname, name) = place(&t.path);
        let Some(kind) = kinds.get(&k).copied() else {
            warnings.push(format!("{k}: {} isn't a type FigCraft variables hold; skipped", t.ty.as_deref().unwrap_or("this value")));
            continue;
        };
        let Some(cid) = coll_ids.get(&cname).cloned() else { continue };
        match doc.variables.variables.iter().find(|v| v.collection == cid && v.name == name) {
            Some(v) if v.kind != kind => warnings.push(format!("{k}: `{name}` is a {} variable here; skipped", v.kind.id())),
            Some(v) => {
                ids.insert(k, v.id.clone());
                updated += 1;
            }
            None => {
                let id = doc.new_variable_id();
                doc.variables.variables.push(Variable {
                    id: id.clone(),
                    name,
                    collection: cid,
                    kind,
                    values: BTreeMap::new(),
                    description: t.description.clone(),
                    scopes: None,
                    hidden_from_publishing: false,
                    extends: None,
                    overrides: BTreeSet::new(),
                    code_syntax: BTreeMap::new(),
                });
                ids.insert(k, id);
                added += 1;
            }
        }
    }
    // Values, per mode.
    for (t, rs) in tokens.iter().zip(&reads) {
        let k = key(&t.path);
        let Some(id) = ids.get(&k).cloned() else { continue };
        let (cname, _) = place(&t.path);
        for (m, r) in rs {
            let Some(mode) = mode_ids.get(&(cname.clone(), m.clone())).cloned() else { continue };
            let value = match r {
                Some(Read::Value(kind, v)) if kinds.get(&k) == Some(kind) => v.clone(),
                Some(Read::Ref(p)) => match resolve_path(p, &kinds).and_then(|t| ids.get(&t)) {
                    Some(target) if *target != id => VariableValue::Alias(target.clone()),
                    _ => {
                        warnings.push(format!("{k}: {{{}}} isn't a token here; skipped", p.join(".")));
                        continue;
                    }
                },
                _ => {
                    warnings.push(format!("{k}: a value doesn't fit its type; skipped"));
                    continue;
                }
            };
            if let Some(v) = doc.variables.variables.iter_mut().find(|v| v.id == id) {
                v.values.insert(mode, value);
            }
        }
    }
    // Every mode holds a value.
    for id in ids.values() {
        let Some(cid) = doc.variables.variables.iter().find(|v| v.id == *id).map(|v| v.collection.clone()) else { continue };
        let all: Vec<ModeId> = doc.variables.collections.iter().find(|c| c.id == cid).map(|c| c.modes.iter().map(|m| m.id.clone()).collect()).unwrap_or_default();
        if let Some(v) = doc.variables.variables.iter_mut().find(|v| v.id == *id) {
            let first = all.iter().find_map(|m| v.values.get(m)).cloned().unwrap_or_else(|| VariableValue::default_for(v.kind));
            for m in &all {
                v.values.entry(m.clone()).or_insert_with(|| first.clone());
            }
        }
    }
    Ok(json!({"variables": added, "updated": updated, "modes": added_modes, "warnings": warnings}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_go_out_and_come_back() {
        let mut s = Session::new();
        let c = s.execute("variable.createCollection", &json!({"name": "Brand"})).unwrap()["id"].as_str().unwrap().to_string();
        let dark = s.execute("variable.addMode", &json!({"collection": c, "name": "Dark"})).unwrap()["id"].as_str().unwrap().to_string();
        let blue = s.execute("variable.create", &json!({"collection": c, "name": "color/blue", "type": "color", "value": "#1F4BFF"})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.set", &json!({"variable": blue, "mode": dark, "value": "#88A0FF"})).unwrap();
        let prim = s.execute("variable.create", &json!({"collection": c, "name": "color/primary", "type": "color", "value": "#000000"})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.set", &json!({"variable": prim, "value": {"alias": blue}})).unwrap();
        s.execute("variable.set", &json!({"variable": prim, "mode": dark, "value": {"alias": blue}})).unwrap();
        let sp = s.execute("variable.create", &json!({"collection": c, "name": "space/m", "type": "number", "value": 16})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.update", &json!({"variable": sp, "scopes": ["GAP"]})).unwrap();
        s.execute("variable.create", &json!({"collection": c, "name": "font/body", "type": "string", "value": "Inter"})).unwrap();
        let out = s.execute("variable.exportDtcg", &json!({})).unwrap();
        let v: Value = serde_json::from_str(out["text"].as_str().unwrap()).unwrap();
        assert_eq!(v["Brand"]["color"]["blue"]["$type"], "color");
        assert_eq!(v["Brand"]["color"]["blue"]["$value"], "#1F4BFF");
        assert_eq!(v["Brand"]["color"]["blue"]["$extensions"][MODES]["Dark"], "#88A0FF");
        assert_eq!(v["Brand"]["color"]["primary"]["$value"], "{Brand.color.blue}", "an alias is a reference");
        assert_eq!(v["Brand"]["space"]["m"]["$type"], "dimension");
        assert_eq!(v["Brand"]["space"]["m"]["$value"], json!({"value": 16.0, "unit": "px"}));
        assert_eq!(v["Brand"]["font"]["body"]["$type"], "fontFamily");
        // Into a new file: the same collection, modes, aliases.
        let mut t = Session::new();
        let r = t.execute("variable.importDtcg", &json!({"json": out["text"]})).unwrap();
        assert_eq!(r["variables"], 4, "{r}");
        assert!(r["warnings"].as_array().unwrap().is_empty(), "{r}");
        let doc = t.doc().unwrap();
        let coll = doc.variables.collections.iter().find(|c| c.name == "Brand").unwrap();
        assert_eq!(coll.modes.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), vec!["Default", "Dark"]);
        let blue2 = doc.variables.get("color/blue").unwrap();
        assert_eq!(blue2.values.get(&coll.modes[1].id), Some(&VariableValue::Color(Color::from_hex("#88A0FF").unwrap())));
        let prim2 = doc.variables.get("color/primary").unwrap();
        assert_eq!(prim2.values.get(&coll.modes[0].id), Some(&VariableValue::Alias(blue2.id.clone())));
        assert_eq!(doc.variables.get("space/m").unwrap().values.get(&coll.modes[0].id), Some(&VariableValue::Number(16.0)));
    }

    #[test]
    fn other_tools_tokens_read_too() {
        let mut s = Session::new();
        let json = r##"{
          "color": { "$type": "color",
            "accent": { "$value": { "colorSpace": "srgb", "components": [1, 0, 0.5], "hex": "#ff0080" } },
            "link": { "$value": "{color.accent}" } },
          "size": { "base": { "$type": "dimension", "$value": "1rem" }, "gap": { "value": "8px", "type": "dimension" } },
          "weight": { "bold": { "$type": "fontWeight", "$value": "bold" } },
          "shadow": { "card": { "$type": "shadow", "$value": { "offsetX": "0px" } } }
        }"##;
        let r = s.execute("variable.importDtcg", &json!({"json": json, "collection": "Imported"})).unwrap();
        assert_eq!(r["variables"], 5, "{r}");
        assert_eq!(r["warnings"].as_array().unwrap().len(), 1, "the shadow is skipped: {r}");
        let doc = s.doc().unwrap();
        let get = |n: &str| doc.variables.get(n).unwrap().values.values().next().unwrap().clone();
        assert_eq!(get("color/accent"), VariableValue::Color(Color::from_hex("#FF0080").unwrap()));
        assert!(matches!(get("color/link"), VariableValue::Alias(_)));
        assert_eq!(get("size/base"), VariableValue::Number(16.0));
        assert_eq!(get("size/gap"), VariableValue::Number(8.0), "the older value / type form");
        assert_eq!(get("weight/bold"), VariableValue::Number(700.0));
    }
}
