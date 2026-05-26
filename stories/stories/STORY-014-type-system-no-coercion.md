---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-014
title: "Type System: No Implicit Coercion + ${{ seq }} Disambiguation"
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
  - BC-1.02.003
  - BC-1.02.004
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-011
  - STORY-012
blocks:
  - STORY-066
  - STORY-067
estimated_days: 2
---

# STORY-014: Type System: No Implicit Coercion + ${{ seq }} Disambiguation

## Summary

Implement and test the no-implicit-coercion invariant (BC-1.02.003) in `slideforge-eval`
and the `${{ expr }}` special-case lexer/evaluator disambiguation (BC-1.02.004). These
are correctness contracts, not new features — they formalize and test behaviors that
MUST already be present in STORY-011 but require dedicated story coverage to produce
named test vectors, snapshot tests, and edge case documentation.

**No implicit coercion (BC-1.02.003):** String `"NO"` stays string `"NO"` throughout
evaluation. String `"1.10"` stays string `"1.10"`. Numeric operations on string values
produce E-EVL-003 with the actual and expected types. No YAML-style coercion rules apply.
Explicit format filters (`| int`, `| float`, `| bool`, `| string`) are the only type
conversion mechanism.

**${{ seq }} disambiguation (BC-1.02.004):** The sequence `${{` in text mode is parsed
as a text-mode interpolation with a literal `$` prefix, NOT as the start of a math mode
block. This covers DEC-009.

This story adds targeted tests, documents the invariant in code comments, and ensures
the Kani VP stubs are in place for STORY-066/067.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-1.02.003 | No implicit type coercion — values are never silently converted | All 5 postconditions + 3 invariants |
| BC-1.02.004 | Handle ${{ seq }} as text interpolation, not math delimiter | All 4 postconditions + 3 invariants |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| BC files (2 BCs) | ~2,000 |
| STORY-011 eval.rs reference | ~2,000 |
| Target test files to write | ~4,000 |
| Kani proof stubs | ~1,000 |
| **Total** | **~12,500** |

Agent context budget: 200k tokens. This story is ~6% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001** — `vars: { flag: "NO" }` / `title "{{ flag }}"` evaluates to
  `title="NO"` (the literal string "NO", not "false" or any boolean representation).
  (traces to BC-1.02.003 postcondition 1, postcondition 2)

- [ ] **AC-002** — `vars: { v: "1.10" }` / `title "{{ v }}"` evaluates to
  `title="1.10"` (the literal string "1.10", not "1.1").
  (traces to BC-1.02.003 postcondition 1, postcondition 3)

- [ ] **AC-003** — `vars: { v: "NO" }` in arithmetic `{{ v + 1 }}` produces E-EVL-003:
  `Type mismatch: operator '+' expects Numeric (Int or Float), got String at <file>:<line>:<col>.`
  Build exits 2.
  (traces to BC-1.02.003 postcondition 4)

- [ ] **AC-004** — `vars: { v: "NO" }` in comparison `@if v == false:` produces E-EVL-003:
  `Type mismatch: comparison '==' between String and Bool at <file>:<line>:<col>. Use explicit conversion.`
  (traces to BC-1.02.003 postcondition 5)

- [ ] **AC-005** — `vars: { yes_value: "yes" }` / `@if yes_value:` produces E-EVL-003
  (no implicit boolean coercion of string "yes").
  (traces to BC-1.02.003 invariant 2)

- [ ] **AC-006** — Explicit conversion `{{ v | int }}` on `v = "42"` produces `Int(42)`.
  Explicit conversion `{{ v | float }}` on `v = "3.14"` produces `Float(3.14)`.
  Explicit `| bool` is NOT a valid conversion path (no implicit bool coercion — use
  `== "true"` comparison instead). `| bool` filter does not exist; E-EVL-004 if used.
  (traces to BC-1.02.003 invariant 1)

