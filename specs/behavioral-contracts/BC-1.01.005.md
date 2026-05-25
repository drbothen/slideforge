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
capability: CAP-001
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

# BC-1.01.005: Reject Reserved Keywords with Named-Feature Error Message

## Description

slideforge reserves approximately 30 keywords for planned v2+ features (e.g., `component`,
`extends`, `raw`, `@fn`, `@mixin`). Any use of a reserved keyword in a position where it
would be interpreted as a DSL construct produces E-PAR-006, naming the reserved keyword,
the planned feature it is reserved for, and a suggestion if applicable.

## Preconditions

1. A .sf source file is being parsed.
2. A token matches one of the ~30 reserved keywords in a syntactically meaningful position.

## Postconditions

1. E-PAR-006 is emitted: `Reserved keyword '<word>' at <file>:<line>:<col>. '<word>' is reserved for <feature> (planned v2+). Did you mean '<suggestion>'?`
2. The suggestion field is populated when a non-reserved alternative exists; omitted otherwise.
3. Build exits with code 1.
4. No AST is produced.

## Invariants

1. Reserved keywords are rejected at parse time — they are never interpreted as identifiers.
2. The error message always names the planned feature (DI-021).
3. The reserved keyword list is fixed; it is not configurable or overridable.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@fn` used as a directive | E-PAR-006: `'@fn' is reserved for user-defined functions (planned v2+)` |
| EC-002 | `component` used as a slide block keyword | E-PAR-006: `'component' is reserved for reusable components (planned v2+)` |
| EC-003 | `@mixin` used as a directive | E-PAR-006: `'@mixin' is reserved for style mixins (planned v2+)` |
| EC-004 | `extends` used in an alias or type definition | E-PAR-006: `'extends' is reserved for type extension (planned v2+). Use alias instead.` |
| EC-005 | Reserved keyword used inside a quoted string value | No error — reserved keywords in string literals are not parsed as keywords |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@fn compute(x):` | E-PAR-006: '@fn' reserved for user-defined functions; exit 1 | error |
| `@while true:` | E-PAR-006 (see BC-1.04.003); exit 1 | error |
| `title "extends the prior slide"` (in string) | No error | happy-path |
| `component header:` as slide block | E-PAR-006: 'component' reserved; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every reserved keyword in a DSL position triggers E-PAR-006 | unit test per reserved keyword in the list |
| VP-TBD | Reserved keywords inside string literals are not flagged | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — reserved keyword rejection is a first-class requirement of the indentation-significant grammar |
| L2 Domain Invariants | DI-021 (reserved keywords must be rejected with descriptive errors) |
| Architecture Module | slideforge-syntax crate — lexer/keyword table (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.006 — related to (variable name collision with keywords is the sibling rule)
- BC-1.04.003 — composes with (reserved loop keywords covered there)
- BC-1.15.001 — depends on (file:line:col span requirement)

## Architecture Anchors

- `architecture/system-overview.md` — reserved keyword table

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
