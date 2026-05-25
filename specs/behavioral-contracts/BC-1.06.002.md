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
capability: CAP-006
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

# BC-1.06.002: Detect and Reject Circular @include Chains (Fail-Closed)

## Description

The include resolver tracks the in-progress include stack during parsing. If an
`@include` directive would cause the resolver to visit a file already on the include
stack, the cycle is detected immediately and E-PAR-004 is emitted showing the full
cycle path. No infinite loop or stack overflow occurs. This covers DEC-003 from the
domain edge-case catalog.

## Preconditions

1. An `@include` chain contains a cycle (A includes B includes A, or longer chains).

## Postconditions

1. E-PAR-004 is emitted showing the full cycle path: `Include cycle detected: <path1> → <path2> → ... → <path1>`.
2. Build exits with code 1.
3. No AST is produced; no output files are written.

## Invariants

1. Cycle detection is performed before any cycled file is read (fail-closed: no partial parsing).
2. The cycle detection covers `@include` only; `@import` of packages has separate lockfile validation.
3. A file may be included from multiple distinct paths (diamond include) — only true cycles are errors.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-003) | `deck.sf` includes `a.sf` includes `b.sf` includes `deck.sf` | E-PAR-004: `deck.sf → a.sf → b.sf → deck.sf`; exit 1 |
| EC-002 | Self-include: file includes itself | E-PAR-004: `file.sf → file.sf`; exit 1 |
| EC-003 | Diamond include: A includes B and C; B includes D; C includes D (no cycle) | No error — D is included twice but no cycle exists |
| EC-004 | Very deep include chain (50 levels, no cycle) | No error; all files resolved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| a.sf: `@include "b.sf"` / b.sf: `@include "a.sf"` | E-PAR-004: `a.sf → b.sf → a.sf`; exit 1 | error (DEC-003) |
| Self-include: deck.sf: `@include "deck.sf"` | E-PAR-004: `deck.sf → deck.sf`; exit 1 | error |
| Diamond: A→B, A→C, B→D, C→D (D included twice, no cycle) | Parsed successfully; D inlined twice | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Cycle detection terminates for any finite include graph | kani (bounded: ≤ 200 files) |
| VP-TBD | Diamond includes do not produce false-positive cycle errors | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 |
| Capability Anchor Justification | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 — cycle detection is an explicitly stated requirement of the @include capability |
| L2 Domain Invariants | DI-007 (include cycles are compile errors) |
| L2 Edge Cases | DEC-003 (circular include chain) |
| Architecture Module | slideforge-syntax crate — include resolver with cycle detection (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.004 — related to (same cycle detection applied to @include in parse stage; see also BC-1.07.003 for variant cycles)
- BC-1.06.001 — depends on (this BC is the safety constraint for BC-1.06.001)

## Architecture Anchors

- `architecture/system-overview.md` — include graph cycle detection (DFS-based)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