- [ ] **AC-007** — `vars: { arr: 1000000 }` / `stat "${{ arr | currency }}"` evaluates
  to `stat="$1,000,000"`. The `$` is a literal prefix prepended to the interpolation
  result. Math mode is NOT activated.
  (traces to BC-1.02.004 postcondition 1, postcondition 2, postcondition 4)

- [ ] **AC-008** — `title "${{ count }} items"` with `count=5` evaluates to `title="$5 items"`.
  (traces to BC-1.02.004 postcondition 4)

- [ ] **AC-009** — `$` as a standalone character followed by a space (e.g., `"Cost: $ {{ price }}"`)
  is NOT a `${{` interpolation trigger. `$` is literal text; `{{ price }}` is normal
  interpolation.
  (traces to BC-1.02.004 invariant 2)

- [ ] **AC-010** — `$${{ n }}` is NOT a `${{` interpolation. `$$` opens math display mode;
  `{{` inside is NOT text interpolation (math mode uses `@{var}` not `{{ }}`).
  (traces to BC-1.02.004 invariant 3)

- [ ] **AC-011** — Kani proof stub functions are present in `crates/slideforge-eval/src/proofs/`
  with `#[cfg(kani)]` guards. The stubs compile with `cargo check`. Full proofs
  implemented in STORY-067.
  (traces to VP-004 stub, VP-005 stub)

- [ ] **AC-012** — `cargo test -p slideforge-eval` passes. New tests for this story
  are added to a dedicated `tests/type_system_tests.rs` integration test file.

- [ ] **AC-013** — `#![forbid(unsafe_code)]`, zero `.unwrap()`, `clippy::pedantic` clean,
  all public items documented.
  (traces to NFR-021, NFR-022, NFR-023, NFR-024)

## Previous Story Intelligence

STORY-011 already implements the E-EVL-003 type mismatch path in `eval_expr()`. This
story does NOT re-implement that logic — it adds dedicated test coverage and documents
the contract formally.

Key STORY-011 behaviors to preserve:
- `eval_expr_to_string()` converts `Value::Bool(b)` to `"true"` / `"false"` explicitly.
- No `From<bool>` or `From<i64>` for `Value` exists (verified in STORY-001 AC-015).
- `${{` token disambiguation is implemented in STORY-005 (lexer) — the evaluator receives
  a pre-tokenized stream where `${{` is already resolved.

Lessons from prior stories: explicitly test edge cases that YAML tools get wrong:
- `"NO"` must not become `false`
- `"true"` must not become `true`
- `"1.10"` must not become `1.1`
- `"0"` must not become `0` (integer)
These are the exact examples from BC-1.02.003's test vectors. Cover all four.

## Architecture Compliance Rules

1. **Invariant DI-004 enforcement:** The no-coercion invariant (DI-004) is enforced at
   the evaluator level, not just at the type definition level. Every arithmetic and
   comparison operation MUST check operand types before proceeding.
2. **No `bool` filter:** There is no `| bool` filter. Boolean conversion is only via
   explicit comparison: `{{ v == "true" }}` or `{{ v == 1 }}`. Implementing `| bool`
   would create an implicit coercion path. The test suite MUST verify E-EVL-004 for
   `| bool`.
3. **`${{` is a lexer concern:** The `${{` → text-mode interpolation disambiguation is
   handled in STORY-005 (lexer). The evaluator receives a `DollarInterpolation` AST
   node, not a raw `${{` token. This story verifies the full pipeline end-to-end.
4. **Kani VP stubs:** Place stub proofs in `src/proofs/no_coercion.rs` and
   `src/proofs/no_overflow.rs` under `#[cfg(kani)]`. Stubs compile to `todo!()` bodies.
   Full proofs are in STORY-067.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace | `Value`, `SourceSpan` |
