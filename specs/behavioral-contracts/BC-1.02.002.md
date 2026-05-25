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

# BC-1.02.002: Reject Undefined Variable Reference with Scope Path

## Description

Any `{{ name }}` interpolation or `@{name}` math interpolation that references a
variable not declared in any active scope level is a compile error. The error
message shows the variable name, source location, and the list of variables that
ARE in scope — enabling the user to identify typos or missing declarations. There
is no fallback to empty string or "undefined" sentinel.

## Preconditions

1. An expression `{{ name }}` or `@{name}` appears in a field value.
2. `name` is not declared in the deck-level `vars:` block, the current `@for` loop scope, a variant `vars:` block, or any enclosing scope level.
3. The evaluation stage is processing the expression (parse stage has succeeded).

## Postconditions

1. E-EVL-001 is emitted: `Undefined variable '{{ <name> }}' at <file>:<line>:<col>. Variables in scope: [<list>]`.
2. The scope list in the error shows all variable names visible at that point in the evaluation tree.
3. Build exits with code 2 (validation error) in strict mode.
4. No output is produced in strict mode.
5. In `--warn-only` mode: an error-slide placeholder replaces the affected slide, and the build continues.

## Invariants

1. The error is emitted for EVERY undefined variable reference in the source — not just the first one (DI-018).
2. The scope list is accurate: it reflects the actual scope chain at the point of the reference.
3. No silent substitution (empty string, "undefined" text) ever occurs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Same variable undefined in two different slides | Two separate E-EVL-001 errors, one per reference |
| EC-002 | Variable defined in outer @for scope, referenced inside inner @for | No error — outer scope variables are accessible (DEC-017) |
| EC-003 | Variable name is a math expression variable (@{var}) in $...$  | E-EVL-002 (math context variant of the error) |
| EC-004 | Variable defined in a different variant's vars: block (not the active variant) | E-EVL-001 — inactive variant vars are not in scope |
| EC-005 | Variable defined AFTER it is referenced in declaration order | Whether forward references are supported depends on evaluation model — in slideforge v1.0, all vars: blocks are evaluated before slide evaluation. So this should be: NO ERROR — vars: blocks are hoisted. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { name: "World" }` / `title "Hello {{ name }}"` | title="Hello World", no error | happy-path |
| `title "Hello {{ greeting }}"` with no vars block | E-EVL-001: `Undefined variable '{{ greeting }}' ... Variables in scope: []` | error |
| `vars: { a: "x" }` / `title "{{ b }}"` | E-EVL-001: `... Variables in scope: [a]` | error |
| `@for item in items:` / `title "{{ item.name }}"` (item defined) | No error — item is in @for scope | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every undefined variable reference produces exactly one error entry | proptest (count errors vs. occurrences) |
| VP-TBD | Scope list in error is a subset of actual declared variables | unit test with known scope chain |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 |
| Capability Anchor Justification | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 — this BC defines the failure contract for missing variable names, which is a direct sub-requirement of the evaluation capability |
| L2 Domain Invariants | DI-006 (undefined variables are compile errors), DI-018 (accumulate all errors) |
| Architecture Module | slideforge-eval crate — evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.001 — depends on (this BC is the error path; BC-1.02.001 is the happy path)
- BC-1.03.003 — related to (same pattern but for missing data fields)
- BC-1.15.001 — depends on (all errors use file:line:col per BC-1.15.001)

## Architecture Anchors

- `architecture/system-overview.md#evaluation-scope` — scope chain resolution

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
