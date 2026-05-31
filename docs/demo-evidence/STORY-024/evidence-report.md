# Evidence Report: STORY-024 — Brand Extraction (BrandExtractor library)

**Story ID:** STORY-024
**Epic:** EPIC-06
**Crate:** `slideforge-brand`
**BC Trace:** BC-2.01.003
**HEAD SHA verified:** `88f31e53`
**Workspace test count:** 211 tests run: 211 passed, 0 skipped
**Recording tool:** VHS (terminal — library story, nextest harness demos)
**Font:** FiraCode Nerd Font Mono

> **Library-only story note:** STORY-024 implements `BrandExtractor` at the library level only.
> The CLI subcommand `slideforge extract-brand <template.pptx>` is wired in STORY-057.
> All demos therefore invoke the AC-specific tests via `cargo nextest run -p slideforge-brand`
> rather than a `slideforge` binary. This is the established pattern for library stories
> (see STORY-020, STORY-021 for precedent).

---

## Coverage Summary

All 11 acceptance criteria have recorded demo evidence. Each demo drives the relevant
integration tests in `slideforge-brand::extractor::tests` via `cargo nextest run -p slideforge-brand`.

| AC | Description | Demo File | Tests Covered | Status |
|----|-------------|-----------|--------------|--------|
| AC-001 | `extract()` writes `brand.toml` to `output_dir`; returns `BrandExtractionResult` | `AC-001-extract-writes-brand-toml` | `test_bc_2_01_003_extract_writes_brand_toml`, `test_bc_2_01_003_extraction_result_fields_accessible` | PASSED |
| AC-002 | `OutputExists` error when `force=false`; `force=true` overwrites | `AC-002-output-exists-guard` | `test_bc_2_01_003_rejects_existing_output_without_force`, `test_bc_2_01_003_force_overwrites_existing_brand_toml`, `test_bc_2_01_003_output_exists_error_message` | PASSED |
| AC-003 | All 12 OOXML color slots in `[colors]`; ECMA-376 sequential order | `AC-003-all-12-color-slots` | `test_bc_2_01_003_all_12_color_slots_in_colors_section`, `test_bc_2_01_003_invariant_color_slots_in_ecma376_order` | PASSED |
| AC-004 | Logo bytes copied to `brand.assets/logo.<ext>`; `[logo]` section written; absent logo omits section | `AC-004-logo-copied-to-brand-assets` | `test_bc_2_01_003_copies_logo_to_brand_assets`, `test_bc_2_01_003_no_logo_omits_logo_section` | PASSED |
| AC-005 | `sysClr` elements use `lastClr` attribute as hex in `brand.toml` | `AC-005-sysclr-uses-lastclr` | `test_bc_2_01_003_sysclr_uses_last_clr_value` | PASSED |
| AC-006 | `lumMod`/`tint`/`shade` transforms: base hex with inline TOML comment; resolvable refs write resolved hex | `AC-006-tint-shade-inline-comment` | `test_bc_2_01_003_ec003_scheme_ref_slot_gets_inline_toml_comment`, `test_bc_2_01_003_ec003_resolvable_scheme_ref_writes_resolved_hex`, `test_bc_2_01_003_ec003_scheme_ref_end_to_end_round_trip` | PASSED |
| AC-007 | Multiple slide masters: only `slideMaster1.xml` extracted; lint warning emitted | `AC-007-multiple-masters-uses-master1` | `test_bc_2_01_003_multiple_slide_masters_uses_master1_only` | PASSED |
| AC-008 | Source `.pptx` never modified; SHA-256 before/after identical | `AC-008-source-file-read-only` | `test_bc_2_01_003_invariant_source_file_unmodified` | PASSED |
| AC-009 | `brand.toml` has `[colors]`/`[fonts]`/`[logo]`/`[footer]` headers; one field per line; stable order | `AC-009-stable-toml-output-order` | `test_bc_2_01_003_brand_toml_has_section_headers_and_stable_order` | PASSED |
| AC-010 | Round-trip: `.pptx` → `brand.toml` → `BrandTemplate` colors match original | `AC-010-round-trip-pptx-to-brand-toml` | `test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back`, `test_bc_2_01_003_effectful_extract_load_from_toml_round_trip` | PASSED |
| AC-011 | `#![forbid(unsafe_code)]` present; clippy::pedantic clean; `=` version pinning | `AC-011-forbid-unsafe-clippy-pinning` | `grep forbid(unsafe_code)` in `lib.rs`, `cargo clippy -D warnings` | PASSED |

