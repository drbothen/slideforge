---
story: STORY-022
phase: red-gate
date: 2026-05-27
status: PASSED
---

# Red Gate Log — STORY-022: Brand Loading: .pptx/.docx Template Extraction

## Summary

**Red Gate: PASSED**

All implementation-exercising tests fail against `todo!()` stubs. The crate
compiles cleanly. 23 infrastructure tests pass (constants, struct construction,
pure helper functions). 18 tests fail at stub boundaries.

## Counts

| Category | Count |
|----------|-------|
| Total tests | 41 |
| Passed (infrastructure/metadata only) | 23 |
| Failed (Red Gate tests — hit `todo!()`) | 18 |
| Doctests passed | 0 |

## Passing Tests (Infrastructure — No Stub Exercised)

These 23 tests pass because they test error message formatting, constant values,
struct construction, and fully-implemented pure helper functions — none exercise
`todo!()` stubs.

### `error.rs` (4 tests)
- `test_bc_2_01_001_error_codes` — verifies E_BRD_001 through E_BRD_004 string constants
- `test_bc_2_01_001_file_not_found_message` — verifies `BrandError::FileNotFound` display format
- `test_bc_2_01_001_parse_error_message` — verifies `BrandError::ParseError` display format
- `test_bc_2_01_001_missing_color_slot_message` — verifies `BrandError::MissingColorSlot` display format
- `test_bc_2_01_006_font_unavailable_message` — verifies `BrandError::FontUnavailable` display format

### `context.rs` (2 tests)
- `test_bc_2_01_001_brand_load_context_for_test` — struct construction
- `test_bc_2_01_006_brand_load_context_font_check_flag` — field access

### `font.rs` (3 tests)
- `test_bc_2_01_006_fallback_chain_non_empty` — `FALLBACK_CHAIN` constant
- `test_bc_2_01_006_fallback_chain_order` — `FALLBACK_CHAIN` ordering
- `test_bc_2_01_006_font_search_dirs_non_empty` — `font_search_dirs()` is fully implemented (pure `cfg!` + `dirs::home_dir()` logic, no `todo!()`)

### `loader.rs` (2 tests)
- `test_bc_2_01_001_internal_path_constants` — `PPTX_THEME_PATH` / `DOCX_THEME_PATH` constants
- `test_bc_2_01_001_brand_loader_provider_id` — `BrandLoader::id()` returns a string literal

### `logo.rs` (3 tests)
- `test_bc_2_01_001_slide_master_rels_path` — `SLIDE_MASTER_RELS_PATH` constant
- `test_bc_2_01_001_media_type_from_extension` — `media_type_from_extension()` fully implemented
- `test_bc_2_01_001_media_type_case_insensitive` — same function, case sensitivity

### `template.rs` (8 tests)
- `test_bc_2_01_001_brand_template_default_construction` — struct construction
- `test_bc_2_01_001_invariant_always_12_color_slots` — array length assertion
- `test_bc_2_01_001_color_order_matches_ecma_376` — `COLOR_SLOT_NAMES` constant
- `test_bc_2_01_001_color_by_name_lookup` — `BrandTemplate::color_by_name()` fully implemented
- `test_bc_2_01_001_color_by_name_unknown_returns_none` — same method, negative case
- `test_bc_2_01_001_logo_asset_construction` — struct construction
- `test_bc_2_01_001_layout_names_stored_correctly` — `Vec<Arc<str>>` field access
- `test_bc_2_01_001_color_slot_names_constant_length` — `COLOR_SLOT_NAMES.len()` == 12

All 23 passing tests exercise purely declared values or trivial struct accessors.
They exercise no XML parsing, ZIP loading, or font availability logic.
The passing tests are correct behavior under the Red Gate protocol.

## Failing Tests (Red Gate Confirmed)

All 18 failures are `thread ... panicked at ... not yet implemented: ...` from
`todo!()` stubs. Every failure has the correct BC-based test name and exercises
a specific behavioral contract clause.

### By module

| Module | Failing tests | BC covered |
|--------|--------------|------------|
| `color.rs` | 6 | BC-2.01.001 postcondition 1, invariant 1, AC-002, AC-003, EC-003 |
| `font.rs` | 4 | BC-2.01.001 AC-004, BC-2.01.006 postconditions |
| `loader.rs` | 8 | BC-2.01.001 all ACs, BC-2.01.006 invariant 1 |
| **Total** | **18** | |

