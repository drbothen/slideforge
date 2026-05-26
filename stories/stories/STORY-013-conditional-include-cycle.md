---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-013
title: "@if/@elif/@else Evaluation + @include Cycle Detection"
epic: EPIC-03
wave: 2
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-eval
subsystems: [SS-02]
target_module: slideforge-eval
behavioral_contracts:
  - BC-1.05.001
  - BC-1.05.002
  - BC-1.06.002
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-012
blocks:
  - STORY-018
  - STORY-035
estimated_days: 2
---

# STORY-013: @if/@elif/@else Evaluation + @include Cycle Detection

## Summary

Extend `slideforge-eval` with two capabilities: conditional rendering (`@if/@elif/@else`)
at all four scopes (slide, element, field, section), and post-AST-merge `@include` cycle
detection. The conditional evaluator uses the same expression evaluator and type rules
as `{{ expr }}` — no implicit truthiness, no separate "condition mode". The `@include`
cycle detection runs after all `@include` directives have been inlined at the AST merge
stage (STORY-008) to catch any remaining diamond-include or circular patterns not caught
by the parser.

Both features integrate into `eval_deck()` from STORY-012. The conditional evaluator
extends the `eval_deck()` dispatch table. The cycle detection runs as a pre-pass on the
merged `DeckNode` before expression evaluation begins.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.05.001 | @if/@elif/@else at slide, element, field, and section scope | All 5 postconditions + 3 invariants |
| BC-1.05.002 | Conditional expression evaluates with same type rules as {{ expr }} | All 4 postconditions + 2 invariants |
| BC-1.06.002 | Detect and reject circular @include chains (Fail-Closed) | All 3 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| BC files (3 BCs) | ~3,000 |
| STORY-011 eval_expr reference | ~1,500 |
| STORY-012 eval_deck reference | ~1,500 |
| Target source files to write | ~5,000 |
| Test files | ~3,000 |
| **Total** | **~18,000** |

Agent context budget: 200k tokens. This story is ~9% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `@if env == "prod":` / slide block: the slide is included in output
  when `env = "prod"`, excluded when `env = "dev"`. No partial rendering; the entire
  slide block is included or excluded atomically.
  (traces to BC-1.05.001 postcondition 1)

- [ ] **AC-002** — `@if env == "prod":` / slide A / `@elif env == "staging":` / slide B /
  `@else:` / slide C: exactly one slide is rendered for any value of `env`. Conditions
  tested in order; first truthy branch wins.
  (traces to BC-1.05.001 postcondition 2, postcondition 3; BC-1.05.001 invariant 1)

- [ ] **AC-003** — `@if` at four scopes compiles and renders correctly:
  - Slide scope: wraps entire `slide` block.
  - Element scope: wraps a visual element within a slide.
  - Field scope: selects between two field values (`title @if is_draft: "DRAFT" @else: "Final"`).
  - Section scope: wraps a `section methodology:` block.
  (traces to BC-1.05.001 postcondition 1)

- [ ] **AC-004** — Condition evaluation is lazy: only conditions up to the first truthy
  branch are evaluated. A condition with an undefined variable in a non-evaluated branch
  does NOT produce E-EVL-001.
  (traces to BC-1.05.001 invariant 2)

- [ ] **AC-005** — `@if flag:` where `flag = "true"` (string) emits E-EVL-003:
  `String used as boolean condition at <file>:<line>:<col>. Use @if flag == "true": instead.`
  (traces to BC-1.05.002 postcondition 2, postcondition 3)

- [ ] **AC-006** — `@if count > 0:` where `count = 5` (integer comparison) evaluates to
  `true` and the block renders. Boolean comparison operators `==`, `!=`, `<`, `>`, `<=`,
  `>=`, `&&`, `||`, `!` all work in conditions.
  (traces to BC-1.05.002 postcondition 1)

- [ ] **AC-007** — `@if items | length > 0:` (pipe filter in condition expression) evaluates
  correctly. Pipe filters work in conditional expressions.
  (traces to BC-1.05.002 postcondition 1)

- [ ] **AC-008** — `@if undefined_cond:` emits E-EVL-001 and build exits 2 in strict mode.
  (traces to BC-1.05.001 edge case EC-005)

- [ ] **AC-009** — `@include` cycle: `a.sf` includes `b.sf` includes `a.sf` produces
  E-PAR-004: `Include cycle detected: a.sf → b.sf → a.sf` and build exits 1.
  (traces to BC-1.06.002 postcondition 1, postcondition 2)

- [ ] **AC-010** — Self-include: `deck.sf` includes itself produces E-PAR-004:
  `Include cycle detected: deck.sf → deck.sf`.
  (traces to BC-1.06.002 postcondition 1)

- [ ] **AC-011** — Diamond include: `A` includes `B` and `C`; `B` includes `D`; `C` includes
  `D` — no cycle. `D` content is inlined twice without error.
  (traces to BC-1.06.002 invariant 3)

- [ ] **AC-012** — Cycle detection terminates for include chains up to 200 files deep
  (no stack overflow; bounded DFS).
  (traces to BC-1.06.002 verification property)

