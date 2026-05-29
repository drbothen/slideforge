//! `LayoutError` — errors returned by [`crate::layout::run`].
//!
//! ## Field-naming convention (interface-definitions.md §8 / BC-3.04.001 item L)
//!
//! All variants include a **`source_slide_index: usize`** where applicable so
//! that error messages can point to the offending slide by position in the
//! input [`slideforge_types::Deck`].
//!
//! The canonical field name is **`source_slide_index`** — not `slide_index`,
//! `idx`, `slide_idx`, or any other spelling. This is the source of truth for
//! all `LayoutError` variant authoring.

use thiserror::Error;

use crate::types::BoundingBox;
use slideforge_types::SourceSpan;

/// Errors produced by the layout engine.
///
/// `LayoutError` is returned by [`crate::layout::run`] when an internal
/// invariant cannot be satisfied. Validation errors (missing required fields,
/// alt-text absent, etc.) are caught by the validation stage before layout
/// runs — they are NOT represented here.
///
/// All variants implement `Debug + Clone + PartialEq + Eq + Hash` for
/// comemo compatibility and proptest `Arbitrary` derivability (AC-010).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
pub enum LayoutError {
    /// The input `Deck` contains zero slides.
    ///
    /// A zero-slide deck cannot produce any layout output. The validation stage
    /// (STORY-016) should have rejected this before layout runs, but layout
    /// provides a defensive check per EC-001.
    ///
    /// `source_slide_index` is always `0` for this variant (there are no slides
    /// to index into), but is included for structural consistency with other
    /// variants and to satisfy the field-uniformity requirement.
    #[error(
        "layout error: input deck contains zero slides (source_slide_index: {source_slide_index})"
    )]
    EmptyDeck {
        /// Always `0` — included for structural consistency.
        source_slide_index: usize,
    },

    /// The number of slides in the produced `LaidOutDeck` does not match the
    /// number of slides in the input `Deck`.
    ///
    /// This indicates an internal bug in the layout engine — a slide was
    /// accidentally skipped or duplicated. Per AC-002 / BC-3.06.001, the
    /// transformation MUST preserve slide count exactly.
    #[error("layout error: slide count mismatch — expected {expected} slides, produced {actual}")]
    SlideCountMismatch {
        /// The number of slides in the input `Deck`.
        expected: usize,
        /// The number of slides actually produced by the layout engine.
        actual: usize,
        /// Index of the first slide where divergence was detected.
        source_slide_index: usize,
    },

    /// A slide's type keyword has no registered region map.
    ///
    /// Every slide type keyword known to the registry must have a corresponding
    /// region map. If an unregistered or unknown keyword reaches layout, this
    /// error is returned (EC-002).
    #[error("layout error: slide {source_slide_index}: unknown slide type '{slide_type_keyword}'")]
    UnknownSlideType {
        /// Zero-based index of the offending slide in `Deck.slides`.
        source_slide_index: usize,
        /// The slide type keyword that has no region map.
        slide_type_keyword: String,
    },

    /// A manually authored `section <type>:` block has an unrecognised type name.
    ///
    /// Supported section types are: `executive_summary`, `risk_register`,
    /// `methodology`, `scope`, `approval`, `appendix`, `glossary`.
    /// `executive_summary` and `risk_register` are allowed as manual overrides
    /// (AC-006 / BC-3.02.001 EC-002).  Any other name produces this error
    /// (AC-004 / BC-3.02.002 EC-001).
    #[error(
        "layout error: unknown section type '{name}' at {span}. \
         Known types: [executive_summary, risk_register, methodology, scope, \
         approval, appendix, glossary]"
    )]
    UnknownSectionType {
        /// The unrecognised section type name from the `.sf` source.
        name: String,
        /// Source location of the unrecognised `section <type>:` block.
        span: SourceSpan,
    },

    /// A `BoundingBox` in the produced layout has invalid coordinates.
    ///
    /// Per AC-014 / BC-3.06.003, every bounding box must satisfy:
    /// `x >= 0`, `y >= 0`, `width > 0`, `height > 0`,
    /// `x + width <= page_width`, `y + height <= page_height`.
    #[error(
        "layout error: slide {source_slide_index} frame {frame_index}: invalid bounding box {bbox:?}"
    )]
    InvalidBoundingBox {
        /// Zero-based index of the slide containing the invalid frame.
        source_slide_index: usize,
        /// Zero-based index of the frame within the slide.
        frame_index: usize,
        /// The offending bounding box.
        bbox: BoundingBox,
    },

    /// A `takeaway:` field value on a slide is not a resolved plain string.
    ///
    /// The layout stage expects the evaluator to have resolved all
    /// `FieldValue::Expr` and `FieldValue::Interpolated` values before layout
    /// runs. If an unresolved variant is encountered on a `takeaway:` field,
    /// this error is returned (HIGH-002 / BC-3.02.001).
    #[error(
        "layout error: slide {source_slide_index}: takeaway field is unresolved \
         (expected Literal(Str), found a non-literal FieldValue variant)"
    )]
    UnresolvedTakeaway {
        /// Zero-based index of the slide with the unresolved takeaway.
        source_slide_index: usize,
    },

    /// A required field is missing from a risk card entry.
    ///
    /// Each entry in the `cards:` list of a `severity_cards` slide must be a
    /// `Value::Map` containing `title`, `severity`, `description`, and `owner`
    /// keys. When a key is absent or not a plain string, this error is returned
    /// (HIGH-001 / BC-3.02.001).
    #[error(
        "layout error: slide {source_slide_index}: risk card at index {card_index} is missing \
         required field '{field}' (or it is not a plain string)"
    )]
    MissingRiskCardField {
        /// Zero-based index of the `severity_cards` slide.
        source_slide_index: usize,
        /// Zero-based index of the card within the slide's `cards:` list.
        card_index: usize,
        /// The name of the missing or non-string field.
        field: String,
    },

    /// The `cards:` field on a `severity_cards` slide is present but holds a
    /// wrong-typed `Literal` value (not a `List`).
    ///
    /// This is a type-error in the .sf source — `cards:` must be a YAML-style
    /// list of maps, not a scalar or map at the top level (HIGH-003).
    #[error(
        "layout error: slide {source_slide_index}: 'cards' field has wrong type — expected List, \
         found a non-List Literal value: {reason}"
    )]
    MalformedSeverityCards {
        /// Zero-based index of the `severity_cards` slide.
        source_slide_index: usize,
        /// Human-readable description of the actual type found.
        reason: String,
    },

    /// The `cards:` field on a `severity_cards` slide is an unresolved
    /// non-Literal `FieldValue` (e.g., `Expr`, `Interpolated`, `Inlines`).
    ///
    /// The evaluator must resolve all field values before layout runs.
    /// An unresolved `cards:` field indicates an evaluator bug (HIGH-003).
    #[error(
        "layout error: slide {source_slide_index}: 'cards' field is an unresolved FieldValue variant \
         (expected Literal(List)); this indicates an evaluator bug"
    )]
    UnresolvedSeverityCards {
        /// Zero-based index of the `severity_cards` slide.
        source_slide_index: usize,
    },

    /// A `shape:` block specifies an unknown shape type keyword (BC-3.04.001
    /// invariant 4 / E-PAR-012-SHP / F-MED-005).
    ///
    /// The closed v1.0 vocabulary is: `rect`, `ellipse`, `arrow`, `line`, `star`,
    /// `roundRect`. Any other keyword produces this error. There is NO `Custom`
    /// fallback — the type system enforces the closed vocabulary.
    ///
    /// `span` points to the offending `shape:` block in the source file.
    #[error(
        "layout error: slide {source_slide_index}: unknown shape type '{shape_type}' at {span}. \
         Known types: [rect, ellipse, arrow, line, star, roundRect]"
    )]
    UnknownShapeType {
        /// The unrecognised shape type keyword from the `.sf` source.
        shape_type: String,
        /// Zero-based index of the slide containing the offending `shape:` block.
        source_slide_index: usize,
        /// Source location of the offending `shape:` block.
        span: SourceSpan,
    },

    /// Arithmetic overflow in EMU conversion (BC-3.04.001 Invariant 8 / VP-048).
    ///
    /// `from_inches` and `from_em` use `checked_mul` (per BC-3.04.001 Invariant 8
    /// / interface-definitions.md §9.4) and return `None` on overflow. When any of
    /// the four position fields (x, y, width, height) overflows, `layout_shapes`
    /// accumulates this error via the multi-error accumulation pattern (DI-018).
    ///
    /// An `i64::MAX`-class input exceeds any physically meaningful slide dimension
    /// by many orders of magnitude; the correct production behaviour is to reject
    /// it explicitly rather than silently clamp (VP-048).
    #[error(
        "layout error: slide {source_slide_index}: arithmetic overflow in EMU conversion at {span}"
    )]
    ArithmeticOverflow {
        /// Zero-based index of the slide containing the overflowing shape.
        source_slide_index: usize,
        /// Source location of the value that overflowed.
        span: SourceSpan,
    },

    /// Inline node nesting exceeded the maximum safe depth (BC-3.05.001 E-LAY-005 / F-MED-006).
    ///
    /// The maximum allowed inline nesting depth is
    /// [`crate::inline::MAX_INLINE_DEPTH`] (64). Deeper nesting is rejected at
    /// layout time to prevent stack overflow in recursive traversal.
    #[error(
        "layout error: slide {source_slide_index}: inline nesting depth {depth} exceeds maximum \
         ({max}) — simplify the formatting nesting"
    )]
    InlineDepthExceeded {
        /// Zero-based index of the slide containing the over-nested inline content.
        source_slide_index: usize,
        /// The actual depth that was detected (>= `max`).
        depth: usize,
        /// The maximum allowed depth ([`crate::inline::MAX_INLINE_DEPTH`]).
        max: usize,
    },

    /// A `Shape` node reached the layout stage without `alt` text or
    /// `decorative: true` (BC-3.04.001 EC-001 / DI-001).
    ///
    /// This is a defensive check — the primary alt-text enforcement is in
    /// `slideforge-validate` (STORY-015). If this error fires, it indicates
    /// the validation stage was bypassed or produced a false-pass.
    ///
    /// Layout returns this error rather than produce a shape without alt text,
    /// because WCAG-AA compliance requires that every non-decorative visual
    /// element have programmatically-determinable alternative text.
    ///
    /// `span` points to the offending `shape:` block in the source file
    /// (BC-3.04.001 invariant 7 / CLAUDE.md error-handling rule: all errors
    /// carry source spans).
    #[error(
        "layout error: slide {source_slide_index}: shape node reached layout without alt text or \
         `decorative: true` at {span} (internal invariant violation — validation should have caught this)"
    )]
    MissingAlt {
        /// Zero-based index of the slide containing the shape without alt text.
        source_slide_index: usize,
        /// Source location of the offending `shape:` block.
        span: SourceSpan,
    },

    /// Multiple `LayoutError`s accumulated from a single operation (BC-3.04.001 item G).
    ///
    /// Used by [`crate::shapes::layout_shapes`] to accumulate all `MissingAlt`
    /// and `ArithmeticOverflow` errors from a slide's shape set before returning.
    /// This variant allows callers that accept `Result<_, LayoutError>` to receive
    /// all errors at once rather than bailing on the first failure (DI-018).
    ///
    /// ## Invariants
    ///
    /// - The inner `Vec` MUST be non-empty. An empty `Multiple([])` is a layout
    ///   engine bug — use `Ok(...)` when there are no errors.
    /// - `Multiple` MUST NOT be nested: inner errors are flat `LayoutError`
    ///   variants, never another `Multiple`.
    /// - Even a single error is returned as `Multiple { inner: vec![err] }` for
    ///   uniform return type. Use [`LayoutError::multiple`] to construct.
    ///
    /// ## Display
    ///
    /// Displays the count and the first error's message. Full inspection requires
    /// matching the variant and iterating the inner `Vec`.
    #[error(
        "layout error: {count} accumulated errors; first: {first}",
        count = inner.len(),
        first = inner.first().map_or_else(|| "(none)".to_owned(), std::string::ToString::to_string)
    )]
    Multiple {
        /// The accumulated errors, in source order.
        inner: Vec<LayoutError>,
    },
}

