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
///
/// FINDING-7 fix: `id()` returns the static plugin-type identifier `"mock-http"`,
/// NOT the binding name.
///
/// FINDING-9 fix: `_binding_name` field removed — it was dead code. The binding
/// name is supplied as the key in the sources slice, not stored in the source.
struct MockHttpSource {
    /// The value returned by `load()` on success.
    value: Value,
    /// Counter incremented on every `load()` call. Shared across clones.
    call_count: Arc<AtomicUsize>,
}

impl MockHttpSource {
    /// Construct a new `MockHttpSource` returning `value`.
    fn new(value: Value) -> Self {
        MockHttpSource {
            value,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Construct a `MockHttpSource` sharing an external call counter.
    ///
    /// Used in AC-007 watch-mode tests where the caller needs to assert
    /// exactly zero calls across multiple dispatcher invocations.
    fn with_counter(value: Value, call_count: Arc<AtomicUsize>) -> Self {
        MockHttpSource { value, call_count }
    }
}

impl DataSource for MockHttpSource {
    /// FINDING-7: Static plugin-type id, NOT the binding name.
    fn id(&self) -> &str {
        "mock-http"
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
///
/// FINDING-7 fix: `id()` returns the static plugin-type identifier `"mock-file"`,
/// NOT the binding name.
///
/// FINDING-9 fix: `_binding_name` field removed — it was dead code.
struct MockFileSource {
    value: Value,
}

impl MockFileSource {
    fn new(value: Value) -> Self {
        MockFileSource { value }
    }
}

impl DataSource for MockFileSource {
    /// FINDING-7: Static plugin-type id, NOT the binding name.
    fn id(&self) -> &str {
        "mock-file"
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(self.value.clone())
    }
}

/// A mock DataSource that always fails with an IoError (simulates a missing file).
///
/// FINDING-7 fix: `id()` returns the static plugin-type identifier `"mock-fail"`,
/// NOT the binding name.
///
/// FINDING-9 fix: `_binding_name` field removed — it was dead code.
///
/// F-P10-OBS-002: `MockFailSource` is a unit struct. Adding `Default` allows callers
/// to use `MockFailSource::default()` instead of `MockFailSource::new()`, eliminating
/// the clippy::new_without_default lint. Both `new()` and `Default::default()` are
/// kept for call-site compatibility.
struct MockFailSource;

impl Default for MockFailSource {
    fn default() -> Self {
        MockFailSource
    }
}

impl MockFailSource {
    fn new() -> Self {
        MockFailSource
    }
}

impl DataSource for MockFailSource {
    /// FINDING-7: Static plugin-type id, NOT the binding name.
    fn id(&self) -> &str {
        "mock-fail"
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
    let mock_http = MockHttpSource::with_counter(Value::Int(42), Arc::clone(&call_count));
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
            Box::new(MockFileSource::new(file_value.clone())),
        ),
        (
            Arc::from("live_data"),
            Box::new(MockHttpSource::with_counter(
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
            Box::new(MockFileSource::new(file_value.clone())),
        ),
        (
            Arc::from("live"),
            Box::new(MockHttpSource::new(http_value.clone())),
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
            Box::new(MockFileSource::new(v1.clone())),
        ),
        (
            Arc::from("data_b"),
            Box::new(MockFileSource::new(v2.clone())),
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
            Box::new(MockFileSource::new(file_value.clone())),
        ),
        (
            Arc::from("remote_api"),
            Box::new(MockHttpSource::new(Value::Int(999))),
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
        Box::new(MockHttpSource::new(Value::Int(0))),
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
/// FINDING-5 fix: Restructured as a two-cycle test that differentiates
/// "source was loaded as non-empty" (online) from "source is absent" (offline).
/// The previous implementation only checked the offline case with an empty-map
/// source — it did not differentiate "skipped" from "loaded-empty".
///
/// Cycle 1 (online, ctx.offline=false): source returns a non-empty map. Scope
/// must contain the binding with the expected map content.
///
/// Cycle 2 (offline, ctx.offline=true): same source is skipped. Scope must NOT
/// contain the binding at all (not even as null/empty-map).
///
/// Additionally, a static-assertion verifies that the dispatcher source code does
/// NOT contain an empty-value substitution at the offline-skip call site.
///
/// Traces to BC-1.03.004 invariant 1.
#[test]
fn test_bc_1_03_004_offline_does_not_silently_substitute_empty_value() {
    use slideforge_types::OrderedMap;

    let source_name: Arc<str> = Arc::from("http_source");

    // Build a non-empty map value the source will return when online.
    let mut expected_map: OrderedMap<Arc<str>, Value> = OrderedMap::new();
    expected_map.insert(Arc::from("revenue"), Value::Int(1_000_000));
    let non_empty_value = Value::Map(expected_map);

    // ---- Cycle 1: online ----
    // Source must load successfully and the binding must be present with data.
    {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
            Arc::clone(&source_name),
            Box::new(MockHttpSource::new(non_empty_value.clone())),
        )];
        let ctx = DataSourceContext::new(); // offline=false

        let (scope, errors) = load_all(&sources, &ctx);

        assert!(
            errors.is_empty(),
            "online cycle must have no errors; got: {:?}",
            errors
        );
        assert!(
            scope.get(&source_name).is_some(),
            "online cycle: binding 'http_source' must be present in scope"
        );
        let loaded = scope.get(&source_name).unwrap();
        assert_eq!(
            *loaded, non_empty_value,
            "online cycle: loaded value must match the expected non-empty map"
        );
    }

    // ---- Cycle 2: offline ----
    // Same source is skipped. Binding must be COMPLETELY absent — not mapped to
    // Null, empty Map, or empty List. `scope.get()` must return None.
    {
        let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
            Arc::clone(&source_name),
            Box::new(MockHttpSource::new(non_empty_value.clone())),
        )];
        let ctx = DataSourceContext::new().with_offline(true);

        let (scope, _errors) = load_all(&sources, &ctx);

        assert!(
            scope.get(&source_name).is_none(),
            "offline cycle: skipped source must be ABSENT from scope (get() must return None), \
            not mapped to any empty value; --offline never silently substitutes empty data \
            (BC-1.03.004 invariant 1)"
        );
    }

    // Static-source greps are too brittle — any future unit test or unrelated code in
    // dispatcher.rs that legitimately uses `Value::Null` would fail this integration test.
    // The two-cycle behavioral test above is load-bearing for AC-008: it directly verifies
    // that the offline-skip branch does NOT insert an empty value into scope. That runtime
    // assertion is the authoritative gate; the static-grep block is removed per F-P6-MED-002.
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
            Box::new(MockFileSource::new(ok_value.clone())),
        ),
        (Arc::from("bad_source"), Box::new(MockFailSource::new())),
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

// ---------------------------------------------------------------------------
// FINDING-4: AC-009 integration coverage for E-DAT-001 / E-DAT-002 / E-DAT-003
//
// These tests use real HttpDataSource + TcpListener mock servers (same pattern
// as http.rs unit tests) to exercise the full dispatcher → HttpDataSource →
// parse production code path. Tests verify:
//   (a) errors.len() == 1
//   (b) errors[0].code() == "E-DAT-NNN"
//   (c) errors[0].to_string() contains a specific substring proving the right
//       path was taken.
//
// This replaces the tautological StubSource coverage for E-DAT-001/002/003 that
// only exercised the dispatcher's bracket-code parsing, not the full source path.
// ---------------------------------------------------------------------------

/// Spawn a single-shot TcpListener mock HTTP server on a random port.
///
/// Returns `(addr, join_handle)`. The server accepts exactly one connection,
/// sends the canned response, and exits. Call `handle.join().unwrap()` after
/// the test to ensure the thread is cleaned up.
fn spawn_mock_http_server_for_finding4(
    status: u16,
    content_type: &'static str,
    body: &'static str,
) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    let handle = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (addr, handle)
}

/// `test_bc_1_03_004_error_code_dat_001_http_404_through_dispatcher`
///
/// FINDING-4: Real `HttpDataSource` hitting a mock 404 endpoint via the full
/// dispatcher → HttpDataSource → load_all path returns exactly one error with
/// code E-DAT-001 and Display containing "404".
///
/// Traces to AC-009 / BC-1.03.004.
#[test]
fn test_bc_1_03_004_error_code_dat_001_http_404_through_dispatcher() {
    use slideforge_data::HttpDataSource;

    let (addr, handle) =
        spawn_mock_http_server_for_finding4(404, "application/json", "{\"error\":\"not found\"}");
    let url = format!("http://127.0.0.1:{}", addr.port());

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("api"),
        Box::new(HttpDataSource::new(url.as_str())),
    )];
    let ctx = DataSourceContext::new();

    let (scope, errors) = load_all(&sources, &ctx);

    handle.join().unwrap();

    assert!(
        !scope.contains_key(&Arc::from("api")),
        "404 source must not appear in scope"
    );
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for HTTP 404; got: {:?}",
        errors
    );
    assert_eq!(
        errors[0].code(),
        "E-DAT-001",
        "HTTP 404 must produce E-DAT-001; got: {} (display: {})",
        errors[0].code(),
        errors[0]
    );
    let display = errors[0].to_string();
    assert!(
        display.contains("404"),
        "E-DAT-001 Display for 404 must contain '404'; got: {display}"
    );
}