- [ ] **AC-013** — `cargo test -p slideforge-eval` passes with all tests added in this story.

- [ ] **AC-014** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

## Previous Story Intelligence

STORY-012 established `eval_deck()` as the entry point and `@for` evaluation. Key points:

- `eval_deck()` dispatches on AST node type. This story extends that dispatch to handle
  `IfNode` (and chains of `ElifBranch` / `ElseBranch`).
- `Env::push_scope()` / `pop_scope()` are already available — no need for a new scope
  mechanism for `@if` (unlike `@for`, `@if` does NOT introduce new variable bindings).
- `eval_expr()` from STORY-011 is the expression evaluator reused for `@if` conditions.
  The type check (condition must be boolean) is an assertion AFTER evaluation, using the
  same `EvalError::TypeMismatch` variant.

Lesson: Conditions MUST return `Value::Bool`. If `eval_expr` returns `Value::Str` or
`Value::Int`, that is NOT an implicit truthy/falsy conversion — it is an E-EVL-003 error.
This is unlike Python, JavaScript, and YAML — make this explicit in every test.

## Architecture Compliance Rules

1. **Pure Core (SS-02):** `@if` evaluation is in-memory. Conditions cannot trigger I/O.
2. **Lazy evaluation of branches:** The evaluator MUST NOT evaluate conditions of
   branches after the first truthy branch. Failing to implement laziness could produce
   false E-EVL-001 errors for unused branches.
3. **@include cycle detection is a pre-pass:** Run cycle detection on the merged AST
   BEFORE any expression evaluation. This ensures cycle detection is fail-closed — no
   partial evaluation of a cyclic deck.
4. **DFS with visited set:** The cycle detector uses depth-first search with an
   `in_progress: HashSet<PathBuf>` to detect back-edges (cycles) and a
   `completed: HashSet<PathBuf>` to skip already-validated subtrees (diamond optimization).
5. **Forbidden dependencies:** Same as STORY-011 and STORY-012.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Deck`, `Slide`, `Value`, `SourceSpan` |
| `slideforge-syntax` | workspace | `DeckNode`, `IfNode`, `ElifBranch`, `ElseBranch`, `IncludeNode` |
| `thiserror` | `=2.0.18` | EvalError extension |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | Diagnostic impls |
| `indexmap` | `"2"` (already in Cargo.toml from STORY-011) | Used in Env (already present) |

Dev dependencies: `insta` (compatible)

## File Structure Requirements

Files to extend (modify, do NOT replace):

```
crates/slideforge-eval/
├── src/
│   ├── eval.rs           # EXTEND: dispatch for IfNode; add if-chain evaluation
│   ├── error.rs          # EXTEND: add IncludeCycle variant for E-PAR-004
```

Files to create (new in this story):

```
crates/slideforge-eval/
├── src/
│   ├── if_eval.rs        # @if/@elif/@else chain evaluator
│   └── include_cycle.rs  # @include cycle detection (DFS pre-pass)
```

## Tasks

1. **Extend `src/error.rs`** — add `EvalError::IncludeCycle { cycle_path: Vec<Arc<str>>, span: SourceSpan }`
   with error code `E_PAR_004`. Format: `Include cycle detected: {path[0]} → {path[1]} → ...` (30 min)
2. **Write `src/include_cycle.rs`** — DFS cycle detector over include graph.
   Input: `&DeckNode` after include inlining. Track `in_progress: HashSet<PathBuf>`.
   On cycle: push E-PAR-004 to sink, return `Err(())`. Diamond includes (visited twice,
   not via back-edge) must NOT produce false positives. (45 min)
3. **Write `src/if_eval.rs`** — `eval_if_chain(env: &Env, if_node: &IfNode, config: &EvalConfig, sink: &mut DiagnosticSink) -> Vec<Slide>`:
   1. Evaluate the `@if` condition using `eval_expr(env, &if_node.condition, sink)`.
   2. If None (error), return empty vec (don't try to evaluate body).
   3. If `Some(value)`: check `value` is `Value::Bool` — if not, push E-EVL-003 and
      return empty vec.
   4. If `Bool(true)`: evaluate the body nodes; return slides.
   5. If `Bool(false)`: try `elif` branches in order; if none truthy, evaluate `else`
      body if present.
   6. Implement lazy evaluation: stop testing conditions after first truthy branch. (60 min)
4. **Extend `src/eval.rs`** — integrate `eval_if_chain()` into `eval_deck()` dispatch
   table; integrate `include_cycle::check()` as pre-pass at start of `eval_deck()`. (30 min)
5. **Write unit tests**. See Test Strategy. (45 min)
6. **Run `cargo test -p slideforge-eval`** and confirm all tests pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in `if_eval.rs`)

- `test_if_true_renders_block()`: `@if true:` → slide included.
- `test_if_false_no_output()`: `@if false:` without else → 0 slides; 0 errors.
- `test_if_elif_else_first_truthy()`: first branch true → only first branch rendered.
- `test_if_elif_else_second_truthy()`: first false, second true → second branch rendered.
- `test_if_else_fallback()`: all conditions false → else branch rendered.
- `test_if_string_condition_type_error()`: `flag = "true"`, `@if flag:` → E-EVL-003.
- `test_if_int_condition_type_error()`: `n = 0`, `@if n:` → E-EVL-003.
- `test_if_bool_condition_valid()`: `flag = true`, `@if flag:` → valid, block rendered.
- `test_if_lazy_evaluation()`: second branch has undefined var; first branch is true; NO E-EVL-001 for second branch's undefined var.
- `test_if_pipe_in_condition()`: `@if items | length > 0:` with non-empty list → block rendered.
- `test_if_undefined_condition_var()`: `@if undefined:` → E-EVL-001; exit 2.

### Unit tests (`#[cfg(test)] mod tests` in `include_cycle.rs`)