impl LayoutError {
    /// Construct a `Multiple` error with flattening and non-empty invariant.
    ///
    /// This is the canonical constructor for `Multiple` — callers MUST use it
    /// instead of constructing `LayoutError::Multiple { inner: ... }` directly.
    ///
    /// # Invariants enforced
    ///
    /// - `errors` MUST be non-empty (`debug_assert!` guards this in debug builds).
    /// - Nested `Multiple` variants are flattened into a single level.
    ///
    /// # Panics (debug builds only)
    ///
    /// Asserts that `errors` is non-empty. An empty `Multiple` is a layout-engine
    /// bug — use `Ok(...)` when there are no errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_layout::error::LayoutError;
    /// let err = LayoutError::multiple(vec![
    ///     LayoutError::EmptyDeck { source_slide_index: 0 },
    /// ]);
    /// // Single error → Multiple { inner: [EmptyDeck] } (uniform).
    /// assert!(matches!(err, LayoutError::Multiple { .. }));
    /// ```
    #[must_use]
    pub fn multiple(errors: Vec<Self>) -> Self {
        debug_assert!(!errors.is_empty(), "LayoutError::multiple requires at least one error");
        // Flatten nested Multiple variants into a single level (invariant: no nesting).
        let flattened: Vec<Self> = errors
            .into_iter()
            .flat_map(|e| match e {
                LayoutError::Multiple { inner } => inner,
                other => vec![other],
            })
            .collect();
        LayoutError::Multiple { inner: flattened }
    }
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod tests {
    use super::*;
    use crate::types::{BoundingBox, Emu};

    // ────────────────────────────────────────────────────────────────────────
    // BC-3.06.NNN — error variant construction tests
    // ────────────────────────────────────────────────────────────────────────

    /// BC-3.06.001 — `LayoutError::EmptyDeck` can be constructed and displays a
    /// meaningful message.
    #[test]
    fn test_bc_3_06_001_error_empty_deck_variant_exists() {
        let err = LayoutError::EmptyDeck {
            source_slide_index: 0,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("zero slides"),
            "EmptyDeck message must mention 'zero slides'; got: {msg}"
        );
    }

    /// BC-3.06.001 — `LayoutError::SlideCountMismatch` can be constructed and
    /// displays expected/actual counts.
    #[test]
    fn test_bc_3_06_001_error_slide_count_mismatch_variant_exists() {
        let err = LayoutError::SlideCountMismatch {
            expected: 3,
            actual: 2,
            source_slide_index: 0,
        };
        let msg = err.to_string();
        assert!(
            msg.contains('3') && msg.contains('2'),
            "SlideCountMismatch message must include counts; got: {msg}"
        );
    }

    /// BC-3.06.002 — `LayoutError::UnknownSlideType` carries the keyword and
    /// slide index.
    #[test]
    fn test_bc_3_06_002_error_unknown_slide_type_variant_exists() {
        let err = LayoutError::UnknownSlideType {
            source_slide_index: 5,
            slide_type_keyword: "frobnicator".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("frobnicator"),
            "UnknownSlideType message must include keyword; got: {msg}"
        );
        assert!(
            msg.contains('5'),
            "UnknownSlideType message must include slide index; got: {msg}"
        );
    }

    /// BC-3.06.003 — `LayoutError::InvalidBoundingBox` carries slide/frame
    /// indices and the bbox.
    #[test]
    fn test_bc_3_06_003_error_invalid_bounding_box_variant_exists() {
        let bbox = BoundingBox {
            x: Emu(-1),
            y: Emu(0),
            width: Emu(1_000_000),
            height: Emu(500_000),
        };
        let err = LayoutError::InvalidBoundingBox {
            source_slide_index: 2,
            frame_index: 0,
            bbox,
        };
        let msg = err.to_string();
        assert!(
            msg.contains('2'),
            "InvalidBoundingBox message must include slide index; got: {msg}"
        );
    }

    /// AC-010 — `LayoutError` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_bc_3_06_001_layout_error_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let err = LayoutError::EmptyDeck {
            source_slide_index: 0,
        };
        let err2 = err.clone();
        assert_eq!(err, err2);

