---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-086
title: "Stage 2b: Post-Eval Field-to-Block Threading Pass (AltText::Unspecified)"
epic: EPIC-03
wave: 4
wave_designation: "4-REMEDIATION"
points: 13
priority: P0
tdd_mode: strict
status: draft
target_module: slideforge-eval, slideforge-types, slideforge-layout, slideforge-validate
subsystems: [SS-02, SS-03, SS-05, SS-15]
behavioral_contracts: [BC-1.16.001, BC-5.02.001, BC-5.01.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023]
closes_findings: [BLK-002, F-G3-CRIT-001, F-G3-HIGH-001, F-G3-HIGH-002]
depends_on:
  - STORY-049
  - STORY-050
blocks:
  - STORY-046
  - STORY-055
  - STORY-056
estimated_days: 5
remediation_context: >
  Wave 4 integration gate (2026-06-05) confirmed Slide.blocks is always vec![] after eval,
  making all three exporters content-empty and the strict a11y gate unconditionally
  unsatisfiable for chart/image/diagram slides. This story implements the Stage 2b fix
  authorized by the human on 2026-06-05 (ADR-019). It must merge and re-pass Wave 4
  Gate 3 (strict-mode) and Gate 5 (holdout) before Wave 5 advances.
---

# STORY-086: Stage 2b — Post-Eval Field-to-Block Threading Pass (AltText::Unspecified)

## Subsystem Anchor Justifications

- SS-02 (Evaluator, `slideforge-eval`) owns the new `field_to_block.rs` module and the
  exported `thread_fields_to_blocks(deck: &mut Deck)` function. The field-to-block pass
  is architecturally a post-eval, pre-layout semantic operation — it converts field values
  already resolved by the evaluator into typed `ContentBlock` semantic nodes. Per ADR-019
  Decision 2 and ADR-005 (Two-IR Model), this belongs in the evaluator crate, not layout.
  SS-02 is the single-responsibility owner of the semantic compilation chain.

- SS-03 (Validation, `slideforge-validate`) is modified to update `alt_text.rs` match
  arms: `AltText::Unspecified` fires E-A11-001; `AltText::Decorative` is now valid.
  ARCH-INDEX assigns all `Validator` implementations to SS-03.

- SS-05 (Layout Engine, `slideforge-layout`) is modified in two places: `regions.rs`
  (5 structural placeholder sites changed from `Decorative` to `Unspecified`) and
  `layout.rs` (`thread_media_alt_into_frames` fallback and line 163-166 comment).
  The layout crate already owns these files; the changes are targeted bug fixes within
  SS-05 scope.

- SS-15 (Types, `slideforge-types`) receives the new `AltText::Unspecified` variant in
  `specs.rs`. SS-15 is the foundational types crate; `AltText` is defined there. A
  sibling-site sweep (TD-VSDD-060) must cover every `AltText` match site across
  all crates that import `slideforge-types`.

The `slideforge/src/lib.rs` (root crate, SS-01 pipeline driver) receives the Stage 2b
call insertion and `build_inner` doc comment update. SS-01 owns pipeline wiring per
ADR-016 and ADR-019 Decision 9.

## Dependency Anchor Justifications

- Depends on STORY-049 (Plugin Registry Assembly): `build_inner` wires Stage 2b as a new
  pipeline stage. STORY-049 delivered the canonical `build_inner` pipeline driver.
  Stage 2b is inserted into that same driver. This story cannot land without the base
  pipeline being present.

- Depends on STORY-050 (E2E Integration Tests): STORY-050 delivered the failing gate
  tests that expose BLK-002, F-G3-CRIT-001, F-G3-HIGH-001, and F-G3-HIGH-002. This
  story must turn those tests green. Implementing Stage 2b without the E2E test
  infrastructure would have no verifiable closure.

- Blocks STORY-046 (HTML Exporter): the HTML exporter must consume `FrameContent::TextRun`
  frames that are only populated after Stage 2b threads title/body into ContentBlocks and
  layout converts them to frames. The HTML exporter tests require non-empty content.

- Blocks STORY-055 (CLI build command): CLI integration tests assert that `slideforge build`
  produces real output with visible content. These tests fail until Stage 2b is live.

- Blocks STORY-056 (CLI watch mode): same dependency path — watch mode drives `build_inner`
  in a loop; non-empty output requires Stage 2b.

## Summary

This story closes the systemic content-threading gap exposed by the Wave 4 integration gate.
`slideforge-eval` always produced `Slide.blocks = vec![]` (an intentional Wave 2 deferral
recorded at `for_eval.rs:342`). The consequence was total: title text, body text, bullets,
chart metadata, image metadata, and diagram metadata never reached the layout engine or
exporters. All three exporters (PPTX, PDF, DOCX) produced structurally valid but
content-empty output. The strict a11y gate was unconditionally unsatisfiable for any
chart/image/diagram slide because `thread_media_alt_into_frames` had nothing to thread.