---

## AC-001: Extract Writes brand.toml

**File:** `AC-001-extract-writes-brand-toml.{tape,gif,webm}`
**BC trace:** BC-2.01.003 postcondition 1 — `brand.toml` written with `[colors]`, `[fonts]`, `[logo]`, `[footer]`

`BrandExtractor::extract(source, out_dir, force=false)` calls `BrandLoader::load()` to read the
source `.pptx`, converts the resulting `BrandTemplate` to `BrandConfig`, serializes to TOML, and
writes `out_dir/brand.toml`. On success it returns `BrandExtractionResult { brand_toml_path, logo_asset_path }`.
`test_bc_2_01_003_extraction_result_fields_accessible` verifies both result fields are accessible
and carry the expected types.

Tests: `test_bc_2_01_003_extract_writes_brand_toml`, `test_bc_2_01_003_extraction_result_fields_accessible`

---

## AC-002: OutputExists Guard

**File:** `AC-002-output-exists-guard.{tape,gif,webm}`
**BC trace:** BC-2.01.003 edge case EC-001 — existing `brand.toml` without `--force`

When `output_dir/brand.toml` already exists and `force=false`, `extract()` returns
`BrandError::OutputExists { path }` with message `"<path>: brand.toml already exists. Use --force to overwrite."`.
The existing file content is verified to be unmodified. When `force=true`, the file is overwritten
and `BrandExtractionResult` is returned successfully.
`test_bc_2_01_003_output_exists_error_message` validates the exact error message text including
the `--force` hint and the `E-BRD-006` error code constant.

Tests: `test_bc_2_01_003_rejects_existing_output_without_force`,
`test_bc_2_01_003_force_overwrites_existing_brand_toml`,
`test_bc_2_01_003_output_exists_error_message`

---

## AC-003: All 12 Color Slots in ECMA-376 Order

**File:** `AC-003-all-12-color-slots.{tape,gif,webm}`
**BC trace:** BC-2.01.003 postcondition 2 — all 12 OOXML slots represented; invariant 1 — ECMA-376 sequential order

The `[colors]` section in `brand.toml` contains all 12 semantic field names:
`dk1`, `lt1`, `dk2`, `lt2`, `acc1`–`acc6`, `hlink`, `fol_hlink` (TOML-safe rename for `folHlink`).
`test_bc_2_01_003_invariant_color_slots_in_ecma376_order` verifies the order is deterministic
across repeated extraction calls by asserting field positions in the serialized TOML string.

Tests: `test_bc_2_01_003_all_12_color_slots_in_colors_section`,
`test_bc_2_01_003_invariant_color_slots_in_ecma376_order`

---

## AC-004: Logo Copied to brand.assets

**File:** `AC-004-logo-copied-to-brand-assets.{tape,gif,webm}`
**BC trace:** BC-2.01.003 postcondition 3 — logo copied to `brand.assets/logo.<ext>`

When the source `.pptx` contains a logo image embedded in `slideMaster1.xml.rels`, its bytes are
written verbatim to `output_dir/brand.assets/logo.<ext>` where `<ext>` matches the original ZIP
entry's file extension (`png`, `jpg`, `svg`, etc.). The `brand.toml` `[logo]` section contains
`path = "brand.assets/logo.<ext>"`. When no logo is found, the `[logo]` section is omitted entirely
from the TOML output.

