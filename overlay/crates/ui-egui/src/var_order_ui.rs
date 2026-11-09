//! FigCraft Plus: ordering in the Variables editor.
//!
//! - Variables: drag the selection (FigCraft's: click, Shift+click, Ctrl/Cmd+click) (by its name) to another place in the table — or onto another group's header, or a
//!   group in the sidebar, to move it into that group (renamed). Alt+↑/↓ moves the selection a row.
//! - Groups: drag a group's header in the table, or a group in the sidebar, before or after another
//!   one; drop it on "All variables" to move it to the top level.
//! - Collections: drag them in the sidebar.
//! - Sorting: a group's or collection's context menu sorts it by name (xs, s, m … 6xl; 1, 2, 10)
//!   or by value (`variable.sort`).
//!
//! Groups show in the order of their first variable, as the file keeps them (not alphabetically),
//! so moving a group moves it in the table and the sidebar alike. Each move or sort is one undo
//! step (`figcraft_engine::cmd::var_order`).

use egui::{Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use serde_json::json;

use crate::FigApp;
use crate::right::menu_item;
use crate::theme::{Tokens, regular, semibold};

/// What is being dragged.
#[derive(Clone, Debug, PartialEq)]
enum Drag {
    /// Variables (ids) of collection `coll`.
    Vars { coll: String, ids: Vec<String> },
    /// Group `path` of collection `coll`.
    Group { coll: String, path: String },
    /// A collection.
    Coll { id: String },
}

/// Where a drop would go.
#[derive(Clone, Debug, PartialEq)]
enum Drop {
    Vars { before: Option<String>, group: String },
    Group { before: Option<String>, into: String },
    Coll { before: Option<String> },
}

/// The visible rows of the table: (collection, [(id, name)]), set every frame.
#[derive(Clone, Debug, Default)]
struct Visible {
    coll: String,
    rows: Vec<(String, String)>,
}

fn drag_id() -> Id {
    Id::unique("vo_drag")
}
fn drop_id() -> Id {
    Id::unique("vo_drop")
}
fn sel_id() -> Id {
    Id::unique("vo_sel")
}
fn visible_id() -> Id {
    Id::unique("vo_visible")
}
fn groups_id() -> Id {
    Id::unique("vo_groups")
}
fn colls_id() -> Id {
    Id::unique("vo_colls")
}

fn group_of(name: &str) -> &str {
    name.rsplit_once('/').map(|(g, _)| g).unwrap_or("")
}

fn get<T: Clone + Send + Sync + 'static + Default>(ctx: &egui::Context, id: Id) -> T {
    ctx.data(|d| d.get_temp::<T>(id)).unwrap_or_default()
}

fn dragging(ctx: &egui::Context) -> Option<Drag> {
    ctx.data(|d| d.get_temp::<Option<Drag>>(drag_id())).flatten()
}

fn set_drop(ctx: &egui::Context, d: Drop) {
    ctx.data_mut(|m| m.insert_temp(drop_id(), Some(d)));
}

/// The selected variables' ids.
fn selection(ctx: &egui::Context) -> Vec<String> {
    get(ctx, sel_id())
}

// ---- ordering what the editor lists -------------------------------------------------------------

/// The sort key of group path `g` given the group order `first` (path → first position): the
/// first positions of its ancestors and itself, so a group follows its parent and siblings keep
/// the file's order.
fn tree_key(g: &str, first: &std::collections::HashMap<String, usize>) -> Vec<usize> {
    let mut key = Vec::new();
    let mut acc = String::new();
    for part in g.split('/').filter(|p| !p.is_empty()) {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        key.push(first.get(&acc).copied().unwrap_or(usize::MAX));
    }
    key
}

/// Where each group path (and its ancestors) first comes in `names`.
fn first_positions<'a>(names: impl Iterator<Item = &'a str>) -> std::collections::HashMap<String, usize> {
    let mut first = std::collections::HashMap::new();
    for (i, n) in names.enumerate() {
        let mut g = group_of(n);
        while !g.is_empty() {
            first.entry(g.to_string()).or_insert(i);
            g = group_of(g);
        }
    }
    first
}

/// The table's rows in the file's order, grouped (groups by their first variable, nested under
/// their parents) — in place of an alphabetical sort of groups.
pub(crate) fn sort_rows<T>(list: &mut [T], name: impl Fn(&T) -> &str) {
    let first = first_positions(list.iter().map(&name));
    list.sort_by_cached_key(|v| tree_key(group_of(name(v)), &first));
}

/// The sidebar's group paths in the file's order (parents before children) — in place of `sort`.
pub(crate) fn order_groups(groups: &mut [String], names: &[&str]) {
    let first = first_positions(names.iter().copied());
    groups.sort_by_cached_key(|g| tree_key(g, &first));
}

