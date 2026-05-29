---
document_type: behavioral-contract
level: L3
version: "1.3.1"
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
    reason: "Initial draft with basic SQLite contract"
  - version: "1.3"
    date: 2026-05-28
    reason: "Adversary pass 1 adjudications: strict UTF-8 decode (E), base64 STANDARD canonical (F), extension+magic-byte policy (H), DML rejection error differentiation (I). All decisions concrete per canonical principle."
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

TEXT columns are decoded as strict UTF-8 (via `std::str::from_utf8` — not
`String::from_utf8_lossy`); invalid UTF-8 sequences produce ParseError rather than
silently substituting U+FFFD replacement characters. BLOB columns are base64-encoded
using the STANDARD encoding with padding (RFC 4648 §4). Extensions `.db`, `.sqlite`,
and `.sqlite3` are accepted; other extensions are rejected as unsupported format.
Magic-byte detection additionally validates that accepted-extension files are genuine
SQLite databases (magic string `"SQLite format 3\000"` at byte offset 0).

## Preconditions

1. A `@data name from "path.db" query: "SELECT ..."` directive appears in the deck source.
2. The file at `path` is readable and has a `.db`, `.sqlite`, or `.sqlite3` extension.
3. The `query:` field contains a valid SQL SELECT statement.

## Postconditions

1. The SELECT query is executed against the database with read-only connection flags
   (`SQLITE_OPEN_READONLY`).
2. Result set rows are loaded as a list of maps keyed by column names (or aliases if
   aliased in the SELECT).
3. NULL database values are represented as null in the data value tree.
4. Integer, real, text, and blob columns are mapped as follows:
   - `ValueRef::Integer(n)` → `Value::Int(n)`
   - `ValueRef::Real(f)` → `Value::Float(OrderedFloat(f))`
   - `ValueRef::Text(bytes)`: decoded via `std::str::from_utf8(bytes)`. If `Ok(s)` →
     `Value::Str(Arc::from(s))`. If `Err` → `DataError::ParseError` per EC-008.
   - `ValueRef::Blob(bytes)`: encoded via `base64::engine::general_purpose::STANDARD.encode(bytes)`
     (RFC 4648 §4, with `=` padding). Result → `Value::Str(Arc::from(encoded))`.
5. The data is bound to `name` in the deck scope; `{{ name }}`, `{{ name[0] }}`,
   and `{{ name[0].column_name }}` are valid.
6. Build exits 0 on success.
7. The file has one of the accepted extensions (`.db`, `.sqlite`, `.sqlite3`) AND passes
   SQLite magic-byte detection: the first 16 bytes begin with `b"SQLite format 3\0"`.
   A file with accepted extension but failing magic-byte check produces `DataError::ParseError`
   per EC-009.

## Invariants

1. All `@data` sources are resolved before any slide evaluation begins.
2. The database connection is opened in read-only mode; any write attempt raises an error at
   the OS/SQLite driver level. (DI-004 — no mutation of data sources during build)
3. No implicit type coercion: SQLite types are mapped to DSL types as defined in postconditions. (DI-004)
4. The `query:` field is required for SQLite data sources — no default query exists.
5. Only SELECT statements are permitted. INSERT/UPDATE/DELETE/DROP/CREATE produce E-DAT-003.
   DML detection uses an explicit SELECT-prefix check (case-insensitive, trimming leading
   whitespace) for the dedicated DML-rejection path. Other query failures (e.g., table does
   not exist) pass through as the actual rusqlite error message, not the generic DML message.
6. TEXT column bytes are decoded with strict UTF-8 (`std::str::from_utf8`). The lossy
   variant (`String::from_utf8_lossy`) is FORBIDDEN for TEXT column decoding because it
   silently substitutes U+FFFD replacement characters, violating the canonical principle
   (no silent fallback). (CLAUDE.md canonical principle; DI-004)