Tests: `test_bc_2_01_003_copies_logo_to_brand_assets`,
`test_bc_2_01_003_no_logo_omits_logo_section`

---

## AC-005: sysClr Uses lastClr

**File:** `AC-005-sysclr-uses-lastclr.{tape,gif,webm}`
**BC trace:** BC-2.01.003 edge case EC-002 — `sysClr` → `lastClr`

When `theme1.xml` contains `<a:sysClr>` elements (Windows system color references), the extractor
reads the `lastClr` attribute value and uses it as the hex color written to `brand.toml`. This
mirrors the same resolution logic already present in `BrandLoader` from STORY-022. The test
constructs a fixture PPTX with `sysClr val="windowText" lastClr="000000"` and asserts the
extracted `dk1` slot value is `"#000000"`.

Tests: `test_bc_2_01_003_sysclr_uses_last_clr_value`

---

## AC-006: Tint/Shade Transforms — Inline TOML Comment

**File:** `AC-006-tint-shade-inline-comment.{tape,gif,webm}`
**BC trace:** BC-2.01.003 edge case EC-003 — tint/shade transforms noted in comment

Color slots with `lumMod`/`tint`/`shade` modifier children in `theme1.xml` are written as their
best-effort base hex, with an inline TOML comment `# derived via tint/shade; may not match exact color`.
For `schemeClr` references that can be resolved to a sibling slot's hex, the resolved hex is
written (without comment). For self-references or unresolvable mnemonics (`tx1`, `bg1`, `phClr`),
the color default hex is used with the comment. The end-to-end round-trip test verifies that
extraction + re-synthesis produces a `BrandTemplate` whose colors are within the expected
best-effort bounds.

Tests: `test_bc_2_01_003_ec003_scheme_ref_slot_gets_inline_toml_comment`,
`test_bc_2_01_003_ec003_resolvable_scheme_ref_writes_resolved_hex`,
`test_bc_2_01_003_ec003_scheme_ref_end_to_end_round_trip`

---

## AC-007: Multiple Slide Masters Uses Master1 Only

**File:** `AC-007-multiple-masters-uses-master1.{tape,gif,webm}`
**BC trace:** BC-2.01.003 edge case EC-004 — multiple slide masters

When the source `.pptx` ZIP contains more than one slide master (e.g., `slideMaster1.xml` and
`slideMaster2.xml`), only `slideMaster1.xml` is processed. A `tracing::warn!` message is emitted:
`"Source .pptx has multiple slide masters; extracting from slideMaster1.xml only."` The test
uses `tracing_test::traced_test` to capture and assert the warning log line.

Tests: `test_bc_2_01_003_multiple_slide_masters_uses_master1_only`

---

## AC-008: Source File Never Modified (VP-052)

**File:** `AC-008-source-file-read-only.{tape,gif,webm}`
**BC trace:** BC-2.01.003 invariant 2 — extraction is read-only; exercises VP-052

The test computes the SHA-256 digest of the source `.pptx` file before calling `extract()`,
then recomputes it after the call completes and asserts byte-for-byte identity. This provides
formal evidence that `BrandExtractor` never opens the source in write mode or modifies the ZIP
archive in place.

Tests: `test_bc_2_01_003_invariant_source_file_unmodified`

---

## AC-009: Stable TOML Output Order

**File:** `AC-009-stable-toml-output-order.{tape,gif,webm}`
**BC trace:** BC-2.01.003 invariant 3 — semantic field names are fixed; implies stable ordering

The serialized `brand.toml` uses `IndexMap`-backed `BrandConfig` structs (from STORY-023) so
that `toml::to_string()` emits fields in insertion order rather than `HashMap` iteration order.
The test asserts that the TOML string contains all four section headers in the canonical sequence
(`[colors]` → `[fonts]` → `[logo]` → `[footer]`), that all 12 color field names appear in
ECMA-376 order, and that running `extract()` twice on the same input produces byte-identical
output.

