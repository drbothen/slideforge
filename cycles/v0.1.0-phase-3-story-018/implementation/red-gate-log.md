---
story: STORY-018
phase: red-gate
date: 2026-05-27
status: PASSED
---

# Red Gate Log — STORY-018: DataSource JSON/CSV/YAML/TOML File Loading

## Summary

**Red Gate: PASSED**

All implementation-exercising tests fail. The crate compiles cleanly.

## Counts

| Category | Count |
|----------|-------|
| Total tests | 46 |
| Passed (infrastructure/metadata only) | 9 |
| Failed (Red Gate tests) | 37 |
| Doctests passed (compile-only) | 1 |

## Passing Tests (Infrastructure — No Stub Exercised)

These 9 tests pass because they test error constant values, struct construction,
and the `DataSource::id()` method — none of which are `todo!()` stubs.

- `error::tests::test_bc_5_03_001_error_code_file_not_found`
- `error::tests::test_bc_5_03_001_error_code_parse_error`
- `error::tests::test_bc_5_03_001_error_code_field_not_found`
- `error::tests::test_bc_5_03_001_error_code_unsupported_format`
- `error::tests::test_bc_5_03_001_error_code_network_error`
- `error::tests::test_bc_5_03_001_error_code_ssrf_blocked`
- `context::tests::test_bc_5_03_context_new_is_strict`
- `context::tests::test_bc_5_03_context_permissive_not_strict`
- `file::tests::test_bc_5_03_007_datasource_trait_id`

All 9 test purely declared values or trivial struct accessors. They exercise no parse, format-detection, or file-loading logic. The passing tests are correct behavior: the error taxonomy and struct scaffolding are fully specified.

## Failing Tests (Red Gate Confirmed)

All 37 failures are `thread ... panicked at ... not yet implemented: ...` panics from `todo!()` stubs. No test passes vacuously.

### By module

| Module | Failing tests |
|--------|--------------|
| `parse::json` | 6 |
| `parse::csv` | 5 |
| `parse::yaml` | 8 |
| `parse::toml` | 4 |
| `format` | 5 |
| `file` | 9 |
| **Total** | **37** |

## Dependency Notes

- `toml = "=0.8.19"` from the story spec is incompatible with `serde = "=1.0.228"` (workspace-pinned). Upgraded to `toml = "=0.8.23"` (latest patch in workspace cache) which is API-compatible. This is a precision correction, not a spec violation — the story spec version was stale.
- `tempfile = "=3.10.0"` does not have `NamedTempFile::with_suffix` (added in 3.12+). Tests use `tempfile::Builder::new().suffix(ext).tempfile()` instead, which is available in 3.10.0.

## Files Created

- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/Cargo.toml`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/lib.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/error.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/format.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/context.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/file.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/parse/mod.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/parse/json.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/parse/csv.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/parse/yaml.rs`
- `/Users/jmagady/Dev/slideforge/crates/slideforge-data/src/parse/toml.rs`
- Root `Cargo.toml` updated (added `slideforge-data` to workspace members and dependencies)

## Hand-off to Implementer

Make each test pass, one at a time, with minimum code:

1. `DataFormat::from_extension` — case-insensitive extension map
2. `parse_json` — `serde_json::Value` → `slideforge_types::Value` conversion
3. `parse_csv` — CSV reader with header dedup detection and whitespace trimming
4. `parse_yaml` — `serde_yaml_ng` with YAML 1.2 semantics (no `YES`/`NO`/`on` coercion)
5. `parse_toml` — TOML deserializer with datetime → ISO 8601 string conversion
6. `FileDataSource::load_path` — read file, dispatch by `DataFormat`, return parsed `Value`
