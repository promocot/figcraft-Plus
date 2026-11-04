//! FigCraft Plus: variables as CSS custom properties, both ways — the way code writes them.
//!
//! - `variable.exportCss {collections?, colors?: hex|oklch, unit?: px|rem, root?, formulas?, path?}`
//!   writes FigCraft's CSS export (the default mode on `:root`, each other mode on
//!   `[data-<collection>="<mode>"]`) with what it leaves out: colours as `oklch()`, lengths in
//!   `rem` of the file's root size, and calculations as CSS — a derived colour as a relative colour
//!   (`oklch(from var(--brand) calc(l - 0.08) c h)`), a mix as `color-mix()`, a calculated number as
//!   `calc()`, FigCraft's own colour calculations (alpha variants, mixes) as `color-mix()`. Formulas
//!   CSS can't write (HSB) are written as the colour they show.
//! - `variable.importCss {css (text or a path), collection?}` reads custom properties into a
//!   collection: `:root` / `@theme` is the first mode, every other block (`[data-theme="dark"]`,
//!   `.dark`, `@media (prefers-color-scheme: dark)`) a mode named after it. Colours (hex, names,
//!   `rgb()`, `hsl()`, `oklch()`), lengths (px; rem and em of the file's root size), numbers and
//!   strings become variables; `var(--x)` an alias; relative colours, `color-mix()` and `calc()`
//!   calculated variables that follow what they read. `--space-m` lands in the group `space` when
//!   several properties share the prefix.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use figcraft_color::oklch_profile::Oklch;
use figcraft_color::{Color, ColorProfile};
use figcraft_doc::{Collection, ColorOperand, Document, Mode, ModeId, Variable, VariableId, VariableType, VariableValue, trim_num};
use serde_json::{Value, json};

use super::CommandSpec;
use crate::codegen::CssNames;
use crate::library::{is_remote_collection, local_collections};
use crate::oklch_links::{self as links, Base, Channel, Derive, Expr, Formula, Model};
use crate::params::{get, string};
use crate::token_roles::{NumberRole, Roles};
use crate::{EngineError, Result, Session, bad, spec};

/// The page data key holding the file's root size (FigCraft Plus's Units).
const ROOT_KEY: &str = "figcraftplus.remRoot";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "variable.exportCss",
            "Export variables as CSS",
            "{collections? [ids or names; default every collection of this file], colors? (hex (default) | oklch), unit? (px (default) | rem: lengths in rem of `root`), root? (px per rem; default the file's root size, else 16), formulas? (default true: derived colours as relative colours, mixes as color-mix(), calculated numbers as calc(), reading the variables they use), path? (write it there)} → {text}, or {path} when written",
            false,
            export
        ),
        spec!(
            "variable.importCss",
            "Import CSS variables",
            "{css (a stylesheet's text, or a path to a .css file), collection? (name; default \"Tokens\": joined when it exists)} — :root / @theme is the first mode, every other block a mode named after its selector ([data-theme=\"dark\"] → dark, .dark → dark, @media (prefers-color-scheme: dark) → dark); colours, lengths (px, rem, em), numbers and strings become variables, var() aliases, relative colours / color-mix() / calc() calculated variables → {variables (added), updated, modes, warnings}",
            true,
            import
        ),
    ]
}

// ---- export ---------------------------------------------------------------------------------------

/// The file's root size (px per rem): its own, else 16.
fn doc_root(doc: &Document) -> f64 {
    doc.pages
        .iter()
        .find_map(|p| doc.node(*p)?.plugin_data.get(ROOT_KEY)?.trim().parse::<f64>().ok())
        .filter(|r| r.is_finite() && *r > 0.0)
        .unwrap_or(16.0)
}

/// How the export writes values.
struct Out<'a> {
    doc: &'a Document,
    oklch: bool,
    rem: Option<f64>,
    formulas: bool,
    roles: Roles,
    /// Number variables colours read in their channels: plain numbers in CSS.
    in_colours: BTreeSet<VariableId>,
    /// Each variable's custom property, unique in the file (FigCraft's own names).
    names: CssNames,
    /// Number variables line heights follow as a percentage (`leading`): a unitless ratio in CSS.
    leading: BTreeSet<VariableId>,
}

