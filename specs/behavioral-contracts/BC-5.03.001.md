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

# BC-5.03.001: slideforge package install Adds Dep to slideforge.toml and sf.lock with sha256

## Description

`slideforge package install <git-url>[@<ref>]` fetches a content package from a git
repository, verifies the package manifest, adds the dependency to `[dependencies]` in
`slideforge.toml`, and writes/updates `sf.lock` with the resolved commit SHA and SHA-256
checksum of the package archive. This provides reproducible builds per DI-019.

## Preconditions

1. The user runs `slideforge package install <git-url>` in a project directory with
   a `slideforge.toml`.
2. Network access is available (not `--offline` mode).
3. The git URL is reachable and the repository contains a valid `sf-package.toml`
   manifest.

## Postconditions

1. `slideforge.toml` `[dependencies]` section contains an entry for the new package
   with the resolved version/ref.
2. `sf.lock` contains an entry for the package with:
   - `name`: package name from `sf-package.toml`
   - `url`: the git URL
   - `rev`: the resolved commit SHA (full 40-character)
   - `sha256`: SHA-256 checksum of the fetched archive
3. The package content is cached locally (in `~/.slideforge/cache/` or project-local
   cache per workspace config).
4. A subsequent `slideforge build` using `@import "package/item"` succeeds without
   re-fetching.
5. `slideforge package install` with a package already in `sf.lock` at the same version
   is a no-op (no duplicate entry added).

## Invariants

1. `sf.lock` entries are always pinned to a specific commit SHA — never a mutable ref
   like a branch name. (DI-019)
2. The SHA-256 checksum is computed over the package archive, not just the git commit.
3. If `sf.lock` does not exist, it is created. If it does exist, the new entry is
   appended; existing entries are preserved unchanged.
4. The package install does NOT modify any user .sf source files.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Network unreachable during install | E-PKG-002 emitted; slideforge.toml and sf.lock unchanged |
| EC-002 | Invalid git URL (malformed) | E-PKG-002 with malformed URL message; no partial state written |
| EC-003 | Repository lacks `sf-package.toml` | E-PKG-002: "not a valid slideforge package"; no entry added |
| EC-004 | Re-install same package at same version | No-op; sf.lock unchanged; success exit |
| EC-005 | Install with `@v1.2.3` tag ref | sf.lock entry has resolved commit SHA of that tag, not the tag name |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge package install https://github.com/example/sf-brand-pkg` | slideforge.toml dep added; sf.lock entry with sha256; build succeeds | happy-path |
| Same install command a second time | No-op; sf.lock has exactly 1 entry for the package; success | edge-case |
| Install with unreachable URL | E-PKG-002; slideforge.toml unchanged | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | sf.lock entry after install has 40-char commit SHA | unit test: parse sf.lock, assert rev length = 40 |
| VP-TBD | sf.lock entry sha256 matches archive content | unit test: re-compute sha256 of downloaded archive, assert match |
| VP-TBD | slideforge.toml unchanged on network error | unit test: mock failing network, assert toml unchanged |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 ("Package Management") per capabilities.md §CAP-025 |
| Capability Anchor Justification | CAP-025 ("Package Management") per capabilities.md §CAP-025 — "Install versioned content packages from git repositories via slideforge package install. Manage dependencies in slideforge.toml, lock in sf.lock." is verbatim from CAP-025 |
| L2 Domain Invariants | DI-019 (sf.lock must be committed; reproducible builds) |
| Architecture Module | slideforge-cli package subcommand + package resolver (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.03.002 — composes with (@import fails without sf.lock entry; install is the prerequisite)
- BC-5.03.003 — related to (missing sf.lock warning for projects with deps)

## Architecture Anchors

- `architecture/plugin-architecture.md#package-management` — package install and lock file design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
