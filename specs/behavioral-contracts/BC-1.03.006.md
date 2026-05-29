---
document_type: behavioral-contract
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-05-28T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-003
lifecycle_status: active
introduced: v1.0.0
modified:
  - version: "1.2"
    date: 2026-05-24
    reason: "Initial draft with basic XLSX contract"
  - version: "1.3"
    date: 2026-05-28
    reason: "Adversary pass 1 adjudications: partial-empty header rejection (A), non-string header rejection (B), whole-number Float promotion rule (C), DateTimeIso validation (D), extension+magic-byte policy (H). All decisions concrete per canonical principle."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.03.006: Load @data from Excel (.xlsx) Spreadsheet at Compile Time

## Description

The `@data name from "path.xlsx"` directive loads tabular data from an Excel
spreadsheet at compile time. The first sheet is used by default; a specific sheet
can be selected via `sheet: "Sheet2"`. Rows are loaded as a list of maps keyed by
the header row (first row of the sheet). Empty cells are represented as null. This
extends the DataSource plugin surface with XLSX format support per CAP-003.

Header cells MUST be non-empty strings in the source file. Non-string header cells
(Integer, Float, Boolean, Date) and partially-empty header rows are both rejected
with ParseError — no phantom column names are invented, no silent type coercion is
applied. Numeric data cells are promoted to `Value::Int` when the float value is
whole-numbered and representable as i64; otherwise `Value::Float`.

## Preconditions

1. A `@data name from "path.xlsx"` directive appears in the deck source.
2. The file at `path` is readable and has a `.xlsx` extension.
3. The file contains at least one sheet with a header row.

## Postconditions

1. All rows in the target sheet are parsed into a list of maps.
2. The first row is used as map keys (column headers). Every cell in the header row
   MUST be a non-empty, String-typed cell in the source XLSX. Partial-empty header rows
   and non-string header cells both trigger `DataError::ParseError` (E-DAT-003).
3. Subsequent rows are loaded as objects with those keys.
4. Empty cells are loaded as null (not empty string, not omitted).
5. Numeric cells are loaded as follows:
   - `calamine::Data::Float(f)` where `f.fract() == 0.0 && f.is_finite()` and
     `f` is within `i64::MIN as f64..=i64::MAX as f64` → `Value::Int(f as i64)`
   - `calamine::Data::Float(f)` otherwise → `Value::Float(OrderedFloat(f))`
   - `calamine::Data::Int(n)` → `Value::Int(n)` (pass-through; calamine may surface
     integers directly from integer-typed cells)
6. Date/time cells with calamine-validated ISO 8601 content are loaded as ISO 8601 string.
   Calamine `Data::DateTimeIso(s)` values are validated with
   `chrono::DateTime::parse_from_rfc3339` or `chrono::NaiveDate::parse_from_str("%Y-%m-%d")`.
   A value calamine claims is DateTimeIso but fails ISO 8601 parse produces
   `DataError::ParseError` (E-DAT-003) with the offending cell coordinate.
7. The data is bound to `name` in the deck scope; `{{ name }}`, `{{ name[0] }}`,
   and `{{ name[0].ColumnHeader }}` are valid.
8. Build exits 0 on success.
9. The file has `.xlsx` extension AND passes XLSX magic-byte detection (ZIP local file
   header: `50 4B 03 04`). Files with correct extension but wrong magic bytes produce
   `DataError::ParseError` (E-DAT-003: not a valid XLSX/ZIP archive).

## Invariants

1. All `@data` sources are resolved before any slide evaluation begins.
2. No implicit type coercion: boolean cells stay as bool, string cells as string. (DI-004)
3. Only `.xlsx` format is supported (not `.xls` — legacy format is out of scope for v1.0).
   Extension validation is enforced first (reject `.xls` with UnsupportedFormat before
   attempting to read bytes). For `.xlsx` extension, magic-byte detection additionally
   validates the file is a ZIP/XLSX archive.
