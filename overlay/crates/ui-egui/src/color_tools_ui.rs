//! FigCraft Plus: more in the colour picker for print and accessibility.
//!
//! - **CMYK** in the model menu: C, M, Y, K in percent, converted the way FigCraft's CMYK PDF
//!   export converts (`K = 1 − max(R, G, B)`), so the numbers are the ones the PDF will hold.
//! - **Copy as**: the colour as Hex, RGB, HSL, HSB, LCH, OKLCH, CMYK or CSS, to the clipboard — a
//!   converter between colour spaces.
//! - **Contrast both ways** under the fields: the colour on what it sits on (the layers painted
//!   under the layer, else the page), and that background's colour on the colour — an `Aa` sample
//!   of each with the WCAG 2 ratio and level and the APCA lightness contrast (Lc). WCAG gives the
//!   same ratio both ways, APCA doesn't (light on dark reads differently from dark on light), so
//!   the pairing that reads better is marked.

use egui::{Id, Ui};
use figcraft_color::oklch_profile::Oklch;
use figcraft_color::{Color, ColorProfile, Hsb, Hsl, Lch, PaintKind};
use figcraft_doc::{Document, NodeId};

use crate::FigApp;
use crate::colorpicker::PickerTarget;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::widgets::{self, FieldEdit, Lead};

// ---- CMYK -------------------------------------------------------------------------------------------

/// sRGB → CMYK (0..1 each), the formula FigCraft's CMYK PDF export uses.
pub(crate) fn cmyk_of(c: Color) -> [f64; 4] {
    let unit = |v: f32| if v.is_finite() { f64::from(v.clamp(0.0, 1.0)) } else { 0.0 };
    let (r, g, b) = (unit(c.r), unit(c.g), unit(c.b));
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 - 1e-6 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let ink = |v: f64| ((1.0 - v - k) / (1.0 - k)).clamp(0.0, 1.0);
    [ink(r), ink(g), ink(b), k]
}

/// CMYK (0..1 each) → the colour, keeping alpha `a`.
pub(crate) fn from_cmyk([c, m, y, k]: [f64; 4], a: f32) -> Color {
    let ch = |v: f64| ((1.0 - v.clamp(0.0, 1.0)) * (1.0 - k.clamp(0.0, 1.0))) as f32;
    Color::new(ch(c), ch(m), ch(y), a)
}

/// The CMYK row: C, M, Y and K in percent. Returns the edited colour.
pub(crate) fn cmyk_row(ui: &mut Ui, c: Color) -> Option<Color> {
    let v = cmyk_of(c).map(|x| (x * 100.0).round());
    let mut out = None;
    for (k, name) in ["C", "M", "Y", "K"].into_iter().enumerate() {
        let id = Id::unique(("cp_cmyk", k));
        let label = format!("CMYK {name}");
        match widgets::number_field(ui, id, Lead::Named(&label), Some(v[k]), "", CMYK_W, 1.0) {
            Some(FieldEdit::Set(x) | FieldEdit::Scrub(x, _)) => {
                let mut nv = v;
                nv[k] = x.clamp(0.0, 100.0);
                out = Some(from_cmyk(nv.map(|p| p / 100.0), c.a));
            }
            None => {}
        }
    }
    out
}

/// A CMYK field's width.
pub(crate) const CMYK_W: f32 = 40.0;

// ---- Copy as -------------------------------------------------------------------------------------------

fn n(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if s == "-0" { "0".into() } else { s }
}

/// `c` written in every format "Copy as" offers: (label, text).
pub(crate) fn formats(c: Color, profile: ColorProfile) -> Vec<(&'static str, String)> {
    let a = f64::from(c.a);
    let alpha = |s: String| if a < 0.999 { format!("{s} / {}%", n(a * 100.0, 1)) } else { s };
    let [r, g, b, _] = c.to_rgba8();
    let hsl = Hsl::from_color(c);
    let hsb = Hsb::from_color(c);
    let lch = Lch::from_color(c);
    let ok = Oklch::from_color(c, profile);
    let [cc, m, y, k] = cmyk_of(c).map(|x| x * 100.0);
    vec![
        ("Hex", c.to_css_hex()),
        ("RGB", format!("{})", alpha(format!("rgb({r} {g} {b}")))),
        ("HSL", format!("{})", alpha(format!("hsl({} {}% {}%", n(f64::from(hsl.h), 1), n(f64::from(hsl.s) * 100.0, 1), n(f64::from(hsl.l) * 100.0, 1))))),
        ("HSB", format!("hsb({} {}% {}%)", n(f64::from(hsb.h), 1), n(f64::from(hsb.s) * 100.0, 1), n(f64::from(hsb.b) * 100.0, 1))),
        ("LCH", format!("{})", alpha(format!("lch({}% {} {}", n(f64::from(lch.l), 2), n(f64::from(lch.c), 2), n(f64::from(lch.h), 2))))),
        ("OKLCH", format!("{})", alpha(format!("oklch({} {} {}", n(ok.l, 4), n(ok.c, 4), n(ok.h, 2))))),
        ("CMYK", format!("device-cmyk({}% {}% {}% {}%)", n(cc, 1), n(m, 1), n(y, 1), n(k, 1))),
        ("CSS", c.to_css()),
    ]
}

