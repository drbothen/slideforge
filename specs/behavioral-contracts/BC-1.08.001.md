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
capability: CAP-008
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

# BC-1.08.001: set <type>: <field> <value> Applies as Default to All Instances of Type

## Description

The `set` keyword declares a deck-scoped default field value for a named slide type or
section type. Every slide of that type in the deck inherits the set value unless it
explicitly overrides it at the slide level. Set rules follow the 11-level precedence
chain: slide-level values win over set rules.

## Preconditions

1. A `set <type>: <field> <value>` declaration appears at deck scope (outside any slide block).
2. The slide type name is a valid known type (one of the 31 built-in types or a user alias).
3. The field name is a valid field for that slide type.
4. The value is a valid literal, `{{ expr }}`, or `brand.*` reference.

## Postconditions

1. Every `slide <type>:` block in the deck that does NOT declare `<field>` explicitly inherits the set default.
2. Every `slide <type>:` block that DOES declare `<field>` explicitly uses its own value (set rule is overridden).
3. The set rule is evaluated after brand loading and before slide rendering.
4. Multiple `set` declarations for the same type+field are merged in declaration order; last declaration wins.

## Invariants

1. Set rules never override explicit slide-level field values.
2. Set rules apply only within the deck scope (or workspace scope via `.sfconfig`) where they are declared.
3. Set rules apply to aliases of the named type (alias expansion happens before set rule application).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | set rule for unknown slide type | E-PAR-007 compile error: unknown slide type with suggestion |
| EC-002 | set rule for invalid field on a valid type | Compile error naming the invalid field and valid fields for that type |
| EC-003 | Two conflicting set rules for the same type+field | Last declaration wins; no error (documented merge semantics) |
| EC-004 | set rule in an @include file | Included set rules apply at deck scope exactly as if declared inline |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `set content: footer "Confidential"` + 3 content slides with no footer | All 3 slides have footer="Confidential" in output; exit 0 | happy-path |
| `set content: footer "Default"` + slide with explicit `footer "Override"` | Slide uses "Override"; others use "Default" | happy-path |
| `set unknown_type: footer "x"` | E-PAR-007 compile error; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Set rule applies to N slides of the type; slide with explicit value is not affected | unit test: build deck with set rule + override slide |
| VP-TBD | Last-wins for duplicate set rules on same type+field | unit test with 2 conflicting set rules |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 |
| Capability Anchor Justification | CAP-008 ("Set Rules for Slide-Type Defaults") per capabilities.md §CAP-008 — this BC defines the exact semantics of set rule application and precedence that CAP-008 specifies |
| L2 Domain Invariants | DI-020 (merge semantics are fixed; last-wins for scalars) |
| Architecture Module | slideforge-eval crate — SetRuleEvaluator (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.08.002 — composes with (set rules support interpolation and brand refs)
- BC-1.08.003 — composes with (brand.* references evaluated after brand load)
- BC-1.07.002 — related to (variant vars also follow the 11-level precedence chain)

## Architecture Anchors

- `architecture/authoring-subsystem.md#set-rules` — set rule precedence and evaluation order

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
