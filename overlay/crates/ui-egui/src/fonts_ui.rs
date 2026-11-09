//! FigCraft Plus: your own fonts, and more in the font list.
//!
//! - Font files (.ttf, .otf, .ttc) in a `fonts` folder next to the app are added at start, so a
//!   portable copy carries them; "Add fonts…" in the font list copies chosen files there.
//! - FigCraft's font list gains an "In this file" section (the families the open file uses) and
//!   tags the families from the fonts folder "Added".

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use egui::{Align2, CornerRadius, Rect, Sense, Ui, pos2, vec2};

use crate::FigApp;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::theme::{Tokens, regular};

/// The font file types read.
const EXTS: [&str; 3] = ["ttf", "otf", "ttc"];

/// The folder of your fonts: `fonts` next to the app.
fn dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("fonts")))
}

fn is_font(p: &Path) -> bool {
    p.extension().and_then(|x| x.to_str()).is_some_and(|x| EXTS.iter().any(|e| x.eq_ignore_ascii_case(e)))
}

/// Every font file under `root` (subfolders too, a few levels deep).
fn font_files(root: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else { return };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            if depth < 3 {
                font_files(&p, depth + 1, out);
            }
        } else if is_font(&p) {
            out.push(p);
        }
    }
}

/// Add the faces of the font file `path` that aren't known yet (a family and style already there,
/// bundled or installed, keeps its own). The families added.
fn install_file(path: &Path) -> Vec<String> {
    let Ok(bytes) = std::fs::read(path) else { return Vec::new() };
    let shared = std::sync::Arc::new(bytes);
    let count = face_count(&shared);
    let mut added = Vec::new();
    let mut db = figcraft_text::db().write().unwrap_or_else(PoisonError::into_inner);
    for i in 0..count {
        let Some(face) = figcraft_text::Face::from_bytes(figcraft_text::fontdb::FontBytes::Shared(shared.clone()), i, None, None) else { continue };
        let have = db.has_family(&face.family) && db.styles(&face.family).iter().any(|s| s.eq_ignore_ascii_case(&face.style));
        if have {
            continue;
        }
        if !added.iter().any(|f: &String| f.eq_ignore_ascii_case(&face.family)) {
            added.push(face.family.clone());
        }
        db.add(face);
    }
    drop(db);
    if !added.is_empty() {
        figcraft_text::clear_layout_cache();
    }
    added
}

/// How many fonts the file holds: a collection (`ttcf`) says so in its header, any other file is
/// one (or none, when it isn't a font: no face reads then).
fn face_count(bytes: &[u8]) -> u32 {
    match bytes.get(..12) {
        Some(h) if h.starts_with(b"ttcf") => h.get(8..12).and_then(|b| <[u8; 4]>::try_from(b).ok()).map_or(1, u32::from_be_bytes).min(64),
        _ => 1,
    }
}

// ---------------------------------------------------------------------------------------------
// State

#[derive(Default)]
struct State {
    started: bool,
    /// Lower-case names of the families added from the fonts folder.
    added: BTreeSet<String>,
    /// The families the open file uses.
    doc_families: Vec<String>,
    doc_key: Option<(u64, u64)>,
    /// "Add fonts…" was clicked: the next frame asks for files.
    want_files: bool,
    ctx: Option<egui::Context>,
}

fn state() -> MutexGuard<'static, State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(Default::default).lock().unwrap_or_else(PoisonError::into_inner)
}

fn note_added(families: Vec<String>) {
    let ctx = {
        let mut s = state();
        for f in families {
            s.added.insert(f.to_lowercase());
        }
        s.ctx.clone()
    };
    if let Some(c) = ctx {
        c.request_repaint();
    }
}