        let mut set = HashSet::new();
        set.insert(LayoutError::EmptyDeck {
            source_slide_index: 0,
        });
        set.insert(LayoutError::EmptyDeck {
            source_slide_index: 0,
        }); // duplicate
        assert_eq!(set.len(), 1);
    }

    /// BC-3.04.001 item F — `LayoutError::MissingAlt` carries `source_slide_index`
    /// and `span` fields (interface-definitions §8.1 canonical field name).
    #[test]
    fn test_bc_3_04_001_missing_alt_carries_source_slide_index_and_span() {
        let err = LayoutError::MissingAlt {
            source_slide_index: 4,
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("alt text") || msg.contains("decorative"),
            "MissingAlt message must mention alt text; got: {msg}"
        );
        assert!(
            msg.contains('4'),
            "MissingAlt message must include source_slide_index; got: {msg}"
        );
        // Verify Clone + PartialEq + Eq + Hash hold.
        let err2 = err.clone();
        assert_eq!(err, err2);
    }

    /// BC-3.04.001 item G — `LayoutError::Multiple` accumulates inner errors
    /// and displays the count and first error message.
    #[test]
    fn test_bc_3_04_001_multiple_variant_carries_inner_errors() {
        use std::collections::HashSet;

        let inner = vec![
            LayoutError::MissingAlt {
                source_slide_index: 0,
                span: SourceSpan::default(),
            },
            LayoutError::MissingAlt {
                source_slide_index: 0,
                span: SourceSpan::default(),
            },
        ];
        let err = LayoutError::Multiple {
            inner: inner.clone(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains('2'),
            "Multiple display must include the error count; got: {msg}"
        );

        // Clone + PartialEq + Eq + Hash
        let err2 = err.clone();
        assert_eq!(err, err2);

        let mut set = HashSet::new();
        set.insert(err);
        assert_eq!(set.len(), 1);
    }

    /// interface-definitions §8.1 — renamed variants use `source_slide_index`,
    /// NOT `slide_index`. This test constructs all four renamed variants to
    /// ensure the canonical field name is enforced at compile time.
    #[test]
    fn test_interface_definitions_s8_canonical_field_name_source_slide_index() {
        // MissingAlt
        let _ = LayoutError::MissingAlt {
            source_slide_index: 0,
            span: SourceSpan::default(),
        };
        // MissingRiskCardField
        let _ = LayoutError::MissingRiskCardField {
            source_slide_index: 1,
            card_index: 0,
            field: "title".to_owned(),
        };
        // MalformedSeverityCards
        let _ = LayoutError::MalformedSeverityCards {
            source_slide_index: 2,
            reason: "expected List".to_owned(),
        };
        // UnresolvedSeverityCards
        let _ = LayoutError::UnresolvedSeverityCards {
            source_slide_index: 3,
        };
        // If this test compiles, the canonical field name is correctly applied.
    }
}
