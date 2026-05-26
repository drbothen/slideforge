---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-018
title: "DataSource: JSON/CSV/YAML/TOML File Loading"
epic: EPIC-05
wave: 3
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-data
subsystems: [SS-10]
target_module: slideforge-data
behavioral_contracts: [BC-1.03.001, BC-1.03.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-001
  - STORY-011
blocks:
  - STORY-019
  - STORY-020
  - STORY-021
estimated_days: 2
---

# STORY-018: DataSource: JSON/CSV/YAML/TOML File Loading

## Summary

Implement the `DataSource` trait for local file-based data loading in the `slideforge-data`
crate. Supported formats: JSON, CSV, YAML, and TOML. Format is determined by file extension
(`.json`, `.csv`, `.yaml` / `.yml`, `.toml`). Loaded data is represented as the `Value` IR
type from `slideforge-types`. Missing field access on loaded data produces `E-DAT-005`
(implemented via the evaluator integration, but the field-access contract — BC-1.03.003 —
is verified here with unit tests on the data loader's return shape). This story also covers
YAML anti-coercion: `YES`, `on`, `no`, `false` YAML 1.1 booleans must remain as strings
(DI-004, BC-1.02.003).

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4,000 |
| `crates/slideforge-data/src/` files to write | ~5,000 |
| Test fixtures + test code | ~3,000 |
| Cargo.toml | ~600 |
| BC files consulted (BC-1.03.001, BC-1.03.003) | ~2,000 |
| **Total** | **~14,600** |

Agent context budget: 200k tokens. This story is ~7.3% of budget — within limit.

## Acceptance Criteria

- [ ] **AC-001:** `DataSource` trait is implemented for `FileDataSource` in `slideforge-data`. The trait method signature is:
  ```rust
  fn load(&self, ctx: &DataSourceContext) -> Result<Value, DataError>;
  ```
  `FileDataSource` holds `path: Arc<str>` and delegates to format-specific parsers based on file extension.
  (traces to BC-1.03.001 postcondition 1 — data parsed into typed value tree)

- [ ] **AC-002:** JSON files (`.json`) are parsed via `serde_json = "=1.0.140"` into `Value`. JSON objects become `Value::Map(IndexMap<Arc<str>, Value>)`. JSON arrays become `Value::List(Vec<Value>)`. JSON `null` becomes `Value::Null`. JSON booleans become `Value::Bool`. JSON integers become `Value::Int(i64)`. JSON floats become `Value::Float(OrderedFloat<f64>)`.
  (traces to BC-1.03.001 postcondition 1 — typed value tree)

- [ ] **AC-003:** CSV files (`.csv`) are parsed via `csv = "=1.3.1"` into `Value::List(Vec<Value::Map>)`. The first row is treated as the header. Each subsequent row produces a `Value::Map` keyed by header fields. Duplicate column headers produce `E-DAT-003` with message: `CSV has duplicate column header '<name>'`.
  (traces to BC-1.03.001 invariant 3 — CSV rows as list of maps keyed by header row)

- [ ] **AC-004:** YAML files (`.yaml` or `.yml`) are parsed via `serde_yaml_ng = "=0.10.0"` into `Value`. YAML 1.1 special values (`YES`, `yes`, `on`, `ON`, `no`, `NO`, `off`, `OFF`) are loaded as `Value::Str` strings — NOT booleans. Only the YAML 1.2 booleans `true` / `false` (lowercase) are accepted as `Value::Bool`.
  (traces to BC-1.03.001 edge case EC-004 — DI-004 no coercion; BC-1.02.003 no implicit type coercion)

- [ ] **AC-005:** TOML files (`.toml`) are parsed via `toml = "=0.8.19"` into `Value`. TOML integer → `Value::Int(i64)`. TOML float → `Value::Float(OrderedFloat<f64>)`. TOML boolean → `Value::Bool`. TOML string → `Value::Str`. TOML datetime → `Value::Str` (ISO 8601 representation).
  (traces to BC-1.03.001 postcondition 1 — typed value tree)

- [ ] **AC-006:** A missing file at the declared path produces `DataError::FileNotFound { path: Arc<str>, span: SourceSpan }` which maps to error code `E-DAT-004`. The error includes the fully resolved path and source span.
  (traces to BC-1.03.001 edge case EC-001 — E-DAT-004 with resolved path and source span)

- [ ] **AC-007:** A file with a malformed body (extension says `.json` but content is invalid JSON) produces `DataError::ParseError { path: Arc<str>, format: DataFormat, reason: Arc<str>, span: SourceSpan }` which maps to `E-DAT-003`. Example: `E-DAT-003: cannot parse 'data.json' as JSON: <serde_json error>`.
  (traces to BC-1.03.001 edge case EC-002 — E-DAT-003 for malformed content)

- [ ] **AC-008:** JSON `null` values in loaded data are preserved as `Value::Null`. They are NOT substituted with empty string, zero, or any other value. A `{{ data.field }}` interpolation that resolves to `Value::Null` produces the string `"null"` or uses `| default` filter (evaluated by the expr evaluator, not the data loader).
  (traces to BC-1.03.001 invariant 2 — JSON null fields preserved as null)

- [ ] **AC-009:** All `@data` sources are resolved before any slide evaluation begins. The `DataSourceContext` passed to `load()` is constructed during the pre-evaluation stage, before the evaluator processes any slide. Loading is sequential and blocking (no async).
  (traces to BC-1.03.001 invariant 1 — all @data sources resolved before slide evaluation)

- [ ] **AC-010:** `DataError::FieldNotFound { path: Arc<str>, source_name: Arc<str>, span: SourceSpan }` exists in `DataError` and maps to `E-DAT-005`. This error is produced by the evaluator's field access resolver (STORY-011), but the `slideforge-data` crate must export this error variant for use across the crate boundary.
  (traces to BC-1.03.003 postcondition 1 — E-DAT-005 emitted for missing field access)

- [ ] **AC-011:** `#![forbid(unsafe_code)]` is present on the crate root (NFR-024). `#![warn(missing_docs)]` is present; all public items have rustdoc (NFR-023). `cargo clippy -p slideforge-data -- -D warnings` is clean (NFR-022). All production deps use `=` version pinning (NFR-025).

- [ ] **AC-012:** An empty JSON object (`{}`) or empty JSON array (`[]`) is loaded without error and produces `Value::Map(IndexMap::new())` or `Value::List(vec![])` respectively. Field access on an empty collection produces `E-DAT-005`, not a panic.
  (traces to BC-1.03.001 edge case EC-005 — empty file is no error at load time)

## Previous Story Intelligence

This is the first story in EPIC-05 (Data Sources). Key lessons from Wave 1/2 pattern:

- Use `=` version pinning on ALL production deps — no exceptions.
- `Value` types come from `slideforge-types` (STORY-001). Do not redefine them.
- YAML serde requires custom deserialization to intercept YAML 1.1 boolean coercion. The `serde_yaml_ng` crate (maintained fork of the deprecated `serde_yaml`) uses YAML 1.2 semantics by default (so `yes`/`on` stay as strings). Verify this behavior in tests — do not assume.
- CSV headers may contain whitespace — trim before using as map keys.

## Architecture Compliance Rules

Sourced from `architecture/module-decomposition.md` and `architecture/dependency-graph.md`:

1. **SS-10 (Data Sources) — Effectful classification:** `slideforge-data` is Effectful (reads from filesystem and network). It MUST NOT call into Pure Core crates except `slideforge-types`. It MUST NOT call into `slideforge-eval`, `slideforge-layout`, or any exporter crate.
2. **`DataSource` trait:** The trait is defined in `slideforge-plugin-api` (STORY-002). `slideforge-data` implements it. The trait signature must not be changed in this story — if the signature from STORY-002 differs from what is written here, the STORY-002 definition wins.
3. **Error type owns `DataError`:** All data-loading errors are `DataError` variants in `slideforge-data::error`. The evaluator re-wraps them at the `EvalError` boundary.
4. **Forbidden dependencies:** `slideforge-data` MUST NOT depend on `slideforge-syntax`, `slideforge-eval`, `slideforge-validate`, `slideforge-layout`, `slideforge-pptx`, `slideforge-pdf`, `slideforge-html`, or `slideforge-cli`. Build will fail if such a dep appears.
5. **`IndexMap` for maps:** All `Value::Map` types use `indexmap::IndexMap<Arc<str>, Value>`. Never `std::collections::HashMap`.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `serde` | `=1.0.217` | Deserialization framework (with `derive` feature) |
| `serde_json` | `=1.0.140` | JSON parsing |
| `csv` | `=1.3.1` | CSV parsing with header support |
| `serde_yaml_ng` | `=0.10.0` | YAML parsing (maintained fork of deprecated serde_yaml; YAML 1.2 semantics — avoids 1.1 boolean coercion by default) |
| `toml` | `=0.8.19` | TOML parsing |
| `thiserror` | `=2.0.18` | `DataError` derive |
| `slideforge-types` | workspace | `Value`, `SourceSpan`, `Arc<str>` types |
| `slideforge-plugin-api` | workspace | `DataSource` trait, `DataSourceContext` |
| `indexmap` | `=2.2` | `IndexMap` for `Value::Map` construction |
| `ordered-float` | `=4.2` | `OrderedFloat<f64>` for float values |

Dev dependencies:
- `tempfile = "=3.10"` (for test fixtures without real file paths)
- `insta = "=1.39.0"` (snapshot tests on parsed value shape)

## File Structure Requirements

Files to create:

```
crates/slideforge-data/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # crate root; pub use; #![forbid(unsafe_code)]
│   ├── error.rs                # DataError enum (FileNotFound, ParseError, FieldNotFound, ...)
│   ├── format.rs               # DataFormat enum (Json, Csv, Yaml, Toml) + extension detection
│   ├── file.rs                 # FileDataSource struct implementing DataSource
│   ├── parse/
│   │   ├── mod.rs              # pub use json, csv, yaml, toml parsers
│   │   ├── json.rs             # parse_json(bytes: &[u8]) -> Result<Value, DataError>
│   │   ├── csv.rs              # parse_csv(bytes: &[u8]) -> Result<Value, DataError>
│   │   ├── yaml.rs             # parse_yaml(bytes: &[u8]) -> Result<Value, DataError>
│   │   └── toml.rs             # parse_toml(bytes: &[u8]) -> Result<Value, DataError>
│   └── context.rs              # DataSourceContext struct (path root, offline flag)
```

Files NOT to modify: Anything in other crates.

## Tasks

1. **Create `crates/slideforge-data/Cargo.toml`** with edition 2024, resolver 3, all pinned deps listed above. (15 min)
2. **Write `src/error.rs`** — `DataError` enum with variants: `FileNotFound`, `ParseError`, `FieldNotFound`, `NetworkError` (placeholder for STORY-019), `SsrfBlocked` (placeholder for STORY-019), `UnsupportedFormat`. Implement `thiserror::Error` on each. Include error code constants as doc comments. (20 min)
3. **Write `src/format.rs`** — `DataFormat` enum (`Json`, `Csv`, `Yaml`, `Toml`, `Xlsx`, `Sqlite`). Implement `DataFormat::from_extension(ext: &str) -> Option<DataFormat>`. (15 min)
4. **Write `src/parse/json.rs`** — `parse_json(bytes: &[u8], span: SourceSpan) -> Result<Value, DataError>`. Use `serde_json::from_slice` → convert `serde_json::Value` to `slideforge_types::Value`. Handle null, bool, int, float, string, array, object. (30 min)
5. **Write `src/parse/csv.rs`** — `parse_csv(bytes: &[u8], span: SourceSpan) -> Result<Value, DataError>`. Use `csv::Reader`. First row = headers. Detect duplicate headers → `DataError::ParseError` with `E-DAT-003` message. Trim header whitespace. (30 min)
6. **Write `src/parse/yaml.rs`** — `parse_yaml(bytes: &[u8], span: SourceSpan) -> Result<Value, DataError>`. Use `serde_yaml_ng::from_slice`. Verify YAML 1.1 strings stay as strings (write tests). Note: `serde_yaml_ng 0.10` uses `serde_yaml_ng::Value::String` for `yes`/`no` — confirm in tests. (30 min)
7. **Write `src/parse/toml.rs`** — `parse_toml(bytes: &[u8], span: SourceSpan) -> Result<Value, DataError>`. Use `toml::from_str`. Convert `toml::Value` to `slideforge_types::Value`. Datetime → ISO 8601 `Value::Str`. (20 min)
8. **Write `src/file.rs`** — `FileDataSource` struct + `DataSource` impl. Resolve path, read bytes, dispatch to format-specific parser. Produce `DataError::FileNotFound` on missing file. (25 min)
9. **Write `src/context.rs`** — `DataSourceContext { root_dir: PathBuf, offline: bool, span: SourceSpan }`. (10 min)
10. **Write `src/lib.rs`** — pub use, crate attributes. (10 min)
11. **Write `#[cfg(test)] mod tests`** in each parse module (see Test Strategy). (45 min)
12. **Run `cargo clippy -p slideforge-data -- -D warnings`** and fix all warnings. (15 min)
13. **Run `cargo test -p slideforge-data`** and confirm all tests pass. (10 min)

## Test Strategy

### Unit tests (in `#[cfg(test)] mod tests` in each parse module)

**`parse/json.rs` tests:**
- Happy path: parse `{"revenue": 42, "label": "Q1", "active": true, "score": 3.14, "items": [1, 2], "meta": null}` → verify each field maps to correct `Value` variant.
- Null preservation: parse `{"x": null}` → `Value::Null`, NOT `Value::Str("".into())`.
- Malformed JSON: parse `{broken` → `DataError::ParseError` with `E-DAT-003` message containing "JSON".
- Empty object: parse `{}` → `Value::Map(IndexMap::new())`, no error.
- Empty array: parse `[]` → `Value::List(vec![])`, no error.

**`parse/csv.rs` tests:**
- Happy path: `"name,revenue\nAlice,100\nBob,200"` → `Value::List` of 2 `Value::Map` entries, each with keys `name` and `revenue`.
- Duplicate headers: `"name,name\nAlice,Alice"` → `DataError::ParseError` with message containing "duplicate column header 'name'".
- Empty CSV (headers only): `"name,val\n"` → `Value::List(vec![])`, no error.
- Header whitespace trimming: `" name , val "` → keys are `"name"` and `"val"` (trimmed).

**`parse/yaml.rs` tests:**
- YAML 1.1 boolean strings preserved: parse `"flag: YES"` → `Value::Str("YES".into())` (NOT `Value::Bool(true)`).
- Parse `"flag: on"` → `Value::Str("on".into())`.
- Parse `"flag: no"` → `Value::Str("no".into())`.
- Parse `"flag: true"` → `Value::Bool(true)` (YAML 1.2 lowercase `true` is bool).
- Happy path: `"count: 5\nlabel: hello"` → `Value::Map` with `count: Value::Int(5)`, `label: Value::Str("hello")`.

**`parse/toml.rs` tests:**
- Happy path: `[section]\ncount = 5\nname = "hello"\nactive = true\nfloat = 1.5` → `Value::Map` with nested map for `section`.
- Datetime: `ts = 2024-01-01T00:00:00Z` → `Value::Str("2024-01-01T00:00:00Z")`.

**`file.rs` tests:**
- Missing file: construct `FileDataSource` with non-existent path → `DataError::FileNotFound`.
- Format dispatch: create temp files with `.json`, `.csv`, `.yaml`, `.toml` extensions, verify correct parser is invoked.

### Snapshot tests

Use `insta` to snapshot the `Value` tree output from a reference JSON fixture and a reference CSV fixture. These snapshots serve as regression tests.

## Dependencies

**Depends on:**
- STORY-001 (IR Core Types) — `Value`, `SourceSpan`, `Emu` types.
- STORY-011 (Expression Evaluator Core) — the evaluator calls `DataSource::load()` during pre-evaluation; this story provides the impl. STORY-011 may be written in parallel but must be compatible.

Dependency justification: STORY-018 depends on STORY-001 because `Value` and `SourceSpan` are the data types produced by the file loader and they are defined in `slideforge-types`. STORY-011 is listed as a co-dependency because the evaluator integration (BC-1.03.003 field-access errors) requires both the loader and the evaluator to agree on the `DataError::FieldNotFound` type.

**Blocks:**
- STORY-019 (HTTP DataSource) — builds on `FileDataSource` infrastructure; shares `DataError`, `DataFormat`, parse modules.
- STORY-020 (Excel + SQLite) — adds new parsers to the same `slideforge-data` crate structure.
- STORY-021 (offline mode) — adds offline gate logic to `DataSourceContext`; depends on `FileDataSource` + STORY-019's `HttpDataSource`.

## Implementation Notes

### `DataSource` Trait Signature (from STORY-002)

The trait is defined in `slideforge-plugin-api`. Do NOT redefine it here. The expected
signature (per STORY-002) is:

```rust
pub trait DataSource: Send + Sync {
    fn name(&self) -> &str;
    fn load(&self, ctx: &DataSourceContext) -> Result<Value, DataError>;
    fn supports_offline(&self) -> bool { false }
}
```

`FileDataSource::supports_offline()` returns `true` — file sources always load regardless
of `--offline` flag.

### JSON Integer Precision

`serde_json` represents all JSON numbers as either `i64` or `f64` depending on the value.
When converting to `Value`:
- If `as_i64()` succeeds → `Value::Int(n)`.
- If `as_f64()` succeeds (failed `as_i64()`) → `Value::Float(OrderedFloat(f))`.
- Never lose precision silently.

### YAML Anti-Coercion Implementation

`serde_yaml_ng 0.10` (used here) does NOT automatically coerce `YES`/`no`/`on`/`off` to booleans.
It uses YAML 1.2 semantics. However, write explicit tests confirming this behavior so that
if the dep is ever upgraded to a version that changes semantics, CI catches the regression.

serde_yaml_ng is the maintained fork of the deprecated serde_yaml crate. API is identical.

### CSV Empty Value Handling

Empty CSV cells (e.g., `Alice,,Boston`) produce `Value::Null` (not `Value::Str("")`).
This is consistent with JSON null preservation behavior across all formats.

### Error Code Constants

Define these as module-level constants in `error.rs` for documentation consistency:

```rust
pub const E_DAT_001: &str = "E-DAT-001"; // HTTP non-2xx response
pub const E_DAT_002: &str = "E-DAT-002"; // network unreachable
pub const E_DAT_003: &str = "E-DAT-003"; // parse error / format error
pub const E_DAT_004: &str = "E-DAT-004"; // file not found
pub const E_DAT_005: &str = "E-DAT-005"; // missing field access
pub const E_DAT_006: &str = "E-DAT-006"; // SSRF domain blocked
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist at declared path | `DataError::FileNotFound` → E-DAT-004 with resolved path and span |
| EC-002 | `.json` file with content that is valid YAML but not JSON | `DataError::ParseError` → E-DAT-003 with "cannot parse as JSON" |
| EC-003 | CSV with duplicate column headers | `DataError::ParseError` → E-DAT-003 with "CSV has duplicate column header '<name>'" |
| EC-004 | YAML file uses `YES`, `on`, `no` (YAML 1.1 booleans) | Loaded as `Value::Str("YES")` — DI-004, no coercion |
| EC-005 | Empty JSON `{}` or `[]` | `Value::Map(IndexMap::new())` or `Value::List(vec![])` — no error |
| EC-006 | JSON null field | `Value::Null` — never converted to empty string |
| EC-007 | CSV with only a header row, no data rows | `Value::List(vec![])` — no error |
| EC-008 | File extension `.yml` (not `.yaml`) | Parsed as YAML — both extensions are accepted |
| EC-009 | File with unknown extension (e.g., `.txt`) | `DataError::UnsupportedFormat` → E-DAT-003 with hint listing supported extensions |
| EC-010 | TOML datetime value | Converted to ISO 8601 `Value::Str` |
