---
document_type: architecture-section
section: verification-coverage-matrix
version: "1.1"
status: approved
producer: architect
timestamp: 2026-05-30T00:00:00
traces_to: ARCH-INDEX.md
---

# Verification Coverage Matrix

> VP-INDEX.md is the authoritative source of truth. This matrix must stay in sync.
> Totals row: VP-INDEX total = sum of per-tool counts = VP row count.

## VP-to-Module Mapping

| VP ID | Description | Module | Tool | Phase | Priority |
|-------|-------------|--------|------|-------|---------|
| VP-001 | Tab detection byte span accuracy | slideforge-syntax | Kani | P6 | P0 |
| VP-002 | Alt-missing produces error before layout (in validation stage) | slideforge-validate | Kani | P6 | P0 |
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
| VP-016 | XLSX: first row becomes map keys | slideforge-data | unit | P3 | P1 |
| VP-017 | XLSX: empty cells produce null not empty string | slideforge-data | unit | P3 | P1 |
| VP-018 | XLSX: missing file always produces E-DAT-004 | slideforge-data | unit | P3 | P1 |
| VP-019 | XLSX: partial-empty header row produces ParseError (no phantom column) | slideforge-data | unit | P3 | P1 |
| VP-020 | XLSX: non-string header cell (Int) produces ParseError | slideforge-data | unit | P3 | P1 |
| VP-021 | XLSX: whole-number Float 95.0 loads as Value::Int(95) | slideforge-data | Kani | P6 | P1 |
| VP-022 | XLSX: non-whole Float 3.14 loads as Value::Float | slideforge-data | Kani | P6 | P1 |
| VP-023 | XLSX: non-finite Float (NaN/Infinity) produces ParseError | slideforge-data | Kani | P6 | P1 |
| VP-024 | XLSX: valid DateTimeIso passes through as Value::Str | slideforge-data | unit | P3 | P1 |
| VP-025 | XLSX: invalid DateTimeIso string produces ParseError | slideforge-data | unit | P3 | P1 |
| VP-026 | XLSX: correct extension + wrong magic bytes produces ParseError | slideforge-data | unit | P3 | P1 |
| VP-027 | SQLite: connection opened with SQLITE_OPEN_READONLY | slideforge-data | unit | P3 | P1 |
| VP-028 | SQLite: result row column names become map keys | slideforge-data | unit | P3 | P1 |
| VP-029 | SQLite: DML in query: field produces E-DAT-003 | slideforge-data | unit | P3 | P1 |
| VP-030 | SQLite: NULL db values produce null in data value tree | slideforge-data | unit | P3 | P1 |
| VP-031 | SQLite: TEXT column with valid UTF-8 produces Value::Str | slideforge-data | unit | P3 | P1 |
| VP-032 | SQLite: TEXT column with invalid UTF-8 produces ParseError | slideforge-data | unit | P3 | P1 |
| VP-033 | SQLite: BLOB column produces base64 STANDARD with padding | slideforge-data | unit | P3 | P1 |
| VP-034 | SQLite: .db file with non-SQLite magic bytes produces ParseError | slideforge-data | unit | P3 | P1 |
| VP-035 | SQLite: extension .db3 produces UnsupportedFormat | slideforge-data | unit | P3 | P1 |
| VP-036 | SQLite: non-existent table produces actual SQLite error not DML message | slideforge-data | unit | P3 | P1 |
| VP-037 | Shape: position EMU conversion matches declared user-unit values | slideforge-layout | Kani | P6 | P1 |
| VP-038 | Shape: without alt or decorative always produces LayoutError::MissingAlt | slideforge-layout | unit | P3 | P1 |
| VP-039 | Shape: unknown shape type keyword produces E-PAR-012 (no Custom fallback) | slideforge-layout | unit | P3 | P1 |
| VP-040 | Shape: hex color case-insensitive — lowercase = uppercase for 6-digit forms | slideforge-layout | Kani | P6 | P1 |
| VP-041 | Shape: x + width == page_width is NOT off-canvas; +1 EMU IS off-canvas | slideforge-layout | Kani | P6 | P1 |
| VP-042 | Shape: slide with N shapes missing alt returns Vec with N MissingAlt errors | slideforge-layout | unit | P3 | P1 |
| VP-043 | Inline: all 12 variant types produce distinct non-empty XML in PPTX output | slideforge-pptx | unit | P3 | P1 |
| VP-044 | Inline: Bold via markdown pattern does not trigger b=1 in output | slideforge-pptx | unit | P3 | P1 |
| VP-045 | Inline: tree at depth 65 produces InlineDepthExceeded error | slideforge-layout | Kani | P6 | P1 |
| VP-046 | Inline: Xref inside MathNode is NOT flagged by xref validation pass | slideforge-layout | unit | P3 | P1 |
| VP-047 | Inline: all 12 variants survive layout pass in FrameContent::TextRun | slideforge-layout | unit | P3 | P1 |
| VP-048 | Shape: from_inches / from_em with i64::MAX returns Err(ArithmeticOverflow) — not silent saturation | slideforge-layout | Kani | P6 | P1 |
| VP-049 | LaidOutDeck.warnings populated with XrefTargetNotFound and OffCanvas from layout::run (not dropped) | slideforge-layout | unit | P3 | P1 |
| VP-050 | Shape frames in LaidOutDeck.frames appear at index >= region_count (after all placeholder frames) | slideforge-layout | unit | P3 | P1 |
| VP-051 | Brand round-trip extraction: extract brand.toml from .pptx → synthesize → color values match | slideforge-brand | integration | P3 | P1 |
| VP-052 | Brand extraction is read-only: source .pptx byte-identical before and after extract | slideforge-brand | integration | P3 | P1 |

