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

# BC-1.06.001: @include Resolves Local .sf File and Inlines at Parse Time

## Description

The `@include "path.sf"` directive resolves a path relative to the including file's
directory and inlines the contents of the included file at the point of the directive
during the parse stage. The result is equivalent to physically concatenating the files.
Variables, set rules, aliases, and slide blocks from the included file are all inlined.

## Preconditions

1. An `@include "path.sf"` directive appears in the deck source.
2. The resolved path points to a readable .sf file.
3. No circular include chain exists (covered by BC-1.06.002).

## Postconditions

1. The included file's content is inlined at the position of the `@include` directive.
2. Source spans in errors from included content reference the included file's path, not the entry file.
3. The resulting AST is equivalent to the merged source.
4. Build exits with code 0 on success.

## Invariants

1. Include resolution happens at parse time, before evaluation.
2. Path resolution is relative to the including file's directory (not the working directory).
3. Variable scoping: included content shares the deck-level scope.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@include "subdir/slides.sf"` (subdirectory) | Resolved relative to including file; inlined correctly |
| EC-002 | `@include "{{ client }}/slides.sf"` (variable path per DEC-006) | Variable resolved first; E-PAR-005 if file not found at resolved path |
| EC-003 | Included file itself contains `@include` directives | Resolved recursively; cycle detection applies |
| EC-004 | Included file with syntax errors | E-PAR-* errors with included file's path in span |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@include "header.sf"` where header.sf has 1 title slide | Title slide inlined at include position | happy-path |
| `@include "missing.sf"` | E-PAR-005: file not found; exit 1 | error |
| Included file has indentation error | E-PAR-001 with included file path; exit 1 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | AST from `@include` is identical to AST from physically merged source | unit test (fixture comparison) |
| VP-TBD | Error spans in included content cite the included file path | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 |
| Capability Anchor Justification | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 — local file inclusion is the primary mechanism of CAP-006 |
| L2 Domain Invariants | DI-007 (include cycles are compile errors — handled by BC-1.06.002) |
| Architecture Module | slideforge-syntax crate — include resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.06.002 — composes with (cycle detection for @include)
- BC-1.06.003 — related to (variable path in @include)
- BC-1.01.001 — depends on (include resolution is part of the parse stage)

## Architecture Anchors

- `architecture/authoring-subsystem.md` — @include resolution at parse time

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