/// "Copy as": a button that lists the colour in every format; a click copies one.
fn copy_as(app: &mut FigApp, ui: &mut Ui, c: Color, profile: ColorProfile) {
    let resp = widgets::dropdown(ui, tr("Copy as"), 96.0, None);
    crate::right::menu_popup(&resp).width(260.0).show(|ui| {
        for (label, text) in formats(c, profile) {
            if crate::right::menu_item(ui, &text, label) {
                crate::egui_copy_text(ui.ctx(), &text);
                app.toast(format!("{} {text}", tr("Copied")));
                ui.close();
            }
        }
    });
}

// ---- contrast ------------------------------------------------------------------------------------------

/// APCA lightness contrast (Lc, APCA-W3 0.0.98G) of `text` on `bg`, both opaque sRGB: positive for
/// dark text on a light background, negative for light on dark.
pub(crate) fn apca(text: Color, bg: Color) -> f64 {
    let y = |c: Color| {
        let ch = |v: f32| f64::from(v.clamp(0.0, 1.0)).powf(2.4);
        let y = 0.2126729 * ch(c.r) + 0.7151522 * ch(c.g) + 0.0721750 * ch(c.b);
        if y < 0.022 { y + (0.022 - y).powf(1.414) } else { y }
    };
    let (yt, yb) = (y(text), y(bg));
    if (yb - yt).abs() < 0.0005 {
        return 0.0;
    }
    let out = if yb > yt {
        let s = (yb.powf(0.56) - yt.powf(0.57)) * 1.14;
        if s < 0.1 { 0.0 } else { s - 0.027 }
    } else {
        let s = (yb.powf(0.65) - yt.powf(0.62)) * 1.14;
        if s > -0.1 { 0.0 } else { s + 0.027 }
    };
    out * 100.0
}

/// What layer `id` is seen over, at its centre: the layers painted before it there (its frames,
/// and any layer under it — a rectangle behind a button counts, not only the frame it sits in),
/// top down to the first opaque fill, half-transparent ones blended over it; the page's
/// background under all. Locked and hidden-by-clipping layers are taken as they are seen.
fn backdrop(doc: &Document, id: Option<NodeId>) -> Color {
    let page_bg = |p: NodeId| match doc.node(p).map(|n| &n.kind) {
        Some(figcraft_doc::NodeKind::Page { background, .. }) => Some(background.with_alpha(1.0)),
        _ => None,
    };
    let current = doc.current_page.and_then(page_bg).unwrap_or(Color::WHITE);
    let Some(id) = id else { return current };
    let Some(page) = doc.ancestors(id).into_iter().find(|a| doc.node(*a).is_some_and(|n| n.is_page())) else { return current };
    let base = page_bg(page).unwrap_or(current);
    let Some(pt) = doc.world_bounds(id).map(|b| b.center()) else { return base };
    // Every layer seen at `pt`, back to front, up to `id`.
    fn walk(doc: &Document, n: NodeId, pt: figcraft_geom::Point, stop: NodeId, out: &mut Vec<NodeId>) -> bool {
        if n == stop {
            return true;
        }
        let Some(node) = doc.node(n) else { return false };
        if !node.visible {
            return false;
        }
        let inside = doc.world_bounds(n).is_some_and(|b| b.contains(pt));
        if inside {
            out.push(n);
        }
        // Children of a clipping frame show only inside it.
        if node.clip && node.kind.is_frame_like() && !inside && !doc.ancestors(stop).contains(&n) {
            return false;
        }
        node.paint_order().any(|c| walk(doc, c, pt, stop, out))
    }
    let mut under = Vec::new();
    if let Some(p) = doc.node(page) {
        let _ = p.children.iter().any(|c| walk(doc, *c, pt, id, &mut under));
    }
    // From the top: each layer's fills (top fill first), down to an opaque one.
    let mut layers: Vec<Color> = Vec::new();
    'down: for n in under.iter().rev() {
        let Some(node) = doc.node(*n) else { continue };
        if matches!(node.kind, figcraft_doc::NodeKind::Text(_)) {
            continue;
        }
        for p in node.fills.iter().rev().filter(|p| p.visible) {
            let c = match &p.kind {
                PaintKind::Solid { color } => *color,
                // A gradient: its stops' average.
                PaintKind::Gradient(g) if !g.stops.is_empty() => {
                    let k = g.stops.len() as f32;
                    let sum = g.stops.iter().fold([0.0f32; 4], |a, s| [a[0] + s.color.r, a[1] + s.color.g, a[2] + s.color.b, a[3] + s.color.a]);
                    Color::new(sum[0] / k, sum[1] / k, sum[2] / k, sum[3] / k)
                }
                _ => continue,
            };
            let a = (c.a * p.opacity * node.opacity).clamp(0.0, 1.0);
            layers.push(c.with_alpha(a));
            if a >= 0.999 {
                break 'down;
            }
        }
    }
    layers.iter().rev().fold(base, |bg, c| over(*c, bg))
}

