//! FigCraft Plus: a resolution (DPI) per top-level frame, and PDF/X-4 (`figcraft_engine::dpi`,
//! `figcraft_engine::pdfx`).
//!
//! - **Layout ▸ Resolution** (a top-level frame selected, or a layer in one): the frame's DPI,
//!   typed or picked (72 web · 96 · 150 · 300 print · 600). Changing it scales the frame and its
//!   contents so its physical size stays, and moves what is beside it.
//! - Lengths in mm, cm, in and pt are read at the selection's frame resolution ([`dpi`]).
//! - **Preferences ▸ FigCraft Plus ▸ Resolution of new frames**.
//! - **Export ▸ PDF ▸ PDF/X-4** under *CMYK colour*: the file's CMYK PDFs become PDF/X-4 for coated
//!   FOGRA39 paper.

use std::cell::Cell;

use egui::{Id, Ui};
use figcraft_doc::{Document, NodeId};
use serde_json::json;

use crate::FigApp;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::widgets::{self, FieldEdit, Lead};

thread_local! {
    /// The resolution lengths are read at (the selection's frame's).
    static DPI: Cell<f64> = const { Cell::new(72.0) };
    /// Whether the open file asks for PDF/X (for the export options, which see no document).
    static PDFX: Cell<bool> = const { Cell::new(false) };
    /// A PDF/X switch clicked in the export options, applied next frame.
    static PENDING_PDFX: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The presets the Resolution menu offers.
const PRESETS: [(f64, &str); 5] = [(72.0, "72 · web"), (96.0, "96"), (150.0, "150"), (300.0, "300 · print"), (600.0, "600")];

/// The resolution lengths in mm, cm, in and pt are read at now (the selection's frame's).
pub(crate) fn dpi() -> f64 {
    DPI.with(Cell::get)
}

/// Tests: the resolution lengths are read at.
#[cfg(test)]
pub(crate) fn set_dpi_for_test(d: f64) {
    DPI.with(|x| x.set(d));
}

/// Each frame: the default for new frames, the selection's resolution, a pending PDF/X switch.
pub(crate) fn tick(app: &mut FigApp) {
    figcraft_engine::dpi::set_default(crate::prefs_plus_ui::get().dpi);
    if let Some(on) = PENDING_PDFX.with(Cell::take) {
        app.run("print.pdfx", json!({"on": on}));
    }
    let doc = app.session.doc().ok();
    let d = doc
        .and_then(|d| app.session.selection().first().map(|id| figcraft_engine::dpi::of(d, *id)))
        .unwrap_or(figcraft_engine::dpi::WEB);
    DPI.with(|x| x.set(d));
    PDFX.with(|p| p.set(doc.is_some_and(figcraft_engine::pdfx::on)));
}

/// The top-level frame the selection is in (all of it), if any.
fn frame(doc: &Document, sel: &[NodeId]) -> Option<NodeId> {
    let mut it = sel.iter().map(|i| figcraft_engine::dpi::frame_of(doc, *i));
    let first = it.next()??;
    it.all(|f| f == Some(first)).then_some(first)
}

/// In Layout, after the frame's options: its resolution (a top-level frame, or layers in one).
pub(crate) fn row(app: &mut FigApp, ui: &mut Ui, doc: &Document, sel: &[NodeId]) {
    // Only for the frame itself, so the row doesn't come and go with every layer clicked.
    let Some(f) = frame(doc, sel).filter(|f| sel == [*f]) else { return };
    let now = figcraft_engine::dpi::of_frame(doc, f);
    let t = Tokens::get(ui.ctx());
    crate::right::row(ui, 32.0, |ui| {
        ui.label(egui::RichText::new(tr("Resolution")).font(crate::theme::regular(11.0)).color(t.text_secondary)).on_hover_text(tr(
            "The resolution this frame is made for. Changing it scales the frame and its contents so its size in mm stays; what is beside it moves aside.",
        ));
        let w = 76.0;
        if let Some(FieldEdit::Set(v) | FieldEdit::Scrub(v, true)) = widgets::number_field(ui, Id::unique("figcraftplus_dpi"), Lead::None, Some(now), " dpi", w, 1.0)
            && v.is_finite()
            && (v - now).abs() > 1e-9
        {
            app.run("frame.resolution", json!({"ids": [f.0], "dpi": v.clamp(figcraft_engine::dpi::MIN, figcraft_engine::dpi::MAX)}));
        }
        let resp = widgets::dropdown(ui, "", 28.0, None).on_hover_text(tr("Resolution presets"));
        crate::right::menu_popup(&resp).show(|ui| {
            for (d, label) in PRESETS {
                if crate::right::menu_check(ui, (d - now).abs() < 1e-9, label, "") {
                    app.run("frame.resolution", json!({"ids": [f.0], "dpi": d}));
                    ui.close();
                }
            }
        });
    });
}

/// In the PDF export options, under CMYK colour: PDF/X-4.
pub(crate) fn pdfx_checkbox(ui: &mut Ui, cmyk: bool) {
    let on = PDFX.with(Cell::get);
    let t = Tokens::get(ui.ctx());
    ui.add_enabled_ui(cmyk, |ui| {
        if widgets::checkbox(ui, on && cmyk, tr("PDF/X-4 (coated FOGRA39)")) {
            PENDING_PDFX.with(|p| p.set(Some(!on)));
        }
    })
    .response
    .on_disabled_hover_text(tr("PDF/X-4 needs CMYK colour"));
    if on && cmyk {
        widgets::label(ui, tr("An output intent with its ICC profile; colours converted through it. For this file."), t.text_secondary);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_lengths_follow_the_frames_resolution() {
        set_dpi_for_test(300.0);
        let mm = crate::units_ui::per("mm", 16.0);
        assert!((mm - 300.0 / 25.4).abs() < 1e-9, "a mm at 300 DPI: {mm}");
        assert!((crate::rem_ui::unit(210.0, "mm") - 2480.31).abs() < 0.01, "210 mm typed is 2480 px");
        assert!((crate::units_ui::per("pt", 16.0) - 300.0 / 72.0).abs() < 1e-9, "a point is 1/72 in");
        set_dpi_for_test(72.0);
        assert!((crate::rem_ui::unit(210.0, "mm") - 595.28).abs() < 0.01, "at 72, as before");
    }
}
