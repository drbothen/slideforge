---
story_id: STORY-036
phase: red-gate
timestamp: 2026-05-31
agent: test-writer
---

# Red Gate Log — STORY-036

## Summary

STORY-036 Red Gate is SATISFIED. All AC-008/EC tests that exercise BleedChecker
methods fail with `todo!()` panics. Ignored exporter-dependent tests are
correctly skipped. Default build (no feature) is clean.

## Spec Reconciliations

### 1. Feature gate: `test-utils` Cargo feature (not `#[cfg(test)]`)

The story spec mentions both `#[cfg(test)]` and a `test-utils` feature. The
spec reconciliation instruction resolves this: `BleedChecker` MUST be gated
behind `#[cfg(feature = "test-utils")]` because `#[cfg(test)]` items are not
visible to external crates. STORY-037 and STORY-041 exporter crates must be
able to import `BleedChecker` via dev-dependencies with the feature enabled.

Decision: Used `#[cfg(feature = "test-utils")]` on the entire `bleed_check`
module and re-exported `BleedChecker` from `lib.rs` under the same gate.

### 2. `zip` dependency version: `=2.3.0` (not `=4.2.0`)

The story spec cites `zip = "=4.2.0"` but that version does not exist on
crates.io. The Cargo.lock contains `zip 2.3.0` (used by `slideforge-brand`
and `calamine`). Using `=2.3.0` reuses the already-resolved lock entry with
zero additional download.

Decision: Used `zip = { version = "=2.3.0", optional = true }` under
`[dependencies]` (optional, gated on `test-utils` feature) AND `zip = "=2.3.0"`
under `[dev-dependencies]` so that integration tests in `tests/bleed_tests.rs`
can use `zip::ZipWriter` directly to construct synthetic ZIP fixtures.

### 3. STORY-035 types used

The actual types from STORY-035 on `develop` (now merged to the worktree base):

- `slideforge_types::Register` (enum: Notes/Report/Detail, Copy, Ord, Hash)
- `slideforge_types::RegisteredContent` (struct: register, content: Vec<InlineNode>)
- `slideforge_types::Slide` has `register_content: Vec<RegisteredContent>`
- `slideforge_types::Slide` has `register: Option<Register>` (slide-level gate)
- `extract_register_content(&Slide) -> Vec<RegisteredContent>` from `slideforge_eval::register_routing`

No changes to these types were needed.

## Files Created

| File | Purpose |
|------|---------|
| `crates/slideforge-eval/src/bleed_check.rs` | BleedChecker struct with 4 `todo!()` stub methods |
| `crates/slideforge-eval/tests/bleed_tests.rs` | All bleed invariant tests (AC-001 through AC-008, EC-003, EC-004, fixture parse) |
| `crates/slideforge-eval/tests/fixtures/three-register-slide.sf` | Canonical three-register fixture |

## Files Modified

| File | Change |
|------|--------|
| `crates/slideforge-eval/Cargo.toml` | Added `[features] test-utils = ["dep:zip"]`, optional `zip = "=2.3.0"` dep, dev-dep `zip = "=2.3.0"` |
| `crates/slideforge-eval/src/lib.rs` | Added `bleed_check` module and `BleedChecker` re-export under `#[cfg(feature = "test-utils")]` |

## Test Run Results

Command: `cargo nextest run -p slideforge-eval --features test-utils --no-fail-fast`

```
Summary: 237 tests run: 232 passed, 5 failed, 7 skipped
```

### Failing tests (NOW — Red Gate confirmed)

All 5 fail with `not yet implemented` (todo!() panic from BleedChecker stubs):

