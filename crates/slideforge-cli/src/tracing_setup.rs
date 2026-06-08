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
//! tokio async runtime.
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
/// [`init_tracing`] detect the lock is set and return `Ok(())` immediately.
static TRACING_INIT: OnceLock<()> = OnceLock::new();

/// Initialize the global `tracing` subscriber.
///
/// Safe to call multiple times: subsequent calls are no-ops (guarded by
/// [`OnceLock`]).  In test code, call `try_init()` on the fmt subscriber and
/// discard the error so concurrent test threads do not panic.
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
/// # Errors
///
/// Returns an error string if the subscriber could not be installed (rare —
/// only happens if another subscriber was installed outside this function).
pub fn init_tracing(global: &GlobalFlags) -> Result<(), String> {
    // If already initialized, silently succeed.
    if TRACING_INIT.get().is_some() {
        return Ok(());
    }

    let result = init_tracing_inner(global);

    if result.is_ok() {
        // Mark as initialized. Ignore the error from set() — it can only fail
        // if another thread raced us here and already set the lock, which is
        // exactly the idempotency case we want to handle gracefully.
        let _ = TRACING_INIT.set(());
    }

    result
}

/// Inner initialization logic — separated from the `OnceLock` guard so that
/// the actual subscriber setup is not duplicated between the otel and non-otel
/// paths.
fn init_tracing_inner(global: &GlobalFlags) -> Result<(), String> {
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
        return init_tracing_with_otel(global, env_filter, endpoint);
    }

    // Default path: fmt subscriber only.
    init_tracing_fmt(global, env_filter)
}

/// Initialize a plain `tracing_subscriber::fmt` subscriber (no `OTel`).
///
/// The `--json` flag controls *diagnostic* rendering (via `DiagnosticRenderer`),
/// not the tracing subscriber format.  The tracing subscriber always uses the
/// human-readable fmt layer; only the verbosity level changes.
fn init_tracing_fmt(
    _global: &GlobalFlags,
    env_filter: tracing_subscriber::EnvFilter,
) -> Result<(), String> {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|e| format!("tracing subscriber init failed: {e}"))
}

/// Initialize the subscriber with an OpenTelemetry OTLP export layer.
///
/// Only compiled when the `otel` Cargo feature is enabled.
/// Uses the current builder API (`SpanExporter::builder().with_tonic()...`)
/// per ADR-021 — NOT the deprecated `new_pipeline()` / `install_batch()` pattern.
///
/// Per ADR-021 the `opentelemetry_sdk` `rt-tokio` feature requires the tokio
/// async runtime.
#[cfg(feature = "otel")]
fn init_tracing_with_otel(
    _global: &GlobalFlags,
    env_filter: tracing_subscriber::EnvFilter,
    endpoint: &str,
) -> Result<(), String> {
    // Required trait imports for method resolution with the `OTel` 0.32 builder API.
    // `WithExportConfig` provides `with_endpoint`; `TracerProvider` provides `tracer`.
    // These are NOT deprecated — use of `new_pipeline()` / `set_text_map_propagator()`
    // IS deprecated and must NOT be used (AC-015).
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig as _;
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

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
        .map_err(|e| format!("tracing subscriber init failed: {e}"))
}
