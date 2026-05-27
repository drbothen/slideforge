//! [`SlideType`] trait — defines a visual slide pattern and its layout algorithm.
//!
//! A `SlideType` plugin maps a slide-type keyword (e.g., `"title"`, `"bullets"`,
//! `"chart"`) to:
//! 1. A set of required and optional field definitions ([`FieldDef`])
//! 2. The PPTX layout name to inherit from
//! 3. A layout algorithm that positions content on a [`Canvas`]
//!
//! All 31 built-in slide types are implemented as `SlideType` plugins. External
//! plugins can register additional slide types without modifying the core.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Emu, SLIDE_HEIGHT, SLIDE_WIDTH, Slide, Value};
use thiserror::Error;

/// Describes a content field accepted by a [`SlideType`].
///
/// `FieldDef` is used by the validator and IDE tooling to provide completion and
/// type-checking for slide fields in `.sf` source files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldDef {
    /// The field name as it appears in the `.sf` source (e.g., `"title"`,
    /// `"bullets"`, `"chart"`).
    pub name: Arc<str>,

    /// A human-readable description of what the field contains. Used in error
    /// messages and IDE completions.
    pub description: Arc<str>,

    /// When `true`, the field must be present; the compiler emits an error if
    /// it is absent. When `false`, the field may be omitted.
    pub required: bool,

    /// Default value used when the field is absent and `required` is `false`.
    /// `None` means no default (field is simply absent).
    pub default_value: Option<Value>,
}

/// The slide canvas dimensions passed to [`SlideType::lay_out`].
///
/// The layout engine creates a `Canvas` from the brand's configured slide
/// dimensions (typically 16:9 widescreen). Plugins should produce positions
/// within `[0, width) × [0, height)` in EMU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Canvas {
    /// Slide width in English Metric Units.
    pub width: Emu,

    /// Slide height in English Metric Units.
    pub height: Emu,
}

impl Default for Canvas {
    /// Returns the standard widescreen slide canvas (16:9, 13.33 × 7.5 inches).
    fn default() -> Self {
        Self {
            width: SLIDE_WIDTH,
            height: SLIDE_HEIGHT,
        }
    }
}

/// Error returned by [`SlideType::lay_out`].
#[derive(Debug, Error)]
pub enum LayoutError {
    /// A required field is missing from the slide.
    #[error("slide type '{slide_type}': required field '{field}' is missing")]
    MissingRequiredField {
        /// The slide type keyword.
        slide_type: String,
        /// The name of the missing field.
        field: String,
    },

    /// A field value has the wrong type for this slide type.
    #[error(
        "slide type '{slide_type}': field '{field}' expected {expected_type}, got {actual_type}"
    )]
    FieldTypeMismatch {
        /// The slide type keyword.
        slide_type: String,
        /// The field name.
        field: String,
        /// The expected type name.
        expected_type: String,
        /// The actual type name from the value.
        actual_type: String,
    },

    /// The content does not fit the canvas (text overflow or dimension violation).
    #[error("layout overflow in slide type '{slide_type}': {message}")]
    LayoutOverflow {
        /// The slide type keyword.
        slide_type: String,
        /// Description of the overflow condition.
        message: String,
    },

    /// An internal layout error occurred.
    #[error("layout internal error for '{slide_type}': {message}")]
    InternalError {
        /// The slide type keyword.
        slide_type: String,
        /// Description of the internal error.
        message: String,
    },
}

/// A plugin that defines a slide type's field schema and layout algorithm.
///
/// `SlideType` plugins provide three capabilities:
/// 1. **Schema** — `required_fields` and `optional_fields` define what the
///    compiler accepts in a `slide:` block with this type keyword.
/// 2. **PPTX layout name** — `layout_name` maps this type to the OOXML slide
///    layout to inherit from.
/// 3. **Layout algorithm** — `lay_out` produces a [`LaidOutSlide`] from a
///    [`Slide`] by computing EMU positions for every content element.
///
/// Register implementations with [`crate::PluginRegistry::register_slide_type`].
///
/// ## Thread safety
///
/// All implementations must be `Send + Sync`.
pub trait SlideType: Send + Sync {
    /// A unique identifier for this slide type, matching the keyword used in
    /// `.sf` source (e.g., `"title"`, `"bullets"`, `"chart-bar"`).
    fn id(&self) -> &'static str;

