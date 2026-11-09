//! FigCraft Plus: the style dialog — "Create new text / color / effect / layout grid
//! style" from the Local styles "+", and the same dialog to edit a style from the Local styles list.
//!
//! It shows a preview, the name and description, and the style's properties: a text style's font,
//! style (or weight), size, line height, letter spacing, decoration and case; a colour style's
//! paints; an effect style's effects and a grid style's grids, each with its settings in a popover
//! (effect type, offsets, blur, spread, colour…; grid type, size or count, alignment, margin,
//! gutter, colour). Fields that styles can bind take a variable (the hexagon on hover).
//!
//! The style exists while the dialog is open, so the colour picker and variables act on it: Create
//! style keeps it (with the name typed), the cross or Escape removes a style being created. Edits
//! apply at once (each one undo step).

use egui::{Align2, Color32, CornerRadius, Id, Rect, Sense, Stroke, Ui, pos2, vec2};
use figcraft_color::{Color, Paint, PaintKind};
use figcraft_doc::{
    BlurType, Document, Effect, EffectKind, GridAlign, GridKind, LayoutGrid, LetterSpacing, LineHeight, ListStyle, MAX_BLUR_RADIUS, ParagraphStyle, Style,
    StyleKind, TextCase, TextDecoration, TextStyle, VariableType,
};
use serde_json::{Value, json};

use crate::FigApp;
use crate::a11y::Tip;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::right::{menu_check, menu_popup, row};
use crate::style_picker::StyleSlot;
use crate::styles_plus_ui::text_value;
use crate::theme::{Tokens, regular, semibold};
use crate::widgets::{self, FieldEdit, Lead};

/// The dialog's width.
const W: f32 = 300.0;
/// The settings popover's width.
const POP_W: f32 = 250.0;

#[derive(Clone, Debug)]
struct Dialog {
    /// The style shown (created when the dialog opened, in create mode).
    style: String,
    /// Being created: Create style keeps it, closing removes it.
    creating: bool,
    name: String,
    description: String,
    /// The property whose settings popover is open (an effect or grid index).
    open: Option<usize>,
    /// Screen y the dialog opens at.
    y: f32,
    /// Show the text style's extra settings (decoration, case).
    more: bool,
    /// Select the name on the first frame.
    focus_name: bool,
}

fn dialog_id() -> Id {
    Id::unique("figcraftplus_style_dialog")
}

fn get(ctx: &egui::Context) -> Option<Dialog> {
    ctx.data(|d| d.get_temp::<Dialog>(dialog_id()))
}

fn put(ctx: &egui::Context, d: Option<Dialog>) {
    let open = d.is_some();
    ctx.data_mut(|m| match d {
        Some(d) => {
            m.insert_temp(dialog_id(), d);
        }
        None => m.remove::<Dialog>(dialog_id()),
    });
    if !open {
        crate::popover_ui::forget(area_id());
    }
}

/// The style the dialog shows (tests).
#[cfg(test)]
pub(crate) fn editing(ctx: &egui::Context) -> Option<String> {
    get(ctx).map(|d| d.style)
}

/// The base of a new style's name per type ("Text style 2").
fn base_name(slot: StyleSlot) -> &'static str {
    match slot {
        StyleSlot::Text => "Text style",
        StyleSlot::Effect => "Effect style",
        StyleSlot::Grid => "Layout guide style",
        _ => "Color style",
    }
}

fn next_name(doc: &Document, base: &str) -> String {
    (1..).map(|n| if n == 1 { base.to_string() } else { format!("{base} {n}") }).find(|n| doc.styles.iter().all(|s| &s.name != n)).unwrap_or_default()
}

fn type_key(slot: StyleSlot) -> &'static str {
    match slot {
        StyleSlot::Text => "text",
        StyleSlot::Effect => "effect",
        StyleSlot::Grid => "grid",
        _ => "paint",
    }
}

/// Create a `slot` style from `value` and open the dialog on it (the Local styles "+").
pub(crate) fn open_create(app: &mut FigApp, ctx: &egui::Context, slot: StyleSlot, value: Value, y: f32) {
    discard(app, ctx);
    let Some(name) = app.session.doc().ok().map(|d| next_name(d, base_name(slot))) else { return };
    let made = app.run("style.create", json!({"type": type_key(slot), "name": name, "value": value}));
    let Some(id) = made.as_ref().and_then(|v| v["id"].as_str()).map(str::to_string) else { return };
    put(
        ctx,
        Some(Dialog { style: id, creating: true, name: String::new(), description: String::new(), open: None, y, more: false, focus_name: true }),
    );
}

/// Open the dialog on the style `id` just made from a layer (the style picker's "+"): as being
/// created, so closing it removes the style again.
pub(crate) fn open_created(app: &mut FigApp, ctx: &egui::Context, id: &str, y: f32) {
    discard(app, ctx);
    put(
        ctx,
        Some(Dialog {
            style: id.to_string(),
            creating: true,
            name: String::new(),
            description: String::new(),
            open: None,
            y,
            more: false,
            focus_name: true,
        }),
    );
}

/// Open the dialog to edit this file's style `id` (from the Local styles list).
pub(crate) fn open_edit(app: &mut FigApp, ctx: &egui::Context, id: &str, y: f32) {
    discard(app, ctx);
    let Some(st) = app.session.doc().ok().and_then(|d| d.style(id)).cloned() else { return };
    put(
        ctx,
        Some(Dialog { style: st.id, creating: false, name: st.name, description: figcraft_engine::leading::without_rule(&st.description), open: None, y, more: false, focus_name: false }),
    );
}

/// Close the dialog; a style still being created is removed.
fn discard(app: &mut FigApp, ctx: &egui::Context) {
    if let Some(d) = get(ctx) {
        if d.creating {
            app.run("style.delete", json!({"style": d.style}));
        }
        if matches!(app.ui.picker, Some(crate::colorpicker::PickerTarget::Style { .. })) {
            app.ui.picker = None;
        }
        put(ctx, None);
    }
}

/// Set the style's value (`style.update`), as one step or a scrub (`Some(done)`).
fn set_value(app: &mut FigApp, style: &str, value: Value, scrub: Option<bool>) {
    let p = json!({"style": style, "value": value});
    match scrub {
        None => {
            app.run("style.update", p);
        }
        Some(done) => {
            let _ = app.session.begin_interaction("Edit style");
            let _ = app.session.execute("style.update", &p);
            if done {
                let _ = app.session.commit_interaction();
            }
        }
    }
}

/// Apply a field edit through `f` (the new value from the number).
fn edit_with(app: &mut FigApp, style: &str, e: Option<FieldEdit>, f: impl Fn(f64) -> Value) {
    match e {
        Some(FieldEdit::Set(v)) => set_value(app, style, f(v), None),
        Some(FieldEdit::Scrub(v, done)) => set_value(app, style, f(v), Some(done)),
        None => {}
    }
}

fn json_of<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

/// The variable bound to `field` of `st` (its name), the key it is kept under.
fn bound(doc: &Document, st: &Style, field: &str) -> Option<String> {
    let key = figcraft_engine::style_vars::canonical(field);
    let v = st.bound.get(&key)?;
    Some(doc.variables.get(v).map_or_else(|| v.clone(), |x| x.name.clone()))
}

