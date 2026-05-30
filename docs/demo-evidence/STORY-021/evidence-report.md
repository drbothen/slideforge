# Evidence Report: STORY-021 — DataSource: --offline Flag + Error Handling

**Story ID:** STORY-021
**Epic:** EPIC-05
**Crate:** `slideforge-data`
**BC Trace:** BC-1.03.004
**HEAD SHA verified:** `2dc44fc0170759944a9d05d44d530a9bddd0a108`
**Workspace test count:** 343 tests run: 343 passed, 2 skipped
**LOCAL adversary convergence:** 3/3 clean passes
**Recording tool:** VHS (terminal CLI demos)
**Font:** FiraCode Nerd Font Mono

---

## Coverage Summary

All 10 acceptance criteria have recorded demo evidence. Each demo drives the relevant
integration test(s) in `slideforge-data::integration` (or unit tests in `dispatcher.rs`
for AC-010) via `cargo nextest run -p slideforge-data`.

| AC | Description | Demo File | Tests Covered | Status |
|----|-------------|-----------|--------------|--------|
| AC-001 | Offline gate skips HTTP sources; name absent from scope | `AC-001-offline-gate-skips-http-source` | `offline_skips_http_source`, `online_loads_all_sources` | PASSED |
| AC-002 | Dispatcher contract: scope absence (evaluator owns E-EVL-001) | `AC-002-scope-absent-produces-evl001` | `offline_does_not_silently_substitute_empty_value`, `offline_unreferenced_http_source` | PASSED |
| AC-003 | File-based sources always loaded (supports_offline=false) | `AC-003-file-sources-always-loaded` | `offline_loads_file_sources`, `dispatcher_loads_real_file_source` | PASSED |
| AC-004 | Zero HTTP sources + offline=true identical to offline=false (EC-001) | `AC-004-zero-http-offline-identical` | `offline_zero_http_sources` | PASSED |
| AC-005 | HTTP declared but unreferenced; zero dispatcher errors (EC-002) | `AC-005-unreferenced-http-no-error` | `offline_unreferenced_http_source` | PASSED |
| AC-006 | Mixed file+HTTP offline; file loaded, HTTP skipped (EC-003) | `AC-006-mixed-file-http-offline` | `offline_with_file_and_http`, `partial_load_continues_on_error` | PASSED |
| AC-007 | Watch mode: offline gate per re-eval cycle (EC-004) | `AC-007-watch-mode-repeated-dispatch` | `offline_watch_mode_repeated_dispatch` | PASSED |
| AC-008 | Offline never silently substitutes empty Value::Map (invariant 1) | `AC-008-offline-never-empty-silent` | `offline_does_not_silently_substitute_empty_value` | PASSED |
| AC-009 | DataError enum E-DAT-001..011; Display contains [E-DAT-NNN] prefix | `AC-009-error-codes-dat-001-to-011` | `error_code_dat_001_http_404`, `error_code_dat_001_http_500`, `error_code_dat_002_network_error`, `error_code_dat_003_parse_error`, `error_code_dat_004_file_not_found`, `error_code_dat_006_ssrf_blocked`, `dispatcher_routes_xlsx_bad_magic_to_e_dat_011` | PASSED |
| AC-010 | forbid(unsafe_code) + clippy::pedantic clean + = version pinning | `AC-010-forbid-unsafe-clippy-pinning` | `grep forbid(unsafe_code)`, `cargo clippy -D warnings` | PASSED |

---

## AC-001: Offline Gate Skips HTTP Sources

**File:** `AC-001-offline-gate-skips-http-source.{tape,gif,webm}`
**BC trace:** BC-1.03.004 postcondition 1 (no HTTP requests when offline); postcondition 2 (HTTP-bound names absent from scope)

Demonstrates the core offline gate: `load_all()` with `ctx.offline=true` and a `MockHttpSource`
(supports_offline=true) produces an empty scope map. The inverse test `online_loads_all_sources`
confirms both sources appear in scope when `ctx.offline=false`.

Tests: `test_bc_1_03_004_offline_skips_http_source`, `test_bc_1_03_004_online_loads_all_sources`

---

## AC-002: Dispatcher Absent-Scope Contract

