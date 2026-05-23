---
title: OOXML PresentationML Foundations
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: architect (ADR-001 input)
companion_to: brand-template-patterns.md
---

# OOXML PresentationML Foundations

## Executive Summary

A `.pptx` is an OPC (Open Packaging Conventions) ZIP archive whose logical structure is a directed graph of "parts" connected by relationship (`.rels`) files. ECMA-376 Part 1 (PresentationML) defines the XML; ECMA-376 Part 2 (OPC, ISO/IEC 29500-2) defines the packaging. For slideforge to **synthesize** a valid `.pptx` from a brand.toml (no base file), the architect must internalize five non-negotiables:

1. **Every part needs both a content type (in `[Content_Types].xml`) and at least one inbound relationship (in some `.rels` file).** Orphan parts and dangling rIds both cause "PowerPoint found a problem with content" repairs.
2. **Element order inside theme/master/layout/slide XML is schema-significant.** `<a:clrScheme>` must contain exactly 12 colors in the order `dk1, lt1, dk2, lt2, accent1..accent6, hlink, folHlink`. `<p:sldMaster>` must contain `cSld` → `clrMap` → `sldLayoutIdLst` → (optional `transition`, `timing`, `hf`) → `txStyles` in that order.
3. **EMU is the only positioning unit and everything must be an integer.** 914,400 EMU = 1 in = 2.54 cm; 360,000 EMU = 1 cm; 12,700 EMU = 1 point. Slide width must precisely match aspect ratio (16:9 = 12,192,000 × 6,858,000 = 13.333" × 7.5" in modern Office; 9,144,000 × 5,143,500 in some templates; 4:3 = 9,144,000 × 6,858,000).
4. **IDs in three name-spaces must never collide.** `sldId` ≥ 256 (sequential, unique, never reused), `sldMasterId` and `sldLayoutId` ≥ 2,147,483,648 (0x80000000) per the de-facto Microsoft convention. `rId` strings are scoped per-.rels file and conventionally `rId1..rIdN`.
5. **`clrMap` is the bridge between theme abstract colors and slide semantic colors.** Slide content uses `<a:schemeClr val="bg1"/>` etc.; the master's `<p:clrMap>` rewrites `bg1→lt1`, `tx1→dk1`, etc. A layout may override with `<p:clrMapOvr><a:overrideClrMapping .../></p:clrMapOvr>` for dark-themed dividers.