### `color.rs` (6 failing)
- `test_bc_2_01_001_parse_12_srgb_colors` — hits `parse_theme_colors` todo
- `test_bc_2_01_001_parse_sysclr_uses_lastclr` — hits `parse_theme_colors` todo
- `test_bc_2_01_001_missing_slot_emits_warning` — hits `parse_theme_colors` todo
- `test_bc_2_01_001_color_order_preserved` — hits `parse_theme_colors` todo
- `test_bc_2_01_001_empty_theme_all_missing` — hits `parse_theme_colors` todo
- `test_bc_2_01_001_hex_values_uppercase_normalized` — hits `parse_theme_colors` todo

### `font.rs` (4 failing)
- `test_bc_2_01_001_parse_major_minor_fonts` — hits `parse_theme_fonts` todo
- `test_bc_2_01_001_missing_fonts_fallback` — hits `parse_theme_fonts` todo
- `test_bc_2_01_001_parse_arial_font` — hits `parse_theme_fonts` todo
- `test_bc_2_01_006_font_available_does_not_panic` — hits `font_available` todo

### `loader.rs` (8 failing)
- `test_bc_2_01_001_load_valid_pptx` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_load_valid_docx` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_load_missing_file` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_load_corrupt_file` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_load_partial_colors` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_load_no_logo` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_001_invariant_source_file_unmodified` — hits `BrandLoader::load_template` todo
- `test_bc_2_01_006_font_unavailable_is_cosmetic` — hits `BrandLoader::load_template` todo

## Dependency Notes

- `zip = "=2.6.1"` (story spec) is **yanked** on crates.io. Used `=2.3.0` instead
  (latest non-yanked 2.x release). API-compatible — same `ZipArchive`, `ZipWriter`,
  `SimpleFileOptions` public API surface.
- `quick-xml = "=0.36.2"` (story spec) — only `0.36.0` exists in that minor series.
  Used `=0.36.0`. API-compatible.
- `dirs = "=5.0.1"` added for `font_search_dirs()` platform-specific home directory
  detection. This was implied by the story spec's `font_available` implementation
  notes but not listed as a dependency. Added correctly per production-grade default.

## Files Created

- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/Cargo.toml`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/lib.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/error.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/template.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/context.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/color.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/font.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/logo.rs`
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-022/crates/slideforge-brand/src/loader.rs`
- Root `Cargo.toml` updated (added `slideforge-brand` to workspace members)

## Hand-off to Implementer

Make each failing test pass, one at a time, with minimum code:

### Priority 1: `color.rs` — `parse_theme_colors`
SAX-parse `theme1.xml` bytes using `quick-xml::Reader`. Walk `<a:clrScheme>` children.
For each recognized slot name (`dk1`, `lt1`, etc.): check for `<a:srgbClr val="...">` (use hex directly)
or `<a:sysClr lastClr="...">` (use lastClr). Normalize to `#RRGGBB`. Track which slots were found;
for each missing slot emit `BrandError::MissingColorSlot` and fill with a placeholder value.

### Priority 2: `font.rs` — `parse_theme_fonts` + `font_available`
SAX-parse `theme1.xml` for `<a:majorFont>` and `<a:minorFont>`, extract `typeface` attributes.
Fallback to `"Calibri"` if absent. For `font_available`: use `font_search_dirs()` and scan for
any file whose stem matches the font name (case-insensitive).

### Priority 3: `loader.rs` — `BrandLoader::load_template`
1. Check file exists → `BrandError::FileNotFound` if not.
2. Open as ZIP → `BrandError::ParseError` if not a valid ZIP.
3. Detect PPTX vs DOCX by checking `PPTX_THEME_PATH` / `DOCX_THEME_PATH` in ZIP entries.
4. Read theme XML bytes from the ZIP entry.
5. Call `parse_theme_colors`, `parse_theme_fonts`, `extract_logo` (PPTX only).
6. Emit `tracing::warn!` for each `BrandError::MissingColorSlot` warning.
7. If `ctx.check_font_availability`: call `font_available()` for each font, emit
   `BrandError::FontUnavailable` warnings but continue.
8. Construct and return `BrandTemplate`.