/// A control that can take a variable: the bound variable's chip (click: another, detach button:
/// unbind) or `control` with the variable button on hover. `field` is the style's field
/// (`fontSize`, `effects/0/radius`…); `kind` the variable type.
#[allow(clippy::too_many_arguments)]
fn bindable(
    app: &mut FigApp,
    ui: &mut Ui,
    doc: &Document,
    st: &Style,
    field: &str,
    kind: VariableType,
    width: f32,
    control: impl FnOnce(&mut FigApp, &mut Ui),
) {
    let area = Rect::from_min_size(ui.cursor().min, vec2(width, widgets::FIELD_H));
    // FigCraft Plus: a line height following a variable in %.
    if field == crate::leading_ui::FIELD {
        let r = Rect::from_min_size(area.min, area.size());
        if crate::leading_ui::style_chip(app, ui, doc, &st.id, r) {
            ui.allocate_exact_size(area.size(), Sense::hover());
            return;
        }
    }
    if let Some(name) = bound(doc, st, field) {
        let (r, _) = ui.allocate_exact_size(area.size(), Sense::hover());
        match widgets::bound_field(ui, Id::unique(("sd_chip", &st.id, field)), Lead::None, r, &name) {
            Some(widgets::BindEvent::Open(at)) => crate::variable_ui::open_style(ui, &st.id, field, kind, at),
            Some(widgets::BindEvent::Detach) => {
                app.run("style.bind", json!({"style": st.id, "field": field, "variable": null}));
            }
            None => {}
        }
        return;
    }
    control(app, ui);
    if crate::variable_ui::variable_button(ui, area, Id::unique(("sd_bind", &st.id, field))) {
        crate::variable_ui::open_style(ui, &st.id, field, kind, area.left_bottom());
    }
}

/// Draw the dialog (and its settings popover); call once per frame.
pub(crate) fn show(app: &mut FigApp, ctx: &egui::Context) {
    let Some(mut d) = get(ctx) else { return };
    let Ok(doc) = app.session.doc().cloned() else { return };
    let Some(st) = doc.style(&d.style).cloned() else {
        put(ctx, None);
        return;
    };
    let t = Tokens::get(ctx);
    let screen = ctx.content_rect();
    let x = (screen.max.x - app.ui.right_width - W - 16.0).max(screen.min.x + 8.0);
    let h = crate::popover_ui::rect(area_id()).map_or(420.0, |r| r.height());
    let y = d.y.clamp(screen.min.y + 48.0, (screen.max.y - h - 8.0).max(screen.min.y + 48.0));
    let mut close = false;
    let mut create = false;
    // Whole sentences per type: languages word them differently.
    let (create_title, edit_title, hint) = match &st.kind {
        StyleKind::Text { .. } => (tr("Create new text style"), tr("Edit text style"), tr("New text style")),
        StyleKind::Paint { .. } => (tr("Create new color style"), tr("Edit color style"), tr("New color style")),
        StyleKind::Effect { .. } => (tr("Create new effect style"), tr("Edit effect style"), tr("New effect style")),
        StyleKind::Grid { .. } => (tr("Create new layout guide style"), tr("Edit layout guide style"), tr("New layout guide style")),
    };
    let title = if d.creating { create_title } else { edit_title };
    let (x_clicked, dialog_rect) = crate::popover_ui::show(ctx, area_id(), title, W, crate::popover::Open::At(pos2(x, y)), |ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
        preview(ui, &st, &t);
        separator(ui, &t);
        // Name and description.
        ui.add_space(4.0);
        labelled(ui, tr("Name"), &t, |ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(hint).font(regular(11.0)).desired_width(W - 112.0));
            if d.focus_name {
                r.request_focus();
                d.focus_name = false;
            }
            if !d.creating && r.lost_focus() && !d.name.trim().is_empty() && d.name.trim() != st.name {
                app.run("style.update", json!({"style": st.id, "name": d.name.trim()}));
            }
        });
        labelled(ui, tr("Description"), &t, |ui| {
            let r = ui
                .add(egui::TextEdit::singleline(&mut d.description).hint_text(tr("What it's for")).font(regular(11.0)).desired_width(W - 112.0));
            // FigCraft Plus: the line-height variable's line stays out of sight and in place.
            let description = figcraft_engine::leading::keep_rule(&d.description, &st.description);
            if !d.creating && r.lost_focus() && description != st.description {
                app.run("style.update", json!({"style": st.id, "description": description}));
            }
        });
        ui.add_space(6.0);
        separator(ui, &t);
        // Properties.
        let (pr, _) = ui.allocate_exact_size(vec2(W, 36.0), Sense::hover());
        ui.painter().text(pos2(pr.min.x + 16.0, pr.center().y), Align2::LEFT_CENTER, tr("Properties"), semibold(11.0), t.text);
        if !matches!(st.kind, StyleKind::Text { .. }) {
            let ar = Rect::from_center_size(pos2(pr.max.x - 22.0, pr.center().y), vec2(24.0, 24.0));
            let aresp = ui.interact(ar, Id::unique("sd_add"), Sense::click()).tip(tr("Add"));
            if aresp.hovered() {
                ui.painter().rect_filled(ar, CornerRadius::same(5), t.hover);
            }
            icons::paint(ui.painter(), ar.shrink(6.0), Icon::Plus, t.icon);
            if aresp.clicked() {
                add_property(app, &st);
            }
        }
        match &st.kind {
            StyleKind::Text { style, paragraph } => text_props(app, ui, &doc, &st, style, paragraph, &mut d.more),
            StyleKind::Paint { paints } => paint_rows(app, ui, &doc, &st, paints),
            StyleKind::Effect { effects } => {
                let labels: Vec<String> = effects.iter().map(|e| tr(e.label()).to_string()).collect();
                let vis: Vec<bool> = effects.iter().map(|e| e.visible).collect();
                list_rows(app, ui, &st, &labels, &vis, Icon::Effects, &mut d.open);
            }
            StyleKind::Grid { grids } => {
                let labels: Vec<String> = grids.iter().map(grid_label).collect();
                let vis: Vec<bool> = grids.iter().map(|g| g.visible).collect();
                list_rows(app, ui, &st, &labels, &vis, Icon::Grid, &mut d.open);
            }
        }
        ui.add_space(8.0);
        // Footer.
        if d.creating {
            separator(ui, &t);
            let (fr, _) = ui.allocate_exact_size(vec2(W, 48.0), Sense::hover());
            let br = Rect::from_min_max(pos2(fr.max.x - 120.0, fr.min.y + 10.0), pos2(fr.max.x - 14.0, fr.max.y - 10.0));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(br));
            if widgets::button(&mut child, tr("Create style"), true, true).clicked() {
                create = true;
            }
        }
    });
    close |= x_clicked;
    // The settings popover of the open effect or grid, left of the dialog.
    if let Some(i) = d.open {
        let shown = match &st.kind {
            StyleKind::Effect { effects } => effects.get(i).is_some(),
            StyleKind::Grid { grids } => grids.get(i).is_some(),
            _ => false,
        };
        if shown {
            let px = (dialog_rect.min.x - POP_W - 8.0).max(screen.min.x + 8.0);
            let py = (dialog_rect.min.y + 180.0 + i as f32 * 32.0).min(screen.max.y - 320.0).max(screen.min.y + 48.0);
            let mut shut = false;
            egui::Area::new(Id::unique("figcraftplus_style_popover")).order(egui::Order::Foreground).fixed_pos(pos2(px, py)).show(ctx, |ui| {
                crate::popover_ui::frame(ui).show(ui, |ui| {
                    ui.set_width(POP_W);
                    ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                    match &st.kind {
                        StyleKind::Effect { effects } => effect_settings(app, ui, &doc, &st, effects, i, &mut shut),
                        StyleKind::Grid { grids } => grid_settings(app, ui, &doc, &st, grids, i, &mut shut),
                        _ => {}
                    }
                    ui.add_space(8.0);
                });
            });
            if shut {
                d.open = None;
            }
        } else {
            d.open = None;
        }
    }
    let typing = ctx.memory(|m| m.focused().is_some());
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !typing && !crate::variable_ui::picker_open(ctx) && app.ui.picker.is_none() {
        if d.open.is_some() {
            d.open = None;
        } else {
            close = true;
        }
    }
    if create {
        let name = d.name.trim().to_string();
        let mut p = json!({"style": st.id, "description": figcraft_engine::leading::keep_rule(&d.description, &st.description)});
        if !name.is_empty() {
            p["name"] = json!(name);
        }
        app.run("style.update", p);
        if matches!(app.ui.picker, Some(crate::colorpicker::PickerTarget::Style { .. })) {
            app.ui.picker = None;
        }
        put(ctx, None);
        return;
    }
    if close {
        discard(app, ctx);
        return;
    }
    put(ctx, Some(d));
}

