---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-036
title: "Register-Aware Rendering: No Content Bleed Invariant"
epic: EPIC-18
wave: 4
points: 5
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.14.004]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-eval
target_module: slideforge-eval
subsystems: [SS-02, SS-06, SS-08]
depends_on:
  - STORY-035
blocks:
  - STORY-037
  - STORY-041
estimated_days: 2
---

# STORY-036: Register-Aware Rendering: No Content Bleed Invariant

## Subsystem Anchor Justification

SS-02 (Evaluator) owns the core no-bleed invariant because register routing is an
evaluate-stage concern per BC-1.14.004 invariant 2. SS-06 (PPTX Export) and SS-08
(DOCX Export) are secondarily involved because they must read only their allowed
registers and must be tested to confirm they exclude disallowed registers. This story
establishes the cross-cutting bleed-detection test harness that runs against all
exporters from this point forward.

## Dependency Anchor Justifications

- Depends on STORY-035: The `register_content` field and `Register` enum produced by
  STORY-035 are the inputs to the bleed tests. Without the routing infrastructure, no
  bleed test can run.
- Blocks STORY-037 (PPTX Core Serialization): STORY-037 must pass bleed-invariant tests
  as a precondition for its acceptance criteria. The test harness built here is imported
  by STORY-037's test suite.
- Blocks STORY-041 (DOCX Core Serialization): same — STORY-041's tests use this harness.

## Summary

Implement the no-content-bleed invariant: a cross-cutting test harness and a set of
runtime assertions embedded in each exporter that confirm no register content appears
in the wrong output format. This story does NOT implement any exporter — it builds the
invariant infrastructure that exporters must satisfy:

1. A `BleedChecker` helper in `slideforge-eval/src/bleed_check.rs` that scans any
   serialized output (XML bytes, etc.) for register-content strings and asserts their
   absence.
2. A canonical "all three registers" test fixture `.sf` deck used by all exporter tests.
3. Runtime assertions (debug builds) in the export pipeline: before writing output,
   each exporter checks that it has not included disallowed register content.

### No-Bleed Matrix

| Content | PPTX slide body | PPTX speaker notes | DOCX body | DOCX notes section | PDF body | HTML canvas |
|---------|----------------|-------------------|-----------|-------------------|----------|-------------|
| `notes` | NEVER | YES | NEVER | YES | NEVER | NEVER (aside only) |
| `report` | NEVER | NEVER | YES | NEVER | YES | aside/section only |
| `detail` | NEVER | NEVER | YES (extended) | NEVER | YES (appendix) | NEVER |

The word "NEVER" in the table above is an invariant — not a design preference. A test
failure on any NEVER cell is a P0 defect.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.14.004 | No register content bleeds to wrong format | AC-001 through AC-007 |

## Acceptance Criteria

### AC-001: notes content absent from PPTX slide body
(traces to BC-1.14.004 postcondition 2 — notes NOT in PPTX slide body)

A test deck has one slide: `notes "NOTES_SENTINEL_VALUE"` and a title "Hello". Built to
PPTX, the test opens the PPTX ZIP, reads all `ppt/slides/slide*.xml` files, and asserts
that "NOTES_SENTINEL_VALUE" is absent. The string MAY appear in `ppt/notesSlides/`.

### AC-002: report content absent from PPTX slide body
(traces to BC-1.14.004 postcondition 4 — report NOT in PPTX slide body)

A test deck has one slide: `report "REPORT_SENTINEL_VALUE"` and a title "Hello". Built
to PPTX, the test reads all `ppt/slides/slide*.xml` and asserts "REPORT_SENTINEL_VALUE"
is absent. Exit code 0.

### AC-003: detail content absent from PPTX entirely
(traces to BC-1.14.004 postcondition 6 — detail NOT in PPTX)

A test deck has one slide: `detail "DETAIL_SENTINEL_VALUE"`. Built to PPTX, the test
reads ALL files in the PPTX ZIP (slides, notes, masters, everything) and asserts
"DETAIL_SENTINEL_VALUE" is absent from the entire ZIP. Exit code 0.

### AC-004: notes content absent from DOCX report body
(traces to BC-1.14.004 postcondition 2 — notes NOT in DOCX report body)

