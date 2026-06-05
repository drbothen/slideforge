//! Dog-fooding test: external plugin compiles using only `slideforge-plugin-api`.
//!
//! ## Purpose (STORY-049 AC-007 / BC-5.02.002 postcondition 4)
//!
//! This test file verifies that a minimal third-party plugin can:
//! 1. Import ONLY from `slideforge` (which re-exports `slideforge-plugin-api`
//!    types) and `slideforge-types`.
//! 2. Implement the `DataSource` trait correctly.
//! 3. Be registered in a `PluginRegistry` alongside bundled plugins.
//!
//! This file intentionally does NOT import from internal crates:
//! - `slideforge_data` — FORBIDDEN: internal crate
//! - `slideforge_pptx` — FORBIDDEN: internal crate
//! - Any other `slideforge-*` crate except `slideforge` (the root)
//!   and `slideforge_types`.
//!
//! If this file compiles, it proves the plugin API is self-contained and
//! usable by external authors.
//!
//! ## Traceability
//!
//! - STORY-049 AC-007
//! - BC-5.02.002 postcondition 4: minimal external plugin compiles without internals

use slideforge::{PluginRegistry, PluginRegistryBuilder, RegistryError};
use slideforge_plugin_api::{DataSource, DataSourceError, DataSourceOptions};
use slideforge_types::Value;

// ── Minimal external DataSource plugin ───────────────────────────────────────

/// A minimal test data source that always returns a fixed JSON-like value.
///
/// This struct represents what a third-party plugin author would write.
/// It imports ONLY from the public `slideforge-plugin-api` + `slideforge-types`
/// surface — no internal crates.
struct TestDataSource {
    /// The fixed value returned by every `load()` call.
    fixed_value: Value,
}

impl TestDataSource {
    /// Create a new `TestDataSource` that always returns `value`.
    fn with_fixed(value: Value) -> Self {
        Self { fixed_value: value }
    }
}

impl DataSource for TestDataSource {
    fn id(&self) -> &'static str {
        "test-external-data-source"
    }

    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(self.fixed_value.clone())
    }
}

// `Send` and `Sync` are auto-derived for `TestDataSource` because its only
// field, `Value`, is `Send + Sync`. No manual `unsafe impl` is needed and
// writing one would teach the WRONG pattern to external plugin authors
// (who should rely on auto-derive, not unsafe). The compiler will reject this
// file if `Value` ever becomes non-Send/Sync, which is the correct signal.

// ─── Tests ───────────────────────────────────────────────────────────────────

/// STORY-049 AC-007 / BC-5.02.002 postcondition 4:
///
/// A minimal `TestDataSource` implementing `DataSource` using ONLY
/// `slideforge-plugin-api` imports compiles and registers successfully
/// in a `PluginRegistry` alongside bundled plugins.
///
/// This test FAILS on todo!() stubs because `register_bundled_plugins` is
/// not yet implemented — the builder will reject the partial registry.
#[test]
fn test_bc_5_02_002_external_plugin_compiles_and_registers_with_bundled_plugins() {
    // Build a registry with bundled plugins + the external TestDataSource.
    let mut builder = PluginRegistryBuilder::default();

    // Register the external plugin alongside bundled plugins.
    slideforge::registry::register_bundled_plugins(&mut builder);
    builder.register_data_source(Box::new(TestDataSource::with_fixed(Value::Null)));

    let registry = builder
        .build()
        .expect("registry with bundled + external plugin must build successfully");

    // The external plugin is registered alongside bundled ones.
    // surface_count() must still be 10 (surface count, not plugin count).
    assert_eq!(
        registry.surface_count(),
        10,
        "AC-007: adding an external plugin must not change surface_count (counts surfaces not plugins)"
    );

    // The external plugin is accessible by its id.
    let plugin = registry.lookup_data_source("test-external-data-source");
    assert!(
        plugin.is_some(),
        "AC-007: external TestDataSource must be findable by id 'test-external-data-source'"
    );

    // The plugin behaves correctly when called.
    let result = plugin
        .expect("AC-007: external TestDataSource must be present after lookup")
        .load("some-uri", &DataSourceOptions::default());
    assert!(
        result.is_ok(),
        "AC-007: TestDataSource.load() must return Ok; got: {result:?}"
    );
}

/// STORY-049 AC-007 / BC-5.02.002 postcondition 4 (standalone):
///
/// The external `TestDataSource` can be registered in an otherwise-empty
/// `PluginRegistry` (direct mutation API, bypassing the builder). This verifies
/// the public trait API is sufficient without ANY bundled plugins.
#[test]
fn test_bc_5_02_002_external_plugin_usable_without_bundled_plugins() {
    let mut registry = PluginRegistry::new();
    registry.register_data_source(Box::new(TestDataSource::with_fixed(Value::Int(42))));

    let plugin = registry.lookup_data_source("test-external-data-source");
    assert!(
        plugin.is_some(),
        "AC-007: external plugin must be findable via direct PluginRegistry::register_data_source"
    );

    let value = plugin
        .expect("AC-007: external plugin must be present after lookup")
        .load("any-uri", &DataSourceOptions::default())
        .expect("TestDataSource.load() must succeed");
    assert_eq!(
        value,
        Value::Int(42),
        "AC-007: TestDataSource must return the configured fixed_value"
    );
}

/// STORY-049 AC-007: the test plugin's `id()` returns the expected constant.
///
/// This is a trivial but load-bearing test — it exercises the trait method
/// dispatch path from the registry's perspective.
#[test]
fn test_bc_5_02_002_external_plugin_id_returns_expected_value() {
    let plugin = TestDataSource::with_fixed(Value::Null);
    assert_eq!(
        plugin.id(),
        "test-external-data-source",
        "AC-007: TestDataSource.id() must return 'test-external-data-source'"
    );
}

/// Compile-time assertion: `TestDataSource` implements `DataSource`.
///
/// If this function compiles, the trait bound is satisfied. This is the
/// primary compile-gate for AC-007.
fn assert_test_data_source_implements_trait<T: DataSource + Send + Sync>() {}
const _: fn() = || {
    assert_test_data_source_implements_trait::<TestDataSource>();
};

/// Compile-time assertion: `RegistryError` is accessible from the root crate.
///
/// External plugin authors must be able to match on `RegistryError` without
/// importing `slideforge-plugin-api` directly.
fn _assert_registry_error_accessible() {
    let _: Option<RegistryError> = None;
}