| Test | BC Clause | AC | Failure Reason |
|------|-----------|----|---------------|
| `test_BC_1_14_004_bleedchecker_absent_from_slides_passes_when_not_present` | invariant 1 | AC-008 | `todo!()` in `assert_absent_from_pptx_slides` |
| `test_BC_1_14_004_bleedchecker_absent_from_all_passes_when_not_present` | invariant 1 | AC-008 | `todo!()` in `assert_absent_from_pptx_all` |
| `test_BC_1_14_004_bleedchecker_present_in_docx_body_passes_when_present` | postcondition 3 | AC-008 | `todo!()` in `assert_present_in_docx_body` |
| `test_BC_1_14_004_bleedchecker_absent_from_docx_body_passes_when_not_present` | postcondition 2 | AC-008 | `todo!()` in `assert_absent_from_docx_body` |
| `test_BC_1_14_004_ec003_sentinel_in_nonscanned_path_not_flagged_as_bleed` | EC-003 | AC-008 | `todo!()` in `assert_absent_from_pptx_slides` |

### Passing tests in bleed_tests.rs (expected at Red Gate)

These pass correctly because they use `#[should_panic]` or `std::panic::catch_unwind`:

| Test | Why it passes at Red Gate |
|------|--------------------------|
| `test_BC_1_14_004_bleedchecker_absent_from_slides_panics_when_present` | `#[should_panic]` — `todo!()` satisfies the panic expectation |
| `test_BC_1_14_004_bleedchecker_absent_from_all_panics_when_present` | `#[should_panic]` — same |
| `test_BC_1_14_004_bleedchecker_present_in_docx_body_panics_when_absent` | `#[should_panic]` — same |
| `test_BC_1_14_004_bleedchecker_absent_from_docx_body_panics_when_present` | `#[should_panic]` — same |
| `test_BC_1_14_004_ec004_sentinel_detected_in_decoded_text` | `catch_unwind` — verifies panic occurs (which it does via todo!()) |
| `test_BC_1_14_004_fixture_parses_without_errors` | Fixture parses correctly through real parser |

Note: `#[should_panic]` tests passing at Red Gate is expected and correct —
once the implementation lands, they must continue to pass (the methods must
still panic when an invariant is violated).

### Skipped (ignored) tests — 7

All 7 are exporter-dependent ACs waiting for STORY-037/041/046:

| Test | Ignore Reason |
|------|--------------|
| `test_BC_1_14_004_ac001_notes_absent_from_pptx_slide_body` | requires STORY-037 PPTX exporter |
| `test_BC_1_14_004_ac002_report_absent_from_pptx_slide_body` | requires STORY-037 PPTX exporter |
| `test_BC_1_14_004_ac003_detail_absent_from_pptx_all_parts` | requires STORY-037 PPTX exporter |
| `test_BC_1_14_004_ac004_notes_absent_from_docx_body` | requires STORY-041 DOCX exporter |
| `test_BC_1_14_004_ac005_report_present_in_docx_body` | requires STORY-041 DOCX exporter |
| `test_BC_1_14_004_ac006_detail_absent_from_html_canvas` | requires STORY-046 HTML exporter |
| `test_BC_1_14_004_ac007_all_three_registers_no_bleed_cross_format` | requires STORY-037 PPTX exporter |

## Compile Results

```
cargo build -p slideforge-eval --features test-utils  →  COMPILES (with expected unused-var warnings on todo!() stubs)
cargo build -p slideforge-eval                        →  COMPILES (clean, zero warnings)
```

Warnings on the feature build are expected: unused parameter names in `todo!()`
stub bodies. These will be resolved when the implementer fills in the stubs.

## Red Gate Verdict

RED GATE: SATISFIED

- 5 NOW tests fail (todo!() panics from BleedChecker stubs)
- 7 exporter tests skipped (ignored, compile correctly)
- 232 pre-existing tests continue to pass (no regressions)
- Default build clean
- `zip` is optional+feature-gated (not a hard production dep)
- `slideforge-eval` has NO exporter-crate production deps (confirmed: only `slideforge-types`, `slideforge-syntax`, `thiserror`, `miette`, `indexmap`, `ordered-float`, `tracing`, `zip` optional)
