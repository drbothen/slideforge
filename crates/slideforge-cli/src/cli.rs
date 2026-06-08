//! CLI argument structures for the `slideforge` binary.
//!
//! Defines the root [`Cli`] struct, [`GlobalFlags`] (inherited by all
//! subcommands), the [`Command`] enum, [`BuildArgs`], and [`OutputFormat`].
//!
//! All types use `clap` 4.6 derive macros.  The `Command` variants for
//! `Watch`, `Init`, `ExtractBrand`, `Config`, and `Package` are stub
//! forwarding types defined in their respective future stories; their
//! `Args` structs are declared here as empty placeholders to allow the
//! CLI struct to compile before those stories are implemented.

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

use crate::commands::{extract_brand::ExtractBrandArgs, init::InitArgs, watch::WatchArgs};

// ── Cli ──────────────────────────────────────────────────────────────────────

/// The root CLI struct for the `slideforge` binary.
///
/// Parses global flags and dispatches to a [`Command`] subcommand.
///
/// # Example
///
/// ```text
/// slideforge build deck.sf --format pptx
/// slideforge build deck.sf --warn-only --output-dir out/
/// slideforge --json build deck.sf
/// ```
#[derive(Debug, Parser)]
#[command(
    name = "slideforge",
    version,
    about = "Compile .sf files into branded presentations"
)]
pub struct Cli {
    /// Global flags shared across all subcommands.
    #[command(flatten)]
    pub global: GlobalFlags,

    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

// ── GlobalFlags ───────────────────────────────────────────────────────────────

/// Global flags inherited by all `slideforge` subcommands.
///
/// These flags must be provided before the subcommand name on the command line,
/// or after it when the flag is declared `global = true` in clap.
// The struct holds 5 boolean flags because the CLI spec (STORY-055) defines
// exactly these global flags.  Using a bitfield or enum would break clap derive.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args, Clone)]
pub struct GlobalFlags {
    /// Emit diagnostics as JSON on stderr instead of human-readable text.
    ///
    /// When set, `DiagnosticSink::to_json()` output is written to stderr and
    /// ANSI color rendering is suppressed.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress all non-error output to stdout.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Increase log verbosity.
    ///
    /// Specify multiple times for higher verbosity (`-v`, `-vv`, `-vvv`).
    /// Maps to tracing log levels: 0=warn, 1=info, 2=debug, 3+=trace.
    #[arg(long, short = 'v', global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Disable ANSI color output.
    ///
    /// Also respected via the `NO_COLOR` environment variable per the
    /// <https://no-color.org/> convention.
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,

    /// Treat validation errors as warnings; do not fail the build on
    /// Error-severity diagnostics.
    ///
    /// Note: parse errors (E-PAR-*) and export errors (E-EXP-*) are NEVER
    /// demoted by `--warn-only` (BC-1.15.003 invariants 1 and 2).
    #[arg(long, global = true)]
    pub warn_only: bool,

    /// Disable all network access (data sources, package fetching).
    #[arg(long, global = true)]
    pub offline: bool,

    /// Send tracing spans to an OpenTelemetry OTLP endpoint.
    ///
    /// Requires the `otel` Cargo feature to be enabled at compile time.
    /// When absent, the default `tracing-subscriber` fmt layer is used.
    /// Per ADR-021, the `opentelemetry_sdk` `rt-tokio` feature requires the
    /// tokio async runtime.
    #[arg(long, global = true, value_name = "URL")]
    pub otel_endpoint: Option<String>,
}

// ── Command ───────────────────────────────────────────────────────────────────

/// Top-level subcommands for the `slideforge` binary.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile a `.sf` file into one or more output formats.
    Build(BuildArgs),

    /// Watch a `.sf` file for changes and rebuild automatically (STORY-056).
    Watch(WatchArgs),

    /// Initialize a new slideforge project (STORY-057).
    Init(InitArgs),

    /// Extract brand configuration from an existing PPTX or DOCX file (STORY-057).
    ExtractBrand(ExtractBrandArgs),
}

// ── BuildArgs ─────────────────────────────────────────────────────────────────

/// Arguments for the `slideforge build` subcommand.
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to the `.sf` source file to compile.
    pub source: PathBuf,

    /// Output directory for compiled artifacts.
    ///
    /// Defaults to `dist/` relative to the current working directory.
    #[arg(long, default_value = "dist")]
    pub output_dir: PathBuf,

    /// Comma-separated list of output formats to produce.
    ///
    /// Defaults to all four formats: `pptx,docx,pdf,html`.
    ///
    /// Valid values: `pptx`, `docx`, `pdf`, `html`.
    #[arg(long, value_delimiter = ',', default_values_t = all_formats())]
    pub format: Vec<OutputFormat>,

    /// Select a named deck variant defined with `variant:` in the source.
    #[arg(long)]
    pub variant: Option<String>,
}

// ── OutputFormat ──────────────────────────────────────────────────────────────

/// Output format selection for the `--format` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum OutputFormat {
    /// Microsoft `PowerPoint` `.pptx` format.
    Pptx,
    /// Microsoft Word `.docx` format.
    Docx,
    /// Portable Document Format `.pdf`.
    Pdf,
    /// `HyperText` Markup Language `.html` format.
    Html,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Pptx => write!(f, "pptx"),
            OutputFormat::Docx => write!(f, "docx"),
            OutputFormat::Pdf => write!(f, "pdf"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the default set of output formats: all four supported formats.
///
/// Used as the `default_values_t` for `BuildArgs::format`.
#[must_use]
pub fn all_formats() -> Vec<OutputFormat> {
    vec![
        OutputFormat::Pptx,
        OutputFormat::Docx,
        OutputFormat::Pdf,
        OutputFormat::Html,
    ]
}
