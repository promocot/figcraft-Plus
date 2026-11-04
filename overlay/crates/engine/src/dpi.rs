//! FigCraft Plus: a resolution (DPI / PPI) per top-level frame, for print.
//!
//! FigCraft counts a pixel as a point (72 per inch), as the web and its PDF export do. A frame made
//! for print wants more: at 300 DPI an A4 page is 2480 × 3508 px. Each top-level frame can say the
//! resolution it is made for — 72 by default, or what Preferences ▸ FigCraft Plus sets for new
//! frames:
//!
//! - **Changing it scales the frame and everything in it** by the ratio (72 → 300: × 4.17), from
//!   its top-left corner, so its physical size stays the same: 210 × 297 mm stays 210 × 297 mm.
//!   Layers beside it (to its right, below it) move by as much as it grew or shrank, so nothing
//!   overlaps and the gaps stay.
//! - Lengths in mm, cm, in and pt are read at the frame's resolution (the fields, the rulers, Dev
//!   Mode), and exports keep it: a PNG or JPG says its DPI, a PDF page has the frame's physical size.
//!
//! The file keeps plain px; the resolution is a line of the frame's data (`figcraftplus.dpi`), which
//! the official FigCraft keeps and ignores.

use std::cell::Cell;
use std::collections::BTreeSet;

use figcraft_doc::{Document, NodeId};
use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::params::get;
use crate::{Result, Session, bad, spec};

/// The layer data key holding a top-level frame's resolution (dots per inch).
pub const KEY: &str = "figcraftplus.dpi";

/// The web's resolution: a px is a point.
pub const WEB: f64 = 72.0;

/// The range a resolution may take.
pub const MIN: f64 = 1.0;
pub const MAX: f64 = 4800.0;

thread_local! {
    /// The resolution new top-level frames get (Preferences ▸ FigCraft Plus).
    static DEFAULT: Cell<f64> = const { Cell::new(WEB) };
}

/// The resolution new top-level frames get.
pub fn set_default(dpi: f64) {
    DEFAULT.with(|d| d.set(if dpi.is_finite() { dpi.clamp(MIN, MAX) } else { WEB }));
}

pub fn default() -> f64 {
    DEFAULT.with(Cell::get)
}

fn parse(text: &str) -> Option<f64> {
    text.trim().trim_end_matches("dpi").trim().parse::<f64>().ok().filter(|d| d.is_finite() && (MIN..=MAX).contains(d))
}

/// The top-level frame `id` is, or is in (None for a layer outside frames).
pub fn frame_of(doc: &Document, id: NodeId) -> Option<NodeId> {
    doc.top_level_frame(id)
}

/// A top-level frame's own resolution.
pub fn of_frame(doc: &Document, frame: NodeId) -> f64 {
    doc.node(frame).and_then(|n| n.plugin_data.get(KEY)).and_then(|t| parse(t)).unwrap_or(WEB)
}

/// The resolution layer `id` is made at (its top-level frame's; 72 outside frames).
pub fn of(doc: &Document, id: NodeId) -> f64 {
    frame_of(doc, id).map_or(WEB, |f| of_frame(doc, f))
}

/// Give new top-level frames the default resolution (called by the engine's normalisation).
pub fn resolve(doc: &mut Document, before: &Document, touched: &BTreeSet<NodeId>) {
    let d = default();
    if (d - WEB).abs() < 1e-9 {
        return;
    }
    let new: Vec<NodeId> = touched
        .iter()
        .copied()
        .filter(|id| !before.contains(*id) && doc.top_level_frame(*id) == Some(*id))
        .filter(|id| doc.node(*id).is_some_and(|n| !n.plugin_data.contains_key(KEY)))
        .collect();
    for id in new {
        if let Some(n) = doc.node_mut(id) {
            n.plugin_data.insert(KEY.into(), figcraft_doc::trim_num(d));
        }
    }
}

// ---- command ------------------------------------------------------------------------------------

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "frame.resolution",
        "Frame resolution (DPI)",
        "{ids? (top-level frames, or layers in them), dpi (1–4800; 72 is the web's)} — the frame and its contents scale by new / old from its top-left corner, so its physical size stays; the layers to its right and below move by as much as it grew, so nothing overlaps",
        true,
        set
    )]
}

fn overlaps(a0: f64, a1: f64, b0: f64, b1: f64) -> bool {
    a0 < b1 - 0.5 && b0 < a1 - 0.5
}