- `test_no_cycle_single_file()`: single file, no includes → no error.
- `test_direct_cycle()`: `a.sf → b.sf → a.sf` → E-PAR-004 with cycle path.
- `test_self_include()`: `deck.sf → deck.sf` → E-PAR-004: `deck.sf → deck.sf`.
- `test_diamond_include_no_cycle()`: `A→B`, `A→C`, `B→D`, `C→D` → no error; D visited twice.
- `test_deep_chain_no_cycle()`: 50-level chain with no cycle → no error.

## Dependencies

**Depends on:** STORY-012 (eval_deck, Env, eval_expr, @for evaluation).

**Blocks:**
- STORY-018: data source loading provides `Value::List` collections consumed by `@for`
  and `@if` conditions.
- STORY-035: writing register routing extends `eval_deck()` built on this story.

### Dependency Anchor Justifications

- SS-02 owns this story's scope because SS-02 is the Evaluator subsystem.
- STORY-013 depends on STORY-012 because `@if` evaluation extends `eval_deck()` from
  STORY-012, and uses `eval_expr()` from STORY-011 (via STORY-012 transitive dep).
- STORY-013 blocks STORY-018/035 because `eval_deck()` is the integration point those
  stories extend.

## Implementation Notes

### Type Check for Conditions

After `eval_expr()` returns `Some(value)`, the conditional evaluator MUST perform:

```rust
match value {
    Value::Bool(b) => Ok(b),
    other => {
        sink.push(EvalError::TypeMismatch {
            message: format!("condition expression evaluated to {}, not Bool; \
                use a comparison operator (==, !=, <, >, <=, >=) to produce a boolean",
                other.type_name()),
            span,
        });
        Err(())
    }
}
```

Do NOT use Rust's concept of "truthy" values. Only `Value::Bool(true)` is truthy.
`Value::Int(1)`, `Value::Str("true")`, `Value::Str("yes")` are all type errors.

### Include Cycle Detection: DFS Algorithm

```
fn check_cycles(node: &DeckNode, in_progress: &mut HashSet<PathBuf>,
                completed: &mut HashSet<PathBuf>, sink: &mut DiagnosticSink)
{
    for include in node.includes() {
        let path = include.resolved_path();
        if in_progress.contains(&path) {
            // Back-edge: cycle detected
            let cycle = build_cycle_path(in_progress, &path);
            sink.push(EvalError::IncludeCycle { cycle_path: cycle, span: include.span });
            continue;
        }
        if completed.contains(&path) {
            // Already validated (diamond pattern) — skip
            continue;
        }
        in_progress.insert(path.clone());
        check_cycles(include.ast(), in_progress, completed, sink);
        in_progress.remove(&path);
        completed.insert(path);
    }
}
```

Note: In STORY-008, `@include` directives are inlined at parse time. By the time the
evaluator receives the merged `DeckNode`, the include AST has already been merged.
The cycle detector operates on the merged AST's `IncludeNode` metadata (source path
of each included file), not on a live filesystem. The include graph is still a DAG
property of the merged AST structure.

### @if at Field Scope

Field-scope `@if` (selecting between two field values) is implemented as a special-case
`ExprNode` rather than a top-level AST node:

```
title @if is_draft: "DRAFT: {{ title }}" @else: "{{ title }}"
```

This produces an `ExprNode::IfExpr { condition, then_val, else_val }` in the field
value expression. The conditional evaluator handles this inline — it does not produce
slides, just a `Value` for the field.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@if` at slide scope wrapping entire slide block | Slide atomically included or excluded; no half-rendered slide with some fields evaluated |
| EC-002 | All `@elif` conditions are false and no `@else:` | Zero output for that block; no error; build continues |
| EC-003 | `@elif` condition references loop variable inside `@for` | Variable is in scope (Env frame from @for); condition evaluates correctly per iteration |
| EC-004 | `@if` condition contains pipe: `@if items | length > 0:` | Pipe evaluated in condition context; result must be `Bool` — `length > 0` produces `Bool`, so this is valid |
| EC-005 | `@if` condition short-circuit: `flag == true && items | length > 0` — flag is false | `&&` is short-circuit; items|length not evaluated |
| EC-006 | Include cycle detected AND expression errors in same file | Both E-PAR-004 and E-EVL-001 accumulated; include cycle is pre-pass so it runs first |
