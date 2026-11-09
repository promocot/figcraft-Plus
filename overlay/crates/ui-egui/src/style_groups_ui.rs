//! FigCraft Plus: groups in the Design panel's Local styles.
//!
//! Styles of one type (text, colour, effect, layout grid) sit in nested groups, kept as "/" in
//! their names ("Brand/Blue/500" is the style "500" in the group "Blue" inside "Brand"), so files
//! stay readable by the official FigCraft. Types never mix: a text style can't go into a colour
//! group.
//!
//! - A click selects a style (Ctrl/Shift-click: several); double-click or the edit button on hover
//!   opens the style dialog.
//! - Ctrl+G (or "Group" in the context menu) puts the selected styles in a new group; "+" ▸ New
//!   group makes an empty one to drag styles into.
//! - Styles and whole groups drag: between rows, into another group (its header), out to the top.
//! - A group's arrow collapses it (remembered, also in the style picker); double-click its name to
//!   rename it; its context menu also ungroups it.

use std::collections::BTreeSet;

use egui::{Align2, CornerRadius, Id, Key, KeyboardShortcut, Modifiers, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};
use figcraft_color::Paint;
use figcraft_doc::{Document, Style, StyleKind};
use figcraft_engine::library;
use serde_json::{Value, json};

use crate::FigApp;
use crate::a11y::Tip;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::style_picker::StyleSlot;
use crate::theme::{Tokens, regular, semibold};
use crate::widgets;

const ROW_H: f32 = 28.0;
const HEAD_H: f32 = 26.0;
const INDENT: f32 = 14.0;
const SLOTS: [StyleSlot; 4] = [StyleSlot::Text, StyleSlot::Fill, StyleSlot::Effect, StyleSlot::Grid];

/// A name's parts, trimmed ("Brand / Blue/500" → ["Brand", "Blue", "500"]).
fn parts(name: &str) -> Vec<&str> {
    name.split('/').map(str::trim).collect()
}

/// The group path of a style name ("Brand/Blue" for "Brand / Blue/500"; "" outside groups).
pub(crate) fn folder_of(name: &str) -> String {
    let p = parts(name);
    p.get(..p.len().saturating_sub(1)).map(|f| f.join("/")).unwrap_or_default()
}

/// The style's own name ("500").
fn leaf(name: &str) -> &str {
    name.rsplit('/').next().map_or(name, str::trim)
}

/// `path` joined under `folder`.
fn join(folder: &str, path: &str) -> String {
    if folder.is_empty() { path.to_string() } else { format!("{folder}/{path}") }
}

/// The last part of a group path ("Blue" for "Brand/Blue").
fn last(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The group holding group `path` ("Brand" for "Brand/Blue").
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(p, _)| p)
}

/// Whether the style named `name` is inside group `path` (at any depth).
fn inside(name: &str, path: &str) -> bool {
    let f = folder_of(name);
    f == path || f.starts_with(&format!("{path}/"))
}

/// A line of the list.
#[derive(Clone, Debug, PartialEq)]
enum Line {
    /// A group header: its path and depth.
    Group { path: String, depth: usize },
    /// A style (its index in the type's styles) and its depth.
    Style { index: usize, depth: usize },
}

/// The lines for one type's `styles` (in file order): each group's own styles first, then its
/// groups in the order they first appear; `empty` groups made but not filled yet are listed too;
/// collapsed groups (`folds`) hide what's in them.
fn lines(styles: &[&Style], empty: &[String], folds: &BTreeSet<String>) -> Vec<Line> {
    // Every group path in first-appearance order (their parents first).
    let mut groups: Vec<String> = Vec::new();
    let add = |path: &str, groups: &mut Vec<String>| {
        let p: Vec<&str> = path.split('/').collect();
        for k in 1..=p.len() {
            let g = p.get(..k).map(|x| x.join("/")).unwrap_or_default();
            if !g.is_empty() && !groups.contains(&g) {
                groups.push(g);
            }
        }
    };
    for s in styles {
        add(&folder_of(&s.name), &mut groups);
    }
    for e in empty {
        add(e, &mut groups);
    }
    let mut out = Vec::new();
    fn walk(path: &str, depth: usize, styles: &[&Style], groups: &[String], folds: &BTreeSet<String>, out: &mut Vec<Line>) {
        for (i, s) in styles.iter().enumerate() {
            if folder_of(&s.name) == path {
                out.push(Line::Style { index: i, depth });
            }
        }
        for g in groups.iter().filter(|g| parent(g) == path && !g.is_empty()) {
            out.push(Line::Group { path: g.clone(), depth });
            if !folds.contains(g) {
                walk(g, depth + 1, styles, groups, folds, out);
            }
        }
    }
    walk("", 0, styles, &groups, folds, &mut out);
    out
}

