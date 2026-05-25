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

# BC-1.06.003: @include with Variable Path Reports Resolved Path on Error

## Description

When `@include "{{ var }}/template.sf"` is used and the resolved path does not exist,
the error message reports the fully resolved path (after variable substitution), not the
template string. This allows users to understand which dynamic path failed, not just
which template pattern was used. This covers DEC-006 from the domain edge-case catalog.

## Preconditions

1. An `@include "{{ var }}/path.sf"` directive has a variable component.
2. The variable is defined and resolves to a string.
3. The resolved path points to a file that does not exist.

## Postconditions

1. E-PAR-005 is emitted with the fully resolved path: `File not found: '<resolved-path>' (referenced at <file>:<line>:<col>)`.
2. The resolved path (e.g., `acme/template.sf`) is shown, not the template (e.g., `{{ client }}/template.sf`).
3. Build exits with code 1.

## Invariants

1. Variable interpolation in include paths is resolved before the file-not-found check.
2. The error always shows the resolved path, not the template string.
3. If the variable itself is undefined, E-EVL-001 is emitted for the undefined variable (before the file-not-found check).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-006) | `@include "{{ client_name }}/template.sf"` with client_name="acme" and acme/template.sf missing | E-PAR-005: `File not found: 'acme/template.sf'` |
| EC-002 | Variable in path is undefined | E-EVL-001 for the undefined variable; file check not reached |
| EC-003 | Variable resolves to empty string: `@include "{{ name }}/slides.sf"` where name="" | E-PAR-005: `File not found: '/slides.sf'` (path with empty component) |
| EC-004 | Variable path resolves to an existing file | File inlined normally per BC-1.06.001 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `vars: { client: "acme" }` / `@include "{{ client }}/deck.sf"` (file missing) | E-PAR-005: `File not found: 'acme/deck.sf'`; exit 1 | error (DEC-006) |
| `@include "{{ undefined_var }}/deck.sf"` | E-EVL-001: undefined variable; exit 2 | error |
| `vars: { client: "acme" }` / `@include "{{ client }}/deck.sf"` (file exists) | File inlined; no error | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-PAR-005 message contains the resolved (not template) path | unit test with mock file system |
| VP-TBD | Undefined variable in path produces E-EVL-001 before file check | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 |
| Capability Anchor Justification | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 — variable paths in @include are explicitly supported per CAP-006 |
| L2 Edge Cases | DEC-006 (@include path with variable that resolves to nonexistent file) |
| Architecture Module | slideforge-syntax crate — include resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.06.001 — depends on (happy-path include resolution)
- BC-1.02.002 — composes with (undefined variable error when variable in path is missing)

## Architecture Anchors

- `architecture/authoring-subsystem.md` — dynamic include path resolution

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
