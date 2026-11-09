//! FigCraft Plus: the colour picker follows the selection.
//!
//! FigCraft places the picker beside the Design panel at the height of the control that opened it,
//! and closes it on Escape, a press outside it and a change of selection. With *Picker follows the
//! selection* on (Preferences ▸ FigCraft Plus; on by default), Plus
//! keeps it open instead when other layers are selected — on the canvas or in Layers — and they
//! have the same slot (Fill 1 stays Fill 1, the second shadow stays the second shadow): it moves to
//! them. When they have no such slot, or nothing is selected, it closes as FigCraft's does. A click
//! elsewhere that selects nothing new still closes it; drags (moving layers, gradient handles,
//! scrolling) leave it open. Turned off, the picker behaves exactly as FigCraft's. Opened from a
//! style dialog, it opens beside the dialog rather than over it.

use egui::{Id, Pos2, Rect, pos2};
use figcraft_doc::NodeId;

use crate::FigApp;
use crate::colorpicker::PickerTarget;
use crate::popover::Open;

/// The selection the open picker follows.
fn followed_id() -> Id {
    Id::unique("figcraftplus_picker_selection")
}

/// `t` on the layers `ids` (paints and effects; other targets don't follow the selection).
fn retarget(t: &PickerTarget, ids: &[NodeId]) -> Option<PickerTarget> {
    match t {
        PickerTarget::Paint { fills, index, .. } => Some(PickerTarget::Paint { fills: *fills, index: *index, ids: ids.to_vec() }),
        PickerTarget::Effect { index, secondary, .. } => Some(PickerTarget::Effect { ids: ids.to_vec(), index: *index, secondary: *secondary }),
        _ => None,
    }
}

/// A press that may close the picker on its release: where, and the selection and target then.
#[derive(Clone)]
struct Press {
    at: Pos2,
    selection: Vec<NodeId>,
    target: PickerTarget,
}

fn press_id() -> Id {
    Id::unique("figcraftplus_picker_press")
}

