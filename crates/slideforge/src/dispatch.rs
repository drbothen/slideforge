//! Plugin dispatch boundary — `catch_unwind` wrapper.
//!
//! This module is the **only production use** of `std::panic::catch_unwind`
//! in the slideforge workspace. Every call to a plugin implementation goes
//! through [`dispatch_plugin`] so that a misbehaving third-party plugin cannot
//! crash the slideforge process.
//!
//! ## Design rationale
//!
//! Plugin implementations are `Box<dyn Trait + Send + Sync>` values that may
//! be supplied by third parties. A panic in a plugin must be caught before it
//! unwinds across the plugin dispatch boundary and terminates the process.
//!
//! `catch_unwind` is safe here because:
//! - All plugin traits are `Send + Sync`.
//! - The closure passed to `catch_unwind` only borrows plugin state — no
//!   `RefCell`, `Mutex` poisoning concerns, or FFI unwind issues.
//! - We immediately convert the `Err` into a [`PluginError::PluginPanic`].
//!
//! ## Traceability
//!
//! - BC-5.02.001 edge case EC-003
//! - STORY-049 AC-008: plugin panic → `Err(PluginError::PluginPanic)`, no crash

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::error::PluginError;

/// Invoke a plugin closure, catching any panic and converting it to
/// [`PluginError::PluginPanic`].
///
/// ## Arguments
///
/// - `plugin_name`: the `id()` of the plugin being dispatched. Used in the
///   error message if a panic occurs.
/// - `f`: a closure that calls the plugin method. Must be `UnwindSafe` (or
///   wrapped in [`AssertUnwindSafe`] when the caller knows it is safe).
///
/// ## Returns
///
/// - `Ok(T)` — the plugin returned normally.
/// - `Err(PluginError::PluginPanic { .. })` — the plugin panicked. The process
///   is NOT terminated.
///
/// ## Errors
///
/// Returns [`PluginError::PluginPanic`] if `f` panics.
///
/// ## Example
///
/// ```rust,no_run
/// # use slideforge::dispatch::dispatch_plugin;
/// # use slideforge::error::PluginError;
/// let result: Result<(), PluginError> = dispatch_plugin("my-plugin", || {
///     // call the plugin here
/// });
/// ```
pub fn dispatch_plugin<F, T>(plugin_name: &str, f: F) -> Result<T, PluginError>
where
    F: FnOnce() -> T,
{
    // SAFETY: We wrap with AssertUnwindSafe because:
    // - Plugin traits are Send + Sync and designed for use from multiple threads.
    // - No shared mutable state is captured by the closure in production paths.
    // - This is the single designated panic-boundary in the workspace
    //   (STORY-049 AC-008 architecture rule).
    catch_unwind(AssertUnwindSafe(f)).map_err(|panic_payload| {
        // Best-effort extraction of the panic message.
        let message: Arc<str> = if let Some(s) = panic_payload.downcast_ref::<&str>() {
            Arc::from(*s)
        } else if let Some(s) = panic_payload.downcast_ref::<String>() {
            Arc::from(s.as_str())
        } else {
            Arc::from("<non-string panic payload>")
        };

        PluginError::PluginPanic {
            plugin_name: Arc::from(plugin_name),
            message,
        }
    })
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── AC-008 / BC-5.02.001 EC-003: panic is caught, returns PluginError::PluginPanic ──

    /// STORY-049 AC-008: a closure that panics with a `&str` payload must return
    /// `Err(PluginError::PluginPanic { plugin_name, message })`.
    ///
    /// The process must NOT crash (the test running to completion proves this).
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_ec003_panic_str_returns_plugin_panic_error() {
        let result = dispatch_plugin("test-plugin", || {
            panic!("intentional test panic");
        });

        // The test panics above — but we are still here because catch_unwind caught it.
        let err = result.expect_err("panicking plugin must return Err");
        match err {
            PluginError::PluginPanic {
                ref plugin_name,
                ref message,
            } => {
                assert_eq!(
                    plugin_name.as_ref(),
                    "test-plugin",
                    "AC-008: plugin_name must match the id passed to dispatch_plugin"
                );
                assert!(
                    message.contains("intentional test panic"),
                    "AC-008: message must contain the panic string; got: {message:?}"
                );
            },
            // Wildcard arm retained for documentation: PluginError is #[non_exhaustive].
            // Any future variant added to PluginError would reach this arm instead of
            // silently passing the test.
            _ => panic!("AC-008: expected PluginPanic variant, got: {err:?}"),
        }
    }

    /// AC-008: a closure that panics with a `String` payload is caught correctly.
    #[test]
    fn test_bc_5_02_001_ec003_panic_string_returns_plugin_panic_error() {
        let result = dispatch_plugin("string-panic-plugin", || {
            panic!("{}", "string-panic-payload".to_owned());
        });
        let err = result.expect_err("panicking plugin must return Err");
        assert!(
            matches!(err, PluginError::PluginPanic { .. }),
            "AC-008: String panic payload must produce PluginPanic; got: {err:?}"
        );
    }

    /// AC-008: a closure that does NOT panic returns Ok.
    #[test]
    fn test_bc_5_02_001_ec003_no_panic_returns_ok() {
        let result = dispatch_plugin("well-behaved-plugin", || 42_u32);
        assert_eq!(
            result.expect("non-panicking plugin must return Ok"),
            42,
            "AC-008: non-panicking plugin must return Ok(value)"
        );
    }

    /// AC-008: `PluginError::PluginPanic` carries the correct `plugin_name` field.
    #[test]
    fn test_bc_5_02_001_ec003_plugin_panic_carries_plugin_name() {
        let result: Result<(), PluginError> = dispatch_plugin("my-exporter-plugin", || {
            panic!("export failed");
        });
        let err = result.expect_err("panicking closure must return Err");
        let PluginError::PluginPanic {
            plugin_name,
            message: _,
        } = err;
        assert_eq!(
            plugin_name.as_ref(),
            "my-exporter-plugin",
            "AC-008: plugin_name in PluginPanic must match the name passed to dispatch_plugin"
        );
    }

    /// AC-008: `PluginError` implements `Display` — the panic message must appear in
    /// the rendered output.
    #[test]
    fn test_bc_5_02_001_plugin_error_display_contains_plugin_name_and_message() {
        let err = PluginError::PluginPanic {
            plugin_name: Arc::from("my-data-source"),
            message: Arc::from("null pointer deref"),
        };
        let display = err.to_string();
        assert!(
            display.contains("my-data-source"),
            "Display must contain plugin_name; got: {display:?}"
        );
        assert!(
            display.contains("null pointer deref"),
            "Display must contain message; got: {display:?}"
        );
    }
}