Stage 2b is a new pure post-eval pass (`thread_fields_to_blocks`) inserted between
Stage 2 (eval) and Stage 3 (brand load) in `build_inner`. It reads resolved `Slide.fields`
and populates `Slide.blocks` with typed `ContentBlock` entries. It is 100% pure (no I/O,
deterministic), Kani-amenable, and ADR-005-compliant (operates within the semantic IR).

This story also bundles the `AltText::Unspecified` type change authorized simultaneously:
a third `AltText` variant that makes the a11y validator's match arms unambiguous
(`Unspecified` = pipeline gap → E-A11-001; `Decorative` = author opt-out → valid).

**Findings closed:** BLK-002, F-G3-CRIT-001, F-G3-HIGH-001, F-G3-HIGH-002.
**Wave gate re-check:** This story must merge and re-pass Wave 4 Gate 3 (strict-mode
a11y builds) and Gate 5 (holdout must-pass ≥ 0.6) before Wave 5 advances.

## Narrative

As a user of slideforge who writes titled slides with content, charts, images, and diagrams,
I want `slideforge build` to produce output with the text and media I declared in my `.sf` file
visible in PPTX, PDF, and DOCX exports,
so that my presentations and documents contain actual content instead of structurally valid
but empty shells.

## Behavioral Contracts

| BC | Title | Version | Role in This Story |
|----|-------|---------|-------------------|
| BC-1.16.001 | Post-Eval Field-to-Block Threading Pass | v1.0 | Primary: defines the Stage 2b function signature, fields→blocks mapping, alt-resolution rule, block ordering, purity contract, and all edge cases |
| BC-5.02.001 | All 10 Plugin Trait Surfaces (Validator post-layout dispatch) | v1.6 | AltTextValidator post-layout dispatch now fires on `AltText::Unspecified`; Postcondition 7 and EC-005, EC-009 define the exact discrimination semantics |
| BC-5.01.001 | Missing alt on Visual Element Is Compile Error | v1.3 | E-A11-001 fires on `AltText::Unspecified` (pipeline gap) not `AltText::Decorative` (author opt-out); Invariant 2 defines the post-layout enforcement contract |

## Acceptance Criteria

### AC-001 — Titled deck builds to PPTX with visible title text (LESSON-13 positive content vector)
A fixture deck with a `slide title:` block containing `title "My Title"` builds
successfully with `strict: true`. The resulting PPTX ZIP contains a slide XML where
the `<p:sp>` title placeholder carries at least one `<a:r><a:t>` text run with
text "My Title". The run is non-empty; the text node content matches the declared title.
(traces to BC-1.16.001 postcondition 1 — title field produces ContentBlock::Text(Title))

### AC-002 — Content slide body text visible in PDF via show_text operator (LESSON-13 positive content vector)
A fixture deck with a `slide content:` block containing `title "Heading"` and
`body "Body paragraph text"` builds successfully. The resulting PDF, when parsed for
text content (e.g., via `pdftotext` or internal text-run extraction), contains both
"Heading" and "Body paragraph text". At minimum, the PDF byte stream includes
at least one `show_text` or `Tj` operator carrying non-empty text.
(traces to BC-1.16.001 postconditions 1, 4 — title and body produce ContentBlock::Text)

### AC-003 — DOCX Heading1 run carries title text (LESSON-13 positive content vector)
A fixture deck with a `slide title:` block containing `title "Report Title"` builds
to DOCX. The DOCX `word/document.xml` contains at least one `<w:p>` paragraph whose
first `<w:r><w:t>` run contains "Report Title" (the Heading1 path). The heading run
must be non-empty.
(traces to BC-1.16.001 postcondition 1 — title ContentBlock threads to Heading1 in DOCX exporter)

### AC-004 — strict + chart WITH alt → build Ok, PPTX placeholder carries descr (LESSON-13 positive content vector)
A fixture deck with `slide chart: title "Q3 Revenue" chart_type "bar" alt "Bar chart showing Q3 revenue by region"`
builds successfully with `strict: true`. The build returns `Ok(BuildOutput)`. The PPTX
slide XML for the chart slide contains `<p:ph>` or equivalent element carrying
`descr="Bar chart showing Q3 revenue by region"` (or the equivalent PPTX accessibility
attribute). No E-A11-001 diagnostic is present in the build output.
(traces to BC-1.16.001 postcondition 9 + alt-resolution rule postcondition 12;
BC-5.01.001 postcondition 1 — Provided alt → valid, no error;
BC-5.02.001 postcondition 7 — AltText::Provided → no E-A11-001)

### AC-005 — strict + chart WITHOUT alt → Err(ValidationFailed) with exactly one E-A11-001
A fixture deck with `slide chart: title "Revenue" chart_type "bar"` (no `alt` field,
no `decorative: true`) built with `strict: true` returns `Err(BuildError::ValidationFailed)`
where the diagnostics list contains at least one `Diagnostic { code: "E-A11-001", .. }`.
No output bytes are produced. The diagnostic count for E-A11-001 is exactly one (one chart,
one missing alt). The frame's `alt` field must be `AltText::Unspecified` at the point the
validator fires (not `AltText::Decorative`).
(traces to BC-1.16.001 postcondition 12 — alt=None when neither alt nor decorative present;
BC-5.01.001 postcondition 1 — E-A11-001 on AltText::Unspecified;
BC-5.02.001 postcondition 7 — Unspecified → E-A11-001)