/// Set `frame`'s resolution to `dpi`, scaling it (and moving what is beside it) as described above.
pub fn set_frame(doc: &mut Document, frame: NodeId, dpi: f64) -> Result<()> {
    let old = of_frame(doc, frame);
    let k = dpi / old;
    if (k - 1.0).abs() > 1e-9 {
        let ob = doc.world_bounds(frame).ok_or_else(|| bad("frame.resolution", "the frame has no size"))?;
        crate::cmd::arrange::scale_node(doc, frame, k)?;
        if let Some(after) = doc.world_bounds(frame) {
            crate::cmd::arrange::move_world(doc, frame, ob.x0 - after.x0, ob.y0 - after.y0)?;
        }
        let nb = doc.world_bounds(frame).unwrap_or(ob);
        let (dx, dy) = (nb.x1 - ob.x1, nb.y1 - ob.y1);
        // What sits beside it in its container (the page, a section) moves along.
        let parent = doc.get(frame)?.parent;
        let siblings: Vec<NodeId> = parent.and_then(|p| doc.node(p)).map(|p| p.children.clone()).unwrap_or_default();
        let (ys0, ys1) = (ob.y0.min(nb.y0), ob.y1.max(nb.y1));
        let (xs0, xs1) = (ob.x0.min(nb.x0), ob.x1.max(nb.x1));
        let mut moves = Vec::new();
        for s in siblings.into_iter().filter(|s| *s != frame) {
            let Some(sb) = doc.world_bounds(s) else { continue };
            if sb.x0 >= ob.x1 - 0.5 && overlaps(sb.y0, sb.y1, ys0, ys1) {
                moves.push((s, dx, 0.0));
            } else if sb.y0 >= ob.y1 - 0.5 && overlaps(sb.x0, sb.x1, xs0, xs1) {
                moves.push((s, 0.0, dy));
            }
        }
        for (s, mx, my) in moves {
            crate::cmd::arrange::move_world(doc, s, mx, my)?;
            doc.mark_dirty([s]);
        }
    }
    let n = doc.get_mut(frame)?;
    if (dpi - WEB).abs() < 1e-9 {
        n.plugin_data.remove(KEY);
    } else {
        n.plugin_data.insert(KEY.into(), figcraft_doc::trim_num(dpi));
    }
    doc.mark_dirty([frame]);
    Ok(())
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "frame.resolution";
    let dpi = match get(p, "dpi") {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(t)) => parse(t),
        _ => None,
    }
    .filter(|d| d.is_finite() && (MIN..=MAX).contains(d))
    .ok_or_else(|| bad(cmd, "dpi: a number from 1 to 4800"))?;
    let ids = crate::params::req_targets(cmd, s, p)?;
    let doc = s.doc_mut()?;
    let mut frames: Vec<NodeId> = Vec::new();
    for id in ids {
        let f = frame_of(doc, id).ok_or_else(|| bad(cmd, "a top-level frame, or a layer in one"))?;
        if !frames.contains(&f) {
            frames.push(f);
        }
    }
    // Left to right, top to bottom: each one's neighbours move before the next one grows.
    frames.sort_by(|a, b| {
        let (ba, bb) = (doc.world_bounds(*a).unwrap_or_default(), doc.world_bounds(*b).unwrap_or_default());
        ba.x0.total_cmp(&bb.x0).then(ba.y0.total_cmp(&bb.y0))
    });
    for f in &frames {
        set_frame(doc, *f, dpi)?;
    }
    Ok(json!({"frames": frames.iter().map(|f| f.0).collect::<Vec<_>>(), "dpi": dpi}))
}

// ---- exports --------------------------------------------------------------------------------------

