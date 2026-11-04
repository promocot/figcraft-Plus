<p align="center">
  <img src="docs/images/hero.png" width="100%" alt="FigCraft Plus — professional color and design-system tools for the FigCraft design editor">
</p>

<p align="center">
  <a href="../../releases/latest"><img alt="Download for Windows" src="https://img.shields.io/badge/Download_for_Windows-portable_zip-1fbf75?style=for-the-badge"></a>
  &nbsp;
  <a href="ROADMAP.md"><img alt="Roadmap" src="https://img.shields.io/badge/Roadmap-what's_next-7c5cff?style=for-the-badge"></a>
  &nbsp;
  <a href="../../issues/new?template=feature_request.yml"><img alt="Request a feature" src="https://img.shields.io/badge/Request_a_feature-%F0%9F%91%8D-3a3f4b?style=for-the-badge"></a>
</p>

<p align="center">
  <b>FigCraft Plus 15</b> · built on FigCraft <b>v0.15.0</b>, the latest official release ·
  <code>.figcraft</code> files stay compatible with the official app, both ways
</p>

**FigCraft Plus turns [FigCraft](https://github.com/storytold/figcraft) into a stronger tool for
color and design systems.** It is applied to every official FigCraft release and built for
Windows automatically: you get the editor you know, with professional tools on top — and nothing
to migrate.

It only adds what FigCraft doesn't have yet. When an official release ships a feature of its own,
Plus steps aside and uses it ([what has moved into FigCraft](#now-in-figcraft-itself-removed-from-plus)):
the goal is a better FigCraft, not a different one.

## Why FigCraft Plus

<table>
<tr>
<td width="33%" valign="top">

### 🎨 Advanced color

An **OKLCH picker** that shows the real gamut, **CMYK** fields, colors written as **CSS
formulas**, *Copy as* any format, gradients **blended in OKLCH**, a **WCAG + APCA contrast check**
against what is really under the layer, and print-ready **PDF/X-4** at each frame's **DPI**.

</td>
<td width="33%" valign="top">

### 🧩 Stronger design systems

**Calculated variables** and colors **derived** from others (`oklch(from …)`, `color-mix()`,
`calc()`), channels driven by number variables, **units** (px, rem, mm, cm, in, pt) in every field, style dialogs and
groups, scoped variable pickers, variable ordering.

</td>
<td width="33%" valign="top">

### 🔁 Native file compatibility

Plus never touches FigCraft's file format — the build stops if a change would. Formulas and
settings are kept as text the official app ignores, so files open there with every color and size
as you left it.

</td>
</tr>
</table>

## See it in action

<table>
<tr>
<td width="50%" valign="top">

**OKLCH picker** — lightness and hue move evenly; the plane shows the exact gamut edge.

<img src="docs/images/demo-oklch.gif" width="100%" alt="Changing lightness, then hue, in the OKLCH picker: the plane and the swatch follow">

</td>
<td width="50%" valign="top">

**Calculated variables** — change one number, the whole palette follows: hover, transparency,
tint and complement are formulas of `primary`.

<img src="docs/images/demo-calculated-variables.gif" width="100%" alt="brand/hue goes from 280 to 150 to 25: primary, hover, soft, tint and accent update together">

</td>
</tr>
<tr>
<td width="50%" valign="top">

**Contrast that sees the background** — the same white button on three backgrounds: WCAG ratio,
level and APCA Lc update as it moves.

<img src="docs/images/demo-contrast.gif" width="100%" alt="A white button moved across sand, berry and navy rectangles: 1.21:1 Fail, 6.46:1 AA, AAA">

</td>
<td width="50%" valign="top">

**Libraries in the picker** — search the colour variables and apply one; only those scoped to
what you edit are offered.

<img src="docs/images/demo-libraries.gif" width="60%" alt="The Libraries tab: typing prm, then clicking --prm-4 colors the button">

</td>
</tr>
</table>

**Before / after** — the same color in FigCraft's picker and in FigCraft Plus:

<img src="docs/images/before-after-picker.png" width="720" alt="Left: FigCraft's HSB square and rainbow. Right: FigCraft Plus's OKLCH plane with the exact gamut edge and an OKLCH hue strip">

> ### 🔁 Compatibility first
>
> FigCraft Plus extends the editor without a file format of its own. A `.figcraft` file saved by
> Plus is written by FigCraft's own, unchanged code — the build checks that no add-on touches the
> document model or file format, and stops if one would. What Plus adds (formulas, rem settings,
> style groups) is stored as text inside the file, which the official app keeps and ignores: it
> opens the file and shows the last computed colors and sizes. Nothing to convert, in either
> direction.

> ### 🗺 Roadmap — what's next
>
> **Just done:** export presets and design tokens (DTCG) in Plus 15; line height in % from a
> variable, gradients blended in OKLCH, **a resolution (DPI) per frame** and PDF/X-4 in Plus 14.
>
> **Next:** your feedback — try the features with the [**guide**](docs/GUIDE.md) and
> [report](../../issues/new/choose) what doesn't work or could be easier.
>
> **[See the full roadmap →](ROADMAP.md)** · **[Ask for a feature →](../../issues/new?template=feature_request.yml)**
> — the most 👍-ed requests move up.

## Contents

- [Why FigCraft Plus](#why-figcraft-plus) · [See it in action](#see-it-in-action) · [Roadmap](ROADMAP.md) · [Changelog](CHANGELOG.md)
- **[Guide: every feature step by step](docs/GUIDE.md)** · [Reporting bugs](#reporting-bugs)
- [Download and update](#download-and-update)
- [Feature comparison](#feature-comparison)
- [Requested in FigCraft's issues, made here](#requested-in-figcrafts-issues-made-here)
- [Now in FigCraft itself (removed from Plus)](#now-in-figcraft-itself-removed-from-plus)
- [Add-ons](#add-ons)
  - [OKLCH, done the oklch.com way](#oklch-done-the-oklchcom-way)
  - [Derived colours and calculated variables](#derived-colours-and-calculated-variables)
  - [Units everywhere: px, rem, mm, cm, in, pt](#units-everywhere-px-rem-mm-cm-in-pt)
  - [CMYK and Copy as in the colour picker](#cmyk-and-copy-as-in-the-colour-picker)
  - [Contrast both ways: WCAG and APCA](#contrast-both-ways-wcag-and-apca)
  - [Colour variables in the picker: the Libraries tab](#colour-variables-in-the-picker-the-libraries-tab)
  - [FigCraft Plus preferences](#figcraft-plus-preferences)
  - [A better picker](#a-better-picker)
  - [A picker that stays with what you're editing](#a-picker-that-stays-with-what-youre-editing)
  - [Colours driven by number variables](#colours-driven-by-number-variables)
  - [Variables in text: weight, paragraph spacing and indent](#variables-in-text-weight-paragraph-spacing-and-indent)
  - [Your own fonts, travelling with the app](#your-own-fonts-travelling-with-the-app)
  - [Styles: create and edit them in a dialog](#styles-create-and-edit-them-in-a-dialog)
  - [Style groups in Local styles](#style-groups-in-local-styles)
  - [Variable details & scope, one click away](#variable-details--scope-one-click-away)
  - [A richer variable picker](#a-richer-variable-picker)
  - [Reorder variables: drag & drop and sorting](#reorder-variables-drag--drop-and-sorting)
  - [Responsive layout: %, vw, vh and layout from variables](#responsive-layout-vw-vh-and-layout-from-variables)
  - [Align to a key object](#align-to-a-key-object)
  - [Design System Inspector](#design-system-inspector)
  - [Variables as CSS for code, and from CSS](#variables-as-css-for-code-and-from-css)
  - [Text styles together](#text-styles-together)
  - [Missing fonts: replace, and remember](#missing-fonts-replace-and-remember)
  - [Line height in % from a variable](#line-height-in--from-a-variable)
  - [Gradient tokens: gradients blended in OKLCH](#gradient-tokens-gradients-blended-in-oklch)
  - [Resolution (DPI) per frame](#resolution-dpi-per-frame)
  - [PDF/X-4 for print](#pdfx-4-for-print)
  - [Export presets](#export-presets)
  - [Design tokens (DTCG)](#design-tokens-dtcg)
- [Reporting bugs](#reporting-bugs)
- [Community](#community)
- [FAQ](#faq)
- [How it works](#how-it-works)
  - [Run a build by hand](#run-a-build-by-hand)
  - [Build locally](#build-locally)
  - [When FigCraft adds a feature of its own](#when-figcraft-adds-a-feature-of-its-own)
  - [Add a feature](#add-a-feature)
- [License](#license)

## Feature comparison

**🟣 Exclusive** — FigCraft has nothing like it yet. **🔵 Enhanced** — FigCraft has a basic
version; Plus goes further. Features FigCraft has since added itself are
[listed separately](#now-in-figcraft-itself-removed-from-plus) — they are no longer part of Plus.

| Feature | In FigCraft v0.15.0 | In FigCraft Plus | |
|---|---|---|---|
| [OKLCH, done the oklch.com way](#oklch-done-the-oklchcom-way) | FigCraft has OKLCH fields: lightness in percent, sRGB only, and the HSB square and rainbow stay HSB. | The whole picker works in OKLCH (an OKLCH plane with the exact gamut edge, an OKLCH hue strip), values match oklch.com and CSS, and Display P3 files are read in P3. | 🔵 Enhanced |
| [Derived colours and calculated variables](#derived-colours-and-calculated-variables) | A variable can be an *alias* of another — an exact copy, nothing more. | A colour can be *made from* another (transparency, lighter / darker, its complement, a mix of two), and a number from other numbers, all kept in step in every mode. | 🟣 Exclusive |
| [Units everywhere: px, rem, mm, cm, in, pt](#units-everywhere-px-rem-mm-cm-in-pt) | Everything is shown and typed in px; mm, cm and in only in the PDF export's settings; Dev Mode rem uses a fixed 16. | Each file picks a **layout** unit (px, rem, mm, cm, in, pt) and a **text** unit (px, rem, pt) with its own root size; every field, the rulers, Dev Mode and number variables show it, and any field takes any unit (`210mm`, `1.5rem`, `1.5em`, `10mm + 2`). | 🟣 Exclusive |
| [CMYK and Copy as in the colour picker](#cmyk-and-copy-as-in-the-colour-picker) | The picker offers Hex, RGB, HSL, HSB, LCH, OKLCH and CSS; CMYK exists only as a PDF export option. | **CMYK** fields in the picker, and **Copy as** — the colour in any format, one click. | 🟣 Exclusive |
| [Contrast both ways: WCAG and APCA](#contrast-both-ways-wcag-and-apca) | No contrast check in the interface. | Under the picker's fields, two `Aa` samples — the colour on what is really under the layer, and that background on the colour — each with the WCAG ratio and level and the APCA Lc; the better pairing is marked. | 🟣 Exclusive |
| [Colour variables in the picker: the Libraries tab](#colour-variables-in-the-picker-the-libraries-tab) | Colour variables are applied from the style menu next to Fill or Stroke; the picker itself only edits a colour. | A **Libraries** tab beside **Custom** in the colour picker: this file's and the enabled libraries' colour variables, with search — only those whose scope fits what you edit (a stroke offers stroke colours). | 🔵 Enhanced |
| [FigCraft Plus preferences](#figcraft-plus-preferences) | The colour picker always opens in Hex; there are no units to choose. | Preferences ▸ FigCraft Plus: the layout and text units and root size for new files, decimals shown, the model the colour picker opens in, whether the picker follows the selection, the key object, the resolution of new frames. | 🟣 Exclusive |
| [A better picker](#a-better-picker) | Fields without captions, one fixed width. | A caption over every field, a width that fits the model, and a draggable edge. | 🔵 Enhanced |
| [A picker that stays with what you're editing](#a-picker-that-stays-with-what-youre-editing) | The picker opens beside the Design panel and closes when the selection changes. | It moves to the same slot of the newly selected layer, so you keep editing without reopening it; opened from a style dialog, it opens beside the dialog. | 🔵 Enhanced |
| [Colours driven by number variables](#colours-driven-by-number-variables) | A colour can be bound to a colour variable as a whole. | Each channel (L, C, H, R, G, B, H, S, alpha…) can follow its own number variable: one `brand/hue` restyles every colour built on it. | 🟣 Exclusive |
| [Variables in text: weight, paragraph spacing and indent](#variables-in-text-weight-paragraph-spacing-and-indent) | Font family and font style take string variables. | The weight takes a number variable too (400, 700), and paragraph spacing and indent take number variables. | 🔵 Enhanced |
| [Your own fonts, travelling with the app](#your-own-fonts-travelling-with-the-app) | Fonts come from the system (and Adobe Fonts / font managers on macOS). | A `fonts` folder next to `figcraft.exe` — your fonts travel on the USB drive — with *Add fonts…*, an *In this file* section and *Added* tags. | 🟣 Exclusive |
| [Styles: create and edit them in a dialog](#styles-create-and-edit-them-in-a-dialog) | Styles are created from the selection and edited in FigCraft's style popover. | A **+** on Local styles creates any kind of style from scratch in a dialog with a live preview, with variables in effect, grid and colour fields too. | 🔵 Enhanced |
| [Style groups in Local styles](#style-groups-in-local-styles) | Local styles are grouped by the folders in their names. | Real nested groups: Ctrl+G, New group, drag styles and groups, collapse, rename, ungroup. | 🔵 Enhanced |
| [Variable details & scope, one click away](#variable-details--scope-one-click-away) | Right-click a variable ▸ Edit variable. | An *Edit variable* button on every row, opening beside the editor and following it. | 🔵 Enhanced |
| [A richer variable picker](#a-richer-variable-picker) | A plain list of the variables that fit. | One card everywhere: search, library filter, collections and groups, swatches and values, arrow keys and Enter. | 🔵 Enhanced |
| [Responsive layout: %, vw, vh and layout from variables](#responsive-layout-vw-vh-and-layout-from-variables) | Sizes are px, Hug or Fill; auto layout settings are fixed per frame. | **50%** of the parent, **100vw / 100vh** of the frame it sits in, kept in step; direction, wrap, alignment and sizing **from variables**, so modes switch them. | 🟣 Exclusive |
| [Align to a key object](#align-to-a-key-object) | Several layers align to their combined box. | Click one of the selection: it becomes the key object, the others align to it and it stays put. | 🔵 Enhanced |
| [Design System Inspector](#design-system-inspector) | No check of the design system as a whole. | One list of what keeps it from holding together — values typed by hand that a variable holds, broken references, circular formulas, unused, unscoped and duplicate variables — each jumping to its layer or variable, most with a one-click fix. | 🟣 Exclusive |
| [Variables as CSS for code, and from CSS](#variables-as-css-for-code-and-from-css) | Variables export as CSS in hex and px (formulas flattened) and import from JSON. | **CSS for code:** `oklch()`, `rem`, relative colours, `color-mix()`, `calc()`; and **import CSS custom properties** — `:root` and theme blocks as modes. | 🔵 Enhanced |
| [Text styles together](#text-styles-together) | Text styles are edited one at a time. | Select several: set font, size, line height, letter spacing for all; bind variables to all of them by name, previewed, one undo step. | 🟣 Exclusive |
| [Missing fonts: replace, and remember](#missing-fonts-replace-and-remember) | Missing fonts are listed and the font store's ones downloaded. | **Replace…** with an installed font — every run and text style — remembered for the next file that uses it. | 🔵 Enhanced |
| [Line height in % from a variable](#line-height-in--from-a-variable) | A number variable bound to the line height is pixels: one value for every text size. | The line height can follow a number variable **as a percentage** — `leading/normal = 140%` (or `1.4`) suits every size, as CSS writes it; layers and text styles, in every mode. | 🔵 Enhanced |
| [Gradient tokens: gradients blended in OKLCH](#gradient-tokens-gradients-blended-in-oklch) | Gradient stops take colour variables; gradients mix their stops in sRGB (blue → yellow passes through grey). | **Blend · OKLCH** in the picker: the gradient — a layer's or a paint style's — is mixed in OKLCH between its stops and kept in step with their variables, as CSS's `linear-gradient(in oklch, …)`. | 🔵 Enhanced |
| [Resolution (DPI) per frame](#resolution-dpi-per-frame) | A pixel is a point (72 per inch) everywhere. | Each top-level frame has its own **DPI**: 72 → 300 scales it and its contents so its size in mm stays, and moves what is beside it; mm, cm, in and pt follow it; PNG / JPG say their DPI and a PDF page keeps the physical size. | 🟣 Exclusive |
| [PDF/X-4 for print](#pdfx-4-for-print) | A CMYK PDF with bleed, marks and page boxes, converted by formula, without an output intent. | **PDF/X-4** for coated FOGRA39 paper: the output intent with its ICC profile, colours converted through it, transparency in CMYK — what print shops' preflight asks for. | 🔵 Enhanced |
| [Export presets](#export-presets) | Each layer keeps its own export settings; files are named after the layer with a suffix. | **Presets…**: several sizes and formats for every selected layer at once, files named by a pattern (`{name}_{w}x{h}`, folders with `/`); presets saved and edited in Preferences. | 🔵 Enhanced |
| [Design tokens (DTCG)](#design-tokens-dtcg) | Variables export as JSON, CSS, Tailwind, Dart and import from FigCraft's JSON. | **DTCG JSON** both ways — the W3C Design Tokens format other tools read: `$type`, `$value`, aliases, every mode. | 🟣 Exclusive |
| [Reorder variables: drag & drop and sorting](#reorder-variables-drag--drop-and-sorting) | Modes can be reordered; variables keep the order they were made in. | Drag variables, groups and collections, multi-select, Alt+↑/↓, and natural or by-value sorting — also as commands for scripts and MCP. | 🟣 Exclusive |

Everything stays a normal `.figcraft` file: the official app opens it and sees the last computed
colours and sizes; formulas and settings are kept as text it ignores.

## Download and update

1. Open **[Releases](../../releases/latest)** and take
   `figcraft-plus-<n>-figcraft-<version>-windows-x64-portable.zip`.
2. Unzip it anywhere (a USB drive works) and run `figcraft.exe`. Nothing is installed.
3. The build isn't code-signed: if Windows SmartScreen warns, choose **More info → Run anyway**.

**Updating** to a newer Plus: unzip the new release over the old folder (or into a new one and
copy your `fonts` folder and `figcraft-plus.json` across). Your `.figcraft` files live wherever
you saved them and need nothing — every Plus version, and the official app, opens them.

To hear about new builds, use **Watch → Custom → Releases** on this page; what changed is in the
[changelog](CHANGELOG.md).

The build is FigCraft from the same release tag, built the same way (release profile, static C
runtime, embedded fonts, `figcraft-cli.exe` included), plus the add-ons below.

## Requested in FigCraft's issues, made here

Features FigCraft users asked for in the official tracker that FigCraft itself hasn't taken up:

| Request | In FigCraft Plus |
|---|---|
| [storytold/figcraft#1280 — Variable Referencing](https://github.com/storytold/figcraft/issues/1280) | 2 — [derived colours and calculated variables](#derived-colours-and-calculated-variables) |
| [storytold/figcraft#1325 — REM as a typography input unit, with a configurable root size](https://github.com/storytold/figcraft/issues/1325) | 2 — [rem units](#units-everywhere-px-rem-mm-cm-in-pt); 7 — every unit, everywhere |

## Now in FigCraft itself (removed from Plus)

| Was in Plus | In FigCraft since |
|---|---|
| Spacing field for several layers in a row or column | 0.7 — smart-selection spacing, with handles on the canvas |
| OKLCH in the colour picker's model menu | 0.12 — LCH and OKLCH models |
| Font family and font style bound to string variables | 0.12 |
| Colour variables offered by fill / stroke scope in the style picker | 0.12 |
| One popover style (header, close button, shadow), closed by Escape and a press outside | 0.13 — FigCraft's own popover, which Plus's popovers now use |
| The colour picker opening beside the Design panel, at the swatch's height | 0.13 |
| The style menu restyled light in the light theme | 0.13 keeps its dark style menu; Plus no longer recolours it |
| Google Fonts download | removed earlier (fonts folder instead) |

## Add-ons

### OKLCH, done the oklch.com way

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** FigCraft has OKLCH fields: lightness in percent, sRGB only, and the HSB square and rainbow stay HSB.
>
> **FigCraft Plus:** The whole picker works in OKLCH (an OKLCH plane with the exact gamut edge, an OKLCH hue strip), values match oklch.com and CSS, and Display P3 files are read in P3.

<img src="docs/images/02-oklch-picker.png" width="330" alt="The picker in OKLCH: the OKLCH plane, the OKLCH hue strip and the L, C, H, A fields">

| Field | Meaning | Range |
|---|---|---|
| **L** | lightness | 0–1 |
| **C** | chroma | 0–0.4 |
| **H** | hue | degrees |

- Same units and precision as [oklch.com](https://oklch.com) and CSS `oklch(L C H)`: paste a hex
  copied from FigCraft into oklch.com and you get the same numbers. Fields also take CSS forms
  (`64.56%` for L, `55%` for C = 0.22).
- Values are read and written in the **file's colour profile** (sRGB or Display P3,
  File ▸ Color profile). Colours outside the gamut are mapped in with the CSS Color 4 algorithm
  (chroma reduced, lightness and hue kept).
- The whole picker works in OKLCH: the HSB square becomes an **OKLCH plane** (chroma across,
  lightness down, at the current hue) with the exact gamut edge — and, in a Display P3 file, the
  sRGB edge dashed — and the rainbow becomes an **OKLCH hue strip** at the current lightness and
  chroma, so dragging the hue keeps the lightness.
- Verified against the [coloraide](https://github.com/facelessuser/coloraide) reference
  implementation (CSS Color 4) on 6,000 colours: largest difference 0.000001.

### Derived colours and calculated variables

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** A variable can be an *alias* of another — an exact copy, nothing more.
>
> **FigCraft Plus:** A colour can be *made from* another (transparency, lighter / darker, its complement, a mix of two), and a number from other numbers, all kept in step in every mode.

*Requested in [storytold/figcraft#1280](https://github.com/storytold/figcraft/issues/1280).*

A colour can be made from another one, in CSS's own syntax, and stays in step with it — in every
mode (Light / Dark), and down a chain (a hover from a faded colour from the brand colour):

| Typed | Gives |
|---|---|
| `oklch(from {Brand/Primary} l c h / 50%)` | the brand colour, half transparent |
| `oklch(from {Brand/Primary} calc(l - 0.08) c h)` | a darker hover |
| `oklch(from {Brand/Primary} l c calc(h + 180))` | its complement |
| `color-mix(in oklch, {Brand/Primary} 20%, {Surface})` | a tint of the surface |
| `hsl(from #3A81F6 h s calc(l * {Shade}))` | a fixed colour, shaded by a number variable |

- **Type it into the colour picker's CSS field** (the model menu's last entry) — for a colour
  variable, a style's paint or a layer's fill, stroke, gradient stop or effect. The picker shows
  the formula under the fields, with a button that detaches it (the colour stays as it is).
- `from` takes a colour variable `{name}` or a hex colour; the model's keywords (`l c h`, `h s l`,
  `h s b`, `r g b`, `alpha`) are the base colour's channels, in the picker's units, and work in
  `calc()` with numbers and number variables. Without `/ alpha` a derived colour keeps its base's
  alpha.
- `color-mix` mixes in `oklch`, `hsl`, `hsb` or `srgb`, hues the shorter way round; editing a mixed
  colour by hand detaches it.

**Number variables from other number variables:** a line `calc({Space/Base} * 2)` in a number
variable's description keeps its value at twice `Space/Base`, in every mode. Editing the value by
hand detaches it; variables that read each other in a loop are left as they are.

The formulas live in the descriptions (and layer data) as text, so the official FigCraft opens
the files and keeps the last computed values.

### Units everywhere: px, rem, mm, cm, in, pt

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** Everything is shown and typed in px; mm, cm and in exist only in the PDF export's print settings, and Dev Mode converts px → rem with a fixed 16.
>
> **FigCraft Plus:** Each file chooses the unit its fields show — one for **layout** (px, rem, mm, cm, in, pt) and one for **text** (px, rem, pt) — with its own root size. Every number field shows and takes it, and any field takes any unit typed by hand.

*rem requested in [storytold/figcraft#1325](https://github.com/storytold/figcraft/issues/1325).*

<img src="docs/images/36-units.png" width="720" alt="The Units popover: Layout mm, Text pt, root size 16px; the Design panel shows X, Y, W, H and the corner radius in mm, and the canvas label 170mm × 80mm">

**Where to set it**

| What | Where |
|---|---|
| This file's units and root size | **Units** — the gear in the **Layout** or **Typography** header, or the **Units** row of the Page section (nothing selected) |
| The units and root size for new files | **Preferences ▸ FigCraft Plus** ▸ *Layout unit*, *Text unit*, *Default root size* |

- **Layout unit** — position, width and height, min / max, corner radius, gap and padding, stroke
  weights and dashes, shadow offsets, blur and spread, layout grid sizes, vector points, the
  spacing between selected layers; also the selection's size on the canvas and the measurements.
  Lay out a print page in **mm**, a web page in **rem**.
- **Text unit** — font size, line height, letter spacing, paragraph spacing and indent, list
  spacing: **pt** for print, **rem** for the web.
- **Switch units from any field:** a length field names its unit right after its value — click
  it — or **right-click** any length field (the way in px, where fields look and behave exactly as
  FigCraft's). The list switches the **document's** unit — layout or text — and *Units settings…*
  opens the rest. One unit per document, never one per property: the file stays plain px.

  <img src="docs/images/38-unit-menu.png" width="230" alt="The W field's unit, mm, clicked: px, rem, mm (checked), cm, in, pt and Units settings…">
- **Any field takes any unit** typed with the number, whatever it shows: `210mm`, `5.5cm`, `1in`,
  `12pt`, `1.5rem`, `24px`, and arithmetic across them (`10mm + 2`, `2rem + 4`).
- A px is a point (1/72 in), as FigCraft's own print export reads it: a frame of `210mm` × `297mm`
  exports as an exact A4 page, `85mm` × `55mm` as a business card. rem is the file's root size.
- Values are stored in px, as FigCraft stores them; the units and root size are kept in the file as
  text the official FigCraft ignores, so it opens the file as before (and shows px).
<img src="docs/images/40-rulers-dev-mode.png" width="720" alt="A document in mm: the rulers count 0, 20, 40 … mm, the measurements read 14.11mm and 125.33mm, Dev Mode lists Width 70.56mm and Height 28.22mm while its CSS says width: 200px">

- **Rulers** tick on round numbers of the layout unit (every 10 mm, every 0.5 in), and guides
  show their position in it.
- **Dev Mode in two units:** the properties it lists show the document's units (and copy as
  shown); the code it generates keeps its own unit — px or rem, of the file's root size, stored
  in the file — so a CSS file in millimetres never happens by accident.
- **Number variables** show the unit of what they size in the Variables editor: one bound to (or
  scoped for) sizes, spacing, radius or strokes the layout unit, one for type the text unit;
  opacity, weight and plain numbers stay bare. The variable list shows numbers in the unit of the
  field they would bind to. The stored value never changes.
  <img src="docs/images/41-number-variables.png" width="560" alt="The Variables editor: space/m (scoped to gap) reads 5.64mm, font/body (font size) 12pt, opacity/muted 0.6 and count 3 stay bare">
- **em** for line height and letter spacing: `1.5em` is 150%, `0.02em` is 2% — relative to the
  font size, as in CSS, stored as FigCraft's percentage.
- `%`, `vw` and `vh` in W / H: see [Responsive layout](#responsive-layout-vw-vh-and-layout-from-variables).
- A frame made for print reads mm, cm, in and pt at **its own resolution**: see
  [Resolution (DPI) per frame](#resolution-dpi-per-frame).

<img src="docs/images/31-print-units.png" width="508" alt="Typing 210mm in W; after Enter the frame is 595.28 × 841.89 — an A4 page">

### CMYK and Copy as in the colour picker

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** The picker offers Hex, RGB, HSL, HSB, LCH, OKLCH and CSS; CMYK exists only as a PDF export option.
>
> **FigCraft Plus:** **CMYK** fields in the picker, and **Copy as** — the colour in any format, one click.

<img src="docs/images/32-cmyk-contrast.png" width="328" alt="The picker in CMYK: C, M, Y, K fields, Copy as, and the contrast both ways — 3.61:1 AA Large, Lc 60 and Lc -65">

- **CMYK** in the model menu: C, M, Y and K in percent. They are converted exactly the way
  FigCraft's CMYK PDF export converts (`K = 1 − max(R, G, B)`), so what you see is what the PDF
  holds. (It's the profile-free formula: proof against your print shop's profile for production.)
- Want the picker to always open in CMYK (or OKLCH)? **Preferences ▸ FigCraft Plus ▸ Colour picker
  opens in**.
- **Copy as** under the fields: Hex, `rgb()`, `hsl()`, HSB, `lch()`, `oklch()` (oklch.com
  precision), `device-cmyk()` or CSS — a converter between colour spaces.

### Contrast both ways: WCAG and APCA

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** No contrast check in the interface.
>
> **FigCraft Plus:** Under the picker's fields, two `Aa` samples — the colour on what is really under the layer, and that background's colour on the colour — each with the WCAG 2 ratio and level and the APCA Lc. The pairing that reads better is marked.

<img src="docs/images/34-contrast-backdrop.png" width="257" alt="A white button on a berry rectangle: the picker measures 6.46:1 AA and Lc -86 against the berry, Lc 81 the other way">

<img src="docs/images/39-contrast-both-ways.png" width="362" alt="Under the picker's fields: white Aa on blue, 8.72:1 AAA, Lc -94, marked Reads better; blue Aa on white, 8.72:1 AAA, Lc 89">

- **The backdrop is what is really under the layer**, at its centre: any shape, frame or
  gradient behind it (a button on a coloured rectangle is checked against the rectangle, not the
  page), hidden and clipped-away layers left out. Half-transparent layers are blended in, down to
  the first opaque one and the page's background. Hover a sample to see the colour it found.
- **Both ways:** WCAG gives the same ratio whichever colour is the text; APCA doesn't — light text
  on dark reads differently from dark text on light — so the sample with the larger |Lc| is marked
  *Reads better*. A quick way to see whether swapping foreground and background would help.
- WCAG levels for normal text: **AAA** ≥ 7, **AA** ≥ 4.5, **AA Large** ≥ 3; below 4.5 the line is
  shown in the warning colour. APCA Lc follows APCA-W3 (black on white ≈ 106, white on black ≈ −108).

### Colour variables in the picker: the Libraries tab

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Colour variables are applied from the style menu next to Fill or Stroke; the picker itself only edits a colour.
>
> **FigCraft Plus:** A **Libraries** tab beside **Custom** in the colour picker: this file's and the enabled libraries' colour variables, with search — only those whose scope fits what you edit.

<img src="docs/images/35-picker-libraries.png" width="530" alt="Libraries for a fill: Primary and Blue colours; for a stroke of the same layer: only the Border colours">

- **Only variables whose scope fits** are listed: a fill lists the variables scoped to frame,
  shape or text fills (by what the layer is), a stroke those scoped to strokes, a shadow's colour
  those scoped to effects. Set a variable's scope in **Variables ▸ Edit variable**; with nothing
  fitting, the tab says so.
- Collections and groups as headers, the swatch and value of each colour, **search**, a library
  filter, arrow keys and Enter. The variable in use is checked; a click applies it.
- Works for a layer's fills, strokes and effect colours, a colour style's paint, and a colour
  variable's value in the Variables editor (it becomes an alias; variables that would loop are left
  out).
- The picker opens on the tab you used last.

### FigCraft Plus preferences

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** The colour picker always opens in Hex; there is no typography unit to choose.
>
> **FigCraft Plus:** **Preferences ▸ FigCraft Plus**: the layout and text units and the root size for new files, and the model the colour picker opens in.

<img src="docs/images/37-preferences.png" width="440" alt="Preferences ▸ FigCraft Plus: layout unit, text unit, default root size, decimals shown, the model the colour picker opens in, Picker follows the selection, Key object for aligning, Resolution of new frames 72 dpi">

| Setting | What it does |
|---|---|
| **Layout unit** — px, rem, mm, cm, in or pt | The unit layout fields show in files that don't set their own (Units) |
| **Text unit** — px, rem or pt | The unit text fields show in files that don't set their own |
| **Default root size** | 1rem in files that don't set their own |
| **Decimals shown** — 0 to 4 | How many decimals fields show; values keep every digit |
| **Colour picker opens in** — Hex, RGB, HSL, HSB, LCH, OKLCH, CMYK or CSS | The model the picker shows when it opens |
| **Picker follows the selection** — on by default | Clicking another layer keeps the colour picker open on the same fill, stroke or effect; off, it closes as FigCraft's |
| **Key object for aligning** — on by default | A click on one layer of a selection makes it the one the others align to |
| **Resolution of new frames** — 72 by default | The DPI new top-level frames are made at: 72 for the web, 300 for print |

They are saved in `figcraft-plus.json` next to `figcraft.exe`, so a copy on a USB drive keeps them.

### A better picker

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Fields without captions, one fixed width.
>
> **FigCraft Plus:** A caption over every field, a width that fits the model, and a draggable edge.

- **Captions over every field** in every model (R G B, H S L, H S B, L C H, A).
- **Width adapts** to the chosen model; drag the picker's **left edge** to make it wider
  (double-click the edge to reset).

### A picker that stays with what you're editing

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** The picker opens beside the Design panel at the swatch's height, and closes when the selection changes.
>
> **FigCraft Plus:** It moves to the same slot of the newly selected layer, so you keep editing without reopening it.

- **Select another layer** (click it on the canvas or in Layers) and it switches to the same slot
  of that layer — Fill 1 stays Fill 1, the second shadow stays the second shadow. If the layer has
  no such slot, or nothing is selected, it closes as FigCraft's does. A click that selects nothing
  new closes it; dragging (moving layers, gradient handles) leaves it open.
- **Not for you?** Turn off **Preferences ▸ FigCraft Plus ▸ Picker follows the selection**: the
  picker then closes on any press outside it and on every change of selection, exactly as
  FigCraft's.
- **Opened from a style dialog**, it opens beside the dialog instead of over it.
- Where it opens otherwise, Escape and a press outside closing it are FigCraft's own (since 0.13).

### Colours driven by number variables

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** A colour can be bound to a colour variable as a whole.
>
> **FigCraft Plus:** Each channel (L, C, H, R, G, B, H, S, alpha…) can follow its own number variable: one `brand/hue` restyles every colour built on it.

<p>
<img src="docs/images/05-bound-hue-280.png" width="440" alt="H bound to brand/hue = 280: a violet card">
<img src="docs/images/06-bound-hue-130.png" width="440" alt="The same card after brand/hue is set to 130: green">
</p>

*Left: H is bound to `brand/hue` = 280. Right: the variable set to 130 — the card follows.*

Every channel of a colour can be bound to a number variable, right in the picker — just like
FigCraft's own number fields. Hover a field in the **RGB**, **HSL**, **HSB** or **OKLCH** row
(R G B, H S L, H S B, L C H, and alpha **A**): the variable button appears at its right end. Pick
a number variable and the field shows its name; the detach button turns it back into a number.

Bind H to `brand/hue = 280`, change the variable to 130, and every colour linked to it follows —
layers, styles and variables — in one undo step.

**How to bind a channel**

1. Open the picker and choose RGB, HSL, HSB or OKLCH. Hover a field: the variable button appears.

   <img src="docs/images/03-variable-button.png" width="340" alt="The variable button on the H field">

2. Click it and pick a number variable (grouped by collection).

   <img src="docs/images/04-variable-list.png" width="400" alt="The number variable list">

3. The field now shows the variable's name. Alpha binds the same way — here H follows
   `brand/hue` and A follows `brand/fade` = 60%.

   <img src="docs/images/07-hue-and-alpha-bound.png" width="560" alt="H and alpha both bound">

4. To unbind, hover the name and click the detach button at its right; the field gets its
   current value back.

Switching to another model shows which channels are linked elsewhere; binding in the current
model replaces those links.

<img src="docs/images/08-linked-in-other-model.png" width="345" alt="The HSL row noting links made in OKLCH">

- Works wherever the picker edits a colour: layer fills and strokes, **gradient stops**, effect
  colours (shadows, noise), the **page background**, **colour styles** and **colour variables**.
  (Not yet: a range of characters inside a text layer, plugin colour fields.)
- Each model binds in its own terms: H in OKLCH is OKLCH hue, H in HSL is HSL hue. A colour is
  linked in one model at a time; the picker notes links made in another model, and binding in the
  current one replaces them.
- Units are the fields': OKLCH L 0–1 (a variable above 1 is read as a percentage), C 0–0.4,
  H degrees; HSL / HSB H degrees, S L B 0–100; RGB 0–255; alpha 0–100.
- Variables are read in the modes that apply where the colour is: a layer's frames (switch a
  frame to Dark and its layers re-read Dark values), a colour variable's own collection mode;
  styles use the default modes.
- Editing the colour itself (hex, square, eyedropper) updates the unbound channels; bound ones
  stay bound.
- The links are stored as text — a layer's data `figcraftplus.colors`, or a line in a style's or
  colour variable's description, e.g. `fills/0 oklch(0.62 0.19 {brand/hue} / {fade})`. Variables
  are referred to by name: rename one → update the name in the line too. The official FigCraft
  opens such files fine and keeps the last computed colours.

### Variables in text: weight, paragraph spacing and indent

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Font family and font style take string variables.
>
> **FigCraft Plus:** The weight takes a number variable too (400, 700), and paragraph spacing and indent take number variables.

<p>
<img src="docs/images/12-text-variables.png" width="560" alt="A text layer with font family, weight and paragraph spacing bound to variables">
<img src="docs/images/13-weight-or-style-list.png" width="272" alt="The font style dropdown's list: number and string variables">
</p>

FigCraft binds the font family and font style to string variables. FigCraft Plus binds the rest of
the Typography section:

| Field | Variable | Example |
|---|---|---|
| **Font style / weight** | a **number** (the weight) as well as a string (the style) | `weight/bold` = 700 |
| **Paragraph spacing** | number | `space/para` = 16 |
| **Paragraph indent** (Typography ▸ …) | number | `space/indent` = 24 |

- *Apply variable* in the font style menu lists **both** kinds: a number sets the weight (the
  nearest style the font has is used), a string sets the style by name. Binding one replaces the
  other in one undo step.
- Variables follow their modes like everywhere else (e.g. a Mobile mode with a smaller paragraph
  spacing), and lists respect each variable's **scopes**.
- While editing a range of characters these fields take plain values (bindings are per layer).

### Your own fonts, travelling with the app

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** Fonts come from the system (and Adobe Fonts / font managers on macOS).
>
> **FigCraft Plus:** A `fonts` folder next to `figcraft.exe` — your fonts travel on the USB drive — with *Add fonts…*, an *In this file* section and *Added* tags.

- Font files (`.ttf`, `.otf`, `.ttc`) in a **`fonts` folder next to `figcraft.exe`** are added at
  start — on a USB drive your fonts travel with the app.
- **Add fonts…** at the top of the font menu copies chosen files into that folder and adds them at
  once; they are tagged *Added* in the list.
- The font menu gains an **In this file** section (the families the open file uses), between
  FigCraft's *Recent* and *All fonts*.

### Styles: create and edit them in a dialog

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Styles are created from the selection and edited in FigCraft's style popover.
>
> **FigCraft Plus:** A **+** on Local styles creates any kind of style from scratch in a dialog with a live preview, with variables in effect, grid and colour fields too.

<p>
<img src="docs/images/15-create-style-menu.png" width="236" alt="The + menu on Local styles: Text, Color, Effect, Layout grid">
<img src="docs/images/22-style-dialog-text.png" width="306" alt="Create new text style: preview, name Heading/H1, Inter, the weight bound to weight/bold, size 40">
</p>
<p>
<img src="docs/images/23-style-dialog-effect.png" width="440" alt="Create new effect style with the drop shadow's settings beside it">
<img src="docs/images/24-style-dialog-grid.png" width="440" alt="Create new layout guide style with the columns' settings">
</p>
<img src="docs/images/25-style-dialog-color.png" width="440" alt="Create new color style with the colour picker beside it">

The **Local styles** header (Design panel, nothing selected) has a **+**: choose **Text**,
**Color**, **Effect** or **Layout grid** and the **Create new … style** dialog opens — a preview,
**Name** and **Description**, and **Properties**:

- **Text** — font, style (or weight), size, line height, letter spacing, and under **…**
  decoration and case.
- **Color** — its paints: click the swatch for the full colour picker (solid, gradients, image…,
  OKLCH); hex and opacity in the row; **+** adds a paint.
- **Effect** — a row per effect; click it for its settings beside the dialog: the **type**
  (Drop shadow, Inner shadow, Layer blur, Background blur, Noise, Texture, Glass), position,
  blur, spread, colour, "show behind transparent areas"… **+** adds an effect.
- **Layout grid** — a row per grid; its settings: **Grid / Columns / Rows**, size or count, type
  (Stretch, Left, Center, Right), width, margin, gutter, colour. **+** adds a grid.

Every row has a visibility toggle and **−**. **Create style** keeps the style; the cross (or Esc)
drops it. Click a style in Local styles — or **+** and edit in a layer's style list — to open the same
dialog (**Edit … style**; a style made from a layer opens as **Create new … style**) — changes
apply as you make them, each one undo step.

**Variables in styles.** Fields a style can bind take a variable (the hexagon on hover):

| Style | Fields |
|---|---|
| Text | font family (string), style or weight (string or number, as in the Typography panel), size, line height, letter spacing (numbers) |
| Color | each solid paint's colour |
| Effect | shadow X, Y, blur, spread, colour; blur radius |
| Layout grid | size, count, margin, gutter |

Layers using the style follow the variables in their own modes (a frame set to Mobile shows the
Mobile size).

### Style groups in Local styles

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Local styles are grouped by the folders in their names.
>
> **FigCraft Plus:** Real nested groups: Ctrl+G, New group, drag styles and groups, collapse, rename, ungroup.

Styles sit in **nested groups** (kept as `/` in their names, so the official FigCraft reads them as
folders). Click to select (Ctrl/Shift-click for several), double-click to edit; **Ctrl+G** or
*Group* puts the selection in a group, **+ ▸ New group** makes an empty one; drag styles and whole
groups between rows, into a group or out to the top; collapse a group with its arrow, double-click
its name to rename it, ungroup it from its context menu.

### Variable details & scope, one click away

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Right-click a variable ▸ Edit variable.
>
> **FigCraft Plus:** An *Edit variable* button on every row, opening beside the editor and following it.

<img src="docs/images/14-edit-variable.png" width="640" alt="The Edit variable button on a row and the variable's details and scopes">

Hover a row in the Variables editor: the **Edit variable** button (two sliders) appears at the end
of the name. It opens the variable's name, description, code syntax and **scoping** — which
properties pickers offer it for (corner radius, gap, font weight, font size, paragraph spacing,
fills, strokes…). The window opens beside the Variables editor at the variable's row, always above
it, and follows the editor when it moves. Click the button again to close. (Right-click ▸ Edit variable still works.)

### A richer variable picker

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** A plain list of the variables that fit.
>
> **FigCraft Plus:** One card everywhere: search, library filter, collections and groups, swatches and values, arrow keys and Enter.

One picker everywhere a variable is applied (layer properties, styles, aliases, prototype
actions, colour channels): search, a library filter, collections and groups as headers, each
variable's swatch or type and its value; arrow keys and Enter work. A variable with every scope
turned off is offered nowhere but the Variables editor, so lists stay short.

### Reorder variables: drag & drop and sorting

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** Modes can be reordered; variables keep the order they were made in.
>
> **FigCraft Plus:** Drag variables, groups and collections, multi-select, Alt+↑/↓, and natural or by-value sorting — also as commands for scripts and MCP.

Variables, groups and collections keep the order you give them: in the editor, in the file and
in exports. That fixes imports that arrive in the wrong order (`n-xs, n-6xl, n-s, n-m …` or
`1, 10, 2, 3`).

<img src="docs/images/11-drag-several.png" width="560" alt="Two selected variables dragged in the table, with the drop line">

- **Drag a variable** up or down in the table; a blue line shows where it lands. Drop it on
  another group's rows or header to move it into that group (it is renamed `group/name`, e.g.
  `space/gap` → `number/gap`). Variables stay in their collection.
- **Select several**: click, then **Shift**-click for a range or **Ctrl**-click to add or remove
  one; drag any selected row to move them all. **Esc** clears the selection (or cancels a drag).
- **Alt + ↑ / ↓** moves the selected variables one row within their group.
- **Drag a group** by its header in the table or its row in the sidebar (to the top level: drop on
  **All variables**). Subgroups move with it.
- **Drag a collection** in the sidebar.
- **Sort**: right-click a group or a collection in the sidebar.

  <img src="docs/images/09-sort-menu.png" width="560" alt="The group menu with Sort by name and Sort by value">
  <img src="docs/images/10-sorted.png" width="560" alt="The group after Sort by name: n-xs, n-s, n-m, n-l, n-xl, n-2xl … n-6xl">

  - **Sort by name** is natural and knows sizes: `xxs, xs, s, m, l, xl, 2xl … 6xl` and
    `1, 2, 10`.
  - **Sort by value** goes low → high: numbers by value, colours light → dark, then text and
    booleans.
  - Groups keep their places; variables are sorted inside each group.
- Each drag, Alt+arrow or sort is one undo step. Read-only libraries can't be reordered.
- The same operations are commands for scripts, the CLI and MCP: `variable.move`,
  `variable.moveGroup`, `variable.moveCollection`, `variable.sort`.

### Responsive layout: %, vw, vh and layout from variables

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** A size is px, Hug or Fill; an auto layout frame's direction and alignment are set by hand per frame.
>
> **FigCraft Plus:** Sizes that are a share of the parent or of the frame on the canvas, kept in step; auto layout that follows variables, so a mode switches it.

<img src="docs/images/46-relative-sizes.png" width="560" alt="A frame with a bar 50% of its width; the W sizing menu offers Fixed, Percent of parent (%), Viewport width (vw)">

- **`%` of the parent**, **`vw` / `vh` of the frame** the layer sits in (its viewport). Pick them in
  the W / H sizing menu (*Percent of parent*, *Viewport width / height*) — layers outside auto
  layout get the menu too — or type `100vw`, `50vh` in W / H. The menu then reads `50%`.
- Resize the parent (or the frame) and the layer follows, its children by their constraints. A
  share of a share works. Inside auto layout the parent's padding is left out, as in CSS.
- Typing a size by hand, dragging a handle, or picking Fixed / Hug / Fill ends the share for that
  side. On a side that is a percentage, `60%` changes the share (elsewhere `%` keeps FigCraft's
  meaning: a share of the value there now).
- **Layout from variables:** the variable button at the end of an auto layout frame's direction
  row binds its **direction** (row / column / grid), **wrap**, **main and cross alignment**
  (start, center, end, space-between…) and its **W / H sizing** (hug, fill, fixed) to string or
  boolean variables. Switch the frame's mode — Desktop / Mobile — and the layout follows: one card
  that is a row on Desktop and a column on Mobile.

  <img src="docs/images/47-layout-variables.png" width="320" alt="Layout from variables: Direction bound to Layout/Card direction, the others None">
- Files stay plain px with FigCraft's own layout: the shares and bindings are kept as text the
  official app ignores; it shows the last sizes and layout.

### Align to a key object

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Several selected layers align to the box around all of them; a click on one of them selects just it.
>
> **FigCraft Plus:** With several layers selected, click one: it becomes the **key object** (a thicker outline), and the align buttons move the others to it while it stays put.

<img src="docs/images/48-key-object.png" width="420" alt="Three selected rectangles, the middle one outlined thicker as the key object">

- Click the key again to select just it, as before. Turn it off in **Preferences ▸ FigCraft Plus
  ▸ Key object for aligning**.

### Design System Inspector

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** No check of the design system as a whole.
>
> **FigCraft Plus:** **Inspect** in the Variables editor's header lists what keeps the design system from holding together, grouped, each finding one click from its layer or variable, most with a fix.

<img src="docs/images/42-design-system-inspector.png" width="390" alt="The inspector: Not on a variable · 4 (fills and corner radii typed by hand that a variable holds, each with Fix), Unused variables · 7, Fix all buttons">

| Finding | Fix |
|---|---|
| **Not on a variable** — a solid fill or stroke, a corner radius, gap, padding, stroke weight or font size typed by hand although a variable holds exactly that value (one scoped for it, or named for it) | Binds it |
| **Broken references** — an alias or a formula reading a variable that's gone; an alias to a variable of another type; a number that isn't finite | — (jump to it) |
| **Circular formulas** — aliases or formulas that read themselves in the end | — |
| **Unused variables** — nothing binds, aliases or reads them | Deletes them |
| **Duplicate tokens** — colours of one collection, or numbers / text of one group, with the same values in every mode | — |
| **Without a scope** — offered for every property | — |

**Fix all** fixes a whole group as one undo step; the list follows every change.

### Variables as CSS for code, and from CSS

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Variables export as CSS custom properties — colours in hex, sizes in px, calculations flattened to their values — and import from its variables JSON.
>
> **FigCraft Plus:** **Export ▸ CSS for code…** writes them the way code does, and **Import ▸ CSS custom properties…** reads a stylesheet back.

<img src="docs/images/43-css-for-code.png" width="270" alt="CSS for code: Colours Hex / OKLCH, Sizes px / rem, Formulas as CSS, Export…">

- **Colours** as Hex or `oklch()`; **sizes** in px or `rem` of the file's root size (opacity and
  weights stay bare).
- **Formulas as CSS:** a derived colour as a relative colour —
  `oklch(from var(--brand) calc(l - 0.08) c h)` — a mix as `color-mix()`, a calculated number as
  `calc(var(--space-m) * 2)`, FigCraft's own alpha variants and mixes as CSS too.
- **Import** reads `--name: value` from `:root` (or `@theme`) as the first mode, and every other
  block — `[data-theme="dark"]`, `.dark`, `@media (prefers-color-scheme: dark)` — as a mode named
  after it. Colours (hex, names, `rgb()`, `hsl()`, `oklch()`), lengths (px; rem and em of the
  root size), numbers and font stacks become variables, `var(--x)` an alias, relative colours,
  `color-mix()` and `calc()` calculated variables that follow what they read. `--space-m` lands in
  the group `space` when several properties share the prefix; code keeps the CSS name.

### Text styles together

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** Text styles are edited one at a time.
>
> **FigCraft Plus:** Select several text styles in Local styles (Ctrl/Shift-click) ▸ right-click ▸ **Edit N text styles…**

<img src="docs/images/44-text-styles-together.png" width="310" alt="Edit 3 text styles: Set for all (font family, style, size Mixed, line height, letter spacing) and Bind variables by name: Font size from Type · font-size, Heading/H1 → font-size/h1, Heading/H2 → font-size/h2, Body → font-size/body, Bind 3">

- **Set for all:** font family, style, size, line height, letter spacing. A field shows the shared
  value or *Mixed*; what you type goes to every style as one undo step (`1.4em` works).
- **Bind variables by name:** pick the property (font size, line height, letter spacing, family,
  style) and the group of variables; each style finds the variable of the same name
  (`Heading/H1` ↔ `font-size/h1`), shown before you **Bind**. No guessing between equally good
  names: those stay unmatched.

### Missing fonts: replace, and remember

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Text ▸ Missing fonts lists the fonts a file uses that aren't installed, and downloads the ones the font store has.
>
> **FigCraft Plus:** **Replace…** each with an installed font, remembered for the next file.

<img src="docs/images/45-missing-font-replace.png" width="430" alt="Missing fonts: Gone Sans · 1 text layer · Replace… open on a searchable list of installed fonts; Remember replacements for other files">

- Every use moves: whole layers, single runs (the other runs keep their fonts), text styles — one
  undo step.
- With **Remember replacements for other files** on, the next file that uses that font gets the
  same replacement as it opens, with a note. **Preferences ▸ FigCraft Plus** lists the remembered
  ones; × forgets one.

### Line height in % from a variable

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** A number variable bound to the line height is pixels — `leading = 24` gives every text 24 px, whatever its size.
>
> **FigCraft Plus:** The line height can follow a number variable as a **percentage of the font size**, as CSS writes it: `leading/normal = 140%` suits every size.

<img src="docs/images/49-line-height-percent.png" width="560" alt="The line height's variable list with the px | % switch on %: leading/tight 120%, normal 140%, relaxed 160%; the field reads 140% normal">

- Hover the line height field, click the variable button, and pick **%** in the *Line height in px | %*
  switch under the search — the list then shows each variable as a percentage. The field shows its
  chip, `140% normal`; a click on it picks another, its detach button keeps the value.
- A value up to 5 is a ratio, as a unitless CSS `line-height`: `1.4` is 140%. Above, a percentage.
- Text layers and **text styles** (the style dialog's line height) take it; the layers using a
  style follow it. Modes switch it (tight leading on Mobile, relaxed on Desktop).
- Typing a line height, or binding a variable the px way, ends it. **CSS for code** writes these
  variables as ratios (`--leading-normal: 1.4`), ready for `line-height: var(--leading-normal)`.
- The file keeps a plain percentage; which variable it follows is text the official app ignores.

### Gradient tokens: gradients blended in OKLCH

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Gradient stops can take colour variables; the gradient mixes them in sRGB, so blue → yellow passes through grey.
>
> **FigCraft Plus:** **Blend · OKLCH**: the gradient is mixed in OKLCH between its stops — the colours stay alive — and follows their variables. A paint style with it is a gradient token.

<img src="docs/images/50-gradient-oklch.png" width="640" alt="The same blue → yellow gradient twice: blended in sRGB it goes grey in the middle; blended in OKLCH it passes through teal and green. The picker shows Blend · sRGB | OKLCH under the stop actions">

- In a gradient's colour picker, under *Remove stop · Flip · Rotate 90°*: **Blend · sRGB | OKLCH**.
  Works for fills and strokes, and for paint styles — make the gradient a style, bind its stops to
  your palette, and every layer using it follows the palette and its modes.
- In OKLCH, Plus places small steps between each two of your stops, mixed in OKLCH the way CSS's
  `linear-gradient(in oklch, …)` does (the shorter way round the hue). The picker's bar and the
  canvas show only your stops; add, move or remove them as usual, and the steps follow.
- Back to sRGB, only your stops stay. The file keeps plain stops, which the official app draws as
  they are.

### Resolution (DPI) per frame

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** A pixel is a point: 72 per inch, everywhere.
>
> **FigCraft Plus:** Each top-level frame has its own resolution. Start a flyer at 72, switch it to 300: it scales, content and all, and stays 210 × 297 mm.

<img src="docs/images/51-frame-resolution.png" width="720" alt="An A4 flyer at 300 DPI: W 210 mm, H 297 mm in Layout, Resolution 300 dpi with its presets menu open (72 · web, 96, 150, 300 · print, 600); the frame beside it has moved aside">

- **Layout ▸ Resolution** of a top-level frame: type it or pick 72 · web, 96, 150, 300 · print,
  600. The frame and everything in it scale by the ratio (72 → 300: × 4.17) from its top-left
  corner, so its size in mm stays — A4 is now 2480 × 3508 px. **What is to its right and below it
  moves by as much as it grew**, so nothing overlaps; back to 72, everything returns.
- **mm, cm, in and pt** are read at the selection's frame resolution: the fields, the rulers, the
  measurements, Dev Mode; `210mm` typed in a 300 DPI frame is 2480 px.
- **Exports keep it:** a PNG or JPG of the frame says its DPI (300, or 600 at 2x), and a PDF page
  has the frame's physical size, not its pixel size.
- **Preferences ▸ FigCraft Plus ▸ Resolution of new frames**: 300 if you mostly print.
- The file stays plain px; the resolution is text the official app ignores.

### PDF/X-4 for print

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** The print PDF has bleed, crop and bleed marks, page boxes and CMYK colour — converted by formula, with no output intent.
>
> **FigCraft Plus:** **PDF/X-4** for coated FOGRA39 paper: the file a print shop's preflight expects.

<img src="docs/images/52-pdfx.png" width="222" alt="A PDF export setting's print options: 8.5 pt bleed, crop marks, CMYK colour, and PDF/X-4 (coated FOGRA39) ticked">

- In a PDF export's print options, under **CMYK colour**: **PDF/X-4 (coated FOGRA39)**. Every
  CMYK PDF of the file is then PDF/X-4:
  - an **output intent** for coated FOGRA39 (ISO 12647-2, the usual condition for sheet-fed
    offset on coated paper) with its **ICC profile** embedded;
  - colours and images **converted through that profile** (relative colorimetric), not by formula;
  - transparency **blended in CMYK**, a trim box on every page, `PDF/X-4` and *not trapped* in the
    metadata.
- The profile is `craft-coated-fogra39.icc` from storytold's craft-libs (MIT or Apache-2.0), built
  from the FOGRA39 characterisation data. Ask your print shop which condition it prints to; for
  another one, convert in your RIP or preflight tool.
- The choice is kept in the file as text the official app ignores; it exports FigCraft's own PDF.

### Export presets

**🔵 Enhanced** — better in FigCraft Plus

> **FigCraft:** Each layer has its own export settings (size, suffix, format); files are named after the layer.
>
> **FigCraft Plus:** **Presets…** in the Export section: every selected layer in several sizes and formats at once, named by a pattern.

- Four presets to start with: *Web · PNG 1x, 2x, 3x* (`{name}@{scale}`), *Social · 1080 px PNG and
  JPG*, *Every format* (a folder per format), *Print PDF* (CMYK, bleed, crop marks).
- Patterns: `{name}` `{frame}` `{page}` `{w}` `{h}` `{scale}` `{format}` `{n}` `{date}`; a `/`
  makes folders. A name used twice gets a number.
- *Save these settings as a preset* keeps the selected layer's own export settings; *Edit
  presets…* opens **Preferences ▸ FigCraft Plus ▸ Export presets** (rename, pattern, remove).
- The same as a command for scripts and MCP: `export.preset`.

### Design tokens (DTCG)

**🟣 Exclusive** — only in FigCraft Plus (for now)

> **FigCraft:** Variables export as JSON, CSS, Tailwind and Dart, and import from FigCraft's own JSON.
>
> **FigCraft Plus:** The **W3C Design Tokens (DTCG)** format, both ways — what Style Dictionary, Tokens Studio and other tools exchange.

- **Variables ▸ Export ▸ Design tokens (DTCG)…**: collections as top-level groups, `/` as nested
  groups; `$type` (`color`, `dimension`, `number`, `fontWeight`, `fontFamily`) and `$value`;
  aliases as `{Brand.color.blue}`; the first mode as `$value` and every mode in
  `$extensions["figcraft.modes"]`; a Plus formula in `$extensions["figcraft.formula"]`.
- **Variables ▸ Import ▸ Design tokens (DTCG)…**: groups become names, top-level groups
  collections, modes come back; tokens from other tools read too — colours (hex, CSS colours,
  `{colorSpace, components}`), dimensions (px, rem of 16), numbers, font weights (`bold` = 700),
  durations and font families. Composite tokens (shadow, typography, gradient) are skipped with a
  warning.
- Commands: `variable.exportDtcg`, `variable.importDtcg`.

## Reporting bugs

FigCraft Plus adds to FigCraft; it doesn't compete with it or replace it. So its issue tracker takes
**only FigCraft Plus's own features**:

1. Find the feature in the [**guide**](docs/GUIDE.md) and follow its steps.
2. Try the same in the **official FigCraft of the same version** (the release name says which). If
   it happens there too, report it in [FigCraft's tracker](https://github.com/storytold/figcraft/issues):
   their fix reaches Plus with their next release.
3. Otherwise, [**report it here**](../../issues/new?template=bug_report.yml) — the form asks for the
   feature, the steps and what you expected.

Plus doesn't patch FigCraft's own behaviour: a fix of ours would only have to be undone when
theirs arrives. Ideas and "this could be easier" go in a [feature request](../../issues/new?template=feature_request.yml).

## Community

FigCraft Plus grows from what its users need. Every way to take part:

| | |
|---|---|
| 💡 **[Request a feature](../../issues/new?template=feature_request.yml)** | Describe what you want to do and why it helps |
| 👍 **[Vote for features](../../issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement+sort%3Areactions-%2B1-desc)** | A 👍 on a request moves it up the [roadmap](ROADMAP.md) |
| 🐞 **[Report a bug](../../issues/new?template=bug_report.yml)** | Bugs in the add-ons; FigCraft's own bugs go to [its tracker](https://github.com/storytold/figcraft/issues) |
| 🛠 **[Contribute](#add-a-feature)** | Add-ons are small, self-contained files and hooks — see *Add a feature* |
| ⭐ **Star the repository** | It helps others find FigCraft Plus |

## FAQ

<details>
<summary><b>Will my files still open in the official FigCraft?</b></summary>

Yes. Plus doesn't change FigCraft's file format code at all (the build stops if an add-on would),
so a file saved in Plus is written exactly as FigCraft writes it. Formulas, rem settings and style
groups are kept as text the official app keeps and ignores; it shows the last computed colors
and sizes. Files from the official app open in Plus unchanged too.
</details>

<details>
<summary><b>Windows SmartScreen says the app is unrecognised. Is it safe?</b></summary>

The build isn't code-signed, so SmartScreen warns about any new download. It is built in the open
by [GitHub Actions](../../actions) from FigCraft's own release tag plus the files in this
repository — you can read every step in `.github/workflows/build.yml`. Choose **More info → Run
anyway**.
</details>

<details>
<summary><b>How do I update?</b></summary>

Unzip the new release over the old folder. Settings (`figcraft-plus.json`) and your `fonts`
folder sit next to `figcraft.exe`; keep them if you unzip somewhere else.
</details>

<details>
<summary><b>What happens when a new FigCraft version comes out?</b></summary>

The workflow checks for new FigCraft releases every day, applies the add-ons, runs FigCraft's
tests and ours, and publishes a new Plus build — usually within a day. If an add-on no longer
fits the new version, nothing is published and the previous release stays available until it is
fixed. Features FigCraft now has itself are removed from Plus.
</details>

<details>
<summary><b>Is there a macOS or Linux build?</b></summary>

Not yet: builds are Windows-only for now. You can [build it yourself](#build-locally) on any
platform FigCraft supports.
</details>

<details>
<summary><b>Is FigCraft Plus affiliated with FigCraft?</b></summary>

No. It is an independent, community project built on the open-source FigCraft, applied to its
official releases.
</details>

## How it works

```
overlay/          the additions' own files, copied into FigCraft at the same paths
hooks/            where they connect: "insert after this line" / "replace this text"
scripts/apply.py  copies overlay/ and applies hooks/ to a FigCraft checkout
CUSTOM_VERSION    the FigCraft Plus version (bump after changing the additions)
.github/workflows/build.yml   the daily build
```

Every day at 04:17 UTC the workflow checks FigCraft's latest release. If this repository hasn't
built it yet (or the additions changed), it checks the release out, runs `scripts/apply.py`, runs
the additions' tests and publishes `v<figcraft version>-plus.<CUSTOM_VERSION>` under Releases.

If a FigCraft update moves code a hook relies on, the build stops at **Connect custom features**
with the hook named (e.g. `0001-oklch-colour-picker.json #5 (picker: OKLCH fields): no line
matches …`), GitHub e-mails the owner, and the previous release stays available. Fix the hook's
anchor text — usually one line — and the next build goes through.

### Run a build by hand

**Actions → Build FigCraft Plus → Run workflow**. Optionally give a FigCraft tag (e.g.
`v0.6.0`) or tick **force** to rebuild a version that is already released.

### Build locally

```sh
git clone https://github.com/storytold/figcraft
python3 scripts/apply.py figcraft          # --check lists every hook that no longer fits
cd figcraft && cargo run --release -p figcraft
```

### When FigCraft adds a feature of its own

Remove ours: delete its hook (and its `overlay/` files, or the parts of them it no longer needs),
move it to *Now in FigCraft itself* above and bump `CUSTOM_VERSION`.

### Add a feature

1. Put its code in new files under `overlay/` (same paths as in FigCraft).
2. Connect them with a new `hooks/<next number>-….json` (see `0001` for the format).
3. Bump `CUSTOM_VERSION`. Pushing starts a build.

## License

FigCraft is MIT OR Apache-2.0; these additions are offered under the same terms. The coated
FOGRA39 ICC profile used for PDF/X (`overlay/crates/pdf/src/craft-coated-fogra39.icc`) is
craft-cms's, from [storytold/craft-libs](https://github.com/storytold/craft-libs), also MIT OR
Apache-2.0. Not affiliated with the ArtCraft team. FigCraft is a trademark of its owners.
