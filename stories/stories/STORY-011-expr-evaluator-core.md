---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-011
title: "Expression Evaluator Core"
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
  - BC-1.02.001
  - BC-1.02.002
verification_properties: [VP-005, VP-015]
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-001
  - STORY-005
  - STORY-006
  - STORY-010
blocks:
  - STORY-012
  - STORY-014
  - STORY-018
  - STORY-035
estimated_days: 3
---

# STORY-011: Expression Evaluator Core

## Summary

Implement the core expression evaluator in `slideforge-eval`. This evaluator processes
`{{ expr }}` interpolations in field values, resolving variable references, arithmetic
operations, comparison operators, dot-notation field access, and pipe filter chains.
The evaluator accepts an `Env` (scope chain) and returns a `Value` or an accumulated
`DiagnosticSink` error. All ~15 built-in filter functions are implemented as pure
functions. Error accumulation (DI-018) ensures all expression errors are collected in
a single pass. This story also implements the undefined-variable error path (BC-1.02.002)
including scope-path reporting.

The evaluator consumes the `DeckNode` AST from `slideforge-syntax` (STORY-006 output)
and produces evaluated `Value` results. The full Deck IR construction (slide iteration,
@for, @if, @include cycle detection) is in subsequent stories (STORY-012, STORY-013).

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.02.001 | Evaluate {{ expr }} interpolation with arithmetic and pipe filters | All 5 postconditions + 3 invariants |
| BC-1.02.002 | Reject undefined variable reference with scope path | All 5 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,500 |
| BC files (BC-1.02.001, BC-1.02.002) | ~2,000 |
| STORY-001 types reference | ~1,500 |
| STORY-010 DiagnosticSink reference | ~1,000 |
| Target source files to write | ~7,000 |
| Test files | ~4,000 |
| Cargo.toml | ~500 |
| **Total** | **~20,500** |

Agent context budget: 200k tokens. This story is ~10% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `{{ n * 2 }}` where `n = 5` evaluates to `Value::Str("10")` (result is
  coerced to string for embedding). Arithmetic operators `+`, `-`, `*`, `/`, `%` are
  supported on `Value::Int` and `Value::Float` operands.
  (traces to BC-1.02.001 postcondition 1, postcondition 2)

- [ ] **AC-002** — `{{ name | upper }}` where `name = "world"` evaluates to `Value::Str("WORLD")`.
  Pipe chains apply filters left-to-right. All 15 built-in filters (see Implementation
  Notes) produce correct output.
  (traces to BC-1.02.001 postcondition 4)

- [ ] **AC-003** — `{{ item.price }}` where `item = Map { "price": Int(42) }` evaluates to
  `Value::Int(42)` via dot-notation traversal. Nested access `{{ a.b.c }}` resolves
  step-by-step; E-DAT-005 is emitted if any level is missing.
  (traces to BC-1.02.001 postcondition 3)

- [ ] **AC-004** — `{{ value | unknown_filter }}` emits E-EVL-004:
  `Filter 'unknown_filter' not found at <file>:<line>:<col>. Available: [upper, lower, currency, round, int, float, string, join, length, trim, replace, starts_with, ends_with, contains, default]`
  (traces to BC-1.02.001 postcondition 5)

- [ ] **AC-005** — `{{ 10 / 0 }}` emits E-EVL-003: `Division by zero at <file>:<line>:<col>`
  and does not panic. Exit code 2.
  (traces to BC-1.02.001 invariant 3)

- [ ] **AC-006** — `{{ greeting }}` with no `greeting` variable in any scope emits E-EVL-001:
  `Undefined variable '{{ greeting }}' at <file>:<line>:<col>. Variables in scope: []`
  (traces to BC-1.02.002 postcondition 1)

- [ ] **AC-007** — The scope list in E-EVL-001 accurately shows all variable names visible
  at the reference point. If `vars: { a: "x", b: "y" }` is declared and `{{ c }}` is
  referenced, the error shows `Variables in scope: [a, b]`.
  (traces to BC-1.02.002 postcondition 2)

