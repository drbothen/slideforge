//! Integration tests for STORY-021: --offline flag + error-handling coverage.
//!
//! All tests in this file are named `test_BC_1_03_004_*` or `test_offline_*` /
//! `test_error_code_*` following the BC-based naming convention from
//! `.factory/VSDD.md` and the test-writer agent contract.
//!
//! ## Red Gate
//!
//! Every test in this file drives `slideforge_data::dispatcher::load_all`, which
//! is currently a `todo!()` stub. ALL tests MUST FAIL (panic with the `todo`
//! message or an assertion error) before implementation begins. This is the
//! Red Gate for STORY-021 / BC-1.03.004.
//!
//! ## Test-to-AC traceability
//!
//! | Test | AC(s) covered |
//! |------|--------------|
//! | `test_BC_1_03_004_offline_skips_http_source` | AC-001 |
//! | `test_BC_1_03_004_offline_loads_file_sources` | AC-003 |
//! | `test_BC_1_03_004_online_loads_all_sources` | AC-001 (inverse) |
//! | `test_BC_1_03_004_offline_zero_http_sources` | AC-004 |
//! | `test_BC_1_03_004_offline_with_file_and_http` | AC-006 |
//! | `test_BC_1_03_004_offline_unreferenced_http_source` | AC-005 |
//! | `test_BC_1_03_004_offline_does_not_silently_substitute_empty_value` | AC-008 |
//! | `test_BC_1_03_004_offline_watch_mode_repeated_dispatch` | AC-007 |
//! | `test_BC_1_03_004_error_code_dat_004_file_not_found` | AC-009 |
//! | `test_BC_1_03_004_error_code_dat_006_ssrf_blocked` | AC-009 |
//! | `test_BC_1_03_004_partial_load_continues_on_error` | AC-006 (partial-load) |
//! | `test_BC_1_03_004_zero_sources_returns_empty_scope_zero_errors` | EC-006 |

#![allow(clippy::unwrap_used)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use slideforge_data::{DataSourceContext, dispatcher::load_all};
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

// ---------------------------------------------------------------------------
// MockHttpSource
//
// A minimal DataSource implementation that simulates an HTTP source for
// integration testing. It does NOT make any network calls. It:
//   - Returns a configurable Value on success
//   - Increments an Arc<AtomicUsize> call counter on each `load()` call
//     (used in AC-007 watch-mode tests to assert zero calls when offline)
//
// MockHttpSource overrides `supports_offline()` to return `true`, which is what
// the dispatcher calls to decide whether to skip this source in offline mode.
// The MockHttpSource itself always returns Ok if called — so if the dispatcher
// incorrectly calls it when offline, the test will fail because the scope WILL
// contain the entry (violating the "absent from scope" assertion).
// ---------------------------------------------------------------------------

/// A mock DataSource that simulates an HTTP source.
///
/// Used in integration tests to verify offline-gate behavior without making
/// real network requests. Each call to `load()` increments `call_count`.
struct MockHttpSource {
    /// The data name this source is registered under (used for `id()`).
    name: Arc<str>,
    /// The value returned by `load()` on success.
    value: Value,
    /// Counter incremented on every `load()` call. Shared across clones.
    call_count: Arc<AtomicUsize>,
}