### AC-006 — decorative:true on a chart → strict Ok, empty descr / PDF Artifact
A fixture deck with `slide chart: title "Background" chart_type "area" decorative: true`
built with `strict: true` returns `Ok(BuildOutput)`. No E-A11-001 is emitted.
The PPTX slide XML carries the chart element with an empty or absent alt description
(decorative opt-out). The PDF tag engine marks the frame as a PDF Artifact (no `/Alt`
entry in the structure element). The frame's `alt` field is `AltText::Decorative`
throughout the pipeline.
(traces to BC-1.16.001 postcondition 12 — decorative=true produces AltText::Decorative;
BC-5.01.001 EC-007 — Decorative is valid; BC-5.02.001 EC-009 — Decorative → Ok)

### AC-007 — Bullets slide produces ≥N text runs in PPTX output (LESSON-13 positive content vector)
A fixture deck with `slide content: title "Agenda" bullets: ["Item A", "Item B", "Item C"]`
builds successfully. The PPTX slide XML for this slide contains at least 3 `<a:r>` text
runs corresponding to the 3 bullet items. Each run must carry non-empty text content
matching the declared bullet strings.
(traces to BC-1.16.001 postcondition 7 — bullets field produces ContentBlock::Bullets;
BC-1.16.001 invariant 2 — canonical block order: title before bullets)

### AC-008 — Empty title string produces no ContentBlock and no text run in output
A deck with `title ""` (explicit empty string) builds successfully. The resulting PPTX
slide XML for the slide does NOT contain a title placeholder text run. No `ContentBlock::Text`
with an empty-string body is present in `Slide.blocks` after threading.
(traces to BC-1.16.001 postcondition 6 — empty strings silently skipped;
BC-1.16.001 EC-001 — empty title produces no block)

### AC-009 — thread_fields_to_blocks is pure: same input → same output across two calls
A unit test constructs a `Deck` with one slide containing title + body fields.
It calls `thread_fields_to_blocks` once, records `slide.blocks`, resets `slide.blocks`
to `vec![]`, calls `thread_fields_to_blocks` again. The two recorded `slide.blocks`
vecs are equal (same types, same content, same order). No panic, no side effect.
(traces to BC-1.16.001 invariant 1 — pure pass; BC-1.16.001 postcondition 15 — determinism;
BC-1.16.001 invariant 7 — idempotency guard)

### AC-010 — Block canonical ordering: title → subtitle → body → bullets → chart
A unit test constructs a slide with all five field types (title, subtitle, body, bullets,
chart_type+alt) set. After `thread_fields_to_blocks`, `Slide.blocks` contains exactly
five ContentBlocks in order: `[Text(Title), Text(Subtitle), Text(Body), Bullets, Chart]`.
No other ordering is valid.
(traces to BC-1.16.001 postcondition 13 — canonical block ordering;
BC-1.16.001 invariant 2 — canonical block order invariant)

### AC-011 — Shape blocks are NOT populated by Stage 2b
After calling `thread_fields_to_blocks` on a deck where the slide's `.shapes` field is
non-empty, `Slide.blocks` contains NO `ContentBlock::Shape` entries. The `.shapes` field
is unchanged. Stage 2b must not read, modify, or append shape blocks.
(traces to BC-1.16.001 postcondition 14 — shape exclusion;
BC-1.16.001 invariant 4 — shape blocks not touched)

### AC-012 — AltText::Unspecified variant is added with complete sibling-site sweep
The `AltText` enum in `crates/slideforge-types/src/specs.rs` has exactly three variants:
`Provided(Arc<str>)`, `Decorative`, `Unspecified`. All `match` expressions on `AltText`
in the workspace compile without `non_exhaustive_patterns` warnings. No existing match
arm produces incorrect behavior for `Unspecified` (the sweep covers: `regions.rs` 5 sites,
`layout.rs` 3 sites, `alt_text.rs`, `block.rs`, and any test code).
(traces to BC-5.02.001 postcondition 7 — AltText::Unspecified distinction from Decorative;
BC-5.01.001 invariant 2 — Unspecified fires E-A11-001, Decorative does not)

### AC-013 — regions.rs structural placeholders use AltText::Unspecified (5 sites)
After the fix, all five occurrences in `crates/slideforge-layout/src/regions.rs` where
`FrameContent::Chart/Image/Diagram { alt: AltText::Decorative }` was used as a structural
placeholder are changed to `alt: AltText::Unspecified`. A unit test constructs a
`LaidOutDeck` from a chart slide without calling Stage 2b (simulating pre-threading state)
and asserts that the resulting frame carries `AltText::Unspecified`, not `AltText::Decorative`.
(traces to BC-5.01.001 invariant 2 — regions.rs Unspecified; ADR-019 Decision 5.1)

