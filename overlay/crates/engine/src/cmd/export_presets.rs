//! FigCraft Plus: export presets — a set of sizes and formats with a file-name pattern, exported in
//! one go for every selected layer.
//!
//! `export.preset {ids?, items: [{size?, format?, …export setting}], pattern?, dir?, zip?}` exports
//! each layer once per item (an item is what `export.add` takes: `size` `1x` / `2x` / `512w` /
//! `1080h`, `format` png / jpg / svg / pdf / webp…) and names each file by `pattern`:
//!
//! ```text
//! {name}    the layer's name          {frame}  its top-level frame's name   {page}  its page's
//! {w} {h}   the file's size in px     {scale}  1x, 2x…                      {format} png, jpg…
//! {n}       1, 2, 3… (the layer's place in the selection)                   {date}  2026-11-04
//! ```
//!
//! The extension follows; a `/` makes folders (`..` never leaves the export folder); a name used
//! twice gets a number. Default pattern: `{name}_{w}x{h}`.

use std::collections::BTreeSet;

use base64::Engine as _;
use figcraft_doc::{Document, ExportSetting, NodeId};
use serde_json::{Value, json};

use super::CommandSpec;
use crate::params::{get, string};
use crate::{EngineError, Result, Session, bad, spec};

/// The pattern when none is given.
pub const DEFAULT_PATTERN: &str = "{name}_{w}x{h}";

/// The most files one preset export makes.
const MAX_FILES: usize = 2000;

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "export.preset",
        "Export with a preset",
        "{ids? (default the selection), items: [{size? (1x, 2x, 512w, 1080h…), format? (png, jpg, svg, pdf, webp…), …as export.add}], pattern? (default {name}_{w}x{h}; {name} {frame} {page} {w} {h} {scale} {format} {n} {date}; / makes folders), dir? (write the files there), zip? (one zip instead of base64 files)} → {files: [{name, id, bytes, path? | data?}], data? (zip)}",
        false,
        export
    )]
}

/// A path part made safe: no separators of its own, no `.` / `..`.
fn safe_part(s: &str) -> String {
    let t = super::file::safe_file_name(s.trim());
    if t == "." || t == ".." { "_".into() } else { t }
}

/// The file name for layer `id` (the `n`th) exported with `setting` at `scale` (`w` × `h` px).
#[allow(clippy::too_many_arguments)]
pub fn file_name(doc: &Document, id: NodeId, n: usize, setting: &ExportSetting, scale: f64, w: f64, h: f64, pattern: &str, date: &str) -> String {
    let name = doc.node(id).map(|x| x.name.clone()).unwrap_or_default();
    let frame = doc.top_level_frame(id).and_then(|f| doc.node(f)).map(|f| f.name.clone()).unwrap_or_else(|| name.clone());
    let page = doc.page_of(id).and_then(|p| doc.node(p)).map(|p| p.name.clone()).unwrap_or_default();
    let ext = setting.format.ext();
    let scale_label = format!("{}x", figcraft_doc::trim_num((scale * 100.0).round() / 100.0));
    let filled = pattern
        .replace("{name}", &name)
        .replace("{frame}", &frame)
        .replace("{page}", &page)
        .replace("{w}", &format!("{}", w.round() as i64))
        .replace("{h}", &format!("{}", h.round() as i64))
        .replace("{scale}", &scale_label)
        .replace("{format}", ext)
        .replace("{ext}", ext)
        .replace("{n}", &n.to_string())
        .replace("{date}", date);
    let parts: Vec<String> = filled.split('/').map(safe_part).filter(|p| !p.is_empty() && p != "_" || filled.split('/').count() == 1).collect();
    let mut path = parts.join("/");
    if path.trim().is_empty() {
        path = "export".into();
    }
    let dot_ext = format!(".{ext}");
    if !path.to_ascii_lowercase().ends_with(&dot_ext) {
        path.push_str(&dot_ext);
    }
    path
}

/// `name` made unique among `used` ("a.png", "a 2.png"…).
fn unique(name: String, used: &mut BTreeSet<String>) -> String {
    if used.insert(name.to_ascii_lowercase()) {
        return name;
    }
    let (stem, ext) = name.rsplit_once('.').map_or((name.clone(), String::new()), |(s, e)| (s.to_string(), format!(".{e}")));
    for k in 2.. {
        let n = format!("{stem} {k}{ext}");
        if used.insert(n.to_ascii_lowercase()) {
            return n;
        }
    }
    name
}

