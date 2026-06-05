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
/// Wraps errors from each stage of the pipeline: registry assembly, parsing,
/// evaluation, layout, and export.
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

    /// The DSL source could not be parsed.
    ///
    /// Wraps `slideforge_syntax::ParseError` — concrete type TBD when
    /// `slideforge-syntax` exposes its error type (STORY-002).
    #[error("parse error: {0}")]
    Parse(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// The parsed AST could not be evaluated into a semantic `Deck`.
    ///
    /// Wraps `slideforge_eval::EvalError` (STORY-003).
    #[error("eval error: {0}")]
    Eval(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// The `Deck` could not be laid out into a `LaidOutDeck`.
    ///
    /// Wraps `slideforge_layout::LayoutError` (STORY-015).
    #[error("layout error: {0}")]
    Layout(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// The export stage failed to produce output bytes.
    ///
    /// Wraps `slideforge_plugin_api::ExportError`.
    #[error("export error: {0}")]
    Export(#[from] slideforge_plugin_api::ExportError),

    /// A plugin panicked during the pipeline.
    #[error("plugin dispatch error: {0}")]
    Plugin(#[from] PluginError),
}