### AC-014 — thread_media_alt_into_frames fallback produces AltText::Unspecified (3 sites)
When `ContentBlock::Chart/Image/Diagram` has `alt = None`, the three fallback arms in
`thread_media_alt_into_frames` emit `AltText::Unspecified` (not `AltText::Decorative`).
The tracing::warn message is updated to accurately describe the state. A unit test
verifies that a chart ContentBlock with `alt = None` produces a frame with
`AltText::Unspecified` after `thread_media_alt_into_frames` runs.
(traces to BC-5.01.001 EC-004 — None alt maps to Unspecified;
ADR-019 Decision 5.2)

### AC-015 — validate_post_layout match arms updated (Unspecified → error, Decorative → valid)
The `validate_post_layout` match arms in `crates/slideforge-validate/src/alt_text.rs`
are updated so that `AltText::Unspecified` emits E-A11-001 and `AltText::Decorative`
emits no diagnostic. A unit test confirms:
- chart frame with `AltText::Unspecified` → E-A11-001 in strict mode.
- chart frame with `AltText::Decorative` → no E-A11-001 in strict mode.
- chart frame with `AltText::Provided("desc")` → no E-A11-001 in strict mode.
(traces to BC-5.01.001 postcondition 1 — E-A11-001 on Unspecified;
BC-5.02.001 postcondition 7 — three-variant discrimination;
ADR-019 Decision 5.3)

### AC-016 — Stale comments corrected (three files, ADR-019 Decision 6)
The following three stale comments are updated as part of this story:
1. `crates/slideforge-eval/src/for_eval.rs` lines 336–342: updated to read
   "Block-level content is populated by the post-eval field-to-block threading pass
   (Stage 2b, ADR-019). AltTextValidator now runs post-layout via ADR-018 Decision 3 —
   not on this Deck output."
2. `crates/slideforge-layout/src/layout.rs` lines 163–166: updated to cite
   "Stage 2b (ADR-019), see `slideforge-eval::field_to_block::thread_fields_to_blocks`"
   instead of STORY-027.
3. `crates/slideforge-validate/src/alt_text.rs` lines 175–193: updated to cite
   Story A (`STORY-086`) instead of the generic "Wave 3+" placeholder.
A reviewer can grep the three comment sites and confirm the updated text is present.
(traces to BC-1.16.001 — general correctness contract; ADR-019 Decision 6)

### AC-017 — build_inner Stage 2b call + doc comment update (pipeline wiring)
`crates/slideforge/src/lib.rs` is updated so that `build_inner` calls
`slideforge_eval::thread_fields_to_blocks(&mut deck)` immediately after `eval_deck`
returns and before the brand load stage. The `build_inner` doc comment enumerates
the pipeline stages in the sequence defined by ADR-019 Decision 1 (Stage 2 → Stage 2a
→ Stage 2b → Stage 3 → ... → Stage 7). The `use` import list gains
`slideforge_eval::thread_fields_to_blocks`.
(traces to BC-1.16.001 precondition 5 — threading runs pre-brand;
ADR-019 Decision 1 and Decision 9)

### AC-018 — Wave 4 Gate 3 re-pass: end-to-end fixture deck builds cleanly under strict=true
The slideforge E2E integration test suite (`STORY-050` infrastructure) passes with
`strict: true` on a fixture deck containing: a `title` slide, a `content` slide with
bullets, and a `chart` slide with `alt "..."`. All three output formats (PPTX, PDF, DOCX)
build successfully. The PPTX contains non-empty title runs. The PDF contains text content.
The DOCX Heading1 run is non-empty. No E-A11-001 is emitted for correctly-alt'd charts.
This test vector was the primary failing vector for Gate 3 and Gate 5.
(traces to BC-5.02.001 postcondition 5 — Stage 6b ordering;
BC-5.01.001 postcondition 1 — no false-positive on Provided alt;
LESSON-13/LESSON-14 positive content vectors)

## Architecture Mapping

| Component | Crate | File | Change Type | Pure/Effectful |
|-----------|-------|------|-------------|---------------|
| `thread_fields_to_blocks` | `slideforge-eval` | `src/field_to_block.rs` (NEW) | New module + function | Pure |
| `lib.rs` export | `slideforge-eval` | `src/lib.rs` | Add `pub mod field_to_block; pub use` | Pure |
| `build_inner` Stage 2b call | `slideforge` | `src/lib.rs` | Insert call + update doc comment + add use | Effectful (pipeline driver) |
| `AltText::Unspecified` | `slideforge-types` | `src/specs.rs` | Add variant to enum | Pure (type change) |
| `regions.rs` 5 sites | `slideforge-layout` | `src/regions.rs` | `Decorative` → `Unspecified` on structural placeholders | Pure |
| `thread_media_alt_into_frames` 3 arms | `slideforge-layout` | `src/layout.rs` | Fallback `Decorative` → `Unspecified`; warn msg update | Pure |
| Comment at layout.rs:163–166 | `slideforge-layout` | `src/layout.rs` | Update stale STORY-027 reference | Comment only |
| `validate_post_layout` match arms | `slideforge-validate` | `src/alt_text.rs` | `Unspecified → E-A11-001`, `Decorative → valid` | Pure |
| Comment at alt_text.rs:175–193 | `slideforge-validate` | `src/alt_text.rs` | Cite STORY-086 instead of "Wave 3+" | Comment only |
| `for_eval.rs` comment | `slideforge-eval` | `src/for_eval.rs` | Update stale blocks:vec![] deferral comment | Comment only |
| `block.rs` AltText match | `slideforge-types` | `src/block.rs` | `produces_structure_group()` Unspecified arm | Pure |
| Sibling-site sweep | all crates | various | All `match AltText` sites updated | Mixed |

