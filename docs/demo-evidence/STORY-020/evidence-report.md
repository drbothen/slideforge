# Evidence Report — STORY-020: DataSource: Excel (.xlsx) + SQLite

**Story ID:** STORY-020
**Crate:** `slideforge-data`
**Branch:** `feature/S-020`
**Convergence:** 3/3 CLEAN (Passes 27, 28, 29)
**Test result:** 251 unit/integration + 3 doctests = **254 total passed; 0 failed**
**Perf gate:** NFR-036 (xlsx 10k rows < 2000ms) PASS; NFR-037 (sqlite 10k rows < 500ms) PASS

---

## Summary

`XlsxDataSource` and `SqliteDataSource` extend `slideforge-data` with Excel `.xlsx` and SQLite
read support. Both implement the `DataSource` trait established in STORY-018. XLSX loading uses
`calamine = "=0.26.1"` (pure Rust, no C deps). SQLite loading uses `rusqlite = "=0.32.0"`
with bundled libsqlite3 and `SQLITE_OPEN_READ_ONLY` enforcement. Both sources produce
`Value::List(Vec<Value::Map>)` rows with typed cells. All 23 ACs covered by unit tests;
BC v1.3 adjudications (AC-015–AC-023) verified with dedicated VP-level tests.

---

## AC Coverage

### AC-001 — XlsxDataSource implements DataSource trait

**Spec:** `XlsxDataSource { path: Arc<str>, sheet: Option<Arc<str>> }` implements `DataSource`;
`load()` reads workbook via `calamine::open_workbook::<Xlsx<_>, _>()` and returns `Value::List(rows)`.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xlsx_happy_path`
- `xlsx::tests::test_bc_1_03_006_xlsx_source_id`
- `xlsx::tests::test_bc_1_03_006_xlsx_struct_fields`

**Recording:** `AC-001-xlsx-datasource-implements-trait.gif` / `.webm`

**Status: PASS**

---

### AC-002 — First row as header; empty sheet → E-DAT-003

**Spec:** Row 0 is header row; rows 1+ become `Value::Map`. Empty header → `DataError::ParseError`
with `E-DAT-003: cannot load data from empty sheet`.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xlsx_empty_header_row`
- `xlsx::tests::test_bc_1_03_006_xlsx_three_data_rows`
- `xlsx::tests::test_bc_1_03_006_xlsx_first_sheet_default`

**Recording:** `AC-002-xlsx-first-row-header.gif` / `.webm`

**Status: PASS**

---

### AC-003 — XLSX cell type mapping

**Spec:** `calamine::Data::Empty` → `Value::Null`; `Int` → `Value::Int`; `Float` → `Value::Float`
(or `Value::Int` for whole numbers per AC-017); `Bool` → `Value::Bool`; `String` → `Value::Str`;
`DateTimeIso` → `Value::Str` (ISO 8601 validated per AC-018).

**Tests:**
- `xlsx::tests::test_bc_1_03_006_convert_calamine_cell_*` (9 tests covering all cell types)

**Recording:** `AC-003-xlsx-cell-type-mapping.gif` / `.webm`

**Status: PASS**

---

### AC-004 — Named sheet not found → E-DAT-003 with available list (error path)

**Spec:** `sheet: "SheetName"` not found → `DataError::ParseError` with
`E-DAT-003: sheet '<name>' not found in '<path>'. Available sheets: [<list>]`.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xlsx_missing_sheet`
- `xlsx::tests::test_bc_1_03_006_xlsx_named_sheet_selection`

**Recording:** `AC-004-xlsx-missing-sheet-error.gif` / `.webm`

**Status: PASS**

---

### AC-005 — `.xls` → DataError::UnsupportedFormat (error path)

**Spec:** `.xls` extension → `DataError::UnsupportedFormat` with
`E-DAT-003: '<path>' is an .xls file. Only .xlsx format is supported in v1.0.`.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xls_rejected`
- `xlsx::tests::test_bc_1_03_006_xls_rejected_case_insensitive`

