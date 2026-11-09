//! FigCraft Plus: align to a key object.
//!
//! With several layers selected, **Alt+click** (not a drag) on one of them makes it the **key
//! object**: the selection stays, and the align commands then move the others to it while it stays
//! put. Alt+click on the key again lets it go. A plain click selects just the layer, as in FigCraft. The key is
//! the canvas's (one per app, like the selection it belongs to): it lapses when it leaves the
//! selection. Preferences ▸ FigCraft Plus can turn the click off (`set_enabled`).

use std::cell::Cell;

use figcraft_doc::NodeId;

thread_local! {
    static KEY: Cell<Option<NodeId>> = const { Cell::new(None) };
    static ENABLED: Cell<bool> = const { Cell::new(true) };
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

/// A click on `clicked` in the multi-selection `selection` (`alt`: with Alt held): true when the
/// selection stays — Alt made `clicked` the key, or let the key go; false when the click selects
/// just it (a plain click, or key objects off).
pub fn click(selection: &[NodeId], clicked: NodeId, alt: bool) -> bool {
    if !alt || !enabled() || !selection.contains(&clicked) {
        clear();
        return false;
    }
    if key_in(selection) == Some(clicked) {
        clear();
    } else {
        KEY.with(|k| k.set(Some(clicked)));
    }
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
        assert!(click(&nodes, nodes[1], true), "Alt+click picks the key");
        s.execute("align.top", &json!({"ids": ids})).unwrap();
        let y = |s: &Session, i: usize| s.doc().unwrap().world_bounds(nodes[i]).unwrap().y0;
        assert_eq!((y(&s, 0), y(&s, 1), y(&s, 2)), (50.0, 50.0, 50.0), "to the key's top, not the selection's");
        assert!(click(&nodes, nodes[1], true), "Alt+click on the key lets it go, the selection stays");
        assert_eq!(key_in(&nodes), None);
        assert!(click(&nodes, nodes[2], true));
        assert!(!click(&nodes, nodes[0], false), "a plain click selects just the layer, as in FigCraft");
        assert_eq!(key_in(&nodes), None);
        clear();
    }
}