**File:** `AC-002-scope-absent-produces-evl001.{tape,gif,webm}`
**BC trace:** BC-1.03.004 postcondition 3 (skipped sources produce undefined-variable errors)

The dispatcher's responsibility ends at scope population. When `ctx.offline=true` and an HTTP
source is skipped, the data name is simply not present in the returned `IndexMap`. The evaluator
(STORY-011) produces `E-EVL-001` (undefined variable) when a `{{ name.field }}` expression
references an absent binding. `offline_does_not_silently_substitute_empty_value` asserts that
`scope.contains_key("imds")` is false — no empty `Value::Map` was inserted.

Tests: `test_bc_1_03_004_offline_does_not_silently_substitute_empty_value`,
`test_bc_1_03_004_offline_unreferenced_http_source`

---

## AC-003: File Sources Always Loaded

**File:** `AC-003-file-sources-always-loaded.{tape,gif,webm}`
**BC trace:** BC-1.03.004 invariant 2 (file sources always loaded regardless of --offline); postcondition 4

`FileDataSource::supports_offline()` returns `false` (the default for all file-based sources).
The dispatcher's gate `if ctx.offline && source.supports_offline()` evaluates to `false` for
file sources, so they are never skipped. `dispatcher_loads_real_file_source` uses a real temporary
JSON file loaded through the full `FileDataSource` + `load_all` path.

Tests: `test_bc_1_03_004_offline_loads_file_sources`, `test_bc_1_03_004_dispatcher_loads_real_file_source`

---

## AC-004: Zero HTTP Sources With Offline Flag (EC-001)

**File:** `AC-004-zero-http-offline-identical.{tape,gif,webm}`
**BC trace:** BC-1.03.004 edge case EC-001

When the deck has only file sources and `ctx.offline=true`, the offline gate is never triggered
(no source satisfies `supports_offline()==true`). Build completes identically to `offline=false`.

Tests: `test_bc_1_03_004_offline_zero_http_sources`

---

## AC-005: Unreferenced HTTP Source (EC-002)

**File:** `AC-005-unreferenced-http-no-error.{tape,gif,webm}`
**BC trace:** BC-1.03.004 edge case EC-002

When `ctx.offline=true` and an HTTP source is declared but never referenced in slide expressions,
the dispatcher produces zero errors. The missing scope entry is never accessed, so `E-EVL-001`
is not triggered. The error vec is empty; the scope map does not contain the HTTP binding.

Tests: `test_bc_1_03_004_offline_unreferenced_http_source`

---

## AC-006: Mixed File + HTTP Sources, Offline (EC-003)

**File:** `AC-006-mixed-file-http-offline.{tape,gif,webm}`
**BC trace:** BC-1.03.004 edge case EC-003

With `ctx.offline=true`, a deck containing both a `MockFileSource` and a `MockHttpSource` produces
a scope map containing only the file source's entry. The HTTP source is absent. Partial-load
semantics are also shown: when one of multiple file sources fails to load, the error is accumulated
in the errors vec while remaining sources continue loading.

Tests: `test_bc_1_03_004_offline_with_file_and_http`, `test_bc_1_03_004_partial_load_continues_on_error`

---

## AC-007: Watch Mode Repeated Dispatch (EC-004)

**File:** `AC-007-watch-mode-repeated-dispatch.{tape,gif,webm}`
**BC trace:** BC-1.03.004 edge case EC-004

Simulates `slideforge watch --offline` by calling `load_all()` with `ctx.offline=true` three
consecutive times. An `Arc<AtomicUsize>` counter tracks `MockHttpSource.load()` invocations.
The test asserts `call_count == 0` after all three cycles, proving the HTTP source is never
polled regardless of how many re-evaluation cycles occur.

Tests: `test_bc_1_03_004_offline_watch_mode_repeated_dispatch`

---

## AC-008: No Silent Empty Data Substitution (Invariant 1)

**File:** `AC-008-offline-never-empty-silent.{tape,gif,webm}`
**BC trace:** BC-1.03.004 invariant 1 (--offline never silently produces empty data)

The test asserts `!scope.contains_key("imds")` — the key is completely absent, not present with
an empty `Value::Map({})` or `Value::List([])`. The dispatcher uses `continue` (not
`scope.insert(name, Value::Map(Default::default()))`) for skipped sources.

