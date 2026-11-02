# Changelog

Every FigCraft Plus release, newest first. Each one is built on an official FigCraft release
(named in the heading); downloads are under [Releases](../../releases). What comes next is in the
[roadmap](ROADMAP.md).

**🟣 Exclusive** — FigCraft has nothing like it yet. **🔵 Enhanced** — FigCraft has a basic
version; Plus goes further.

## Plus 13 · FigCraft v0.14.0 — 2026-11-02

- Fixed: the colour picker opened from a colour style's swatch covered the style dialog; it opens
  beside the dialog again.
- Fixed: in a narrow colour picker (Hex, CSS) the two contrast samples cut their captions ("✓ Reads
  better"); there they now stand one under the other.
- README: every screenshot and demo retaken on FigCraft 0.14.

## Plus 12 · FigCraft v0.14.0 — 2026-11-01

Every add-on carried over to FigCraft 0.14.0 (nothing it added duplicates Plus; the CSS export now
uses FigCraft's unique custom property names). Roadmap stage 4:

- 🟣 **Sizes in %, vw and vh.** A width or height can be a share of the parent (`50%`) or of the
  frame the layer is in (`100vw`, `100vh`), kept in step when they change. Pick it in the W / H
  sizing menu or type `100vw` / `50vh`; a size typed by hand ends it.
- 🟣 **Auto layout from variables.** Direction, wrap, main and cross alignment and the W / H
  sizing follow string or boolean variables — the variable button in the direction row. A mode
  switches the layout.
- 🔵 **Align to a key object.** With several layers selected, a click on one makes it the key
  (thicker outline): the align buttons move the others to it. Click it again to select just it;
  Preferences can turn it off.

## Plus 11 · FigCraft v0.13.1 — 2026-11-01

Roadmap stages 2 and 3.

- 🟣 **Design System Inspector** — *Inspect* in the Variables editor: values typed by hand that a
  variable holds (fills, strokes, radii, gaps, paddings, stroke weights, font sizes), broken
  references, circular formulas, unused, unscoped and duplicate variables, invalid values. Every
  finding jumps to its layer or variable; *Fix* / *Fix all* bind values and delete unused
  variables, one undo step.
- 🔵 **CSS for code** (Export ▸ CSS for code…): colours as `oklch()`, sizes in `rem`, derived
  colours as relative colours, mixes as `color-mix()`, calculated numbers as `calc()`.
- 🔵 **Import CSS custom properties** (Import ▸ CSS custom properties…): `:root` / `@theme` and
  theme blocks as modes; colours, lengths, numbers and font stacks; `var()` as aliases; relative
  colours, `color-mix()` and `calc()` as calculated variables.
- 🟣 **Text styles together:** select several text styles ▸ *Edit N text styles…* — set font,
  size, line height, letter spacing for all; bind variables to all of them by name, previewed.
- 🔵 **Missing fonts: Replace…** with an installed font (every run and text style), remembered for
  the next file that uses it; Preferences lists and forgets them.
- Fixed: *Units settings…* in a field's unit menu could freeze the app.

## Plus 10 · FigCraft v0.13.1 — 2026-10-31

- 🔵 **The colour picker follows the selection again.** Since FigCraft 0.13 a press outside the
  picker closed it, so clicking another layer on the canvas or in Layers closed it too. Now a
  click there moves it to the same fill, stroke or effect of the newly selected layer, as before;
  a click that selects nothing new still closes it, and the Design panel closes it as FigCraft's
  does. **Preferences ▸ FigCraft Plus ▸ Picker follows the selection** — on by default; off, the
  picker behaves exactly as FigCraft's.

## Plus 9 · FigCraft v0.13.1 — 2026-10-31

Every add-on carried over to FigCraft 0.13.1, and what FigCraft 0.13 now does itself was removed:
its new popover — one card with a header, close button and shadow, which closes on Escape and on
a press outside — now draws the colour picker, the variable list, the Units settings and the style
dialogs, in place of Plus's own; the picker's placement beside the Design panel and FigCraft's dark
style menu are FigCraft's again. Plus keeps what FigCraft doesn't do: the picker follows the
selection to the same slot of other layers, and opens beside a style dialog rather than over it.

- 🟣 **Rulers in the document's unit.** In mm, cm, in, pt or rem the rulers tick on round numbers
  of that unit (every 10 mm, every 0.5 in…), and guides show their position in it.
- 🔵 **Dev Mode in two units.** The properties it lists — size, position, radius, gap, padding,
  stroke, font size, line height, spacing — show the document's units, and copy as shown. The code
  it generates keeps its own unit, px or rem (of the file's root size), stored in the file: a CSS
  file in millimetres never happens by accident.
- 🟣 **Number variables with a unit.** A number variable used for sizes, spacing, radius or
  strokes shows (and takes) the layout unit in the Variables editor; one used for type, the text
  unit; opacity, weight and numbers nothing tells about stay bare. What it measures comes from the
  fields bound to it, else its scopes. The variable list names numbers in the unit of the field
  they would bind to. The stored value never changes.
- 🟣 **em for line height and letter spacing:** `1.5em` is 150%, `0.02em` is 2% — relative to the
  font size, as in CSS, and stored as FigCraft's percentage.
- 🟣 **Contrast both ways.** Under the picker's fields, two `Aa` samples: the colour on its
  background, and the background's colour on it, each with the WCAG ratio and level and the APCA
  Lc. WCAG gives the same ratio both ways, APCA doesn't, so the pairing that reads better is marked.

## Plus 8 · FigCraft v0.12.1 — 2026-10-30

- 🟣 **Switch units from any field.** A length field names its unit right after its value: click
  it — or right-click any length field, also in px, where fields stay exactly as FigCraft draws
  them — and pick a unit: it becomes the **document's** unit (layout or text), never a single
  property's. *Units settings…* at the end of the list. Font size now works like every other text
  length.
- 🔵 **Popovers close the same way:** their close button, Escape, a click outside and a change of
  selection — the variable list, the style menu, Units, and the colour picker (which still moves to
  the same slot of a newly selected layer). Compact menus stay compact.
- 🟣 **Decimals shown** (0–4) in Preferences ▸ FigCraft Plus. Only what fields show is rounded;
  values keep every digit.

## Plus 7 · FigCraft v0.12.1 — 2026-10-30

- 🟣 **Units everywhere.** Each file picks the unit its fields show: one for **layout** — px,
  rem, mm, cm, in or pt — and one for **text** — px, rem or pt — with its own root size. Position,
  size, min / max, radius, gap and padding, strokes, shadows and blurs, layout grids, vector
  points, the spacing of selected layers, the selection's size on the canvas and the measurements
  all show the layout unit; font size, line height, letter and paragraph spacing, indent and list
  spacing the text unit. Any field still takes any unit typed by hand (`210mm`, `1.5rem`).
  Set it in **Units**: the gear in the Layout or Typography header, or the Units row of the Page
  section; defaults in Preferences ▸ FigCraft Plus.
- 🔵 **One popover style** across the Design panel: the colour picker, the variable list, the
  style menu (now light in the light theme), the Units settings and the style dialogs share one
  card — border, radius, shadow, a header with the title and a close button, a divider under it.

## Plus 6 · FigCraft v0.12.1 — 2026-10-30

- 🔵 **Libraries tab in the colour picker.** Beside *Custom*: this file's and the enabled
  libraries' colour variables, with search, collections and groups. Only variables whose scope
  fits what you edit are listed (a stroke offers stroke colours, a frame's fill frame-fill
  colours, a shadow effect colours). Works for fills, strokes, effect colours, colour styles and
  variable values (as an alias).
- 🟣 **Contrast check sees the real background.** WCAG and APCA are measured against the layers
  painted under the layer — a button on a coloured rectangle is checked against the rectangle,
  not the page. Half-transparent layers are blended; gradients count by their average.
- The variable picker offers, for several selected layers, only variables scoped for all of them.
- README: hero, *Why FigCraft Plus*, animated demos, before / after, compatibility, community,
  FAQ. The build now stops if an add-on would touch FigCraft's file format.

## Plus 5 · FigCraft v0.12.1 — 2026-10-30

- The Typography settings popover (the gear) opens reliably, with a solid background, beside the
  properties panel and always inside the window.
- Screenshots of every new setting in the README.

## Plus 4 · FigCraft v0.12.1 — 2026-10-30

- 🟣 **Typography settings** (the gear in Typography): this file's unit — px or rem — and root
  size, as asked in [storytold/figcraft#1325](https://github.com/storytold/figcraft/issues/1325).
- 🟣 **Preferences ▸ FigCraft Plus:** the typography unit and root size for new files, and the
  model the colour picker opens in (Hex, RGB, HSL, HSB, LCH, OKLCH, CMYK or CSS).

## Plus 3 · FigCraft v0.12.1 — 2026-10-30

- 🟣 **Print units** in every number field: `mm`, `cm`, `in`, `pt`, mixed with arithmetic
  (`210mm` × `297mm` is an exact A4 page).
- 🟣 **CMYK** in the colour picker (converted as FigCraft's CMYK PDF export converts) and
  **Copy as**: the colour as Hex, `rgb()`, `hsl()`, HSB, `lch()`, `oklch()`, `device-cmyk()` or
  CSS.
- 🟣 **Contrast check** under the picker's fields: WCAG 2 ratio and level, APCA Lc.

## Plus 2 · FigCraft v0.12.1 — 2026-10-29

- 🟣 **Derived colours and calculated variables** — `oklch(from {Brand/Primary} …)`,
  `color-mix()`, `calc()` between number variables, kept in step in every mode — as asked in
  [storytold/figcraft#1280](https://github.com/storytold/figcraft/issues/1280).
- 🟣 **rem units** with a root size per file, as asked in
  [storytold/figcraft#1325](https://github.com/storytold/figcraft/issues/1325).

## Plus 1 · FigCraft v0.12.1

The first release of FigCraft Plus on FigCraft 0.12: everything FigCraft 0.12 now does itself was
removed (OKLCH in the model menu, string variables for fonts, scoped colour variables in the style
picker, smart-selection spacing), and the rest carried over:

- 🔵 OKLCH the oklch.com way: an OKLCH plane with the exact gamut edge, an OKLCH hue strip, values
  that match oklch.com and CSS, Display P3 files read in P3.
- 🟣 Colour channels driven by number variables.
- 🔵 A picker that opens beside what you edit and follows the selection; captions on every field
  and a resizable width.
- 🔵 Variables in text: weight, paragraph spacing and indent.
- 🔵 Styles created and edited in a dialog with a live preview; nested style groups.
- 🔵 A richer variable picker; *Edit variable* on every row; 🟣 variable reordering and sorting.
- 🟣 A portable `fonts` folder next to the app.
