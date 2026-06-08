//! `slideforge` — command-line interface entry point.
//!
//! Parses command-line arguments via `clap`, initializes the `tracing`
//! subscriber, and dispatches to the appropriate subcommand handler.
//!
//! # Architecture (STORY-055)
//!
//! This binary is an **effectful shell** crate.  It performs I/O (stdin,
//! stdout, stderr, file writes, TTY detection) but contains NO pipeline
//! logic.  All compile logic lives in the root `slideforge` crate or
//! specialist crates.
//!
//! # Exit code model (BC-1.15.003)
//!
//! | Condition | Code |
//! |-----------|------|
//! | Success | 0 |
//! | Parse error (E-PAR-*) | 1 |
//! | Validation error in strict mode | 2 |
//! | Export error (E-EXP-*) | 3 |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

use clap::Parser as _;
use slideforge_cli::{cli::Cli, commands, tracing_setup};

fn main() -> ExitCode {
    // Parse CLI arguments.  clap handles --help / --version and exits early.
    let cli = Cli::parse();

    // Initialize the tracing subscriber.  Errors here are non-fatal: if
    // initialization fails (e.g., because a test already installed one) we
    // log a warning to stderr and continue.
    //
    // The guard MUST be kept alive across `commands::dispatch` — when the
    // `otel` feature is active it owns the Tokio runtime that backs the OTLP
    // batch exporter.  Dropping it early would shut down the runtime before
    // spans finish exporting.
    let _tracing_guard = match tracing_setup::init_tracing(&cli.global) {
        Ok(guard) => guard,
        Err(e) => {
            // Use eprintln! here only — this is main() (not a library crate).
            eprintln!("warning: could not initialize tracing subscriber: {e}");
            // Return a no-op guard so dispatch still proceeds.
            tracing_setup::TracingGuard::default()
        },
    };

    // Dispatch to the appropriate subcommand handler.
    // `_tracing_guard` is dropped after dispatch returns, which flushes any
    // queued OTel spans before the process exits.
    commands::dispatch(&cli)
}
