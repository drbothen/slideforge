---
document_type: verification-property
vp_id: VP-004
title: 'No implicit coercion: "1.10" stays string'
module: slideforge-eval
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-1.02.003, DI-004]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-004: No Implicit Type Coercion

## Property Statement

For any string value `s` produced by the lexer as a `Token::Float(s)` or
`Token::String(s)`, the evaluator MUST NOT coerce `s` to a `Value::Float` or
`Value::Bool` without an explicit pipe filter call.

Formally: if the AST contains `Value::Str("1.10")` at a field position, the
evaluated result at that position is `Value::Str("1.10")`, never `Value::Float(1.1)`.

## Motivation

DI-004: "Values in the DSL are never silently coerced." This is a competitive
differentiator (R3 research): implicit YAML-style coercion was identified as a major
cause of corrupted data-driven slides in competitor tools. The lexer already preserves
`"1.10"` as a string token (S4 spike confirmed). The evaluator must not re-coerce it.

## Feasibility Assessment

Feasible. The evaluator's type system has explicit `Value` variants: `Value::Str`,
`Value::Int`, `Value::Float`, `Value::Bool`. Coercion only happens via explicit
pipe filters (`| to_int`, `| to_float`). Kani can prove that no code path from
`Value::Str` to `Value::Float` exists in the evaluator without an explicit filter.

## Proof Harness Skeleton

```rust
// crates/slideforge-eval/src/proofs/no_coercion.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn string_stays_string() {
        let input = Value::Str("1.10".into());
        // Evaluate in an empty scope, no pipe filters
        let ctx = EvalContext::empty();
        let result = eval_value(&input, &ctx);
        // Must remain a string
        kani::assert!(matches!(result, Ok(Value::Str(_))));
        // Must NOT become a float
        kani::assert!(!matches!(result, Ok(Value::Float(_))));
    }

    #[kani::proof]
    fn no_string_stays_no_bool() {
        let input = Value::Str("NO".into());
        let ctx = EvalContext::empty();
        let result = eval_value(&input, &ctx);
        kani::assert!(matches!(result, Ok(Value::Str(_))));
        kani::assert!(!matches!(result, Ok(Value::Bool(_))));
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit test: `test_no_implicit_type_coercion` in S4 spike lexer tests (passing).
Production test: `test_eval_string_no_coerce` in `slideforge-eval/src/tests/`.
