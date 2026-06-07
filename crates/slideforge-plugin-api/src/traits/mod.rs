//! Plugin trait modules — one module per extensibility surface.
//!
//! This module re-exports all trait and supporting type definitions for the
//! 10 plugin surfaces. Consumers can import directly from the surface modules
//! or use the re-exports at the crate root via `slideforge_plugin_api::*`.

pub mod brand_provider;
pub mod chart_renderer;
pub mod data_source;
pub mod diagram_renderer;
pub mod exporter;
pub mod inline_format;
pub mod math_renderer;
pub mod section_type;
pub mod slide_type;
pub mod validator;

// Flatten all public items from each trait module so crate root re-exports
// can use a single `pub use traits::*;` style import.
pub use brand_provider::{BrandError, BrandProvider, BrandSource};
pub use chart_renderer::{ChartError, ChartRenderer};
pub use data_source::{DataSource, DataSourceError, DataSourceOptions};
pub use diagram_renderer::{DiagramError, DiagramOptions, DiagramRenderer};
pub use exporter::{ExportError, ExportOptions, Exporter};
pub use inline_format::{InlineError, InlineFormat, InlineOutputFormat, InlineRenderContext};
pub use math_renderer::{MathError, MathOutputFormat, MathRenderer};
pub use section_type::{SectionBlock, SectionType};
pub use slide_type::{Canvas, FieldDef, FieldType, LayoutError, SlideType, type_matches, value_type_name};
pub use validator::{Diagnostic, DiagnosticSeverity, Validator, ValidatorOptions};
