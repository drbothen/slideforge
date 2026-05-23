---
title: Enterprise Brand Template Patterns
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: architect (ADR-001 input)
---

# Brand Template Patterns — research findings

## Executive Summary

Real corporate `.pptx` templates are layered OOXML packages where a single `theme1.xml` (palette + fonts + format effects), a single `slideMaster1.xml` (chrome + text styles + clrMap), and exactly **11 default `slideLayoutN.xml` parts** are nearly universal — PowerPoint silently regenerates missing pieces and downstream tools (Keynote, LibreOffice, Google Slides) assume the standard set exists. Enterprise customizations layer **on top of** this baseline: extra custom layouts (agenda, divider, quote, contacts), a logo embedded on the slide master (occasionally suppressed on title layout), header/footer placeholders for confidentiality banners, and `<a:fld>` field codes for slide numbers. The placeholder schema is small and rigid: `title`, `ctrTitle`, `subTitle`, `body`, `pic`, `dt`, `ftr`, `sldNum`, plus the inheritance-via-`idx` model that drops formatting from layout to slide. For slideforge, the hardest correctness requirements will be (a) strict element ordering in `theme1.xml` (`clrScheme` → `fontScheme` → `fmtScheme`, in the exact 12-slot color order), (b) the `[Content_Types].xml` + `_rels` graph completeness (PowerPoint silently "repairs" or rejects malformed packages), and (c) keeping `notesMaster1.xml` + `handoutMaster1.xml` present even if unused. Mapping the 23 slideforge slide types to the 11 standard layouts requires creating ~10-12 **custom layouts** beyond the defaults — slideforge cannot fit its visual taxonomy into Microsoft's 11 alone.

## OOXML Template Anatomy

A minimal corporate `.pptx` template unzipped looks like this (paths relative to package root):

```
[Content_Types].xml                          ← MIME type registry for every part
_rels/.rels                                  ← package-level: points to ppt/presentation.xml
docProps/
  app.xml, core.xml                          ← presentation metadata (title, author, app)
ppt/
  presentation.xml                           ← slide list, master list, sldSz, defaultTextStyle
  _rels/presentation.xml.rels                ← references to slideMasters/, notesMasters/, etc.
  theme/
    theme1.xml                               ← color scheme + font scheme + format scheme
  slideMasters/
    slideMaster1.xml                         ← chrome + clrMap + text styles + layout list
    _rels/slideMaster1.xml.rels              ← rels to theme1.xml + all 11 layouts
  slideLayouts/
    slideLayout1.xml … slideLayout11.xml     ← the 11 standard layouts (see table below)
    _rels/slideLayout1.xml.rels … etc.       ← each rels back to its parent slideMaster
  notesMasters/
    notesMaster1.xml                         ← required-in-practice even if unused
    _rels/notesMaster1.xml.rels              ← rels to a notes-specific theme
  handoutMasters/
    handoutMaster1.xml                       ← required-in-practice even if unused
  media/
    image1.png                               ← embedded logo, embedded background, etc.
  slides/
    slide1.xml … slideN.xml                  ← actual content slides
    _rels/slide1.xml.rels … etc.             ← each rels back to its layout + any media
```

### theme1.xml — the palette and font definition

Element order is **strict** per ECMA-376 / ISO-29500-1 schema `CT_BaseStyles` (confirmed in c-rex.net OOXML reference, accessed 2026-05-23, https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_themeElements_topic_ID0EYXIMB.html):

```xml
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office"> ... 12 color slots in fixed order ... </a:clrScheme>
    <a:fontScheme name="Office"> <a:majorFont/> <a:minorFont/> </a:fontScheme>
    <a:fmtScheme name="Office"> ... fill / line / effect / bg styles ... </a:fmtScheme>
  </a:themeElements>
  <a:objectDefaults/>            <!-- optional -->
  <a:extraClrSchemeLst/>         <!-- optional, often empty -->
</a:theme>
```

PowerPoint will silently downgrade to a default theme if any element is out of order. Source: https://learn.microsoft.com/en-us/office/open-xml/drawingml/overview (Microsoft Learn, accessed 2026-05-23).

### slideMaster1.xml structure

The slide master XML carries (in order):
1. `<p:cSld>` containing `<p:bg>` (theme background reference) and `<p:spTree>` (the title placeholder template + the body placeholder template + any logo/chrome shapes).
2. `<p:clrMap ...>` — the color map (see "clrMap" section below).
3. `<p:sldLayoutIdLst>` listing the layouts under this master.
4. `<p:txStyles>` — the three text style hierarchies (`titleStyle`, `bodyStyle`, `otherStyle`), each with 9 levels (`<a:lvl1pPr>` … `<a:lvl9pPr>`).

Reference: http://officeopenxml.com/prSlide-styles-textStyles.php (accessed 2026-05-23).

### presentation.xml minimum

