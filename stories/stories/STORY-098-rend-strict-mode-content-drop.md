---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-098
title: "REND-005/010a: Strict-mode exit on W-VAL-103 content drop + chart no-data enforcement"
epic: EPIC-04
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.2"
# Changelog:
# v1.1 (2026-06-11): Fix path mis-anchor F-098-P1-006 — body/content drift site is
#   crates/slideforge-eval/src/field_to_block.rs (thread_one_slide ~188-203), NOT
#   crates/slideforge-validate/src/field_to_block.rs:135. Also synced AC-004/AC-005
#   to BC-1.11.002 v1.2 per PO adjudication of F-098-P1-007 (missing `data:` field
#   treated identically to empty-evaluating binding; E-LAY-003 message self-prefixed).
# v1.2 (2026-06-11): PO binding adjudication F-098-ADJ-BODY-CONTENT — resolution (a)
#   confirmed: `body` is a valid optional field on `content` slide type. AC-002 and
#   AC-003 updated to reflect decision (a). BC-3.03.002 v1.3 is the updated spec.
#   The exit-2 test for body-on-content (T-004) is REPLACED by a render-success test.
#   AC-002 now verifies that body prose renders on a content slide (no W-VAL-103 emitted).
#   AC-003 now verifies consistent schema-valid behavior (both validator and eval accept
#   body on content).
created: "2026-06-11"
source_findings: [REND-005, REND-010]
behavioral_contracts: [BC-3.03.002, BC-1.11.002, BC-3.03.001]
# BC-3.03.002 v1.2 (rendering-fix wave 2026-06-11): W-VAL-103 content-drop sub-case
# promoted to broken/exit-2 in strict mode per Invariant 4 (Route A). Error taxonomy
# v2.29 confirms: no new E-VAL-105 code — same W-VAL-103 message, context-sensitive
# severity based on key in {"shape", "body"}.
# Implementer action site: validate_fields in
# crates/slideforge-plugin-api/src/slide_types/registry.rs
# BC-1.11.002 all BCs are authored. BC-3.03.001 referenced for body/content schema drift.
verification_properties: []
nfr_refs: []
closes_findings: [REND-005, REND-010]
depends_on:
  - STORY-016
  - STORY-032
  - STORY-089
blocks: []
target_module: slideforge-eval, slideforge-plugin-api
# Note: slideforge-validate removed — body/content drift fix site is
# crates/slideforge-eval/src/field_to_block.rs (thread_one_slide), not slideforge-validate.
subsystems: [SS-03, SS-02]
estimated_days: 3
---

# STORY-098: REND-005/010a — Strict-Mode W-VAL-103 Exit + Chart No-Data Enforcement

## Subsystem Anchor Justification