/// What is being dragged: styles (ids) or a whole group (its path), of one type.
#[derive(Clone, Debug)]
struct Drag {
    slot: &'static str,
    styles: Vec<String>,
    group: Option<String>,
    label: String,
}

/// Where a drop lands: into group `folder`, before style `before` (None: after the type's last).
#[derive(Clone, Debug, PartialEq)]
struct Drop {
    folder: String,
    before: Option<String>,
}

/// The commands that move `moved` (style ids, a whole group `group` or single styles) of the
/// type's `styles` to `drop`: renamed into its group (a group keeps its own name and inner groups)
/// and placed before `drop.before`.
fn move_commands(styles: &[&Style], moved: &[String], group: Option<&str>, drop: &Drop) -> Vec<Value> {
    let mut cmds = Vec::new();
    let ids: Vec<&String> = styles.iter().map(|s| &s.id).filter(|id| moved.contains(id)).collect();
    if ids.is_empty() {
        return cmds;
    }
    for s in styles.iter().filter(|s| moved.contains(&s.id)) {
        let rel = match group {
            Some(g) => {
                let inner = parts(&s.name).get(g.split('/').count()..).map(|x| x.join("/")).unwrap_or_default();
                join(last(g), &inner)
            }
            None => leaf(&s.name).to_string(),
        };
        let name = join(&drop.folder, &rel);
        if name != parts(&s.name).join("/") {
            cmds.push(json!({"command": "style.update", "params": {"style": s.id, "name": name}}));
        }
    }
    let mut mv = json!({"styles": ids});
    if let Some(b) = &drop.before {
        mv["before"] = json!(b);
    }
    cmds.push(json!({"command": "style.move", "params": mv}));
    cmds
}

/// The first style after `after` in file order that isn't moving (the place to insert before).
fn next_unmoved(styles: &[&Style], after: usize, moved: &[String]) -> Option<String> {
    styles.iter().skip(after + 1).find(|s| !moved.contains(&s.id)).map(|s| s.id.clone())
}

/// The end of group `folder`'s styles (all depths): before the first unmoved style after its last.
fn end_of(styles: &[&Style], folder: &str, moved: &[String]) -> Option<String> {
    let lastpos = styles.iter().rposition(|s| folder.is_empty() || inside(&s.name, folder));
    match lastpos {
        Some(p) => next_unmoved(styles, p, moved),
        None => None,
    }
}

/// A new group's name in `folder`: "Group N" with the smallest N not taken there.
fn new_group_name(styles: &[&Style], empty: &[String], folder: &str) -> String {
    let taken = |n: &str| {
        let path = join(folder, n);
        styles.iter().any(|s| inside(&s.name, &path)) || empty.contains(&path)
    };
    (1..).map(|n| format!("Group {n}")).find(|n| !taken(n)).unwrap_or_else(|| "Group".into())
}

/// Per-session list state: selected styles, empty groups, the group being renamed.
#[derive(Clone, Default)]
struct State {
    selected: Vec<String>,
    /// Groups made with "New group" and not filled yet: (type key, path).
    empty: Vec<(String, String)>,
    renaming: Option<(String, String)>,
    drag: Option<Drag>,
    folds_loaded: bool,
}

fn state_id() -> Id {
    Id::unique("figcraftplus_style_groups")
}

/// Where the collapsed groups are kept between sessions (next to the app, like its fonts).
fn folds_file() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok()?.parent().map(|d| d.join("style-groups-collapsed.txt"))
}

fn load_folds(app: &mut FigApp) {
    if let Some(text) = folds_file().and_then(|f| std::fs::read_to_string(f).ok()) {
        app.ui.style_folds.extend(text.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from));
    }
}