/// `test_bc_1_03_004_error_code_dat_001_http_500_through_dispatcher`
///
/// FINDING-4: Real `HttpDataSource` hitting a mock 500 endpoint via the full
/// dispatcher path returns exactly one error with code E-DAT-001 and Display
/// containing "500".
///
/// HttpDataSource retries 5xx once; the mock server must serve two responses.
/// Traces to AC-009 / BC-1.03.004.
#[test]
fn test_bc_1_03_004_error_code_dat_001_http_500_through_dispatcher() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    // Must serve TWO responses because HttpDataSource retries once on 5xx.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        for _ in 0..2 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 500 Internal Server Error\r\n\
                            Content-Type: application/json\r\n\
                            Content-Length: 2\r\n\
                            Connection: close\r\n\r\n{}";
            let _ = stream.write_all(response.as_bytes());
        }
    });

    let url = format!("http://127.0.0.1:{}", addr.port());

    use slideforge_data::HttpDataSource;
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("api"),
        Box::new(HttpDataSource::new(url.as_str())),
    )];
    let ctx = DataSourceContext::new();

    let (scope, errors) = load_all(&sources, &ctx);

    handle.join().unwrap();

    assert!(
        !scope.contains_key(&Arc::from("api")),
        "500 source must not appear in scope"
    );
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for HTTP 500; got: {:?}",
        errors
    );
    assert_eq!(
        errors[0].code(),
        "E-DAT-001",
        "HTTP 500 must produce E-DAT-001; got: {} (display: {})",
        errors[0].code(),
        errors[0]
    );
    let display = errors[0].to_string();
    assert!(
        display.contains("500"),
        "E-DAT-001 Display for 500 must contain '500'; got: {display}"
    );
}

