//! [`SlideType`] trait — defines a visual slide pattern and its layout algorithm.
//!
//! A `SlideType` plugin maps a slide-type keyword (e.g., `"title"`, `"bullets"`,
//! `"chart"`) to:
//! 1. A set of required and optional field definitions ([`FieldDef`])
//! 2. The PPTX layout name to inherit from
//! 3. A layout algorithm that positions content on a [`Canvas`]
//!
//! All 34 built-in slide types are implemented as `SlideType` plugins
//! (31 original + `status`, `progress_bar`, `weighted_composite` added in STORY-087).
//! External plugins can register additional slide types without modifying the core.

use std::sync::Arc;

use slideforge_layout::LaidOutSlide;
use slideforge_types::{Brand, Emu, SLIDE_HEIGHT, SLIDE_WIDTH, Slide, Value};
use thiserror::Error;

/// Schema-level type constraint for a [`FieldDef`].
///
/// `FieldType` declares what runtime [`Value`] variant is valid for a field.
/// It is checked by `validate_fields` at Stage 5 (pre-layout) via
/// [`type_matches`]. A mismatch produces an E-VAL-104 diagnostic.
///
/// `None` on [`FieldDef::expected_type`] is semantically identical to
/// `Some(FieldType::Any)` — both skip type checking for that field.
///
/// # ADR-020
///
/// Defined per ADR-020 Decision 1. `#[non_exhaustive]` allows future variants
/// (e.g., `Format(FormatKind)` for T3 format validation) without breaking
/// existing exhaustive match arms outside this crate.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Accept any [`Value`] variant — type checking is skipped. Behaviorally
    /// identical to `expected_type: None`. Use for polymorphic fields.
    Any,
    /// Accepts only [`Value::Str`].
    Str,
    /// Accepts only [`Value::Int`].
    Int,
    /// Accepts only [`Value::Float`]. Note: `Value::Int` is NOT accepted —
    /// authors must use explicit float literals (e.g., `1.0`, not `1`).
    Float,
    /// Accepts only [`Value::Bool`].
    Bool,
    /// Accepts only [`Value::List`].
    List,
    /// Accepts only [`Value::Map`].
    Map,
    /// Accepts only `Value::Str(s)` where `s` is in the allowlist.
    ///
    /// For non-`Str` values, T1 type-mismatch fires before the allowlist check.
    /// The inner `Vec` holds the permitted string values as `Arc<str>`.
    OneOf(Vec<Arc<str>>),
}

impl FieldType {
    /// Returns a human-readable display name for use in E-VAL-104 messages.
    ///
    /// The returned string matches the message format specified in BC-1.18.001
    /// postconditions 2 and 3 (e.g., `"integer"`, `"string"`, `"boolean"`).
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Any => "any",
            // `OneOf` and `Str` both display as "string": OneOf constrains the allowlist
            // of strings, so the T1 type-mismatch message always reports "expected string".
            // T2 (disallowed value) has its own message branch in validate_fields.
            Self::Str | Self::OneOf(_) => "string",
            Self::Int => "integer",
            Self::Float => "float",
            Self::Bool => "boolean",
            Self::List => "list",
            Self::Map => "map",
        }
    }
}

/// Returns a human-readable name for the runtime [`Value`] variant.
///
/// Used in E-VAL-104 T1 messages to describe the actual value type received,
/// e.g., `"got string"`, `"got integer"`.
#[must_use]
pub fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Str(_) => "string",
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::Bool(_) => "boolean",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Null => "null",
    }
}

