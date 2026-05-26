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
capability: CAP-026
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

# BC-5.06.002: slideforge init Rejects Existing Project Without --force (E-CFG-008)

## Description

When `slideforge init` is run in a directory that already contains a `slideforge.toml`,
the command fails with E-CFG-008 and makes no filesystem changes. This prevents
accidental destruction of an existing project. The user must explicitly pass `--force`
to overwrite, and even then only scaffold files are overwritten — user data files are
not deleted.

## Preconditions

1. A `slideforge.toml` already exists in the target directory.
2. The user runs `slideforge init` without the `--force` flag.

## Postconditions

1. E-CFG-008 is emitted with a message including the target directory path.
2. A hint is shown: how to use `--force` to overwrite, or remove `slideforge.toml` first.
3. No files are created, overwritten, or deleted in the target directory.
4. Exit code is 4 (EXIT_CONFIG_ERROR).

## Invariants

1. Without `--force`, no filesystem mutation occurs when `slideforge.toml` exists.
2. With `--force`, only the four scaffold files (`deck.sf`, `brand.toml`, `slideforge.toml`,
   `.gitignore`) are overwritten. The `assets/` directory and all user files are preserved.
3. Error message references the actual path where slideforge.toml was found.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Init without --force, slideforge.toml exists | E-CFG-008; no writes; exit 4 |
| EC-002 | Init with --force, slideforge.toml exists | Scaffold files overwritten; user data untouched; exit 0 |
| EC-003 | Init with --force, deck.sf does not exist | deck.sf created fresh; other scaffold files also created; exit 0 |
| EC-004 | Init in subdirectory where parent has slideforge.toml | Only checks target dir, not parent; succeeds in subdirectory |
| EC-005 | `slideforge init my-project` where my-project/slideforge.toml exists | E-CFG-008 referencing my-project/slideforge.toml; no writes |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge init` in dir with slideforge.toml (no --force) | E-CFG-008; exit 4; zero files written | happy-path (error case) |
| `slideforge init --force` in dir with slideforge.toml | Scaffold files overwritten; deck.sf valid; exit 0 | error recovery |
| `slideforge init` in empty dir | Success; E-CFG-008 not triggered | negative test |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | E-CFG-008 exit code is 4 (not 64/USAGE) | unit test: assert process exit status |
| VP-TBD | No files mutated when E-CFG-008 fires | unit test: snapshot dir state before/after failed init |
| VP-TBD | --force preserves user files not in scaffold set | integration test: add user data file; --force init; assert user file intact |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 |
| Capability Anchor Justification | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 — slideforge.toml is the project/workspace configuration file; protecting it from accidental overwrite is a workspace integrity concern |
| L2 Domain Invariants | (none — filesystem safety is a self-contained postcondition of this BC) |
| Architecture Module | slideforge-cli init subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.06.001 — composes with (defines the success path that this BC's error condition guards)
- BC-5.04.001 — related to (slideforge.toml is the workspace config entry point being protected)

## Architecture Anchors

- `architecture/plugin-architecture.md` — project scaffolding design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