impl Out<'_> {
    /// A variable by name (or id).
    fn var(&self, key: &str) -> Option<&Variable> {
        let vars = &self.doc.variables.variables;
        vars.iter().find(|v| v.name == key).or_else(|| vars.iter().find(|v| v.id == key))
    }

    fn reference(&self, key: &str) -> Option<String> {
        self.var(key).map(|v| format!("var({})", self.names.name(v)))
    }

    fn color(&self, c: Color) -> String {
        if !self.oklch {
            return c.to_css();
        }
        let ok = Oklch::from_color(c, self.doc.color_profile);
        let h = if ok.c < 1e-4 { 0.0 } else { ok.h };
        let body = format!("oklch({} {} {}", num(ok.l, 4), num(ok.c, 4), num(h, 2));
        if c.a >= 0.999 { format!("{body})") } else { format!("{body} / {}%)", num(f64::from(c.a) * 100.0, 1)) }
    }

    fn unitless(&self, v: &Variable) -> bool {
        self.roles.role(v) == NumberRole::Unitless || self.in_colours.contains(&v.id)
    }

    fn number(&self, v: &Variable, x: f64) -> String {
        if self.leading.contains(&v.id) {
            return num(crate::leading::percent_of(x) / 100.0, 4);
        }
        match self.rem {
            _ if self.unitless(v) => trim_num(x),
            Some(root) => format!("{}rem", num(x / root, 4)),
            None => format!("{}px", trim_num(x)),
        }
    }

    /// A Plus formula as CSS (None when CSS can't write it: HSB, or a variable it can't find).
    fn formula(&self, f: &Formula) -> Option<String> {
        if f.model == Model::Hsb {
            return None;
        }
        let space = if f.model == Model::Rgb { "srgb" } else { f.model.name() };
        let base = |b: &Base| match b {
            Base::Var(n) => self.reference(n),
            Base::Color(c) => Some(self.color(*c)),
        };
        if let Some(Derive::Mix { a, pa, b, pb }) = &f.derive {
            let side = |x: &Base, p: &Option<f64>| -> Option<String> {
                let s = base(x)?;
                Some(match p {
                    Some(p) => format!("{s} {}%", num(*p, 2)),
                    None => s,
                })
            };
            return Some(format!("color-mix(in {space}, {}, {})", side(a, pa)?, side(b, pb)?));
        }
        let keys = keys(f.model);
        let mut ch = Vec::new();
        for (k, c) in f.ch.iter().enumerate() {
            let alpha = k == 3;
            ch.push(match c {
                Channel::Fixed(x) if alpha => format!("{}%", num(*x, 1)),
                Channel::Fixed(x) => num(*x, if f.model == Model::Oklch && k < 2 { 4 } else { 2 }),
                // Alpha is 0–100 in a formula, 0–1 in CSS.
                Channel::Var(n) if alpha => format!("calc({} / 100)", self.reference(n)?),
                Channel::Var(n) => self.reference(n)?,
                Channel::Expr(Expr::Key(i)) if *i == k => keys[k].to_string(),
                Channel::Expr(e) if alpha => format!("calc(({}) / 100)", self.expr(e, &keys, true)?),
                Channel::Expr(e) => format!("calc({})", strip_outer(&self.expr(e, &keys, false)?)),
            });
        }
        let from = match &f.derive {
            Some(Derive::From(b)) => format!("from {} ", base(b)?),
            _ => String::new(),
        };
        let plain_alpha = match &f.ch[3] {
            Channel::Fixed(a) => f.derive.is_none() && (a - 100.0).abs() < 0.05,
            Channel::Expr(Expr::Key(3)) => f.derive.is_some(),
            _ => false,
        };
        let body = format!("{}({from}{} {} {}", f.model.name(), ch[0], ch[1], ch[2]);
        Some(if plain_alpha { format!("{body})") } else { format!("{body} / {})", ch[3]) })
    }

    /// An expression as CSS; in an alpha channel (`alpha`) the `from` colour's alpha is read in
    /// the formula's 0–100.
    fn expr(&self, e: &Expr, keys: &[&str; 4], alpha: bool) -> Option<String> {
        Some(match e {
            Expr::Num(x) => num(*x, 4),
            Expr::Var(n) => self.reference(n)?,
            Expr::Key(3) if alpha => "(alpha * 100)".into(),
            Expr::Key(k) => keys.get(*k).copied().unwrap_or("alpha").to_string(),
            Expr::Neg(x) => format!("-{}", self.expr(x, keys, alpha)?),
            Expr::Bin(a, op, b) => format!("({} {op} {})", self.expr(a, keys, alpha)?, self.expr(b, keys, alpha)?),
        })
    }

    /// A number variable's `calc()` as CSS.
    fn calc(&self, e: &Expr) -> Option<String> {
        Some(format!("calc({})", strip_outer(&self.expr(e, &["l", "c", "h", "alpha"], false)?)))
    }

    /// FigCraft's own colour calculations as `color-mix()`.
    fn native(&self, v: &VariableValue) -> Option<String> {
        let operand = |o: &ColorOperand| match o {
            ColorOperand::Variable { variable } => self.doc.variables.get_by_id(variable).map(|t| format!("var({})", self.names.name(t))),
            ColorOperand::Literal { literal } => Some(self.color(*literal)),
        };
        match v {
            VariableValue::ColorAlpha { variable, opacity } => {
                let t = self.doc.variables.get_by_id(variable)?;
                Some(format!("rgb(from var({}) r g b / {}%)", self.names.name(t), num(f64::from(*opacity) * 100.0, 2)))
            }
            VariableValue::ColorMix { from, to, t } => {
                Some(format!("color-mix(in srgb, {} {}%, {})", operand(from)?, num(f64::from(1.0 - t) * 100.0, 2), operand(to)?))
            }
            _ => None,
        }
    }
}