Tests: `test_bc_2_01_003_brand_toml_has_section_headers_and_stable_order`

---

## AC-010: Round-Trip Verification (VP-051)

**File:** `AC-010-round-trip-pptx-to-brand-toml.{tape,gif,webm}`
**BC trace:** BC-2.01.003 verification property — round-trip integration test; exercises VP-051

The round-trip test extracts `brand.toml` from a fixture `.pptx`, then loads the resulting
`brand.toml` back via `BrandLoader::load_from_toml()` (STORY-022) and compares the 12 color hex
values of the reconstructed `BrandTemplate` against the original template's colors. The effectful
round-trip test (`test_bc_2_01_003_effectful_extract_load_from_toml_round_trip`) exercises the
full `BrandExtractor::extract()` → filesystem → `BrandSynthesizer::load_from_toml()` path,
verifying that all 12 hex values survive the serialization/deserialization cycle unchanged.

Tests: `test_bc_2_01_003_round_trip_pptx_to_brand_toml_and_back`,
`test_bc_2_01_003_effectful_extract_load_from_toml_round_trip`

---

## AC-011: Code-Quality Gates

**File:** `AC-011-forbid-unsafe-clippy-pinning.{tape,gif,webm}`
**Traces to:** NFR-024 (no unsafe), NFR-022 (clippy pedantic), NFR-025 (= version pinning)

The demo runs two commands:
1. `grep -n 'forbid(unsafe_code)' crates/slideforge-brand/src/lib.rs` — confirms the attribute is present.
2. `cargo clippy -p slideforge-brand --all-targets --all-features -- -D warnings` — exits clean with zero warnings.

Version pinning (`=` prefix on all production deps in `Cargo.toml`) was verified during
implementation and is enforced by the pre-push hook.

---

## File Manifest

```
docs/demo-evidence/STORY-024/
├── AC-001-extract-writes-brand-toml.tape
├── AC-001-extract-writes-brand-toml.gif
├── AC-001-extract-writes-brand-toml.webm
├── AC-002-output-exists-guard.tape
├── AC-002-output-exists-guard.gif
├── AC-002-output-exists-guard.webm
├── AC-003-all-12-color-slots.tape
├── AC-003-all-12-color-slots.gif
├── AC-003-all-12-color-slots.webm
├── AC-004-logo-copied-to-brand-assets.tape
├── AC-004-logo-copied-to-brand-assets.gif
├── AC-004-logo-copied-to-brand-assets.webm
├── AC-005-sysclr-uses-lastclr.tape
├── AC-005-sysclr-uses-lastclr.gif
├── AC-005-sysclr-uses-lastclr.webm
├── AC-006-tint-shade-inline-comment.tape
├── AC-006-tint-shade-inline-comment.gif
├── AC-006-tint-shade-inline-comment.webm
├── AC-007-multiple-masters-uses-master1.tape
├── AC-007-multiple-masters-uses-master1.gif
├── AC-007-multiple-masters-uses-master1.webm
├── AC-008-source-file-read-only.tape
├── AC-008-source-file-read-only.gif
├── AC-008-source-file-read-only.webm
├── AC-009-stable-toml-output-order.tape
├── AC-009-stable-toml-output-order.gif
├── AC-009-stable-toml-output-order.webm
├── AC-010-round-trip-pptx-to-brand-toml.tape
├── AC-010-round-trip-pptx-to-brand-toml.gif
├── AC-010-round-trip-pptx-to-brand-toml.webm
├── AC-011-forbid-unsafe-clippy-pinning.tape
├── AC-011-forbid-unsafe-clippy-pinning.gif
├── AC-011-forbid-unsafe-clippy-pinning.webm
└── evidence-report.md
```

Total: 33 recording files (11 AC x 3 formats each) + this report = 34 files.