7. BLOB columns are encoded using `base64::engine::general_purpose::STANDARD` (RFC 4648 §4,
   alphabet A-Za-z0-9+/, with `=` padding to 4-character boundary). This encoding is
   canonical and stable — downstream consumers can rely on it. Alternative engines
   (URL_SAFE, NO_PAD, etc.) MUST NOT be used for BLOB encoding.
8. Extension validation accepts exactly: `.db`, `.sqlite`, `.sqlite3` (case-insensitive).
   All other extensions (including `.db3`, `.s3db`, `.sl3`) produce `DataError::UnsupportedFormat`.
   Magic-byte check additionally validates the first 16 bytes against the SQLite file header
   magic string `b"SQLite format 3\0"`. Both checks apply.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist | E-DAT-004: file data source not found; exit 2 |
| EC-002 | File exists but is not a valid SQLite database (rusqlite error) | E-DAT-003: cannot open '<path>' as SQLite database: <sqlite-error> |
| EC-003 | Query text starts with INSERT/UPDATE/DELETE/DROP/CREATE (case-insensitive, after trimming whitespace) | E-DAT-003: only SELECT queries are allowed in @data sqlite sources (at <file>:<line>); exit 2 |
| EC-004 | Query references a non-existent table | E-DAT-003: SQLite error: no such table: '<table>' (at <file>:<line>). Pass-through of actual rusqlite error. |
| EC-005 | Query returns zero rows | Bound as empty list; no error at load time (access errors occur per BC-1.03.003) |
| EC-006 | Query has duplicate column names (no alias) | E-DAT-003: SELECT result has duplicate column name '<name>'; use aliases |
| EC-007 | Missing query: field | E-PAR-NNN: @data sqlite source requires query: field at <file>:<line> |
| EC-008 | TEXT column contains bytes that are not valid UTF-8 | E-DAT-003: TEXT column '<column_name>' at row <row_idx> in '<path>' contains invalid UTF-8 bytes. SQLite TEXT values must be valid UTF-8.; exit 2 |
| EC-009 | File has accepted extension (.db/.sqlite/.sqlite3) but fails SQLite magic-byte check | E-DAT-003: '<path>' has .<ext> extension but is not a valid SQLite database (SQLite file header not found). File may be corrupted or misnamed.; exit 2 |
| EC-010 | Extension is not in accepted set (e.g., `.db3`, `.s3db`, or no extension) | E-DAT-003: unsupported extension for SQLite data source: '<ext>'. Accepted extensions: .db, .sqlite, .sqlite3; exit 2 |
| EC-011 | BLOB column — any byte sequence | Value::Str containing base64 STANDARD (RFC 4648 §4) encoding with padding. Downstream consumers can decode with any RFC 4648 §4 decoder. |
| EC-012 | Non-SELECT query reaches SqliteDataSource::load() (defense-in-depth path) | Read-only connection flag causes rusqlite to reject write; additionally, SELECT-prefix check triggers E-DAT-003 per EC-003 message. The SELECT-prefix check fires first. |

## Acceptance Criteria (Adjudicated v1.3 — Adversary Pass 1)

The following ACs supplement the story-level ACs. These are binding BC-level requirements.

**AC-BC-006 (Item E — Strict UTF-8 decode):**
`rusqlite::types::ValueRef::Text(bytes)` MUST be decoded via `std::str::from_utf8(bytes)`,
NOT `std::str::from_utf8_lossy(bytes)` or `String::from_utf8_lossy(bytes)`.
On `Err` → `DataError::ParseError` with message conforming to EC-008. On `Ok(s)` →
`Value::Str(Arc::from(s))`.
Rationale: `from_utf8_lossy` silently substitutes U+FFFD for invalid bytes, violating
the canonical principle (no silent fallback) and DI-004 (no implicit coercion). A user
storing non-UTF-8 bytes in a TEXT column that they expect to retrieve cleanly would
receive silently corrupted data with no indication of the problem. The error forces
the user to fix their data.

