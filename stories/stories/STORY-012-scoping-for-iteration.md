---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-012
title: "Variable Scoping + @for Evaluation + Termination"
epic: EPIC-03
wave: 2
points: 8
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-eval
subsystems: [SS-02]
target_module: slideforge-eval
behavioral_contracts:
  - BC-1.02.005
  - BC-1.04.001
  - BC-1.04.002
  - BC-1.04.003
  - BC-1.07.002
  - BC-1.08.001
  - BC-1.08.002
  - BC-1.08.003
verification_properties: [VP-003, VP-010]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-011
blocks:
  - STORY-013
  - STORY-014
  - STORY-035
estimated_days: 3
---

# STORY-012: Variable Scoping + @for Evaluation + Termination

## Summary

Extend `slideforge-eval` to support `@for` loop evaluation, lexical variable scoping
across nested loops, and compile-time termination enforcement. The `@for item in collection:`
construct iterates over a finite collection and generates one `Slide` (or content block)
per element. The iteration variable is bound in an inner scope frame and is accessible
inside the loop body via `{{ item.field }}`. Outer scope variables remain accessible
at all nesting levels (lexical scoping, BC-1.02.005). Empty collections produce zero
slides without error (BC-1.04.002). The termination guarantee (BC-1.04.003) is enforced
by disallowing `@while`, `@fn`, and any construct that could produce unbounded iteration.

This story builds on the `Env` struct and expression evaluator from STORY-011. It
adds the `eval_deck()` entry point that consumes a `DeckNode` and produces a `Deck` IR
(defined in `slideforge-types`). The `@for` evaluator calls `Env::push_scope()` /
`Env::pop_scope()` from STORY-011 to implement lexical scoping.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.02.005 | Lexical scoping in nested @for loops — outer vars remain in scope | All 5 postconditions + 2 invariants |
| BC-1.04.001 | @for over finite collection generates slides/blocks per item | All 4 postconditions + 4 invariants |
| BC-1.04.002 | @for over empty collection generates zero items without error | All 4 postconditions + 2 invariants |
| BC-1.04.003 | Reject any construct that could produce non-termination | All 6 postconditions + 3 invariants |
| BC-1.07.002 | Variant vars: overrides deck-level vars per 11-level precedence chain | Eval-time: variant scope frame injected before slide evaluation |
| BC-1.08.001 | set <type>: <field> <value> applies as default to all instances | Eval-time: SetRule nodes applied as defaults in eval_deck() |
| BC-1.08.002 | set rules support {{ }} interpolation and brand.* references | Eval-time: SetRuleValue::Template and ::BrandRef resolved in eval_deck() |
| BC-1.08.003 | brand.* references in set rules are evaluated after brand loading | Eval-time: brand refs resolved during the brand stage, not at parse time |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| BC files (8 BCs) | ~7,000 |
| STORY-011 reference (Env, eval_expr) | ~2,000 |
| STORY-001 types reference | ~1,500 |
| Target source files to write | ~7,000 |
| Test files | ~4,000 |
| **Total** | **~26,000** |

Agent context budget: 200k tokens. This story is ~13% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `@for item in items:` where `items = [Map{"name":"A"}, Map{"name":"B"}]`
  generates exactly 2 slides with `{{ item.name }}` resolved correctly per element.
  Slide order matches collection order.
  (traces to BC-1.04.001 postcondition 1, postcondition 2, postcondition 3)

- [ ] **AC-002** — After `@for` loop completes, the iteration variable `item` is NOT
  accessible outside the loop body (E-EVL-001 if referenced).
  (traces to BC-1.04.001 invariant 3)

- [ ] **AC-003** — The number of generated slides equals the length of the collection for
  any finite input (verified with list lengths 0, 1, 5, 100).
  (traces to BC-1.04.001 invariant 1)

- [ ] **AC-004** — `@for x in []:` (empty inline literal) generates zero slides without
  emitting any error. Build continues; exit code 0.
  (traces to BC-1.04.002 postcondition 1, postcondition 2)

- [ ] **AC-005** — `@for x in []:` where the loop is the only slide-generating construct
  produces a zero-slide Deck IR. The zero-slide E-LAY-002 error is raised by the
  validation stage (STORY-016), NOT by the evaluator.
  (traces to BC-1.04.002 invariant 2)

- [ ] **AC-006** — Nested `@for` loops: `@for project in projects:` / `@for item in project.items:`
  — inside the inner loop, `{{ project.name }}` (outer variable) and `{{ item.name }}`
  (inner variable) are both accessible and resolve correctly.
  (traces to BC-1.02.005 postcondition 1, postcondition 2)

