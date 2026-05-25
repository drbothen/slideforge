---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-002
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.02.003: No Implicit Type Coercion — Values Are Never Silently Converted

## Description

slideforge does not perform implicit type coercion on DSL values. String "NO" remains
the string "NO" — it is never coerced to boolean false. String "1.10" remains "1.10"
— it is never coerced to the float 1.1. Numeric operations on string values produce a
type error at compile time. This invariant prevents the silent data corruption identified
in competitor tools and in the R3 research on DSL pain points.

## Preconditions

1. A value is declared in a `vars:` block or data source.
2. The value is used in an expression or interpolation.

## Postconditions

1. String values declared as strings remain strings throughout evaluation.
2. `"NO"` interpolated via `{{ var }}` produces the literal text "NO" — not "false" or any boolean representation.
3. `"1.10"` interpolated via `{{ var }}` produces the literal text "1.10" — not "1.1".
4. Arithmetic on a string value (e.g., `"NO" + 1`) produces E-EVL-003 (type mismatch).
5. Comparison `"NO" == false` produces E-EVL-003 (type mismatch — string vs boolean).

## Invariants

1. No value ever changes type without an explicit format function call (e.g., `| int`, `| float`, `| bool`).
2. YAML-style implicit coercion rules are never applied.
3. Type errors from coercion-free operations are reported as E-EVL-003 with the actual and expected types.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `vars: { flag: "NO" }` used in `@if flag ==` condition | Type error: comparison of string "NO" vs boolean without explicit conversion |
| EC-002 | `vars: { rate: "1.10" }` used in `{{ rate * 100 }}` | E-EVL-003: operator `*` expects numeric, got string. Hint: use `{{ rate | float * 100 }}` |
| EC-003 | `vars: { count: 42 }` (integer) used in string concat | Explicit conversion required: `{{ count | string }}` or `{{ "Items: " ~ count }}` |
| EC-004 | Data source field is numeric in JSON but used as string | Field is typed as number from JSON source. String operations require `| string` filter |
| EC-005 | `vars: { yes_value: "yes" }` — does "yes" coerce to true? | No. "yes" stays "yes". No coercion to boolean. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { flag: "NO" }` / `title "{{ flag }}"` | title="NO" (string, not false) | happy-path |
| `vars: { v: "1.10" }` / `title "{{ v }}"` | title="1.10" (not "1.1") | happy-path (DI-004) |
| `vars: { v: "NO" }` / `@if v ==` (comparing string to boolean) | E-EVL-003: type mismatch | error |
| `vars: { rate: "2.5" }` / `stat "{{ rate * 100 }}"` | E-EVL-003: `*` expects numeric, got string | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Roundtrip: a value declared as string-type evaluates to the identical string | proptest (arbitrary string inputs) |
| VP-TBD | No coercion path exists: any implicit coercion attempt → type error | kani (for a bounded set of types) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 |
| Capability Anchor Justification | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 — no-coercion is a stated invariant of the evaluation system |
| L2 Domain Invariants | DI-004 (no implicit type coercion) |
| Architecture Module | slideforge-eval crate — type system (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.001 — composes with (same evaluation system)
- BC-1.02.002 — related to (type errors and undefined var errors use same error mechanism)

## Architecture Anchors

- `architecture/system-overview.md` — type model description

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
