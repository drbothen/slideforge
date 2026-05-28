---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-023
title: "Brand Synthesis: brand.toml → 31 Layouts + 12 OOXML Slots"
epic: EPIC-06
wave: 3
points: 13
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-brand
subsystems: [SS-04]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.002, BC-2.01.004, BC-2.01.005]
verification_properties: [VP-012]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-022
blocks:
  - STORY-024
  - STORY-037
  - STORY-038
  - STORY-039
  - STORY-040
estimated_days: 5
---

# STORY-023: Brand Synthesis: brand.toml → 31 Layouts + 12 OOXML Slots

## Summary

Implement the `BrandSynthesizer` in `slideforge-brand` that reads a `brand.toml` file
and produces a complete in-memory `BrandTemplate` containing: all 12 OOXML color slots
(with deterministic inference for missing slots), typography settings, logo reference,
footer text, and exactly 31 slide layouts (11 standard + 20 custom `SF *` layouts per
Spike S5 taxonomy). The synthesis is deterministic: same `brand.toml` → same
`BrandTemplate`. The synthesized brand is structurally equivalent to what PowerPoint
produces via "Edit Theme Colors." This is the largest story in EPIC-06 (13 points)
because it defines the layout taxonomy for all 31 slide types used throughout the system.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~6,000 |
| `crates/slideforge-brand/src/synthesizer.rs` | ~5,000 |
| `crates/slideforge-brand/src/layouts.rs` | ~6,000 |
| `crates/slideforge-brand/src/toml_schema.rs` | ~2,500 |
| `crates/slideforge-brand/src/inference.rs` | ~2,000 |
| Test code + snapshot fixtures | ~4,000 |
| BC files consulted (BC-2.01.002, BC-2.01.004, BC-2.01.005) | ~3,000 |
| **Total** | **~28,500** |

Agent context budget: 200k tokens. This story is ~14.25% of budget — within limit (< 20-30%).

## Acceptance Criteria

### brand.toml Schema and Parsing — BC-2.01.002

- [ ] **AC-001:** `brand.toml` is parsed via `toml = "=0.8"` into a strongly-typed `BrandConfig` struct. The schema has these top-level sections:
  ```toml
  [colors]
  dk1 = "#1F2937"        # required for non-inferred synthesis; all slots optional
  lt1 = "#FFFFFF"
  dk2 = "#374151"
  lt2 = "#F9FAFB"
  acc1 = "#3B82F6"
  acc2 = "#10B981"
  acc3 = "#F59E0B"
  acc4 = "#EF4444"
  acc5 = "#8B5CF6"
  acc6 = "#EC4899"
  hlink = "#2563EB"
  fol_hlink = "#1D4ED8"

  [fonts]
  heading = "Aptos Display"   # optional; fallback: "Calibri"
  body = "Aptos"              # optional; fallback: "Calibri"

  [logo]
  path = "brand.assets/logo.png"   # required for synthesized brand

  [footer]
  text = "Confidential"     # optional; default: ""
  show_slide_number = true  # optional; default: true
  show_date = false         # optional; default: false
  ```
  (traces to BC-2.01.002 postcondition 1 — `Brand` struct with 12 OOXML slots, typography, logo, footer)

- [ ] **AC-002:** All 12 OOXML color slots are always present in the returned `BrandTemplate`. If a slot is absent from `brand.toml`, it is inferred using the deterministic algorithm in AC-003. Zero slots in `brand.toml` → all 12 inferred (12 E-BRD-003 warnings). All 12 declared → zero warnings.
  (traces to BC-2.01.002 invariant 1 — 12 slots always populated; BC-2.01.004 postcondition 1)

