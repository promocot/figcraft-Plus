//! FigCraft Plus: align to a key object.
//!
//! With several layers selected, a click (not a drag) on one of them makes it the **key object**:
//! the selection stays, and the align commands then move the others to it while it stays put. A
//! second click on the key selects just it, as a click in a selection does in FigCraft. The key is
//! the canvas's (one per app, like the selection it belongs to): it lapses when it leaves the
//! selection. Preferences ▸ FigCraft Plus can turn the click off (`set_enabled`).

use std::cell::Cell;

use figcraft_doc::NodeId;

thread_local! {
    static KEY: Cell<Option<NodeId>> = const { Cell::new(None) };
    // Off in this crate's tests, which check FigCraft's own click rules; the key object's tests turn
    // it on.
    static ENABLED: Cell<bool> = const { Cell::new(!cfg!(test)) };
}

/// Whether a click in a selection picks the key object.
pub fn set_enabled(on: bool) {
    ENABLED.with(|e| e.set(on));
    if !on {
        clear();
    }
}

pub fn enabled() -> bool {
    ENABLED.with(Cell::get)
}

/// The key object, when it is one of `selection`.
pub fn key_in(selection: &[NodeId]) -> Option<NodeId> {
    let k = KEY.with(Cell::get)?;
    if selection.len() > 1 && selection.contains(&k) {
        Some(k)
    } else {
        None
    }
}

pub fn clear() {
    KEY.with(|k| k.set(None));
}

/// A click on `clicked` in the multi-selection `selection`: true when it made `clicked` the key
/// (the selection stays); false when the click should select just it (it was the key already, or
/// key objects are off).
pub fn click(selection: &[NodeId], clicked: NodeId) -> bool {
    if !enabled() || !selection.contains(&clicked) || key_in(selection) == Some(clicked) {
        clear();
        return false;
    }
    KEY.with(|k| k.set(Some(clicked)));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    use serde_json::json;

    #[test]
    fn the_others_align_to_the_key_which_stays_put() {
        let mut s = Session::new();
        let mut ids = Vec::new();
        for (x, y) in [(0, 0), (200, 50), (400, 120)] {
            ids.push(s.execute("shape.rectangle", &json!({"x": x, "y": y, "width": 50, "height": 50})).unwrap()["id"].as_u64().unwrap());
        }
        let nodes: Vec<NodeId> = ids.iter().map(|i| NodeId(*i)).collect();
        set_enabled(true);
        assert!(click(&nodes, nodes[1]), "the first click picks the key");
        s.execute("align.top", &json!({"ids": ids})).unwrap();
        let y = |s: &Session, i: usize| s.doc().unwrap().world_bounds(nodes[i]).unwrap().y0;
        assert_eq!((y(&s, 0), y(&s, 1), y(&s, 2)), (50.0, 50.0, 50.0), "to the key's top, not the selection's");
        assert!(!click(&nodes, nodes[1]), "a second click on the key selects just it");
        assert_eq!(key_in(&nodes), None);
        clear();
    }
}