fn separator(ui: &mut Ui, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(W, 1.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::ZERO, t.border);
}

/// A "Label   [field]" row.
fn labelled(ui: &mut Ui, label: &str, t: &Tokens, field: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let (r, _) = ui.allocate_exact_size(vec2(80.0, 24.0), Sense::hover());
        ui.painter().text(r.left_center(), Align2::LEFT_CENTER, label, regular(11.0), t.text_secondary);
        field(ui);
    });
}

/// The preview at the top: the text's sample, the colours, the effects on a disc, the grid's kind.
fn preview(ui: &mut Ui, st: &Style, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(W, 130.0), Sense::hover());
    let inner = r.shrink(12.0);
    widgets::checker(&ui.painter().with_clip_rect(inner), inner, 8.0);
    let p = ui.painter();
    match &st.kind {
        StyleKind::Text { style, .. } => {
            let mut budget = 2;
            match crate::font_preview::texture(ui.ctx(), style, "Rag 123", &mut budget) {
                Some(tex) => {
                    let w = (tex.size_vec2() / ui.ctx().pixels_per_point().max(0.5)).x.min(inner.width() - 16.0);
                    crate::font_preview::paint(
                        p,
                        &tex,
                        pos2(inner.center().x - w / 2.0, inner.center().y),
                        inner.width() - 16.0,
                        inner.height() - 8.0,
                        t.text,
                    );
                }
                None => {
                    p.text(inner.center(), Align2::CENTER_CENTER, "Rag 123", regular(14.0), t.text);
                    ui.ctx().request_repaint();
                }
            }
        }
        StyleKind::Paint { paints } => {
            for paint in paints.iter().filter(|x| x.visible) {
                widgets::paint_swatch(p, inner, paint);
            }
        }
        StyleKind::Effect { effects } => {
            let disc = Rect::from_center_size(inner.center(), vec2(80.0, 80.0));
            for e in effects.iter().filter(|e| e.visible) {
                if let EffectKind::DropShadow(s) = &e.kind {
                    let shadow = egui::epaint::Shadow {
                        offset: [s.offset.0.clamp(-100.0, 100.0) as i8, s.offset.1.clamp(-100.0, 100.0) as i8],
                        blur: s.radius.clamp(0.0, 255.0) as u8,
                        spread: s.spread.clamp(0.0, 255.0) as u8,
                        color: crate::theme::doc_c32(ui.ctx(), s.color),
                    };
                    p.add(shadow.as_shape(disc, CornerRadius::same(40)));
                }
            }
            p.circle_filled(disc.center(), 40.0, Color32::WHITE);
            for e in effects.iter().filter(|e| e.visible) {
                if let EffectKind::InnerShadow(s) = &e.kind {
                    p.circle_stroke(
                        disc.center() + vec2(s.offset.0 as f32, s.offset.1 as f32) * 0.2,
                        38.0,
                        Stroke::new(3.0, crate::theme::doc_c32(ui.ctx(), s.color)),
                    );
                }
            }
        }
        StyleKind::Grid { grids } => {
            let box_r = Rect::from_center_size(inner.center(), vec2(44.0, 44.0));
            p.rect_filled(box_r, CornerRadius::same(4), Color32::WHITE);
            p.rect_stroke(box_r, CornerRadius::same(4), Stroke::new(2.0, Color32::BLACK), egui::StrokeKind::Inside);
            let cell = box_r.shrink(9.0);
            match grids.first().map(|g| g.kind) {
                // Three stripes across or down; a 3×3 grid of dots.
                Some(GridKind::Columns) => {
                    for k in 0..3 {
                        let x = cell.min.x + k as f32 * 10.0;
                        p.rect_filled(Rect::from_min_size(pos2(x, cell.min.y), vec2(6.0, cell.height())), CornerRadius::ZERO, Color32::BLACK);
                    }
                }
                Some(GridKind::Rows) => {
                    for k in 0..3 {
                        let y = cell.min.y + k as f32 * 10.0;
                        p.rect_filled(Rect::from_min_size(pos2(cell.min.x, y), vec2(cell.width(), 6.0)), CornerRadius::ZERO, Color32::BLACK);
                    }
                }
                _ => {
                    for k in 0..9 {
                        let c = pos2(cell.min.x + 3.0 + (k % 3) as f32 * 10.0, cell.min.y + 3.0 + (k / 3) as f32 * 10.0);
                        p.rect_filled(Rect::from_center_size(c, vec2(5.0, 5.0)), CornerRadius::ZERO, Color32::BLACK);
                    }
                }
            }
        }
    }
}

/// Add another paint, effect or grid to the style.
fn add_property(app: &mut FigApp, st: &Style) {
    let v = match &st.kind {
        StyleKind::Paint { paints } => {
            let mut p = paints.clone();
            p.push(Paint::solid(Color::new(0.85, 0.85, 0.85, 1.0)));
            json_of(&p)
        }
        StyleKind::Effect { effects } => {
            let mut e = effects.clone();
            e.push(Effect::drop_shadow());
            json_of(&e)
        }
        StyleKind::Grid { grids } => {
            let mut g = grids.clone();
            g.push(LayoutGrid::default());
            json_of(&g)
        }
        StyleKind::Text { .. } => return,
    };
    set_value(app, &st.id, v, None);
}

