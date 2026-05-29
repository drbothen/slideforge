# Evidence Report — STORY-023: Brand Synthesis: brand.toml → 31 Layouts + 12 OOXML Slots

**Story ID:** STORY-023
**Crate:** `slideforge-brand`
**Branch:** `feature/S-023`
**Convergence:** 3/3 CLEAN (Passes 18, 19, 20)
**Test result:** 176 unit/integration + 5 load_from_toml + 1 doctest = **182 total passed; 0 failed**

---

## Summary

`BrandSynthesizer` reads `brand.toml` and deterministically produces a complete `BrandTemplate`
containing 12 OOXML color slots (with inference for missing ones), 31 `SlideLayoutDef` entries
(11 standard + 20 SF-custom), `notesMaster1.xml` + `handoutMaster1.xml` stubs,
`[Content_Types].xml` registration for all 31 layouts, and OOXML-schema-compliant master/layout IDs.

---

## AC Coverage

### AC-001 — brand.toml parsed to BrandConfig

**Spec:** `brand.toml` parsed via `toml = "=0.8"` into strongly-typed `BrandConfig` with sections
`[colors]`, `[fonts]`, `[logo]`, `[footer]`.

**Implementation:** `crates/slideforge-brand/src/toml_schema.rs` — `BrandConfig`, `ColorConfig`,
`FontConfig`, `LogoConfig`, `FooterConfig` structs with `#[serde(deny_unknown_fields)]`.

**Load path:** `synthesizer.rs:100` — `BrandSynthesizer::load_from_toml(path)`.

**Load tests:**
- `tests/load_from_toml.rs:15` — `test_load_brand_toml_end_to_end`
- `tests/load_from_toml.rs:157` — `test_load_brand_toml_invalid_toml_returns_parse_error`

**Status: PASS**

---

### AC-002 — All 12 OOXML color slots always present

**Spec:** `BrandTemplate` always has exactly 12 color slots. Missing slots are inferred; zero slots
in `brand.toml` triggers 12 E-BRD-003 warnings.

**Implementation:** `synthesizer.rs:202` — `BrandSynthesizer::synthesize` calls
`infer_missing_slots` and validates slot count before returning `BrandTemplate`.

**Tests:**
- `synthesizer::tests::test_bc_2_01_002_synthesized_template_has_12_color_slots`
- `synthesizer::tests::test_bc_2_01_005_synthesize_with_inferred_colors`

**Status: PASS**

---

### AC-003 — Missing slot inference algorithm

**Spec:** Deterministic inference: `dk1` from darkest declared; `lt1` always `#FFFFFF`;
`dk2 = acc1` if declared; `lt2 = #F9FAFB`; `acc2`–`acc6` from 30°/60°/90°/120°/150° hue rotation;
`hlink = acc1` darkened 15%; `fol_hlink = hlink` darkened 10%.

**Implementation:** `crates/slideforge-brand/src/inference.rs:110` — `infer_missing_slots`.
Color helpers `darken_hex`, `lighten_hex`, `rotate_hue` in same file.

**Tests:**
- `inference::tests::test_bc_2_01_004_all_12_declared_no_inference`
- `inference::tests::test_bc_2_01_004_lt1_always_ffffff`
- `inference::tests::test_bc_2_01_004_dk2_from_acc1_when_available`
- `inference::tests::test_bc_2_01_004_acc2_hue_rotation_30_degrees`
- `inference::tests::test_bc_2_01_004_hlink_darkened_from_acc1`
- `inference::tests::test_bc_2_01_004_fol_hlink_darkened_from_hlink`
- `inference::tests::test_bc_2_01_004_lt2_inferred_as_near_white`

**Status: PASS**

---

### AC-004 — LogoRequired error on missing logo path (error path demo)

**Spec:** `BrandConfig.logo.path` required; missing path → `BrandError::LogoRequired { span }` +
`E-BRD-001`.

**Implementation:** `synthesizer.rs` — `synthesize` checks logo presence before producing
`BrandTemplate`; `error.rs` — `BrandError::LogoRequired`.

