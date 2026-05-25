---
document_type: verification-property-index
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
traces_to: .factory/specs/architecture/ARCH-INDEX.md
---

# VP-INDEX — slideforge Verification Properties

> Authoritative enumeration of all verification properties.
> Changes to this index MUST propagate to verification-architecture.md and
> verification-coverage-matrix.md in the same commit burst.

---

## Summary

| Metric | Count |
|--------|-------|
| Total VPs | 15 |
| Kani proofs | 8 |
| Proptest suites | 5 |
| Fuzz targets | 2 |
| P0 (Phase 6 blocking) | 7 |
| P1 (stretch goals) | 8 |

---

## Registry

| VP ID | Description | Module | Tool | Phase | Priority | Status |
|-------|-------------|--------|------|-------|---------|--------|
| VP-001 | Tab detection byte span accuracy | slideforge-syntax | Kani | P6 | P0 | draft |
| VP-002 | Alt-missing produces error before parse completion | slideforge-syntax | Kani | P6 | P0 | draft |
| VP-003 | @for over bounded collection always terminates | slideforge-syntax | Kani | P6 | P0 | draft |
| VP-004 | No implicit coercion: "1.10" stays string | slideforge-eval | Kani | P6 | P0 | draft |
| VP-005 | Integer arithmetic in {{ expr }} no overflow | slideforge-eval | Kani | P6 | P1 | draft |
| VP-006 | EMU-to-PDF coordinate mapping: correct Y-axis flip, no overflow | slideforge-pdf | Kani | P6 | P0 | draft |
| VP-007 | WCAG contrast formula: correct luminance linearization (0.04045) | slideforge-validate | Kani | P6 | P0 | draft |
| VP-008 | Alt diagnostic invariant: image with empty alt always produces diagnostic | slideforge-validate | Kani | P6 | P0 | draft |
| VP-009 | Parse of valid .sf source produces non-empty AST | slideforge-syntax | proptest | P3 | P1 | draft |
| VP-010 | Variable scoping: outer @for vars visible in inner @for scope | slideforge-eval | proptest | P3 | P1 | draft |
| VP-011 | LaidOutDeck slide count equals Deck slide count | slideforge-layout | proptest | P3 | P1 | draft |
| VP-012 | 12-slot palette round-trip: synthesize → extract → same colors | slideforge-brand | proptest | P3 | P1 | draft |
| VP-013 | Every synthesized PPTX is valid ZIP with [Content_Types].xml | slideforge-pptx | proptest | P3 | P1 | draft |
| VP-014 | Parser fuzz: any input terminates and produces errors or AST | slideforge-syntax | fuzz | P6 | P1 | draft |
| VP-015 | Eval fuzz: any valid AST terminates eval within time bound | slideforge-eval | fuzz | P6 | P1 | draft |

---

## BC Traceability

| VP ID | Traced BC / Invariant |
|-------|----------------------|
| VP-001 | BC-1.01.003, DI-018 |
| VP-002 | BC-5.01.001, DI-001 |
| VP-003 | BC-1.04.003, DI-005 |
| VP-004 | BC-1.02.003, DI-004 |
| VP-005 | BC-1.02.001, DI-010 |
| VP-006 | BC-4.03.005, DI-010 |
| VP-007 | BC-5.01.003, DI-002 |
| VP-008 | BC-5.01.001, DI-001 |
| VP-009 | BC-1.01.001, DI-018 |
| VP-010 | BC-1.02.005 |
| VP-011 | DI-009, DI-012 |
| VP-012 | BC-2.01.002, DI-015 |
| VP-013 | BC-4.01.001 |
| VP-014 | BC-1.01.001, DI-018 |
| VP-015 | BC-1.02.001, DI-005 |