impl MockHttpSource {
    /// Construct a new `MockHttpSource` returning `value` under `name`.
    fn new(name: impl Into<Arc<str>>, value: Value) -> Self {
        MockHttpSource {
            name: name.into(),
            value,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Construct a `MockHttpSource` sharing an external call counter.
    ///
    /// Used in AC-007 watch-mode tests where the caller needs to assert
    /// exactly zero calls across multiple dispatcher invocations.
    fn with_counter(name: impl Into<Arc<str>>, value: Value, call_count: Arc<AtomicUsize>) -> Self {
        MockHttpSource {
            name: name.into(),
            value,
            call_count,
        }
    }
}

impl DataSource for MockHttpSource {
    fn id(&self) -> &str {
        self.name.as_ref()
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.value.clone())
    }

    /// HTTP mock sources declare themselves as network-dependent.
    fn supports_offline(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Helper: a minimal file-based source mock (NOT network-dependent).
//
// Returns a configurable Value without touching the filesystem. Used when we
// need a "file source" in the test without a real temp file, to isolate the
// dispatcher logic from filesystem I/O.
// ---------------------------------------------------------------------------

/// A mock DataSource that simulates a file-based source (not network-dependent).
struct MockFileSource {
    name: Arc<str>,
    value: Value,
}

impl MockFileSource {
    fn new(name: impl Into<Arc<str>>, value: Value) -> Self {
        MockFileSource {
            name: name.into(),
            value,
        }
    }
}

impl DataSource for MockFileSource {
    fn id(&self) -> &str {
        self.name.as_ref()
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(self.value.clone())
    }
}

/// A mock DataSource that always fails with an IoError (simulates a missing file).
struct MockFailSource {
    name: Arc<str>,
}

impl MockFailSource {
    fn new(name: impl Into<Arc<str>>) -> Self {
        MockFailSource { name: name.into() }
    }
}

impl DataSource for MockFailSource {
    fn id(&self) -> &str {
        self.name.as_ref()
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Err(DataSourceError::IoError {
            uri: "mock://missing".to_owned(),
            message: "[E-DAT-004] file not found: /nonexistent/path.json (at mock)".to_owned(),
        })
    }
}

// ---------------------------------------------------------------------------
// AC-001: Dispatcher skips HTTP source when ctx.offline == true
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_skips_http_source`
///
/// AC-001: With one MockHttpSource (supports_offline=true) and ctx.offline=true,
/// the dispatcher must return an EMPTY scope map and the mock's load() must
/// NOT be called (zero call_count).
///
/// Traces to BC-1.03.004 postcondition 1 (no HTTP requests when offline) and
/// postcondition 2 (HTTP-bound names absent from scope).
#[test]
fn test_bc_1_03_004_offline_skips_http_source() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let mock_http =
        MockHttpSource::with_counter("metrics", Value::Int(42), Arc::clone(&call_count));
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("metrics"), Box::new(mock_http))];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, errors) = load_all(&sources, &ctx);

    // The scope must be empty — HTTP source was skipped.
    assert!(
        scope.is_empty(),
        "scope must be empty when offline=true and only HTTP sources present; \
        got {} entries: {:?}",
        scope.len(),
        scope.keys().collect::<Vec<_>>()
    );
    // No errors produced for a skipped source.
    assert!(
        errors.is_empty(),
        "skipping an HTTP source must not produce errors; got: {:?}",
        errors
    );
    // The mock's load() must not have been called.
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        0,
        "offline gate must prevent load() from being called on the HTTP source"
    );
}

// ---------------------------------------------------------------------------
// AC-003: File sources are always loaded regardless of offline flag
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_loads_file_sources`
///
/// AC-003: With a file source (supports_offline=false) and an HTTP source
/// (supports_offline=true), ctx.offline=true must load the file source and
/// skip the HTTP source.
///
/// Traces to BC-1.03.004 invariant 2 + postcondition 4.
#[test]
fn test_bc_1_03_004_offline_loads_file_sources() {
    let file_value = Value::Int(100);
    let http_call_count = Arc::new(AtomicUsize::new(0));

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
        (
            Arc::from("report"),
            Box::new(MockFileSource::new("report", file_value.clone())),
        ),
        (
            Arc::from("live_data"),
            Box::new(MockHttpSource::with_counter(
                "live_data",
                Value::Int(999),
                Arc::clone(&http_call_count),
            )),
        ),
    ];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, errors) = load_all(&sources, &ctx);

    // File source must be in scope.
    assert!(
        scope.contains_key(&Arc::from("report")),
        "file source 'report' must be present in scope even when offline=true"
    );
    assert_eq!(
        scope.get(&Arc::from("report")).cloned(),
        Some(file_value),
        "file source value must match"
    );

    // HTTP source must NOT be in scope.
    assert!(
        !scope.contains_key(&Arc::from("live_data")),
        "HTTP source 'live_data' must be ABSENT from scope when offline=true"
    );

    // No errors expected.
    assert!(errors.is_empty(), "no errors expected; got: {:?}", errors);

    // HTTP load() must not have been called.
    assert_eq!(
        http_call_count.load(Ordering::SeqCst),
        0,
        "HTTP source load() must not be called when offline=true"
    );
}

// ---------------------------------------------------------------------------
// AC-001 inverse: All sources loaded when ctx.offline == false
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_online_loads_all_sources`
///
/// AC-001 (inverse): With ctx.offline=false, BOTH file and HTTP sources must
/// be loaded. The scope must contain both entries.
///
/// Traces to BC-1.03.004 happy-path (offline=false → all sources loaded).
#[test]
fn test_bc_1_03_004_online_loads_all_sources() {
    let file_value = Value::Int(10);
    let http_value = Value::Int(20);

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
        (
            Arc::from("static"),
            Box::new(MockFileSource::new("static", file_value.clone())),
        ),
        (
            Arc::from("live"),
            Box::new(MockHttpSource::new("live", http_value.clone())),
        ),
    ];
    let ctx = DataSourceContext::new(); // offline = false (default)

    let (scope, errors) = load_all(&sources, &ctx);

    assert!(
        scope.contains_key(&Arc::from("static")),
        "file source 'static' must be loaded when online"
    );
    assert!(
        scope.contains_key(&Arc::from("live")),
        "HTTP source 'live' must be loaded when online (offline=false)"
    );
    assert_eq!(
        scope.len(),
        2,
        "scope must have exactly 2 entries when online"
    );
    assert!(errors.is_empty(), "no errors expected; got: {:?}", errors);
}

// ---------------------------------------------------------------------------
// AC-004: Offline flag with zero HTTP sources — no behavior change
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_zero_http_sources`
///
/// AC-004 / EC-001: When ctx.offline=true and there are ONLY file sources
/// (none return supports_offline=true), the behavior is identical to offline=false.
/// All file sources are loaded normally.
///
/// Traces to BC-1.03.004 edge case EC-001.
#[test]
fn test_bc_1_03_004_offline_zero_http_sources() {
    let v1 = Value::Int(1);
    let v2 = Value::Int(2);

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
        (
            Arc::from("data_a"),
            Box::new(MockFileSource::new("data_a", v1.clone())),
        ),
        (
            Arc::from("data_b"),
            Box::new(MockFileSource::new("data_b", v2.clone())),
        ),
    ];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        scope.len(),
        2,
        "all file sources must be loaded when offline=true and no HTTP sources exist"
    );
    assert!(
        scope.contains_key(&Arc::from("data_a")),
        "'data_a' must be present"
    );
    assert!(
        scope.contains_key(&Arc::from("data_b")),
        "'data_b' must be present"
    );
    assert!(errors.is_empty(), "no errors expected; got: {:?}", errors);
}

