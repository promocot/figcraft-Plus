# FigCraft Plus — guide and test scenarios

Every FigCraft Plus feature as a short walkthrough: **what it does**, **try it** (where to click),
**you should see** (the result, with numbers where there are some), **also check** (the edges
worth trying) and, where it helps, **not a Plus bug if…** (what FigCraft itself does).

Use it to learn a feature, or to test one: go through the steps, and when what you see differs
from *You should see*, that is a bug — [report it](#reporting-what-you-find) with the feature's
name and the step. When something works but is awkward, that is worth reporting too, as an
improvement.

**Before you start:** the version you test is in the window title and in the release name
(*FigCraft Plus N · FigCraft vX*). Keep the official FigCraft of the same version at hand
([its releases](https://github.com/storytold/figcraft/releases)): if something also happens
there, it is FigCraft's, not Plus's.

**Conventions:** *Design panel* is the right panel; a **›** is a menu step; `code` is what you
type. Unless a step says otherwise, start from a new file (**Ctrl+N**).

## Contents

- **Colour:** [1 OKLCH picker](#1-oklch-picker) · [2 CMYK and Copy as](#2-cmyk-and-copy-as) ·
  [3 Contrast both ways](#3-contrast-both-ways) · [4 Libraries tab](#4-libraries-tab-in-the-colour-picker) ·
  [5 Picker follows the selection](#5-the-picker-follows-the-selection) ·
  [6 Channels from number variables](#6-colour-channels-from-number-variables) ·
  [7 Derived colours and calculated variables](#7-derived-colours-and-calculated-variables) ·
  [8 Gradients in OKLCH](#8-gradients-blended-in-oklch)
- **Units and layout:** [9 Units](#9-units-per-file) · [10 Unit menu on fields](#10-the-unit-menu-on-a-field) ·
  [11 Print units and em](#11-print-units-and-em-typed-in-any-field) · [12 Rulers and Dev Mode](#12-rulers-and-dev-mode-in-the-files-unit) ·
  [13 %, vw, vh](#13-sizes-in--vw-and-vh) · [14 Layout from variables](#14-auto-layout-from-variables) ·
  [15 Key object](#15-align-to-a-key-object) · [16 DPI per frame](#16-resolution-dpi-per-frame)
- **Variables and styles:** [17 Number variables with a unit](#17-number-variables-with-a-unit) ·
  [18 Variable picker](#18-the-variable-picker) · [19 Variable details](#19-variable-details-one-click-away) ·
  [20 Reorder variables](#20-reorder-and-sort-variables) · [21 Text variables](#21-variables-in-text-weight-paragraph-spacing-and-indent) ·
  [22 Line height in %](#22-line-height-in--from-a-variable) · [23 Style dialog](#23-styles-created-in-a-dialog) ·
  [24 Style groups](#24-style-groups) · [25 Text styles together](#25-text-styles-together) ·
  [26 Design System Inspector](#26-design-system-inspector)
- **Code and files:** [27 CSS for code](#27-css-for-code) · [28 CSS import](#28-import-css-custom-properties) ·
  [29 DTCG tokens](#29-design-tokens-dtcg) · [30 Export presets](#30-export-presets) · [31 PDF/X-4](#31-pdfx-4-for-print) ·
  [32 Fonts folder](#32-your-own-fonts-folder) · [33 Missing fonts](#33-missing-fonts-replace-and-remember) ·
  [34 Preferences](#34-figcraft-plus-preferences) · [35 Files stay compatible](#35-files-stay-compatible)

---

## Colour

### 1. OKLCH picker

**What it does.** The colour picker in OKLCH draws an OKLCH plane (lightness × chroma at one hue)
with the exact edge of what the screen can show, and an OKLCH hue strip. Values match oklch.com.

**Try it.** Draw a rectangle (**R**, drag). Design panel › **Fill** › click the swatch. In the model
menu under the plane (Hex by default) pick **OKLCH**.

**You should see.** A triangle-like plane (not a square), its right edge curved; L, C, H, A fields;
the hue strip goes red → yellow → green → blue → magenta with even brightness. Type `0.7` `0.15`
`250` in L C H: the fill becomes a calm blue, and `oklch(0.7 0.15 250)` on oklch.com shows the same.

**Also check.** Drag across the plane past the edge: the point stays on the edge. Change H: the
plane's shape changes. Pick **Preferences › FigCraft Plus › Colour picker opens in › OKLCH** and
reopen the picker: it opens in OKLCH.

**Not a Plus bug if…** Hex, RGB, HSL and HSB look as in FigCraft: those are FigCraft's.

### 2. CMYK and Copy as

**Try it.** Open a fill's picker; model menu › **CMYK**. Under the fields: **Copy as**.

**You should see.** C, M, Y, K fields in percent. `#3B7BF5` reads C 76 M 50 Y 0 K 4. **Copy as** ›
`oklch()` puts `oklch(…)` on the clipboard; paste it in a text editor to check.

**Also check.** Type CMYK values: the colour follows. Every Copy as format pastes something CSS or
your tool understands.

### 3. Contrast both ways

**Try it.** Draw a big rectangle filled `#A3365F`, then a smaller one on it filled `#FFFFFF`. Open
the white one's fill picker.

**You should see.** Under Copy as, two `Aa` samples: white on berry and berry on white, each with
**6.46:1 AA** and an APCA **Lc** (−86 and 81). One is marked **✓ Reads better**.

**Also check.** Move the white rectangle off the berry one: the ratio changes to the page's
background. In a narrow picker (Hex) the two samples stand one under the other, nothing cut.
Hover a sample: it says which colour it measured against.

### 4. Libraries tab in the colour picker

**Try it.** Variables (left bar) › create a collection with colour variables. Give one the scope
*Stroke* only (**Edit variable** › Scoping). Open a rectangle's fill picker › **Libraries** tab.

**You should see.** This file's colour variables by collection and group, with search; the
stroke-only one is missing. Open the stroke's picker › Libraries: only the stroke one shows.
Clicking a variable binds it.

### 5. The picker follows the selection

**Try it.** Draw three rectangles. Open the first one's fill picker. Click the second rectangle on
the canvas, then the third in Layers.

**You should see.** The picker stays open and now edits the newly selected layer's fill. A click on
empty canvas closes it.

**Also check.** **Preferences › FigCraft Plus › Picker follows the selection** off: a click on
another layer closes it (as in FigCraft). A colour style's dialog (see 23): its swatch opens the
picker **beside** the dialog, not over it.

### 6. Colour channels from number variables

**Try it.** Variables › new collection › number variable `brand/hue` = `280`. Draw a rectangle,
open its fill picker in OKLCH, hover the **H** field › variable button › `brand/hue`.

**You should see.** H shows `hue`; the colour is violet. In Variables set `brand/hue` to `130`: the
rectangle turns green at once. Bind **A** (alpha) to another number variable (`60`): 60% opacity.

**Also check.** Switch the picker to HSL: a note says the channels are linked in OKLCH. One undo
(Ctrl+Z) after the variable change brings back both the value and the colour.

### 7. Derived colours and calculated variables

**Try it.** Variables › a colour variable `primary`. A second colour variable `primary-hover` ›
**Edit variable** › Description: `oklch(from {primary} calc(l - 0.1) c h)`. A number `space/m` = 16
and `space/l` with description `calc({space/m} * 2)`.

**You should see.** `primary-hover` is a darker `primary`, `space/l` is 32. Change `primary` or
`space/m`: they follow, in every mode.

**Also check.** `color-mix(in oklch, {primary} 30%, #FFFFFF)`. A reference to a variable that
doesn't exist: the value stays as it was (the Inspector, 26, lists it).

### 8. Gradients blended in OKLCH

**Try it.** Draw a rectangle; fill picker › the gradient icon (second) › set the stops to `#1F4BFF`
and `#FFD60A`. Under **Remove stop · Flip · Rotate 90°**: **Blend › OKLCH**.

**You should see.** In sRGB the middle is grey; in OKLCH it passes through teal and green. The bar
and the canvas still show **two** stops.

**Also check.** Bind a stop to a colour variable (the row under Blend) and change the variable: the
whole gradient follows. Add a stop (click the bar), move or remove one: the blend stays smooth.
**Blend › sRGB** brings back only your stops. A paint style with a gradient (23) has the switch too.

**Not a Plus bug if…** In OKLCH, the brightest greens and blues can show faint bands: they lie
beyond what the screen shows, as in CSS.

## Units and layout

### 9. Units per file

**Try it.** Click empty canvas › Design panel › **Units** row (Page section) › Layout **mm**, Text
**pt**, Root size `16px`.

**You should see.** Every length field shows mm (X, Y, W, H, radius, gap, padding, stroke,
shadows…), text fields pt. A 170 × 80 px rectangle reads 59.97 × 28.22 mm; the canvas label too.

**Also check.** Save, reopen: the units are kept in the file. A new file uses **Preferences ›
FigCraft Plus › Layout unit / Text unit**. **Decimals shown** (0–4) rounds what fields show only.

### 10. The unit menu on a field

**Try it.** Select a rectangle. Click the unit after the W value (e.g. `px` or `mm`) — or
right-click any length field.

**You should see.** px, rem, mm, cm, in, pt with the current one ticked, then **Units settings…**.
Picking one switches the **file's** layout unit (every field), not one property.

### 11. Print units and em, typed in any field

**Try it.** Select a frame; type `210mm` in W and `297mm` in H.

**You should see.** An A4 page: 595.28 × 841.89 px (at 72 DPI; see 16). Also `1in` (72), `12pt`,
`2cm`, `10mm + 2`, `1.5rem`. In a text's line height `1.5em` is 150%, letter spacing `0.02em` 2%.

### 12. Rulers and Dev Mode in the file's unit

**Try it.** Units › Layout mm. Rulers on (**Shift+R**, or right-click the canvas › Rulers). Then Dev Mode (the `</>` toggle in the
toolbar) and select a layer.

**You should see.** Rulers count round mm (0, 20, 40…); guides show mm. Dev Mode lists the
properties in mm and copies them as shown; its code stays in px (or rem — Dev Mode's code unit,
stored in the file).

### 13. Sizes in %, vw and vh

**Try it.** Draw a frame 400 × 300, a frame 200 × 100 inside it. Select the inner one; W's sizing
menu (under W) › **Percent of parent (%)**.

**You should see.** The layer keeps its size; **W** now reads `50%` and the menu **Percent**. Type
`75` in W: it is 300 wide (75% of 400). Resize the outer frame to 600: it becomes 450.

**Also check.** **Viewport width (vw)** / **Viewport height (vh)** the same way (W reads `100vw`,
the menu **Viewport**). In a W still in px, `50%` (on a side already in %), `100vw` or `50vh` typed
with the unit make it relative too. **Fixed**, **Hug** or **Fill** in the menu ends the share: W
is in px again. Inside auto layout, the parent's padding is left out.

### 14. Auto layout from variables

**Try it.** Variables › collection `Layout`, modes Desktop and Mobile; string variable
`card/direction` = `row` (Desktop), `column` (Mobile). Make an auto layout frame with three
children (Shift+A). Layout › the variable button at the end of the direction row › **Direction ›
card/direction**.

**You should see.** The frame is a row. In the frame's **Variable modes** section (Design panel) set `Layout`
to Mobile: it becomes a column. Also wrap, main and cross alignment, W / H sizing.

### 15. Align to a key object

**Try it.** Draw three rectangles at different heights. Select all three, then **Alt+click** the
middle one.

**You should see.** It gets a thicker outline (the key). **Align top**: the others move to its top;
it stays put. Alt+click it again: the outline goes, the selection stays. A plain click selects just
that layer, as in FigCraft.

**Also check.** **Preferences › FigCraft Plus › Key object for aligning** off: Alt+click does
nothing special.

### 16. Resolution (DPI) per frame

**Try it.** Units › Layout mm. Draw a frame `210mm` × `297mm` with some text and shapes; a second
frame to its right. Select the first frame › Layout › **Resolution** › the arrow › **300 · print**.

**You should see.** W 210 mm, H 297 mm stay; in px the frame is 2480 × 3508 and everything in it
is 4.17 times bigger, text included. The frame to its right moved right by as much as the frame
grew. **72 · web** brings everything back.

**Also check.** Type `600` in Resolution. In the 300 DPI frame, type `10mm` in a child's W: 118 px.
Export the frame as PNG: the file says 300 DPI (Windows › Properties › Details). Export as PDF:
the page is A4 (210 × 297 mm), not 2480 pt. **Preferences › Resolution of new frames** `300`: new
top-level frames start at 300.

## Variables and styles

### 17. Number variables with a unit

**Try it.** Units › Layout mm, Text pt. Variables › number `space/m` = 16, bind it to a frame's gap;
number `font/body` = 16, bind it to a text's size; `opacity/muted` = 0.6.

**You should see.** In the Variables editor `space/m` reads 5.64mm, `font/body` 12pt,
`opacity/muted` 0.6 (no unit). The variable list shows numbers in the unit of the field.

### 18. The variable picker

**Try it.** Hover any field that takes a variable (a fill, a radius) › the variable button.

**You should see.** One card: search, collections and groups, swatches or values, the one in use
checked. ↑ ↓ move, Enter applies. A field offers only variables whose scope fits it.

### 19. Variable details, one click away

**Try it.** Variables › hover a row › the button at the end of its name.

**You should see.** *Edit variable* opens beside the editor (name, description, code syntax,
scopes) and follows the row.

### 20. Reorder and sort variables

**Try it.** Variables › click a row, Ctrl+click another › drag them by the name. Right-click a group
in the left list › **Sort by name** / **Sort by value**.

**You should see.** A drop line, a "2 variables" badge; they land there. Sort by name orders
`n-xs, n-s, n-m, n-l, n-xl, n-2xl…`. Alt+↑ / ↓ moves the selection a row. Each is one undo step.

**Not a Plus bug if.** Selecting rows (click, Shift, Ctrl) and the right-click menu for several
variables (Delete, Move to group, Scope) are FigCraft's own since 0.17; Plus adds the dragging,
Alt+↑ / ↓ and sorting.

### 21. Variables in text: weight, paragraph spacing and indent

**Try it.** Number variables `weight/bold` = 700 and `space/para` = 16. Select a text; Typography ›
the font style dropdown › **Apply variable** › `bold`. Hover paragraph spacing › variable button ›
`para`.

**You should see.** The text is Bold; the style field shows the variable. Paragraph spacing shows
`para`. The Apply variable list has number and string variables.

### 22. Line height in % from a variable

**Try it.** Number variables `leading/tight` = 120, `leading/normal` = 140, `leading/relaxed` = 1.6.
Select a text; hover **line height** › variable button › switch **Line height in › %** › `normal`.

**You should see.** The rows read 120%, 140%, 160%. The field shows `140% normal`. Change the font
size from 24 to 40: the lines stay 140% of it.

**Also check.** Change `leading/normal` to 150: the text follows. Type `30` in line height: the link
ends (30 px). Detach (the button on the chip) keeps 140%. A text style's dialog (23) has the same.
Variables › Export › CSS for code: `leading/normal` is written `1.4`. **px** in the switch binds as
FigCraft does (pixels). The switch opens on your last choice, after a restart too.

### 23. Styles created in a dialog

**Try it.** Click empty canvas › Design panel › **Local styles** › **+** › **Text** (then Color,
Effect, Layout grid).

**You should see.** *Create new text style*: a live preview (Rag 123), name, description, font,
weight, size, line height, letter spacing — each can take a variable — paragraph spacing and indent,
and under **⋯** decoration, case and the list style. **Create style** keeps it;
the close button removes it. Effect and grid: a settings card beside the dialog. Color: the swatch
opens the picker beside the dialog.

**Also check.** Right-click a text style › **Edit style**: change its size; its paragraph spacing,
indent and list stay as they were.

### 24. Style groups

**Try it.** Make three colour styles. In Local styles select two, **Ctrl+G**.

**You should see.** A group; collapse, rename (double-click), drag styles and groups in and out;
right-click › Ungroup.

### 25. Text styles together

**Try it.** Make three text styles. Local styles › select them › right-click › **Edit 3 text
styles…**.

**You should see.** *Set for all* (font, style, size, line height, letter spacing; Mixed where they
differ) and *Bind variables by name* (font-size/h1 to Heading/H1…), previewed; one undo step.

### 26. Design System Inspector

**Try it.** A colour variable `brand` = `#1F4BFF`; a rectangle filled `#1F4BFF` by hand; an unused
number variable. Variables › **Inspect**.

**You should see.** *Not on a variable · 1* (the rectangle's fill — **Fix** binds it), *Unused
variables · 1* (**Fix** deletes it), and also broken references, circular formulas, unscoped and
duplicate variables when there are some. Each finding jumps to its layer or variable. **Fix all**
is one undo step.

## Code and files

### 27. CSS for code

**Try it.** Variables › **Export** › **CSS for code…** › Colours **OKLCH**, Sizes **rem**,
Formulas as CSS on › **Export…**.

**You should see.** A `.css` file: `--brand: oklch(0.52 0.27 265.49);`, sizes in rem, derived
colours as `oklch(from var(--primary) …)`, mixes as `color-mix()`, calculations as `calc()`,
each mode in its own block.

### 28. Import CSS custom properties

**Try it.** A `.css` file with `:root { --brand: #1F4BFF; --space-m: 1rem; }` and
`[data-theme="dark"] { --brand: #88A0FF; }`. Variables › **Import** › **CSS custom properties…**.

**You should see.** A collection named after the file, modes *Default* and *dark*, `brand` (colour)
and `space/m` (16). A toast says how many were added and updated.

### 29. Design tokens (DTCG)

**Try it.** Variables › **Export** › **Design tokens (DTCG)…** › save `tokens.json`. Open a new
file › Variables › **Import** › **Design tokens (DTCG)…** › the same file.

**You should see.** The JSON has your collections as top-level groups, `$type` and `$value`
(colours `#RRGGBB`, sizes `{"value": 16, "unit": "px"}`, aliases `{Brand.color.blue}`, other modes
in `$extensions["figcraft.modes"]`), in the order the variables have. The import rebuilds the same
collections, modes and aliases, in the file's order.

**Also check.** Import tokens made by another tool (Tokens Studio, Style Dictionary): colours,
dimensions (`1rem` = 16), numbers, font weights (`bold` = 700), font families come in; shadows and
typography composites are skipped with a warning in the toast.

### 30. Export presets

**Try it.** Select one or more frames › Design panel › **Export** (the section's header, left of
**+**) › **Presets…** ›
**Web · PNG 1x, 2x, 3x** › choose a folder.

**You should see.** Three files per frame: `Card@1x.png`, `Card@2x.png`, `Card@3x.png`. Other
presets: *Social · 1080 px PNG and JPG* (`Card_1080x1440.png` …), *Every format* (folders png/,
jpg/, svg/, pdf/), *Print PDF* (CMYK, 3 mm bleed, crop marks).

**Also check.** Add export settings to a layer (**+**), then **Presets…** › **Save
these settings as a preset**: it appears in the list. **Edit presets…** opens Preferences ›
**Export presets**: rename, change the pattern (`{frame}/{name}_{w}x{h}`), remove. Two layers
with the same name get `Card.png` and `Card 2.png`.

### 31. PDF/X-4 for print

**Try it.** Select a frame › Export **+** › format **PDF** › the `…` on the print row › tick **CMYK
colour**, then **PDF/X-4 (coated FOGRA39)** › **Export**.

**You should see.** A PDF that Acrobat's *Preflight* (or the print shop's) reports as PDF/X-4 with
the output intent *Coated FOGRA39*; colours are CMYK (Acrobat › Output Preview). Without CMYK the
PDF/X box is disabled.

**Also check.** The choice is kept per file. A frame at 300 DPI (16) still prints at its mm size.

### 32. Your own fonts folder

**Try it.** Put a `.ttf` / `.otf` into the `fonts` folder next to `figcraft.exe`; restart.

**You should see.** The font in the font list (also *Added* tags and *Add fonts…* in the font menu,
an *In this file* section).

### 33. Missing fonts: replace, and remember

**Try it.** Open a file using a font you don't have. Main menu › Text › **Missing fonts…** › **Replace…** › pick
an installed font; tick *Remember replacements for other files*.

**You should see.** Every run and text style with that font switches; the next file with it is
replaced on opening. **Preferences › FigCraft Plus** lists the remembered replacements (× forgets).

### 34. FigCraft Plus preferences

**Try it.** **Preferences** (Ctrl+,) › the **FigCraft Plus** section.

**You should see.** Layout unit, Text unit, Default root size, Decimals shown, Colour picker opens
in, Picker follows the selection, Key object for aligning, Resolution of new frames, Export presets
(collapsed), remembered font replacements. Hover a label for what it does. They are kept in
`figcraft-plus.json` next to `figcraft.exe`.

### 35. Files stay compatible

**Try it.** Use several features above in one file, save it, open it in the **official** FigCraft
of the same version.

**You should see.** It opens; colours, sizes and layouts are the last ones Plus computed. Formulas,
units, % sizes, DPI and the like are inactive there (kept as text it ignores) and come back when
the file is opened in Plus again.

---

## Reporting what you find

[**New issue › Bug in a FigCraft Plus feature**](../../../issues/new?template=bug_report.yml). Say
which feature (the numbers above), the steps, what you expected and what happened, the version and
Windows version. Screenshots help.

**Only FigCraft Plus's features.** FigCraft Plus adds to FigCraft; it doesn't compete with it or
replace it. If the same happens in the official FigCraft of the same version, report it to
[FigCraft](https://github.com/storytold/figcraft/issues): they fix it, and the fix reaches Plus
with their next release. Plus doesn't patch FigCraft's own behaviour, so it never has to undo a
patch when their fix arrives.

**Improvements** (it works, but could be easier) go in [**Feature request**](../../../issues/new?template=feature_request.yml).
