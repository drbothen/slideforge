# Evidence Report: STORY-036 — Register-Aware Rendering: No Content Bleed Invariant

**Story ID:** STORY-036
**Epic:** EPIC-18
**Crate:** `slideforge-eval` (BleedChecker utility + bleed_tests integration target)
**BC Trace:** BC-1.14.004 (all postconditions)
**HEAD SHA verified:** `feature/S-036` branch
**Workspace test count (slideforge-eval):** 275 tests run: 275 passed, 7 skipped
**Recording tool:** VHS 0.10.0 (terminal — library story, nextest harness demos)
**Font:** FiraCode Nerd Font Mono

> **Library-only story note:** STORY-036 delivers the `BleedChecker` test utility behind
> the `test-utils` Cargo feature, a canonical three-register fixture deck, and bleed
> invariant integration tests. The exporters (PPTX/DOCX/PDF/HTML) do not yet exist, so
> AC-001..AC-007 (build-to-exporter bleed tests) are `#[ignore]`'d and cannot be demo'd.
> All demos invoke the load-bearing-NOW deliverables via
> `cargo nextest run -p slideforge-eval --features test-utils`. This follows the
> established pattern for library stories (see STORY-035 for precedent).

---

## Coverage Summary

Four demos cover all load-bearing deliverables available before exporters exist.
AC-001..AC-007 are fully written and compile but are `#[ignore]`'d pending STORY-037/041/046.

| Demo | Description | Demo File | Tests Covered | Status |
|------|-------------|-----------|--------------|--------|
| AC-008 | BleedChecker 4-method utility: pass direction + panic direction (8 bidirectional tests) | `AC-008-bleedchecker-utility` | 8 `test_BC_1_14_004_bleedchecker_*` tests | PASSED |
| EC-003 | Path scoping: sentinel in `ppt/notesSlides/` and `ppt/theme/` NOT flagged by slide-body check | `EC-003-path-scoping` | `test_BC_1_14_004_ec003_*` (2 tests) | PASSED |
| EC-004 | XML-entity decoding: `R&amp;D` detected as `R&D`; unknown `&copy;` maps to U+FFFD (no false pos/neg); malformed `&#xZZ;` does not mask adjacent sentinel | `EC-004-xml-entity-decoding` | `test_BC_1_14_004_ec004_*` (4 tests) | PASSED |
| FIXTURE-PARSE | `three-register-slide.sf` parses with zero fatal errors; exactly 1 slide item produced | `FIXTURE-PARSE-three-register-slide` | `test_BC_1_14_004_fixture_parses_without_errors` (1 test) | PASSED |

---

## AC-008: BleedChecker Utility — 4 Methods, Both Directions

**File:** `AC-008-bleedchecker-utility.{tape,gif,webm}`
**BC trace:** BC-1.14.004 invariant 1 — no register content crosses format boundaries
**Nextest filter:** `binary(bleed_tests) & test(bleedchecker)` — 8 tests

The `BleedChecker` struct is implemented in `crates/slideforge-eval/src/bleed_check.rs` behind
the `test-utils` Cargo feature. It exposes four `pub` static methods:

```rust
fn assert_absent_from_pptx_slides(pptx_bytes: &[u8], sentinel: &str)
fn assert_absent_from_pptx_all(pptx_bytes: &[u8], sentinel: &str)
fn assert_present_in_docx_body(docx_bytes: &[u8], sentinel: &str)
fn assert_absent_from_docx_body(docx_bytes: &[u8], sentinel: &str)
```

The tests construct minimal synthetic PPTX-like and DOCX-like ZIP bytes in-memory using
the `zip` crate. Each of the 4 methods is exercised in BOTH directions:

- **Pass direction** (no panic expected): sentinel absent from scanned paths / sentinel
  present in expected path. The test completes without panic.
- **Panic direction** (`#[should_panic]`): sentinel present in disallowed path (absence
  check) or absent from required path (presence check). The test must panic with the
  correct message, proving the checker detects violations.

The 8 method-direction pairs are:

