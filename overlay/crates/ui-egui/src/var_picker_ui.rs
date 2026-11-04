//! FigCraft Plus: the variable picker, and variable scopes everywhere.
//!
//! - One picker for every place a variable is applied (layer properties, shared styles, alias
//!   values, prototype actions, colour channels): a light rounded card (dark in the dark theme) with
//!   a title and a close button, a search field, a library filter, the collections and their groups
//!   as headers, and rows with the variable's swatch or type icon, its short name and its value;
//!   the variable in use is checked. Arrow keys move through the rows and Enter applies one.
//! - Scopes: a property offers only the variables scoped to it, a shared style's fields too (a text
//!   style's size, a shadow's colour…). A variable with every scope off is offered nowhere but the
//!   Variables editor (prototyping and aliases still see it). Fields without a scope
//!   (colour channels, layout grids) offer every number variable that isn't hidden.

use egui::{Align2, Color32, CornerRadius, Id, Key, Modifiers, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use figcraft_doc::{Document, StyleKind, Variable, VariableType, VariableValue};
use figcraft_engine::cmd::lists::{BoundList, list_binding};
use serde_json::json;

use crate::FigApp;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::theme::{Tokens, regular, semibold};
use crate::variable_ui::{self, Pick, Picker, Target};
use crate::widgets;

/// The card's width.
const W: f32 = 260.0;
/// A variable row's height.
const ROW_H: f32 = 28.0;

/// Whether `v` has every scope turned off: only the Variables editor shows it then.
pub(crate) fn hidden(v: &Variable) -> bool {
    v.scopes.as_ref().is_some_and(Vec::is_empty)
}

/// Whether field `field` of a shared style of kind `kind` offers `v` (by the scope the same field
/// of a layer has; layout grid fields take any number variable).
pub(crate) fn style_fits(kind: Option<&StyleKind>, field: &str, v: &Variable) -> bool {
    if hidden(v) {
        return false;
    }
    if field == crate::text_vars_ui::WEIGHT_OR_STYLE {
        return crate::text_vars_ui::weight_fits(v);
    }
    let scopes: &[&str] = match (kind, list_binding(field)) {
        (_, Some((BoundList::Fills, _, _))) | (Some(StyleKind::Paint { .. }), None) => &["FRAME_FILL", "SHAPE_FILL", "TEXT_FILL", "STROKE_COLOR"],
        (_, Some((BoundList::Strokes, _, _))) => &["STROKE_COLOR"],
        (_, Some((BoundList::Effects, _, "color"))) => &["EFFECT_COLOR"],
        (_, Some((BoundList::Effects, _, _))) => &["EFFECT_FLOAT"],
        (_, Some((BoundList::Grids, _, _))) => return true,
        (_, None) => match field {
            "fontSize" => &["FONT_SIZE"],
            "lineHeight" => &["LINE_HEIGHT"],
            "letterSpacing" => &["LETTER_SPACING"],
            "fontFamily" => &["FONT_FAMILY"],
            "fontStyle" => &["FONT_STYLE"],
            "fontWeight" => &["FONT_WEIGHT"],
            "paragraphSpacing" => &["PARAGRAPH_SPACING"],
            "paragraphIndent" => &["PARAGRAPH_INDENT"],
            _ => return true,
        },
    };
    scopes.iter().any(|s| v.in_scope(s))
}

/// A variable the picker lists.
#[derive(Clone, Debug)]
pub(crate) struct Item {
    /// What picking it applies: its library (None: this file) and id.
    pub pick: Pick,
    /// The library's name (None: this file).
    pub library: Option<String>,
    /// Its collection's name.
    pub collection: String,
    /// Its full name, groups and all ("color/brand/primary").
    pub name: String,
    pub kind: VariableType,
    /// Its value in the default modes (aliases resolved).
    pub value: Option<VariableValue>,
}

impl Item {
    /// The header the collection shows under: its library's name first for library variables.
    fn source(&self) -> String {
        match &self.library {
            Some(l) => format!("{l} · {}", self.collection),
            None => self.collection.clone(),
        }
    }

    /// The group path ("color/brand") and the short name ("primary").
    fn split(&self) -> (&str, &str) {
        match self.name.rsplit_once('/') {
            Some((g, n)) => (g.trim(), n.trim()),
            None => ("", self.name.trim()),
        }
    }
}

/// The picker's own state: search text, library filter, highlighted row.
#[derive(Clone, Default)]
struct State {
    query: String,
    /// None: all libraries; Some(None): this file; Some(Some(name)): that library.
    filter: Option<Option<String>>,
    filter_open: bool,
    highlight: usize,
    /// Scroll the highlighted row into view (it moved by keyboard).
    reveal: bool,
}

/// What the card did this frame.
pub(crate) enum Outcome {
    Open,
    Pick(usize),
    Close,
}

/// A line of the list: a collection header, a group header, or a variable (its index in `items`).
#[derive(Debug, PartialEq)]
enum Line {
    Source(String),
    Group(String),
    Item(usize),
}

/// A group's path and its items.
type Grouped = (String, Vec<usize>);

/// The lines for the items passing the search and the library filter: each collection under its
/// header, its ungrouped variables first, then each group under its own (in the order they come).
fn lines(items: &[Item], q: &str, filter: &Option<Option<String>>) -> Vec<Line> {
    let q = q.trim().to_lowercase();
    // (source, [(group, [items])]) in first-appearance order.
    let mut sources: Vec<(String, Vec<Grouped>)> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        if filter.as_ref().is_some_and(|f| *f != it.library) {
            continue;
        }
        if !q.is_empty() && !it.name.to_lowercase().contains(&q) && !it.collection.to_lowercase().contains(&q) {
            continue;
        }
        let s = it.source();
        let at = match sources.iter().position(|(x, _)| *x == s) {
            Some(at) => at,
            None => {
                sources.push((s, vec![(String::new(), Vec::new())]));
                sources.len() - 1
            }
        };
        let Some((_, groups)) = sources.get_mut(at) else { continue };
        let g = it.split().0.to_string();
        match groups.iter_mut().find(|(x, _)| *x == g) {
            Some((_, list)) => list.push(i),
            None => groups.push((g, vec![i])),
        }
    }
    let mut out = Vec::new();
    for (s, groups) in sources {
        out.push(Line::Source(s));
        for (g, list) in groups {
            if list.is_empty() {
                continue;
            }
            if !g.is_empty() {
                out.push(Line::Group(g.split('/').map(str::trim).collect::<Vec<_>>().join(" / ")));
            }
            out.extend(list.into_iter().map(Line::Item));
        }
    }
    out
}

