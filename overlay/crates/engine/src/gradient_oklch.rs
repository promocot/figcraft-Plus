//! FigCraft Plus: gradient tokens — gradients blended in OKLCH.
//!
//! FigCraft binds gradient stops to colour variables already, and draws a gradient by mixing its
//! stops in sRGB: blue → yellow passes through grey. CSS can blend in OKLCH instead
//! (`linear-gradient(in oklch, …)`), which keeps the colours alive. Here a gradient — a layer's
//! fill or stroke, or a paint style's, so a gradient becomes a token of the design system — can be
//! set to blend in OKLCH: the engine keeps its stops (the *key* stops, with their variables) and
//! puts small steps between each two, mixed in OKLCH, so the sRGB drawing follows the OKLCH path.
//! Change a variable, a key stop or the palette's mode and the steps follow.
//!
//! The file keeps plain stops, which the official FigCraft draws as they are. Which stops are the
//! keys is a line of text (a layer's data `figcraftplus.gradients`, a paint style's description):
//!
//! ```text
//! fills/0 in oklch 0 0.4 1        the key stops' positions
//! ```
//!
//! A stop added, moved or removed by hand is simply a new set of keys: every stop that isn't one of
//! the steps is a key. Back to sRGB, only the keys stay.

use std::collections::{BTreeMap, BTreeSet};

use figcraft_color::oklch_profile::Oklch;
use figcraft_color::{Color, ColorProfile, ColorStop, Gradient, Paint, PaintKind};
use figcraft_doc::{Document, NodeId, StyleKind, VariableId, VariableValue};
use serde_json::{Value, json};

use crate::cmd::{CommandSpec, ok};
use crate::params::{get, req_string, string};
use crate::{Result, Session, bad, spec};

/// The layer data key holding a layer's OKLCH gradients, one per line.
pub const KEY: &str = "figcraftplus.gradients";

/// Steps between two key stops.
const STEPS: usize = 16;

/// Two key stops nearer than this get no steps between them.
const MIN_GAP: f32 = 0.04;

/// How near a stop must be to a step's place to be that step.
const NEAR: f32 = 1e-3;

/// One OKLCH gradient: the paint list (`fills` or `strokes`), the paint's index, its keys.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub list: String,
    pub index: usize,
    pub keys: Vec<f32>,
}

impl Rule {
    fn slot(&self) -> String {
        format!("{}/{}", self.list, self.index)
    }
}

pub fn parse_line(line: &str) -> Option<Rule> {
    let mut words = line.split_whitespace();
    let slot = words.next()?;
    let (list, index) = slot.split_once('/')?;
    if !matches!(list, "fills" | "strokes") {
        return None;
    }
    let index = index.parse().ok()?;
    if words.next()? != "in" || words.next()? != "oklch" {
        return None;
    }
    let keys = words.map(|w| w.parse::<f32>().ok().filter(|p| p.is_finite())).collect::<Option<Vec<f32>>>()?;
    Some(Rule { list: list.to_string(), index, keys })
}

pub fn format(r: &Rule) -> String {
    let keys: Vec<String> = r.keys.iter().map(|k| figcraft_doc::trim_num(f64::from((*k * 10_000.0).round() / 10_000.0))).collect();
    format!("{} in oklch {}", r.slot(), keys.join(" ")).trim_end().to_string()
}

fn rules_in(text: &str) -> Vec<Rule> {
    text.lines().filter_map(parse_line).collect()
}

/// `text` with the rule of `list/index` replaced by `new` (or removed); other lines kept.
fn with_rule(text: &str, list: &str, index: usize, new: Option<&Rule>) -> String {
    let mut lines: Vec<String> = text.lines().filter(|l| parse_line(l).is_none_or(|r| r.list != list || r.index != index)).map(str::to_string).collect();
    if let Some(r) = new {
        lines.push(format(r));
    }
    lines.join("\n").trim_matches('\n').to_string()
}

/// A layer's OKLCH gradients.
pub fn layer_rules(doc: &Document, id: NodeId) -> Vec<Rule> {
    doc.node(id).and_then(|n| n.plugin_data.get(KEY)).map(|t| rules_in(t)).unwrap_or_default()
}