**Tests:**
- `synthesizer::tests::test_bc_2_01_002_ac004_synthesize_rejects_missing_logo`
- `synthesizer::tests::test_f13_logo_required_has_span_field`
- `synthesizer::tests::test_f14_empty_logo_path_is_rejected`
- `tests/load_from_toml.rs` — `test_load_brand_toml_missing_logo_returns_file_not_found`

**Recording:** `AC-004-logo-required-error.gif` / `.webm`

**Status: PASS**

---

### AC-005 — Empty/zero-colors brand.toml yields 12 E-BRD-003 warnings

**Spec:** Completely empty `brand.toml` → 12 `MissingColorSlot` warnings, all 12 slots inferred
from default baseline. Build continues.

**Implementation:** `inference.rs` — all `None` declared slots fall through to fallback chain.

**Tests:**
- `inference::tests::test_bc_2_01_004_empty_colors_produces_12_warnings`
- `synthesizer::tests::test_bc_2_01_005_synthesize_empty_colors_yields_12_warnings`
- `tests/load_from_toml.rs` — `test_load_brand_toml_empty_colors_section_yields_12_warnings`

**Status: PASS**

---

### AC-006 — Lowercase hex accepted and normalized to uppercase

**Spec:** User-typed lowercase hex (e.g., `#3b82f6`) accepted and normalized to uppercase
(`#3B82F6`). Inferred colors always uppercase. No CSS named colors or alpha channels.

**Implementation:** `inference.rs` — `validate_and_normalize_hex` called on every declared slot
before inference; `hsl_to_hex` always produces uppercase output.

**Tests:**
- `inference::tests::test_validate_hex_accepts_lowercase_and_normalizes`
- `inference::tests::test_validate_hex_accepts_mixed_case_and_normalizes`
- `inference::tests::test_validate_hex_uppercase_passthrough`
- `inference::tests::test_f6_validate_hex_rejects_named_color`
- `inference::tests::test_f6_validate_hex_rejects_alpha_hex`

**Recording:** `AC-006-lowercase-hex-normalised.gif` / `.webm`

**Status: PASS**

---

### AC-007 — Synthesis is deterministic (VP-012)

**Spec:** Same `brand.toml` bytes → identical `BrandTemplate`. Two calls with same input compare
equal.

**Implementation:** `synthesizer.rs:202` — `synthesize(&config)` is a pure function. No I/O, no
random state. `inference.rs` uses only arithmetic operations on input hex values.

**Tests:**
- `synthesizer::tests::test_bc_2_01_002_invariant_synthesis_is_deterministic`
- `synthesizer::tests::proptest_vp012::test_bc_2_01_002_invariant_synthesis_is_deterministic_proptest` (proptest — arbitrary `BrandConfig`)
- `synthesizer::tests::proptest_vp012::test_bc_2_01_004_invalid_hex_synthesis_is_deterministic` (invalid hex variant)
- `inference::tests::test_bc_2_01_004_inference_is_deterministic`

**Recording:** `AC-007-determinism-proptest.gif` / `.webm`

**Status: PASS**

---

### AC-008 — 31 layouts generated (11 standard + 20 SF custom)

**Spec:** `BrandTemplate.layouts` contains exactly 31 `SlideLayoutDef` entries. SL-01–SL-11 are
standard OOXML types. CL-01–CL-20 are custom SF layouts with `"SF "` name prefix.

**Implementation:** `crates/slideforge-brand/src/layouts.rs:241` — `generate_all_layouts`.
All 31 layouts defined as structured data with `index`, `name`, `ooxml_type`, `placeholders`,
`has_color_override`.

**Tests:**
- `layouts::tests::test_bc_2_01_005_generate_31_layouts`
- `layouts::tests::test_bc_2_01_005_11_standard_layouts_present`
- `layouts::tests::test_bc_2_01_005_20_sf_custom_layouts_present`
- `layouts::tests::test_bc_2_01_005_standard_layouts_have_ooxml_types`
- `synthesizer::tests::test_bc_2_01_005_synthesized_template_has_31_layouts`
- `synthesizer::tests::test_snapshot_all_31_layouts` (insta snapshot)