- [ ] **AC-008** — Two undefined variable references in the same deck produce two separate
  E-EVL-001 diagnostics (both accumulated before halting); exit code 2.
  (traces to BC-1.02.002 invariant 1; BC-1.15.002)

- [ ] **AC-009** — In `--warn-only` mode, an undefined variable reference emits E-EVL-001
  as a warning and produces an error-slide placeholder for the affected slide. Build
  exits 0. No silent substitution ("undefined" string or empty string) occurs.
  (traces to BC-1.02.002 postcondition 5; BC-1.02.002 invariant 3)

- [ ] **AC-010** — Filter functions are pure: calling `upper("hello")` twice produces
  identical results. The evaluator has no mutable state that a filter can modify.
  (traces to BC-1.02.001 invariant 1)

- [ ] **AC-011** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean. All public items have rustdoc comments.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

- [ ] **AC-012** — All production deps in `Cargo.toml` use `=` version pinning.
  (traces to NFR-025)

- [ ] **AC-013** — `cargo test -p slideforge-eval` passes. Unit tests confirm all 15
  built-in filter functions produce correct output for their canonical test vectors.

- [ ] **AC-014** — Type mismatch in arithmetic (e.g., `"NO" + 1`) produces E-EVL-003:
  `Type mismatch: operator '+' expects numeric, got String at <file>:<line>:<col>`.
  No implicit coercion occurs.
  (traces to BC-1.02.001 invariant 2)

## Previous Story Intelligence

This is the first story in EPIC-03 (Evaluator). Key dependencies:

- **STORY-001** defines `Value`, `Deck`, `Slide`, `SourceSpan`, `Register` — use these
  types directly; do not redefine.
- **STORY-010** defines `DiagnosticSink` — the evaluator emits all errors through the
  sink, never returning `Err` directly from individual expression evaluations (error
  accumulation invariant).
- **STORY-006** defines `DeckNode`, `SlideNode`, `FieldValue` — the evaluator input is
  these AST nodes.

Lesson from STORY-006: chumsky 0.10's `Rich` error spans are extracted via `Rich::span()`
and mapped to `SourceSpan`. The evaluator does not use chumsky — it works on the
already-parsed AST. Span information comes from `Spanned<T>` wrappers in the AST nodes.

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md` and `architecture/purity-boundary-map.md`:

1. **Pure Core classification (SS-02):** `slideforge-eval` is a Pure Core crate. It MUST NOT
   introduce any I/O, filesystem access, network, or async code. All evaluation is in-memory.
2. **No side effects in filter functions:** All 15 built-in filters must be stateless pure
   functions. A filter MUST NOT write to global state, spawn threads, or perform I/O.
3. **Forbidden dependencies:** `slideforge-eval` MUST NOT depend on: `slideforge-data`,
   `slideforge-brand`, `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`,
   `slideforge-pdf`, `slideforge-cli`. Build MUST fail if such a dep appears.
4. **DiagnosticSink protocol:** All errors go through `DiagnosticSink::push()`. Expression
   evaluation returns `Option<Value>` (None = error pushed to sink). Never propagate
   `Err(Vec<_>)` directly.
5. **`Hash + Eq + Clone` on all IR types (ADR-005):** The `Env` type must implement
   `Clone` to support snapshot semantics for `@for` scope entry/exit. Do NOT use `Rc<_>`
   or `Arc<Mutex<_>>` for the environment — use plain owned structures.
6. **Integer EMUs not `f64` (ADR-013):** The evaluator handles `Value::Float` (which uses
   `OrderedFloat<f64>`) for user-declared floats. Do NOT introduce new f64 fields in
   evaluator types.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Value`, `Deck`, `Slide`, `SourceSpan`, `Register` |
| `slideforge-syntax` | workspace | `DeckNode`, `SlideNode`, `FieldNode`, `FieldValue`, `Spanned<T>` |
| `thiserror` | `=2.0.18` | Error derives for `EvalError` enum |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | `Diagnostic` trait impl on `EvalError` |
| `indexmap` | `"=2.7.1"` (add to workspace deps or crate-local) | `IndexMap` for `Env` scope frame |
| `ordered-float` | workspace re-export via `slideforge-types` (4.6.0) — do not re-declare | `OrderedFloat<f64>` in `Value::Float` |

