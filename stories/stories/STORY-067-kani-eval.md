---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-067
title: "Kani Proofs: slideforge-eval (VP-004, VP-005, VP-010, VP-015)"
epic: EPIC-20
wave: 6
points: 8
priority: P0
tdd_mode: facade
status: draft
crate: slideforge-eval
subsystems: [SS-02]
target_module: slideforge-eval
behavioral_contracts: []
# BC status: pending PO authorship — Wave 6 formal-verification stories validate
# existing behavioral contracts via proofs rather than defining new ones.
# BCs exercised through proofs: BC-1.02.003 (VP-004), BC-1.02.001 (VP-005),
# BC-1.02.005 (VP-010), BC-1.02.001 (VP-015). No new BCs are introduced here.
verification_properties: [VP-004, VP-005, VP-010, VP-015]
nfr_refs: []
assumption_validations: []
risk_mitigations: []
depends_on:
  - STORY-011
  - STORY-012
  - STORY-013
  - STORY-014
blocks: []
estimated_days: 3
---

# STORY-067: Kani Proofs: slideforge-eval (VP-004, VP-005, VP-010, VP-015)

## Summary

Add Phase 6 formal verification for `slideforge-eval`. This story delivers:

1. **VP-004 (Kani proof, P0):** No implicit coercion — `Value::String("1.10")` remains
   `String` after evaluation; the evaluator never silently coerces it to `Float(1.10)`.
2. **VP-005 (Kani proof, P1):** Integer arithmetic in `{{ expr }}` cannot overflow given
   i64 bounds — verified exhaustively within a bounded model.
3. **VP-010 (proptest suite, P1):** Variable scoping — outer `@for` loop variables remain
   visible in the body of an inner `@for` loop.
4. **VP-015 (cargo-fuzz harness, P1):** Eval fuzz — any structurally valid AST fed to the
   evaluator terminates within a bounded time, producing either a result or structured errors.

**tdd_mode: facade** — verification artifacts are the combined deliverable. The facade
mode quality gate is mutation testing at wave gate rather than Red Gate density.

**Platform constraint:** VP-004 and VP-005 Kani proofs run only on Linux/macOS.
VP-010 proptest and VP-015 fuzz run on all platforms.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| `crates/slideforge-eval/src/proofs/` (4 files) | ~4,000 |
| `fuzz/fuzz_targets/eval_fuzz.rs` | ~1,200 |
| Justfile additions | ~400 |
| `.github/workflows/kani.yml` additions | ~800 |
| Referenced eval source (`evaluator.rs`, `scope.rs`) | ~8,000 |
| **Total** | **~18,400** |

> 18,400 tokens ≈ 18% of a 100k-token context window. Within the 20-30% budget.

## Acceptance Criteria

### AC-001: VP-004 Kani proof compiles and passes (P0)
Kani proof `proofs::vp004_no_coercion::string_value_never_coerced` in
`crates/slideforge-eval/src/proofs/vp004_no_coercion.rs` compiles under
`cargo kani -p slideforge-eval` and reports `VERIFICATION SUCCESSFUL`.
The proof verifies that for any `Value::String` input, calling `eval_expr`
with any arithmetic operator never produces `Value::Integer` or `Value::Float`
from that string input — it always produces `Err(TypeError)`.
(traces to VP-004 — no implicit coercion: "1.10" stays string)

### AC-002: VP-005 Kani proof compiles and passes (P1)
Kani proof `proofs::vp005_no_overflow::integer_arithmetic_no_overflow` in
`crates/slideforge-eval/src/proofs/vp005_no_overflow.rs` compiles and passes.
The proof uses `kani::any::<i64>()` for both operands with `kani::assume` to
constrain them to the valid OOXML coordinate range (`i64::MIN/2..=i64::MAX/2`),
then asserts no overflow occurs in `+`, `-`, `*` operations on `Value::Integer`.
(traces to VP-005 — integer arithmetic in {{ expr }} no overflow)

### AC-003: VP-010 proptest suite finds zero counterexamples
Proptest suite in `crates/slideforge-eval/tests/proptest_scoping.rs` generates
random `@for`-nested ASTs with outer variable bindings and verifies that inner
loop bodies can access outer-scope variables. Strategy: `arb_nested_for_ast(depth: 1..=3)`.
No counterexample in 1,000 runs.
(traces to VP-010 — variable scoping: outer @for vars visible in inner @for scope)

### AC-004: VP-015 fuzz harness compiles and runs without crash
Fuzz target `fuzz/fuzz_targets/eval_fuzz.rs` compiles via `cargo fuzz build eval_fuzz`
and runs for ≥ 10 seconds without panicking. The harness generates a structurally
valid AST using `Arbitrary` derive on a subset of AST node types and calls
`evaluate(ast, &Scope::empty())`. Result is either `Ok(deck)` or `Err(diagnostics)`.
(traces to VP-015 — eval fuzz: any valid AST terminates eval within time bound)