/// Rows of effects or grids: settings button, label (both open the settings), visibility, remove.
fn list_rows(app: &mut FigApp, ui: &mut Ui, st: &Style, labels: &[String], visible: &[bool], icon: Icon, open: &mut Option<usize>) {
    let t = Tokens::get(ui.ctx());
    for (i, label) in labels.iter().enumerate().rev() {
        let (r, _) = ui.allocate_exact_size(vec2(W, 32.0), Sense::hover());
        let body = Rect::from_min_max(pos2(r.min.x + 12.0, r.min.y + 3.0), pos2(r.max.x - 64.0, r.max.y - 3.0));
        let resp = ui.interact(body, Id::unique(("sd_row", i)), Sense::click());
        let on = *open == Some(i);
        if on || resp.hovered() {
            ui.painter().rect_filled(body, CornerRadius::same(5), if on { t.selected_row } else { t.hover });
        }
        icons::paint(ui.painter(), Rect::from_center_size(pos2(body.min.x + 14.0, body.center().y), vec2(16.0, 16.0)), icon, t.icon);
        ui.painter().text(pos2(body.min.x + 32.0, body.center().y), Align2::LEFT_CENTER, label, regular(11.0), if on { t.accent } else { t.text });
        if resp.clicked() {
            *open = if on { None } else { Some(i) };
        }
        let vis = visible.get(i).copied().unwrap_or(true);
        let er = Rect::from_center_size(pos2(r.max.x - 46.0, r.center().y), vec2(24.0, 24.0));
        if ui.interact(er, Id::unique(("sd_eye", i)), Sense::click()).tip(tr("Toggle visibility")).clicked() {
            toggle(app, st, i);
        }
        icons::paint(ui.painter(), er.shrink(5.0), if vis { Icon::Eye } else { Icon::EyeOff }, t.icon);
        let mr = Rect::from_center_size(pos2(r.max.x - 20.0, r.center().y), vec2(24.0, 24.0));
        if ui.interact(mr, Id::unique(("sd_minus", i)), Sense::click()).tip(tr("Remove")).clicked() {
            remove(app, st, i);
            if *open == Some(i) {
                *open = None;
            }
        }
        icons::paint(ui.painter(), mr.shrink(5.0), Icon::Minus, t.icon);
    }
}

fn toggle(app: &mut FigApp, st: &Style, i: usize) {
    let v = match &st.kind {
        StyleKind::Paint { paints } => {
            let mut p = paints.clone();
            if let Some(x) = p.get_mut(i) {
                x.visible = !x.visible;
            }
            json_of(&p)
        }
        StyleKind::Effect { effects } => {
            let mut e = effects.clone();
            if let Some(x) = e.get_mut(i) {
                x.visible = !x.visible;
            }
            json_of(&e)
        }
        StyleKind::Grid { grids } => {
            let mut g = grids.clone();
            if let Some(x) = g.get_mut(i) {
                x.visible = !x.visible;
            }
            json_of(&g)
        }
        StyleKind::Text { .. } => return,
    };
    set_value(app, &st.id, v, None);
}

fn remove(app: &mut FigApp, st: &Style, i: usize) {
    let v = match &st.kind {
        StyleKind::Paint { paints } if paints.len() > 1 => {
            let mut p = paints.clone();
            p.remove(i.min(p.len() - 1));
            json_of(&p)
        }
        StyleKind::Effect { effects } if effects.len() > 1 => {
            let mut e = effects.clone();
            e.remove(i.min(e.len() - 1));
            json_of(&e)
        }
        StyleKind::Grid { grids } if grids.len() > 1 => {
            let mut g = grids.clone();
            g.remove(i.min(g.len() - 1));
            json_of(&g)
        }
        // A style keeps at least one property.
        _ => return,
    };
    set_value(app, &st.id, v, None);
}

// ---------------------------------------------------------------------------------------------
// Text

