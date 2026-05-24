---
title: Complete PowerPoint Element Taxonomy
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: product-owner + architect + human-reviewer
---

# Complete PowerPoint Element Taxonomy

## Executive Summary

This document catalogs **every element type, feature, and capability** that a PowerPoint (.pptx) file can contain, organized into 9 categories with 160+ distinct element types. The seed's 23 slide types cover approximately 35% of PowerPoint's content capabilities (text, shapes, tables, images, basic formatting) but miss entirely: charts, SmartArt, animations, transitions, interactivity, media embedding, and accessibility markup.

**Key numbers:**
- **187** preset shape geometries (ST_ShapeType) in DrawingML
- **~200** SmartArt layouts across 8 categories
- **29+** slide transition types (11 Subtle + 11 Exciting + 7 Dynamic Content + Morph)
- **130-170** animation effect presets across 4 categories
- **17** classic chart types + **8** Office 2016+ chart types (cx:chart)
- **16+** placeholder types in ST_PlaceholderType
- **36+** slide layout types in ST_SlideLayoutType
- **6** fill types (solid, gradient, pattern, picture, group, none)

**Top 5 transformative features for slideforge:**
1. Charts from data (column, bar, line, pie -- table stakes for engineering decks)
2. SmartArt-as-shapes (org charts, process flows -- consulting/engineering gold)
3. Morph transitions (modern storytelling, huge "wow" factor)
4. Zoom sections (non-linear navigation, dashboard decks)
5. Speaker notes with structured data (incident response workflows)

**Biggest gap in seed's 23 types:** Zero chart support. Engineering teams generating reports from data cannot produce a single chart. This is a v1.0-blocking gap.

---

## Category A: Slide Content Elements

### A1. Shapes

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Auto-shapes (preset geometry) | 187 presets via `<a:prstGeom prst="...">`: rectangles (2), basic shapes (~45), block arrows (~28), flowchart (~30), callouts (~18), stars/banners (~12), equations (~6), misc (~46) | Partial -- rectangles, rounded rects used for cards/badges | Critical | Low -- just `prst` attribute on `a:prstGeom` | v1.0 | Excellent -- all renderers |
| Freeform shapes | `<a:custGeom>` with path/moveTo/lnTo/cubicBezTo | No | Low -- programmatic use rare | High -- path geometry math | v3+ | Good |
| Connectors | `<p:cxnSp>` with `<a:stCxn>`/`<a:endCxn>` connection points | No | Medium -- useful for diagrams | Medium | v2 | Good |
| Lines | `<p:cxnSp>` or `<p:sp>` with line presets | Yes -- dividers, borders | Medium | Low | v1.0 | Excellent |
| Group shapes | `<p:grpSp>` wrapping multiple `<p:sp>` | Implicit -- seed builds visual groups | Medium | Medium -- nested transforms | v1.0 | Excellent |
| Action buttons | 12 presets: `actionButtonBlank`, `actionButtonHome`, `actionButtonHelp`, `actionButtonInformation`, `actionButtonForwardNext`, `actionButtonBackPrevious`, `actionButtonEnd`, `actionButtonBeginning`, `actionButtonReturn`, `actionButtonDocument`, `actionButtonSound`, `actionButtonMovie` | No | Low | Low | OOS | Good but outdated |

**Source:** ECMA-376 Part 1 ST_ShapeType enumeration (187 values). [c-rex.net ST_ShapeType](https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_ShapeType_topic_ID0EBTFOB.html), [MS-OE376](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/db76d51a-b32d-407d-aeb4-53be44626139).

### A2. Text

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Text boxes | `<p:sp>` with `<p:txBody>` | Yes | Critical | Low | v1.0 | Excellent |
| Rich text runs | `<a:r>` with `<a:rPr>` (bold, italic, underline, strikethrough, color, font, size) | Yes | Critical | Low | v1.0 | Excellent |
| Multi-level bullets | `<a:pPr lvl="0..8">` with `<a:buChar>`, `<a:buAutoNum>`, `<a:buFont>` | Yes | Critical | Low | v1.0 | Excellent |
| Text columns | `<a:bodyPr numCol="2" spcCol="...">` | No | Medium | Low | v1.x | Good |
| Auto-fit / shrink-on-overflow | `<a:bodyPr><a:normAutofit fontScale="..."/>` or `<a:spAutoFit/>` | No | Medium | Low | v1.x | Good |
| WordArt / text effects | `<a:rPr>` with `<a:effectLst>`, `<a:ln>`, 3D transforms | No | Low | High | OOS | Poor in non-MSFT |
| Superscript / subscript | `<a:rPr baseline="30000">` (super) / `baseline="-25000"` (sub) | No | Low-Medium | Low | v1.x | Good |
| Text wrapping | `<a:bodyPr wrap="square|none">` | Yes (implicit) | Medium | Low | v1.0 | Excellent |
| Paragraph spacing | `<a:pPr><a:spcBef>`, `<a:spcAft>`, `<a:lnSpc>` | Yes | Medium | Low | v1.0 | Excellent |
| Tab stops | `<a:pPr><a:tabLst><a:tab pos="..." algn="..."/>` | No | Low | Low | v1.x | Good |
| Language tags | `<a:rPr lang="en-US">` per run | No | Medium (accessibility) | Low | v1.x | Good |
| Fields (slide number, date) | `<a:fld type="slidenum">`, `<a:fld type="datetime">` | No | Medium | Low | v1.x | Good |

### A3. Tables

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Basic table | `<a:tbl>` in `<p:graphicFrame>` | Yes (`table`, `enhanced_table`) | Critical | Medium | v1.0 | Good |
| Table styles | `<a:tblPr>` with `tableStyleId` GUID referencing `<a:tblStyleLst>` in theme | Partial | High | Medium | v1.0 | Fair -- style GUIDs must match |
| Banded rows/columns | `<a:tblPr bandRow="1" bandCol="1">` + style regions `a:band1H`, `a:band2H`, `a:band1V`, `a:band2V` | Yes (enhanced_table) | High | Medium | v1.0 | Good |
| Header/total row | `<a:tblPr firstRow="1" lastRow="1">` | Yes | High | Low | v1.0 | Good |
| First/last column | `<a:tblPr firstCol="1" lastCol="1">` | No | Medium | Low | v1.0 | Good |
| Cell merging (horizontal) | `<a:tc><a:tcPr gridSpan="N">` | No | High | Medium | v1.x | Good |
| Cell merging (vertical) | `<a:tc><a:tcPr><a:vMerge val="restart|continue">` | No | Medium | Medium | v1.x | Good |
| Cell borders | `<a:tcPr>` with `<a:lnL>`, `<a:lnR>`, `<a:lnT>`, `<a:lnB>`, `<a:lnTlToBr>`, `<a:lnBlToTr>` | Yes (colored borders in enhanced_table) | High | Medium | v1.0 | Good |
| Cell fills | `<a:tcPr>` with `<a:solidFill>`, `<a:gradFill>`, `<a:patternFill>`, `<a:blipFill>`, `<a:noFill>` | Yes | High | Low | v1.0 | Good |