SS-03 (Compile-Time Validation) owns REND-005: W-VAL-103 is emitted by `validate_fields`
in `slideforge-plugin-api` and the strict-mode exit gate in `slideforge-validate` must
treat content-drop warnings as fatal. SS-03 also owns the `body`/`content` schema drift
fix in `field_to_block.rs`. SS-02 (Evaluator) participates in the chart no-data path
per BC-1.11.002 invariant 2 (ChartRenderer not called with empty data — validator must
intercept). EPIC-04 (Compile-Time Validation) is the owning epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-016]` — Strict/warn-only mode gating was implemented in STORY-016;
  this story changes the classification of W-VAL-103 to trigger the strict exit.
- `depends_on: [STORY-032]` — Chart empty-data error-slide placeholder behavior (BC-1.11.002).
  REND-010a reveals this BC is violated in practice; this story closes the gap.
- `depends_on: [STORY-089]` — Field-type validation (W-VAL-103 formal registration and
  E-VAL-104 new behavior); schema reconciliation for `body`/`content` builds on STORY-089.

## Narrative

As a slideforge user building in strict mode (the default), I want the build to exit
non-zero when authored content would be silently dropped due to field validation warnings,
and I want a chart with no data to emit a clear error rather than rendering silently empty.

## Previous Story Intelligence

STORY-016 implemented strict/warn-only mode gating. STORY-032 implemented the
error-slide placeholder for chart empty data. STORY-089 formally registered W-VAL-103.
Two post-merge defects remain:

1. W-VAL-103 is classified as "cosmetic/exit 0" — but content-drop behavior makes this
   classification wrong in strict mode. When an authored field is silently dropped
   (shape:, body), strict mode MUST exit non-zero.
2. Chart with no data renders empty without an error, suggesting BC-1.11.002's validator
   interception (invariant 2) is not wired end-to-end.
3. `body` field on `content` slide type: eval passes it in `thread_one_slide`
   (`crates/slideforge-eval/src/field_to_block.rs` ~lines 188-203) but `content.rs`
   schema rejects it — inconsistency lets some `body` content through while rejecting
   it in other paths. (F-098-P1-006: previously mis-cited as slideforge-validate/src/
   field_to_block.rs:135 — the actual site is the eval crate.)

## Architecture Compliance Rules

- **Route A is confirmed (error-taxonomy v2.29):** No new E-VAL-105 code. W-VAL-103 uses
  context-sensitive severity. When the unknown field key is `"shape"` or `"body"` AND the
  build mode is `strict`, accumulate W-VAL-103 with `broken` severity (exit 2). In
  `--warn-only`, accumulate as `cosmetic` regardless of key. All other unknown-field keys:
  `cosmetic` always (unchanged from v2.20).
- **Implementer action site (BC-3.03.002 v1.2 Invariant 4):** In `validate_fields` in
  `crates/slideforge-plugin-api/src/slide_types/registry.rs`, add the key-set check
  `{"shape", "body"}` at the W-VAL-103 accumulation point. No other files need changing
  to implement the severity promotion.
- Per BC-3.03.002 v1.2 postcondition 3: the W-VAL-103 message format is UNCHANGED.
  Severity is determined at accumulation time by the field-key check.
- Per BC-3.03.002 invariant 1 (DI-017): all-or-nothing in strict mode.
- Per BC-1.11.002 invariant 2: the ChartRenderer plugin MUST NOT be called with empty
  data. The validator intercepts before plugin invocation.
- `slideforge-validate` MUST NOT bypass the existing error-accumulation model (DI-018).
- body/content schema drift resolution (PO adjudication F-098-ADJ-BODY-CONTENT, 2026-06-11):
  `body` on `content` slide type is schema-VALID — resolution (a) confirmed. `body` is
  declared in `ContentSlideType.optional` (content.rs), in `known_fields("content")`
  (known_fields.rs), and `thread_one_slide`'s `slide_type_supports_body` gate allows
  threading when known_fields includes "body". No W-VAL-103 is emitted for body on content.
  BC-3.03.002 v1.3 EC-007 has been reversed. (F-098-P1-006 path correction: site is eval
  crate's field_to_block.rs, not slideforge-validate.)

## Library & Framework Requirements

- No new dependencies. Internal crates: `slideforge-plugin-api`, `slideforge-validate`,
  `slideforge-eval`.

## File Structure Requirements

Files to modify:
- `crates/slideforge-plugin-api/src/slide_types/registry.rs` — promote W-VAL-103 (or
  add E-VAL-105) for content-drop fields in strict mode.
- `crates/slideforge-eval/src/field_to_block.rs` (`thread_one_slide`, ~lines 188-203) —
  reconcile `body` field handling to match `content.rs` schema (either reject consistently
  or accept consistently based on PO intent). NOTE: this is the eval crate, NOT
  slideforge-validate; the previous citation `crates/slideforge-validate/src/
  field_to_block.rs:135` was incorrect (F-098-P1-006).
- `crates/slideforge-eval/src/` (chart validation path) — verify E-LAY-003 is emitted
  and intercepted before ChartRenderer is invoked when data is empty.
- Test files for both crates.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `slideforge-plugin-api/src/slide_types/registry.rs` | ~3,000 |
| `slideforge-eval/src/field_to_block.rs` (thread_one_slide ~188-203) | ~2,000 |
| Chart validation path across validate + eval | ~3,000 |
| `error-taxonomy.md` (if E-VAL-105 needed) | ~1,000 |
| Test files | ~2,500 |
| **Total** | **~13,500** |

## Acceptance Criteria

### AC-001: Strict mode exits non-zero when shape: field is silently dropped (W-VAL-103 content-drop sub-case)
(traces to BC-3.03.002 v1.2 postcondition 3 + Invariant 4 — Route A: W-VAL-103 {shape} sub-case = broken/exit-2 in strict mode; error-taxonomy v2.29)

`slideforge build deck.sf` (strict mode, default) where `deck.sf` contains a `shape:`
field on a slide type that does not support it exits with code 2, NOT code 0.
The W-VAL-103 message format is unchanged: `Unknown field 'shape' for slide type '<type>'...`.
The severity is `broken` because the field key is `"shape"` (in the content-drop set
`{"shape", "body"}`). This is Route A: context-sensitive severity at accumulation time
in `validate_fields` (`crates/slideforge-plugin-api/src/slide_types/registry.rs`).
No new error code E-VAL-105 is introduced (Route B was rejected — see error-taxonomy v2.29).

Verified by: unit test in `slideforge-validate`; build a deck with `shape:` on a slide type
that does not support it; assert exit 2; assert W-VAL-103 message in stderr with unchanged format.

### AC-002: body field on content slide renders prose content (schema-VALID — PO adjudication F-098-ADJ-BODY-CONTENT)
(traces to BC-3.03.002 v1.3 EC-007 REVERSED + BC-4.01.001 v1.2 PC-11)

`slideforge build deck.sf` (strict mode) where a `content:` slide has a `body:` field
builds successfully: exit 0, output written, NO W-VAL-103 emitted. The `body` prose text
appears in the slide's content area (PPTX body placeholder / DOCX Normal paragraph) per
BC-4.01.001 PC-11. `body` is a declared known field on the `content` type — it is NOT
an unknown field and does NOT trigger the W-VAL-103 content-drop path.

This closes the body/content schema drift (F-098-P1-006): after this story, `body` on
`content` type is consistently ACCEPTED by both `crates/slideforge-eval/src/field_to_block.rs`
(`thread_one_slide`, F-098-P1-002 `slide_type_supports_body` gate) and
`slideforge-plugin-api/registry.rs::validate_fields` (body in known_fields, no W-VAL-103).

Verified by: unit test with a `content:` slide carrying `body: "Prose text"` in strict mode;
assert exit 0; assert NO W-VAL-103 in stderr; assert LaidOutDeck contains a Body-tagged frame
with the prose text.

Note: `body` on slide types that do NOT declare `body` in their known_fields (e.g., `quote`,
`title`, `stat_callout`) still triggers W-VAL-103 broken/exit-2 in strict mode — that
behavior is unchanged and tested by AC-001 (shape: case) and the separate known-bad-field
test required by this story.

### AC-003: body field on content slide is consistent across validator and eval — resolution (a) confirmed
(traces to BC-3.03.002 v1.3 Invariant 4 — known_fields() is the authority)

PO adjudication F-098-ADJ-BODY-CONTENT resolves this to option (a): `body` is
schema-valid for `content`. Both codepaths now accept `body` on `content`:
- `crates/slideforge-plugin-api/src/slide_types/content.rs` declares `body` in
  `ContentSlideType::optional` fields.
- `crates/slideforge-syntax/src/known_fields.rs` includes `"body"` in the `content` arm.
- `crates/slideforge-eval/src/field_to_block.rs` (`thread_one_slide`) gates body
  threading on `slide_type_supports_body` (uses `known_fields()`), so body is threaded
  for `content` and NOT threaded for types that do not declare `body`.
- `slideforge-plugin-api/registry.rs::validate_fields` does not emit W-VAL-103 for
  `body` on `content` because `body` is in `content`'s known_fields.

Verified by: unit test asserting that a content slide with `body:` field passes
validation (no W-VAL-103) AND that the threaded LaidOutDeck contains a Body-tagged
frame. Also: unit test asserting that `body:` on a type that does NOT declare it (e.g.,
`quote`) still triggers W-VAL-103 broken/exit-2 (regression guard for AC-001 logic).

### AC-004: Chart with no data emits E-LAY-003 and non-zero exit in strict mode
(traces to BC-1.11.002 v1.2 postcondition 2 — PO adjudication F-098-P1-007)

`slideforge build deck.sf` (strict mode) where a `slide chart:` block has a data binding
that evaluates to an empty collection exits with code 2 and emits:
`[E-LAY-003] Chart data is empty for slide '<title>'. Rendering error-slide placeholder.`
The chart renderer is never invoked.

BC-1.11.002 v1.2 clarification (F-098-P1-007 PO adjudication): a chart with a missing
`data:` field (no binding at all) is treated identically to a binding that evaluates to
an empty collection — both trigger the E-LAY-003 path. The message is self-prefixed with
the error code (`[E-LAY-003] ...`) per error-taxonomy v2.31.

Verified by: (a) unit test with empty-evaluating data binding — assert exit 2, `[E-LAY-003]`
in stderr, ChartRenderer.render() NOT called; (b) unit test with missing `data:` field —
same assertions. Assert ChartRenderer.render() mock is NOT called in both cases.

### AC-005: Chart with no data shows error-slide placeholder in warn-only mode
(traces to BC-1.11.002 v1.2 postcondition 3 — PO adjudication F-098-P1-007)

`slideforge build deck.sf --warn-only` with an empty chart data binding (or a missing
`data:` field — treated identically per BC-1.11.002 v1.2): `[E-LAY-003] Chart data is
empty for slide '<title>'. Rendering error-slide placeholder.` emitted as warning; an
error-slide placeholder renders at the chart slide's position; build exits 0.

BC-1.11.002 v1.2 clarification: missing `data:` and empty-evaluating `data:` are both
handled by this same path. Message is self-prefixed `[E-LAY-003]` per error-taxonomy v2.31.

Verified by: (a) unit test with `--warn-only` + empty data — assert exit 0, placeholder
in output, `[E-LAY-003]` in stderr; (b) unit test with `--warn-only` + missing `data:` —
same assertions.

## Tasks

- [ ] **T-001:** Read error-taxonomy.md to understand W-VAL-103 vs E-VAL-105 routing decision.
- [ ] **T-002:** Read BC-3.03.002 and BC-1.11.002 in full before writing any code.
- [ ] **T-003 (RED):** Write `test_shape_content_drop_strict_exits_2()` in slideforge-validate.
- [ ] **T-004 (RED):** Write `test_body_on_content_renders_and_exits_0()` — asserts body prose renders on content slide, exit 0, no W-VAL-103. (REPLACED from original exit-2 test per PO adjudication F-098-ADJ-BODY-CONTENT.)
- [ ] **T-005 (RED):** Write `test_chart_empty_data_strict_exits_2()`.
- [ ] **T-006 (RED):** Write `test_chart_empty_data_warn_only_placeholder()`.
- [ ] **T-007 (GREEN):** Promote W-VAL-103 to exit-2 for content-drop cases in strict mode
  (or register E-VAL-105 if reclassification is rejected).
- [ ] **T-008 (GREEN):** Confirm `body`/`content` schema drift is resolved via resolution (a): `body` declared in ContentSlideType.optional, known_fields("content"), and thread_one_slide gated on slide_type_supports_body. Verify no W-VAL-103 emitted for body-on-content. (PO adjudication F-098-ADJ-BODY-CONTENT.)
- [ ] **T-009 (GREEN):** Wire E-LAY-003 emission and validator interception before
  ChartRenderer invocation for empty-data charts.
- [ ] **T-010:** Update error-taxonomy.md if E-VAL-105 is added.
- [ ] **T-011:** Run `cargo nextest run -p slideforge-validate -p slideforge-plugin-api -p slideforge-eval --no-fail-fast`.
- [ ] **T-012:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Unknown field that is not a shape/body (e.g., unknown metadata) | W-VAL-103 cosmetic/exit 0 (non-content-drop warning, no change) |
| EC-002 | shape: field on a slide type that explicitly supports it | No warning; shape renders correctly |
| EC-003 | Chart with non-empty data | Normal rendering; no E-LAY-003 |
| EC-004 | Multiple validation errors including W-VAL-103 content drop | All errors reported; exit 2 |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-3.03.002 | Strict Mode Produces No Output on Validation Error | AC-001, AC-002, AC-003 |
| BC-1.11.002 | Chart with Empty Data Produces Error-Slide Placeholder | AC-004, AC-005 |

## Test Strategy

TDD strict mode. Four failing tests first. The W-VAL-103 reclassification decision must
be made in T-001 before writing any test (it affects what code path to test). Document
the decision in the PR description.
