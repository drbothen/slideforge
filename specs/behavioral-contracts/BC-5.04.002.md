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

# BC-5.04.002: .sfconfig Cascade (Max 3 Levels) Applies Family-Specific Overrides

## Description

`.sfconfig` files in subdirectories of a workspace apply family-specific overrides on
top of the workspace root `slideforge.toml`. The cascade is: deck-local `.sfconfig` >
intermediate directory `.sfconfig` > workspace root `slideforge.toml`, with a maximum
depth of 3 levels. This allows organizations to have brand families (e.g., one brand
for client-facing decks, another for internal) without duplicating configuration.

## Preconditions

1. A workspace with at least one member deck is configured.
2. A `.sfconfig` file exists at one or more directory levels (workspace root excluded —
   the root config is `slideforge.toml`).
3. A build is invoked for a member deck in a directory with a `.sfconfig` ancestor.

## Postconditions

1. Settings from `.sfconfig` at the deck's directory level override workspace-root
   `slideforge.toml` settings for that deck only.
2. If an intermediate `.sfconfig` exists (one level above deck, one below workspace root),
   it applies as a second override layer (deck-local > intermediate > workspace root).
3. The cascade depth is max 3 levels: workspace root → intermediate → deck-local.
   A 4th `.sfconfig` level is ignored with E-CFG-006 warning.
4. Only the settings present in `.sfconfig` are overridden; absent settings fall through
   to the higher-level config.
5. A malformed `.sfconfig` produces E-CFG-006 with the path and parse detail.

## Invariants

1. The cascade depth cap is exactly 3 levels — no unbounded directory traversal.
2. `.sfconfig` uses the same TOML format as `slideforge.toml` (subset — workspace-level
   keys like `[workspace]` are not valid in `.sfconfig`).
3. The merge rule for config values: last-wins scalars, replace lists, deep-merge maps.
   (DI-020 — merge semantics are not configurable)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No `.sfconfig` at any level | Workspace root `slideforge.toml` settings apply directly |
| EC-002 | `.sfconfig` with `[workspace]` section | E-CFG-006: workspace section not valid in .sfconfig; that key ignored |
| EC-003 | `.sfconfig` at depth 4 (beyond max) | E-CFG-006 warning; that file ignored; cascade stops at depth 3 |
| EC-004 | `.sfconfig` with unknown key | E-CFG-006 warning (unknown key ignored); build continues |
| EC-005 | `.sfconfig` overrides brand template path | Brand loaded from .sfconfig-specified path for that deck family |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Workspace root sets `brand = "corp.pptx"`; deck-local `.sfconfig` sets `brand = "client.pptx"` | Deck uses "client.pptx"; other decks without .sfconfig use "corp.pptx" | happy-path |
| Malformed `.sfconfig` (invalid TOML) | E-CFG-006 with file path; deck-level build fails | error |
| `.sfconfig` at depth 4 | E-CFG-006 warning; file ignored; depth-3 config applies | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | deck-local .sfconfig value overrides workspace root for that deck only | integration test: two decks, one with .sfconfig override |
| VP-TBD | Cascade depth beyond 3 triggers E-CFG-006 | unit test |
| VP-TBD | Malformed .sfconfig produces E-CFG-006 with file path | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 |
| Capability Anchor Justification | CAP-026 ("Cargo-Style Workspace Configuration") per capabilities.md §CAP-026 — "apply family-specific overrides via .sfconfig cascade (max 3 levels)" is verbatim from CAP-026 |
| L2 Domain Invariants | DI-020 (merge semantics are fixed — last-wins scalars, replace lists, deep-merge maps) |
| Architecture Module | slideforge-cli config loader (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.04.001 — composes with (workspace build uses .sfconfig cascade per member)
- BC-5.04.003 — composes with (config explain shows .sfconfig cascade provenance)

## Architecture Anchors

- `architecture/cross-cutting.md#workspace` — .sfconfig cascade design (Q20, Q21)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
