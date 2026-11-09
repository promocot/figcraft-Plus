//! FigCraft Plus: line height in % from a number variable.
//!
//! FigCraft binds a number variable to the line height as pixels: `leading = 24` gives every text
//! 24 px, whatever its size. CSS writes leading as a share of the font size instead
//! (`line-height: 140%`, or `1.4`), so one token — `leading/normal` — suits every size. Here a text
//! layer, or a text style, can take its line height from a number variable as a percentage:
//!
//! ```text
//! lineHeight {leading/normal}      140 → 140% · 1.4 → 140% (a value up to 5 is a ratio, as in CSS)
//! ```
//!
//! The line is a layer's data (`figcraftplus.lineHeight`) or a line of a text style's description;
//! the line height itself stays a plain percentage in the file, which the official FigCraft keeps.
//! The variable is read in the modes that apply to the layer (a style: its collection's default
//! mode). Typing a line height by hand, or binding a variable FigCraft's way (in px), ends it.

use std::collections::{BTreeMap, BTreeSet};

use figcraft_doc::{Document, LineHeight, NodeId, StyleKind, VariableType, VariableValue};
use serde_json::{Value, json};

use crate::cmd::{CommandSpec, ok};
use crate::params::string;
use crate::{Result, Session, bad, spec};

/// The layer data key holding a text layer's line-height variable.
pub const KEY: &str = "figcraftplus.lineHeight";

/// The field FigCraft binds the line height under (a layer's or a text style's).
const FIELD: &str = "lineHeight";