/// A paint style's OKLCH gradients (its paints are `fills`).
pub fn style_rules(doc: &Document, style: &str) -> Vec<Rule> {
    doc.style(style).map(|s| rules_in(&s.description)).unwrap_or_default()
}

/// The places of the steps between `keys` (sorted).
fn step_places(keys: &[f32]) -> Vec<f32> {
    let mut out = Vec::new();
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b - a < MIN_GAP {
            continue;
        }
        for k in 1..=STEPS {
            out.push(a + (b - a) * k as f32 / (STEPS + 1) as f32);
        }
    }
    out
}

/// Which stops of `g` are steps of `keys` (true) rather than keys.
pub fn steps_of(g: &Gradient, keys: &[f32]) -> Vec<bool> {
    let places = step_places(keys);
    g.stops.iter().map(|s| places.iter().any(|p| (p - s.position).abs() < NEAR)).collect()
}

/// `a` → `b` at `t` in OKLCH (the shorter way round the hue; a grey takes the other's hue), alpha
/// premultiplied as CSS mixes.
pub fn mix(a: Color, b: Color, t: f64, profile: ColorProfile) -> Color {
    let (oa, ob) = (Oklch::from_color(a, profile), Oklch::from_color(b, profile));
    let (aa, ab) = (f64::from(a.a), f64::from(b.a));
    let alpha = aa + (ab - aa) * t;
    let grey = |o: &Oklch| o.c < 1e-4;
    let (ha, hb) = match (grey(&oa), grey(&ob)) {
        (true, false) => (ob.h, ob.h),
        (false, true) => (oa.h, oa.h),
        _ => (oa.h, ob.h),
    };
    let mut dh = hb - ha;
    if dh > 180.0 {
        dh -= 360.0;
    } else if dh < -180.0 {
        dh += 360.0;
    }
    let h = (ha + dh * t).rem_euclid(360.0);
    let pre = |x: f64, y: f64| if alpha > 1e-9 { (x * aa + (y * ab - x * aa) * t) / alpha } else { x + (y - x) * t };
    let l = pre(oa.l, ob.l);
    let c = pre(oa.c, ob.c).max(0.0);
    Oklch::new(l, c, h).to_color(profile, alpha as f32)
}

/// The stops of an OKLCH gradient: `keys` (sorted by position) with the steps between them.
pub fn stops_for(keys: &[ColorStop], profile: ColorProfile) -> Vec<ColorStop> {
    let mut out = Vec::new();
    for (i, k) in keys.iter().enumerate() {
        out.push(*k);
        let Some(next) = keys.get(i + 1) else { break };
        if next.position - k.position < MIN_GAP {
            continue;
        }
        for s in 1..=STEPS {
            let t = s as f32 / (STEPS + 1) as f32;
            out.push(ColorStop { position: k.position + (next.position - k.position) * t, color: mix(k.color, next.color, f64::from(t), profile) });
        }
    }
    out
}

/// A bound stop's colour, in `modes`.
type Resolve<'a> = dyn Fn(&VariableId) -> Option<Color> + 'a;