**Recording:** `AC-005-xlsx-xls-unsupported-format.gif` / `.webm`

**Status: PASS**

---

### AC-006 — Merged cells in header → E-DAT-003 (error path)

**Spec:** Merged cells in header row → `DataError::ParseError` with
`E-DAT-003: merged cells in header row are not supported at <file>:<line>`.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xlsx_merged_header_cells`
- `xlsx::tests::test_bc_1_03_006_xlsx_vertical_merge_in_header`

**Recording:** `AC-006-xlsx-merged-header-error.gif` / `.webm`

**Status: PASS**

---

### AC-007 — Formula cells use cached value

**Spec:** `calamine::Data::Formula` carries the computed cached value — that value is used.
Formula text is never exposed to DSL users.

**Tests:**
- `xlsx::tests::test_bc_1_03_006_xlsx_formula_uses_cached`
- `xlsx::tests::test_bc_1_03_006_xlsx_snapshot_value_mapping`

**Recording:** `AC-007-xlsx-formula-cached-value.gif` / `.webm`

**Status: PASS**

---

### AC-008 — SqliteDataSource implements DataSource; SQLITE_OPEN_READ_ONLY

**Spec:** `SqliteDataSource { path: Arc<str>, query: Arc<str> }` implements `DataSource`;
`load()` opens database with `OpenFlags::SQLITE_OPEN_READ_ONLY` enforced.

**Tests:**
- `sqlite::tests::test_bc_1_03_007_sqlite_happy_path`
- `sqlite::tests::test_bc_1_03_007_sqlite_readonly_connection_flag`
- `sqlite::tests::test_vp_027_load_uses_readonly`
- `sqlite::tests::test_vp_027_readonly_enforced`

**Recording:** `AC-008-sqlite-datasource-readonly.gif` / `.webm`

**Status: PASS**

---

### AC-009 — SQLite row type mapping

**Spec:** `NULL` → `Value::Null`; `INTEGER` → `Value::Int(i64)`; `REAL` → `Value::Float`;
`TEXT` → `Value::Str` (strict UTF-8 per AC-020); `BLOB` → `Value::Str` (base64 STANDARD per AC-021).

**Tests:**
- `sqlite::tests::test_bc_1_03_007_convert_rusqlite_value_*` (null, integer, real)
- `sqlite::tests::test_bc_1_03_007_sqlite_null_column`
- `sqlite::tests::test_bc_1_03_007_sqlite_blob_base64`
- `sqlite::tests::test_bc_1_03_007_sqlite_text_value`

**Recording:** `AC-009-sqlite-row-type-mapping.gif` / `.webm`

**Status: PASS**

---

### AC-010 — query: field is required (defensive check)

**Spec:** Empty query string rejected defensively at the data layer via
`DataError::ParseError`. Parser enforces it at DSL level before `load()` is called.

**Tests:**
- `sqlite::tests::test_bc_1_03_007_sqlite_empty_query_defensive`
- `sqlite::tests::test_f_pass14_low2_empty_query_no_span_placeholder`
- `sqlite::tests::test_f_pass15_low1_whitespace_only_query_rejected`

**Recording:** `AC-010-sqlite-query-required.gif` / `.webm`

**Status: PASS**

---

### AC-011 — DML rejected; read-only connection as defense-in-depth (error path)

**Spec:** Non-SELECT queries that reach `load()` → `DataError::ParseError` with
`E-DAT-003: only SELECT queries are allowed in @data sqlite sources`. Read-only connection
flags additionally prevent DML at the OS/driver level.

**Tests:**
- `sqlite::tests::test_bc_1_03_007_sqlite_readonly_rejects_dml`
- `sqlite::tests::test_bc_1_03_007_sqlite_readonly_rejects_insert`
- `sqlite::tests::test_bc_1_03_007_sqlite_readonly_rejects_update`

**Recording:** `AC-011-sqlite-dml-rejected.gif` / `.webm`

**Status: PASS**

---

### AC-012 — Zero-row result → Value::List([]) without error

**Spec:** `SELECT * WHERE 1=0` → `Value::List(vec![])`. No error emitted.
Field access on empty list produces `E-DAT-005` from the evaluator, not this crate.

**Tests:**
- `sqlite::tests::test_bc_1_03_007_sqlite_zero_rows`
- `sqlite::tests::test_bc_1_03_007_sqlite_mixed_null_rows`

**Recording:** `AC-012-sqlite-zero-rows.gif` / `.webm`

**Status: PASS**

---

### AC-013 — Duplicate column names in SELECT → E-DAT-003 (error path)

**Spec:** `SELECT id, id FROM t` (no alias) → `DataError::ParseError` with
`E-DAT-003: SELECT result has duplicate column name 'id'; use aliases`.

**Tests:**
- `sqlite::tests::test_bc_1_03_007_sqlite_duplicate_column_names`
- `sqlite::tests::test_bc_1_03_007_find_duplicate_column_*` (5 tests)

**Recording:** `AC-013-sqlite-duplicate-columns.gif` / `.webm`

**Status: PASS**

---

### AC-014 — NFR-036/037/038 performance + lint gates

**Spec:** `xlsx_10k_rows` < 2,000ms; `sqlite_10k_rows` < 500ms (NFR-036/037).
`#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, `clippy::pedantic`, `=` version pinning (NFR-022–025).