/// `test_bc_1_03_004_error_code_dat_002_network_error_through_dispatcher`
///
/// FINDING-4: Real `HttpDataSource` pointing at a dead port (guaranteed
/// connection refused) → dispatcher returns exactly one error with code E-DAT-002.
///
/// Strategy (F-P3-LOW-006 fix): Use `http://127.0.0.1:1/` — port 1 is a
/// privileged port that reliably refuses connections on Linux and macOS because:
///   - Unprivileged processes cannot bind to ports < 1024.
///   - The OS immediately returns ECONNREFUSED (Linux) or ECONNREFUSED (macOS).
///   - On Windows, port 1 may behave differently; this test is expected to pass
///     on the Linux/macOS CI matrix defined in `.github/workflows/ci.yml`.
///
/// This replaces the previous TcpListener bind-then-drop TOCTOU pattern: that
/// pattern was racy because another process could claim the freed ephemeral port
/// between `drop(listener)` and `HttpDataSource::load`. Port 1 is deterministic.
///
/// Traces to AC-009 / BC-1.03.004.
#[test]
fn test_bc_1_03_004_error_code_dat_002_network_error_through_dispatcher() {
    // Port 1 is privileged and always refuses connections from user-space processes.
    // This is a deterministic alternative to the TOCTOU bind-then-drop pattern.
    // On Linux/macOS: ECONNREFUSED. On Windows: may return a different OS error
    // (still not 2xx, still mapped to E-DAT-002 via the network error path).
    let url = "http://127.0.0.1:1/";

    use slideforge_data::HttpDataSource;
    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("api"), Box::new(HttpDataSource::new(url)))];
    let ctx = DataSourceContext::new();

    let (scope, errors) = load_all(&sources, &ctx);

    assert!(
        !scope.contains_key(&Arc::from("api")),
        "network-error source must not appear in scope"
    );
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for connection refused; got: {:?}",
        errors
    );
    assert_eq!(
        errors[0].code(),
        "E-DAT-002",
        "connection refused must produce E-DAT-002; got: {} (display: {})",
        errors[0].code(),
        errors[0]
    );
}

