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

# BC-1.07.003: Detect and Reject Cyclic Variant Inheritance Graph (Fail-Closed)

## Description

A deck's `variants:` blocks may declare `inherits:` relationships. These relationships
must form a directed acyclic graph (DAG). Any cycle in variant inheritance (A inherits B
inherits A) is detected at parse/compile time and produces E-PAR-011 showing the full
cycle path. No output is produced. This prevents undefined merge results and potential
infinite evaluation loops.

## Preconditions

1. A deck contains `variants:` block with one or more variants that declare `inherits:` relationships.
2. The inheritance graph contains a cycle.

## Postconditions

1. E-PAR-011 is emitted showing the full cycle: `<var1> → <var2> → ... → <var1>`.
2. Build exits with code 1 (parse error).
3. No output is produced.

## Invariants

1. The variant inheritance graph must be a DAG for any successful build. (DI-022)
2. Cycle detection is performed at parse/compile time, not at evaluation time.
3. A variant can inherit from multiple parents (flat multiple inheritance) — but the combined graph must remain acyclic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Self-inheritance: variant exec inherits exec | E-PAR-011: `exec → exec` |
| EC-002 | Two-variant cycle: A inherits B, B inherits A | E-PAR-011: `A → B → A` |
| EC-003 | Valid diamond: A and B both inherit C (no cycle) | No error — diamond DAG is valid |
| EC-004 | Deep linear chain A → B → C → D (no cycle) | No error — linear DAG is valid |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| exec inherits exec | E-PAR-011: `exec → exec`; exit 1 | error |
| exec inherits full; full inherits exec | E-PAR-011: `exec → full → exec`; exit 1 | error |
| exec inherits base; full inherits base (diamond, valid) | No error; both inherit from base | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Cycle detection terminates for any finite variant graph | kani (bounded: ≤ 100 variants) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Variant-Based Deck Segmentation") per capabilities.md §CAP-007 — variant inheritance cycles are explicitly called out as a requirement in CAP-007's inherits: list with documented merge semantics |
| L2 Domain Invariants | DI-022 (variant inheritance graph must be acyclic) |
| Architecture Module | slideforge-eval crate — variant resolution (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.01.004 — related to (analogous cycle detection for @include chains)
- BC-1.07.001 — composes with (variant filtering requires valid inheritance graph)
- BC-1.07.002 — composes with (precedence chain depends on acyclic graph)

## Architecture Anchors

- `architecture/system-overview.md` — variant merge semantics

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