A test deck has one slide: `notes "NOTES_DOCX_SENTINEL"`. Built to DOCX, the test opens
the DOCX ZIP, reads `word/document.xml` (the body), and asserts "NOTES_DOCX_SENTINEL"
is absent from `document.xml`. The string MAY appear in a designated presenter notes
section if the DOCX exporter implements one (STORY-041 will add this).

### AC-005: report content present in DOCX body
(traces to BC-1.14.004 postcondition 3 — report content ONLY in DOCX body)

A test deck has one slide: `report "REPORT_DOCX_SENTINEL"`. Built to DOCX, the test
reads `word/document.xml` and asserts "REPORT_DOCX_SENTINEL" IS present (positive test).
This confirms the routing is active, not just the negative bleed tests.

### AC-006: detail content absent from HTML/preview canvas
(traces to BC-1.14.004 postcondition 6 — detail NOT in HTML preview)

A test deck has one slide: `detail "DETAIL_HTML_SENTINEL"`. Built to static HTML,
the test parses the HTML output and asserts "DETAIL_HTML_SENTINEL" does not appear
inside any `<div class="slide-canvas">` or equivalent slide container. Exit code 0.
This test runs with the HTML exporter stub (STORY-046); if HTML is not yet built,
skip with `#[ignore = "requires STORY-046"]`.

### AC-007: all three registers on same slide — no bleed
(traces to BC-1.14.004 postcondition 1-6 — all combinations)

A test deck has one slide with all three registers plus visual content:
```
title "Test Slide"
notes "SENTINEL_NOTES"
report "SENTINEL_REPORT"
detail "SENTINEL_DETAIL"
```
Built to PPTX: slide body has NO sentinel strings; notes slide has SENTINEL_NOTES but
not SENTINEL_REPORT or SENTINEL_DETAIL. Built to DOCX: `document.xml` body has
SENTINEL_REPORT but NOT SENTINEL_NOTES or SENTINEL_DETAIL.

### AC-008: BleedChecker test utility available
(traces to BC-1.14.004 invariant 1 — no register content crosses format boundaries)

A `BleedChecker` utility struct is available in `slideforge-eval` gated behind
a `test-utils` Cargo feature (`#[cfg(feature = "test-utils")]`) with methods:

Exporter crates list `slideforge-eval = { ..., features = ["test-utils"] }` in
their `[dev-dependencies]` to access it.
```rust
fn assert_absent_from_pptx_slides(pptx_bytes: &[u8], sentinel: &str);
fn assert_absent_from_pptx_all(pptx_bytes: &[u8], sentinel: &str);
fn assert_present_in_docx_body(docx_bytes: &[u8], sentinel: &str);
fn assert_absent_from_docx_body(docx_bytes: &[u8], sentinel: &str);
```
These methods are used by all exporter test suites from STORY-037 onward.

## Tasks

- [ ] Create `crates/slideforge-eval/src/bleed_check.rs` with `BleedChecker` struct, gated behind the `test-utils` Cargo feature
  - Implement `assert_absent_from_pptx_slides(bytes, sentinel)`: unzip, scan slide XMLs
  - Implement `assert_absent_from_pptx_all(bytes, sentinel)`: unzip, scan all files
  - Implement `assert_present_in_docx_body(bytes, sentinel)`: unzip, scan document.xml
  - Implement `assert_absent_from_docx_body(bytes, sentinel)`: unzip, scan document.xml
- [ ] Create canonical test fixture file `tests/fixtures/three-register-slide.sf` with all three registers plus visual content
- [ ] Write bleed invariant tests in `crates/slideforge-eval/src/tests/bleed_tests.rs`:
  - AC-001: notes sentinel absent from PPTX slides (stubbed — depends on STORY-037)
  - AC-002: report sentinel absent from PPTX slides (stubbed)
  - AC-003: detail sentinel absent from PPTX all parts
  - AC-004: notes sentinel absent from DOCX body (stubbed — depends on STORY-041)
  - AC-005: report sentinel present in DOCX body (positive, stubbed)
  - AC-007: all-three-registers no-bleed (most important; stubbed until exporters exist)
