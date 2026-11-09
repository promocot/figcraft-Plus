# FigCraft Plus — Roadmap

What we are working on, what comes next, and how to get a feature onto the list.

FigCraft Plus follows the official FigCraft releases. Each time a new FigCraft version comes out,
we first make sure every add-on still works on it, remove what FigCraft now does itself, and then
continue down this list. The work is planned in **stages**, each tied to a FigCraft version: a
stage starts once the add-ons are checked on that version. There are no dates — FigCraft's own
release pace sets them — and the plan is a forecast: requests and what FigCraft ships can move
things.

**Our three specialities:** 🎨 **Advanced color** · 🧩 **Design systems** · 🔁 **Design to code**.
Everything below serves one of them.

**Want something?** [Ask for a feature](../../issues/new?template=feature_request.yml) — and give a
👍 to the requests you'd like too. The most wanted ones move up.

| Stage | When | Theme |
|---|---|---|
| **1** | ✅ done in Plus 9 | Measurement, part 2 · contrast both ways |
| **2** | ✅ done in Plus 11 | Design system ↔ code |
| **3** | ✅ done in Plus 11 | Design System Inspector |
| **4** | ✅ done in Plus 12 | Responsive layout |
| **5** | ✅ done in Plus 14 | Web typography and colour tokens |
| **6** | ✅ done in Plus 14 | Print: resolution per frame, PDF/X |
| **7** | ✅ done in Plus 15 | Export presets, design tokens (DTCG) |
| **8** | ▶ now — Plus 17 | Everyday comfort |
| **9** | Plus 18 | Export and tokens, complete |
| **10** | Plus 19 | 🎨 Colour: tonal scales and the gamut |
| **11** | Plus 20 | Responsive states and control |

Between the stages: your feedback from the [guide](docs/GUIDE.md)'s scenarios — urgent bugs get a
release of their own, the rest joins the next stage.

---

## ▶ Stage 8 — Plus 17: everyday comfort

- **W / H in %, vw and vh in the field itself.** Choosing *Percent of parent* turns the field's
  pixels into a share (200 px of a 400 px parent reads `50%`, the layer keeps its size) and the
  share is edited there; the sizing menu only names the mode. The same for vw / vh.
- **Key object with Alt+click.** A plain click on one of several selected layers selects just it,
  as in FigCraft; Alt+click makes it the key object for aligning.
- **Line height px / %** remembers the last choice.
- **Presets…** moves into the Export section's header.
- **The style dialog shows paragraph settings** FigCraft 0.17 gave text styles (spacing, indent,
  lists).
- **A FigCraft release Plus doesn't fit opens an issue** here (the daily build already finds it;
  today only an e-mail says so).

## Stage 9 — Plus 18: export and tokens, complete

- **A full preset editor:** add or remove sizes, pick formats, not only the name and pattern.
- **The file names before exporting** with a preset.
- **DTCG composites:** typography and shadow tokens become text and effect styles, and styles go out
  as tokens too.

## Stage 10 — Plus 19: colour

- **OKLCH tonal scale generator:** 50 … 950 from one colour, as variables that follow it.
- **Gamut views in the picker:** lightness, chroma and hue slices of the gamut, to see how far a
  colour can go.

## Stage 11 — Plus 20: responsive states and control

- **Responsive component states** (Desktop / Tablet / Mobile) switched by variable modes.
- **A key to hide the selection outline** while editing.
- **Remappable keyboard shortcuts.**

## After stage 9: the guide in pictures

Every step of the [guide](docs/GUIDE.md) checked in the app, with a screenshot, then a GIF per
feature for the README.

## 💡 Ideas we are considering

Not planned yet — a 👍 on a [feature request](../../issues?q=is%3Aissue+label%3Aenhancement) helps
decide.

- Yours: [ask for a feature](../../issues/new?template=feature_request.yml)

## Already in FigCraft — use it there