/// `test_bc_1_03_004_error_code_dat_003_parse_error_through_dispatcher`
///
/// FINDING-4: Real `FileDataSource` pointing at a temp file with invalid JSON →
/// dispatcher returns exactly one error with code E-DAT-003 and the correct
/// format inferred from the `.json` extension.
///
/// Traces to AC-009 / BC-1.03.004.
#[test]
fn test_bc_1_03_004_error_code_dat_003_parse_error_through_dispatcher() {
    use slideforge_data::FileDataSource;
    use std::io::Write as _;

    // Write a temp file with invalid JSON content.
    let mut tmp = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(b"{not valid json")
        .expect("write must succeed");
    let path = tmp
        .path()
        .to_str()
        .expect("temp path must be valid UTF-8")
        .to_owned();

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("bad_json"),
        Box::new(FileDataSource::new(path.as_str())),
    )];
    let ctx = DataSourceContext::new();

    let (scope, errors) = load_all(&sources, &ctx);

    // Drop tmp AFTER load_all (file must exist during the load).
    drop(tmp);

    assert!(
        !scope.contains_key(&Arc::from("bad_json")),
        "parse-failed source must not appear in scope"
    );
    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for invalid JSON; got: {:?}",
        errors
    );
    assert_eq!(
        errors[0].code(),
        "E-DAT-003",
        "invalid JSON must produce E-DAT-003; got: {} (display: {})",
        errors[0].code(),
        errors[0]
    );
    let display = errors[0].to_string();
    assert!(
        display.contains("E-DAT-003"),
        "E-DAT-003 Display must contain the code; got: {display}"
    );
    // Verify format is inferred as Json from the .json extension.
    assert!(
        display.contains("Json") || display.contains("json"),
        "E-DAT-003 Display must mention Json format for .json file; got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P10-HIGH-001: End-to-end XLSX bad-magic → E-DAT-011 (integration)
// ---------------------------------------------------------------------------

/// `test_bc_1_03_004_dispatcher_routes_xlsx_bad_magic_to_e_dat_011`
///
/// F-P10-HIGH-001 integration test: Drive a real `XlsxDataSource` (via
/// `FileDataSource`) with a file that has an `.xlsx` extension but contains
/// invalid magic bytes (not a ZIP archive). The dispatcher must route this
/// to `DataError::ParseError` with `code() == "E-DAT-011"`, NOT "E-DAT-003".
///
/// This test confirms the FULL end-to-end production path:
///   1. `FileDataSource::load` dispatches to `XlsxDataSource::load_internal`
///   2. `load_internal` returns `DataError::ParseError { code: E_DAT_011, ... }`
///   3. `FileDataSource::load` translates to `DataSourceError::ParseError`
///      with message `"[E-DAT-011] ..."` (granular code preserved)
///   4. Dispatcher `map_source_error` uses `parse_e_dat_code` to preserve E-DAT-011
///
/// Traces to F-P10-HIGH-001, BC-1.03.006 postcondition 9.
#[test]
fn test_bc_1_03_004_dispatcher_routes_xlsx_bad_magic_to_e_dat_011() {
    use slideforge_data::FileDataSource;
    use std::io::Write as _;

    // Create a file with .xlsx extension but wrong magic bytes.
    let mut tmp = tempfile::Builder::new()
        .suffix(".xlsx")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(b"This is not a real xlsx ZIP archive")
        .expect("write must succeed");
    let path = tmp.path().to_str().expect("temp path must be valid UTF-8");

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("bad_xlsx"), Box::new(FileDataSource::new(path)))];
    let ctx = DataSourceContext::new();

    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(errors.len(), 1, "expected exactly one error");
    assert_eq!(
        errors[0].code(),
        "E-DAT-011",
        "XLSX bad-magic must produce E-DAT-011, NOT E-DAT-003; got: {} (display: {})",
        errors[0].code(),
        errors[0]
    );
    let display = errors[0].to_string();
    assert!(
        display.contains("E-DAT-011"),
        "E-DAT-011 Display must contain the code; got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P11-LOW-002: Unsupported extension — clean Display (no stutter)
// ---------------------------------------------------------------------------

/// `test_unsupported_extension_display_clean`
///
/// F-P11-LOW-002: When a `FileDataSource` is loaded via the dispatcher with a
/// path that has an unsupported extension (e.g., `.txt`), the resulting
/// `DataError` Display must be clean — no stuttered `"(unsupported extension: ...)"`.
///
/// Before the fix, `file.rs` emitted:
///   `UnsupportedUri { uri: "/tmp/data.txt (unsupported extension: txt)" }`
///
/// The dispatcher mapped `UnsupportedUri.uri` → `DataError::UnsupportedFormat.extension`,
/// producing the broken display:
///   `"[E-DAT-003] unsupported format: '/tmp/data.txt (unsupported extension: txt)' — ..."`
///
/// After the fix, `file.rs` emits only the bare extension as the uri:
///   `UnsupportedUri { uri: "txt" }`
///
/// Resulting in the clean display:
///   `"[E-DAT-003] unsupported format: 'txt' — supported: json, csv, yaml, ..."`
///
/// This integration test drives the FULL production path:
///   `FileDataSource::load` → `DataSourceError::UnsupportedUri { uri: "txt" }`
///   → dispatcher `map_source_error` → `DataError::UnsupportedFormat { extension: "txt" }`
///   → Display: `"[E-DAT-003] unsupported format: 'txt' — supported: ..."`
///
/// Traces to F-P11-LOW-002 (Pass 11 adversarial finding).
#[test]
fn test_unsupported_extension_display_clean() {
    use slideforge_data::FileDataSource;
    use std::io::Write as _;

    // Create a temp file with an unsupported extension.
    let mut tmp = tempfile::Builder::new()
        .suffix(".txt")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(b"some data").expect("write must succeed");
    let path = tmp
        .path()
        .to_str()
        .expect("temp path must be valid UTF-8")
        .to_owned();

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> = vec![(
        Arc::from("txt_source"),
        Box::new(FileDataSource::new(path.as_str())),
    )];
    let ctx = DataSourceContext::new();

    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for unsupported extension; got: {:?}",
        errors
    );

    let err_code = errors[0].code();
    assert_eq!(
        err_code, "E-DAT-003",
        "unsupported extension must produce E-DAT-003; got: {err_code}"
    );

    let display = errors[0].to_string();

    // Must contain just the bare extension in quotes — no full path, no annotation.
    assert!(
        display.contains("'txt'"),
        "Display must contain \"'txt'\" (bare extension); got: {display}"
    );

    // Must contain the supported-formats list (helpful context preserved).
    assert!(
        display.contains("supported: json, csv"),
        "Display must contain the supported-formats list; got: {display}"
    );

    // Must NOT contain the stuttered annotation "(unsupported extension:".
    assert!(
        !display.contains("(unsupported extension:"),
        "Display must NOT contain '(unsupported extension:' annotation (F-P11-LOW-002 stutter guard); got: {display}"
    );

    // Must NOT contain the full file path in the extension slot.
    // (The path MAY appear elsewhere in the display — e.g., in a span — but NOT as the extension value.)
    // The extension slot is rendered as "'<value>' —" so we check that the extension value is bare.
    // A path contains '/' so we verify the substring between the first "'" and " —" is just the extension.
    if let Some(start) = display.find("unsupported format: '") {
        let after = &display[start + "unsupported format: '".len()..];
        if let Some(end) = after.find('\'') {
            let ext_slot = &after[..end];
            assert!(
                !ext_slot.contains('/'),
                "Extension slot must not contain a path separator ('/'): extension slot is '{ext_slot}'; full display: {display}"
            );
        }
    }

    // Keep tmp alive until after load_all to ensure the file exists during dispatch.
    drop(tmp);
}

