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

# BC-5.04.001: slideforge.toml [workspace] Declares Multi-Deck Members; build --workspace Builds All

## Description

A `slideforge.toml` with a `[workspace]` section and `members = [...]` declares a
multi-deck workspace, following the Cargo workspace pattern. `slideforge build --workspace`
discovers all member decks, builds each with its own brand and variant configuration,
and reports per-deck errors. Individual deck builds still work for single-deck iteration.
This is the primary mechanism for large organizations building many branded decks from
one repository.

## Preconditions

1. A `slideforge.toml` exists in the workspace root with a `[workspace]` section.
2. `members = [...]` contains at least one glob or explicit path.
3. `slideforge build --workspace` is invoked from the workspace root.

## Postconditions

1. All member decks matching the `members` globs are discovered.
2. Each member deck is built with its own local `slideforge.toml` overrides (if present).
3. Output files are written to per-deck output directories (configurable; default:
   `<deck-dir>/dist/`).
4. Build errors in one member deck do NOT prevent other members from building
   (error isolation per member).
5. Summary report: N decks built, M failures, with per-deck exit codes.
6. Overall exit code: 0 if all succeed, non-zero if any member fails.
7. `slideforge build --workspace` without a `[workspace]` section produces E-CFG-005.

## Invariants

1. The `--workspace` flag is mutually exclusive with specifying a source file argument.
   `slideforge build deck.sf --workspace` produces E-CFG-002. (DI-020 — flag interactions)
2. Member discovery is deterministic (alphabetical order within glob expansion).
3. Workspace root `slideforge.toml` settings apply to all members (unless overridden
   by member-local `slideforge.toml`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `members = []` (empty list) | E-CFG-005 or warning: "no workspace members found"; exit non-zero |
| EC-002 | One member has a parse error | That member fails with E-PAR-NNN; other members build successfully |
| EC-003 | `slideforge build deck.sf --workspace` | E-CFG-002: mutually exclusive flags |
| EC-004 | Nested workspace (member contains another [workspace]) | Inner [workspace] is ignored for outer build; no recursion |
| EC-005 | Glob `decks/*` matches 50 members | All 50 built; summary report shows 50 entries |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Workspace with 3 member decks | 3 .pptx files in respective dist/ dirs; exit 0 | happy-path |
| Workspace with 1 failing member | 2 succeed, 1 fails; exit non-zero; per-deck errors shown | edge-case |
| `slideforge build deck.sf --workspace` | E-CFG-002: mutually exclusive; exit 64 | error |
| No [workspace] section in slideforge.toml | E-CFG-005; exit 4 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | N output files produced for N member decks | integration test: workspace fixture, count outputs |
| VP-TBD | One failing member does not block others | integration test: fixture with deliberate parse error in one member |
| VP-TBD | E-CFG-002 on combined source + --workspace | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 |
| Capability Anchor Justification | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 — "Declare multi-deck workspaces in slideforge.toml, build all members with slideforge build --workspace" is verbatim from CAP-026 |
| L2 Domain Invariants | DI-020 (merge semantics not configurable — flag interactions are fixed) |
| Architecture Module | slideforge-cli workspace subcommand (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.04.002 — composes with (.sfconfig cascade applies within each workspace member)
- BC-5.04.003 — composes with (config explain works within workspace context)

## Architecture Anchors

- `architecture/cross-cutting.md#workspace` — workspace discovery and multi-deck build design

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
