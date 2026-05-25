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

# BC-1.06.004: @import Resolves to Installed Package or Fails with Install Hint

## Description

The `@import "package/item"` directive is semantically distinct from `@include`. It
references content from a versioned, installed package declared in `slideforge.toml`
and locked in `sf.lock`. If the package is not installed (not in `sf.lock`), the build
fails with a clear error message that includes the `slideforge package install` command
the user should run. This covers DEC-019 from the domain edge-case catalog.

## Preconditions

1. An `@import "package-name/item"` directive appears in the deck source.
2. The package is declared in `slideforge.toml` or not yet declared.

## Postconditions (happy path)

1. The package is found in `sf.lock`.
2. The named item from the package is resolved and inlined at the import position.
3. Build exits with code 0 on success.

## Postconditions (package not installed)

1. E-PAR-005 variant is emitted: `Package '<name>' not found in sf.lock. Run: slideforge package install <source>`.
2. Build exits with code 1.

## Invariants

1. `@import` never falls back to local file lookup — it is strictly a package operation.
2. Package content is resolved from the installed package directory (via sf.lock SHA), not from a network fetch at build time.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-019) | Package declared in slideforge.toml but `sf.lock` is missing | E-DAT warning per BC-5.03.003 + build error for the import |
| EC-002 | Package installed but specific item path not found | E-PAR-005: `Item '<path>' not found in package '<name>'` |
| EC-003 | `@import` of a package name that looks like a file path (`"./local.sf"`) | Parse error: @import requires a package name, not a file path. Use @include instead. |
| EC-004 | Package installed at a different SHA than sf.lock declares | Build error: lockfile mismatch; run `slideforge package verify` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@import "1898-slides/incident-card"` (package installed) | Incident card content inlined | happy-path |
| `@import "1898-slides/incident-card"` (not in sf.lock) | E-PAR-005 with install hint; exit 1 | error (DEC-019) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Missing package always includes install command hint in error | unit test |
| VP-TBD | @import never touches filesystem outside installed package directory | unit test with mock fs |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 |
| Capability Anchor Justification | CAP-006 ("Multi-File Composition via Includes") per capabilities.md §CAP-006 — @import for installed packages is explicitly listed as distinct from @include |
| L2 Edge Cases | DEC-019 (@import resolves to package not in sf.lock) |
| Architecture Module | slideforge-syntax crate / package manager — import resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.06.001 — related to (sibling: @include for local files)
- BC-5.03.001 — depends on (package installation creates the sf.lock entry)
- BC-5.03.002 — composes with (same error scenario: package not in sf.lock)

## Architecture Anchors

- `architecture/authoring-subsystem.md` — @import package resolution

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