| Test | Method | Direction | Panic message (if applicable) |
|------|--------|-----------|-------------------------------|
| `absent_from_slides_passes_when_not_present` | `assert_absent_from_pptx_slides` | Pass | — |
| `absent_from_slides_panics_when_present` | `assert_absent_from_pptx_slides` | Panic | `BleedChecker: register-content bleed detected in PPTX slide body` |
| `absent_from_all_passes_when_not_present` | `assert_absent_from_pptx_all` | Pass | — |
| `absent_from_all_panics_when_present` | `assert_absent_from_pptx_all` | Panic | `BleedChecker: register-content bleed detected in PPTX archive` |
| `present_in_docx_body_passes_when_present` | `assert_present_in_docx_body` | Pass | — |
| `present_in_docx_body_panics_when_absent` | `assert_present_in_docx_body` | Panic | `BleedChecker: expected register content NOT found in DOCX body` |
| `absent_from_docx_body_passes_when_not_present` | `assert_absent_from_docx_body` | Pass | — |
| `absent_from_docx_body_panics_when_present` | `assert_absent_from_docx_body` | Panic | `BleedChecker: register-content bleed detected in DOCX body` |

---

## EC-003: Path Scoping — Non-Slide Paths Not Flagged

**File:** `EC-003-path-scoping.{tape,gif,webm}`
**BC trace:** BC-1.14.004 EC-003 — BleedChecker checks specifically-scoped ZIP paths
**Nextest filter:** `binary(bleed_tests) & test(ec003)` — 2 tests

`assert_absent_from_pptx_slides` scans ONLY `ppt/slides/slide*.xml`. A sentinel in any
other ZIP member — including `ppt/notesSlides/notesSlide1.xml` (the spec's canonical
EC-003 scenario) and `ppt/theme/theme1.xml` — must not trigger a false positive.

Two tests cover the two out-of-scope paths:

- `test_BC_1_14_004_ec003_sentinel_in_nonscanned_path_not_flagged_as_bleed` — sentinel
  in `ppt/theme/theme1.xml`; slide body is clean; must not panic.
- `test_BC_1_14_004_ec003_sentinel_in_notes_slide_not_flagged_as_bleed` — sentinel in
  `ppt/notesSlides/notesSlide1.xml` (legitimate notes placement); must not panic.

Both tests use `make_pptx_zip_with_extra` / `make_pptx_zip` helpers that construct
minimal in-memory ZIPs with precisely controlled member paths.

---

## EC-004: XML-Entity Decoding — No False Negatives or False Positives

**File:** `EC-004-xml-entity-decoding.{tape,gif,webm}`
**BC trace:** BC-1.14.004 EC-004 — BleedChecker works on decoded XML text, not raw bytes
**Nextest filter:** `binary(bleed_tests) & test(ec004)` — 4 tests

The `BleedChecker` uses a per-token tolerant decoder (`decode_xml_entities`) that:

1. Decodes predefined entities (`&amp;` → `&`, `&lt;` → `<`, `&gt;` → `>`, `&quot;` → `"`, `&apos;` → `'`)
2. Decodes decimal and hex numeric character references (`&#65;` → `A`, `&#x41;` → `A`)
3. Maps unknown named entities (`&copy;`, `&reg;`) to U+FFFD — a non-joining placeholder
   that prevents false joins like `A&copy;B` → `AB`
4. Maps malformed numeric refs (`&#xZZ;`) to U+FFFD — processing continues for remaining tokens

The four tests cover:

- `test_BC_1_14_004_ec004_xml_escaped_content_detected` — slide body contains `R&amp;D roadmap`
  (escaped); sentinel is `R&D`; both `assert_absent_from_pptx_all` and
  `assert_absent_from_pptx_slides` must PANIC (bleed detected after decoding). A raw-bytes
  search would produce a false negative.
- `test_BC_1_14_004_ec004_lt_gt_escaped_content_detected` — slide body contains `&lt;item&gt;`;
  sentinel is `<item>`; `assert_absent_from_pptx_slides` must PANIC.