/// `c` seen over `bg` (alpha composited).
fn over(c: Color, bg: Color) -> Color {
    let a = c.a.clamp(0.0, 1.0);
    Color::new(c.r * a + bg.r * (1.0 - a), c.g * a + bg.g * (1.0 - a), c.b * a + bg.b * (1.0 - a), 1.0)
}

/// The WCAG level a ratio reaches for normal text.
fn level(ratio: f32) -> &'static str {
    if ratio >= 7.0 {
        "AAA"
    } else if ratio >= 4.5 {
        "AA"
    } else if ratio >= 3.0 {
        "AA Large"
    } else {
        "Fail"
    }
}

/// The two ways to pair a colour `fg` (opaque, as seen) with its backdrop `bg`: `fg` on `bg`, and
/// `bg` on `fg`, each as (text, background, WCAG ratio, APCA Lc); and which one reads better (the
/// larger |Lc|; None when they are within 1 Lc of each other).
pub(crate) fn both_ways(fg: Color, bg: Color) -> ([(Color, Color, f32, f64); 2], Option<usize>) {
    let ratio = fg.contrast(bg);
    let ways = [(fg, bg, ratio, apca(fg, bg)), (bg, fg, ratio, apca(bg, fg))];
    let (a, b) = (ways[0].3.abs(), ways[1].3.abs());
    let better = if (a - b).abs() < 1.0 {
        None
    } else if a > b {
        Some(0)
    } else {
        Some(1)
    };
    (ways, better)
}

/// One pairing: an `Aa` sample (`text` on `bg`), the ratio and level, the Lc; `better` marks it.
fn pairing(ui: &mut Ui, w: f32, (text, bg, ratio, lc): (Color, Color, f32, f64), caption: &str, better: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(egui::vec2(w, 40.0), egui::Sense::hover());
    let p = ui.painter();
    let sample = egui::Rect::from_min_size(r.min, egui::vec2(40.0, 40.0));
    let to32 = |c: Color| crate::theme::doc_c32(ui.ctx(), c);
    p.rect_filled(sample, egui::CornerRadius::same(6), to32(bg));
    p.rect_stroke(
        sample,
        egui::CornerRadius::same(6),
        egui::Stroke::new(if better { 1.5 } else { 1.0 }, if better { t.accent } else { t.border }),
        egui::StrokeKind::Inside,
    );
    p.text(sample.center(), egui::Align2::CENTER_CENTER, "Aa", crate::theme::semibold(15.0), to32(text));
    let x = sample.max.x + 8.0;
    let clip = egui::Rect::from_min_max(egui::pos2(x, r.min.y), r.max);
    let p = p.with_clip_rect(clip);
    let col = if ratio >= 4.5 { t.text } else { t.warning };
    p.text(egui::pos2(x, r.min.y + 2.0), egui::Align2::LEFT_TOP, format!("{}:1 {}", n(f64::from(ratio), 2), level(ratio)), crate::theme::regular(10.5), col);
    p.text(egui::pos2(x, r.min.y + 15.0), egui::Align2::LEFT_TOP, format!("Lc {}", n(lc, 0)), crate::theme::regular(10.5), t.text_secondary);
    let mark = if better { format!("✓ {}", tr("Reads better")) } else { String::new() };
    p.text(egui::pos2(x, r.min.y + 28.0), egui::Align2::LEFT_TOP, mark, crate::theme::regular(9.5), t.accent_text);
    resp.on_hover_text(caption)
}

