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

# BC-1.02.001: Evaluate {{ expr }} Interpolation with Arithmetic and Pipe Filters

## Description

The evaluator processes `{{ expr }}` interpolations in string fields, resolving variable
references, arithmetic expressions, comparisons, field access (dot notation), and pipe
filter chains. The ~15 built-in filters (e.g., `| upper`, `| currency`, `| round`,
`| int`, `| float`, `| string`, `| join`) are pure functions. The result is always
converted to string for embedding in the output.

## Preconditions

1. A `{{ expr }}` appears in a field value in a valid, parsed AST.
2. All variables referenced in the expression are declared in scope (deck vars, @for scope, or variant vars).
3. Data sources referenced have been loaded.

## Postconditions

1. The expression is evaluated and the result is coerced to a string for embedding in the field.
2. Arithmetic operators `+`, `-`, `*`, `/`, `%` work on numeric values.
3. Dot-notation field access (`{{ item.name }}`) resolves nested map/object fields.
4. Pipe chains (`{{ value | filter1 | filter2 }}`) apply filters left-to-right.
5. Unknown filter names produce E-EVL-004.
6. The build exits with code 0 on success.

## Invariants

1. Filter functions are pure — same input always produces same output.
2. No implicit type coercion (see BC-1.02.003) — type mismatches produce E-EVL-003.
3. Division by zero produces E-EVL-003 with a descriptive message.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `{{ price * 1.2 | round(2) }}` — chained arithmetic then filter | Evaluated left-to-right: arithmetic first, then filter; result is string |
| EC-002 | `{{ items | join(", ") }}` — filter with argument | Argument is passed to filter; output is comma-joined string |
| EC-003 | `{{ 10 / 0 }}` — division by zero | E-EVL-003: division by zero at <span> |
| EC-004 | `{{ name | unknown_filter }}` — non-existent filter | E-EVL-004: filter 'unknown_filter' not found; available: [...] |
| EC-005 | Deeply nested field access `{{ a.b.c.d }}` | Resolved step by step; E-DAT-005 if any level is missing |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { n: 5 }` / `stat "{{ n * 2 }}"` | stat="10" | happy-path |
| `vars: { name: "world" }` / `title "Hello {{ name | upper }}"` | title="Hello WORLD" | happy-path |
| `{{ price | currency }}` where price=1234.5 | "1,234.50" (or locale-appropriate format) | happy-path |
| `{{ 10 / 0 }}` | E-EVL-003: division by zero; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Arithmetic evaluation is deterministic | proptest (commutative/associative properties for integer ops) |
| VP-TBD | All 15 built-in filters produce correct output for representative inputs | unit test per filter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 |
| Capability Anchor Justification | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 — this BC covers the core happy-path contract for expression evaluation |
| L2 Domain Invariants | DI-004 (no implicit type coercion) |
| Architecture Module | slideforge-eval crate — expression evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.002 — composes with (error path: undefined variable in expression)
- BC-1.02.003 — composes with (type system invariant for expressions)
- BC-1.02.004 — related to (the ${{ }} ambiguity handled separately)

## Architecture Anchors

- `architecture/system-overview.md` — expression evaluation design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