/// Check whether a runtime [`Value`] satisfies a [`FieldType`] schema constraint.
///
/// This is a **pure function** with no side effects, no I/O, and no global state.
/// It is fully determined by its two arguments, making it Kani-amenable for
/// formal verification in Phase 6 (ADR-020 Decision 5, BC-1.18.001 postcondition 8).
///
/// # Return value
///
/// Returns `true` when `value` is compatible with `expected`; `false` otherwise.
/// `validate_fields` emits E-VAL-104 when this function returns `false`.
///
/// # Semantics
///
/// - [`FieldType::Any`] always returns `true`.
/// - [`FieldType::OneOf`]: returns `true` only when `value` is `Value::Str(s)`
///   AND `s` is in the allowlist. For non-Str values, returns `false` (T1 fires).
/// - All other variants match exactly by discriminant — no implicit coercion.
///   `Value::Int` on a `FieldType::Float` field returns `false` (BC-1.18.001 Invariant 3).
///
/// # STUB — STORY-089
///
/// STUB: STORY-089 — returns `true` unconditionally; implementer adds real type matching.
///
/// Deliberately returns `true` for all inputs so that the behavioural tests
/// (which expect `false` for mismatched types) FAIL at the Red Gate.
///
/// Per LESSON-17: `todo!()` is forbidden here because it would panic during
/// test execution, which could cause `#[should_panic]` tests to pass spuriously.
/// A constant `true` return makes all "mismatch → false" tests fail (Red Gate held).
/// Implementer: replace this body with the real match expression in T4.
#[must_use]
#[allow(unused_variables)] // STORY-089 stub: args unused until implementer fills body
pub fn type_matches(value: &Value, expected: &FieldType) -> bool {
    // STUB: STORY-089 — returns true unconditionally; implementer adds real type matching.
    // This makes all "wrong type → should be false" tests fail RED as required.
    true
}

/// Describes a content field accepted by a [`SlideType`].
///
/// `FieldDef` is used by the validator and IDE tooling to provide completion and
/// type-checking for slide fields in `.sf` source files.
///
/// # Construction
///
/// External callers MUST use [`FieldDef::new`] or [`FieldDef::with_type`].
/// Struct literal construction is prevented by `#[non_exhaustive]`.
///
/// Internal code in `slideforge-plugin-api` may use struct literals, but MUST
/// include `expected_type` explicitly (the compiler enforces this).
///
/// # ADR-020
///
/// `#[non_exhaustive]` applied per ADR-020 Decision 3. `expected_type` added
/// per ADR-020 Decision 2.
#[non_exhaustive]
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

    /// Schema-level type constraint for this field.
    ///
    /// `None` is semantically identical to `Some(FieldType::Any)`: the
    /// type-checking arm in `validate_fields` is skipped for this field.
    /// Set `Some(FieldType::T)` to enforce that the runtime `Value` variant
    /// matches `T`; a mismatch emits E-VAL-104.
    ///
    /// Per ADR-020 Decision 2: `None` ≡ `Any` semantics.
    pub expected_type: Option<FieldType>,
}

impl FieldDef {
    /// Construct a `FieldDef` with no type constraint (`expected_type: None`).
    ///
    /// This is the stable constructor for all fields that do not require a
    /// specific type annotation. `expected_type: None` means type checking is
    /// skipped for this field (equivalent to `Some(FieldType::Any)`).
    ///
    /// # Arguments
    ///
    /// * `name` — field name as it appears in the `.sf` source
    /// * `description` — human-readable description for error messages and IDE
    /// * `required` — whether the field must be present
    /// * `default_value` — optional default used when the field is absent
    #[must_use]
    pub fn new(
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        required: bool,
        default_value: Option<Value>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required,
            default_value,
            expected_type: None,
        }
    }

    /// Construct a `FieldDef` with an explicit type constraint.
    ///
    /// Use this constructor when the field must hold a specific [`Value`]
    /// variant. `validate_fields` will emit E-VAL-104 when the runtime value
    /// does not match `expected_type`.
    ///
    /// # Arguments
    ///
    /// * `name` — field name as it appears in the `.sf` source
    /// * `description` — human-readable description
    /// * `required` — whether the field must be present
    /// * `default_value` — optional default used when the field is absent
    /// * `expected_type` — the required `FieldType` constraint
    #[must_use]
    pub fn with_type(
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        required: bool,
        default_value: Option<Value>,
        expected_type: FieldType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required,
            default_value,
            expected_type: Some(expected_type),
        }
    }
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
///
/// `#[non_exhaustive]` allows adding variants in minor releases without
/// breaking downstream plugin authors or consumer crates.
#[non_exhaustive]
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
            expected_type: None,
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
            expected_type: None,
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
