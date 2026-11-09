//! FigCraft Plus: several text styles at once (Local styles ▸ select text styles ▸ right-click ▸
//! *Edit N text styles…*).
//!
//! - **Set for all:** font family, style, size, line height and letter spacing. A field shows the
//!   value the styles share, or *Mixed*; what you type goes to every style, one undo step.
//! - **Bind variables by name:** pick a property (size, line height, letter spacing, family,
//!   style) and a group of variables; each style is matched to the variable of the same name
//!   (`Heading/H1` ↔ `font-size/h1`, `Body/M` ↔ `type/size/m`), shown as a preview, and **Bind**
//!   binds them all as one undo step. Styles without a match are listed and left alone.

use std::collections::BTreeMap;

use egui::{Id, Ui};
use figcraft_doc::{Document, LetterSpacing, LineHeight, StyleKind, TextStyle, VariableType};
use serde_json::{Value, json};

use crate::FigApp;
use crate::i18n::{tr, trf};
use crate::theme::{Tokens, regular, semibold};
use crate::widgets::{self, FieldEdit, Lead};

/// The dialog's state: the styles it edits and what binding it prepares.
#[derive(Clone, Debug, Default)]
struct Bulk {
    styles: Vec<String>,
    y: f32,
    field: usize,
    /// The variable group bound from: (collection id, group path).
    group: Option<(String, String)>,
}

fn state_id() -> Id {
    Id::unique("figcraftplus_text_bulk")
}

fn area_id() -> Id {
    Id::unique("figcraftplus_text_bulk_area")
}

const W: f32 = 300.0;

/// The text style fields a variable can bind: (`style.bind` field, label, variable type).
const FIELDS: [(&str, &str, VariableType); 5] = [
    ("fontSize", "Font size", VariableType::Number),
    ("lineHeight", "Line height", VariableType::Number),
    ("letterSpacing", "Letter spacing", VariableType::Number),
    ("fontFamily", "Font family", VariableType::String),
    ("fontStyle", "Font style", VariableType::String),
];

/// Open the dialog on text styles `ids` (at height `y`).
pub(crate) fn open(ctx: &egui::Context, ids: Vec<String>, y: f32) {
    ctx.data_mut(|d| d.insert_temp(state_id(), Bulk { styles: ids, y, ..Default::default() }));
}

fn close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<Bulk>(state_id()));
    crate::popover_ui::forget(area_id());
}

/// What a name is matched on: lower case letters and digits.
fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// For each style `(id, name)`, the variable `(id, name)` it binds to: the one whose last name part
/// is the style's, and when several are, the one sharing the most trailing parts.
pub(crate) fn match_names(styles: &[(String, String)], vars: &[(String, String)]) -> Vec<(String, Option<String>)> {
    let parts = |n: &str| n.split('/').map(norm).filter(|p| !p.is_empty()).collect::<Vec<_>>();
    styles
        .iter()
        .map(|(sid, sname)| {
            let sp = parts(sname);
            let Some(leaf) = sp.last() else { return (sid.clone(), None) };
            let shared = |vp: &[String]| sp.iter().rev().zip(vp.iter().rev()).take_while(|(a, b)| a == b).count();
            let mut best: Option<(usize, &String, bool)> = None;
            for (vid, vname) in vars {
                let vp = parts(vname);
                if vp.last() != Some(leaf) {
                    continue;
                }
                let k = shared(&vp);
                best = match best {
                    Some((b, id, _)) if b > k => Some((b, id, false)),
                    Some((b, _, _)) if b == k => Some((k, vid, true)),
                    _ => Some((k, vid, false)),
                };
            }
            // A tie between equally good names stays unmatched rather than guessed.
            (sid.clone(), best.filter(|(_, _, tie)| !tie).map(|(_, id, _)| id.clone()))
        })
        .collect()
}

/// The styles' text values (skipping any that aren't text styles).
fn texts<'a>(doc: &'a Document, ids: &[String]) -> Vec<(&'a str, &'a str, &'a TextStyle)> {
    ids.iter()
        .filter_map(|id| {
            let s = doc.style(id)?;
            match &s.kind {
                StyleKind::Text { style, .. } => Some((s.id.as_str(), s.name.as_str(), style)),
                _ => None,
            }
        })
        .collect()
}

/// The value every style shares, else None.
fn shared<T: PartialEq + Clone>(all: &[(&str, &str, &TextStyle)], f: impl Fn(&TextStyle) -> T) -> Option<T> {
    let first = all.first().map(|(_, _, t)| f(t))?;
    all.iter().all(|(_, _, t)| f(t) == first).then_some(first)
}