| `slideforge-syntax` | workspace | `DeckNode`, `ExprNode`, `DollarInterpolation` |
| `thiserror` | `=2.0.18` | (already in Cargo.toml) |
| `miette` | workspace (`"7"` with `fancy` feature — resolves to latest 7.x, currently 7.6.0) | (already in Cargo.toml) |

Dev dependencies: `insta` (compatible)

## File Structure Requirements

Files to create (new in this story):

```
crates/slideforge-eval/
├── src/
│   └── proofs/
│       ├── mod.rs              # #[cfg(kani)] mod; pub mod no_coercion; pub mod no_overflow
│       ├── no_coercion.rs      # Kani stub proof: no implicit coercion (VP-004)
│       └── no_overflow.rs      # Kani stub proof: no arithmetic overflow (VP-005)
├── tests/
│   └── type_system_tests.rs    # Integration tests for BC-1.02.003 and BC-1.02.004
```

Files to extend (modify, do NOT replace):

```
crates/slideforge-eval/
├── src/
│   └── expr.rs    # ADD: explicit type-check comments citing DI-004 and E-EVL-003 at each
│                  # binary op; no logic changes — documentation and assertion only
```

## Tasks

1. **Write `tests/type_system_tests.rs`** — integration tests for all ACs.
   See Test Strategy. (60 min)
2. **Create `src/proofs/mod.rs`** with `#[cfg(kani)]` gate; declare stub modules. (10 min)
3. **Write `src/proofs/no_coercion.rs`** — stub Kani proof with `#[cfg(kani)]`:
   ```rust
   #[cfg(kani)]
   #[kani::proof]
   fn verify_no_string_to_int_coercion() {
       todo!("Implement in STORY-067")
   }
   ```
   (15 min)
4. **Write `src/proofs/no_overflow.rs`** — stub Kani proof for arithmetic overflow. (10 min)
5. **Annotate `src/expr.rs`** with `// DI-004: no implicit coercion — E-EVL-003 below`
   comments at each type-check site. No logic changes. (15 min)
6. **Extend `src/lib.rs`** to include `#[cfg(kani)] mod proofs;`. (5 min)
7. **Run `cargo test -p slideforge-eval`** and confirm all tests pass. (10 min)

## Test Strategy

### Integration tests (`tests/type_system_tests.rs`)

**No-coercion tests (BC-1.02.003):**
- `test_no_flag_string_stays_no()`: `flag = "NO"` / `{{ flag }}` → `"NO"`.
- `test_version_string_stays_unchanged()`: `v = "1.10"` / `{{ v }}` → `"1.10"` (not `"1.1"`).
- `test_yes_string_stays_yes()`: `yes_value = "yes"` / `{{ yes_value }}` → `"yes"`.
- `test_zero_string_stays_string()`: `zero = "0"` / `{{ zero }}` → `"0"` (not 0).
- `test_arith_on_string_produces_type_error()`: `{{ "NO" + 1 }}` → E-EVL-003.
- `test_arith_string_rate()`: `rate = "2.5"` / `{{ rate * 100 }}` → E-EVL-003 with hint `use {{ rate | float * 100 }}`.
- `test_compare_string_to_bool()`: `"NO" == false` → E-EVL-003 (string vs Bool).
- `test_explicit_int_conversion()`: `v = "42"` / `{{ v | int + 1 }}` → `"43"`.
- `test_explicit_float_conversion()`: `v = "3.14"` / `{{ v | float * 2 }}` → `"6.28"`.
- `test_bool_filter_does_not_exist()`: `{{ flag | bool }}` → E-EVL-004 (no such filter).
- `test_string_in_condition_type_error()`: `@if "true":` → E-EVL-003.
- `test_int_in_condition_type_error()`: `@if 0:` → E-EVL-003.

