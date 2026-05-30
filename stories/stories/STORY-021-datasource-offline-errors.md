---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-021
title: "DataSource: --offline Flag + Error Handling"
epic: EPIC-05
wave: 3
points: 3
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-data
subsystems: [SS-10]
target_module: slideforge-data
behavioral_contracts: [BC-1.03.004]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-019
  - STORY-020
blocks:
  - STORY-055
  - STORY-056
estimated_days: 1
---

# STORY-021: DataSource: --offline Flag + Error Handling

## Summary

Implement the `--offline` flag gate in `slideforge-data`. When `DataSourceContext.offline`
is `true`, any `DataSource` implementation that returns `supports_offline() = true`
(i.e., HTTP sources from STORY-019) is skipped without making a network request. The
data name for the skipped source is simply not added to the deck scope, and any
`{{ name.field }}` references produce `E-EVL-001` (undefined variable) from the
evaluator. File-based sources (`FileDataSource`, `XlsxDataSource`, `SqliteDataSource`)
are always loaded regardless of the offline flag. This story also consolidates the
complete error-handling integration test suite for the `slideforge-data` crate.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2,500 |
| `crates/slideforge-data/src/dispatcher.rs` | ~1,500 |
| Integration test additions | ~2,500 |
| BC file consulted (BC-1.03.004) | ~1,000 |
| **Total** | **~7,500** |

Agent context budget: 200k tokens. This story is ~3.75% of budget — well within limit.

## Acceptance Criteria

- [ ] **AC-001:** A `DataSourceDispatcher` (or equivalent function `load_all_sources`) iterates over all `@data` directives in a `Deck`. For each source where `source.supports_offline() == true` AND `ctx.offline == true`, the source is skipped: no network request is made, and the source's data name is NOT added to the scope map.
  (traces to BC-1.03.004 postcondition 1 — no HTTP requests when offline; postcondition 2 — HTTP-bound names absent from scope)

- [ ] **AC-002:** When `ctx.offline == true` and an HTTP source is skipped, any `{{ name.field }}` expressions that reference the skipped data name produce `E-EVL-001` (undefined variable). This error is produced by the evaluator (STORY-011), not the data dispatcher. The data dispatcher's responsibility is only to NOT populate the scope entry.
  (traces to BC-1.03.004 postcondition 3 — skipped sources produce undefined-variable errors)

- [ ] **AC-003:** File-based sources (`FileDataSource`, `XlsxDataSource`, `SqliteDataSource`) are ALWAYS loaded regardless of the `offline` flag. Their `supports_offline()` returns `false`, meaning the offline flag has no effect on them.
  (traces to BC-1.03.004 invariant 2 — file sources always loaded regardless of --offline; postcondition 4)

- [ ] **AC-004:** When `ctx.offline == true` and the deck has zero HTTP sources (only file sources), behavior is identical to `offline == false`. Build completes normally.
  (traces to BC-1.03.004 edge case EC-001)

- [ ] **AC-005:** When `ctx.offline == true` and an HTTP source is declared but never referenced in any slide expression, no error is produced. The missing scope entry is never accessed, so no `E-EVL-001` is triggered.
  (traces to BC-1.03.004 edge case EC-002)

- [ ] **AC-006:** When `ctx.offline == true` and the deck has both file sources and HTTP sources, file sources load normally and HTTP sources are skipped. The scope contains data from file sources only.
  (traces to BC-1.03.004 edge case EC-003)

- [ ] **AC-007:** `slideforge watch --offline` behavior: the `DataSourceDispatcher` respects `ctx.offline = true` on every re-evaluation cycle. HTTP sources are not polled even in watch mode.
  (traces to BC-1.03.004 edge case EC-004)

- [ ] **AC-008:** `--offline` NEVER produces empty data silently. If a data name is skipped, it is simply absent from scope. There is no mechanism that substitutes an empty `Value::Map` or `Value::List` for a skipped HTTP source.
  (traces to BC-1.03.004 invariant 1 — --offline never silently produces empty data)