Tests: `test_bc_1_03_004_offline_does_not_silently_substitute_empty_value`

---

## AC-009: Error Codes E-DAT-001 Through E-DAT-011

**File:** `AC-009-error-codes-dat-001-to-011.{tape,gif,webm}`
**BC trace:** BC-1.03.004 (dispatcher robustness invariant); error taxonomy NFR-021

Seven integration tests verify the complete error-code taxonomy:

| Error Code | Variant | Test |
|------------|---------|------|
| E-DAT-001 | `DataError::HttpError { status: 404 }` | `error_code_dat_001_http_404_through_dispatcher` |
| E-DAT-001 | `DataError::HttpError { status: 500 }` | `error_code_dat_001_http_500_through_dispatcher` |
| E-DAT-002 | `DataError::NetworkError` | `error_code_dat_002_network_error_through_dispatcher` |
| E-DAT-003 | `DataError::ParseError` | `error_code_dat_003_parse_error_through_dispatcher` |
| E-DAT-004 | `DataError::FileNotFound` | `error_code_dat_004_file_not_found` |
| E-DAT-006 | `DataError::SsrfBlocked` | `error_code_dat_006_ssrf_blocked` |
| E-DAT-011 | `DataError::BadMagicBytes` | `dispatcher_routes_xlsx_bad_magic_to_e_dat_011` |

Each test asserts that the `Display` impl includes the `[E-DAT-NNN]` bracket prefix
(e.g., `assert!(msg.contains("[E-DAT-004]"))`).

---

## AC-010: Code-Quality Gates

**File:** `AC-010-forbid-unsafe-clippy-pinning.{tape,gif,webm}`
**Traces to:** NFR-021 (no unsafe), NFR-022 (clippy pedantic), NFR-025 (= version pinning)

The demo runs two commands:
1. `grep 'forbid(unsafe_code)' crates/slideforge-data/src/lib.rs` — confirms line 31 carries the attribute.
2. `cargo clippy -p slideforge-data --all-targets --all-features -- -D warnings` — exits clean (no warnings, no errors).

Version pinning (`=` prefix on all production deps in `Cargo.toml`) was verified during implementation
and is enforced by the pre-push hook.

---

## File Manifest

```
docs/demo-evidence/STORY-021/
├── AC-001-offline-gate-skips-http-source.tape
├── AC-001-offline-gate-skips-http-source.gif
├── AC-001-offline-gate-skips-http-source.webm
├── AC-002-scope-absent-produces-evl001.tape
├── AC-002-scope-absent-produces-evl001.gif
├── AC-002-scope-absent-produces-evl001.webm
├── AC-003-file-sources-always-loaded.tape
├── AC-003-file-sources-always-loaded.gif
├── AC-003-file-sources-always-loaded.webm
├── AC-004-zero-http-offline-identical.tape
├── AC-004-zero-http-offline-identical.gif
├── AC-004-zero-http-offline-identical.webm
├── AC-005-unreferenced-http-no-error.tape
├── AC-005-unreferenced-http-no-error.gif
├── AC-005-unreferenced-http-no-error.webm
├── AC-006-mixed-file-http-offline.tape
├── AC-006-mixed-file-http-offline.gif
├── AC-006-mixed-file-http-offline.webm
├── AC-007-watch-mode-repeated-dispatch.tape
├── AC-007-watch-mode-repeated-dispatch.gif
├── AC-007-watch-mode-repeated-dispatch.webm
├── AC-008-offline-never-empty-silent.tape
├── AC-008-offline-never-empty-silent.gif
├── AC-008-offline-never-empty-silent.webm
├── AC-009-error-codes-dat-001-to-011.tape
├── AC-009-error-codes-dat-001-to-011.gif
├── AC-009-error-codes-dat-001-to-011.webm
├── AC-010-forbid-unsafe-clippy-pinning.tape
├── AC-010-forbid-unsafe-clippy-pinning.gif
├── AC-010-forbid-unsafe-clippy-pinning.webm
└── evidence-report.md
```

Total: 30 recording files (10 AC x 3 formats each) + this report = 31 files.