/// Bring paint `list/index` of `paints` in step with `rule`: the keys (the stops that aren't
/// steps), their colours (a bound stop's from its variable), the steps between them. `bound` is
/// the layer's (or style's) bindings, renumbered with the stops; `colors` its stop formulas
/// (`figcraftplus.colors` lines), renumbered too. Returns the new rule (None: no longer a gradient
/// with two keys: the rule ends), and whether anything changed.
fn settle(
    paints: &mut [Paint],
    rule: &Rule,
    bound: &mut BTreeMap<String, VariableId>,
    colors: Option<&mut String>,
    resolve: &Resolve<'_>,
    profile: ColorProfile,
) -> (Option<Rule>, bool) {
    let Some(paint) = paints.get_mut(rule.index) else { return (None, false) };
    let PaintKind::Gradient(g) = &mut paint.kind else { return (None, false) };
    let steps = steps_of(g, &rule.keys);
    let prefix = format!("{}/{}/stops/", rule.list, rule.index);
    // The keys, in order, with their old index (for their bindings and formulas).
    let mut keys: Vec<(usize, ColorStop)> = g.stops.iter().enumerate().filter(|(i, _)| !steps.get(*i).copied().unwrap_or(false)).map(|(i, s)| (i, *s)).collect();
    keys.sort_by(|a, b| a.1.position.total_cmp(&b.1.position));
    if keys.len() < 2 {
        return (None, false);
    }
    for (i, s) in &mut keys {
        if let Some(c) = bound.get(&format!("{prefix}{i}/color")).and_then(|v| resolve(v)) {
            s.color = c;
        }
    }
    let new_stops = stops_for(&keys.iter().map(|(_, s)| *s).collect::<Vec<_>>(), profile);
    // Where each key ends up.
    let mut at = BTreeMap::new();
    let mut k = 0usize;
    for (j, s) in new_stops.iter().enumerate() {
        if keys.get(k).is_some_and(|(_, ks)| ks.position == s.position && ks.color == s.color) {
            if let Some((old, _)) = keys.get(k) {
                at.insert(*old, j);
            }
            k += 1;
        }
    }
    let new_rule = Rule { keys: keys.iter().map(|(_, s)| s.position).collect(), ..rule.clone() };
    let same_stops = g.stops.len() == new_stops.len()
        && g.stops.iter().zip(&new_stops).all(|(a, b)| (a.position - b.position).abs() < 1e-6 && a.color == b.color);
    let renumbered = at.iter().any(|(o, n)| o != n);
    if same_stops && !renumbered {
        return (Some(new_rule), false);
    }
    g.stops = new_stops;
    // Bindings and formulas follow their stops; steps have none.
    let old: Vec<(String, VariableId)> = bound.iter().filter(|(f, _)| f.starts_with(&prefix)).map(|(f, v)| (f.clone(), v.clone())).collect();
    for (f, _) in &old {
        bound.remove(f);
    }
    for (f, v) in old {
        let rest = &f[prefix.len()..];
        if let Some((j, tail)) = rest.split_once('/')
            && let Ok(j) = j.parse::<usize>()
            && let Some(n) = at.get(&j)
        {
            bound.insert(format!("{prefix}{n}/{tail}"), v);
        }
    }
    if let Some(text) = colors {
        let lines: Vec<String> = text
            .lines()
            .filter_map(|l| {
                let Some(rest) = l.strip_prefix(&prefix) else { return Some(l.to_string()) };
                let (j, tail) = rest.split_once(|c: char| c == ' ' || c == '/')?;
                let n = at.get(&j.parse::<usize>().ok()?)?;
                let sep = &rest[j.len()..j.len() + 1];
                Some(format!("{prefix}{n}{sep}{tail}"))
            })
            .collect();
        *text = lines.join("\n");
    }
    (Some(new_rule), true)
}

/// Keep every OKLCH gradient in step (called by the engine's normalisation, inside the edit's undo
/// step).
pub fn resolve(doc: &mut Document, before: &Document, touched: &BTreeSet<NodeId>) {
    let revalued = doc.variables != before.variables;
    let profile = doc.color_profile;
    // Layers.
    let ids: Vec<NodeId> = doc.ids().filter(|id| doc.node(*id).is_some_and(|n| n.plugin_data.contains_key(KEY))).collect();
    for id in ids {
        if !revalued && !touched.contains(&id) && !doc.ancestors(id).iter().any(|a| touched.contains(a)) {
            continue;
        }
        let rules = layer_rules(doc, id);
        let modes = doc.modes_for(id);
        let vars = doc.variables.clone();
        let resolve = move |v: &VariableId| match vars.resolve(v, &modes) {
            Some(VariableValue::Color(c)) => Some(c),
            _ => None,
        };
        let Some(n) = doc.node_mut(id) else { continue };
        let mut text = n.plugin_data.get(KEY).cloned().unwrap_or_default();
        let mut colors = n.plugin_data.get(crate::oklch_links::NODE_KEY).cloned();
        let mut changed = false;
        for r in rules {
            let paints = if r.list == "fills" { &mut n.fills } else { &mut n.strokes };
            let (new, c) = settle(paints, &r, &mut n.bound, colors.as_mut(), &resolve, profile);
            changed |= c;
            text = with_rule(&text, &r.list, r.index, new.as_ref());
        }
        if text.trim().is_empty() {
            n.plugin_data.remove(KEY);
        } else if n.plugin_data.get(KEY) != Some(&text) {
            n.plugin_data.insert(KEY.into(), text);
        }
        if let Some(c) = colors
            && n.plugin_data.get(crate::oklch_links::NODE_KEY) != Some(&c)
        {
            n.plugin_data.insert(crate::oklch_links::NODE_KEY.into(), c);
        }
        if changed {
            doc.mark_dirty([id]);
        }
    }
    // Paint styles (the layers using them follow as FigCraft propagates a style).
    let styles: Vec<String> = doc.styles.iter().filter(|s| !rules_in(&s.description).is_empty()).map(|s| s.id.clone()).collect();
    if styles.is_empty() {
        return;
    }
    let vars = doc.variables.clone();
    let resolve = move |v: &VariableId| match vars.resolve(v, &BTreeMap::new()) {
        Some(VariableValue::Color(c)) => Some(c),
        _ => None,
    };
    for sid in styles {
        let rules = style_rules(doc, &sid);
        let Some(st) = doc.styles.iter_mut().find(|s| s.id == sid) else { continue };
        let StyleKind::Paint { paints } = &mut st.kind else {
            st.description = rules.iter().fold(st.description.clone(), |t, r| with_rule(&t, &r.list, r.index, None));
            continue;
        };
        let mut text = st.description.clone();
        for r in rules {
            let (new, _) = settle(paints, &r, &mut st.bound, None, &resolve, profile);
            text = with_rule(&text, &r.list, r.index, new.as_ref());
        }
        st.description = text;
    }
}

