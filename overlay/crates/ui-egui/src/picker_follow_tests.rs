//! FigCraft Plus: the colour picker follows the selection (Preferences ▸ FigCraft Plus ▸ *Picker
//! follows the selection*), or closes on another selection as FigCraft's does when it is off.

use egui::Pos2;
use figcraft_doc::NodeId;
use figcraft_engine::Session;
use serde_json::json;

use crate::colorpicker::PickerTarget;
use crate::layers_drag_tests::Ui;

fn rect(s: &mut Session, x: f64) -> NodeId {
    let v = s.execute("shape.rectangle", &json!({"x": x, "y": 0, "width": 40, "height": 40, "props": {"fill": "#3366FF"}})).unwrap();
    NodeId(v["id"].as_u64().unwrap())
}

/// Two rectangles, the first selected, its fill in the picker.
fn two() -> (Ui, NodeId, NodeId) {
    let mut s = Session::new();
    let a = rect(&mut s, 0.0);
    let b = rect(&mut s, 120.0);
    s.execute("select.set", &json!({"ids": [a]})).unwrap();
    let mut u = Ui::new(s);
    u.app.ui.picker = Some(PickerTarget::Paint { fills: true, index: 0, ids: vec![a] });
    u.frame(vec![]);
    u.frame(vec![]);
    (u, a, b)
}

/// The screen point of page point (x, y).
fn at(u: &Ui, x: f64, y: f64) -> Pos2 {
    u.app.to_screen(x, y)
}

fn click(u: &mut Ui, p: Pos2) {
    u.frame(vec![egui::Event::PointerMoved(p)]);
    u.button(p, true);
    u.button(p, false);
    u.frame(vec![]);
    u.frame(vec![]);
}

#[test]
fn clicking_another_layer_moves_the_picker_to_it() {
    crate::picker_place_ui::set_follows_for_test(Some(true));
    let (mut u, a, b) = two();
    let p = at(&u, 140.0, 20.0);
    click(&mut u, p);
    assert_eq!(u.app.session.selection(), vec![b]);
    assert_eq!(u.app.ui.picker, Some(PickerTarget::Paint { fills: true, index: 0, ids: vec![b] }), "Fill 1 of the new layer");
    // In Layers too.
    let row = u.row_at(a);
    click(&mut u, row);
    assert_eq!(u.app.session.selection(), vec![a]);
    assert_eq!(u.app.ui.picker, Some(PickerTarget::Paint { fills: true, index: 0, ids: vec![a] }), "Fill 1 of the layer picked in Layers");
    // A click on empty canvas closes it.
    let p = at(&u, -200.0, 300.0);
    click(&mut u, p);
    assert!(u.app.ui.picker.is_none(), "{:?}", u.app.ui.picker);
    crate::picker_place_ui::set_follows_for_test(None);
}

#[test]
fn turned_off_the_picker_closes_on_another_layer() {
    crate::picker_place_ui::set_follows_for_test(Some(false));
    let (mut u, _, b) = two();
    let p = at(&u, 140.0, 20.0);
    click(&mut u, p);
    assert_eq!(u.app.session.selection(), vec![b]);
    assert!(u.app.ui.picker.is_none(), "as in FigCraft: {:?}", u.app.ui.picker);
    crate::picker_place_ui::set_follows_for_test(None);
}

#[test]
fn a_style_dialogs_swatch_opens_the_picker_beside_the_dialog() {
    let mut s = Session::new();
    let id = s.execute("style.create", &json!({"type": "paint", "name": "Accent", "value": "#3366FF"})).unwrap()["id"].as_str().unwrap().to_string();
    let mut u = Ui::new(s);
    let ctx = u.ctx.clone();
    crate::style_dialog_ui::open_edit(&mut u.app, &ctx, &id, 200.0);
    for _ in 0..3 {
        u.frame(vec![]);
    }
    let dialog = ctx.memory(|m| m.area_rect(egui::Id::unique("figcraftplus_style_dialog_area"))).expect("the dialog is open");
    // The paint's swatch, at the left of its row in Properties: click up the dialog's left side
    // until one opens the picker.
    let mut y = dialog.max.y - 8.0;
    while u.app.ui.picker.is_none() && y > dialog.center().y {
        click(&mut u, egui::pos2(dialog.min.x + 29.0, y));
        y -= 4.0;
    }
    assert_eq!(u.app.ui.picker, Some(PickerTarget::Style { id, index: 0 }));
    let picker = ctx.memory(|m| m.area_rect(crate::colorpicker::window_id())).expect("the picker is open");
    assert!(picker.max.x <= dialog.min.x + 1.0, "beside the dialog, not over it: {picker:?} {dialog:?}");
}
