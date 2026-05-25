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

# BC-1.01.004: Resolve @include Directives and Detect Cycles (Fail-Closed for Writes)

## Description

The parser resolves `@include "path.sf"` directives by inlining the referenced file
at the point of include. If the include chain forms a cycle (A → B → C → A), the
build halts with a compile error showing the full cycle path before any output is
produced. This is a fail-closed behavior: no partial output is written on cycle detection.

## Preconditions

1. Source file contains one or more `@include "path.sf"` directives.
2. Each included path is either an absolute path or relative to the including file's directory.
3. The parser has already resolved any variable interpolation in include paths (e.g., `@include "{{ client }}/template.sf"`).

## Postconditions

1. **Happy path:** All included files are resolved and inlined at their include points. The final AST represents the complete merged source.
2. **Cycle detected:** Build halts with E-PAR-004. Message shows the complete cycle path: `file1.sf → file2.sf → ... → file1.sf`. No output is produced.
3. **File not found:** E-PAR-005 with the unresolved path and the source file:line:col of the @include directive.
4. No output is written if any include error is encountered.

## Invariants

1. The include resolution graph is a DAG (directed acyclic graph) for any successful build.
2. Include resolution is performed before evaluation — all includes must be resolvable at parse time.
3. The cycle detection algorithm is O(N) where N is the number of include edges — it must not have unbounded complexity.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Direct self-include: file includes itself | E-PAR-004: cycle path = `file.sf → file.sf` |
| EC-002 | Two-file cycle: A includes B, B includes A | E-PAR-004: `A.sf → B.sf → A.sf` |
| EC-003 | Three-file cycle (DEC-003) | E-PAR-004: `deck.sf → a.sf → b.sf → deck.sf` |
| EC-004 | Deep DAG (no cycle): A → B → C → D (all valid) | All 4 files resolved; AST = merged content |
| EC-005 | Diamond include: A includes B and C; both include D (no cycle) | D is included ONCE (deduplication behavior — or included twice? Specify: included at both points) |
| EC-006 | @include with interpolated path that resolves to nonexistent file (DEC-006) | E-PAR-005 shows the resolved path, not the template path |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `deck.sf` includes `header.sf` (valid); `header.sf` exists | AST = merged content; 0 errors; exit 0 | happy-path |
| A → B → A circular chain (DEC-003) | E-PAR-004: `a.sf → b.sf → a.sf`; exit 1 | error |
| `@include "{{ client }}/template.sf"` where `client="acme"` and file exists | AST includes acme/template.sf content; 0 errors | happy-path |
| `@include "{{ client }}/template.sf"` where `client="acme"` but file missing | E-PAR-005: `File not found: 'acme/template.sf' (referenced at deck.sf:5:3)`; exit 1 | error |
| Self-include: `a.sf` includes itself | E-PAR-004: `a.sf → a.sf`; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Cycle detection terminates for any finite include graph | kani (bounded graph, provably terminating) |
| VP-TBD | No duplicate include errors: each cycle is reported exactly once | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 |
| Capability Anchor Justification | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 — this BC is the exact specification for @include resolution and cycle detection, which CAP-006 names as the cycle detection requirement |
| L2 Domain Invariants | DI-007 (include cycles are compile errors) |
| Architecture Module | slideforge-syntax crate — include resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.001 — composes with (include resolution is part of parse stage)
- BC-1.06.001 — related to (this BC covers @include; BC-1.06.001 covers the successful resolution case)
- BC-1.06.002 — related to (BC-1.06.002 covers cycle detection from CAP-006 perspective)

## Architecture Anchors

- `architecture/system-overview.md` — include graph traversal

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
