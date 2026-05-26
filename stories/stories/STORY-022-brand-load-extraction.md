---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-022
title: "Brand Loading: .pptx/.docx Template Extraction"
epic: EPIC-06
wave: 3
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-brand
subsystems: [SS-04]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.001, BC-2.01.006]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-001
blocks:
  - STORY-023
  - STORY-024
  - STORY-025
  - STORY-037
  - STORY-038
estimated_days: 2
---

# STORY-022: Brand Loading: .pptx/.docx Template Extraction

## Summary

Implement the `BrandProvider` trait for loading brand data from an existing `.pptx` or
`.docx` template file in `slideforge-brand`. The `BrandLoader` reads the OOXML ZIP
package using `quick-xml = "=0.36.2"` and `zip = "=2.6.1"`, extracts all 12 OOXML theme
color slots (dk1, lt1, dk2, lt2, acc1-acc6, hlink, folHlink) from `ppt/theme/theme1.xml`
(PPTX) or `word/theme/theme1.xml` (DOCX), extracts heading/body font names, detects logo
image from `ppt/slideMasters/slideMaster1.xml.rels` (PPTX), and constructs a
`BrandTemplate` struct. Missing color slots (fewer than 12 in the source) trigger
`E-BRD-003` warnings but do NOT fail the build — the slot count invariant (DI-015) is
maintained via inference. Font unavailability is cosmetic only (BC-2.01.006): the
declared font name is written to OOXML regardless of host availability, fallback is
used only for build-time metric calculations.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| `crates/slideforge-brand/src/loader.rs` | ~3,500 |
| `crates/slideforge-brand/src/color.rs` | ~2,000 |
| `crates/slideforge-brand/src/font.rs` | ~1,500 |
| Test fixtures + test code | ~3,000 |
| BC files consulted (BC-2.01.001, BC-2.01.006) | ~2,000 |
| **Total** | **~16,000** |

Agent context budget: 200k tokens. This story is ~8% of budget — within limit.

## Acceptance Criteria

### Template Loading — BC-2.01.001

- [ ] **AC-001:** `BrandLoader::load(path: &str, ctx: &BrandLoadContext) -> Result<BrandTemplate, BrandError>` opens the file as a ZIP archive (`.pptx` or `.docx`) and reads `ppt/theme/theme1.xml` (PPTX) or `word/theme/theme1.xml` (DOCX). On success, returns a `BrandTemplate` with all required fields populated.
  (traces to BC-2.01.001 postcondition 1 — `BrandTemplate` struct returned with 12 color slots)

- [ ] **AC-002:** All 12 OOXML theme color slots are extracted in ECMA-376 sequential order: `dk1`, `lt1`, `dk2`, `lt2`, `acc1`, `acc2`, `acc3`, `acc4`, `acc5`, `acc6`, `hlink`, `folHlink`. `srgbClr` elements → `"#RRGGBB"` hex. `sysClr` elements → `lastClr` attribute as hex. `schemeClr` with `lumMod`/`tint`/`shade` → extracted value with inline warning comment.
  (traces to BC-2.01.001 postcondition 1; BC-2.01.003 invariant 1 — ECMA-376 sequential order)

- [ ] **AC-003:** If any of the 12 expected color slots is absent in the source `theme1.xml`, `BrandError::MissingColorSlot { slot_name: Arc<str> }` produces a cosmetic `E-BRD-003` warning (via `tracing::warn!`). The build continues with the slot filled by the inference algorithm (same as BC-2.01.004 — see STORY-023). All 12 slots are always present in the returned `BrandTemplate`.
  (traces to BC-2.01.001 invariant 1 — always 12 slots; BC-2.01.001 edge case EC-003)

- [ ] **AC-004:** Heading and body font names are extracted from `<a:majorFont>` and `<a:minorFont>` elements in `theme1.xml`. If either element is absent, `"Calibri"` is used as fallback and a `tracing::warn!` is emitted.
  (traces to BC-2.01.001 postcondition 1 — `BrandTemplate` includes font configuration)

