//! `LayoutError` — errors returned by [`crate::layout::run`].
//!
//! All variants include a `source_slide_index: usize` where applicable so that
//! error messages can point to the offending slide by position in the input
//! [`slideforge_types::Deck`].

use thiserror::Error;

use crate::types::BoundingBox;

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
    /// Supported section types are: `methodology`, `scope`, `approval`,
    /// `appendix`, `glossary`. Any other name produces this error (AC-004 /
    /// BC-3.02.002).
    #[error("layout error: unknown section type '{name}'")]
    UnknownSectionType {
        /// The unrecognised section type name from the `.sf` source.
        name: String,
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
        "layout error: slide {slide_index}: risk card at index {card_index} is missing \
         required field '{field}' (or it is not a plain string)"
    )]
    MissingRiskCardField {
        /// Zero-based index of the `severity_cards` slide.
        slide_index: usize,
        /// Zero-based index of the card within the slide's `cards:` list.
        card_index: usize,
        /// The name of the missing or non-string field.
        field: String,
    },
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
}