/// Each frame: at start add the fonts folder's fonts (in the background), note the open file's
/// families, and answer "Add fonts…".
pub(crate) fn tick(app: &mut FigApp, ctx: &egui::Context) {
    let start = {
        let mut s = state();
        let first = !s.started;
        s.started = true;
        s.ctx.get_or_insert_with(|| ctx.clone());
        first
    };
    if start && let Some(root) = dir() {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let mut files = Vec::new();
            font_files(&root, 0, &mut files);
            let added: Vec<String> = files.iter().flat_map(|f| install_file(f)).collect();
            note_added(added);
        });
        #[cfg(target_arch = "wasm32")]
        drop(root);
    }
    if std::mem::take(&mut state().want_files) {
        add_fonts(app);
    }
    let Some(d) = app.session.docs.get(app.session.active) else { return };
    let key = (d.uid, d.revision);
    if state().doc_key == Some(key) {
        return;
    }
    let mut fams: Vec<String> = Vec::new();
    let mut note = |f: &str| {
        if !f.is_empty() && !fams.iter().any(|x| x.eq_ignore_ascii_case(f)) {
            fams.push(f.to_string());
        }
    };
    for n in d.doc.nodes() {
        if let Some(t) = n.kind.text() {
            note(&t.style.font_family);
            for r in &t.runs {
                note(&r.style.font_family);
            }
        }
    }
    for s in &d.doc.styles {
        if let figcraft_doc::StyleKind::Text { style, .. } = &s.kind {
            note(&style.font_family);
        }
    }
    fams.sort_by_key(|f| f.to_lowercase());
    let mut s = state();
    s.doc_key = Some(key);
    s.doc_families = fams;
}

/// Ask for font files, copy them into the fonts folder and add them.
fn add_fonts(app: &mut FigApp) {
    if dir().is_none() {
        return;
    }
    app.pick_open(tr("Add fonts"), &EXTS, |_, src| copy_and_add(&src));
}

/// Copy the font file `src` into the fonts folder and add it.
fn copy_and_add(src: &Path) {
    let Some(root) = dir() else { return };
    if std::fs::create_dir_all(&root).is_err() {
        return;
    }
    let Some(name) = src.file_name() else { return };
    let dst = root.join(name);
    if dst != src && std::fs::copy(&src, &dst).is_err() {
        return;
    }
    note_added(install_file(&dst));
}

// ---------------------------------------------------------------------------------------------
// The font list (FigCraft's own menu, with its search, keyboard and hover preview; these add to it)

const ROW: f32 = 26.0;

/// The installed families (of `all`) the open file uses, for the list's "In this file" section.
pub(crate) fn in_file(all: &[String]) -> Vec<String> {
    let s = state();
    s.doc_families.iter().filter_map(|f| all.iter().find(|a| a.eq_ignore_ascii_case(f)).cloned()).collect()
}

/// Whether `family` came from the fonts folder next to the app.
fn added(family: &str) -> bool {
    state().added.contains(&family.to_lowercase())
}

/// "Added" at the right end of a family row from the fonts folder.
pub(crate) fn tag(ui: &Ui, rect: Rect, family: &str, hovered: bool) {
    if !added(family) {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let col = if hovered { t.accent_on } else { t.text_tertiary };
    ui.painter().text(pos2(rect.max.x - 6.0, rect.center().y), Align2::RIGHT_CENTER, tr("Added"), regular(10.0), col);
}

/// "Add fonts…" at the top of the font menu: pick font files; they are copied into the fonts
/// folder next to the app (a portable copy carries them) and added at once.
pub(crate) fn add_fonts_row(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(vec2(w, ROW), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(5), t.hover);
    }
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 10.0, r.center().y), vec2(12.0, 12.0)), Icon::Plus, t.icon);
    ui.painter().text(pos2(r.min.x + 22.0, r.center().y), Align2::LEFT_CENTER, tr("Add fonts…"), regular(11.5), t.text);
    if resp.on_hover_text(tr("Font files are kept in the fonts folder next to the app")).clicked() {
        state().want_files = true;
        ui.close();
        ui.ctx().request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_font_files() {
        assert_eq!(face_count(b"ttcf\0\x01\0\0\0\0\0\x03rest"), 3);
        assert_eq!(face_count(b"\0\x01\0\0"), 1);
        assert!(is_font(Path::new("a/B.TTF")) && is_font(Path::new("x.otf")) && !is_font(Path::new("x.woff2")));
    }

    #[test]
    fn reads_the_files_in_a_folder() {
        let root = std::env::temp_dir().join(format!("figcraftplus-fonts-{}", std::process::id()));
        let _ = std::fs::create_dir_all(root.join("sub"));
        std::fs::write(root.join("a.ttf"), b"x").unwrap();
        std::fs::write(root.join("sub").join("b.otf"), b"x").unwrap();
        std::fs::write(root.join("notes.txt"), b"x").unwrap();
        let mut files = Vec::new();
        font_files(&root, 0, &mut files);
        let names: Vec<String> = files.iter().filter_map(|p| p.file_name()?.to_str().map(String::from)).collect();
        assert_eq!(names, ["a.ttf", "b.otf"]);
        assert!(install_file(&root.join("a.ttf")).is_empty(), "not a font: nothing added");
        let _ = std::fs::remove_dir_all(&root);
    }
}