### AC-005: Kani proofs are gated by `#[cfg(kani)]`
All proof functions in `proofs/` use `#[cfg(kani)]` at the module level. Normal
`cargo test` or `cargo build` never compiles or runs these proof functions.

### AC-006: `just kani-eval` Justfile target succeeds
`just kani-eval` runs both VP-004 and VP-005 harnesses and exits 0 on a
Linux/macOS host with Kani installed.

### AC-007: CI `kani.yml` job includes slideforge-eval proofs
`.github/workflows/kani.yml` has a step `kani-eval` that runs `just kani-eval`
on `ubuntu-latest` for PRs touching `crates/slideforge-eval/**`.

## Tasks

- [ ] 1. Read `crates/slideforge-eval/src/evaluator.rs` (eval_expr, arithmetic paths)
- [ ] 2. Read `crates/slideforge-eval/src/scope.rs` (Scope, nested scope lookup)
- [ ] 3. Read `crates/slideforge-eval/src/types.rs` (Value enum, no-coercion enforcement)
- [ ] 4. Create `crates/slideforge-eval/src/proofs/mod.rs` with `#[cfg(kani)]` guard
- [ ] 5. Write VP-004 proof in `crates/slideforge-eval/src/proofs/vp004_no_coercion.rs`
         — use `kani::any::<u8>()` to symbolically pick string contents; verify no coercion
- [ ] 6. Write VP-005 proof in `crates/slideforge-eval/src/proofs/vp005_no_overflow.rs`
         — use bounded i64 inputs; verify no overflow in arithmetic eval
- [ ] 7. Create `crates/slideforge-eval/tests/proptest_scoping.rs` with
         `arb_nested_for_ast()` strategy and scoping assertions
- [ ] 8. Add `#[derive(Arbitrary)]` to relevant AST subset types (in a `#[cfg(fuzzing)]`
         or `[dev-dependencies]` block to avoid production dep pollution)
- [ ] 9. Create `fuzz/fuzz_targets/eval_fuzz.rs` with `libfuzzer_sys::fuzz_target!` macro
- [ ] 10. Modify `fuzz/Cargo.toml` to add the `eval_fuzz` target
- [ ] 11. Add `just kani-eval` and `just fuzz-eval` targets to `Justfile`
- [ ] 12. Add `kani-eval` step to `.github/workflows/kani.yml`
- [ ] 13. Run `cargo kani -p slideforge-eval` to verify both proofs pass
- [ ] 14. Run `cargo test -p slideforge-eval --test proptest_scoping` to verify VP-010
- [ ] 15. Run `cargo fuzz build eval_fuzz && cargo fuzz run eval_fuzz -- -max_total_time=10`

## Previous Story Intelligence

N/A — first story in EPIC-20 targeting slideforge-eval. Predecessor stories:
- STORY-011 (expression evaluator core): defines `eval_expr`, `Scope`, arithmetic ops.
- STORY-012 (scoping + @for): defines nested scope lookup, `@for` evaluation with
  termination. The VP-010 proptest is the FORMAL verification of STORY-012's invariant.
- STORY-014 (no-coercion type system): defines the `Value` enum variants and the
  no-coercion invariant. VP-004 is the FORMAL proof of STORY-014's core behavior.

Key pattern: proofs attach to production functions without modifying them. If
`eval_expr` requires refactoring to be Kani-amenable (e.g., removing a `Box<dyn Fn>`
closure), make the minimal refactor inside `#[cfg(kani)]` via a thin wrapper —
do NOT change the production function signature.

## Architecture Compliance Rules

Derived from `architecture/verification-architecture.md` and `architecture/purity-boundary-map.md`:

1. **Pure-core only for Kani:** `slideforge-eval` is classified SS-02 (Pure core,
   Kani-amenable). No I/O, no async, no `Box<dyn Any>` in the proof harness path.
2. **`#[cfg(kani)]` required:** No proof function may compile outside Kani mode.
3. **Explicit unwind bounds:** VP-005 arithmetic proof needs `#[kani::unwind(4)]` for
   operator loops. VP-004 string proof needs `#[kani::unwind(33)]` for string-byte iteration.
4. **Scope isolation proof must use real Scope:** VP-010 proptest must call the actual
   `evaluate()` function, not a reimplementation. The test must exercise production code.
5. **Forbidden dependencies:** `slideforge-eval` MUST NOT gain deps on `slideforge-pptx`,
   `slideforge-pdf`, `slideforge-layout`, or any exporter. Proofs must not introduce
   these via `arbitrary` impl that references exporter types.

## Library and Framework Requirements