- `test_BC_1_14_004_ec004_unescaped_sentinel_absent_passes` — slide body contains `R&amp;D roadmap`
  but the absence check is for `DETAIL_SENTINEL` (a different string); must NOT panic (no false
  positive from decoding).
- `test_BC_1_14_004_ec004_unknown_entity_does_not_mask_adjacent_sentinel` — slide body contains
  `&copy; R&amp;D roadmap` (unknown entity adjacent to escaped sentinel); must still detect
  `R&D roadmap` (the U+FFFD placeholder for `&copy;` does not obstruct the search).

---

## FIXTURE-PARSE: Canonical three-register-slide.sf Parses Cleanly

**File:** `FIXTURE-PARSE-three-register-slide.{tape,gif,webm}`
**BC trace:** BC-1.14.004 invariant 2 — routing rules determined at Evaluate stage
**Nextest filter:** `test(fixture_parses)` — 1 test

The canonical fixture at `crates/slideforge-eval/tests/fixtures/three-register-slide.sf`:

```
slideforge_version "1"
lang "en-US"
brand "default.pptx"

slide content:
  title "SENTINEL_TITLE"
  notes "SENTINEL_NOTES"
  report "SENTINEL_REPORT"
  detail "SENTINEL_DETAIL"
```

The test `test_BC_1_14_004_fixture_parses_without_errors` drives the real
`slideforge-syntax` parser (`parse_checked`) on this fixture. Assertions:

1. `parse_checked` returns `Some(deck_node)` (no fatal parse failure)
2. `sink.has_fatal()` is false (zero fatal-severity diagnostics)
3. The parsed `DeckNode.items` contains exactly 1 `BlockItem::Slide` (one slide, no
   control flow items)

The test gates on `has_fatal()` rather than `errors().is_empty()` so that benign warnings
(e.g., missing brand file at parse time) do not cause spurious failures.

---

## Deferred Evidence (Exporter-Dependent ACs)

AC-001 through AC-007 (build-to-exporter bleed tests) are fully written and compile
in `crates/slideforge-eval/tests/bleed_tests.rs` but carry `#[ignore]` annotations.
They will be un-ignored by the stories that provide the required exporters:

| AC | Blocked by | Un-ignore trigger |
|----|-----------|-------------------|
| AC-001 | STORY-037 PPTX exporter | STORY-037 PR |
| AC-002 | STORY-037 PPTX exporter | STORY-037 PR |
| AC-003 | STORY-037 PPTX exporter | STORY-037 PR |
| AC-004 | STORY-041 DOCX exporter | STORY-041 PR |
| AC-005 | STORY-041 DOCX exporter | STORY-041 PR |
| AC-006 | STORY-046 HTML exporter | STORY-046 PR |
| AC-007 | STORY-037 + STORY-041 | STORY-041 PR |

---

## Harness Files Added

No new harness files were created for demo recording. All tests demonstrated are
co-located with the STORY-036 TDD delivery:

| File | Tests |
|------|-------|
| `crates/slideforge-eval/src/bleed_check.rs` | Production: `BleedChecker` implementation |
| `crates/slideforge-eval/tests/bleed_tests.rs` | AC-008, EC-003, EC-004, fixture-parse |
| `crates/slideforge-eval/tests/fixtures/three-register-slide.sf` | FIXTURE-PARSE |

No production code was modified during demo recording.

---

## File Manifest

```
docs/demo-evidence/STORY-036/
├── AC-008-bleedchecker-utility.tape
├── AC-008-bleedchecker-utility.gif
├── AC-008-bleedchecker-utility.webm
├── EC-003-path-scoping.tape
├── EC-003-path-scoping.gif
├── EC-003-path-scoping.webm
├── EC-004-xml-entity-decoding.tape
├── EC-004-xml-entity-decoding.gif
├── EC-004-xml-entity-decoding.webm
├── FIXTURE-PARSE-three-register-slide.tape
├── FIXTURE-PARSE-three-register-slide.gif
├── FIXTURE-PARSE-three-register-slide.webm
└── evidence-report.md
```

Total: 12 recording files (4 demos x 3 formats each) + this report = 13 files.
