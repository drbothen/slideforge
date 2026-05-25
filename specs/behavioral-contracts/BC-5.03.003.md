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

# BC-5.03.003: sf.lock Missing for Project with Deps Produces Build Warning

## Description

If a project declares package dependencies in `slideforge.toml` `[dependencies]` but no
`sf.lock` file exists in the project root, the build produces E-PKG-004 (degraded severity,
exit 0) warning and continues. This is distinct from BC-5.03.002 (which fails at @import
resolution time): E-PKG-004 fires earlier at project load time when dependencies are
declared but not locked. The warning directs the user to run `slideforge package lock`.

## Preconditions

1. `slideforge.toml` exists and contains a non-empty `[dependencies]` section.
2. `sf.lock` does NOT exist in the project root.
3. The build is invoked (not just `slideforge config explain` or similar).

## Postconditions

1. E-PKG-004 is emitted: `sf.lock not committed or missing for project with dependencies.
   Run: slideforge package lock. Builds may not be reproducible.`
2. Severity is degraded (exit 0); the build continues.
3. The warning is emitted exactly once per build (not once per dependency).
4. If @import is then resolved and a matching package IS in the local cache (from a prior
   install), the build may succeed with the warning. If the package is NOT in cache,
   E-PKG-001 will be emitted (and that IS fatal).

## Invariants

1. E-PKG-004 is always cosmetic/degraded — it never blocks output. The sf.lock
   existence check is a warning, not a gate. (DI-019 — warning reflects the invariant
   but does not enforce it at the file level)
2. The warning is suppressed if `--no-lock-check` flag is passed (for CI environments
   that intentionally run without a lockfile, e.g., dependency upgrade workflows).
3. E-PKG-004 is NOT emitted if `[dependencies]` is empty or absent.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No [dependencies] section in slideforge.toml | No E-PKG-004 (no deps, no lock needed) |
| EC-002 | sf.lock exists but is empty | No E-PKG-004 (file exists; content check is not performed here) |
| EC-003 | sf.lock exists with all dependencies locked | No E-PKG-004 |
| EC-004 | slideforge.toml has [dependencies] but no @import in .sf files | E-PKG-004 still emitted (deps declared but lock missing) |
| EC-005 | --no-lock-check flag | E-PKG-004 suppressed; build continues without warning |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge.toml` with `[dependencies]`, no `sf.lock` | E-PKG-004 warning; build continues; exit 0 | happy-path |
| `slideforge.toml` with empty `[dependencies]`, no `sf.lock` | No E-PKG-004 | edge-case |
| `slideforge.toml` with deps, `sf.lock` present | No E-PKG-004 | happy-path |
| With `--no-lock-check` flag | No E-PKG-004; build proceeds | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-PKG-004 emitted once when [dependencies] present and sf.lock absent | unit test |
| VP-TBD | E-PKG-004 NOT emitted when sf.lock present | unit test |
| VP-TBD | Exit code 0 when only E-PKG-004 is present | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "lock in sf.lock" with the implied reproducibility requirement is the basis for this warning contract |
| L2 Domain Invariants | DI-019 (sf.lock must be committed to version control for reproducible builds) |
| Architecture Module | slideforge-cli project loader (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.001 — related to (install is how the lock is created)
- BC-5.03.002 — composes with (@import failure is the downstream consequence; this BC is the early warning)

## Architecture Anchors

- `architecture/cross-cutting.md#package-management` — lock file presence check at project load

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
