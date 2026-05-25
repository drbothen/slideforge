---
document_type: adr
adr_id: ADR-005
title: Two-IR model (Deck + LaidOutDeck)
status: accepted
date: 2026-05-24
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-005: Two-IR Model (Deck + LaidOutDeck)

## Context

The pipeline must produce multiple output formats from one source. Each format requires
different information: PPTX needs semantic structure for placeholder inheritance; PDF needs
precise coordinates for structure tree construction; HTML needs both for SVG rendering.
A single "everything" IR type would mix semantic and geometric concerns (the failure mode
observed in python-pptx per R6 research).

## Decision

Maintain two strictly distinct IR types (DI-009):

**`Deck`** — semantic, pre-layout. `slideforge-eval` produces this. Contains slide type,
field values, resolved variable bindings, register assignments, iteration results, alt text.
Does NOT contain coordinates, dimensions, font metrics, or any geometric information.

**`LaidOutDeck`** — geometric, post-layout. `slideforge-layout` produces this. Contains
all Deck semantic information PLUS positioned shapes (EMU), text flows, font metrics,
MCID reading order. Exporters receive both.

## Consequences

**Positive:**
- Exporters are independently pluggable — they share the same input contract.
- Pure/effectful boundary is clean: `Deck → LaidOutDeck` is a pure function.
- Semantic information is preserved through layout (no lossy conversion).
- Kani proofs can verify properties of both IRs independently.

**Type constraints (DI-010, DI-011):**
- ALL types in both IRs implement `Hash + Eq + Clone` from initial declaration.
- All coordinate/dimension values use integer EMU (`i64`), never `f64`.
- Use `Arc<str>` for shared string fields (cheap clone, correct Hash/Eq).

**Invariant:** No exporter may write geometric information back into the `Deck` IR.
No layout output may drop semantic information that exporters need. If a gap is
discovered, the fix is to add the missing field — not to relax the invariant.

**Comemo compatibility:** `Hash + Eq + Clone` constraints are required for future
`comemo` incremental compilation (v1.x). Designing this in from day one avoids
API-breaking changes later.