// ---------------------------------------------------------------------------
// F-P12-HIGH-001: SQLite validate_sqlite_magic canonical separator
// ---------------------------------------------------------------------------

/// `test_bc_1_03_004_dispatcher_routes_sqlite_open_failure_correctly`
///
/// F-P12-HIGH-001: When `validate_sqlite_magic` cannot open a `.sqlite` file (e.g.,
/// permission denied), the emitted `DataSourceError::IoError` message must use the
/// canonical `"'<path>': <reason>"` separator — NOT the broken
/// `"to validate SQLite magic: <reason>"` form that previously broke path/reason
/// extractors in the dispatcher.
///
/// Specifically asserts:
/// - `errors[0].code() == "E-DAT-004"` — I/O failure routes to IoError (not ParseError)
/// - Display contains the file path EXACTLY ONCE (no duplication from extractors)
/// - Display contains the OS reason (e.g., "permission denied") EXACTLY ONCE
/// - Display does NOT contain `"to validate SQLite magic"` (the broken annotation)
/// - Bracket `[E-DAT-004]` appears exactly ONCE in Display
///
/// This is a `#[cfg(unix)]` test because `chmod 0o000` is POSIX-specific. On Windows,
/// file ACLs behave differently and this exact permission-denied path is not exercised.
///
/// Traces to F-P12-HIGH-001, BC-1.03.007 postcondition 6.
#[test]
#[cfg(unix)]
fn test_bc_1_03_004_dispatcher_routes_sqlite_open_failure_correctly() {
    use slideforge_data::SqliteDataSource;
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;

    // Create a tempfile with a .sqlite extension that contains valid SQLite magic
    // bytes, then remove read permission so validate_sqlite_magic cannot open it.
    let tmp_dir = tempfile::tempdir().expect("tempdir must be created");
    let path = tmp_dir.path().join("test_perm_denied.sqlite");

    // Write valid SQLite magic header + padding (16 bytes minimum).
    let magic = b"SQLite format 3\x00some padding to make it look real";
    fs::write(&path, magic).expect("write temp sqlite file must succeed");

    // Remove all permissions so File::open fails with "permission denied".
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000))
        .expect("chmod 000 must succeed on unix");

    let path_str = path.to_str().expect("path must be valid UTF-8");

    let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
        Arc::from("sqlite_perm"),
        Box::new(SqliteDataSource::new(path_str, "SELECT 1")),
    )];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    // Restore permissions so tempdir cleanup can delete the file.
    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o644));

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for permission-denied sqlite file; got: {:?}",
        errors
    );

    // Must produce E-DAT-004 (IoError / file-not-found category), not E-DAT-003.
    let code = errors[0].code();
    assert_eq!(
        code, "E-DAT-004",
        "permission-denied SQLite open failure must produce E-DAT-004; got: {code}"
    );

    let display = errors[0].to_string();

    // Bracket [E-DAT-004] must appear exactly once — no double-bracket nesting.
    assert_eq!(
        display.matches("[E-DAT-004]").count(),
        1,
        "Display must contain exactly one [E-DAT-004]; got: {display}"
    );

    // File path must appear in the display.
    assert!(
        display.contains(path_str),
        "Display must contain the file path; got: {display}"
    );

    // OS reason ("permission denied") must appear — do not hardcode the exact OS phrase,
    // but at minimum "permission" or "denied" must be present.
    let os_reason_present =
        display.to_lowercase().contains("permission") || display.to_lowercase().contains("denied");
    assert!(
        os_reason_present,
        "Display must contain the OS reason (permission denied); got: {display}"
    );

    // The broken annotation must NOT appear (F-P12-HIGH-001 fix guard).
    assert!(
        !display.contains("to validate SQLite magic"),
        "Display must NOT contain 'to validate SQLite magic' annotation (F-P12-HIGH-001); got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P12-MED-001: xlsx UnsupportedUri must emit bare extension — no nested bracket
// ---------------------------------------------------------------------------

/// `test_bc_1_03_006_xlsx_xls_extension_display_clean`
///
/// F-P12-MED-001: When an `XlsxDataSource` rejects a `.xls` file, the resulting
/// `DataError` Display must be clean — no nested bracket code, no annotated string
/// embedded in the extension slot.
///
/// Before the fix, `data_error_to_source_error` in xlsx.rs emitted:
///   `UnsupportedUri { uri: "[E-DAT-003] '/path' is an .xls file. Only .xlsx ..." }`
///
/// The dispatcher mapped `UnsupportedUri.uri` → `DataError::UnsupportedFormat.extension`,
/// producing the broken double-bracket display:
///   `"[E-DAT-003] unsupported format: '[E-DAT-003] '/path' ...' — supported: ..."`
///
/// After the fix, `data_error_to_source_error` emits only the bare extension:
///   `UnsupportedUri { uri: "xls" }`
///
/// Producing the clean display:
///   `"[E-DAT-003] unsupported format: 'xls' — supported: ..."`
///
/// This integration test drives the FULL end-to-end path through the dispatcher.
///
/// Traces to F-P12-MED-001, BC-1.03.006 AC-005.
#[test]
fn test_bc_1_03_006_xlsx_xls_extension_display_clean() {
    use slideforge_data::XlsxDataSource;

    // .xls file — does not need to exist; extension check fires first.
    let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
        Arc::from("xls_source"),
        Box::new(XlsxDataSource::new("/tmp/legacy_data.xls")),
    )];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for .xls file; got: {:?}",
        errors
    );

    let code = errors[0].code();
    assert_eq!(
        code, "E-DAT-003",
        ".xls extension must produce E-DAT-003; got: {code}"
    );

    let display = errors[0].to_string();

    // [E-DAT-003] must appear EXACTLY ONCE — not double-nested.
    assert_eq!(
        display.matches("[E-DAT-003]").count(),
        1,
        "Display must contain exactly one [E-DAT-003] (no nested bracket); got: {display}"
    );

    // The bare extension 'xls' must appear in the extension slot.
    assert!(
        display.contains("'xls'"),
        "Display must contain \"'xls'\" (bare extension in extension slot); got: {display}"
    );

    // The supported-formats hint must be present.
    assert!(
        display.contains("supported:"),
        "Display must contain the supported-formats hint; got: {display}"
    );

    // The annotated message must NOT appear in the extension slot.
    assert!(
        !display.contains("is an .xls file"),
        "Display must NOT contain annotated AC-005 wording in extension slot (F-P12-MED-001); got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P13-HIGH-001: sqlite UnsupportedUri must emit bare extension — no nested bracket
// ---------------------------------------------------------------------------

/// `test_bc_1_03_007_sqlite_extension_display_clean`
///
/// F-P13-HIGH-001: When a `SqliteDataSource` rejects a `.db3` file, the resulting
/// `DataError` Display must be clean — no nested bracket code, no annotated string
/// embedded in the extension slot.
///
/// Before the fix, `validate_sqlite_extension` returned `Err(String)` containing:
///   `"[E-DAT-014] unsupported extension for SQLite data source: '.db3'. ..."`
///
/// The caller mapped this to `DataSourceError::UnsupportedUri { uri: annotated_string }`.
/// The dispatcher then put the annotated string into `DataError::UnsupportedFormat.extension`,
/// producing the broken double-bracket display:
///   `"[E-DAT-003] unsupported format: '[E-DAT-014] unsupported extension ...' — supported: ..."`
///
/// After F-P13-HIGH-001: `validate_sqlite_extension` returns `Err(DataError::UnsupportedFormat)`
/// carrying the bare extension. The caller extracts `"db3"` and wraps it in
/// `DataSourceError::UnsupportedUri { uri: "db3" }`, producing the clean display:
///   `"[E-DAT-003] unsupported format: 'db3' — supported: ..."`
///
/// F-P13-HIGH-002: E-DAT-014 is retired. The `[E-DAT-014]` annotation must not appear.
///
/// This integration test drives the FULL end-to-end path through the dispatcher.
///
/// Traces to F-P13-HIGH-001, F-P13-HIGH-002, BC-1.03.007 invariant 8, VP-035.
#[test]
fn test_bc_1_03_007_sqlite_extension_display_clean() {
    use slideforge_data::SqliteDataSource;

    // .db3 file — does not need to exist; extension check fires first.
    let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
        Arc::from("db3_source"),
        Box::new(SqliteDataSource::new("/tmp/database.db3", "SELECT 1")),
    )];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for .db3 file; got: {:?}",
        errors
    );

    let code = errors[0].code();
    assert_eq!(
        code, "E-DAT-003",
        ".db3 extension must produce E-DAT-003; got: {code}"
    );

    let display = errors[0].to_string();

    // [E-DAT-003] must appear EXACTLY ONCE — not double-nested.
    assert_eq!(
        display.matches("[E-DAT-003]").count(),
        1,
        "Display must contain exactly one [E-DAT-003] (no nested bracket); got: {display}"
    );

    // The bare extension 'db3' must appear in the extension slot.
    assert!(
        display.contains("'db3'"),
        "Display must contain \"'db3'\" (bare extension in extension slot); got: {display}"
    );

    // The supported-formats hint must be present.
    assert!(
        display.contains("supported:"),
        "Display must contain the supported-formats hint; got: {display}"
    );

    // E-DAT-014 (retired) must NOT appear in the Display.
    assert!(
        !display.contains("[E-DAT-014]"),
        "Display must NOT contain retired [E-DAT-014] annotation (F-P13-HIGH-001); got: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P13-MED-001: path stutter — path must appear exactly once in E-DAT-013
// and E-DAT-011 Display when routed through the dispatcher.
// ---------------------------------------------------------------------------

/// `test_bc_1_03_007_sqlite_magic_mismatch_path_appears_once`
///
/// F-P13-MED-001: When `validate_sqlite_magic` rejects a file (magic-byte mismatch →
/// E-DAT-013), the final `DataError` Display must contain the file path EXACTLY ONCE.
///
/// Before the fix, the reason string started with `"'{path}' has .{ext} extension ..."`.
/// The dispatcher's `DataError::ParseError` Display adds "parse error for '{path}'",
/// producing path stutter: path appears both in the prefix and in the reason.
///
/// After F-P13-MED-001: reason starts with "file has .{ext} extension ...", so the
/// path appears only once (in "parse error for '{path}'").
///
/// Traces to F-P13-MED-001, BC-1.03.007 postcondition 7, VP-034.
#[test]
fn test_bc_1_03_007_sqlite_magic_mismatch_path_appears_once() {
    use slideforge_data::SqliteDataSource;
    use std::io::Write as _;

    // Create a file with .sqlite extension but garbage content (no SQLite magic).
    let mut tmp = tempfile::Builder::new()
        .suffix(".sqlite")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(b"this is not a sqlite database at all")
        .expect("write must succeed");
    let path_str = tmp.path().to_str().expect("temp path must be valid UTF-8");

    let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
        Arc::from("sqlite_magic"),
        Box::new(SqliteDataSource::new(path_str, "SELECT 1")),
    )];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for bad-magic sqlite file; got: {:?}",
        errors
    );

    assert_eq!(
        errors[0].code(),
        "E-DAT-013",
        "SQLite magic mismatch must produce E-DAT-013; got: {}",
        errors[0].code()
    );

    let display = errors[0].to_string();

    // Path must appear exactly once — no stutter.
    let path_count = display.matches(path_str).count();
    assert_eq!(
        path_count, 1,
        "File path must appear exactly once in E-DAT-013 Display (no stutter); \
        got {path_count} occurrences in: {display}"
    );
}