- [ ] **AC-009:** The `DataError` enum has all error codes (E-DAT-001 through E-DAT-006) represented as variants. The `Display` impl for each variant includes the error code prefix (e.g., `"E-DAT-004: file data source not found: {path}"`). Integration tests verify error code output.

- [ ] **AC-010:** `#![forbid(unsafe_code)]` (NFR-024), clippy clean (NFR-022), `=` version pinning (NFR-025) maintained across all changes in this story.

### BC Trace Clarification (Pass-5)

**AC-009 trace to BC-1.03.004:** AC-009 traces to BC-1.03.004 via the broader "dispatcher robustness" invariant. Error reporting (correct error-code assignment, Display format, and test coverage) is part of the dispatcher contract even though BC-1.03.004's enumerated postconditions focus on offline-gate semantics. AC-009 was added to this story for delivery coverage of the full error taxonomy for the data subsystem.

**AC-010 trace:** AC-010 (forbid unsafe, clippy, version pinning) does NOT trace to BC-1.03.004. It traces to NFR-021 through NFR-025 from the NFR catalog. It is included in the AC list here because it is a non-negotiable delivery gate for every story in the project, not because it is a behavioral property of the offline flag.

## Previous Story Intelligence

This story finalizes EPIC-05 (Data Sources). All four `DataSource` implementations
are complete after STORY-018, STORY-019, STORY-020. This story adds the dispatcher
and integration-level offline gate:

- `DataSourceContext.offline: bool` field was added in STORY-019.
- `HttpDataSource::supports_offline()` returns `true` from STORY-019.
- This story adds the dispatch loop that checks `supports_offline()` before calling `load()`.

The key architectural point: the offline gate lives in the dispatcher (data layer), NOT
in the CLI (STORY-055/056). The CLI simply sets `ctx.offline = true` from the `--offline`
flag. The data dispatcher enforces the gate.

## Architecture Compliance Rules

1. **Dispatcher is pure coordination logic:** The `DataSourceDispatcher` makes no I/O decisions itself; it delegates to each `DataSource` impl. The `supports_offline()` trait method is the coordination protocol.
2. **Scope population is the dispatcher's output:** The dispatcher returns a `HashMap<Arc<str>, Value>` (or `IndexMap`) of successfully loaded data. HTTP sources that were skipped are absent from this map. The evaluator receives this map.
3. **No `--offline` error:** The dispatcher does NOT emit any error for skipped HTTP sources. Skipped = absent. The error comes later, from the evaluator, when a `{{ name.field }}` references an absent binding.
4. **Forbidden dependencies:** Same as STORY-018 — no `slideforge-eval`, `slideforge-syntax`, exporter crates.

## Library and Framework Requirements

No new libraries required. All dependencies established in STORY-018, STORY-019, STORY-020.

## File Structure Requirements

Files to create:

```
crates/slideforge-data/src/
├── dispatcher.rs         # DataSourceDispatcher: load_all() → IndexMap<Arc<str>, Value>
└── tests/
    └── integration.rs    # Integration tests for offline gate and full error coverage
```

Files to modify:

```
crates/slideforge-data/src/
└── lib.rs                # Add: pub mod dispatcher; pub mod tests (cfg test)
```

## Tasks

1. **Write `src/dispatcher.rs`** — `DataSourceDispatcher` struct (or free function `load_all`). Input: `Vec<(Arc<str>, Box<dyn DataSource>)>` (name-source pairs), `DataSourceContext`. Output: `(IndexMap<Arc<str>, Value>, Vec<DataError>)`. For each source: if `ctx.offline && source.supports_offline()` → skip (do not add to map); else call `source.load(&ctx)` → on success, add to map; on error, add to error vec. Return both map and errors. (25 min)
2. **Write integration tests** in `tests/integration.rs` — see Test Strategy. (30 min)
3. **Run `cargo clippy -p slideforge-data -- -D warnings`** — fix all warnings. (10 min)
4. **Run `cargo test -p slideforge-data`** — all tests pass. (10 min)
5. **Final verification:** `cargo test -p slideforge-data --all-features -- --nocapture` to confirm all 30+ tests pass cleanly with descriptive output. (5 min)