Confirmed required by python-pptx, Microsoft Open XML SDK examples, and Stack Overflow community (sources: https://learn.microsoft.com/en-us/office/open-xml/presentation/how-to-create-a-presentation-document-by-providing-a-file-name, https://daobook.github.io/python-pptx/dev/analysis/sld-slide.html, both accessed 2026-05-23):

```xml
<p:presentation
    xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
    xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
    xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>          <!-- 2^31 -->
  </p:sldMasterIdLst>
  <p:notesMasterIdLst>
    <p:notesMasterId r:id="rId2"/>
  </p:notesMasterIdLst>
  <p:handoutMasterIdLst>
    <p:handoutMasterId r:id="rId3"/>
  </p:handoutMasterIdLst>
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId4"/>                       <!-- IDs start at 256, increment -->
  </p:sldIdLst>
  <p:sldSz cx="12192000" cy="6858000" type="screen16x9"/> <!-- EMU; 16:9 widescreen -->
  <p:notesSz cx="6858000" cy="9144000"/>                  <!-- notes are portrait 4:3 -->
  <p:defaultTextStyle> ... </p:defaultTextStyle>
</p:presentation>
```

## Theme Color Scheme Conventions

The 12 color slots in `<a:clrScheme>` are defined in a **fixed sequence** by ECMA-376 schema `CT_ColorScheme`. Source: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrScheme_topic_ID0ES2FMB.html (accessed 2026-05-23).

| # | XML element  | UI label          | Common corporate use |
|---|--------------|-------------------|----------------------|
| 1 | `<a:dk1>`    | "Text/Bg Dark 1"  | Primary body text color (usually near-black) |
| 2 | `<a:lt1>`    | "Text/Bg Light 1" | Primary slide background (usually white) |
| 3 | `<a:dk2>`    | "Text/Bg Dark 2"  | Secondary text or alternate background (often brand primary) |
| 4 | `<a:lt2>`    | "Text/Bg Light 2" | Secondary background (often very light grey) |
| 5 | `<a:accent1>`| "Accent 1"        | Primary brand color — used by chart series 1, hyperlinks fallback, accent shapes |
| 6 | `<a:accent2>`| "Accent 2"        | Secondary brand color |
| 7 | `<a:accent3>`| "Accent 3"        | Tertiary brand color |
| 8 | `<a:accent4>`| "Accent 4"        | Fourth brand color (often a complementary tone) |
| 9 | `<a:accent5>`| "Accent 5"        | Fifth color (rarely customized in tight brands) |
| 10| `<a:accent6>`| "Accent 6"        | Sixth color (often a warning/highlight tone) |
| 11| `<a:hlink>`  | "Hyperlink"       | Unvisited link color |
| 12| `<a:folHlink>`| "Followed Link"  | Visited link color |

**Important**: the XML names are `dk1/lt1/dk2/lt2`, but `clrMap` (see below) remaps these to the semantic names `bg1/tx1/bg2/tx2` that drive most styling. Names like "bg1" never appear inside `clrScheme`.

### Mapping for slideforge brand.toml

A natural `brand.toml` schema:

```toml
[colors]
text_primary    = "#1A1A1A"   # → dk1
background      = "#FFFFFF"   # → lt1
text_secondary  = "#003366"   # → dk2
surface_alt     = "#F5F5F5"   # → lt2
brand_primary   = "#0066CC"   # → accent1
brand_secondary = "#00A3A1"   # → accent2
accent_3        = "#FFC700"   # → accent3
accent_4        = "#FF6F61"   # → accent4
accent_5        = "#7B5BC7"   # → accent5
accent_6        = "#3FAA52"   # → accent6
hyperlink       = "#0563C1"   # → hlink
hyperlink_visited = "#954F72" # → folHlink
```

This 1:1 mapping is the simplest design; slideforge could also expose `bg1/tx1` semantic aliases that auto-fill `dk1/lt1` via the default clrMap.

## Font Scheme Conventions

`<a:fontScheme>` contains exactly `majorFont` (used for title/heading styles via `+mj-lt`, `+mj-ea`, `+mj-cs` references) and `minorFont` (used for body via `+mn-lt`, `+mn-ea`, `+mn-cs`). Source: ECMA-376 `CT_FontCollection` + https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_fontScheme_topic_ID0E6F4JB.html (accessed 2026-05-23).

```xml
<a:fontScheme name="Slideforge">
  <a:majorFont>
    <a:latin typeface="Inter"/>           <!-- required: Latin scripts -->
    <a:ea typeface=""/>                   <!-- optional: East Asian (CJK) -->
    <a:cs typeface=""/>                   <!-- optional: Complex Script (Arabic/Hebrew/Thai) -->
    <a:font script="Jpan" typeface="Yu Gothic"/>   <!-- script-specific overrides -->
    <a:font script="Hang" typeface="Malgun Gothic"/>
    <a:font script="Hans" typeface="DengXian"/>
    <a:font script="Arab" typeface="Traditional Arabic"/>
  </a:majorFont>
  <a:minorFont>
    <a:latin typeface="Inter"/>
    <a:ea typeface=""/>
    <a:cs typeface=""/>
  </a:minorFont>
</a:fontScheme>
```

**Modern Microsoft default**: As of 2024, the Office Theme default is `majorFont/latin = "Aptos"`, `minorFont/latin = "Aptos Narrow"` (changed from Calibri). Source: https://support.microsoft.com/en-us/office/new-office-theme-e7bbfe02-d1fb-4c4d-b3b7-6a47f0cefd3f (accessed 2026-05-23).

**Theme font references in placeholder text**: Real templates almost never hard-code a typeface in a placeholder. Instead they reference the theme: `<a:latin typeface="+mn-lt"/>` (minor font, latin) or `<a:latin typeface="+mj-lt"/>` (major font, latin). This is how a single typeface change in `theme1.xml` propagates through every slide.

**slideforge implication**: Inter, Calibri, and Aptos all work as the latin typeface. The brand.toml should expose `[fonts] heading = "Inter"; body = "Inter"` and slideforge generates the `<a:latin typeface="..."/>` references. CJK/RTL fallbacks should be optional second-tier fields.

## Standard 11 Layouts (the "default" set Office assumes exists)

These 11 layouts are the canonical Office set; every renderer (Keynote, Google Slides, LibreOffice) silently falls back to them. The placeholder `type` attribute is governed by ECMA-376, but per-layout `idx` values are theme-specific and must be discovered by extracting a real .pptx. **No single Microsoft document publishes the complete idx mapping** — see Perplexity research dated 2026-05-23.

| Layout # | Name | Placeholders (type) | Typical Use |
|----------|------|---------------------|-------------|
| 1  | Title Slide              | `ctrTitle` + `subTitle`                                    | Opening cover slide; centered title, subtitle below |
| 2  | Title and Content        | `title` + `body` (single content area)                     | Workhorse layout: title + bullets/chart/table/picture |
| 3  | Section Header           | `title` (or `ctrTitle`) + `body`/`subTitle` (intro text)   | Divider between sections in a long deck |
| 4  | Two Content              | `title` + `body` + `body` (side-by-side)                   | Two side-by-side text/content blocks |
| 5  | Comparison               | `title` + 4× `body` (2 headers + 2 bodies)                 | Side-by-side compare/contrast |
| 6  | Title Only               | `title` (no content placeholder)                           | Title bar + user draws shapes manually below |
| 7  | Blank                    | (no placeholders at all)                                   | Empty canvas; full creative freedom |
| 8  | Content with Caption     | 2× `body` (caption + main content)                         | Text caption on one side, content on the other |
| 9  | Picture with Caption     | `body` (caption) + `pic` (picture placeholder)             | Picture frame + caption text |
| 10 | Title and Vertical Text  | `title` + `body` (with `vert="eaVert"` orient)             | Vertical East Asian–style body text |
| 11 | Vertical Title and Text  | `title` (vert) + `body` (vert)                             | Both title and body rendered vertically |

Sources: Microsoft Support https://support.microsoft.com/en-us/office/add-edit-or-remove-a-placeholder-on-a-slide-layout-a8d93d28-66cb-43fd-9f9d-e12d0a7a1f06 (accessed 2026-05-23); confirmed by ECMA-376 PresentationML `CT_SlideLayout` enum; placeholder structure verified via MathWorks Open XML reference https://www.mathworks.com/help/rptgen/ug/access-powerpoint-template-elements.html (accessed 2026-05-23).

### Placeholder type attribute values (ECMA-376 enum)

Confirmed from the PresentationML schema `ST_PlaceholderType`:

| `type` value | Purpose | Notes |
|--------------|---------|-------|
| `title`     | Standard title             | Top of slide, single line |
| `ctrTitle`  | Centered title             | Title-slide convention |
| `subTitle`  | Subtitle                   | Appears under `ctrTitle` |
| `body`      | Body / content placeholder | Generic; also used for "content" placeholders with icon menu |
| `pic`       | Picture placeholder        | Auto-frames inserted images |
| `chart`     | Chart placeholder          | Rarely used; usually `body` with chart inserted |
| `tbl`       | Table placeholder          | Rarely used; usually `body` with table |
| `dgm`      | SmartArt diagram           | Rare |
| `media`     | Audio/video placeholder    | Rare |
| `dt`        | Date placeholder           | Footer line; pairs with `<a:fld type="datetime">` |
| `ftr`       | Footer placeholder         | Plain text footer |
| `sldNum`    | Slide-number placeholder   | Pairs with `<a:fld type="slidenum">` |
| `hdr`       | Header (notes only)        | Notes pages, not slides |

## Common Custom Layouts in Enterprise Templates

Microsoft's Copilot brand-template guidance (https://support.microsoft.com/en-us/topic/keep-your-presentation-on-brand-with-copilot-046c23d5-012e-49e0-8579-fe49302959fc, accessed 2026-05-23) recommends enterprises **add the following beyond the 11 defaults**:

| Custom layout name           | Typical placeholders                       | Maps to slideforge type |
|------------------------------|--------------------------------------------|-------------------------|
| Agenda                       | `title` + `body` (numbered list)           | (no slideforge equivalent — agenda is content) |
| Section Header / Divider     | `ctrTitle` over a colored full-bleed bg    | `divider` |
| Data Visualization Dashboard | `title` + 2–4× `body` (chart placeholders) | `status`, `progress_bar`, `metric_tree` |
| Timelines                    | `title` + structured shapes                | `vertical_timeline`, `horizontal_timeline` |
| Process / Flow / List        | `title` + numbered card grid               | `numbered_actions`, `card_rows` |
| Quote / Statement            | `ctrTitle` large-italic + `subTitle` attr  | (potential new slideforge type) |
| Q&A                          | `title` only                               | (use `title-only`) |
| Summary / Takeaways          | `title` + `body` (3–5 key points)          | (use `content`) |
| Conclusion / Thank You       | full-bleed brand color + small text        | `end` |
| Team / Bio                   | `title` + `pic` + `body` (per person)      | (potential new type) |
| Contacts                     | `title` + `body` (contact card grid)       | (no slideforge equivalent) |
| Case Study                   | `title` + `pic` + 2× `body`                | (potential new type) |

This list is informative — slideforge does **not** need to provide all of them, but corporate users **will** expect Section Divider, Agenda, Quote, Thank You, and Dashboard layouts to be present and brand-consistent.

## Logo / Chrome Embedding Patterns

**Where logos live** (confirmed via Microsoft Support https://support.microsoft.com/en-us/office/customize-a-slide-master-036d317b-3251-4237-8ddc-22f4668e2b56, accessed 2026-05-23):

- **Slide Master**: the canonical location. Logo added once, appears on every slide. Image stored at `ppt/media/imageN.png`, related from `ppt/slideMasters/_rels/slideMaster1.xml.rels` with `Type=".../image"`.
- **Slide Layout (override)**: corporate convention — **suppress the logo on the title layout** by either (a) putting a white/brand-color rectangle over it on that layout, or (b) having a separate "title" master with no logo. Easier in slideforge: define logo at master level only on layouts that need it (a per-layout decision in the synthesis code).
- **Individual slide**: discouraged; defeats brand consistency.

**Typical logo position** (corporate examples): top-right corner, ~0.5" tall, ~10% from right edge. In EMU at 16:9 (12192000 × 6858000):
- x: ~10800000 EMU (about 11.8" from left, 0.5" from right edge assuming 1" logo width)
- y: ~228600 EMU (0.25" from top)
- cx: ~914400 EMU (1" wide)
- cy: ~457200 EMU (0.5" tall)

**Image relationship pattern**:
```xml
<!-- ppt/slideMasters/_rels/slideMaster1.xml.rels -->
<Relationship Id="rId10" 
              Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
              Target="../media/image1.png"/>

<!-- inside slideMaster1.xml spTree -->
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="100" name="Corporate Logo"/>
    <p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr>
    <p:nvPr/>
  </p:nvPicPr>
  <p:blipFill>
    <a:blip r:embed="rId10"/>
    <a:stretch><a:fillRect/></a:stretch>
  </p:blipFill>
  <p:spPr>
    <a:xfrm><a:off x="10800000" y="228600"/><a:ext cx="914400" cy="457200"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
  </p:spPr>
</p:pic>
```

**Slide number as field code** (confirmed via https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.field accessed 2026-05-23):

```xml
<p:sp>
  <p:nvSpPr>
    <p:cNvPr id="4" name="Slide Number Placeholder"/>
    <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
    <p:nvPr><p:ph type="sldNum" sz="quarter" idx="3"/></p:nvPr>
  </p:nvSpPr>
  <p:spPr/>
  <p:txBody>
    <a:bodyPr/><a:lstStyle/>
    <a:p>
      <a:fld id="{GUID-HERE}" type="slidenum">
        <a:rPr lang="en-US"/>
        <a:t>‹#›</a:t>
      </a:fld>
    </a:p>
  </p:txBody>
</p:sp>
```

**Confidentiality banners**: in corporate templates these are either (a) a fixed shape on the master, (b) the `ftr` placeholder text driven by header/footer dialog, or (c) **multiple slide masters** — one per classification level (Public, Internal, Confidential, Restricted). Source: https://learn.microsoft.com/en-us/answers/questions/5194270/can-i-add-automated-data-classification-to-a-power (accessed 2026-05-23). Slideforge v1 should support a `confidentiality_text` field in brand.toml that becomes a master-level text shape.

## Slide → Layout → Master → Theme Inheritance

The inheritance chain (formatting cascade):

```
theme1.xml       ← defines palette, font scheme, format scheme
   ↓
slideMaster1.xml ← defines clrMap, body/title/other text styles (9 levels each), chrome shapes
   ↓
slideLayoutN.xml ← inherits master text styles; can override clrMap via <p:clrMapOvr>; defines placeholder positions
   ↓
slide.xml        ← inherits layout placeholders by matching ph idx; can override anything per shape
```

**The `<p:clrMap>` element** (the key inheritance hinge) — confirmed via http://officeopenxml.com/prSlide-styles-textStyles.php and https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrMap_topic_ID0ETDFMB.html (both accessed 2026-05-23):

```xml
<!-- standard mapping in slideMaster1.xml -->
<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2"
          accent1="accent1" accent2="accent2" accent3="accent3"
          accent4="accent4" accent5="accent5" accent6="accent6"
          hlink="hlink" folHlink="folHlink"/>
```

This says: "for slides under this master, when something asks for color `bg1` (background 1), look up `lt1` in the theme's `clrScheme`." A layout can override this with `<p:clrMapOvr>` to flip — e.g., a dark-background section divider layout might map `bg1="dk1"` and `tx1="lt1"` so the same theme produces dark slides for that one layout.

**When to override**:
- Override per-shape (in slide XML) for: one-off colors, custom fonts on a callout, specific bullet styling.
- Override per-layout (clrMapOvr): when a layout has a fundamentally different background/text relationship (dark-themed divider, accent-themed cover).
- Override per-master: when supporting multiple brand variants (Public vs Confidential as separate masters).
- Inherit by default: any normal title/body styling — let it flow from master.txStyles.

**For slideforge synthesis**: the dark "section divider" and "end" slides MUST use a layout with `<p:clrMapOvr bg1="dk2" tx1="lt1" .../>` (or similar) so they inherit the brand color as background while keeping accessibility-safe light text.

## Mapping slideforge 23 slide types → standard layouts

This is tentative; the architect locks it in S5/ADR-001.

| slideforge type        | Maps to standard 11? | Custom layout needed? | Notes |
|------------------------|----------------------|------------------------|-------|
| `title` / `divider`    | #3 Section Header (modified) | YES — "Section Divider" with `clrMapOvr` | Full-bleed brand color + centered title; ADR-001 must define the override pattern |
| `content`              | #2 Title and Content | No (use as-is) | The workhorse; baseline bullets |
| `two_column`           | #4 Two Content       | No (use as-is) | |
| `content_stat`         | (none)               | YES — "Content + Stat Card" | `title` + `body` (left) + `body` styled as stat (right) |
| `stat_callout` / `key_metrics` | (none)        | YES — "Stat Grid"          | `title` + 2-6 `body` slots in grid |
| `stats_summary`        | (none)               | YES — "Stats + Summary"    | `title` + `body` (effort) + `body` (summary band) |
| `highlight`            | (none)               | YES — "Highlight Callout"  | `title` + `body` (callout box) + `body` (supporting bullets) |
| `highlight_boxes`      | (none)               | YES — "Two Boxes Stacked"  | `title` + 2× full-width `body` |
| `split_contrast`       | #5 Comparison (modified) | YES (variant)          | Different visual; comparison is too text-heavy |
| `card_rows`            | (none)               | YES — "Card Grid"          | `title` + 2-column checklist of `body` shapes |
| `severity_cards`       | (none)               | YES — "Severity Grid"      | `title` + grid with colored badges |
| `numbered_actions`     | (none)               | YES — "Numbered Cards"     | |
| `vertical_timeline`    | (none)               | YES — "Vertical Timeline"  | |
| `horizontal_timeline`  | (none)               | YES — "Horizontal Timeline"| |
| `enhanced_table`       | (none)               | YES — "Table (Enhanced)"   | `title` + `tbl` |
| `table`                | (none)               | YES — "Table (Plain)"      | |
| `status`               | (none)               | YES — "Status Dashboard"   | |
| `progress_bar`         | (none)               | YES — "Progress Bar"       | |
| `metric_tree`          | (none)               | YES — "Metric Tree"        | |
| `formula`              | (none)               | YES — "Formula"            | |
| `weighted_composite`   | (none)               | YES — "Weighted Composite" | |
| `end`                  | #6 Title Only or #1 Title Slide (with `clrMapOvr`) | YES — "End Slide" | Full-bleed brand color closing slide |

**Architect's decision required**: ADR-001 must specify whether slideforge generates:
- (a) **All 11 standard layouts + ~20 custom layouts** (33 total) — maximum compatibility, larger output
- (b) **Only the standard 11 + a slim custom set** (15-ish) — and use slide-level overrides for the rest
- (c) **Custom layouts only matching the 23 types** — smaller, more focused, but loses "open in PowerPoint and pick from familiar layouts" UX

**Recommendation from this research**: (a) — generate all 11 standards (cheap; mostly empty placeholders) plus one custom layout per slideforge type. This maximizes Keynote/Google-Slides rendering parity because those tools special-case the standard layouts.

## Common Pitfalls in From-Scratch .pptx Generation

Confirmed via Stack Overflow https://stackoverflow.com/questions/37353189/create-powerpoint-using-open-xml-sdk, python-pptx GitHub https://github.com/scanny/python-pptx/issues/108, https://github.com/scanny/python-pptx/issues/140, and Microsoft Learn https://learn.microsoft.com/en-us/office/open-xml/presentation/how-to-create-a-presentation-document-by-providing-a-file-name (all accessed 2026-05-23).

### Top pitfalls

1. **`[Content_Types].xml` must register every part type used.** Missing an override (e.g., notesSlide ContentType) causes "PowerPoint found a problem with content" repair dialog. Required ContentTypes include theme, slideMaster, slideLayout, slide, notesMaster, notesSlide, handoutMaster, presentationProperties, viewProperties, tableStyles, plus image MIME types as defaults.

2. **`notesMaster1.xml` and `handoutMaster1.xml` are nominally optional but PowerPoint expects them.** Omitting them causes silent "repair" on open. Even unused, slideforge must generate trivial-but-valid stubs.

3. **Theme XML element ordering is strict.** `<a:clrScheme>` → `<a:fontScheme>` → `<a:fmtScheme>` inside `<a:themeElements>`. The 12 color slots inside `<a:clrScheme>` must appear in fixed order: `dk1, lt1, dk2, lt2, accent1..6, hlink, folHlink`. Wrong order → PowerPoint silently resets to default theme.

4. **Namespace declarations** must be on every root element. Common namespaces:
   - `xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"`
   - `xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"`
   - `xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"`
   - Plus `xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main"` for extLst features like `assocLst`.

5. **Slide ID conventions**: in `presentation.xml`, `<p:sldId id="...">` IDs start at **256** and increment by 1 per slide. `<p:sldMasterId id="...">` starts at **2147483648** (2^31). These are spec-mandated minimums; deviating causes opaque errors.

6. **Relationship ID scoping**: `rId1` in `/_rels/.rels` is independent of `rId1` in `/ppt/_rels/presentation.xml.rels` is independent of `rId1` in `/ppt/slides/_rels/slide1.xml.rels`. Uniqueness only within a single `.rels` file. Cross-referencing is by Target attribute, not by global ID.

7. **`r:id` vs `id` confusion**: `<p:sldId id="256" r:id="rId4"/>` — `id="256"` is the slide's stable identity within the presentation; `r:id="rId4"` is the relationship pointing to `slides/slideN.xml`. Easy to swap by accident.

8. **Layout-to-master rels**: each `slideLayoutN.xml` has its own `_rels/slideLayoutN.xml.rels` that points back to `slideMaster1.xml` with `Type="...slideMaster"`. Many homemade generators forget this back-pointer, causing PowerPoint to "lose" the layout.

9. **theme rels graph**: slideMaster1.xml → theme1.xml via `Type="...theme"`. notesMaster typically references its own theme part (often theme2.xml) — though slideforge can share theme1.xml across both.

10. **Image part naming**: `media/image1.png` not `media/logo.png`. PowerPoint accepts the latter but some downstream tools (LibreOffice older versions, Google Slides) struggle with non-conventional names. Use the convention.

11. **`<p:defaultTextStyle>` is "optional per spec" but in practice required**. Omitting it leaves slides with unstyled fallback fonts. Always include at least minimal levels.

12. **EMU coordinate space**: 914400 EMU = 1 inch; 12700 EMU = 1 point. The 16:9 slide is `cx=12192000 cy=6858000` (= 13.333" × 7.5"); 4:3 is `cx=9144000 cy=6858000` (= 10" × 7.5"). Notes pages are portrait 4:3 (`cx=6858000 cy=9144000`). Sources: https://learn.microsoft.com/en-us/typography/font-list/aptos, https://www.lauramfoley.com/slide-sizes/ (accessed 2026-05-23).

13. **Layout list in master**: `<p:sldLayoutIdLst>` must enumerate every layout under that master with valid relationship IDs. PowerPoint regenerates this on save if inconsistent, but other renderers may not.

## Accessibility-relevant Template Properties

Confirmed via WebAIM https://webaim.org/techniques/powerpoint/, Government of Canada https://a11y.canada.ca/en/accessible-powerpoint-presentations-in-microsoft-365/, Penn State https://accessibility.psu.edu/microsoftoffice/powerpoint/slidemaster/ (all accessed 2026-05-23).

The Microsoft PowerPoint Accessibility Checker flags:

1. **Missing alt text on images and shapes**. Logos and decorative shapes on the slide master that don't have alt text trigger a warning **for every slide** in the deck. Fix: mark decorative items as `decorative="1"` on the `<p:nvSpPr>` extLst (`<a:ext uri="...">` carrying `<adec:decorative val="1"/>`).

2. **Missing slide titles**. Slides without a `title` or `ctrTitle` placeholder trigger a warning. Every slideforge layout (except blank) MUST have a title placeholder.

3. **Duplicate slide titles**. The checker warns when two slides have identical title text. Slideforge can't prevent this in synthesis, but layout naming should not encourage it (don't name layouts with hard-coded placeholder text).

4. **Reading order issues**. PowerPoint uses the bottom-to-top order in the Selection Pane as reading order. Slideforge synthesis must arrange `<p:spTree>` children in the **intended reading order** (top-to-bottom in XML = bottom-to-top in pane = reading order). Practical rule: title first in spTree, then body content top-to-bottom, then chrome (logo, page number) last.

5. **Color contrast**. The Accessibility Checker does NOT do WCAG contrast math, but Microsoft guidance strongly recommends WCAG AA ratios (4.5:1 for body text, 3:1 for large text). Slideforge brand.toml validation should warn when accent-on-background combinations fall below 4.5:1.

6. **Descriptive placeholder names** (via Selection Pane). The `<p:cNvPr name="...">` attribute is what screen readers and the Selection Pane show. Slideforge should set semantic names: `name="Title 1"`, `name="Body 1"`, `name="Stat Card 1"`, NOT `name="Shape 5"`.

7. **Notes pages with content**. Notes slides should have a notes placeholder with actual content for the visually impaired. Slideforge should generate empty-but-present notes placeholders so authors can fill them.

These directly feed **ADR-011 (WCAG AA tooling)** and **R5 (accessibility risks)**.

## Sources

- ECMA-376 / ISO-29500-1 Office Open XML (PDF mirror): https://web.mit.edu/~stevenj/www/ECMA-376-new-merged.pdf (accessed 2026-05-23)
- ECMA-376 spec page: https://ecma-international.org/publications-and-standards/standards/ecma-376/ (accessed 2026-05-23)
- ECMA-376 5th edition GitHub mirror: https://github.com/QtExcel/ecma-376-5th (accessed 2026-05-23)
- Microsoft Learn — Structure of a PresentationML document: https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document (accessed 2026-05-23)
- Microsoft Learn — Working with presentations: https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-presentations (accessed 2026-05-23)
- Microsoft Learn — Create a presentation document by providing a file name: https://learn.microsoft.com/en-us/office/open-xml/presentation/how-to-create-a-presentation-document-by-providing-a-file-name (accessed 2026-05-23)
- Microsoft Learn — DrawingML overview: https://learn.microsoft.com/en-us/office/open-xml/drawingml/overview (accessed 2026-05-23)
- Microsoft Learn — DocumentFormat.OpenXml.Drawing.Field (a:fld): https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.field (accessed 2026-05-23)
- Microsoft Support — Customize a slide master: https://support.microsoft.com/en-us/office/customize-a-slide-master-036d317b-3251-4237-8ddc-22f4668e2b56 (accessed 2026-05-23)
- Microsoft Support — Add/edit/remove a placeholder on a slide layout: https://support.microsoft.com/en-us/office/add-edit-or-remove-a-placeholder-on-a-slide-layout-a8d93d28-66cb-43fd-9f9d-e12d0a7a1f06 (accessed 2026-05-23)
- Microsoft Support — Keep your presentation on-brand with Copilot (custom layouts list): https://support.microsoft.com/en-us/topic/keep-your-presentation-on-brand-with-copilot-046c23d5-012e-49e0-8579-fe49302959fc (accessed 2026-05-23)
- Microsoft Support — New Office theme (Aptos): https://support.microsoft.com/en-us/office/new-office-theme-e7bbfe02-d1fb-4c4d-b3b7-6a47f0cefd3f (accessed 2026-05-23)
- Microsoft Learn Q&A — Classification banner approach: https://learn.microsoft.com/en-us/answers/questions/5194270/can-i-add-automated-data-classification-to-a-power (accessed 2026-05-23)
- Microsoft Typography — Aptos font: https://learn.microsoft.com/en-us/typography/font-list/aptos (accessed 2026-05-23)
- Microsoft Open Specifications — MS-OE376: https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/db9b9b72-b10b-4e7e-844c-09f88c972219 (accessed 2026-05-23)
- C-REX OOXML reference — clrScheme: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrScheme_topic_ID0ES2FMB.html (accessed 2026-05-23)
- C-REX OOXML reference — fontScheme: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_fontScheme_topic_ID0E6F4JB.html (accessed 2026-05-23)
- C-REX OOXML reference — themeElements: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_themeElements_topic_ID0EYXIMB.html (accessed 2026-05-23)
- C-REX OOXML reference — clrMap: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrMap_topic_ID0ETDFMB.html (accessed 2026-05-23)
- C-REX OOXML reference — overrideClrMapping: https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_overrideClrMapping_topic_ID0ELVHMB.html (accessed 2026-05-23)
- OfficeOpenXML.com — text styles: http://officeopenxml.com/prSlide-styles-textStyles.php (accessed 2026-05-23)
- python-pptx documentation: https://python-pptx.readthedocs.io (accessed 2026-05-23)
- python-pptx dev — slide ID analysis: https://daobook.github.io/python-pptx/dev/analysis/sld-slide.html (accessed 2026-05-23)
- python-pptx dev — relationships: https://python-pptx.readthedocs.io/en/latest/dev/resources/about_relationships.html (accessed 2026-05-23)
- python-pptx issues — handout master: https://github.com/scanny/python-pptx/issues/108 (accessed 2026-05-23)
- python-pptx issues — theme support: https://github.com/scanny/python-pptx/issues/140 (accessed 2026-05-23)
- Stack Overflow — minimal pptx with Open XML SDK: https://stackoverflow.com/questions/37353189/create-powerpoint-using-open-xml-sdk (accessed 2026-05-23)
- Stack Overflow — notes master Open XML SDK: https://stackoverflow.com/questions/47318824/notes-master-open-xml-sdk (accessed 2026-05-23)
- ListenLabs blog — reverse-engineering PowerPoint XML: https://listenlabs.ai/blog/ppt-generator (accessed 2026-05-23)
- Slide sizes reference: https://www.lauramfoley.com/slide-sizes/ (accessed 2026-05-23)
- Microsoft 365 blog — aspect ratios 16:9 vs 4:3: https://www.microsoft.com/en-us/microsoft-365/blog/2010/06/23/ready-for-widescreen-how-to-manage-aspect-ratios-in-powerpoint-169-vs-43/ (accessed 2026-05-23)
- WebAIM — PowerPoint accessibility: https://webaim.org/techniques/powerpoint/ (accessed 2026-05-23)
- Government of Canada — Accessible PowerPoint in Microsoft 365: https://a11y.canada.ca/en/accessible-powerpoint-presentations-in-microsoft-365/ (accessed 2026-05-23)
- Penn State — Accessible slide master: https://accessibility.psu.edu/microsoftoffice/powerpoint/slidemaster/ (accessed 2026-05-23)
- BrightCarbon — accessible placeholder names: https://www.brightcarbon.com/blog/powerpoint-placeholder-names-accessible/ (accessed 2026-05-23)
- Section508.gov — creating PowerPoint templates: https://www.section508.gov/training/presentations/creating-powerpoint-templates/ (accessed 2026-05-23)
- MathWorks — Access PowerPoint template elements: https://www.mathworks.com/help/rptgen/ug/access-powerpoint-template-elements.html (accessed 2026-05-23)
- VerdanaBold — PowerPoint OOXML training: https://www.verdanabold.com/post/powerpoint-ooxml-training (accessed 2026-05-23)
- EchosVoice — adding color schemes to a THMX: https://echosvoice.com/wp-content/uploads/pptfiles/addcolorschemes2.pdf (accessed 2026-05-23)
- data2type — CT_BaseStyles fontScheme: https://repository.data2type.de/OOXML/v_2006/html/el.CT_BaseStyles_fontScheme.html (accessed 2026-05-23)
- pptcrafter — masters/layouts/placeholders: https://pptcrafter.wordpress.com/2019/12/03/powerpoint-secrets-slide-masters-layouts-and-placeholders/ (accessed 2026-05-23)
- Axes4 — slide masters and layouts: https://support.axes4.com/hc/en-us/articles/16173198594450-Slide-Masters-and-Layouts (accessed 2026-05-23)

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | Deep multi-source investigation of (a) theme/master/layout XML structure + 11 standard layouts + placeholder schema, (b) corporate logo/banner/RTL/Office defaults/accessibility patterns |
| Perplexity perplexity_ask | 5 | Specific schema-order questions (theme element order, clrScheme 12 slots, fontScheme structure); the 11 standard layouts with placeholder types/idx; from-scratch generation gotchas; clrMap inheritance + accessibility checker behavior; Office default theme + EMU dimensions + RTL/field-code patterns |
| Tavily tavily_search | 1 | Enterprise custom layout conventions (agenda, divider, quote) |
| Context7 | 0 | Not used — research is spec/XML focused, not library API focused |
| WebFetch | 0 | Not used — Perplexity returned full-text citations |
| WebSearch | 0 | Not used — superseded by Perplexity/Tavily |
| Read (local) | 2 | Verified slideforge's 23 slide types and decisions-applied context |
| Training data | 1 area | EMU constants (914400/inch, 12700/point) and standard 4:3 / 16:9 dimensions cross-checked against external sources |

**Total MCP tool calls:** 8
**Training data reliance:** **low** — every non-trivial claim was sourced from ECMA-376, Microsoft Learn, c-rex.net, OfficeOpenXML.com, python-pptx docs, WebAIM, or comparable independent sources. The only training-data-derived facts are well-known constants (EMU per inch, standard slide dimensions) and these were cross-verified against at least one cited source.

**Inconclusive areas explicitly flagged:**
- Per-layout placeholder `idx` numerical values are **not published authoritatively** by Microsoft for the 11 standard layouts — they vary by theme. Slideforge's `extract-brand` command must read them from each input .pptx; the synthesis command can pick any consistent numbering.
- No single canonical mapping table exists from "layout name" → exact placeholder schema. The Title and Content layout, for example, is type=`title` + type=`body` in nearly all cases, but the `idx` and exact `<a:bodyPr>` defaults vary.
- The exact list of `[Content_Types].xml` overrides PowerPoint "expects" is not formally specified — community consensus and Open XML SDK examples are the practical reference, not a single Microsoft spec section.
