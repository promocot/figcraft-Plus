//! FigCraft Plus: PDF/X-4 for print.
//!
//! FigCraft's print PDF has bleed, marks, page boxes and CMYK colour, but no output intent: the
//! printing condition its CMYK values are meant for. A file can ask for **PDF/X-4** instead: every
//! CMYK PDF it exports then names the coated FOGRA39 condition (ISO 12647-2, the usual one for
//! sheet-fed offset in Europe) with its ICC profile, is converted to CMYK through that profile
//! rather than by formula, blends its transparency in CMYK, and says it is PDF/X-4 (untrapped) —
//! what a print shop's preflight looks for (`figcraft_pdf::plus`).
//!
//! The choice is a line of the file's first page data (`figcraftplus.pdfx`), which the official
//! FigCraft keeps and ignores.

use figcraft_doc::Document;
use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::params::get;
use crate::{Result, Session, bad, spec};

/// The page data key (on the file's first page) asking for PDF/X.
pub const KEY: &str = "figcraftplus.pdfx";

/// The PDF/X version written.
pub const VERSION: &str = "PDF/X-4";

/// Whether the file's CMYK PDFs are PDF/X-4.
pub fn on(doc: &Document) -> bool {
    doc.pages.first().and_then(|p| doc.node(*p)).and_then(|n| n.plugin_data.get(KEY)).is_some_and(|v| v.trim().eq_ignore_ascii_case(VERSION))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "print.pdfx",
        "PDF/X-4 for print",
        "{on: bool} — the file's CMYK PDF exports are PDF/X-4 for coated FOGRA39 paper: an output intent with its ICC profile, colours converted through it, transparency blended in CMYK",
        true,
        set
    )]
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "print.pdfx";
    let on = get(p, "on").and_then(Value::as_bool).ok_or_else(|| bad(cmd, "on: true or false"))?;
    let doc = s.doc_mut()?;
    let page = *doc.pages.first().ok_or_else(|| bad(cmd, "the file has no page"))?;
    let n = doc.get_mut(page)?;
    if on {
        n.plugin_data.insert(KEY.into(), VERSION.into());
    } else {
        n.plugin_data.remove(KEY);
    }
    doc.mark_dirty([page]);
    Ok(json!({"pdfx": on}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cmyk_pdf_of_a_pdfx_file_is_pdfx_4() {
        let mut s = Session::new();
        let f = s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 595.28, "height": 841.89, "props": {"fills": ["#E8ECF8"]}})).unwrap()["id"].clone();
        s.execute("shape.rectangle", &json!({"x": 40, "y": 40, "width": 200, "height": 100, "parent": f, "props": {"fills": ["#D7263D"], "opacity": 0.8}})).unwrap();
        s.execute("print.pdfx", &json!({"on": true})).unwrap();
        assert!(on(s.doc().unwrap()));
        let r = s.execute("file.export", &json!({"ids": [f], "format": "pdf", "cmyk": true, "bleed": 8.5, "cropMarks": true})).unwrap();
        let data = r["data"].as_str().expect("the PDF as base64");
        use base64::Engine as _;
        let pdf = base64::engine::general_purpose::STANDARD.decode(data).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/GTS_PDFX"), "an output intent");
        assert!(text.contains("FOGRA39"), "for FOGRA39");
        assert!(text.contains("/DestOutputProfile"), "with its profile");
        assert!(text.contains("PDF/X-4"), "says it is PDF/X-4");
        assert!(text.contains("/TrimBox"), "every page has a trim box");
        assert!(!text.contains("/DeviceRGB"), "no RGB left");
        // Without CMYK it stays FigCraft's own PDF.
        let r = s.execute("file.export", &json!({"ids": [f], "format": "pdf"})).unwrap();
        let pdf = base64::engine::general_purpose::STANDARD.decode(r["data"].as_str().unwrap()).unwrap();
        assert!(!String::from_utf8_lossy(&pdf).contains("/GTS_PDFX"));
    }
}