#[allow(clippy::too_many_arguments)]
fn text_props(app: &mut FigApp, ui: &mut Ui, doc: &Document, st: &Style, ts: &TextStyle, ps: &ParagraphStyle, more: &mut bool) {
    let full = W - 32.0;
    let half = (full - 8.0) / 2.0;
    let sid = st.id.clone();
    // Every rewrite carries the paragraph settings, which `style.update` would reset otherwise.
    let set = move |app: &mut FigApp, ts2: &TextStyle| set_value(app, &sid, text_value(ts2, ps), None);
    let sid = st.id.clone();
    let set_para = move |app: &mut FigApp, ps2: &ParagraphStyle| set_value(app, &sid, text_value(ts, ps2), None);
    // Font family.
    row(ui, 32.0, |ui| {
        bindable(app, ui, doc, st, "fontFamily", VariableType::String, full, |app, ui| {
            let resp = widgets::dropdown(ui, &ts.font_family, full, None).on_hover_text(tr("Font"));
            // FigCraft's font menu (search, keyboard, the families with ours: In this file, Add fonts).
            let pick = crate::typography::font_picker(ui, &resp, Some(&ts.font_family), full.max(260.0), |_| false);
            if let Some(f) = pick.and_then(|p| p.picked) {
                let mut ts2 = ts.clone();
                ts2.font_family = f.clone();
                let styles = figcraft_text::family_styles(&f);
                if !styles.is_empty() && !styles.contains(&ts2.font_style) {
                    ts2.set_font_style(styles.iter().find(|s| s.eq_ignore_ascii_case("Regular")).or(styles.first()).cloned().unwrap_or_default());
                }
                set(app, &ts2);
            }
        });
    });
    // Style (or weight) + size.
    row(ui, 32.0, |ui| {
        let weight_or_style = crate::text_vars_ui::WEIGHT_OR_STYLE;
        let bound_field = ["fontWeight", "fontStyle"].into_iter().find(|f| bound(doc, st, f).is_some());
        match bound_field {
            Some(f) => {
                let kind = if f == "fontWeight" { VariableType::Number } else { VariableType::String };
                bindable(app, ui, doc, st, f, kind, half, |_, _| {});
            }
            None => {
                let area = Rect::from_min_size(ui.cursor().min, vec2(half, widgets::FIELD_H));
                let resp = widgets::dropdown(ui, &ts.font_style, half, None).on_hover_text(tr("Font style"));
                menu_popup(&resp).width(200.0).show(|ui| {
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        for s in figcraft_text::family_styles(&ts.font_family) {
                            if menu_check(ui, ts.font_style == s, &s, "") {
                                let mut ts2 = ts.clone();
                                ts2.set_font_style(s.clone());
                                set(app, &ts2);
                                ui.close();
                            }
                        }
                    });
                });
                if crate::variable_ui::variable_button(ui, area, Id::unique(("sd_bind_w", &st.id))) {
                    crate::variable_ui::open_style(ui, &st.id, weight_or_style, VariableType::Number, area.left_bottom());
                }
            }
        }
        bindable(app, ui, doc, st, "fontSize", VariableType::Number, half, |app, ui| {
            let e = widgets::number_field(ui, Id::unique(("sd_size", &st.id)), Lead::None, Some(ts.font_size), "", half, 1.0);
            edit_with(app, &st.id, e, |v| {
                let mut ts2 = ts.clone();
                ts2.font_size = v.clamp(1.0, 5000.0);
                text_value(&ts2, ps)
            });
        });
    });
    // Line height + letter spacing.
    widgets::sublabels(ui, tr("Line height"), Some(tr("Letter spacing")));
    row(ui, 32.0, |ui| {
        bindable(app, ui, doc, st, "lineHeight", VariableType::Number, half, |app, ui| {
            if let Some(v) = widgets::text_field(ui, Id::unique(("sd_lh", &st.id)), &ts.line_height.label(), half, tr("Line height"))
                && let Some(lh) =
                    crate::typography::parse_line_height(&v, Some(ts.line_height)).and_then(|v| serde_json::from_value::<LineHeight>(v).ok())
            {
                let mut ts2 = ts.clone();
                ts2.line_height = lh;
                set(app, &ts2);
            }
        });
        bindable(app, ui, doc, st, "letterSpacing", VariableType::Number, half - 28.0, |app, ui| {
            if let Some(v) = widgets::text_field(ui, Id::unique(("sd_ls", &st.id)), &ts.letter_spacing.label(), half - 28.0, tr("Letter spacing"))
                && let Some(ls) =
                    crate::typography::parse_letter_spacing(&v, Some(ts.letter_spacing)).and_then(|v| serde_json::from_value::<LetterSpacing>(v).ok())
            {
                let mut ts2 = ts.clone();
                ts2.letter_spacing = ls;
                set(app, &ts2);
            }
        });
        if icons::button(ui, Icon::Dots, 24.0, *more, tr("More type settings")).clicked() {
            *more = !*more;
        }
    });
    // Paragraph spacing + indent (what text styles carry at the paragraph level since FigCraft 0.17).
    widgets::sublabels(ui, tr("Paragraph spacing"), Some(tr("Paragraph indent")));
    row(ui, 32.0, |ui| {
        let e = widgets::number_field(ui, Id::unique(("sd_pspace", &st.id)), Lead::None, Some(ps.paragraph_spacing), "", half, 1.0);
        edit_with(app, &st.id, e, |v| {
            let mut ps2 = ps.clone();
            ps2.paragraph_spacing = v.clamp(0.0, 10_000.0);
            text_value(ts, &ps2)
        });
        let e = widgets::number_field(ui, Id::unique(("sd_pindent", &st.id)), Lead::None, Some(ps.paragraph_indent), "", half, 1.0);
        edit_with(app, &st.id, e, |v| {
            let mut ps2 = ps.clone();
            ps2.paragraph_indent = v.clamp(0.0, 10_000.0);
            text_value(ts, &ps2)
        });
    });
    if *more {
        widgets::sublabels(ui, tr("Decoration"), Some(tr("Case")));
        row(ui, 32.0, |ui| {
            let decos = [
                (TextDecoration::None, tr("None")),
                (TextDecoration::Underline, tr("Underline")),
                (TextDecoration::Strikethrough, tr("Strikethrough")),
            ];
            let cur = decos.iter().find(|(d, _)| *d == ts.decoration).map_or("", |x| x.1);
            let resp = widgets::dropdown(ui, cur, half, None);
            menu_popup(&resp).show(|ui| {
                for (d, l) in decos {
                    if menu_check(ui, ts.decoration == d, l, "") {
                        let mut ts2 = ts.clone();
                        ts2.decoration = d;
                        set(app, &ts2);
                        ui.close();
                    }
                }
            });
            let cases = [
                (TextCase::Original, tr("As typed")),
                (TextCase::Upper, tr("Uppercase")),
                (TextCase::Lower, tr("Lowercase")),
                (TextCase::Title, tr("Title case")),
            ];
            let cur = cases.iter().find(|(c, _)| *c == ts.case).map_or("", |x| x.1);
            let resp = widgets::dropdown(ui, cur, half, None);
            menu_popup(&resp).show(|ui| {
                for (c, l) in cases {
                    if menu_check(ui, ts.case == c, l, "") {
                        let mut ts2 = ts.clone();
                        ts2.case = c;
                        set(app, &ts2);
                        ui.close();
                    }
                }
            });
        });
        widgets::sublabels(ui, tr("List style"), None);
        row(ui, 32.0, |ui| {
            let lists = [(ListStyle::None, tr("No list")), (ListStyle::Bulleted, tr("Bulleted list")), (ListStyle::Numbered, tr("Numbered list"))];
            let cur = lists.iter().find(|(l, _)| *l == ps.list).map_or("", |x| x.1);
            let resp = widgets::dropdown(ui, cur, half, None);
            menu_popup(&resp).show(|ui| {
                for (l, label) in lists {
                    if menu_check(ui, ps.list == l, label, "") {
                        let mut ps2 = ps.clone();
                        ps2.list = l;
                        set_para(app, &ps2);
                        ui.close();
                    }
                }
            });
        });
    }
}

// ---------------------------------------------------------------------------------------------
// Colour

fn paint_rows(app: &mut FigApp, ui: &mut Ui, doc: &Document, st: &Style, paints: &[Paint]) {
    let t = Tokens::get(ui.ctx());
    for (i, p) in paints.iter().enumerate().rev() {
        row(ui, 32.0, |ui| {
            let field = format!("fills/{i}/color");
            let solid = matches!(p.kind, PaintKind::Solid { .. });
            let w = W - 32.0 - 56.0;
            if solid {
                bindable(app, ui, doc, st, &field, VariableType::Color, w, |app, ui| paint_field(app, ui, st, paints, i, p, w));
            } else {
                paint_field(app, ui, st, paints, i, p, w);
            }
            if icons::button(ui, if p.visible { Icon::Eye } else { Icon::EyeOff }, 24.0, false, tr("Toggle visibility")).clicked() {
                toggle(app, st, i);
            }
            if icons::button(ui, Icon::Minus, 24.0, false, tr("Remove")).clicked() {
                remove(app, st, i);
            }
        });
    }
    let _ = t;
}

/// A paint: swatch (opens the colour picker on it), hex (or kind) and opacity.
fn paint_field(app: &mut FigApp, ui: &mut Ui, st: &Style, paints: &[Paint], i: usize, p: &Paint, w: f32) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(w, widgets::FIELD_H), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(5), t.field);
    let sw = Rect::from_center_size(pos2(r.min.x + 13.0, r.center().y), vec2(14.0, 14.0));
    widgets::paint_swatch(ui.painter(), sw, p);
    if ui.interact(sw.expand(3.0), Id::unique(("sd_sw", &st.id, i)), Sense::click()).tip(tr("Edit colour")).clicked() {
        // With its button, so the picker opens beside the dialog (`picker_place_ui::open`).
        app.open_picker(crate::colorpicker::PickerTarget::Style { id: st.id.clone(), index: i }, Some(sw));
    }
    let text_r = Rect::from_min_max(pos2(sw.max.x + 8.0, r.min.y + 4.0), pos2(r.max.x - 52.0, r.max.y - 4.0));
    match &p.kind {
        PaintKind::Solid { color } => {
            if let Some(h) = widgets::text_field_inline(ui, Id::unique(("sd_hex", &st.id, i)), &color.to_hex(), text_r, regular(11.0))
                && let Some(c) = Color::from_hex(&h)
            {
                let mut ps = paints.to_vec();
                if let Some(x) = ps.get_mut(i) {
                    x.kind = PaintKind::Solid { color: c.with_alpha(color.a) };
                }
                set_value(app, &st.id, json_of(&ps), None);
            }
        }
        _ => {
            ui.painter().text(text_r.left_center(), Align2::LEFT_CENTER, tr(paint_kind(p)), regular(11.0), t.text);
        }
    }
    let op_r = Rect::from_min_max(pos2(r.max.x - 48.0, r.min.y), r.max);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(op_r));
    let e =
        widgets::number_field(&mut child, Id::unique(("sd_op", &st.id, i)), Lead::None, Some((f64::from(p.opacity) * 100.0).round()), "%", 48.0, 1.0);
    edit_with(app, &st.id, e, |v| {
        let mut ps = paints.to_vec();
        if let Some(x) = ps.get_mut(i) {
            x.opacity = (v / 100.0).clamp(0.0, 1.0) as f32;
        }
        json_of(&ps)
    });
}