**Forbidden Dependencies:**
- `slideforge-eval::field_to_block` MUST NOT import `slideforge-layout`, `slideforge-pptx`,
  `slideforge-pdf`, `slideforge-docx`, or `slideforge-html`. This function is pure; importing
  any output-format crate would be an ADR-005 violation. Build-time enforcement: if the
  `slideforge-eval` crate dependency graph includes any exporter crate, the build MUST fail
  (cargo deny / dependency-check).
- `slideforge-types::specs::AltText` MUST remain in `slideforge-types` (the leaf crate).
  It MUST NOT be moved to any crate higher in the dependency graph.

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~4,000 |
| ADR-019 full text | ~6,500 |
| ADR-005 (Two-IR model) | ~1,500 |
| ADR-016 (pipeline driver, excerpt) | ~1,500 |
| ADR-018 (post-layout validation, excerpt) | ~1,500 |
| BC-1.16.001 full text | ~3,000 |
| BC-5.02.001 relevant sections | ~2,000 |
| BC-5.01.001 full text | ~1,500 |
| `wave4-content-threading-assessment.md` key sections | ~2,500 |
| `crates/slideforge-eval/src/for_eval.rs` (excerpt around lines 330-350) | ~500 |
| `crates/slideforge-eval/src/lib.rs` (current) | ~500 |
| `crates/slideforge-types/src/specs.rs` (AltText + ContentBlock defs) | ~1,500 |
| `crates/slideforge-layout/src/regions.rs` (5 structural placeholder sites) | ~2,000 |
| `crates/slideforge-layout/src/layout.rs` (thread_media_alt_into_frames) | ~2,000 |
| `crates/slideforge-validate/src/alt_text.rs` (validate_post_layout) | ~1,500 |
| `crates/slideforge-types/src/block.rs` (produces_structure_group) | ~500 |
| `crates/slideforge/src/lib.rs` (build_inner) | ~1,500 |
| Unit test files (new) | ~3,000 |
| E2E fixture + integration test file | ~2,000 |
| Tool outputs (compiler messages, test results) | ~3,000 |
| **TOTAL ESTIMATED** | **~41,500 tokens** |

41,500 tokens is ~20% of a 200k context window — within the 20-30% per-story budget.
Read only the excerpts listed above; do NOT read full crate files end-to-end.

## Previous Story Intelligence

This story is in the Wave 4 remediation track. It follows Wave 4 Batch C (STORY-083,
STORY-084, STORY-085, STORY-049, STORY-050) which completed the plugin registry assembly
and E2E integration suite. Key lessons from Batch C:

- LESSON-13 (STORY-049): Positive content vectors are MANDATORY in E2E tests. Structure-only
  checks (valid ZIP, non-empty bytes) let the empty-content bug survive 59 merged stories.
  Every AC in this story that tests build output MUST assert on VISIBLE CONTENT (text runs,
  heading text, show_text operators), not just structural validity.
- LESSON-16 (STORY-083): Run `cargo clippy --workspace --all-targets -- -D warnings` before
  declaring convergence. Pedantic lints (`doc_markdown`, `uninlined_format_args`,
  `unnecessary_literal_bound`) fire on new public APIs and doc comments. All public items in
  `field_to_block.rs` require rustdoc. The `missing_docs` lint fires on new public items.
- LESSON-17 (STORY-084): Validate Red Gate density BEFORE test-writer dispatch. The Red
  Gate pass requires `todo!()` stubs in all non-trivial function bodies; a story that ships
  pre-filled bodies fails the Red Gate density check (≥0.5 threshold).
- TD-VSDD-059 (paper-fix): All AC closures must be verified by load-bearing tests, not
  doc comments or renames. The adversary will independently verify that each AC has a test
  that would fail if the implementation regressed.
- STORY-039 (alt-threading): 11-pass cascade found silent fallbacks, missing AltText arms,
  and incorrect match semantics. Expect the sibling-site sweep for `AltText::Unspecified`
  to surface 8-12 match sites across 5+ crates. The implementer must grep
  `AltText::Decorative` workspace-wide and audit every hit before the sweep is declared
  complete.

## Architecture Compliance Rules

These rules are extracted from ADR-005, ADR-016, ADR-018, ADR-019, and CLAUDE.md.
Violating any of these constitutes a CRIT finding in adversarial review.