fn save_folds(app: &FigApp) {
    if let Some(f) = folds_file() {
        let text: Vec<&str> = app.ui.style_folds.iter().map(String::as_str).collect();
        let _ = std::fs::write(f, text.join("\n"));
    }
}

/// "+" ▸ New group: an empty group of `slot`'s type (in the selected styles' group), renamed at once.
pub(crate) fn new_group(app: &mut FigApp, ctx: &egui::Context, slot: StyleSlot) {
    let Ok(doc) = app.session.doc().cloned() else { return };
    let mut st: State = ctx.data(|d| d.get_temp(state_id())).unwrap_or_default();
    let styles: Vec<&Style> = library::local_styles(&doc).filter(|s| slot.holds(&s.kind)).collect();
    let picked: Vec<&&Style> = styles.iter().filter(|s| st.selected.contains(&s.id)).collect();
    if !picked.is_empty() {
        group_selected(app, ctx);
        return;
    }
    let empty: Vec<String> = st.empty.iter().filter(|(k, _)| k == slot.key()).map(|(_, p)| p.clone()).collect();
    let path = new_group_name(&styles, &empty, "");
    st.empty.push((slot.key().to_string(), path.clone()));
    st.renaming = Some((slot.key().to_string(), path));
    ctx.data_mut(|d| d.insert_temp(state_id(), st));
}

/// Ctrl+G: the selected styles (of one type) go into a new group, in the group of the first one.
fn group_selected(app: &mut FigApp, ctx: &egui::Context) {
    let Ok(doc) = app.session.doc().cloned() else { return };
    let mut st: State = ctx.data(|d| d.get_temp(state_id())).unwrap_or_default();
    for slot in SLOTS {
        let styles: Vec<&Style> = library::local_styles(&doc).filter(|s| slot.holds(&s.kind)).collect();
        let Some(first) = styles.iter().position(|s| st.selected.contains(&s.id)) else { continue };
        let folder = styles.get(first).map(|s| folder_of(&s.name)).unwrap_or_default();
        let empty: Vec<String> = st.empty.iter().filter(|(k, _)| k == slot.key()).map(|(_, p)| p.clone()).collect();
        let path = join(&folder, &new_group_name(&styles, &empty, &folder));
        let moved: Vec<String> = styles.iter().filter(|s| st.selected.contains(&s.id)).map(|s| s.id.clone()).collect();
        // In place: before what follows the first selected style.
        let before = next_unmoved(&styles, first, &moved);
        let cmds = move_commands(&styles, &moved, None, &Drop { folder: path.clone(), before });
        app.run("command.batch", json!({"label": "Group styles", "commands": cmds}));
        st.renaming = Some((slot.key().to_string(), path));
        break;
    }
    ctx.data_mut(|d| d.insert_temp(state_id(), st));
}