- [ ] **AC-003:** Missing color slot inference algorithm (deterministic, per BC-2.01.004 postcondition 3):
  - `dk1`: darkest declared color (by luminance); fallback `"#1F2937"`.
  - `lt1`: always `"#FFFFFF"` (white).
  - `dk2`: `acc1` if declared; else `dk1` lightened 20% (hex arithmetic).
  - `lt2`: `"#F9FAFB"` (near-white; 96% lightness).
  - Missing `acc2`–`acc6`: generated from `acc1` using 30°/60°/90°/120°/150° hue rotation on HSL color wheel.
  - `hlink`: `acc1` darkened 15% if not declared.
  - `fol_hlink`: `hlink` darkened 10% if not declared.
  For each inferred slot, exactly one `tracing::warn!` with `E-BRD-003: Color slot '<name>' not declared in brand.toml. Using inferred value '#<hex>'.` is emitted.
  (traces to BC-2.01.004 postcondition 3 — deterministic derivation rules; postcondition 2 — one warning per inferred slot)

- [ ] **AC-004:** `BrandConfig.logo.path` is required when `brand.toml` is the brand source (not loaded from .pptx). If absent, `BrandError::LogoRequired { span: SourceSpan }` mapping to `E-BRD-001: logo path required in synthesized brand` is returned.
  (traces to BC-2.01.002 edge case EC-005 — missing logo is fatal for synthesized brand)

- [ ] **AC-005:** A `brand.toml` that omits the `[colors]` section entirely — including a literally-empty file (zero bytes) and a file with non-color sections only — produces exactly 12 E-BRD-003 warnings (one per slot) and synthesizes the entire palette from the default baseline. Build continues.
  (traces to BC-2.01.004 edge case EC-002 — empty brand.toml)

- [ ] **AC-006:** All inferred colors are valid 6-digit uppercase hex RGB strings (e.g., `"#3B82F6"`). No alpha channel. No CSS named colors. No lowercase hex.
  (traces to BC-2.01.004 invariant 3 — all inferred colors are valid 6-digit uppercase hex RGB)

- [ ] **AC-007:** Synthesis is deterministic: given the same `brand.toml` file content (same bytes), the returned `BrandTemplate` is always structurally identical. Two calls with identical inputs produce `BrandTemplate` values that compare equal (`==`).
  (traces to BC-2.01.002 invariant 4 — synthesis is deterministic)

### 31 Layout Generation — BC-2.01.005

- [ ] **AC-008:** The synthesized `BrandTemplate` contains exactly 31 `SlideLayoutDef` entries — 11 standard + 20 custom. The full list in index order:

  **Standard layouts (SL-01 through SL-11):**
  1. `title` — Title Slide (`<p:sldLayout type="title">`)
  2. `obj` — Title and Content (`<p:sldLayout type="obj">`)
  3. `secHead` — Section Header (`<p:sldLayout type="secHead">`)
  4. `twoObj` — Two Content (`<p:sldLayout type="twoObj">`)
  5. `twoColTx` — Comparison (`<p:sldLayout type="twoColTx">`)
  6. `titleOnly` — Title Only (`<p:sldLayout type="titleOnly">`)
  7. `blank` — Blank (`<p:sldLayout type="blank">`)
  8. `objTx` — Content with Caption (`<p:sldLayout type="objTx">`)
  9. `picTx` — Picture with Caption (`<p:sldLayout type="picTx">`)
  10. `vertTitleAndTx` — Vertical Title and Text (`<p:sldLayout type="vertTitleAndTx">`)
  11. `vertTx` — Vertical Text (`<p:sldLayout type="vertTx">`)

  **Custom layouts (CL-01 through CL-20), all with name prefix "SF ":**
  12. `CL-01` — "SF Section Divider" (dark layout — `clrMapOvr` with `bg1="dk2" tx1="lt1"`)
  13. `CL-02` — "SF Stat Grid"
  14. `CL-03` — "SF Quote"
  15. `CL-04` — "SF Timeline"
  16. `CL-05` — "SF Agenda"
  17. `CL-06` — "SF TOC"
  18. `CL-07` — "SF Bio"
  19. `CL-08` — "SF Team Grid"
  20. `CL-09` — "SF Comparison Table"
  21. `CL-10` — "SF Full-Bleed Image"
  22. `CL-11` — "SF End Slide" (dark layout — `clrMapOvr` with `bg1="dk2" tx1="lt1"`)
  23. `CL-12` — "SF Data"
  24. `CL-13` — "SF Diagram"
  25. `CL-14` — "SF Chart"
  26. `CL-15` — "SF Map"
  27. `CL-16` — "SF Risk Register"
  28. `CL-17` — "SF Executive Summary"
  29. `CL-18` — "SF Two Column"
  30. `CL-19` — "SF Methodology"
  31. `CL-20` — "SF Appendix"

  (traces to BC-2.01.005 postconditions 1, 2, 3 — exactly 31 layouts; 11 standard types; 20 custom "SF " names)