4. The sheet selection (`sheet:`) is optional; default is the first sheet.
5. Partial-empty header rows are rejected. "Partial-empty" means one or more cells in
   the header row are blank/empty while others are populated. Phantom column names (e.g.,
   `"__empty_0"`) MUST NOT be invented. (CLAUDE.md canonical principle: no silent fallback;
   DI-004: no implicit type coercion)
6. Non-string header cells are rejected. A header cell that calamine parses as
   Int, Float, Bool, or DateTime MUST produce `DataError::ParseError` naming the cell's
   row/column coordinate and the offending calamine type. The implementer MUST NOT call
   `.to_string()` on non-string header cells and silently accept the result. (DI-004)
7. Whole-number Float promotion is mandatory for data cells (not header cells).
   The rule (postcondition 5) is applied uniformly — no cell may remain as
   `Value::Float` if its value is whole-numbered and in i64 range. (DI-004: types
   must match user intent; `95.0` written in Excel as an integer cell is `Int(95)`)
8. DateTimeIso validation is mandatory. calamine claiming a cell is DateTimeIso does
   not make the string valid ISO 8601. An ISO parse failure produces ParseError, not
   a pass-through of the malformed string.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist | E-DAT-004: file data source not found; exit 2 |
| EC-002 | File is `.xls` (legacy format) | E-DAT-003: unsupported format; message references .xlsx requirement |
| EC-003 | Named sheet does not exist in workbook | E-DAT-003: sheet '<name>' not found in '<path>'. Available sheets: [<list>] |
| EC-004 | Sheet has no header row (completely empty) | E-DAT-003: cannot load data from empty sheet; no rows produced |
| EC-005 | Merged cells in header row | E-DAT-003: merged cells in header row are not supported; message gives location |
| EC-006 | Sheet with formula cells | Formula result (the computed value) is used; formula text is not exposed |
| EC-007 | Partial-empty header row (some header cells blank) | E-DAT-003: header row at '<path>' has empty cell at column <idx> (0-indexed). All header cells must be non-empty strings. Do not use blank column headers; remove unused columns or name all headers.; exit 2 |
| EC-008 | Non-string header cell (e.g., header cell contains integer 2024 or float 3.14) | E-DAT-003: header cell at column <idx> in '<path>' has type <calamine-type> (value: <repr>). Header cells must be String-typed. Use a string label as the column header.; exit 2 |
| EC-009 | DateTimeIso cell with malformed ISO string (e.g., calamine claims ISO but string is "not-a-date") | E-DAT-003: datetime cell at <col>:<row> in '<path>' has invalid ISO 8601 value '<value>'. Expected format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS±HH:MM; exit 2 |
| EC-010 | Float data cell with whole-number value (e.g., Excel stores 95 as Float 95.0) | Loaded as Value::Int(95), not Value::Float(95.0) |
| EC-011 | Float data cell with non-whole value (e.g., 3.14) | Loaded as Value::Float(OrderedFloat(3.14)) |
| EC-012 | Float data cell with NaN or Infinity | E-DAT-003: numeric cell at <col>:<row> in '<path>' has non-finite value (<NaN|Infinity>). Non-finite floats are not representable in slideforge values.; exit 2 |
| EC-013 | File has `.xlsx` extension but is not a XLSX/ZIP archive (wrong magic bytes) | E-DAT-003: '<path>' has .xlsx extension but is not a valid XLSX archive (ZIP magic bytes not found). File may be corrupted or misnamed.; exit 2 |

## Acceptance Criteria (Adjudicated v1.3 — Adversary Pass 1)

The following ACs supplement the story-level ACs. These are binding BC-level requirements.