/// The table lists these rows this frame (for Shift ranges, drop places and Alt+arrows).
pub(crate) fn visible<'a>(ctx: &egui::Context, coll: &str, rows: impl Iterator<Item = (&'a str, &'a str)>) {
    let v = Visible { coll: coll.to_string(), rows: rows.map(|(i, n)| (i.to_string(), n.to_string())).collect() };
    ctx.data_mut(|d| d.insert_temp(visible_id(), v));
}

// ---- rows ---------------------------------------------------------------------------------------

fn line(ui: &Ui, r: Rect, top: bool) {
    let t = Tokens::get(ui.ctx());
    let y = if top { r.min.y } else { r.max.y };
    ui.painter().line_segment([pos2(r.min.x + 8.0, y), pos2(r.max.x - 8.0, y)], Stroke::new(2.0, t.accent));
    ui.painter().circle_stroke(pos2(r.min.x + 8.0, y), 3.0, Stroke::new(1.5, t.accent));
}

fn outline(ui: &Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    ui.painter().rect_stroke(r.shrink(1.0), CornerRadius::same(5), Stroke::new(2.0, t.accent), StrokeKind::Inside);
}

/// Whether the pointer is over `r`, and in its top half.
fn over(ui: &Ui, r: Rect) -> Option<bool> {
    let p = ui.ctx().pointer_hover_pos()?;
    r.contains(p).then(|| p.y < r.center().y)
}

/// A variable's row in the table (after FigCraft allocates it): selection, dragging, dropping.
/// A variable row: drag it (with the rows selected with it) to another place. `selected` is
/// FigCraft's own selection of variables (click, Shift+click, Ctrl/Cmd+click), which the drag and
/// Alt+↑/↓ move.
#[allow(clippy::too_many_arguments)]
pub(crate) fn var_row(ui: &mut Ui, editable: bool, coll: &str, id: &str, name: &str, r: Rect, resp: &Response, selected: &[String]) {
    let ctx = ui.ctx().clone();
    if !editable {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(sel_id(), selected.to_vec()));
    let sel = selected.to_vec();
    if resp.drag_started() {
        let ids = if sel.iter().any(|s| s == id) {
            // The selection, in the table's order.
            let vis: Visible = get(&ctx, visible_id());
            vis.rows.iter().filter(|(i, _)| sel.contains(i)).map(|(i, _)| i.clone()).collect()
        } else {
            ctx.data_mut(|d| d.insert_temp(sel_id(), vec![id.to_string()]));
            vec![id.to_string()]
        };
        ctx.data_mut(|d| d.insert_temp(drag_id(), Some(Drag::Vars { coll: coll.to_string(), ids })));
    }
    // A drop place: before this row (top half) or after it (bottom half).
    if let (Some(Drag::Vars { coll: c, ids }), Some(top)) = (dragging(&ctx), over(ui, r))
        && c == coll
        && !ids.iter().any(|x| x == id)
    {
        let g = group_of(name).to_string();
        let before = if top {
            Some(id.to_string())
        } else {
            let vis: Visible = get(&ctx, visible_id());
            let at = vis.rows.iter().position(|(i, _)| i == id);
            at.and_then(|k| vis.rows.get(k + 1)).filter(|(i, n)| group_of(n) == g && !ids.contains(i)).map(|(i, _)| i.clone())
        };
        line(ui, r, top);
        set_drop(&ctx, Drop::Vars { before, group: g });
    }
}

/// A group's header in the table: dragging it moves the group; dropping variables on it moves
/// them into the group (at its top); dropping a group on it puts that one before it. Its context
/// menu sorts the group.
pub(crate) fn table_group(app: &mut FigApp, ui: &mut Ui, editable: bool, coll: &str, g: &str, width: f32) {
    let ctx = ui.ctx().clone();
    let t = Tokens::get(&ctx);
    let (r, resp) = ui.allocate_exact_size(vec2(width, 28.0), if editable { Sense::click_and_drag() } else { Sense::hover() });
    ui.painter().text(pos2(r.min.x + 16.0, r.center().y + 2.0), Align2::LEFT_CENTER, g, semibold(11.0), t.text_secondary);
    if !editable {
        return;
    }
    if resp.hovered() && dragging(&ctx).is_none() {
        ui.painter().text(pos2(r.max.x - 16.0, r.center().y + 2.0), Align2::RIGHT_CENTER, "⠿", regular(11.0), t.text_tertiary);
    }
    if resp.drag_started() {
        ctx.data_mut(|d| d.insert_temp(drag_id(), Some(Drag::Group { coll: coll.to_string(), path: g.to_string() })));
    }
    match (dragging(&ctx), over(ui, r)) {
        (Some(Drag::Vars { coll: c, .. }), Some(_)) if c == coll => {
            let vis: Visible = get(&ctx, visible_id());
            let before = vis.rows.iter().find(|(_, n)| group_of(n) == g).map(|(i, _)| i.clone());
            outline(ui, r);
            set_drop(&ctx, Drop::Vars { before, group: g.to_string() });
        }
        (Some(Drag::Group { coll: c, path }), Some(_)) if c == coll && path != g && !g.starts_with(&format!("{path}/")) => {
            line(ui, r, true);
            set_drop(&ctx, Drop::Group { before: Some(g.to_string()), into: group_of(g).to_string() });
        }
        _ => {}
    }
    crate::right::context_popup(&resp).show(|ui| sort_menu(app, ui, coll, Some(g)));
}