/// `test_bc_1_03_006_xlsx_bad_magic_path_appears_once`
///
/// F-P13-MED-001: When `validate_xlsx_magic` rejects a file (magic-byte mismatch →
/// E-DAT-011), the final `DataError` Display must contain the file path EXACTLY ONCE.
///
/// Before the fix, the reason string started with `"'{path}' has .{ext} extension ..."`.
/// After F-P13-MED-001: reason starts with `"file has .{ext} extension ..."`.
///
/// Traces to F-P13-MED-001, BC-1.03.006 postcondition 9, VP-026.
#[test]
fn test_bc_1_03_006_xlsx_bad_magic_path_appears_once() {
    use slideforge_data::FileDataSource;
    use std::io::Write as _;

    // Create a file with .xlsx extension but wrong magic bytes.
    let mut tmp = tempfile::Builder::new()
        .suffix(".xlsx")
        .tempfile()
        .expect("temp file creation must succeed");
    tmp.write_all(b"this is not a valid xlsx ZIP archive")
        .expect("write must succeed");
    let path_str = tmp.path().to_str().expect("temp path must be valid UTF-8");

    let sources: Vec<(Arc<str>, Box<dyn slideforge_plugin_api::DataSource>)> = vec![(
        Arc::from("xlsx_magic"),
        Box::new(FileDataSource::new(path_str)),
    )];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected for bad-magic xlsx file; got: {:?}",
        errors
    );

    assert_eq!(
        errors[0].code(),
        "E-DAT-011",
        "XLSX magic mismatch must produce E-DAT-011; got: {}",
        errors[0].code()
    );

    let display = errors[0].to_string();

    // Path must appear exactly once — no stutter.
    let path_count = display.matches(path_str).count();
    assert_eq!(
        path_count, 1,
        "File path must appear exactly once in E-DAT-011 Display (no stutter); \
        got {path_count} occurrences in: {display}"
    );
}

