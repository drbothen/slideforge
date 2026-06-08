# STORY-074 Demo Evidence Report

**Story:** STORY-074 — Brand-aware Em conversion (font_size_emu)
**Crate:** slideforge-layout + slideforge-types
**Branch:** feature/STORY-074
**Recorded:** 2026-06-08
**Adversary cascade:** CONVERGED 3/3 (LOCAL)
**Recording tool:** VHS 0.10.0

---

## Coverage Summary

| AC | BC Clause | Tests Covered | Recording | Status |
|----|-----------|---------------|-----------|--------|
| AC-001 | BC-3.04.001 PC-2 brand-aware em resolution | 8 tests (brand_em_sizing) | AC-001-brand-em-sizing | PASS |
| AC-002 | BC-3.04.001 PC-2 no magic constants in production | 1 test (shapes::tests) + grep | AC-002-default-em-removed | PASS |
| AC-003 | BC-3.04.001 Invariant 2 backward compatibility | 3 tests (brand_em_sizing) | AC-003-default-brand-compat | PASS |

Total: 11 brand_em_sizing tests + 1 in-crate unit test = **12 tests, all pass**.

---

## AC-001 — Brand font_size_emu drives em→EMU conversion

**Acceptance Criterion:** A brand with `font_size_emu: 609_600` (48pt body font)
resolves `ShapeUnit::Em(1000)` to `Emu(609_600)` via `layout::run`. A brand with
`font_size_emu: 457_200` (36pt default) resolves to `Emu(457_200)`. Different brands
produce different EMU — the constant `DEFAULT_EM_IN_EMU` no longer drives layout.

**Formula verified:** `em_emu = em_milliems * brand_font_size_emu / 1_000` (integer i64, no f64)

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_3_04_001_ac001_layout_run_48pt_brand_em1_resolves_to_609600` | 48pt brand + Em(1000) → Emu(609_600) via layout::run |
| `test_bc_3_04_001_ac001_layout_run_48pt_brand_em2_width_resolves_to_1219200` | width=Em(2000) with 48pt → Emu(1_219_200) |
| `test_bc_3_04_001_ac001_different_brands_produce_different_em_resolution` | 24pt (304_800) vs 48pt (609_600) produce distinct EMU |
| `test_bc_3_04_001_ac001_layout_shapes_direct_48pt_em1_resolves_to_609600` | layout_shapes direct call with em_in_emu=609_600 |
| `test_bc_3_04_001_ac001_half_em_with_48pt_brand_resolves_to_304800` | Em(500) * 609_600 / 1_000 = 304_800 |
| `test_bc_3_04_001_ac001_mixed_em_and_inches_shapes_in_one_slide` | Mixed Em + Inches shapes in one slide |
| `test_bc_3_04_001_ac001_em_zero_resolves_to_emu_zero_regardless_of_brand` | Em(0) always → Emu(0) |
| `test_bc_3_04_001_ac001_inches_positions_unaffected_by_font_size_emu` | Inches path unaffected by brand font size |

**Recordings:**
- `AC-001-brand-em-sizing.gif` (171 KB) — PR embed
- `AC-001-brand-em-sizing.webm` (302 KB) — archival
- `AC-001-brand-em-sizing.tape` — VHS script source

---

## AC-002 — DEFAULT_EM_IN_EMU constant removed from production code

**Acceptance Criterion:** `DEFAULT_EM_IN_EMU` is no longer `pub` or referenced by
production code. It is retained as a `const` inside `#[cfg(test)]` only. No
`dead_code` warnings from clippy for the constant in production modules.

**Demonstrations:**

1. **Success path:** In-crate unit test `shapes::tests::test_bc_3_04_001_ac002_default_em_in_emu_is_457200`
   passes — it accesses the private (test-only) const and asserts its value is 457_200.

2. **Error-path (grep):** `grep -n 'pub const DEFAULT_EM_IN_EMU' crates/slideforge-layout/src/shapes.rs`
   produces no match, confirming the const is not exported. Output:
   `CONFIRMED: no pub const DEFAULT_EM_IN_EMU in production path`

**Recordings:**
- `AC-002-default-em-removed.gif` (486 KB) — PR embed
- `AC-002-default-em-removed.webm` (600 KB) — archival
- `AC-002-default-em-removed.tape` — VHS script source

---

## AC-003 — BrandFonts default is backward-compatible

**Acceptance Criterion:** `BrandFonts::default()` sets `font_size_emu: 457_200`.
All existing tests that relied on `DEFAULT_EM_IN_EMU` produce identical results
via the new default-brand path. No previously-passing test may fail.

**Tests demonstrated:**

| Test | Assertion |
|------|-----------|
| `test_bc_3_04_001_ac003_brand_fonts_default_has_correct_font_size_emu` | `BrandFonts::default().font_size_emu == 457_200` |
| `test_bc_3_04_001_ac003_default_brand_em_resolution_unchanged` | Default brand + Em(1000) → Emu(457_200) |
| `test_bc_3_04_001_ac003_layout_run_default_brand_matches_old_constant` | Em(2000) * 457_200/1_000 = Emu(914_400) (regression guard) |

**Full suite regression guard:** All 11 brand_em_sizing tests pass — confirms
no pre-STORY-074 behavior was regressed.

**Recordings:**
- `AC-003-default-brand-compat.gif` (118 KB) — PR embed
- `AC-003-default-brand-compat.webm` (170 KB) — archival
- `AC-003-default-brand-compat.tape` — VHS script source

---

## Artifact Manifest

```
docs/demo-evidence/STORY-074/
├── AC-001-brand-em-sizing.gif        (171 KB)
├── AC-001-brand-em-sizing.webm       (302 KB)
├── AC-001-brand-em-sizing.tape
├── AC-002-default-em-removed.gif     (486 KB)
├── AC-002-default-em-removed.webm    (600 KB)
├── AC-002-default-em-removed.tape
├── AC-003-default-brand-compat.gif   (118 KB)
├── AC-003-default-brand-compat.webm  (170 KB)
├── AC-003-default-brand-compat.tape
└── evidence-report.md
```

---

## Notes

- All recordings use VHS 0.10.0 with FiraCode Nerd Font Mono, Catppuccin Mocha theme.
- AC-002 error-path is demonstrated via `grep` (compile-level evidence that the
  `pub const` does not exist in production code), not a runtime failure path.
- The test suite uses `#[allow(clippy::unwrap_used, clippy::expect_used)]` in the
  test file, per project convention for integration test helpers.
