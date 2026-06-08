//! Error types for the root `slideforge` crate.
//!
//! Three error enums are defined here:
//!
//! - [`PluginError`] — errors arising from the plugin dispatch boundary (`catch_unwind`).
//! - [`BuildError`] — errors arising anywhere in the `build()` pipeline: registry
//!   assembly, parsing, evaluation, layout, or export.
//!
//! [`RegistryError`] is defined in `slideforge-plugin-api` and re-exported here
//! for callers that only depend on `slideforge` (not `slideforge-plugin-api` directly).
//!
//! ## Traceability
//!
//! - BC-5.02.001 edge case EC-003: plugin panic → [`PluginError::PluginPanic`]
//! - STORY-049 AC-008: panic caught at dispatch boundary → `Err(PluginError::PluginPanic)`

use std::sync::Arc;

use slideforge_plugin_api::Diagnostic;

// Re-export RegistryError so callers of `slideforge` don't need to depend on
// `slideforge-plugin-api` just to match on registry errors.
pub use slideforge_plugin_api::RegistryError;

// ──────────────────────────────────────────────────────────────────────────────
// PluginError
// ──────────────────────────────────────────────────────────────────────────────

/// Error returned when a plugin operation fails at the dispatch boundary.
///
/// The most important variant is [`PluginError::PluginPanic`]: if a registered
/// plugin panics during execution, `std::panic::catch_unwind` in `dispatch.rs`
/// catches it and converts it into this error. The slideforge process does NOT
/// crash (BC-5.02.001 edge case EC-003, STORY-049 AC-008).
///
/// ## Non-exhaustive
///
/// New dispatch failure modes may be added in future versions.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// A plugin panicked during execution.
    ///
    /// `plugin_name` is the `id()` of the plugin that panicked.
    /// `message` is the panic payload serialized to a string (best-effort).
    #[error("plugin '{plugin_name}' panicked: {message}")]
    PluginPanic {
        /// The `id()` of the plugin that panicked.
        plugin_name: Arc<str>,
        /// The panic message, extracted from the panic payload (best-effort).
        message: Arc<str>,
    },
}

// ──────────────────────────────────────────────────────────────────────────────
// BuildError
// ──────────────────────────────────────────────────────────────────────────────