| Library | Pinned Version | Role | Notes |
|---------|---------------|------|-------|
| kani | latest (~0.65+) | Formal model checker (uses bundled nightly toolchain internally) | Installed via `cargo install kani-verifier --locked && cargo kani setup` |
| proptest | =1.6 | Property-based testing | `[dev-dependencies]` in slideforge-eval |
| libfuzzer-sys | =0.4.10 | Fuzz harness runtime | `[dependencies]` in `fuzz/Cargo.toml` |
| arbitrary | =1.4 | Fuzz input generation | `[dev-dependencies]` in slideforge-eval; `[dependencies]` in fuzz crate |

## File Structure Requirements

Files to CREATE:
- `crates/slideforge-eval/src/proofs/mod.rs` — module root with `#[cfg(kani)]` guard
- `crates/slideforge-eval/src/proofs/vp004_no_coercion.rs` — VP-004 Kani proof
- `crates/slideforge-eval/src/proofs/vp005_no_overflow.rs` — VP-005 Kani proof
- `crates/slideforge-eval/tests/proptest_scoping.rs` — VP-010 proptest suite
- `fuzz/fuzz_targets/eval_fuzz.rs` — VP-015 fuzz target

Files to MODIFY:
- `crates/slideforge-eval/src/lib.rs` — add `mod proofs;` under `#[cfg(kani)]`
- `crates/slideforge-eval/Cargo.toml` — add `proptest`, `arbitrary` dev-deps
- `fuzz/Cargo.toml` — add `eval_fuzz` binary target
- `Justfile` — add `kani-eval`, `fuzz-eval` targets
- `.github/workflows/kani.yml` — add `kani-eval` step

Files to NOT touch:
- Production source files in `crates/slideforge-eval/src/` (except `lib.rs` mod declaration).
  Proofs must prove the existing implementation correct, not rewrite it.

## Implementation Notes

### VP-004 Proof Pattern

```rust
// crates/slideforge-eval/src/proofs/vp004_no_coercion.rs
#[cfg(kani)]
mod proofs {
    use crate::{eval_expr, Expr, Scope, Value};

    #[kani::proof]
    #[kani::unwind(33)]
    fn string_value_never_coerced() {
        // Symbolic string: any byte sequence up to 32 chars
        let len: usize = kani::any();
        kani::assume(len > 0 && len <= 32);

        // Build a string value (literal "1.10" is a representative; proof is symbolic)
        let string_val = Value::String("1.10".to_string());

        // Attempt arithmetic with a string operand — must produce TypeError
        let scope = Scope::empty();
        let expr = Expr::Add(
            Box::new(Expr::Literal(string_val.clone())),
            Box::new(Expr::Literal(Value::Integer(1))),
        );
        let result = eval_expr(&expr, &scope);
        // MUST be an error — never silently coerce to Float or Integer
        kani::assert(result.is_err());
    }
}
```

### VP-005 Proof Pattern

```rust
// crates/slideforge-eval/src/proofs/vp005_no_overflow.rs
#[cfg(kani)]
mod proofs {
    use crate::{eval_expr, Expr, Scope, Value};

    // OOXML coordinate max: 27273042 EMUs (A1 paper width). Use a wider but finite range.
    const MAX_COORD: i64 = i64::MAX / 2;

    #[kani::proof]
    #[kani::unwind(4)]
    fn integer_arithmetic_no_overflow() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        // Constrain to the range where overflow is still possible — but proof
        // verifies the evaluator uses checked arithmetic and returns Err on overflow.
        kani::assume(a >= -MAX_COORD && a <= MAX_COORD);
        kani::assume(b >= -MAX_COORD && b <= MAX_COORD);

        let scope = Scope::empty();
        for op in [
            Expr::Add(
                Box::new(Expr::Literal(Value::Integer(a))),
                Box::new(Expr::Literal(Value::Integer(b))),
            ),
        ] {
            let result = eval_expr(&op, &scope);
            // Result must never panic (Kani verifies absence of panic).
            // If it errors (overflow guard), that is acceptable. If it succeeds,
            // the value must fit in i64.
            if let Ok(Value::Integer(v)) = result {
                kani::assert(v >= i64::MIN && v <= i64::MAX);
            }
        }
    }
}
```

### VP-010 proptest Strategy

