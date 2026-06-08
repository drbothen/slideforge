//! `slideforge-cli` — public library surface for integration testing.
//!
//! This crate provides the `slideforge` binary. The library surface exposed
//! here is intentionally minimal: it exists so that integration tests in
//! `tests/` can import types without invoking `main()`.
//!
//! # Architecture
//!
//! Per the STORY-055 architecture compliance rules, `slideforge-cli` is an
//! **effectful shell** crate. It performs I/O (stdout, stderr, file writes,
//! TTY detection) but contains NO pipeline logic. All compile logic lives in
//! the root `slideforge` crate or specialist crates; the CLI is an
//! orchestration façade only.
//!
//! # Forbidden dependencies
//!
//! - Must NOT import `chumsky` directly (parsing is SS-01's domain).
//! - Must NOT import `ooxmlsdk`, `pdf-writer`, or `krilla` (export is
//!   SS-06/07/08/09's domain).
//! - Must NOT import internal crates below `slideforge`; the CLI talks to the
//!   root crate only.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod cli;
pub mod commands;
pub mod exit_code;
pub mod output;
pub mod tracing_setup;
