---
document_type: adr
adr_id: ADR-021
title: Async runtime adoption — tokio for preview server, watch-mode HTTP, interactive keypress, and OTLP export
status: accepted
date: 2026-06-07
subsystems_affected:
  - SS-09
  - SS-10
  - SS-18
supersedes: null
superseded_by: null
related_adrs:
  - ADR-008
  - ADR-003
traces_to:
  - ARCH-INDEX.md
  - .factory/specs/prd-supplements/nfr-catalog.md
---

# ADR-021: Async Runtime Adoption — tokio for Preview Server, Watch-Mode HTTP, Interactive Keypress, and OTLP Export

## Context

The slideforge workspace was initially designed as a fully synchronous pipeline: parse →
evaluate → brand → validate → layout → export, all on a single thread. The CLI
(`slideforge-cli`) and all crates compiled without a tokio dependency. `ureq` was used
for any synchronous HTTP calls in `slideforge-data`.

Four implementation stories require concurrent or event-driven I/O that cannot be built
cleanly on a synchronous threading model:

1. **STORY-047 — Web preview server (`slideforge serve`)**: The preview server is an
   `axum`-based HTTP + WebSocket server that pushes slide-change diffs to connected
   browsers. axum 0.8 is natively async; running it on a dedicated `std::thread` with a
   `sync` channel bridge is possible but architecturally incoherent — axum 0.8 requires
   a tokio runtime even for `.await`-free handler bodies because of trait bounds
   (`IntoResponse`, `FromRequest`, the tower-service blanket impls). There is no
   non-tokio execution mode in axum 0.8.

2. **STORY-056 — Watch-mode data-source HTTP re-polling**: `slideforge watch` re-polls
   `@data(url: ...)` sources on file change. With a synchronous HTTP client (`ureq`),
   re-polling blocks the file-watch thread, introducing latency proportional to HTTP
   response time. `reqwest` async client on a tokio runtime allows the file-watcher and
   HTTP poll to run concurrently.

3. **Interactive `r`-keypress in watch mode**: The user presses `r` to force a rebuild
   during `slideforge watch`. `crossterm::event::read()` is a blocking syscall — it
   consumes a thread while waiting. `crossterm`'s `event-stream` feature provides an
   async stream (`EventStream`) that yields `crossterm::event::Event` values on a tokio
   executor, allowing keypress handling to share the async runtime without a blocking
   thread park.

4. **Optional OTLP telemetry export (behind `otel` cargo feature)**: The OpenTelemetry
   Rust SDK 0.32.x uses tokio as its batch-exporter runtime (`rt-tokio` feature). A
   batch OTLP span exporter requires `tokio::spawn` to drain the span queue
   asynchronously. Without a tokio runtime, OTLP export is unavailable. The `otel`
   feature is optional — disabling it drops all OTel deps. When enabled, the same
   runtime used for the preview server and watch-mode polling services the OTLP drain.

Prior to this ADR, the workspace declared `tokio = "=1.38"` in `[workspace.dependencies]`
but this version is superseded by the Wave-5 remove-uncertainty pin of `=1.52.3`
(the registry-verified latest stable as of 2026-06-07). This ADR documents the decision
to formally adopt tokio, records the correct version and feature set, and establishes
constraints on async surface expansion.

## Decision

slideforge adopts `tokio = "=1.52.3"` as the workspace async runtime with a curated
feature set. The runtime is initialized once in `slideforge-cli` and shared across all
async surfaces via the implicit task-local context. Library crates (`slideforge-preview`,
`slideforge-data`) are async-capable but do NOT spawn their own runtimes — they expose
`async fn`s consumed by the CLI runtime.

### Pinned dependency set

