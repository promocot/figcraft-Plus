//! FigCraft Plus: the "+" on the Design panel's Local styles header. It offers the
//! style types — Text, Color, Effect, Layout grid — and creates the chosen one from default values
//! (nothing needs to be selected) and opens the style dialog on it (`style_dialog_ui`).

use egui::{Align2, CornerRadius, Rect, Sense, Ui, pos2, vec2};
use figcraft_color::{Color, Paint};
use figcraft_doc::{Document, Effect, LayoutGrid, TextStyle};
use serde_json::Value;

use crate::FigApp;
use crate::a11y::Tip;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::right::{menu_item, menu_popup};
use crate::style_picker::StyleSlot;
use crate::theme::{Tokens, semibold};
use crate::widgets;

/// The types the menu offers: (slot, label).
const TYPES: [(StyleSlot, &str); 4] =
    [(StyleSlot::Text, "Text"), (StyleSlot::Fill, "Color"), (StyleSlot::Effect, "Effect"), (StyleSlot::Grid, "Layout grid")];

/// A new style's starting value (`style.create {value}`): Inter Regular 12, a light grey, a soft
/// drop shadow, a 10 px grid.
pub(crate) fn default_value(slot: StyleSlot) -> Value {
    let v = match slot {
        StyleSlot::Text => serde_json::to_value(TextStyle::default()),
        StyleSlot::Fill | StyleSlot::Stroke => serde_json::to_value(vec![Paint::solid(Color::new(0.85, 0.85, 0.85, 1.0))]),
        StyleSlot::Effect => serde_json::to_value(vec![Effect::drop_shadow()]),
        StyleSlot::Grid => serde_json::to_value(vec![LayoutGrid::default()]),
    };
    v.unwrap_or(Value::Null)
}

/// The Local styles header: its title (dim without styles) and the "+" that opens the type menu.
pub(crate) fn header(app: &mut FigApp, ui: &mut Ui, doc: &Document) {
    let t = Tokens::get(ui.ctx());
    let dim = figcraft_engine::library::local_styles(doc).next().is_none();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
    let title = tr("Local styles");
    ui.painter().text(
        pos2(rect.min.x + widgets::PAD, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        semibold(11.0),
        if dim { t.text_secondary } else { t.text },
    );
    let b = Rect::from_min_size(pos2(rect.max.x - 32.0, rect.center().y - 12.0), vec2(24.0, 24.0));
    let resp = ui.interact(b, ui.scope_id().with("local_styles_create"), Sense::click()).tip(tr("Create style"));
    if resp.hovered() {
        ui.painter().rect_filled(b, CornerRadius::same(5), t.hover);
    }
    icons::paint(ui.painter(), b.shrink(5.0), Icon::Plus, t.icon);
    let mut chosen = None;
    let mut group = None;
    menu_popup(&resp).width(160.0).show(|ui| {
        for (slot, label) in TYPES {
            if menu_item(ui, tr(label), "") {
                chosen = Some(slot);
                ui.close();
            }
        }
        ui.separator();
        // An empty group of a type, to drag styles into (style_groups_ui).
        ui.menu_button(tr("New group"), |ui| {
            for (slot, label) in TYPES {
                if menu_item(ui, tr(label), "") {
                    group = Some(slot);
                    ui.close();
                }
            }
        });
    });
    if let Some(slot) = chosen {
        crate::style_dialog_ui::open_create(app, ui.ctx(), slot, default_value(slot), b.min.y);
    }
    if let Some(slot) = group {
        crate::style_groups_ui::new_group(app, ui.ctx(), slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_doc::StyleKind;
    use figcraft_engine::Session;
    use serde_json::json;

    #[test]
    fn every_type_is_created_from_its_default_value() {
        let mut s = Session::new();
        for (slot, _) in TYPES {
            let r = s.execute(
                "style.create",
                &json!({"type": match slot {
                StyleSlot::Text => "text",
                StyleSlot::Effect => "effect",
                StyleSlot::Grid => "grid",
                _ => "paint",
            }, "name": "x", "value": default_value(slot)}),
            );
            assert!(r.is_ok(), "{slot:?}: {r:?}");
        }
        let doc = s.doc().unwrap();
        let kinds: Vec<&StyleKind> = doc.styles.iter().map(|st| &st.kind).collect();
        assert!(matches!(kinds[0], StyleKind::Text { style, .. } if style.font_family == "Inter" && style.font_size == 12.0));
        assert!(matches!(kinds[1], StyleKind::Paint { paints } if paints.len() == 1));
        assert!(matches!(kinds[2], StyleKind::Effect { effects } if effects.len() == 1));
        assert!(matches!(kinds[3], StyleKind::Grid { grids } if grids.len() == 1));
    }
}