**AC-BC-001 (Item A — Partial-empty header rejection):**
A header row where any cell is calamine `Data::Empty` while at least one other cell
is non-empty MUST produce `DataError::ParseError` with message conforming to EC-007.
The implementer MUST NOT substitute `"__empty_<idx>"` or any phantom column name.
Rationale: CLAUDE.md "no silent fallback"; DI-004 no implicit coercion; phantom column
names would silently corrupt user data without surfacing the authoring error.

**AC-BC-002 (Item B — Non-string header cell rejection):**
A header cell that calamine parses as `Data::Int`, `Data::Float`, `Data::Bool`,
`Data::DateTime`, or `Data::DateTimeIso` MUST produce `DataError::ParseError` with
message conforming to EC-008. The implementer MUST NOT call `.to_string()` on the
cell value and accept the stringified result as a header. Rationale: DI-004 (no
implicit type coercion); CLAUDE.md forbidden pattern: "Implicit type coercion
(NO → bool, 1.10 → float)" — the symmetric rule applies to coercing Int to string
when string is the required type.

**AC-BC-003 (Item C — Whole-number Float promotion):**
For every calamine `Data::Float(f)` in a data cell (row index > 0):
- If `f.fract() == 0.0 && f.is_finite() && f >= i64::MIN as f64 && f <= i64::MAX as f64`
  → produce `Value::Int(f as i64)`
- If `f.is_nan() || f.is_infinite()` → produce `DataError::ParseError` per EC-012
- Otherwise → produce `Value::Float(OrderedFloat(f))`
No other code paths are acceptable. In particular, `Data::Float` MUST NOT unconditionally
produce `Value::Float`. Rationale: AC-003 in the story spec says "Numeric cells with
integer values are loaded as Value::Int(i64)" — calamine may surface whole-number
floats from cells that the user wrote as integers (e.g., rust_xlsxwriter writes `95i32`
as `Data::Float(95.0)` for some cell types). The promotion rule makes this reliable.

**AC-BC-004 (Item D — DateTimeIso validation):**
For every calamine `Data::DateTimeIso(s)` cell, validate `s` via:
1. Try `chrono::DateTime::parse_from_rfc3339(s)` (full datetime with timezone)
2. Try `chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")` (date-only)
3. Try `chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")` (local datetime)
If none succeeds → `DataError::ParseError` per EC-009. If success → `Value::Str(Arc::from(s))`.
Do NOT pass `s` through without validation. calamine claiming a cell is DateTimeIso
is not authoritative. Rationale: defense-in-depth; no silent corrupt data.

**AC-BC-005 (Item H — Extension + magic-byte policy):**
Extension check is performed first:
1. If path extension is not `.xlsx` (case-insensitive) → check if it is `.xls`:
   - `.xls` → `DataError::UnsupportedFormat` per EC-002
   - Other extension → `DataError::UnsupportedFormat` with "only .xlsx extension supported"
2. If extension is `.xlsx`, open the file and check the first 4 bytes:
   - Bytes `[0x50, 0x4B, 0x03, 0x04]` present → continue loading
   - Bytes absent → `DataError::ParseError` per EC-013
