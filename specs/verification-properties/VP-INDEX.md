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
| Total VPs | 50 |
| Kani proofs | 16 |
| Proptest suites | 5 |
| Fuzz targets | 2 |
| Unit test VPs | 27 |
| P0 (Phase 6 blocking) | 7 |
| P1 (stretch goals) | 43 |

---

## Registry

| VP ID | Description | Module | Tool | Phase | Priority | Status |
|-------|-------------|--------|------|-------|---------|--------|
| VP-001 | Tab detection byte span accuracy | slideforge-syntax | Kani | P6 | P0 | draft |
| VP-002 | Alt-missing produces error before layout (in validation stage) | slideforge-validate | Kani | P6 | P0 | draft |
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
| VP-016 | XLSX: first row becomes map keys | slideforge-data | unit | P3 | P1 | draft |
| VP-017 | XLSX: empty cells produce null not empty string | slideforge-data | unit | P3 | P1 | draft |
| VP-018 | XLSX: missing file always produces E-DAT-004 | slideforge-data | unit | P3 | P1 | draft |
| VP-019 | XLSX: partial-empty header row produces ParseError (no phantom column) | slideforge-data | unit | P3 | P1 | draft |
| VP-020 | XLSX: non-string header cell (Int) produces ParseError | slideforge-data | unit | P3 | P1 | draft |
| VP-021 | XLSX: whole-number Float 95.0 loads as Value::Int(95) | slideforge-data | Kani | P6 | P1 | draft |
| VP-022 | XLSX: non-whole Float 3.14 loads as Value::Float | slideforge-data | Kani | P6 | P1 | draft |
| VP-023 | XLSX: non-finite Float (NaN/Infinity) produces ParseError | slideforge-data | Kani | P6 | P1 | draft |
| VP-024 | XLSX: valid DateTimeIso passes through as Value::Str | slideforge-data | unit | P3 | P1 | draft |
| VP-025 | XLSX: invalid DateTimeIso string produces ParseError | slideforge-data | unit | P3 | P1 | draft |
| VP-026 | XLSX: correct extension + wrong magic bytes produces ParseError | slideforge-data | unit | P3 | P1 | draft |
| VP-027 | SQLite: connection opened with SQLITE_OPEN_READONLY | slideforge-data | unit | P3 | P1 | draft |
| VP-028 | SQLite: result row column names become map keys | slideforge-data | unit | P3 | P1 | draft |
| VP-029 | SQLite: DML in query: field produces E-DAT-003 | slideforge-data | unit | P3 | P1 | draft |
| VP-030 | SQLite: NULL db values produce null in data value tree | slideforge-data | unit | P3 | P1 | draft |
| VP-031 | SQLite: TEXT column with valid UTF-8 produces Value::Str | slideforge-data | unit | P3 | P1 | draft |
| VP-032 | SQLite: TEXT column with invalid UTF-8 produces ParseError | slideforge-data | unit | P3 | P1 | draft |
| VP-033 | SQLite: BLOB column produces base64 STANDARD with padding | slideforge-data | unit | P3 | P1 | draft |
| VP-034 | SQLite: .db file with non-SQLite magic bytes produces ParseError | slideforge-data | unit | P3 | P1 | draft |
| VP-035 | SQLite: extension .db3 produces UnsupportedFormat | slideforge-data | unit | P3 | P1 | draft |
| VP-036 | SQLite: non-existent table produces actual SQLite error not DML message | slideforge-data | unit | P3 | P1 | draft |
| VP-037 | Shape: position EMU conversion matches declared user-unit values | slideforge-layout | Kani | P6 | P1 | draft |
| VP-038 | Shape: without alt or decorative always produces LayoutError::MissingAlt | slideforge-layout | unit | P3 | P1 | draft |
| VP-039 | Shape: unknown shape type keyword produces E-PAR-012 (no Custom fallback) | slideforge-layout | unit | P3 | P1 | draft |
| VP-040 | Shape: hex color case-insensitive — lowercase = uppercase for 6-digit forms; Rgb stores integer bytes not strings | slideforge-layout | Kani | P6 | P1 | draft |
| VP-041 | Shape: x + width == page_width is NOT off-canvas; + 1 EMU IS off-canvas | slideforge-layout | Kani | P6 | P1 | draft |
| VP-042 | Shape: slide with N shapes missing alt returns Multiple with N MissingAlt entries (N=1 also returns Multiple, not unwrapped) | slideforge-layout | unit | P3 | P1 | draft |
| VP-043 | Inline: all 12 variant types produce distinct non-empty XML in PPTX output | slideforge-pptx | unit | P3 | P1 | draft |
| VP-044 | Inline: Bold via markdown pattern does not trigger b=1 in output | slideforge-pptx | unit | P3 | P1 | draft |
| VP-045 | Inline: tree at depth 65 produces InlineDepthExceeded error | slideforge-layout | Kani | P6 | P1 | draft |
| VP-046 | Inline: Xref inside MathNode is NOT flagged by xref validation pass | slideforge-layout | unit | P3 | P1 | draft |
| VP-047 | Inline: all 12 variants survive layout pass in FrameContent::TextRun | slideforge-layout | unit | P3 | P1 | draft |
| VP-048 | Shape: from_inches / from_em with i64::MAX returns Err(ArithmeticOverflow) — not silent saturation | slideforge-layout | Kani | P6 | P1 | draft |
| VP-049 | LaidOutDeck.warnings populated with XrefTargetNotFound and OffCanvas from layout::run (not dropped) | slideforge-layout | unit | P3 | P1 | draft |
| VP-050 | Shape frames in LaidOutDeck.frames appear at index >= region_count (after all placeholder frames) | slideforge-layout | unit | P3 | P1 | draft |

