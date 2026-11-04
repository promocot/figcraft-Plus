//! FigCraft Plus: its preferences and the Units popover (storytold/figcraft#1325's options A and
//! C, for every unit).
//!
//! - **Preferences ▸ FigCraft Plus**: the layout unit (px, rem, mm, cm, in, pt) and text unit
//!   (px, rem, pt) new and unset documents use, the default root size, and the model the colour
//!   picker opens in (OKLCH, CMYK, …). Kept in `figcraft-plus.json` next to the app, so a
//!   portable copy carries them.
//! - **Units** — the gear in Typography and in Layout, or the Units row of the Page section: this
//!   document's units and root size (kept in the file).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use egui::{Id, Ui};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::FigApp;
use crate::i18n::tr;
use crate::theme::{Tokens, regular, semibold};
use crate::widgets::{self, FieldEdit, Lead};

/// FigCraft Plus's own preferences.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Prefs {
    /// Earlier versions: text shows in rem (else px). Read when `text` is empty.
    pub rem: bool,
    /// The text unit (px, rem or pt) where a document doesn't say.
    #[serde(rename = "textUnit")]
    pub text: String,
    /// The layout unit (px, rem, mm, cm, in or pt) where a document doesn't say.
    pub layout_unit: String,
    /// The root size (px) where a document doesn't set one.
    pub root: f64,
    /// The colour picker's model when it opens ("Hex", "OKLCH", "CMYK", …).
    pub color_model: String,
    /// Decimals number fields show (0–4); values themselves keep every digit.
    pub precision: u8,
    /// The colour picker stays open, on the same slot, when other layers are selected.
    pub picker_follows_selection: bool,
    /// Missing font → the installed family that replaces it (remembered in Missing fonts).
    pub font_replacements: std::collections::BTreeMap<String, String>,
    /// A click on a layer of a multi-selection makes it the key object the others align to.
    pub key_object: bool,
    /// The resolution (DPI) new top-level frames are made at.
    pub dpi: f64,
    /// Export presets (Export ▸ Export with a preset…).
    pub export_presets: Vec<crate::presets_ui::Preset>,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { rem: false, text: String::new(), layout_unit: "px".into(), root: 16.0, color_model: "Hex".into(), precision: 2, picker_follows_selection: true, font_replacements: Default::default(), key_object: true, dpi: 72.0, export_presets: crate::presets_ui::defaults() }
    }
}

impl Prefs {
    /// The text unit (an earlier version's rem switch when none is set).
    pub(crate) fn text_unit(&self) -> String {
        if !self.text.trim().is_empty() {
            self.text.clone()
        } else if self.rem {
            "rem".into()
        } else {
            "px".into()
        }
    }
}

fn path() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|d| d.join("figcraft-plus.json"))
}

fn store() -> MutexGuard<'static, Prefs> {
    static P: OnceLock<Mutex<Prefs>> = OnceLock::new();
    P.get_or_init(|| {
        let p = path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Mutex::new(p)
    })
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
}

/// The preferences now.
pub(crate) fn get() -> Prefs {
    store().clone()
}

/// Decimals fields show now (read often: kept apart from the preferences' lock).
static PRECISION: AtomicU8 = AtomicU8::new(u8::MAX);

/// How many decimals number fields show (0–4).
pub(crate) fn precision() -> usize {
    let mut p = PRECISION.load(Ordering::Relaxed);
    if p == u8::MAX {
        p = get().precision.min(4);
        PRECISION.store(p, Ordering::Relaxed);
    }
    usize::from(p.min(4))
}

pub(crate) fn set(f: impl FnOnce(&mut Prefs)) {
    let mut p = store();
    f(&mut p);
    PRECISION.store(p.precision.min(4), Ordering::Relaxed);
    if let (Some(path), Ok(text)) = (path(), serde_json::to_string_pretty(&*p)) {
        let _ = std::fs::write(path, text);
    }
}

/// Remember that `with` replaces the missing font `family`.
pub(crate) fn remember_font(family: &str, with: &str) {
    let family = family.trim().to_string();
    set(|p| {
        p.font_replacements.retain(|k, _| !k.eq_ignore_ascii_case(&family));
        p.font_replacements.insert(family, with.trim().to_string());
    });
}

