//! `tracing-subscriber` initialization for the `slideforge` CLI.
//!
//! Provides [`init_tracing`] which must be called exactly once per process.
//! Calling it a second time is safe: the function uses `OnceLock` internally
//! and silently ignores subsequent calls.
//!
//! # Default behavior (no `otel` feature)
//!
//! `tracing_subscriber::fmt` with `EnvFilter` driven by `RUST_LOG`.  When
//! `--json` is set, the subscriber emits structured JSON log lines.  The
//! CLI verbosity level (`-v`, `-vv`) sets a fallback filter level when
//! `RUST_LOG` is absent.
//!
//! # `OTel` behavior (`otel` feature enabled)
//!
//! When `--otel-endpoint <url>` is provided and the binary was compiled with
//! the `otel` feature, an additional `tracing-opentelemetry` layer is
//! installed that exports spans to the OTLP endpoint via gRPC (tonic).
//!
//! Uses the current builder API (NOT the deprecated `new_pipeline()`):
//!
//! ```rust,ignore
//! let exporter = opentelemetry_otlp::SpanExporter::builder()
//!     .with_tonic()
//!     .with_endpoint(endpoint_url)
//!     .build()?;
//! let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
//!     .with_batch_exporter(exporter)
//!     .build();
//! let tracer = provider.tracer("slideforge");
//! let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
//! ```
//!
//! Per ADR-021, the `opentelemetry_sdk` `rt-tokio` feature requires the
//! tokio async runtime.  [`init_tracing`] constructs a dedicated
//! `tokio::runtime::Runtime` when the `OTel` path is taken and returns a
//! [`TracingGuard`] that **owns** that runtime.  The caller must keep the
//! guard alive for the lifetime of the pipeline — dropping it shuts down the
//! runtime and flushes any queued spans before the process exits.
//!
//! # Traceability
//!
//! - NFR-032: `tracing` instrumentation throughout the pipeline
//! - AC-011: six pipeline stage spans emitted per build
//! - AC-015: `OTel` export via `--otel-endpoint` + `otel` feature gate

use std::sync::OnceLock;

use crate::cli::GlobalFlags;

/// Guard to ensure the tracing subscriber is initialized at most once per process.
///
/// `OnceLock<()>` is set on first successful initialization. Subsequent calls to
/// [`init_tracing`] detect the lock is set and return a no-op guard immediately.
static TRACING_INIT: OnceLock<()> = OnceLock::new();

// ── TracingGuard ─────────────────────────────────────────────────────────────

/// Lifetime guard returned by [`init_tracing`].
///
/// In the default (non-otel) build this is a zero-sized type.  When the
/// `otel` feature is active **and** `--otel-endpoint` was supplied, the guard
/// owns the `tokio::runtime::Runtime` that backs the batch span processor.
/// The runtime must outlive the pipeline (`commands::dispatch`) so that all
/// spans are exported before the process exits.
///
/// Dropping the guard:
/// - (non-otel) is a no-op.
/// - (otel) shuts down the `SdkTracerProvider` (flushes queued spans) and
///   then drops the Tokio runtime.
///
/// `main()` should bind this as `let _guard = init_tracing(...)?;` and NOT
/// drop it early.
///
/// Implements `Debug` manually because `tokio::runtime::Runtime` does not
/// implement `Debug`.
pub struct TracingGuard {
    /// Owned Tokio runtime for the `OTel` batch exporter.  `None` when the
    /// default (non-otel) subscriber is active.
    #[cfg(feature = "otel")]
    otel_runtime: Option<tokio::runtime::Runtime>,

    /// `OTel` tracer provider — held so we can call `shutdown()` on drop.
    #[cfg(feature = "otel")]
    otel_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
}

impl std::fmt::Debug for TracingGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "otel")]
        {
            f.debug_struct("TracingGuard")
                .field(
                    "otel_runtime",
                    &self.otel_runtime.as_ref().map(|_| "<Runtime>"),
                )
                .field(
                    "otel_provider",
                    &self.otel_provider.as_ref().map(|_| "<SdkTracerProvider>"),
                )
                .finish()
        }
        #[cfg(not(feature = "otel"))]
        f.debug_struct("TracingGuard").finish()
    }
}

#[cfg(feature = "otel")]
impl Drop for TracingGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.otel_provider.take() {
            // Best-effort flush + shutdown.  If this returns an error there
            // is nothing useful we can do — we are already in drop().
            let _ = provider.shutdown();
        }
        // Runtime drops after provider so the batch processor can finish
        // exporting before the reactor disappears.
    }
}

