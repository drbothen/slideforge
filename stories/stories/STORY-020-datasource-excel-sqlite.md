---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-020
title: "DataSource: Excel (.xlsx) + SQLite"
epic: EPIC-05
wave: 3
points: 5
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.1.0"
crate: slideforge-data
subsystems: [SS-10]
target_module: slideforge-data
behavioral_contracts: [BC-1.03.006, BC-1.03.007]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025, NFR-036, NFR-037, NFR-038]
depends_on:
  - STORY-018
blocks:
  - STORY-021
estimated_days: 2
---

# STORY-020: DataSource: Excel (.xlsx) + SQLite

## Summary

Extend `slideforge-data` with two additional `DataSource` implementations:
`XlsxDataSource` using `calamine = "=0.26.1"` for Excel `.xlsx` files, and
`SqliteDataSource` using `rusqlite = "=0.32.0"` for SQLite databases. Both follow the
same `DataSource` trait pattern established in STORY-018. For XLSX: first row is header;
subsequent rows become `Value::Map` entries; empty cells are `Value::Null`; only `.xlsx`
is supported (`.xls` is rejected). For SQLite: only `SELECT` statements are permitted;
connection opened with `SQLITE_OPEN_READONLY`; result rows become `Value::Map` keyed by
column names; `NULL` columns become `Value::Null`.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file, v1.1.0 with BC v1.3 ACs) | ~6,000 |
| `crates/slideforge-data/src/xlsx.rs` | ~2,500 |
| `crates/slideforge-data/src/sqlite.rs` | ~2,500 |
| Test fixtures (xlsx + db files generated in-memory) | ~3,500 |
| BC files consulted (BC-1.03.006 v1.3, BC-1.03.007 v1.3) | ~4,000 |
| NFR catalog entries (NFR-036/037/038) | ~500 |
| **Total** | **~19,000** |

Agent context budget: 200k tokens. This story is ~9.5% of budget — within limit.

## Acceptance Criteria

### Excel (.xlsx) — BC-1.03.006

- [ ] **AC-001:** `XlsxDataSource` struct implements `DataSource` with fields: `path: Arc<str>`, `sheet: Option<Arc<str>>` (None = first sheet). The `load()` method reads the workbook via `calamine::open_workbook::<Xlsx<_>, _>()`, selects the target sheet, and returns `Value::List(rows)`.
  (traces to BC-1.03.006 postcondition 7 — data bound to name as `Value::List`)

- [ ] **AC-002:** The first row of the selected sheet is treated as the header row (column names). Each subsequent row is a `Value::Map(IndexMap<Arc<str>, Value>)` keyed by header values. If header row is empty (no cells with data), `DataError::ParseError` is returned with `E-DAT-003: cannot load data from empty sheet`.
  (traces to BC-1.03.006 postconditions 2 and 3 — first row as keys, subsequent rows as maps)

- [ ] **AC-003:** Empty cells are loaded as `Value::Null`. Numeric cells with integer values (e.g., `42`) are loaded as `Value::Int(i64)`. Numeric cells with float values (e.g., `3.14`) are loaded as `Value::Float(OrderedFloat(f64))`. Boolean cells are loaded as `Value::Bool`. String cells are loaded as `Value::Str`. Date/datetime cells are loaded as `Value::Str` in ISO 8601 format (`"2024-01-15"` or `"2024-01-15T09:00:00"`).
  (traces to BC-1.03.006 postconditions 4, 5, 6 — cell type mapping)

- [ ] **AC-004:** If `sheet: "SheetName"` is declared and the named sheet does not exist in the workbook, `DataError::ParseError` is returned with `E-DAT-003: sheet '<name>' not found in '<path>'. Available sheets: [<list>]`.
  (traces to BC-1.03.006 edge case EC-003)

- [ ] **AC-005:** A `.xls` file (legacy format) produces `DataError::UnsupportedFormat` with `E-DAT-003: '<path>' is an .xls file. Only .xlsx format is supported in v1.0. Convert to .xlsx before use.`
  (traces to BC-1.03.006 invariant 3 — only .xlsx supported; edge case EC-002)

- [ ] **AC-006:** Merged cells in the header row produce `DataError::ParseError` with `E-DAT-003: merged cells in header row are not supported at <file>:<line>`.
  (traces to BC-1.03.006 edge case EC-005)

