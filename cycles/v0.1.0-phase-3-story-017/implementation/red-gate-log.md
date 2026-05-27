# Red Gate Log — STORY-017: Color-Coded Label + WCAG Contrast Enforcement

**Date:** 2026-05-26
**Worktree:** `.worktrees/STORY-017` — branch `feature/S-017`

## Red Gate Result: PASS

All 43 new STORY-017 tests fail correctly via `todo!()` panics with no implementation.
The 87 existing STORY-015/016 tests continue to pass (no regression).

## Test Inventory

| Module | Test Count | Status | Red Gate Method |
|--------|-----------|--------|-----------------|
| `wcag` | 17 | FAIL | `todo!()` panic on all 5 functions |
| `label_check` | 16 | FAIL | `todo!()` panic on `validate()` |
| `lang_validator` | 12 | FAIL | `todo!()` panic on `validate()` and `inject_lang_default()` |
| **STORY-017 total** | **43** | **FAIL** | |
| `alt_text` (STORY-015) | 30 | PASS | pre-existing implementation |
| `canvas_overflow` (STORY-016) | 17 | PASS | pre-existing implementation |
| `error_slide` (STORY-016) | 11 | PASS | pre-existing implementation |
| `mode` (STORY-016) | 5 | PASS | pre-existing implementation |
| `utils` (STORY-015) | 8 | PASS | pre-existing implementation |
| `zero_slide` (STORY-016) | 5 | PASS | pre-existing implementation |
| **Total** | **130** | 87 ok / 43 FAIL | |

### Passing STORY-017 tests (correct — not vacuously true)

3 tests from the new modules pass before implementation:

| Test | Module | Reason |
|------|--------|--------|
| `test_BC_5_01_003_color_coded_types_const` | `label_check` | Tests the `COLOR_CODED_TYPES` const data — no behavior |
| `test_BC_5_01_003_validator_id` | `label_check` | `id()` returns a `'static str` literal directly |
| `test_BC_5_01_004_validator_id` | `lang_validator` | `id()` returns a `'static str` literal directly |

These are not vacuously true — they verify scaffolding data that the implementer must not change. If the implementer renames a validator ID or removes a type from `COLOR_CODED_TYPES`, these tests catch the regression immediately.

## Cargo Build

```
cargo test -p slideforge-validate --no-run   → OK (0 errors, 0 warnings)
cargo test -p slideforge-validate --no-fail-fast
  → 87 passed, 43 failed (all failures via todo!() panics)
```

## Test Files

- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-017/crates/slideforge-validate/src/wcag.rs`
  — 17 tests covering sRGB linearization, relative luminance, contrast ratio, hex parsing,
    and `wcag_aa_passes` threshold logic
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-017/crates/slideforge-validate/src/label_check.rs`
  — 16 tests covering all 4 color-coded slide types, empty/whitespace labels, decorative
    exemption (negative test), multiple missing labels, contrast warnings, adequate contrast
- `/Users/jmagady/Dev/slideforge/.worktrees/STORY-017/crates/slideforge-validate/src/lang_validator.rs`
  — 12 tests covering missing/empty/whitespace lang, present lang, BCP-47 preservation
    (zh-Hant-TW), inject_lang_default mutation, noop on valid lang

## BC Coverage

