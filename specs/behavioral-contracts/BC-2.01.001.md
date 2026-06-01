---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-31T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-018
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.2"
    date: 2026-05-31
    reason: "STORY-076 BC widening: add EC-006 covering srgbClr elements with lumMod/lumOff/tint/shade child transform elements — base val stored with is_derived flag, tracing::warn emitted, extractor emits inline TOML comment. Product-owner decision: Option B (is_derived flag + comment) selected for v1.0; exact HSL resolution deferred to v2. Widen postcondition 1 to clarify srgbClr transform-awareness. Add test vector for transform-bearing srgbClr."
  - version: "1.3"
    date: 2026-06-01
    reason: "SPEC DEFECT correction (STORY-075 adversary H-1): footer visibility flags were incorrectly specified as children of <p:showPr> in presProps.xml. ECMA-376 CT_ShowProperties has no ftr/dt/sldNum children — those are boolean attributes on the <p:hf> (CT_HeaderFooter) element present on slide masters, slide layouts, and slides. Postcondition 1 corrected: footer_flags are read from <p:hf> attributes on slideMaster1.xml (with slideLayoutN.xml as override/fallback). EC-004 and EC-007 rewritten accordingly. AC-001 multi-run text clarification: footer text is the full concatenation of all <a:t> runs in the footer placeholder paragraph (not first-run-only)."
  - version: "1.4"
    date: 2026-06-01
    reason: "v1/v2 scope reconciliation (STORY-075 adversary pass 2 OBS-3 — EC-007 scope correction): EC-007 previously stated that v1 emits a tracing::debug! for layout-level <p:hf> overrides, which is inconsistent with the v1 read-master-only scope. The v1 brand loader does not inspect slideLayoutN.xml for <p:hf> at all, so no debug can be emitted for an override it never reads. EC-007 rewritten to make explicit: (1) v1 reads ONLY slideMaster1.xml for <p:hf> baseline; (2) layout-override detection AND the tracing::debug! for that detection are both v2-deferred. No change to v1 behavioral requirements that are implemented (master <p:hf> flag reading, multi-run text, defaults)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.001: Load Brand from Existing .pptx or .docx Template File

## Description

When a deck declares `brand "path/to/template.pptx"` (or `.docx`), the brand loader
reads the OOXML package, extracts theme colors (all 12 slots), font configuration,
logo reference, and footer text into an in-memory `BrandTemplate` struct. This struct
is then consumed by the PPTX exporter to apply the brand to synthesized slides.

## Preconditions

1. The template path resolves to a readable file.
2. The file has a `.pptx` or `.docx` extension.
3. The file is a valid OOXML ZIP package with `ppt/theme/theme1.xml` (PPTX) or
   `word/theme/theme1.xml` (DOCX) present.

## Postconditions

1. A `BrandTemplate` struct is returned containing all 12 OOXML scheme color slots
   (dk1, lt1, dk2, lt2, acc1-acc6, hlink, folHlink), heading/body font names,
   logo image bytes (if detected in slideMaster1.xml.rels), footer text (if
   detected from slide master/layout `<p:ph type="ftr"/>` placeholders — the FULL
   concatenated text of all `<a:t>` runs in document order; see STORY-075 AC-001),
   and footer visibility flags (`show_footer`, `show_date`, `show_slide_number`)
   read from the `ftr`, `dt`, and `sldNum` boolean attributes of the `<p:hf>`
   (CT_HeaderFooter) element on `slideMaster1.xml`. If a slide layout overrides the
   master's `<p:hf>` attributes, the effective per-layout visibility is the layout's
   value (see EC-007). See EC-006 for srgbClr transform handling; STORY-075 for
   footer detection implementation.
   Each `ColorSlot` carries an `is_derived: bool` flag; slots extracted from
   `srgbClr` elements that have `lumMod`, `lumOff`, `tint`, or `shade` child
   transform elements are stored with `is_derived = true` and the base `val` hex
   (see EC-006). Slots extracted from `srgbClr` without transform children are
   stored with `is_derived = false`.
2. All 31 slide layouts are discoverable from the loaded template (slideLayout1.xml
   through slideLayoutN.xml are mapped by name to the taxonomy in Spike S5).
3. Build exits with code 0.

## Invariants

1. The loaded `BrandTemplate` always exposes all 12 OOXML color slots — any missing
   slot in the source template triggers E-BRD-003 with an inferred value. (DI-015)
2. The loading is read-only; the source template file is never modified.
3. Loading a PPTX brand template does not affect the `Deck` IR; it only affects
   the export stage.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Template file not found at resolved path | E-BRD-001 (broken, exit 4): brand file not found |