/// Forget the card `id`'s search and filter (when it closes).
pub(crate) fn reset(ctx: &egui::Context, id: Id) {
    ctx.data_mut(|d| d.remove::<State>(id));
    crate::popover_ui::forget(id.with("area"));
}

/// The picker card `id` under `at` (a field's bottom left corner): `title`, the `items`, the one in
/// use (`current`: its pick), and `empty` when there are none.
pub(crate) fn card(ctx: &egui::Context, id: Id, at: Pos2, title: &str, items: &[Item], current: Option<&Pick>, empty: &str) -> Outcome {
    let t = Tokens::get(ctx);
    let fresh = ctx.data(|d| d.get_temp::<State>(id)).is_none();
    let mut st: State = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
    let esc = ctx.input(|i| i.key_pressed(Key::Escape));
    let mut out = if esc { Outcome::Close } else { Outcome::Open };
    if let Some(i) = keys(ctx, &mut st, items, current, fresh) {
        out = Outcome::Pick(i);
    }
    // Under the field, inside the window (above it when there's no room below).
    let screen = ctx.content_rect();
    let size = crate::popover_ui::rect(id.with("area")).map_or(vec2(W, 360.0), |r| r.size());
    let x = at.x.min(screen.max.x - size.x - 8.0).max(screen.min.x + 8.0);
    let mut y = at.y + 4.0;
    if y + size.y > screen.max.y - 8.0 {
        y = (at.y - 30.0 - size.y).max(screen.min.y + 8.0).min(screen.max.y - size.y - 8.0);
    }
    let query_before = st.query.clone();
    let (closed, rect) = crate::popover_ui::show(ctx, id.with("area"), title, W, crate::popover::Open::At(pos2(x, y)), |ui| {
        if let Some(i) = body(ui, &t, id, &mut st, items, current, empty, W, 0.0) {
            out = Outcome::Pick(i);
        }
    });
    if closed {
        out = Outcome::Close;
    }
    if st.query != query_before {
        st.highlight = 0;
    }
    let pressed_outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p)));
    if pressed_outside && matches!(out, Outcome::Open) {
        out = Outcome::Close;
    }
    match out {
        Outcome::Open => ctx.data_mut(|d| {
            d.insert_temp(id, st);
        }),
        _ => reset(ctx, id),
    }
    out
}