- [ ] Mark PPTX-dependent tests `#[ignore = "requires STORY-037 PPTX exporter"]`
- [ ] Mark DOCX-dependent tests `#[ignore = "requires STORY-041 DOCX exporter"]`
- [ ] The non-stubbed tests (AC-008 `BleedChecker` utility unit tests) must pass immediately
- [ ] Gate `BleedChecker` behind a `test-utils` Cargo feature in `crates/slideforge-eval/Cargo.toml`. Exporter crates add `slideforge-eval/test-utils` to `[dev-dependencies]` to access it. Update `crates/slideforge-eval/src/lib.rs` to re-export `BleedChecker` under `#[cfg(feature = "test-utils")]`.

## Previous Story Intelligence

STORY-035 establishes `register_content: Vec<RegisteredContent>` on `LaidOutSlide`.
This story uses those types. The `BleedChecker` utility works at the byte/XML level
(post-serialization), not at the IR level — it is the final defense against bleed bugs.

The sentinel-based approach (using unique strings like "NOTES_SENTINEL_VALUE") is
intentional: it allows the bleed tests to be independent of the exporter implementation
details. Any accidental inclusion of register content produces a clear test failure
with the sentinel string visible in the assertion error.

## Architecture Compliance Rules

1. **BC-1.14.004 invariant 2**: Routing rules are determined at the Evaluate stage,
   not the Export stage. This story implements the TEST that verifies exporters honor
   those rules — the tests run against post-serialization bytes.
2. **`BleedChecker` is test-only**: `#[cfg(test)]`. No production code path calls it.
   It is a test utility, not a runtime guard.
3. **SS-02 pure-core boundary**: The `BleedChecker` utility is pure computation
   (ZIP parsing + string search). No I/O in production code.
4. **Stub tests unlock exporter stories**: The stubbed tests (marked `#[ignore]`) are
   designed to be un-ignored once STORY-037 and STORY-041 provide exporter output. This
   creates a clear dependency chain: STORY-037's PR must include un-ignoring AC-001/002.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-eval` (self) | workspace | `BleedChecker` implementation |
| `slideforge-types` (workspace) | workspace | `Register`, `RegisteredContent` |
| `zip` | `=4.2.0` | ZIP unpacking in `BleedChecker` (test-only dev-dep; compatible with ooxmlsdk 0.6.1) |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-eval/src/bleed_check.rs` | Create | `BleedChecker` test utility (`#[cfg(test)]`) |
| `crates/slideforge-eval/src/tests/bleed_tests.rs` | Create | Bleed invariant tests (some stubbed) |
| `tests/fixtures/three-register-slide.sf` | Create | Canonical three-register test deck |
| `crates/slideforge-eval/src/lib.rs` | Modify | Re-export `BleedChecker` for exporter test use |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,500 |
| BC-1.14.004 | ~1,500 |
| STORY-035 types reference | ~800 |
| `zip` crate API | ~500 |
| Test files to write | ~2,000 |
| **Total** | **~7,300** |

## Test Strategy

- **Unit tests for BleedChecker**: Construct minimal PPTX-like ZIP bytes with known
  strings; verify `assert_absent_from_pptx_slides` passes and `assert_present_in_docx_body`
  passes; verify assertion panics when invariant violated.
- **Bleed invariant tests**: Run against real PPTX/DOCX output once STORY-037/041 exist;
  initially stubbed with `#[ignore]`.
- **Fixture file test**: The `three-register-slide.sf` fixture is itself tested to
  parse without errors (evaluator test).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only register content (no visual fields) | All sentinels route to correct sections; PPTX slide body is empty |
| EC-002 | `detail` content when building PPTX only | No detail content in PPTX; exit 0 (not an error) |
| EC-003 | Sentinel string appears in slide `title` field (not in register) | Must NOT be detected as bleed — `BleedChecker` checks specifically for the sentinel, not all content |
| EC-004 | XML-escaped sentinel value (e.g., `&amp;` in content) | `BleedChecker` works on decoded XML text, not raw bytes, to avoid false negatives |

## Forbidden Dependencies

Same as STORY-035:
- `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html`, `slideforge-preview`
  must NOT appear in `slideforge-eval/Cargo.toml` as production deps.
- The `zip` crate is acceptable as a `[dev-dependencies]` dep only (used only in `#[cfg(test)]`).
