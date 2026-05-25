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
capability: CAP-003
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

# BC-1.03.006: Load @data from Excel (.xlsx) Spreadsheet at Compile Time

## Description

The `@data name from "path.xlsx"` directive loads tabular data from an Excel
spreadsheet at compile time. The first sheet is used by default; a specific sheet
can be selected via `sheet: "Sheet2"`. Rows are loaded as a list of maps keyed by
the header row (first row of the sheet). Empty cells are represented as null. This
extends the DataSource plugin surface with XLSX format support per CAP-003.

## Preconditions

1. A `@data name from "path.xlsx"` directive appears in the deck source.
2. The file at `path` is readable and has a `.xlsx` extension.
3. The file contains at least one sheet with a header row.

## Postconditions

1. All rows in the target sheet are parsed into a list of maps.
2. The first row is used as map keys (column headers).
3. Subsequent rows are loaded as objects with those keys.
4. Empty cells are loaded as null (not empty string, not omitted).
5. Numeric cells are loaded as the appropriate numeric type (integer or float).
6. Date/time cells are loaded as ISO 8601 string.
7. The data is bound to `name` in the deck scope; `{{ name }}`, `{{ name[0] }}`,
   and `{{ name[0].ColumnHeader }}` are valid.
8. Build exits 0 on success.

## Invariants

1. All `@data` sources are resolved before any slide evaluation begins.
2. No implicit type coercion: boolean cells stay as bool, string cells as string. (DI-004)
3. Only `.xlsx` format is supported (not `.xls` — legacy format is out of scope for v1.0).
4. The sheet selection (`sheet:`) is optional; default is the first sheet.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist | E-DAT-004: file data source not found; exit 2 |
| EC-002 | File is `.xls` (legacy format) | E-DAT-003: unsupported format; message references .xlsx requirement |
| EC-003 | Named sheet does not exist in workbook | E-DAT-003: sheet '<name>' not found in '<path>'. Available sheets: [<list>] |
| EC-004 | Sheet has no header row (completely empty) | E-DAT-003: cannot load data from empty sheet; no rows produced |
| EC-005 | Merged cells in header row | E-DAT-003: merged cells in header row are not supported; message gives location |
| EC-006 | Sheet with formula cells | Formula result (the computed value) is used; formula text is not exposed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data kpis from "kpis.xlsx"` (file has 3 data rows) | kpis bound as list of 3 maps; first-row headers as keys | happy-path |
| `@data d from "data.xlsx" sheet: "Q3"` (sheet exists) | Q3 sheet rows loaded as list of maps | happy-path |
| `@data d from "missing.xlsx"` | E-DAT-004; exit 2 | error |
| `@data d from "data.xlsx" sheet: "NoSuchSheet"` | E-DAT-003: sheet not found; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | First row of .xlsx becomes map keys | unit test: fixture xlsx, assert header row = map keys |
| VP-TBD | Empty cells produce null (not empty string) | unit test: xlsx with empty cell, assert null in loaded data |
| VP-TBD | Missing file always produces E-DAT-004 | unit test: assert error code on missing file |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — "Load structured data at compile time from JSON files, CSV files, YAML files, TOML files, HTTP/HTTPS URLs, Excel spreadsheets, and SQLite databases" explicitly names Excel spreadsheets as a supported source |
| L2 Domain Invariants | DI-004 (no implicit type coercion — XLSX cell types preserved as-is) |
| Architecture Module | slideforge-eval crate — DataSource plugin, xlsx variant (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.001 — related to (file-based variant for JSON/CSV/YAML/TOML; xlsx is the same plugin surface)
- BC-1.03.007 — related to (SQLite is the other structured data source added in parallel with xlsx)
- BC-1.03.003 — composes with (missing-field error contract applies to xlsx-loaded data too)
- BC-1.02.001 — depends on (expression evaluation uses the xlsx-loaded data)

## Architecture Anchors

- `architecture/authoring-subsystem.md#data-binding` — DataSource plugin surface

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
