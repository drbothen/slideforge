---
document_type: architecture-section
section: verification-coverage-matrix
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Verification Coverage Matrix

> VP-INDEX.md is the authoritative source of truth. This matrix must stay in sync.
> Totals row: VP-INDEX total = sum of per-tool counts = VP row count.

## VP-to-Module Mapping

| VP ID | Description | Module | Tool | Phase | Priority |
|-------|-------------|--------|------|-------|---------|
| VP-001 | Tab detection byte span accuracy | slideforge-syntax | Kani | P6 | P0 |
| VP-002 | Alt-missing produces error before parse completion | slideforge-validate | Kani | P6 | P0 |
| VP-003 | @for over bounded collection always terminates | slideforge-syntax | Kani | P6 | P0 |
| VP-004 | No implicit coercion: "1.10" stays string | slideforge-eval | Kani | P6 | P0 |
| VP-005 | Integer arithmetic in {{ expr }} no overflow | slideforge-eval | Kani | P6 | P1 |
| VP-006 | EMU-to-PDF coordinate mapping: correct Y-axis flip | slideforge-pdf | Kani | P6 | P0 |
| VP-007 | WCAG contrast formula: 0.04045 luminance threshold | slideforge-validate | Kani | P6 | P0 |
| VP-008 | Alt diagnostic: image with empty alt always produces diagnostic | slideforge-validate | Kani | P6 | P0 |
| VP-009 | Parse of valid .sf produces non-empty AST | slideforge-syntax | proptest | P3 | P1 |
| VP-010 | Variable scoping: outer @for vars visible in inner scope | slideforge-eval | proptest | P3 | P1 |
| VP-011 | LaidOutDeck slide count equals Deck slide count | slideforge-layout | proptest | P3 | P1 |
| VP-012 | 12-slot palette round-trip: synthesize → extract → same colors | slideforge-brand | proptest | P3 | P1 |
| VP-013 | Every synthesized PPTX is valid ZIP with [Content_Types].xml | slideforge-pptx | proptest | P3 | P1 |
| VP-014 | Parser fuzz: any input terminates and produces errors or AST | slideforge-syntax | fuzz | P6 | P1 |
| VP-015 | Eval fuzz: any valid AST terminates eval within time bound | slideforge-eval | fuzz | P6 | P1 |

## Per-Module Counts

| Module | Kani | Proptest | Fuzz | Integration | Total |
|--------|------|----------|------|-------------|-------|
| slideforge-syntax | 2 | 1 | 1 | 0 | 4 |
| slideforge-eval | 2 | 1 | 1 | 0 | 4 |
| slideforge-validate | 3 | 0 | 0 | 0 | 3 |
| slideforge-layout | 0 | 1 | 0 | 0 | 1 |
| slideforge-brand | 0 | 1 | 0 | 0 | 1 |
| slideforge-pptx | 0 | 1 | 0 | 0 | 1 |
| slideforge-pdf | 1 | 0 | 0 | 0 | 1 |

## Totals

| Metric | Count |
|--------|-------|
| Total VPs | 15 |
| Kani proofs | 8 |
| Proptest suites | 5 |
| Fuzz targets | 2 |
| Integration VPs | 0 |
| P0 (Phase 6 blocking) | 7 |
| P1 (stretch / Phase 3+) | 8 |

**Arithmetic check:** 8 + 5 + 2 + 0 = 15 total. Consistent.
