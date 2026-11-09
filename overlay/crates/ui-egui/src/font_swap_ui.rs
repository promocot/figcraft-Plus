//! FigCraft Plus: replacing missing fonts, and remembering the replacements.
//!
//! FigCraft's Missing fonts window lists the fonts a file uses that aren't installed and downloads
//! the ones the font store has. Plus adds **Replace…** on each: pick an installed family and every
//! text using the missing one — whole layers, single runs, text styles — switches to it, one undo
//! step. With *Remember replacements* on (the default), the choice is kept in
//! `figcraft-plus.json`: the next file that uses that font gets the same replacement as it opens,
//! with a note saying so. Preferences ▸ FigCraft Plus lists the remembered ones, each removable.

use std::collections::BTreeMap;

use egui::{Id, Ui};
use figcraft_doc::{Document, NodeKind, StyleKind};
use serde_json::{Value, json};

use crate::FigApp;
use crate::i18n::{tr, trf};
use crate::theme::{Tokens, regular};
use crate::widgets;

fn pending_id() -> Id {
    Id::unique("figcraftplus_font_swap")
}

/// The commands that move every use of `family` (any case) in `doc` to `with`: text runs and
/// layers (`text.style` per run, so other runs keep their fonts) and text styles.
pub(crate) fn commands(doc: &Document, family: &str, with: &str) -> Vec<Value> {
    let same = |f: &str| f.trim().eq_ignore_ascii_case(family.trim());
    let mut out = Vec::new();
    for n in doc.nodes() {
        let NodeKind::Text(t) = &n.kind else { continue };
        let spans = t.spans();
        if spans.is_empty() && same(&t.style.font_family) {
            out.push(json!({"command": "text.style", "params": {"id": n.id, "fontFamily": with}}));
            continue;
        }
        if same(&t.style.font_family) {
            // The layer's own font too: all of it, then the runs in other fonts back.
            out.push(json!({"command": "text.style", "params": {"id": n.id, "fontFamily": with}}));
            for (start, end, st) in spans {
                if !same(&st.font_family) {
                    out.push(json!({"command": "text.style", "params": {"id": n.id, "start": start, "end": end, "fontFamily": st.font_family}}));
                }
            }
            continue;
        }
        for (start, end, st) in spans {
            if same(&st.font_family) {
                out.push(json!({"command": "text.style", "params": {"id": n.id, "start": start, "end": end, "fontFamily": with}}));
            }
        }
    }
    for s in figcraft_engine::library::local_styles(doc) {
        if let StyleKind::Text { style, paragraph } = &s.kind
            && same(&style.font_family)
        {
            // With its paragraph settings, which `style.update` would reset otherwise.
            let mut v = crate::styles_plus_ui::text_value(style, paragraph);
            v["font_family"] = json!(with);
            out.push(json!({"command": "style.update", "params": {"style": s.id, "value": v}}));
        }
    }
    out
}

/// Replace `family` with `with` in the file in front (one undo step); remember it when asked.
pub(crate) fn replace(app: &mut FigApp, family: &str, with: &str, remember: bool) {
    let Ok(doc) = app.session.doc() else { return };
    let list = commands(doc, family, with);
    if !list.is_empty() {
        app.run("command.batch", json!({"label": "Replace font", "commands": list}));
    }
    if remember {
        crate::prefs_plus_ui::remember_font(family, with);
    }
    app.toast(trf("Replaced {family} with {with}", &[("family", &family), ("with", &with)]));
}

/// The remembered replacements of `missing` that are installed now.
pub(crate) fn remembered_for(missing: &[String]) -> BTreeMap<String, String> {
    let known = crate::prefs_plus_ui::get().font_replacements;
    missing
        .iter()
        .filter_map(|m| {
            let with = known.iter().find(|(k, _)| k.eq_ignore_ascii_case(m)).map(|(_, v)| v.clone())?;
            figcraft_text::has_family(&with).then(|| (m.clone(), with))
        })
        .collect()
}

/// A file `uid` was just listed for missing fonts: apply the remembered replacements (when it is
/// the file in front).
pub(crate) fn auto(app: &mut FigApp, uid: u64) {
    let Ok(ds) = app.session.ds() else { return };
    if ds.uid != uid {
        return;
    }
    let missing: Vec<String> = figcraft_engine::fonts::missing(&ds.doc).into_iter().map(|m| m.family).collect();
    let swaps = remembered_for(&missing);
    if swaps.is_empty() {
        return;
    }
    let Ok(doc) = app.session.doc() else { return };
    let list: Vec<Value> = swaps.iter().flat_map(|(f, w)| commands(doc, f, w)).collect();
    if !list.is_empty() {
        app.run("command.batch", json!({"label": "Replace fonts (remembered)", "commands": list}));
    }
    let text = swaps.iter().map(|(f, w)| format!("{f} → {w}")).collect::<Vec<_>>().join(", ");
    app.toast(trf("Remembered font replacements: {list}", &[("list", &text)]));
}

