---
title: "STORY-050 E2E Test Gap Analysis"
created: 2026-06-05
status: DRAFT
author: architect
traces_to: BC-5.02.001
---

# STORY-050 E2E Test Gap Analysis

7 of 62 E2E tests fail. This document provides root-cause analysis, design-correct
fix, scope verdict, and blast radius for each gap. No BC or story files are modified
here — those changes are flagged for PO and story-writer action at the end.

---

## Gap 1 — PDF Export Fails: `NoDocumentTitle` (AC-003, AC-008 PDF, EC-005)

### Failing Tests (4)
- `e2e_pipeline_pdf::test_bc_5_02_001_ac003_pdf_build_returns_ok_with_pdf_header`
- `e2e_pipeline_pdf::test_bc_5_02_001_ac008_pdf_page_count_matches_slide_count`
- `e2e_multi_format::test_bc_5_02_001_ac008_multi_format_all_three_produce_valid_output`
- `e2e_multi_format::test_bc_5_02_001_ec005_all_formats_built_from_single_fixture`

### Exact Error
```
Export(RenderError { message: "PDF validation error: PDF/UA-1 validation
failed: NoDocumentTitle" })
```

### Root Cause (grounded)

**eval.rs:441** (slideforge-eval):
```rust
let title = None; // Title is not present in DeckNode (comes from a slide); leave None.
```
`eval_deck_with_variant` always sets `DeckMetadata.title = None`.

**exporter.rs:505-511** (slideforge-pdf):
```rust
// Build the metadata object: language is always present (guarded above);
// title is wired when present.
let mut meta = Metadata::new().language(lang.as_ref().to_owned());
if let Some(title) = &deck.metadata.title {
    meta = meta.title(title.as_ref().to_owned());
}
document.set_metadata(meta);
```
The PDF exporter sets `Metadata.title` only when `deck.metadata.title` is `Some`.
When `title` is `None`, no document title is placed in the krilla metadata dictionary.

krilla's `Validator::UA1` enforces ISO 14289-1 §7.1, which requires a non-empty
document title in the PDF metadata stream. When the title is absent, krilla returns
`KrillaError::Validation([..NoDocumentTitle..])` → `PdfExportError::ValidationFailed`
→ `ExportError::RenderError`.

**Result:** PDF export is completely non-functional for any deck — every `build(...,
format="pdf")` call fails.

### Design-Correct Source of Document Title

The DSL spec (`dsl-spec.md:33-38`) shows a `metadata:` block with `title "..."`.
However, `DeckNode` (slideforge-syntax/src/ast.rs:513-545) has **no `title` field**
and the parser has **no `metadata:` block implementation** — `metadata:` block parsing
is an unimplemented feature.

There are two viable sources, ordered by correctness per the DSL design:

1. **Derive from the first `slide title:` block's `title` field** — the test fixture
   `test-3slide.sf` has `slide title:` with `title "Incident Response Review"`. This
   is the pragmatic path: the deck's presentation title is the first title slide's
   `title` field, which is already in `Slide.fields["title"]` (a `Value::Str`).
   This requires extracting `slides[0].fields["title"]` in `eval_deck_with_variant`
   when `deck_node` has no explicit title source.

2. **Implement the `metadata: title "..."` DSL feature** — the correct design-complete
   path, adding a `title` field to `DeckNode` and teaching the parser to parse
   `metadata:` blocks. This is a larger scope change touching slideforge-syntax.

For STORY-050's purposes (unblock PDF export end-to-end), option 1 is the minimum
correct fix. Option 2 is the correct long-term design that follows the spec.

### Scope Verdict

**IN-SCOPE for STORY-050 with a targeted fix in `slideforge-eval`.**

Rationale under the Production-Grade Default: PDF has been non-functional for every
deck since the PDF exporter was merged (STORY-044). This is a genuine product defect
in merged code, not a missing feature. The fix is contained to a single function in
`slideforge-eval/src/eval.rs` (eval_deck_with_variant, ~5 lines).