/// The rows a card lists now and the items among them (in order).
fn visible_items(items: &[Item], st: &State) -> (Vec<Line>, Vec<usize>) {
    let list = lines(items, &st.query, &st.filter);
    let visible = list.iter().filter_map(|l| if let Line::Item(i) = l { Some(*i) } else { None }).collect();
    (list, visible)
}

/// Arrow keys move the highlight (a fresh list starts on the variable in use); Enter picks it.
fn keys(ctx: &egui::Context, st: &mut State, items: &[Item], current: Option<&Pick>, fresh: bool) -> Option<usize> {
    let (_, visible) = visible_items(items, st);
    if fresh && let Some(k) = visible.iter().position(|i| items.get(*i).is_some_and(|it| current == Some(&it.pick))) {
        st.highlight = k;
        st.reveal = true;
    }
    let (down, up, enter) =
        ctx.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::ArrowDown), i.consume_key(Modifiers::NONE, Key::ArrowUp), i.key_pressed(Key::Enter)));
    if down && !visible.is_empty() {
        st.highlight = (st.highlight + 1).min(visible.len() - 1);
        st.reveal = true;
    }
    if up {
        st.highlight = st.highlight.saturating_sub(1);
        st.reveal = true;
    }
    st.highlight = st.highlight.min(visible.len().saturating_sub(1));
    if enter { visible.get(st.highlight).copied() } else { None }
}

