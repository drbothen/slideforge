---
document_type: adr
adr_id: ADR-018
title: Post-layout validation pass for ContentBlock-level accessibility checks
status: accepted
date: 2026-06-05
accepted_date: 2026-06-05
version: "1.1"
amended: 2026-06-05
subsystems_affected: [SS-03, SS-05, SS-14]
supersedes: null
superseded_by: null
context: >
  STORY-050 gap analysis (Gap 2) established that AltTextValidator never fires
  because Slide.blocks is always vec![] after eval; ContentBlock::Chart/Image/Diagram
  are created only at layout time. Human authorized Option A (post-layout validation
  pass) on 2026-06-05. This ADR records the selected design and implementation contract.
  v1.1 amendment (2026-06-05): adds Decision 5a — error-precedence rule when
  layout::run fails and pre-layout Error-severity diagnostics exist.
human_gate_required: false
human_gate_resolved: >
  Human authorized Option A (post-layout validation pass) on 2026-06-05. No further
  human gates required before implementation.
traces_to: ARCH-INDEX.md
---

# ADR-018: Post-Layout Validation Pass for ContentBlock-Level Accessibility Checks

## Status

**ACCEPTED.** Human authorized Option A (post-layout validation pass) on 2026-06-05.
Implementation may proceed without further authorization.

## Amendment Log

| Date | Version | Author | Change |
|------|---------|--------|--------|
| 2026-06-05 | v1.0 | architect | Initial ADR. Decisions 1–5 (post-layout validation pass, additive-defaulted `validate_post_layout`, AltTextValidator migration, validator classification rule, strict-mode aggregation contract). |
| 2026-06-05 | v1.1 | architect | Added Decision 5a: error-precedence rule — when `layout::run` returns `Err` and pre-layout Error-severity diagnostics exist, `build()` returns `BuildError::ValidationFailed` (not `BuildError::Layout`). Root-cause reporting: validation errors take precedence over downstream layout symptoms. This rule was implemented during STORY-050 fix burst; codified here to prevent regression in future refactors and to make EC-001 traceability explicit. Implementation note wording corrected to match as-built STORY-050 worktree code: the pre-layout early-return gate was removed (OBS-5 fix); the single combined gate fires after Stage 6b, not before layout. |

## Context

slideforge guarantees that `alt "..."` is required on every visual element and
that its absence is a compile error (CLAUDE.md §Accessibility). This guarantee is
enforced by `AltTextValidator` in `slideforge-validate/src/alt_text.rs`, which
iterates `slide.blocks` looking for `ContentBlock::Chart`, `ContentBlock::Image`,
and `ContentBlock::Diagram` entries whose `alt` field is `None`.

The guarantee is non-functional end-to-end. The root cause is a **pipeline sequencing
gap** established across three files:

1. `slideforge-eval/src/for_eval.rs:342` — `eval_slide_node` always sets
   `blocks: vec![]`. The evaluator explicitly defers ContentBlock construction to
   later pipeline stages (Wave 3+) and records this in a comment.

2. `slideforge/src/lib.rs:441` — the validation gate runs immediately after eval,
   on the `Deck` whose slides all have `blocks: vec![]`. At this point, no
   `ContentBlock::Chart` or `ContentBlock::Image` exists anywhere in the IR.

3. `slideforge-layout/src/layout.rs:428` — `thread_media_alt_into_frames` (called
   during `layout::run`) creates `FrameContent::Chart`, `FrameContent::Image`, and
   `FrameContent::Diagram` entries in `LaidOutSlide.frames` — but this happens
   **after** the validate gate has already passed cleanly.