The fix: after evaluating slides, set `DeckMetadata.title` by scanning `slides` for
the first slide with a non-empty `title` field (as a `Value::Str`). If none is found,
derive a fallback from the DSL version field (e.g., `"Untitled Presentation"`). This
matches the intent of the spec comment at eval.rs:441 ("comes from a slide").

The `metadata:` block DSL feature (parser + DeckNode.title) is a larger scope addition
that should be anchored to a dedicated story (see Defer section below).

### Blast Radius

- **slideforge-eval** — `eval_deck_with_variant` in `eval.rs` (1 change)
- No changes to slideforge-pdf, slideforge-types, or slideforge-syntax

### Flag for PO / Story-Writer

- BC-5.02.001 should explicitly state that `DeckMetadata.title` is populated by
  the evaluator from the first title slide's `title` field (not only from a `metadata:`
  block). The `metadata:` block feature should be a distinct story.
- Story-writer should create a new story: "DSL `metadata:` block with `title`,
  `author`, `date` fields — parser + eval + export threading" (Wave 3+).

---

## Gap 2 — AC-009 Alt-Text Validation Unreachable Through DSL→Eval Path

### Failing Tests (2)
- `e2e_error_propagation::test_bc_5_02_001_ac009_missing_alt_returns_validation_failed`
- `e2e_error_propagation::test_bc_5_02_001_ac009_validation_failed_contains_e_a11_001`

### Exact Behavior
`build()` returns `Ok(BuildOutput)` instead of `Err(BuildError::ValidationFailed)` for
`test-missing-alt.sf`, which contains:
```
slide chart:
  title "Revenue Trend"
  chart_type "bar"
  data "revenue.json"
```

### Root Cause (grounded)

**slideforge-eval/src/for_eval.rs:336-342** (`eval_slide_node`):
```rust
// Block-level content (images, charts, diagrams, shapes) is populated
// by later pipeline stages (layout, PPTX generation, Waves 3+). In
// Wave 2, the evaluator only resolves field values.
blocks: vec![],
```

`Slide.blocks` is always `vec![]` after eval. `AltTextValidator` in
`slideforge-validate/src/alt_text.rs:57-129` iterates `slide.blocks` —
if blocks is empty, no `ContentBlock::Chart` can be found, so no `E-A11-001`
is ever emitted.

The fixture's `chart_type "bar"` and `data "revenue.json"` are parsed as scalar
field values in `Slide.fields` (an `OrderedMap`), not as `ContentBlock::Chart`.
The eval stage intentionally defers `ContentBlock` construction to the layout stage.
The layout stage (`slideforge-layout`) does produce `ContentBlock::Chart` frames,
but **validators run on the pre-layout `Deck`** (lib.rs:441-463 validate stage runs
before `layout_run`). By the time `ContentBlock::Chart` exists (post-layout), the
strict validation gate has already passed.

This is a **fundamental pipeline sequencing gap**: the validator is being called
with a `Deck` in which `blocks: vec![]` for every slide, so it can never detect
missing alt text through the live DSL→eval→validate path.

### Is this a Feature Gap or Wrong DSL Syntax?

Feature gap. The fixture DSL is correct per the slide type catalog (`slide chart:`
is a valid type). The issue is architectural: alt-text validation requires
`ContentBlock` objects, which are only produced at layout time, but validators
run pre-layout. Two sub-paths:

**Sub-path A (in-scope fix):** Move alt-text validation to post-layout by inspecting
`LaidOutDeck` instead of `Deck`. This requires a new validator hook or a post-layout
validation pass. This is a significant architecture change touching
`slideforge-plugin-api` (Validator trait signature takes `&Deck`, not `&LaidOutDeck`).
**Not in scope for STORY-050.**

