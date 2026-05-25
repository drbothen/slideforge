---
document_type: adr
adr_id: ADR-013
title: Integer EMU coordinate system
status: accepted
date: 2026-05-24
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-013: Integer EMU Coordinate System

## Context

All OOXML geometry uses English Metric Units (EMU). Floating-point coordinates are
incompatible with `Hash + Eq` (required for comemo and Kani — DI-010, DI-011). The
OOXML schema requires integer coordinates. Using `f64` anywhere in the coordinate
system creates silent precision loss and prevents formal verification.

## Decision

All coordinate and dimension values in `LaidOutDeck` use `i64` EMU. Conversion to/from
user-facing units happens ONLY at DSL parse/layout boundaries.

Key constants:
- `EMU_PER_INCH: i64 = 914_400`
- `EMU_PER_POINT: i64 = 12_700` (1/72 inch)
- Standard 16:9 slide: `9_144_000 × 5_143_500` EMU

## Consequences

**Hash + Eq:** Integer types implement `Hash + Eq` naturally. All IR types can derive
these traits without custom implementations. comemo compatibility is automatic.

**Kani proofs (VP-006):** Integer EMU arithmetic is bounded-model-checkable. The
EMU-to-PDF coordinate mapping (pure integer arithmetic followed by one float division
at the boundary) has provable no-overflow properties for valid slide dimensions.

**OOXML correctness:** ECMA-376 defines coordinates as `ST_Coordinate` (integer EMUs).
Passing integers directly eliminates any rounding that could shift elements by ±1 EMU.

**PDF boundary:** The only float conversion is the final `emu_to_pt()` call at the
PDF export boundary: `emu.0 as f32 / EMU_PER_POINT as f32`. This is outside the pure
IR types and occurs in the effectful `slideforge-pdf` exporter.

**User-facing DSL:** Users specify coordinates in inches or cm (e.g., `x: 1.5in`).
The parser converts to EMU at parse time: `1.5 * 914_400 = 1_371_600 EMU`. The
conversion truncates sub-EMU fractions (< 1/72000 inch — imperceptible).

**No `f64` forbidden pattern:** The CLAUDE.md forbidden patterns table explicitly
prohibits `f64` in IR coordinate/size fields. CI `clippy::pedantic` cannot enforce
this automatically, but the adversary agent checks it during review.