// ---- commands -----------------------------------------------------------------------------------

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "gradient.blend",
        "Gradient blend space",
        "{ids? | style (a paint style's id or name), list? (fills|strokes; default fills), index? (default 0), space: oklch|srgb} — oklch: the gradient is mixed in OKLCH between its stops (steps are added and kept in step with the stops and their variables); srgb: back to FigCraft's own mix, keeping only the key stops",
        true,
        blend
    )]
}

fn blend(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "gradient.blend";
    let list = string(p, "list").unwrap_or_else(|| "fills".into());
    if !matches!(list.as_str(), "fills" | "strokes") {
        return Err(bad(cmd, "list: fills|strokes"));
    }
    let index = get(p, "index").and_then(Value::as_u64).unwrap_or(0) as usize;
    let oklch = match req_string(cmd, p, "space")?.to_ascii_lowercase().as_str() {
        "oklch" => true,
        "srgb" => false,
        _ => return Err(bad(cmd, "space: oklch|srgb")),
    };
    let is_gradient = |paints: &[Paint]| matches!(paints.get(index).map(|p| &p.kind), Some(PaintKind::Gradient(_)));
    if let Some(key) = string(p, "style") {
        let doc = s.doc_mut()?;
        let st = doc.styles.iter_mut().find(|x| x.id == key || x.name == key).ok_or_else(|| bad(cmd, "no such style"))?;
        let StyleKind::Paint { paints } = &mut st.kind else { return Err(bad(cmd, "a paint style")) };
        if list != "fills" || !is_gradient(paints) {
            return Err(bad(cmd, "the style's paint there is no gradient"));
        }
        let (text, stops) = switch(&st.description, paints, &list, index, oklch);
        st.description = text;
        if let Some(stops) = stops
            && let Some(PaintKind::Gradient(g)) = paints.get_mut(index).map(|p| &mut p.kind)
        {
            g.stops = stops;
        }
        let id = st.id.clone();
        return Ok(json!({"style": id}));
    }
    let ids = crate::params::req_targets(cmd, s, p)?;
    let doc = s.doc_mut()?;
    for id in ids {
        let n = doc.get_mut(id)?;
        let paints = if list == "fills" { &mut n.fills } else { &mut n.strokes };
        if !is_gradient(paints) {
            return Err(bad(cmd, "the paint there is no gradient"));
        }
        let text = n.plugin_data.get(KEY).cloned().unwrap_or_default();
        let (text, stops) = switch(&text, paints, &list, index, oklch);
        if let Some(stops) = stops {
            // Back to sRGB: only keys remain, so their bindings move to their new places.
            if let Some(PaintKind::Gradient(g)) = paints.get_mut(index).map(|p| &mut p.kind) {
                let prefix = format!("{list}/{index}/stops/");
                let keep: Vec<usize> = steps_of(g, &rules_in(n.plugin_data.get(KEY).map_or("", String::as_str)).into_iter().find(|r| r.list == list && r.index == index).map(|r| r.keys).unwrap_or_default())
                    .iter()
                    .enumerate()
                    .filter(|(_, step)| !**step)
                    .map(|(i, _)| i)
                    .collect();
                let old: Vec<(String, VariableId)> = n.bound.iter().filter(|(f, _)| f.starts_with(&prefix)).map(|(f, v)| (f.clone(), v.clone())).collect();
                for (f, _) in &old {
                    n.bound.remove(f);
                }
                for (f, v) in old {
                    if let Some((j, tail)) = f[prefix.len()..].split_once('/')
                        && let Some(new) = j.parse::<usize>().ok().and_then(|j| keep.iter().position(|k| *k == j))
                    {
                        n.bound.insert(format!("{prefix}{new}/{tail}"), v);
                    }
                }
                g.stops = stops;
            }
        }
        if text.trim().is_empty() {
            n.plugin_data.remove(KEY);
        } else {
            n.plugin_data.insert(KEY.into(), text);
        }
        doc.mark_dirty([id]);
    }
    ok()
}