- [ ] **AC-007:** Excel formula cells — the computed/cached value (not the formula text) is used. `calamine` provides the cached result via `DataType::Formula`. The formula string is never exposed to the DSL user.
  (traces to BC-1.03.006 edge case EC-006)

### SQLite — BC-1.03.007

- [ ] **AC-008:** `SqliteDataSource` struct implements `DataSource` with fields: `path: Arc<str>`, `query: Arc<str>`. The `load()` method opens the database in read-only mode via `rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)` and executes the query.
  (traces to BC-1.03.007 postcondition 1 — SQLITE_OPEN_READONLY)

- [ ] **AC-009:** Result set rows are loaded as `Value::List(Vec<Value::Map>)`. Each row is a `Value::Map(IndexMap<Arc<str>, Value>)` keyed by column name (or alias if aliased in SELECT). NULL SQLite values → `Value::Null`. INTEGER → `Value::Int(i64)`. REAL → `Value::Float(OrderedFloat(f64))`. TEXT → `Value::Str`. BLOB → `Value::Str` (base64-encoded via `base64 = "=0.22.1"`).
  (traces to BC-1.03.007 postconditions 2, 3, 4 — row mapping and type mapping)

- [ ] **AC-010:** If the `query:` field is absent from the `@data` directive, the parser (in `slideforge-syntax`) produces a parse error `E-PAR-NNN` before `SqliteDataSource::load()` is ever called. At the `slideforge-data` level, `query` is a required `Arc<str>` field — panicking or returning an error if empty is acceptable as a defensive measure (should not occur in practice if the parser enforces it).
  (traces to BC-1.03.007 invariant 4 — query: field is required)

- [ ] **AC-011:** Only SELECT statements are permitted. The query text is checked at the DSL parse level (parser rejects non-SELECT). As a defense-in-depth measure at the data layer, `rusqlite` read-only connection flags prevent any DML at the OS/driver level. If a non-SELECT query somehow reaches `SqliteDataSource::load()`, the read-only connection causes it to fail with `DataError::ParseError` mapping to `E-DAT-003: only SELECT queries are allowed in @data sqlite sources`.
  (traces to BC-1.03.007 invariant 5 and edge case EC-003)

- [ ] **AC-012:** A query returning zero rows produces `Value::List(vec![])` without error. Field access on an empty list produces `E-DAT-005` per BC-1.03.003 (handled by the evaluator, not this crate).
  (traces to BC-1.03.007 edge case EC-005)

- [ ] **AC-013:** Duplicate column names in SELECT result (no alias) produce `DataError::ParseError` with `E-DAT-003: SELECT result has duplicate column name '<name>'; use aliases`.
  (traces to BC-1.03.007 edge case EC-006)

- [ ] **AC-014:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023), clippy clean (NFR-022), `=` version pinning (NFR-025) applied to all new code. A `cargo bench` target `xlsx_10k_rows` validates the large-sheet load time < 2,000ms wall-clock (NFR-036); a `sqlite_10k_rows` target validates SQLite 10k-row query time < 500ms (NFR-037); peak Value-tree memory for a 10,000-row XLSX sheet stays < 64MB (NFR-038). These benchmarks serve as CI gates.

## BC-Traced Acceptance Criteria (BC-1.03.006 v1.3 + BC-1.03.007 v1.3 Adjudications)

The following ACs were added per product-owner adjudication (factory-artifacts commit c8788a21)
to align the story with BC v1.3 postconditions, invariants, and edge cases.
They APPEND to AC-001–AC-014 without renumbering existing ACs.

### Excel (.xlsx) — BC-1.03.006 v1.3 adjudications

- [ ] **AC-015 (BC-1.03.006 AC-BC-001 — Partial-empty header rejection):**
  A header row where at least one cell is `calamine::Data::Empty` while at least one
  other cell is non-empty MUST produce `DataError::ParseError` with message conforming
  to EC-007: `"header row at '<path>' has empty cell at column <idx> (0-indexed). All
  header cells must be non-empty strings. Do not use blank column headers; remove unused
  columns or name all headers."` The implementer MUST NOT substitute phantom column
  names such as `"__empty_<idx>"`.
  (traces to BC-1.03.006 invariant 5 — partial-empty header rows rejected)