// ---------------------------------------------------------------------------
// AC-006: Mixed file + HTTP sources, offline=true → file loaded, HTTP skipped
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_with_file_and_http`
///
/// AC-006 / EC-003: With a mix of file and HTTP sources and ctx.offline=true,
/// file sources appear in scope; HTTP sources are absent.
///
/// Traces to BC-1.03.004 edge case EC-003.
#[test]
fn test_bc_1_03_004_offline_with_file_and_http() {
    let file_value = Value::Int(55);

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
        (
            Arc::from("local_config"),
            Box::new(MockFileSource::new("local_config", file_value.clone())),
        ),
        (
            Arc::from("remote_api"),
            Box::new(MockHttpSource::new("remote_api", Value::Int(999))),
        ),
    ];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, errors) = load_all(&sources, &ctx);

    // File source present.
    assert!(
        scope.contains_key(&Arc::from("local_config")),
        "'local_config' (file source) must be in scope"
    );
    assert_eq!(
        scope.get(&Arc::from("local_config")).cloned(),
        Some(file_value),
    );

    // HTTP source absent.
    assert!(
        !scope.contains_key(&Arc::from("remote_api")),
        "'remote_api' (HTTP source) must be absent from scope when offline=true"
    );

    assert!(errors.is_empty(), "no errors expected; got: {:?}", errors);
    assert_eq!(
        scope.len(),
        1,
        "scope must contain exactly 1 entry (the file source)"
    );
}

// ---------------------------------------------------------------------------
// AC-005: Skipped HTTP source produces NO error from dispatcher
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_unreferenced_http_source`
///
/// AC-005 / EC-002: A skipped HTTP source that is never referenced in any
/// slide expression must not produce any error from the dispatcher. The error
/// only comes later, from the evaluator, when `{{ name.field }}` is referenced.
///
/// The dispatcher's contract is: skipped = absent. No error.
///
/// Traces to BC-1.03.004 edge case EC-002.
#[test]
fn test_bc_1_03_004_offline_unreferenced_http_source() {
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("unused_http"),
        Box::new(MockHttpSource::new("unused_http", Value::Int(0))),
    )];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, errors) = load_all(&sources, &ctx);

    // The dispatcher must NOT emit an error for the skipped source.
    assert!(
        errors.is_empty(),
        "a skipped HTTP source must produce ZERO errors from the dispatcher; \
        errors come from the evaluator on access; got: {:?}",
        errors
    );

    // Scope is empty (source was skipped).
    assert!(scope.is_empty(), "scope must be empty");
}