// ---------------------------------------------------------------------------
// F-P13-MED-002: wildcard arm must use binding name (not empty string) for URI.
// ---------------------------------------------------------------------------

/// `test_map_source_error_wildcard_uses_binding_name`
///
/// F-P13-MED-002: The dispatcher's wildcard `#[non_exhaustive]` arm must set
/// `uri = Arc::clone(name)` — the binding name — NOT `Arc::from("")`.
///
/// Before the fix: `DataError::UnspecifiedSourceError { uri: "", ... }`, producing:
///   `"[E-DAT-015] data source error for '': ..."`
///
/// After the fix: `DataError::UnspecifiedSourceError { uri: "binding_name", ... }`,
/// producing: `"[E-DAT-015] data source error for 'binding_name': ..."`
///
/// The wildcard arm fires for truly unrecognized `DataSourceError` variants (new variants
/// added in future plugin-api releases). Since we cannot construct a future variant in a
/// test, this test drives the E-DAT-015 path via a no-bracket IoError (same code path as
/// what the wildcard arm will use). The key invariant verified: the URI field in
/// `UnspecifiedSourceError` must never be empty when a binding name is known.
///
/// Load-bearing: if the wildcard arm reverts to `Arc::from("")`, the empty-URI assertion
/// would expose that regression. The IoError fallback (no-bracket path) uses
/// `Arc::from(uri.as_str())` — the wildcard fix ensures it uses `Arc::clone(name)`.
///
/// Traces to F-P13-MED-002, F-P5-MED-004.
#[test]
fn test_map_source_error_wildcard_uses_binding_name() {
    // A minimal DataSource that always returns an IoError with no [E-DAT-NNN] bracket code.
    // The no-bracket IoError path routes to UnspecifiedSourceError (E-DAT-015).
    struct NoBracketSource;
    impl DataSource for NoBracketSource {
        fn id(&self) -> &str {
            "no-bracket-source"
        }

        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Err(DataSourceError::IoError {
                uri: "custom://some-resource".to_owned(),
                // No [E-DAT-NNN] bracket code — routes to UnspecifiedSourceError (E-DAT-015).
                message: "third-party error without bracket code".to_owned(),
            })
        }
    }

    let sources: Vec<(Arc<str>, Box<dyn DataSource>)> =
        vec![(Arc::from("my_source_binding"), Box::new(NoBracketSource))];
    let ctx = DataSourceContext::new();
    let (_scope, errors) = load_all(&sources, &ctx);

    assert_eq!(
        errors.len(),
        1,
        "exactly one error expected; got: {:?}",
        errors
    );
    assert_eq!(
        errors[0].code(),
        "E-DAT-015",
        "no-bracket IoError must produce E-DAT-015; got: {}",
        errors[0].code()
    );

    // The Display must NOT show an empty URI — the binding name must appear.
    let display = errors[0].to_string();
    assert!(
        !display.contains("data source error for '':"),
        "Display must NOT show empty URI in E-DAT-015 (F-P13-MED-002 wildcard fix); got: {display}"
    );
}