## Test Strategy

### Integration tests in `tests/integration.rs`

Updated to reflect actual implementation (Pass-5, 2026-05-30).

| Test Name | Setup | Expected | Traces |
|-----------|-------|----------|--------|
| `test_bc_1_03_004_offline_skips_http_source` | Dispatcher with one `MockHttpSource` (supports_offline=true), `ctx.offline=true` | Scope map is empty; zero network calls | AC-001 |
| `test_bc_1_03_004_offline_loads_file_sources` | Dispatcher with `FileDataSource` (JSON) + `MockHttpSource`, `ctx.offline=true` | File source loaded normally; HTTP skipped | AC-003 |
| `test_bc_1_03_004_online_loads_all_sources` | Dispatcher with file source + `MockHttpSource` returning JSON, `ctx.offline=false` | Both loaded; scope map has 2 entries | AC-001 (inverse) |
| `test_bc_1_03_004_offline_zero_http_sources` | Dispatcher with only file sources, `ctx.offline=true` | All file sources loaded; no change in behavior | AC-004 / EC-001 |
| `test_bc_1_03_004_offline_with_file_and_http` | `MockFileSource` + `MockHttpSource`, `ctx.offline=true` | File source loaded; HTTP source absent from scope; zero errors | AC-006 / EC-003 |
| `test_bc_1_03_004_offline_unreferenced_http_source` | Skipped HTTP source; evaluator not involved | No error from dispatcher | AC-005 |
| `test_bc_1_03_004_offline_watch_mode_repeated_dispatch` | Repeated load_all with offline=true | Consistent results across calls | AC-007 |
| `test_bc_1_03_004_zero_sources_returns_empty_scope_zero_errors` | Empty source list | Empty scope, zero errors | EC-006 |
| `test_bc_1_03_004_error_code_dat_004_file_not_found` | `FileDataSource` with missing file | `DataError::FileNotFound`; Display contains "E-DAT-004" | AC-009 |
| `test_bc_1_03_004_error_code_dat_006_ssrf_blocked` | `HttpDataSource` with blocked domain | `DataError::SsrfBlocked`; Display contains "E-DAT-006" | AC-009 |
| `test_bc_1_03_004_partial_load_continues_on_error` | Two file sources; one missing, one valid | Error vec contains `FileNotFound`; scope map contains the valid one | AC-006 (partial-load) |
| `test_bc_1_03_004_error_code_dat_003_parse_error_through_dispatcher` | Source emits ParseError with E-DAT-003 | `DataError::ParseError`; Display contains "E-DAT-003" | AC-009 |
| `test_bc_1_03_004_error_code_dat_002_network_error_through_dispatcher` | Source emits IoError with E-DAT-002 | `DataError::NetworkError`; Display contains "E-DAT-002" | AC-009 |
| `test_bc_1_03_004_error_code_dat_001_http_404_through_dispatcher` | Source emits IoError with E-DAT-001 + HTTP 404 | `DataError::HttpError { status: 404 }`; Display contains "E-DAT-001" | AC-009 |
| `test_bc_1_03_004_error_code_dat_001_http_500_through_dispatcher` | Source emits IoError with E-DAT-001 + HTTP 500 | `DataError::HttpError { status: 500 }`; Display contains "E-DAT-001" | AC-009 |
| `test_bc_1_03_004_offline_does_not_silently_substitute_empty_value` | Offline skips HTTP; evaluator not involved | Skipped source has no entry in scope | AC-008 |
| `test_bc_1_03_004_dispatcher_loads_real_file_source` | Real `FileDataSource` via load_all with tmp file | Scope contains the file's data; zero errors | AC-003 |

### Unit tests in dispatcher.rs (selected)