1. **ADR-019 Decision 2 — Stage 2b must be in `slideforge-eval::field_to_block`.**
   The function MUST reside in this crate and this module. Moving it to `slideforge-layout`
   or any exporter crate is a rejected option (ADR-019 option (c)) and an ADR-005 violation.

2. **ADR-005 — No geometry in Stage 2b.** `thread_fields_to_blocks` produces
   `ContentBlock` semantic nodes. It MUST NOT compute EMU coordinates, PPTX shape positions,
   PDF page coordinates, or any geometric value. No `f64`. No import from `slideforge-layout`
   or any exporter.

3. **ADR-016 / ADR-019 Decision 9 — `build_inner` stage sequence must match.**
   After this story, the `build_inner` doc comment must enumerate: Stage 1 (assemble plugin
   registry) → Stage 2 (parse) → Stage 2a (eval) → Stage 2b (field-to-block threading,
   ADR-019) → Stage 3 (brand) → Stage 4 (lang default) → Stage 5 (validate pre-layout)
   → Stage 6 (layout) → Stage 6b (validate post-layout, ADR-018) → Stage 7 (export).
   Any other numbering is non-conformant.

4. **CLAUDE.md `#![forbid(unsafe_code)]`** — No `unsafe` blocks permitted in new code.
   The `AltText` enum, `ContentBlock` constructions, and match arms are all safe Rust.

5. **CLAUDE.md zero `.unwrap()` in non-test code.** All field access via `fields.get(key)`
   returns `Option`; use `if let Some(...)` or `.unwrap_or_else(...)`. No `.unwrap()` calls
   in `thread_fields_to_blocks` or any production code path.

6. **TD-VSDD-060 sibling-site sweep** — When adding `AltText::Unspecified`, grep
   `AltText::Decorative` workspace-wide before committing. Every match arm that currently
   handles `Decorative` must be reviewed to determine whether `Unspecified` needs a
   separate arm. The sweep result must be documented in the PR description.

7. **`#![warn(missing_docs)]`** — The new `field_to_block.rs` module and its public
   function `thread_fields_to_blocks` must have rustdoc comments. The `AltText::Unspecified`
   variant must have a rustdoc doc comment explaining its pipeline semantics.

8. **`Hash + Eq + Clone` on all new types** — All `ContentBlock`, `TextBlock`, `BulletItem`
   types already implement these. Verify `AltText::Unspecified` does not break any derived
   impl (it is a unit variant; derives propagate automatically).

## Library and Framework Requirements

All versions from the project dependency graph (authoritative source:
`crates/slideforge-eval/Cargo.toml`, `crates/slideforge-types/Cargo.toml`):

| Library | Version | Purpose |
|---------|---------|---------|
| `tracing` | 0.1 (workspace) | `tracing::warn!` in fallback arms (absent chart_type, absent src) |
| `arc-str` via `Arc<str>` | std | String interning for `AltText::Provided`, `InlineNode::Plain` |
| `slideforge-types` | workspace path | `ContentBlock`, `AltText`, `TextBlock`, `BulletItem`, `InlineNode` |

No new external dependencies are added by this story. `field_to_block.rs` imports
only `slideforge-types` and `tracing` (both already in `slideforge-eval`'s dependency
tree per STORY-011/012/013).

## File Structure Requirements

Files to CREATE:
```
crates/slideforge-eval/src/field_to_block.rs   [new module: thread_fields_to_blocks]
crates/slideforge-eval/tests/field_to_block_unit.rs   [unit tests for AC-009, AC-010, AC-011]
```

Files to MODIFY:
```
crates/slideforge-eval/src/lib.rs              [pub mod field_to_block; pub use]
crates/slideforge-eval/src/for_eval.rs         [update comment lines 336-342]
crates/slideforge-types/src/specs.rs           [add AltText::Unspecified variant]
crates/slideforge-types/src/block.rs           [produces_structure_group Unspecified arm]
crates/slideforge-layout/src/regions.rs        [5 sites: Decorative→Unspecified for structural placeholders]
crates/slideforge-layout/src/layout.rs         [3 sites in thread_media_alt_into_frames: Decorative→Unspecified fallback; update comment 163-166]
crates/slideforge-validate/src/alt_text.rs     [match arms: Unspecified→E-A11-001, Decorative→valid; update comment 175-193]
crates/slideforge/src/lib.rs                   [insert Stage 2b call; update build_inner doc comment; add use]
```

Files that RECEIVE new tests or test updates:
```
crates/slideforge-eval/tests/field_to_block_unit.rs    [new: unit tests for Stage 2b]
tests/integration/e2e_build_tests.rs                   [update: add positive content vector assertions for Wave 4 Gate 3 re-pass (AC-001..AC-007, AC-018)]
crates/slideforge-validate/src/alt_text.rs             [cfg(test) block: add AC-013, AC-014, AC-015 unit tests]
crates/slideforge-layout/src/regions.rs                [cfg(test) block: add AC-013 unit test for Unspecified placeholder]
```