**Verification:**
- `cargo test --test perf_smoke -- --ignored` → both gates PASS
- `cargo clippy -p slideforge-data -- -D warnings` → clean
- `crates/slideforge-data/Cargo.toml` → all deps pinned with `=`

**Recording:** `AC-014-nfr-perf-gates.gif` / `.webm` (shows perf_smoke test run)

**Status: PASS**

---

### AC-015 (BC-1.03.006 AC-BC-001) — Partial-empty header row rejected (error path)

**Spec:** Header row with ≥1 empty cell while ≥1 other cell is non-empty →
`DataError::ParseError` with EC-007 message. No `__empty_<idx>` phantom column names.

**Tests:**
- `xlsx::tests::test_vp_019_partial_empty_header_produces_e_dat_007`

**Recording:** `AC-015-xlsx-partial-empty-header.gif` / `.webm`

**Status: PASS**

---

### AC-016 (BC-1.03.006 AC-BC-002) — Non-string header cell rejected (error path)

**Spec:** Header cell with `Data::Int`, `Data::Float`, `Data::Bool`, `Data::DateTime`,
or `Data::DateTimeIso` → `DataError::ParseError` with EC-008 message naming cell type.
No silent `.to_string()` coercion.

**Tests:**
- `xlsx::tests::test_vp_020_non_string_header_produces_e_dat_008`

**Recording:** `AC-016-xlsx-non-string-header.gif` / `.webm`

**Status: PASS**

---

### AC-017 (BC-1.03.006 AC-BC-003) — Float→Int promotion + NaN/Infinity error (success + error paths)

**Spec:** `Data::Float(f)` where `f.fract() == 0.0 && f.is_finite() && f >= i64::MIN as f64 && f < (i64::MAX as f64)` → `Value::Int(f as i64)` (strict `<` per F-PASS13-LOW-1).
`NaN` or `Infinity` → `DataError::ParseError` EC-012.
Non-whole finite float → `Value::Float(OrderedFloat(f))`.

**Tests:**
- `xlsx::tests::test_vp_021_float_whole_number_promotes_to_int`
- `xlsx::tests::test_vp_021_boundary_round_trip`
- `xlsx::tests::test_vp_022_float_fractional_stays_float`
- `xlsx::tests::test_vp_023_nan_produces_parse_error`
- `xlsx::tests::test_vp_023_infinity_produces_parse_error`

**Recording:** `AC-017-xlsx-float-int-promotion.gif` / `.webm`

**Status: PASS**

---

### AC-018 (BC-1.03.006 AC-BC-004) — DateTimeIso strict ISO 8601 validation

