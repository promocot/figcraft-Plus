//! FigCraft Plus: export presets (`figcraft_engine::cmd::export_presets`).
//!
//! In the Export section, **Presets…** exports the selected layers with a saved set of sizes and
//! formats, the files named by its pattern (`{name}_{w}x{h}`…), into a folder (a zip in the
//! browser). *Save these settings as a preset* keeps the selected layer's own export settings as a
//! new one. Presets live in Preferences ▸ FigCraft Plus, where they are renamed, their pattern
//! changed, and removed.

use egui::{Id, Ui};
use figcraft_doc::{Document, NodeId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::FigApp;
use crate::i18n::{tr, trn};
use crate::prefs_plus_ui::{get, set};
use crate::theme::Tokens;
use crate::widgets;

/// One preset: a name, the files' name pattern, and its sizes and formats (export settings as
/// `export.add` takes them).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Preset {
    pub name: String,
    pub pattern: String,
    pub items: Vec<Value>,
}

impl Default for Preset {
    fn default() -> Self {
        Preset { name: String::new(), pattern: figcraft_engine::cmd::export_presets::DEFAULT_PATTERN.into(), items: Vec::new() }
    }
}

/// The presets a new install starts with.
pub(crate) fn defaults() -> Vec<Preset> {
    let p = |name: &str, pattern: &str, items: Value| Preset { name: name.into(), pattern: pattern.into(), items: items.as_array().cloned().unwrap_or_default() };
    vec![
        p("Web · PNG 1x, 2x, 3x", "{name}@{scale}", json!([{"size": "1x", "format": "png"}, {"size": "2x", "format": "png"}, {"size": "3x", "format": "png"}])),
        p("Social · 1080 px PNG and JPG", "{name}_{w}x{h}", json!([{"size": "1080w", "format": "png"}, {"size": "1080w", "format": "jpg"}])),
        p("Every format", "{format}/{name}", json!([{"size": "1x", "format": "png"}, {"size": "1x", "format": "jpg"}, {"format": "svg"}, {"format": "pdf"}])),
        p("Print PDF", "{name}", json!([{"format": "pdf", "cmyk": true, "bleed": 8.5, "cropMarks": true}])),
    ]
}