/// A group row of the sidebar (`g` None: "All variables"). Call right after FigCraft allocates it
/// (its response senses drags).
pub(crate) fn sidebar_group(ui: &mut Ui, editable: bool, coll: &str, g: Option<&str>, r: Rect, resp: &Response, groups: &[String]) {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.insert_temp(groups_id(), groups.to_vec()));
    if !editable {
        return;
    }
    if let Some(g) = g
        && resp.drag_started()
    {
        ctx.data_mut(|d| d.insert_temp(drag_id(), Some(Drag::Group { coll: coll.to_string(), path: g.to_string() })));
    }
    match (dragging(&ctx), over(ui, r)) {
        (Some(Drag::Vars { coll: c, .. }), Some(_)) if c == coll => {
            outline(ui, r);
            set_drop(&ctx, Drop::Vars { before: None, group: g.unwrap_or("").to_string() });
        }
        (Some(Drag::Group { coll: c, path }), Some(top)) if c == coll => match g {
            None => {
                outline(ui, r);
                set_drop(&ctx, Drop::Group { before: None, into: String::new() });
            }
            Some(g) if g != path && !g.starts_with(&format!("{path}/")) => {
                let parent = group_of(g).to_string();
                let before = if top {
                    Some(g.to_string())
                } else {
                    // The next group at the same level after this one (and its subgroups).
                    let at = groups.iter().position(|x| x == g).unwrap_or(groups.len());
                    groups.iter().skip(at + 1).find(|x| group_of(x) == parent && *x != &path).cloned()
                };
                line(ui, r, top);
                set_drop(&ctx, Drop::Group { before, into: parent });
            }
            _ => {}
        },
        _ => {}
    }
}

/// A collection row of the sidebar. Call right after FigCraft allocates it.
pub(crate) fn sidebar_collection(ui: &mut Ui, editable: bool, id: &str, r: Rect, resp: &Response, colls: &[String]) {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.insert_temp(colls_id(), colls.to_vec()));
    if !editable {
        return;
    }
    if resp.drag_started() {
        ctx.data_mut(|d| d.insert_temp(drag_id(), Some(Drag::Coll { id: id.to_string() })));
    }
    if let (Some(Drag::Coll { id: dragged }), Some(top)) = (dragging(&ctx), over(ui, r))
        && dragged != id
    {
        let before = if top {
            Some(id.to_string())
        } else {
            let at = colls.iter().position(|x| x == id).unwrap_or(colls.len());
            colls.iter().skip(at + 1).find(|x| *x != &dragged).cloned()
        };
        line(ui, r, top);
        set_drop(&ctx, Drop::Coll { before });
    }
}

/// The sort items of a group's (`g`) or collection's context menu.
pub(crate) fn sort_menu(app: &mut FigApp, ui: &mut Ui, coll: &str, g: Option<&str>) {
    let mut by = None;
    if menu_item(ui, "Sort by name (xs → 6xl, 1 → 10)", "") {
        by = Some("name");
    }
    if menu_item(ui, "Sort by value (low → high)", "") {
        by = Some("value");
    }
    if let Some(by) = by {
        let mut p = json!({"collection": coll, "by": by});
        if let Some(g) = g {
            p["group"] = json!(g);
        }
        app.run("variable.sort", p);
        ui.close();
    }
}

// ---- every frame ----------------------------------------------------------------------------------

