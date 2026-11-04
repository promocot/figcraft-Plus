//! FigCraft Plus: line height in % from a number variable (`figcraft_engine::leading`).
//!
//! - The variable picker of a line height (a text layer's, a text style's) has a **px | %** switch
//!   under its search field: px binds the variable FigCraft's way, % makes the line height follow it
//!   as a percentage of the font size (140 → 140%, 1.4 → 140%). The rows then show their values as
//!   percentages (this file's variables only). The switch opens on how the field is bound now, else on the last choice (px, as FigCraft binds, until % is picked).
//! - A line height that follows a variable shows its chip, as a bound field does, with a `%`: a
//!   click on it opens the picker, its detach button keeps the value and ends the link.

use std::cell::Cell;

use egui::{Align2, CornerRadius, Id, Rect, Sense, Ui, pos2, vec2};
use figcraft_doc::{Document, NodeId};
use serde_json::json;

use crate::FigApp;
use crate::i18n::tr;
use crate::theme::{Tokens, regular};
use crate::variable_ui::{Pick, Target};
use crate::widgets::{self, BindEvent, Lead};

/// The field FigCraft binds the line height under.
pub(crate) const FIELD: &str = "lineHeight";

thread_local! {
    /// While the open picker is a line height's: whether it binds in % (Some) — read by the card's
    /// body to draw the switch and by the rows for their values.
    static ACTIVE: Cell<Option<bool>> = const { Cell::new(None) };
}

fn remembered_id() -> Id {
    Id::unique("figcraftplus_leading_percent")
}

/// Whether `target` is a line height (a text layer's or a text style's).
pub(crate) fn is_line_height(target: &Target) -> bool {
    matches!(target, Target::Bind { field, .. } | Target::Style { field, .. } if field == FIELD)
}

/// The variable the line height of every one of `ids` follows as %, if they agree.
pub(crate) fn layer_rule(doc: &Document, ids: &[NodeId]) -> Option<String> {
    let mut it = ids.iter().map(|i| figcraft_engine::leading::layer_rule(doc, *i));
    let first = it.next().flatten()?;
    it.all(|r| r.as_deref() == Some(first.as_str())).then_some(first)
}

/// Whether the picker for `target` opens in % (how it is bound now, else the last choice).
fn opens_in_percent(ctx: &egui::Context, doc: &Document, target: &Target) -> bool {
    let (rule, px) = match target {
        Target::Bind { ids, field } if field == FIELD => (
            layer_rule(doc, ids).is_some(),
            crate::variable_ui::bound_name(doc, ids, FIELD).is_some(),
        ),
        Target::Style { style, field } if field == FIELD => (
            figcraft_engine::leading::style_rule(doc, style).is_some(),
            doc.style(style).is_some_and(|s| s.bound.contains_key(FIELD)),
        ),
        _ => (false, false),
    };
    if rule {
        return true;
    }
    if px {
        return false;
    }
    ctx.data(|d| d.get_temp::<bool>(remembered_id())).unwrap_or(false)
}

/// The picker for `target` is about to draw: switch the line-height mode on (or off for any other
/// target). Returns whether it binds in %.
pub(crate) fn begin(ctx: &egui::Context, doc: &Document, target: &Target) -> Option<bool> {
    let state = is_line_height(target).then(|| {
        let key = remembered_id().with("open");
        // The mode lives while this picker is open; a new one starts from how the field is bound.
        ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or_else(|| {
            let p = opens_in_percent(ctx, doc, target);
            ctx.data_mut(|d| d.insert_temp(key, p));
            p
        })
    });
    ACTIVE.with(|a| a.set(state));
    state
}

/// The picker closed: its mode is forgotten (the last choice stays as the default).
pub(crate) fn end(ctx: &egui::Context) {
    ACTIVE.with(|a| a.set(None));
    ctx.data_mut(|d| d.remove::<bool>(remembered_id().with("open")));
}

/// The picker has drawn this frame: other lists drawn later show their numbers as usual.
pub(crate) fn frame_done() {
    ACTIVE.with(|a| a.set(None));
}

/// Whether the rows being drawn are a line height's in % (their values shown as percentages).
pub(crate) fn percent_rows() -> bool {
    ACTIVE.with(Cell::get) == Some(true)
}

/// A number in the list as a line height percentage.
pub(crate) fn row_value(x: f64) -> String {
    format!("{}%", figcraft_doc::text::trim_num(figcraft_engine::leading::percent_of(x)))
}

