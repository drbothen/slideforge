//! Plugin trait API for slideforge — the 10 extension surfaces.
//!
//! This crate declares the public extension contracts that every slideforge
//! plugin implements. Both bundled plugins (shipped with slideforge) and
//! external plugins (written by third parties) implement these traits.
//!
//! ## The 10 Plugin Surfaces
//!
//! | Surface | Trait | Purpose |
//! |---------|-------|---------|
//! | 1 | [`DataSource`] | Fetch external data (JSON, CSV, HTTP, `SQLite`, …) |
//! | 2 | [`Exporter`] | Produce output format bytes (PPTX, DOCX, PDF, HTML) |
//! | 3 | [`ChartRenderer`] | Render `ChartSpec` → SVG bytes |
//! | 4 | [`DiagramRenderer`] | Render diagram source (Mermaid) → SVG bytes |
//! | 5 | [`Validator`] | Check `Deck` for content and accessibility issues |
//! | 6 | [`MathRenderer`] | Render `MathNode` (LaTeX) → OMML or `MathML` bytes |
//! | 7 | [`BrandProvider`] | Load brand configuration from TOML/PPTX/DOCX |
//! | 8 | [`SlideType`] | Define slide field schema and layout algorithm |
//! | 9 | [`SectionType`] | Generate document section structure from slides |
//! | 10 | [`InlineFormat`] | Render inline nodes to OOXML/HTML/Markdown |
//!
//! ## Dog-fooding Guarantee (BC-5.02.002)
//!
//! All bundled slideforge plugins implement these traits using ONLY this crate
//! and `slideforge-types`. No bundled plugin bypasses the trait API. This is
//! enforced by the `tests/dog_food_test.rs` integration test, which compiles a
//! minimal plugin using only the public API surface.
//!
//! ## Thread Safety
//!
//! All traits are `Send + Sync`. The [`PluginRegistry`] is also `Send + Sync`
//! and is intended to be wrapped in `Arc<PluginRegistry>` for shared access
//! from multiple pipeline threads.
//!
//! ## No Dynamic Plugin Loading
//!
//! All v1.0 plugins are statically compiled. There is no WASM loader or
//! `libloading`-based dynamic dispatch in this crate. The `PluginRegistry`
//! stores `Box<dyn Trait + Send + Sync>` which is sufficient for static dispatch
//! to plugin implementations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod registry;
pub mod traits;

// Re-export everything at the crate root for ergonomic usage.
// Consumers can write `use slideforge_plugin_api::DataSource;` without
// the module path.
pub use registry::PluginRegistry;
pub use traits::{
    BrandError, BrandProvider, BrandSource, ChartError, ChartRenderer, DataSource, DataSourceError,
    DataSourceOptions, DiagramError, DiagramOptions, DiagramRenderer, Diagnostic,
    DiagnosticSeverity, ExportError, ExportOptions, Exporter, FieldDef, InlineError, InlineFormat,
    InlineOutputFormat, LayoutError, MathError, MathOutputFormat, MathRenderer, SectionBlock,
    SectionType, SlideType, ValidatorError, ValidatorOptions, Validator,
    Canvas,
};

// Compile-time assertion: PluginRegistry is Send + Sync.
// This function is never called; it exists solely to prove the bound at
// compile time and satisfy AC-014.
#[allow(dead_code)]
fn assert_registry_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRegistry>();
}