Forbidden: Do NOT create `crates/slideforge-layout/src/field_to_block.rs` or any
`field_to_block` module outside `slideforge-eval`. Per ADR-019 Decision 2, Stage 2b is
exclusively owned by `slideforge-eval`.

## Tasks

- [ ] **T1 — Red Gate: write all failing tests first**
  - [ ] T1.1: Write unit tests for `thread_fields_to_blocks` (AC-009, AC-010, AC-011, AC-008) in `field_to_block.rs` `#[cfg(test)] mod tests` block. All tests must FAIL (function not yet implemented — `todo!()`).
  - [ ] T1.2: Write unit tests for `AltText::Unspecified` match discrimination (AC-012, AC-013, AC-014, AC-015) in respective `#[cfg(test)]` blocks. Tests must FAIL.
  - [ ] T1.3: Update E2E integration tests (AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-018) to assert VISIBLE CONTENT (text runs, heading text, show_text operators) in all three output formats. Tests must FAIL.
  - [ ] T1.4: Verify Red Gate density ≥0.5: count `todo!()` sites vs. non-trivial function bodies. Gate must pass BEFORE implementation starts.

- [ ] **T2 — Add AltText::Unspecified variant + sibling-site sweep**
  - [ ] T2.1: Add `Unspecified` variant to `AltText` enum in `slideforge-types/src/specs.rs` with rustdoc.
  - [ ] T2.2: Run `grep -r "AltText::Decorative" crates/` — document all N match sites found.
  - [ ] T2.3: For each match site: determine whether `Unspecified` requires a new arm or shares semantics with `Decorative`. Add `Unspecified` arms everywhere they are semantically distinct.
  - [ ] T2.4: Update `produces_structure_group()` in `block.rs` — `Unspecified` has same non-structure-producing semantics as `Decorative`.
  - [ ] T2.5: Run `cargo build --workspace` — verify no `non_exhaustive_patterns` warnings.

- [ ] **T3 — Fix regions.rs structural placeholders (5 sites)**
  - [ ] T3.1: In `regions.rs`, change all 5 occurrences of `FrameContent::Chart/Image/Diagram { alt: AltText::Decorative }` structural placeholders to `alt: AltText::Unspecified`.
  - [ ] T3.2: Update comments at those 5 sites to read: "structural placeholder — overwritten by thread_media_alt_into_frames when Stage 2b populates Slide.blocks; if never overwritten, validate_post_layout emits E-A11-001."
  - [ ] T3.3: Run AC-013 unit test — must now pass.

- [ ] **T4 — Fix thread_media_alt_into_frames fallback arms (3 sites) + layout.rs comment**
  - [ ] T4.1: In `layout.rs`, change 3 fallback arms in `thread_media_alt_into_frames` from `AltText::Decorative` to `AltText::Unspecified`. Update `tracing::warn!` message text.
  - [ ] T4.2: Update comment at lines 163–166 in `layout.rs` to cite "Stage 2b (ADR-019)" instead of STORY-027.
  - [ ] T4.3: Run AC-014 unit test — must now pass.

- [ ] **T5 — Fix validate_post_layout match arms + alt_text.rs comment**
  - [ ] T5.1: In `alt_text.rs`, update `validate_post_layout` match arms so `Unspecified → E-A11-001`, `Decorative → valid` (no diagnostic), `Provided(_) → valid`.
  - [ ] T5.2: Update comment at lines 175–193 to cite "STORY-086 (Stage 2b, ADR-019)" and remove the "MUST be revisited for Wave 3+" language.
  - [ ] T5.3: Run AC-015 unit tests — must now pass.

- [ ] **T6 — Implement thread_fields_to_blocks in field_to_block.rs**
  - [ ] T6.1: Create `crates/slideforge-eval/src/field_to_block.rs` with the function signature from ADR-019 Decision 2, full rustdoc, and a stub body of `todo!()`.
  - [ ] T6.2: Export from `slideforge-eval/src/lib.rs`: add `pub mod field_to_block; pub use field_to_block::thread_fields_to_blocks;`.
  - [ ] T6.3: Implement the fields→blocks mapping per BC-1.16.001 postconditions 1–14 and ADR-019 Decision 3:
    - Text content: title (prepend), subtitle, body (append).
    - Empty-string/whitespace guard: skip without emitting a block.
    - Bullets from `Value::List` and `FieldValue::Inlines`.
    - Chart: read `chart_type`, apply alt-resolution rule (decorative → Decorative, alt str → Provided, neither → None).
    - Image: read `src`, apply alt-resolution rule.
    - Diagram: read `source`, apply alt-resolution rule.
    - Canonical block order: title → subtitle → body → bullets → media.
    - At most one media block per slide (chart type wins if conflict).
  - [ ] T6.4: Update `for_eval.rs` comment at lines 336–342 to cite Stage 2b (ADR-019).
  - [ ] T6.5: Run unit tests T1.1 — all must now pass.

