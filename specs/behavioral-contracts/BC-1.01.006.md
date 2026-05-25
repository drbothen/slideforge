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

# BC-1.01.006: Reject Variable Names Colliding with Slide Type Keywords

## Description

A user may not declare a variable whose name is identical to a slide type keyword
(e.g., `chart`, `title`, `content`, `agenda`). Such a collision would make the variable
inaccessible via `{{ var }}` interpolation and could confuse the parser when the name
appears in structural positions. The parser detects this at declaration time and produces
E-PAR-008. This covers DEC-010 from the domain edge-case catalog.

## Preconditions

1. A `vars:` block declares a variable with a name that matches a slide type keyword or
   a reserved structural keyword.
2. The collision is detected during parse-time scope analysis.

## Postconditions

1. E-PAR-008 is emitted: `Variable name '<name>' collides with reserved keyword at <file>:<line>:<col>. Choose a different name.`
2. Build exits with code 1.
3. No AST is produced.

## Invariants

1. The forbidden variable name set is the union of: all slide type keywords (31 types), all directive keywords (`@for`, `@if`, `@data`, etc.), and all structural keywords (`vars`, `set`, `alias`, `variants`, `section`).
2. Reserved keywords from DI-021 are also in this set.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-010) | `vars: { chart: "my-chart-data" }` | E-PAR-008: 'chart' collides with slide type keyword; exit 1 |
| EC-002 | `vars: { title: "My Deck" }` | E-PAR-008: 'title' is a reserved keyword; exit 1 |
| EC-003 | `vars: { chart_data: "my-chart-data" }` (suffix, not exact match) | No error — only exact matches are rejected |
| EC-004 | `vars: { my_chart: "value" }` (prefix, not exact match) | No error |
| EC-005 | Reserved keyword as a map key in data source (not vars: declaration) | No error — only vars: declarations in .sf files are checked |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { chart: "data.json" }` | E-PAR-008: 'chart' collides with keyword; exit 1 | error |
| `vars: { agenda: "Q3 items" }` | E-PAR-008: 'agenda' collides with keyword; exit 1 | error |
| `vars: { my_data: "data.json" }` | No error; variable accessible as `{{ my_data }}` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Every exact keyword match in vars: declaration triggers E-PAR-008 | unit test per slide type keyword |
| VP-TBD | Non-exact matches (prefix, suffix, case-different) do not trigger E-PAR-008 | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("DSL Source Parsing") per capabilities.md §CAP-001 — variable-name/keyword collision is a parse-time correctness rule in the indentation-significant grammar |
| L2 Domain Invariants | DI-021 (reserved keywords must be rejected with descriptive errors) |
| L2 Edge Cases | DEC-010 (slide type keyword used as variable name) |
| Architecture Module | slideforge-syntax crate — scope/keyword analysis (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.005 — composes with (reserved keyword rejection — this BC extends the same mechanism to vars: declarations)
- BC-1.15.001 — depends on (file:line:col span requirement)

## Architecture Anchors

- `architecture/system-overview.md` — keyword collision check during parse

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
