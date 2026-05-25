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
capability: CAP-009
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

# BC-1.09.001: alias <name> = <type>: Defines Named Preset That Resolves at Parse Time

## Description

The `alias` keyword defines a named shorthand for an existing slide type with optional
pre-filled field defaults. Aliases are resolved at parse time: an `alias exec_title = title:`
block with preset fields is expanded in the AST before evaluation. The alias name
behaves exactly like the base type name for all downstream purposes except that it carries
the preset field values.

## Preconditions

1. An `alias <name> = <type>:` declaration appears at deck scope (outside slide blocks).
2. The base `<type>` is a known slide type (one of the 31 built-ins).
3. The alias `<name>` does not collide with a reserved keyword or existing slide type name.
4. Any field values declared inside the alias block are valid fields of the base type.

## Postconditions

1. The alias name is registered in the parser's type registry alongside the 31 built-in types.
2. A `slide <alias_name>:` block is parsed and treated identically to `slide <base_type>:` with the preset fields pre-applied.
3. If the slide using the alias omits a preset field, the alias default is used (same precedence as a set rule).
4. If the slide using the alias declares a preset field explicitly, the slide's value wins.
5. Aliases resolve at parse time — the alias declaration is not present in the final AST; it is fully expanded.

## Invariants

1. Alias names are resolved at parse time, not evaluation time.
2. An alias is syntactic sugar only — it cannot add new fields, only preset existing ones (enforced by BC-1.09.002).
3. Aliases inherit all validation rules of the base type.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Alias name same as an existing slide type | Compile error: alias name collides with built-in type |
| EC-002 | Alias name same as a reserved keyword | E-PAR-006: reserved keyword collision |
| EC-003 | Two aliases with the same name | Compile error: duplicate alias name |
| EC-004 | `slide <alias>:` in an @for loop | Valid — alias expands identically per iteration |
| EC-005 | Alias of an alias (transitive aliasing) | Compile error or supported per Q2 decision — flag for architect |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `alias exec_title = title:` + `footer "Internal"` → `slide exec_title:` with no footer | Slide has footer="Internal" in output; exit 0 | happy-path |
| `alias exec_title = title:` → `slide exec_title:` with explicit `footer "Override"` | Slide uses "Override"; alias default not applied | happy-path |
| `alias title = title:` (alias name = built-in type) | Compile error: alias name collides with built-in | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Alias expansion is idempotent: `slide alias:` with all fields explicit produces identical AST to `slide base_type:` with same fields | unit test |
| VP-TBD | Alias default is overridden by slide-level explicit value | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Parametric Slide-Type Aliases") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Parametric Slide-Type Aliases") per capabilities.md §CAP-009 — "aliases resolve at parse time" and "only preset existing fields" are the core contract specified in CAP-009 |
| L2 Domain Invariants | DI-021 (reserved keywords must be rejected with descriptive errors) |
| Architecture Module | slideforge-syntax crate — AliasRegistry (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.09.002 — composes with (alias cannot add new fields; only preset existing)
- BC-1.08.001 — related to (set rules and aliases both establish defaults; precedence relationship)

## Architecture Anchors

- `architecture/system-overview.md#aliases` — alias parse-time expansion design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