- [ ] **T7 — Wire Stage 2b into build_inner**
  - [ ] T7.1: In `crates/slideforge/src/lib.rs`, add `use slideforge_eval::thread_fields_to_blocks;` to imports.
  - [ ] T7.2: Insert `thread_fields_to_blocks(&mut deck);` call immediately after `eval_deck` returns and before brand load.
  - [ ] T7.3: Update the `build_inner` doc comment to enumerate the Stage 2b step per ADR-019 Decision 1 sequence.
  - [ ] T7.4: Run `cargo test -p slideforge` — verify pipeline wiring does not break existing tests.

- [ ] **T8 — Green pass: all ACs passing**
  - [ ] T8.1: Run `cargo nextest run -p slideforge-eval --no-fail-fast` — all field_to_block unit tests pass.
  - [ ] T8.2: Run `cargo nextest run -p slideforge-validate --no-fail-fast` — AC-015 tests pass.
  - [ ] T8.3: Run `cargo nextest run -p slideforge-layout --no-fail-fast` — AC-013, AC-014 tests pass.
  - [ ] T8.4: Run `cargo nextest run -p slideforge --no-fail-fast` — E2E tests (AC-001..AC-007, AC-018) pass.
  - [ ] T8.5: Run `just check` (full workspace pre-push gate: fmt + clippy pedantic + nextest + doctests).

- [ ] **T9 — Wave 4 Gate re-check**
  - [ ] T9.1: Confirm the E2E fixture that previously triggered Gate 3 failure now produces non-empty PPTX, PDF, DOCX output.
  - [ ] T9.2: Confirm that a chart slide with `alt "..."` builds cleanly under `strict: true` (Gate 3 pass condition).
  - [ ] T9.3: Confirm that a chart slide without alt produces E-A11-001 under `strict: true` (expected behavior, not regression).
  - [ ] T9.4: Signal wave-gate for re-evaluation of Gate 3 and Gate 5.

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-1.16.001 EC-001 | `fields["title"] = Value::Str("")` | No ContentBlock::Text produced; empty string silently skipped |
| EC-002 | BC-1.16.001 EC-002 | `fields["title"] = Value::Str("  ")` whitespace-only | No ContentBlock::Text produced; whitespace-only treated as empty after trim |
| EC-003 | BC-1.16.001 EC-003 | `fields["alt"] = Value::Str("")` on chart slide | alt = None; layout produces AltText::Unspecified; E-A11-001 in strict mode |
| EC-004 | BC-1.16.001 EC-004 | Both `decorative: true` AND `fields["alt"] = Str("desc")` on chart | `alt = Some(AltText::Decorative)`, `decorative = true`; pre-layout emits W-A11-002; frame carries Decorative |
| EC-005 | BC-1.16.001 EC-005 | Chart slide with no `fields["chart_type"]` | `tracing::warn!` emitted; no ContentBlock::Chart produced; layout region carries Unspecified placeholder; E-A11-001 in strict mode |
| EC-006 | BC-1.16.001 EC-006 | Image slide with no `fields["src"]` | `tracing::warn!` emitted; no ContentBlock::Image produced; same Unspecified outcome |
| EC-007 | BC-1.16.001 EC-007 | Slide with title + body + bullets all set | Three blocks produced: [Text(Title), Text(Body), Bullets(...)] in canonical order |
| EC-008 | BC-1.16.001 EC-008 | `fields["bullets"] = Value::List([])` empty list | ContentBlock::Bullets(vec![]) produced; layout handles zero-item bullet list |
| EC-009 | BC-1.16.001 EC-009 | Non-media slide type (title, content, toc) with no src/chart_type/source | Only text/bullet blocks produced; no media block; no warn emitted |
| EC-010 | BC-1.16.001 EC-010 | `thread_fields_to_blocks` on deck with zero slides | No-op; Deck.slides remains empty; no error |
| EC-011 | BC-1.16.001 EC-011 | alt as `FieldValue::Inlines(...)` (rich-text alt) | Threading pass reads only `Literal(Str)` for alt; FieldValue::Inlines alt treated as absent; alt = None |
| EC-012 | ADR-019 Decision 4.1 | Existing test code with `AltText::Decorative` match in test assertions | Sweep must update test match arms to handle Unspecified; tests may need new fixture variants |
| EC-013 | wave4-content-threading-assessment §2.3 | Deck with NO visual media slides (title, content, agenda, toc) built with strict=true | Decks without chart/image/diagram slides must build cleanly under strict mode after Unspecified fix; false-positive E-A11-001 is eliminated |

## Dependency Graph

```
STORY-049 (build_inner pipeline driver)
  └─ STORY-050 (E2E integration suite — provides failing test infrastructure)
       └─ [STORY-086 — THIS STORY — Stage 2b threading + AltText::Unspecified] (Wave 4 REMEDIATION)
            ├─ STORY-046 (HTML exporter — needs FrameContent::TextRun frames)
            ├─ STORY-055 (CLI build command — integration tests need real content)
            └─ STORY-056 (CLI watch mode — drives build_inner in loop)
```

Wave 5 stories that depend on content-correctness (STORY-055, STORY-056, STORY-046, etc.)
are blocked by this story. STORY-086 must merge before Wave 5 dispatch begins.