**Recording:** `AC-008-31-layouts-generated.gif` / `.webm`

**Status: PASS**

---

### AC-009 — CL-01 and CL-11 dark layouts with clrMapOvr + explicit white text

**Spec:** "SF Section Divider" (CL-01) and "SF End Slide" (CL-11) carry `clrMapOvr` with
`bg1="dk2" tx1="lt1"`. Placeholder runs have explicit `<a:solidFill><a:srgbClr val="FFFFFF"/>`.

**Implementation:** `layout_xml.rs:87` — `serialize_layout_to_xml` checks `def.has_color_override`;
emits `<p:clrMapOvr>` and explicit white run colors for dark layouts.
`layouts.rs` — `CL-01` and `CL-11` have `has_color_override: true`,
`color_override_bg: Some("dk2")`, `color_override_tx: Some("lt1")`.

**Tests:**
- `layouts::tests::test_bc_2_01_005_cl01_section_divider_dark_layout`
- `layouts::tests::test_bc_2_01_005_cl11_end_slide_dark_layout`
- `layout_xml::tests::test_bc_2_01_005_dark_layout_xml_has_clr_map_ovr`
- `layout_xml::tests::test_bc_2_01_005_dark_layout_xml_has_explicit_white_text`
- `layout_xml::tests::test_bc_2_01_005_light_layout_xml_has_no_clr_map_ovr`

**Status: PASS**

---

### AC-010 — Semantic placeholder names (not "Shape N"); Blank layout has zero placeholders

**Spec:** All `<p:cNvPr name="...">` use semantic labels. Blank layout (SL-07) has zero
placeholders.

**Implementation:** `layouts.rs` — each `LayoutPlaceholder` carries a semantic `name` field.
`layout_xml.rs` — `serialize_layout_to_xml` writes `name` into `<p:cNvPr>`.

**Tests:**
- `layouts::tests::test_bc_2_01_005_blank_layout_has_zero_placeholders`
- `layouts::tests::test_bc_2_01_005_layout_placeholder_names_are_semantic`
- `layouts::tests::test_bc_2_01_005_non_blank_layouts_have_title_placeholder`
- `layout_xml::tests::test_bc_2_01_005_layout_xml_uses_semantic_names`

**Status: PASS**

---

### AC-011 — master_id = 2^31; layout_id_start = 2^31 + 1; slide_id_start = 256

**Spec:** OOXML schema-compliant IDs. `BrandTemplate.master_ids.master_id = 2147483648`.

**Implementation:** `template.rs` — `MasterIds { master_id: u32, layout_id_start: u32, slide_id_start: u32 }`.
`synthesizer.rs` — hardcoded in `synthesize`: `master_id: 2u32.pow(31)`,
`layout_id_start: 2u32.pow(31) + 1`, `slide_id_start: 256`.

**Tests:**
- `synthesizer::tests::test_bc_2_01_005_ac011_master_id_is_2_pow_31`
- `synthesizer::tests::test_bc_2_01_005_ac011_layout_id_start_is_2_pow_31_plus_1`

**Recording:** `AC-011-master-id-2pow31.gif` / `.webm`

**Status: PASS**

---

### AC-012 — notesMaster1.xml + handoutMaster1.xml stubs present

**Spec:** Both stubs present as `Vec<u8>` in `BrandTemplate`. Must start with valid XML.

**Implementation:** `layout_xml.rs:40` — `NOTES_MASTER_STUB` constant; `layout_xml.rs:52` —
`HANDOUT_MASTER_STUB` constant. Both are hardcoded valid XML bytes.
`synthesizer.rs:248` — stubs cloned into `BrandTemplate`.

**Tests:**
- `layout_xml::tests::test_bc_2_01_002_notes_master_stub_is_non_empty`
- `layout_xml::tests::test_bc_2_01_002_handout_master_stub_is_non_empty`
- `synthesizer::tests::test_bc_2_01_002_ac012_notes_master_stub_present`
- `synthesizer::tests::test_bc_2_01_002_ac012_handout_master_stub_present`

