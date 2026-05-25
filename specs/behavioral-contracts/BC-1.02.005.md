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

# BC-1.02.005: Lexical Scoping in Nested @for Loops — Outer Vars Remain in Scope

## Description

slideforge uses lexical scoping for variable resolution. In nested `@for` loops, variables
declared in an outer scope (deck vars, outer loop variables) remain accessible in inner
loops. An inner loop variable does not shadow outer loop variables unless they share the
same name. This covers DEC-017 from the domain edge-case catalog.

## Preconditions

1. A source file contains nested `@for` blocks.
2. The inner `@for` references variables from the outer `@for` scope.

## Postconditions

1. Outer `@for` loop variable is accessible in all nested scopes.
2. Deck-level `vars:` variables are accessible at all nesting levels.
3. Inner loop variable is accessible only within its own block.
4. If an inner variable name matches an outer variable name, the inner variable shadows the outer for that block only.
5. No undefined-variable errors are raised for outer-scope variables referenced in inner blocks.

## Invariants

1. Scope chain is: deck vars → outer @for var → inner @for var (inner takes precedence when names collide).
2. A variable that goes out of scope at loop end is not accessible outside the loop.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-017) | `@for project in projects:` / `@for item in project.items:` / `title "{{ project.name }}: {{ item.name }}"` | Both `project` and `item` are in scope; output uses both | 
| EC-002 | Inner loop var same name as outer: `@for item in outer:` / `@for item in inner:` | Inner `item` shadows outer `item` within inner block; outer resumes after inner block |
| EC-003 | Three levels of nesting; innermost accesses vars from all outer levels | All outer variables accessible; scope chain preserved |
| EC-004 | Deck var `name: "global"` with outer loop also `name: "outer"` | Outer loop var shadows deck var within loop; deck var resumes after loop |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@for project in projects:` / `@for item in project.items:` / `title "{{ project.name }}: {{ item.name }}"` | Slide per item with correct project name | happy-path (DEC-017) |
| `vars: { label: "prefix" }` / `@for x in list:` / `title "{{ label }}-{{ x }}"` | Deck var accessible inside loop | happy-path |
| Inner `@for item` shadows outer `@for item` | Inner uses inner value; outer resumes correctly after inner block | boundary |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Outer-scope variable access inside nested @for never produces E-EVL-001 | unit test with 2-level nesting fixture |
| VP-TBD | Inner variable does not persist outside its @for block | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 |
| Capability Anchor Justification | CAP-002 ("Variable Interpolation and Expression Evaluation") per capabilities.md §CAP-002 — lexical scoping is part of the expression evaluation semantics |
| L2 Edge Cases | DEC-017 (nested @for with outer scope variable) |
| Architecture Module | slideforge-eval crate — scope chain (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.02.002 — composes with (undefined variable error is what this BC prevents for outer-scope vars)
- BC-1.04.001 — depends on (iteration semantics that produce the scope)

## Architecture Anchors

- `architecture/system-overview.md#evaluation-scope` — scope chain resolution

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