**Note:** `indexmap` is not currently in `[workspace.dependencies]`. Either add it there
(recommended: `indexmap = "2"`) or declare it crate-local. The `ordered-float` type is
re-exported through `slideforge-types` — do not add a direct dependency.

Dev dependencies:
- `insta` (compatible, no pin required) — for snapshot tests

## File Structure Requirements

Files to create:

```
crates/slideforge-eval/
├── Cargo.toml                     # crate manifest; workspace dep on slideforge-types, slideforge-syntax
├── src/
│   ├── lib.rs                     # crate root; pub use; #![forbid(unsafe_code)]
│   ├── env.rs                     # Env struct — scope chain; push_scope/pop_scope/lookup
│   ├── expr.rs                    # Expression evaluator entry point: eval_expr(&Env, &ExprNode) -> Option<Value>
│   ├── filters.rs                 # 15 built-in filter implementations (pure functions)
│   ├── error.rs                   # EvalError enum + miette::Diagnostic impls; error code constants
│   └── eval.rs                    # eval_field_value, eval_expr_to_string (conversion wrapper)
```

Files NOT to modify in this story: AST definitions in `slideforge-syntax` (read-only).

## Tasks

1. **Create `crates/slideforge-eval/Cargo.toml`** with workspace deps on
   `slideforge-types`, `slideforge-syntax`, `thiserror = { workspace = true }`,
   `miette = { workspace = true }`, and `indexmap = "2"` (add to workspace deps or
   declare crate-local). (15 min)
2. **Write `src/lib.rs`** with `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`,
   `#![deny(clippy::pedantic)]`, and pub module declarations. (10 min)
3. **Write `src/error.rs`** — `EvalError` enum with variants `UndefinedVariable`,
   `TypeMismatch`, `FilterNotFound`, `DivisionByZero`, `FieldAccessFailed`. Each variant
   carries `span: SourceSpan` and `hint: String`. Implement `miette::Diagnostic` on each.
   Define error code constants: `E_EVL_001` through `E_EVL_004`, `E_DAT_005`. (45 min)
4. **Write `src/env.rs`** — `Env` struct with `frames: Vec<IndexMap<Arc<str>, Value>>`.
   Methods: `Env::new(deck_vars: IndexMap<Arc<str>, Value>) -> Env`, `push_scope()`,
   `pop_scope()`, `lookup(&str) -> Option<&Value>`, `all_names() -> Vec<Arc<str>>`.
   Implement `Clone`. (30 min)
5. **Write `src/filters.rs`** — implement all 15 built-in filters as
   `fn filter_name(val: Value, args: &[Value]) -> Result<Value, EvalError>`. Each filter
   must be a standalone pure function. See filter spec in Implementation Notes. (90 min)
