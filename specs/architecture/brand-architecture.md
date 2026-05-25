---
document_type: architecture-section
section: brand-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Brand Architecture

## Subsystem Role (SS-04)

`slideforge-brand` implements the `BrandProvider` plugin surface. It is effectful
(reads `brand.toml` and optional `.pptx` template from disk) but its synthesis
logic — `Brand::synthesize()` — is a pure function that is isolatable and testable
without I/O.

## 31-Layout Taxonomy (ADR-010, S5)

Every synthesized brand template contains 11 standard OOXML layouts plus 20 custom
slideforge layouts. The full taxonomy is locked in S5 spike. Custom layouts use
the `SF ` name prefix for grouping in LibreOffice's layout picker.

Two layouts require `clrMapOvr` (dark backgrounds):
- `CL-01` — SF Section Divider (slide type: `title`)
- `CL-11` — SF End Slide (slide type: `end`)

## Dark Layout Invariant (Feasibility Note 4)

S6 BUG-001 finding: LibreOffice's `clrMapOvr` support is incomplete in versions
before 24.x. Dark-layout slides may render with incorrect text color.

**Binding invariant:** For layouts CL-01 and CL-11, `slideforge-brand` MUST write
BOTH of the following on every placeholder text run:
1. `<p:clrMapOvr>` with `overrideClrMapping bg1="dk2" tx1="lt1"` (for compliant renderers)
2. Explicit `<a:solidFill><a:srgbClr val="FFFFFF"/>` on the text run color (for LibreOffice)

This dual-write pattern is not optional. Any future change to dark-layout color handling
must update both paths. A unit test in `slideforge-brand` MUST verify that both
`clrMapOvr` AND explicit white run colors are present in the synthesized XML for
CL-01 and CL-11.

**Test invariant:** `test_dark_layout_dual_write` in `slideforge-brand/src/tests/` must:
1. Synthesize a brand template with default 1898 & Co. colors
2. Extract the CL-01 layout XML
3. Assert `<p:clrMapOvr>` is present with `bg1="dk2"`
4. Assert `<a:srgbClr val="FFFFFF"/>` is present on the title placeholder's default run properties

## Bidirectional Bridge (ADR-010)

`BrandProvider` supports two directions:
- **Synthesis:** `brand.toml` → `BrandTemplate` → PPTX template bytes (+ DOCX template)
- **Extraction:** existing `.pptx` bytes → `BrandConfig` TOML representation

Extraction recovers: 12 color slots (ECMA-376 order), heading/body font names,
logo image reference, footer text, slide number presence, and dark-layout flags.

Round-trip fidelity: colors and fonts round-trip exactly. Logo extraction requires
parsing the `slideMaster1.xml.rels` media relationships (production implementation;
not in S5 prototype).

## Brand Validation

`slideforge-validate` crate's `BrandValidator` runs contrast checks on the
synthesized brand palette (S3: `BrandValidator::check_theme_pairs()`). This is
a pure function operating on the `Brand` struct — Kani-amenable.

The 12 OOXML theme color slots must all be populated (DI-015). A brand with missing
color slots is a compile error (E-BRN-001).

## Color Slot Mapping

| ECMA-376 Slot | brand.toml Key | Semantic Meaning |
|--------------|---------------|-----------------|
| dk1 | text_primary | Primary text (dark) |
| lt1 | background | Page/slide background |
| dk2 | text_secondary | Secondary text, brand primary |
| lt2 | surface_alt | Alternate surface background |
| accent1-6 | brand_primary, brand_secondary, accent_3-6 | Brand palette |
| hlink | hyperlink | Hyperlink color |
| folHlink | hyperlink_visited | Visited hyperlink |