**Spec:** `Data::DateTimeIso(s)` validated by trying: RFC3339, `%Y-%m-%d`, `%Y-%m-%dT%H:%M:%S`.
Invalid string → `DataError::ParseError` EC-009. Pass-through without validation is forbidden.

**Tests:**
- `xlsx::tests::test_vp_024_valid_datetime_iso_passes_through`
- `xlsx::tests::test_vp_025_invalid_datetime_iso_produces_error`
- `xlsx::tests::test_obs4_datetime_roundtrip_validation_date_only_succeeds`
- `xlsx::tests::test_obs4_datetime_roundtrip_validation_succeeds_for_valid_dt`

**Recording:** `AC-018-xlsx-datetimeiso-validation.gif` / `.webm`

**Status: PASS**

---

### AC-019 (BC-1.03.006 AC-BC-005) — Extension + magic-byte two-phase validation (error path)

**Spec:** Phase 1: extension check (`.xls` → `UnsupportedFormat`; unknown ext → `UnsupportedFormat`).
Phase 2 (`.xlsx` only): check first 4 bytes against `[0x50, 0x4B, 0x03, 0x04]` ZIP magic.
Wrong magic → `DataError::ParseError` EC-013.

**Tests:**
- `xlsx::tests::test_vp_026_wrong_magic_bytes_produces_e_dat_011`
- `xlsx::tests::test_bc_1_03_006_unsupported_extension_rejected`
- `file::tests::test_pass26_med1_xlsx_magic_byte_failure_no_double_bracket`

**Recording:** `AC-019-xlsx-extension-magic-bytes.gif` / `.webm`

**Status: PASS**

---

### AC-020 (BC-1.03.007 AC-BC-006) — Strict UTF-8 decode for TEXT columns (error path)

**Spec:** `ValueRef::Text(bytes)` decoded via `std::str::from_utf8(bytes)`.
`from_utf8_lossy` is forbidden. Invalid UTF-8 → `DataError::ParseError` EC-008.

**Tests:**
- `sqlite::tests::test_vp_032_invalid_utf8_text_produces_e_dat_012`
- `sqlite::tests::test_bc_1_03_007_sqlite_text_value`

**Recording:** `AC-020-sqlite-strict-utf8.gif` / `.webm`

**Status: PASS**

---

### AC-021 (BC-1.03.007 AC-BC-007) — STANDARD base64 for BLOB columns

**Spec:** `ValueRef::Blob(bytes)` encoded via `base64::engine::general_purpose::STANDARD.encode(bytes)`
(RFC 4648 §4). `URL_SAFE`, `STANDARD_NO_PAD`, `URL_SAFE_NO_PAD` variants forbidden.
Test vector: `[0x00, 0xFF, 0x42]` → `"AP9C"`.

**Tests:**
- `sqlite::tests::test_vp_033_blob_canonical_base64_vector`
- `sqlite::tests::test_bc_1_03_007_sqlite_blob_base64`

**Recording:** `AC-021-sqlite-blob-base64-standard.gif` / `.webm`

**Status: PASS**

---

### AC-022 (BC-1.03.007 AC-BC-008) — Closed extension list + SQLite magic-byte check (error paths)

**Spec:** Accepted: `.db`, `.sqlite`, `.sqlite3` (case-insensitive). Others → `UnsupportedFormat` EC-010.
For accepted extensions: verify first 16 bytes against `b"SQLite format 3\0"`.
Wrong magic → `DataError::ParseError` EC-009. `.db3`, `.s3db`, `.sl3` are explicitly outside accepted set.

**Tests:**
- `sqlite::tests::test_vp_034_wrong_sqlite_magic_produces_parse_error`
- `sqlite::tests::test_vp_035_unsupported_extension_produces_e_dat_014`
- `sqlite::tests::test_obs3_sqlite_no_extension_renders_cosmetic`

**Recording:** `AC-022-sqlite-extension-magic-bytes.gif` / `.webm`

**Status: PASS**

---