| EC-002 | File is not a valid ZIP/OOXML package (corrupt) | E-BRD-002 (broken, exit 4): cannot parse brand template |
| EC-003 | Template has only 8 of 12 color slots in theme1.xml | 4 E-BRD-003 warnings for missing slots; build continues with inferred values |
| EC-004 | `<p:hf>` element is absent from `slideMaster1.xml` (master has no header/footer settings element) | `BrandTemplate.footer_flags` defaults to all-false (`FooterFlags::default()`); no error. A `tracing::debug!` is emitted noting the absence. This is a valid OOXML state — the master may simply not declare visibility overrides. |
| EC-005 | Template has no logo (no image relationship from slideMaster) | BrandTemplate.logo = None; no error |
| EC-006 | `<a:srgbClr val="RRGGBB">` element has one or more child transform elements (`<a:lumMod>`, `<a:lumOff>`, `<a:tint>`, `<a:shade>`) | **Product-owner decision (v1.0 — Option B):** The base `val` hex is stored verbatim as the `ColorSlot.hex` value; `ColorSlot.is_derived` is set to `true`. The loader does NOT compute the mathematically resolved color (HSL transform is deferred to v2). A `tracing::warn!` is emitted at loading time identifying the slot name and the transform type(s) found, for example: `"slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with derived flag"`. When `BrandExtractor` serializes a derived slot to `brand.toml`, it emits the same inline TOML comment as for schemeClr transforms: `# derived via tint/shade; may not match exact color`. Multiple transform children on the same `srgbClr` element all set `is_derived = true`; a single warning is emitted listing all transform types found. `srgbClr` elements without transform children are unaffected (no derived flag, no warning, no comment). No new E-BRD-NNN error code — the `tracing::warn!` is an observability log, not a user-facing diagnostic. |
| EC-007 | Slide layout overrides master `<p:hf>` (e.g., `slideLayout3.xml` has `<p:hf ftr="0"/>` while master has `<p:hf ftr="1"/>`) | **v1.0 scope: master `<p:hf>` only.** The v1 brand loader reads `slideMaster1.xml` for the `<p:hf>` baseline and does NOT inspect any `slideLayoutN.xml` for `<p:hf>` elements. Because layout-level `<p:hf>` is never read in v1, no `tracing::debug!` is emitted for a layout override — detecting and logging layout overrides is explicitly **v2-deferred** (together with the actual per-layout `FooterFlags` merge). A single master-level `FooterFlags` is stored. Note: "multi-master" behavior (EC-004 in v1.2 addressing multiple slide masters) is now renumbered here; see v1.3 changelog. Only `slideMaster1.xml` is processed; a lint warning is emitted if the ZIP contains `slideMaster2.xml` or higher. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid corporate .pptx with all 12 colors | BrandTemplate with exact colors; exit 0 | happy-path |
| Valid .docx brand template | BrandTemplate loaded from word/theme/theme1.xml; exit 0 | happy-path |
| Non-existent path "missing.pptx" | E-BRD-001; exit 4 | error |
| Corrupt file (not a ZIP) | E-BRD-002; exit 4 | error |
| Template with 8 colors in theme | BrandTemplate with 12 colors (4 inferred); 4 E-BRD-003 warnings | edge-case |
| .pptx with `<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>` for slot dk2 | BrandTemplate.dk2.hex = "#003087"; dk2.is_derived = true; tracing::warn! emitted for slot dk2 with lumMod; exit 0 | edge-case (EC-006) |
| .pptx srgbClr with no transform children (e.g., `<a:srgbClr val="FF0000"/>`) | ColorSlot.hex = "#FF0000"; is_derived = false; no warning; no TOML comment | regression (EC-006 boundary) |
| .pptx srgbClr with both lumMod and tint children | is_derived = true; single tracing::warn! listing both transform types found; exit 0 | edge-case (EC-006 multi-transform) |
| .pptx with `<p:hf ftr="1" dt="1" sldNum="0"/>` in slideMaster1.xml | footer_flags = FooterFlags { show_footer: true, show_date: true, show_slide_number: false }; exit 0 | happy-path (EC-004 corrected) |
| .pptx with NO `<p:hf>` element in slideMaster1.xml | footer_flags = FooterFlags::default() (all false); tracing::debug! emitted; exit 0 | edge-case (EC-004) |
| .pptx footer placeholder with multiple `<a:r>` runs: `<a:r><a:t>Q4 </a:t></a:r><a:r><a:t>Report</a:t></a:r>` | footer_text = Some("Q4 Report") (all runs concatenated in document order) | edge-case (AC-001 multi-run) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Loaded template always has exactly 12 color entries | unit test (count fields) |
| VP-TBD | Source template file is unmodified after loading | integration test (file hash before/after) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 |
| Capability Anchor Justification | CAP-018 ("Brand Template Loading and Synthesis") per capabilities.md §CAP-018 — loading from an existing .pptx/.docx is the first loading path explicitly described in CAP-018 |
| L2 Domain Invariants | DI-015 (brand palette must cover all 12 OOXML slots), DI-016 (single master architecture) |
| Architecture Module | slideforge-brand crate — BrandLoader (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-2.01.002 — related to (synthesis from brand.toml is the complementary loading path)
- BC-2.01.003 — related to (extract-brand produces brand.toml from the same OOXML this BC loads)
- BC-4.01.001 — depends on (PPTX exporter consumes the BrandTemplate produced here)

## Architecture Anchors

- `architecture/brand-architecture.md` — brand loading from existing .pptx/.docx
- `architecture/system-overview.md` — brand resolution as a pre-export stage

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