/// The variable named in a rule line (`lineHeight {name}`), if `line` is one.
fn parse_line(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix(FIELD)?.trim_start();
    let name = rest.strip_prefix('{')?.strip_suffix('}')?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn format(name: &str) -> String {
    format!("{FIELD} {{{name}}}")
}

/// The variable a text layer's line height follows, as a percentage.
pub fn layer_rule(doc: &Document, id: NodeId) -> Option<String> {
    doc.node(id)?.plugin_data.get(KEY).and_then(|t| t.lines().find_map(parse_line))
}

/// The variable a text style's line height follows, as a percentage.
pub fn style_rule(doc: &Document, style: &str) -> Option<String> {
    doc.style(style).and_then(|s| s.description.lines().find_map(parse_line))
}

/// `description` with its rule line replaced by `name`'s (or removed: None); its own text kept.
fn with_rule(description: &str, name: Option<&str>) -> String {
    let mut lines: Vec<String> = description.lines().filter(|l| parse_line(l).is_none()).map(str::to_string).collect();
    if let Some(n) = name {
        lines.push(format(n));
    }
    lines.join("\n").trim_matches('\n').to_string()
}

/// `description` without the rule line (what a style's dialog shows as its description).
pub fn without_rule(description: &str) -> String {
    with_rule(description, None)
}

/// An edited description (`edited`, shown without the rule line) with the rule line of `current`
/// kept.
pub fn keep_rule(edited: &str, current: &str) -> String {
    let name = current.lines().find_map(parse_line);
    with_rule(edited, name.as_deref())
}

/// A number variable's value as a line height percentage: up to 5 it is a ratio (1.4 → 140%), as
/// a unitless CSS `line-height`; above, a percentage.
pub fn percent_of(v: f64) -> f64 {
    if v.abs() <= 5.0 { v * 100.0 } else { v }
}

fn number(doc: &Document, name: &str, modes: &BTreeMap<figcraft_doc::CollectionId, figcraft_doc::ModeId>) -> Option<f64> {
    let v = doc.variables.get(name)?;
    match doc.variables.resolve(&v.id, modes)? {
        VariableValue::Number(x) if x.is_finite() => Some(x),
        _ => None,
    }
}

/// The percentage `name` gives a layer `id` (in its modes).
fn layer_percent(doc: &Document, id: NodeId, name: &str) -> Option<f64> {
    number(doc, name, &doc.modes_for(id)).map(percent_of).map(|p| p.max(0.0))
}

/// The percentage `name` gives a text style (its collection's default mode).
fn style_percent(doc: &Document, name: &str) -> Option<f64> {
    number(doc, name, &BTreeMap::new()).map(percent_of).map(|p| p.max(0.0))
}

fn same(a: LineHeight, b: LineHeight) -> bool {
    match (a, b) {
        (LineHeight::Percent(x), LineHeight::Percent(y)) | (LineHeight::Pixels(x), LineHeight::Pixels(y)) => (x - y).abs() < 1e-6,
        (LineHeight::Auto, LineHeight::Auto) => true,
        _ => false,
    }
}

/// Every line height a text layer has (its style's, then its runs').
fn layer_heights(doc: &Document, id: NodeId) -> Option<Vec<LineHeight>> {
    let t = doc.node(id)?.kind.text()?;
    Some(std::iter::once(t.style.line_height).chain(t.runs.iter().map(|r| r.style.line_height)).collect())
}

fn style_height(doc: &Document, style: &str) -> Option<LineHeight> {
    match &doc.style(style)?.kind {
        StyleKind::Text { style, .. } => Some(style.line_height),
        _ => None,
    }
}

fn drop_layer_rule(doc: &mut Document, id: NodeId) {
    if let Some(n) = doc.node_mut(id) {
        n.plugin_data.remove(KEY);
    }
}

fn set_style_rule(doc: &mut Document, style: &str, name: Option<&str>) {
    if let Some(st) = doc.styles.iter_mut().find(|s| s.id == style) {
        st.description = with_rule(&st.description, name);
    }
}

/// Keep every line-height variable in step (called by the engine's normalisation, inside the
/// edit's undo step).
pub fn resolve(doc: &mut Document, before: &Document, touched: &BTreeSet<NodeId>) {
    let revalued = doc.variables != before.variables;
    // Text layers.
    let ids: Vec<NodeId> = doc.ids().filter(|id| doc.node(*id).is_some_and(|n| n.plugin_data.contains_key(KEY))).collect();
    for id in ids {
        // A mode set on a frame around it counts too.
        if !revalued && !touched.contains(&id) && !doc.ancestors(id).iter().any(|a| touched.contains(a)) {
            continue;
        }
        let Some(name) = layer_rule(doc, id) else {
            drop_layer_rule(doc, id);
            continue;
        };
        // FigCraft's own binding (in px) takes over; so does a layer that is no text.
        if doc.node(id).is_some_and(|n| n.bound.contains_key(FIELD) || n.kind.text().is_none()) {
            drop_layer_rule(doc, id);
            continue;
        }
        // Typed by hand: the line height changed in this edit while the rule held before.
        let rule_was = layer_rule(before, id).as_deref() == Some(name.as_str());
        if rule_was
            && let (Some(was), Some(now), Some(p)) = (layer_heights(before, id), layer_heights(doc, id), layer_percent(before, id, &name))
            && was.iter().all(|h| same(*h, LineHeight::Percent(p)))
            && was.len() == now.len()
            && was.iter().zip(&now).any(|(a, b)| !same(*a, *b))
        {
            drop_layer_rule(doc, id);
            continue;
        }
        let Some(p) = layer_percent(doc, id, &name) else { continue };
        let want = LineHeight::Percent(p);
        if layer_heights(doc, id).is_some_and(|hs| hs.iter().all(|h| same(*h, want))) {
            continue;
        }
        if let Some(t) = doc.node_mut(id).and_then(|n| n.kind.text_mut()) {
            t.restyle(0, usize::MAX, |s| s.line_height = want);
        }
        doc.mark_dirty([id]);
    }
    // Text styles (the layers using them follow as FigCraft propagates a style).
    let styles: Vec<String> = doc.styles.iter().filter(|s| s.description.lines().any(|l| parse_line(l).is_some())).map(|s| s.id.clone()).collect();
    for sid in styles {
        let Some(name) = style_rule(doc, &sid) else { continue };
        if doc.style(&sid).is_some_and(|s| s.bound.contains_key(FIELD) || !matches!(s.kind, StyleKind::Text { .. })) {
            set_style_rule(doc, &sid, None);
            continue;
        }
        let rule_was = style_rule(before, &sid).as_deref() == Some(name.as_str());
        if rule_was
            && let (Some(was), Some(now), Some(p)) = (style_height(before, &sid), style_height(doc, &sid), style_percent(before, &name))
            && same(was, LineHeight::Percent(p))
            && !same(was, now)
        {
            set_style_rule(doc, &sid, None);
            continue;
        }
        let Some(p) = style_percent(doc, &name) else { continue };
        let want = LineHeight::Percent(p);
        if let Some(st) = doc.styles.iter_mut().find(|s| s.id == sid)
            && let StyleKind::Text { style, .. } = &mut st.kind
            && !same(style.line_height, want)
        {
            style.line_height = want;
        }
    }
}

// ---- commands -----------------------------------------------------------------------------------

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "text.lineHeightVariable",
            "Line height from a variable (%)",
            "{ids?, variable? (name or id of a number variable; none: stop following it)} — the line height is the variable as a percentage of the font size (140 → 140%; up to 5, a ratio: 1.4 → 140%), in each layer's modes",
            true,
            layer
        ),
        spec!(
            "style.lineHeightVariable",
            "Text style line height from a variable (%)",
            "{style (id or name), variable? (name or id of a number variable; none: stop following it)} — the style's line height is the variable as a percentage of its font size",
            true,
            text_style
        ),
    ]
}