- [ ] **AC-005:** Logo image bytes are extracted by reading `ppt/slideMasters/_rels/slideMaster1.xml.rels` (PPTX only), finding a relationship with `Type` ending in `/image`, and reading the target file from the ZIP. The logo is stored as `Option<LogoAsset>` — `None` if no image relationship is found (not an error). `LogoAsset` has fields `bytes: Vec<u8>`, `media_type: Arc<str>` (e.g., `"image/png"`), `original_path: Arc<str>`.
  (traces to BC-2.01.001 postcondition 1 — logo image bytes; edge case EC-005 — no logo = None, no error)

- [ ] **AC-006:** `BrandTemplate` contains all discoverable slide layout names from `ppt/slideLayouts/slideLayout*.xml` (PPTX). The layout names are used in STORY-023 to map layouts to the 31-layout taxonomy. Store as `Vec<Arc<str>>` (layout XML names, not semantic names).
  (traces to BC-2.01.001 postcondition 2 — all 31 layouts discoverable)

- [ ] **AC-007:** File not found at resolved path produces `BrandError::FileNotFound { path: Arc<str>, span: SourceSpan }` mapping to `E-BRD-001`. Build exits with code 4.
  (traces to BC-2.01.001 edge case EC-001 — E-BRD-001, exit 4)

- [ ] **AC-008:** File is not a valid ZIP/OOXML package (corrupt file, not a ZIP) produces `BrandError::ParseError { path: Arc<str>, reason: Arc<str> }` mapping to `E-BRD-002`. Build exits with code 4.
  (traces to BC-2.01.001 edge case EC-002 — E-BRD-002, exit 4)

- [ ] **AC-009:** Template with multiple slide masters (more than one `slideMaster*.xml`): only `slideMaster1.xml` is extracted. A lint warning is emitted: `tracing::warn!("Template '{}' has multiple slide masters; only slideMaster1.xml is used.", path)`.
  (traces to BC-2.01.001 edge case EC-004)

### Font Fallback — BC-2.01.006

- [ ] **AC-010:** When `BrandLoadContext.check_font_availability = true` and a font declared in the brand template is not installed on the build host, `BrandError::FontUnavailable { font_name: Arc<str>, fallback: Arc<str> }` emits exactly one `tracing::warn!` per unavailable font: `"Font '{font_name}' not available. Using '{fallback}' for build-time metrics."`. The OOXML XML output always writes the declared font name (not the fallback).
  (traces to BC-2.01.006 postconditions 1, 2, 3 — warning, OOXML uses declared name, fallback for metrics)

- [ ] **AC-011:** Font availability check uses the platform-specific font directory. Build exits with code 0 regardless of font availability. Font unavailability is never fatal.
  (traces to BC-2.01.006 invariant 1 — never fails build; invariant 3 — deterministic fallback chain: Aptos → Calibri → Arial → system sans-serif)

- [ ] **AC-012:** The OOXML `theme1.xml` output produced by `BrandLoader` always writes the user's declared font name in `<a:majorFont>` / `<a:minorFont>` regardless of whether the font is installed. Only the text metrics subsystem (layout engine) uses the fallback font.
  (traces to BC-2.01.006 invariant 2 — declared font name preserved in OOXML output)

- [ ] **AC-013:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023), clippy clean (NFR-022), `=` version pinning (NFR-025) applied.

## Previous Story Intelligence

N/A — first story in EPIC-06. Key context:
- The `BrandTemplate` struct skeleton was defined in STORY-001 (`Brand`, `BrandPalette`, `BrandFonts`). This story populates the full `BrandTemplate` struct with OOXML-specific fields.
- The `BrandProvider` trait is defined in `slideforge-plugin-api` (STORY-002). `BrandLoader` implements it.
- `quick-xml` usage pattern: use `quick_xml::Reader::from_str()` with event-based SAX-style parsing. Do not use `serde`-based deserialization for OOXML — element ordering is schema-significant and serde does not preserve order.

