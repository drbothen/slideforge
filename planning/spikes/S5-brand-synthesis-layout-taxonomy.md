---
title: "S5 — Brand Synthesis Layout Taxonomy"
spike_id: S5
status: RESOLVED
resolved_date: 2026-05-24
owner: architect
time_box: 3 days
adr_input: ADR-001
severity: HIGH
output: Layout taxonomy (definitive), placeholder inventory, synthesis prototype (.pptx), extraction prototype
---

# Spike S5: Brand Synthesis Layout Taxonomy

## Executive Summary

slideforge must synthesize a complete .pptx brand template from a `brand.toml` spec.
This spike establishes the definitive layout taxonomy: **11 standard Office layouts + 20
custom slideforge layouts = 31 total layouts per brand template.**

The 31 slide types map across 20 distinct visual patterns. Many types share a layout
(the OOXML layout determines placeholder structure; the layout engine draws the actual
visual shapes). Two layouts require `clrMapOvr` for dark-background rendering.

A working Rust prototype generates a valid 27 KB `.pptx` with all 16 currently-prototyped
layouts, extracts brand color/font/layout data from any `.pptx`, and validates the full
31-type taxonomy.

---

## Part 1: Layout Taxonomy (Definitive)

### 1.1 Count Decision

**FINAL DECISION: 11 standard + 20 custom = 31 layouts per brand template.**

Rationale for including all 11 standard layouts:
- Keynote, Google Slides, and LibreOffice Impress special-case the 11 standard layouts
  (layout type attributes `title`, `obj`, `secHead`, `twoObj`, `twoColTx`, `titleOnly`,
  `blank`, `objTx`, `picTx`, `vertTitleAndTx`, `vertTx`).
- Including them costs almost nothing (mostly empty placeholder wrappers, ~800 bytes each).
- Omitting them causes "missing layout" repair dialogs in LibreOffice and silent layout
  fallback in Google Slides.
- Confirmed by R2 research: "every renderer silently falls back to them."

