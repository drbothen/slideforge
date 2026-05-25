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

# BC-1.05.001: @if/@elif/@else at Slide, Element, Field, and Section Scope

## Description

Conditional rendering is available at all four scopes: slide-level (include/exclude
an entire slide), element-level (include/exclude a visual element within a slide),
field-level (choose between two field values), and section-level (include/exclude a
document section). The `@if / @elif / @else:` syntax is consistent across all scopes.

## Preconditions

1. An `@if condition:` block appears in the deck source at one of the four supported scopes.
2. The condition is a valid boolean expression evaluatable with current scope variables.

## Postconditions

1. If the condition evaluates to truthy, the enclosed slides/elements/fields are included in the output.
2. If falsy and `@elif` branches exist, each is tested in order; the first truthy branch is included.
3. If all conditions are falsy and an `@else:` branch exists, the else branch is included.
4. If all conditions are falsy and no `@else:` exists, the block produces no output.
5. Build exits with code 0 on success.

## Invariants

1. Exactly zero or one branch is rendered for any given `@if/@elif/@else` chain.
2. Conditions are evaluated lazily — only conditions up to the first truthy branch are evaluated.
3. The four scopes (slide, element, field, section) use the same `@if` syntax.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@if` at slide scope wrapping entire slide block | Slide included or excluded; no half-rendered slide |
| EC-002 | `@if` at field scope: `title @if is_draft: "DRAFT: {{ title }}" @else: "{{ title }}"` | Field value selected based on condition |
| EC-003 | `@elif` condition references loop variable inside `@for` | Variable is in scope; condition evaluated per iteration |
| EC-004 | All branches false, no `@else:` | Zero output for this block; no error |
| EC-005 | Condition uses undefined variable | E-EVL-001 emitted; build fails in strict mode |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { env: "prod" }` / `@if env == "prod":` / slide block | Slide included when env=prod | happy-path |
| `vars: { env: "dev" }` / `@if env == "prod":` / slide / `@else:` / other slide | Other slide included | happy-path |
| `@if undefined_var:` | E-EVL-001: undefined variable; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Exactly one branch is rendered for any well-formed @if/@elif/@else chain | proptest (random truth tables) |
| VP-TBD | @if at all four scopes produces structurally correct output AST | unit test per scope |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 ("Conditional Rendering") per capabilities.md §CAP-005 |
| Capability Anchor Justification | CAP-005 ("Conditional Rendering") per capabilities.md §CAP-005 — this BC covers the core conditional rendering contract at all four scopes |
| Architecture Module | slideforge-eval crate — conditional evaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.05.002 — composes with (type rules for condition expressions)
- BC-1.02.001 — depends on (expression evaluation for conditions)
- BC-1.02.002 — composes with (undefined variable errors in conditions)

## Architecture Anchors

- `architecture/system-overview.md#computation-model` — conditional rendering design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
