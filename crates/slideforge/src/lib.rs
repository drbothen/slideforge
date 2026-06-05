//! # slideforge
//!
//! Compile a structured DSL into branded presentations (PPTX, DOCX, PDF, HTML).
//!
//! This crate is the root assembly point and public API entry point for the
//! slideforge pipeline. It:
//!
//! 1. Re-exports the plugin registry API from `slideforge-plugin-api` so
//!    callers only need one `use slideforge::*` import.
//! 2. Provides `build()` — the high-level pipeline entry point:
//!    `parse → eval → layout → export`.
//! 3. Wires together all bundled plugin implementations via
//!    [`registry::register_bundled_plugins`].
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use slideforge::{BuildOptions, BuildOutput, build};
//! use slideforge_plugin_api::BrandSource;
//! use std::sync::Arc;
//!
//! let source = r#"
//! slide: title
//!   title: "Hello, slideforge"
//! "#;
//!
//! let options = BuildOptions {
//!     brand_source: Some(BrandSource::TomlFile(Arc::from("brand.toml"))),
//!     ..Default::default()
//! };
//! let output: BuildOutput = build(source, &options)?;
//! # Ok::<(), slideforge::error::BuildError>(())
//! ```
//!
//! ## Architecture
//!
//! Per ADR-016 Decision 3, this crate is the **pipeline driver** — not merely
//! an assembly facade. The `build()` function wires the four pipeline stages:
//!
//! ```text
//! slideforge-syntax → slideforge-eval → slideforge-layout → (exporter via plugin)
//! ```
//!
//! All cross-crate plugin interaction goes through `Box<dyn Trait>` dispatch.
//! No plugin logic is implemented in this crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod dispatch;
pub mod error;
pub mod registry;

// ── Re-export the plugin registry API ─────────────────────────────────────────

/// Re-export of [`slideforge_plugin_api::PluginRegistry`].
///
/// Callers that only depend on `slideforge` can use this type without a direct
/// dependency on `slideforge-plugin-api`.
pub use slideforge_plugin_api::PluginRegistry;

/// Re-export of [`slideforge_plugin_api::PluginRegistryBuilder`].
pub use slideforge_plugin_api::PluginRegistryBuilder;

/// Re-export of [`slideforge_plugin_api::RegistryError`].
pub use slideforge_plugin_api::RegistryError;

// ── Public pipeline types ─────────────────────────────────────────────────────

/// Options passed to [`build`].
///
/// Controls output format, brand configuration, validation mode, and other
/// pipeline knobs.
#[derive(Debug, Default, Clone)]
pub struct BuildOptions {
    /// Output format identifier (e.g., `"pptx"`, `"pdf"`, `"docx"`).
    ///
    /// If `None`, the pipeline defaults to `"pptx"`.
    pub format: Option<String>,

    /// Brand configuration source.
    ///
    /// The `build()` pipeline requires a [`slideforge_plugin_api::BrandSource`]
    /// to load brand colors, fonts, and layout geometry. If `None`, `build()`
    /// returns [`error::BuildError::NoBrandSource`].
    ///
    /// Supported sources:
    /// - [`slideforge_plugin_api::BrandSource::TomlFile`] — load from a
    ///   slideforge brand TOML file.
    /// - [`slideforge_plugin_api::BrandSource::PptxFile`] — extract brand from
    ///   an existing PPTX template.
    /// - [`slideforge_plugin_api::BrandSource::DocxFile`] — extract brand from
    ///   an existing DOCX template.
    pub brand_source: Option<slideforge_plugin_api::BrandSource>,

    /// If `true`, validation warnings are treated as errors.
    pub strict: bool,
}

/// Output produced by [`build`].
///
/// Contains the rendered file bytes and metadata.
#[derive(Debug, Clone)]
pub struct BuildOutput {
    /// The rendered file bytes (e.g., a `.pptx` or `.pdf` file).
    pub bytes: Vec<u8>,

    /// The file extension for the rendered output (e.g., `"pptx"`, `"pdf"`).
    pub extension: String,
}

// ── Public pipeline entry point ───────────────────────────────────────────────

/// Compile a slideforge DSL source string into a rendered output.
///
/// This is the primary library API. It runs the full pipeline:
///
/// 1. Assemble the plugin registry (all 10 bundled surfaces).
/// 2. Parse `source` into an AST (`slideforge-syntax`).
/// 3. Evaluate the AST into a semantic `Deck` IR (`slideforge-eval`).
/// 4. Lay out the `Deck` into a [`slideforge_layout::LaidOutDeck`]
///    (`slideforge-layout`).
/// 5. Export the `LaidOutDeck` using the configured exporter plugin.
///
/// ## Brand configuration
///
/// A brand source must be supplied via [`BuildOptions::brand_source`].
/// If none is provided, `build()` returns [`error::BuildError::NoBrandSource`].
///
/// ## Errors
///
/// Returns [`error::BuildError`] if any stage fails.
///
/// ## Example
///
/// ```rust,no_run
/// use slideforge::{BuildOptions, build};
/// use slideforge_plugin_api::BrandSource;
/// use std::sync::Arc;
///
/// let opts = BuildOptions {
///     brand_source: Some(BrandSource::TomlFile(Arc::from("brand.toml"))),
///     ..Default::default()
/// };
/// let output = build("slide: title\n  title: \"Hello\"\n", &opts)?;
/// std::fs::write("output.pptx", &output.bytes)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn build(source: &str, options: &BuildOptions) -> Result<BuildOutput, error::BuildError> {
    use slideforge_eval::{EvalConfig, eval_deck};
    use slideforge_layout::run as layout_run;
    use slideforge_plugin_api::ExportOptions;
    use slideforge_syntax::{DiagnosticSink, SourceMap, parse_checked};

    // Stage 1: assemble the plugin registry.
    let registry = registry::default_registry()?;

    // Stage 2: load brand via the BrandProvider plugin.
    let brand_source = options
        .brand_source
        .as_ref()
        .ok_or(error::BuildError::NoBrandSource)?;
    let brand_provider = registry
        .lookup_brand_provider("file")
        .ok_or(error::BuildError::NoBrandProvider)?;
    let brand = brand_provider
        .load(brand_source)
        .map_err(error::BuildError::Brand)?;

    // Stage 3: parse the DSL source.
    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(std::sync::Arc::from("<build>"), std::sync::Arc::from(source));
    let mut sink = DiagnosticSink::new();
    let deck_node = parse_checked(source, file_id, &source_map, &mut sink)
        .ok_or_else(|| error::BuildError::ParseFailed(sink.errors().len()))?;

    // Stage 4: evaluate the AST into a semantic Deck.
    let eval_config = EvalConfig::default();
    let deck = eval_deck(&deck_node, &eval_config, &mut sink)
        .ok_or(error::BuildError::EvalFailed)?;

    // Stage 5: lay out the Deck into a LaidOutDeck.
    let laid_out = layout_run(&deck, &brand).map_err(error::BuildError::Layout)?;

    // Stage 6: select the exporter and produce output bytes.
    let format = options.format.as_deref().unwrap_or("pptx");
    let exporter = registry
        .lookup_exporter(format)
        .ok_or_else(|| error::BuildError::UnknownFormat(format.to_owned()))?;
    let export_opts = ExportOptions::default();
    let bytes = exporter
        .export(&deck, &laid_out, &brand, &export_opts)
        .map_err(error::BuildError::Export)?;

    Ok(BuildOutput {
        bytes,
        extension: format.to_owned(),
    })
}