**Sub-path B (in-scope, correct for AC-009 INTENT):** The test fixture
`test-missing-alt.sf` can be updated so the missing-alt assertion is triggered via a
path that IS reachable — specifically, the `ZeroSlideValidator` (E-LAY-002) path is
already reachable (proven by EC-001's passing test). However, AC-009 specifically
tests `E-A11-001`, not `E-LAY-002`.

**Sub-path C (correct architectural fix — deferred):** Compute alt-text metadata
during eval from DSL fields (`alt "..."` keyword parsed into `Slide.fields`) and
populate a partial/stub `ContentBlock` list during eval. This requires story-level
scope for the alt-field-at-eval story.

### v1 Risk Assessment

This is a HIGH-severity v1 product risk:
- `alt "..."` is declared a **compile error if absent** in CLAUDE.md (Accessibility section)
- WCAG AA enforcement is a non-negotiable quality bar gate
- The AC-009 test failure proves the enforcement is completely bypassed end-to-end

A user can ship a deck without any alt text and the build succeeds silently. The test
`test_bc_5_02_001_ac009_warn_only_does_not_return_validation_failed` (which passes) masks
this: `strict=false` correctly passes, but `strict=true` also passes — it should fail.

This risk must be surfaced to the human for authorization before proceeding.

### Scope Verdict

**NEEDS-HUMAN-AUTHORIZATION** on the architectural fix (validator timing).

The correct fix requires one of:
1. Post-layout validation pass (new pipeline stage / trait signature change)
2. Partial ContentBlock construction during eval for alt-bearing DSL fields

Both options require architectural decisions that expand story scope. STORY-050
cannot close AC-009 with a purely in-scope fix without architectural guidance.

**Recommended interim position for STORY-050:** Replace the two failing AC-009 tests
with a test that documents the known gap (`#[ignore]` with comment citing the
blocking story), plus a unit test that confirms `AltTextValidator` DOES fire when
`ContentBlock::Chart(ChartSpec { alt: None, ... })` is present in the deck's slide
blocks (this works — the validator logic is correct; the gap is that eval never
produces blocks). The human must authorize this deferral and anchor it to a story.

### Which Story Owns the Fix?

The chart/eval/ContentBlock story. The story-writer should create:
"Eval stage: populate `Slide.blocks` with `ContentBlock::Chart`/`Image`/`Diagram`
from DSL fields when `alt`, `chart_type`, `data`, `src` scalar fields are parsed"
(Wave 2 or 3, must precede PDF/PPTX chart rendering stories).

### Blast Radius (for the correct fix)

- **slideforge-eval** — `eval_slide_node` in `for_eval.rs` (ContentBlock construction)
- **slideforge-types** — Deck/Slide (no change, just needs blocks populated)
- **slideforge-validate** — `AltTextValidator` (no change needed; logic is correct)
- **STORY-050 test suite** — `error_propagation.rs` (2 tests need #[ignore] + anchor)

---

## Gap 3 — AC-007 Observability: `tracing_test` Subscriber Does Not Capture INFO Events

### Failing Tests (1)
- `e2e_observability::test_bc_5_02_001_ac007_all_pipeline_stage_markers_emitted`

### Exact Error
```
AC-007: at least one 'build_inner' log message must appear; if this fails,
tracing_test is not capturing INFO events from the slideforge crate. This is
a test-infrastructure gap: add RUST_LOG=slideforge=info to capture INFO events.
```

### Root Cause (grounded)

**build_inner in lib.rs:333-338** emits `tracing::info!` events:
```rust
tracing::info!(
    stage = "pipeline_start",
    format,
    strict = options.strict,
    "build_inner: starting pipeline"
);
```

The `#[tracing_test::traced_test]` macro installs a tracing subscriber that captures
events. By default, `tracing-test` captures WARN+ events; INFO-level events from
library crates require explicit `RUST_LOG=slideforge=info` or equivalent filter
configuration at subscriber construction time.

The production code is correct: 8 `tracing::info!` events are emitted with the
right stage names (as documented in the `observability.rs` module header). The gap
is that the test subscriber doesn't see INFO events from the `slideforge` crate
without additional filter setup.

### NFR-032 Misalignment: Events vs Spans

NFR-032 (`nfr-catalog.md:112`) specifies "6 SPANS" with names
`parse/evaluate/brand/validate/layout/export`. The implementation has 8 INFO
**events** (not spans) with slightly different names:
`pipeline_start/brand_load/parse/eval/validate/inject_lang_default/layout/export`.

Key differences:
- Count: 8 events vs. 6 spans
- `brand_load` vs. `brand` (name mismatch)
- `eval` vs. `evaluate` (name mismatch)
- `pipeline_start` and `inject_lang_default` are extra events not in the spec
- Events have no timing/enter/exit semantics — spans do

### Scope Verdict

**IN-SCOPE for STORY-050 with a two-part fix:**

**Part 1 — Fix test subscriber configuration (test infrastructure, small):**
Update `observability.rs` to configure the `tracing_test` subscriber to capture
INFO events from the `slideforge` crate. `tracing-test` supports this via
`RUST_LOG` environment variable or subscriber filter initialization. The fix:
set `RUST_LOG=slideforge=info` via `std::env::set_var` before the test (or use
`tracing_test`'s filter parameter if available). This is a test-only change in
`tests/e2e/observability.rs`.

**Part 2 — Align with NFR-032 (small targeted change in lib.rs):**
Promote the 6 canonical pipeline stages from `tracing::info!` events to
`tracing::info_span!` spans — or at minimum make the stage names match
(`brand_load` → `brand`, `eval` → `evaluate`). The extra events
(`pipeline_start`, `inject_lang_default`) can remain as INFO events in addition
to the spans. This is a small change in `slideforge/src/lib.rs`.

The spec (`observability.rs` module doc) already documents the discrepancy and flags
it for orchestrator decision. The STORY-050 test has self-correctly adapted to test
the 8-event reality. The fix is to make the test infrastructure work.

Recommend Option A (most faithful to NFR-032): Change `build_inner` to use
`tracing::info_span!` for the 6 canonical stage names and keep the INFO events
as sub-events. Fix the test subscriber to use INFO level. No BC/spec change needed
if the test spec is updated to match 8 events (the test already does this).

### Blast Radius

- **slideforge** — `lib.rs` `build_inner` (events → spans for 6 canonical stages)
- **slideforge** — `tests/e2e/observability.rs` (subscriber filter fix)

---

## Gap 4 — API Ergonomics: `BuildOptions` Constructors and Multi-Format API

### Failing Tests

None — this is documented in the test code itself (`multi_format.rs:12-17`) as a
known reconciliation without a dedicated failing test. The 4 PDF failures and
2 AC-009 failures dominate. This gap is informational.

### Root Cause (grounded)

The STORY-050 spec referenced `BuildOptions::pptx()`, `BuildOptions::docx()`,
`BuildOptions::pdf()`, `BuildOptions::all_formats()` factory methods and
`BuildError::ParseErrors`/`ValidationErrors`/`DataErrors` variant names.

The actual API (`slideforge/src/lib.rs:189-227`, `error.rs:69-190`) has:
- `BuildOptions` is a plain struct with `Default` — no factory methods
- Variant names: `ParseFailed` (not `ParseErrors`), `ValidationFailed` (not
  `ValidationErrors`), no `DataErrors`
- `build()` is single-format — no `build_all()` multi-format API

The test code already reconciles this correctly: it uses `BuildOptions { format:
Some(...), ..Default::default() }` struct literal syntax and calls `build()` three
times for three formats. The `multi_format.rs` module doc explicitly documents the
discrepancy and flags it for orchestrator decision.

### Scope Verdict

**DEFER to anchored story (small ergonomic addition).**

The missing factory methods (`BuildOptions::pptx()` etc.) are pure ergonomic sugar
with no behavioral impact. Adding them is a low-risk, low-value change for STORY-050.
The test suite already works without them by using struct-literal syntax.

The story-writer should add these as a small task to the first story that modifies
the `slideforge` public API (likely a CLI story or a Wave 3 DX polish story). This
is PO/story-writer scope.

**Flag for PO:** AC-008 says "build_all() or equivalent returns all formats." This
BC language should be updated to reflect the three-separate-calls design (which is
already working). PO should amend BC-5.02.001 AC-008 postcondition to say "three
separate `build()` calls" rather than implying a single `build_all()`.

### Blast Radius (if added later)

- **slideforge** — `lib.rs` (add factory methods, ~10 lines)

---

## Summary Table

| Gap | Root Cause (file:line) | Design-Correct Fix | Scope Verdict | Blast Radius |
|-----|----------------------|-------------------|---------------|-------------|
| 1 — PDF `NoDocumentTitle` | `eval.rs:441` `title = None` always | Derive title from first title-slide's `title` field in `eval_deck_with_variant` | **IN-SCOPE** (slideforge-eval fix) | slideforge-eval only |
| 2 — Alt-text unreachable | `for_eval.rs:342` `blocks: vec![]` + validator runs pre-layout | Post-layout validation or eval-time ContentBlock construction | **NEEDS-HUMAN-AUTHORIZATION** (architectural) | slideforge-eval, plugin-api (trait), STORY-050 tests |
| 3 — Tracing not captured | `tracing_test` default filter = WARN; `build_inner` emits INFO; also events vs. spans mismatch vs NFR-032 | Fix test subscriber filter; promote 6 stages to spans in `lib.rs` | **IN-SCOPE** (lib.rs + test infra) | slideforge lib.rs, e2e observability.rs |
| 4 — API ergonomics | No `BuildOptions::pptx()` factory methods; no `build_all()` | Add convenience constructors; update BC-5.02.001 AC-008 language | **DEFER** (story-writer + PO action) | slideforge lib.rs (future story) |

---

## Overall Recommendation

**STORY-050 can converge with the following combination:**

1. **IN-SCOPE fixes (implementer should fix before PR merge):**
   - Gap 1: ~5-line fix in `slideforge-eval/src/eval.rs` to derive `title` from
     first title-slide's `title` field. Closes 4 failing PDF tests.
   - Gap 3: `lib.rs` events-to-spans promotion for 6 canonical stages; test
     subscriber filter fix for INFO capture. Closes 1 failing observability test.

2. **NEEDS-HUMAN-AUTHORIZATION before STORY-050 can close AC-009 cleanly:**
   - Gap 2: Human must decide between (A) post-layout validation pass, (B) eval-time
     ContentBlock stub construction, or (C) documented #[ignore] with story anchor.
   - Without authorization, the 2 AC-009 tests cannot pass and STORY-050 is blocked.

3. **DEFERRED (no STORY-050 action required):**
   - Gap 4: API ergonomics to a future story. Existing test suite already works.

**Total remaining failing tests after in-scope fixes:** 2 (AC-009 tests blocked by
Gap 2). If the human authorizes the `#[ignore]` + story-anchor path for AC-009,
all 7 tests are resolved and STORY-050 can converge.

**Severity note on Gap 2:** The alt-text accessibility guarantee being completely
non-functional end-to-end is a CRITICAL v1 product quality risk. The human should
explicitly authorize how this is handled before Phase 4 (Holdout Evaluation) runs.

---

## Flag for PO (Spec Changes Required)

The following BC/story changes are flagged for PO/story-writer action. This
analysis document does not modify them.

1. **BC-5.02.001 AC-003, AC-008 (PDF):** Add postcondition: "PDF export requires
   `deck.metadata.title` to be non-None. The evaluator derives it from the first
   title slide's `title` field when no `metadata: title` block is present."

2. **BC-5.02.001 AC-008 (multi-format):** Update language from "build_all()" to
   "three separate build() calls — one per format." Remove reference to
   `BuildOptions::all_formats()`.

3. **BC-5.02.001 AC-009 (alt-text):** Flag for architectural resolution: validator
   runs pre-layout on `Deck` with `blocks: vec![]`; AC-009 cannot currently be
   satisfied end-to-end. PO must create the anchored story for eval-time
   ContentBlock construction or post-layout validation pass.

4. **NFR-032 (Observability):** Reconcile "6 spans" with the actual 8-event
   implementation. Recommend updating NFR-032 to say "8 pipeline stage markers
   (6 spanning the primary stages + 2 bookkeeping events)" or direct the implementer
   to promote to spans.