- [ ] **AC-016 (BC-1.03.006 AC-BC-002 — Non-string header cell rejection):**
  A header cell that calamine parses as `Data::Int`, `Data::Float`, `Data::Bool`,
  `Data::DateTime`, or `Data::DateTimeIso` MUST produce `DataError::ParseError` with
  message conforming to EC-008: `"header cell at column <idx> in '<path>' has type
  <calamine-type> (value: <repr>). Header cells must be String-typed. Use a string
  label as the column header."` The implementer MUST NOT call `.to_string()` on
  non-string header cells and silently accept the result.
  (traces to BC-1.03.006 invariant 6 — non-string header cells rejected)

- [ ] **AC-017 (BC-1.03.006 AC-BC-003 — Whole-number Float promotion in data cells):**
  For every `calamine::Data::Float(f)` in a data cell (row index > 0):
  - `f.fract() == 0.0 && f.is_finite() && f >= i64::MIN as f64 && f < (i64::MAX as f64)`
    → `Value::Int(f as i64)`
    (Strict `<`: f64 cannot represent i64::MAX exactly — it rounds to 2^63, causing
    silent off-by-one corruption if `<=` were used. Matches production code per
    F-PASS12-MED-1 fix; amended per F-PASS13-LOW-1 spec/code alignment.)
  - `f.is_nan() || f.is_infinite()` → `DataError::ParseError` per EC-012
    (`"numeric cell at <col>:<row> in '<path>' has non-finite value (<NaN|Infinity>)..."`)
  - Otherwise → `Value::Float(OrderedFloat(f))`
  `Data::Float` MUST NOT unconditionally produce `Value::Float`.
  (traces to BC-1.03.006 invariant 7 — whole-number Float promotion mandatory)

- [ ] **AC-018 (BC-1.03.006 AC-BC-004 — DateTimeIso strict ISO 8601 validation):**
  For every `calamine::Data::DateTimeIso(s)` cell, validate `s` by trying in order:
  1. `chrono::DateTime::parse_from_rfc3339(s)` (full datetime with timezone)
  2. `chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")` (date-only)
  3. `chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")` (local datetime)
  If none succeeds → `DataError::ParseError` per EC-009
  (`"datetime cell at <col>:<row> in '<path>' has invalid ISO 8601 value '<value>'..."`).
  If any succeeds → `Value::Str(Arc::from(s))`. Passing `s` through without validation
  is forbidden.
  (traces to BC-1.03.006 invariant 8 — DateTimeIso validation mandatory)

- [ ] **AC-019 (BC-1.03.006 AC-BC-005 — Extension + magic-byte two-phase validation):**
  Extension check is performed first (before opening the file):
  1. If path extension is not `.xlsx` (case-insensitive): `.xls` → `DataError::UnsupportedFormat`
     per EC-002; any other extension → `DataError::UnsupportedFormat` "only .xlsx extension
     supported". Do NOT read bytes.
  2. If extension is `.xlsx`, open file and check first 4 bytes against
     `[0x50, 0x4B, 0x03, 0x04]` (ZIP local file header magic).
     - Bytes present → continue loading.
     - Bytes absent → `DataError::ParseError` per EC-013
       (`"'<path>' has .xlsx extension but is not a valid XLSX archive (ZIP magic bytes not
       found). File may be corrupted or misnamed."`).
  Extension check alone is insufficient; magic-byte check always follows for `.xlsx` files.
  (traces to BC-1.03.006 postcondition 9 — extension + magic-byte both required)

### SQLite — BC-1.03.007 v1.3 adjudications

- [ ] **AC-020 (BC-1.03.007 AC-BC-006 — Strict UTF-8 decode for TEXT columns):**
  `rusqlite::types::ValueRef::Text(bytes)` MUST be decoded via `std::str::from_utf8(bytes)`.
  The lossy variant (`std::str::from_utf8_lossy` / `String::from_utf8_lossy`) is
  FORBIDDEN. On `Err` → `DataError::ParseError` per EC-008:
  `"TEXT column '<column_name>' at row <row_idx> in '<path>' contains invalid UTF-8 bytes.
  SQLite TEXT values must be valid UTF-8."` On `Ok(s)` → `Value::Str(Arc::from(s))`.
  (traces to BC-1.03.007 invariant 6 — strict UTF-8; no silent U+FFFD substitution)