impl TracingGuard {
    /// Construct the no-op guard used by the non-otel path (and by the
    /// already-initialized early-return path).
    #[cfg(not(feature = "otel"))]
    fn noop() -> Self {
        Self {}
    }

    /// Construct the no-op guard used by the non-otel path when built with
    /// the otel feature but without an endpoint (or already-initialized).
    #[cfg(feature = "otel")]
    fn noop() -> Self {
        Self {
            otel_runtime: None,
            otel_provider: None,
        }
    }

    /// Construct a guard that owns the given runtime and provider.
    #[cfg(feature = "otel")]
    fn with_runtime(
        runtime: tokio::runtime::Runtime,
        provider: opentelemetry_sdk::trace::SdkTracerProvider,
    ) -> Self {
        Self {
            otel_runtime: Some(runtime),
            otel_provider: Some(provider),
        }
    }
}

impl Default for TracingGuard {
    /// The default guard is a no-op — used by `main()` when subscriber
    /// initialization fails, so dispatch can still proceed.
    fn default() -> Self {
        Self::noop()
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Initialize the global `tracing` subscriber.
///
/// Safe to call multiple times: subsequent calls are no-ops (guarded by
/// [`OnceLock`]) and return a no-op [`TracingGuard`].  In test code, call
/// `try_init()` on the fmt subscriber and discard the error so concurrent
/// test threads do not panic.
///
/// # Verbosity mapping
///
/// | `global.verbose` | Level (when `RUST_LOG` absent) |
/// |-----------------|-------------------------------|
/// | 0 | warn |
/// | 1 | info |
/// | 2 | debug |
/// | 3+ | trace |
///
/// # Runtime ownership
///
/// When the `otel` feature is active and `--otel-endpoint` is provided, the
/// returned [`TracingGuard`] owns the `tokio::runtime::Runtime` that backs
/// the OTLP batch exporter.  The caller **must** keep it alive for the
/// lifetime of the pipeline (do NOT drop early).  Dropping it triggers a
/// graceful provider shutdown + span flush.
///
/// # Errors
///
/// Returns an error string if the subscriber could not be installed (rare —
/// only happens if another subscriber was installed outside this function).
pub fn init_tracing(global: &GlobalFlags) -> Result<TracingGuard, String> {
    // If already initialized via this function, return a no-op guard immediately.
    if TRACING_INIT.get().is_some() {
        return Ok(TracingGuard::noop());
    }

    let result = init_tracing_inner(global);

    match result {
        Ok(guard) => {
            // Our subscriber was installed.  Mark as initialized so subsequent
            // calls short-circuit above.
            let _ = TRACING_INIT.set(());
            Ok(guard)
        },
        Err(ref e) if e.contains("already been set") || e.contains("already initialized") => {
            // A global dispatcher was already installed by another component
            // (e.g., a test framework subscriber, another call on a racing
            // thread).  Tracing infrastructure IS operational — we just did
            // not install our own subscriber on top of an existing one.
            // Treat this as success: mark initialized, return a no-op guard.
            // This prevents `init_tracing` from competing with `#[traced_test]`
            // in test environments and from returning `Err` when another
            // subscriber is legitimately present.
            let _ = TRACING_INIT.set(());
            Ok(TracingGuard::noop())
        },
        Err(e) => Err(e),
    }
}

/// Inner initialization logic — separated from the `OnceLock` guard so that
/// the actual subscriber setup is not duplicated between the otel and non-otel
/// paths.
fn init_tracing_inner(global: &GlobalFlags) -> Result<TracingGuard, String> {
    use tracing_subscriber::EnvFilter;

    // Build the `EnvFilter`.  `RUST_LOG` takes precedence; if absent, the CLI
    // verbosity level sets the fallback.
    let default_level = match global.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    #[cfg(feature = "otel")]
    if let Some(ref endpoint) = global.otel_endpoint {
        return init_tracing_with_otel(env_filter, endpoint);
    }

    // Default path: fmt subscriber only.
    init_tracing_fmt(env_filter)
}

/// Initialize a plain `tracing_subscriber::fmt` subscriber (no `OTel`).
///
/// The `--json` flag controls *diagnostic* rendering (via `DiagnosticRenderer`),
/// not the tracing subscriber format.  The tracing subscriber always uses the
/// human-readable fmt layer; only the verbosity level changes.
fn init_tracing_fmt(env_filter: tracing_subscriber::EnvFilter) -> Result<TracingGuard, String> {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|e| format!("tracing subscriber init failed: {e}"))?;

    Ok(TracingGuard::noop())
}

/// Initialize the subscriber with an OpenTelemetry OTLP export layer.
///
/// Only compiled when the `otel` Cargo feature is enabled.
/// Uses the current builder API (`SpanExporter::builder().with_tonic()...`)
/// per ADR-021 — NOT the deprecated `new_pipeline()` / `install_batch()` pattern.
///
/// # Runtime construction
///
/// Per ADR-021 the `opentelemetry_sdk` `rt-tokio` feature requires the tokio
/// async runtime for the batch span processor.  This function constructs a
/// dedicated `tokio::runtime::Runtime` (`multi_thread`, 1 worker thread),
/// enters it (so the batch processor's `tokio::spawn` calls succeed), and
/// returns it inside the [`TracingGuard`].  The guard's `Drop` impl calls
/// `provider.shutdown()` (span flush) before the runtime is dropped.
///
/// The runtime is deliberately NOT shared with any application-level async
/// work — it exists solely to service the `OTel` exporter background tasks.
#[cfg(feature = "otel")]
fn init_tracing_with_otel(
    env_filter: tracing_subscriber::EnvFilter,
    endpoint: &str,
) -> Result<TracingGuard, String> {
    // Required trait imports for method resolution with the `OTel` 0.32 builder API.
    // `WithExportConfig` provides `with_endpoint`; `TracerProvider` provides `tracer`.
    // These are NOT deprecated — use of `new_pipeline()` / `set_text_map_propagator()`
    // IS deprecated and must NOT be used (AC-015).
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig as _;
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    // Build a dedicated Tokio runtime for the OTel batch exporter.
    // `multi_thread` with 1 worker is sufficient — the batch processor needs
    // only a reactor for I/O and a thread for background flushing.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("slideforge-otel")
        .enable_all()
        .build()
        .map_err(|e| format!("failed to build tokio runtime for OTel exporter: {e}"))?;

    // Enter the runtime so that code in this thread can call tokio APIs
    // (including the batch span processor's internal `tokio::spawn`).
    // The enter-guard keeps the runtime context active on this thread
    // until it is explicitly dropped below.
    let enter_guard = runtime.enter();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| format!("`OTel` OTLP exporter init failed: {e}"))?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("slideforge");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .try_init()
        .map_err(|e| format!("tracing subscriber init failed: {e}"))?;