/// A card's contents, `w` wide: the search field, the library filter and the list. The item
/// clicked, if any.
#[allow(clippy::too_many_arguments)]
fn body(ui: &mut Ui, t: &Tokens, id: Id, st: &mut State, items: &[Item], current: Option<&Pick>, empty: &str, w: f32, min_h: f32) -> Option<usize> {
    let libs: Vec<Option<String>> = items.iter().fold(Vec::new(), |mut acc, it| {
        if !acc.contains(&it.library) {
            acc.push(it.library.clone());
        }
        acc
    });
    let (list, visible) = visible_items(items, st);
    let mut picked = None;
    // Search.
    let (sr, _) = ui.allocate_exact_size(vec2(w, 44.0), Sense::hover());
    let field = Rect::from_min_max(pos2(sr.min.x + 10.0, sr.min.y + 8.0), pos2(sr.max.x - 10.0, sr.max.y - 6.0));
    ui.painter().rect_filled(field, CornerRadius::same(6), t.field);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(field.min.x + 14.0, field.center().y), vec2(12.0, 12.0)), Icon::Search, t.icon_secondary);
    let te = Rect::from_min_max(pos2(field.min.x + 28.0, field.center().y - 8.0), pos2(field.max.x - 8.0, field.center().y + 8.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(te));
    let r = child.add(
        egui::TextEdit::singleline(&mut st.query)
            .id(id.with("search"))
            .frame(egui::Frame::NONE)
            .font(regular(12.0))
            .text_color(t.text)
            .hint_text(tr("Search"))
            .align(egui::Align2::LEFT_CENTER)
            .desired_width(te.width()),
    );
    if r.has_focus() {
        ui.painter().rect_stroke(field, CornerRadius::same(6), Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
    } else if !crate::a11y::field_focused(ui.ctx()) {
        r.request_focus();
    }
    // FigCraft Plus: a line height binds in px or in %.
    crate::leading_ui::switch(ui, w);
    // The library filter, when variables come from more than one place.
    if libs.len() > 1 {
        filter_row(ui, t, id, st, &libs, w);
    }
    ui.painter().hline(ui.min_rect().x_range(), ui.min_rect().max.y + 0.5, Stroke::new(1.0, t.border));
    ui.add_space(4.0);
    egui::ScrollArea::vertical().id_salt(id.with("scroll")).max_height(320.0f32.max(min_h)).min_scrolled_height(min_h).auto_shrink([false, true]).show(ui, |ui| {
        ui.set_width(w);
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        if visible.is_empty() {
            let msg = if items.is_empty() { empty } else { tr("No matching variables") };
            let galley = ui.painter().layout(msg.to_string(), regular(11.0), t.text_secondary, w - 28.0);
            let (r, _) = ui.allocate_exact_size(vec2(w, galley.size().y + 20.0), Sense::hover());
            ui.painter().galley(pos2(r.min.x + 14.0, r.min.y + 10.0), galley, t.text_secondary);
        }
        let mut k = 0usize;
        for line in &list {
            match line {
                Line::Source(s) => {
                    let (r, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
                    let s = widgets::elide_font(ui, s, w - 28.0, semibold(11.0));
                    ui.painter().text(pos2(r.min.x + 14.0, r.center().y + 2.0), Align2::LEFT_CENTER, s, semibold(11.0), t.text);
                }
                Line::Group(g) => {
                    let (r, _) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
                    let g = widgets::elide_font(ui, g, w - 28.0, regular(11.0));
                    ui.painter().text(pos2(r.min.x + 14.0, r.center().y), Align2::LEFT_CENTER, g, regular(11.0), t.text_secondary);
                }
                Line::Item(i) => {
                    let Some(it) = items.get(*i) else { continue };
                    let on = current == Some(&it.pick);
                    let (r, resp) = ui.allocate_exact_size(vec2(w, ROW_H), Sense::click());
                    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, &it.name));
                    if resp.hovered() && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
                        st.highlight = k;
                    }
                    if k == st.highlight {
                        ui.painter().rect_filled(r.shrink2(vec2(6.0, 1.0)), CornerRadius::same(6), t.hover);
                        if st.reveal {
                            ui.scroll_to_rect(r, None);
                            st.reveal = false;
                        }
                    }
                    row(ui, t, r, it, on);
                    if resp.clicked() {
                        picked = Some(*i);
                    }
                    k += 1;
                }
            }
        }
        ui.add_space(6.0);
    });
    picked
}

/// The variable list inside another window (the colour picker's Libraries tab), `w` wide: the
/// item picked (a click, or Enter on the highlighted one).
pub(crate) fn embedded(ui: &mut Ui, id: Id, items: &[Item], current: Option<&Pick>, empty: &str, w: f32) -> Option<usize> {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let fresh = ctx.data(|d| d.get_temp::<State>(id)).is_none();
    let mut st: State = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
    let mut out = keys(&ctx, &mut st, items, current, fresh);
    let before = st.query.clone();
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        if let Some(i) = body(ui, &t, id, &mut st, items, current, empty, w, 360.0) {
            out = Some(i);
        }
    });
    if st.query != before {
        st.highlight = 0;
    }
    ctx.data_mut(|d| d.insert_temp(id, st));
    out
}