- [ ] **AC-021 (BC-1.03.007 AC-BC-007 — Base64 STANDARD encoding for BLOB columns):**
  `rusqlite::types::ValueRef::Blob(bytes)` MUST be encoded using exactly
  `base64::engine::general_purpose::STANDARD.encode(bytes)` (RFC 4648 §4, alphabet
  A-Za-z0-9+/, with `=` padding to 4-character boundary). Alternative engines
  (`URL_SAFE`, `STANDARD_NO_PAD`, `URL_SAFE_NO_PAD`) MUST NOT be used. The encoded
  string becomes a `Value::Str`. Example: `[0x00, 0xFF, 0x42]` → `Value::Str("AP9C")`.
  (traces to BC-1.03.007 invariant 7 — STANDARD base64 canonical and stable)

- [ ] **AC-022 (BC-1.03.007 AC-BC-008 — Closed extension list + magic-byte check):**
  Extension check is performed first (before opening the file):
  1. If path extension is not one of `.db`, `.sqlite`, `.sqlite3` (case-insensitive) →
     `DataError::UnsupportedFormat` per EC-010:
     `"unsupported extension for SQLite data source: '<ext>'. Accepted extensions: .db,
     .sqlite, .sqlite3"`. Do NOT attempt to open the file.
  2. If extension is in the accepted set, open the file and read the first 16 bytes.
     Check against `b"SQLite format 3\0"` (canonical SQLite file header magic).
     - Match fails → `DataError::ParseError` per EC-009:
       `"'<path>' has .<ext> extension but is not a valid SQLite database (SQLite file
       header not found). File may be corrupted or misnamed."`
     - Match succeeds → proceed with `rusqlite::Connection::open_with_flags(...)`.
  Extensions `.db3`, `.s3db`, `.sl3` are explicitly outside the accepted set.
  (traces to BC-1.03.007 invariant 8 — closed extension list + magic-byte both required)

- [ ] **AC-023 (BC-1.03.007 AC-BC-009 — SELECT/WITH prefix DML rejection + honest pass-through):**
  DML rejection is implemented as an explicit SELECT-prefix check before `prepare()`:
  1. Trim leading whitespace from the query string.
  2. If the trimmed query does NOT start with `SELECT` or `WITH` (case-insensitive) →
     `DataError::ParseError` per EC-003:
     `"only SELECT queries are allowed in @data sqlite sources (at <file>:<line>)"`.
  3. If the trimmed query starts with `SELECT` or `WITH`, call `connection.prepare(query)`.
     Any error from `prepare()` or subsequent `query_map()` → `DataError::ParseError`
     with the ACTUAL rusqlite error message (not the generic DML string). Example:
     `"SQLite error: no such table: 'events'"`.
  The implementer MUST NOT use the "only SELECT queries are allowed" string for
  `prepare()`/`query_map()` failures unrelated to DML keywords.
  (traces to BC-1.03.007 invariant 5 — DML vs other-error message differentiation)

## Previous Story Intelligence

Continues from STORY-018 (file parsing patterns established). The `DataError` enum, `DataFormat`, `DataSourceContext`, and `Value` conversion patterns are fully established. This story follows those exact patterns for two new formats.

Key notes:
- `calamine` uses its own `DataType` enum for cell values. Write a `convert_calamine_cell(cell: &DataType) -> Value` helper function.
- `rusqlite` column values are accessed via `rusqlite::types::ValueRef`. Write a `convert_rusqlite_value(val: ValueRef) -> Value` helper function.
- Both XLSX and SQLite sources return `Value::List(rows)` at the top level (list of maps), matching CSV behavior.

## Architecture Compliance Rules

1. **SS-10 Effectful — file I/O:** Both XLSX and SQLite reading are file I/O operations. The `slideforge-data` crate is correctly classified as Effectful (SS-10). No async.
2. **`rusqlite` SQLITE_OPEN_READONLY:** This is a hard security requirement from BC-1.03.007 invariant 2. The connection flags `OpenFlags::SQLITE_OPEN_READ_ONLY` must be used. Do NOT use `Connection::open()` (which defaults to read-write).
3. **`rusqlite` bundled feature:** Use `rusqlite = { version = "=0.32.0", features = ["bundled"] }` to ship SQLite with the binary and avoid runtime dependency on the system libsqlite3. This is required for reproducible builds and Windows support.
4. **Forbidden dependencies:** Same as STORY-018. No `slideforge-eval`, `slideforge-syntax`, exporter crates.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `calamine` | `=0.26.1` | Excel .xlsx reading (no C dependencies; pure Rust) |
| `rusqlite` | `=0.32.0` + `features = ["bundled"]` | SQLite read-only query execution (bundles libsqlite3) |
| `base64` | `=0.22.1` | BLOB column → base64-encoded `Value::Str` |
| All from STORY-018 | see STORY-018 | Reused (`DataError`, `DataFormat`, `DataSourceContext`, etc.) |

