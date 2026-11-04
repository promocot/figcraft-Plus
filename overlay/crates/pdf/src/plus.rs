//! FigCraft Plus in the PDF export.
//!
//! - **A frame's resolution** (`figcraftplus.dpi`, set in the Design panel): a frame made at 300
//!   DPI is 2480 px wide for A4, and its page is still 210 mm wide — the page is scaled by
//!   72 / DPI (a PDF point is 1/72 in).
//! - **PDF/X-4** (`figcraftplus.pdfx` on the file's first page, a CMYK export): an output intent for
//!   coated FOGRA39 paper with its ICC profile, CMYK converted through that profile (relative
//!   colorimetric, perceptual for images) instead of by formula, transparency blended in CMYK, a
//!   trim box on every page, `/GTS_PDFXVersion (PDF/X-4)` and `/Trapped /False`.
//!
//! The profile is craft-cms's `craft-coated-fogra39.icc` (MIT OR Apache-2.0, storytold/craft-libs),
//! built from the FOGRA39 characterisation data.

use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

use figcraft_doc::{Document, NodeId};

/// The layer data key holding a top-level frame's resolution (DPI).
pub const DPI_KEY: &str = "figcraftplus.dpi";

/// The coated FOGRA39 profile the output intent names.
pub const FOGRA39_ICC: &[u8] = include_bytes!("craft-coated-fogra39.icc");

/// What the output intent says.
const CONDITION: &str = "FOGRA39";
const CONDITION_INFO: &str = "Coated FOGRA39 (ISO 12647-2:2004)";
const REGISTRY: &str = "http://www.color.org";

thread_local! {
    /// While a PDF/X export runs.
    static ACTIVE: RefCell<bool> = const { RefCell::new(false) };
}

/// sRGB → FOGRA39, made once.
fn transform() -> Option<&'static Arc<moxcms::Transform8BitExecutor>> {
    static T: OnceLock<Option<Arc<moxcms::Transform8BitExecutor>>> = OnceLock::new();
    T.get_or_init(|| {
        let dst = moxcms::ColorProfile::new_from_slice(FOGRA39_ICC).ok()?;
        let src = moxcms::ColorProfile::new_srgb();
        let options = moxcms::TransformOptions { rendering_intent: moxcms::RenderingIntent::RelativeColorimetric, ..Default::default() };
        src.create_transform_8bit(moxcms::Layout::Rgb, &dst, moxcms::Layout::Rgba, options).ok()
    })
    .as_ref()
}

/// Run `f` (an export) as PDF/X-4 when `on`.
pub fn with_pdfx<R>(on: bool, f: impl FnOnce() -> R) -> R {
    let on = on && transform().is_some();
    let was = ACTIVE.with(|a| a.replace(on));
    let out = f();
    ACTIVE.with(|a| *a.borrow_mut() = was);
    out
}

/// Whether the export running is PDF/X.
pub fn pdfx_on() -> bool {
    ACTIVE.with(|a| *a.borrow())
}

/// A colour (sRGB, 0..1) in FOGRA39 CMYK (0..1) while a PDF/X export runs.
pub fn cmyk(r: f32, g: f32, b: f32) -> Option<[f32; 4]> {
    if !pdfx_on() {
        return None;
    }
    let t = transform()?;
    let byte = |v: f32| if v.is_finite() { (v.clamp(0.0, 1.0) * 255.0).round() as u8 } else { 0 };
    let src = [byte(r), byte(g), byte(b)];
    let mut dst = [0u8; 4];
    t.transform(&src, &mut dst).ok()?;
    Some(dst.map(|v| f32::from(v) / 255.0))
}

/// The krilla document for this export: PDF/X-4's settings while one runs.
pub fn document() -> krilla::Document {
    if !pdfx_on() {
        return krilla::Document::new();
    }
    let Some(profile) = krilla::icc::ICCProfile::<4>::new(FOGRA39_ICC) else { return krilla::Document::new() };
    let settings = krilla::SerializeSettings {
        xmp_metadata: true,
        group_color_space: krilla::GroupColorSpace::Cmyk,
        output_intent: Some(krilla::OutputIntent {
            profile,
            condition_identifier: CONDITION.into(),
            condition: Some(CONDITION_INFO.into()),
            registry_name: Some(REGISTRY.into()),
            info: Some(CONDITION_INFO.into()),
        }),
        trapped: Some(krilla::Trapped::False),
        pdfx_version: Some("PDF/X-4".into()),
        ..Default::default()
    };
    krilla::Document::new_with(settings)
}

/// How much a page of `ids` is scaled: 72 / DPI for one top-level frame made at its own resolution,
/// 1 otherwise.
pub fn page_scale(doc: &Document, ids: &[NodeId]) -> f64 {
    let [one] = ids else { return 1.0 };
    let dpi = doc
        .node(*one)
        .filter(|_| doc.top_level_frame(*one) == Some(*one))
        .and_then(|n| n.plugin_data.get(DPI_KEY))
        .and_then(|t| t.trim().parse::<f64>().ok())
        .filter(|d| d.is_finite() && (1.0..=4800.0).contains(d));
    dpi.map_or(1.0, |d| 72.0 / d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_go_through_the_fogra39_profile() {
        assert!(transform().is_some(), "the profile loads");
        assert_eq!(cmyk(1.0, 0.0, 0.0), None, "only during a PDF/X export");
        with_pdfx(true, || {
            let white = cmyk(1.0, 1.0, 1.0).unwrap();
            assert!(white.iter().all(|v| *v < 0.02), "paper white: {white:?}");
            let black = cmyk(0.0, 0.0, 0.0).unwrap();
            assert!(black[3] > 0.7 && black[0] > 0.3, "a rich black, as the profile builds it: {black:?}");
            let red = cmyk(1.0, 0.0, 0.0).unwrap();
            assert!(red[1] > 0.8 && red[2] > 0.8 && red[0] < 0.1, "red is magenta and yellow: {red:?}");
        });
    }
}