/// Under the picker's fields: "Copy as", and the contrast both ways with the colour's backdrop.
pub(crate) fn below_fields(app: &mut FigApp, ui: &mut Ui, c: Color, target: &PickerTarget) {
    let Ok(doc) = app.session.doc() else { return };
    let profile = doc.color_profile;
    let layer = match target {
        PickerTarget::Paint { ids, .. } | PickerTarget::Effect { ids, .. } => ids.first().copied(),
        PickerTarget::TextRange { id, .. } => Some(*id),
        _ => None,
    };
    let bg = backdrop(doc, layer);
    let fg = over(c, bg);
    let (ways, better) = both_ways(fg, bg);
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        copy_as(app, ui, c, profile);
    });
    let on_bg = format!("{} · {} {}", tr("This colour on its background"), tr("Contrast with"), bg.to_css_hex());
    let swapped = format!("{} · {} {}", tr("Its background on this colour"), tr("Contrast with"), fg.to_css_hex());
    // Side by side when both fit; one under the other in a narrow picker (Hex, CSS), so their
    // captions aren't cut.
    let full = ui.available_width() - 32.0;
    if full >= 2.0 * PAIRING_W + 8.0 {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.spacing_mut().item_spacing.x = 8.0;
            let w = (full - 8.0) / 2.0;
            pairing(ui, w, ways[0], &on_bg, better == Some(0));
            pairing(ui, w, ways[1], &swapped, better == Some(1));
        });
    } else {
        for (i, caption) in [on_bg, swapped].iter().enumerate() {
            if i > 0 {
                ui.add_space(6.0);
            }
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                pairing(ui, full.max(PAIRING_W), ways[i], caption, better == Some(i));
            });
        }
    }
}

/// The width one contrast sample needs: the swatch, and "4.50:1 AA Large" beside it.
const PAIRING_W: f32 = 132.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmyk_round_trips_and_matches_the_pdf_export() {
        let c = Color::from_hex("#EF3D88").unwrap();
        let k = cmyk_of(c);
        assert_eq!(k.map(|x| (x * 100.0).round()), [0.0, 74.0, 43.0, 6.0]);
        assert_eq!(from_cmyk(k, 1.0).to_hex(), "EF3D88");
        assert_eq!(cmyk_of(Color::BLACK), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(cmyk_of(Color::WHITE), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn contrast_wcag_and_apca() {
        let (b, w) = (Color::BLACK, Color::WHITE);
        assert!((b.contrast(w) - 21.0).abs() < 0.01);
        // APCA's published reference values: black on white 106.04, white on black -107.88.
        assert!((apca(b, w) - 106.04).abs() < 0.1, "{}", apca(b, w));
        assert!((apca(w, b) + 107.88).abs() < 0.1, "{}", apca(w, b));
        assert_eq!(level(4.6), "AA");
        let half = over(Color::new(0.0, 0.0, 0.0, 0.5), w);
        // Both ways: the same WCAG ratio, APCA prefers one.
        let (ways, better) = both_ways(Color::from_hex("#FFFFFF").unwrap(), Color::from_hex("#1E40AF").unwrap());
        assert_eq!(ways[0].2, ways[1].2);
        assert!(ways[0].3 < 0.0 && ways[1].3 > 0.0, "light on dark is negative, dark on light positive");
        assert_eq!(better, Some(0), "white on a dark blue reads better than the blue on white in APCA");
        assert_eq!(both_ways(Color::BLACK, Color::BLACK).1, None);
        assert_eq!(half.to_hex(), "808080");
    }

    #[test]
    fn copy_as_lists_every_format() {
        let f = formats(Color::from_hex("#3A81F6").unwrap(), ColorProfile::Srgb);
        let get = |l: &str| f.iter().find(|(k, _)| *k == l).map(|(_, v)| v.clone()).unwrap();
        assert_eq!(get("Hex"), "#3A81F6");
        assert_eq!(get("RGB"), "rgb(58 129 246)");
        assert!(get("OKLCH").starts_with("oklch(0.6"), "{}", get("OKLCH"));
        assert!(get("CMYK").starts_with("device-cmyk(76.4% 47.6% 0% 3.5%"), "{}", get("CMYK"));
        assert_eq!(f.len(), 8);
    }
}
