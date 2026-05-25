---
document_type: adr
adr_id: ADR-001
title: PPTX generation via ooxmlsdk 0.6.1
status: accepted
date: 2026-05-24
spike_input: S1-ooxmlsdk-pptx-coverage.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-001: PPTX Generation via ooxmlsdk 0.6.1

## Context

slideforge-pptx must serialize `LaidOutDeck` + `Brand` into a valid `.pptx` ZIP archive
that opens correctly in PowerPoint, Keynote, Google Slides, and LibreOffice. The library
must cover: master/layout/slide relationship chain, placeholder inheritance (by type and by
idx), clrMapOvr, notes/handout masters, slide IDs, Content_Types, text runs, images, tables,
and shapes with EMU coordinates.

Spike S1 validated ooxmlsdk 0.6.1 against 57 capability checks.

## Decision

Adopt `ooxmlsdk = "=0.6.1"` as the PPTX generation library for `slideforge-pptx`.

## Consequences

**Positive:**
- 55/57 capability checks pass. All blocking capabilities confirmed available.
- Typed Rust API for all OOXML primitives; element ordering matches ECMA-376 via struct field order.
- Active 0.6.x maintenance.

**Workarounds required (bounded):**
- W1: Table serialization via raw XML string. Tables: `table.to_xml_bytes()` → `GraphicData.xml_children`. Encapsulated in `table_builder.rs`.
- W2: Content_Types.xml lacks `<Default>` entries. Post-process in `opc_postprocess.rs` (~50 lines): inject Default entries for `.rels` and `.xml` extensions after `to_package_bytes()`.

**Rejected alternatives:**
- `quick-xml` (raw XML): no type safety; element ordering bugs are silent.
- `openxml` (Rust): abandoned 2020; incomplete PML coverage.
- `python-pptx` via subprocess: not Rust; incompatible with single-binary.

## Implementation Notes

- Import PML types and DML types in separate `use` blocks to avoid namespace collision.
- All shapes, pictures, graphic frames go via `ShapeTreeChoice` enum variants.
- Navigate parts via relationships — never hard-code auto-numbered part paths.
- `ShapeTreeChoice`, `ParagraphChoice`, `ColorMapOverrideChoice` are choice enums, not individual fields.
- `ApplicationNonVisualDrawingProperties.placeholder_shape` (not `.placeholder`) for placeholder type.
