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

# BC-5.04.003: slideforge config explain Shows Configuration Provenance for Any Setting

## Description

`slideforge config explain [<key>]` prints the resolved value of a configuration key and
its provenance (which config file contributed the value, at what cascade level). Without
a key argument, it prints the full resolved configuration with provenance for every key.
This is the primary debugging tool for cascade configuration and is required per CAP-026.

## Preconditions

1. `slideforge config explain` is invoked from a project directory (with or without a
   workspace).
2. At least a `slideforge.toml` exists in the current directory or a parent directory.

## Postconditions

1. For each configuration key, the output shows:
   - The resolved value.
   - The file that contributed the value (full path).
   - The cascade level: `workspace-root` | `sfconfig-level-2` | `sfconfig-level-1` |
     `deck-local` | `cli-flag` | `default`.
2. `slideforge config explain brand` shows only the brand key with its value and provenance.
3. `slideforge config explain` (no key) shows ALL keys with provenance.
4. Unknown key produces E-CFG-006 with a list of valid keys.
5. Output is human-readable plain text (not JSON by default; `--format json` for
   machine-readable output).

## Invariants

1. The `config explain` output is read-only — it does NOT modify any config files.
2. CLI flags override all config-file settings; their provenance is shown as `cli-flag`.
3. Defaults (from built-in defaults, not user files) are shown with provenance `default`
   and no file path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Run from a directory with no slideforge.toml | E-CFG-005: workspace root not found; hint to run from project directory |
| EC-002 | `slideforge config explain unknown-key` | E-CFG-006: unrecognized key; list of valid keys |
| EC-003 | `--format json` flag | Output is valid JSON: `{"key": {"value": "...", "provenance": "...", "source": "..."}` |
| EC-004 | Run with `--template path/to/brand.pptx` CLI flag | `template` key shows `cli-flag` provenance with the path value |
| EC-005 | No .sfconfig, only workspace root slideforge.toml | All keys show provenance `workspace-root` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `slideforge config explain brand` in workspace with deck-local .sfconfig | Shows: `brand = "client.pptx"  (from: .sfconfig, level: deck-local)` | happy-path |
| `slideforge config explain brand` without .sfconfig | Shows: `brand = "corp.pptx"  (from: slideforge.toml, level: workspace-root)` | happy-path |
| `slideforge config explain` (all) + `--format json` | Valid JSON with all keys + provenance fields | edge-case |
| `slideforge config explain unknown` | E-CFG-006 with valid key list | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Provenance shows correct source file for each level | integration test: workspace fixture + .sfconfig; parse explain output |
| VP-TBD | `--format json` output is valid JSON | unit test: parse JSON output |
| VP-TBD | Read-only: no files modified by config explain | unit test: capture file system state before/after |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 |
| Capability Anchor Justification | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 — "Inspect configuration provenance with slideforge config explain" is verbatim from CAP-026 |
| L2 Domain Invariants | DI-020 (merge semantics are not configurable — explain shows the fixed cascade result) |
| Architecture Module | slideforge-cli config subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.04.001 — composes with (workspace config is what config explain reads)
- BC-5.04.002 — composes with (.sfconfig cascade provenance is shown by config explain)

## Architecture Anchors

- `architecture/cross-cutting.md#workspace` — config explain design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