- [ ] **AC-007** — Inner loop variable name matches outer loop variable name (e.g., both
  named `item`): inner `item` shadows outer `item` within the inner block only. After
  the inner loop, the outer `item` is restored.
  (traces to BC-1.02.005 postcondition 4; BC-1.02.005 invariant 1)

- [ ] **AC-008** — Three-level nesting: innermost block can access variables from all
  outer `@for` scopes without E-EVL-001.
  (traces to BC-1.02.005 postcondition 1)

- [ ] **AC-009** — `@while true:` in source produces E-PAR-006:
  `'while' is reserved for future iteration constructs (planned v2+)` and build exits 1.
  The check occurs at AST traversal time (evaluator rejects `@while` nodes).
  (traces to BC-1.04.003 postcondition 2, postcondition 5)

- [ ] **AC-010** — `@fn myfunc(x):` in source produces E-PAR-006:
  `'@fn' is reserved for user-defined functions (planned v2+)` and build exits 1.
  (traces to BC-1.04.003 postcondition 5)

- [ ] **AC-011** — Large collection (`items` with 1000 elements) produces 1000 slides.
  A lint warning is emitted: `Large iteration: @for over 1000 items may produce a very
  large deck. Consider filtering data at source.` but build continues.
  (traces to BC-1.04.003 edge case EC-001)

- [ ] **AC-012** — `@for` over `@data items from "..."` binds the loaded collection
  correctly. The evaluator calls into the `DataSource` abstraction (placeholder for
  STORY-018; for this story, use a `MockDataSource` in tests).
  (traces to BC-1.04.001 precondition 2)

- [ ] **AC-013** — When a deck declares `variants: exec` with `vars: { color: "red" }` and
  is evaluated with `--variant exec`, the variant's `vars` scope frame is pushed onto the
  `Env` stack BEFORE slide evaluation begins. A slide referencing `{{ color }}` resolves
  to `"red"`, overriding any deck-level `vars: { color: "blue" }`.
  (traces to BC-1.07.002 postcondition 1 — variant vars override deck-level vars per
  11-level merge precedence chain)

- [ ] **AC-014** — `eval_deck()` applies `SetRule` nodes as defaults before evaluating any
  slides. Given `set content: footer "Confidential"`, every `slide content:` that does
  NOT explicitly set `footer` receives `footer = "Confidential"` in its evaluated output.
  A slide that DOES explicitly set `footer "Custom"` retains `"Custom"` (slide-level wins
  over set-rule default).
  (traces to BC-1.08.001 postcondition 1 — set rules apply as defaults to all instances)

- [ ] **AC-015** — `set content: footer "{{ brand.footer }}"` in the evaluated deck: the
  `SetRuleValue::Template` containing a `brand.*` field access expression is resolved by
  `eval_deck()`. If brand has not yet been loaded, a `BrandRef` placeholder is preserved
  for the brand stage. No panic or E-EVL-001 is emitted for brand refs at eval time.
  (traces to BC-1.08.002 postcondition 1 — set rules support template interpolation;
  BC-1.08.003 invariant 1 — brand refs not evaluated at eval time)

- [ ] **AC-016** — `cargo test -p slideforge-eval` passes with new tests added in this story.

- [ ] **AC-017** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean. All public items have rustdoc.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

## Previous Story Intelligence

STORY-011 established the `Env` struct with `push_scope()` / `pop_scope()` / `lookup()`.
This story consumes that API directly. Key design constraint from STORY-011:

- `Env` is `Clone` — snapshot before entering a loop body; restore on exit via
  `pop_scope()`.
- `eval_expr()` returns `Option<Value>`; None means an error was pushed to `DiagnosticSink`.
- All 15 filter functions are already implemented and tested.

Do NOT redefine `Env` or `EvalError` in this story — extend `slideforge-eval` with new
modules that call into the existing infrastructure.

## Architecture Compliance Rules

1. **Pure Core (SS-02):** All @for evaluation is in-memory. No I/O, no filesystem access,
   no network. Data source loading is a separate concern (STORY-018 in Wave 3) — this story
   accepts pre-loaded `Value::List` collections as input.
2. **Termination enforcement:** The evaluator MUST NOT allow recursive descent into
   user-defined functions (no `@fn` nodes in AST). The grammar already prohibits this,
   but the evaluator MUST also assert the absence of such nodes and emit E-PAR-006 if
   encountered (defense-in-depth).
3. **Termination bound:** The maximum slides generated per `@for` is bounded by the
   product of all collection sizes. The evaluator tracks `total_slides_generated: usize`
   and emits a lint warning (not an error) when `total_slides_generated > LARGE_DECK_WARN_THRESHOLD`
   (default: 500 slides). This is configurable via `EvalConfig`.