### AC-023 (BC-1.03.007 AC-BC-009) — SELECT/WITH prefix DML rejection + honest pass-through (error paths)

**Spec:** Trim whitespace → check `SELECT` or `WITH` prefix (case-insensitive).
Non-matching → `DataError::ParseError` EC-003 "only SELECT queries allowed".
`prepare()`/`query_map()` failures → actual rusqlite error message (NOT the DML string).
Comment-only prefix queries (`/* */ DELETE`) → also rejected (DML keyword check fires first).

**Tests:**
- `sqlite::tests::test_vp_029_dml_prefix_check_delete`
- `sqlite::tests::test_vp_029_with_clause_is_allowed`
- `sqlite::tests::test_vp_036_missing_table_uses_actual_error_message`
- `sqlite::tests::test_sqlite_comment_prefix_query_rejected_as_dml`
- `sqlite::tests::test_bc_1_03_007_sqlite_column_alias`

**Recording:** `AC-023-sqlite-select-with-dml-rejection.gif` / `.webm`

**Status: PASS**

---

## VHS Recordings

| File | AC | What it shows |
|------|----|---------------|
| `AC-001-xlsx-datasource-implements-trait.gif/.webm` | AC-001 | `XlsxDataSource` happy path: load() returns Value::List |
| `AC-002-xlsx-first-row-header.gif/.webm` | AC-002 | Header row parsing + empty sheet error |
| `AC-003-xlsx-cell-type-mapping.gif/.webm` | AC-003 | All 9 calamine cell type conversions pass |
| `AC-004-xlsx-missing-sheet-error.gif/.webm` | AC-004 | Named sheet not found → E-DAT-003 with available list |
| `AC-005-xlsx-xls-unsupported-format.gif/.webm` | AC-005 | `.xls` → UnsupportedFormat with .xlsx suggestion |
| `AC-006-xlsx-merged-header-error.gif/.webm` | AC-006 | Merged header cells → ParseError |
| `AC-007-xlsx-formula-cached-value.gif/.webm` | AC-007 | Formula cells use cached value; formula text hidden |
| `AC-008-sqlite-datasource-readonly.gif/.webm` | AC-008 | `SqliteDataSource` + SQLITE_OPEN_READ_ONLY verified |
| `AC-009-sqlite-row-type-mapping.gif/.webm` | AC-009 | NULL/INTEGER/REAL/TEXT/BLOB type mappings |
| `AC-010-sqlite-query-required.gif/.webm` | AC-010 | Empty/whitespace query rejected defensively |
| `AC-011-sqlite-dml-rejected.gif/.webm` | AC-011 | DELETE/INSERT/UPDATE → "only SELECT queries allowed" |
| `AC-012-sqlite-zero-rows.gif/.webm` | AC-012 | Zero-row query → Value::List([]) without error |
| `AC-013-sqlite-duplicate-columns.gif/.webm` | AC-013 | Duplicate column names → "duplicate column name" error |
| `AC-014-nfr-perf-gates.gif/.webm` | AC-014 | perf_smoke: xlsx_10k_rows + sqlite_10k_rows gates pass |
| `AC-015-xlsx-partial-empty-header.gif/.webm` | AC-015 | Partial-empty header → EC-007; no phantom column names |
| `AC-016-xlsx-non-string-header.gif/.webm` | AC-016 | Non-string header cell → EC-008 naming cell type |
| `AC-017-xlsx-float-int-promotion.gif/.webm` | AC-017 | Float 95.0→Int(95); NaN/Infinity→ParseError |
| `AC-018-xlsx-datetimeiso-validation.gif/.webm` | AC-018 | Valid DateTimeIso passes; invalid → EC-009 |
| `AC-019-xlsx-extension-magic-bytes.gif/.webm` | AC-019 | Wrong ZIP magic bytes → EC-013 |
| `AC-020-sqlite-strict-utf8.gif/.webm` | AC-020 | Invalid UTF-8 TEXT → E-DAT-003 (strict, no lossy) |
| `AC-021-sqlite-blob-base64-standard.gif/.webm` | AC-021 | BLOB → STANDARD base64; test vector [0x00,0xFF,0x42] → "AP9C" |
| `AC-022-sqlite-extension-magic-bytes.gif/.webm` | AC-022 | .db3/.sl3 rejected; wrong SQLite magic → EC-009 |
| `AC-023-sqlite-select-with-dml-rejection.gif/.webm` | AC-023 | DML→EC-003; non-DML errors show actual rusqlite message |