/// `style.update` for every style with `key` set to `value` (one undo step).
fn set_all(app: &mut FigApp, all: &[(&str, &str, &TextStyle)], key: &str, value: Value) {
    let doc = app.session.doc().ok().cloned();
    let commands: Vec<Value> = all
        .iter()
        .map(|(id, _, t)| {
            // With its paragraph settings, which `style.update` would reset otherwise.
            let paragraph = doc.as_ref().and_then(|d| d.style(id)).and_then(|s| match &s.kind {
                StyleKind::Text { paragraph, .. } => Some((**paragraph).clone()),
                _ => None,
            });
            let mut v = crate::styles_plus_ui::text_value(t, &paragraph.unwrap_or_default());
            v[key] = value.clone();
            json!({"command": "style.update", "params": {"style": id, "value": v}})
        })
        .collect();
    if !commands.is_empty() {
        app.run("command.batch", json!({"label": "Edit text styles", "commands": commands}));
    }
}

/// The variable groups of `kind` in this file: (collection id, group path) → label.
fn groups(doc: &Document, kind: VariableType) -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    for v in doc.variables.variables.iter().filter(|v| v.kind == kind) {
        let group = v.name.rsplit_once('/').map(|(g, _)| g.to_string()).unwrap_or_default();
        let coll = doc.variables.collections.iter().find(|c| c.id == v.collection).map(|c| c.name.clone()).unwrap_or_default();
        let label = if group.is_empty() { coll } else { format!("{coll} · {group}") };
        out.insert((v.collection.clone(), group), label);
    }
    out
}

/// The dialog, while open (call once a frame).
pub(crate) fn draw(app: &mut FigApp, ctx: &egui::Context) {
    let Some(mut b) = ctx.data(|d| d.get_temp::<Bulk>(state_id())) else { return };
    let Ok(doc) = app.session.doc().cloned() else { return };
    let all = texts(&doc, &b.styles);
    if all.len() < 2 {
        close(ctx);
        return;
    }
    let t = Tokens::get(ctx);
    let screen = ctx.content_rect();
    let x = (screen.max.x - app.ui.right_width - W - 16.0).max(screen.min.x + 8.0);
    let title = trf("Edit {n} text styles", &[("n", &all.len().to_string())]);
    let mut edits: Vec<(&str, Value)> = Vec::new();
    let mut bind: Option<Vec<Value>> = None;
    let (closed, rect) = crate::popover_ui::show(ctx, area_id(), &title, W, crate::popover::Open::At(egui::pos2(x, b.y.max(screen.min.y + 48.0))), |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 14 }).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
            let mixed = tr("Mixed");
            let label = |ui: &mut Ui, s: &str| {
                ui.add_sized([96.0, 24.0], egui::Label::new(egui::RichText::new(s).font(regular(11.0)).color(t.text_secondary)));
            };
            let fw = W - 32.0 - 104.0;
            ui.label(egui::RichText::new(tr("Set for all")).font(semibold(11.0)).color(t.text));
            ui.horizontal(|ui| {
                label(ui, tr("Font family"));
                let now = shared(&all, |s| s.font_family.clone()).unwrap_or_default();
                if let Some(v) = widgets::text_field(ui, Id::unique("tb_family"), &now, fw, mixed)
                    && !v.trim().is_empty()
                {
                    edits.push(("font_family", json!(v.trim())));
                }
            });
            ui.horizontal(|ui| {
                label(ui, tr("Font style"));
                let now = shared(&all, |s| s.font_style.clone()).unwrap_or_default();
                if let Some(v) = widgets::text_field(ui, Id::unique("tb_style"), &now, fw, mixed)
                    && !v.trim().is_empty()
                {
                    edits.push(("font_style", json!(v.trim())));
                }
            });
            ui.horizontal(|ui| {
                label(ui, tr("Font size"));
                crate::units_ui::field_key("fontSize");
                let now = shared(&all, |s| s.font_size);
                if let Some(FieldEdit::Set(v) | FieldEdit::Scrub(v, true)) =
                    widgets::number_field(ui, Id::unique("tb_size"), Lead::None, now, "", fw, 1.0)
                {
                    edits.push(("font_size", json!(v.max(1.0))));
                }
            });
            ui.horizontal(|ui| {
                label(ui, tr("Line height"));
                let now: Option<LineHeight> = shared(&all, |s| s.line_height);
                let text = now.map(|l| l.label()).unwrap_or_default();
                if let Some(v) = widgets::text_field(ui, Id::unique("tb_lh"), &text, fw, mixed)
                    && let Some(val) = crate::typography::parse_line_height(&v, Some(now.unwrap_or(LineHeight::Auto)))
                {
                    edits.push(("line_height", val));
                }
            });
            ui.horizontal(|ui| {
                label(ui, tr("Letter spacing"));
                let now: Option<LetterSpacing> = shared(&all, |s| s.letter_spacing);
                let text = now.map(|l| l.label()).unwrap_or_default();
                if let Some(v) = widgets::text_field(ui, Id::unique("tb_ls"), &text, fw, mixed)
                    && let Some(val) = crate::typography::parse_letter_spacing(&v, now)
                {
                    edits.push(("letter_spacing", val));
                }
            });
            ui.add_space(6.0);
            widgets::separator(ui);
            ui.add_space(4.0);
            // Binding by name.
            ui.label(egui::RichText::new(tr("Bind variables by name")).font(semibold(11.0)).color(t.text));
            let (field, field_label, kind) = FIELDS[b.field.min(FIELDS.len() - 1)];
            ui.horizontal(|ui| {
                label(ui, tr("Property"));
                let resp = widgets::dropdown(ui, tr(field_label), fw, None);
                crate::right::menu_popup(&resp).show(|ui| {
                    for (i, (_, l, _)) in FIELDS.iter().enumerate() {
                        if crate::right::menu_check(ui, i == b.field, tr(l), "") {
                            b.field = i;
                            b.group = None;
                            ui.close();
                        }
                    }
                });
            });
            let gs = groups(&doc, kind);
            if b.group.as_ref().is_none_or(|g| !gs.contains_key(g)) {
                // The group whose names match the most styles.
                let styles: Vec<(String, String)> = all.iter().map(|(id, n, _)| (id.to_string(), n.to_string())).collect();
                b.group = gs
                    .keys()
                    .max_by_key(|g| {
                        let vars = vars_in(&doc, g, kind);
                        match_names(&styles, &vars).iter().filter(|(_, v)| v.is_some()).count()
                    })
                    .cloned();
            }
            ui.horizontal(|ui| {
                label(ui, tr("From"));
                let now = b.group.as_ref().and_then(|g| gs.get(g)).cloned().unwrap_or_else(|| tr("No variables").to_string());
                let resp = widgets::dropdown(ui, &now, fw, None);
                crate::right::menu_popup(&resp).show(|ui| {
                    for (g, l) in &gs {
                        if crate::right::menu_check(ui, b.group.as_ref() == Some(g), l, "") {
                            b.group = Some(g.clone());
                            ui.close();
                        }
                    }
                });
            });
            let Some(g) = b.group.clone() else {
                widgets::label(ui, tr("No variables of this type in the file."), t.text_secondary);
                return;
            };
            let vars = vars_in(&doc, &g, kind);
            let styles: Vec<(String, String)> = all.iter().map(|(id, n, _)| (id.to_string(), n.to_string())).collect();
            let pairs = match_names(&styles, &vars);
            let name_of = |id: &str| vars.iter().find(|(v, _)| v == id).map(|(_, n)| n.clone()).unwrap_or_default();
            for (sid, vid) in &pairs {
                let sname = styles.iter().find(|(i, _)| i == sid).map(|(_, n)| n.as_str()).unwrap_or("");
                let (right, col) = match vid {
                    Some(v) => (name_of(v), t.text),
                    None => (tr("no match").to_string(), t.text_tertiary),
                };
                ui.label(egui::RichText::new(format!("{sname}  →  {right}")).font(regular(10.5)).color(col));
            }
            let n = pairs.iter().filter(|(_, v)| v.is_some()).count();
            ui.add_space(4.0);
            if widgets::button(ui, &trf("Bind {n}", &[("n", &n.to_string())]), true, n > 0).clicked() && n > 0 {
                bind = Some(
                    pairs
                        .iter()
                        .filter_map(|(s, v)| v.as_ref().map(|v| json!({"command": "style.bind", "params": {"style": s, "field": field, "variable": v}})))
                        .collect(),
                );
            }
        });
    });
    for (key, value) in edits {
        set_all(app, &all, key, value);
    }
    if let Some(commands) = bind {
        app.run("command.batch", json!({"label": "Bind variables to text styles", "commands": commands}));
    }
    let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p))) && !egui::Popup::is_any_open(ctx);
    let typing = ctx.memory(|m| m.focused().is_some());
    if closed || (outside && rect.is_positive()) || (!typing && ctx.input(|i| i.key_pressed(egui::Key::Escape))) {
        close(ctx);
    } else {
        ctx.data_mut(|d| d.insert_temp(state_id(), b));
    }
}

