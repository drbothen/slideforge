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
crate: slideforge-data
subsystems: [SS-10]
target_module: slideforge-data
behavioral_contracts: [BC-1.03.006, BC-1.03.007]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
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
| Story spec (this file) | ~4,000 |
| `crates/slideforge-data/src/xlsx.rs` | ~2,500 |
| `crates/slideforge-data/src/sqlite.rs` | ~2,500 |
| Test fixtures (xlsx + db files generated in-memory) | ~3,000 |
| BC files consulted (BC-1.03.006, BC-1.03.007) | ~2,000 |
| **Total** | **~14,000** |

Agent context budget: 200k tokens. This story is ~7.0% of budget — within limit.

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

- [ ] **AC-014:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023), clippy clean (NFR-022), `=` version pinning (NFR-025) applied to all new code.

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
7. **Run `cargo clippy -p slideforge-data -- -D warnings`** and fix. (15 min)
8. **Run `cargo test -p slideforge-data`** — all tests pass. (10 min)

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
fn convert_calamine_cell(cell: &calamine::DataType) -> Value {
    match cell {
        calamine::DataType::Int(n)       => Value::Int(*n),
        calamine::DataType::Float(f)     => Value::Float(OrderedFloat(*f)),
        calamine::DataType::String(s)    => Value::Str(Arc::from(s.as_str())),
        calamine::DataType::Bool(b)      => Value::Bool(*b),
        calamine::DataType::DateTime(dt) => Value::Str(Arc::from(dt.to_string().as_str())), // ISO 8601
        calamine::DataType::Empty        => Value::Null,
        calamine::DataType::Error(_)     => Value::Null, // formula errors → null
        calamine::DataType::Formula(s)   => Value::Str(Arc::from(s.as_str())), // cached string value
        // Note: calamine::DataType::Formula actually contains the cached result; check calamine 0.26 API
    }
}
```

Verify the exact `calamine 0.26.1` API — the `Formula` variant may carry the computed value differently.

NOTE: In calamine 0.26+, the cell data enum may be `Data` instead of `DataType`. Verify exact enum name at implementation time.

### rusqlite ValueRef Conversion

```rust
fn convert_rusqlite_value(val: rusqlite::types::ValueRef<'_>) -> Value {
    match val {
        ValueRef::Null         => Value::Null,
        ValueRef::Integer(n)   => Value::Int(n),
        ValueRef::Real(f)      => Value::Float(OrderedFloat(f)),
        ValueRef::Text(bytes)  => Value::Str(Arc::from(std::str::from_utf8(bytes).unwrap_or(""))),
        ValueRef::Blob(bytes)  => Value::Str(Arc::from(base64::encode(bytes).as_str())),
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
| EC-013 | SQLite BLOB column | `Value::Str` (base64-encoded) |