/// The index in `models` of the model the picker opens in.
pub(crate) fn default_model(models: &[&str]) -> usize {
    let want = get().color_model;
    models.iter().position(|m| m.eq_ignore_ascii_case(&want)).unwrap_or(0)
}

// ---- Preferences ▸ FigCraft Plus --------------------------------------------------------------------

/// A dropdown of `units` showing `now`; the unit picked.
fn unit_menu(ui: &mut Ui, now: &str, units: &[&'static str]) -> Option<&'static str> {
    let mut out = None;
    let resp = widgets::dropdown(ui, now, 80.0, None);
    crate::right::menu_popup(&resp).show(|ui| {
        for u in units {
            if crate::right::menu_check(ui, now.eq_ignore_ascii_case(u), u, "") {
                out = Some(*u);
                ui.close();
            }
        }
    });
    out
}

const MODELS: [&str; 8] = ["Hex", "RGB", "HSL", "HSB", "LCH", "OKLCH", "CMYK", "CSS"];

/// The FigCraft Plus section of Preferences.
pub(crate) fn preferences(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let p = get();
    ui.add_space(8.0);
    ui.separator();
    ui.label(egui::RichText::new("FigCraft Plus").font(semibold(11.5)).color(t.text));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(tr("Layout unit"));
        if let Some(u) = unit_menu(ui, &p.layout_unit, &crate::units_ui::LAYOUT_UNITS) {
            set(|p| p.layout_unit = u.to_string());
        }
    });
    ui.horizontal(|ui| {
        ui.label(tr("Text unit"));
        if let Some(u) = unit_menu(ui, &p.text_unit(), &crate::units_ui::TEXT_UNITS) {
            set(|p| p.text = u.to_string());
        }
    });
    ui.horizontal(|ui| {
        ui.label(tr("Default root size"));
        if let Some(FieldEdit::Set(v) | FieldEdit::Scrub(v, true)) =
            widgets::number_field(ui, Id::unique("figcraftplus_pref_root"), Lead::None, Some(p.root), "px", 80.0, 1.0)
        {
            set(|p| p.root = v.clamp(1.0, 1000.0));
        }
    });
    widgets::label(ui, tr("Applies to new documents. Each file stores its own units and root size."), t.text_secondary);
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(tr("Decimals shown")).on_hover_text(tr("Fields round what they show; values keep every digit."));
        const DIGITS: [&str; 5] = ["0", "1", "2", "3", "4"];
        let now = DIGITS[usize::from(p.precision.min(4))];
        if let Some(d) = unit_menu(ui, now, &DIGITS) {
            set(|p| p.precision = d.parse().unwrap_or(2));
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(tr("Colour picker opens in"));
        let resp = widgets::dropdown(ui, &p.color_model, 100.0, None);
        crate::right::menu_popup(&resp).show(|ui| {
            for m in MODELS {
                if crate::right::menu_check(ui, p.color_model.eq_ignore_ascii_case(m), m, "") {
                    set(|p| p.color_model = m.to_string());
                    ui.close();
                }
            }
        });
    });
    ui.add_space(4.0);
    // The explanation on hover keeps the section short enough for the window.
    ui.scope(|ui| {
        if widgets::checkbox(ui, p.picker_follows_selection, tr("Picker follows the selection")) {
            set(|p| p.picker_follows_selection = !p.picker_follows_selection);
        }
    })
    .response
    .on_hover_text(tr("The colour picker stays open on the same fill, stroke or effect of the layers you select next. Off: it closes, as in FigCraft."));
    ui.add_space(4.0);
    // The explanation on hover keeps the section short enough for the window.
    ui.scope(|ui| {
        if widgets::checkbox(ui, p.key_object, tr("Key object for aligning")) {
            set(|p| p.key_object = !p.key_object);
        }
    })
    .response
    .on_hover_text(tr("With several layers selected, click one: the others align to it. Click it again to select just it."));
    ui.horizontal(|ui| {
        ui.label(tr("Resolution of new frames")).on_hover_text(tr("72 for the web; 300 for print. Each frame keeps its own, in Layout."));
        if let Some(FieldEdit::Set(v) | FieldEdit::Scrub(v, true)) =
            widgets::number_field(ui, Id::unique("figcraftplus_pref_dpi"), Lead::None, Some(p.dpi), " dpi", 80.0, 1.0)
        {
            set(|p| p.dpi = v.clamp(figcraft_engine::dpi::MIN, figcraft_engine::dpi::MAX));
        }
    });
    crate::presets_ui::prefs(ui);
    if !p.font_replacements.is_empty() {
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr("Remembered font replacements")).font(regular(11.0)).color(t.text));
        for (family, with) in &p.font_replacements {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{family} → {with}")).font(regular(11.0)).color(t.text_secondary));
                if ui.small_button("×").on_hover_text(tr("Forget")).clicked() {
                    let f = family.clone();
                    set(|p| {
                        p.font_replacements.remove(&f);
                    });
                }
            });
        }
    }
}