/// The library filter: "All libraries" with a chevron; open, the choices under it.
fn filter_row(ui: &mut Ui, t: &Tokens, id: Id, st: &mut State, libs: &[Option<String>], w: f32) {
    let label = |f: &Option<Option<String>>| match f {
        None => tr("All libraries").to_string(),
        Some(None) => tr("This file").to_string(),
        Some(Some(l)) => l.clone(),
    };
    let (r, resp) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
    let text = label(&st.filter);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, &text));
    let w = ui.painter().layout_no_wrap(text.clone(), regular(11.0), t.text).size().x;
    let chip = Rect::from_min_size(pos2(r.min.x + 10.0, r.min.y + 2.0), vec2(w + 28.0, 22.0));
    if resp.hovered() || st.filter_open {
        ui.painter().rect_filled(chip, CornerRadius::same(5), t.hover);
    }
    ui.painter().text(pos2(chip.min.x + 6.0, chip.center().y), Align2::LEFT_CENTER, text, regular(11.0), t.text);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(chip.max.x - 10.0, chip.center().y), vec2(9.0, 9.0)), Icon::Chevron, t.icon_secondary);
    if resp.clicked() {
        st.filter_open = !st.filter_open;
    }
    if !st.filter_open {
        return;
    }
    let mut choices: Vec<Option<Option<String>>> = vec![None];
    choices.extend(libs.iter().cloned().map(Some));
    for c in choices {
        let (r, resp) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, st.filter == c, label(&c)));
        if resp.hovered() {
            ui.painter().rect_filled(r.shrink2(vec2(6.0, 1.0)), CornerRadius::same(6), t.hover);
        }
        if st.filter == c {
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 20.0, r.center().y), vec2(10.0, 10.0)), Icon::Check, t.text);
        }
        ui.painter().text(pos2(r.min.x + 34.0, r.center().y), Align2::LEFT_CENTER, label(&c), regular(11.0), t.text);
        if resp.clicked() {
            st.filter = c;
            st.filter_open = false;
            st.highlight = 0;
        }
    }
    let _ = id;
}

/// A variable row: check (the one in use), swatch or type icon, short name, value.
fn row(ui: &Ui, t: &Tokens, r: Rect, it: &Item, on: bool) {
    let p = ui.painter();
    if on {
        icons::paint(p, Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(10.0, 10.0)), Icon::Check, t.text);
    }
    let lead = Rect::from_center_size(pos2(r.min.x + 38.0, r.center().y), vec2(16.0, 16.0));
    match &it.value {
        Some(VariableValue::Color(c)) => {
            widgets::paint_swatch(p, lead, &figcraft_color::Paint::solid(*c));
            p.rect_stroke(lead, CornerRadius::same(4), Stroke::new(1.0, Color32::from_black_alpha(30)), egui::StrokeKind::Inside);
        }
        _ => type_icon(p, t, lead, it.kind),
    }
    let value = match &it.value {
        Some(VariableValue::Color(c)) => c.to_hex().trim_start_matches('#').to_uppercase(),
        Some(VariableValue::Number(x)) if crate::leading_ui::percent_rows() => crate::leading_ui::row_value(*x),
        Some(VariableValue::Number(x)) => crate::units_ui::list_number(*x),
        Some(v) => v.display(),
        None => String::new(),
    };
    let value = widgets::elide_font(ui, &value, 80.0, regular(11.0));
    let vw = if value.is_empty() { 0.0 } else { p.layout_no_wrap(value.clone(), regular(11.0), t.text_secondary).size().x + 8.0 };
    if !value.is_empty() {
        p.text(pos2(r.max.x - 14.0, r.center().y), Align2::RIGHT_CENTER, value, regular(11.0), t.text_secondary);
    }
    let name = widgets::elide_font(ui, it.split().1, r.width() - 54.0 - 14.0 - vw, regular(12.0));
    p.text(pos2(r.min.x + 54.0, r.center().y), Align2::LEFT_CENTER, name, regular(12.0), t.text);
}