## Architecture Compliance Rules

1. **SS-04 (Brand) — Effectful (file I/O):** `slideforge-brand` reads ZIP files from the filesystem. It is Effectful.
2. **`quick-xml` SAX parsing required:** OOXML element ordering is schema-significant (R4 finding). Do not use `quick-xml`'s serde feature. Use event-based `Reader` to parse `theme1.xml` in element order.
3. **Read-only invariant:** `BrandLoader` MUST NOT modify the source template file. The file is opened in read mode only. Validated by integration test (file hash before/after).
4. **`BrandProvider` trait from STORY-002:** Do not redefine the trait. If the STORY-002 definition differs from what is used here, STORY-002 wins.
5. **Forbidden dependencies:** `slideforge-brand` MUST NOT depend on `slideforge-eval`, `slideforge-syntax`, `slideforge-validate`, `slideforge-layout`, `slideforge-pptx`, `slideforge-pdf`, `slideforge-html`, or `slideforge-cli`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `quick-xml` | `=0.36.2` | SAX-style OOXML XML parsing (no serde feature) |
| `zip` | `=2.6.1` | Reading PPTX/DOCX as ZIP archives |
| `thiserror` | `=2.0.18` | `BrandError` enum |
| `tracing` | `=0.1` | Cosmetic warnings (E-BRD-003, E-BRD-004) |
| `slideforge-types` | workspace | `SourceSpan`, `Arc<str>` |
| `slideforge-plugin-api` | workspace | `BrandProvider` trait |
| `indexmap` | `=2.2` | `IndexMap` for color slot preservation in order |

