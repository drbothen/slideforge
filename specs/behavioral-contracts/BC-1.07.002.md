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
capability: CAP-007
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

# BC-1.07.002: Variant vars: Overrides Deck-Level vars per 11-Level Precedence Chain

## Description

Each variant may declare a `vars:` block whose values override deck-level `vars:` during
evaluation for that variant. The override follows the 11-level precedence chain (Q25
decision): variant-specific vars are higher priority than deck-level vars. Inherited
variant vars are lower priority than own vars. This covers DEC-004 from the domain edge-
case catalog.

## Preconditions

1. A variant declares a `vars:` block with one or more keys.
2. Some of those keys also exist in the deck-level `vars:` block.
3. The build targets the variant.

## Postconditions

1. Within the variant build, the variant's vars values take precedence over deck-level vars for the same key.
2. Keys not overridden by the variant retain their deck-level values.
3. Scalar overrides follow last-wins semantics; list overrides use replace semantics; map overrides use deep-merge semantics.
4. Build exits with code 0 on success.

## Invariants

1. Merge semantics are fixed per DI-020: scalar = last-wins, list = replace, map = deep-merge.
2. The 11-level precedence chain is enforced — no per-field override configuration.
3. Variant vars override is applied before slide evaluation, not at interpolation time.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 (DEC-004) | Deck: `color: "blue"`, variant exec: `color: "red"`, child of exec: `color: "green"` | Child variant wins: "green" |
| EC-002 | Variant overrides a key that exists in deck vars | Variant value used; deck value discarded for this build |
| EC-003 | Variant overrides a list: deck `items: [1,2]`, variant `items: [3]` | Replace semantics: `items: [3]` (not `[1,2,3]`) |
| EC-004 | Variant overrides a nested map field | Deep merge: nested keys from variant override matching deck keys; unmatched deck keys preserved |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Deck `vars: { theme: "light" }` / exec variant `vars: { theme: "dark" }` / `--variant exec` | Slides render with theme="dark" | happy-path (DEC-004) |
| Deck `vars: { color: "blue" }` / no variant override | Slides render with color="blue" | happy-path |
| Child variant overrides parent variant (inherited exec, child sets theme) | Child value wins over parent | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Variant var override is applied before slide evaluation | unit test (inspect evaluated scope) |
| VP-TBD | Scalar override: variant value replaces deck value | unit test |
| VP-TBD | List override: variant list replaces deck list (not appended) | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 — vars: overrides with documented merge semantics are explicitly part of CAP-007 |
| L2 Domain Invariants | DI-020 (merge semantics are not configurable) |
| L2 Edge Cases | DEC-004 (variable shadowing across variant inheritance) |
| Architecture Module | slideforge-eval crate — variant scope resolution (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.07.001 — composes with (filtering comes first, then var override)
- BC-1.07.003 — depends on (acyclic inheritance required before merge semantics apply)

## Architecture Anchors

- `architecture/authoring-subsystem.md#variant-resolution` — 11-level precedence chain

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
