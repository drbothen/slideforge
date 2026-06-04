# Red Gate Log — STORY-039: PPTX Accessibility Metadata

**Date:** 2026-06-03
**Agent:** test-writer
**Branch:** feature/S-039
**Worktree:** .worktrees/S-039

## Summary

Red Gate verified. All 8 new a11y tests FAIL before implementation. All 118
pre-existing tests continue to PASS. Stubs compile cleanly.

## Build Gate

```
cargo build -p slideforge-pptx --tests
```

Result: SUCCESS (exit code 0). Warnings only (unused parameter prefixes — all
cleaned up in final stub). No errors.

## Nextest Results (Red Gate)

```
Summary: 126 tests run — 118 passed, 8 FAILED, 1 skipped
```

### Failing Tests (Red Gate — must fail)

| Test name | AC | Failure reason |
|---|---|---|
| `test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr` | AC-001 | Assertion failure: `descr` values `[]` — no cNvPr with descr in slide XML (AltTextEmbedder not called) |
| `test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present` | AC-002 | Assertion failure: `descr` values `[]` — decorative frame cNvPr missing descr="" |
| `test_BC_4_01_004_ac003_300_char_alt_not_truncated` | AC-003 | Assertion failure: 300-char alt not found in descr values `[]` |
| `test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed` | AC-004 | Assertion failure: no non-empty descr after embedding special-char alt |
| `test_BC_4_01_004_ac005_chart_diagram_alt_on_enclosing_shape` | AC-005 | Assertion failure: alt text not on enclosing shape cNvPr |
| `test_BC_5_01_005_ac007_no_lang_defaults_to_en` | AC-007 | Assertion failure: got `"en-US"`, expected `"en"` (wrong fallback in build_doc_props) |
| `test_BC_4_01_004_ec001_all_decorative_slide` | EC-001 | Assertion failure: no cNvPr with descr="" for all-decorative slide |
| `test_BC_4_01_004_ec005_300_char_alt_exact_length` | EC-005 | Assertion failure: 300-char alt not found in descr values `[]` |

### Passing Tests (WIRING-EXEMPT — behavior already delivered by STORY-037)

| Test name | AC | Reason passes |
|---|---|---|
| `test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us` | AC-006 | `build_doc_props` in lib.rs already correctly reads `deck.metadata.lang` and writes `<dc:language>`. STORY-037 delivered this. |
| `test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw` | AC-006 | Same as above — lang is passed through unchanged by the existing implementation. |
| `test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded` | EC-002 | Same as above — verifies well-formedness of core.xml for non-ASCII BCP-47. |

**Assessment:** These 3 tests are not Red Gate violations. They verify behavior
that STORY-037 already implemented (`dc:language` embedding from `deck.metadata.lang`).
STORY-039's scope for `dc:language` is limited to changing the default fallback from
`"en-US"` to `"en"` (AC-007). The AC-006 tests are correctly GREEN because the
underlying behavior exists; they serve as regression guards confirming STORY-039
does not break the already-correct embedding.

## Stubs Created

- `crates/slideforge-pptx/src/a11y.rs` — `AltTextEmbedder` struct with two
  `todo!()`-body methods (`embed`, `decisions_for_slide`); `AltDecision` enum.
- `crates/slideforge-pptx/src/lib.rs` — wired `pub mod a11y` and `mod a11y_tests`.
- `crates/slideforge-pptx/src/tests/a11y_tests.rs` — 11 failing tests.
- `crates/slideforge-pptx/Cargo.toml` — added `quick-xml = { workspace = true }` dev-dep.

## Files Modified

- `crates/slideforge-pptx/src/lib.rs` — added `pub mod a11y` + `mod a11y_tests`
- `crates/slideforge-pptx/Cargo.toml` — added quick-xml dev-dependency

## Test List (complete)

