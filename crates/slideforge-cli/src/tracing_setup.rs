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
//! # OTel behavior (`otel` feature enabled)
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
//! - AC-015: OTel export via `--otel-endpoint` + `otel` feature gate

use crate::cli::GlobalFlags;

/// Initialize the global `tracing` subscriber.
///
/// Safe to call multiple times: subsequent calls are no-ops (guarded by
/// `OnceLock`).  In test code, call `try_init()` on the fmt subscriber and
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
    todo!()
}