Dev dependencies:
- `insta = "=1.39.0"` — snapshot tests on extracted `BrandTemplate`
- `zip = "=2.6.1"` in dev deps for constructing test ZIP fixtures (same version)

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/
├── Cargo.toml
├── src/
│   ├── lib.rs                     # crate root; pub use all; #![forbid(unsafe_code)]
│   ├── error.rs                   # BrandError enum (FileNotFound, ParseError, MissingColorSlot, FontUnavailable)
│   ├── template.rs                # BrandTemplate struct (full definition)
│   ├── loader.rs                  # BrandLoader struct + BrandProvider impl
│   ├── color.rs                   # parse_theme_colors(xml: &str) -> Result<[ColorSlot; 12], BrandError>
│   ├── font.rs                    # parse_theme_fonts(xml: &str) -> Result<BrandFonts, BrandError>
│   │                              # font_available(name: &str) -> Option<PathBuf>
│   ├── logo.rs                    # extract_logo(zip: &mut ZipArchive) -> Option<LogoAsset>
│   └── context.rs                 # BrandLoadContext { check_font_availability: bool, span: SourceSpan }
```

## Tasks

1. **Create `Cargo.toml`** — edition 2024, resolver 3, all pinned deps. (10 min)
2. **Write `src/error.rs`** — `BrandError` enum: `FileNotFound`, `ParseError`, `MissingColorSlot`, `FontUnavailable`. `thiserror::Error` derive on each. Error code constants `E_BRD_001` through `E_BRD_004`. (20 min)
3. **Write `src/template.rs`** — full `BrandTemplate` struct:
   ```rust
   pub struct BrandTemplate {
       pub colors: [ColorSlot; 12],       // dk1, lt1, dk2, lt2, acc1-6, hlink, folHlink
       pub fonts: BrandFonts,             // heading_font, body_font
       pub logo: Option<LogoAsset>,       // bytes + media type
       pub footer_text: Option<Arc<str>>, // from deck-level footer
       pub layout_names: Vec<Arc<str>>,   // from slideLayout*.xml names
   }
   pub struct ColorSlot { pub name: Arc<str>, pub hex: Arc<str> } // "#RRGGBB"
   pub struct BrandFonts { pub heading: Arc<str>, pub body: Arc<str> }
   pub struct LogoAsset { pub bytes: Vec<u8>, pub media_type: Arc<str>, pub original_path: Arc<str> }
   ``` (20 min)
4. **Write `src/color.rs`** — `parse_theme_colors(xml_bytes: &[u8]) -> Result<[ColorSlot; 12], Vec<BrandError>>`. SAX-parse `theme1.xml` looking for `<a:dk1>`, `<a:lt1>`, etc. Handle `<a:srgbClr val="...">`, `<a:sysClr lastClr="...">`. Missing slots → `BrandError::MissingColorSlot`. (40 min)
5. **Write `src/font.rs`** — `parse_theme_fonts(xml_bytes: &[u8]) -> Result<BrandFonts, BrandError>`. Extract `<a:majorFont>` (`typeface` attr) and `<a:minorFont>` (`typeface` attr). Fallback to `"Calibri"` if absent. Implement `font_available(name: &str) -> bool` with platform-specific font directory search. (25 min)
6. **Write `src/logo.rs`** — `extract_logo(zip: &mut ZipArchive<File>) -> Option<LogoAsset>`. Read `ppt/slideMasters/_rels/slideMaster1.xml.rels`, find image relationship, read target file bytes. Return `None` if not found. (25 min)
7. **Write `src/loader.rs`** — `BrandLoader`. Implement `BrandProvider::load_brand(path: &str, ctx: &BrandLoadContext) -> Result<BrandTemplate, BrandError>`. Open ZIP → detect PPTX vs DOCX by checking internal paths → read theme XML → call `parse_theme_colors`, `parse_theme_fonts`, `extract_logo`. Handle font availability check. (30 min)
8. **Write `src/context.rs`** — `BrandLoadContext { check_font_availability: bool, span: SourceSpan, root_dir: PathBuf }`. (10 min)
9. **Write unit + integration tests** — see Test Strategy. (30 min)
10. **Run `cargo clippy -p slideforge-brand -- -D warnings`** — fix. (10 min)
11. **Run `cargo test -p slideforge-brand`** — all tests pass. (10 min)

## Test Strategy

### Unit tests in `color.rs`

- `test_parse_12_srgb_colors` — XML with 12 `<a:srgbClr>` elements → `[ColorSlot; 12]`, no errors.
- `test_parse_sysclr_uses_lastclr` — XML with `<a:sysClr lastClr="FFFFFF">` for `dk1` → `ColorSlot { hex: "#FFFFFF" }`.
- `test_missing_slot_emits_warning` — XML with only 8 color elements → `Vec<BrandError>` length 4.
- `test_color_order_preserved` — Colors must be extracted in dk1, lt1, dk2, lt2, acc1..acc6, hlink, folHlink order (ECMA-376 sequential).

### Unit tests in `font.rs`

- `test_parse_major_minor_fonts` — XML with `<a:majorFont typeface="Calibri">` and `<a:minorFont typeface="Arial">` → `BrandFonts { heading: "Calibri", body: "Arial" }`.
- `test_missing_fonts_fallback` — XML with no font elements → fallback `"Calibri"` used; `tracing::warn!` emitted.
- `test_font_available_system` — test with a known-installed font (`"Arial"` on macOS/Windows) → `true`.

### Integration tests in `loader.rs`

- `test_load_valid_pptx` — construct a minimal valid PPTX ZIP fixture (in-memory, using the `zip` crate) with a `theme1.xml` containing all 12 colors. Verify `BrandTemplate.colors` has 12 entries. Verify source file hash unchanged after load.
- `test_load_valid_docx` — same for DOCX (different internal path for theme XML).
- `test_load_missing_file` — non-existent path → `BrandError::FileNotFound`.
- `test_load_corrupt_file` — write random bytes to a `.pptx` file → `BrandError::ParseError`.
- `test_load_partial_colors` — `theme1.xml` with only 8 colors → 4 `E-BRD-003` warnings; `BrandTemplate` still has 12 color slots.

## Dependencies

**Depends on:**
- STORY-001 (IR Core Types) — `SourceSpan`, `Arc<str>`, `Brand` skeleton struct.

Dependency justification: STORY-022 depends on STORY-001 because `BrandTemplate` extends the `Brand` skeleton from `slideforge-types`, and `SourceSpan` is used in error types.

**Blocks:**
- STORY-023 (Brand Synthesis) — the synthesizer uses `BrandTemplate` from this story as its output type. The `ColorSlot` array and `BrandFonts` struct must exist before synthesis.
- STORY-024 (Brand Extraction CLI) — uses `BrandLoader` from this story for the extraction pipeline.
- STORY-025 (Brand Overlay) — brand overlay applies to a `BrandTemplate` loaded by this story.
- STORY-037 (PPTX Core Serialization) — PPTX exporter consumes `BrandTemplate`.
- STORY-038 (PPTX Layout Compliance) — depends on `BrandTemplate.layout_names`.

## Implementation Notes

### OOXML Theme Color Slot Order (ECMA-376)

The 12 color slots MUST be extracted in this exact ECMA-376 sequential order. Do not
use random-access lookup by element name — parse in document order and verify position:

```
Position 0: dk1  (dark 1 — primary text)
Position 1: lt1  (light 1 — background)
Position 2: dk2  (dark 2 — secondary text)
Position 3: lt2  (light 2 — secondary background)
Position 4: acc1 (accent 1)
Position 5: acc2 (accent 2)
Position 6: acc3 (accent 3)
Position 7: acc4 (accent 4)
Position 8: acc5 (accent 5)
Position 9: acc6 (accent 6)
Position 10: hlink    (hyperlink)
Position 11: folHlink (followed hyperlink)
```

### ZIP Internal Path Detection

- PPTX: `theme1.xml` at `ppt/theme/theme1.xml`
- DOCX: `theme1.xml` at `word/theme/theme1.xml`

To detect format: check if `ppt/theme/theme1.xml` exists in ZIP entries. If yes → PPTX. If no, check `word/theme/theme1.xml` → DOCX. If neither → `BrandError::ParseError`.

### Font Detection Platform Paths

```rust
fn font_available(name: &str) -> bool {
    let dirs = if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/System/Library/Fonts"),
            PathBuf::from("/Library/Fonts"),
            dirs::home_dir().map(|h| h.join("Library/Fonts")).unwrap_or_default(),
        ]
    } else if cfg!(target_os = "windows") {
        vec![PathBuf::from("C:/Windows/Fonts")]
    } else {
        // Linux
        vec![
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
            dirs::home_dir().map(|h| h.join(".fonts")).unwrap_or_default(),
        ]
    };
    // Simplified: check for any file matching font name (case-insensitive prefix)
    // Full implementation uses font_kit or system font enumeration
    // For v1.0, a best-effort name-based scan is acceptable
    dirs.iter().any(|d| d.join(name).exists() || /* fuzzy check */ false)
}
```

A full font availability check via `font-kit` is desirable but the dependency adds
complexity. For v1.0, the name-based scan is acceptable. Document this limitation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Template file not found | `BrandError::FileNotFound` → E-BRD-001; exit 4 |
| EC-002 | File is not a valid ZIP (corrupt) | `BrandError::ParseError` → E-BRD-002; exit 4 |
| EC-003 | Template has only 8 of 12 color slots | 4 `E-BRD-003` warnings; 4 slots inferred; build continues |
| EC-004 | Template has multiple slide masters | Only `slideMaster1.xml` extracted; lint warning emitted |
| EC-005 | Template has no logo | `BrandTemplate.logo = None`; no error |
| EC-006 | Heading font not installed on build host | `E-BRD-004` warning; fallback used for metrics; OOXML still writes declared name |
| EC-007 | Both heading and body fonts unavailable | Two `E-BRD-004` warnings; build continues |
| EC-008 | DOCX brand template (word/theme/theme1.xml) | Loads correctly from DOCX internal path |