// ---- Units ---------------------------------------------------------------------------------------

fn popover_id() -> Id {
    Id::unique("figcraftplus_units")
}

/// Width of the Units popover.
const W: f32 = 280.0;

/// Open or close the Units popover, at `at` (its top left corner) or beside the properties panel
/// level with the control that opened it (once per frame: a panel may be laid out twice).
fn toggle(ui: &Ui, at: Option<egui::Pos2>) {
    let ctx = ui.ctx();
    let frame = ctx.cumulative_frame_nr();
    if ctx.data(|d| d.get_temp::<u64>(popover_id().with("toggled"))) == Some(frame) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(popover_id().with("toggled"), frame));
    let open: Option<egui::Pos2> = ctx.data(|d| d.get_temp(popover_id()));
    ctx.data_mut(|d| match open {
        Some(_) => {
            d.remove_temp::<egui::Pos2>(popover_id());
        }
        None => {
            let p = at.unwrap_or_else(|| egui::pos2(ui.max_rect().left() - W - 12.0, (ui.cursor().top() - 40.0).max(48.0)));
            d.insert_temp(popover_id(), p);
        }
    });
}

/// Open the Units popover at `at` (from a field's unit menu).
pub(crate) fn open_units(ctx: &egui::Context, at: egui::Pos2) {
    let now = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| {
        d.insert_temp(popover_id(), at);
        d.insert_temp(popover_id().with("toggled"), now);
    });
}

/// A section header's Units button was clicked (`clicked`): open or close the popover; draw it
/// while open. Called by the Typography and Layout sections.
pub(crate) fn units_popover(app: &mut FigApp, ui: &mut Ui, clicked: bool) {
    if clicked {
        toggle(ui, None);
        return;
    }
    draw(app, ui.ctx());
}

/// The Page section's Units row: this document's units as a button that opens the popover.
pub(crate) fn units_row(app: &mut FigApp, ui: &mut Ui) {
    let Ok(doc) = app.session.doc() else { return };
    let summary = format!(
        "{} · {} · 1rem = {}px",
        crate::units_ui::doc_layout(doc),
        crate::units_ui::doc_text(doc),
        widgets::fmt_num(crate::rem_ui::doc_root(doc))
    );
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let resp = widgets::dropdown(ui, &summary, 200.0, None);
        if resp.clicked() {
            toggle(ui, Some(egui::pos2(resp.rect.left() - W - 28.0, resp.rect.top() - 60.0)));
        }
    });
    draw(app, ui.ctx());
}