/// Error returned by [`crate::build`].
///
/// Wraps errors from each stage of the pipeline: registry assembly, brand
/// loading, parsing, evaluation, layout, and export.
///
/// ## Non-exhaustive
///
/// New pipeline stages may add variants. All four export formats (pptx, docx,
/// pdf, html) are registered in the default plugin registry.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// The plugin registry could not be assembled (a required surface is missing).
    ///
    /// This wraps [`RegistryError::MissingSurface`] from `slideforge-plugin-api`.
    #[error("plugin registry error: {0}")]
    Registry(#[from] RegistryError),

    /// No brand source was provided in [`crate::BuildOptions::brand_source`].
    ///
    /// The `build()` pipeline requires a brand configuration to load colors,
    /// fonts, and layout geometry. Callers must set
    /// `BuildOptions::brand_source` to a [`slideforge_plugin_api::BrandSource`].
    #[error(
        "no brand source configured; set BuildOptions::brand_source to a BrandSource (TomlFile, \
         PptxFile, or DocxFile)"
    )]
    NoBrandSource,

    /// No `BrandProvider` plugin is registered for the requested brand source.
    ///
    /// This typically indicates the brand provider with id
    /// `"slideforge-brand/default"` (the `BrandLoader`) was not registered
    /// during plugin assembly. It should never occur with the default bundled
    /// plugin set (which always registers `BrandLoader` under its canonical id).
    #[error(
        "no BrandProvider plugin found for the requested brand source; \
         ensure register_bundled_plugins was called and the BrandSource variant is supported"
    )]
    NoBrandProvider,

    /// The brand source could not be loaded.
    ///
    /// Wraps [`slideforge_plugin_api::BrandError`].
    #[error("brand loading failed: {0}")]
    Brand(#[source] slideforge_plugin_api::BrandError),

    /// The DSL source could not be parsed.
    ///
    /// Contains the structured diagnostics collected in the diagnostic sink
    /// during parsing. Each entry carries a `file:line:col` span, an error code
    /// from the error taxonomy, a human-readable message, and an optional
    /// correction hint (as required by CLAUDE.md error-handling mandate).
    ///
    /// The `count` field is the number of diagnostics for ergonomic Display.
    #[error("parse failed with {count} error(s); inspect `diagnostics` for file:line:col details")]
    ParseFailed {
        /// Structured parse diagnostics with spans and error codes.
        diagnostics: Vec<slideforge_syntax::BoxDiagnostic>,
        /// Cached count of diagnostics (equals `diagnostics.len()`).
        count: usize,
    },

    /// The parsed AST could not be evaluated into a semantic `Deck`.
    ///
    /// Contains the structured diagnostics accumulated in the evaluation
    /// diagnostic sink. Callers can inspect these for evaluation-phase errors
    /// (undefined variables, type mismatches, unknown slide types).
    #[error("evaluation failed with {count} error(s); inspect `diagnostics` for details")]
    EvalFailed {
        /// Structured evaluation diagnostics.
        diagnostics: Vec<slideforge_syntax::BoxDiagnostic>,
        /// Cached count of diagnostics.
        count: usize,
    },

    /// One or more validators reported `Error`-severity diagnostics and
    /// `BuildOptions::strict` was `true`.
    ///
    /// `diagnostics` contains **all** diagnostics from all validators — both
    /// Error-severity and Warning-severity entries are included so that nothing
    /// is silently dropped. The strict-failure decision is made by checking
    /// whether any `Error`-severity diagnostic is present; `count` reflects
    /// only the number of `Error`-severity entries.
    ///
    /// In `strict=false` (warn-only) mode all diagnostics are emitted as
    /// `tracing::warn` events and the build continues.
    ///
    /// ## Traceability
    ///
    /// - ADR-016 Decision 3: validate stage in the pipeline
    /// - STORY-049 C3, HIGH-2
    #[error("validation failed with {count} error(s); run with --warn-only to demote to warnings")]
    ValidationFailed {
        /// All diagnostics from all registered validators (Error + Warning + Info).
        ///
        /// Never a strict subset — all collected diagnostics are carried so
        /// callers can show the complete picture to the user.
        diagnostics: Vec<Diagnostic>,
        /// Count of `Error`-severity diagnostics only (not total diagnostics).
        ///
        /// Equals `diagnostics.iter().filter(|d| d.severity == Error).count()`.
        /// Used in the `Display` message.
        count: usize,
    },

    /// Both the evaluator and one or more validators reported `Error`-severity
    /// diagnostics in the same strict-mode build run.
    ///
    /// This variant is produced when `compile_inner` accumulates non-fatal eval
    /// errors (e.g. `E-EVL-001` undefined variable — evaluator returns `Some(Deck)`
    /// rather than `None`) AND the subsequent validator stage also produces
    /// `Error`-severity diagnostics (e.g. `E-A11-001` missing alt text).
    ///
    /// Both groups of diagnostics are carried so that the CLI (and any other
    /// caller) can render ALL errors in a single pass — satisfying
    /// BC-1.15.002 PC1 ("all N errors reported in a single build output") and
    /// invariant 3 ("accumulation applies to parser, evaluator, validators
    /// COLLECTIVELY").
    ///
    /// ## Rendering
    ///
    /// The CLI uses `eval_sort_keys` (parallel to `eval_diagnostics`) and
    /// `slideforge_plugin_api::Diagnostic::span` to merge-sort the two sets in
    /// source-file order (ascending `(file, line, col)`) at render time.
    /// This satisfies BC-1.15.002 PC2 (HIGH-P3-001 fix).
    ///
    /// `eval_severities` (parallel to `eval_diagnostics`) carries the ACTUAL
    /// [`slideforge_syntax::ParseSeverity`] of each eval diagnostic. The JSON
    /// renderer uses this to emit the correct `"severity"` field instead of
    /// hardcoding `"error"` for all eval entries (OBS-P4-003 fix).
    ///
    /// ## Exit code
    ///
    /// Exit code 2 — same as `EvalFailed` and `ValidationFailed`.
    ///
    /// ## Traceability
    ///
    /// - BC-1.15.002 PC1, PC2, invariant 3
    /// - STORY-055 AC-006 / F-P2-MED-001 fix
    /// - HIGH-P3-001 fix (source-order interleaving)
    /// - OBS-P4-003 fix (eval severity fidelity in JSON output)
    #[error(
        "build failed with {eval_count} eval error(s) and {validator_count} validator \
         error(s); run with --warn-only to demote to warnings"
    )]
    MultistageFailed {
        /// Eval-stage diagnostics (from `DiagnosticSink` — `Box<dyn miette::Diagnostic>`).
        ///
        /// Contains all non-fatal eval errors (e.g. `E-EVL-001` undefined variable)
        /// that were accumulated when `eval_deck_with_variant` returned `Some(Deck)`.
        ///
        /// Sorted by `(file, line, col)` ascending before being placed here
        /// (HIGH-P3-001 fix in `compile_inner`).
        eval_diagnostics: Vec<slideforge_syntax::BoxDiagnostic>,

        /// Source positions `(file, line, col)` parallel to `eval_diagnostics`.
        ///
        /// `eval_sort_keys[i]` is the sort key for `eval_diagnostics[i]`.
        /// Used by the CLI renderer to interleave eval and validator diagnostics
        /// in source-file order (BC-1.15.002 PC2 / HIGH-P3-001 fix).
        ///
        /// Avoids the need to downcast `BoxDiagnostic` at render time.
        eval_sort_keys: Vec<(String, u32, u32)>,

        /// Per-diagnostic severity parallel to `eval_diagnostics`.
        ///
        /// `eval_severities[i]` is the [`slideforge_syntax::ParseSeverity`] for
        /// `eval_diagnostics[i]`, captured at push time from the `DiagnosticSink`.
        /// Always the same length as `eval_diagnostics` and `eval_sort_keys`.
        ///
        /// Used by the JSON renderer to emit the ACTUAL `"severity"` field (e.g.
        /// `"warning"`, `"error"`) for each eval entry rather than hardcoding
        /// `"error"` (OBS-P4-003 fix).
        eval_severities: Vec<slideforge_syntax::ParseSeverity>,

        /// Validator-stage diagnostics (from all registered `Validator` plugins).
        ///
        /// Contains all diagnostics (Error + Warning) from Stage 5 + Stage 6b
        /// validators — same set that `ValidationFailed` would carry.
        ///
        /// Sorted by `(span.file, span.line, span.col)` ascending and deduped
        /// (BC-1.15.002 PC2, EC-004) before being placed here.
        validator_diagnostics: Vec<Diagnostic>,

        /// Count of `Error`-severity diagnostics in `eval_diagnostics`.
        eval_count: usize,

        /// Count of `Error`-severity diagnostics in `validator_diagnostics`.
        validator_count: usize,
    },

    /// The `Deck` could not be laid out into a `LaidOutDeck`.
    ///
    /// Wraps [`slideforge_layout::LayoutError`].
    #[error("layout failed: {0}")]
    Layout(#[source] slideforge_layout::LayoutError),

    /// No exporter is registered for the requested format.
    ///
    /// `format` is the format string from [`crate::BuildOptions::format`]
    /// (or `"pptx"` if none was set).
    #[error("no exporter registered for format '{0}'; supported: pptx, docx, pdf, html")]
    UnknownFormat(String),

    /// The export stage failed to produce output bytes.
    ///
    /// Wraps [`slideforge_plugin_api::ExportError`].
    ///
    /// `#[source]` ensures `Error::source()` chains through to the inner
    /// `ExportError`, which preserves the full error chain for observability.
    /// Matches the pattern of `Brand(#[source]..)` and `Layout(#[source]..)`.
    #[error("export error: {0}")]
    Export(#[source] slideforge_plugin_api::ExportError),

    /// A plugin panicked during the pipeline.
    #[error("plugin dispatch error: {0}")]
    Plugin(#[from] PluginError),
}