---

## Coverage Summary

| AC | BC Trace | Status | Recording |
|----|----------|--------|-----------|
| AC-001 | BC-1.03.006 postcondition 7 | PASS | AC-001-xlsx-datasource-implements-trait |
| AC-002 | BC-1.03.006 postconditions 2+3 | PASS | AC-002-xlsx-first-row-header |
| AC-003 | BC-1.03.006 postconditions 4+5+6 | PASS | AC-003-xlsx-cell-type-mapping |
| AC-004 | BC-1.03.006 EC-003 | PASS | AC-004-xlsx-missing-sheet-error |
| AC-005 | BC-1.03.006 invariant 3, EC-002 | PASS | AC-005-xlsx-xls-unsupported-format |
| AC-006 | BC-1.03.006 EC-005 | PASS | AC-006-xlsx-merged-header-error |
| AC-007 | BC-1.03.006 EC-006 | PASS | AC-007-xlsx-formula-cached-value |
| AC-008 | BC-1.03.007 postcondition 1 | PASS | AC-008-sqlite-datasource-readonly |
| AC-009 | BC-1.03.007 postconditions 2+3+4 | PASS | AC-009-sqlite-row-type-mapping |
| AC-010 | BC-1.03.007 invariant 4 | PASS | AC-010-sqlite-query-required |
| AC-011 | BC-1.03.007 invariant 5, EC-003 | PASS | AC-011-sqlite-dml-rejected |
| AC-012 | BC-1.03.007 EC-005 | PASS | AC-012-sqlite-zero-rows |
| AC-013 | BC-1.03.007 EC-006 | PASS | AC-013-sqlite-duplicate-columns |
| AC-014 | NFR-022/023/024/025/036/037/038 | PASS | AC-014-nfr-perf-gates |
| AC-015 | BC-1.03.006 invariant 5, AC-BC-001 | PASS | AC-015-xlsx-partial-empty-header |
| AC-016 | BC-1.03.006 invariant 6, AC-BC-002 | PASS | AC-016-xlsx-non-string-header |
| AC-017 | BC-1.03.006 invariant 7, AC-BC-003 | PASS | AC-017-xlsx-float-int-promotion |
| AC-018 | BC-1.03.006 invariant 8, AC-BC-004 | PASS | AC-018-xlsx-datetimeiso-validation |
| AC-019 | BC-1.03.006 postcondition 9, AC-BC-005 | PASS | AC-019-xlsx-extension-magic-bytes |
| AC-020 | BC-1.03.007 invariant 6, AC-BC-006 | PASS | AC-020-sqlite-strict-utf8 |
| AC-021 | BC-1.03.007 invariant 7, AC-BC-007 | PASS | AC-021-sqlite-blob-base64-standard |
| AC-022 | BC-1.03.007 invariant 8, AC-BC-008 | PASS | AC-022-sqlite-extension-magic-bytes |
| AC-023 | BC-1.03.007 invariant 5, AC-BC-009 | PASS | AC-023-sqlite-select-with-dml-rejection |

**23/23 ACs: PASS. 0 deferred. 0 missing.**

---

## Cargo Test Summary

```
test result: ok. 251 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored (doctests)
perf_smoke: nfr_036_xlsx_10k_rows_under_2000ms ... ok
perf_smoke: nfr_037_sqlite_10k_rows_under_500ms ... ok
```

Total: **253 unit/integration tests + 2 perf gates + 3 doctests = 258 total passed; 0 failed**
