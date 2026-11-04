//! FigCraft Plus: gradients blended in OKLCH (`figcraft_engine::gradient_oklch`).
//!
//! The colour picker of a gradient — a layer's fill or stroke, a paint style's — gets a
//! **Blend · sRGB | OKLCH** switch under its stop actions. In OKLCH the stops the engine adds
//! between two key stops are steps, not stops of yours: the picker's bar and the canvas show only
//! the key stops.

use egui::{Align2, CornerRadius, Id, Rect, Sense, Ui, pos2, vec2};
use figcraft_color::Gradient;
use figcraft_doc::{Document, NodeId};
use figcraft_engine::gradient_oklch::{self, Rule};
use serde_json::json;

use crate::FigApp;
use crate::colorpicker::PickerTarget;
use crate::i18n::tr;
use crate::theme::{Tokens, regular};

/// The OKLCH rule of the gradient `target` edits, if it blends in OKLCH.
fn rule(doc: &Document, target: &PickerTarget) -> Option<Rule> {
    match target {
        PickerTarget::Paint { fills, index, ids } => {
            let list = if *fills { "fills" } else { "strokes" };
            gradient_oklch::layer_rules(doc, *ids.first()?).into_iter().find(|r| r.list == list && r.index == *index)
        }
        PickerTarget::Style { id, index } => gradient_oklch::style_rules(doc, id).into_iter().find(|r| r.index == *index),
        _ => None,
    }
}

/// Whether stop `i` of `g` (the gradient `target` edits) is one of the engine's steps.
pub(crate) fn is_step(app: &FigApp, target: &PickerTarget, g: &Gradient, i: usize) -> bool {
    let Some(r) = app.session.doc().ok().and_then(|d| rule(d, target)) else { return false };
    gradient_oklch::steps_of(g, &r.keys).get(i).copied().unwrap_or(false)
}

/// On the canvas: whether stop `i` of layer `id`'s gradient `g` is a step.
pub(crate) fn canvas_step(app: &FigApp, id: NodeId, g: &Gradient, i: usize) -> bool {
    let Ok(doc) = app.session.doc() else { return false };
    gradient_oklch::layer_rules(doc, id).iter().any(|r| gradient_oklch::steps_of(g, &r.keys).get(i).copied().unwrap_or(false))
}

/// The Blend switch (`w` wide), under the gradient's stop actions.
pub(crate) fn row(app: &mut FigApp, ui: &mut Ui, target: &PickerTarget, w: f32) {
    let (params, on) = {
        let Ok(doc) = app.session.doc() else { return };
        let on = rule(doc, target).is_some();
        let params = match target {
            PickerTarget::Paint { fills, index, ids } => json!({"ids": ids, "list": if *fills { "fills" } else { "strokes" }, "index": index}),
            PickerTarget::Style { id, index } => json!({"style": id, "index": index}),
            _ => return,
        };
        (params, on)
    };
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
    let label_resp = ui.interact(Rect::from_min_max(r.min, pos2(r.min.x + 90.0, r.max.y)), Id::unique("figcraftplus_blend_label"), Sense::hover());
    label_resp.on_hover_text(tr("How the gradient mixes its colours between two stops: sRGB as drawn by default, or OKLCH, as CSS writes linear-gradient(in oklch, …)"));
    ui.painter().text(pos2(r.min.x + 16.0, r.center().y), Align2::LEFT_CENTER, tr("Blend"), regular(11.0), t.text_secondary);
    let seg_w = 56.0;
    let track = Rect::from_min_max(pos2(r.max.x - 16.0 - seg_w * 2.0, r.min.y + 4.0), pos2(r.max.x - 16.0, r.max.y - 4.0));
    ui.painter().rect_filled(track, CornerRadius::same(6), t.field);
    for (i, (text, is_on)) in [("sRGB", !on), ("OKLCH", on)].into_iter().enumerate() {
        let seg = Rect::from_min_size(pos2(track.min.x + seg_w * i as f32, track.min.y), vec2(seg_w, track.height())).shrink(2.0);
        let resp = ui.interact(seg, Id::unique(("figcraftplus_blend_seg", i)), Sense::click());
        if is_on {
            ui.painter().rect_filled(seg, CornerRadius::same(5), t.panel);
        } else if resp.hovered() {
            ui.painter().rect_filled(seg, CornerRadius::same(5), t.hover);
        }
        ui.painter().text(seg.center(), Align2::CENTER_CENTER, text, regular(11.0), if is_on { t.text } else { t.text_secondary });
        if resp.clicked() && !is_on {
            let mut p = params.clone();
            p["space"] = json!(if i == 1 { "oklch" } else { "srgb" });
            app.run("gradient.blend", p);
            // The selected stop may have been a step: start from the first.
            ui.data_mut(|d| d.insert_temp(crate::colorpicker::stop_id(), 0usize));
        }
    }
}