fn keys(m: Model) -> [&'static str; 4] {
    match m {
        Model::Oklch => ["l", "c", "h", "alpha"],
        Model::Hsl => ["h", "s", "l", "alpha"],
        Model::Hsb => ["h", "s", "b", "alpha"],
        Model::Rgb => ["r", "g", "b", "alpha"],
    }
}

/// `(a + b)` → `a + b`.
fn strip_outer(s: &str) -> String {
    if s.starts_with('(') && s.ends_with(')') {
        // Only when the outer pair encloses everything.
        let mut depth = 0i32;
        for (i, ch) in s.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && i < s.len() - 1 {
                return s.to_string();
            }
        }
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

fn num(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if s == "-0" { "0".into() } else { s }
}

/// The CSS for `colls` of `doc`.
pub fn css(doc: &Document, colls: &[&Collection], oklch: bool, rem: Option<f64>, formulas: bool) -> String {
    let mut in_colours = BTreeSet::new();
    let vars = &doc.variables;
    let lookup = |key: &str| vars.variables.iter().find(|v| v.name == key || v.id == key).map(|v| v.id.clone());
    for v in vars.variables.iter().filter(|v| v.kind == VariableType::Color) {
        if let Some(f) = links::formula_in(&v.description, None) {
            for (k, c) in f.ch.iter().enumerate() {
                let mut names = Vec::new();
                match c {
                    Channel::Var(n) => names.push(n.clone()),
                    Channel::Expr(_) => names.extend(f.reads()),
                    Channel::Fixed(_) => {}
                }
                let _ = k;
                in_colours.extend(names.iter().filter_map(|n| lookup(n)));
            }
        }
    }
    // Line-height variables (`leading`): `line-height: var(--leading-normal)` takes a ratio.
    let leading: BTreeSet<VariableId> = doc
        .ids()
        .filter_map(|id| crate::leading::layer_rule(doc, id))
        .chain(doc.styles.iter().filter_map(|st| crate::leading::style_rule(doc, &st.id)))
        .filter_map(|n| doc.variables.get(&n).map(|v| v.id.clone()))
        .collect();
    let out = Out { doc, oklch, rem, formulas, roles: Roles::new(doc), in_colours, names: CssNames::new(doc), leading };
    let flattener = vars.flattener();
    let mut text = String::new();
    for c in colls {
        let attr = crate::codegen::css_var_name(&c.name);
        for (i, m) in c.modes.iter().enumerate() {
            let sel = if i == 0 { ":root".to_string() } else { format!("[data-{}=\"{}\"]", attr.trim_start_matches('-'), safe(&m.name)) };
            text.push_str(&format!("/* {} · {} */\n{sel} {{\n", safe(&c.name), safe(&m.name)));
            for v in vars.variables.iter().filter(|v| v.collection == c.id) {
                let Some(raw) = v.values.get(&m.id) else { continue };
                let written = if out.formulas {
                    match v.kind {
                        VariableType::Color => links::formula_in(&v.description, None).filter(|f| f.linked()).and_then(|f| out.formula(&f)),
                        VariableType::Number => links::number_formula_in(&v.description).and_then(|e| out.calc(&e)),
                        _ => None,
                    }
                    .or_else(|| out.native(raw))
                } else {
                    None
                };
                let value = match (written, raw) {
                    (Some(w), _) => w,
                    (None, VariableValue::Alias(a)) if vars.get_by_id(a).is_some() => format!("var({})", out.names.name(vars.get_by_id(a).unwrap_or(v))),
                    _ => match flattener.value(v, &m.id, raw) {
                        Some(VariableValue::Color(col)) => out.color(col),
                        Some(VariableValue::Number(x)) => out.number(v, x),
                        Some(VariableValue::String(t)) => string_css(&t),
                        Some(VariableValue::Boolean(b)) => b.to_string(),
                        _ => continue,
                    },
                };
                text.push_str(&format!("  {}: {value};\n", out.names.name(v)));
            }
            text.push_str("}\n");
        }
    }
    text
}

/// A string variable as CSS: a list (a font stack) item by item, names with spaces quoted;
/// anything else as one quoted string.
fn string_css(t: &str) -> String {
    if t.contains(',') {
        return t
            .split(',')
            .map(|x| {
                let x = x.trim();
                if x.contains(' ') { format!("\"{}\"", safe(x)) } else { safe(x) }
            })
            .collect::<Vec<_>>()
            .join(", ");
    }
    format!("\"{}\"", safe(t))
}

fn safe(text: &str) -> String {
    text.replace("*/", "* /").replace(['"', '\\', '\n', '\r', '\u{c}'], " ")
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "variable.exportCss";
    let doc = s.doc()?;
    let colls: Vec<&Collection> = match get(p, "collections") {
        None | Some(Value::Null) => local_collections(doc).collect(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| x.as_str().and_then(|k| doc.variables.collection(k)).ok_or_else(|| bad(cmd, format!("no collection {x}"))))
            .collect::<Result<_>>()?,
        Some(_) => return Err(bad(cmd, "collections: [collection ids or names]")),
    };
    let oklch = match string(p, "colors").as_deref() {
        None | Some("hex") => false,
        Some("oklch") => true,
        Some(_) => return Err(bad(cmd, "colors: hex|oklch")),
    };
    let root = get(p, "root").and_then(Value::as_f64).filter(|r| r.is_finite() && *r > 0.0).unwrap_or_else(|| doc_root(doc));
    let rem = match string(p, "unit").as_deref() {
        None | Some("px") => None,
        Some("rem") => Some(root),
        Some(_) => return Err(bad(cmd, "unit: px|rem")),
    };
    let formulas = get(p, "formulas").and_then(Value::as_bool).unwrap_or(true);
    let text = css(doc, &colls, oklch, rem, formulas);
    match string(p, "path") {
        Some(path) => {
            super::file::write(&path, text.as_bytes())?;
            Ok(json!({"path": path}))
        }
        None => Ok(json!({"text": text})),
    }
}

// ---- import -------------------------------------------------------------------------------------

/// One custom property read from a block.
#[derive(Clone, Debug)]
struct Decl {
    /// Without the leading `--`.
    name: String,
    value: String,
}

/// The stylesheet's custom properties per mode, in order: (mode name, declarations); the first is
/// the default mode (`:root`, `@theme`, `html`).
fn blocks(css: &str) -> Vec<(Option<String>, Vec<Decl>)> {
    // Comments out.
    let mut text = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        text.push_str(&rest[..i]);
        rest = rest[i + 2..].find("*/").map_or("", |j| &rest[i + 2 + j + 2..]);
    }
    text.push_str(rest);
    let mut out: Vec<(Option<String>, Vec<Decl>)> = Vec::new();
    // Walk `selector { … }`, descending into at-rules (their own selector names the mode when the
    // inner one is :root).
    fn walk(text: &str, outer: Option<String>, out: &mut Vec<(Option<String>, Vec<Decl>)>) {
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let selector = rest[..open].trim().rsplit([';', '}']).next().unwrap_or("").trim().to_string();
            // The matching close.
            let mut depth = 0usize;
            let mut close = None;
            for (i, ch) in rest[open..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(open + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let Some(close) = close else { break };
            let body = &rest[open + 1..close];
            if selector.starts_with('@') && !selector.starts_with("@theme") {
                walk(body, mode_of(&selector).or_else(|| outer.clone()), out);
            } else {
                let decls: Vec<Decl> = body
                    .split(';')
                    .filter_map(|d| {
                        let (k, v) = d.split_once(':')?;
                        let k = k.trim().strip_prefix("--")?;
                        let v = v.trim().trim_end_matches("!important").trim();
                        (!k.is_empty() && !v.is_empty()).then(|| Decl { name: k.to_string(), value: v.to_string() })
                    })
                    .collect();
                if !decls.is_empty() {
                    let own = mode_of(&selector);
                    let mode = match (own, &outer) {
                        (None, Some(o)) => Some(o.clone()),
                        (m, _) => m,
                    };
                    match out.iter_mut().find(|(m, _)| *m == mode) {
                        Some((_, d)) => d.extend(decls),
                        None => out.push((mode, decls)),
                    }
                }
            }
            rest = &rest[close + 1..];
        }
    }
    walk(&text, None, &mut out);
    // The default mode first.
    out.sort_by_key(|(m, _)| m.is_some());
    out
}

/// The mode a selector or at-rule stands for (None: the default mode).
fn mode_of(selector: &str) -> Option<String> {
    let s = selector.trim();
    if s.is_empty() || s == ":root" || s == "html" || s.starts_with("@theme") || s == ":host" {
        return None;
    }
    if let Some(i) = s.find("prefers-color-scheme") {
        let v = s[i..].split(':').nth(1).unwrap_or("").trim_matches(|c: char| !c.is_alphanumeric()).to_string();
        return (!v.is_empty()).then_some(v);
    }
    // [data-theme="dark"] → dark; .dark / :root.dark → dark; the last word otherwise.
    if let Some(i) = s.find('=') {
        let v: String = s[i + 1..].chars().filter(|c| !matches!(c, '"' | '\'' | ']' | ' ')).collect();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let word = s.rsplit(['.', ' ', ':', '#', '[']).find(|w| !w.is_empty() && *w != "root")?;
    Some(word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').to_string()).filter(|w| !w.is_empty())
}

/// `--space-m` → `space/m` when `prefix` groups several properties.
fn variable_name(css: &str, grouped: &BTreeSet<String>) -> String {
    match css.split_once('-') {
        Some((g, leaf)) if grouped.contains(g) && !leaf.is_empty() => format!("{g}/{leaf}"),
        _ => css.to_string(),
    }
}

/// What a CSS value is.
#[derive(Clone, Debug, PartialEq)]
enum Parsed {
    Color(Color),
    Number(f64),
    String(String),
    Alias(String),
    /// A Plus formula line (a colour's) with a first value.
    ColorFormula(String),
    /// A `calc(…)` of numbers and number variables.
    NumberFormula(String),
}

/// `var(--x)` → `x`.
fn var_ref(s: &str) -> Option<&str> {
    let inner = s.trim().strip_prefix("var(")?.strip_suffix(')')?;
    let name = inner.split(',').next()?.trim().strip_prefix("--")?;
    (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')).then_some(name)
}

/// A length or number in px (`16px`, `1.5rem`, `2em`, `12`, `50%` as 50).
fn length(s: &str, root: f64) -> Option<f64> {
    let s = s.trim();
    let (n, k) = if let Some(x) = s.strip_suffix("rem") {
        (x, root)
    } else if let Some(x) = s.strip_suffix("em") {
        (x, root)
    } else if let Some(x) = s.strip_suffix("px") {
        (x, 1.0)
    } else if let Some(x) = s.strip_suffix('%') {
        (x, 1.0)
    } else {
        (s, 1.0)
    };
    n.trim().parse::<f64>().ok().filter(|v| v.is_finite()).map(|v| v * k)
}

/// A colour CSS writes without variables.
fn color(s: &str, profile: ColorProfile) -> Option<Color> {
    let s = s.trim();
    if let Some(c) = Color::from_hex_or_name(s) {
        return Some(c);
    }
    let lower = s.to_ascii_lowercase();
    let (fname, args) = lower.split_once('(')?;
    let args = args.strip_suffix(')')?;
    let (chans, alpha) = match args.split_once('/') {
        Some((c, a)) => (c, Some(a.trim())),
        None => (args, None),
    };
    let parts: Vec<&str> = chans.split([' ', ',']).filter(|p| !p.is_empty()).collect();
    let parts: Vec<&str> = if alpha.is_none() && parts.len() == 4 { parts[..3].to_vec() } else { parts.clone() };
    let legacy_alpha = if alpha.is_none() && chans.split([' ', ',']).filter(|p| !p.is_empty()).count() == 4 {
        chans.split([' ', ',']).filter(|p| !p.is_empty()).nth(3)
    } else {
        None
    };
    if parts.len() != 3 {
        return None;
    }
    let a = match alpha.or(legacy_alpha) {
        None => 1.0,
        Some(a) if a.ends_with('%') => a.trim_end_matches('%').parse::<f64>().ok()? / 100.0,
        Some(a) => a.parse::<f64>().ok()?,
    } as f32;
    let pct = |p: &str, full: f64| -> Option<f64> {
        match p.strip_suffix('%') {
            Some(x) => x.parse::<f64>().ok().map(|v| v / 100.0 * full),
            None => p.trim_end_matches("deg").parse::<f64>().ok(),
        }
    };
    let c = match fname.trim() {
        "rgb" | "rgba" => {
            let ch = |p: &str| pct(p, 255.0).map(|v| (v / 255.0) as f32);
            Color::new(ch(parts[0])?, ch(parts[1])?, ch(parts[2])?, a)
        }
        "hsl" | "hsla" => {
            let h = pct(parts[0], 360.0)?;
            let (sat, l) = (pct(parts[1], 100.0)?, pct(parts[2], 100.0)?);
            figcraft_color::Hsl { h: h as f32, s: (sat / 100.0) as f32, l: (l / 100.0) as f32 }.to_color().with_alpha(a)
        }
        "oklch" => {
            let l = match parts[0].strip_suffix('%') {
                Some(x) => x.parse::<f64>().ok()? / 100.0,
                None => parts[0].parse::<f64>().ok()?,
            };
            let c = pct(parts[1], 0.4)?;
            let h = pct(parts[2], 360.0)?;
            Oklch::new(l, c, h).to_color(profile, a)
        }
        _ => return None,
    };
    Some(c)
}

/// `var(--x)` → `{Name}` (the variable each property became), lengths in px.
fn plus_refs(s: &str, names: &HashMap<String, String>, root: f64) -> Option<String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("var(") {
        out.push_str(&rest[..i]);
        let close = rest[i..].find(')')? + i;
        let name = var_ref(&rest[i..=close])?;
        out.push_str(&format!("{{{}}}", names.get(name)?));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    // Lengths in px (a calc() of 1rem + {Gap}).
    let words: Vec<String> = out
        .split(' ')
        .map(|w| {
            let core = w.trim_matches(|c| c == '(' || c == ')');
            match (core.ends_with("rem") || core.ends_with("px") || core.ends_with("em"), length(core, root)) {
                (true, Some(px)) => w.replace(core, &trim_num(px)),
                _ => w.to_string(),
            }
        })
        .collect();
    Some(words.join(" "))
}

fn parse(value: &str, names: &HashMap<String, String>, root: f64, profile: ColorProfile) -> Option<Parsed> {
    let v = value.trim();
    if let Some(n) = var_ref(v) {
        return names.get(n).map(|x| Parsed::Alias(x.clone()));
    }
    if (v.starts_with('"') && v.ends_with('"') && v.len() >= 2) || (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2) {
        return Some(Parsed::String(v[1..v.len() - 1].to_string()));
    }
    if let Some(c) = color(v, profile) {
        return Some(Parsed::Color(c));
    }
    if let Some(x) = length(v, root) {
        return Some(Parsed::Number(x));
    }
    let lower = v.to_ascii_lowercase();
    if lower.starts_with("calc(") {
        let text = plus_refs(v, names, root)?;
        return links::number_formula_in(&text).map(|_| Parsed::NumberFormula(text));
    }
    if lower.contains("from ") || lower.starts_with("color-mix(") {
        let text = plus_refs(v, names, root)?;
        return links::parse_line(&text).filter(|(s, _)| s.is_none()).map(|_| Parsed::ColorFormula(text));
    }
    // Font stacks and other words: text (quotes off each item).
    (!v.contains('(')).then(|| Parsed::String(v.split(',').map(|x| x.trim().trim_matches(['"', '\'']).to_string()).collect::<Vec<_>>().join(", ")))
}

fn read_css(cmd: &str, p: &Value) -> Result<String> {
    let text = string(p, "css").ok_or_else(|| bad(cmd, "css: a stylesheet's text or a path"))?;
    if text.contains('{') || text.contains("--") {
        return Ok(text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let meta = std::fs::metadata(&text).map_err(|e| bad(cmd, format!("{text}: {e}")))?;
        if meta.len() > 16 * 1024 * 1024 {
            return Err(bad(cmd, "the stylesheet is over 16 MiB"));
        }
        std::fs::read_to_string(&text).map_err(|e| EngineError::Other(format!("{text}: {e}")))
    }
    #[cfg(target_arch = "wasm32")]
    Err(bad(cmd, "css: the stylesheet's text"))
}

fn import(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "variable.importCss";
    let css = read_css(cmd, p)?;
    let coll_name = string(p, "collection").filter(|n| !n.trim().is_empty()).unwrap_or_else(|| "Tokens".into());
    let blocks = blocks(&css);
    if blocks.is_empty() {
        return Err(bad(cmd, "no custom properties (--name: value) found"));
    }
    let doc = s.doc_mut()?;
    let root = doc_root(doc);
    let profile = doc.color_profile;
    let mut warnings: Vec<String> = Vec::new();
    // Names: prefixes shared by several properties become groups.
    let all: BTreeSet<String> = blocks.iter().flat_map(|(_, d)| d.iter().map(|x| x.name.clone())).collect();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for n in &all {
        if let Some((g, _)) = n.split_once('-') {
            *counts.entry(g.to_string()).or_default() += 1;
        }
    }
    let grouped: BTreeSet<String> = counts.into_iter().filter(|(_, c)| *c >= 2).map(|(g, _)| g).collect();
    let names: HashMap<String, String> = all.iter().map(|n| (n.clone(), variable_name(n, &grouped))).collect();
    // The collection (joined when it exists, never a library copy) and its modes.
    let cid = match doc.variables.collections.iter().find(|c| c.name == coll_name && c.extends.is_none() && !is_remote_collection(doc, &c.id)) {
        Some(c) => c.id.clone(),
        None => {
            let id = doc.new_collection_id();
            let name = super::variables::free_collection_name(doc, &coll_name);
            doc.variables.collections.push(Collection { id: id.clone(), name, modes: Vec::new(), extends: None });
            let first = doc.new_mode_id(&id).ok_or_else(|| EngineError::Other("no mode id".into()))?;
            if let Some(c) = doc.variables.collections.iter_mut().find(|c| c.id == id) {
                c.modes.push(Mode { id: first, name: "Default".into() });
            }
            id
        }
    };
    let mut modes: Vec<ModeId> = Vec::new();
    let mut added_modes = 0usize;
    for (i, (mode, _)) in blocks.iter().enumerate() {
        let coll = doc.variables.collections.iter().find(|c| c.id == cid);
        let found = match mode {
            None => coll.and_then(|c| c.modes.first()).map(|m| m.id.clone()),
            Some(name) => coll.and_then(|c| c.modes.iter().find(|m| m.name.eq_ignore_ascii_case(name))).map(|m| m.id.clone()),
        };
        let id = match found {
            Some(id) => id,
            None => {
                let first = coll.and_then(|c| c.modes.first()).map(|m| m.id.clone());
                let Some(id) = doc.new_mode_id(&cid) else { continue };
                let name = mode.clone().unwrap_or_else(|| format!("Mode {}", i + 1));
                if let Some(c) = doc.variables.collections.iter_mut().find(|c| c.id == cid) {
                    c.modes.push(Mode { id: id.clone(), name });
                }
                for var in doc.variables.variables.iter_mut().filter(|v| v.collection == cid) {
                    if let Some(val) = first.as_ref().and_then(|f| var.values.get(f)).cloned() {
                        var.values.insert(id.clone(), val);
                    }
                }
                added_modes += 1;
                id
            }
        };
        modes.push(id);
    }
    // Every property's type from its first value that tells.
    let mut kinds: BTreeMap<String, VariableType> = BTreeMap::new();
    let mut parsed: Vec<Vec<(String, Option<Parsed>)>> = Vec::new();
    for (_, decls) in &blocks {
        parsed.push(decls.iter().map(|d| (d.name.clone(), parse(&d.value, &names, root, profile))).collect());
    }
    // Aliases take their target's type: settle the direct ones first, then the chains.
    for block in &parsed {
        for (n, pv) in block {
            let kind = match pv {
                Some(Parsed::Color(_) | Parsed::ColorFormula(_)) => VariableType::Color,
                Some(Parsed::Number(_) | Parsed::NumberFormula(_)) => VariableType::Number,
                Some(Parsed::String(_)) => VariableType::String,
                _ => continue,
            };
            kinds.entry(n.clone()).or_insert(kind);
        }
    }
    let by_var: HashMap<&String, &String> = names.iter().map(|(k, v)| (v, k)).collect();
    for _ in 0..8 {
        for block in &parsed {
            for (n, pv) in block {
                if let Some(Parsed::Alias(target)) = pv
                    && !kinds.contains_key(n)
                    && let Some(k) = by_var.get(target).and_then(|css| kinds.get(*css)).copied()
                {
                    kinds.insert(n.clone(), k);
                }
            }
        }
    }
    // Create or find the variables.
    let mut ids: BTreeMap<String, VariableId> = BTreeMap::new();
    let (mut added, mut updated) = (0usize, 0usize);
    for n in &all {
        let Some(kind) = kinds.get(n).copied() else {
            warnings.push(format!("--{n}: not a colour, length, number or text CSS FigCraft can hold"));
            continue;
        };
        let name = &names[n];
        match doc.variables.variables.iter().find(|v| v.collection == cid && v.name == *name) {
            Some(v) if v.kind != kind => warnings.push(format!("--{n}: `{name}` is a {} variable here; skipped", v.kind.id())),
            Some(v) => {
                ids.insert(n.clone(), v.id.clone());
                updated += 1;
            }
            None => {
                let id = doc.new_variable_id();
                doc.variables.variables.push(Variable {
                    id: id.clone(),
                    name: name.clone(),
                    collection: cid.clone(),
                    kind,
                    values: BTreeMap::new(),
                    description: String::new(),
                    scopes: None,
                    hidden_from_publishing: false,
                    extends: None,
                    overrides: BTreeSet::new(),
                    code_syntax: BTreeMap::from([("WEB".to_string(), format!("var(--{n})"))]),
                });
                ids.insert(n.clone(), id);
                added += 1;
            }
        }
    }
    // Values, per mode.
    let all_modes: Vec<ModeId> = doc.variables.collections.iter().find(|c| c.id == cid).map(|c| c.modes.iter().map(|m| m.id.clone()).collect()).unwrap_or_default();
    for (block, mode) in parsed.iter().zip(&modes) {
        for (n, pv) in block {
            let Some(id) = ids.get(n).cloned() else { continue };
            let Some(kind) = kinds.get(n).copied() else { continue };
            let (value, formula) = match pv {
                Some(Parsed::Color(c)) if kind == VariableType::Color => (VariableValue::Color(*c), None),
                Some(Parsed::Number(x)) if kind == VariableType::Number => (VariableValue::Number(*x), None),
                Some(Parsed::String(t)) if kind == VariableType::String => (VariableValue::String(t.clone()), None),
                Some(Parsed::ColorFormula(f)) if kind == VariableType::Color => (VariableValue::Color(Color::BLACK), Some(f.clone())),
                Some(Parsed::NumberFormula(f)) if kind == VariableType::Number => (VariableValue::Number(0.0), Some(f.clone())),
                Some(Parsed::Alias(target)) => {
                    let Some(t) = by_var.get(target).and_then(|css| ids.get(*css)) else { continue };
                    if *t == id {
                        continue;
                    }
                    (VariableValue::Alias(t.clone()), None)
                }
                _ => {
                    warnings.push(format!("--{n}: its value in a mode doesn't fit its type; skipped"));
                    continue;
                }
            };
            if let Some(v) = doc.variables.variables.iter_mut().find(|v| v.id == id) {
                v.values.insert(mode.clone(), value);
                if let Some(f) = formula {
                    // A calculation follows what it reads (in every mode: the formula is the variable's).
                    let rest = links::without_number_formula(&v.description);
                    let rest: String = rest.lines().filter(|l| links::parse_line(l).is_none()).collect::<Vec<_>>().join("\n");
                    v.description = if rest.trim().is_empty() { f } else { format!("{}\n{f}", rest.trim_end()) };
                }
            }
        }
    }
    // Every mode holds a value.
    for id in ids.values() {
        if let Some(v) = doc.variables.variables.iter_mut().find(|v| v.id == *id) {
            let first = all_modes.iter().find_map(|m| v.values.get(m)).cloned().unwrap_or_else(|| VariableValue::default_for(v.kind));
            for m in &all_modes {
                v.values.entry(m.clone()).or_insert_with(|| first.clone());
            }
        }
    }
    Ok(json!({"variables": added, "updated": updated, "modes": added_modes, "warnings": warnings}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).ok();
        s
    }

    #[test]
    fn css_round_trips_colours_lengths_aliases_and_modes() {
        let mut s = session();
        let css = r#"
            /* tokens */
            :root {
              --brand: #3366ff;
              --brand-hover: oklch(from var(--brand) calc(l - 0.08) c h);
              --surface: oklch(0.98 0 0);
              --space-s: 8px;
              --space-m: 1rem;
              --space-l: calc(var(--space-m) * 2);
              --font-body: "Inter", sans-serif;
              --accent: var(--brand);
            }
            [data-theme="dark"] { --surface: #111111; }
            @media (prefers-color-scheme: dark) { :root { --brand: #5588ff; } }
        "#;
        let r = s.execute("variable.importCss", &json!({"css": css})).unwrap();
        assert_eq!(r["variables"], 8, "{r}");
        let doc = s.doc().unwrap();
        let c = doc.variables.collection("Tokens").unwrap();
        let modes: Vec<&str> = c.modes.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(modes, ["Default", "dark"], "[data-theme=dark] and the dark media query are one mode");
        let get = |n: &str| doc.variables.variables.iter().find(|v| v.name == n).unwrap_or_else(|| panic!("no {n}"));
        let m0 = &c.modes[0].id;
        let m1 = &c.modes[1].id;
        assert_eq!(get("space/m").values[m0], VariableValue::Number(16.0), "1rem of the 16 px root");
        assert!(get("space/l").description.contains("calc({space/m} * 2)"), "{}", get("space/l").description);
        assert_eq!(get("space/l").values[m0], VariableValue::Number(32.0), "the calculation was evaluated");
        assert!(matches!(get("accent").values[m0], VariableValue::Alias(_)));
        assert_eq!(get("font-body").values[m0], VariableValue::String("Inter, sans-serif".into()));
        assert_eq!(get("surface").values[m1], VariableValue::Color(Color::from_hex("#111111").unwrap()));
        assert_eq!(get("brand").values[m1], VariableValue::Color(Color::from_hex("#5588ff").unwrap()));
        let hover = get("brand-hover");
        assert!(hover.description.starts_with("oklch(from {brand}"), "{}", hover.description);
        let VariableValue::Color(h) = hover.values[m0] else { panic!() };
        let base = Oklch::from_color(Color::from_hex("#3366ff").unwrap(), ColorProfile::Srgb);
        assert!((Oklch::from_color(h, ColorProfile::Srgb).l - (base.l - 0.08)).abs() < 0.01, "the hover is darker");
        assert_eq!(get("brand").code_syntax.get("WEB").map(String::as_str), Some("var(--brand)"), "code keeps the CSS name");

        // Back out: oklch, rem, formulas.
        let out = s.execute("variable.exportCss", &json!({"colors": "oklch", "unit": "rem"})).unwrap();
        let text = out["text"].as_str().unwrap();
        assert!(text.contains("--space-m: 1rem;"), "{text}");
        assert!(text.contains("--space-l: calc(var(--space-m) * 2);"), "{text}");
        assert!(text.contains("--brand-hover: oklch(from var(--brand) calc(l - 0.08) c h);"), "{text}");
        assert!(text.contains("--accent: var(--brand);"), "{text}");
        assert!(text.contains("--font-body: Inter, sans-serif;"), "{text}");
        assert!(text.contains("--surface: oklch(0.9") || text.contains("--surface: oklch(0.98"), "{text}");
        assert!(text.contains("[data-tokens=\"dark\"]"), "{text}");
        // Plain: FigCraft's colours, px.
        let plain = s.execute("variable.exportCss", &json!({"formulas": false})).unwrap();
        let plain = plain["text"].as_str().unwrap();
        assert!(plain.contains("--space-l: 32px;") && plain.contains("--brand: #3366FF;"), "{plain}");
    }

    #[test]
    fn modes_from_selectors() {
        assert_eq!(mode_of(":root"), None);
        assert_eq!(mode_of("@theme static"), None);
        assert_eq!(mode_of("[data-theme=\"dark\"]").as_deref(), Some("dark"));
        assert_eq!(mode_of(".dark").as_deref(), Some("dark"));
        assert_eq!(mode_of(":root.compact").as_deref(), Some("compact"));
        assert_eq!(mode_of("@media (prefers-color-scheme: dark)").as_deref(), Some("dark"));
    }

    #[test]
    fn colours_css_writes() {
        let p = ColorProfile::Srgb;
        assert_eq!(color("rgb(255 0 0 / 50%)", p).map(|c| c.to_css()), Some("rgba(255, 0, 0, 0.50)".into()));
        assert_eq!(color("rgba(0, 0, 255, 1)", p).map(|c| c.to_hex()), Some("0000FF".into()));
        assert_eq!(color("hsl(120 100% 50%)", p).map(|c| c.to_hex()), Some("00FF00".into()));
        assert!(color("oklch(62.8% 0.2577 29.23)", p).is_some_and(|c| c.to_hex() == "FF0000"));
        assert_eq!(color("tomato", p).map(|c| c.to_hex()), Some("FF6347".into()));
        assert_eq!(length("1.5rem", 16.0), Some(24.0));
        assert_eq!(strip_outer("(a + b)"), "a + b");
        assert_eq!(strip_outer("(a) + (b)"), "(a) + (b)");
    }
}