/// The Design panel's Local styles (shown with nothing selected), by type, in groups.
pub(crate) fn local_styles(app: &mut FigApp, ui: &mut Ui, doc: &Document) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let mut st: State = ctx.data(|d| d.get_temp(state_id())).unwrap_or_default();
    if !st.folds_loaded {
        st.folds_loaded = true;
        load_folds(app);
    }
    let mut cmds: Option<(&'static str, Value)> = None;
    let mut edit: Option<String> = None;
    let mut fold_changed = false;
    let typing = ctx.memory(|m| m.focused().is_some());
    // Ctrl+G groups the selected styles; Escape clears the selection.
    if !st.selected.is_empty() && !typing {
        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::G))) {
            ctx.data_mut(|d| d.insert_temp(state_id(), st.clone()));
            group_selected(app, &ctx);
            return;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            st.selected.clear();
        }
    }
    let pointer = ctx.input(|i| i.pointer.interact_pos());
    let released = ctx.input(|i| i.pointer.any_released());
    let mut drop_at: Option<(Drop, Rect, bool)> = None;
    for slot in SLOTS {
        let styles: Vec<&Style> = library::local_styles(doc).filter(|s| slot.holds(&s.kind)).collect();
        let empty: Vec<String> = st.empty.iter().filter(|(k, p)| k == slot.key() && !styles.iter().any(|s| inside(&s.name, p))).map(|(_, p)| p.clone()).collect();
        if styles.is_empty() && empty.is_empty() {
            continue;
        }
        crate::right::row(ui, 24.0, |ui| widgets::label(ui, slot.title(), t.text_secondary));
        let list = lines(&styles, &empty, &app.ui.style_folds);
        let dragging_here = st.drag.as_ref().filter(|d| d.slot == slot.key()).cloned();
        let mut rects: Vec<(Rect, Line)> = Vec::new();
        for line in &list {
            let w = ui.available_width();
            match line {
                Line::Group { path, depth } => {
                    let (r, resp) = ui.allocate_exact_size(vec2(w, HEAD_H), Sense::click_and_drag());
                    let x = r.min.x + widgets::PAD + *depth as f32 * INDENT;
                    let closed = app.ui.style_folds.contains(path);
                    if resp.hovered() && st.drag.is_none() {
                        ui.painter().rect_filled(r, CornerRadius::ZERO, t.hover);
                    }
                    let chevron = Rect::from_center_size(pos2(x + 5.0, r.center().y), vec2(9.0, 9.0));
                    icons::paint(ui.painter(), chevron, if closed { Icon::ChevronRight } else { Icon::Chevron }, t.icon_secondary);
                    let text_r = Rect::from_min_max(pos2(x + 16.0, r.min.y + 3.0), pos2(r.max.x - widgets::PAD, r.max.y - 3.0));
                    let renaming = st.renaming.as_ref().is_some_and(|(k, p)| k == slot.key() && p == path);
                    if renaming {
                        let rid = Id::unique(("style_group_rename", slot.key(), path));
                        let start = ctx.data(|d| d.get_temp::<bool>(rid.with("started"))).is_none();
                        let (out, open) = widgets::rename_field(ui, rid, last(path), text_r, semibold(11.0), start);
                        ctx.data_mut(|d| d.insert_temp(rid.with("started"), true));
                        if let Some(name) = out.map(|n| n.replace('/', " ").trim().to_string()).filter(|n| !n.is_empty() && n != last(path)) {
                            let to = join(parent(path), &name);
                            if styles.iter().any(|s| inside(&s.name, path)) {
                                cmds = Some(("style.renameFolder", json!({"folder": path, "name": to})));
                            }
                            for e in st.empty.iter_mut().filter(|(k, p)| k == slot.key() && p == path) {
                                e.1 = to.clone();
                            }
                            if app.ui.style_folds.remove(path) {
                                app.ui.style_folds.insert(to);
                                fold_changed = true;
                            }
                        }
                        if !open {
                            st.renaming = None;
                            ctx.data_mut(|d| d.remove::<bool>(rid.with("started")));
                        }
                    } else {
                        let count = styles.iter().filter(|s| inside(&s.name, path)).count();
                        let name = widgets::elide_font(ui, last(path), text_r.width() - 30.0, semibold(11.0));
                        ui.painter().text(text_r.left_center(), Align2::LEFT_CENTER, name, semibold(11.0), t.text);
                        if count == 0 {
                            ui.painter().text(
                                pos2(r.max.x - widgets::PAD, r.center().y),
                                Align2::RIGHT_CENTER,
                                tr("Drag styles here"),
                                regular(10.0),
                                t.text_tertiary,
                            );
                        }
                    }
                    let resp = resp.tip(path.replace('/', " / "));
                    if resp.double_clicked() {
                        st.renaming = Some((slot.key().to_string(), path.clone()));
                    } else if resp.clicked() && !renaming {
                        if !app.ui.style_folds.remove(path) {
                            app.ui.style_folds.insert(path.clone());
                        }
                        fold_changed = true;
                    }
                    if resp.drag_started() && !renaming {
                        let ids = styles.iter().filter(|s| inside(&s.name, path)).map(|s| s.id.clone()).collect();
                        st.drag = Some(Drag { slot: slot.key(), styles: ids, group: Some(path.clone()), label: last(path).to_string() });
                    }
                    crate::right::context_popup(&resp).show(|ui| {
                        if crate::right::menu_item(ui, tr("Rename"), "") {
                            st.renaming = Some((slot.key().to_string(), path.clone()));
                            ui.close();
                        }
                        if crate::right::menu_item(ui, tr("Ungroup"), "") {
                            let moved: Vec<String> = styles.iter().filter(|s| inside(&s.name, path)).map(|s| s.id.clone()).collect();
                            // Its styles and groups take its place in the group above.
                            let mut list = Vec::new();
                            for s in styles.iter().filter(|s| moved.contains(&s.id)) {
                                let inner = parts(&s.name).get(path.split('/').count()..).map(|x| x.join("/")).unwrap_or_default();
                                list.push(json!({"command": "style.update", "params": {"style": s.id, "name": join(parent(path), &inner)}}));
                            }
                            st.empty.retain(|(k, p)| !(k == slot.key() && p == path));
                            if !list.is_empty() {
                                cmds = Some(("command.batch", json!({"label": "Ungroup styles", "commands": list})));
                            }
                            ui.close();
                        }
                    });
                    rects.push((r, line.clone()));
                }
                Line::Style { index, depth } => {
                    let Some(s) = styles.get(*index) else { continue };
                    let (r, resp) = ui.allocate_exact_size(vec2(w, ROW_H), Sense::click_and_drag());
                    let on = st.selected.contains(&s.id);
                    let moving = dragging_here.as_ref().is_some_and(|d| d.styles.contains(&s.id));
                    if on {
                        ui.painter().rect_filled(r, CornerRadius::ZERO, t.selected_row);
                    } else if resp.hovered() && st.drag.is_none() {
                        ui.painter().rect_filled(r, CornerRadius::ZERO, t.hover);
                    }
                    let x = r.min.x + widgets::PAD + *depth as f32 * INDENT + 14.0;
                    let icon_r = Rect::from_center_size(pos2(x + 8.0, r.center().y), vec2(16.0, 16.0));
                    let dim = |c: egui::Color32| if moving { c.gamma_multiply(0.4) } else { c };
                    match &s.kind {
                        StyleKind::Paint { paints } => {
                            let p = paints.last().cloned().unwrap_or_else(|| Paint::solid(figcraft_color::Color::TRANSPARENT));
                            widgets::paint_swatch(ui.painter(), icon_r, &p);
                        }
                        _ => icons::paint(ui.painter(), icon_r, slot.icon(), dim(t.icon_secondary)),
                    }
                    let hovered = resp.hovered() || ui.rect_contains_pointer(r);
                    let mut name_w = r.max.x - widgets::PAD - (x + 24.0);
                    // The edit button on hover; a text style's size / line height otherwise.
                    let eb = Rect::from_center_size(pos2(r.max.x - widgets::PAD - 10.0, r.center().y), vec2(22.0, 22.0));
                    let mut edit_clicked = false;
                    if hovered && st.drag.is_none() {
                        let eresp = ui.interact(eb, Id::unique(("style_edit_btn", &s.id)), Sense::click()).tip(tr("Edit style"));
                        if eresp.hovered() {
                            ui.painter().rect_filled(eb, CornerRadius::same(4), t.field);
                        }
                        icons::paint(ui.painter(), eb.shrink(5.0), Icon::Adjust, t.icon);
                        edit_clicked = eresp.clicked();
                        name_w -= 28.0;
                    } else if let StyleKind::Text { style, .. } = &s.kind {
                        let meta = format!("{}/{}", figcraft_doc::trim_num(style.font_size), style.line_height.label());
                        let g = ui.painter().text(pos2(r.max.x - widgets::PAD, r.center().y), Align2::RIGHT_CENTER, meta, regular(10.0), t.text_secondary);
                        name_w -= g.width() + 8.0;
                    }
                    crate::bidi::paint_name(ui.painter(), pos2(x + 24.0, r.center().y), leaf(&s.name), regular(11.0), dim(t.text), name_w);
                    let tip = if s.description.is_empty() { s.name.clone() } else { format!("{}\n{}", s.name, s.description) };
                    let resp = resp.tip(tip);
                    if edit_clicked || resp.double_clicked() {
                        edit = Some(s.id.clone());
                    } else if resp.clicked() {
                        let m = ui.input(|i| i.modifiers);
                        if m.command || m.shift {
                            if !st.selected.retain_toggle(&s.id) {
                                st.selected.push(s.id.clone());
                            }
                            // Types never mix in a selection.
                            st.selected.retain(|id| styles.iter().any(|x| &x.id == id));
                        } else {
                            st.selected = vec![s.id.clone()];
                        }
                    }
                    if resp.drag_started() {
                        let ids: Vec<String> = if on { st.selected.clone() } else { vec![s.id.clone()] };
                        let label = if ids.len() > 1 { format!("{} ×{}", leaf(&s.name), ids.len()) } else { leaf(&s.name).to_string() };
                        st.drag = Some(Drag { slot: slot.key(), styles: ids, group: None, label });
                    }
                    crate::right::context_popup(&resp).show(|ui| {
                        if crate::right::menu_item(ui, tr("Edit style"), "") {
                            edit = Some(s.id.clone());
                            ui.close();
                        }
                        // Several text styles selected: edit and bind them together.
                        let texts: Vec<String> = if slot == StyleSlot::Text && on {
                            styles.iter().filter(|x| st.selected.contains(&x.id)).map(|x| x.id.clone()).collect()
                        } else {
                            Vec::new()
                        };
                        if texts.len() > 1 && crate::right::menu_item(ui, &crate::i18n::trf("Edit {n} text styles…", &[("n", &texts.len().to_string())]), "") {
                            let y = ui.ctx().input(|i| i.pointer.interact_pos()).map_or(140.0, |p| p.y - 12.0);
                            crate::text_bulk_ui::open(ui.ctx(), texts, y);
                            ui.close();
                        }
                        if crate::right::menu_item(ui, tr("Group"), crate::shortcuts::shortcut("object.group").unwrap_or("")) {
                            if !st.selected.contains(&s.id) {
                                st.selected = vec![s.id.clone()];
                            }
                            cmds = Some(("figcraftplus.group", Value::Null));
                            ui.close();
                        }
                        if !folder_of(&s.name).is_empty() && crate::right::menu_item(ui, tr("Remove from group"), "") {
                            let f = folder_of(&s.name);
                            let mut name = parent(&f).to_string();
                            name = join(&name, leaf(&s.name));
                            cmds = Some(("style.update", json!({"style": s.id, "name": name})));
                            ui.close();
                        }
                        if crate::right::menu_item(ui, tr("Duplicate"), "") {
                            cmds = Some(("style.duplicate", json!({"style": s.id})));
                            ui.close();
                        }
                        if crate::right::menu_item(ui, tr("Delete"), "") {
                            cmds = Some(("style.delete", json!({"style": s.id})));
                            ui.close();
                        }
                    });
                    rects.push((r, line.clone()));
                }
            }
        }
        // The drop target under the pointer while this type's styles drag.
        if let (Some(d), Some(p)) = (&dragging_here, pointer)
            && let Some(found) = drop_target(&styles, &rects, d, p)
        {
            drop_at = Some(found);
        }
    }
    // The dragged rows follow the pointer; the drop shows as a line (between rows) or an outline
    // (into a group).
    if let (Some(d), Some(p)) = (&st.drag, pointer) {
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::unique("style_group_drag")));
        let font = regular(11.0);
        let g = painter.layout_no_wrap(d.label.clone(), font, t.text);
        let chip = Rect::from_min_size(p + vec2(12.0, 6.0), g.size() + vec2(16.0, 10.0));
        painter.rect_filled(chip, CornerRadius::same(5), t.panel);
        painter.rect_stroke(chip, CornerRadius::same(5), Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
        painter.galley(chip.min + vec2(8.0, 5.0), g, t.text);
        if let Some((_, r, into)) = &drop_at {
            if *into {
                painter.rect_stroke(*r, CornerRadius::same(4), Stroke::new(1.5, t.accent), egui::StrokeKind::Inside);
            } else {
                painter.hline(r.x_range(), r.min.y, Stroke::new(2.0, t.accent));
            }
        }
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        if released {
            if let Some((drop, _, _)) = drop_at.clone() {
                let styles: Vec<&Style> = library::local_styles(doc).filter(|s| SLOTS.iter().any(|sl| sl.key() == d.slot && sl.holds(&s.kind))).collect();
                let list = move_commands(&styles, &d.styles, d.group.as_deref(), &drop);
                if !list.is_empty() {
                    cmds = Some(("command.batch", json!({"label": "Move styles", "commands": list})));
                }
                // A filled empty group isn't empty any more; a moved group keeps its fold.
                st.empty.retain(|(k, p)| !(k == d.slot && *p == drop.folder));
                if let Some(g) = &d.group
                    && app.ui.style_folds.remove(g)
                {
                    app.ui.style_folds.insert(join(&drop.folder, last(g)));
                    fold_changed = true;
                }
            }
            st.drag = None;
        }
    } else if released || !ctx.input(|i| i.pointer.any_down()) {
        st.drag = None;
    }
    if fold_changed {
        save_folds(app);
    }
    let group_now = matches!(cmds, Some(("figcraftplus.group", _)));
    ctx.data_mut(|d| d.insert_temp(state_id(), st));
    if group_now {
        group_selected(app, &ctx);
    } else if let Some((cmd, q)) = cmds {
        app.run(cmd, q);
    }
    if let Some(id) = edit {
        let y = ctx.input(|i| i.pointer.interact_pos()).map_or(140.0, |p| p.y - 12.0);
        crate::style_dialog_ui::open_edit(app, &ctx, &id, y);
    }
}

