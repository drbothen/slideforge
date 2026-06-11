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
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-005, REND-010]
behavioral_contracts: [BC-3.03.002, BC-1.11.002, BC-3.03.001]
# BC status: BC-3.03.002 (strict mode produces no output on validation error — W-VAL-103
# currently exits 0 while content is silently dropped; this violates strict-default invariant
# and DI-017). BC-1.11.002 (chart with empty data produces error-slide placeholder — REND-010a
# shows chart with no data renders silently empty, violating BC-1.11.002 postconditions 1/2).
# BC-3.03.001 (canvas overflow warning path — referenced for body/content schema drift).
# All BCs are authored.
#
# REND-005 secondary root cause (body/content schema drift): validate vs eval inconsistency
# for `body` field on `content` slide type. slideforge-validate/src/field_to_block.rs:135
# passes `body` through while content.rs schema rejects it. Fix: reconcile.
verification_properties: []
nfr_refs: []
closes_findings: [REND-005, REND-010]
depends_on:
  - STORY-016
  - STORY-032
  - STORY-089
blocks: []
target_module: slideforge-validate, slideforge-eval, slideforge-plugin-api
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
3. `body` field on `content` slide type: validator passes it (field_to_block.rs:135)
   but content.rs schema rejects it — inconsistency lets some `body` content through
   while rejecting it in other paths.

## Architecture Compliance Rules

- Per BC-3.03.002 invariant 1 (DI-017): all-or-nothing in strict mode. Silent content
  drop IS a validation error in strict mode; W-VAL-103 must be promoted to exit-2-triggering
  when the dropped content is user-authored (shape: or body fields on unsupported types).
- Per error-taxonomy (W-VAL-103 Note, v2.20): W-VAL-103 is "cosmetic/exit 0" as a
  formal registration. Changing this requires either: (a) reclassifying W-VAL-103 to
  broken/exit 2, or (b) introducing E-VAL-105 for the content-drop sub-case. Route (a) is
  simplest; if PO objects to reclassifying the general unknown-field warning, use route (b).
  Document the chosen route in the story. The human directive says no deferral — pick and
  implement.
- Per BC-1.11.002 invariant 2: the ChartRenderer plugin MUST NOT be called with empty
  data. The validator intercepts before plugin invocation.
- No new E-* codes needed unless route (b) above is taken; in that case register E-VAL-105
  in error-taxonomy.md before implementing.
- `slideforge-validate` MUST NOT bypass the existing error-accumulation model (DI-018).

## Library & Framework Requirements

- No new dependencies. Internal crates: `slideforge-plugin-api`, `slideforge-validate`,
  `slideforge-eval`.

## File Structure Requirements

Files to modify:
- `crates/slideforge-plugin-api/src/slide_types/registry.rs` — promote W-VAL-103 (or
  add E-VAL-105) for content-drop fields in strict mode.
- `crates/slideforge-validate/src/field_to_block.rs:135` — reconcile `body` field
  handling to match `content.rs` schema (either reject consistently or accept consistently
  based on PO intent).
- `crates/slideforge-validate/src/` (chart validation path) — verify E-LAY-003 is
  emitted and intercepted before ChartRenderer is invoked when data is empty.
- `crates/slideforge-eval/src/` — confirm ChartRenderer not called with empty data.
- Test files for both crates.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `slideforge-plugin-api/src/slide_types/registry.rs` | ~3,000 |
| `slideforge-validate/src/field_to_block.rs` | ~2,000 |
| Chart validation path across validate + eval | ~3,000 |
| `error-taxonomy.md` (if E-VAL-105 needed) | ~1,000 |
| Test files | ~2,500 |
| **Total** | **~13,500** |

## Acceptance Criteria

### AC-001: Strict mode exits non-zero when shape: field is silently dropped (W-VAL-103)
(traces to BC-3.03.002 postcondition 3 — exit code 2 in strict mode on validation error)

`slideforge build deck.sf` (strict mode, default) where `deck.sf` contains a `shape:`
field on a slide type that does not support it exits with code 2, NOT code 0. All
authored content is either emitted or the build fails — no silent drops.

Verified by: unit test in `slideforge-validate`/`slideforge-cli` integration: build a
deck with an unsupported `shape:` field; assert exit code == 2 and error message cites
the dropped content.

### AC-002: Strict mode exits non-zero when body field silently dropped on content slide
(traces to BC-3.03.002 postcondition 3)

`slideforge build deck.sf` (strict mode) where a `content:` slide has a `body:` field
(schema-invalid for `content` type) exits with code 2, NOT code 0.

Verified by: unit test with a `content:` slide carrying `body:` field; assert exit 2.

### AC-003: Reconcile body/content schema drift
(traces to BC-3.03.002 invariant 2 — strict mode is the default)

The behavior of `body` on `content` slide type is consistent between `field_to_block.rs`
and `content.rs` schema. Either: (a) `body` is schema-valid for `content` (validator
allows it, eval processes it) — in which case the schema is extended; or (b) `body` is
schema-invalid (validator and eval both reject it consistently). The PO must confirm the
intended behavior. If (b), AC-002 covers this. Document the chosen resolution in the
story's commit message.

Verified by: add a unit test asserting consistent behavior; the test passes in both
validator and eval codepaths.

### AC-004: Chart with no data emits E-LAY-003 and non-zero exit in strict mode
(traces to BC-1.11.002 postcondition 2)

`slideforge build deck.sf` (strict mode) where a `slide chart:` block has a data binding
that evaluates to an empty collection exits with code 2 and emits:
`E-LAY-003: Chart data is empty for slide '<title>'. Rendering error-slide placeholder.`
The chart renderer is never invoked.

Verified by: unit test building a deck with a chart where data binding returns []; assert
exit 2 and E-LAY-003 message in stderr. Assert ChartRenderer.render() mock is NOT called.

### AC-005: Chart with no data shows error-slide placeholder in warn-only mode
(traces to BC-1.11.002 postcondition 3)

`slideforge build deck.sf --warn-only` with an empty chart data binding: E-LAY-003
emitted as warning; an error-slide placeholder renders at the chart slide's position;
build exits 0.

Verified by: unit test with `--warn-only`; assert exit 0, placeholder in output.

## Tasks

- [ ] **T-001:** Read error-taxonomy.md to understand W-VAL-103 vs E-VAL-105 routing decision.
- [ ] **T-002:** Read BC-3.03.002 and BC-1.11.002 in full before writing any code.
- [ ] **T-003 (RED):** Write `test_shape_content_drop_strict_exits_2()` in slideforge-validate.
- [ ] **T-004 (RED):** Write `test_body_on_content_strict_exits_2()`.
- [ ] **T-005 (RED):** Write `test_chart_empty_data_strict_exits_2()`.
- [ ] **T-006 (RED):** Write `test_chart_empty_data_warn_only_placeholder()`.
- [ ] **T-007 (GREEN):** Promote W-VAL-103 to exit-2 for content-drop cases in strict mode
  (or register E-VAL-105 if reclassification is rejected).
- [ ] **T-008 (GREEN):** Reconcile `body`/`content` schema drift.
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