// ---------------------------------------------------------------------------
// AC-008: Skipped source is ABSENT from scope, not present-as-empty
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_does_not_silently_substitute_empty_value`
///
/// AC-008 / BC-1.03.004 invariant 1: `--offline` NEVER silently substitutes
/// an empty value. The skipped data name must be completely ABSENT from scope
/// (not mapped to `Value::Null`, `Value::Map({})`, or `Value::List([])`).
///
/// Traces to BC-1.03.004 invariant 1.
#[test]
fn test_bc_1_03_004_offline_does_not_silently_substitute_empty_value() {
    let source_name: Arc<str> = Arc::from("http_source");
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::clone(&source_name),
        Box::new(MockHttpSource::new(
            "http_source",
            Value::Map(Default::default()),
        )),
    )];
    let ctx = DataSourceContext::new().with_offline(true);

    let (scope, _errors) = load_all(&sources, &ctx);

    // The key must be COMPLETELY absent — not mapped to Null, empty Map, or empty List.
    assert!(
        !scope.contains_key(&source_name),
        "skipped source must be ABSENT from scope, not present with an empty value; \
        --offline never silently substitutes empty data (BC-1.03.004 invariant 1)"
    );
}

// ---------------------------------------------------------------------------
// AC-007: Watch-mode — repeated dispatcher calls never invoke HTTP load()
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_offline_watch_mode_repeated_dispatch`
///
/// AC-007 / EC-004: In watch mode, `load_all` is called repeatedly on each
/// file-change event. With ctx.offline=true, HTTP sources must not be polled
/// even across multiple dispatcher invocations.
///
/// This test calls `load_all` three times and asserts the HTTP call_count
/// remains at zero throughout.
///
/// Traces to BC-1.03.004 edge case EC-004.
#[test]
fn test_bc_1_03_004_offline_watch_mode_repeated_dispatch() {
    let http_call_count = Arc::new(AtomicUsize::new(0));
    let ctx = DataSourceContext::new().with_offline(true);

    for invocation in 1..=3_usize {
        // Rebuild sources on each iteration (as watch mode would recreate them).
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
            Arc::from("live_feed"),
            Box::new(MockHttpSource::with_counter(
                "live_feed",
                Value::Int(invocation as i64),
                Arc::clone(&http_call_count),
            )),
        )];

        let (scope, errors) = load_all(&sources, &ctx);

        assert!(
            scope.is_empty(),
            "scope must be empty on invocation {invocation} (offline=true)"
        );
        assert!(
            errors.is_empty(),
            "no errors on invocation {invocation}; got: {:?}",
            errors
        );
        assert_eq!(
            http_call_count.load(Ordering::SeqCst),
            0,
            "HTTP load() must not be called across {invocation} watch-mode invocations"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-009: Error code verification — E-DAT-004 (file not found)
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_error_code_dat_004_file_not_found`
///
/// AC-009: When a file data source fails because the file path does not
/// exist, the `DataError` returned in the errors vec must include "E-DAT-004"
/// in its Display output.
///
/// Uses the real `FileDataSource` on a guaranteed-missing path so that the
/// error code enforcement is tested against production code, not just a mock.
///
/// Traces to AC-009 and BC-1.03.004 (error-code coverage).
#[test]
fn test_bc_1_03_004_error_code_dat_004_file_not_found() {
    use slideforge_data::FileDataSource;

    // Use a path that cannot exist. FileDataSource::new() takes the path at
    // construction time; the dispatcher calls load("", opts) and the source
    // falls back to self.path, which resolves to the missing path → E-DAT-004.
    let missing_path = "/STORY-021-red-gate-guaranteed-missing-12345/data.json";
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("missing"),
        Box::new(FileDataSource::new(missing_path)),
    )];
    let ctx = DataSourceContext::new();

    // The dispatcher calls load("", opts); FileDataSource falls back to self.path
    // (the missing path baked in at construction), which triggers E-DAT-004.
    let (scope, errors) = load_all(&sources, &ctx);

    // The scope must not contain the missing source.
    assert!(
        !scope.contains_key(&Arc::from("missing")),
        "failed source must not appear in scope"
    );

    // There must be exactly one error.
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected; got: {:?}",
        errors
    );

    let err_display = errors[0].to_string();
    assert!(
        err_display.contains("E-DAT-004"),
        "file-not-found error must contain 'E-DAT-004'; got: {err_display}"
    );
}

// ---------------------------------------------------------------------------
// AC-009: Error code verification — E-DAT-006 (SSRF blocked)
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_error_code_dat_006_ssrf_blocked`
///
/// AC-009: When an HTTP source is blocked by the SSRF allowlist, the
/// `DataError` in the errors vec must contain "E-DAT-006" in its Display
/// output.
///
/// Uses the real `HttpDataSource` with an SSRF-blocking allowlist so that the
/// error-code enforcement is tested against production code.
///
/// Traces to AC-009.
#[test]
fn test_bc_1_03_004_error_code_dat_006_ssrf_blocked() {
    use slideforge_data::{HttpDataSource, allowlist::AllowlistConfig};

    // Configure an HTTP source pointing to a blocked domain.
    let blocked_url = "http://169.254.169.254/metadata"; // IMDS endpoint
    let http_src = HttpDataSource::new(blocked_url).with_allowlist(AllowlistConfig {
        domains: Some(vec![Arc::from("allowed-only.example.com")]),
    });

    // HttpDataSource::supports_offline() returns true, but ctx.offline is false,
    // so the dispatcher WILL attempt the load and collect the SSRF error.
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("imds"), Box::new(http_src))];
    let ctx = DataSourceContext::new();

    // Red Gate: panics on todo!() until implemented.
    let (scope, errors) = load_all(&sources, &ctx);

    assert!(
        !scope.contains_key(&Arc::from("imds")),
        "SSRF-blocked source must not appear in scope"
    );
    assert_eq!(
        errors.len(),
        1,
        "exactly one SSRF error expected; got: {:?}",
        errors
    );

    let err_display = errors[0].to_string();
    assert!(
        err_display.contains("E-DAT-006"),
        "SSRF-blocked error must contain 'E-DAT-006'; got: {err_display}"
    );
}

