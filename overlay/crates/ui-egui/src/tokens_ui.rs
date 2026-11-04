//! FigCraft Plus: variables to and from CSS, the way code writes them (`variable.exportCss`,
//! `variable.importCss`).
//!
//! - **Export ▸ CSS for code…** in the Variables editor opens a small popover: colours as Hex or
//!   OKLCH, sizes in px or rem (of the file's root size), and formulas written as CSS (relative
//!   colours, `color-mix()`, `calc()`), then a save dialog.
//! - **Import** becomes a menu: the variables JSON (FigCraft's own) or **CSS custom properties…**:
//!   a stylesheet's `--tokens`, its `:root` / theme blocks as modes.

use egui::{Id, Pos2, Response, Ui, pos2};
use serde_json::json;

use crate::FigApp;
use crate::i18n::{tr, trf, trn};
use crate::theme::{Tokens, regular};
use crate::widgets;

/// The export popover's choices (kept for the session).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Choice {
    oklch: bool,
    rem: bool,
    formulas: bool,
}

fn open_id() -> Id {
    Id::unique("figcraftplus_css_export")
}

/// Popover width.
const W: f32 = 260.0;

/// Open the export popover (from the Export menu).
pub(crate) fn open_export(ctx: &egui::Context) {
    let now = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| d.insert_temp(open_id().with("open"), now));
}

fn close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove_temp::<u64>(open_id().with("open")));
    crate::popover_ui::forget(open_id());
}

/// The export popover under the Export button (`export`), while open.
pub(crate) fn export_popover(app: &mut FigApp, ctx: &egui::Context, export: &Response) {
    let Some(opened) = ctx.data(|d| d.get_temp::<u64>(open_id().with("open"))) else { return };
    let Ok(doc) = app.session.doc() else { return };
    let page = doc.pages.first().and_then(|p| doc.node(*p));
    let code_rem = page.and_then(|p| p.plugin_data.get(crate::units_ui::CODE_KEY)).is_some_and(|u| u == "rem");
    let mut c: Choice = ctx.data(|d| d.get_temp(open_id())).unwrap_or(Choice { oklch: true, rem: code_rem, formulas: true });
    let root = crate::rem_ui::doc_root(doc);
    let t = Tokens::get(ctx);
    let at: Pos2 = pos2(export.rect.right() - W, export.rect.bottom() + 6.0);
    let mut go = false;
    let (closed, rect) = crate::popover_ui::show(ctx, open_id(), tr("CSS for code"), W, crate::popover::Open::At(at), |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 14 }).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 6.0);
            let caption = |ui: &mut Ui, s: &str| {
                ui.label(egui::RichText::new(s).font(regular(11.0)).color(t.text));
            };
            caption(ui, tr("Colours"));
            if let Some(i) = ui.push_id("figcraftplus_css_colors", |ui| widgets::segmented_text(ui, &["Hex", "OKLCH"], usize::from(c.oklch))).inner {
                c.oklch = i == 1;
            }
            caption(ui, tr("Sizes"));
            if let Some(i) = ui.push_id("figcraftplus_css_unit", |ui| widgets::segmented_text(ui, &["px", "rem"], usize::from(c.rem))).inner {
                c.rem = i == 1;
            }
            if c.rem {
                ui.label(egui::RichText::new(format!("1rem = {}px", widgets::fmt_num(root))).font(regular(10.5)).color(t.text_secondary));
            }
            ui.add_space(2.0);
            if widgets::checkbox(ui, c.formulas, tr("Formulas as CSS")) {
                c.formulas = !c.formulas;
            }
            ui.label(egui::RichText::new(tr("Derived colours as relative colours, mixes as color-mix(), calculated numbers as calc().")).font(regular(10.5)).color(t.text_secondary));
            ui.add_space(6.0);
            go = widgets::button(ui, tr("Export…"), true, true).clicked();
        });
    });
    ctx.data_mut(|d| d.insert_temp(open_id(), c));
    let now = ctx.cumulative_frame_nr();
    let outside = now > opened + 1
        && ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p) && !export.rect.contains(p)));
    if closed || outside || go || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close(ctx);
    }
    if go {
        export_css(app, c);
    }
}

