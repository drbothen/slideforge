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
capability: CAP-025
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

# BC-5.03.005: slideforge package remove Removes Package from slideforge.toml and sf.lock

## Description

`slideforge package remove <name>` removes the named package from `[dependencies]` in
`slideforge.toml` and its entry from `sf.lock`. The local cache entry is also removed.
If the package is referenced by any `.sf` source file via `@import`, a warning is
emitted listing the affected files (but removal still completes — the broken `@import`
becomes a compile-time error on the next build).

## Preconditions

1. The user runs `slideforge package remove <name>` in a project directory with a `slideforge.toml`.
2. The named package exists in `slideforge.toml` `[dependencies]`.

## Postconditions

1. The package entry is removed from `slideforge.toml` `[dependencies]`.
2. The package entry is removed from `sf.lock` (if present).
3. The local cache directory for the package is deleted.
4. If the package name appears in any `.sf` file `@import` statements, a warning is
   printed listing the affected files: "Warning: package '<name>' is still imported in [files]."
5. Exit code is 0 on success (including when warnings are emitted).

## Invariants

1. Only the named package is removed — all other `[dependencies]` entries and their `sf.lock`
   entries are preserved unchanged.
2. No `.sf` source files are modified by remove (user must update imports manually).
3. If the package is not in `[dependencies]`, the command fails with E-CFG-007 (package not found).
4. Remove is atomic: both `slideforge.toml` and `sf.lock` are updated, or neither is.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Package not installed (not in [dependencies]) | E-CFG-007: package '<name>' not found; no changes |
| EC-002 | Package has @import in deck.sf | Warning listing affected files; remove completes; exit 0 |
| EC-003 | sf.lock absent | Remove from slideforge.toml only; no error for missing sf.lock |
| EC-004 | Local cache already deleted | Remove from slideforge.toml and sf.lock; no error for missing cache |
| EC-005 | Two packages share cache path (alias conflict) | Only the named package's entry removed; other package unaffected |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Remove installed package with no @import usage | Entry removed from toml + sf.lock; exit 0 | happy-path |
| Remove package referenced in deck.sf | Removed; warning printed; exit 0; next build fails with E-PKG-001 | edge-case |
| Remove package not in [dependencies] | E-CFG-007; no changes; exit 4 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | slideforge.toml [dependencies] no longer contains package after remove | unit test: parse toml after remove, assert key absent |
| VP-TBD | sf.lock no longer contains package entry after remove | unit test: parse lock after remove, assert entry absent |
| VP-TBD | Other packages in toml and sf.lock unchanged | unit test: install 2 packages, remove 1, assert other still present |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "Manage dependencies in slideforge.toml, lock in sf.lock" explicitly covers adding and removing package dependencies |
| L2 Domain Invariants | DI-019 (sf.lock must be committed; reproducible builds — remove keeps lock consistent with toml) |
| Architecture Module | slideforge-cli package subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.001 — composes with (install is the inverse operation)
- BC-5.03.004 — related to (list shows what remove operates on)
- BC-5.03.006 — related to (verify after remove can confirm clean state)

## Architecture Anchors

- `architecture/cross-cutting.md#package-management` — package install and lock file design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