## File Structure Requirements

Files to create:

```
crates/slideforge-data/src/
├── xlsx.rs               # XlsxDataSource + convert_calamine_cell()
├── sqlite.rs             # SqliteDataSource + convert_rusqlite_value()
```

Files to modify:

```
crates/slideforge-data/src/
├── format.rs             # Add: DataFormat::Xlsx, DataFormat::Sqlite in from_extension()
├── lib.rs                # Add: pub mod xlsx; pub mod sqlite;
├── Cargo.toml            # Add: calamine, rusqlite+bundled, base64
```

## Tasks

1. **Add `calamine`, `rusqlite+bundled`, `base64` to `Cargo.toml`** with `=` pinning. (10 min)
2. **Extend `format.rs`** — add `DataFormat::Xlsx` and `DataFormat::Sqlite` to `from_extension()`. `.xlsx` → `Xlsx`. `.db`, `.sqlite`, `.sqlite3` → `Sqlite`. (10 min)
3. **Write `src/xlsx.rs`:**
   - `XlsxDataSource { path: Arc<str>, sheet: Option<Arc<str>> }`.
   - Helper `convert_calamine_cell(cell: &calamine::DataType) -> Value`.
   - `DataSource` impl: open workbook, select sheet, validate header row, convert rows.
   - Handle: missing file → `FileNotFound`; missing sheet → `ParseError`; `.xls` → `UnsupportedFormat`; merged header cells → `ParseError`; formula cells → use cached value. (45 min)
4. **Write `src/sqlite.rs`:**
   - `SqliteDataSource { path: Arc<str>, query: Arc<str> }`.
   - Helper `convert_rusqlite_value(val: rusqlite::types::ValueRef<'_>) -> Value`.
   - `DataSource` impl: open with `SQLITE_OPEN_READ_ONLY`, prepare statement, step through rows, build `Value::List(rows)`.
   - Handle: missing file → `FileNotFound`; corrupt database → `ParseError`; DML query (defense-in-depth) → `ParseError`; zero rows → `Value::List(vec![])`; duplicate column names → `ParseError`. (45 min)
5. **Write unit tests for `xlsx.rs`** — use `calamine`'s in-memory test support or generate a small XLSX file using `rust_xlsxwriter = "=0.64"` as a test-only dep. (30 min)
6. **Write unit tests for `sqlite.rs`** — use `rusqlite` in-memory database (`:memory:` path) for all tests. (30 min)
7. **Extend unit tests for `xlsx.rs`** for BC v1.3 adjudications (AC-015–AC-019): partial-empty header, non-string header, Float→Int promotion, DateTimeIso validation, extension+magic-byte check. (30 min)
8. **Extend unit tests for `sqlite.rs`** for BC v1.3 adjudications (AC-020–AC-023): strict UTF-8 decode, STANDARD base64 for BLOB, closed extension list + magic-byte check, DML vs non-DML error message differentiation. (30 min)
9. **Write `benches/data_sources.rs`** — Criterion benchmarks `xlsx_10k_rows` and `sqlite_10k_rows` for NFR-036/037 gates. Add `heaptrack` measurement notes in comments for NFR-038 CI gate (heaptrack is a CI-only gate, not run in unit benchmark). (20 min)
10. **Run `cargo clippy -p slideforge-data -- -D warnings`** and fix. (15 min)
11. **Run `cargo test -p slideforge-data`** — all tests pass. (10 min)

## Test Strategy

### Unit tests for `xlsx.rs`