The consequence: a deck with `slide chart:` and no `alt "..."` builds to `Ok`
silently. The BC-5.02.001 §Accessibility guarantee ("alt required — compile error
if absent") is unenforceable through the current pipeline ordering, regardless of
the correctness of `AltTextValidator`'s internal logic. This was confirmed by the
two failing tests in STORY-050:

- `e2e_error_propagation::test_bc_5_02_001_ac009_missing_alt_returns_validation_failed`
- `e2e_error_propagation::test_bc_5_02_001_ac009_validation_failed_contains_e_a11_001`

The gap analysis (`.factory/specs/story-050-gap-analysis.md`) assessed three fix
paths and identified this as a HIGH-severity v1 product quality risk. Option A (a
post-layout validation pass) was selected and authorized by the human on 2026-06-05.

This ADR amends the pipeline stage ordering established by ADR-016 Decision 3.
ADR-016 Decision 3 places the validate gate as "Stage 5" in `build_inner`, running
on `&Deck` before layout. This ADR adds a new "Stage 6b" (post-layout pass) between
the existing layout stage and the export stage.

## Decision

### Decision 1: Add a post-layout validation pass as Stage 6b in build_inner

After `layout::run` succeeds (Stage 6) and before the exporter is invoked (Stage 7),
`build_inner` runs a second validation pass over the `LaidOutDeck`. This pass invokes
a new defaulted method `validate_post_layout` on every registered `Validator` plugin.

The post-layout pass is an **additive pipeline stage**. It does not replace or
modify Stage 5 (the pre-layout pass on `&Deck`). Both passes run in sequence, and
their diagnostics are accumulated together before the strict-mode gate is evaluated.

#### Pipeline stage ordering (updated)

```
Stage 1: parse (slideforge-syntax)
Stage 2: eval (slideforge-eval)
Stage 3: brand load (slideforge-brand)
Stage 4: inject lang default (slideforge-validate)
Stage 5: validate pre-layout — validate(&Deck, &ValidatorOptions) on all validators
Stage 6: layout (slideforge-layout) → LaidOutDeck
Stage 6b: validate post-layout — validate_post_layout(&LaidOutDeck, &ValidatorOptions)
          on all validators (NEW — ADR-018)
Stage 7: export
```

Stage 6b diagnostics are appended to the same `all_validator_diagnostics` vec used
by Stage 5. The strict-mode gate evaluates the combined diagnostic list exactly once,
after Stage 6b completes.

### Decision 2: Extend the Validator trait with a defaulted validate_post_layout method

The `Validator` trait in `slideforge-plugin-api/src/traits/validator.rs` gains a
second method with a complete default implementation:

```rust
/// Validate `laid_out` after the layout pass and return all diagnostics found.
///
/// This method is called by the pipeline AFTER `layout::run` has produced a
/// [`LaidOutDeck`]. It provides access to `FrameContent` variants
/// (`Chart`, `Image`, `Diagram`) that are only present in the geometric IR,
/// not in the semantic [`Deck`] passed to [`Validator::validate`].
///
/// The default implementation is a no-op (returns an empty `Vec`). Validators
/// that only need semantic-IR data (e.g., `ZeroSlideValidator`, `LangValidator`,
/// `OverflowValidator`) do not need to override this method. Validators that
/// check ContentBlock-level attributes — particularly accessibility validators
/// checking alt-text presence on visual elements — MUST override this method.
///
/// ## Contract
///
/// - Called once per `build()` invocation, after `layout::run` succeeds.
/// - Returns diagnostics with the same `Diagnostic` type as `validate()`.
/// - An empty `Vec` means the laid-out deck is clean for this validator.
/// - In strict mode, any `Error`-severity diagnostic returned from this method
///   causes `BuildError::ValidationFailed` (identical treatment to `validate()`).
/// - This method is called on every registered `Validator`, regardless of whether
///   that validator also overrides `validate()`.
///
/// ## Additive-defaulted extension
///
/// This is a defaulted method per BC-5.02.001 invariant 2 and ADR-017 Option A
/// precedent. Existing `Validator` implementations compile unchanged and inherit
/// the default no-op. No existing implementor is required to add code.
fn validate_post_layout(
    &self,
    laid_out: &LaidOutDeck,
    opts: &ValidatorOptions,
) -> Vec<Diagnostic> {
    let _ = (laid_out, opts); // default no-op
    vec![]
}
```

The `LaidOutDeck` type is imported from `slideforge-layout`. The
`slideforge-plugin-api` crate adds `slideforge-layout` as a dependency to expose
this type in the trait signature. See the dependency graph note in Consequences.

### Decision 3: AltTextValidator migrates to validate_post_layout

`AltTextValidator` in `slideforge-validate/src/alt_text.rs` moves its content-block
inspection logic from `validate()` to `validate_post_layout()`.

**Before (current — non-functional):**

```rust
impl Validator for AltTextValidator {
    fn validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic> {
        // iterates deck.slides[*].blocks — always vec![] after eval
        // never produces E-A11-001
    }
}
```

**After (ADR-018):**

```rust
impl Validator for AltTextValidator {
    fn validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic> {
        // Semantic-IR checks only: none currently for AltTextValidator.
        // Reserved for future checks on Slide.fields (e.g., slide-level
        // decorative opt-out field) before ContentBlock data is available.
        vec![]
    }

    fn validate_post_layout(
        &self,
        laid_out: &LaidOutDeck,
        opts: &ValidatorOptions,
    ) -> Vec<Diagnostic> {
        // Iterates laid_out.slides[*].frames, checks FrameContent::Chart,
        // FrameContent::Image, FrameContent::Diagram for AltText::Decorative
        // where the source slide did not explicitly set decorative: true.
        // Emits E-A11-001 for every missing alt text.
        // ...
    }
}
```

The `validate()` body for `AltTextValidator` becomes a no-op stub. This is correct:
the semantic `Deck.slides[*].blocks` are always `vec![]` after eval (for_eval.rs:342)
and will remain so until a future story populates them from eval-time DSL fields.
Any pre-layout alt-text check on `Deck.slides[*].blocks` would be dead code and
misleadingly silent.

### Decision 4: Validator classification rule

Validators are classified as pre-layout, post-layout, or both, based on what IR
data they need:

| Classification | Runs in | Receives | Use when |
|----------------|---------|----------|---------|
| Pre-layout only | Stage 5 | `&Deck` | Needs semantic slide fields, zero-slide count, lang presence, or structural invariants resolvable from field values |
| Post-layout only | Stage 6b | `&LaidOutDeck` | Needs ContentBlock-level data: alt text on visual elements, canvas overflow geometry, frame positioning |
| Both | Stage 5 + Stage 6b | Both | Needs both semantic and geometric data (e.g., a future validator checking slide title presence AND canvas overflow of that title's frame) |

**Current validator classification:**

| Validator | ID | Pre-layout (`validate`) | Post-layout (`validate_post_layout`) | Rationale |
|-----------|----|------------------------|--------------------------------------|-----------|
| ZeroSlideValidator | `"zero-slide"` | Yes (checks `deck.slides.len()`) | No (default no-op) | Needs semantic slide count only |
| LangValidator | `"lang"` | Yes (checks `deck.metadata.lang`) | No (default no-op) | Needs semantic metadata only |
| AltTextValidator | `"alt-text"` | No (becomes no-op) | Yes (checks `FrameContent` alt fields) | ContentBlock-level data; only available post-layout |

Future validators (CanvasOverflowValidator, ContrastValidator) are classified at the
time they are introduced, using this rule as the guide.

### Decision 5: Strict-mode error aggregation contract

Both passes feed into one accumulated diagnostic list. The strict-mode gate fires
exactly once, after Stage 6b completes. The accumulation contract:

```
pre_diags  = Stage 5: all_validator_diagnostics (Vec<Diagnostic>)
post_diags = Stage 6b: all_post_layout_diagnostics (Vec<Diagnostic>)
combined   = pre_diags + post_diags  // extend in place
```

The existing strict-mode gate logic (lib.rs:465–483) is applied to `combined`:
any `DiagnosticSeverity::Error` in `combined` causes `BuildError::ValidationFailed`
carrying the **full combined diagnostic list** (Error + Warning + Info from both
passes). No partial visibility: the caller always sees diagnostics from both passes.

In non-strict mode (`strict = false`): both passes run, diagnostics from both passes
are logged via `tracing::warn!`, and `build_inner` continues to export. The combined
diagnostic list is NOT returned in the `BuildOutput` (current behavior unchanged).

The accumulation is collect-all, not fail-on-first. Both passes complete fully
before the gate evaluates. A Stage 5 error does NOT abort before Stage 6b.

**Rationale:** A user with both a missing `lang` attribute (pre-layout E-A11-003)
AND a missing chart `alt` (post-layout E-A11-001) sees both errors in one
`build()` call. This is consistent with ADR-012 (error accumulation) and the
DSL design principle that all errors are surfaced in a single pass.

### Decision 5a: Error-precedence rule when layout fails and pre-layout errors exist

When `layout::run` (Stage 6) returns `Err(LayoutError)` AND pre-layout
(Stage 5) Error-severity diagnostics already exist in `all_validator_diagnostics`,
`build()` MUST return `BuildError::ValidationFailed` carrying the full accumulated
diagnostic list — NOT `BuildError::Layout`. The `LayoutError` is subsumed.

Formal statement:

```
IF layout::run returns Err(e)
  AND all_validator_diagnostics.any(|d| d.severity == DiagnosticSeverity::Error)
THEN return Err(BuildError::ValidationFailed {
       diagnostics: all_validator_diagnostics,
       count: error_count,
     })
     // LayoutError `e` is consumed and not surfaced.

IF layout::run returns Err(e)
  AND all_validator_diagnostics.all(|d| d.severity != DiagnosticSeverity::Error)
THEN return Err(BuildError::Layout(e))
     // No pre-layout Error diagnostics; layout error is the root cause.
```

**Rationale (root-cause reporting).** When a deck has zero slides,
`ZeroSlideValidator` emits `E-LAY-002` (pre-layout Error-severity) AND
`layout::run` returns `Err(LayoutError::EmptyDeck)`. Both describe the same
user mistake. `BuildError::Layout(EmptyDeck)` is a downstream symptom of the
real problem — the deck violates the zero-slide constraint. Returning
`ValidationFailed` with `E-LAY-002` gives the caller a structured, code-keyed
diagnostic (actionable: "add at least one slide"). Returning `BuildError::Layout`
would give a raw layout-engine error that bypasses the diagnostic pipeline
and breaks the EC-001 contract (BC-5.02.001 / BC-5.01.001 / STORY-050 EC-001).

**Implementation note.** In the as-built STORY-050 pipeline (`build_inner` in
`slideforge/src/lib.rs`), there is NO pre-layout strict gate. Stage 5 diagnostics
are collected into `all_validator_diagnostics` (lib.rs:~450–477), then
`layout::run` executes unconditionally (lib.rs:~516–520), then Stage 6b
post-layout diagnostics are appended (lib.rs:~557–574), and finally the SINGLE
combined strict gate fires once, after Stage 6b (lib.rs:~576–593). The
precedence rule is enforced at the layout-error handling branch in
`build_inner` (lib.rs:~521–542): if `layout::run` returns `Err` AND
`all_validator_diagnostics` contains at least one `Error`-severity diagnostic
AND `strict=true`, `build()` returns `ValidationFailed` (subsuming the layout
error) at that branch. The combined gate (lib.rs:~576–593) is the single
decision point for the non-error-with-layout-failure cases. The OBS-5 fix
removed the former pre-layout early-return gate; this implementation note
reflects the as-built code.

The precedence rule is written as an explicit contract (not just an implementation
note) because:
1. It must hold in ALL future implementations of `build_inner` — e.g., if the
   pipeline is refactored to run Stage 6b (post-layout validation) and Stage 6
   in parallel or in a different order.
2. It is the observable contract that EC-001 (zero-slide deck → `ValidationFailed`
   with `E-LAY-002`) depends on. If the rule is not explicit, a future refactor
   could silently break EC-001 by returning `BuildError::Layout(EmptyDeck)` instead.
3. It documents the deliberate design choice: layout errors are symptoms; validation
   errors are root causes. The diagnostic pipeline takes precedence.

## Rationale

**Why post-layout validation rather than eval-time ContentBlock construction (Option B)?**

Option B (populate `Slide.blocks` from DSL fields during eval) would require the
evaluator to construct `ContentBlock::Chart`, `ContentBlock::Image`, and
`ContentBlock::Diagram` from scalar field values (e.g., `chart_type "bar"`,
`data "revenue.json"`) in `for_eval.rs:eval_slide_node`. This is architecturally
incorrect in the two-IR model (ADR-005): the evaluator's contract is to resolve
_field values_, not to construct _content geometry_. ContentBlock construction is
a layout concern — the eval stage should not know about the geometric data that
block construction produces. Constructing stub blocks in eval would create a
hybrid half-semantic/half-layout IR that pollutes the pure eval → layout boundary
(ADR-005 Decision).

Option A (post-layout pass) respects the IR boundary: it operates on the IR stage
where content blocks actually exist (`LaidOutDeck.slides[*].frames`). It is
consistent with the two-IR model and does not require the evaluator to produce
geometric output.

**Why an additive-defaulted method rather than a separate pipeline hook or a
separate validator registry?**

The additive-defaulted method pattern was already established and human-authorized
by ADR-017 (Option A, `render_with_context` on `InlineFormat`). The BC explicitly
permits this pattern (BC-5.02.001 invariant 2, v1.4). This approach:

- Requires zero changes to existing `Validator` implementations (all inherit the
  default no-op and compile unchanged).
- Keeps the validator abstraction unified: one `dyn Validator` object handles both
  passes, preventing the registry from needing two separate validator surfaces.
- Is consistent with the `InlineFormat::render_with_context` precedent in this
  codebase, reducing cognitive overhead for future implementors.
- Does not require a second validator registry or a parallel registration path.

A separate `PostLayoutValidator` trait (an alternative considered) would require
a new registry surface, a new `BC-5.02.001` postcondition, and would break the
plugin-first principle by requiring implementors to choose which trait to implement
instead of overriding a single object's methods.

**Why does AltTextValidator's validate() become a no-op rather than being removed?**

The `validate(&Deck)` method cannot be removed — it is a required method on the
`Validator` trait (non-defaulted). `AltTextValidator` must provide an implementation.
The no-op stub body is the correct implementation: `Deck.slides[*].blocks` is always
`vec![]` after eval, so any logic there is dead code. The no-op is documented with
a comment explaining the deferral and pointing to `validate_post_layout`. This is
not a silent bypass; it is a correct architectural allocation.

When a future story populates `Slide.blocks` from eval-time DSL fields, the
`validate()` body can be activated to provide earlier detection (before layout).
The post-layout pass would remain as the authoritative check even then, because it
operates on the resolved geometric IR rather than the partially-constructed eval IR.

**Why does Stage 6b run unconditionally (not only in strict mode)?**

Validators in non-strict mode produce warnings (not errors). Running Stage 6b
unconditionally means alt-text warnings are emitted even in `strict=false` mode.
This is correct: the `--warn-only` flag is intended for iterative authoring, not
for silencing the entire validation layer. The tracing-based warning output is the
user's feedback mechanism when strict mode is off. Skipping Stage 6b in non-strict
mode would silently suppress accessibility warnings, which violates the accessibility
quality bar.

## Consequences

### Positive

- `AltTextValidator` becomes functionally correct end-to-end. The BC guarantee
  ("alt required — compile error if absent") is now enforceable through `build()`.
- Both AC-009 tests in STORY-050 can now pass with a correct `build()` pipeline.
- The additive-defaulted method extends cleanly to future ContentBlock-level validators
  (CanvasOverflowValidator, ContrastValidator) without further trait changes.
- Existing validators (ZeroSlideValidator, LangValidator) and any third-party
  validators compiled against the current trait compile unchanged.
- The two-IR boundary (ADR-005) is preserved: the eval stage remains a pure
  field-resolution pass; the layout stage remains the single ContentBlock construction site.

### Negative / Trade-offs

- `build_inner` calls the validator loop twice (Stage 5 and Stage 6b). For
  validators that override only `validate()` (most), the Stage 6b call is a
  trivially cheap no-op (default returns `vec![]` immediately). The runtime cost
  is negligible.
- No new dependency edges are required. `slideforge-plugin-api` already lists
  `slideforge-layout` as a dependency (`slideforge-plugin-api/Cargo.toml:21`),
  confirmed by the `Exporter` trait which already imports
  `slideforge_layout::LaidOutDeck` in
  `slideforge-plugin-api/src/traits/exporter.rs:8`. The `validate_post_layout`
  method signature uses `&slideforge_layout::LaidOutDeck` through the same
  existing dependency edge. The dependency graph is unchanged:

  ```
  slideforge-plugin-api → slideforge-types   (existing)
  slideforge-plugin-api → slideforge-layout  (existing — Exporter trait dep)
  slideforge-layout → slideforge-types       (existing)
  ```

  No new crate deps. No cycle. The Exporter trait's existing `LaidOutDeck` import
  already resolved this concern.

- `AltTextValidator.validate()` becomes a no-op stub. This is architecturally
  correct but requires documentation to explain why the seemingly-empty method body
  is intentional. The comment in the implementation MUST cite this ADR and
  `for_eval.rs:342`.

### Status as of 2026-06-05

This decision is accepted but not yet implemented. The pipeline sequencing gap is
confirmed by two failing E2E tests (AC-009). Implementation is in-scope for the
STORY-050 fix burst or its designated follow-on story.

## Alternatives Considered

- **Option B — Eval-time ContentBlock stub construction:** Populate `Slide.blocks`
  during `eval_slide_node` from DSL scalar fields (`chart_type`, `data`, `src`).
  Rejected because it violates the two-IR model (ADR-005): the evaluator must not
  construct geometric ContentBlock objects. This would create a hybrid IR and blur
  the eval/layout boundary that the architecture depends on for purity and Kani
  amenability.

- **Option C — #[ignore] + story anchor (documented gap):** Mark the two AC-009
  tests as `#[ignore]` with a comment anchoring them to a future story. Rejected as
  the primary resolution because the alt-text guarantee is a non-negotiable v1.0
  quality bar item (CLAUDE.md). Allowing `build()` to silently succeed on a
  chart-with-no-alt deck through v1.0 violates the production-grade default. This
  option is only appropriate if a genuine blocking dependency prevents implementation
  (none exists here).

- **Separate PostLayoutValidator trait:** Define a second trait `PostLayoutValidator`
  with `validate_post_layout(&LaidOutDeck, &ValidatorOptions)`. Rejected because it
  requires a new registry surface (11th surface, requiring BC-5.02.001 amendment),
  splits the validator abstraction across two traits (increasing implementor
  cognitive overhead), and provides no architectural benefit over an additive-defaulted
  method on the existing `Validator` trait.

- **Pre-layout ContentBlock construction in layout::run fed back to Deck:** Have
  `layout::run` return both a `LaidOutDeck` and an updated `Deck` with blocks
  populated, then re-run validators on the updated `Deck`. Rejected because it
  requires `layout::run` to mutate the semantic IR, violating the unidirectional
  `Deck → LaidOutDeck` transformation contract. The layout stage must not produce
  semantic IR as output.

## Source / Origin

- `slideforge-eval/src/for_eval.rs:342` — `blocks: vec![]` with comment explaining
  deferral to "later pipeline stages (layout, PPTX generation, Waves 3+)".
- `slideforge/src/lib.rs:441–483` — Stage 5 validate gate runs on `&Deck` pre-layout.
- `slideforge-layout/src/layout.rs:428` — `thread_media_alt_into_frames` creates
  `FrameContent::Chart/Image/Diagram` during layout, after the validate gate.
- `.factory/specs/story-050-gap-analysis.md §Gap 2` — root-cause analysis confirming
  the pipeline sequencing gap and assessing fix options.
- BC-5.02.001 invariant 2 (v1.4) — authorizes additive-defaulted methods.
- ADR-016 Decision 3 — establishes the validate gate as Stage 5 of `build_inner`.
- ADR-017 Option A — establishes the additive-defaulted method pattern for
  `InlineFormat::render_with_context`; ADR-018 applies the same pattern to
  `Validator::validate_post_layout`.
- CLAUDE.md §Accessibility — "alt required (compile error)" as a non-negotiable
  quality bar gate.
- Human authorization for Option A: 2026-06-05.

## Implementation Checklist

The following files require changes. This list is the definitive scope for
implementation. Sibling-site sweep (TD-VSDD-060) must be completed for the type
move before committing.

| File | Change |
|------|--------|
| `crates/slideforge-plugin-api/src/traits/validator.rs` | Add `validate_post_layout` defaulted method; import `LaidOutDeck` from `slideforge_layout` (dep already in Cargo.toml) |
| `crates/slideforge-plugin-api/Cargo.toml` | No change — `slideforge-layout` is already a listed dependency |
| `crates/slideforge-validate/src/alt_text.rs` | Migrate content-block loop to `validate_post_layout`; replace `validate()` body with no-op stub + ADR-018 citation comment; add `slideforge-layout` to `slideforge-validate/Cargo.toml` deps |
| `crates/slideforge-validate/Cargo.toml` | Add `slideforge-layout` dependency (needed by `AltTextValidator::validate_post_layout` to iterate `LaidOutDeck.slides`) |
| `crates/slideforge/src/lib.rs` | Add Stage 6b after `layout_run` call: loop all registered validators calling `validate_post_layout(&laid_out, &validator_opts)`, extend `all_validator_diagnostics` with results, then apply the existing strict-mode gate (lib.rs:465–483) to the combined list |

No type moves required. No new workspace crates required. Sibling-site sweep
(TD-VSDD-060) scope: grep `validate_post_layout` across the workspace after adding
the trait method to confirm no stale implementations remain that would now compile
against the default but whose authors intended a real body.

## Flagged Human Sign-off Items

No items require further human authorization before implementation. The human
authorized Option A (post-layout validation pass) on 2026-06-05. The specific
design choices in this ADR (additive-defaulted method, single combined diagnostic
list, validator classification rule) are architectural decisions within the architect's
authority, consistent with established precedents (ADR-005, ADR-017).

No human gates remain open. The dependency graph analysis during ADR drafting
confirmed that `slideforge-plugin-api` already depends on `slideforge-layout`
(for the `Exporter` trait's `LaidOutDeck` parameter), so the `validate_post_layout`
signature requires zero new dependency edges — the implementation checklist is
fully self-contained.