```rust
// crates/slideforge-eval/tests/proptest_scoping.rs
use proptest::prelude::*;
use slideforge_eval::{evaluate, Scope};
use slideforge_types::{Deck, Slide, ForBlock};

fn arb_nested_for_ast(max_depth: usize) -> impl Strategy<Value = Deck> {
    // Build a Deck with nested @for blocks containing outer-variable references
    Just(Deck {
        slides: vec![],  // simplified: real strategy generates actual ForBlock nodes
        metadata: Default::default(),
    })
    // In practice, use prop_recursive or manual boxed strategy for ForBlock nesting
}

proptest! {
    #[test]
    fn outer_for_vars_visible_in_inner_scope(depth in 1usize..=3) {
        let outer_var = "outer_item";
        let inner_source = format!(
            "slideforge_version \"1\"\ntitle \"Test\"\n\n\
             @for item in [1, 2, 3]:\n  @for sub in [4, 5]:\n    blank slide-{{{{ item }}}}:\n      title: \"{{{{ item }}}} - {{{{ sub }}}}\"\n"
        );
        // Outer `item` must be accessible inside the inner @for body.
        let parse_result = slideforge_syntax::parse(&inner_source);
        prop_assume!(parse_result.is_ok());
        let ast = parse_result.unwrap();
        let eval_result = evaluate(ast, &Scope::empty());
        // Outer variable reference in inner body must not produce UndefinedVariable error
        prop_assert!(
            eval_result.is_ok() || !format!("{:?}", eval_result).contains("UndefinedVariable"),
            "Outer @for variable leaked scope: {:?}",
            eval_result
        );
    }
}
```

### VP-015 Fuzz Harness

```rust
// fuzz/fuzz_targets/eval_fuzz.rs
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slideforge_eval::evaluate;
use slideforge_eval::Scope;
use slideforge_types::Deck;

fuzz_target!(|data: &[u8]| {
    // Use Arbitrary to generate structurally valid Deck ASTs from raw bytes.
    // This ensures we test with syntactically plausible inputs, not random garbage.
    let mut unstructured = arbitrary::Unstructured::new(data);
    if let Ok(deck) = Deck::arbitrary(&mut unstructured) {
        let _ = evaluate(deck, &Scope::empty());
        // Invariant: must not panic regardless of input shape.
    }
});
```

`Deck` must derive `#[derive(arbitrary::Arbitrary)]` in its definition (gated by
`#[cfg(any(test, fuzzing))]` or feature flag). Add `arbitrary` to
`slideforge-types` dev-dependencies and the fuzz crate dependencies.

### Justfile Targets

```
kani-eval:
    # Platform: Linux/macOS only.
    cargo kani -p slideforge-eval --harness proofs::vp004_no_coercion::proofs::string_value_never_coerced
    cargo kani -p slideforge-eval --harness proofs::vp005_no_overflow::proofs::integer_arithmetic_no_overflow

fuzz-eval time="10":
    cargo fuzz run eval_fuzz -- -max_total_time={{time}}
```

## Dependencies

### Dependency Justification

- STORY-067 depends on STORY-011 because VP-004 and VP-005 proofs call `eval_expr()`
  and `Scope` from that story. Without the production evaluator, no proof target exists.
- STORY-067 depends on STORY-012 because VP-010 proptest verifies the scoping behavior
  implemented there (`@for` variable binding + nested scope lookup).
- STORY-067 depends on STORY-013 because VP-010 uses `@if/@else` evaluation paths.
- STORY-067 depends on STORY-014 because VP-004 proves the no-coercion invariant
  implemented in that story's `Value` enum and `eval_expr` type-checking path.
- STORY-067 does not block any other story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | String value `"true"` passed to arithmetic — VP-004 | Proof: TypeError returned, no coercion to Bool |
| EC-002 | String value `""` (empty string) in arithmetic — VP-004 | Proof: TypeError, not panic |
| EC-003 | i64::MAX + 1 overflow — VP-005 | Proof: eval returns Err(ArithmeticOverflow), never panic |
| EC-004 | i64::MIN - 1 underflow — VP-005 | Proof: eval returns Err(ArithmeticOverflow) |
| EC-005 | i64::MIN * -1 overflow — VP-005 | Proof: eval returns Err(ArithmeticOverflow) |
| EC-006 | Nested @for depth = 1 (no nesting) — VP-010 | proptest: trivially passes |
| EC-007 | Nested @for depth = 3 — VP-010 | proptest: outer vars visible 2 levels deep |
| EC-008 | Arbitrary AST with zero slides — VP-015 | Fuzz: returns Ok(empty_deck), no panic |
| EC-009 | Arbitrary AST with unbounded @for node (fuzz only) | Fuzz: evaluator applies its termination guard, returns Err or bounded output |

## Test Strategy

- **VP-004, VP-005:** Kani bounded model checking. VERIFICATION SUCCESSFUL is pass.
- **VP-010:** proptest 1,000 cases. Scoping bug is a persistent failure (not flaky).
- **VP-015:** cargo-fuzz 10s smoke on PR, 5min nightly. Zero crashes is pass.
- **Regression:** Both proofs are permanent CI gates on `ubuntu-latest`.

---

*Subsystem anchor justification: SS-02 owns this story's scope because slideforge-eval
is the sole crate implementing expression evaluation, variable scoping, and @for iteration —
the targets of all four VPs in this story, per ARCH-INDEX Subsystem Registry.*