/// The icon of a variable type without a swatch: number "#", string "T", boolean a switch.
fn type_icon(p: &egui::Painter, t: &Tokens, r: Rect, kind: VariableType) {
    p.rect_filled(r, CornerRadius::same(4), t.field);
    match kind {
        VariableType::String => icons::paint(p, r.shrink(3.0), Icon::Text, t.icon),
        VariableType::Boolean => {
            let pill = Rect::from_center_size(r.center(), vec2(11.0, 7.0));
            p.rect_stroke(pill, CornerRadius::same(4), Stroke::new(1.2, t.icon), egui::StrokeKind::Middle);
            p.circle_filled(pos2(pill.max.x - 3.5, pill.center().y), 2.0, t.icon);
        }
        _ => {
            p.text(r.center() + vec2(0.0, 0.5), Align2::CENTER_CENTER, "#", semibold(11.0), t.icon);
        }
    }
}

/// The picker items for the groups `variable_ui::groups` made.
pub(crate) fn items(doc: &Document, groups: &[variable_ui::Group<'_>]) -> Vec<Item> {
    let mut out = Vec::new();
    for (lib, c, vars) in groups {
        let all = lib.map_or(&doc.variables, |l| &l.doc.variables);
        for v in vars {
            out.push(Item {
                pick: Pick { library: lib.map(|l| l.manifest.id.clone()), id: v.id.clone() },
                library: lib.map(|l| figcraft_engine::library::library_name(doc, &l.manifest.id).to_string()),
                collection: c.name.clone(),
                name: v.name.clone(),
                kind: v.kind,
                value: all.resolve(&v.id, &std::collections::BTreeMap::new()),
            });
        }
    }
    out
}

/// The variable in use where the picker applies one (this file's variables).
fn current(doc: &Document, target: &Target) -> Option<Pick> {
    let id = match target {
        Target::Bind { ids, field } => {
            let fields: Vec<&str> =
                if field == crate::text_vars_ui::WEIGHT_OR_STYLE { vec!["fontWeight", "fontStyle"] } else { vec![field.as_str()] };
            let first = ids.first().and_then(|i| doc.node(*i))?;
            // A paint's colour has two spellings ("fill" is "fills/0/color").
            let same = |k: &str, f: &str| k == f || list_binding(k).is_some_and(|b| list_binding(f) == Some(b));
            fields.iter().find_map(|f| first.bound.iter().find(|(k, _)| same(k, f)).map(|(_, v)| v.clone()))?
        }
        Target::Style { style, field } => {
            let st = doc.style(style)?;
            let fields: Vec<&str> =
                if field == crate::text_vars_ui::WEIGHT_OR_STYLE { vec!["fontWeight", "fontStyle"] } else { vec![field.as_str()] };
            fields.iter().find_map(|f| st.bound.get(*f)).cloned()?
        }
        Target::Alias { variable, mode } => match doc.variables.get(variable)?.values.get(mode)? {
            VariableValue::Alias(a) => a.clone(),
            _ => return None,
        },
        Target::Proto(_) => return None,
    };
    Some(Pick { library: None, id })
}

/// The variables a picker for `target` offers (of type `kind`; None: any), and the one in use.
pub(crate) fn offered(app: &FigApp, doc: &Document, target: &Target, kind: Option<VariableType>) -> (Vec<Item>, Option<Pick>) {
    let libs = variable_ui::libraries(app, doc);
    // Bindings offer variables scoped to the property on every layer (styles: to the same
    // property of a layer); aliases any that wouldn't loop (library variables never lead back to
    // this file's).
    let (scopes, own): (Vec<&str>, _) = match target {
        Target::Bind { ids, field } => (ids.iter().filter_map(|id| figcraft_engine::cmd::variables::field_scope(doc, *id, field)).collect(), None),
        Target::Alias { variable, .. } => (Vec::new(), doc.variables.get(variable)),
        Target::Proto(_) | Target::Style { .. } => (Vec::new(), None),
    };
    let style_kind = match target {
        Target::Style { style, .. } => doc.style(style).map(|s| s.kind.clone()),
        _ => None,
    };
    // Prototypes and styles use this file's variables.
    let local_only = matches!(target, Target::Proto(_) | Target::Style { .. });
    let keep = |lib: Option<&figcraft_doc::PublishedLibrary>, v: &Variable| {
        let fits = match target {
            Target::Bind { field, .. } if field == crate::text_vars_ui::WEIGHT_OR_STYLE => crate::text_vars_ui::weight_fits(v),
            Target::Style { field, .. } => kind.is_none_or(|k| v.kind == k) || field == crate::text_vars_ui::WEIGHT_OR_STYLE,
            // Text content also takes a number variable (as FigCraft's own picker offers).
            Target::Bind { field, .. } if field == "characters" => kind.is_none_or(|k| v.kind == k || v.kind == VariableType::Number),
            _ => kind.is_none_or(|k| v.kind == k),
        };
        let scoped = match target {
            Target::Bind { .. } => !hidden(v) && scopes.iter().all(|s| v.in_scope(s)),
            Target::Style { field, .. } => style_fits(style_kind.as_ref(), field, v),
            Target::Alias { .. } | Target::Proto(_) => true,
        };
        fits && scoped
            && !(local_only && lib.is_some())
            && (lib.is_some() || own.is_none_or(|o| figcraft_engine::cmd::variables::alias_target(&doc.variables, o, &v.id).is_ok()))
    };
    let groups = variable_ui::groups(doc, &libs, keep);
    (items(doc, &groups), current(doc, target))
}

/// Apply `pick` where `target` says.
pub(crate) fn apply(app: &mut FigApp, target: &Target, pick: &Pick) {
    match (target, &pick.library) {
        (Target::Bind { ids, field }, _) => variable_ui::bind(app, ids, field, pick),
        (Target::Alias { variable, mode }, None) => {
            app.run("variable.set", json!({"variable": variable, "mode": mode, "value": {"alias": pick.id}}));
        }
        (Target::Alias { variable, mode }, Some(lib)) => {
            app.run("library.aliasVariable", json!({"library": lib, "alias": pick.id, "variable": variable, "mode": mode}));
        }
        (Target::Proto(slot), _) => crate::proto_editor::pick_variable(app, slot, &pick.id),
        (Target::Style { style, field }, _) => crate::text_vars_ui::bind_style(app, style, field, pick),
    }
}

/// The variable picker (replaces FigCraft's): call once per frame.
pub(crate) fn picker(app: &mut FigApp, ctx: &egui::Context) {
    let Some(p) = ctx.data(|d| d.get_temp::<Picker>(variable_ui::picker_id())) else { return };
    let Ok(doc) = app.session.ds().map(|d| d.doc.clone()) else { return };
    // A line height binds in px (FigCraft's way) or in % (`leading_ui`).
    let percent = crate::leading_ui::begin(ctx, &doc, &p.target) == Some(true);
    let (mut items, mut current) = offered(app, &doc, &p.target, p.kind);
    if percent {
        items.retain(|it| it.pick.library.is_none());
        current = crate::leading_ui::current(&doc, &p.target);
    }
    let empty = match p.kind {
        Some(VariableType::Color) => tr("No colour variables fit here."),
        Some(VariableType::Number) => tr("No number variables fit here."),
        Some(VariableType::String) => tr("No string variables fit here."),
        Some(VariableType::Boolean) => tr("No boolean variables fit here."),
        None => tr("No variables fit here."),
    };
    let id = Id::unique("figcraftplus_variable_picker");
    let close = |ctx: &egui::Context| {
        ctx.data_mut(|d| d.remove::<Picker>(variable_ui::picker_id()));
        crate::leading_ui::end(ctx);
        crate::popover_ui::forget_selection(ctx, id);
    };
    // A change of selection closes it, like every popover.
    if crate::popover_ui::selection_changed(ctx, id, &app.session.selection()) {
        reset(ctx, id);
        close(ctx);
        return;
    }
    // Numbers in the unit of the field they bind to.
    crate::units_ui::list_for(match &p.target {
        Target::Bind { field, .. } => Some(field.as_str()),
        _ => None,
    });
    let outcome = card(ctx, id, p.at, tr("Variables"), &items, current.as_ref(), empty);
    crate::units_ui::list_for(None);
    // The switch may have changed the mode this frame: what a pick does follows it.
    let percent = crate::leading_ui::percent_rows();
    crate::leading_ui::frame_done();
    match outcome {
        Outcome::Open => {}
        Outcome::Close => close(ctx),
        Outcome::Pick(i) => {
            close(ctx);
            let Some(pick) = items.get(i).map(|it| it.pick.clone()) else { return };
            if percent {
                crate::leading_ui::apply(app, &p.target, &pick);
            } else {
                apply(app, &p.target, &pick);
            }
        }
    }
}

/// Where the variable picker is on screen.
pub(crate) fn rect(ctx: &egui::Context) -> Option<Rect> {
    ctx.memory(|m| m.area_rect(Id::unique("figcraftplus_variable_picker").with("area")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(kind: VariableType, scopes: Option<Vec<&str>>) -> Variable {
        Variable {
            id: "VariableID:1".into(),
            name: "v".into(),
            collection: "c".into(),
            kind,
            values: Default::default(),
            description: String::new(),
            scopes: scopes.map(|s| s.into_iter().map(String::from).collect()),
            hidden_from_publishing: false,
            extends: None,
            overrides: Default::default(),
            code_syntax: Default::default(),
        }
    }

    fn item(lib: Option<&str>, coll: &str, name: &str) -> Item {
        Item {
            pick: Pick { library: lib.map(String::from), id: name.into() },
            library: lib.map(String::from),
            collection: coll.into(),
            name: name.into(),
            kind: VariableType::Number,
            value: None,
        }
    }

    #[test]
    fn styles_offer_the_variables_scoped_to_their_fields() {
        let size = var(VariableType::Number, Some(vec!["FONT_SIZE"]));
        let gap = var(VariableType::Number, Some(vec!["GAP"]));
        let off = var(VariableType::Number, Some(vec![]));
        let all = var(VariableType::Number, None);
        assert!(style_fits(None, "fontSize", &size));
        assert!(!style_fits(None, "fontSize", &gap));
        assert!(style_fits(None, "fontSize", &all));
        assert!(!style_fits(None, "fontSize", &off), "every scope off: only the Variables editor");
        assert!(style_fits(None, "grids/0/size", &gap), "layout grids take any number");
        assert!(!style_fits(None, "grids/0/size", &off));
        let shadow = var(VariableType::Color, Some(vec!["EFFECT_COLOR"]));
        let fill = var(VariableType::Color, Some(vec!["FRAME_FILL"]));
        assert!(style_fits(None, "effects/0/color", &shadow));
        assert!(!style_fits(None, "effects/0/color", &fill));
        assert!(style_fits(None, "fills/0/color", &fill));
        assert!(!style_fits(None, "fills/0/color", &shadow));
    }

    #[test]
    fn the_list_shows_collections_and_groups_as_headers() {
        let items = vec![
            item(None, "Tokens", "space/s"),
            item(None, "Tokens", "space/m"),
            item(None, "Tokens", "radius"),
            item(Some("Kit"), "Brand", "color/primary"),
        ];
        let l = lines(&items, "", &None);
        assert_eq!(
            l,
            vec![
                Line::Source("Tokens".into()),
                Line::Item(2),
                Line::Group("space".into()),
                Line::Item(0),
                Line::Item(1),
                Line::Source("Kit · Brand".into()),
                Line::Group("color".into()),
                Line::Item(3),
            ]
        );
        assert_eq!(lines(&items, "prim", &None), vec![Line::Source("Kit · Brand".into()), Line::Group("color".into()), Line::Item(3)]);
        assert_eq!(lines(&items, "", &Some(Some("Kit".into()))).len(), 3);
        assert_eq!(lines(&items, "", &Some(None)).len(), 5);
        assert_eq!(items[0].split(), ("space", "s"));
    }
}