```toml
# [workspace.dependencies]

# Async runtime — curated features only, NOT "full"
tokio = { version = "=1.52.3", features = [
    "rt-multi-thread",  # multi-threaded scheduler (required for axum + concurrent HTTP polling)
    "macros",           # #[tokio::main], #[tokio::test]
    "net",              # TcpListener for axum serve
    "time",             # timeout, sleep (watch-mode debounce)
    "sync",             # broadcast, Mutex, RwLock, oneshot (preview push channel)
    "signal",           # tokio::signal::unix — SIGTERM handler (unix-only, see below)
    "fs",               # tokio::fs for async file I/O where needed
    "io-util",          # AsyncReadExt, AsyncWriteExt
] }

# HTTP server — WebSocket built in, no tokio-tungstenite production dep
axum = { version = "=0.8.9", features = ["ws"] }
# NOTE: axum 0.8.2 was yanked. 0.8.9 is the registry-verified safe minimum.
# axum 0.8 path syntax: /{id} NOT /:id
# Graceful shutdown: axum::serve(listener, app).with_graceful_shutdown(signal_future)

# HTTP client — no native-tls, no OpenSSL (musl + Windows compatible)
reqwest = { version = "=0.13.4", default-features = false, features = [
    "rustls",       # TLS backend: rustls (pure Rust, works on musl + Windows)
                    # NOTE: renamed from "rustls-tls" (reqwest 0.11/0.12) to "rustls" in reqwest 0.13.x
    "charset",      # charset detection for text/html responses
    "http2",        # HTTP/2 support
] }
# reqwest 0.13 default TLS is rustls; pinning features makes the backend
# line-independent regardless of future reqwest default changes.

# Async keypress (watch-mode interactive r-key)
crossterm = { version = "=0.29.0", features = ["event-stream"] }
# crossterm::event::read() is blocking; "event-stream" enables EventStream (async).
# Guard KeyEventKind::Press on Windows: crossterm fires Release + Repeat events too.

# OTLP telemetry (optional, behind otel cargo feature)
opentelemetry         = { version = "=0.32.0", optional = true }
opentelemetry_sdk     = { version = "=0.32.0", features = ["rt-tokio"], optional = true }
opentelemetry-otlp    = { version = "=0.32.0", features = ["trace", "grpc-tonic"], optional = true }
tracing-opentelemetry = { version = "=0.33.0", optional = true }
# tracing-opentelemetry 0.33.0 intentionally ships one minor ahead of opentelemetry 0.32.0.
# This is the crate's published cadence — it is correct, not a version mismatch.

# Dev-only WebSocket test client (aligns with axum 0.8.9's internal tungstenite)
tokio-tungstenite = { version = "=0.29.0", optional = true }
# tokio-tungstenite is a DEV-DEPENDENCY ONLY — it is the test client for WS protocol
# tests in slideforge-preview. It must NOT appear as a production dep.
```

### SIGTERM handling

```rust
// UNIX-only SIGTERM shutdown hook in slideforge-cli/src/serve.rs:
#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM");
    let mut sigint  = signal(SignalKind::interrupt()).expect("failed to register SIGINT");
    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv()  => {},
    }
}
```

The `signal` feature in tokio enables `tokio::signal::unix` on UNIX targets. Windows
does not support POSIX signals; the `#[cfg(unix)]` gate ensures the Windows build
compiles without this branch. Windows shutdown is handled via `tokio::signal::ctrl_c()`,
which is signal-feature-gated and cross-platform.

### OTLP initialization (new builder API)

The old `opentelemetry-otlp` `new_pipeline()...install_batch()` flow is removed in
OpenTelemetry Rust 0.32. The correct initialization sequence is:

```rust
// Behind #[cfg(feature = "otel")]
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_opentelemetry::OpenTelemetryLayer;

let exporter = SpanExporter::builder()
    .with_tonic()
    .build()?;

let provider = SdkTracerProvider::builder()
    .with_batch_exporter(exporter)
    .build();

let tracer = provider.tracer("slideforge");
let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
// Install otel_layer into tracing_subscriber::Registry
```

### Consuming surfaces and crate ownership

| Surface | Story | Crate | Async boundary |
|---------|-------|-------|----------------|
| Preview server | STORY-047 | slideforge-preview | `axum::serve` + `WebSocketUpgrade` |
| Watch-mode HTTP re-poll | STORY-056 | slideforge-data | `reqwest::Client::get(...).await` |
| Interactive r-keypress | STORY-056 | slideforge-cli | `crossterm::event::EventStream` |
| OTLP export | future (otel feature) | slideforge-cli | `SdkTracerProvider` batch drain |

### Constraint: no async in pure-core crates

Pure-core crates (`slideforge-syntax`, `slideforge-eval`, `slideforge-validate`,
`slideforge-layout`, `slideforge-types`, `slideforge-plugin-api`, `slideforge-math`,
`slideforge-charts`) are prohibited from depending on tokio or declaring `async fn`
in their public APIs. These crates are Kani-amenable precisely because they contain
no I/O and no concurrency primitives. Introducing tokio as a dependency (even
`tokio::sync::Mutex`) would pollute the purity boundary map (ADR-005, purity-boundary-map.md)
and make Kani proofs infeasible for the affected modules.