/// The variables of `kind` in group `g` (and its subgroups): (id, name within the group).
fn vars_in(doc: &Document, g: &(String, String), kind: VariableType) -> Vec<(String, String)> {
    doc.variables
        .variables
        .iter()
        .filter(|v| v.kind == kind && v.collection == g.0)
        .filter(|v| g.1.is_empty() || v.name.starts_with(&format!("{}/", g.1)))
        .map(|v| (v.id.clone(), v.name.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    #[test]
    fn styles_match_variables_by_name() {
        let styles = pairs(&[("s1", "Heading/H1"), ("s2", "Heading/H2"), ("s3", "Body/M"), ("s4", "Caption"), ("s5", "Label/M")]);
        let vars = pairs(&[("v1", "font-size/h1"), ("v2", "font-size/H 2"), ("v3", "body/m"), ("v4", "label/m")]);
        let got = match_names(&styles, &vars);
        let want = [("s1", Some("v1")), ("s2", Some("v2")), ("s3", Some("v3")), ("s4", None), ("s5", Some("v4"))];
        for ((s, v), (ws, wv)) in got.iter().zip(want) {
            assert_eq!((s.as_str(), v.as_deref()), (ws, wv));
        }
        // Two equally good names: no guess.
        let tie = match_names(&pairs(&[("s", "M")]), &pairs(&[("a", "x/m"), ("b", "y/m")]));
        assert_eq!(tie[0].1, None);
    }
}
