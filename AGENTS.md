# FigCraft Plus — guide for AI agents

Read this first in every new session. It holds what a previous agent learned building this
project with its owner, so you can carry on without rediscovering it. Then read
[`ROADMAP.md`](ROADMAP.md) (what's next) and the top of [`CHANGELOG.md`](CHANGELOG.md) (what just
shipped).

## 1. What this is

**FigCraft Plus** adds professional colour, design-system and design-to-code tools to
[FigCraft](https://github.com/storytold/figcraft), an open-source Rust/egui design editor. This
repository holds **only the additions**. A GitHub Actions workflow checks out each official
FigCraft release, applies them, runs the tests and publishes a portable Windows build under
Releases as `v<figcraft>-plus.<N>`, named "FigCraft Plus N · FigCraft vX".

- Repository: `promocot/FigCraft-Plus` (formerly `Figcraft-addon` and `figcraft-oklch-support`;
  GitHub redirects the old names).
- Upstream: `storytold/figcraft`, and it moves fast. Its own `AGENTS.md`, `ROADMAP.md` and
  `docs/gaps.md` say what it plans next. Read them before planning, so Plus doesn't build what
  FigCraft is about to ship.
- Our three specialities: **Advanced colour · Design systems · Design to code.**

### Working with the owner

- The owner is a designer and frontend developer. **Talk to him in Russian.** Repository text
  (README, docs, code, commits) is in English.
- He often relays answers from another AI reviewer. Give an honest technical assessment: say
  where you agree, where you'd do otherwise and why. Don't just comply.
- When he says not to develop yet («не разрабатывай»), only plan, assess or edit docs.
- For big or ambiguous work, ask a short question first, with a recommended option. Then work
  through it with a task list.
- Release only after the tests pass **and** you have checked the feature in the running app
  (section 5). Report briefly: what shipped, what you verified, any deliberate deviation and why.

## 2. Non-negotiables

1. **Never name the commercial design tool FigCraft is modelled on** (the incumbent its own docs
   compare against) — not in the README, docs, changelog, commit messages, UI strings, release
   notes, topics or images, and not in this file. The owner wants zero legal exposure. Compare only
   with FigCraft ("In FigCraft v0.14.0 …"). Upstream FigCraft's own files may name it; ours never
   do. Before every push: `grep -rIni "<that name>" --exclude-dir=.git .` must find nothing.
2. **Files stay compatible with the official FigCraft, both ways.**
   - Never change FigCraft's document model or file format. `scripts/apply.py` stops the build if
     any overlay file or hook targets `crates/doc/` or `crates/format/`.
   - Plus data goes into fields FigCraft already keeps as text: layer/page `plugin_data`
     (`figcraftplus.*` keys) and variable/style descriptions.
   - Values are stored as FigCraft stores them (lengths in px). Units, formulas and settings are
     only presentation or derivation on top.
3. **Add only what FigCraft lacks.** When an official release ships a feature, remove ours: delete
   its hook and overlay parts, and move it to *Now in FigCraft itself* in the README. Before
   proposing anything, grep the upstream source — FigCraft already has a lot (see *Already in
   FigCraft* in the roadmap).
4. **No third-party assets.** No icons, images or UI kits from other products. Images in `docs/`
   are our own renders and screenshots of our own app.
5. **`main` is the release branch.** A push to `main` that touches `overlay/`, `hooks/`,
   `scripts/`, `CUSTOM_VERSION` or `build.yml` builds and publishes. Work in progress goes on a
   branch (`plus-N`, `support`, …). Push WIP there when a hook asks you to commit.
6. **Some account actions belong to the owner**, and the session proxy blocks most of them:
   - repository topics, social preview, renames;
   - deleting releases or tags;
   - GitHub Sponsors and Ko-fi.

   Explain the clicks to him instead. GitHub's GraphQL API is blocked here: use REST
   (`gh api repos/promocot/FigCraft-Plus/...`). `gh release list` fails, `gh api .../releases`
   works.

## 3. Repository map

```
overlay/crates/<crate>/src/…   our own source files, copied into FigCraft at the same paths
hooks/NNNN-name.json           where they connect to FigCraft's code (see §4)
scripts/apply.py               copies overlay/, applies hooks/, guards the file format
scripts/dev/fc.py              control-channel client: drive the running app, take screenshots
CUSTOM_VERSION                 the Plus number N (bump for every release)
CHANGELOG.md                   per-release notes; the release text is taken from it (§6)
ROADMAP.md                     staged plan by FigCraft version
docs/images/                   README screenshots, GIFs, hero; docs/social-preview.png
.github/workflows/build.yml    plan → test (Linux) → windows build → release
.github/ISSUE_TEMPLATE/        feature request, bug report, links
```

### The add-ons (overlay modules)

| Module | What it does |
|---|---|
| `color/oklch_profile.rs` | OKLCH ⇄ RGB in the file's profile (sRGB / Display P3), gamut mapping (CSS Color 4) |
| `engine/oklch_links.rs` | Colour channels bound to number variables; **derived colours** (`oklch(from {X} …)`, `color-mix`) and **calculated numbers** (`calc({A} * 2)`) kept in variable descriptions / layer data, resolved in every mode, cycle-safe |
| `engine/cmd/var_order.rs` | Reordering and sorting variables, groups, collections (also as commands) |
| `ui-egui/oklch_fields.rs` | Colour picker models (Hex, RGB, HSL, HSB, LCH, OKLCH, CMYK, CSS), the OKLCH plane and hue strip, captions, resizable width, the CSS field with formulas |
| `ui-egui/color_tools_ui.rs` | CMYK row (same naive formula as FigCraft's CMYK PDF export), *Copy as*, WCAG/APCA contrast against the layers really under the layer |
| `ui-egui/picker_place_ui.rs` | The picker follows the selection to the same slot; opens beside a style dialog (FigCraft 0.13 places and closes it otherwise) |
| `ui-egui/picker_libraries_ui.rs` | The picker's **Libraries** tab: colour variables filtered by the scope of what is edited |
| `ui-egui/var_picker_ui.rs` | One variable picker card everywhere (`card`, `embedded`, `offered`, `apply`); scopes |
| `ui-egui/units_ui.rs` | **Units**: per-document layout unit (px, rem, mm, cm, in, pt) and text unit (px, rem, pt); which fields are lengths; the field wrapper's conversion; the unit chip and right-click unit menu; canvas size label and measurements |
| `ui-egui/rem_ui.rs` | Root size, parsing typed units (`210mm`, `1.5rem`, the "explicit unit" flag), text-unit helpers |
| `ui-egui/prefs_plus_ui.rs` | Preferences ▸ FigCraft Plus (`figcraft-plus.json` next to the exe: units, root size, decimals, picker model); the Units popover |
| `ui-egui/popover_ui.rs` | Our popovers (variable list, Units, style dialogs) on FigCraft 0.13's `popover::Popover`, kept per id; `open_rects` (held by the colour picker); selection-change closing |
| `ui-egui/style_dialog_ui.rs`, `styles_plus_ui.rs`, `style_groups_ui.rs` | Create/edit styles in a dialog with preview; the **+** on Local styles; nested style groups |
| `ui-egui/text_vars_ui.rs` | Weight, paragraph spacing and indent from variables |
| `ui-egui/fonts_ui.rs` | Portable `fonts` folder, *Add fonts…*, *In this file* |
| `ui-egui/var_order_ui.rs` | Drag and drop and sorting in the Variables editor |
| `ui-egui/oklch_picker_tests.rs` | Headless egui tests of the picker |

Page `plugin_data` keys: `figcraftplus.layoutUnit`, `figcraftplus.remUnit` (the **text** unit:
px, rem or pt — the name is historical), `figcraftplus.remRoot`.

## 4. How add-ons attach (hooks)

A hook file is a JSON list of edits to FigCraft's source:

```json
{"note": "…", "file": "crates/ui-egui/src/lib.rs", "after": "^pub mod windows;$", "insert": "mod units_ui;"}
{"note": "…", "file": "crates/…/x.rs", "find": "exact text, once", "replace": "new text"}
```

- `python3 scripts/apply.py <figcraft-dir>` applies everything. It skips hooks whose result is
  already there, so it is safe to re-run. `--check` lists every hook that no longer fits instead of
  stopping at the first.
- Keep hooks **small**: one line to call into an overlay module, which holds the logic. This keeps
  porting to a new FigCraft version cheap.
- If you **edit** an existing hook's text, re-apply on a *clean* tree. Otherwise both the old and
  the new insert end up there:
  `git -C <figcraft> checkout -- . && git -C <figcraft> clean -fdq crates && python3 scripts/apply.py <figcraft>`.
- UI strings: every `tr("…")` key must exist in `crates/ui-egui/locales/en.toml` **and** be used.
  FigCraft's i18n test fails on unknown or unused keys. Add them through a locale hook, in English
  and Russian (see `0013-rem.json`).
- `0012-upstream-tests.json` marks FigCraft tests that conflict with intended Plus behaviour as
  `#[ignore = "FigCraft Plus: why"]`. Add to it only with a reason; prefer fixing our code.

**On a new FigCraft release:** run `apply.py --check` and fix the anchors that moved. Grep for
renamed APIs, build, run the tests, and check the README *Now in FigCraft itself* list against the
new release notes.

## 5. Local development loop

```sh
git clone --depth 1 --branch v0.14.0 https://github.com/storytold/figcraft work   # the tag CI builds
python3 scripts/apply.py work
cd work
cargo build --bin figcraft             # dev profile: ~11 min the first time, ~2 min after
cargo test -p figcraft-ui-egui --lib   # ~45 s once built; CI also runs the engine/color tests in build.yml
```

Build in the background and keep working. A full UI test run must pass before a release.

**Check every feature in the running app** (Linux, no display):

```sh
Xvfb :99 -screen 0 1440x900x24 &
DISPLAY=:99 HOME=/tmp/fc-home ./target/debug/figcraft --control 7979 &
```

Then drive the app with `scripts/dev/fc.py`: engine commands, clicks, typing, screenshots (docstring
inside). Lessons learned:

- Screenshot paths must be **absolute**.
- Look at the screenshot to find click targets. If the interface size isn't 100%, set
  `fc.SCALE` (points = pixels / scale).
- `figcraft-plus.json` lives next to the binary (`target/debug/`). Write it before launching to
  set preferences (`{"layoutUnit": "mm", "textUnit": "pt", "precision": 2, "colorModel": "OKLCH"}`).
- Stop the app with `pkill -x figcraft` (`pkill -f figcraft` kills your own shell).
- GIFs: snapshot frames with `c.shot`, then ffmpeg with per-frame palettes
  (`palettegen=stats_mode=single`, `paletteuse=new=1`). Crop to the subject; keep each GIF under
  ~2.5 MB.

### Pitfalls that cost time before

- **Don't put clickable widgets inside FigCraft's own fields in px.** Upstream tests click the
  middle of fields. Make extra affordances appear only in Plus modes, or use a right-click.
- Extra interactive areas next to fields use `Sense::CLICK`, not `Sense::click()`. The latter is a
  Tab stop and breaks Tab/Shift+Tab between fields.
- Two widgets with the same labels in one scope clash on ids (red error text in the UI). Wrap them
  in `ui.push_id(...)`.
- A popover opened by a click must ignore that same click as "outside" (remember the frame it
  toggled in).
- `ctx.read_response(id)` gives a widget's response before it is drawn (rects from the last pass).
  Useful for context menus on FigCraft's widgets.
- Number formatting for display goes through `widgets::fmt_num`, which honours *Decimals shown*.
  Never round stored values.

## 6. Releasing

1. Bump `CUSTOM_VERSION` to N.
2. `CHANGELOG.md`: add `## Plus N · FigCraft vX — YYYY-MM-DD` at the top. The workflow copies this
   section into the GitHub release notes, so write it for users. Mark items 🟣 *Exclusive* (FigCraft
   has nothing like it) or 🔵 *Enhanced* (FigCraft has a basic version).
3. `ROADMAP.md`: tick or move the items, and add a row to *Done*.
4. `README.md`:
   - the version line under the hero;
   - the feature's section (FigCraft vs FigCraft Plus quote, screenshot, bullets);
   - its row in *Feature comparison*;
   - the TOC.

   Screenshots go to `docs/images/NN-name.png`; the next free number is in the folder.
5. Push to the branch, merge to `main` (fast-forward), push. Watch the run:
   `gh api "repos/promocot/FigCraft-Plus/actions/runs?per_page=1"`, then the release via
   `gh api "repos/promocot/FigCraft-Plus/releases?per_page=1"`.
6. Docs-only pushes (README, ROADMAP, CHANGELOG, images) don't build. Changing `build.yml`
   rebuilds the current version and refreshes its notes.

## 7. Style of the docs

- Plain, concrete English. Say what the user sees and does, in UI words ("Preferences ▸ FigCraft
  Plus ▸ Decimals shown").
- Plus's own UI strings and the README prose say "colour" (British); FigCraft's strings say
  "Color"; the hero image says "color". Don't mass-rename: text must match what the app shows.
- README order: hero → badges → version line → Why (3 pillars) → See it in action (GIFs) →
  Compatibility → Roadmap block → Contents → Feature comparison → Download and update → Requested
  in FigCraft's issues → Now in FigCraft itself → Add-ons sections → Community → FAQ → How it works.

## 8. Where things stand (update this section when you finish work)

- **Released:** Plus 14 on FigCraft v0.14.0 (2026-11-04): roadmap stages 5 and 6.
  - Engine `leading.rs`: line height in % from a number variable (layer data
    `figcraftplus.lineHeight`, a text style's description line `lineHeight {name}`; ≤ 5 is a
    ratio); commands `text.lineHeightVariable`, `style.lineHeightVariable`. UI `leading_ui.rs`: the
    px | % switch in the variable picker (`var_picker_ui` calls `begin` / `switch` / `frame_done`),
    the chip in Typography and the style dialog (which hides the rule line from its description).
    CSS for code writes these variables as ratios.
  - Engine `gradient_oklch.rs`: `fills/0 in oklch <key positions>` (layer data
    `figcraftplus.gradients`, a paint style's description); 16 steps between keys, mixed in OKLCH;
    stops that aren't steps are keys; bindings and stop formulas renumbered. Command
    `gradient.blend`. UI `gradient_oklch_ui.rs` (Blend row; steps hidden on the bar and canvas).
  - Engine `dpi.rs` (`figcraftplus.dpi` on top-level frames; `frame.resolution` scales with
    `arrange::scale_node` and moves right / lower siblings; default for new frames; PNG `pHYs` /
    JPEG JFIF density) and `pdfx.rs` (`figcraftplus.pdfx` on the first page; `print.pdfx`). The PDF
    crate's `plus.rs` scales a frame's page by 72 / DPI and, for PDF/X-4, writes krilla's
    output intent with craft-cms's `craft-coated-fogra39.icc` (copied into the overlay; MIT OR
    Apache-2.0) and converts CMYK through it with moxcms (hooked into `print::cmyk_of`; the pdf
    crate gains `moxcms`, and the hook edits `Cargo.lock` for `--locked`). UI `dpi_ui.rs`: Layout ▸
    Resolution, the DPI lengths are read at (`units_ui::per`, `rem_ui::unit`), the PDF/X checkbox.
  - Hooks 0024 (typography and colour tokens) and 0025 (print). Build with `CARGO_INCREMENTAL=0`
    when the session disk is tight.
- **Plus 13** on FigCraft v0.14.0 (2026-11-02): fixes. The style dialog's swatch opens
  the picker with `app.open_picker(target, Some(rect))` — setting `app.ui.picker` alone leaves
  `picker_button` empty, and `picker_place_ui::open` can't place it beside the dialog (test in
  `picker_follow_tests.rs`); the contrast samples stack below 2 × 132 px. README screenshots
  retaken on 0.14 (light theme; the demo GIFs dark).
- **Plus 12** on FigCraft v0.14.0 (2026-11-01): ported (all hooks applied as they
  were; `tokens_css` uses 0.14's `CssNames`), roadmap stage 4. Engine `responsive.rs` (layer data
  `figcraftplus.layout`: `width 50%`, `height 100vh`, `direction {Var}`…; `resolve` runs in
  `derived::normalize` after the OKLCH links; commands `layout.relative`, `layout.bindVariable`)
  and `key_object.rs` (thread-local key; the move tool's click-in-selection makes it; `align.*`
  uses it); UI `responsive_ui.rs`; hook 0023.
- **Plus 11** on FigCraft v0.13.1 (2026-11-01): roadmap stages 2–3. Engine commands
  `variable.exportCss` / `variable.importCss` (`engine/cmd/tokens_css.rs`) and
  `designSystem.inspect` (`engine/cmd/ds_inspect.rs`); UI `tokens_ui.rs` (Export ▸ CSS for code,
  Import menu), `ds_inspect_ui.rs` (Inspect button), `text_bulk_ui.rs` (Edit N text styles),
  `font_swap_ui.rs` (Missing fonts ▸ Replace…, `fontReplacements` in `figcraft-plus.json`); hooks
  0019–0022. Lesson: never call a `ctx` method inside `ctx.data_mut(|d| …)` — it deadlocks
  (read `cumulative_frame_nr()` first).
- **Plus 10** on FigCraft v0.13.1 (2026-10-31): the picker follows the selection again
  (0.13's outside press had closed it on a canvas / Layers click); `pickerFollowsSelection` in
  `figcraft-plus.json`, Preferences checkbox, default on; `picker_place_ui::held` holds the canvas
  and left panels from FigCraft's outside-press rule, `follow` closes on a click that selects
  nothing new. Tests: `picker_follow_tests.rs`.
- **Plus 9** on FigCraft v0.13.1 (2026-10-31). Ported to 0.13.1; removed what 0.13
  does itself (picker frame, placement and outside-press closing; the style-menu restyle, hook
  0018 deleted; our popover frame — Plus popovers now use FigCraft's `Popover`). Stage 1 done:
  rulers and guides in the layout unit (`units_ui::ruler_step`), Dev Mode properties in the
  document units and the code unit kept per file (`figcraftplus.codeUnit`), number variables shown
  by role (`units_ui::number_roles`, own mapping: line height / letter spacing count as text), `em`
  for line height and letter spacing (`rem_ui::em`, stored as %), contrast both ways
  (`color_tools_ui::both_ways`).
- **Next — roadmap Stage 5 (on 0.14):** line height % variables, gradient tokens. Then Stage 6 (print: DPI / PPI per top-level frame — page plugin
  data, default in Preferences, 72 for the web; changing it scales the frame and its content by
  new/old, keeps the physical size, moves neighbouring frames aside, mm/cm/in/pt and exports read
  the frame's DPI; PDF/X). Done items below are kept for reference.
- **Was Stage 2 (done in Plus 11):**
  - richer CSS variable export (FigCraft already exports JSON, CSS in px, Tailwind and Dart, and
    imports JSON; Plus adds `oklch()`, `rem`, relative colours, `calc()`, and CSS import);
  - bulk editing and binding of text styles; remembered missing-font replacements (0.13 downloads
    missing fonts but doesn't remember replacements).
  - Idea: write simple alpha / sRGB mixes as FigCraft 0.13's native `ColorAlpha` / `ColorMix`
    variable values instead of a description formula.
- **Stage 3 (done in Plus 11):** Design System Inspector.
- **After 0.14:** %, vw, vh; auto layout from variables; line height % tokens; gradient tokens.
- **Known gaps:**
  - Style dialogs keep their own closing rules (an outside click must not lose a style being
    created).
  - Some effect fields (noise, texture), grid tracks and prototype offsets stay in px.
- **Parked:**
  - Branch `support` holds a "Support FigCraft Plus" section and `.github/FUNDING.yml` with a
    `KOFI_NAME` placeholder. It waits for the owner's GitHub Sponsors approval or Ko-fi name.
  - A duplicated component outside a frame doesn't create an instance — an upstream bug; re-check
    on FigCraft 0.13 before touching it.
  - Repository topics and the social preview must be set by the owner in the GitHub UI.