Effectful shell crates (`slideforge-preview`, `slideforge-data`, `slideforge-cli`,
`slideforge-brand`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`,
`slideforge-package`, `slideforge-config`) MAY depend on tokio per their declared purity
classification.

## Rationale

**Why tokio and not async-std or smol?** axum 0.8 has an unconditional tokio dependency —
it is not executor-agnostic. Adopting a non-tokio executor would require either forking
axum or using a tokio compatibility shim, both of which add maintenance burden. tokio is
the dominant Rust async runtime with the broadest ecosystem alignment (reqwest, crossterm
event-stream, the OpenTelemetry Rust SDK all require tokio). Single-runtime simplicity
is the correct call.

**Why `rt-multi-thread` and not `rt` (single-thread)?** The preview server handles
concurrent WebSocket connections from multiple browser tabs while the file-watcher runs
concurrently. A single-threaded executor serializes these — a slow WebSocket push could
delay the file-watcher notification. `rt-multi-thread` allows work-stealing across the
preview, watcher, and HTTP-poll tasks.

**Why NOT tokio feature `"full"`?** The `full` feature activates every tokio subsystem
including `process` (subprocess spawn), `net-pipe` (Unix domain sockets), and others
that slideforge does not use. Curated features reduce compilation time, binary size, and
the attack surface — fewer tokio subsystems means fewer code paths compiled into the CLI.
The supply-chain implication of `full` is that a future tokio release adding a
vulnerability in a subsystem we activate (but never use) would trigger `cargo audit`
flags. Curated features avoid this class of spurious advisory.

**Why reqwest over ureq for async HTTP?** `ureq` is a synchronous HTTP client. It works
correctly for the initial data-source fetch at build time, but STORY-056 watch-mode
re-polling requires non-blocking HTTP on the async runtime. `reqwest` with
`default-features = false, features = ["rustls", ...]` is the idiomatic async HTTP
client in the tokio ecosystem, with zero native-tls / OpenSSL linkage — critical for
musl static binaries and Windows builds where OpenSSL is not available.
(Note: reqwest 0.13.x renamed the TLS feature from `rustls-tls` to `rustls`.)

**Why no tokio-tungstenite in production?** axum 0.8 provides `axum::extract::ws::WebSocketUpgrade`
with a built-in WebSocket implementation backed by tungstenite internally. There is no
need to add tokio-tungstenite as a production dependency — doing so would create a
duplicate tungstenite version in the dependency tree (the version axum uses internally
and the version we pin directly may diverge). tokio-tungstenite as a dev-dependency for
test clients is correct: tests exercise the WS protocol from the client side and need
explicit tungstenite control.

**`#![forbid(unsafe_code)]` is unaffected.** tokio is itself implemented with unsafe
code internally, but slideforge crates do not write unsafe code — `forbid(unsafe_code)`
applies to slideforge's own sources. tokio's internal unsafe is audited by the tokio
maintainers and is not within scope of the slideforge `forbid` declaration.

**Supply-chain implication of the async closure.** Adding tokio, axum, reqwest, and
crossterm introduces approximately 40–60 additional transitive dependencies into the
workspace tree. These are audited by `cargo deny check` (licenses + advisories) and
pinned via `Cargo.lock`. The `otel` feature, when disabled (default), adds zero
transitive deps. SBOM generation (Phase release gate) captures the full closure. The
net supply-chain risk is accepted: these crates are well-audited, widely used in
production Rust services, and have no known critical advisories as of 2026-06-07.

## Consequences

### Positive

- Watch-mode and preview become first-class async services without blocking threads.
- axum 0.8 WebSocket handling is idiomatic and well-tested.
- reqwest with `rustls` feature enables cross-platform HTTP (no OpenSSL on musl/Windows).
- OTLP export unlocks structured distributed tracing for CI pipeline performance
  analysis when the `otel` feature is enabled.
- crossterm `event-stream` makes interactive keypress handling non-blocking.

### Negative / Trade-offs

- tokio adds ~40–60 transitive deps (all audited, `cargo deny` enforced).
- Compilation time increases; mitigated by curated feature set (no `full`).
- Pure-core crates must remain tokio-free: this constraint requires vigilance when
  adding new functionality to pure-core modules. Architecture reviews and the
  purity-boundary-map.md must be consulted before any pure-core crate dep changes.
- `#[tokio::test]` is the test macro for async test bodies in slideforge-preview and
  slideforge-data; synchronous tests in pure-core crates continue to use `#[test]`.
- The OTLP initialization API changed significantly in opentelemetry-rust 0.32
  (new builder pattern replaces `new_pipeline()...install_batch()`). Any future
  upgrade of the OTel crates must re-verify the initialization sequence against the
  new builder API.

## Alternatives Considered

- **Synchronous threading bridge for axum**: Run `axum::serve` inside `std::thread::spawn`
  with a `tokio::runtime::Builder::new_multi_thread().build()` per-thread runtime.
  Rejected: non-idiomatic, introduces two runtimes, makes graceful shutdown coordination
  complex. The correct approach is one runtime owned by the CLI entrypoint.

- **async-std**: Not tokio-compatible; axum and reqwest both require tokio. Rejected.

- **smol + blocking threads for axum**: smol is executor-agnostic in principle but axum
  0.8 has explicit tokio trait bounds (`tokio::net::TcpListener`). Rejected.

- **Keep ureq for watch-mode HTTP, use blocking thread**: Workable for STORY-056 in
  isolation, but does not compose with the preview server (STORY-047) already requiring
  tokio. Having two HTTP crates (ureq + reqwest) in the workspace for different
  callers adds unnecessary complexity. Unified on reqwest. ureq is removed from
  `slideforge-data` when STORY-056 ships.

## Source / Origin

- Human decision recorded 2026-06-07 during Wave-5 remove-uncertainty pass.
- Registry-verified versions as of 2026-06-07 by research-agent. MSRV ≤ 1.88 confirmed
  for all listed crates.
- Consuming stories: STORY-047 (preview server), STORY-056 (watch-mode HTTP + keypress).
- OTLP telemetry surfaces: future story (otel cargo feature, wave TBD).