/// The popover, while open.
fn draw(app: &mut FigApp, ctx: &egui::Context) {
    let Some(at) = ctx.data(|d| d.get_temp::<egui::Pos2>(popover_id())) else { return };
    // Draw it once a frame however many sections ask.
    let frame = ctx.cumulative_frame_nr();
    if ctx.data(|d| d.get_temp::<u64>(popover_id().with("drawn"))) == Some(frame) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(popover_id().with("drawn"), frame));
    let Ok(doc) = app.session.doc() else { return };
    let page = doc.pages.first().copied();
    let root = crate::rem_ui::doc_root(doc);
    let layout = crate::units_ui::doc_layout(doc);
    let text = crate::units_ui::doc_text(doc);
    let t = Tokens::get(ctx);
    let area_id = popover_id().with("area");
    let (close, rect) = crate::popover_ui::show(ctx, area_id, tr("Units"), W, crate::popover::Open::At(at), |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 14 }).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 6.0);
            let caption = |ui: &mut Ui, s: &str| {
                ui.label(egui::RichText::new(s).font(regular(11.0)).color(t.text));
            };
            let hint = |ui: &mut Ui, s: &str| {
                ui.label(egui::RichText::new(s).font(regular(10.5)).color(t.text_secondary));
            };
            hint(ui, tr("Applies to this document"));
            ui.add_space(4.0);
            caption(ui, tr("Layout"));
            let active = crate::units_ui::LAYOUT_UNITS.iter().position(|u| *u == layout).unwrap_or(0);
            let picked = ui.push_id("figcraftplus_layout_unit", |ui| widgets::segmented_text(ui, &crate::units_ui::LAYOUT_UNITS, active)).inner;
            if let (Some(i), Some(page)) = (picked, page) {
                let u = crate::units_ui::LAYOUT_UNITS[i];
                app.run("node.set", json!({"ids": [page], "props": {"pluginData": {crate::units_ui::LAYOUT_KEY: u}}}));
            }
            hint(ui, tr("Position, size, radius, spacing, strokes, effects"));
            ui.add_space(6.0);
            caption(ui, tr("Text"));
            let active = crate::units_ui::TEXT_UNITS.iter().position(|u| *u == text).unwrap_or(0);
            let picked = ui.push_id("figcraftplus_text_unit", |ui| widgets::segmented_text(ui, &crate::units_ui::TEXT_UNITS, active)).inner;
            if let (Some(i), Some(page)) = (picked, page) {
                let u = crate::units_ui::TEXT_UNITS[i];
                app.run("node.set", json!({"ids": [page], "props": {"pluginData": {crate::rem_ui::UNIT_KEY: u}}}));
            }
            hint(ui, tr("Font size, line height, letter and paragraph spacing"));
            ui.add_space(6.0);
            caption(ui, tr("Root size (1rem =)"));
            if let (Some(FieldEdit::Set(v) | FieldEdit::Scrub(v, true)), Some(page)) =
                (widgets::number_field(ui, Id::unique("figcraftplus_rem_root"), Lead::None, Some(root), "px", W - 32.0, 1.0), page)
            {
                let v = v.clamp(1.0, 1000.0);
                app.run("node.set", json!({"ids": [page], "props": {"pluginData": {crate::rem_ui::PAGE_KEY: format!("{v}")}}}));
            }
            hint(ui, &format!("1rem = {}px, {}", widgets::fmt_num(root), tr("stored in the file")));
            hint(ui, tr("Default for new documents: Preferences"));
        });
    });
    // A click outside closes it (not the click on a Units button that just toggled it); Esc too.
    let toggled_now = ctx.data(|d| d.get_temp::<u64>(popover_id().with("toggled"))) == Some(frame);
    let outside = !toggled_now
        && ctx.input(|i| i.pointer.primary_clicked())
        && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| !rect.contains(p));
    let reselected = crate::popover_ui::selection_changed(ctx, popover_id(), &app.session.selection());
    if close || outside || reselected || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        ctx.data_mut(|d| d.remove_temp::<egui::Pos2>(popover_id()));
        crate::popover_ui::forget_selection(ctx, popover_id());
        crate::popover_ui::forget(area_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_read_and_pick_the_picker_model() {
        let p: Prefs = serde_json::from_str(r#"{"rem": true, "colorModel": "OKLCH"}"#).unwrap();
        assert_eq!((p.rem, p.root, p.color_model.as_str()), (true, 16.0, "OKLCH"), "missing keys take defaults");
        assert!(p.picker_follows_selection, "the picker follows the selection unless turned off");
        let off: Prefs = serde_json::from_str(r#"{"pickerFollowsSelection": false}"#).unwrap();
        assert!(!off.picker_follows_selection);
        assert_eq!((p.text_unit().as_str(), p.layout_unit.as_str()), ("rem", "px"), "an earlier rem switch is the text unit");
        assert_eq!(MODELS.iter().position(|m| m.eq_ignore_ascii_case(&p.color_model)), Some(5));
        let p: Prefs = serde_json::from_str(r#"{"textUnit": "pt", "layoutUnit": "mm"}"#).unwrap();
        assert_eq!((p.text_unit().as_str(), p.layout_unit.as_str()), ("pt", "mm"));
    }
}