**${{ seq }} disambiguation tests (BC-1.02.004):**
- `test_dollar_interpolation_basic()`: `arr = 1000000` / `"${{ arr | currency }}"` → `"$1,000,000"`.
- `test_dollar_interpolation_count()`: `count = 5` / `"${{ count }} items"` → `"$5 items"`.
- `test_dollar_space_not_interpolation()`: `"$ {{ price }}"` (space between $ and {{) → `$ ` + price value separately.
- `test_dollar_dollar_brace_is_math_mode()`: `$${{ n }}` — `$$` opens math mode; `{{ n }}` inside is not text interpolation.

### Kani stubs (compile-only check)

- `cargo check -p slideforge-eval` with `--cfg kani` must not produce compile errors
  for stub proof modules.

## Dependencies

**Depends on:**
- STORY-011 (eval_expr, EvalError::TypeMismatch — the actual enforcement is here)
- STORY-012 (eval_deck, Env — needed to construct test scenarios with deck vars)

**Blocks:**
- STORY-066 (Kani proofs for slideforge-syntax — indirect)
- STORY-067 (Kani proofs for slideforge-eval — fills in the stubs from this story)

### Dependency Anchor Justifications

- SS-02 owns this story's scope because SS-02 is the Evaluator subsystem and this story
  tests the evaluator's type invariants.
- STORY-014 depends on STORY-011 because the no-coercion contract is enforced in
  `eval_expr()` from STORY-011; this story tests that behavior.
- STORY-014 depends on STORY-012 because `eval_deck()` from STORY-012 is the entry
  point for integration tests.
- STORY-014 blocks STORY-067 because the Kani proof stubs created here are the anchor
  points that STORY-067 fills in.

## Implementation Notes

### Why `| bool` Must Not Exist

The `| bool` filter would allow `{{ "yes" | bool }}` to silently produce `true`,
introducing implicit coercion through a filter. The correct pattern is `{{ v == "yes" }}`.
This is an intentional design decision documented in the Q3 decisions (q3-decision-final.md).

The test `test_bool_filter_does_not_exist()` is a regression guard — it must never pass.
If someone adds `| bool` in a future story, this test catches it immediately.

### `DollarInterpolation` AST Node

In STORY-005 (lexer), the `${{` sequence is tokenized as `Token::DollarInterp` (or
similar). In STORY-007 (parser), this produces a `FieldValue::DollarInterp(ExprNode)`
AST node. The evaluator (STORY-011) handles this by:
1. Evaluating `ExprNode` normally via `eval_expr()`.
2. Converting result to string via `eval_expr_to_string()`.
3. Prepending `"$"` to the string result.

This story verifies the end-to-end pipeline produces the correct output. If the
`DollarInterp` AST node name differs from what STORY-007 produces, update accordingly.

### Precision Preservation for "1.10"

The `"1.10"` invariant is subtle: JSON and YAML parsers commonly normalize `1.10` to
`1.1` by dropping trailing zeros. slideforge must NOT do this. The test vector
`test_version_string_stays_unchanged()` guards against regressions.

Since `"1.10"` is declared as a string (with quotes), it is stored as `Value::Str("1.10")`.
The string identity invariant is trivially satisfied — no float parsing occurs.
The test confirms the full pipeline from `vars: { v: "1.10" }` to `{{ v }}` output.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `vars: { flag: "NO" }` used in `@if flag == false:` | E-EVL-003: comparing String to Bool |
| EC-002 | `vars: { rate: "1.10" }` used in `{{ rate * 100 }}` | E-EVL-003 with hint about `| float` |
| EC-003 | `vars: { count: 42 }` (integer) used in string concat without explicit conversion | E-EVL-003: `+` expects String on both sides when one operand is Str; use `~` for concat or `| string` for conversion |
| EC-004 | `"${{ amount }} USD"` with amount=1500 | Result: `"$1500 USD"` |
| EC-005 | `$${{ n }}` in source | `$$` is math display mode open; `{{ n }}` inside is not text interpolation; E-EVL-002 if `n` is undefined in math context |
| EC-006 | `| bool` filter used | E-EVL-004: filter 'bool' not found. Available: [upper, lower, ...] (bool is not in the list) |