/// A PNG's bytes with a `pHYs` chunk saying `dpi` (in place of one it had).
pub fn png_with_dpi(png: &[u8], dpi: f64) -> Vec<u8> {
    const SIG: usize = 8;
    if png.len() < SIG + 25 || &png[..SIG] != b"\x89PNG\r\n\x1a\n" || !dpi.is_finite() || dpi <= 0.0 {
        return png.to_vec();
    }
    let ppm = (dpi / 0.0254).round().clamp(1.0, f64::from(u32::MAX)) as u32;
    let mut data = Vec::with_capacity(9);
    data.extend(ppm.to_be_bytes());
    data.extend(ppm.to_be_bytes());
    data.push(1); // the unit is the metre
    let mut chunk = Vec::with_capacity(21);
    chunk.extend((data.len() as u32).to_be_bytes());
    let mut typed = b"pHYs".to_vec();
    typed.extend(&data);
    chunk.extend(&typed);
    chunk.extend(crc32(&typed).to_be_bytes());
    let mut out = Vec::with_capacity(png.len() + chunk.len());
    out.extend(&png[..SIG]);
    let mut at = SIG;
    let mut placed = false;
    while at + 8 <= png.len() {
        let len = u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]) as usize;
        let end = (at + 12 + len).min(png.len());
        let kind = &png[at + 4..at + 8];
        if kind != b"pHYs" {
            out.extend(&png[at..end]);
        }
        if kind == b"IHDR" && !placed {
            out.extend(&chunk);
            placed = true;
        }
        at = end;
    }
    out
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for b in bytes {
        c ^= u32::from(*b);
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// A JPEG's bytes with its JFIF density set to `dpi` (when it has a JFIF header).
pub fn jpeg_with_dpi(jpeg: &[u8], dpi: f64) -> Vec<u8> {
    let mut out = jpeg.to_vec();
    // SOI, then APP0 "JFIF\0": version (2), units (1), X density (2), Y density (2).
    if out.len() > 20 && out[0] == 0xFF && out[1] == 0xD8 && out[2] == 0xFF && out[3] == 0xE0 && &out[6..11] == b"JFIF\0" && dpi.is_finite() && dpi > 0.0 {
        let d = (dpi.round().clamp(1.0, 65_535.0) as u16).to_be_bytes();
        out[13] = 1;
        out[14] = d[0];
        out[15] = d[1];
        out[16] = d[0];
        out[17] = d[1];
    }
    out
}

/// An exported PNG's or JPG's `bytes`, saying the resolution of the frame `ids` are in (at
/// `scale`); other formats, and layers of no resolution of their own, as they are.
pub fn with_export_dpi(doc: &Document, ids: &[NodeId], format: figcraft_doc::ExportFormat, scale: f64, bytes: Vec<u8>) -> Vec<u8> {
    let Some(d) = export_dpi(doc, ids, scale) else { return bytes };
    match format {
        figcraft_doc::ExportFormat::Png => png_with_dpi(&bytes, d),
        figcraft_doc::ExportFormat::Jpg => jpeg_with_dpi(&bytes, d),
        _ => bytes,
    }
}

/// The resolution an export of `ids` at `scale` carries: their frame's, times the scale (None
/// when they aren't in one frame of their own resolution).
pub fn export_dpi(doc: &Document, ids: &[NodeId], scale: f64) -> Option<f64> {
    let mut frames = ids.iter().map(|i| frame_of(doc, *i));
    let first = frames.next()??;
    if !frames.all(|f| f == Some(first)) {
        return None;
    }
    let d = of_frame(doc, first);
    doc.node(first)?.plugin_data.contains_key(KEY).then_some(d * if scale.is_finite() && scale > 0.0 { scale } else { 1.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(v: Value) -> NodeId {
        NodeId(v["id"].as_u64().unwrap())
    }

    fn b(s: &Session, n: NodeId) -> (f64, f64, f64, f64) {
        let r = s.doc().unwrap().world_bounds(n).unwrap();
        let q = |v: f64| (v * 100.0).round() / 100.0;
        (q(r.x0), q(r.y0), q(r.width()), q(r.height()))
    }

    #[test]
    fn a_frame_scales_to_its_resolution_and_its_neighbours_move_aside() {
        let mut s = Session::new();
        // An A4 page at 72 (595.28 × 841.89), a frame right of it, one below it, one far away.
        let a4 = id(s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 595.28, "height": 841.89})).unwrap());
        let t = id(s.execute("text.create", &json!({"text": "Title", "x": 56.69, "y": 56.69, "parent": a4, "props": {"fontSize": 24}})).unwrap());
        let right = id(s.execute("frame.create", &json!({"x": 700, "y": 100, "width": 200, "height": 200})).unwrap());
        let below = id(s.execute("frame.create", &json!({"x": 100, "y": 1000, "width": 200, "height": 200})).unwrap());
        let above = id(s.execute("frame.create", &json!({"x": 0, "y": -500, "width": 200, "height": 200})).unwrap());
        s.execute("frame.resolution", &json!({"ids": [a4], "dpi": 300})).unwrap();
        let k = 300.0 / 72.0;
        assert_eq!(b(&s, a4), (0.0, 0.0, 2480.33, 3507.88), "A4 at 300 DPI, from its top-left corner");
        assert_eq!(of(s.doc().unwrap(), t), 300.0, "its layers are at its resolution");
        let fs = s.doc().unwrap().node(t).unwrap().kind.text().unwrap().style.font_size;
        assert!((fs - 24.0 * k).abs() < 0.01, "its contents scale too: {fs}");
        let near = |a: f64, b: f64| (a - b).abs() < 0.02;
        assert!(near(b(&s, right).0, 700.0 + 2480.33 - 595.28), "the frame to its right moved by as much as it grew: {:?}", b(&s, right));
        assert!(near(b(&s, below).1, 1000.0 + 3507.88 - 841.89), "the one below too: {:?}", b(&s, below));
        assert_eq!(b(&s, above), (0.0, -500.0, 200.0, 200.0), "what isn't beside it stays");
        // Back to 72: everything returns.
        s.execute("frame.resolution", &json!({"ids": [t], "dpi": 72})).unwrap();
        assert_eq!(b(&s, a4), (0.0, 0.0, 595.28, 841.89));
        assert_eq!(b(&s, right).0, 700.0);
        assert!(!s.doc().unwrap().node(a4).unwrap().plugin_data.contains_key(KEY), "72 needs no line");
        s.undo().unwrap();
        assert_eq!(b(&s, a4).2, 2480.33, "one undo step");
    }

    #[test]
    fn new_frames_take_the_default_resolution() {
        let mut s = Session::new();
        set_default(300.0);
        let f = id(s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 100, "height": 100})).unwrap());
        let inner = id(s.execute("frame.create", &json!({"x": 10, "y": 10, "width": 20, "height": 20, "parent": f})).unwrap());
        set_default(WEB);
        let doc = s.doc().unwrap();
        assert_eq!(of_frame(doc, f), 300.0);
        assert!(!doc.node(inner).unwrap().plugin_data.contains_key(KEY), "only top-level frames");
        assert_eq!(of(doc, inner), 300.0);
    }

    #[test]
    fn a_pdf_page_has_the_frames_physical_size() {
        let mut s = Session::new();
        let a4 = id(s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 595.28, "height": 841.89, "props": {"fills": ["#FFFFFF"]}})).unwrap());
        s.execute("frame.resolution", &json!({"ids": [a4], "dpi": 300})).unwrap();
        let r = s.execute("file.export", &json!({"ids": [a4], "format": "pdf"})).unwrap();
        use base64::Engine as _;
        let pdf = base64::engine::general_purpose::STANDARD.decode(r["data"].as_str().unwrap()).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        let at = text.find("/MediaBox").expect("a media box");
        let nums: Vec<f64> = text[at + 9..].trim_start().trim_start_matches('[').split(']').next().unwrap().split_whitespace().filter_map(|n| n.parse().ok()).collect();
        assert!((nums[2] - 595.28).abs() < 0.1 && (nums[3] - 841.89).abs() < 0.1, "A4 in points, not 2480 px: {nums:?}");
        let png = s.execute("file.export", &json!({"ids": [a4], "format": "png"})).unwrap();
        let png = base64::engine::general_purpose::STANDARD.decode(png["data"].as_str().unwrap()).unwrap();
        assert!(png.windows(4).any(|w| w == b"pHYs"), "the PNG says 300 DPI");
    }

    #[test]
    fn exports_say_their_resolution() {
        // A 1×1 PNG.
        let png = base64_png();
        let out = png_with_dpi(&png, 300.0);
        let at = out.windows(4).position(|w| w == b"pHYs").expect("a pHYs chunk");
        let ppm = u32::from_be_bytes([out[at + 4], out[at + 5], out[at + 6], out[at + 7]]);
        assert_eq!(ppm, 11811, "300 DPI in pixels per metre");
        assert_eq!(out.len(), png.len() + 21);
        assert_eq!(png_with_dpi(&out, 300.0).len(), out.len(), "an old pHYs is replaced");
    }

    fn base64_png() -> Vec<u8> {
        // 1×1 transparent PNG.
        let hex = "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c636000000000050001a5f645400000000049454e44ae426082";
        (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect()
    }
}