4. **No mutable global state:** The `@for` evaluator is a pure function of `(DeckNode,
   EvalConfig, DiagnosticSink)` → `(Deck, DiagnosticSink)`. No global counters.
5. **Forbidden dependencies:** Same as STORY-011 — `slideforge-eval` MUST NOT depend on
   effectful crates.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `Value`, `SourceSpan`, `Register` |
| `slideforge-syntax` | workspace | `DeckNode`, `SlideNode`, `ForNode`, `IfNode`, `FieldValue`, `Spanned<T>` |
| `thiserror` | `=2.0.18` | Extends `EvalError` from STORY-011 |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | Diagnostic impls |
| `indexmap` | `"2"` (already in Cargo.toml from STORY-011) | `IndexMap` in `Env` |

Dev dependencies:
- `insta` (compatible) — snapshot tests

## File Structure Requirements

Files to extend (modify, do NOT replace):

```
crates/slideforge-eval/
├── src/
│   ├── eval.rs           # EXTEND: add eval_deck(), eval_for_block(), eval_slide_node()
│   ├── error.rs          # EXTEND: add ReservedKeyword variant for E-PAR-006
│   └── env.rs            # READ-ONLY: Env already defined in STORY-011
```

Files to create (new in this story):

```
crates/slideforge-eval/
├── src/
│   ├── for_eval.rs       # @for block evaluator; scoping logic
│   └── config.rs         # EvalConfig struct (large_deck_warn_threshold, etc.)
```

## Tasks

1. **Write `src/config.rs`** — `EvalConfig` struct with:
   - `large_deck_warn_threshold: usize = 500` — emit lint warning when slides generated
     by a single `@for` block exceeds this value.
   - `max_total_slides: Option<usize> = None` — optional hard cap (None = unlimited).
   - Implement `Default`. (20 min)
2. **Extend `src/error.rs`** — add `EvalError::ReservedKeyword { keyword: Arc<str>, hint: String, span: SourceSpan }` with error code `E_PAR_006`. (15 min)
3. **Write `src/for_eval.rs`** — implement `eval_for_block(env: &mut Env, for_node: &ForNode, config: &EvalConfig, sink: &mut DiagnosticSink) -> Vec<Slide>`. Algorithm:
   1. Evaluate the collection expression: `eval_expr(env, &for_node.collection, sink)`.
   2. If None (error), return empty vec.
   3. Expect `Value::List(items)` — if not a list, push E-EVL-003 and return empty vec.
   4. For each `item` in items: push scope `{ for_node.var_name: item.clone() }`,
      evaluate the loop body (slides or content blocks), pop scope.
   5. Track `total_slides` and emit lint warning if threshold exceeded.
   6. Return the generated slides in order. (90 min)
4. **Extend `src/eval.rs`** — implement `eval_deck(deck_node: &DeckNode, config: &EvalConfig, sink: &mut DiagnosticSink) -> Deck`:
   1. Build base `Env` from `deck_node.vars`.
   2. Evaluate `set_rules` (apply as defaults).
   3. For each top-level node (slide, @for, @if): dispatch to appropriate evaluator.
   4. Return `Deck { slides, vars, metadata, registers }`. (60 min)
5. **Write unit tests** for scoping and @for evaluation. See Test Strategy. (60 min)
6. **Run `cargo test -p slideforge-eval`** — confirm all STORY-011 tests + new tests pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in `for_eval.rs`)

- `test_for_generates_correct_count()`: `@for x in [1,2,3]` → 3 slides generated.
- `test_for_item_value_per_slide()`: `@for x in [1,2,3]`, body references `{{ x }}` — verify slide 1 has x=1, slide 2 has x=2, slide 3 has x=3.
- `test_for_empty_collection()`: `@for x in []` → 0 slides; 0 errors in sink.
- `test_for_empty_only_slide()`: single `@for x in []` → Deck with 0 slides (E-LAY-002 from validator, not evaluator).
- `test_for_scope_exit()`: after loop, `x` not in env; `env.lookup("x")` == None.
- `test_nested_for_outer_var_accessible()`: nested loops; inner body references outer `project.name` — no E-EVL-001.
- `test_nested_for_shadowing()`: inner `@for item in ...` shadows outer `@for item in ...`; after inner loop, outer `item` is restored.
- `test_for_three_levels_nesting()`: 3-level nest; innermost accesses all three iteration vars — no errors.
- `test_while_reserved()`: `@while` node in AST → E-PAR-006 with message about v2+.
- `test_fn_reserved()`: `@fn` node in AST → E-PAR-006 with message about v2+.
- `test_large_collection_warning()`: 600-item collection → lint warning in sink; slides generated.

### Snapshot tests