- [ ] **AC-009:** Two dark layouts (CL-01: "SF Section Divider" and CL-11: "SF End Slide") carry `clrMapOvr` with `bg1="dk2" tx1="lt1"`. These layouts ALSO include explicit white text colors on all placeholder runs (R4 mitigation): each run element has `<a:rPr>` with explicit `<a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill>`. This ensures correct rendering in LibreOffice 7.x which has partial `clrMapOvr` support.
  (traces to BC-2.01.005 postcondition 4 — CL-01 and CL-11 have `clrMapOvr` + explicit white text; BC-2.01.005 edge case EC-003)

- [ ] **AC-010:** Every layout EXCEPT the Blank layout (SL-07) has at least one title or `ctrTitle` placeholder. The Blank layout has zero placeholders (that is correct for a blank layout). All `<p:cNvPr name="...">` attributes use semantic labels, NOT "Shape N" (per Spike S5 §2.3, DI-001 accessibility).
  (traces to BC-2.01.005 invariant 2 — every layout except Blank has title placeholder; invariant 4 — semantic placeholder names)

- [ ] **AC-011:** Slide master IDs start at `2^31 = 2147483648`. Layout IDs start at `2^31 + 1 = 2147483649` and increment by 1. Slide IDs in generated decks start at 256 (enforced by PPTX exporter in STORY-037, but the `BrandTemplate` must carry this constraint as metadata).
  (traces to BC-2.01.005 invariant 3 — master ID = 2^31; BC-2.01.002 postcondition 4 — slide IDs start at 256)

- [ ] **AC-012:** `notesMaster1.xml` and `handoutMaster1.xml` stub entries are present in the synthesized `BrandTemplate` as `Option<Vec<u8>>` (stub XML bytes). Even if empty content, these stubs must be present for OOXML compliance.
  (traces to BC-2.01.002 postcondition 5 — `notesMaster1.xml` and `handoutMaster1.xml` present)

- [ ] **AC-013:** `[Content_Types].xml` registration for all 31 layouts is generated as a string and stored in `BrandTemplate` for use by the PPTX exporter (STORY-037). Each layout gets a `PartName="/ppt/slideLayouts/slideLayout{N}.xml"` entry.
  (traces to BC-2.01.005 postcondition 6 — `[Content_Types].xml` registers all 31 layouts)

- [ ] **AC-014:** `BrandTemplate` passes `VP-012` (proptest — 12-slot palette round-trip): synthesize from `brand.toml`, serialize color slots, re-parse → same hex values. This proptest is part of the Wave 6 formal verification (STORY-069), but the `BrandTemplate` struct must support it from this story.

- [ ] **AC-015:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023), clippy clean (NFR-022), `=` version pinning (NFR-025) maintained.

## Previous Story Intelligence

Continues from STORY-022. The `BrandTemplate` struct and all supporting types (`ColorSlot`, `BrandFonts`, `LogoAsset`, `BrandError`) are established. This story adds:
- `BrandConfig` (the `brand.toml` schema) as a new input type.
- `BrandSynthesizer` — the primary new component.
- `SlideLayoutDef` — the layout definition type.
- Color inference algorithm.
- The 31-layout generation algorithm.