fn export_css(app: &mut FigApp, c: Choice) {
    let params = json!({"colors": if c.oklch { "oklch" } else { "hex" }, "unit": if c.rem { "rem" } else { "px" }, "formulas": c.formulas});
    let name = app.session.doc().map(|d| format!("{}.tokens.css", d.name)).unwrap_or_else(|_| "tokens.css".into());
    if app.services.downloads() {
        let Some(r) = app.run("variable.exportCss", params) else { return };
        let text = r["text"].as_str().unwrap_or_default();
        if app.services.download(&name, text.as_bytes()) {
            app.toast(trf("Exported {file}", &[("file", &name)]));
        }
        return;
    }
    app.pick_save(tr("Export variables"), &name, &["css"], move |app, p| {
        let mut params = params;
        params["path"] = json!(p.to_string_lossy());
        if app.run("variable.exportCss", params).is_some() {
            let file = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            app.toast(trf("Exported {file}", &[("file", &file)]));
        }
    });
}

/// The Import button's menu: FigCraft's variables JSON (`json`: its own dialog), or CSS.
pub(crate) fn import_menu(app: &mut FigApp, import: &Response, json: fn(&mut FigApp)) {
    let mut pick: Option<bool> = None;
    let mut dtcg = false;
    crate::right::menu_popup(import).show(|ui| {
        if crate::right::menu_item(ui, tr("Variables JSON…"), "") {
            pick = Some(false);
            ui.close();
        }
        if crate::right::menu_item(ui, tr("CSS custom properties…"), "") {
            pick = Some(true);
            ui.close();
        }
        if crate::right::menu_item(ui, tr("Design tokens (DTCG)…"), "") {
            dtcg = true;
            ui.close();
        }
    });
    match pick {
        Some(false) => json(app),
        Some(true) => import_css(app),
        None if dtcg => import_dtcg(app),
        None => {}
    }
}

/// Export ▸ Design tokens (DTCG)…: this file's variables as DTCG JSON.
pub(crate) fn export_dtcg(app: &mut FigApp) {
    let name = app.session.doc().map(|d| format!("{}.tokens.json", d.name)).unwrap_or_else(|_| "tokens.json".into());
    if app.services.downloads() {
        let Some(r) = app.run("variable.exportDtcg", json!({})) else { return };
        let text = r["text"].as_str().unwrap_or_default();
        if app.services.download(&name, text.as_bytes()) {
            app.toast(trf("Exported {file}", &[("file", &name)]));
        }
        return;
    }
    app.pick_save(tr("Export variables"), &name, &["json"], move |app, p| {
        if app.run("variable.exportDtcg", json!({"path": p.to_string_lossy()})).is_some() {
            let file = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
            app.toast(trf("Exported {file}", &[("file", &file)]));
        }
    });
}

fn import_dtcg(app: &mut FigApp) {
    app.pick_open(tr("Import variables"), &["json"], |app, p| {
        let Some(r) = app.run("variable.importDtcg", json!({"json": p.to_string_lossy()})) else { return };
        let (added, updated) = (r["variables"].as_u64().unwrap_or(0), r["updated"].as_u64().unwrap_or(0));
        let skipped = r["warnings"].as_array().map_or(0, Vec::len);
        let mut msg = trn("Imported {n} new variables, updated {updated}", added as usize, &[("updated", &updated)]);
        if skipped > 0 {
            msg = format!("{msg} {}", trn("({n} warnings: some entries were skipped)", skipped, &[]));
        }
        app.toast(msg);
    });
}

fn import_css(app: &mut FigApp) {
    app.pick_open(tr("Import variables"), &["css"], |app, p| {
        let collection = p.file_stem().map(|s| s.to_string_lossy().split('.').next().unwrap_or("Tokens").to_string()).unwrap_or_else(|| "Tokens".into());
        let Some(r) = app.run("variable.importCss", json!({"css": p.to_string_lossy(), "collection": collection})) else { return };
        let (added, updated) = (r["variables"].as_u64().unwrap_or(0), r["updated"].as_u64().unwrap_or(0));
        let skipped = r["warnings"].as_array().map_or(0, Vec::len);
        let mut msg = trn("Imported {n} new variables, updated {updated}", added as usize, &[("updated", &updated)]);
        if skipped > 0 {
            msg = format!("{msg} {}", trn("({n} warnings: some entries were skipped)", skipped, &[]));
        }
        app.toast(msg);
    });
}