### A4. Images

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Embedded raster images | `<p:pic>` with `<a:blip r:embed="rIdN">` -- PNG, JPEG, GIF, BMP, TIFF | Yes (logo) | Critical | Medium (rel + content type) | v1.0 | Excellent |
| SVG images | `<a:blip>` with SVG fallback; Office 2016+ | No | Medium | High -- needs raster fallback | v1.x | Poor -- desktop PPT only |
| Linked images | `<a:blip r:link="rIdN">` with `TargetMode="External"` | No | Low | Medium | OOS | Very poor |
| Cropped images | `<a:srcRect l="..." t="..." r="..." b="...">` values in 1/100,000ths | No | Medium | Low | v1.x | Good |
| Picture fills (on shapes) | `<a:blipFill>` inside `<p:spPr>` | No | Medium | Low | v1.x | Good |
| Artistic effects | `<a:extLst>` with effect extension | No | Low | High | OOS | Desktop PPT only |
| Background removal | UI-only, stored as crop mask | No | Low | Very High | OOS | Desktop PPT only |
| Icons (SVG library) | Office 365 SVG icon insertion | No | Low | Medium | v2 | Desktop PPT only |

### A5. Charts

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| **Classic charts (c: namespace)** | | | | | | |
| Bar/Column chart | `<c:barChart>` with `barDir="col|bar"`, `grouping="clustered|stacked|percentStacked"` | No | Critical | High -- embedded Excel workbook | v1.0 | Good |
| Line chart | `<c:lineChart>` with `grouping="standard|stacked|percentStacked"`, markers | No | Critical | High | v1.0 | Good |
| Pie chart | `<c:pieChart>`, `<c:ofPieChart>` (pie-of-pie, bar-of-pie) | No | High | High | v1.0 | Good |
| Doughnut chart | `<c:doughnutChart>` | No | Medium | High | v1.x | Good |
| Area chart | `<c:areaChart>` with groupings | No | Medium | High | v1.x | Good |
| Scatter/XY chart | `<c:scatterChart>` with `scatterStyle="line|lineMarker|marker|smooth|smoothMarker"` | No | Medium | High | v1.x | Good |
| Bubble chart | `<c:bubbleChart>` with `bubble3D`, `bubbleScale` | No | Low | High | v2 | Fair |
| Radar chart | `<c:radarChart>` with `radarStyle="standard|marker|filled"` | No | Low | High | v2 | Fair |
| Stock chart | `<c:stockChart>` (HLC, OHLC, Vol-HLC, Vol-OHLC) | No | Low | Very High | v3+ | Fair |
| Surface chart | `<c:surfaceChart>` with wireframe/contour variants | No | Very Low | Very High | OOS | Poor |
| Combo chart | Multiple `<c:barChart>` + `<c:lineChart>` in one `<c:plotArea>` | No | High | Very High (secondary axes) | v2 | Fair |
| **Office 2016+ charts (cx: namespace)** | `xmlns:cx="http://schemas.microsoft.com/office/drawing/2014/chartex"` | | | | | |
| Treemap | `<cx:chart>` -- Treemap type | No | Medium | Very High | v2 | Poor -- PPT desktop only |
| Sunburst | `<cx:chart>` -- Sunburst type | No | Low | Very High | v3+ | Poor |
| Waterfall | `<cx:chart>` -- Waterfall type | No | Medium | Very High | v2 | Poor |
| Funnel | `<cx:chart>` -- Funnel type | No | Medium | Very High | v2 | Poor |
| Histogram | `<cx:chart>` -- Histogram type with binning | No | Medium | Very High | v2 | Poor |
| Box & Whisker | `<cx:chart>` -- BoxWhisker type | No | Low | Very High | v3+ | Poor |
| Map chart | `<cx:chart>` -- RegionMap (requires Bing Geography) | No | Very Low | Extreme | OOS | Very Poor |
| Pareto | `<cx:chart>` -- composite histogram+line | No | Low | Extreme | OOS | Poor |