Often asked for, and FigCraft has it already, so Plus doesn't add it: **Paste to replace**
(Edit menu) · **stroke weight per side** · **variable font axes** (Typography) · **bleed, crop
marks and CMYK in the print PDF** · **line height in %** (as a value) · **layout grid styles** ·
**gradient stops bound to colour variables** · **outline view** · **variables export** (JSON, CSS,
Tailwind, Dart) **and import** (JSON) · **px or rem in Dev Mode's code** · since 0.13: **one popover
style** that closes on Escape and a press outside · **the colour picker beside the Design panel** ·
**colour variables with an alpha, and mixes of two** (as variable values) · **a portable mode**
(settings next to the app) · **custom dash patterns** for strokes.

## ✅ Done

Full details in the [changelog](CHANGELOG.md).

| Release | Highlights |
|---|---|
| **Plus 16** | On FigCraft 0.17.0 · variables use FigCraft's new multi-select · DTCG keeps the file's order |
| **Plus 15** | On FigCraft 0.15.0 · **export presets** (sizes, formats, a name pattern) · **design tokens (DTCG)** both ways · the [guide](docs/GUIDE.md) |
| **Plus 14** | **Line height in % from a variable** · **gradients blended in OKLCH** (gradient tokens) · **resolution (DPI) per frame** · **PDF/X-4** for coated FOGRA39 |
| **Plus 13** | Fixes: the picker beside a colour style's dialog again; contrast samples one under the other in a narrow picker |
| **Plus 12** | On FigCraft 0.14.0 · **sizes in %, vw and vh** that follow the parent or the viewport frame · **auto layout from variables** (direction, wrap, alignment, sizing per mode) · **align to a key object** |
| **Plus 11** | **CSS for code** (oklch, rem, relative colours, `color-mix()`, `calc()`) and **CSS import** · several text styles at once, variables bound by name · remembered missing-font replacements · the **Design System Inspector** |
| **Plus 10** | The colour picker follows the selection again (clicks on the canvas and in Layers) · *Picker follows the selection* in Preferences, to turn it off |
| **Plus 9** | On FigCraft 0.13.1 (FigCraft's own popovers now) · rulers and Dev Mode in the document's unit, the code in px or rem · number variables shown in the unit of what they size · em for line height and letter spacing · contrast both ways |
| **Plus 8** | Switch the document's unit from any length field (click its unit, or right-click) · one way to close every popover · decimals shown, 0–4 |
| **Plus 7** | **Units everywhere:** a layout unit (px, rem, mm, cm, in, pt) and a text unit (px, rem, pt) per file, in every field and on the canvas · one popover style across the Design panel |
| **Plus 6** | Contrast check against what is really under the layer (any shape behind it) · a **Libraries** tab in the colour picker with the colour variables whose scope fits |
| **Plus 5** | The Typography settings popover opens reliably, beside the panel, inside the window · screenshots of every new setting in the README |
| **Plus 4** | Typography settings (the gear): px / rem and root size per file ([FigCraft #1325](https://github.com/storytold/figcraft/issues/1325)) · Preferences ▸ FigCraft Plus: default typography unit and root size, the model the colour picker opens in |
| **Plus 3** | Print units (mm, cm, in, pt) in every field · CMYK and *Copy as* in the colour picker · WCAG / APCA contrast check |
| **Plus 2** | Derived colours and calculated variables ([requested in FigCraft #1280](https://github.com/storytold/figcraft/issues/1280)) · rem units with a root size per file ([requested in FigCraft #1325](https://github.com/storytold/figcraft/issues/1325)) |
| **Plus 1** | OKLCH the oklch.com way · colour channels from number variables · a picker that stays with what you edit · text, style and variable improvements · portable fonts folder |

---

## How a request gets on the roadmap

1. Open a [feature request](../../issues/new?template=feature_request.yml): what you want to do and
   why it helps.
2. Others give it a 👍; we answer in the issue.
3. Requests that help the most people, fit FigCraft and keep `.figcraft` files compatible with the
   official app go into *Ideas*, then into a release.
4. If FigCraft adds the feature itself, we use theirs and remove ours.

Bugs in FigCraft itself belong in [FigCraft's own tracker](https://github.com/storytold/figcraft/issues);
bugs in the add-ons go [here](../../issues/new?template=bug_report.yml).