/// The drop for drag `d` with the pointer at `p` over one type's rows (`rects`): (drop, the row's
/// rect for the marker, whether it goes into a group).
fn drop_target(styles: &[&Style], rects: &[(Rect, Line)], d: &Drag, p: Pos2) -> Option<(Drop, Rect, bool)> {
    let first = rects.first()?.0;
    let lastr = rects.last()?.0;
    if p.y < first.min.y - 4.0 || p.y > lastr.max.y + 24.0 || p.x < first.min.x || p.x > first.max.x {
        return None;
    }
    let moved = &d.styles;
    // Below the last row: the end of the type's list, outside groups.
    if p.y > lastr.max.y {
        let r = Rect::from_min_max(pos2(lastr.min.x, lastr.max.y), pos2(lastr.max.x, lastr.max.y + 2.0));
        return Some((Drop { folder: String::new(), before: end_of(styles, "", moved) }, r, false));
    }
    let (r, line) = rects.iter().find(|(r, _)| p.y >= r.min.y && p.y < r.max.y).or(rects.first())?;
    let blocked = |folder: &str| d.group.as_ref().is_some_and(|g| folder == g || folder.starts_with(&format!("{g}/")));
    let upper = p.y < r.center().y;
    let out = match line {
        Line::Group { path, depth } => {
            let indent = *depth as f32 * INDENT;
            if p.y < r.min.y + r.height() * 0.3 {
                // Before the group, in the group above it.
                let before = styles.iter().find(|s| inside(&s.name, path) && !moved.contains(&s.id)).map(|s| s.id.clone());
                let f = parent(path).to_string();
                (Drop { folder: f.clone(), before: before.or_else(|| end_of(styles, &f, moved)) }, line_at(*r, r.min.y, indent), false, f)
            } else {
                (Drop { folder: path.clone(), before: end_of(styles, path, moved) }, *r, true, path.clone())
            }
        }
        Line::Style { index, depth } => {
            let s = styles.get(*index)?;
            let indent = *depth as f32 * INDENT;
            let f = folder_of(&s.name);
            if upper {
                let before = if moved.contains(&s.id) { next_unmoved(styles, *index, moved) } else { Some(s.id.clone()) };
                (Drop { folder: f.clone(), before }, line_at(*r, r.min.y, indent), false, f)
            } else {
                (Drop { folder: f.clone(), before: next_unmoved(styles, *index, moved) }, line_at(*r, r.max.y, indent), false, f)
            }
        }
    };
    if blocked(&out.3) {
        return None;
    }
    Some((out.0, out.1, out.2))
}