The ooxmlsdk Rust crate ([/kaisery/ooxmlsdk](https://github.com/kaisery/ooxmlsdk)) provides typed Rust structs for every PresentationML element and round-trips bytes-to-bytes; it does NOT do layout, schema validation, or "make me a valid empty deck" scaffolding. Synthesis is "construct the IR tree, hand it to ooxmlsdk to serialize, write the ZIP."

---

## Minimum Viable .pptx — File Inventory

A bare-minimum **structurally valid** `.pptx` that PowerPoint will open without "repair" prompts requires the following parts. Optional in spec but expected-by-renderers items are flagged `[de facto required]`.

| Path | Content Type | Inbound rel from | Purpose |
|---|---|---|---|
| `[Content_Types].xml` | (the registry itself) | — | Maps every part to its MIME-style content type |
| `_rels/.rels` | `application/vnd.openxmlformats-package.relationships+xml` | (root) | Package-level rels; must point to `ppt/presentation.xml` via type `.../officeDocument` |
| `ppt/presentation.xml` | `application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml` | `_rels/.rels` | Root presentation part — slide/master/layout lists, slide size, default text style |
| `ppt/_rels/presentation.xml.rels` | `application/vnd.openxmlformats-package.relationships+xml` | — | rels from presentation.xml to masters, theme, slides, notesMaster, presProps, viewProps, tableStyles |
| `ppt/theme/theme1.xml` | `application/vnd.openxmlformats-officedocument.theme+xml` | presentation.xml.rels (`.../theme`) AND slideMaster1.xml.rels | Color/font/format scheme |
| `ppt/slideMasters/slideMaster1.xml` | `application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml` | presentation.xml.rels (`.../slideMaster`) | Master shapes, clrMap, sldLayoutIdLst, txStyles |
| `ppt/slideMasters/_rels/slideMaster1.xml.rels` | (relationships) | — | rels from master to its layouts + theme |
| `ppt/slideLayouts/slideLayout1.xml` | `application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml` | slideMaster1.xml.rels (`.../slideLayout`) | At least ONE layout — sldMaster is invalid with empty layout list |
| `ppt/slideLayouts/_rels/slideLayout1.xml.rels` | (relationships) | — | Back-pointer to slideMaster1.xml |
| `ppt/slides/slide1.xml` | `application/vnd.openxmlformats-officedocument.presentationml.slide+xml` | presentation.xml.rels (`.../slide`) | A slide is technically optional but most renderers reject "0-slide" decks |
| `ppt/slides/_rels/slide1.xml.rels` | (relationships) | — | Slide → its layout |
| `ppt/presProps.xml` | `application/vnd.openxmlformats-officedocument.presentationml.presProps+xml` | presentation.xml.rels (`.../presProps`) | **[de facto required]** Presentation-show properties; PowerPoint regenerates if missing but Keynote complains |
| `ppt/viewProps.xml` | `application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml` | presentation.xml.rels (`.../viewProps`) | **[de facto required]** Authoring view properties |
| `ppt/tableStyles.xml` | `application/vnd.openxmlformats-officedocument.presentationml.tableStyles+xml` | presentation.xml.rels (`.../tableStyles`) | **[de facto required]** Empty `<a:tblStyleLst>` is fine; some tools crash on missing |
| `ppt/notesMasters/notesMaster1.xml` | `application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml` | presentation.xml.rels (`.../notesMaster`) | **[optional spec / de facto required]** If a slide has a notes part, this MUST exist |
| `ppt/handoutMasters/handoutMaster1.xml` | `application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml` | presentation.xml.rels (`.../handoutMaster`) | **[optional]** PowerPoint creates on demand; safe to omit |
| `docProps/core.xml` | `application/vnd.openxmlformats-package.core-properties+xml` | `_rels/.rels` (`.../core-properties`) | **[de facto required]** Dublin Core metadata — title, creator, modified |
| `docProps/app.xml` | `application/vnd.openxmlformats-officedocument.extended-properties+xml` | `_rels/.rels` (`.../extended-properties`) | **[de facto required]** App identification — `<Application>slideforge</Application>` |

Sizes for an empty `slideforge`-generated deck (rough): theme1 ~6 KB, slideMaster1 ~4 KB, each slideLayout ~1.5–2 KB, slide1 ~600 B, the rest <500 B each. An empty 1-layout deck ZIPs to ~15–18 KB.

**Slideforge baseline** (from R2 / brand-template-patterns.md): generate **11 standard layouts + 20 custom layouts** (one per slideforge slide type beyond the 11 standards), notesMaster1, handoutMaster1, presProps/viewProps/tableStyles, docProps. That is ~36 XML parts plus 1–2 image media (logo).

Sources: ECMA-376 Part 1 §13.3 (PresentationML), Microsoft Learn "Structure of a PresentationML document" (https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document, accessed 2026-05-23); ooxmlsdk pml_hierarchy.md (https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_hierarchy.md, accessed 2026-05-23).

---

## XML Namespace Boilerplate

Every PresentationML XML part begins with the XML declaration `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` (the `standalone="yes"` is non-negotiable for Office round-trip safety), followed by a single root element whose namespace declarations are listed below.

| Prefix | URI | Used For |
|---|---|---|
| `p` | `http://schemas.openxmlformats.org/presentationml/2006/main` | PresentationML elements: `<p:presentation>`, `<p:sldMaster>`, `<p:sld>`, `<p:sp>`, `<p:ph>`, `<p:clrMap>` |
| `a` | `http://schemas.openxmlformats.org/drawingml/2006/main` | DrawingML — shapes, colors, fills, text runs: `<a:theme>`, `<a:clrScheme>`, `<a:p>`, `<a:r>`, `<a:rPr>`, `<a:srgbClr>`, `<a:blip>` |
| `r` | `http://schemas.openxmlformats.org/officeDocument/2006/relationships` | The `r:id`/`r:embed`/`r:link` attribute namespace — references rIds in the part's .rels |
| `mc` | `http://schemas.openxmlformats.org/markup-compatibility/2006` | Markup compatibility for Office-version extension markers (`mc:AlternateContent`, `mc:Ignorable`) |
| `Relationships` (default) | `http://schemas.openxmlformats.org/package/2006/relationships` | The `<Relationships>` root of every `.rels` file |
| `Types` (default) | `http://schemas.openxmlformats.org/package/2006/content-types` | The `<Types>` root of `[Content_Types].xml` |
| `pic` | `http://schemas.openxmlformats.org/drawingml/2006/picture` | Picture-element namespace inside `<p:pic>` graphic-frames |
| `dgm` | `http://schemas.openxmlformats.org/drawingml/2006/diagram` | SmartArt diagrams (slideforge v1 not in scope) |
| `c` | `http://schemas.openxmlformats.org/drawingml/2006/chart` | Charts (slideforge v1: native rendering, not embedded `c:chart`) |

**Per-part required declarations:**

- `[Content_Types].xml`: declares `xmlns="http://schemas.openxmlformats.org/package/2006/content-types"` on `<Types>` (no other prefixes).
- `.rels` files: declares `xmlns="http://schemas.openxmlformats.org/package/2006/relationships"` on `<Relationships>` (no other prefixes).
- `presentation.xml`: `<p:presentation xmlns:a="..." xmlns:p="..." xmlns:r="..." [xmlns:mc="..."]>`
- `theme1.xml`: root is `<a:theme xmlns:a="...">`; `xmlns:r` only needed if rels are referenced inside theme (rare).
- `slideMaster1.xml`, `slideLayoutN.xml`, `slideN.xml`: `xmlns:a="..." xmlns:p="..." xmlns:r="..."`
- `notesMaster1.xml`, `notesSlideN.xml`: same as slide.

Office is lenient on **extra** declarations (you can declare `xmlns:mc` even when unused). Office is strict on **missing required prefixes used in element names** — `<p:sld>` without `xmlns:p` is a hard parse failure. Office is strict on **2006 vs other-year URIs** — `presentationml/2010` is a different schema; using the wrong year is the #1 cause of "weird-but-it-validates" silent failures.

Sources: Microsoft Learn "Structure of a PresentationML document"; ooxmlsdk pml_hierarchy.md; OfficeOpenXML.com `prPresentation` (http://officeopenxml.com/prPresentation.php, accessed 2026-05-23).

---

## Content Types Registration

`[Content_Types].xml` at the package root maps every part to a content type. Two element kinds:

- `<Default Extension="..." ContentType="..."/>` — applies to every part whose path ends in that extension.
- `<Override PartName="/full/path/to/part.xml" ContentType="..."/>` — applies to one specific part. PartName MUST begin with `/`.

Override beats Default. **Every XML part needs an Override** because the generic `application/xml` Default isn't specific enough for OPC consumers — they use content type to choose which parser/handler to invoke.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml"  ContentType="application/xml"/>
  <Default Extension="png"  ContentType="image/png"/>
  <Default Extension="jpeg" ContentType="image/jpeg"/>
  <Default Extension="jpg"  ContentType="image/jpeg"/>
  <Default Extension="gif"  ContentType="image/gif"/>

  <Override PartName="/ppt/presentation.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/theme/theme1.xml"
            ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slides/slide1.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/notesMasters/notesMaster1.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"/>
  <Override PartName="/ppt/presProps.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.presProps+xml"/>
  <Override PartName="/ppt/viewProps.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml"/>
  <Override PartName="/ppt/tableStyles.xml"
            ContentType="application/vnd.openxmlformats-officedocument.presentationml.tableStyles+xml"/>
  <Override PartName="/docProps/core.xml"
            ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml"
            ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>
```

**Critical rules:**

- Content type strings are **case-sensitive**. `presentationML` (capital ML) is wrong; the standard mixes case as written above.
- Theme content type is **`...officedocument.theme+xml`** NOT `...presentationml.theme+xml` (common synthesis bug — themes are shared with WordprocessingML/SpreadsheetML).
- Forgetting `Default Extension="rels"` makes every `.rels` file unresolved and the package fails to open.
- Forgetting the `Default` for image extensions makes embedded images fail silently (Office shows broken-image icon; Keynote drops the picture entirely).

Sources: ECMA-376 Part 2 §10 (OPC); Microsoft "About Open XML packaging" via python-pptx docs (https://python-pptx.readthedocs.io/en/stable/dev/resources/about_packaging.html, accessed 2026-05-23).

---

## Relationship Model

A relationship part (`.rels` file) lives at `[same-directory-as-target]/_rels/[target-filename].rels`. The root `_rels/.rels` is the special case at package root.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
                Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
                Target="ppt/presentation.xml"/>
  <Relationship Id="rId2"
                Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"
                Target="docProps/core.xml"/>
  <Relationship Id="rId3"
                Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties"
                Target="docProps/app.xml"/>
</Relationships>
```

**Attributes:**

- `Id` — string, conventionally `rId<n>`, unique within the .rels file only. References from XML via `r:id="rId1"` (or `r:embed`, `r:link`).
- `Type` — the relationship type URI. Microsoft's URI taxonomy at `.../officeDocument/2006/relationships/...` lists every valid type.
- `Target` — path **relative to the .rels file's directory**, NOT relative to package root.
- `TargetMode="External"` — optional, only for `r:link` external images and external hyperlinks.

**The directory-relative `Target` is the most error-prone aspect.** From `ppt/slideMasters/_rels/slideMaster1.xml.rels`:
- `Target="../slideLayouts/slideLayout1.xml"` ✓
- `Target="../theme/theme1.xml"` ✓
- `Target="ppt/slideLayouts/slideLayout1.xml"` ✗ (would refer to `ppt/slideMasters/ppt/slideLayouts/...`)
- `Target="/ppt/slideLayouts/slideLayout1.xml"` ✓ (absolute, also valid — leading `/` means package root)

**Relationship-type URIs slideforge will use:**

| Source part | Target part | Type URI (suffix after `.../relationships/`) |
|---|---|---|
| `_rels/.rels` | `ppt/presentation.xml` | `officeDocument` |
| `_rels/.rels` | `docProps/core.xml` | `metadata/core-properties` (note: package namespace, not officeDocument) |
| `_rels/.rels` | `docProps/app.xml` | `extended-properties` |
| `ppt/_rels/presentation.xml.rels` | `slideMasters/slideMaster1.xml` | `slideMaster` |
| `ppt/_rels/presentation.xml.rels` | `theme/theme1.xml` | `theme` |
| `ppt/_rels/presentation.xml.rels` | `slides/slide1.xml` | `slide` |
| `ppt/_rels/presentation.xml.rels` | `notesMasters/notesMaster1.xml` | `notesMaster` |
| `ppt/_rels/presentation.xml.rels` | `presProps.xml` | `presProps` |
| `ppt/_rels/presentation.xml.rels` | `viewProps.xml` | `viewProps` |
| `ppt/_rels/presentation.xml.rels` | `tableStyles.xml` | `tableStyles` |
| `ppt/slideMasters/_rels/slideMaster1.xml.rels` | `../slideLayouts/slideLayoutN.xml` | `slideLayout` |
| `ppt/slideMasters/_rels/slideMaster1.xml.rels` | `../theme/theme1.xml` | `theme` |
| `ppt/slideLayouts/_rels/slideLayoutN.xml.rels` | `../slideMasters/slideMaster1.xml` | `slideMaster` |
| `ppt/slides/_rels/slideN.xml.rels` | `../slideLayouts/slideLayoutN.xml` | `slideLayout` |
| `ppt/slides/_rels/slideN.xml.rels` | `../media/imageN.png` | `image` |
| `ppt/slides/_rels/slideN.xml.rels` | `../notesSlides/notesSlideN.xml` | `notesSlide` |

**Bidirectional rule:** master → layouts (forward) AND layout → master (back). Many homemade generators forget the back-pointer; PowerPoint silently regenerates it on save, but Keynote and Google Slides drop the layout's theme inheritance without it.

Sources: ECMA-376 Part 2 §9; OfficeOpenXML.com `anatomyofOOXML-pptx` (http://officeopenxml.com/anatomyofOOXML-pptx.php); ooxmlsdk pml_hierarchy.md.

---

## Coordinate / Sizing System

**The EMU (English Metric Unit) is the lingua franca of all positioning in DrawingML.** 1 EMU = 1/914,400 inch = 1/360,000 cm.

| From | To EMU | From EMU |
|---|---|---|
| 1 inch | 914,400 | 914,400 EMU = 1 in |
| 1 cm | 360,000 | 360,000 EMU = 1 cm |
| 1 mm | 36,000 | 36,000 EMU = 1 mm |
| 1 point (DTP) | 12,700 | 12,700 EMU = 1 pt |
| 1 pixel @96 DPI | 9,525 | 9,525 EMU = 1 px |
| 1 EMU | 1 | 1 EMU ≈ 0.0254 µm |

**Font sizes** are NOT in EMU; they are in **hundredths of a point** (`sz="2400"` = 24 pt) within `<a:rPr>` and `<a:defRPr>`. Line widths in `<a:ln w="...">` ARE in EMU (`w="9525"` = 0.75 pt).

**Default slide sizes (declared in `<p:sldSz cx="W" cy="H" type="..."/>` inside presentation.xml):**

| Aspect | Width (EMU) | Height (EMU) | Inches | sldSz `type` value |
|---|---|---|---|---|
| 16:9 widescreen (Office 2013+) | 12,192,000 | 6,858,000 | 13.333 × 7.5 | `screen16x9` |
| 16:9 (older Office 2010) | 9,144,000 | 5,143,500 | 10 × 5.625 | `screen16x9` |
| 16:10 | 9,144,000 | 5,715,000 | 10 × 6.25 | `screen16x10` |
| 4:3 standard | 9,144,000 | 6,858,000 | 10 × 7.5 | `screen4x3` |
| A4 portrait | 7,560,000 | 10,692,000 | 8.27 × 11.69 | `A4` |
| US Letter | 9,144,000 | 6,858,000 | 10 × 7.5 (same as 4:3) | `letter` |
| Custom | (any) | (any) | (any) | `custom` |

**Notes size** is declared via `<p:notesSz cx="..." cy="..."/>` in presentation.xml. Standard: 6,858,000 × 9,144,000 (portrait 4:3) — note that **width and height are swapped** from a 4:3 slide.

**Position vs Extent:**
- `<a:off x="..." y="..."/>` — absolute offset of top-left of shape from slide top-left, in EMU.
- `<a:ext cx="..." cy="..."/>` — width and height in EMU (`cx` = "coordinate-x extent" = width, `cy` = height).
- Together they form `<a:xfrm><a:off/><a:ext/></a:xfrm>` — the standard transform group.

Slideforge defaults from brand.toml will be `slide_size = "16x9"` mapping to `cx=12192000 cy=6858000 type="screen16x9"`. **Validation rule slideforge MUST enforce: `cx` and `cy` exactly match the declared type, or PowerPoint silently resets to its default 16:9.**

Sources: ECMA-376 Part 1 §20.1.10.16 (ST_PositiveCoordinate); OfficeOpenXML.com `drwSp-xfrm` (http://officeopenxml.com/drwSp-size.php, accessed 2026-05-23); Microsoft Support "Change slide size" (https://support.microsoft.com/en-us/office/change-the-size-of-your-powerpoint-slides-040a811c-be43-40b9-8d04-0de5ed79987e, accessed 2026-05-23).

---

## Slide ID Allocation

Three independent ID spaces inside `presentation.xml`, all required as `<p:sldIdLst>`, `<p:sldMasterIdLst>`, plus the layout list which lives inside each slideMaster, not presentation.xml.

```xml
<p:presentation>
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:notesMasterIdLst>
    <p:notesMasterId r:id="rId4"/>
  </p:notesMasterIdLst>
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId2"/>
    <p:sldId id="257" r:id="rId3"/>
    <!-- ... -->
  </p:sldIdLst>
  <p:sldSz cx="12192000" cy="6858000" type="screen16x9"/>
  <p:notesSz cx="6858000" cy="9144000"/>
  <p:defaultTextStyle> <!-- required, see below --> </p:defaultTextStyle>
</p:presentation>
```

**Rules:**

- `sldId/@id`: integer in `[256, 2147483647]` (signed 32-bit positive minus the reserved-low range). Conventionally **starts at 256 for slide 1, increments by 1**. Must be globally unique within the deck. **Never reused** even after slide deletion (creates "gaps" in the sequence — PowerPoint deliberately preserves these so external bookmarks survive).
- `sldId/@r:id`: rId string pointing to the slide part in `ppt/_rels/presentation.xml.rels`.
- `sldMasterId/@id`: integer ≥ 2,147,483,648 (`0x80000000`). Microsoft's convention is `2147483648, 2147483649, ...` if multiple masters.
- `sldLayoutId/@id` (inside slideMaster1.xml, NOT presentation.xml): integer ≥ 2,147,483,648, conventionally sequential after the master IDs (`2147483649..2147483659` for 11 standard layouts).
- `notesMasterId/@r:id`: only an rId, no integer ID needed.

**Required-element ordering inside `<p:presentation>`** (ECMA-376 §19.2.1.26 `CT_Presentation`):
1. `sldMasterIdLst` (required, ≥1 child)
2. `notesMasterIdLst` (optional, 0 or 1 child)
3. `handoutMasterIdLst` (optional, 0 or 1 child)
4. `sldIdLst` (required if any slides; **the element must exist even with no children**)
5. `sldSz` (required)
6. `notesSz` (required)
7. `embeddedFontLst` (optional)
8. `custShowLst` (optional)
9. `photoAlbum` (optional)
10. `custDataLst` (optional)
11. `kinsoku` (optional)
12. `defaultTextStyle` (required-in-practice — missing causes "PowerPoint can repair" prompt)
13. `modifyVerifier` (optional)
14. `extLst` (optional)

Out-of-order children fail schema validation in strict consumers (Office in strict-mode).

Sources: ECMA-376 Part 1 §19.2 (PresentationML root); ooxmlsdk pml_hierarchy.md; python-pptx slide-id analysis (https://python-pptx.readthedocs.io/en/stable/dev/analysis/sld-slide.html, accessed 2026-05-23).

---

## theme1.xml — Required Structure

A theme part has root `<a:theme>` with required children **in this order**: `themeElements`, then optional `objectDefaults`, then optional `extraClrSchemeLst`. Inside `themeElements` the order is **strict**: `clrScheme` → `fontScheme` → `fmtScheme`.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Slideforge Brand">
  <a:themeElements>
    <a:clrScheme name="SlideforgeBrand">
      <!-- ORDER IS FIXED: dk1, lt1, dk2, lt2, accent1..6, hlink, folHlink -->
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window"     lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F3864"/></a:dk2>
      <a:lt2><a:srgbClr val="E7E6E6"/></a:lt2>
      <a:accent1><a:srgbClr val="4472C4"/></a:accent1>
      <a:accent2><a:srgbClr val="ED7D31"/></a:accent2>
      <a:accent3><a:srgbClr val="A5A5A5"/></a:accent3>
      <a:accent4><a:srgbClr val="FFC000"/></a:accent4>
      <a:accent5><a:srgbClr val="5B9BD5"/></a:accent5>
      <a:accent6><a:srgbClr val="70AD47"/></a:accent6>
      <a:hlink><a:srgbClr val="0563C1"/></a:hlink>
      <a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="SlideforgeBrand">
      <a:majorFont>
        <a:latin typeface="Inter"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="Inter"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
      </a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="SlideforgeBrand">
      <!-- REQUIRED: exactly 3 fills, 3 lines, 3 effects, 3 bg fills -->
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:lumMod val="110000"/><a:tint val="73000"/></a:schemeClr></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:lumMod val="105000"/><a:shade val="80000"/></a:schemeClr></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350"  cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln>
        <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln>
        <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/><a:miter lim="800000"/></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:tint val="95000"/></a:schemeClr></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:shade val="80000"/></a:schemeClr></a:solidFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>
```

**brand.toml ↔ theme1.xml mapping:**

| brand.toml field | theme1.xml location |
|---|---|
| `colors.background_light` | `<a:lt1>` (use `sysClr val="window"` for OS-adaptive, OR `srgbClr val="FFFFFF"` for hard-coded) |
| `colors.background_dark` | `<a:dk1>` |
| `colors.text_on_light` | `<a:dk2>` (typical) or override via clrMap |
| `colors.accent[1..6]` | `<a:accent1..accent6>` |
| `colors.hyperlink` | `<a:hlink>` |
| `colors.hyperlink_followed` | `<a:folHlink>` |
| `fonts.heading` | `<a:majorFont><a:latin typeface="...">` |
| `fonts.body` | `<a:minorFont><a:latin typeface="...">` |

**Gotchas:**

- All 12 colors MUST be present. PowerPoint silently fills `dk1=000000, lt1=FFFFFF` if missing but Keynote shows a black-on-black slide.
- Color hex MUST be 6 chars, no `#`, uppercase. `val="ff0000"` works but `val="#FF0000"` rejects.
- `phClr` ("placeholder color") in fmtScheme means "fill in with whatever color the consumer is currently styling for" — it is intentional and required, not a bug.
- `<a:fillStyleLst>`, `<a:lnStyleLst>`, `<a:effectStyleLst>`, `<a:bgFillStyleLst>` MUST each have **exactly 3 children**. Two or four causes Office to repair.
- Empty `<a:effectLst/>` is valid; an absent `<a:effectStyle>` is not.

Sources: ECMA-376 Part 1 §20.1.6.9 (CT_BaseStyles, theme), §20.1.6.2 (CT_ColorScheme); ooxmlsdk pml_themes.md (https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_themes.md); c-rex.net theme references (https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_themeElements_topic_ID0EYXIMB.html, accessed 2026-05-23).

---

## slideMaster1.xml — Required Structure

Schema: `CT_SlideMaster` (ECMA-376 §19.3.1.42). Required child order:

1. `<p:cSld>` (common slide data — background + shape tree of master-level placeholders)
2. `<p:clrMap .../>` (color map — theme colors → slide semantic roles)
3. `<p:sldLayoutIdLst>` (list of this master's layouts)
4. `<p:transition>` (optional)
5. `<p:timing>` (optional)
6. `<p:hf .../>` (header/footer settings, optional)
7. `<p:txStyles>` (default text styles for titles, bodies, others)

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:bg>
      <p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm>
      </p:grpSpPr>

      <!-- TITLE PLACEHOLDER (type="title") -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title Placeholder 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="title"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="457200" y="274638"/><a:ext cx="11277600" cy="1143000"/></a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr vert="horz" lIns="91440" tIns="45720" rIns="91440" bIns="45720" rtlCol="0" anchor="ctr"/>
          <a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US"/><a:t>Click to edit Master title style</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>

      <!-- BODY PLACEHOLDER (type="body"), additional master shapes... -->
    </p:spTree>
  </p:cSld>

  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2"
            accent1="accent1" accent2="accent2" accent3="accent3"
            accent4="accent4" accent5="accent5" accent6="accent6"
            hlink="hlink" folHlink="folHlink"/>

  <p:sldLayoutIdLst>
    <p:sldLayoutId id="2147483649" r:id="rId1"/>
    <p:sldLayoutId id="2147483650" r:id="rId2"/>
    <!-- ... -->
  </p:sldLayoutIdLst>

  <p:txStyles>
    <p:titleStyle>
      <a:lvl1pPr algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
        <a:defRPr sz="4400" kern="1200">
          <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
          <a:latin typeface="+mj-lt"/>
          <a:ea typeface="+mj-ea"/>
          <a:cs typeface="+mj-cs"/>
        </a:defRPr>
      </a:lvl1pPr>
    </p:titleStyle>
    <p:bodyStyle>
      <a:lvl1pPr marL="342900" indent="-342900" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
        <a:buFont typeface="Arial" pitchFamily="34" charset="0"/>
        <a:buChar char="&#8226;"/>
        <a:defRPr sz="2800" kern="1200">
          <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
          <a:latin typeface="+mn-lt"/>
        </a:defRPr>
      </a:lvl1pPr>
      <!-- lvl2pPr..lvl9pPr similar with smaller sizes and more indent -->
    </p:bodyStyle>
    <p:otherStyle>
      <a:defPPr><a:defRPr lang="en-US"/></a:defPPr>
      <a:lvl1pPr><a:defRPr sz="1800"/></a:lvl1pPr>
    </p:otherStyle>
  </p:txStyles>
</p:sldMaster>
```

**clrMap demystified:** Slide content uses **semantic** color refs like `<a:schemeClr val="bg1"/>` ("background 1"). The clrMap on the master says "in *this* master's context, semantic `bg1` means theme-color `lt1`." Standard mapping: `bg1=lt1, tx1=dk1, bg2=lt2, tx2=dk2, accent1..6=accent1..6, hlink=hlink, folHlink=folHlink`. **A layout can override** via `<p:clrMapOvr><a:overrideClrMapping bg1="dk2" tx1="lt1" .../></p:clrMapOvr>` to invert backgrounds (dark divider slide with light text — same theme, different perceived palette).

**Font typeface shorthand inside txStyles:**

- `+mj-lt` = major Latin font (heading) from theme
- `+mj-ea` = major East Asian font from theme
- `+mj-cs` = major complex-script font from theme
- `+mn-lt` = minor Latin font (body) from theme
- `+mn-ea`, `+mn-cs` similarly

slideforge uses these in master/layout txStyles so every slide picks up the brand font automatically; only explicit overrides need literal `typeface="Arial"`.

Sources: ECMA-376 §19.3.1.42, §19.3.1.6 (clrMap); OfficeOpenXML.com `prSlide-master` (http://officeopenxml.com/prSlideMaster.php, accessed 2026-05-23); ooxmlsdk pml_hierarchy.md.

---

## slideLayoutN.xml — Required Structure

Schema: `CT_SlideLayout` (ECMA-376 §19.3.1.39). Root attribute `type=` from the 32-value `ST_SlideLayoutType` enum (`title`, `tx`, `obj`, `twoColTx`, `twoObj`, `titleOnly`, `blank`, `picTx`, `secHead`, `cust`, ...).

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             type="obj" preserve="1">
  <p:cSld name="Title and Content">
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/></a:xfrm></p:grpSpPr>

      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="title"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>  <!-- empty = inherit position from master's title placeholder -->
        <p:txBody>
          <a:bodyPr/><a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US"/><a:t>Click to edit title</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>

      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Content Placeholder 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="457200" y="1600200"/><a:ext cx="11277600" cy="4525963"/></a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/><a:lstStyle/>
          <a:p><a:pPr lvl="0"/><a:r><a:rPr lang="en-US"/><a:t>Click to edit content</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>
```

**Placeholder system (`<p:ph>`):**

| Attribute | Purpose | Allowed values |
|---|---|---|
| `type` | Semantic role | `title, ctrTitle, subTitle, body, dt, sldNum, ftr, hdr, obj, chart, tbl, clipArt, dgm, media, sldImg, pic` |
| `idx` | Numeric index for layout→slide matching | unsigned int; `0` is implicit when omitted (= title); usually `1, 2, ...` for body placeholders |
| `orient` | Text orientation | `horz` (default), `vert` |
| `sz` | Size hint | `full` (default), `half`, `quarter` |
| `hasCustomPrompt` | Whether to show layout's prompt text | `0` (default) or `1` |

**Inheritance:**
- **Layout → Master**: layout placeholder inherits from master's placeholder where `type` matches (idx ignored).
- **Slide → Layout**: slide placeholder inherits from layout's placeholder where `idx` matches (type ignored — but conventionally type matches too).
- A title placeholder on a slide should have `<p:ph type="title"/>` (no idx) and inherit position from layout's title placeholder.
- A body content placeholder on a slide uses `<p:ph idx="1"/>` matching layout's `<p:ph idx="1"/>`.
- **Empty `<p:spPr/>`** on a placeholder means "inherit position from layout/master." Specifying `<p:spPr><a:xfrm>...</a:xfrm></p:spPr>` means "override."

**`<p:clrMapOvr>`** with `<a:masterClrMapping/>` = "use master's clrMap as-is." With `<a:overrideClrMapping bg1="dk2".../>` = override the mapping for this layout. Every slideLayout must have a `<p:clrMapOvr>` element; absence triggers Office repair.

**`preserve="1"`** on `<p:sldLayout>` means "don't delete this layout when no slides use it." Defaults to `0`; slideforge should set `1` on all generated layouts so users can apply them later in PowerPoint.

Sources: ECMA-376 §19.3.1.39, §19.3.1.36 (ph element), §19.3.1.13 (clrMapOvr); python-pptx placeholders analysis (https://python-pptx.readthedocs.io/en/latest/dev/analysis/placeholders/, accessed 2026-05-23); ooxmlsdk pml_hierarchy.md.

---

## slideN.xml — Required Structure

Schema: `CT_Slide` (§19.3.1.38). Root `<p:sld>` with child order:
1. `<p:cSld>` (required) — slide background and shape tree
2. `<p:clrMapOvr>` (optional but conventional, almost always `<a:masterClrMapping/>`)
3. `<p:transition>` (optional)
4. `<p:timing>` (optional, for animations)
5. `<p:extLst>` (optional, for extensions)

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/></a:xfrm></p:grpSpPr>

      <!-- Title placeholder — inherits position from layout -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="title"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/><a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US" dirty="0"/><a:t>Quarterly Results</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>

      <!-- Body content with bullets -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Content Placeholder 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/><a:lstStyle/>
          <a:p><a:pPr lvl="0"/><a:r><a:rPr lang="en-US"/><a:t>Revenue up 12%</a:t></a:r></a:p>
          <a:p><a:pPr lvl="1"/><a:r><a:rPr lang="en-US"/><a:t>APAC led growth</a:t></a:r></a:p>
          <a:p><a:pPr lvl="0"/><a:r><a:rPr lang="en-US"/><a:t>Costs down 4%</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>
```

The slide is the leaf of the inheritance chain: theme → master → layout → slide. Slideforge synthesis fills only **deltas** at the slide level; everything else (font, color, padding, default position) inherits.

Sources: ECMA-376 §19.3.1.38; ooxmlsdk pml_hierarchy.md.

---

## Image / Picture Handling

Pictures flow through **four** coordinated artifacts: (1) the binary in `ppt/media/imageN.png`, (2) a Default entry in `[Content_Types].xml`, (3) a relationship in the slide's `.rels` file with `Type="...relationships/image"`, (4) a `<p:pic>` shape in the slide referencing the relationship via `r:embed`.

```xml
<!-- Inside ppt/slides/slide1.xml -->
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="4" name="Logo" descr="Company logo"/>
    <p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr>
    <p:nvPr/>
  </p:nvPicPr>
  <p:blipFill>
    <a:blip r:embed="rId2" cstate="print"/>
    <a:stretch><a:fillRect/></a:stretch>
  </p:blipFill>
  <p:spPr>
    <a:xfrm><a:off x="9144000" y="457200"/><a:ext cx="2286000" cy="571500"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
  </p:spPr>
</p:pic>

<!-- Inside ppt/slides/_rels/slide1.xml.rels -->
<Relationship Id="rId2"
              Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
              Target="../media/image1.png"/>
```

**Image formats Office accepts:** PNG, JPEG (jpg/jpeg), GIF, TIFF, BMP, EMF, WMF, SVG (PowerPoint 2016+ only; for cross-renderer safety rasterize SVG to PNG). Animated GIF: PowerPoint renders animation in slideshow; Keynote, Google Slides flatten to first frame.

**Content type registration:** Default `<Default Extension="png" ContentType="image/png"/>` is sufficient; do not use Overrides on media files.

**Cropping vs stretching:**
- `<a:stretch><a:fillRect/></a:stretch>` = fill the shape, no cropping (default).
- `<a:srcRect l="..." t="..." r="..." b="..."/>` = source-rectangle crop, values are 1/100,000 of the image dimension (e.g. `l="10000"` crops 10% off left).
- `<a:tile .../>` = repeat image.

**External images** (`r:link` instead of `r:embed` with `TargetMode="External"`): legal in spec but **strongly discouraged** — Keynote, Google Slides, LibreOffice frequently fail to fetch.

Sources: ECMA-376 §20.1.8.13 (CT_BlipFillProperties), §20.1.8.55 (CT_Picture); ooxmlsdk drawingml.md (https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/drawingml.md, accessed 2026-05-23).

---

## Text Formatting Model

DrawingML text body model: `<a:txBody>` → `<a:bodyPr>` (body-level properties) → `<a:lstStyle>` (list-style overrides) → one or more `<a:p>` (paragraphs).

A paragraph contains `<a:pPr>` (paragraph properties, optional) then zero or more `<a:r>` (runs) and/or `<a:br>` (line breaks) and/or `<a:fld>` (fields like slide numbers), terminated by an optional `<a:endParaRPr>` (the "phantom" run properties for an empty trailing position).

A run contains `<a:rPr>` (run properties, optional) then exactly one `<a:t>` (literal text).

```xml
<a:p>
  <a:pPr marL="342900" indent="-342900" lvl="0" algn="l" defTabSz="914400" rtl="0">
    <a:lnSpc><a:spcPct val="100000"/></a:lnSpc>      <!-- line spacing 100% -->
    <a:spcBef><a:spcPts val="0"/></a:spcBef>         <!-- 0pt before -->
    <a:spcAft><a:spcPts val="600"/></a:spcAft>       <!-- 6pt after -->
    <a:buFont typeface="Arial" pitchFamily="34" charset="0"/>
    <a:buChar char="&#8226;"/>                       <!-- bullet character (or buAutoNum/buNone) -->
  </a:pPr>
  <a:r>
    <a:rPr lang="en-US" sz="2400" b="0" i="0" u="none" dirty="0">
      <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
      <a:latin typeface="+mn-lt"/>
    </a:rPr>
    <a:t>Body text here.</a:t>
  </a:r>
  <a:endParaRPr lang="en-US" sz="2400"/>
</a:p>
```

**Key `<a:rPr>` attributes:**

| Attribute | Type | Meaning |
|---|---|---|
| `lang` | xs:string (BCP-47) | Language tag — `en-US`. **Required-in-practice** for spell check |
| `sz` | unsigned (hundredths of pt) | Font size; `sz="2400"` = 24 pt |
| `b` | boolean (`0`/`1`) | Bold |
| `i` | boolean | Italic |
| `u` | enum | Underline (`none`, `sng`, `dbl`, `dotted`, ...) |
| `strike` | enum | Strikethrough (`noStrike`, `sngStrike`, `dblStrike`) |
| `baseline` | int (%) | Subscript/superscript offset (e.g. `-25000` = subscript at 25% below) |
| `dirty` | boolean | Spell-check flag (set `0` on generated content) |
| `kern` | unsigned | Kerning threshold in hundredths of pt |

Child elements (in this order): fills (`solidFill`, `gradFill`, `noFill`, `blipFill`), `<a:ln>` (text outline), `<a:effectLst>`, `<a:highlight>`, `<a:uFill>`, `<a:latin>`, `<a:ea>`, `<a:cs>`, `<a:sym>`, `<a:hlinkClick>`, `<a:hlinkMouseOver>`.

**Key `<a:pPr>` attributes:**

| Attribute | Meaning |
|---|---|
| `lvl` | Outline level 0-8; matches master `lvl1pPr..lvl9pPr` (lvl="0" → lvl1pPr) |
| `marL` | Left margin in EMU |
| `marR` | Right margin in EMU |
| `indent` | First-line indent in EMU (negative for hanging indent on bulleted lists) |
| `algn` | Alignment (`l`, `ctr`, `r`, `just`, `dist`) |
| `defTabSz` | Default tab size in EMU |
| `rtl` | Right-to-left (`0`/`1`) |
| `eaLnBrk`, `latinLnBrk`, `hangingPunct` | Asian/Latin line break rules |

**Bullet definitions:**

- `<a:buChar char="•"/>` — literal bullet character (must escape `&#8226;` in XML)
- `<a:buAutoNum type="arabicPeriod" startAt="1"/>` — auto-numbering; type enum includes `arabicPeriod, arabicParenR, romanLcPeriod, alphaUcPeriod, ...` (32 values total)
- `<a:buNone/>` — no bullet (use for title paragraphs)
- `<a:buFont typeface="..."/>` — font for the bullet character (separate from text font)
- `<a:buSzPct val="100000"/>` — bullet size as percent (1/1000) of text size
- `<a:buClr><a:schemeClr val="accent1"/></a:buClr>` — bullet color (independent of text)

slideforge bullet patterns: top-level (`lvl="0"`) gets `•`, second level (`lvl="1"`) gets `–` (en-dash), third (`lvl="2"`) gets `▪`. These typically come from txStyles on the master, not per-paragraph.

Sources: ECMA-376 §21.1.2 (DrawingML text); c-rex.net rPr (https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_rPr_topic_ID0EIB4KB.html); pPr (https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_pPr_topic_ID0ERIUKB.html); both accessed 2026-05-23.

---

## Notes Master / Handout Master

**Per ECMA-376, both are optional.** Per practical PowerPoint compatibility, **notesMaster is required if any slide has speaker notes** (which slideforge will almost always have for AI-generated decks). HandoutMaster is genuinely optional — PowerPoint creates it on demand from a default template if missing.

**Minimum notesMaster1.xml:**

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notesMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
               xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
               xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      <!-- sldImg placeholder for the slide thumbnail -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Slide Image Placeholder 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1" noRot="1" noChangeAspect="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="sldImg"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr><a:xfrm><a:off x="685800" y="457200"/><a:ext cx="5486400" cy="3086100"/></a:xfrm></p:spPr>
      </p:sp>
      <!-- body placeholder for the actual notes text -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Notes Placeholder 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr><a:xfrm><a:off x="685800" y="3886200"/><a:ext cx="5486400" cy="4343400"/></a:xfrm></p:spPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2"
            accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6"
            hlink="hlink" folHlink="folHlink"/>
  <p:notesStyle>
    <a:lvl1pPr><a:defRPr sz="1200"/></a:lvl1pPr>
  </p:notesStyle>
</p:notesMaster>
```

Each notes slide (`ppt/notesSlides/notesSlide1.xml`) references this master via its own `.rels` file with `Type="...relationships/notesMaster"`. Notes slides also need a back-rel `Type="...relationships/slide"` pointing to the slide they annotate.

Sources: ECMA-376 §19.3.1.27 (notesMaster); ooxmlsdk pml_notes.md (https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_notes.md, accessed 2026-05-23).

---

## Top-10 Gotchas That Produce "File is Corrupt"

These are ordered by frequency in production PPTX-synthesis bugs (Perplexity research, 2026-05-23, [1][4][7][10][13]).

1. **Broken relationship chain — rId points to a missing or wrong-named part.** Most common synthesis bug. Office shows "PowerPoint found a problem with content"; repair strips the affected part. Mitigation: maintain a single source-of-truth ID generator that emits both the `<Relationship Id=`>` and the consuming `r:id=` together.

2. **Missing `<p:sldIdLst>`, `<p:sldSz>`, `<p:notesSz>`, or `<p:defaultTextStyle>` in presentation.xml.** All four are de-facto-required. PowerPoint "repairs" by injecting defaults but Keynote refuses to open. The required-element order (`sldMasterIdLst → notesMasterIdLst → handoutMasterIdLst → sldIdLst → sldSz → notesSz → … → defaultTextStyle`) is also schema-significant.

3. **Wrong XML namespace URI — using `presentationml/2010` or `drawingml/2010` instead of `/2006/main`.** Files "parse" but render blank or trigger silent repair. There is no `/2010/main` for PresentationML core schema — that namespace is reserved for extension markup (`mc:AlternateContent` wrappers).

4. **Content-type case mismatch or wrong content type for theme.** Theme MUST be `application/vnd.openxmlformats-officedocument.theme+xml` (not `presentationml.theme+xml`). Case-sensitive throughout.

5. **`<a:clrScheme>` missing one of the 12 required colors or wrong order.** Schema requires `dk1, lt1, dk2, lt2, accent1..accent6, hlink, folHlink` in that order. Eleven colors causes Office to substitute black for the missing slot; nine causes outright rejection.

6. **`<p:clrMapOvr>` absent on a slideLayout or slide.** Schema permits absence but every Microsoft template includes it. Keynote and Google Slides assume it exists and crash-render without.

7. **Slide layout has no back-pointer to its slideMaster in `_rels/`.** PowerPoint silently regenerates on save but other renderers drop the layout's theme inheritance, producing un-themed (black-on-white) slides.

8. **Image referenced but `Default Extension="png"` missing from `[Content_Types].xml`.** Image binary is in the ZIP, the rel is correct, but Office can't dispatch the file. Image renders as broken icon; Keynote silently drops.

9. **UTF-8 BOM or non-UTF-8 encoding in XML parts.** Office is mostly lenient on UTF-8 BOM but strict on encoding declarations. `<?xml version="1.0" encoding="UTF-8"?>` with the file actually being UTF-16 LE causes silent parse failure. **Always emit UTF-8 without BOM** for maximum compatibility.

10. **`sldId` value < 256 or duplicate IDs.** Schema constraint per §19.5.84. ID `1` for the first slide causes Office to renumber on open with a repair prompt. Reusing an ID across two slides causes the second slide to silently drop.

Additional honorable mentions:
- **Non-deflated ZIP entries.** OPC requires DEFLATE or STORE; ZIP-bzip2 or LZMA fails to open.
- **Element-order violations inside theme** — `fmtScheme` before `clrScheme` is a schema violation even though most parsers will tolerate it.
- **Fractional/negative EMU values** — `<a:off x="100.5" y="-200"/>` rejected; EMUs are unsigned integers (with very rare exceptions for offsets in group transforms).

Sources: Perplexity research 2026-05-23 (Microsoft Answers thread, sharayeh.com/en/blog/repair-corrupted-powerpoint-guide, drecov.pandaoffice.com guide); python-pptx GitHub issues #460 and around (https://github.com/scanny/python-pptx/issues/460); brand-template-patterns.md.

---

## Office vs. Keynote vs. Google Slides vs. LibreOffice — Compatibility Notes

| Renderer | Strictness | Auto-repair? | Notable behavior for slideforge |
|---|---|---|---|
| **Microsoft PowerPoint (Office 365, 2021, 2019)** | Strictest on OPC + content types; most lenient on element ordering, missing optional parts, namespace prefix variations. | Yes — silently regenerates missing presProps, viewProps, tableStyles, layout back-rels. Triggers user-facing repair prompt on broken rId chains, malformed XML, missing required attrs. | The reference renderer. If it opens cleanly without a "PowerPoint can repair" dialog, the file is production-quality. |
| **Apple Keynote (macOS/iOS)** | Lenient on missing optional parts and unsupported features; strict on packaging and on layout/master relationships. | No auto-repair; silent feature drop. | Drops SmartArt; flattens animations; substitutes fonts aggressively (Inter → Helvetica). Requires notesMaster1.xml if any slide has notes. **Does not render slides whose layout has missing master back-rel.** |
| **Google Slides** | Most lenient — opens many "broken" PPTX that PowerPoint rejects. | Implicit on re-export (normalizes structure). | Drops unsupported transitions; ignores `<p:clrMapOvr>` complex mappings; falls back to web-safe fonts. Best at salvaging partially-malformed files. |
| **LibreOffice Impress** | Lenient on XML; strict on relationships. | Limited — opens damaged files but simplifies content. | Loses some animation timings; SmartArt → static images; complex effects flattened. Useful as a parity-check oracle. |

**Slideforge cross-renderer parity strategy** (informs ADR-002):
- Generate **all 11 standard layouts** even if some unused — Keynote and Google Slides special-case these by name.
- Include **notesMaster1.xml + handoutMaster1.xml** even if empty — Keynote complains about missing notesMaster.
- Use **standard fonts** (Inter, Calibri, Aptos, Helvetica, Arial) in brand themes; embed fallback typeface refs (`+mj-lt`/`+mn-lt`).
- Avoid **external image links** (`r:link` + `TargetMode="External"`) entirely.
- Avoid SVG embedding (`image/svg+xml`) — rasterize to PNG for v1.
- Run a **golden-deck snapshot test matrix**: PowerPoint (Windows + macOS), Keynote, Google Slides headless, LibreOffice Impress headless.

Sources: Perplexity research 2026-05-23 ([3][9][12][15] above); ADR-002 candidate in decisions-applied.md.

---

## Mapping slideforge concepts → OOXML elements

| slideforge concept | OOXML mapping | Notes |
|---|---|---|
| brand.toml `[colors]` | `theme1.xml` → `<a:clrScheme>` 12 colors | Order fixed: dk1, lt1, dk2, lt2, accent1..6, hlink, folHlink |
| brand.toml `[fonts]` | `theme1.xml` → `<a:fontScheme>` major (heading) + minor (body) | Use `<a:latin typeface="..."/>`; let `+mj-lt`/`+mn-lt` propagate |
| brand.toml `[slide_size]` | `presentation.xml` → `<p:sldSz cx=".." cy=".." type=".."/>` | `cx`/`cy` MUST match the type |
| brand.toml `[logo]` | `ppt/media/imageN.png` + Content_Type Default + `<p:pic>` on master/layout | Embed on master to appear on every slide; embed on layout for per-layout |
| brand.toml `[bullets]` (optional) | master `<p:txStyles><p:bodyStyle><a:lvl1pPr><a:buChar/>` | One `lvl[1-9]pPr` per bullet level |
| Slide title (slideforge `# Heading`) | layout `<p:ph type="title"/>` + slide `<p:sp><p:ph type="title"/>...<a:t>Title</a:t>` | Inherit position via empty `<p:spPr/>` |
| Slide bullets (slideforge `- item`) | slide `<p:sp><p:ph idx="1"/>` with multiple `<a:p>` each at appropriate `lvl=` | Match layout idx="1" |
| Speaker notes | `ppt/notesSlides/notesSlideN.xml` with rels to notesMaster1.xml + slideN.xml | Slide must reference notesSlide via slide.xml.rels |
| Section divider slide (dark background) | Custom layout with `<p:clrMapOvr><a:overrideClrMapping bg1="dk2" tx1="lt1" .../>` | Same theme, inverted palette |
| Image slide | `<p:pic>` shape in slide.spTree + ../media/imageN file + image rel | Use `r:embed="rIdN"` (not r:link) |
| Table | `<p:graphicFrame>` wrapping `<a:graphic><a:graphicData uri=".../table">` + `<a:tbl>` | Out of scope for ADR-001 deep treatment |
| Chart | Native render → embed as `<p:pic>` of generated PNG/SVG, OR `<p:graphicFrame>` with embedded chart part | ADR-001 decides; charts-as-pictures simpler for v1 |
| 23 slideforge slide types | 11 standard `<p:sldLayout type="...">` + 12 custom `<p:sldLayout type="cust">` | See brand-template-patterns.md for the full mapping table |
| `slideforge extract-brand` | Parse existing PPTX with ooxmlsdk, extract `<a:clrScheme>`, `<a:fontScheme>`, master shape positions → emit `brand.toml` | Inverse direction of synthesis |

---

## Open Questions for Architect (ADR-001)

1. **One slideMaster or multiple?** Spec allows multiple. Slideforge's 23 slide types could split across (a) one master with 31 layouts, (b) two masters (light/dark) with 16 layouts each, or (c) one master per slide type. Production templates almost always use (a); see brand-template-patterns.md.
2. **`slideLayoutN.xml` numbering convention.** Sequential by creation order, or by type-stable ordering (slideLayout1 = "title", slideLayout2 = "title and content", etc., matching Microsoft default)? Keynote and Google Slides reportedly key off the **type attribute**, not the filename, but standardization helps debugging.
3. **`<p:clrMapOvr>` strategy.** Always use `<a:masterClrMapping/>` (inherit), or selectively override for dark/accent backgrounds? Affects how brand.toml expresses "this layout is dark."
4. **txStyles depth.** Generate all 9 outline levels per style, or only lvl1pPr and let consumers extrapolate? Microsoft templates always generate all 9.
5. **Notes slide generation.** Always emit a notesSlide per slide (even empty), or only when slideforge IR has a `notes:` field? Latter is smaller but Keynote sometimes injects a default if missing.
6. **Embedded fonts.** OOXML supports `<p:embeddedFontLst>` with `<p:embeddedFont>` parts. Slideforge can embed Inter so users without it installed render correctly. Significantly increases file size (~500 KB per face).
7. **`<p:transition>` and `<p:timing>`.** Slideforge IR has no animation model in v1. Emit nothing? Emit `<p:transition><p:fade/></p:transition>` as a sensible default? Animation timing in particular is a 50-element rabbit hole that we should ignore.
8. **PPTX read-back parity.** When `extract-brand` reads a PPTX and emits brand.toml, then synthesis re-emits from that brand.toml, is the round-trip byte-identical? Almost certainly not (timestamps, rIds reassigned), but **layout-identical** is the goal — ADR-002's snapshot test matrix should enforce this.
9. **DocProps emission.** `<dc:creator>slideforge</dc:creator>` always, or attribute to the slideforge user? `<dcterms:modified>` — current time, or the IR's `built_at`?
10. **Strict vs Transitional schema.** ECMA-376 defines two compliance levels: Transitional (Office 2007/2010 backward-compatible) and Strict (ISO/IEC 29500-1 only). PowerPoint defaults to Transitional. Slideforge should emit **Transitional** (the namespace URIs we listed) for max compatibility; Strict uses different `/2012/` URIs.

---

## Sources

All accessed 2026-05-23.

**ECMA-376 standard:**
- https://www.ecma-international.org/publications-and-standards/standards/ecma-376/ — root standard page
- https://ecma-international.org/wp-content/uploads/OfficeXML-White-Paper-v2008-10-03.pdf — overview white paper
- http://web.mit.edu/~stevenj/www/ECMA-376-new-merged.pdf — merged readable copy

**Microsoft Learn / Open XML SDK:**
- https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document — structural overview
- https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-presentations
- https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-slide-layouts
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.themeelements?view=openxml-3.0.1
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.colorscheme?view=openxml-3.0.1
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.fontscheme?view=openxml-3.0.1
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.presentation.placeholdershape?view=openxml-3.0.1
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.blip?view=openxml-3.0.1
- https://support.microsoft.com/en-us/office/change-the-size-of-your-powerpoint-slides-040a811c-be43-40b9-8d04-0de5ed79987e

**OfficeOpenXML.com (Eric White's reference, used by python-pptx maintainers):**
- http://officeopenxml.com/prPresentation.php
- http://officeopenxml.com/anatomyofOOXML-pptx.php
- http://officeopenxml.com/prSlideMaster.php
- http://officeopenxml.com/prSlideLayout.php
- http://officeopenxml.com/prSlide-color.php
- http://officeopenxml.com/prSlide-styles-textStyles.php
- http://officeopenxml.com/drwSp-size.php
- http://officeopenxml.com/drwSp-custGeom.php

**c-rex.net OOXML element reference (per-element schema browser):**
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_themeElements_topic_ID0EYXIMB.html
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrScheme_topic_ID0ES2FMB.html
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_clrMap_topic_ID0EV3VGB.html
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_sldMaster_topic_ID0EVVBHB.html
- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_sldLayout_topic_ID0E3WAHB.html
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_txStyles_topic_ID0EHHGHB.html
- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_PlaceholderType_topic_ID0EENHIB.html
- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_SlideLayoutType_topic_ID0EKTIIB.html
- https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_rPr_topic_ID0EIB4KB.html
- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_pPr_topic_ID0ERIUKB.html
- https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_buAutoNum_topic_ID0EQZALB.html

**ooxmlsdk (Rust SDK; clean-room spec docs):**
- https://github.com/kaisery/ooxmlsdk
- https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_hierarchy.md — slide hierarchy clean-room spec
- https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_themes.md — themes
- https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/pml_notes.md — notes
- https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/drawingml.md — DrawingML
- https://github.com/kaisery/ooxmlsdk/blob/main/docs/specs/wml_drawing.md — WordprocessingML drawing (shared CT_Picture)

**python-pptx (the gold-standard Python OOXML library; its analysis docs are extraordinarily good):**
- https://python-pptx.readthedocs.io/en/stable/dev/resources/about_packaging.html
- https://python-pptx.readthedocs.io/en/stable/dev/analysis/sld-slide.html
- https://python-pptx.readthedocs.io/en/latest/dev/analysis/placeholders/
- https://python-pptx.readthedocs.io/en/latest/dev/analysis/placeholders/layout-placeholders.html
- https://github.com/scanny/python-pptx/blob/master/docs/dev/analysis/placeholders/slide-placeholders/placeholders-in-new-slide.rst
- https://github.com/scanny/python-pptx/issues/460

**Compatibility / repair / production-grade sources:**
- https://www.tamurajones.net/OfficeFileFormats.xhtml
- https://www.loc.gov/preservation/digital/formats/fdd/fdd000443.shtml — Library of Congress format registry for PPTX
- https://en.wikipedia.org/wiki/Office_Open_XML_file_formats
- https://edutechwiki.unige.ch/en/Open_Packaging_Conventions_and_Office_Open_XML
- https://www.ericwhite.com/blog/introduction-to-open-xml-series/
- https://learn.microsoft.com/en-us/office/dev/add-ins/word/create-better-add-ins-for-word-with-office-open-xml
- https://learn.microsoft.com/en-us/answers/questions/4961482/my-powerpoint-presentation-is-corrupt-in-powerpoin
- https://sharayeh.com/en/blog/repair-corrupted-powerpoint-guide

**Slideforge internal docs (companion):**
- /Users/jmagady/Dev/slideforge/.factory/planning/brand-template-patterns.md — R2 brand-template patterns (conventions, custom layouts, idx mapping)
- /Users/jmagady/Dev/slideforge/.factory/specs/decisions-applied.md — Q4 full synthesis decision

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 3 | Min-viable .pptx structure + slide-level XML + gotchas/compatibility (reasoning_effort=high) |
| Perplexity perplexity_ask | 1 | Gotchas + Keynote/Google/LibreOffice compatibility summary |
| Context7 mcp__context7__resolve-library-id | 1 | Identify ooxmlsdk Rust crate |
| Context7 mcp__context7__query-docs | 4 | ooxmlsdk capabilities (PresentationDocument::create, slide layout, image, notes master) |
| Grep | 4 | Extract sections from prior research files and brand-template-patterns.md |
| Read | 4 | Read prior research and slideforge decisions/planning docs |
| Glob | 2 | Locate factory planning files |
| Training data | 0 areas | All claims sourced — version numbers and namespace URIs verified against ECMA-376 references and ooxmlsdk specs |

**Total MCP tool calls:** 9 (3 perplexity_research + 1 perplexity_ask + 1 resolve-library-id + 4 query-docs)
**Training data reliance:** low — all element names, namespace URIs, content types, EMU values, and ordering rules are cited to ECMA-376 sections or to Microsoft Learn / OfficeOpenXML.com / ooxmlsdk's own clean-room spec docs. The only training-data-only claims are conventional patterns (rId numbering starting at 1, master IDs starting at 0x80000000) which are de-facto Microsoft conventions confirmed across multiple cited sources.
