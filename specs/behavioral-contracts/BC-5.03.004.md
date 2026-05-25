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

# BC-5.03.004: slideforge package list Shows All Installed Packages with Versions and Lock Status

## Description

`slideforge package list` reads `slideforge.toml` `[dependencies]` and `sf.lock`,
then prints a formatted table showing each installed package: name, declared version,
resolved commit SHA (from sf.lock), and whether the lockfile entry is current.
This gives users full visibility into their dependency state without inspecting raw TOML.

## Preconditions

1. The user runs `slideforge package list` in a directory with a `slideforge.toml`.
2. `slideforge.toml` exists and is parseable.
3. `sf.lock` may or may not exist (both cases handled).

## Postconditions

1. A formatted table is printed to stdout listing all packages in `[dependencies]`.
2. Each row contains: package name, declared version/ref, resolved SHA (from sf.lock or "UNLOCKED"), lockfile status (locked/unlocked/mismatch).
3. If no packages are declared, an empty table or "No packages installed." message is printed; exit 0.
4. If `sf.lock` is absent, all packages are shown with status "UNLOCKED" and a warning is printed advising `slideforge package install`.
5. Exit code is 0 in all non-error cases (including empty list).

## Invariants

1. `package list` is a read-only operation — no files are written or mutated.
2. Output is deterministic: packages are sorted alphabetically by name.
3. If `slideforge.toml` has packages not in `sf.lock`, those are shown as "UNLOCKED" (not silently omitted).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No packages in [dependencies] | "No packages installed." message; exit 0 |
| EC-002 | sf.lock absent | All packages shown as UNLOCKED; warning printed; exit 0 |
| EC-003 | Package in sf.lock but not in slideforge.toml [dependencies] | Ignored (sf.lock is authoritative per slideforge.toml) |
| EC-004 | slideforge.toml not found | E-CFG-007; exit 4 |
| EC-005 | sf.lock SHA differs from slideforge.toml declared version | Shown as MISMATCH with both values; warns to re-run package install |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 2 packages in toml + sf.lock | Table with 2 rows; both LOCKED; exit 0 | happy-path |
| 1 package in toml, no sf.lock | 1 row, UNLOCKED; warning; exit 0 | edge-case |
| Empty [dependencies] | "No packages installed."; exit 0 | edge-case |
| Run outside project directory | E-CFG-007; exit 4 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Output rows sorted alphabetically by package name | unit test: install out-of-order; assert sorted output |
| VP-TBD | No files mutated during list | unit test: snapshot dir before/after list, assert no changes |
| VP-TBD | UNLOCKED shown for packages missing from sf.lock | unit test: toml with dep, no sf.lock, assert UNLOCKED in output |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "Install versioned content packages from git repositories via slideforge package install. Manage dependencies in slideforge.toml, lock in sf.lock." includes managing and inspecting the installed package set |
| L2 Domain Invariants | DI-019 (sf.lock must be committed; reproducible builds — list shows lockfile status to surface violations) |
| Architecture Module | slideforge-cli package subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.001 — depends on (package install creates the sf.lock entries that list displays)
- BC-5.03.003 — related to (both surface sf.lock status warnings)
- BC-5.03.005 — related to (remove is the inverse operation)
- BC-5.03.006 — related to (verify checks integrity of what list shows)

## Architecture Anchors

- `architecture/plugin-architecture.md#package-management` — package install and lock file design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