/// A Missing fonts row's **Replace…** menu (`family`: the missing one).
pub(crate) fn row_button(ui: &mut Ui, family: &str) {
    let resp = widgets::button(ui, tr("Replace…"), false, true);
    let query_id = pending_id().with(("query", family));
    crate::right::menu_popup(&resp).show(|ui| {
        let mut q: String = ui.data(|d| d.get_temp(query_id)).unwrap_or_default();
        let r = ui.add(egui::TextEdit::singleline(&mut q).hint_text(tr("Search fonts")).desired_width(200.0));
        if resp.clicked() {
            r.request_focus();
        }
        ui.data_mut(|d| d.insert_temp(query_id, q.clone()));
        let ql = q.to_lowercase();
        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            for f in figcraft_text::family_names().into_iter().filter(|f| ql.is_empty() || f.to_lowercase().contains(&ql)).take(400) {
                if crate::right::menu_item(ui, &f, "") {
                    ui.data_mut(|d| d.insert_temp(pending_id(), (family.to_string(), f.clone())));
                    ui.close();
                }
            }
        });
    });
}

/// Under the Missing fonts list: whether replacements are remembered.
pub(crate) fn remember_toggle(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let on = remember_on(ui.ctx());
    if widgets::checkbox(ui, on, tr("Remember replacements for other files")) {
        ui.data_mut(|d| d.insert_temp(pending_id().with("remember"), !on));
    }
    ui.label(egui::RichText::new(tr("Kept in Preferences ▸ FigCraft Plus, where you can remove them.")).font(regular(10.5)).color(t.text_secondary));
}

fn remember_on(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(pending_id().with("remember"))).unwrap_or(true)
}

/// After the Missing fonts window drew: the replacement picked in it.
pub(crate) fn act(app: &mut FigApp, ctx: &egui::Context) {
    let Some((family, with)) = ctx.data(|d| d.get_temp::<(String, String)>(pending_id())) else { return };
    ctx.data_mut(|d| d.remove::<(String, String)>(pending_id()));
    replace(app, &family, &with, remember_on(ctx));
    crate::dialogs::missing_fonts::request(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_engine::Session;

    #[test]
    fn a_missing_font_is_replaced_in_runs_layers_and_styles() {
        let mut s = Session::new();
        let a = s.execute("text.create", &json!({"x": 0, "y": 0, "text": "Hello world"})).unwrap()["id"].as_u64().unwrap();
        s.execute("text.style", &json!({"id": a, "fontFamily": "Gone Sans"})).unwrap();
        s.execute("text.style", &json!({"id": a, "start": 6, "end": 11, "fontFamily": "Inter"})).unwrap();
        s.execute("style.create", &json!({"type": "text", "name": "Body", "value": {"fontFamily": "Gone Sans", "fontStyle": "Regular", "fontSize": 14}})).unwrap();
        let doc = s.doc().unwrap();
        let list = commands(doc, "gone sans", "Roboto");
        assert!(list.iter().any(|c| c["params"]["start"] == 6 && c["params"]["fontFamily"] == "Inter"), "the Inter run stays Inter: {list:?}");
        assert!(list.iter().any(|c| c["command"] == "style.update"), "the text style too");
        for c in list {
            s.execute(c["command"].as_str().unwrap(), &c["params"]).unwrap();
        }
        let doc = s.doc().unwrap();
        let NodeKind::Text(t) = &doc.node(figcraft_doc::NodeId(a)).unwrap().kind else { panic!() };
        let fams: Vec<&str> = t.spans().iter().map(|(_, _, st)| st.font_family.as_str()).collect();
        assert_eq!(fams, ["Roboto", "Inter"]);
        let left: Vec<String> = figcraft_engine::fonts::missing(doc).into_iter().map(|m| format!("{} {}", m.family, m.layers)).collect();
        let styles: Vec<String> = doc.styles.iter().map(|s| format!("{:?}", s.kind)).collect();
        assert!(left.iter().all(|m| !m.to_lowercase().starts_with("gone sans")), "{left:?} {styles:?}");
    }
}