**AC-BC-007 (Item F — Base64 STANDARD canonical encoding):**
`rusqlite::types::ValueRef::Blob(bytes)` MUST be encoded using exactly:
`base64::engine::general_purpose::STANDARD.encode(bytes)` (from the `base64` crate,
using the `Engine` trait). This engine uses the standard A-Za-z0-9+/ alphabet with
`=` padding characters to align to 4-character blocks (RFC 4648 §4).
Alternative engines (`URL_SAFE`, `STANDARD_NO_PAD`, `URL_SAFE_NO_PAD`) MUST NOT be used.
The encoded string becomes a `Value::Str`. Downstream consumers (templates, exporters,
test vectors) can rely on this encoding being stable across all builds and versions.

**AC-BC-008 (Item H — Extension + magic-byte policy for SQLite):**
Extension check is performed first (before opening file):
1. If path extension is not one of `.db`, `.sqlite`, `.sqlite3` (case-insensitive) →
   `DataError::UnsupportedFormat` per EC-010. Do NOT attempt to open the file.
2. If extension is in the accepted set, open the file and read the first 16 bytes.
   Check if they equal `b"SQLite format 3\0"` (the canonical SQLite file header magic).
   - If match fails → `DataError::ParseError` per EC-009.
   - If match succeeds → proceed with rusqlite open.
This two-phase check prevents rusqlite from producing confusing internal errors for
corrupt or misnamed files, and maintains symmetry with XlsxDataSource's approach
(BC-1.03.006 AC-BC-005).

**AC-BC-009 (Item I — DML rejection error message differentiation):**
The DML-rejection check MUST be implemented as an explicit SELECT-prefix check:
1. Trim leading whitespace from the query string.
2. Check if the trimmed query starts with `SELECT` or `WITH` (case-insensitive).
   (`WITH` is allowed as it is the CTE prefix for valid read-only queries.)
3. If the query does NOT start with `SELECT` or `WITH` → produce `DataError::ParseError`
   with message: `"only SELECT queries are allowed in @data sqlite sources"` (EC-003).
4. If the query starts with `SELECT` or `WITH`, proceed to `connection.prepare(query)`.
   Any error from `prepare()` or subsequent `query_map()` produces `DataError::ParseError`
   with the actual rusqlite error message (not the generic DML message). Example:
   `"SQLite error: no such table: 'events'"`.