Use `rust_xlsxwriter` (dev-only) to write test xlsx files into `tempfile` directories:

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_xlsx_happy_path` | Create xlsx with headers `["name", "score"]`, rows `[["Alice", 95], ["Bob", 87]]` | `Value::List` of 2 maps with correct keys and `Value::Int` scores |
| `test_xlsx_empty_cell_is_null` | Row with one empty cell | `Value::Null` for that cell |
| `test_xlsx_date_cell_iso` | Date cell in xlsx | `Value::Str("2024-01-15")` |
| `test_xlsx_missing_sheet` | Request non-existent sheet | `DataError::ParseError` with "not found" + "Available sheets:" |
| `test_xlsx_xls_rejected` | Create a `.xls` extension file | `DataError::UnsupportedFormat` with ".xlsx" suggestion |
| `test_xlsx_formula_uses_cached` | Formula cell `=2+3` with cached value 5 | `Value::Int(5)` |

### Unit tests for `sqlite.rs`

All tests use `rusqlite` in-memory databases (no temp files needed):

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_sqlite_happy_path` | In-memory db, table `metrics(name TEXT, val INTEGER)`, 3 rows | `Value::List` of 3 maps |
| `test_sqlite_null_column` | Row with NULL in a column | `Value::Null` for that column |
| `test_sqlite_zero_rows` | `SELECT * WHERE 1=0` | `Value::List(vec![])` |
| `test_sqlite_readonly_rejects_delete` | DML query reaches `load()` | `DataError::ParseError` with "only SELECT queries" |
| `test_sqlite_missing_file` | Non-existent `.db` path | `DataError::FileNotFound` |
| `test_sqlite_duplicate_column_names` | `SELECT id, id FROM t` (no alias) | `DataError::ParseError` with "duplicate column name" |
| `test_sqlite_blob_base64` | BLOB column with bytes | `Value::Str` with base64-encoded content |

### Additional unit tests for BC v1.3 adjudications (AC-015–AC-023)

#### xlsx.rs — BC-1.03.006 v1.3

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_xlsx_partial_empty_header_rejected` | Header row `["name", "", "score"]` | `DataError::ParseError` with EC-007 message; no `__empty_` key in result |
| `test_xlsx_non_string_header_int_rejected` | Header row where column 0 cell is `Data::Int(2024)` | `DataError::ParseError` with EC-008 message citing "type Int" |
| `test_xlsx_float_whole_number_promoted_to_int` | Data cell written as `95i32` (calamine surfaces as `Data::Float(95.0)`) | `Value::Int(95)` in result map |
| `test_xlsx_float_non_whole_stays_float` | Data cell with value `3.14` | `Value::Float(OrderedFloat(3.14))` |
| `test_xlsx_float_nan_produces_error` | Data cell returning `Data::Float(f64::NAN)` | `DataError::ParseError` with EC-012 message containing "NaN" |
| `test_xlsx_datetimeiso_valid_rfc3339` | `Data::DateTimeIso("2024-01-15T09:00:00+00:00")` | `Value::Str("2024-01-15T09:00:00+00:00")` |
| `test_xlsx_datetimeiso_valid_date_only` | `Data::DateTimeIso("2024-01-15")` | `Value::Str("2024-01-15")` |
| `test_xlsx_datetimeiso_invalid_string` | `Data::DateTimeIso("not-a-date")` | `DataError::ParseError` with EC-009 message |
| `test_xlsx_wrong_magic_bytes` | File has `.xlsx` extension but first 4 bytes are `[0x00, 0x01, 0x02, 0x03]` | `DataError::ParseError` with EC-013 message |

#### sqlite.rs — BC-1.03.007 v1.3

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_sqlite_text_strict_utf8_valid` | TEXT column with valid UTF-8 string | `Value::Str` with decoded string |
| `test_sqlite_text_strict_utf8_invalid` | TEXT column with invalid UTF-8 bytes (via raw rusqlite injection) | `DataError::ParseError` with EC-008 message |
| `test_sqlite_blob_standard_base64` | BLOB column with bytes `[0x00, 0xFF, 0x42]` | `Value::Str("AP9C")` — RFC 4648 §4 STANDARD encoding |
| `test_sqlite_extension_db3_rejected` | Path with `.db3` extension | `DataError::UnsupportedFormat` with EC-010 message listing accepted extensions |
| `test_sqlite_extension_sl3_rejected` | Path with `.sl3` extension | `DataError::UnsupportedFormat` with EC-010 message |
| `test_sqlite_wrong_magic_bytes` | File with `.db` extension but first 16 bytes are not SQLite header | `DataError::ParseError` with EC-009 message |
| `test_sqlite_dml_delete_produces_dml_error` | Query `"DELETE FROM t"` | `DataError::ParseError` with EC-003 message "only SELECT queries are allowed" |
| `test_sqlite_nonexistent_table_produces_sqlite_error` | Query `"SELECT * FROM nonexistent_table"` | `DataError::ParseError` with message containing "no such table: 'nonexistent_table'" — NOT the DML string |
| `test_sqlite_with_cte_allowed` | Query `"WITH cte AS (SELECT 1) SELECT * FROM cte"` | Succeeds; returns `Value::List` with one row |