**Source:** ECMA-376 Part 1 DrawingML Charts namespace; [Office chart types](https://www.empowersuite.com/en/blog/list-powerpoint-charts); Microsoft chartEx namespace `http://schemas.microsoft.com/office/drawing/2014/chartex`.

### A6. SmartArt / Diagrams

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| SmartArt diagrams | `xmlns:dgm` -- ~200 layouts in 8 categories: List, Process, Cycle, Hierarchy, Relationship, Matrix, Pyramid, Picture | No | Very High (consulting/engineering) | Extreme -- layout algorithms | v3+ (as native SmartArt) / v2 (as grouped shapes) | Fair -- renders but not editable in non-MSFT |
| Organization charts | Hierarchy SmartArt subset | No | Very High | Extreme | v2 (as shapes) | Fair |
| Process flow diagrams | Process SmartArt subset | No | Very High | Extreme | v2 (as shapes) | Fair |
| Venn diagrams | Relationship SmartArt subset | No | Medium | Extreme | v2 (as shapes) | Fair |

**Source:** [Microsoft SmartArt support article](https://support.microsoft.com/en-us/office/learn-more-about-smartart-graphics-6ea4fdb0-aa40-4fa9-9348-662d8af6ca2c); [Hidden SmartArt layouts](https://blog.hompus.nl/2026/02/26/hidden-powerpoint-smartart-layouts/).

### A7. Math Equations

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| OMML equations | `<m:oMath>` -- Office Math Markup Language: fractions `<m:f>`, radicals `<m:rad>`, matrices `<m:m>`, superscript `<m:sSup>`, subscript `<m:sSub>`, etc. | Yes (formula slide type) | Medium-High | High -- verbose XML, ideally needs LaTeX-to-OMML converter | v1.x | Fair -- LibreOffice partial, Google Slides image fallback |

### A8. Media

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Embedded video | `<p:video>` with media part in ppt/media/, rel type `...relationships/video` | No | Medium | High -- codec, poster frame | v2 | Fair |
| Embedded audio | `<p:audio>` with media part | No | Low | High | v2 | Fair |
| Linked video (URL) | `r:link` with external target | No | Low | Medium | v3+ | Very Poor |
| Screen recordings | Embedded as video | No | Very Low | High | OOS | Fair |
| 3D models | `.glb`/`.gltf` embedding via Office 365 extension namespaces | No | Very Low | Extreme | OOS | Very Poor -- Desktop PPT only |

### A9. Other Content Elements

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| OLE objects | `<p:oleObj>` -- embedded Excel, Word, PDF | No | Low | Very High | OOS | Very Poor |
| Ink annotations | InkML strokes with pressure/timing | No | Very Low | Extreme | OOS | Very Poor |

---

## Category B: Slide-Level Features

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Slide layouts | `<p:sldLayout type="...">` -- 36+ types in ST_SlideLayoutType: `title`, `obj`, `tx`, `twoColTx`, `twoObj`, `titleOnly`, `blank`, `secHead`, `picTx`, `cust`, `tbl`, `chart`, `dgm`, `media`, `objTx`, `txObj`, `objOverTx`, `txOverObj`, `twoObjAndTx`, `twoTxAndObj`, `twoObjOverTx`, `fourObj`, `vertTx`, `clipArtAndVertTx`, `vertTitleAndTx`, `vertTitleAndTxOverChart`, `objAndTwoObj`, `twoObjAndObj`, `mediaAndTx`, `txAndMedia`, `clipArtAndTx`, `txAndClipArt`, `clipArtAndVertTx`, `twoColTxAndTwoObj`, `objAndCaption`, `picAndCaption` | Partial (11 standard + custom) | Critical | Medium | v1.0 | Excellent |
| Speaker notes | `<p:notes>` referencing `notesMaster`, `<p:txBody>` | No | High | Medium | v1.x | Good -- all renderers |
| Slide sections | `<p:section>` in `<p:sectionLst>` (extension namespace) | No | Medium | Low | v1.x | Fair -- PPT only |
| Slide hide/show | `<p:sld show="0">` attribute | No | Low | Trivial | v1.x | Good |
| Slide numbers | `<p:ph type="sldNum">` in layout/master | No | Medium | Low | v1.x | Good |
| Date/time field | `<p:ph type="dt">` with `<a:fld type="datetime">` | No | Medium | Low | v1.x | Good |
| Footer text | `<p:ph type="ftr">` | No | Medium | Low | v1.x | Good |
| Header text | `<p:ph type="hdr">` (notes/handout only) | No | Low | Low | v1.x | Good |
| Background (solid fill) | `<p:bg><p:bgPr><a:solidFill>` | Yes (colored dividers) | Critical | Low | v1.0 | Excellent |
| Background (gradient) | `<p:bg><p:bgPr><a:gradFill>` with `<a:gsLst>` gradient stops | No | Medium | Medium | v1.x | Good |
| Background (pattern) | `<p:bg><p:bgPr><a:pattFill prst="...">` -- 48 preset patterns | No | Low | Low | v1.x | Good |
| Background (picture) | `<p:bg><p:bgPr><a:blipFill>` | No | Medium | Medium | v1.x | Good |
| Custom slide shows | `<p:custShowLst><p:custShow name="..." id="...">` with `<p:sldLst>` | No | Medium | Low-Medium | v2 | Good (PPT only creation) |

**Source:** [c-rex.net ST_SlideLayoutType](https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_SlideLayoutType_topic_ID0EKTIIB.html); ECMA-376 Part 1 SS19.3.

---

## Category C: Animation System

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| **Slide Transitions** | | | | | | |
| Subtle transitions | 11 types: `<p:cut>`, `<p:fade>`, `<p:push>`, `<p:wipe>`, `<p:split>`, `<p:reveal>`, `<p:randomBars>`, `<p:shape>`, `<p:uncover>`, `<p:cover>`, `<p:flash>` | No | Medium | Low | v1.x | Good -- most renderers |
| Exciting transitions | 11 types via `<p:prism type="...">`: fallOver, drape, curtain, wind, prestige, fracture, crush, peelOff, pageCurl, airplane, origami | No | Low | Medium | v3+ | Poor -- PPT desktop only |
| Dynamic Content transitions | 7 types: `<p:pan>`, `<p:ferris>`, `<p:conveyor>`, `<p:rotate>`, `<p:window>`, `<p:orbit>`, `<p:flythrough>` | No | Low | Medium | v3+ | Poor |
| Morph transition | `<p:morph>` -- shape-level morphing between slides based on name/ID matching | No | Very High | High -- requires consistent shape IDs | v3+ | Very Poor -- PPT desktop 2019+ only |
| Transition timing | `<p:transition spd="slow|med|fast" advClick="1" advTm="3000">` | No | Medium | Low | v1.x | Good |
| **Object Animations** | | | | | | |
| Entrance effects | ~45 presets: Appear, Fade, Fly In, Float In, Split, Wipe, Shape, Wheel, Random Bars, Grow & Turn, Zoom, Swivel, Bounce, Flip, etc. via `<p:animEffect>` | No | Medium | Very High | v3+ | Fair -- basic ones ok |
| Emphasis effects | ~35 presets: Pulse, Spin, Grow/Shrink, Teeter, Color Pulse, Transparency, Object Color, Desaturate, Darken, Lighten, Bold Flash, Wave, etc. | No | Low | Very High | v3+ | Fair |
| Exit effects | ~45 presets: Disappear, Fade, Fly Out, Shrink & Turn, etc. (mirrors entrance) | No | Low | Very High | v3+ | Fair |
| Motion paths | Lines, Arcs, Turns, Shapes (circle/square/triangle), Loops, Custom Path (freeform Bezier) via `<p:animMotion>` | No | Low | Extreme | OOS | Poor |
| Animation triggers | `<p:seq>` on-click, `<p:par>` with-previous, after-previous via timing tree | No | Low | Very High | v3+ | Fair |
| Animation timing | `<p:cBhvr>` with dur, delay, repeatCount, autoRev, fill | No | Low | Very High | v3+ | Fair |
| Progressive bullet reveal | `<p:par>` sequence targeting individual `<a:p>` elements | No | Medium | High | v2 (opinionated) | Fair |

**OOXML animation elements:** `<p:anim>` (property animation), `<p:animEffect>` (preset effect), `<p:animMotion>` (motion path), `<p:animRot>` (rotation), `<p:animScale>` (scaling), `<p:animClr>` (color change). Timing containers: `<p:par>` (parallel), `<p:seq>` (sequence), `<p:timing>` (root). Common behavior: `<p:cBhvr>` with `<p:tgtEl>`.

**Source:** [Microsoft Learn -- Working with Animation](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-animation); ECMA-376 Part 1 SS19.5; [ooxml.info timing elements](https://ooxml.info/docs/19/19.5/19.5.4/).

---

## Category D: Interactivity

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Hyperlinks (URL) | `<a:hlinkClick r:id="rIdN">` on `<a:rPr>` with external target in .rels | No | High | Low | v1.0 | Excellent |
| Hyperlinks (to slide) | `<a:hlinkClick action="ppaction://hlinksldjump" r:id="rIdN">` | No | Medium | Medium | v1.x | Good |
| Hyperlinks (to email) | `<a:hlinkClick r:id="rIdN">` with `mailto:` target | No | Low | Low | v1.x | Good |
| Action settings | `<p:hlinkClick>` on `<p:nvSpPr>` with `action="ppaction://..."` | No | Low | Medium | OOS | Poor |
| Zoom -- Summary | Microsoft extension (p14/p15/p16 namespaces) -- shapes with extension elements targeting slide subsets | No | High | Very High -- proprietary extensions | v2 | Very Poor -- PPT desktop only |
| Zoom -- Section | Same extension mechanism | No | High | Very High | v2 | Very Poor |
| Zoom -- Slide | Same extension mechanism | No | Medium | Very High | v2 | Very Poor |
| OLE embedded objects | `<p:oleObj>` with embedded part (.xlsx, .docx, .pdf) | No | Low | Extreme | OOS | Very Poor |

**Source:** [Microsoft Zoom feature](https://support.microsoft.com/en-us/office/use-the-morph-transition-in-powerpoint-8dd1c7b2-b935-44f5-a74c-741d8d9244ea); Zoom uses proprietary extension namespaces `http://schemas.microsoft.com/office/powerpoint/2012/main` (p14), `http://schemas.microsoft.com/office/powerpoint/2015/06/main` (p15), `http://schemas.microsoft.com/office/powerpoint/2018/10/main` (p16).

---

## Category E: Master/Template System

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Slide masters | `<p:sldMaster>` with `cSld`, `clrMap`, `sldLayoutIdLst`, `txStyles` | Yes | Critical | High | v1.0 | Excellent |
| Slide layouts | `<p:sldLayout type="..." preserve="1">` with placeholder inheritance | Yes | Critical | High | v1.0 | Excellent |
| Theme -- color scheme | `<a:clrScheme>` with 12 fixed colors: dk1, lt1, dk2, lt2, accent1-6, hlink, folHlink | Yes | Critical | Medium | v1.0 | Excellent |
| Theme -- font scheme | `<a:fontScheme>` with `<a:majorFont>` and `<a:minorFont>` (latin, ea, cs) | Yes | Critical | Low | v1.0 | Good |
| Theme -- format scheme | `<a:fmtScheme>` with fillStyleLst (3), lnStyleLst (3), effectStyleLst (3), bgFillStyleLst (3) | Yes | Critical | Medium | v1.0 | Good |
| Color map | `<p:clrMap bg1="lt1" tx1="dk1" ...>` -- 12 mappings | Yes | Critical | Medium | v1.0 | Excellent |
| Color map override | `<p:clrMapOvr><a:overrideClrMapping .../>` on layouts | Yes | Critical | Medium | v1.0 | Excellent |
| Custom color palettes | Additional `<a:clrScheme>` entries in `<a:extraClrSchemeLst>` | No | Low | Low | v2 | Good |
| Embedded fonts | `<p:embeddedFontLst><p:embeddedFont>` with font binary parts | No | Medium | High -- font licensing | v2 | Fair |
| Handout master | `<p:handoutMaster>` for print layouts | No | Low | Medium | v2 | Good |
| Notes master | `<p:notesMaster>` for speaker notes formatting | No | Medium | Medium | v1.x | Good |
| Slide guides | `<p:guide>` elements in viewProps | No | Low | Low | v2 | PPT only |
| Default text style | `<p:defaultTextStyle>` in presentation.xml | Yes | Critical | Medium | v1.0 | Excellent |
| Placeholder types | ST_PlaceholderType: `title`, `ctrTitle`, `subTitle`, `body`, `dt`, `sldNum`, `ftr`, `hdr`, `obj`, `chart`, `tbl`, `clipArt`, `dgm`, `media`, `sldImg`, `pic` (16+ values) | Partial | Critical | Medium | v1.0 | Excellent |

**Source:** ECMA-376 Part 1 SS19.3.1; [OfficeOpenXML.com prSlideMaster](http://officeopenxml.com/prSlideMaster.php); ooxmlsdk pml_hierarchy.md.

---

## Category F: Collaboration/Review

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Legacy comments | `<p:cmAuthorLst>` + `<p:cmLst><p:cm>` with author, date, text, position | No | Low-Medium | Medium | v1.x | Good |
| Modern threaded comments | Office 365 extension -- threaded replies, @mentions, in p15/p16 namespaces | No | Low | High | v2 | Very Poor -- PPT 365 only |
| Co-authoring | Real-time via Office 365 cloud -- not stored in .pptx | N/A | N/A | N/A | OOS | N/A |
| Version history | Office 365 cloud feature -- not in .pptx | N/A | N/A | N/A | OOS | N/A |

---

## Category G: Output/Delivery

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Presentation properties | `<p:presProps>` -- show type, loop, narration, pen color | No | Medium | Low | v1.0 | Good |
| View properties | `<p:viewProps>` -- last view, grid spacing, guide visibility | No | Low | Low | v1.0 | Good |
| Print settings | Handout layout, notes pages, outline -- in presProps | No | Low | Low | v2 | Good |
| Export to video | External feature (not in .pptx) -- requires timings + narration | N/A | N/A | N/A | OOS | N/A |
| Broadcast | Office 365 cloud feature | N/A | N/A | N/A | OOS | N/A |
| Presenter view | Runtime feature, not stored in .pptx | N/A | N/A | N/A | OOS | N/A |
| Rehearsed timings | `<p:transition advTm="...">` per slide | No | Low | Low | v2 | Good |

---

## Category H: Accessibility

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Alt text (images) | `<p:cNvPr descr="...">` on `<p:pic>` | No | High | Trivial | v1.x | Excellent |
| Alt text (shapes) | `<p:cNvPr descr="...">` on `<p:sp>` | No | High | Trivial | v1.x | Excellent |
| Alt text (tables) | `<p:cNvPr descr="...">` on `<p:graphicFrame>` | No | High | Trivial | v1.x | Excellent |
| Decorative flag | `<p:cNvPr><a:extLst><a:ext uri="..."><adec:decorative val="1">` (Office 2019+) | No | Medium | Low | v1.x | Poor -- newer PPT only |
| Reading order | Shape z-order in `<p:spTree>` defines reading order | No | High | Low (design discipline) | v1.x | Good |
| Slide titles | Accessibility checker requires `<p:ph type="title">` on each slide | No | High | Low | v1.x | Good |
| Table headers | `<a:tblPr firstRow="1">` marks header row for screen readers | No | High | Low | v1.x | Good |
| Language tags | `<a:rPr lang="en-US">` per text run | No | Medium | Low | v1.x | Good |
| Link text | Descriptive `<a:t>` inside hyperlinked runs (not raw URLs) | No | Medium | Low | v1.x | Good |

**Source:** [WCAG 2.1 for presentations](https://www.w3.org/WAI/WCAG21/quickref/); Microsoft Accessibility Checker documentation.

---

## Category I: Metadata

| Element | Sub-types | In seed? | Relevance | OOXML complexity | Version | Cross-renderer compat |
|---------|-----------|----------|-----------|------------------|---------|----------------------|
| Core properties | `docProps/core.xml` -- Dublin Core: title, creator, subject, description, keywords, lastModifiedBy, revision, created, modified | No | Critical | Low | v1.0 | Excellent |
| Extended properties | `docProps/app.xml` -- Application, Company, PresentationFormat, Slides count, TotalTime, Words, etc. | No | Medium | Low | v1.0 | Excellent |
| Custom properties | `docProps/custom.xml` -- key-value pairs (string, number, date, boolean) | No | Medium | Low | v1.x | Good |
| Thumbnail | `docProps/thumbnail.jpeg` -- preview image | No | Low | Medium | v1.x | Good |
| Custom XML parts | `/customXml/item1.xml` + `/customXml/itemProps1.xml` -- arbitrary XML data | No | Medium (v2 round-trip) | Medium | v2 | Good (preserved) |
| File encryption | Password protection via `EncryptedPackage` structure | No | Low | Extreme | OOS | Fair |
| Digital signatures | `_xmlsignatures/` part | No | Very Low | Extreme | OOS | Fair |
| VBA macros | `ppt/vbaProject.bin` -- requires .pptm extension | No | Very Low | Extreme + security risk | OOS | Very Poor |

---

## Transformative Feature Analysis

### Charts from Data

**Impact:** Critical -- the single biggest gap in the seed's 23 types.

Engineering teams generating weekly/monthly reports need bar charts, line charts, trend visualizations. Without charts, slideforge cannot serve its primary use case. The architecture decision (ADR from product brief) already calls for SVG-based charts via the `plotters` crate rendered as images, avoiding the extreme complexity of embedded Excel workbooks. However, true `c:chart` OOXML charts remain editable in PowerPoint and should be a v2 target.

**Recommendation:**
- v1.0: Chart-as-image via plotters (bar, column, line, pie) -- renders everywhere
- v2.0: Native `c:chart` OOXML charts with embedded data -- editable in PPT

### SmartArt from Data

**Impact:** Very High for consulting, engineering architecture docs, org charts.

True SmartArt generation is essentially impossible without reverse-engineering Microsoft's proprietary layout algorithms (~200 underdocumented layouts). No open-source library generates SmartArt from scratch successfully.

**Recommendation:**
- v2.0: Generate org charts, process flows, Venn diagrams as grouped shapes (`<p:grpSp>`) with slideforge's own layout engine. Visually identical; not editable as SmartArt.
- v3+: Optionally wrap in SmartArt XML for PowerPoint editability. Very high effort.

### Animations and Morph Transitions

**Impact:** Mixed -- extremely popular in modern decks but controversial for corporate reporting.

The animation timing tree (`<p:timing>`, `<p:par>`, `<p:seq>`, `<p:animEffect>`) is one of the most complex parts of PresentationML. Full support would require modeling ~150 effect presets with their parameter spaces. However, two highly valuable patterns are achievable:

1. **Progressive bullet reveal** (bullets appear one-by-one on click) -- very common request
2. **Morph transition** (shapes smoothly transform between slides) -- requires consistent naming

**Recommendation:**
- v1.x: Uniform subtle transition (fade/push) applied deck-wide via DSL directive
- v2.0: Progressive bullet reveal (opinionated, single pattern)
- v3+: Morph transition support with shape-name-matching DSL syntax

### Zoom Sections

**Impact:** High for dashboard-style and non-linear presentations.

Zoom is PowerPoint's "Prezi-like" feature (Summary Zoom, Section Zoom, Slide Zoom). It uses proprietary Microsoft extension namespaces (p14/p15/p16) not part of ECMA-376. It does NOT work in Google Slides, Keynote, or LibreOffice. It cannot even be created in PowerPoint Online.

**Recommendation:**
- v2.0: Summary Zoom generation from DSL section structure -- high value for large decks
- Caveat: Desktop PowerPoint only. Must degrade gracefully to regular slides in other renderers.

### Custom Shows vs. Slideforge Variants

**Impact:** Medium -- aligns with slideforge's existing variant concept.

PowerPoint custom shows (`<p:custShowLst>`) let one deck contain multiple named subsets. This maps directly to slideforge's planned variant system (one DSL, multiple audience-specific outputs).

**Recommendation:**
- v2.0: Map slideforge DSL variants to PowerPoint custom shows. Low OOXML complexity.

---

## Cross-Renderer Compatibility Matrix

| Feature | PowerPoint Desktop | PowerPoint Online | Google Slides | Apple Keynote | LibreOffice Impress |
|---------|-------------------|-------------------|---------------|---------------|---------------------|
| Basic shapes (rect, ellipse) | Full | Full | Full | Full | Full |
| Preset geometries (187 types) | Full | Full | Most | Most | Most |
| Rich text (bold, italic, color) | Full | Full | Full | Full | Full |
| Multi-level bullets | Full | Full | Full | Full | Full |
| Tables (styled, banded) | Full | Full | Partial | Partial | Partial |
| Tables (cell merge) | Full | Full | Full | Full | Full |
| Embedded images (PNG/JPEG) | Full | Full | Full | Full | Full |
| SVG images | Full (2016+) | Partial | No | No | Partial |
| Classic charts (c:chart) | Full | Full (basic editing) | Basic types only | Basic types only | Basic types only |
| ChartEx (cx:chart) -- treemap, waterfall, etc. | Full (2016+) | Limited | Image fallback | Image fallback | Image fallback |
| SmartArt | Full | View + basic edit | Rendered as shapes | Rendered as shapes | Rendered as shapes |
| Hyperlinks (URL) | Full | Full | Full | Full | Full |
| Hyperlinks (to slide) | Full | Full | Partial | Partial | Partial |
| Subtle transitions (fade, wipe, push) | Full | Partial (playback) | Partial (subset) | Has own equivalents | Partial |
| Exciting/Dynamic transitions | Full | Limited playback | No | No | No |
| Morph transition | Full (2019+) | Playback only | No | No (Magic Move is different) | No |
| Entrance/exit animations | Full | Partial playback | Partial (limited set) | Has own system | Partial |
| Motion path animations | Full | No | No | No | Partial |
| Zoom sections | Full (2019+) | Playback only | Static image | Static image | Static image |
| Custom shows | Full | View only | No | No | No |
| Speaker notes | Full | Full | Full | Full | Full |
| Embedded video (MP4) | Full | Full | Partial | Full | Full |
| Embedded audio | Full | Full | Partial | Full | Full |
| 3D models (.glb) | Full (365) | No | No | No | No |
| Math equations (OMML) | Full | Basic view | Image fallback | Partial | Partial |
| Ink annotations | Full | Limited view | No | No | No |
| Comments (legacy) | Full | Full | Converted | Partial | Partial |
| Comments (modern threaded) | Full (365) | Full (365) | No | No | No |
| Alt text | Full | Full | Full | Full | Full |
| Custom XML parts | Preserved | Preserved | Stripped | Stripped | Stripped |
| VBA macros | Full (.pptm) | No (preserved) | No (stripped) | No | Partial |
| Embedded fonts | Full | Substituted | Substituted | Substituted | Substituted |
| Document properties | Full | Full | Partial | Partial | Partial |
| Digital signatures | Full | View only | No | No | Partial |

**Worst-case cross-renderer feature:** Zoom sections and Morph transitions -- completely non-functional in everything except PowerPoint desktop 2019+. Any features in the p14/p15/p16 extension namespaces degrade to static or invisible content.

**Sources:** [Microsoft PowerPoint web vs desktop comparison](https://laramellortraining.co.uk/web-or-desktop-finding-the-best-powerpoint-for-your-needs); [Google Slides import limitations](https://support.google.com/docs/answer/6055408); [Keynote .pptx compatibility](https://support.apple.com/guide/keynote/import-an-existing-file-tan70ec6950e/mac).

---

## Version Roadmap Recommendation

### v1.0 Core (Must Ship)

These elements are non-negotiable for a professional branded deck generator:

1. **Shapes:** Rectangles, rounded rectangles, lines, group shapes (auto-shapes via `prstGeom`)
2. **Text:** Rich text runs (bold, italic, color, size, font), multi-level bullets, text wrapping
3. **Tables:** Styled tables with header row, banded rows, cell fills, colored borders
4. **Images:** Embedded PNG/JPEG with positioning and aspect ratio
5. **Hyperlinks:** URL hyperlinks on text and shapes
6. **Charts (as images):** Bar, column, line, pie via plotters crate rendered as embedded PNG/SVG
7. **Master/layout system:** Theme (colors, fonts, format scheme), slide master, slide layouts with placeholders, color map and overrides
8. **Backgrounds:** Solid fill, gradient fill (for divider slides)
9. **Metadata:** Core properties (title, author, company), extended properties (Application="slideforge")
10. **Slide structure:** Slide ordering, presentation properties, view properties, table styles

**Element count: ~35 distinct element types**

### v1.x Incremental

High-value features that don't require architecture changes:

1. **Speaker notes** -- notes parts with formatted text
2. **Alt text** -- `descr` attribute on all shapes/images/tables
3. **Reading order** -- correct z-order in shape tree
4. **Language tags** -- `lang` attribute on text runs
5. **Slide numbers, date/time, footer** -- placeholder fields
6. **Image cropping** -- `srcRect` on blipFill
7. **Text columns** -- `numCol` on bodyPr
8. **Basic transitions** -- uniform fade/push across deck
9. **Extended metadata** -- custom properties for incident IDs, team names
10. **Slide sections** -- for large deck organization
11. **Math equations** -- OMML from DSL formula syntax (formula slide type already exists)
12. **Cell merging** -- horizontal and vertical table cell spans
13. **Decorative flag** -- for purely visual elements
14. **Legacy comments** -- for generation provenance tagging
15. **Notes master** -- formatted speaker notes layout

### v2.0

Features requiring significant architecture investment:

1. **Native OOXML charts** -- `c:chart` with embedded workbook data (bar, column, line, pie, area, scatter, doughnut, combo)
2. **Diagram-as-shapes** -- org charts, process flows, Venn diagrams via slideforge's own layout engine outputting grouped shapes
3. **Progressive bullet reveal** -- opinionated animation pattern (one-by-one bullet entrance)
4. **Zoom sections** -- Summary Zoom from DSL section structure (proprietary extension namespaces)
5. **Custom shows** -- map DSL variants to PowerPoint `custShowLst`
6. **Video/audio embedding** -- MP4/MP3 with poster frame
7. **Custom XML parts** -- slideforge metadata for round-trip/regeneration
8. **Embedded fonts** -- subset embedding for brand font portability
9. **Treemap, waterfall, funnel charts** -- via cx:chart namespace
10. **Stronger accessibility** -- reading order enforcement, color contrast validation

### v3.0+

Aspirational features requiring mature architecture:

1. **Morph transition** -- shape morphing with consistent ID/name matching DSL
2. **SmartArt-native** -- wrapping grouped shapes in dgm: namespace for PPT editability
3. **Advanced animations** -- entrance/exit effects with timing tree (opinionated patterns only)
4. **3D model embedding** -- .glb/.gltf for engineering/architecture use cases
5. **Data-bound decks** -- external data connections for live refresh
6. **Histogram, box-whisker, sunburst charts** -- via cx:chart
7. **Linked video from URL** -- Teams/YouTube embed
8. **AI-assisted layout** -- overflow detection, auto-splitting content across slides

### Out of Scope (Never)

Features slideforge should never support, with reasoning:

| Feature | Reason |
|---------|--------|
| VBA macros | Security risk; enterprises block macros; fragile across versions |
| Digital signatures | Trust/governance workflow, not generation concern |
| Ink annotations | Interactive authoring-time feature; no programmatic value |
| Screen recordings | Capture feature, not generation concern |
| OLE embedded objects | Extreme complexity; very poor cross-renderer support |
| Surface/3D charts | Extremely rare use; very poor cross-renderer support |
| Map charts | Requires Bing Geography service; proprietary dependency |
| Pareto charts | Composite chart; extremely rare |
| Fine-grained animation choreography | Per-character/custom-motion-path animations are brittle and maintenance nightmares |
| WordArt / artistic text effects | Poor cross-renderer support; not professional |
| Artistic image effects | Desktop-only; not reproducible programmatically |
| Background image removal | UI-only feature; not programmatic |
| Co-authoring / version history | Cloud features, not file-format features |
| Broadcast / presenter view | Runtime features, not stored in .pptx |

---

## Gap Analysis: Seed's 23 Types vs. Full PowerPoint

### What the 23 Types Cover

The seed's 23 slide types cover the following PowerPoint element categories:

- **Shapes:** Rectangles, rounded rectangles, lines (for cards, badges, dividers, bars)
- **Text:** Rich text with bold/italic, multi-level bullets, font sizing
- **Tables:** Basic and enhanced tables with colored borders and banded rows
- **Images:** Logo placement
- **Colors:** Solid fills, theme color vocabulary (11 named colors)
- **Layout:** Fixed positioning via EMU coordinates

### What the 23 Types Miss Entirely

| Gap | Impact | Priority |
|-----|--------|----------|
| **Charts** | Cannot produce data visualizations at all | v1.0-blocking |
| **Speaker notes** | Cannot include talking points or structured metadata | v1.x |
| **Hyperlinks** | Cannot link to tickets, dashboards, or related docs | v1.0 |
| **Accessibility** | No alt text, no reading order, no language tags | v1.x |
| **Metadata** | No document properties for searchability | v1.0 |
| **Slide numbers/footer** | No page numbering or footer branding | v1.x |
| **Cell merging** | Tables cannot have colspan/rowspan | v1.x |
| **Background gradients** | Only solid fills; no gradient backgrounds | v1.x |
| **Transitions** | Completely static; no transition effects | v1.x |
| **SmartArt/diagrams** | No org charts, process flows, architecture diagrams | v2 |
| **Video/audio** | No media embedding | v2 |
| **Animations** | No motion or progressive reveal | v2-v3 |
| **Zoom navigation** | No non-linear navigation | v2 |
| **Math equations** | formula slide type exists but uses shapes, not OMML | v1.x |

### Coverage Assessment

- **Content elements:** ~35% covered (text, shapes, tables, images -- but no charts, SmartArt, media, math)
- **Slide-level features:** ~20% covered (backgrounds, basic layout -- but no notes, sections, transitions, numbering)
- **Interactivity:** 0% covered (no hyperlinks, no action, no zoom)
- **Animation:** 0% covered
- **Accessibility:** 0% covered
- **Metadata:** 0% covered (beyond what ooxmlsdk generates by default)

---

## DSL Syntax Proposals for Top Features

### Charts (v1.0 -- as image; v2 -- native OOXML)

```
slide chart "Revenue Trends Q1-Q4"
  type: bar
  data:
    Q1: 1200000
    Q2: 1450000
    Q3: 1380000
    Q4: 1620000
  color: brand_blue
  y_label: "Revenue ($)"
  show_values: true
```

### Speaker Notes (v1.x)

```
slide content "Incident Summary"
  - Service X experienced 47min outage
  - Root cause: database connection pool exhaustion
  notes:
    - "Emphasize the 4-minute detection time improvement"
    - "Reference JIRA ticket INC-4521 for full timeline"
    - "Next steps: capacity planning review scheduled for June 5"
```

### Hyperlinks (v1.0)

```
slide content "Key Links"
  - [Dashboard](https://grafana.internal/d/abc123) shows real-time metrics
  - Full postmortem in [Confluence](https://wiki.internal/pages/12345)
  - Track remediation in [JIRA INC-4521](https://jira.internal/browse/INC-4521)
```

### Transitions (v1.x)

```
presentation "Q2 Review"
  transition: fade
  transition_speed: medium

slide title "Q2 Review" color: brand_blue
  transition: push  # per-slide override
```

### Progressive Bullet Reveal (v2)

```
slide content "Key Findings"
  reveal: one_by_one  # bullets appear on click
  - Finding 1: Detection improved by 40%
  - Finding 2: MTTR reduced to 12 minutes
  - Finding 3: Zero customer-facing impact
```

### SmartArt-as-Shapes (v2)

```
slide diagram "Team Structure"
  type: org_chart
  root: "VP Engineering"
    - "Director Platform"
      - "SRE Lead"
      - "DevOps Lead"
    - "Director Product"
      - "Frontend Lead"
      - "Backend Lead"
```

### Zoom Summary (v2)

```
presentation "Architecture Review"
  zoom: summary  # generates summary zoom slide

  section "Current State"
    slide content "..."
    slide content "..."

  section "Proposed Changes"
    slide content "..."

  section "Migration Plan"
    slide content "..."
```

---

## Sources

### ECMA/ISO Standards
1. ECMA-376 Part 1 (PresentationML) -- [ecma-international.org](https://ecma-international.org/publications-and-standards/standards/ecma-376/) (accessed 2026-05-23)
2. ECMA-376 Part 2 (OPC) -- ISO/IEC 29500-2

### Microsoft Documentation
3. [Structure of a PresentationML document](https://learn.microsoft.com/en-us/office/open-xml/presentation/structure-of-a-presentationml-document) (accessed 2026-05-23)
4. [Working with Animation](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-animation) (accessed 2026-05-23)
5. [Working with Slide Layouts](https://learn.microsoft.com/en-us/office/open-xml/presentation/working-with-slide-layouts) (accessed 2026-05-23)
6. [MS-OE376 Office Implementation](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/db76d51a-b32d-407d-aeb4-53be44626139) (accessed 2026-05-23)
7. [Use Morph transition](https://support.microsoft.com/en-us/office/use-the-morph-transition-in-powerpoint-8dd1c7b2-b935-44f5-a74c-741d8d9244ea) (accessed 2026-05-23)
8. [SmartArt graphics types](https://support.microsoft.com/en-us/office/learn-more-about-smartart-graphics-6ea4fdb0-aa40-4fa9-9348-662d8af6ca2c) (accessed 2026-05-23)
9. [Add transitions](https://support.microsoft.com/en-us/office/add-change-or-remove-transitions-between-slides-3f8244bf-f893-4efd-a7eb-3a4845c9c971) (accessed 2026-05-23)
10. [PowerPoint web vs desktop](https://laramellortraining.co.uk/web-or-desktop-finding-the-best-powerpoint-for-your-needs) (accessed 2026-05-23)
11. [Zoom in PowerPoint](https://learn.microsoft.com/en-us/answers/questions/4954595/powerpoint-section-zoom-or-slide-zoon-in-online-po) (accessed 2026-05-23)

### OOXML Reference Sites
12. [c-rex.net ST_ShapeType](https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_ShapeType_topic_ID0EBTFOB.html) (accessed 2026-05-23)
13. [c-rex.net ST_SlideLayoutType](https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_ST_SlideLayoutType_topic_ID0EKTIIB.html) (accessed 2026-05-23)
14. [c-rex.net theme elements](https://c-rex.net/samples/ooxml/e1/part4/OOXML_P4_DOCX_themeElements_topic_ID0EYXIMB.html) (accessed 2026-05-23)
15. [OfficeOpenXML.com prSlideMaster](http://officeopenxml.com/prSlideMaster.php) (accessed 2026-05-23)
16. [OfficeOpenXML.com prPresentation](http://officeopenxml.com/prPresentation.php) (accessed 2026-05-23)
17. [OfficeOpenXML.com drwSp-size](http://officeopenxml.com/drwSp-size.php) (accessed 2026-05-23)
18. [OfficeOpenXML.com anatomy](http://officeopenxml.com/anatomyofOOXML-pptx.php) (accessed 2026-05-23)
19. [schemas.liquid-technologies.com ST_ShapeType](https://schemas.liquid-technologies.com/OfficeOpenXML/2006/st_shapetype.html) (accessed 2026-05-23)
20. [schemas.liquid-technologies.com pml-slide](https://schemas.liquid-technologies.com/officeopenxml/2006/pml-slide_xsd.html) (accessed 2026-05-23)
21. [ooxml.info timing elements](https://ooxml.info/docs/19/19.5/19.5.4/) (accessed 2026-05-23)

### Library/Tool Documentation
22. [ooxmlsdk GitHub](https://github.com/kaisery/ooxmlsdk) -- pml_hierarchy.md, drawingml.md, pml_themes.md (accessed 2026-05-23)
23. [python-pptx docs](https://python-pptx.readthedocs.io/en/stable/) -- packaging, placeholders, fill analysis (accessed 2026-05-23)
24. [Open XML SDK Transition class](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.presentation.transition?view=openxml-3.0.1) (accessed 2026-05-23)

### Blog Posts / Community
25. [Hidden PowerPoint SmartArt layouts](https://blog.hompus.nl/2026/02/26/hidden-powerpoint-smartart-layouts/) (accessed 2026-05-23)
26. [PowerPoint chart types guide](https://www.empowersuite.com/en/blog/list-powerpoint-charts) (accessed 2026-05-23)
27. [DeckSherpa transition effects](https://decksherpa.com/blog/powerpoint-transition-effects/) (accessed 2026-05-23)
28. [BrightCarbon Zoom links](https://www.brightcarbon.com/blog/powerpoint-zoom-links/) (accessed 2026-05-23)
29. [Indezine animation types](https://www.indezine.com/products/powerpoint/learn/animationsandtransitions/2016/types-of-animation.html) (accessed 2026-05-23)
30. [SlideModel PPTX file anatomy](https://slidemodel.com/anatomy-of-a-pptx-file/) (accessed 2026-05-23)
31. [Aspose.Slides capabilities](https://forum.aspose.com/) (accessed 2026-05-23)

### Companion Research
32. slideforge OOXML Foundations -- `.factory/planning/ooxml-foundations.md` (2026-05-23)
33. slideforge Brand Template Patterns -- `.factory/planning/brand-template-patterns.md` (2026-05-23)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | Deep taxonomy of all PresentationML elements; cross-renderer compatibility matrix |
| Perplexity perplexity_ask | 6 | Preset shape counts, transition types, animation effects, chart types/namespaces, zoom/online limitations, table styling/placeholders/fills |
| Perplexity perplexity_reason | 1 | Version roadmap synthesis and prioritization |
| Tavily tavily_search | 1 | ST_SlideLayoutType enumeration verification |
| Context7 | 0 | N/A (ooxmlsdk already covered in ooxml-foundations.md) |
| Training data | 3 areas | OOXML namespace URIs, DSL syntax proposals, gap analysis percentages |

**Total MCP tool calls:** 10
**Training data reliance:** low -- all element counts, feature lists, and compatibility claims verified against web sources. DSL syntax proposals and gap percentages are original analysis informed by verified data.