/// A thin marker rect at `y` across row `r` from its indent.
fn line_at(r: Rect, y: f32, indent: f32) -> Rect {
    Rect::from_min_max(pos2(r.min.x + widgets::PAD + indent, y - 1.0), pos2(r.max.x - widgets::PAD, y + 1.0))
}

/// Toggle membership; true when it was there (and is now removed).
trait RetainToggle {
    fn retain_toggle(&mut self, id: &str) -> bool;
}

impl RetainToggle for Vec<String> {
    fn retain_toggle(&mut self, id: &str) -> bool {
        let before = self.len();
        self.retain(|x| x != id);
        self.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_engine::Session;

    fn session(names: &[&str]) -> Session {
        let mut s = Session::new();
        for n in names {
            s.execute("style.create", &json!({"type": "paint", "name": n, "value": crate::styles_plus_ui::default_value(StyleSlot::Fill)})).unwrap();
        }
        s
    }

    fn names(s: &Session) -> Vec<String> {
        s.doc().unwrap().styles.iter().map(|x| x.name.clone()).collect()
    }

    fn styles(s: &Session) -> Vec<Style> {
        s.doc().unwrap().styles.clone()
    }

    #[test]
    fn groups_nest_with_their_own_styles_first() {
        let s = session(&["Brand/Blue/500", "Red", "Brand/Ink", "Brand / Blue/600", "Grey/100"]);
        let all = styles(&s);
        let refs: Vec<&Style> = all.iter().collect();
        let l = lines(&refs, &["Empty".into()], &BTreeSet::new());
        let g = |p: &str, depth| Line::Group { path: p.into(), depth };
        let st = |index, depth| Line::Style { index, depth };
        assert_eq!(l, vec![st(1, 0), g("Brand", 0), st(2, 1), g("Brand/Blue", 1), st(0, 2), st(3, 2), g("Grey", 0), st(4, 1), g("Empty", 0)]);
        let folded = lines(&refs, &[], &BTreeSet::from(["Brand".to_string()]));
        assert_eq!(folded, vec![st(1, 0), g("Brand", 0), g("Grey", 0), st(4, 1)]);
    }

    #[test]
    fn a_group_moves_into_another_with_its_inner_groups() {
        let mut s = session(&["Brand/Blue/500", "Brand/Ink", "Grey/100", "Red"]);
        let all = styles(&s);
        let refs: Vec<&Style> = all.iter().collect();
        let moved: Vec<String> = refs.iter().filter(|x| inside(&x.name, "Brand")).map(|x| x.id.clone()).collect();
        let drop = Drop { folder: "Grey".into(), before: end_of(&refs, "Grey", &moved) };
        let cmds = move_commands(&refs, &moved, Some("Brand"), &drop);
        s.execute("command.batch", &json!({"label": "Move", "commands": cmds})).unwrap();
        assert_eq!(names(&s), ["Grey/100", "Grey/Brand/Blue/500", "Grey/Brand/Ink", "Red"]);
    }

    #[test]
    fn a_style_moves_between_groups_and_out_of_them() {
        let mut s = session(&["A/one", "A/two", "B/three"]);
        let all = styles(&s);
        let refs: Vec<&Style> = all.iter().collect();
        let one = refs[0].id.clone();
        let three = refs[2].id.clone();
        let cmds = move_commands(&refs, std::slice::from_ref(&one), None, &Drop { folder: "B".into(), before: Some(three) });
        s.execute("command.batch", &json!({"label": "Move", "commands": cmds})).unwrap();
        assert_eq!(names(&s), ["A/two", "B/one", "B/three"]);
        let all = styles(&s);
        let refs: Vec<&Style> = all.iter().collect();
        let cmds = move_commands(&refs, std::slice::from_ref(&one), None, &Drop { folder: String::new(), before: None });
        s.execute("command.batch", &json!({"label": "Move", "commands": cmds})).unwrap();
        assert_eq!(names(&s), ["A/two", "B/three", "one"]);
    }

    #[test]
    fn new_groups_take_the_next_free_name() {
        let s = session(&["Group 1/a", "x"]);
        let all = styles(&s);
        let refs: Vec<&Style> = all.iter().collect();
        assert_eq!(new_group_name(&refs, &[], ""), "Group 2");
        assert_eq!(new_group_name(&refs, &["Group 2".into()], ""), "Group 3");
        assert_eq!(new_group_name(&refs, &[], "Group 1"), "Group 1");
        assert_eq!(folder_of("Brand / Blue/500"), "Brand/Blue");
        assert_eq!(folder_of("Red"), "");
    }
}
