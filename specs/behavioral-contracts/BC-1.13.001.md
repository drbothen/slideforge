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
capability: CAP-028
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

# BC-1.13.001: slideforge_version "1" Required in Deck Metadata; Reject Forward-Incompatible

## Description

Every .sf source file must declare `slideforge_version "1"` (or the current major version)
in its deck metadata section. The parser validates this declaration early — before any
other parsing — and immediately rejects files with a forward-incompatible version number
(i.e., higher than what the binary supports). Files missing the declaration entirely also
produce an error with an explanation. This enables `slideforge migrate` tooling in future
versions.

## Preconditions

1. A .sf source file is submitted to the slideforge parser.
2. The binary's supported major version is "1".

## Postconditions

1. If `slideforge_version "1"` is present and matches the supported major version: parsing proceeds normally.
2. If `slideforge_version` is missing: E-PAR-010 style warning or error; message includes "Add slideforge_version \"1\" to deck metadata."
3. If `slideforge_version "2"` (or any unsupported higher version): E-PAR-010 immediately; exit 1; no further parsing.
4. If `slideforge_version "0"` (lower than supported): E-PAR-010 with hint to migrate; exit 1.
5. The error for forward-incompatible version is always fatal — it is never demoted to a warning even in `--warn-only` mode.

## Invariants

1. Version check is the FIRST validation performed — before indentation, keyword, or include checks.
2. Forward-incompatible version rejection is fail-fast and non-negotiable.
3. Minor version differences (e.g., "1.1" vs. "1.0") are ignored — only the major version gates.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge_version` is missing entirely | E-PAR-010 with hint to add the declaration; fatal |
| EC-002 | `slideforge_version "2"` on a v1-binary | E-PAR-010: "Version 2 not supported. This binary supports version 1."; exit 1 |
| EC-003 | `slideforge_version "0"` | E-PAR-010 with migrate hint; exit 1 |
| EC-004 | `slideforge_version "1"` with minor suffix `"1.2"` | Major "1" matches; accepted (minor versions are backwards-compatible) |
| EC-005 | `slideforge_version` appears after the first slide block | E-PAR-010 style error: version declaration must appear in deck metadata at top of file |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge_version "1"` at top of file | Parsing proceeds; exit 0 | happy-path |
| File with no `slideforge_version` | E-PAR-010 with add-hint; exit 1 | error |
| `slideforge_version "2"` | E-PAR-010 with version mismatch message; exit 1 | error |
| `slideforge_version "1.2"` | Accepted (major version 1 matches); exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Version check is the first validation; other errors not reported before version error | unit test: build file with wrong version + other errors; verify version error is first |
| VP-TBD | Forward-incompatible version is always fatal, even in --warn-only | unit test with --warn-only flag |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-028 ("DSL Versioning and Forward Compatibility") per capabilities.md §CAP-028 |
| Capability Anchor Justification | CAP-028 ("DSL Versioning and Forward Compatibility") per capabilities.md §CAP-028 — "Require slideforge_version '1' in deck metadata. Gate grammar on major version. Reject forward-incompatible files with a clear error." is verbatim from CAP-028 |
| L2 Domain Invariants | (none directly) |
| Architecture Module | slideforge-syntax crate — version gate in parser entry point (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.001 — depends on (this BC's check runs before BC-1.01.001's full parse)
- BC-1.15.001 — composes with (version error carries file:line:col span)

## Architecture Anchors

- `architecture/authoring-subsystem.md#versioning` — parser version gate design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