Key note from STORY-022: `quick-xml` SAX parsing is the preferred approach for reading OOXML XML. For GENERATING layout XML (in this story), also use `quick-xml::Writer` with `quick_xml::events::*` for element-order-safe XML generation.

## Architecture Compliance Rules

1. **SS-04 Effectful — reads brand.toml from filesystem:** `BrandSynthesizer::load_from_toml(path)` reads a file. Effectful.
2. **Deterministic synthesis:** The synthesis algorithm MUST be a pure function `synthesize(config: &BrandConfig) -> BrandTemplate` with no I/O. The file reading and the synthesis are separate concerns. Tests verify determinism directly on `synthesize()`.
3. **OOXML element ordering in generated layout XML:** When generating layout XML, elements must be in ECMA-376 schema order. The `quick-xml::Writer` does not enforce this — the implementer must emit elements in the correct sequence.
4. **clrMapOvr required on dark layouts:** `CL-01` and `CL-11` MUST have `<p:clrMapOvr><a:overrideClrMapping bg1="dk2" tx1="lt1" .../>` in their layout XML. This is a hard requirement from R4 findings (Spike S5 §6.1).
5. **Forbidden dependencies:** Same as STORY-022.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `toml` | `=0.8` | Parse `brand.toml` into `BrandConfig` |
| `quick-xml` | `=0.36` | Generate layout XML with correct element ordering |
| All from STORY-022 | see STORY-022 | Reused |