/// The number variable `p.variable` names (its name), or None when the parameter is absent.
fn variable(cmd: &str, doc: &Document, p: &Value) -> Result<Option<String>> {
    match string(p, "variable") {
        Some(v) => {
            let x = doc.variables.get(&v).ok_or_else(|| bad(cmd, "no such variable"))?;
            if x.kind != VariableType::Number {
                return Err(bad(cmd, "a number variable"));
            }
            Ok(Some(x.name.clone()))
        }
        None => Ok(None),
    }
}

fn layer(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "text.lineHeightVariable";
    let ids = crate::params::req_targets(cmd, s, p)?;
    let doc = s.doc_mut()?;
    let var = variable(cmd, doc, p)?;
    for id in ids {
        let n = doc.get_mut(id)?;
        if n.kind.text().is_none() {
            return Err(bad(cmd, "a text layer"));
        }
        match &var {
            Some(v) => {
                // In place of a px binding.
                n.bound.remove(FIELD);
                n.plugin_data.insert(KEY.into(), format(v));
            }
            None => {
                n.plugin_data.remove(KEY);
            }
        }
        doc.mark_dirty([id]);
    }
    ok()
}

fn text_style(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "style.lineHeightVariable";
    let key = string(p, "style").ok_or_else(|| bad(cmd, "style: a text style's id or name"))?;
    let doc = s.doc_mut()?;
    let var = variable(cmd, doc, p)?;
    let st = doc.styles.iter_mut().find(|s| s.id == key || s.name == key).ok_or_else(|| bad(cmd, "no such style"))?;
    if !matches!(st.kind, StyleKind::Text { .. }) {
        return Err(bad(cmd, "a text style"));
    }
    if var.is_some() {
        st.bound.remove(FIELD);
    }
    st.description = with_rule(&st.description, var.as_deref());
    Ok(json!({"style": st.id}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(v: Value) -> NodeId {
        NodeId(v["id"].as_u64().unwrap())
    }

    fn lh(s: &Session, n: NodeId) -> LineHeight {
        s.doc().unwrap().node(n).unwrap().kind.text().unwrap().style.line_height
    }

    fn setup() -> (Session, NodeId, String) {
        let mut s = Session::new();
        let c = s.execute("variable.createCollection", &json!({"name": "Type"})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.create", &json!({"collection": c, "name": "leading/normal", "type": "number", "value": 140})).unwrap();
        let t = id(s.execute("text.create", &json!({"text": "Hello", "x": 0, "y": 0, "props": {"fontSize": 20}})).unwrap());
        (s, t, c)
    }

    #[test]
    fn a_text_follows_the_variable_as_a_percentage() {
        let (mut s, t, _) = setup();
        s.execute("text.lineHeightVariable", &json!({"ids": [t], "variable": "leading/normal"})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Percent(140.0), "140 is 140% of the font size");
        s.execute("variable.set", &json!({"variable": "leading/normal", "value": 1.5})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Percent(150.0), "1.5 is a ratio, as in CSS");
        s.execute("node.set", &json!({"ids": [t], "props": {"fontSize": 40}})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Percent(150.0), "it follows the size: no px");
        s.undo().unwrap();
        s.undo().unwrap();
        assert_eq!(lh(&s, t), LineHeight::Percent(140.0), "one undo step with the variable");
    }

    #[test]
    fn modes_switch_the_leading() {
        let (mut s, t, c) = setup();
        let mobile = s.execute("variable.addMode", &json!({"collection": c, "name": "Mobile"})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.set", &json!({"variable": "leading/normal", "mode": mobile, "value": 120})).unwrap();
        let f = id(s.execute("frame.create", &json!({"x": 0, "y": 100, "width": 300, "height": 200})).unwrap());
        let t2 = id(s.execute("text.create", &json!({"text": "Hi", "x": 10, "y": 10, "parent": f, "props": {"fontSize": 20}})).unwrap());
        s.execute("text.lineHeightVariable", &json!({"ids": [t, t2], "variable": "leading/normal"})).unwrap();
        s.execute("variable.setMode", &json!({"ids": [f], "collection": c, "mode": mobile})).unwrap();
        assert_eq!(lh(&s, t2), LineHeight::Percent(120.0), "the Mobile mode's leading inside the frame");
        assert_eq!(lh(&s, t), LineHeight::Percent(140.0), "the default mode outside it");
    }

    #[test]
    fn typing_a_line_height_or_binding_in_px_ends_it() {
        let (mut s, t, _) = setup();
        s.execute("text.lineHeightVariable", &json!({"ids": [t], "variable": "leading/normal"})).unwrap();
        s.execute("node.set", &json!({"ids": [t], "props": {"lineHeight": 30}})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Pixels(30.0));
        assert_eq!(layer_rule(s.doc().unwrap(), t), None, "typed by hand: the rule ends");
        s.execute("variable.set", &json!({"variable": "leading/normal", "value": 160})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Pixels(30.0));
        s.execute("text.lineHeightVariable", &json!({"ids": [t], "variable": "leading/normal"})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Percent(160.0));
        s.execute("variable.bind", &json!({"ids": [t], "field": "lineHeight", "variable": "leading/normal"})).unwrap();
        assert_eq!(lh(&s, t), LineHeight::Pixels(160.0), "FigCraft's binding, in px");
        assert_eq!(layer_rule(s.doc().unwrap(), t), None);
    }

    #[test]
    fn a_text_style_follows_it_and_so_do_its_layers() {
        let (mut s, t, _) = setup();
        let st = s.execute("style.create", &json!({"type": "text", "name": "Body", "value": {"fontSize": 16}})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("style.update", &json!({"style": st, "description": "Body copy"})).unwrap();
        s.execute("style.apply", &json!({"ids": [t], "style": st})).unwrap();
        s.execute("style.lineHeightVariable", &json!({"style": st, "variable": "leading/normal"})).unwrap();
        let doc = s.doc().unwrap();
        assert_eq!(style_height(doc, &st), Some(LineHeight::Percent(140.0)));
        assert_eq!(doc.style(&st).unwrap().description, "Body copy\nlineHeight {leading/normal}", "its own text kept");
        s.execute("variable.set", &json!({"variable": "leading/normal", "value": 125})).unwrap();
        assert_eq!(style_height(s.doc().unwrap(), &st), Some(LineHeight::Percent(125.0)));
        assert_eq!(s.doc().unwrap().node(t).unwrap().styles.text.as_deref(), Some(st.as_str()));
        assert_eq!(lh(&s, t), LineHeight::Percent(125.0), "its layers follow the style");
        s.execute("style.lineHeightVariable", &json!({"style": st})).unwrap();
        assert_eq!(s.doc().unwrap().style(&st).unwrap().description, "Body copy");
    }

    #[test]
    fn css_for_code_writes_the_leading_as_a_ratio() {
        let (mut s, t, _) = setup();
        s.execute("text.lineHeightVariable", &json!({"ids": [t], "variable": "leading/normal"})).unwrap();
        let css = s.execute("variable.exportCss", &json!({})).unwrap();
        let css = css["text"].as_str().unwrap().to_string();
        assert!(css.contains(": 1.4;"), "line-height: var(--leading-normal) takes 1.4: {css}");
    }
}