## Per-Module Counts

| Module | Kani | Unit | Proptest | Fuzz | Integration | Total |
|--------|------|------|----------|------|-------------|-------|
| slideforge-syntax | 2 | 0 | 1 | 1 | 0 | 4 |
| slideforge-eval | 2 | 0 | 1 | 1 | 0 | 4 |
| slideforge-validate | 3 | 0 | 0 | 0 | 0 | 3 |
| slideforge-layout | 5 | 7 | 1 | 0 | 0 | 13 |
| slideforge-brand | 0 | 0 | 1 | 0 | 2 | 3 |
| slideforge-pptx | 0 | 2 | 1 | 0 | 0 | 3 |
| slideforge-pdf | 1 | 0 | 0 | 0 | 0 | 1 |
| slideforge-data | 3 | 18 | 0 | 0 | 0 | 21 |

Notes:
- slideforge-data: VP-021/VP-022/VP-023 use Kani (pure `promote_float` function); VP-016 through VP-020 and VP-024 through VP-036 (minus VP-021/022/023) use unit tests = 18 unit VPs.
- slideforge-layout: VP-037/VP-040/VP-041/VP-045/VP-048 use Kani (pure arithmetic/comparison); VP-038/VP-039/VP-042/VP-046/VP-047/VP-049/VP-050 use unit tests; VP-011 proptest = 5 Kani + 7 unit + 1 proptest. VP-048 (Kani, ArithmeticOverflow checked_mul); VP-049 (unit, warnings not dropped); VP-050 (unit, frame ordering).
- slideforge-pptx: VP-013 proptest + VP-043/VP-044 unit = 1 proptest + 2 unit = 3 total.
- slideforge-brand: VP-012 proptest (round-trip: synthesize → extract) + VP-051 integration (extraction direction: .pptx → brand.toml → synthesize) + VP-052 integration (read-only: source file byte-identical after extract) = 1 proptest + 2 integration = 3 total.

## Totals

| Metric | Count |
|--------|-------|
| Total VPs | 52 |
| Kani proofs | 16 |
| Proptest suites | 5 |
| Fuzz targets | 2 |
| Unit test VPs | 27 |
| Integration VPs | 2 |
| P0 (Phase 6 blocking) | 7 |
| P1 (stretch / Phase 3+) | 45 |

**Arithmetic check:** 16 (Kani) + 5 (proptest) + 2 (fuzz) + 27 (unit) + 2 (integration) = 52 total. Consistent.

**VP-INDEX cross-check:** VP-INDEX total = 52. Coverage matrix VP row count = 52. Per-tool column totals: 16 + 5 + 2 + 27 + 2 = 52. Consistent.
