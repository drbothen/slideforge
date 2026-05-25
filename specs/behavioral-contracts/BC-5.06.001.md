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

# BC-5.06.001: slideforge init Scaffolds a Buildable Starter Project in the Target Directory

## Description

`slideforge init [NAME]` creates a complete, immediately buildable slideforge project
scaffold. When no `NAME` is given, files are placed in the current directory. When
`NAME` is given, a new subdirectory is created first. The generated `deck.sf` must
compile successfully with `slideforge build deck.sf` without any user modification.
A file manifest and next-step instructions are printed to stdout.

## Preconditions

1. The user runs `slideforge init` or `slideforge init <name>` from any directory.
2. The target directory is writable by the current user.
3. No `slideforge.toml` exists in the target directory (or `--force` is passed).

## Postconditions

1. The following files are created in the target directory:
   - `deck.sf` — a valid, buildable 3-slide example presentation
   - `brand.toml` — brand configuration placeholder with all 12 OOXML color slots populated with safe defaults
   - `slideforge.toml` — project configuration with default `[build]` and `[brand]` sections
   - `assets/` — an empty directory for images and data files
   - `.gitignore` — contains at minimum `dist/`
2. `slideforge build deck.sf` (immediately after init) exits 0 and produces `dist/deck.pptx`.
3. A file manifest is printed listing all created files with their paths.
4. A "get started" next-step block is printed showing how to build, watch, and explain config.
5. `init` with `NAME` creates `<NAME>/` directory before scaffolding; all files are inside it.
6. Exit code is 0 on success.

## Invariants

1. The generated `deck.sf` includes `slideforge_version "1"`, `lang "en-US"` (DI-003),
   and at least one slide with required fields populated.
2. No existing files in the target directory are modified without `--force`.
3. If creation of any scaffold file fails mid-operation (e.g., disk full), partially
   created files are cleaned up (atomic: all or nothing). (DI-009 — export atomicity)
4. `--force` overwrites `deck.sf` and `slideforge.toml` but does NOT delete files
   not created by init (user data is preserved).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge.toml` already exists (no --force) | E-CFG-008 with hint to use --force; no files written |
| EC-002 | Target directory is not writable | E-EXP-007: cannot write output to '<path>'; no partial files |
| EC-003 | `slideforge init my-project` where `my-project/` already exists | Scaffold inside existing dir; error only if slideforge.toml exists inside it |
| EC-004 | `slideforge init --force` with existing deck.sf | deck.sf overwritten with fresh scaffold; other user files untouched |
| EC-005 | Disk fills mid-scaffold (after 2 of 4 files written) | Rollback: remove all partially written files; report E-EXP-007 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge init` in empty dir | 4 files + assets/ created; exit 0; file manifest printed | happy-path |
| `slideforge init my-deck` in dir without my-deck/ | `my-deck/` created; 4 files inside; exit 0 | happy-path |
| `slideforge build deck.sf` (after init) | dist/deck.pptx produced; exit 0 | happy-path |
| `slideforge init` when slideforge.toml exists | E-CFG-008; no files written; exit 4 | error |
| `slideforge init --force` when slideforge.toml exists | Files overwritten; exit 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Generated deck.sf parses without errors | unit test: parse generated file, assert zero diagnostics |
| VP-TBD | slideforge build deck.sf exits 0 after init | integration test: init then build in temp dir |
| VP-TBD | E-CFG-008 on existing slideforge.toml without --force | unit test: mock existing file, assert error code |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 |
| Capability Anchor Justification | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 — init creates the slideforge.toml that is the entry point for workspace and project configuration |
| L2 Domain Invariants | DI-003 (lang required at deck level — generated deck.sf must include lang "en-US"), DI-009 (output atomicity) |
| Architecture Module | slideforge-cli init subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.06.002 — composes with (error case: init with --force flag behavior)
- BC-5.04.001 — related to (generated slideforge.toml is the workspace config entry point)
- BC-5.01.004 — related to (generated deck.sf must include lang "en-US" to satisfy DI-003)

## Architecture Anchors

- `architecture/plugin-architecture.md` — project scaffolding design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