fn paint_kind(p: &Paint) -> &'static str {
    match &p.kind {
        PaintKind::Solid { .. } => "Solid",
        PaintKind::Gradient(_) => "Gradient",
        PaintKind::Image(_) => "Image",
        PaintKind::Pattern(_) => "Pattern",
    }
}

// ---------------------------------------------------------------------------------------------
// Effects

/// The settings of effect `i`: its type and every value it has.
fn effect_settings(app: &mut FigApp, ui: &mut Ui, doc: &Document, st: &Style, effects: &[Effect], i: usize, shut: &mut bool) {
    let Some(e) = effects.get(i) else { return };
    let t = Tokens::get(ui.ctx());
    let full = POP_W - 32.0;
    let half = (full - 8.0) / 2.0;
    let sid = st.id.clone();
    let with = |f: &dyn Fn(&mut Effect)| {
        let mut es = effects.to_vec();
        if let Some(x) = es.get_mut(i) {
            f(x);
        }
        json_of(&es)
    };
    // Header: the type menu and close.
    let (hr, _) = ui.allocate_exact_size(vec2(POP_W, 44.0), Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos2(hr.min.x + 12.0, hr.min.y + 8.0), vec2(170.0, 28.0))));
    let resp = widgets::dropdown(&mut child, tr(e.label()), 170.0, None);
    menu_popup(&resp).width(200.0).show(|ui| {
        for (l, ty) in Effect::TYPES {
            if menu_check(ui, e.label() == l, tr(l), "")
                && let Some(mut ne) = Effect::of_type(ty)
            {
                ne.visible = e.visible;
                let v = with(&|x| *x = ne.clone());
                set_value(app, &sid, v, None);
                ui.close();
            }
        }
    });
    close_button(ui, hr, shut, &t);
    let num = |app: &mut FigApp, ui: &mut Ui, key: &str, label: &str, value: f64, w: f32, apply: &dyn Fn(&mut Effect, f64)| {
        let field = format!("effects/{i}/{key}");
        bindable(app, ui, doc, st, &field, VariableType::Number, w, |app, ui| {
            let lead = if label.is_empty() { Lead::None } else { Lead::Text(label) };
            let ed = widgets::number_field(ui, Id::unique(("sd_fx", key, i)), lead, Some(value), "", w, 1.0);
            edit_with(app, &sid, ed, |v| with(&|x| apply(x, v)));
        });
    };
    let plain = |app: &mut FigApp, ui: &mut Ui, key: &str, label: &str, value: f64, suffix: &str, w: f32, apply: &dyn Fn(&mut Effect, f64)| {
        let lead = if label.is_empty() { Lead::None } else { Lead::Text(label) };
        let ed = widgets::number_field(ui, Id::unique(("sd_fx", key, i)), lead, Some(value), suffix, w, 1.0);
        edit_with(app, &sid, ed, |v| with(&|x| apply(x, v)));
    };
    match &e.kind {
        EffectKind::DropShadow(s) | EffectKind::InnerShadow(s) => {
            widgets::sublabels(ui, tr("Position"), None);
            row(ui, 32.0, |ui| {
                num(app, ui, "offsetX", "X", s.offset.0, half, &|x, v| shadow_mut(x, |s| s.offset.0 = v));
                num(app, ui, "offsetY", "Y", s.offset.1, half, &|x, v| shadow_mut(x, |s| s.offset.1 = v));
            });
            widgets::sublabels(ui, tr("Blur"), Some(tr("Spread")));
            row(ui, 32.0, |ui| {
                num(app, ui, "radius", "", s.radius, half, &|x, v| shadow_mut(x, |s| s.radius = v.max(0.0)));
                num(app, ui, "spread", "", s.spread, half, &|x, v| shadow_mut(x, |s| s.spread = v));
            });
            widgets::sublabels(ui, tr("Color"), None);
            row(ui, 32.0, |ui| {
                color_field(app, ui, doc, st, &format!("effects/{i}/color"), s.color, full, &|c| with(&|x| shadow_mut(x, |s| s.color = c)))
            });
            if matches!(e.kind, EffectKind::DropShadow(_)) {
                row(ui, 32.0, |ui| {
                    if widgets::checkbox(ui, s.show_behind, tr("Show behind transparent areas")) {
                        let v = with(&|x| shadow_mut(x, |s| s.show_behind = !s.show_behind));
                        set_value(app, &sid, v, None);
                    }
                });
            }
        }
        EffectKind::LayerBlur(b) | EffectKind::BackgroundBlur(b) => {
            widgets::sublabels(ui, tr("Type"), None);
            row(ui, 32.0, |ui| {
                let resp = widgets::dropdown(ui, tr(b.blur_type.label()), full, None);
                menu_popup(&resp).show(|ui| {
                    for bt in BlurType::ALL {
                        if menu_check(ui, b.blur_type == bt, tr(bt.label()), "") {
                            let v = with(&|x| {
                                if let Some(b2) = x.blur_mut() {
                                    b2.blur_type = bt;
                                }
                            });
                            set_value(app, &sid, v, None);
                            ui.close();
                        }
                    }
                });
            });
            if b.is_progressive() {
                widgets::sublabels(ui, tr("Start"), Some(tr("End")));
                row(ui, 32.0, |ui| {
                    num(app, ui, "startRadius", "", b.start_radius, half, &|x, v| {
                        if let Some(b2) = x.blur_mut() {
                            b2.start_radius = v.clamp(0.0, MAX_BLUR_RADIUS);
                        }
                    });
                    num(app, ui, "radius", "", b.radius, half, &|x, v| {
                        if let Some(b2) = x.blur_mut() {
                            b2.radius = v.clamp(0.0, MAX_BLUR_RADIUS);
                        }
                    });
                });
            } else {
                widgets::sublabels(ui, tr("Blur"), None);
                row(ui, 32.0, |ui| {
                    num(app, ui, "radius", "", b.radius, full, &|x, v| {
                        if let Some(b2) = x.blur_mut() {
                            b2.radius = v.clamp(0.0, MAX_BLUR_RADIUS);
                        }
                    })
                });
            }
        }
        EffectKind::Noise(nz) => {
            row(ui, 32.0, |ui| {
                plain(app, ui, "nsize", tr("Size"), nz.size, "", half, &|x, v| {
                    if let EffectKind::Noise(n) = &mut x.kind {
                        n.size = v.clamp(0.01, 1000.0);
                        n.size_y = None;
                    }
                });
                plain(app, ui, "ndensity", tr("Density"), (nz.density * 100.0).round(), "%", half, &|x, v| {
                    if let EffectKind::Noise(n) = &mut x.kind {
                        n.density = (v / 100.0).clamp(0.0, 1.0);
                    }
                });
            });
            if let Some(c) = e.color(false) {
                widgets::sublabels(ui, tr("Color"), None);
                row(ui, 32.0, |ui| {
                    color_field(app, ui, doc, st, "", c, full, &|c| {
                        with(&|x| {
                            if let Some(slot) = x.color_mut(false) {
                                *slot = c;
                            }
                        })
                    })
                });
            }
        }
        EffectKind::Texture(tx) => {
            row(ui, 32.0, |ui| {
                plain(app, ui, "tsize", tr("Size"), tx.size, "", half, &|x, v| {
                    if let EffectKind::Texture(t2) = &mut x.kind {
                        t2.size = v.clamp(0.01, 1000.0);
                        t2.size_y = None;
                    }
                });
                plain(app, ui, "tradius", tr("Radius"), tx.radius, "", half, &|x, v| {
                    if let EffectKind::Texture(t2) = &mut x.kind {
                        t2.radius = v.max(0.0);
                    }
                });
            });
            row(ui, 32.0, |ui| {
                if widgets::checkbox(ui, tx.clip_to_shape, tr("Clip to shape")) {
                    let v = with(&|x| {
                        if let EffectKind::Texture(t2) = &mut x.kind {
                            t2.clip_to_shape = !t2.clip_to_shape;
                        }
                    });
                    set_value(app, &sid, v, None);
                }
            });
        }
        EffectKind::Glass(g) => {
            let pct = |v: f64| (v * 100.0).round();
            row(ui, 32.0, |ui| {
                plain(app, ui, "glight", tr("Light|glass"), g.light_angle.round(), "°", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.light_angle = v.clamp(-180.0, 180.0);
                    }
                });
                plain(app, ui, "gint", "", pct(g.light_intensity), "%", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.light_intensity = (v / 100.0).clamp(0.0, 1.0);
                    }
                });
            });
            row(ui, 32.0, |ui| {
                plain(app, ui, "grefr", tr("Refraction"), pct(g.refraction), "%", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.refraction = (v / 100.0).clamp(0.0, 1.0);
                    }
                });
                plain(app, ui, "gdepth", tr("Depth"), g.depth, "", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.depth = v.clamp(1.0, 1000.0);
                    }
                });
            });
            row(ui, 32.0, |ui| {
                plain(app, ui, "gdisp", tr("Dispersion"), pct(g.dispersion), "%", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.dispersion = (v / 100.0).clamp(0.0, 1.0);
                    }
                });
                plain(app, ui, "gfrost", tr("Frost"), g.frost, "", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.frost = v.clamp(0.0, 1000.0);
                    }
                });
            });
            row(ui, 32.0, |ui| {
                plain(app, ui, "gsplay", tr("Splay"), pct(g.splay), "%", half, &|x, v| {
                    if let EffectKind::Glass(gl) = &mut x.kind {
                        gl.splay = (v / 100.0).clamp(0.0, 1.0);
                    }
                });
            });
        }
    }
}