6. **Write `src/expr.rs`** — `eval_expr(env: &Env, expr: &ExprNode, sink: &mut DiagnosticSink)
   -> Option<Value>` entry point. Handle: variable references (lookup in env), integer/float
   literals, string literals, binary operators (`+`, `-`, `*`, `/`, `%`), comparison
   operators (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical operators (`&&`, `||`, `!`),
   dot-notation field access, pipe filter chains. (60 min)
7. **Write `src/eval.rs`** — `eval_field_value(env: &Env, fv: &FieldValue, sink: &mut DiagnosticSink)
   -> Option<Value>` and `eval_expr_to_string(env: &Env, expr: &ExprNode, sink: &mut DiagnosticSink)
   -> Option<Arc<str>>` (converts Value to string for embedding). (30 min)
8. **Write unit tests** in each module's `#[cfg(test)] mod tests`. See Test Strategy. (60 min)
9. **Run `cargo clippy -p slideforge-eval -- -D warnings`** and fix all warnings. (15 min)
10. **Run `cargo test -p slideforge-eval`** and confirm all tests pass. (10 min)

## Test Strategy

### Unit tests (`#[cfg(test)] mod tests` in each module)

**`env.rs` tests:**
- `test_env_lookup_deck_var()`: construct Env with `{ a: Int(1) }`; assert `lookup("a") == Some(Int(1))`.
- `test_env_unknown_returns_none()`: lookup of undeclared name returns `None`.
- `test_env_scope_push_pop()`: push scope with `{ b: Str("x") }`; assert `b` accessible; pop scope; assert `b` inaccessible, `a` still accessible.
- `test_env_all_names()`: env with 3 vars; assert `all_names()` contains all 3.

**`filters.rs` tests (one test per filter):**
- `test_filter_upper()`: `upper(Str("hello")) == Str("HELLO")`.
- `test_filter_lower()`: `lower(Str("WORLD")) == Str("world")`.
- `test_filter_currency()`: `currency(Int(1234)) == Str("1,234.00")`.
- `test_filter_currency_float()`: `currency(Float(OrderedFloat(1234.5))) == Str("1,234.50")`.
- `test_filter_round_2()`: `round(Float(OrderedFloat(3.14159)), [Int(2)]) == Str("3.14")`.
- `test_filter_int()`: `int(Str("42")) == Int(42)`.
- `test_filter_float()`: `float(Str("3.14")) == Float(OrderedFloat(3.14))`.
- `test_filter_string()`: `string(Int(99)) == Str("99")`.
- `test_filter_join()`: `join(List([Str("a"), Str("b")]), [Str(", ")]) == Str("a, b")`.
- `test_filter_length_list()`: `length(List([Int(1), Int(2), Int(3)])) == Int(3)`.
- `test_filter_length_str()`: `length(Str("hello")) == Int(5)`.
- `test_filter_trim()`: `trim(Str("  hi  ")) == Str("hi")`.
- `test_filter_default_used()`: `default(Null, [Str("fallback")]) == Str("fallback")`.
- `test_filter_default_not_used()`: `default(Str("val"), [Str("fallback")]) == Str("val")`.
- `test_filter_contains()`: `contains(Str("hello world"), [Str("world")]) == Bool(true)`.
- `test_filter_starts_with()`: `starts_with(Str("hello"), [Str("he")]) == Bool(true)`.
- `test_filter_ends_with()`: `ends_with(Str("hello"), [Str("lo")]) == Bool(true)`.
- `test_filter_replace()`: `replace(Str("foo bar"), [Str("bar"), Str("baz")]) == Str("foo baz")`.

**`expr.rs` tests:**
- `test_arithmetic_mul()`: `{{ n * 2 }}` with `n=5` → `Int(10)`.
- `test_arithmetic_div_zero()`: `{{ 10 / 0 }}` → None + E-EVL-003 in sink.
- `test_string_concat_with_tilde()`: `{{ "Hello " ~ name }}` with `name="World"` → `Str("Hello World")`.
- `test_dot_access()`: `{{ item.price }}` with `item = Map { "price": Int(42) }` → `Int(42)`.
- `test_dot_access_missing()`: `{{ item.missing }}` → None + E-DAT-005.
- `test_pipe_chain()`: `{{ name | upper }}` → `Str("WORLD")` (name="world").
- `test_pipe_unknown_filter()`: `{{ name | bogus }}` → None + E-EVL-004.
- `test_undefined_var()`: `{{ greeting }}` with empty scope → None + E-EVL-001 with scope list `[]`.
- `test_undefined_var_scope_listed()`: `{{ c }}` with `a="x", b="y"` → E-EVL-001 listing `[a, b]`.
- `test_two_errors_accumulated()`: two undefined references → sink has 2 diagnostics.
- `test_type_mismatch_arith()`: `"NO" + 1` → None + E-EVL-003.

### Snapshot tests

- `test_snapshot_eval_minimal_deck()`: evaluate minimal 1-slide deck; snapshot the evaluated
  `Value` for the title field.
- `test_snapshot_pipe_chain()`: `{{ price | currency }}` with known value; snapshot string result.

## Dependencies

**Depends on:**
- STORY-001 (slideforge-types: `Value`, `Deck`, `Slide`, `SourceSpan`, `Register`)
- STORY-005 (slideforge-syntax: lexer token types — indirect)
- STORY-006 (slideforge-syntax: `DeckNode`, `SlideNode`, `FieldNode`, `FieldValue`, `Spanned<T>`)
- STORY-010 (slideforge-syntax/infra: `DiagnosticSink`, error accumulation contract)

**Blocks:**
- STORY-012: variable scoping + @for evaluation builds on `Env` defined here.
- STORY-013: @if evaluation and @include cycle detection use `eval_expr()` defined here.
- STORY-014: no-coercion + ${{ seq }} disambiguation builds on this evaluator.
- STORY-015 through STORY-017: validation passes run after evaluation.
- STORY-018+: data sources produce `Value` IR that this evaluator consumes.
- STORY-035: writing register routing in evaluator extends `eval.rs` in this story.

### Dependency Anchor Justifications

- SS-02 owns this story's scope because SS-02 is the Evaluator subsystem in the
  ARCH-INDEX Subsystem Registry and `slideforge-eval` is its sole crate.
- STORY-011 depends on STORY-001 because `slideforge-eval` consumes `Value`, `Deck`,
  and `SourceSpan` types from `slideforge-types`.
- STORY-011 depends on STORY-006 because the evaluator's input is `DeckNode` + `SlideNode`
  AST nodes produced by the parser.
- STORY-011 depends on STORY-010 because `DiagnosticSink` (defined in STORY-010) is the
  mechanism for error accumulation used throughout the evaluator.
- STORY-011 blocks STORY-012/013/014 because those stories extend the evaluator defined here.

## Implementation Notes

### `Env` Type Design

```rust
/// Lexical scope chain for expression evaluation.
/// Frame 0 is the deck-level vars block; additional frames are pushed
/// for @for iterations and @if branches.
#[derive(Debug, Clone)]
pub struct Env {
    frames: Vec<IndexMap<Arc<str>, Value>>,
}

impl Env {
    /// Create a new Env with the deck-level vars as the base frame.
    pub fn new(deck_vars: IndexMap<Arc<str>, Value>) -> Self { ... }

    /// Push a new scope frame (called on @for entry).
    pub fn push_scope(&mut self, bindings: IndexMap<Arc<str>, Value>) { ... }

    /// Pop the innermost scope frame (called on @for exit).
    /// Panics if called on an empty env (logic error).
    pub fn pop_scope(&mut self) { ... }

    /// Look up a variable in the scope chain, innermost frame first.
    pub fn lookup(&self, name: &str) -> Option<&Value> { ... }

    /// Return all variable names visible at the current scope level.
    /// Used for E-EVL-001 scope-path reporting.
    pub fn all_names(&self) -> Vec<Arc<str>> { ... }
}
```

### `EvalError` Variants and Error Codes

```rust
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum EvalError {
    #[error("Undefined variable '{{{{ {name} }}}}' at {span}. Variables in scope: [{scope_list}]")]
    #[diagnostic(code(E_EVL_001), help("Check spelling or declare the variable in vars: block"))]
    UndefinedVariable { name: Arc<str>, scope_list: String, span: SourceSpan },

    #[error("Type mismatch: {message} at {span}")]
    #[diagnostic(code(E_EVL_003), help("Use | int, | float, or | string to convert values explicitly"))]
    TypeMismatch { message: String, span: SourceSpan },

    #[error("Filter '{name}' not found at {span}. Available: [{available}]")]
    #[diagnostic(code(E_EVL_004), help("Use one of the listed built-in filters"))]
    FilterNotFound { name: Arc<str>, available: String, span: SourceSpan },

    #[error("Division by zero at {span}")]
    #[diagnostic(code(E_EVL_003), help("Ensure divisor is non-zero before dividing"))]
    DivisionByZero { span: SourceSpan },

    #[error("Field '{field}' not found in {parent_type} at {span}")]
    #[diagnostic(code(E_DAT_005), help("Check field name spelling or data source schema"))]
    FieldAccessFailed { field: Arc<str>, parent_type: Arc<str>, span: SourceSpan },
}
```

### 15 Built-in Filter Functions

| Filter | Signature | Behavior |
|--------|-----------|---------|
| `upper` | `(Str) -> Str` | Uppercase the string |
| `lower` | `(Str) -> Str` | Lowercase the string |
| `trim` | `(Str) -> Str` | Strip leading/trailing whitespace |
| `string` | `(Value) -> Str` | Convert any value to string representation |
| `int` | `(Str or Float) -> Int` | Parse string to integer; E-EVL-003 on fail |
| `float` | `(Str or Int) -> Float` | Parse string to float; E-EVL-003 on fail |
| `currency` | `(Int or Float) -> Str` | Format as `"1,234.00"` (comma-thousands, 2 decimal) |
| `round(n)` | `(Float, Int) -> Str` | Round to n decimal places; return string |
| `join(sep)` | `(List, Str) -> Str` | Join list elements with separator |
| `length` | `(Str or List) -> Int` | Count chars (Str) or items (List) |
| `replace(from, to)` | `(Str, Str, Str) -> Str` | Replace all occurrences of `from` with `to` |
| `default(val)` | `(Value, Value) -> Value` | Return first arg if not Null, else second |
| `contains(sub)` | `(Str, Str) -> Bool` | True if str contains sub |
| `starts_with(prefix)` | `(Str, Str) -> Bool` | True if str starts with prefix |
| `ends_with(suffix)` | `(Str, Str) -> Bool` | True if str ends with suffix |

### `ExprNode` — Expected AST Node (from STORY-007)

The expression evaluator in this story is built against the `ExprNode` type that
STORY-007 defines in `slideforge-syntax`. If STORY-007 has not yet been merged when
this story is implemented, define a minimal stub `ExprNode` in `slideforge-eval`'s
test helpers that mirrors the expected shape. The stub must be replaced with the real
type before this story can be merged.

Expected `ExprNode` structure (from STORY-007's Implementation Notes):
```rust
pub enum ExprNode {
    Var { name: Spanned<String> },
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    BinOp { op: BinOp, lhs: Box<ExprNode>, rhs: Box<ExprNode>, span: Span },
    UnaryOp { op: UnaryOp, operand: Box<ExprNode>, span: Span },
    FieldAccess { object: Box<ExprNode>, field: Spanned<String> },
    Pipe { value: Box<ExprNode>, filter: Spanned<String>, args: Vec<ExprNode> },
}
```

### String Concatenation Operator

Use `~` (tilde) as the string concatenation operator, distinct from `+` (numeric addition).
`{{ "Hello " ~ name }}` concatenates two strings. `{{ a + b }}` requires both to be numeric.
This distinction prevents the "string + string = garbage" class of errors from other DSLs.

### `eval_expr_to_string` Contract

This function wraps `eval_expr()` and converts the result `Value` to `Arc<str>` for
field embedding:
- `Value::Str(s)` → `s` (identity)
- `Value::Int(n)` → `n.to_string().into()`
- `Value::Float(f)` → format with enough precision to preserve value (not scientific)
- `Value::Bool(b)` → `"true"` or `"false"` (explicit, never implicit)
- `Value::Null` → `""` (empty string — Null in interpolation context = empty)
- `Value::List(_)` or `Value::Map(_)` → E-EVL-003 (use `| join` for lists; map access requires dot notation)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `{{ price * 1.2 | round(2) }}` — arithmetic then filter | Left-to-right: `price * 1.2` produces Float; `round(2)` formats to 2 decimals |
| EC-002 | `{{ items | join(", ") }}` — filter with string arg | Arg passed to `join`; list → comma-joined string |
| EC-003 | `{{ 10 / 0 }}` — division by zero | E-EVL-003 (not panic); sink collects error |
| EC-004 | Deeply nested field access `{{ a.b.c.d }}` | Step-by-step; E-DAT-005 at first missing level |
| EC-005 | `{{ name | unknown_filter }}` | E-EVL-004 with available filter list |
| EC-006 | Variable name same as filter name (e.g., `upper = "hello"`) | Resolved as variable; `upper` the variable is looked up in env; name collision is user error but not a crash |
| EC-007 | `{{ null_var }}` where null_var is `Value::Null` | Evaluates to `Null`; `eval_expr_to_string` produces empty string |
| EC-008 | Pipe chain of 5 filters | Applied left-to-right without intermediate allocation blowup |