The implementer MUST NOT wrap both prepare-failure and DML-keyword cases under the
same "only SELECT queries are allowed" error string. The error message must reflect
the actual cause. This was a confusion identified in adversary pass 1 (F-MED-2).

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `@data metrics from "metrics.db" query: "SELECT name, value FROM metrics"` (3 rows) | metrics bound as list of 3 maps with keys name, value | happy-path |
| `@data d from "data.db" query: "SELECT * FROM events WHERE status = 'open'"` | Filtered rows loaded; null columns as null | happy-path |
| `@data d from "missing.db" query: "SELECT 1"` | E-DAT-004; exit 2 | error |
| `@data d from "data.db" query: "DELETE FROM events"` | E-DAT-003: only SELECT queries are allowed; exit 2 | error (EC-003) |
| TEXT column with valid UTF-8 | Value::Str with decoded string | happy-path |
| TEXT column with invalid UTF-8 bytes | E-DAT-003: invalid UTF-8; exit 2 | error (EC-008) |
| BLOB column with bytes [0x00, 0xFF, 0x42] | Value::Str("AP9C") (base64 STANDARD encoding) | regression (EC-011) |
| File `data.db` that is actually a PNG (wrong magic bytes) | E-DAT-003: not a valid SQLite database; exit 2 | error (EC-009) |
| `@data d from "data.db3" query: "SELECT 1"` | E-DAT-003: unsupported extension .db3; exit 2 | error (EC-010) |
| Query `"SELECT id, id FROM t"` (duplicate column) | E-DAT-003: duplicate column name 'id'; exit 2 | error |
| Query `"INSERT INTO t VALUES (1)"` | E-DAT-003: only SELECT queries are allowed; exit 2 | error |
| Query `"SELECT * FROM nonexistent_table"` | E-DAT-003: SQLite error: no such table: 'nonexistent_table'; exit 2 | error (EC-004, not DML message) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-027 | SQLite connection opened with SQLITE_OPEN_READONLY | unit test: mock db; attempt write inside query; assert connection error not silent success |
| VP-028 | Result row column names become map keys | unit test: fixture db; run SELECT; assert map keys match column names |
| VP-029 | DML in query: field produces E-DAT-003 | unit test: assert error code on DELETE/INSERT/UPDATE |
| VP-030 | NULL db values produce null in data value tree | unit test: fixture row with NULL column; assert null in loaded map |
| VP-031 | TEXT column with valid UTF-8 produces Value::Str | unit test: in-memory db; insert UTF-8 text; assert Value::Str |
| VP-032 | TEXT column with invalid UTF-8 produces ParseError | unit test: insert raw bytes via BLOB-then-reinterpret or rusqlite raw value injection; assert E-DAT-003 |
| VP-033 | BLOB column produces base64 STANDARD with padding | unit test: insert BLOB [0x00, 0xFF, 0x42]; assert Value::Str("AP9C") — the base64 STANDARD encoding |
| VP-034 | File with .db extension but non-SQLite magic bytes produces ParseError | unit test: write [0x00,0x01,0x02,...] to .db file; assert E-DAT-003 EC-009 message |
| VP-035 | Extension .db3 produces UnsupportedFormat | unit test: pass .db3 path; assert E-DAT-003 EC-010 message |
| VP-036 | Non-existent table produces actual SQLite error, not DML message | unit test: query against missing table; assert message contains "no such table" not "only SELECT queries" |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 |
| Capability Anchor Justification | CAP-003 ("Data Binding from External Sources") per capabilities.md §CAP-003 — "Load structured data at compile time from JSON files, CSV files, YAML files, TOML files, HTTP/HTTPS URLs, Excel spreadsheets, and SQLite databases" explicitly names SQLite databases as a supported source |
| L2 Domain Invariants | DI-004 (no implicit type coercion — SQLite type mapping is explicit and deterministic; strict UTF-8 decode enforced) |
| Architecture Module | slideforge-data crate — DataSource plugin (SS-10) |
| Stories | STORY-020 |

## CHANGELOG

| Version | Date | Change |
|---------|------|--------|
| 1.1 | 2026-05-24 | Initial draft |
| 1.2 | 2026-05-24 | Minor clarifications (pre-adversary) |
| 1.3 | 2026-05-28 | Adversary Pass 1 adjudications (items E, F, H, I): strict UTF-8 decode; base64 STANDARD RFC 4648 §4 canonical; extension closed list (.db/.sqlite/.sqlite3) + magic-byte check; DML vs other-error message differentiation. Added EC-008 through EC-012, AC-BC-006 through AC-BC-009, VP expansions. |
| 1.3.1 | 2026-05-28 | VP propagation burst: assigned VP-027 through VP-036 to all VP-TBD entries. |

## Related BCs

- BC-1.03.001 — related to (file-based data source variant; sqlite uses the same plugin surface)
- BC-1.03.006 — related to (XLSX is the other structured data source added in parallel with SQLite)
- BC-1.03.003 — composes with (missing-field error contract applies to sqlite-loaded data too)
- BC-1.02.001 — depends on (expression evaluation uses the sqlite-loaded data)

## Architecture Anchors

- `architecture/system-overview.md` — DataSource plugin surface

## Story Anchor

STORY-020

## VP Anchors

VP-027, VP-028, VP-029, VP-030, VP-031, VP-032, VP-033, VP-034, VP-035, VP-036