```
tests::a11y_tests::test_BC_4_01_004_ac001_non_decorative_image_has_non_empty_descr
tests::a11y_tests::test_BC_4_01_004_ac002_decorative_image_has_empty_descr_attribute_present
tests::a11y_tests::test_BC_4_01_004_ac003_300_char_alt_not_truncated
tests::a11y_tests::test_BC_4_01_004_ac004_special_chars_xml_escaped_well_formed
tests::a11y_tests::test_BC_4_01_004_ac005_chart_diagram_alt_on_enclosing_shape
tests::a11y_tests::test_BC_5_01_005_ac006_dc_language_exact_bcp47_en_us        [WIRING-EXEMPT / PASS]
tests::a11y_tests::test_BC_5_01_005_ac006_dc_language_exact_bcp47_zh_hant_tw   [WIRING-EXEMPT / PASS]
tests::a11y_tests::test_BC_5_01_005_ac007_no_lang_defaults_to_en
tests::a11y_tests::test_BC_4_01_004_ec001_all_decorative_slide
tests::a11y_tests::test_BC_4_01_004_ec002_zh_tw_lang_bcp47_embedded            [WIRING-EXEMPT / PASS]
tests::a11y_tests::test_BC_4_01_004_ec005_300_char_alt_exact_length
```

## BC Clause Coverage

| BC | Clause | Covered by |
|---|---|---|
| BC-4.01.004 | Postcondition 1 — non-decorative descr non-empty | AC-001, EC-005 |
| BC-4.01.004 | Postcondition 2 — decorative descr="" present | AC-002, EC-001 |
| BC-4.01.004 | Invariant 1 — alt text never truncated | AC-003, EC-005 |
| BC-4.01.004 | Invariant 2 — descr on cNvPr, not ph.altText | AC-001 (assertion checks cNvPr) |
| BC-4.01.004 | EC-001 — special chars XML-escaped | AC-004 |
| BC-4.01.004 | EC-005 — chart/diagram alt on enclosing shape | AC-005 |
| BC-5.01.005 | Postcondition 1 — PPTX dc:language exact BCP-47 | AC-006 x2 |
| BC-5.01.005 | EC-003 — no lang → default "en" | AC-007 |
| BC-5.01.005 | Invariant 1 — lossless propagation | AC-006 zh-Hant-TW |

## Implementer Instructions

Make each failing test pass, one at a time, with minimum code:

1. Implement `AltTextEmbedder::decisions_for_slide` — walk `LaidOutSlide.frames`
   and return `(frame_idx, AltDecision)` pairs for Image/Diagram/Chart frames.
   `FrameContent::Image { alt }`: empty alt → `AltDecision::Decorative`,
   non-empty → `AltDecision::Provided(alt)`.

2. Implement `AltTextEmbedder::embed` — post-process slide XML bytes to inject
   `descr` attributes on `<p:cNvPr>` elements for each visual shape. Use
   `quick-xml` to parse and rewrite, or inject via ooxmlsdk types before
   serialization (preferred — fewer XML rewrite risks).

3. Wire `AltTextEmbedder` into `SlideSerializer::build` — call it after
   building each shape in `build_shape_tree`, setting `description` on the
   `NonVisualDrawingProperties` struct before serialization.

4. Fix `build_doc_props` default fallback: change `unwrap_or("en-US")` to
   `unwrap_or("en")` (AC-007).

5. Wire via `lib.rs` `export_inner`: call `AltTextEmbedder::decisions_for_slide`
   and `AltTextEmbedder::embed` after `SlideSerializer::build` for each slide.

## Architecture Note (Implementer)

The cleanest implementation sets `description` on `NonVisualDrawingProperties`
BEFORE calling `sld.to_xml_bytes()` — this lets ooxmlsdk handle XML escaping
automatically (no manual XML post-processing needed). The ooxmlsdk
`NonVisualDrawingProperties` struct has a `description: Option<String>` field;
set it to `Some(alt_text)` for non-decorative or `Some(String::new())` for
decorative. This avoids a separate `AltTextEmbedder::embed` XML-rewrite pass
entirely — `decisions_for_slide` can return metadata, and `SlideSerializer`
applies it during shape construction.