fn shadow_mut(e: &mut Effect, f: impl FnOnce(&mut figcraft_doc::Shadow)) {
    if let EffectKind::DropShadow(s) | EffectKind::InnerShadow(s) = &mut e.kind {
        f(s);
    }
}

/// A colour of an effect or grid: swatch, hex and opacity, bound as `field` when given (empty:
/// not bindable). `value` turns a colour into the style's new value.
#[allow(clippy::too_many_arguments)]
fn color_field(app: &mut FigApp, ui: &mut Ui, doc: &Document, st: &Style, field: &str, c: Color, w: f32, value: &dyn Fn(Color) -> Value) {
    let draw = |app: &mut FigApp, ui: &mut Ui| {
        let t = Tokens::get(ui.ctx());
        let (r, _) = ui.allocate_exact_size(vec2(w, widgets::FIELD_H), Sense::hover());
        ui.painter().rect_filled(r, CornerRadius::same(5), t.field);
        let sw = Rect::from_center_size(pos2(r.min.x + 13.0, r.center().y), vec2(14.0, 14.0));
        widgets::paint_swatch(ui.painter(), sw, &Paint::solid(c));
        let text_r = Rect::from_min_max(pos2(sw.max.x + 8.0, r.min.y + 4.0), pos2(r.max.x - 52.0, r.max.y - 4.0));
        if let Some(h) = widgets::text_field_inline(ui, Id::unique(("sd_ch", &st.id, field)), &c.to_hex(), text_r, regular(11.0))
            && let Some(nc) = Color::from_hex(&h)
        {
            set_value(app, &st.id, value(nc.with_alpha(c.a)), None);
        }
        let op_r = Rect::from_min_max(pos2(r.max.x - 48.0, r.min.y), r.max);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(op_r));
        let e =
            widgets::number_field(&mut child, Id::unique(("sd_ca", &st.id, field)), Lead::None, Some((f64::from(c.a) * 100.0).round()), "%", 48.0, 1.0);
        edit_with(app, &st.id, e, |v| value(c.with_alpha((v / 100.0).clamp(0.0, 1.0) as f32)));
    };
    if field.is_empty() {
        draw(app, ui);
    } else {
        bindable(app, ui, doc, st, field, VariableType::Color, w, draw);
    }
}

fn close_button(ui: &mut Ui, header: Rect, shut: &mut bool, t: &Tokens) {
    let cr = Rect::from_center_size(pos2(header.max.x - 22.0, header.center().y), vec2(24.0, 24.0));
    let resp = ui.interact(cr, Id::unique("sd_pop_close"), Sense::click()).tip(tr("Close"));
    if resp.hovered() {
        ui.painter().rect_filled(cr, CornerRadius::same(5), t.hover);
    }
    icons::paint(ui.painter(), cr.shrink(6.0), Icon::Close, t.icon);
    if resp.clicked() {
        *shut = true;
    }
}

// ---------------------------------------------------------------------------------------------
// Layout grids

/// A grid's name in the list ("Grid 10px", "5 columns").
fn grid_label(g: &LayoutGrid) -> String {
    let auto = g.count == 0 && g.align != GridAlign::Stretch;
    match g.kind {
        GridKind::Grid => format!("{} {}px", tr("Grid"), trim(g.size)),
        GridKind::Columns if auto => tr("Auto columns").to_string(),
        GridKind::Rows if auto => tr("Auto rows").to_string(),
        GridKind::Columns => crate::i18n::trn("{n} columns", g.count.max(1) as usize, &[]),
        GridKind::Rows => crate::i18n::trn("{n} rows", g.count.max(1) as usize, &[]),
    }
}

