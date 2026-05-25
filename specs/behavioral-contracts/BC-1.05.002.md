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
capability: CAP-005
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

# BC-1.05.002: Conditional Expression Evaluates with Same Type Rules as {{ expr }}

## Description

The expression in an `@if condition:` is evaluated using the same type system and
expression evaluator as `{{ expr }}` interpolations. No implicit coercions occur in
conditions — comparing a string to a boolean is a type error, and a non-boolean
value used as a condition without explicit conversion is a type error.

## Preconditions

1. An `@if condition:` block exists in the source.
2. The condition is an expression involving variables, comparisons, logical operators, or filter pipes.

## Postconditions

1. Condition expressions support: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, parenthesization.
2. The condition result must be boolean. A non-boolean result (e.g., a string or number) produces E-EVL-003.
3. String `"NO"` used directly as a condition (without `== false`) is E-EVL-003.
4. Integer `0` used directly as a condition is E-EVL-003.

## Invariants

1. The condition evaluator reuses the same type rules as the expression evaluator — there is no separate "condition mode."
2. No implicit truthiness: only genuine boolean expressions are accepted without explicit conversion.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@if env == "prod":` (string equality) | Boolean result; valid |
| EC-002 | `@if count > 0:` (numeric comparison) | Boolean result; valid |
| EC-003 | `@if flag:` where `flag` is a string "true" | E-EVL-003: string used as condition without boolean conversion. Use `@if flag == "true":` |
| EC-004 | `@if flag:` where `flag` is a boolean `true` | Valid; no conversion needed |
| EC-005 | `@if items | length > 0:` (pipe in condition) | Valid; pipe filters work in conditions |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { flag: true }` / `@if flag:` | Condition is true; block rendered | happy-path |
| `vars: { flag: "true" }` / `@if flag:` | E-EVL-003: string in boolean position; exit 2 | error |
| `vars: { n: 5 }` / `@if n > 3:` | Condition true; block rendered | happy-path |
| `@if items | length > 0:` with non-empty list | Condition true; block rendered | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Any non-boolean condition value produces E-EVL-003 | unit test with string, int, and null conditions |
| VP-TBD | All comparison operators work correctly in conditions | unit test per operator |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Conditional Rendering") per capabilities.md §CAP-005 |
| Capability Anchor Justification | CAP-005 ("Conditional Rendering") per capabilities.md §CAP-005 — type rules for conditions are a specified part of the conditional rendering capability |
| L2 Domain Invariants | DI-004 (no implicit type coercion) |
| Architecture Module | slideforge-eval crate — expression evaluator / condition evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.05.001 — depends on (this BC provides the type rules for BC-1.05.001's conditions)
- BC-1.02.003 — composes with (no-coercion invariant applies equally to conditions)

## Architecture Anchors

- `architecture/system-overview.md` — type model for conditions

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