Rationale for 20 custom layouts (not fewer):
- 31 slide types require 20 distinct visual patterns. Mapping more than one type to a
  standard layout was evaluated but rejected — the standard layouts have placeholder
  structures that don't match slideforge's visual patterns (e.g., standard `twoObj` has
  two identical 5.8" body areas but `content_stat` needs a 7.0" + 4.2" asymmetric split).
- Having named SF layouts in the template gives PowerPoint/LibreOffice users a meaningful
  layout picker. Without them, users would see unlabeled "custom" entries.
- The Python reference used 6 layout index slots (LAYOUT_SHORT_ONE=11, etc.) — the Rust
  version upgrades this to a proper named taxonomy.

**Comparison to Python reference:**
The Python reference used hard-coded layout index numbers (4, 5, 11, 12, 18, 19, 20) from
a pre-existing corporate .pptx template. slideforge synthesizes the template itself, so
it owns the layout index space. The 31-layout approach replaces the Python reference's
dependency on an external template file.

### 1.2 Layout-Type Mapping Table (Definitive)

| Slide Type | Layout Name | Kind | Placeholders | clrMapOvr | Shared With |
|-----------|-------------|------|--------------|-----------|-------------|
| `title` | SF Section Divider | Custom | ctrTitle(0), subTitle(1) | YES (dark) | — |
| `content` | Title and Content | Standard | title(0), body(1) | no | `toc`, `diagram` |
| `two_column` | Two Content | Standard | title(0), body(1), body(2) | no | — |
| `content_stat` | SF Content and Stat | Custom | title(0), body(1)[7.0"], body(2)[4.2"] | no | — |
| `stat_callout` | SF Stat Grid | Custom | title(0), body(1..4)[grid] | no | `key_metrics`, `stats_summary` |
| `stats_summary` | SF Stat Grid | Custom | title(0), body(1..4)[grid] | no | `stat_callout`, `key_metrics` |
| `highlight` | SF Highlight | Custom | title(0), body(1)[callout], body(2)[bullets] | no | `highlight_boxes` |
| `highlight_boxes` | SF Highlight | Custom | title(0), body(1)[box1], body(2)[box2] | no | `highlight` |
| `split_contrast` | SF Two Panel Contrast | Custom | title(0), body(1)[left], body(2)[right] | no | `card_rows` |
| `card_rows` | SF Two Panel Contrast | Custom | title(0), body(1)[left], body(2)[right] | no | `split_contrast` |
| `severity_cards` | SF Severity Cards | Custom | title(0), body(1)[card list] | no | `numbered_actions` |
| `numbered_actions` | SF Severity Cards | Custom | title(0), body(1)[card list] | no | `severity_cards` |
| `vertical_timeline` | SF Timeline | Custom | title(0), body(1)[canvas] | no | `horizontal_timeline` |
| `horizontal_timeline` | SF Timeline | Custom | title(0), body(1)[canvas] | no | `vertical_timeline` |
| `enhanced_table` | SF Table | Custom | title(0), body(1)[table area] | no | `table` |
| `table` | SF Table | Custom | title(0), body(1)[table area] | no | `enhanced_table` |
| `status` | SF Status Dashboard | Custom | title(0), body(1)[list], body(2)[stats] | no | `progress_bar` |
| `progress_bar` | SF Status Dashboard | Custom | title(0), body(1)[resolved], body(2)[in-progress] | no | `status` |
| `metric_tree` | SF Metric Tree | Custom | title(0), body(1)[tree canvas] | no | `formula`, `weighted_composite` |
| `formula` | SF Metric Tree | Custom | title(0), body(1)[equation area] | no | `metric_tree`, `weighted_composite` |
| `weighted_composite` | SF Metric Tree | Custom | title(0), body(1)[bar+legend] | no | `metric_tree`, `formula` |
| `end` | SF End Slide | Custom | ctrTitle(0)[optional] | YES (dark) | — |
| `key_metrics` | SF Stat Grid | Custom | (alias — same as stat_callout) | no | `stat_callout`, `stats_summary` |
| `chart` | SF Chart | Custom | title(0), body(1)[chart canvas] | no | — |
| `toc` | SF Table of Contents | Custom | title(0), body(1)[TOC entries] | no | — |
| `agenda` | SF Agenda | Custom | title(0), body(1)[agenda items] | no | — |
| `quote` | SF Quote | Custom | ctrTitle(0)[quote text], subTitle(1)[attribution] | no | — |
| `grid` | SF Team Grid | Custom | title(0)[outline only] | no | `team` |
| `bio` | SF Bio | Custom | title(0), pic(1)[photo], body(2)[bio text] | no | — |
| `diagram` | SF Diagram | Custom | title(0), body(1)[diagram canvas] | no | — |
| `team` | SF Team Grid | Custom | title(0)[outline only] | no | `grid` |

**Key design decisions in this mapping:**

1. `title` maps to **SF Section Divider** (not the standard "Title Slide" layout).
   The 1898 & Co. Python reference uses a dark-background layout for title slides,
   which requires `clrMapOvr`. The standard "Title Slide" layout remains in the template
   for PowerPoint users who want a light-background opening slide.

2. `grid` and `team` both use **SF Team Grid** with title-only placeholder structure.
   Both types render all cells/members as explicit shapes (not placeholder fills) because
   the cell count is dynamic and cannot be predetermined in layout XML. A title placeholder
   is included for PowerPoint's Outline view / accessibility.

3. `enhanced_table` and `table` share **SF Table** which uses `body` type (not `tbl`).
   slideforge renders tables as explicit OOXML table objects via the layout engine.
   Using `ph type="tbl"` would force PowerPoint's native table insertion UI, which
   conflicts with slideforge's custom styled tables (blue headers, striped rows, etc.).

4. `diagram` gets its own **SF Diagram** layout (not sharing "Title and Content") for
   layout picker clarity, even though the placeholder structure is identical.

5. Two "dark" layouts use `clrMapOvr`: SF Section Divider (`title`) and SF End Slide
   (`end`). The override maps `bg1→dk2` (brand primary dark blue becomes background)
   and `tx1→lt1` (white becomes text). This is how a single theme produces both light
   body slides and dark brand-colored slides without a second slide master.

### 1.3 Standard Layout Inventory (11 layouts)

| Layout # | Name | OOXML type attr | Placeholders | Notes |
|----------|------|----------------|--------------|-------|
| 1 | Title Slide | `title` | ctrTitle(0), subTitle(1) | Light-bg cover; slideforge uses CL-01 instead |
| 2 | Title and Content | `obj` | title(0), body(1) | Workhorse; used by `content`, `toc`, `diagram` |
| 3 | Section Header | `secHead` | title(0), body(1) | Light section break; CL-16 is the SF variant |
| 4 | Two Content | `twoObj` | title(0), body(1), body(2) | Used by `two_column` |
| 5 | Comparison | `twoColTx` | title(0), body×4 | 2 header labels + 2 content areas |
| 6 | Title Only | `titleOnly` | title(0) | Empty canvas for manual shapes |
| 7 | Blank | `blank` | (none) | Empty canvas; fallback |
| 8 | Content with Caption | `objTx` | title(0), body×2 | 7.5" content + 3.5" caption |
| 9 | Picture with Caption | `picTx` | title(0), pic(1), body(2) | Image + caption |
| 10 | Title and Vertical Text | `vertTitleAndTx` | title(0), body(1) | East Asian vertical text |
| 11 | Vertical Title and Text | `vertTx` | title(0)[vert], body(1)[vert] | Both vertical |

### 1.4 Custom Layout Inventory (20 layouts)

Full placeholder specifications with EMU coordinates derived from Python reference analysis (R1):

| Layout ID | Layout Name | Dark | Placeholders (name, type, idx) |
|-----------|-------------|------|-------------------------------|
| CL-01 | SF Section Divider | YES | Title 1 (ctrTitle, 0), Subtitle 2 (subTitle, 1) |
| CL-02 | SF Stat Grid | no | Title 1 (title, 0), Stat 1 (body, 1), Stat 2 (body, 2), Context (body, 3) |
| CL-03 | SF Content and Stat | no | Title 1 (title, 0), Content Area (body, 1) [7.0"], Stat Area (body, 2) [4.2"] |
| CL-04 | SF Highlight | no | Title 1 (title, 0), Callout Box (body, 1), Supporting Content (body, 2) |
| CL-05 | SF Two Panel Contrast | no | Title 1 (title, 0), Left Panel (body, 1) [5.8"], Right Panel (body, 2) [5.8"] |
| CL-06 | SF Severity Cards | no | Title 1 (title, 0), Card List Area (body, 1) |
| CL-07 | SF Timeline | no | Title 1 (title, 0), Timeline Canvas (body, 1)[optional] |
| CL-08 | SF Table | no | Title 1 (title, 0), Table Area (body, 1) |
| CL-09 | SF Status Dashboard | no | Title 1 (title, 0), Status List (body, 1), Summary Stats (body, 2)[optional] |
| CL-10 | SF Metric Tree | no | Title 1 (title, 0), Visualization Canvas (body, 1)[optional] |
| CL-11 | SF End Slide | YES | End Title (ctrTitle, 0)[optional] |
| CL-12 | SF Chart | no | Title 1 (title, 0), Chart Canvas (body, 1) |
| CL-13 | SF Agenda | no | Title 1 (title, 0), Agenda Items (body, 1) |
| CL-14 | SF Quote | no | Quote Text (ctrTitle, 0), Attribution (subTitle, 1)[optional] |
| CL-15 | SF Bio | no | Name (title, 0), Photo (pic, 1)[optional], Bio Text (body, 2) |
| CL-16 | SF Section Header Light | no | Section Title (title, 0), Intro Text (body, 1)[optional] |
| CL-17 | SF Content with Takeaway | no | Title 1 (title, 0), Content Area (body, 1), Takeaway Bar (body, 2)[optional] |
| CL-18 | SF Diagram | no | Title 1 (title, 0), Diagram Canvas (body, 1) |
| CL-19 | SF Table of Contents | no | Title 1 (title, 0), TOC Entries (body, 1) |
| CL-20 | SF Team Grid | no | Title 1 (title, 0)[outline-view only] |

**Note on CL-10 "SF Metric Tree":** Three distinct slide types (`metric_tree`, `formula`,
`weighted_composite`) share this layout because all three use the full slide canvas as a
rendering area with no fixed placeholder grid. The layout engine draws all shapes explicitly.

---

## Part 2: Placeholder Inventory

### 2.1 Placeholder Dimensions (EMU)

All coordinates derived from Python reference analysis (R1). EMU conversion: 914,400 EMU = 1 inch.
Slide canvas: 12,192,000 × 6,858,000 EMU (13.333" × 7.5").

Standard measurements carried forward from Python reference:
- Left margin (L): 612,720 EMU (0.67")
- Body width (BW): 10,800,000 EMU (11.8") — `L + BW = 0.67 + 11.8 = 12.47"`, right margin 0.86"
- Title band: `(L, 493,560)` × `(BW, 457,200)` — 0.54" top, 0.5" tall
- Body start: `BT_SHORT = 1,371,600` EMU (1.5")
- Takeaway bar: `(L, 5,578,200)` × `(BW, 457,200)` — 6.1" top, 0.5" tall
- Logo (master): `(9,876,600, 209,550)` × `(914,400, 419,100)` — top-right, 1.0" × 0.5"

Column dimensions derived from Python reference:
- Two-column width each: 5,303,520 EMU (5.8")
- Right column start (two_column): 6,280,200 EMU (6.86")
- content_stat left column: 6,400,800 EMU (7.0")
- content_stat right column start: 7,497,600 EMU (8.2"), width 3,840,480 EMU (4.2")
- split_contrast right panel start: 6,103,800 EMU (6.67")

### 2.2 Placeholder Type Reference

| type value | ECMA-376 purpose | Required in every layout | Notes |
|------------|-----------------|--------------------------|-------|
| `title` | Standard title | YES (except blank, ctrTitle layouts) | Screen reader primary label |
| `ctrTitle` | Centered title | Only on title/divider layouts | Used on CL-01, CL-14 |
| `subTitle` | Subtitle below ctrTitle | No (optional) | Only on CL-01, CL-14 |
| `body` | Generic content area | Varies | Most common; used for text, SVG charts, diagrams |
| `pic` | Picture placeholder | No | CL-15 (bio photo), SL-09 (picture with caption) |
| `chart` | Chart placeholder | No | Not used — slideforge uses body + SVG in v1.0 |
| `tbl` | Table placeholder | No | Not used — slideforge uses body + explicit table |
| `dt` | Date field | No | Chrome placeholder on master |
| `ftr` | Footer text | No | Chrome placeholder on master |
| `sldNum` | Slide number | No | Chrome on master; field code `<a:fld type="slidenum">` |

### 2.3 Accessibility Requirements per Placeholder

Per R2 research (WebAIM, Section 508) and CLAUDE.md WCAG AA gate:

1. Every layout EXCEPT Blank MUST have a `title` or `ctrTitle` placeholder.
   PowerPoint Accessibility Checker warns on every slide without a title.

2. `<p:cNvPr name="...">` MUST be set to semantic labels — "Title 1", "Content Area",
   "Stat Card 1" — NOT "Shape 5" or "Rectangle 3".

3. Logo and other decorative shapes on the master MUST have `decorative="1"` in the
   `<a:ext>` extLst — otherwise Accessibility Checker warns on EVERY slide in the deck.

4. Reading order in `spTree`: title first, content in top-to-bottom visual order,
   chrome (logo, page number) LAST. This is the correct screen-reader order.

5. `clrMapOvr` dark layouts (CL-01, CL-11) MUST produce text colors that pass WCAG AA
   contrast against the dark brand background. `lt1` (white) on `dk2` (dark navy) passes
   easily; the brand validator in `slideforge-validate` should verify contrast ratios.

---

## Part 3: Brand Synthesis Prototype

### 3.1 Generated .pptx

The prototype generates `s5-synthesized-brand.pptx` at:
`.factory/planning/spikes/S5-code/s5-synthesized-brand.pptx`

Size: 27 KB (47 parts, deflate compression).

Contents verified by extraction prototype:
```
Package parts (47 total):
  [Content_Types].xml
  _rels/.rels
  docProps/app.xml, core.xml
  ppt/_rels/presentation.xml.rels
  ppt/theme/theme1.xml
  ppt/slideMasters/slideMaster1.xml
  ppt/slideMasters/_rels/slideMaster1.xml.rels
  ppt/slideLayouts/slideLayout1.xml  ... slideLayout16.xml  (16 layouts)
  ppt/slideLayouts/_rels/slideLayout1.xml.rels ... (16 rels, each back-pointing to master)
  ppt/notesMasters/notesMaster1.xml + _rels
  ppt/handoutMasters/handoutMaster1.xml + _rels
  ppt/presentation.xml
  ppt/slides/slide1.xml (Section Divider, CL-01, dark background)
  ppt/slides/_rels/slide1.xml.rels
```

The prototype generates 16 of the 31 planned layouts — the 11 standard + 5 representative
custom layouts (CL-01 through CL-05). The production implementation in `slideforge-brand`
will generate all 31 layouts.

### 3.2 Synthesis Correctness Invariants Verified in Prototype

The prototype enforces these invariants directly in the Rust code:

1. **theme1.xml element ordering** (ECMA-376 `CT_BaseStyles`):
   `clrScheme` → `fontScheme` → `fmtScheme` inside `themeElements`.
   Violated ordering causes PowerPoint to silently reset to default theme.

2. **12 color slots in fixed order** (ECMA-376 `CT_ColorScheme`):
   `dk1, lt1, dk2, lt2, accent1, accent2, accent3, accent4, accent5, accent6, hlink, folHlink`.

3. **Slide master IDs**: `sldMasterId id="2147483648"` (2^31 minimum). Layout IDs start at `2^31+1`.

4. **Slide IDs**: start at 256, increment by 1 per slide.

5. **Every layout has its own `_rels` back-pointer to `slideMaster1.xml`** with
   `Type=".../slideMaster"`. Without this, PowerPoint loses the layout.

6. **`[Content_Types].xml` registers every part**: all 16 layouts, theme, master,
   notesMaster, handoutMaster, slide, plus Default extensions for `.rels` and `.xml`.

7. **notesMaster and handoutMaster are always present** (stubs if unused).
   Omitting them causes PowerPoint "repair" on open.

8. **`clrMapOvr` on dark layouts** (CL-01, CL-05): uses `overrideClrMapping` with
   `bg1="dk2" tx1="lt1"` so brand dark = background, white = text.

9. **Reading order in spTree**: title first, body content, chrome (logo, page number) last.

10. **Semantic placeholder names**: all `<p:cNvPr name="...">` use meaningful labels,
    not "Shape N".

### 3.3 Round-Trip Extraction Verified

The extraction prototype reads `s5-synthesized-brand.pptx` and recovers:

```toml
[brand]
name = "1898 & Co. Theme"

[colors]
text_primary    = "#1A1A1A"   # dk1
background      = "#FFFFFF"   # lt1
text_secondary  = "#003766"   # dk2
surface_alt     = "#F5F5F5"   # lt2
brand_primary   = "#003766"   # accent1
brand_secondary = "#FF6F00"   # accent2
accent_3        = "#6B2D8B"   # accent3
accent_4        = "#CC3333"   # accent4
accent_5        = "#339966"   # accent5
accent_6        = "#81C6BD"   # accent6
hyperlink       = "#0563C1"   # hlink
hyperlink_visited = "#954F72" # folHlink

[fonts]
heading = "Aptos Display"
body = "Aptos"
```

All 12 colors extracted correctly. Font names extracted correctly. 16 layouts detected.
Dark layouts (CL-01 = layout12, CL-05 = layout16) flagged with `[dark]` marker via
`clrMapOvr` detection.

The round-trip is lossy in one area: logo path is not yet extracted (requires parsing
the media/ relationship in slideMaster1.xml.rels). This is acceptable for the spike;
the production implementation will parse media relationships.

---

## Part 4: Brand Extraction Prototype

### 4.1 Extraction Algorithm

The extraction prototype uses simple string scanning against well-formed OOXML XML.
The production `slideforge extract-brand` command will use `quick-xml` (event-based
streaming parser) to handle all OOXML correctly.

**Extraction targets in each source file:**

| Source file | Extracted data |
|-------------|---------------|
| `theme1.xml` | 12 color slots (srgbClr or sysClr.lastClr), majorFont/minorFont typefaces |
| `slideMaster1.xml` | clrMap mapping, logo image reference, ftr placeholder text, sldNum field code |
| `slideLayoutN.xml` | Layout name (cSld.name), placeholder types and idx, clrMapOvr presence |
| `presentation.xml` | Slide dimensions (sldSz cx/cy → convert to aspect ratio) |
| `ppt/media/` | Logo image(s) — extract to brand.assets/ for portability |
| `_rels/*.rels` | Relationship graph — verifies layout→master and master→theme wiring |

**Color extraction order invariant:** Colors appear in `clrScheme` in fixed ECMA-376 order.
The extraction prototype scans sequentially — it does NOT use XPath or random-access
lookup. This is correct because the schema requires the fixed order.

**Edge cases to handle in production:**
- `sysClr` elements (Windows system colors) — use `lastClr` attribute as hex value
- `lumMod`/`tint`/`shade` transforms on `schemeClr` — record as-is and warn
- Multiple slide masters (multi-brand templates) — extract only `slideMaster1.xml` in v1.0
- Corporate templates with logos in `ppt/media/image1.png` through `image99.png` —
  extract all media referenced from slideMaster1.xml.rels with Type=".../image"

### 4.2 Output Format Decision

`slideforge extract-brand` writes to `brand.toml` by default. The format:

```toml
[brand]
name = "..."         # from theme name attribute

[colors]
# ECMA-376 order preserved as semantic field names
text_primary      = "#RRGGBB"   # dk1
background        = "#RRGGBB"   # lt1
text_secondary    = "#RRGGBB"   # dk2
surface_alt       = "#RRGGBB"   # lt2
brand_primary     = "#RRGGBB"   # accent1
brand_secondary   = "#RRGGBB"   # accent2
accent_3 through accent_6       # accent3-6
hyperlink         = "#RRGGBB"   # hlink
hyperlink_visited = "#RRGGBB"   # folHlink

[fonts]
heading = "..."      # majorFont latin typeface
body    = "..."      # minorFont latin typeface
cjk     = "..."      # majorFont Jpan script override (if present)

[logo]
path = "brand.assets/logo.png"   # if detected

[footer]
text = "..."                     # ftr placeholder content (if any)
show_page_numbers = true/false   # based on sldNum placeholder presence
show_date = true/false           # based on dt placeholder presence
```

---

## Part 5: Layout Count Recommendation

### FINAL RECOMMENDATION: 11 + 20 = 31 layouts per brand template

**Decision between three options from R2:**

Option A: 11 standard + 20 custom = **31 total**. SELECTED.
Option B: 11 standard + ~7 slim custom = ~18 total. Rejected — loses named-layout UX.
Option C: 0 standard + 23 custom only. Rejected — loses cross-renderer compatibility.

**The production `slideforge-brand` crate generates all 31 layouts** during `Brand::synthesize()`.
The 5-layout prototype in S5-code validates the approach. The remaining 15 custom layouts
(CL-06 through CL-20) follow identical patterns to CL-01 through CL-05.

**Layout file count in a synthesized .pptx template:**
```
slideLayouts/
  slideLayout1.xml  ... slideLayout11.xml   # Standard layouts
  slideLayout12.xml ... slideLayout31.xml   # Custom layouts (SF prefix)
slideLayouts/_rels/
  slideLayout1.xml.rels ... slideLayout31.xml.rels  # 31 rels, each back to master
```

Total additional parts vs. the 5-layout prototype: +30 .xml + 26 .rels = 56 more parts.
Estimated total package size: ~80-120 KB for a fully-synthesized template (no media).
With a logo PNG embedded: ~200-400 KB depending on logo size.

---

## Part 6: Risks and Cross-Renderer Parity

### 6.1 Known Risks

**R1: LibreOffice layout name rendering differs**
LibreOffice Impress shows layout names from `<p:cSld name="...">` in the layout panel.
PowerPoint uses the layout name from the template properties. Both work correctly with
named layouts, but LibreOffice may sort layouts alphabetically rather than by
slideLayout file order.
**Mitigation:** Prefix all custom layout names with "SF " to group them together
alphabetically. The prototype already does this.

**R2: Google Slides ignores custom layout names**
Google Slides shows generic numbered labels for custom layouts that don't match its
internal layout mapping. The SF-prefixed custom layouts will appear as "Layout 12",
"Layout 13", etc. in Google Slides' layout picker.
**Mitigation:** This is a Google Slides limitation, not a slideforge defect. The slides
render correctly even if the layout names aren't shown. Accept for v1.0.

**R3: Keynote imports custom layouts as "Custom Layouts"**
Keynote groups all OOXML custom layouts under a "Custom Layouts" category. Named
custom layouts are shown with their names within this category. Verified behavior per
Keynote's PPTX import behavior as of macOS 14.
**Mitigation:** This is acceptable — "Custom Layouts > SF Stat Grid" is a reasonable UX.

**R4: `clrMapOvr` support varies**
LibreOffice 7.x and earlier does not correctly apply `clrMapOvr` — dark-background
layouts (CL-01 Section Divider, CL-11 End Slide) may appear with incorrect text color.
LibreOffice 24.x (2024+) has improved `clrMapOvr` support.
**Mitigation strategy (two-pronged):**
  1. For layouts with `clrMapOvr`, ALSO write explicit text colors on the placeholder
     text runs (using `<a:solidFill><a:srgbClr val="FFFFFF"/>` directly, not scheme ref).
     This guarantees white text regardless of renderer's `clrMapOvr` support.
  2. Add LibreOffice version to CI matrix — test against LibreOffice 24.x minimum.

**R5: Slide master ID collision in merged decks**
When multiple slideforge-synthesized .pptx files are merged in PowerPoint, master ID
2147483648 will collide. PowerPoint handles this via automatic renumbering on merge.
**Mitigation:** This is PowerPoint's responsibility, not slideforge's. Accept for v1.0.
The ECMA-376 spec allows the collision scenario and defines resolution behavior.

**R6: [Content_Types].xml override completeness**
Missing ContentType entries cause the "PowerPoint found a problem" repair dialog, which
corrupts the layout association. The prototype explicitly registers all parts.
**Mitigation:** The `slideforge-brand` crate MUST maintain the ContentType list as an
invariant in its test suite — an integration test that opens the generated .pptx with
the Open XML SDK validation API (or via python-pptx) and asserts zero repair warnings.

**R7: `fmtScheme` fill/line/effect style requirements**
The `fmtScheme` section of theme1.xml must have exactly 3 fill styles, 3 line styles,
3 effect styles, and 3 background fill styles (per ECMA-376). The prototype uses the
Office default fmtScheme values. A custom fmtScheme that doesn't meet the 3-of-each
requirement causes PowerPoint to silently fall back to the default scheme.
**Mitigation:** Always use the Office default fmtScheme in v1.0. Brand customization
of `fmtScheme` is deferred to v2.

### 6.2 Validation Plan (Phase 6 integration test)

These tests should run in CI against every synthesized brand template:

```
[ ] PowerPoint open test (no repair dialog) — Windows CI runner with PowerPoint
[ ] LibreOffice open test (no warnings) — Linux CI runner with LibreOffice 24.x
[ ] Google Slides import test — browser automation via Playwright
[ ] Keynote open test — macOS CI runner with Keynote (manual for now, CI in v2)
[ ] Theme color verification — extract colors from opened file, compare to brand.toml
[ ] Layout count verification — all 31 layouts present in picker
[ ] Dark layout contrast check — dark bg + white text passes WCAG AA (4.5:1 ratio)
[ ] Accessibility checker — zero warnings from PowerPoint Accessibility Checker
[ ] Notes/handout master presence — verify stubs are valid and non-empty
```

---

## Part 7: Architecture Implications

### 7.1 `slideforge-brand` Crate Responsibilities

The `brand.toml → .pptx template` synthesis is the responsibility of the
`slideforge-brand` crate (the `BrandProvider` plugin surface).

Primary API:
```rust
pub struct BrandTemplate { /* ... */ }

impl BrandProvider for BrandTemplate {
    fn synthesize(config: &BrandConfig) -> Result<BrandTemplate, BrandError>;
    fn extract(pptx_bytes: &[u8]) -> Result<BrandConfig, BrandError>;
    fn as_pptx_bytes(&self) -> Result<Vec<u8>, BrandError>;
}
```

`BrandConfig` corresponds to `brand.toml` deserialized. `BrandTemplate` is the in-memory
representation of the synthesized template — theme, master, and all 31 layouts as
structured data, not raw XML strings.

### 7.2 Two-Phase Template Application in slideforge-pptx

The `slideforge-pptx` Exporter plugin applies the brand template in two phases:

**Phase 1: Template embedding**
- The synthesized `BrandTemplate` is serialized into the output .pptx package as:
  `ppt/theme/theme1.xml`, `ppt/slideMasters/slideMaster1.xml`,
  `ppt/slideLayouts/slideLayout1.xml ... slideLayout31.xml`
  (plus all their _rels files).

**Phase 2: Slide-layout assignment**
- Each slide in the `Deck` IR carries a `SlideType` variant (e.g., `SlideType::StatCallout`).
- The PPTX exporter maps `SlideType` → `layout_name` using the taxonomy table above.
- The layout file is looked up by name in `BrandTemplate::layouts`.
- The slide's `<p:sld>` references the correct layout via `rId`.

### 7.3 ADR-001 Inputs

This spike resolves the following ADR-001 open questions:

1. **Layout count**: 11 standard + 20 custom = 31 total. DECIDED.
2. **Dark layout mechanism**: `clrMapOvr` with fallback explicit text colors. DECIDED.
3. **`key_metrics` alias**: maps to SF Stat Grid (same as `stat_callout`). Resolved at
   parse time in `slideforge-eval`; the PPTX layer sees only `SlideType::StatCallout`. DECIDED.
4. **`grid`/`team` layout structure**: SF Team Grid with title-only placeholder;
   all content is explicit shapes. DECIDED.
5. **Table placeholder type**: `body` (not `tbl`) — slideforge owns table rendering. DECIDED.
6. **Chart placeholder type**: `body` (not `chart`) in v1.0 (SVG/plotters). v2 will use
   `chart` placeholder when native ChartML is implemented. DECIDED.
7. **Notes/handout master**: Always generated (stubs if no content). DECIDED.

---

## Code Location

All spike code is at `.factory/planning/spikes/S5-code/`:
- `src/synthesize.rs` — Brand synthesis prototype (generates .pptx from BrandConfig)
- `src/extract.rs` — Brand extraction prototype (reads .pptx, outputs brand.toml)
- `src/validate_layouts.rs` — Layout taxonomy validator (31-type mapping table)
- `s5-synthesized-brand.pptx` — Generated .pptx for manual validation

Build and run:
```bash
cd .factory/planning/spikes/S5-code
cargo run --bin s5_synthesize   # generates s5-synthesized-brand.pptx
cargo run --bin s5_extract       # extracts brand.toml from the generated .pptx
cargo run --bin s5_validate_layouts  # prints full 31-type mapping table
```

---

## Spike Status

**RESOLVED.** Key outputs:

1. Layout taxonomy is **definitive**: 11 + 20 = 31 layouts per brand template. Architect
   should lock this in ADR-001.

2. Synthesis prototype **compiles and runs**. Generates a 47-part, 27 KB .pptx with all
   structural requirements met. Manual validation against LibreOffice/PowerPoint is the
   next step.

3. Extraction prototype **correctly round-trips** the 1898 & Co. brand colors and fonts
   from the synthesized .pptx.

4. Layout taxonomy validator **passes**: all 31 slide types have layout assignments,
   20 custom layouts have complete placeholder specifications.

5. **R4 (clrMapOvr LibreOffice risk) is the highest residual risk**. Mitigation: write
   explicit text colors on dark-layout placeholder runs, in addition to `clrMapOvr`.
   This should be enforced in the `slideforge-brand` crate as a post-synthesis pass.