/// Turn the gradient at `list/index` of `paints` to OKLCH (its stops become the keys) or back to
/// sRGB (the keys alone stay: the new stops). The rule text, and the stops to set (sRGB only).
fn switch(text: &str, paints: &[Paint], list: &str, index: usize, oklch: bool) -> (String, Option<Vec<ColorStop>>) {
    let Some(PaintKind::Gradient(g)) = paints.get(index).map(|p| &p.kind) else { return (text.to_string(), None) };
    let current = rules_in(text).into_iter().find(|r| r.list == list && r.index == index);
    if oklch {
        if current.is_some() {
            return (text.to_string(), None);
        }
        let mut keys: Vec<f32> = g.stops.iter().map(|s| s.position).collect();
        keys.sort_by(f32::total_cmp);
        let r = Rule { list: list.to_string(), index, keys };
        return (with_rule(text, list, index, Some(&r)), None);
    }
    let Some(r) = current else { return (text.to_string(), None) };
    let steps = steps_of(g, &r.keys);
    let stops: Vec<ColorStop> = g.stops.iter().zip(steps).filter(|(_, step)| !*step).map(|(s, _)| *s).collect();
    (with_rule(text, list, index, None), Some(stops))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(v: Value) -> NodeId {
        NodeId(v["id"].as_u64().unwrap())
    }

    fn stops(s: &Session, n: NodeId) -> Vec<ColorStop> {
        match &s.doc().unwrap().node(n).unwrap().fills[0].kind {
            PaintKind::Gradient(g) => g.stops.clone(),
            _ => panic!("no gradient"),
        }
    }

    /// A linear gradient fill of `stops` (position, hex).
    fn fill(stops: &[(f32, &str)]) -> Value {
        let st: Vec<Value> = stops.iter().map(|(p, c)| json!({"position": p, "color": serde_json::to_value(Color::from_hex(c).unwrap()).unwrap()})).collect();
        json!([{"type": "gradient", "kind": "linear", "stops": st, "from": [0, 0.5], "to": [1, 0.5], "width": [0, 1.5]}])
    }

    fn setup() -> (Session, NodeId) {
        let mut s = Session::new();
        let c = s.execute("variable.createCollection", &json!({"name": "Palette"})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("variable.create", &json!({"collection": c, "name": "blue", "type": "color", "value": "#0000FF"})).unwrap();
        s.execute("variable.create", &json!({"collection": c, "name": "yellow", "type": "color", "value": "#FFFF00"})).unwrap();
        let r = id(s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 200, "height": 100})).unwrap());
        s.execute("node.set", &json!({"ids": [r], "props": {"fills": fill(&[(0.0, "#0000FF"), (1.0, "#FFFF00")])}})).unwrap();
        (s, r)
    }

    fn sat(c: Color) -> f32 {
        let (mx, mn) = (c.r.max(c.g).max(c.b), c.r.min(c.g).min(c.b));
        mx - mn
    }

    #[test]
    fn blending_in_oklch_puts_live_steps_between_the_stops() {
        let (mut s, r) = setup();
        assert_eq!(stops(&s, r).len(), 2, "the gradient set up");
        s.execute("gradient.blend", &json!({"ids": [r], "space": "oklch"})).unwrap();
        let st = stops(&s, r);
        assert_eq!(st.len(), 2 + STEPS, "the keys and the steps between them");
        let mid = st[st.len() / 2].color;
        // sRGB's midpoint of blue and yellow is grey (0.5, 0.5, 0.5); OKLCH's keeps colour.
        assert!(sat(mid) > 0.2, "a colourful middle, not grey: {mid:?}");
        assert_eq!(layer_rules(s.doc().unwrap(), r)[0].keys, vec![0.0, 1.0]);
        s.execute("gradient.blend", &json!({"ids": [r], "space": "srgb"})).unwrap();
        assert_eq!(stops(&s, r).len(), 2, "back to the keys alone");
        assert!(layer_rules(s.doc().unwrap(), r).is_empty());
    }

    #[test]
    fn bound_stops_drive_the_steps_and_keep_their_bindings() {
        let (mut s, r) = setup();
        s.execute("variable.bind", &json!({"ids": [r], "field": "fills/0/stops/1/color", "variable": "yellow"})).unwrap();
        s.execute("gradient.blend", &json!({"ids": [r], "space": "oklch"})).unwrap();
        let last = 1 + STEPS;
        let doc = s.doc().unwrap();
        assert!(doc.node(r).unwrap().bound.contains_key(&format!("fills/0/stops/{last}/color")), "the binding moved with its stop: {:?}", doc.node(r).unwrap().bound);
        s.execute("variable.set", &json!({"variable": "yellow", "value": "#FF0000"})).unwrap();
        let st = stops(&s, r);
        assert_eq!(st.len(), 2 + STEPS);
        assert_eq!(st[last].color.to_hex(), "FF0000", "the key follows its variable");
        let near_end = st[last - 1].color;
        assert!(near_end.r > near_end.b, "the steps follow it too: {near_end:?}");
        s.undo().unwrap();
        assert_eq!(stops(&s, r)[last].color.to_hex(), "FFFF00", "one undo step");
    }

    #[test]
    fn a_stop_added_by_hand_becomes_a_key() {
        let (mut s, r) = setup();
        s.execute("gradient.blend", &json!({"ids": [r], "space": "oklch"})).unwrap();
        let mut st = stops(&s, r);
        st.push(ColorStop { position: 0.5, color: Color::from_hex("#FF00FF").unwrap() });
        let hex: Vec<(f32, String)> = st.iter().map(|s| (s.position, format!("#{}", s.color.to_hex()))).collect();
        let pairs: Vec<(f32, &str)> = hex.iter().map(|(p, h)| (*p, h.as_str())).collect();
        s.execute("node.set", &json!({"ids": [r], "props": {"fills": fill(&pairs)}})).unwrap();
        let rule = layer_rules(s.doc().unwrap(), r);
        assert_eq!(rule[0].keys, vec![0.0, 0.5, 1.0], "{rule:?}");
        assert_eq!(stops(&s, r).len(), 3 + 2 * STEPS);
    }

    #[test]
    fn a_paint_style_blends_in_oklch_too() {
        let (mut s, _) = setup();
        let st = s.execute("style.create", &json!({"type": "paint", "name": "Sunrise", "value": fill(&[(0.0, "#0000FF"), (1.0, "#FFFF00")])})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("style.bind", &json!({"style": st, "field": "fills/0/stops/0/color", "variable": "blue"})).unwrap();
        s.execute("gradient.blend", &json!({"style": st, "space": "oklch"})).unwrap();
        let n = |s: &Session| match &s.doc().unwrap().style(&st).unwrap().kind {
            StyleKind::Paint { paints } => match &paints[0].kind {
                PaintKind::Gradient(g) => g.stops.clone(),
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(n(&s).len(), 2 + STEPS);
        s.execute("variable.set", &json!({"variable": "blue", "value": "#00FF00"})).unwrap();
        assert_eq!(n(&s)[0].color.to_hex(), "00FF00", "the style's key follows its variable");
        assert_eq!(style_rules(s.doc().unwrap(), &st).len(), 1);
    }
}