Dev dependencies:
- `proptest = "=1.5.0"` — determinism property test (AC-007 and future VP-012)

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/src/
├── synthesizer.rs         # BrandSynthesizer: entry point; loads brand.toml → BrandTemplate
├── toml_schema.rs         # BrandConfig struct (brand.toml serde schema)
├── inference.rs           # infer_missing_slots(declared: &[Option<Arc<str>>; 12]) -> [ColorSlot; 12]
├── layouts.rs             # generate_all_layouts(config: &BrandConfig) -> Vec<SlideLayoutDef>
│                          # SlideLayoutDef struct (name, ooxml_type, placeholders, color_override)
└── layout_xml.rs          # serialize_layout_to_xml(def: &SlideLayoutDef) -> Vec<u8>
```

Files to modify (from STORY-022):

```
crates/slideforge-brand/src/
├── template.rs            # Add: SlideLayoutDef, LayoutPlaceholder, MasterIds; extend BrandTemplate
└── lib.rs                 # Add: pub mod synthesizer, toml_schema, inference, layouts, layout_xml
```

## Tasks

1. **Extend `template.rs`** — add `SlideLayoutDef`, `LayoutPlaceholder`, `MasterIds` types. Extend `BrandTemplate` with `layouts: Vec<SlideLayoutDef>`, `notes_master_stub: Vec<u8>`, `handout_master_stub: Vec<u8>`, `master_id: u32` (= 2^31), `layout_id_start: u32` (= 2^31 + 1). (20 min)
2. **Write `src/toml_schema.rs`** — `BrandConfig` struct with serde derives for all sections listed in AC-001 schema. All fields optional except validated at synthesis time. (30 min)
3. **Write `src/inference.rs`** — `infer_missing_slots(declared: [Option<&str>; 12]) -> [Arc<str>; 12]`. Implement the 7 deterministic rules from AC-003. Include helpers: `darken_hex(hex: &str, pct: f32) -> String`, `lighten_hex(hex: &str, pct: f32) -> String`, `rotate_hue(hex: &str, degrees: f32) -> String`, `darkest_declared(slots: &[Option<&str>]) -> &str`. (45 min)
4. **Write `src/layouts.rs`** — `generate_all_layouts(config: &BrandConfig) -> Vec<SlideLayoutDef>`. Define all 31 layouts as structured data (not raw XML strings). Each `SlideLayoutDef` specifies:
   - `index: usize` (1–31)
   - `name: Arc<str>` (e.g., "Title Slide" or "SF Section Divider")
   - `ooxml_type: Option<Arc<str>>` (e.g., `"title"`, `"obj"`, `None` for custom)
   - `placeholders: Vec<LayoutPlaceholder>` (type, idx, position as Emu values)
   - `has_color_override: bool` (true for CL-01 and CL-11)
   - `color_override_bg: Option<Arc<str>>` (`"dk2"` for dark layouts)
   - `color_override_tx: Option<Arc<str>>` (`"lt1"` for dark layouts)
   (60 min)
5. **Write `src/layout_xml.rs`** — `serialize_layout_to_xml(def: &SlideLayoutDef, master_rel_id: &str) -> Vec<u8>`. Generate complete OOXML for a single `slideLayoutN.xml`. Use `quick_xml::Writer`. Include: `<p:sldLayout>` root, `<p:cSld>`, `<p:spTree>`, one `<p:sp>` per placeholder with `<p:ph type="..." idx="...">`, `<p:nvSpPr>` with semantic name in `<p:cNvPr name="...">`, explicit white text runs for dark layouts. For CL-01 and CL-11, include `<p:clrMapOvr>`. (90 min)
6. **Write `src/synthesizer.rs`** — `BrandSynthesizer`:
   - `load_from_toml(path: &str) -> Result<BrandTemplate, BrandError>`: reads file, parses `BrandConfig`, calls `synthesize`.
   - `synthesize(config: &BrandConfig) -> Result<BrandTemplate, Vec<BrandError>>`: pure function; calls inference, layout generation, XML serialization. Returns `BrandTemplate`.
   (45 min)
7. **Write `BrandProvider` impl** for `BrandSynthesizer` (the `slideforge-plugin-api` trait). (15 min)
8. **Write unit tests for `inference.rs`** — test all 7 derivation rules. (25 min)
9. **Write snapshot tests for `layouts.rs`** — use `insta` to snapshot the `SlideLayoutDef` list. Verify count = 31. Verify CL-01 and CL-11 have `has_color_override = true`. (20 min)
10. **Write proptest for determinism (AC-007)** — generate random `BrandConfig` via proptest; verify `synthesize(config) == synthesize(config)`. (20 min)
11. **Write integration test** — full synthesis from minimal `brand.toml` → count 31 layouts; verify master ID = 2^31; verify no E-BRD-003 warnings when all 12 colors declared. (20 min)
12. **Run `cargo clippy -p slideforge-brand -- -D warnings`** — fix all. (15 min)
13. **Run `cargo test -p slideforge-brand`** — all tests pass. (10 min)

## Test Strategy

### Unit tests for `inference.rs`

| Test | Setup | Expected |
|------|-------|----------|
| `test_all_12_declared_no_inference` | All 12 slots in `BrandConfig.colors` | Zero `E-BRD-003` warnings; returned slots = declared values |
| `test_only_acc1_dk1` | Only `acc1` and `dk1` declared | 10 `E-BRD-003` warnings; `lt1 = "#FFFFFF"` |
| `test_empty_colors_section` | `BrandConfig.colors = {}` (all None) | 12 `E-BRD-003` warnings; `lt1 = "#FFFFFF"` (hardcoded); other slots from default baseline |
| `test_hue_rotation_acc2` | `acc1 = "#3B82F6"`, `acc2` absent | `acc2` hex has 30° rotated hue from `acc1` |
| `test_dk2_from_acc1` | `acc1 = "#3B82F6"`, `dk1 = "#1F2937"`, `dk2` absent | `dk2 = acc1` color |
| `test_inferred_colors_are_uppercase_hex` | Any partial config | All returned hex values are `"#[A-F0-9]{6}"` |

### Snapshot tests for layout generation

```rust
#[test]
fn test_31_layouts_generated() {
    let config = BrandConfig::default_minimal(); // acc1 = "#3B82F6", logo = "test.png"
    let template = BrandSynthesizer::synthesize(&config).unwrap();
    assert_eq!(template.layouts.len(), 31);
    insta::assert_debug_snapshot!("layout_names", template.layouts.iter().map(|l| &l.name).collect::<Vec<_>>());
}