fn trim(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The settings of grid `i`: its kind, size or count, alignment, width, margin, gutter, colour.
fn grid_settings(app: &mut FigApp, ui: &mut Ui, doc: &Document, st: &Style, grids: &[LayoutGrid], i: usize, shut: &mut bool) {
    let Some(g) = grids.get(i) else { return };
    let t = Tokens::get(ui.ctx());
    let full = POP_W - 32.0;
    let half = (full - 8.0) / 2.0;
    let sid = st.id.clone();
    let with = |f: &dyn Fn(&mut LayoutGrid)| {
        let mut gs = grids.to_vec();
        if let Some(x) = gs.get_mut(i) {
            f(x);
        }
        json_of(&gs)
    };
    let (hr, _) = ui.allocate_exact_size(vec2(POP_W, 44.0), Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos2(hr.min.x + 12.0, hr.min.y + 8.0), vec2(130.0, 28.0))));
    let kinds = [(GridKind::Grid, tr("Grid")), (GridKind::Columns, tr("Columns")), (GridKind::Rows, tr("Rows"))];
    let cur = kinds.iter().find(|(k, _)| *k == g.kind).map_or("", |x| x.1);
    let resp = widgets::dropdown(&mut child, cur, 130.0, None);
    menu_popup(&resp).show(|ui| {
        for (k, l) in kinds {
            if menu_check(ui, g.kind == k, l, "") {
                let fresh = match k {
                    GridKind::Grid => LayoutGrid::default(),
                    GridKind::Columns => LayoutGrid::columns(),
                    GridKind::Rows => LayoutGrid::rows(),
                };
                let v = with(&|x| *x = LayoutGrid { visible: x.visible, ..fresh.clone() });
                set_value(app, &sid, v, None);
                ui.close();
            }
        }
    });
    close_button(ui, hr, shut, &t);
    let num = |app: &mut FigApp, ui: &mut Ui, key: &str, value: Option<f64>, w: f32, empty: &str, apply: &dyn Fn(&mut LayoutGrid, f64)| {
        let field = format!("layoutGrids/{i}/{key}");
        bindable(app, ui, doc, st, &field, VariableType::Number, w, |app, ui| {
            let (ed, _) = widgets::number_field_bindable(ui, Id::unique(("sd_grid", key, i)), Lead::None, value, "", w, 1.0, None, widgets::Empty::Text(empty));
            let ed = ed.and_then(|e| e.settled());
            edit_with(app, &sid, ed, |v| with(&|x| apply(x, v)));
        });
    };
    if g.kind == GridKind::Grid {
        widgets::sublabels(ui, tr("Size"), None);
        row(ui, 32.0, |ui| num(app, ui, "size", Some(g.size), full, "", &|x, v| x.size = v.max(1.0)));
    } else {
        let cols = g.kind == GridKind::Columns;
        widgets::sublabels(ui, tr("Count"), Some(tr("Type")));
        row(ui, 32.0, |ui| {
            let auto = g.count == 0 && g.align != GridAlign::Stretch;
            num(app, ui, "count", (!auto).then(|| f64::from(g.count.max(1))), half, tr("Auto"), &|x, v| {
                x.count = if x.align == GridAlign::Stretch { v.round().clamp(1.0, 1000.0) } else { v.round().clamp(0.0, 1000.0) } as u32
            });
            let names =
                if cols { [tr("Stretch"), tr("Left"), tr("Center"), tr("Right")] } else { [tr("Stretch"), tr("Top"), tr("Center"), tr("Bottom")] };
            let aligns = [GridAlign::Stretch, GridAlign::Min, GridAlign::Center, GridAlign::Max];
            let at = aligns.iter().position(|a| *a == g.align).unwrap_or(0);
            let resp = widgets::dropdown(ui, names.get(at).copied().unwrap_or_default(), half, None);
            menu_popup(&resp).show(|ui| {
                for (k, (name, a)) in names.iter().zip(aligns).enumerate() {
                    if menu_check(ui, k == at, name, "") {
                        let v = with(&|x| {
                            x.align = a;
                            if a != GridAlign::Stretch && !(x.size.is_finite() && x.size > 0.0) {
                                x.size = 60.0;
                            }
                            if a == GridAlign::Stretch {
                                x.count = x.count.max(1);
                            }
                        });
                        set_value(app, &sid, v, None);
                        ui.close();
                    }
                }
            });
        });
        let span = if cols { tr("Width") } else { tr("Height") };
        if g.align == GridAlign::Stretch {
            widgets::sublabels(ui, span, None);
            row(ui, 32.0, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    let _ = widgets::text_field(ui, Id::unique(("sd_grid_auto", i)), "", full, tr("Auto"));
                });
            });
            widgets::sublabels(ui, tr("Margin"), Some(tr("Gutter")));
            row(ui, 32.0, |ui| {
                num(app, ui, "offset", Some(g.offset), half, "", &|x, v| x.offset = v.max(0.0));
                num(app, ui, "gutter", Some(g.gutter), half, "", &|x, v| x.gutter = v.max(0.0));
            });
        } else {
            widgets::sublabels(ui, span, Some(if g.align == GridAlign::Center { tr("Gutter") } else { tr("Offset") }));
            row(ui, 32.0, |ui| {
                num(app, ui, "size", Some(g.size), half, "", &|x, v| x.size = v.max(1.0));
                if g.align == GridAlign::Center {
                    num(app, ui, "gutter", Some(g.gutter), half, "", &|x, v| x.gutter = v.max(0.0));
                } else {
                    num(app, ui, "offset", Some(g.offset), half, "", &|x, v| x.offset = v.max(0.0));
                }
            });
            if g.align != GridAlign::Center {
                widgets::sublabels(ui, tr("Gutter"), None);
                row(ui, 32.0, |ui| num(app, ui, "gutter", Some(g.gutter), full, "", &|x, v| x.gutter = v.max(0.0)));
            }
        }
    }
    widgets::sublabels(ui, tr("Color"), None);
    row(ui, 32.0, |ui| color_field(app, ui, doc, st, "", g.color, full, &|c| with(&|x| x.color = c)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_engine::Session;

    #[test]
    fn names_and_labels() {
        let mut s = Session::new();
        let doc = s.doc().unwrap().clone();
        assert_eq!(next_name(&doc, "Text style"), "Text style");
        s.execute("style.create", &json!({"type": "text", "name": "Text style", "value": json_of(&TextStyle::default())})).unwrap();
        assert_eq!(next_name(s.doc().unwrap(), "Text style"), "Text style 2");
        assert_eq!(grid_label(&LayoutGrid::default()), "Grid 10px");
        let mut c = LayoutGrid::columns();
        c.count = 5;
        assert_eq!(grid_label(&c), "5 columns");
        assert_eq!(trim(10.5), "10.5");
    }

    #[test]
    fn a_text_style_takes_a_weight_variable() {
        let mut s = Session::new();
        let c = s.execute("variable.createCollection", &json!({"name": "T"})).unwrap()["id"].as_str().unwrap().to_string();
        let w = s.execute("variable.create", &json!({"collection": c, "name": "w", "type": "number", "value": 700})).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        let mut ts = TextStyle::default();
        ts.set_font_style("Italic");
        let st = s.execute("style.create", &json!({"type": "text", "name": "H", "value": json_of(&ts)})).unwrap()["id"].as_str().unwrap().to_string();
        s.execute("style.bind", &json!({"style": st, "field": "fontWeight", "variable": w})).unwrap();
        let style = |s: &Session| match &s.doc().unwrap().style(&st).unwrap().kind {
            StyleKind::Text { style, .. } => style.font_style.clone(),
            _ => String::new(),
        };
        assert_eq!(style(&s), "Bold Italic");
        s.execute("variable.set", &json!({"variable": w, "mode": s.doc().unwrap().variables.collection(&c).unwrap().modes[0].id, "value": 300}))
            .unwrap();
        assert_eq!(style(&s), "Light Italic");
    }
}

/// The dialog's popover (its egui area).
fn area_id() -> Id {
    Id::unique("figcraftplus_style_dialog_area")
}