// ---------------------------------------------------------------------------
// AC-006 (partial-load): Mixed sources; one fails → scope has 1 entry,
// errors vec has 1 entry; iteration continues.
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_partial_load_continues_on_error`
///
/// Partial-load semantics (story spec §Partial Load Semantics): With two file
/// sources where one succeeds and one fails (simulated by MockFailSource),
/// the dispatcher must:
/// - Continue iterating after the failure (not abort on first error)
/// - Add the successful source to scope
/// - Add the failure to the errors vec
///
/// This ensures the error-accumulation principle (DI-018) is enforced at the
/// dispatcher level.
///
/// Traces to BC-1.03.004 / story-spec §Partial Load Semantics.
#[test]
fn test_bc_1_03_004_partial_load_continues_on_error() {
    let ok_value = Value::Int(77);
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![
        (
            Arc::from("good_source"),
            Box::new(MockFileSource::new("good_source", ok_value.clone())),
        ),
        (
            Arc::from("bad_source"),
            Box::new(MockFailSource::new("bad_source")),
        ),
    ];
    let ctx = DataSourceContext::new();

    // Red Gate: panics on todo!() until implemented.
    let (scope, errors) = load_all(&sources, &ctx);

    // Good source must be in scope.
    assert!(
        scope.contains_key(&Arc::from("good_source")),
        "'good_source' must be in scope after successful load"
    );
    assert_eq!(
        scope.get(&Arc::from("good_source")).cloned(),
        Some(ok_value),
    );

    // Failed source must not be in scope.
    assert!(
        !scope.contains_key(&Arc::from("bad_source")),
        "'bad_source' must be absent from scope (load failed)"
    );

    // Exactly one error in the error vec.
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected (for 'bad_source'); got: {:?}",
        errors
    );

    // Error message must reference E-DAT-004.
    let err_display = errors[0].to_string();
    assert!(
        err_display.contains("E-DAT-004"),
        "file-not-found error from partial load must contain 'E-DAT-004'; got: {err_display}"
    );
}

// ---------------------------------------------------------------------------
// EC-006: Zero sources → empty scope, zero errors (valid state)
// ---------------------------------------------------------------------------

/// `test_BC_1_03_004_zero_sources_returns_empty_scope_zero_errors`
///
/// EC-006: When no data sources are declared (empty sources slice), the
/// dispatcher must return an empty scope and an empty error vec. This is a
/// valid state (a deck that uses no external data).
///
/// Traces to BC-1.03.004 / story-spec §Edge Cases EC-006.
#[test]
fn test_bc_1_03_004_zero_sources_returns_empty_scope_zero_errors() {
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![];
    let ctx = DataSourceContext::new();

    // Red Gate: panics on todo!() until implemented.
    let (scope, errors) = load_all(&sources, &ctx);

    assert!(
        scope.is_empty(),
        "empty sources must produce empty scope; got {} entries",
        scope.len()
    );
    assert!(
        errors.is_empty(),
        "empty sources must produce zero errors; got: {:?}",
        errors
    );
}

// ---------------------------------------------------------------------------
// AC-003 / TD-VSDD-059: Real FileDataSource via dispatcher — production code path.
//
// This test verifies that the dispatcher can successfully load a real
// FileDataSource with a real file. Without this test, the production code
// path was untested — the only FileDataSource test was the missing-path test
// which exercised only the error path.
// ---------------------------------------------------------------------------

/// `test_bc_1_03_004_dispatcher_loads_real_file_source`
///
/// AC-003 / TD-VSDD-059: With a real `FileDataSource` pointing to a temporary
/// JSON file, the dispatcher must successfully load the file and return the
/// parsed value in the scope map.
///
/// This test exercises the FULL dispatcher → FileDataSource → parser production
/// code path and proves the code is not dead. The file is written to disk,
/// dispatched, and the returned value is verified against the written content.
///
/// Without this test, the only FileDataSource integration test was the
/// missing-path error test, leaving the success path untested — a violation of
/// TD-VSDD-059 (paper-fix detection: new tests must exercise real production paths).
///
/// Traces to AC-003 (file sources always loaded).
#[test]
fn test_bc_1_03_004_dispatcher_loads_real_file_source() {
    use slideforge_data::FileDataSource;
    use std::io::Write as _;

    // Write a real JSON file to a temp location.
    let mut tmp = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(br#"{"answer": 42, "label": "test-data"}"#)
        .expect("write must succeed");
    let path = tmp.path().to_str().expect("temp path must be valid UTF-8");

    // Construct a real FileDataSource pointing to the temp file.
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("real_file"), Box::new(FileDataSource::new(path)))];
    let ctx = DataSourceContext::new();

    let (scope, errors) = load_all(&sources, &ctx);

    // No errors — the file exists and is valid JSON.
    assert!(
        errors.is_empty(),
        "real file must load without errors; got: {:?}",
        errors
    );

    // The scope must contain the data under the registered name.
    assert!(
        scope.contains_key(&Arc::from("real_file")),
        "'real_file' must be present in scope after successful load"
    );

    // The parsed value must be a map with 'answer' = Int(42).
    let value = scope
        .get(&Arc::from("real_file"))
        .expect("scope entry must exist");
    let map = value.as_map().expect("JSON object must parse as map");
    assert_eq!(
        map.get("answer"),
        Some(&slideforge_types::Value::Int(42)),
        "'answer' field must be Int(42); got: {:?}",
        map.get("answer")
    );
    assert_eq!(
        map.get("label"),
        Some(&slideforge_types::Value::Str(Arc::from("test-data"))),
        "'label' field must be Str('test-data'); got: {:?}",
        map.get("label")
    );
}