| Test Name | What it pins |
|-----------|-------------|
| `test_bc_1_03_004_dispatcher_routes_ioerror_label_to_ioerror_variant` | Synthetic E-DAT-004 + "I/O error reading" → `DataError::IoError`, NOT `FileNotFound` (F-P4-HIGH-001) |
| `test_bc_1_03_004_dispatcher_routes_permission_denied_to_ioerror_not_filenotfound` | Live chmod 0o000 file → `DataError::IoError` (Unix only; F-P4-HIGH-001) |
| `test_extract_path_after_code_handles_path_containing_at_in_label` | Path with " (at " inside it → rsplit_once finds the trailing span (F-P4-LOW-001) |
| `test_strip_bracket_prefix_empty_body_falls_back_to_input` | "[E-DAT-006]" with no body → returns original input, not empty string (F-P4-LOW-002) |
| `test_strip_bracket_prefix_leaves_malformed_bracket_unchanged` | Long bracket code beyond 16-char window → original input unchanged (F-P4-LOW-003 pinned) |

### Mock DataSource for testing

Define a `MockHttpSource` in the test module:
```rust
struct MockHttpSource { value: Value }
impl DataSource for MockHttpSource {
    fn name(&self) -> &str { "mock_http" }
    fn supports_offline(&self) -> bool { true }
    fn load(&self, _ctx: &DataSourceContext) -> Result<Value, DataError> {
        Ok(self.value.clone())
    }
}
```
This avoids actual HTTP calls in integration tests.

## Dependencies

**Depends on:**
- STORY-019 (HTTP + SSRF) — provides `HttpDataSource::supports_offline() = true` and `DataSourceContext.offline`.
- STORY-020 (Excel + SQLite) — completes the `DataSource` implementation suite.

Dependency justification: STORY-021 depends on STORY-019 and STORY-020 because the dispatcher must be able to handle all four DataSource types (JSON/CSV/YAML/TOML, HTTP, XLSX, SQLite). The offline gate only makes sense once all sources are implemented and the `supports_offline()` protocol is in place.

**Blocks:**
- STORY-055 (CLI build command) — the CLI passes `--offline` flag to `DataSourceContext`; requires dispatcher to be present.
- STORY-056 (CLI watch mode) — watch mode re-creates `DataSourceContext` on each change; offline gate must work in repeated calls.

## Implementation Notes

### Dispatcher Return Signature

```rust
pub fn load_all(
    sources: &[(Arc<str>, Box<dyn DataSource>)],
    ctx: &DataSourceContext,
) -> (IndexMap<Arc<str>, Value>, Vec<DataError>) {
    let mut scope = IndexMap::new();
    let mut errors = Vec::new();
    for (name, source) in sources {
        if ctx.offline && source.supports_offline() {
            // Skip silently — no scope entry, no error
            continue;
        }
        match source.load(ctx) {
            Ok(value) => { scope.insert(name.clone(), value); }
            Err(e)    => { errors.push(e); }
        }
    }
    (scope, errors)
}
```

In strict mode (the default), the caller (evaluator) treats any `errors` as fatal. In
warn-only mode, the caller continues with partial scope and renders error-slide
placeholders for affected slides.

### Partial Load Semantics

If one of 3 file sources fails to load (e.g., file not found), the dispatcher returns:
- Scope map with the 2 successfully loaded sources.
- Error vec with 1 `DataError::FileNotFound`.

The caller decides whether to abort (strict) or continue (warn-only). This design is
consistent with the error-accumulation principle (DI-018): all errors are surfaced, not
just the first.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Deck with no HTTP sources, `--offline` flag | No behavior change; build succeeds |
| EC-002 | HTTP source declared but never referenced in slides | HTTP source skipped; no error from dispatcher; evaluator not affected |
| EC-003 | Deck with file + HTTP sources; `--offline` | File sources loaded; HTTP sources skipped |
| EC-004 | `watch --offline` (watch mode) | Dispatcher receives `ctx.offline=true` on every re-evaluation; HTTP never polled |
| EC-005 | All sources fail to load | Scope map is empty; error vec has N entries; caller handles abort-or-continue |
| EC-006 | Zero data sources declared in deck | Dispatcher returns empty scope map, zero errors — valid state |