- `test_snapshot_for_3items()`: evaluate deck with `@for` over 3-item fixture; snapshot
  the resulting `Deck.slides` field names.
- `test_snapshot_nested_for()`: evaluate nested `@for`; snapshot slide count and first slide's evaluated `title` field.

## Dependencies

**Depends on:** STORY-011 (Env, eval_expr, filters, EvalError — all prerequisites for @for evaluation).

**Blocks:**
- STORY-013: @if evaluation and @include cycle detection extend `eval_deck()` defined here.
- STORY-035: writing register routing extends `eval.rs` built on this story.

### Dependency Anchor Justifications

- SS-02 owns this story's scope because SS-02 is the Evaluator subsystem.
- STORY-012 depends on STORY-011 because `@for` evaluation uses `Env::push_scope()`,
  `Env::pop_scope()`, and `eval_expr()` which are defined in STORY-011.
- STORY-012 blocks STORY-013 because `eval_deck()` defined here is the entry point that
  STORY-013 extends to handle `@if` and `@include` cycle detection.

## Implementation Notes

### @for Termination Bound

The termination invariant (DI-005, BC-1.04.003) is enforced at two levels:

1. **Grammar level** (STORY-007): the grammar has no `@while` production; the AST cannot
   represent unbounded loops.
2. **Evaluator level** (this story): the evaluator traverses the AST recursively. The
   recursion depth is bounded by the nesting depth of `@for` blocks, which is bounded
   by the AST depth (a finite tree). No tail calls or loops can simulate unbounded iteration.

The large-deck lint warning is NOT a termination enforcement mechanism — it is a UX hint.
Actual termination is guaranteed by the above two structural properties.

### `EvalConfig` — Termination-Adjacent Settings

```rust
/// Configuration for the evaluator.
#[derive(Debug, Clone)]
pub struct EvalConfig {
    /// Emit a lint warning when a single @for block generates more than this
    /// many slides. Default: 500.
    pub large_deck_warn_threshold: usize,

    /// Hard cap on total slides in the output Deck.
    /// None means no hard cap (termination is still guaranteed by structure).
    pub max_total_slides: Option<usize>,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self { large_deck_warn_threshold: 500, max_total_slides: None }
    }
}
```

### `ForNode` Expected AST Shape (from STORY-007)

```rust
pub struct ForNode {
    pub var_name: Spanned<String>,
    pub collection: ExprNode,
    pub body: Vec<SlideOrBlock>,  // slides or content blocks in loop body
    pub span: Span,
}
```

If STORY-007 is not yet merged when this story is implemented, use a stub `ForNode`
in test helpers. Replace with real type before merge.

### `eval_deck` Entry Point

```rust
/// Evaluate a parsed DeckNode into a Deck IR.
///
/// All expression interpolations, @for loops, and @if branches are resolved.
/// Errors are accumulated in `sink`; evaluation continues on error
/// (error-slide placeholders for broken slides in warn-only mode).
///
/// Returns None if any fatal error (parse error passed through) was encountered.
pub fn eval_deck(
    deck_node: &DeckNode,
    config: &EvalConfig,
    sink: &mut DiagnosticSink,
) -> Option<Deck>;
```

### Scope Chain Semantics (DEC-017)

The scope chain for a nested `@for` is:

```
Frame 0: deck vars: { a: "global" }
Frame 1: outer @for: { project: Map{...} }
Frame 2: inner @for: { item: Map{...} }
```

Lookup walks frames from innermost (2) to outermost (0). When the inner `@for` exits,
frame 2 is popped. When the outer `@for` exits, frame 1 is popped.

This is a standard lexical scope chain — the same model used by languages like Python,
Rust, and JavaScript. No dynamic scoping.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@for` over HTTP data source (not yet loaded) | Data sources are pre-loaded by `slideforge-data` (STORY-018) and passed as `Value::List` to the evaluator. The evaluator receives the loaded values, not URLs. |
| EC-002 | `@for key, value in config_map:` — map iteration | One iteration per key-value pair; `key` and `value` bound in scope frame |
| EC-003 | `@for item in data.nested.list:` — collection from nested field access | Collection expression resolved via `eval_expr`; dot-notation traversal from STORY-011 |
| EC-004 | `@for` generates slides that contain nested `@if` blocks | `@if` resolved per-iteration with iteration variable in scope (implemented in STORY-013) |
| EC-005 | Empty deck after `@for` + `@if` combination produces zero slides | Evaluator produces Deck with 0 slides; validation stage (STORY-016) emits E-LAY-002 |
| EC-006 | Outer `@for` variable name is same as deck var name | Outer loop variable shadows deck var; deck var resumes after outer loop exits |
