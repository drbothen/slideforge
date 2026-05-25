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

# BC-5.03.006: slideforge package verify Checks All Packages Against sf.lock SHA-256 Checksums

## Description

`slideforge package verify` re-computes the SHA-256 checksum of each installed package
archive and compares it against the value recorded in `sf.lock`. Any mismatch indicates
either a corrupted download or a tampered cache, and is reported as E-PKG-003.
This enforces the supply-chain integrity guarantee per DI-019 and R-012.

## Preconditions

1. The user runs `slideforge package verify` in a project directory with `slideforge.toml`.
2. `sf.lock` exists with at least one package entry.
3. Packages have been installed (local cache exists).

## Postconditions

1. Each package archive in the local cache is re-read and its SHA-256 is computed.
2. The computed SHA-256 is compared byte-for-byte against the value in `sf.lock`.
3. If all checksums match: prints "All N packages verified." and exits 0.
4. If any checksum mismatches: prints E-PKG-003 for each mismatching package (with
   expected and actual hashes), and exits 5.
5. Verify does NOT modify `sf.lock` or re-download packages.

## Invariants

1. `package verify` is a read-only operation — no files are written, downloaded, or
   mutated. (DI-019 — checksums are verified, not updated)
2. All packages in `[dependencies]` are verified if their cache entry exists.
   Packages with missing cache entries are reported as E-PKG-001 (not silently skipped).
3. Exit code reflects the worst finding: 0 = all clean, 5 = any mismatch or missing.
4. Verify is deterministic: re-running on an unchanged cache always produces the same result.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All packages match sf.lock | "All N packages verified."; exit 0 |
| EC-002 | One package archive checksum mismatches | E-PKG-003 for that package; exit 5; other packages still verified and reported |
| EC-003 | Package in [dependencies] but local cache missing | E-PKG-001 for missing package; suggest re-running install; exit 5 |
| EC-004 | sf.lock absent | Error: cannot verify without sf.lock; hint to run `slideforge package install`; exit 5 |
| EC-005 | [dependencies] is empty | "No packages to verify."; exit 0 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 2 packages installed, both checksums correct | "All 2 packages verified."; exit 0 | happy-path |
| 1 of 2 packages has corrupted cache | E-PKG-003 for corrupted package; other reported as OK; exit 5 | edge-case |
| Run with no sf.lock | Error; exit 5; hint to install | error |
| Run with empty [dependencies] | "No packages to verify."; exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Exit 0 when all computed SHA-256s match sf.lock | integration test: install + verify on clean cache |
| VP-TBD | Exit 5 when any checksum mismatches | integration test: corrupt cache entry, run verify, assert exit 5 |
| VP-TBD | Verify does not modify sf.lock or any file | unit test: snapshot all files before verify, assert no changes after |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "Manage dependencies in slideforge.toml, lock in sf.lock" requires integrity verification to ensure the locked dependencies have not been tampered with (NFR-007 source) |
| L2 Domain Invariants | DI-019 (sf.lock must be committed; reproducible builds), NFR-007 (package SHA-256 integrity verified on use, 100% verified) |
| Architecture Module | slideforge-cli package subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.001 — depends on (install creates the sf.lock SHA-256 values that verify checks)
- BC-5.03.004 — related to (list shows what verify operates on)
- BC-5.03.005 — related to (verify can confirm clean state after remove)

## Architecture Anchors

- `architecture/cross-cutting.md#package-management` — package install and lock file design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