/// In the picker's body, under the search field: the px | % switch (only for a line height).
pub(crate) fn switch(ui: &mut Ui, w: f32) {
    let Some(pct) = ACTIVE.with(Cell::get) else { return };
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
    let label = tr("Line height in");
    ui.painter().text(pos2(r.min.x + 14.0, r.center().y), Align2::LEFT_CENTER, label, regular(11.0), t.text_secondary);
    let seg_w = 44.0;
    let track = Rect::from_min_max(pos2(r.max.x - 10.0 - seg_w * 2.0, r.min.y + 4.0), pos2(r.max.x - 10.0, r.max.y - 4.0));
    ui.painter().rect_filled(track, CornerRadius::same(6), t.field);
    for (i, (text, on)) in [("px", !pct), ("%", pct)].into_iter().enumerate() {
        let seg = Rect::from_min_size(pos2(track.min.x + seg_w * i as f32, track.min.y), vec2(seg_w, track.height())).shrink(2.0);
        let resp = ui.interact(seg, Id::unique(("figcraftplus_leading_seg", i)), Sense::click());
        if on {
            ui.painter().rect_filled(seg, CornerRadius::same(5), t.panel);
        } else if resp.hovered() {
            ui.painter().rect_filled(seg, CornerRadius::same(5), t.hover);
        }
        ui.painter().text(seg.center(), Align2::CENTER_CENTER, text, regular(11.0), if on { t.text } else { t.text_secondary });
        if resp.clicked() && !on {
            let now = i == 1;
            ACTIVE.with(|a| a.set(Some(now)));
            ui.data_mut(|d| {
                d.insert_temp(remembered_id().with("open"), now);
                d.insert_temp(remembered_id(), now);
            });
        }
    }
}

/// The variable in use, for the picker's check mark, when it binds in %.
pub(crate) fn current(doc: &Document, target: &Target) -> Option<Pick> {
    let name = match target {
        Target::Bind { ids, .. } => layer_rule(doc, ids)?,
        Target::Style { style, .. } => figcraft_engine::leading::style_rule(doc, style)?,
        _ => return None,
    };
    doc.variables.get(&name).map(|v| Pick { library: None, id: v.id.clone() })
}

/// Apply `pick` (one of this file's variables: the list offers no library ones in %) to the line
/// height of `target` in %.
pub(crate) fn apply(app: &mut FigApp, target: &Target, pick: &Pick) {
    if pick.library.is_some() {
        return;
    }
    match target {
        Target::Bind { ids, .. } => {
            app.run("text.lineHeightVariable", json!({"ids": ids, "variable": pick.id}));
        }
        Target::Style { style, .. } => {
            app.run("style.lineHeightVariable", json!({"style": style, "variable": pick.id}));
        }
        _ => {}
    }
}

/// The chip label of a line height following `name`: its last part and the percentage it gives
/// (`at`: the line height it holds now).
fn chip_label(name: &str, at: Option<figcraft_doc::LineHeight>) -> String {
    let short = name.rsplit('/').next().unwrap_or(name);
    match at {
        // The percentage first: the chip is narrow.
        Some(figcraft_doc::LineHeight::Percent(p)) => format!("{}% {short}", figcraft_doc::text::trim_num(p)),
        _ => format!("% {short}"),
    }
}

/// In the Typography section: when the line height of `ids` follows a variable, its chip in `rect`
/// and true; otherwise nothing and false.
pub(crate) fn layer_chip(app: &mut FigApp, ui: &mut Ui, ids: &[NodeId], rect: Rect) -> bool {
    let Some(doc) = app.session.doc().ok() else { return false };
    let Some(name) = layer_rule(doc, ids) else { return false };
    let at = ids.first().and_then(|i| doc.node(*i)).and_then(|n| n.kind.text()).map(|t| t.style.line_height);
    match widgets::bound_field(ui, Id::unique(("figcraftplus_leading_chip", ids)), Lead::None, rect, &chip_label(&name, at)) {
        Some(BindEvent::Open(at)) => crate::variable_ui::open(ui, ids, FIELD, figcraft_doc::VariableType::Number, at),
        Some(BindEvent::Detach) => {
            app.run("text.lineHeightVariable", json!({"ids": ids}));
        }
        None => {}
    }
    true
}

/// In a text style's dialog: the same, for style `style`.
pub(crate) fn style_chip(app: &mut FigApp, ui: &mut Ui, doc: &Document, style: &str, rect: Rect) -> bool {
    let Some(name) = figcraft_engine::leading::style_rule(doc, style) else { return false };
    let at = match doc.style(style).map(|s| &s.kind) {
        Some(figcraft_doc::StyleKind::Text { style }) => Some(style.line_height),
        _ => None,
    };
    match widgets::bound_field(ui, Id::unique(("figcraftplus_leading_style_chip", style)), Lead::None, rect, &chip_label(&name, at)) {
        Some(BindEvent::Open(at)) => crate::variable_ui::open_style(ui, style, FIELD, figcraft_doc::VariableType::Number, at),
        Some(BindEvent::Detach) => {
            app.run("style.lineHeightVariable", json!({"style": style}));
        }
        None => {}
    }
    true
}