| BC | Clause | Test Names |
|----|--------|-----------|
| BC-5.01.003 | PC1: severity_cards missing label → E-A11-002 | `test_BC_5_01_003_severity_cards_missing_label` |
| BC-5.01.003 | PC1: status/progress_bar/weighted_composite | `test_BC_5_01_003_{status,progress_bar,weighted_composite}_missing_label` |
| BC-5.01.003 | PC2: AC-001 error message quality | Covered by assert on code/severity |
| BC-5.01.003 | PC3: accumulation (2 of 3 missing) | `test_BC_5_01_003_multiple_missing_labels` |
| BC-5.01.003 | INV1: decorative does NOT exempt | `test_BC_5_01_003_decorative_does_not_exempt_label` |
| BC-5.01.003 | INV2: empty string treated as missing | `test_BC_5_01_003_label_empty_string` |
| BC-5.01.003 | INV2: whitespace treated as missing | `test_BC_5_01_003_label_whitespace` |
| BC-5.01.003 | WCAG 1.4.3: contrast warning | `test_BC_5_01_003_low_contrast_warning`, `test_BC_5_01_003_adequate_contrast_no_warning` |
| BC-5.01.003 | EC-009: both errors together | `test_BC_5_01_003_low_contrast_and_missing_label` |
| VP-007 | sRGB linearization | `test_BC_5_01_003_srgb_to_linear_{low,mid,max}` |
| VP-007 | Relative luminance | `test_BC_5_01_003_luminance_{white,black}` |
| VP-007 | Contrast ratio | `test_BC_5_01_003_contrast_{black_white,ratio_4_5,ratio_4_4,large_text_3_0}` |
| VP-007 | Hex parsing | `test_BC_5_01_003_parse_hex_{valid,invalid,black,white,shorthand_invalid,empty_invalid}` |
| VP-007 | AA threshold | `test_BC_5_01_003_wcag_aa_{black_on_white_normal,black_on_white_large,red_on_white_fails_normal,dark_red_on_white_passes_normal}` |
| BC-5.01.004 | PC1: missing lang → E-A11-003 Info | `test_BC_5_01_004_missing_lang_produces_warning` |
| BC-5.01.004 | PC4: cosmetic severity (never blocking) | `test_BC_5_01_004_lang_not_blocking` |
| BC-5.01.004 | INV3: empty string → E-A11-003 | `test_BC_5_01_004_lang_empty_string` |
| BC-5.01.004 | inject_lang_default sets "en" | `test_BC_5_01_004_inject_lang_default_sets_en_when_{absent,empty}` |
| BC-5.01.005 | PC1: lang propagates unchanged | `test_BC_5_01_005_lang_zh_hant_tw_unchanged` |
| BC-5.01.005 | INV1: no normalisation | `test_BC_5_01_005_inject_lang_default_noop_{when_present,zh_hant_tw}` |

## Handoff Instructions for Implementer

Make each test pass one at a time with minimum code:

1. **`wcag.rs`** — 5 pure functions, no dependencies. Implement in order:
   - `srgb_component_to_linear` (gamma expansion formula)
   - `relative_luminance` (weighted sum of linearized channels)
   - `contrast_ratio` ((lighter + 0.05) / (darker + 0.05))
   - `parse_hex_color` (parse `#RRGGBB` → `Option<(u8, u8, u8)>`)
   - `wcag_aa_passes` (threshold check: 4.5 normal, 3.0 large)
   Run: `cargo nextest run -p slideforge-validate -E 'test(wcag::tests)'`

2. **`lang_validator.rs`** — Implement `inject_lang_default` first (pure mutation),
   then `LangValidator::validate` (diagnostic emission). Two separate concerns.
   Run: `cargo nextest run -p slideforge-validate -E 'test(lang_validator::tests)'`

3. **`label_check.rs`** — Depends on `wcag.rs` being complete.
   Implement `make_missing_label_error`, `make_low_contrast_warning`, then
   `LabelCheckValidator::validate`. Walk `deck.slides`, check `slide_type` against
   `COLOR_CODED_TYPES`, look up `slide.fields["label"]`, run contrast check when
   hex colors are present.
   Run: `cargo nextest run -p slideforge-validate -E 'test(label_check::tests)'`

4. **Final gate:** `cargo test -p slideforge-validate --no-fail-fast`
   → All 130 tests must pass.

## Architecture Notes for Implementer

- `LangValidator::validate` emits `DiagnosticSeverity::Info` (not `Warning`) for E-A11-003.
  This maps to the "cosmetic" severity in the story spec — never blocks export.
- `inject_lang_default` takes `&mut Deck` and returns `bool` (was injected?).
  Called BEFORE the trait dispatch loop by the pipeline dispatcher.
- Contrast check in `LabelCheckValidator`: only runs when BOTH `fg_color` and `bg_color`
  fields are present in `slide.fields` as hex strings. Brand palette references skip.
- `decorative: true` does NOT exempt from label check (BC-5.01.003 invariant 1).
  The `decorative` boolean field is consulted in `alt_text` but ignored by `label_check`.
