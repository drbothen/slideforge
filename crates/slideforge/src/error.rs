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
/// New pipeline stages (e.g., HTML exporter in STORY-050) may add variants.
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

    /// The `Deck` could not be laid out into a `LaidOutDeck`.
    ///
    /// Wraps [`slideforge_layout::LayoutError`].
    #[error("layout failed: {0}")]
    Layout(#[source] slideforge_layout::LayoutError),

    /// No exporter is registered for the requested format.
    ///
    /// `format` is the format string from [`crate::BuildOptions::format`]
    /// (or `"pptx"` if none was set).
    #[error("no exporter registered for format '{0}'; supported: pptx, docx, pdf")]
    UnknownFormat(String),

    /// The export stage failed to produce output bytes.
    ///
    /// Wraps [`slideforge_plugin_api::ExportError`].
    #[error("export error: {0}")]
    Export(slideforge_plugin_api::ExportError),

    /// A plugin panicked during the pipeline.
    #[error("plugin dispatch error: {0}")]
    Plugin(#[from] PluginError),
}
