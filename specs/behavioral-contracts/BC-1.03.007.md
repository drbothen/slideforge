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

# BC-1.03.007: Load @data from SQLite Database via Parameterized Query at Compile Time

## Description

The `@data name from "path.db" query: "SELECT ..."` directive loads data from a local
SQLite database at compile time by executing a provided SQL query. The result set rows
are loaded as a list of maps keyed by column names. The query must be a read-only
SELECT statement; DML (INSERT, UPDATE, DELETE) and DDL are rejected at parse time.
This implements the SQLite variant of the DataSource plugin surface per CAP-003.

## Preconditions

1. A `@data name from "path.db" query: "SELECT ..."` directive appears in the deck source.
2. The file at `path` is readable and is a valid SQLite database (`.db`, `.sqlite`, or `.sqlite3` extension).
3. The `query:` field contains a valid SQL SELECT statement.

## Postconditions

1. The SELECT query is executed against the database with read-only connection flags
   (`SQLITE_OPEN_READONLY`).
2. Result set rows are loaded as a list of maps keyed by column names (or aliases if
   aliased in the SELECT).
3. NULL database values are represented as null in the data value tree.
4. Integer, real, text, and blob columns are mapped to integer, float, string, and
   base64-encoded string respectively.
5. The data is bound to `name` in the deck scope; `{{ name }}`, `{{ name[0] }}`,
   and `{{ name[0].column_name }}` are valid.
6. Build exits 0 on success.

## Invariants

1. All `@data` sources are resolved before any slide evaluation begins.
2. The database connection is opened in read-only mode; any write attempt raises an error at
   the OS/SQLite driver level. (DI-004 — no mutation of data sources during build)
3. No implicit type coercion: SQLite types are mapped to DSL types as defined in postconditions. (DI-004)
4. The `query:` field is required for SQLite data sources — no default query exists.
5. Only SELECT statements are permitted. INSERT/UPDATE/DELETE/DROP/CREATE produce E-DAT-003.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist | E-DAT-004: file data source not found; exit 2 |
| EC-002 | File exists but is not a valid SQLite database | E-DAT-003: cannot open '<path>' as SQLite database: <sqlite-error> |
| EC-003 | Query contains UPDATE/INSERT/DELETE | E-DAT-003: only SELECT queries are allowed in @data sqlite sources |
| EC-004 | Query references a non-existent table | E-DAT-003: SQLite error: no such table: '<table>' (at <file>:<line>) |
| EC-005 | Query returns zero rows | Bound as empty list; no error at load time (access errors occur per BC-1.03.003) |
| EC-006 | Query has duplicate column names (no alias) | E-DAT-003: SELECT result has duplicate column name '<name>'; use aliases |
| EC-007 | Missing query: field | E-PAR-NNN: @data sqlite source requires query: field at <file>:<line> |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data metrics from "metrics.db" query: "SELECT name, value FROM metrics"` (3 rows) | metrics bound as list of 3 maps with keys name, value | happy-path |
| `@data d from "data.db" query: "SELECT * FROM events WHERE status = 'open'"` | Filtered rows loaded; null columns as null | happy-path |
| `@data d from "missing.db" query: "SELECT 1"` | E-DAT-004; exit 2 | error |
| `@data d from "data.db" query: "DELETE FROM events"` | E-DAT-003: DML not allowed; exit 2 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | SQLite connection opened with SQLITE_OPEN_READONLY | unit test: mock db; attempt write inside query; assert connection error not silent success |
| VP-TBD | Result row column names become map keys | unit test: fixture db; run SELECT; assert map keys match column names |
| VP-TBD | DML in query: field produces E-DAT-003 | unit test: assert error code on DELETE/INSERT/UPDATE |
| VP-TBD | NULL db values produce null in data value tree | unit test: fixture row with NULL column; assert null in loaded map |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — "Load structured data at compile time from JSON files, CSV files, YAML files, TOML files, HTTP/HTTPS URLs, Excel spreadsheets, and SQLite databases" explicitly names SQLite databases as a supported source |
| L2 Domain Invariants | DI-004 (no implicit type coercion — SQLite type mapping is explicit and deterministic) |
| Architecture Module | slideforge-eval crate — DataSource plugin, sqlite variant (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.03.001 — related to (file-based data source variant; sqlite uses the same plugin surface)
- BC-1.03.006 — related to (XLSX is the other structured data source added in parallel with SQLite)
- BC-1.03.003 — composes with (missing-field error contract applies to sqlite-loaded data too)
- BC-1.02.001 — depends on (expression evaluation uses the sqlite-loaded data)

## Architecture Anchors

- `architecture/authoring-subsystem.md#data-binding` — DataSource plugin surface

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