#[test]
fn test_dark_layouts_have_color_override() {
    let template = BrandSynthesizer::synthesize(&minimal_config()).unwrap();
    let cl01 = &template.layouts[11]; // CL-01 (index 11, 0-based)
    let cl11 = &template.layouts[21]; // CL-11 (index 21)
    assert!(cl01.has_color_override);
    assert!(cl11.has_color_override);
    assert_eq!(cl01.color_override_bg.as_deref(), Some("dk2"));
}

#[test]
fn test_standard_layouts_have_correct_types() {
    let template = BrandSynthesizer::synthesize(&minimal_config()).unwrap();
    assert_eq!(template.layouts[0].ooxml_type.as_deref(), Some("title"));
    assert_eq!(template.layouts[1].ooxml_type.as_deref(), Some("obj"));
    assert_eq!(template.layouts[6].ooxml_type.as_deref(), Some("blank"));
}

#[test]
fn test_blank_layout_has_zero_placeholders() {
    let template = BrandSynthesizer::synthesize(&minimal_config()).unwrap();
    let blank = &template.layouts[6]; // SL-07 blank
    assert_eq!(blank.placeholders.len(), 0);
}
```

### Proptest for determinism

```rust
proptest! {
    #[test]
    fn test_synthesis_is_deterministic(colors in arb_brand_colors()) {
        let config = BrandConfig { colors, ..Default::default() };
        let t1 = BrandSynthesizer::synthesize(&config).unwrap();
        let t2 = BrandSynthesizer::synthesize(&config).unwrap();
        prop_assert_eq!(t1.layouts.len(), t2.layouts.len());
        for (l1, l2) in t1.layouts.iter().zip(t2.layouts.iter()) {
            prop_assert_eq!(&l1.name, &l2.name);
        }
        // color slots equal
        for (c1, c2) in t1.colors.iter().zip(t2.colors.iter()) {
            prop_assert_eq!(&c1.hex, &c2.hex);
        }
    }
}
```

## Dependencies

**Depends on:**
- STORY-022 (Brand Loading) — `BrandTemplate`, `ColorSlot`, `BrandFonts`, `BrandError`, `LogoAsset` types all defined in STORY-022.

Dependency justification: STORY-023 depends on STORY-022 because the synthesizer produces a `BrandTemplate` — the same type loaded from `.pptx`/`.docx` by the brand loader. The struct definition must exist before synthesis can be implemented.

**Blocks:**
- STORY-024 (Brand Extraction CLI) — the extraction uses the synthesizer to verify round-trip: `.pptx` → `brand.toml` → `BrandTemplate`.
- STORY-037 (PPTX Core Serialization) — PPTX exporter consumes `BrandTemplate` with 31 `SlideLayoutDef` entries.
- STORY-038 (PPTX Layout Compliance) — depends on the layout IDs and placeholder definitions from this story.
- STORY-039 (PPTX Accessibility Metadata) — depends on semantic placeholder names from this story.
- STORY-040 (PPTX Notes/Sections/Masters) — depends on `notesMaster1.xml` and `handoutMaster1.xml` stubs.

## Implementation Notes

### Layout Placeholder Positions (Spike S5 §1.4)

Standard Office unit for positions in OOXML `<a:off x="..." y="...">` and sizes `<a:ext cx="..." cy="...">` is EMU (English Metric Units). Use `Emu` type from `slideforge-types`:

Slide dimensions: `9,144,000 EMU × 5,143,500 EMU` (10" × 5.625" at 16:9).

Standard title placeholder (`ctrTitle` or `title`) on most layouts:
- Position: `x = 457,200` (0.5"), `y = 274,638` (0.3")
- Size: `cx = 8,229,600` (9"), `cy = 1,143,000` (1.25")

Body/content placeholder on content layouts:
- Position: `x = 457,200`, `y = 1,600,200` (1.75")
- Size: `cx = 8,229,600`, `cy = 3,543,300` (3.875")

These are the PowerPoint defaults. Custom layouts (SF series) may have different positions per the layout type (e.g., SF Stat Grid has a 2×3 grid). Exact positions per layout are documented in Spike S5 §1.4 — use the values from that document.

For the initial implementation, using approximate standard positions is acceptable. The PPTX visual regression CI (STORY-052) will catch any layout position drift.

### XML Generation with quick-xml

Example of generating a placeholder `<p:sp>` element in correct OOXML schema order:

```rust
use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event, BytesEnd};
use std::io::Cursor;

