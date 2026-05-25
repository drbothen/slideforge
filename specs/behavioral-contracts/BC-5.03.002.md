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

# BC-5.03.002: @import of Package Not in sf.lock Fails with Install Hint

## Description

When a .sf file uses `@import "package/item"` and the named package is not present in
`sf.lock`, the build fails with E-PKG-001 at parse time. The error message includes the
exact `slideforge package install <url>` command the user should run. This enforces the
lockfile-first discipline: packages cannot be imported ad-hoc without explicit installation.
This covers DEC-019.

## Preconditions

1. A .sf source file contains `@import "package-name/item"`.
2. Either `sf.lock` does not exist, OR `sf.lock` exists but does not contain an entry
   for `package-name`.

## Postconditions

1. E-PKG-001 is emitted: `Package 'package-name' not found in sf.lock. Run:
   slideforge package install <repo-url>`.
2. Build exits with code 5 (package error).
3. No output is produced.
4. The error carries the file:line:col of the `@import` directive.

## Invariants

1. The package name in the `@import` path is always the first path component (before `/`).
2. The error message must include a concrete remediation command — it must NOT say
   "package not found" without the install hint.
3. This check happens at parse time (before evaluation) so no partial evaluation occurs
   with a missing package.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | sf.lock exists but package entry has wrong name case | E-PKG-001: package lookup is case-sensitive; "MyPkg" ≠ "mypkg" |
| EC-002 | Multiple @import of same missing package | E-PKG-001 emitted once per missing package (deduplicated), not per @import site |
| EC-003 | sf.lock entry exists but SHA-256 checksum mismatch | E-PKG-003 (checksum mismatch), not E-PKG-001 |
| EC-004 | Package in sf.lock but package cache missing (deleted) | Re-fetch package using sf.lock sha256 for verification; no error if refetch succeeds |
| EC-005 | --offline flag with missing package | E-PKG-001 with added note: "--offline mode; cannot fetch missing package" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@import "brand-kit/logo"` with no sf.lock | E-PKG-001 with install hint; exit 5 | error (DEC-019) |
| `@import "brand-kit/logo"` with sf.lock having `brand-kit` entry | Build continues; item resolved | happy-path |
| `@import "brand-kit/logo"` with sf.lock having `Brand-Kit` entry (case mismatch) | E-PKG-001 (case-sensitive lookup) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-PKG-001 emitted with install hint when package absent from sf.lock | unit test |
| VP-TBD | E-PKG-001 carries correct file:line:col of @import directive | unit test: check error span |
| VP-TBD | No E-PKG-001 when sf.lock contains matching package entry | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "Import package content via @import 'package/item' (distinct from local @include)" with lockfile enforcement is the core of CAP-025 |
| L2 Domain Invariants | DI-019 (sf.lock must exist for reproducible builds) |
| Architecture Module | slideforge-cli or slideforge-eval package resolution (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.001 — depends on (install is the prerequisite that creates the sf.lock entry)
- BC-5.03.003 — related to (missing sf.lock for a project with deps; warning not error)
- BC-1.06.004 — related to (@import is the mechanism; this BC specifies the error path)

## Architecture Anchors

- `architecture/plugin-architecture.md` — @import resolution and sf.lock lookup

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