/// Once a frame, after the editor: finish a drop, draw what is dragged, Alt+↑/↓ and Escape.
pub(crate) fn finish(app: &mut FigApp, ctx: &egui::Context) {
    let drag = dragging(ctx);
    let drop: Option<Drop> = ctx.data_mut(|d| d.remove_temp::<Option<Drop>>(drop_id())).flatten();
    if let Some(dr) = &drag {
        // What is being dragged, by the pointer.
        if let Some(p) = ctx.pointer_hover_pos() {
            let label = match dr {
                Drag::Vars { ids, .. } if ids.len() == 1 => {
                    let vis: Visible = get(ctx, visible_id());
                    vis.rows.iter().find(|(i, _)| i == &ids[0]).map(|(_, n)| n.rsplit('/').next().unwrap_or(n).to_string()).unwrap_or_default()
                }
                Drag::Vars { ids, .. } => format!("{} variables", ids.len()),
                Drag::Group { path, .. } => path.rsplit('/').next().unwrap_or(path).to_string(),
                Drag::Coll { id } => app.session.doc().ok().and_then(|d| d.variables.collection(id).map(|c| c.name.clone())).unwrap_or_default(),
            };
            ghost(ctx, p, &label);
        }
        if ctx.input(|i| i.pointer.any_released() || i.key_pressed(egui::Key::Escape)) {
            let released = ctx.input(|i| i.pointer.any_released());
            ctx.data_mut(|d| d.insert_temp::<Option<Drag>>(drag_id(), None));
            if released && let Some(drop) = drop {
                apply(app, dr, drop);
            }
        }
        return;
    }
    keys(app, ctx);
}

fn ghost(ctx: &egui::Context, p: Pos2, label: &str) {
    let t = Tokens::get(ctx);
    egui::Area::new(Id::unique("vo_ghost")).order(egui::Order::Tooltip).fixed_pos(p + vec2(14.0, 10.0)).interactable(false).show(ctx, |ui| {
        egui::Frame::NONE.fill(t.accent).corner_radius(CornerRadius::same(4)).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
            ui.add(egui::Label::new(egui::RichText::new(label).font(regular(11.0)).color(Color32::WHITE)).extend());
        });
    });
}

fn apply(app: &mut FigApp, drag: &Drag, drop: Drop) {
    match (drag, drop) {
        (Drag::Vars { ids, .. }, Drop::Vars { before, group }) => {
            let mut p = json!({"variables": ids, "group": group});
            if let Some(b) = before {
                p["before"] = json!(b);
            }
            app.run("variable.move", p);
        }
        (Drag::Group { coll, path }, Drop::Group { before, into }) => {
            let mut p = json!({"collection": coll, "group": path, "into": into});
            if let Some(b) = before {
                p["before"] = json!(b);
            }
            app.run("variable.moveGroup", p);
        }
        (Drag::Coll { id }, Drop::Coll { before }) => {
            let mut p = json!({"collection": id});
            if let Some(b) = before {
                p["before"] = json!(b);
            }
            app.run("variable.moveCollection", p);
        }
        _ => {}
    }
}

/// Alt+↑/↓ move the selected variables a row (within their group); Escape clears the selection.
fn keys(app: &mut FigApp, ctx: &egui::Context) {
    let sel = selection(ctx);
    if sel.is_empty() || ctx.memory(|m| m.focused().is_some()) {
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape) && i.modifiers.is_none()) {
        ctx.data_mut(|d| d.insert_temp::<Vec<String>>(sel_id(), Vec::new()));
        return;
    }
    let up = ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp));
    let down = !up && ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown));
    if !up && !down {
        return;
    }
    let vis: Visible = get(ctx, visible_id());
    if vis.coll.is_empty() {
        return;
    }
    let rows = &vis.rows;
    let picked: Vec<usize> = rows.iter().enumerate().filter(|(_, (i, _))| sel.contains(i)).map(|(k, _)| k).collect();
    let (Some(&lo), Some(&hi)) = (picked.first(), picked.last()) else { return };
    let ids: Vec<String> = picked.iter().filter_map(|k| rows.get(*k).map(|r| r.0.clone())).collect();
    let Some(g) = rows.get(lo).map(|r| group_of(&r.1).to_string()) else { return };
    let same = |k: usize| rows.get(k).filter(|(i, n)| group_of(n) == g && !sel.contains(i));
    let before: Option<Option<String>> = if up {
        // Before the row above the first one.
        lo.checked_sub(1).and_then(same).map(|(i, _)| Some(i.clone()))
    } else {
        // After the row below the last one: before the one after it, or the group's end.
        same(hi + 1).map(|_| same(hi + 2).map(|(i, _)| i.clone()))
    };
    let Some(before) = before else { return };
    let mut p = json!({"variables": ids, "group": g});
    if let Some(b) = before {
        p["before"] = json!(b);
    }
    app.run("variable.move", p);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_list_in_the_files_order_with_parents_first() {
        let names = ["space/gap", "color/brand/50", "number/n-xs", "color/grey/50", "color/brand/100", "number/n-s", "top"];
        let mut rows: Vec<&str> = names.to_vec();
        sort_rows(&mut rows, |n| n);
        assert_eq!(rows, ["top", "space/gap", "color/brand/50", "color/brand/100", "color/grey/50", "number/n-xs", "number/n-s"]);
        let mut groups = vec!["color".to_string(), "color/brand".into(), "color/grey".into(), "number".into(), "space".into()];
        order_groups(&mut groups, &names);
        assert_eq!(groups, ["space", "color", "color/brand", "color/grey", "number"]);
    }
}