fn write_placeholder(writer: &mut Writer<Cursor<Vec<u8>>>, ph_type: &str, idx: u32, name: &str) {
    // <p:sp>
    writer.write_event(Event::Start(BytesStart::new("p:sp"))).unwrap();
    // <p:nvSpPr>
    writer.write_event(Event::Start(BytesStart::new("p:nvSpPr"))).unwrap();
    // <p:cNvPr id="2" name="Content Placeholder 1"/>
    let mut cnv = BytesStart::new("p:cNvPr");
    cnv.push_attribute(("id", "2"));
    cnv.push_attribute(("name", name));
    writer.write_event(Event::Empty(cnv)).unwrap();
    // ... (additional elements in ECMA-376 order)
}
```

### Color Manipulation Utilities

Implement hex color manipulation in `inference.rs`:

```rust
fn hex_to_hsl(hex: &str) -> (f32, f32, f32) { /* RGB → HSL */ }
fn hsl_to_hex(h: f32, s: f32, l: f32) -> String { /* HSL → #RRGGBB uppercase */ }

fn darken_hex(hex: &str, pct: f32) -> String {
    let (h, s, l) = hex_to_hsl(hex);
    hsl_to_hex(h, s, (l * (1.0 - pct)).max(0.0))
}

fn lighten_hex(hex: &str, pct: f32) -> String {
    let (h, s, l) = hex_to_hsl(hex);
    hsl_to_hex(h, s, (l + (1.0 - l) * pct).min(1.0))
}

fn rotate_hue(hex: &str, degrees: f32) -> String {
    let (h, s, l) = hex_to_hsl(hex);
    hsl_to_hex((h + degrees) % 360.0, s, l)
}
```

All output hex values must be uppercase 6-digit, no `#` prefix in the internal representation (the `#` is prepended at the `ColorSlot` display level).

### brand.toml Field Names

The `[colors]` section uses the OOXML canonical names for direct mapping. The field
`fol_hlink` (TOML-safe name) maps to OOXML `folHlink`. This mapping is explicit in
`BrandConfig` via a serde rename attribute: `#[serde(rename = "fol_hlink")]`.

### notesMaster1.xml Stub

Minimal valid `notesMaster1.xml` XML bytes to include as stub:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
               xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld><p:spTree/></p:cSld>
  <p:hf/><p:notesStyle/>
</p:notesMaster>
```

Similarly for `handoutMaster1.xml`. These stubs are hardcoded as `&[u8]` constants in `layout_xml.rs`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `brand.toml` with only `acc1` and `dk1` | 10 E-BRD-003 warnings; all 10 remaining slots inferred |
| EC-002 | `brand.toml` completely empty | 12 E-BRD-003 warnings; full palette from default baseline |
| EC-003 | All 12 colors declared | 0 warnings; exact declared values used |
| EC-004 | Font in `[fonts]` not installed on build host | E-BRD-004 warning; fallback used; OOXML writes declared name |
| EC-005 | Missing `[logo]` path in synthesized brand | E-BRD-001 fatal error |
| EC-006 | Logo PNG > 5 MB | Logo embedded; lint warning; no error |
| EC-007 | CL-01 opened in LibreOffice 7.x | White text visible because explicit white run colors are present (R4 mitigation) |
| EC-008 | `slideforge extract-brand` on synthesized PPTX | All 31 layouts detected; colors match `brand.toml` (round-trip) |