/// "1x PNG, 2x PNG, SVG".
pub(crate) fn summary(items: &[Value]) -> String {
    items
        .iter()
        .map(|it| {
            let f = it["format"].as_str().unwrap_or("png").to_ascii_uppercase();
            match it["size"].as_str() {
                Some(s) if !matches!(f.as_str(), "SVG" | "PDF") => format!("{s} {f}"),
                _ => f,
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The selected layer's own export settings, as preset items.
fn own_items(doc: &Document, sel: &[NodeId]) -> Vec<Value> {
    let Some(n) = sel.first().and_then(|i| doc.node(*i)) else { return Vec::new() };
    n.exports.iter().map(crate::export_panel::setting_params).collect()
}

/// Export `ids` with `preset`: into a folder (a zip in the browser).
fn run(app: &mut FigApp, ids: &[NodeId], preset: &Preset) {
    let params = json!({"ids": ids, "items": preset.items, "pattern": preset.pattern});
    if app.services.downloads() {
        let mut p = params;
        p["zip"] = json!(true);
        let name = format!("{}.zip", preset.name);
        if let Some(r) = app.run("export.preset", p)
            && app.download(&name, &r)
        {
            app.toast(trn("Exported {n} files", r["files"].as_array().map_or(0, Vec::len), &[]));
        }
        return;
    }
    app.pick_folder(tr("Export"), move |app, dir| {
        let mut p = params;
        p["dir"] = json!(dir.to_string_lossy());
        if let Some(r) = app.run("export.preset", p) {
            app.toast(trn("Exported {n} files", r["files"].as_array().map_or(0, Vec::len), &[]));
        }
    });
}

/// In the Export section's header, just drawn above, left of its `+`: the Presets menu.
pub(crate) fn header(app: &mut FigApp, ui: &mut Ui, doc: &Document, sel: &[NodeId]) {
    if sel.is_empty() {
        return;
    }
    let presets = get().export_presets;
    let own = own_items(doc, sel);
    let t = Tokens::get(ui.ctx());
    // The header is the 40 pt row that ends where the cursor now starts.
    let bottom = ui.cursor().min.y - ui.spacing().item_spacing.y;
    let label = tr("Presets…");
    let galley = ui.painter().layout_no_wrap(label.to_string(), crate::theme::regular(11.0), t.text);
    let w = galley.size().x + 16.0;
    let right = ui.max_rect().max.x - widgets::PAD_RIGHT - 24.0 - 4.0;
    let r = egui::Rect::from_min_size(egui::pos2(right - w, bottom - 20.0 - 12.0), egui::vec2(w, 24.0));
    let resp = ui.interact(r, Id::unique("figcraftplus_presets_button"), egui::Sense::click()).on_hover_text(tr("Export with a preset: several sizes and formats, files named by a pattern"));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    if resp.hovered() || egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp)) {
        ui.painter().rect_filled(r, egui::CornerRadius::same(5), t.hover);
    }
    ui.painter().galley(egui::pos2(r.min.x + 8.0, r.center().y - galley.size().y / 2.0), galley, t.text);
    let mut chosen: Option<Preset> = None;
    let mut save = false;
    let mut edit = false;
    crate::right::menu_popup(&resp).show(|ui| {
        for p in &presets {
            if crate::right::menu_item(ui, &p.name, &summary(&p.items)) {
                chosen = Some(p.clone());
                ui.close();
            }
        }
        ui.separator();
        if !own.is_empty() && crate::right::menu_item(ui, tr("Save these settings as a preset"), "") {
            save = true;
            ui.close();
        }
        if crate::right::menu_item(ui, tr("Edit presets…"), "") {
            edit = true;
            ui.close();
        }
    });
    if let Some(p) = chosen {
        run(app, sel, &p);
    }
    if save {
        let name = format!("{} {}", tr("Preset"), presets.len() + 1);
        set(|p| p.export_presets.push(Preset { name: name.clone(), pattern: figcraft_engine::cmd::export_presets::DEFAULT_PATTERN.into(), items: own.clone() }));
        app.toast(format!("{} · {}", tr("Saved the preset"), name));
    }
    if edit {
        app.ui.preferences = true;
    }
}

/// In Preferences ▸ FigCraft Plus: every preset's name and pattern, and remove.
pub(crate) fn prefs(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let presets = get().export_presets;
    let mut remove = None;
    egui::CollapsingHeader::new(egui::RichText::new(tr("Export presets")).font(crate::theme::regular(11.0)).color(t.text)).default_open(false).show(ui, |ui| {
        widgets::label(ui, tr("Patterns: {name} {frame} {page} {w} {h} {scale} {format} {n} {date}; a / makes folders"), t.text_secondary);
        for (i, p) in presets.iter().enumerate() {
            ui.horizontal(|ui| {
                if let Some(v) = widgets::text_field(ui, Id::unique(("figcraftplus_preset_name", i)), &p.name, 150.0, tr("Name")) {
                    let v = v.trim().to_string();
                    if !v.is_empty() {
                        set(|x| {
                            if let Some(q) = x.export_presets.get_mut(i) {
                                q.name = v;
                            }
                        });
                    }
                }
                if let Some(v) = widgets::text_field(ui, Id::unique(("figcraftplus_preset_pattern", i)), &p.pattern, 150.0, "{name}_{w}x{h}") {
                    set(|x| {
                        if let Some(q) = x.export_presets.get_mut(i) {
                            q.pattern = v.trim().to_string();
                        }
                    });
                }
                if ui.small_button("×").on_hover_text(tr("Remove")).clicked() {
                    remove = Some(i);
                }
            })
            .response
            .on_hover_text(summary(&p.items));
        }
    });
    if let Some(i) = remove {
        set(|x| {
            if i < x.export_presets.len() {
                x.export_presets.remove(i);
            }
        });
    }
}