fn today() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
        let z = secs.div_euclid(86_400) + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        format!("{y:04}-{m:02}-{d:02}")
    }
    #[cfg(target_arch = "wasm32")]
    String::new()
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "export.preset";
    let ids = crate::params::req_targets(cmd, s, p)?;
    let items = match get(p, "items") {
        Some(Value::Array(a)) if !a.is_empty() => a.clone(),
        _ => return Err(bad(cmd, "items: [{size, format}], at least one")),
    };
    let settings: Vec<ExportSetting> = items.iter().map(|it| super::file::export_setting(cmd, ExportSetting::default(), it)).collect::<Result<_>>()?;
    let pattern = string(p, "pattern").filter(|t| !t.trim().is_empty()).unwrap_or_else(|| DEFAULT_PATTERN.into());
    if ids.len().saturating_mul(settings.len()) > MAX_FILES {
        return Err(bad(cmd, format!("more than {MAX_FILES} files")));
    }
    let doc = s.doc()?;
    let dir = string(p, "dir");
    let zip = dir.is_none() && get(p, "zip").and_then(Value::as_bool).unwrap_or(false);
    let date = today();
    let mut used = BTreeSet::new();
    let mut files = Vec::new();
    let mut members: Vec<(String, Vec<u8>)> = Vec::new();
    for (n, id) in ids.iter().enumerate() {
        for setting in &settings {
            let scale = super::file::export_scale(doc, &[*id], setting);
            let area = super::file::export_area(doc, &[*id], setting).ok_or_else(|| EngineError::Disabled(cmd.into(), "nothing to export".into()))?;
            let k = if setting.format.is_vector() { 1.0 } else { scale };
            let name = unique(file_name(doc, *id, n + 1, setting, scale, area.width() * k, area.height() * k, &pattern, &date), &mut used);
            let (bytes, _) = super::file::export_file(doc, &[*id], setting, scale)?;
            let mut f = json!({"name": name, "id": id.0, "bytes": bytes.len()});
            match &dir {
                Some(d) => {
                    let path = std::path::Path::new(d).join(&name);
                    if let Some(parent) = path.parent() {
                        let _ = crate::file_access::create_dir_all(parent);
                    }
                    let path = path.to_string_lossy().to_string();
                    super::file::write(&path, &bytes)?;
                    f["path"] = json!(path);
                }
                None if zip => members.push((name.clone(), bytes)),
                None => f["data"] = json!(base64::engine::general_purpose::STANDARD.encode(&bytes)),
            }
            files.push(f);
        }
    }
    let mut r = json!({"files": files});
    if zip {
        let archive = figcraft_format::zip_files(members.iter().map(|(n, b)| (n.as_str(), b.as_slice()))).map_err(|e| EngineError::Other(e.to_string()))?;
        r["data"] = json!(base64::engine::general_purpose::STANDARD.encode(&archive));
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_click_exports_every_size_and_format_named_by_the_pattern() {
        let mut s = Session::new();
        let f = s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 300, "height": 400, "name": "Card", "props": {"fills": ["#FF0000"]}})).unwrap()["id"].clone();
        let g = s.execute("frame.create", &json!({"x": 400, "y": 0, "width": 100, "height": 100, "name": "Card", "props": {"fills": ["#00FF00"]}})).unwrap()["id"].clone();
        let r = s
            .execute(
                "export.preset",
                &json!({"ids": [f, g], "pattern": "banners/{name}_{w}x{h}", "items": [{"size": "1x", "format": "png"}, {"size": "1080w", "format": "jpg"}, {"format": "pdf"}]}),
            )
            .unwrap();
        let names: Vec<&str> = r["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![
                "banners/Card_300x400.png",
                "banners/Card_1080x1440.jpg",
                "banners/Card_300x400.pdf",
                "banners/Card_100x100.png",
                "banners/Card_1080x1080.jpg",
                "banners/Card_100x100.pdf"
            ]
        );
        assert!(r["files"].as_array().unwrap().iter().all(|f| f["bytes"].as_u64().unwrap() > 100));
        // A name used twice gets a number; `..` stays in the folder.
        let r = s.execute("export.preset", &json!({"ids": [f, g], "pattern": "../{name}", "items": [{"format": "png"}]})).unwrap();
        let names: Vec<&str> = r["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
        assert_eq!(names, vec!["Card.png", "Card 2.png"]);
    }
}