**Status: PASS**

---

### AC-013 — content_types_layout_entries populated for all 31 layouts

**Spec:** `BrandTemplate.content_types_layout_entries` string registers all 31 layouts with
`PartName="/ppt/slideLayouts/slideLayoutN.xml"`.

**Implementation:** `layout_xml.rs:522` — `generate_content_types_layout_entries(count: usize) -> String`.
`synthesizer.rs:244` — called with `layouts.len()` (31) and stored in `BrandTemplate`.

**Tests:**
- `layout_xml::tests::test_bc_2_01_005_content_types_layout_entries_count`
- `synthesizer::tests::test_bc_2_01_005_synthesize_produces_complete_brand_template`
  (asserts non-empty and contains `"slideLayout"`)

**Status: PASS**

---

### AC-014 — VP-012 proptest passes (palette round-trip determinism)

**Spec:** proptest: synthesize from arbitrary `brand.toml`, serialize slots, re-synthesize →
identical hex values.

**Implementation:** `synthesizer.rs:1286` — `mod proptest_vp012` with two proptest cases.
`arb_brand_colors()` strategy generates arbitrary optional hex strings.

**Tests:**
- `synthesizer::tests::proptest_vp012::test_bc_2_01_002_invariant_synthesis_is_deterministic_proptest`
- `synthesizer::tests::proptest_vp012::test_bc_2_01_004_invalid_hex_synthesis_is_deterministic`

**Recording:** See AC-007 recording (same proptest module).

**Status: PASS**

---

### AC-015 — Lint gates: forbid(unsafe_code), warn(missing_docs), clippy::pedantic, = pinning

**Spec:** All four NFR gates maintained throughout the crate.

**Implementation:**
- `lib.rs` — `#![forbid(unsafe_code)]` (NFR-024)
- `lib.rs` — `#![warn(missing_docs)]` (NFR-023)
- `lib.rs` — `#![warn(clippy::pedantic)]` (NFR-022)
- `Cargo.toml` — all production deps pinned with `=` (NFR-025):
  `toml = "=0.8"`, `quick-xml = "=0.36"`, `serde = "=1.0"`, etc.

**Verification:** `cargo clippy -p slideforge-brand -- -D warnings` passes clean.

**Status: PASS**

---

## VHS Recordings

| File | AC | What it shows |
|------|----|---------------|
| `AC-004-logo-required-error.gif/.webm` | AC-004 | `BrandError::LogoRequired` returned when logo path absent |
| `AC-006-lowercase-hex-normalised.gif/.webm` | AC-006 | Lowercase `#3b82f6` accepted; normalized to `#3B82F6` |
| `AC-007-determinism-proptest.gif/.webm` | AC-007 + AC-014 | VP-012 proptest: 2 proptest cases for synthesis determinism |
| `AC-008-31-layouts-generated.gif/.webm` | AC-008 | `generate_all_layouts` produces exactly 31 layouts |
| `AC-011-master-id-2pow31.gif/.webm` | AC-011 | `master_id = 2147483648`, `layout_id_start = 2147483649` |

---

## Coverage Summary

| AC | Status | Recording |
|----|--------|-----------|
| AC-001 | PASS | — |
| AC-002 | PASS | — |
| AC-003 | PASS | — |
| AC-004 | PASS | AC-004-logo-required-error |
| AC-005 | PASS | — |
| AC-006 | PASS | AC-006-lowercase-hex-normalised |
| AC-007 | PASS | AC-007-determinism-proptest |
| AC-008 | PASS | AC-008-31-layouts-generated |
| AC-009 | PASS | — |
| AC-010 | PASS | — |
| AC-011 | PASS | AC-011-master-id-2pow31 |
| AC-012 | PASS | — |
| AC-013 | PASS | — |
| AC-014 | PASS | (AC-007 recording) |
| AC-015 | PASS | — |

**15/15 ACs: PASS. 0 deferred. 0 missing.**

---

## Cargo Test Summary

```
test result: ok. 176 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (load_from_toml integration)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (doc-tests)
```

Total: **182 passed; 0 failed**