---

## BC Traceability

| VP ID | Traced BC / Invariant |
|-------|----------------------|
| VP-001 | BC-1.01.003, DI-018 |
| VP-002 | BC-5.01.001, DI-001 |
| VP-003 | BC-1.04.003, DI-005 |
| VP-004 | BC-1.02.003, DI-004 |
| VP-005 | BC-1.02.001, DI-004 |
| VP-006 | BC-4.03.005, DI-010 |
| VP-007 | BC-4.01.004, BC-4.03.003 |
| VP-008 | BC-5.01.001, DI-001 |
| VP-009 | BC-1.01.001, DI-018 |
| VP-010 | BC-1.02.005 |
| VP-011 | DI-009, DI-012 |
| VP-012 | BC-2.01.002, DI-015 |
| VP-013 | BC-4.01.001 |
| VP-014 | BC-1.01.001, DI-018 |
| VP-015 | BC-1.02.001, DI-005 |
| VP-016 | BC-1.03.006, DI-004 |
| VP-017 | BC-1.03.006, DI-004 |
| VP-018 | BC-1.03.006 |
| VP-019 | BC-1.03.006, DI-004 |
| VP-020 | BC-1.03.006, DI-004 |
| VP-021 | BC-1.03.006, DI-004 |
| VP-022 | BC-1.03.006, DI-004 |
| VP-023 | BC-1.03.006, DI-004 |
| VP-024 | BC-1.03.006 |
| VP-025 | BC-1.03.006 |
| VP-026 | BC-1.03.006 |
| VP-027 | BC-1.03.007, DI-004 |
| VP-028 | BC-1.03.007 |
| VP-029 | BC-1.03.007, DI-004 |
| VP-030 | BC-1.03.007, DI-004 |
| VP-031 | BC-1.03.007, DI-004 |
| VP-032 | BC-1.03.007, DI-004 |
| VP-033 | BC-1.03.007 |
| VP-034 | BC-1.03.007 |
| VP-035 | BC-1.03.007 |
| VP-036 | BC-1.03.007 |
| VP-037 | BC-3.04.001, DI-010 |
| VP-038 | BC-3.04.001, DI-001 |
| VP-039 | BC-3.04.001, DI-021 |
| VP-040 | BC-3.04.001 |
| VP-041 | BC-3.04.001 |
| VP-042 | BC-3.04.001, DI-018 |
| VP-043 | BC-3.05.001, DI-004 |
| VP-044 | BC-3.05.001, DI-004 |
| VP-045 | BC-3.05.001, DI-018 |
| VP-046 | BC-3.05.001 |
| VP-047 | BC-3.05.001 |
| VP-048 | BC-3.04.001 |
| VP-049 | BC-3.04.001, BC-3.05.001 |
| VP-050 | BC-3.04.001 |