## Dependencies

**Depends on:**
- STORY-018 (file parsing infrastructure) — `DataError`, `DataFormat`, `DataSourceContext`, `Value` conversion patterns.

Dependency justification: STORY-020 depends on STORY-018 because `DataError`, `DataFormat`, `DataSourceContext`, and the `DataSource` trait infrastructure are all established in STORY-018. STORY-020 adds two new `DataSource` implementations to that same crate.

**Blocks:**
- STORY-021 (offline mode) — needs the full set of `DataSource` implementations (including XLSX and SQLite) before the offline gate logic can test "all sources unaffected by --offline when they are file-based".

**Priority rationale:** This story is P0 despite implementing P1 BCs because the full
DataSource pipeline (all 7 formats) is required for STORY-050 (E2E integration tests,
Wave 4) which validates the entire plugin assembly. Downstream P0 stories depend on the
complete data source set.

## Implementation Notes

### calamine DataType Conversion

```rust
// NOTE: calamine 0.26+ uses `calamine::Data` (not `DataType`). Verify exact enum name.
// This function is called only for DATA cells (row_idx > 0). Header cells have separate
// validation logic that rejects non-String variants (AC-016, BC-1.03.006 invariant 6).
fn convert_calamine_data_cell(cell: &calamine::Data, col: usize, row: usize, path: &str)
    -> Result<Value, DataError>
{
    match cell {
        calamine::Data::Int(n)          => Ok(Value::Int(*n)),
        calamine::Data::Float(f) => {
            // AC-017 / BC-1.03.006 invariant 7: whole-number Float promotion is mandatory.
            if f.is_nan() || f.is_infinite() {
                // AC-017 / EC-012: non-finite values produce ParseError
                Err(DataError::ParseError { /* ... EC-012 message ... */ })
            } else if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                Ok(Value::Int(*f as i64))
            } else {
                Ok(Value::Float(OrderedFloat(*f)))
            }
        },
        calamine::Data::String(s)       => Ok(Value::Str(Arc::from(s.as_str()))),
        calamine::Data::Bool(b)         => Ok(Value::Bool(*b)),
        calamine::Data::DateTimeIso(s)  => {
            // AC-018 / BC-1.03.006 invariant 8: validate ISO 8601 strictly
            validate_datetimeiso(s, col, row, path)?;
            Ok(Value::Str(Arc::from(s.as_str())))
        },
        calamine::Data::Empty           => Ok(Value::Null),
        calamine::Data::Error(_)        => Ok(Value::Null), // formula errors → null
        // Formula variant: check calamine 0.26.1 API — may carry cached result as inner Data
        _ => Ok(Value::Null),
    }
}
```

NOTE: In calamine 0.26+, the cell data enum is `Data` (not `DataType`). The `Float` variant
MUST NOT unconditionally produce `Value::Float` — BC-1.03.006 invariant 7 (AC-017) requires
whole-number promotion. Header cell validation (AC-016) is a separate code path that checks
variant type and rejects non-`Data::String` cells with `DataError::ParseError`.

### rusqlite ValueRef Conversion

```rust
use base64::Engine as _;

// AC-020 / BC-1.03.007 invariant 6: strict UTF-8; no lossy fallback.
// AC-021 / BC-1.03.007 invariant 7: STANDARD base64 with padding (RFC 4648 §4).
fn convert_rusqlite_value(
    val: rusqlite::types::ValueRef<'_>,
    col_name: &str,
    row_idx: usize,
    path: &str,
) -> Result<Value, DataError> {
    match val {
        ValueRef::Null         => Ok(Value::Null),
        ValueRef::Integer(n)   => Ok(Value::Int(n)),
        ValueRef::Real(f)      => Ok(Value::Float(OrderedFloat(f))),
        ValueRef::Text(bytes)  => {
            // MUST use strict from_utf8, NOT from_utf8_lossy (AC-020 / invariant 6)
            match std::str::from_utf8(bytes) {
                Ok(s)  => Ok(Value::Str(Arc::from(s))),
                Err(_) => Err(DataError::ParseError { /* EC-008 message */ }),
            }
        },
        ValueRef::Blob(bytes)  => {
            // MUST use general_purpose::STANDARD, NOT URL_SAFE or NO_PAD variants (AC-021 / invariant 7)
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(Value::Str(Arc::from(encoded.as_str())))
        },
    }
}
```