#[cfg(test)]
thread_local! {
    static FOLLOWS_FOR_TEST: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// For tests: the preference on this thread (None: off, FigCraft's own behaviour).
#[cfg(test)]
pub(crate) fn set_follows_for_test(on: Option<bool>) {
    FOLLOWS_FOR_TEST.with(|f| f.set(on));
}

/// Whether the picker follows the selection (the preference).
pub(crate) fn follows() -> bool {
    preference()
}

// Tests check FigCraft's own picker unless they turn following on.
#[cfg(test)]
fn preference() -> bool {
    FOLLOWS_FOR_TEST.with(std::cell::Cell::get).unwrap_or(false)
}

#[cfg(not(test))]
fn preference() -> bool {
    crate::prefs_plus_ui::get().picker_follows_selection
}

/// What FigCraft's picker must not take as a press outside it while it follows the selection: the
/// canvas and the panels left of it, where other layers get selected ([`follow`] decides on the
/// release whether a click there closes it). Presses in the Design panel close it as FigCraft's.
pub(crate) fn held(app: &FigApp, ctx: &egui::Context) -> Option<Rect> {
    let follows_target = app.ui.picker.as_ref().is_some_and(|t| retarget(t, &[]).is_some());
    let canvas = app.canvas.rect;
    (follows() && follows_target && canvas.is_positive()).then(|| Rect::from_min_max(pos2(ctx.content_rect().min.x, canvas.min.y), canvas.max))
}

/// Before the picker draws: move it to the same slot of a new selection (FigCraft then closes it
/// only when that slot has no colour), and close it on a click that selects nothing new.
/// `exists` tells whether a target has a colour to edit. Does nothing when the preference is off.
pub(crate) fn follow(app: &mut FigApp, ctx: &egui::Context, exists: impl Fn(&FigApp, &PickerTarget) -> bool) {
    let target = app.ui.picker.clone();
    let Some(target) = target.filter(|t| follows() && retarget(t, &[]).is_some()) else {
        ctx.data_mut(|d| {
            d.remove::<Vec<NodeId>>(followed_id());
            d.remove::<Press>(press_id());
            d.remove::<(Press, u64)>(decide_id());
        });
        return;
    };
    let selection = app.session.selection();
    let seen: Vec<NodeId> = ctx.data(|d| d.get_temp(followed_id())).unwrap_or_else(|| selection.clone());
    if seen != selection
        && !selection.is_empty()
        && let Some(t) = retarget(&target, &selection).filter(|t| exists(app, t))
    {
        app.ui.picker = Some(t);
    }
    ctx.data_mut(|d| d.insert_temp(followed_id(), selection.clone()));
    // A press outside the picker, its button and the popovers it opened…
    let (pressed, released, pos) = ctx.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_released(), i.pointer.interact_pos()));
    if pressed && let Some(p) = pos {
        let inside = ctx.memory(|m| m.area_rect(crate::colorpicker::window_id())).is_some_and(|r| r.contains(p))
            || app.ui.picker_button.is_some_and(|r| r.contains(p))
            || crate::variable_ui::picker_rect(ctx).is_some_and(|r| r.contains(p))
            || crate::popover_ui::open_rects().iter().any(|r| r.contains(p));
        // The eyedropper's click samples a colour for the picker.
        let sampling = app.session.tool.pick_only || app.session.tool.picked.is_some();
        let held = held(app, ctx).is_some_and(|r| r.contains(p));
        if held && !inside && !sampling && !egui::Popup::is_any_open(ctx) {
            // The selection before this frame: the canvas may have selected on this very press.
            let press = Press { at: p, selection: seen.clone(), target: target.clone() };
            ctx.data_mut(|d| d.insert_temp(press_id(), press));
        }
    }
    // …closes it after its release when it was a click that selected no other layers and opened no
    // other swatch (a drag leaves it open). Decided a frame later: Layers selects on the release,
    // after the picker has drawn.
    let frame = ctx.cumulative_frame_nr();
    if released && let Some(press) = ctx.data(|d| d.get_temp::<Press>(press_id())) {
        let click = pos.is_none_or(|p| p.distance(press.at) < 6.0);
        ctx.data_mut(|d| {
            d.remove::<Press>(press_id());
            if click {
                d.insert_temp(decide_id(), (press, frame));
            }
        });
    } else if let Some((press, at)) = ctx.data(|d| d.get_temp::<(Press, u64)>(decide_id()))
        && frame > at
    {
        ctx.data_mut(|d| d.remove::<(Press, u64)>(decide_id()));
        let reselected = !selection.is_empty() && selection != press.selection;
        let moved = app.ui.picker.as_ref() != Some(&press.target);
        if !reselected && !moved {
            app.ui.picker = None;
        }
    }
}

fn decide_id() -> Id {
    Id::unique("figcraftplus_picker_decide")
}

/// Where the picker (`w` wide) opens: beside the style dialog when its button is in one, else
/// where FigCraft opens it (`open`).
pub(crate) fn open(app: &FigApp, ctx: &egui::Context, open: Open, w: f32) -> Open {
    let Some(button) = app.ui.picker_button else { return open };
    let dialog = ctx.memory(|m| m.area_rect(Id::unique("figcraftplus_style_dialog_area"))).filter(|d| d.contains(button.center()));
    match dialog {
        Some(d) => Open::At(pos2(d.min.x - w - 8.0, button.min.y - 8.0)),
        None => open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_follow_the_selection() {
        let a = PickerTarget::Paint { fills: true, index: 1, ids: vec![NodeId(1)] };
        let b = PickerTarget::Paint { fills: true, index: 1, ids: vec![NodeId(2), NodeId(3)] };
        assert_eq!(retarget(&a, &[NodeId(2), NodeId(3)]), Some(b));
        let fx = PickerTarget::Effect { ids: vec![NodeId(1)], index: 0, secondary: true };
        assert_eq!(retarget(&fx, &[NodeId(4)]), Some(PickerTarget::Effect { ids: vec![NodeId(4)], index: 0, secondary: true }));
        assert_eq!(retarget(&PickerTarget::PageBackground, &[NodeId(4)]), None);
    }
}