Extension check alone is insufficient. Magic-byte check prevents calamine from
attempting to parse corrupt or misnamed files and producing misleading errors.

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data kpis from "kpis.xlsx"` (file has 3 data rows, string headers, numeric data) | kpis bound as list of 3 maps; first-row headers as keys | happy-path |
| `@data d from "data.xlsx" sheet: "Q3"` (sheet exists) | Q3 sheet rows loaded as list of maps | happy-path |
| `@data d from "missing.xlsx"` | E-DAT-004; exit 2 | error |
| `@data d from "data.xlsx" sheet: "NoSuchSheet"` | E-DAT-003: sheet not found; exit 2 | error |
| XLSX with header row `["name", "", "score"]` (blank middle header) | E-DAT-003: empty header cell at column 1; exit 2 | error (EC-007) |
| XLSX with header row where column 0 cell is Int(2024) | E-DAT-003: non-string header cell at column 0, type Int; exit 2 | error (EC-008) |
| XLSX with data cell containing float 95.0 (whole number) | Value::Int(95) in result map | regression (EC-010) |
| XLSX with data cell containing float 3.14 | Value::Float(OrderedFloat(3.14)) in result map | happy-path (EC-011) |
| XLSX with DateTimeIso cell `"2024-01-15"` | Value::Str("2024-01-15") | happy-path |
| XLSX with DateTimeIso cell `"not-a-date"` | E-DAT-003: invalid ISO 8601 value; exit 2 | error (EC-009) |
| File with `.xlsx` extension but ZIP magic bytes missing | E-DAT-003: not a valid XLSX archive; exit 2 | error (EC-013) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | First row of .xlsx becomes map keys | unit test: fixture xlsx, assert header row = map keys |
| VP-TBD | Empty cells produce null (not empty string) | unit test: xlsx with empty cell, assert null in loaded data |
| VP-TBD | Missing file always produces E-DAT-004 | unit test: assert error code on missing file |
| VP-TBD | Partial-empty header row produces ParseError (no phantom column) | unit test: xlsx with blank middle header; assert E-DAT-003, no `__empty_` key in result |
| VP-TBD | Non-string header cell (Int) produces ParseError | unit test: xlsx where header cell is integer; assert E-DAT-003 with type name in message |
| VP-TBD | Whole-number Float 95.0 loads as Value::Int(95) | unit test: xlsx written with rust_xlsxwriter using integer value; assert Int(95) not Float |
| VP-TBD | Non-whole Float 3.14 loads as Value::Float | unit test: xlsx with float 3.14; assert Float(3.14) |
| VP-TBD | Non-finite Float (NaN) produces ParseError | unit test: assert E-DAT-003 for NaN cell |
| VP-TBD | Valid DateTimeIso passes through as Value::Str | unit test: calamine DateTimeIso cell with valid RFC 3339; assert Value::Str |
| VP-TBD | Invalid DateTimeIso string produces ParseError | unit test: stub calamine to return DateTimeIso("garbage"); assert E-DAT-003 |
| VP-TBD | Correct extension + wrong magic bytes produces ParseError | unit test: write non-ZIP bytes to .xlsx file; assert E-DAT-003 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — "Load structured data at compile time from JSON files, CSV files, YAML files, TOML files, HTTP/HTTPS URLs, Excel spreadsheets, and SQLite databases" explicitly names Excel spreadsheets as a supported source |
| L2 Domain Invariants | DI-004 (no implicit type coercion — XLSX cell types preserved as-is; non-string headers and phantom column names both violate this invariant) |
| Architecture Module | slideforge-data crate — DataSource plugin (SS-10) |
| Stories | STORY-020 |

## CHANGELOG

| Version | Date | Change |
|---------|------|--------|
| 1.1 | 2026-05-24 | Initial draft |
| 1.2 | 2026-05-24 | Minor clarifications (pre-adversary) |
| 1.3 | 2026-05-28 | Adversary Pass 1 adjudications (items A, B, C, D, H): reject partial-empty headers; reject non-string headers; Float-to-Int promotion rule; DateTimeIso ISO 8601 validation; extension + magic-byte two-phase check. Added EC-007 through EC-013, AC-BC-001 through AC-BC-005, VP expansions. |

## Related BCs

- BC-1.03.001 — related to (file-based variant for JSON/CSV/YAML/TOML; xlsx is the same plugin surface)
- BC-1.03.007 — related to (SQLite is the other structured data source added in parallel with xlsx)
- BC-1.03.003 — composes with (missing-field error contract applies to xlsx-loaded data too)
- BC-1.02.001 — depends on (expression evaluation uses the xlsx-loaded data)

## Architecture Anchors

- `architecture/system-overview.md` — DataSource plugin surface

## Story Anchor

STORY-020

## VP Anchors

(filled after VP creation)