### SQLite Column Name Deduplication Check

After `stmt.column_names()` returns the list, check for duplicates:
```rust
let names = stmt.column_names();
let mut seen = std::collections::HashSet::new();
for name in &names {
    if !seen.insert(*name) {
        return Err(DataError::ParseError {
            path: self.path.clone(),
            format: DataFormat::Sqlite,
            reason: Arc::from(format!("SELECT result has duplicate column name '{}'; use aliases", name)),
            span,
        });
    }
}
```

### XLSX Sheet Selection

`calamine::open_workbook::<Xlsx<_>, _>(path)` returns an `Xlsx` struct. Get the sheet list via `.sheet_names()`. Select by name using `.worksheet_range(sheet_name)`. If `sheet` is `None`, use `.sheet_names()[0]`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | XLSX file does not exist | `DataError::FileNotFound` → E-DAT-004 |
| EC-002 | `.xls` (legacy format) | `DataError::UnsupportedFormat` → E-DAT-003 with .xlsx suggestion |
| EC-003 | Named sheet does not exist | `DataError::ParseError` → E-DAT-003 with "not found" + available sheets list |
| EC-004 | XLSX sheet has no header row (empty) | `DataError::ParseError` → E-DAT-003 "cannot load data from empty sheet" |
| EC-005 | Merged cells in XLSX header row | `DataError::ParseError` → E-DAT-003 "merged cells in header row" |
| EC-006 | XLSX formula cell | Use cached/computed value; formula text not exposed |
| EC-007 | SQLite file does not exist | `DataError::FileNotFound` → E-DAT-004 |
| EC-008 | SQLite file is not a valid database | `DataError::ParseError` → E-DAT-003 with sqlite error message |
| EC-009 | SQLite DML query (INSERT/UPDATE/DELETE) | Read-only connection rejects it; `DataError::ParseError` → E-DAT-003 |
| EC-010 | SQLite query references non-existent table | `DataError::ParseError` → E-DAT-003 "no such table: '<name>'" |
| EC-011 | SQLite zero-row result | `Value::List(vec![])` — no error |
| EC-012 | SQLite duplicate column names | `DataError::ParseError` → E-DAT-003 "duplicate column name" |
| EC-013 | SQLite BLOB column | `Value::Str` (base64 STANDARD RFC 4648 §4 encoding with padding) |
| EC-014 | XLSX partial-empty header row (some cells blank, some non-empty) | `DataError::ParseError` → E-DAT-003 per EC-007; no phantom column names |
| EC-015 | XLSX non-string header cell (Int, Float, Bool, or DateTime calamine type) | `DataError::ParseError` → E-DAT-003 per EC-008 naming cell type |
| EC-016 | XLSX data Float cell with whole-number value (e.g., `95.0`) | `Value::Int(95)` — Float→Int promotion applied |
| EC-017 | XLSX data Float cell with NaN or Infinity | `DataError::ParseError` → E-DAT-003 per EC-012 |
| EC-018 | XLSX DateTimeIso cell with non-ISO string | `DataError::ParseError` → E-DAT-003 per EC-009 |
| EC-019 | XLSX file with correct `.xlsx` extension but wrong ZIP magic bytes | `DataError::ParseError` → E-DAT-003 per EC-013 |
| EC-020 | SQLite TEXT column with invalid UTF-8 bytes | `DataError::ParseError` → E-DAT-003 per EC-008 (strict decode, not lossy) |
| EC-021 | SQLite file with `.db3`, `.s3db`, or `.sl3` extension | `DataError::UnsupportedFormat` → E-DAT-003 per EC-010 |
| EC-022 | SQLite file with accepted extension but wrong magic bytes (not SQLite header) | `DataError::ParseError` → E-DAT-003 per EC-009 |
| EC-023 | SQLite non-SELECT query (INSERT/UPDATE/DELETE) reaching `load()` | `DataError::ParseError` per EC-003 "only SELECT queries allowed" (DML keyword check fires first) |
| EC-024 | SQLite SELECT/WITH query against non-existent table | `DataError::ParseError` with actual rusqlite error ("no such table: '...'") — NOT the DML string |