    // Drop the enter-guard: the subscriber is now installed globally.
    // The runtime itself stays alive inside TracingGuard — the batch
    // processor will use it to schedule background export tasks.
    drop(enter_guard);

    Ok(TracingGuard::with_runtime(runtime, provider))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "otel"))]
mod tests {
    use tracing_subscriber::EnvFilter;

    use super::init_tracing_with_otel;

    /// AC-015 (unit): `init_tracing_with_otel` constructs an exporter and `OTel`
    /// subscriber layer successfully without requiring an externally-provided
    /// Tokio runtime.
    ///
    /// This test calls the private `init_tracing_with_otel` function directly
    /// (bypassing the `OnceLock` guard in `init_tracing`) so that even when
    /// other tests have already initialized the subscriber, we can verify that
    /// the `OTel`-specific code path — runtime construction, exporter build, and
    /// provider/layer wiring — executes without panic.
    ///
    /// Note: `try_init()` inside `init_tracing_with_otel` may return
    /// `Err("a global default trace dispatcher has already been set")` when
    /// running alongside other tests.  We accept both `Ok` (clean environment)
    /// and the specific already-initialized error (parallel test environment),
    /// but NEVER a panic (which was the defect — "there is no reactor running").
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_1_15_003_ac_015_otel_init_path_no_panic_no_reactor_required() {
        let env_filter = EnvFilter::new("warn");
        let result = init_tracing_with_otel(env_filter, "http://localhost:4317");

        match result {
            Ok(_guard) => {
                // Happy path: fresh test environment, subscriber installed.
            },
            Err(e) => {
                // Acceptable in parallel test environments where another test
                // already installed a global subscriber.  The key assertion is
                // that we reached this point WITHOUT a panic — the panic was
                // "there is no reactor running, must be called from the context
                // of a Tokio 1.x runtime", and that defect is now fixed.
                assert!(
                    e.contains("already been set") || e.contains("already initialized"),
                    "AC-015: unexpected error from init_tracing_with_otel (expected only \
                     already-initialized conflict or Ok, not a reactor panic); got: {e}"
                );
            },
        }
    }
}