    /// The set of required content fields for this slide type.
    ///
    /// The compiler emits an error for every required field that is absent from
    /// a `slide:` block.
    fn required_fields(&self) -> &[FieldDef];

    /// The set of optional content fields for this slide type.
    ///
    /// Optional fields may have default values; the compiler uses the default
    /// when the field is absent.
    fn optional_fields(&self) -> &[FieldDef];

    /// The PPTX slide layout name that this slide type inherits from.
    ///
    /// The layout name must match a layout defined in the brand's PPTX template
    /// or one of the 11 standard OOXML layout names.
    fn layout_name(&self) -> &'static str;

    /// Compute the geometric layout for `slide` on the given `canvas`.
    ///
    /// Returns a [`LaidOutSlide`] with all content elements positioned in EMU.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError`] when required fields are missing, field types
    /// are wrong, or content overflows the canvas.
    fn lay_out(
        &self,
        slide: &Slide,
        brand: &Brand,
        canvas: Canvas,
    ) -> Result<LaidOutSlide, LayoutError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn test_bc_5_02_008_slide_type_trait_is_send_sync() {
        assert_send_sync::<dyn SlideType>();
    }

    #[test]
    fn test_bc_5_02_008_canvas_default_is_slide_dimensions() {
        let canvas = Canvas::default();
        assert_eq!(canvas.width, SLIDE_WIDTH);
        assert_eq!(canvas.height, SLIDE_HEIGHT);
    }

    #[test]
    fn test_bc_5_02_008_canvas_custom_dimensions() {
        let canvas = Canvas {
            width: Emu(9_144_000),
            height: Emu(6_858_000),
        };
        assert_eq!(canvas.width, Emu(9_144_000));
        assert_eq!(canvas.height, Emu(6_858_000));
    }

    #[test]
    fn test_bc_5_02_008_canvas_implements_copy() {
        let c1 = Canvas::default();
        let c2 = c1; // Copy — no move
        let c3 = c1; // Still usable
        assert_eq!(c2, Canvas::default());
        assert_eq!(c3, Canvas::default());
    }

    #[test]
    fn test_bc_5_02_008_field_def_required() {
        let field = FieldDef {
            name: Arc::from("title"),
            description: Arc::from("The slide title"),
            required: true,
            default_value: None,
        };
        assert!(field.required);
        assert!(field.default_value.is_none());
    }

    #[test]
    fn test_bc_5_02_008_field_def_optional_with_default() {
        let field = FieldDef {
            name: Arc::from("background"),
            description: Arc::from("Override background color"),
            required: false,
            default_value: Some(Value::Null),
        };
        assert!(!field.required);
        assert!(field.default_value.is_some());
    }

    #[test]
    fn test_bc_5_02_008_layout_error_missing_required_field() {
        let err = LayoutError::MissingRequiredField {
            slide_type: "bullets".to_owned(),
            field: "title".to_owned(),
        };
        assert!(err.to_string().contains("required field"));
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn test_bc_5_02_008_layout_error_field_type_mismatch() {
        let err = LayoutError::FieldTypeMismatch {
            slide_type: "chart-bar".to_owned(),
            field: "data".to_owned(),
            expected_type: "list".to_owned(),
            actual_type: "string".to_owned(),
        };
        assert!(err.to_string().contains("expected list, got string"));
    }

    #[test]
    fn test_bc_5_02_008_layout_error_layout_overflow() {
        let err = LayoutError::LayoutOverflow {
            slide_type: "bullets".to_owned(),
            message: "bullet list exceeds canvas height".to_owned(),
        };
        assert!(err.to_string().contains("layout overflow"));
    }

    #[test]
    fn test_bc_5_02_008_layout_error_internal_error() {
        let err = LayoutError::InternalError {
            slide_type: "title".to_owned(),
            message: "font metrics unavailable".to_owned(),
        };
        assert!(err.to_string().contains("internal error"));
    }
}
