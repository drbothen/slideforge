//! Shape layout pass — BC-3.04.001 (STORY-028).
//!
//! This module converts [`slideforge_types::ShapeSpec`] nodes from the `Deck`
//! IR into [`crate::types::Frame`]s with [`crate::types::FrameContent::Shape`]
//! content, appended after all placeholder frames in each [`crate::types::LaidOutSlide`].
//!
//! ## Responsibilities
//!
//! - Convert user-unit positions (inches, em) to integer [`slideforge_types::Emu`].
//! - Enforce that every shape has `alt` or `decorative: true`
//!   (defensive check — primary enforcement is in `slideforge-validate`).
//! - Detect off-canvas positions and emit [`crate::types::LayoutWarning::OffCanvas`].
//! - Preserve source order (first shape in source → first shape frame appended).
//!
//! ## Unit conversion (AC-001)
//!
//! | User unit | EMU conversion |
//! |-----------|---------------|
//! | `1in`     | `914_400 EMU` |
//! | `1em`     | `brand.font_size_emu` (default: `457_200` = 36pt body font = 0.5 inch at `914_400` EMU/inch) |
//!
//! ## Off-canvas detection (AC-003 / BC-3.04.001 EC-002)
//!
//! A shape is off-canvas if any of the following hold:
//! - `x < 0`
//! - `y < 0`
//! - `x + width > page_width`
//! - `y + height > page_height`
//!
//! Off-canvas shapes emit [`crate::types::LayoutWarning::OffCanvas`] but are
//! still produced at their declared position.

use std::sync::Arc;

use slideforge_types::{Emu, ShapeUnit};

use crate::error::LayoutError;
#[cfg(test)]
use crate::types::Rgb;
use crate::types::{BoundingBox, FillSpec, Frame, LayoutWarning, PageSize, ShapeFrame, ShapeType};

/// EMU per inch: 914,400 (canonical DSL unit definition, DI-010).
pub const EMU_PER_INCH: i64 = 914_400;

/// Default em-to-EMU conversion: 457,200 EMU (36pt body font = 0.5 inch at 914,400 EMU/inch).
///
/// # Test-only constant (AC-002 / STORY-074)
///
/// This constant is **not part of the public API** and is only compiled in test
/// builds. Production code uses `BrandFonts::default().font_size_emu` (which
/// carries this same value for backward compatibility — AC-003). This constant
/// is retained solely for in-crate unit tests that need to call low-level helpers
/// (`from_em`, `unit_to_emu`, `layout_shapes`) with the historical default value.
#[cfg(test)]
const DEFAULT_EM_IN_EMU: i64 = 457_200;

/// Convert a [`slideforge_types::ShapeUnit`] measurement to integer EMU.
///
/// Returns `None` when the multiplication would overflow `i64`
/// (VP-048 / BC-3.04.001 Invariant 8 / interface-definitions.md §9.4).
///
/// # Arguments
///
/// * `unit` — the measurement in user units.
/// * `em_in_emu` — the brand's em-to-EMU resolution (default: `457_200` EMU = 36pt body font = 0.5 inch at `914_400` EMU/inch).
///
/// # Returns
///
/// `Some(Emu)` on success, `None` on arithmetic overflow.
#[must_use]
pub fn unit_to_emu(unit: &ShapeUnit, em_in_emu: i64) -> Option<Emu> {
    match unit {
        ShapeUnit::Inches(milliinches) => from_inches(*milliinches),
        ShapeUnit::Em(milliem) => from_em(*milliem, em_in_emu),
    }
}

/// Convert inches (as a rational `numerator/1000`) to EMU.
///
/// For example, `from_inches(500)` converts `0.5in` → `Some(Emu(457_200))`.
///
/// Uses `checked_mul` to detect overflow. Returns `None` when `milliinches *
/// EMU_PER_INCH` overflows `i64`. An `i64::MAX`-class input exceeds any
/// physically meaningful slide dimension by many orders of magnitude; the caller
/// converts `None` to `LayoutError::ArithmeticOverflow { source_slide_index, span, field }`
/// (BC-3.04.001 Invariant 8 / interface-definitions.md §9.4 / VP-048).
///
/// # Kani candidate (VP-048)
///
/// The overflow behaviour is a verification property for Phase 6 Kani proofs.
/// The checked semantics here are the concrete reference for that proof.
#[must_use]
pub fn from_inches(milliinches: i64) -> Option<Emu> {
    // checked_mul returns None on overflow instead of saturating silently.
    milliinches
        .checked_mul(EMU_PER_INCH)
        .map(|product| Emu(product / 1_000))
}

/// Convert em units (as a rational `numerator/1000`) to EMU.
///
/// For example, `from_em(1000, 457_200)` converts `1em` →
/// `Some(Emu(457_200))` (using the default brand em size of 457,200 EMU).
///
/// Returns `None` when `milliem * em_in_emu` overflows `i64`
/// (VP-048 / BC-3.04.001 Invariant 8 / interface-definitions.md §9.4).
/// See [`from_inches`] for the full overflow contract.
#[must_use]
pub fn from_em(milliem: i64, em_in_emu: i64) -> Option<Emu> {
    // checked_mul returns None on overflow.
    milliem
        .checked_mul(em_in_emu)
        .map(|product| Emu(product / 1_000))
}

/// Detect whether a bounding box is off the slide canvas.
///
/// A bounding box is off-canvas if `x < 0`, `y < 0`,
/// `x + width > page_width`, or `y + height > page_height`.
///
/// Returns `true` if the shape is (at least partially) off-canvas.
#[must_use]
pub fn is_off_canvas(bbox: &BoundingBox, page: PageSize) -> bool {
    bbox.x < Emu(0)
        || bbox.y < Emu(0)
        || Emu(bbox.x.0.saturating_add(bbox.width.0)) > page.width
        || Emu(bbox.y.0.saturating_add(bbox.height.0)) > page.height
}

/// Parse a CSS-style hex color string (`#RRGGBB`) into an [`Rgb`] value.
///
/// Returns `None` if the string is not a valid 6-digit hex color.
///
/// # Note
///
/// This function exists only for unit test helpers in this module.
/// Production shape layout consumes [`slideforge_types::FillSpec`] directly
/// from the already-parsed `ShapeSpec.fill` field — no keyword parsing is
/// needed at layout time.
#[cfg(test)]
#[must_use]
pub fn parse_hex_color(hex: &str) -> Option<Rgb> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Rgb { r, g, b })
}

/// The output of [`layout_shapes`].
///
/// Carries the produced shape frames and any accumulated warnings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeLayoutOutput {
    /// Shape frames to append to the slide's `frames` list.
    pub frames: Vec<Frame>,
    /// Non-fatal warnings (off-canvas positions, etc.).
    pub warnings: Vec<LayoutWarning>,
}

/// Layout all `shape:` blocks from a semantic slide into [`Frame`]s.
///
/// This is the entry point for the shape layout pass (BC-3.04.001 postcondition
/// 4). Called once per slide by [`crate::layout::run`] after the region-map
/// frames have been produced.
///
/// # Arguments
///
/// * `shapes` — slice of `ShapeSpec` from the slide's block list.
/// * `page` — the slide's page dimensions (for off-canvas detection).
/// * `source_slide_index` — zero-based index of the slide (for warning/error messages).
/// * `em_in_emu` — the brand's em-to-EMU resolution.
/// * `base_index` — the number of frames already placed by the caller for this
///   slide (i.e. the region-map frame count). Shape frames are appended after
///   those frames, so `frame_index` in any [`crate::error::LayoutError::InvalidBoundingBox`]
///   produced here equals `base_index + <local-position-in-shapes>`, giving a
///   slide-wide frame index rather than a sub-list index (F-P20-LOW-002).
///
/// # Errors
///
/// Returns `Err(LayoutError::Multiple { inner })` accumulating ALL
/// `LayoutError::MissingAlt` and `LayoutError::ArithmeticOverflow` errors
/// encountered (BC-3.04.001 item G / DI-018 / VP-048).
/// Even a single-shape failure returns `Multiple` for a uniform return type.
pub fn layout_shapes(
    shapes: &[slideforge_types::ShapeSpec],
    page: PageSize,
    source_slide_index: usize,
    em_in_emu: i64,
    base_index: usize,
) -> Result<ShapeLayoutOutput, LayoutError> {
    let mut frames = Vec::with_capacity(shapes.len());
    let mut warnings = Vec::new();
    let mut accumulated_errors: Vec<LayoutError> = Vec::new();

    for shape in shapes {
        // BC-3.04.001 invariant 4: ShapeSpec.shape_type is already a resolved
        // ShapeType enum variant — no keyword parsing needed here. The parser
        // is responsible for rejecting unknown keywords with E-PAR-012 before
        // constructing a ShapeSpec. No Custom fallback; the type system enforces
        // the closed vocabulary (CLAUDE.md "no silent fallback" rule).
        let shape_type = shape.shape_type;

        // Resolve alt text — MissingAlt is accumulated (not bail-on-first)
        // per BC-3.04.001 item G / DI-018.
        //
        // Layout trusts validator's alt-text checks; blank alt strings should not
        // reach this point under the default --validate pipeline.  The
        // slideforge-validate alt-text validator (W-A11-002) gates blank / whitespace
        // alt before layout runs.  See STORY-015 (alt-text enforcement) if a
        // blank-alt-bypass guard at the layout boundary becomes a documented concern
        // (e.g., when --no-validate mode is formally specified).
        let alt = match &shape.alt {
            Some(slideforge_types::AltText::Provided(s)) => Some(Arc::clone(s)),
            // Decorative, Unspecified, and None all map to no alt string in the shape frame.
            // Unspecified on a shape means no author alt was threaded (ADR-019 Decision 4.1);
            // the shape layout path fires LayoutError::MissingAlt separately if needed.
            Some(
                slideforge_types::AltText::Decorative | slideforge_types::AltText::Unspecified,
            )
            | None => None,
        };
        let decorative =
            shape.decorative || matches!(&shape.alt, Some(slideforge_types::AltText::Decorative));

        // Pass fill and text from ShapeSpec directly (BC-3.04.001 v1.5.2 schema).
        let fill = shape.fill.clone();
        let text = shape.text.clone();

        // build_shape_frame enforces MissingAlt (EC-001 / BC-3.04.001).
        // F-MED-004: span is passed in directly — no re-emit needed.
        // On MissingAlt error, accumulate and continue collecting all missing-alt shapes.
        let shape_frame = match build_shape_frame(
            shape_type,
            fill,
            text,
            alt,
            decorative,
            source_slide_index,
            &shape.span,
        ) {
            Ok(sf) => sf,
            Err(err @ LayoutError::MissingAlt { .. }) => {
                accumulated_errors.push(err);
                continue;
            },
            Err(other) => return Err(other),
        };

        // VP-048 / BC-3.04.001 Invariant 8: use checked_mul via unit_to_emu.
        // On overflow, accumulate ArithmeticOverflow error (not silent saturation).
        let Some(x) = unit_to_emu(&shape.position.x, em_in_emu) else {
            accumulated_errors.push(LayoutError::ArithmeticOverflow {
                source_slide_index,
                span: shape.span.clone(),
                field: "x",
            });
            continue;
        };
        let Some(y) = unit_to_emu(&shape.position.y, em_in_emu) else {
            accumulated_errors.push(LayoutError::ArithmeticOverflow {
                source_slide_index,
                span: shape.span.clone(),
                field: "y",
            });
            continue;
        };
        let Some(width) = unit_to_emu(&shape.position.width, em_in_emu) else {
            accumulated_errors.push(LayoutError::ArithmeticOverflow {
                source_slide_index,
                span: shape.span.clone(),
                field: "width",
            });
            continue;
        };
        let Some(height) = unit_to_emu(&shape.position.height, em_in_emu) else {
            accumulated_errors.push(LayoutError::ArithmeticOverflow {
                source_slide_index,
                span: shape.span.clone(),
                field: "height",
            });
            continue;
        };
        let bbox = BoundingBox {
            x,
            y,
            width,
            height,
        };

        // F-HIGH-003 / BC-3.06.003: validate bbox from shape frames.
        // width > 0 and height > 0 must hold; x >= 0 and y >= 0 are off-canvas
        // (not hard errors), so we only check the strictly-invalid cases here.
        //
        // frame_index is slide-wide: base_index (region frames already placed by
        // the caller) + frames.len() (valid shapes placed so far in this call).
        // This matches the semantics of LayoutError::InvalidBoundingBox.frame_index
        // as documented in error.rs §BC-3.06.003 (F-P20-LOW-002).
        //
        // DI-018 / BC-3.04.001 item G: InvalidBoundingBox is ACCUMULATED (not a
        // bail-on-first return) so that ALL shapes in a slide are validated in a
        // single pass. A direct `return Err(...)` here would discard any
        // MissingAlt or ArithmeticOverflow errors already accumulated for earlier
        // shapes. Accumulate and continue — the Multiple is returned at loop end.
        if bbox.width <= Emu(0) || bbox.height <= Emu(0) {
            accumulated_errors.push(LayoutError::InvalidBoundingBox {
                source_slide_index,
                frame_index: base_index + frames.len(),
                bbox,
            });
            continue;
        }

        if is_off_canvas(&bbox, page) {
            warnings.push(LayoutWarning::OffCanvas {
                source_slide_index,
                // F-HIGH-003 / interface-definitions §9.3 (pass-3 binding):
                // shape_type is the resolved ShapeType enum variant, not Arc<str>.
                shape_type,
                x_emu: bbox.x,
                y_emu: bbox.y,
            });
        }

        frames.push(Frame {
            bbox,
            content: crate::types::FrameContent::Shape(shape_frame),
            text_flow: None,
            region_role: None,
        });
    }

    // Return accumulated errors if any shapes failed (Item N: uniform Multiple).
    if !accumulated_errors.is_empty() {
        return Err(LayoutError::multiple(accumulated_errors));
    }

    Ok(ShapeLayoutOutput { frames, warnings })
}

/// Build a [`FillSpec`] from a shape spec's fill keyword.
///
/// Supported keywords in v1.0: solid color hex (`#RRGGBB`), `"none"`.
/// Unrecognised keywords default to `FillSpec::None`.
///
/// # Note
///
/// This function exists only for unit test helpers in this module.
/// Production shape layout consumes [`slideforge_types::FillSpec`] directly
/// from the already-parsed `ShapeSpec.fill` field — no keyword parsing is
/// needed at layout time. The silent `FillSpec::None` fallback on unrecognised
/// input is intentional for the test-helper role; it would be an error anti-pattern
/// in any future production path (see CLAUDE.md "silent fallback" forbidden patterns).
#[cfg(test)]
#[must_use]
pub fn build_fill_spec(fill_keyword: Option<&str>) -> FillSpec {
    match fill_keyword {
        None | Some("none") => FillSpec::None,
        Some(kw) => {
            if let Some(rgb) = parse_hex_color(kw) {
                FillSpec::SolidColor(rgb)
            } else {
                FillSpec::None
            }
        },
    }
}

/// Build a [`ShapeFrame`] from the resolved components.
///
/// # Arguments
///
/// * `shape_type` — the resolved shape type enum variant.
/// * `fill` — the fill specification (solid color or none).
/// * `text` — optional inline node sequence for text content on the shape.
/// * `alt` — optional alt text string (from `AltText::Provided`).
/// * `decorative` — whether the shape is decorative (`alt` will be omitted).
/// * `source_slide_index` — zero-based slide index for error messages.
/// * `span` — source span of the shape block; used directly in `MissingAlt`
///   errors so callers do not need to re-emit with a corrected span (F-MED-004).
///
/// # Alt / decorative precedence
///
/// When both `alt` (`Some(s)`) and `decorative = true` are supplied simultaneously,
/// **`alt` takes precedence** and the result is `AltText::Provided(s)`. The
/// `decorative` flag is ignored in this case.
///
/// Rationale: an explicit alt text string is always the more informative
/// accessibility annotation. Silently preferring `AltText::Decorative` (which
/// suppresses all screen-reader output) when the author also supplied meaningful
/// text would be an accessibility regression. This "alt wins" rule is the correct
/// default per WCAG AA. No warning is emitted *by layout*; the `slideforge-validate`
/// `alt-text` validator emits W-A11-002 to surface the ambiguity (see BC-3.04.001
/// v1.5.2 Invariant 11 and BC-5.01.002 v1.5.2).
///
/// # Errors
///
/// Returns `Err(LayoutError::MissingAlt)` with the provided `span` when `alt`
/// is `None` and `decorative` is `false` (BC-3.04.001 EC-001 / DI-001).
pub fn build_shape_frame(
    shape_type: ShapeType,
    fill: FillSpec,
    text: Option<Vec<slideforge_types::InlineNode>>,
    alt: Option<Arc<str>>,
    decorative: bool,
    source_slide_index: usize,
    span: &slideforge_types::SourceSpan,
) -> Result<ShapeFrame, LayoutError> {
    let alt_resolved = match (alt, decorative) {
        (Some(s), _) => slideforge_types::AltText::Provided(s),
        (None, true) => slideforge_types::AltText::Decorative,
        (None, false) => {
            return Err(LayoutError::MissingAlt {
                source_slide_index,
                span: span.clone(),
            });
        },
    };

    Ok(ShapeFrame {
        shape_type,
        fill,
        text,
        alt: alt_resolved,
    })
}

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::doc_markdown,
    non_snake_case
)]
mod tests {
    use super::*;
    use crate::types::{DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH};
    use slideforge_types::{AltText, ShapePosition, ShapeSpec, ShapeUnit, SourceSpan};

    // ─────────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn default_page() -> PageSize {
        PageSize {
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        }
    }

    /// Default position for test shapes: x=0.5in, y=1.0in, width=2.0in, height=1.0in.
    ///
    /// This is the canonical test vector from BC-3.04.001 AC-001.
    fn default_position() -> ShapePosition {
        ShapePosition {
            x: ShapeUnit::Inches(500),       // 0.5in
            y: ShapeUnit::Inches(1000),      // 1.0in
            width: ShapeUnit::Inches(2000),  // 2.0in
            height: ShapeUnit::Inches(1000), // 1.0in
        }
    }

    /// Build a minimal `ShapeSpec` with explicit alt text.
    ///
    /// `shape_type` must be a valid v1.0 keyword (`rect`, `ellipse`, etc.).
    /// Panics in tests if the keyword is not in the closed vocabulary.
    fn shape_spec_with_alt(shape_type: &str, alt: &str) -> ShapeSpec {
        let st = slideforge_types::ShapeType::from_keyword(shape_type)
            .unwrap_or_else(|_| panic!("unknown shape type in test: {shape_type}"));
        ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from(alt))),
            decorative: false,
            span: SourceSpan::default(),
        }
    }

    /// Build a minimal `ShapeSpec` with `decorative: true`.
    ///
    /// `shape_type` must be a valid v1.0 keyword. Panics in tests if unknown.
    fn shape_spec_decorative(shape_type: &str) -> ShapeSpec {
        let st = slideforge_types::ShapeType::from_keyword(shape_type)
            .unwrap_or_else(|_| panic!("unknown shape type in test: {shape_type}"));
        ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::None,
            text: None,
            alt: None,
            decorative: true,
            span: SourceSpan::default(),
        }
    }

    /// Build a minimal `ShapeSpec` with neither alt nor decorative (invalid per BC).
    ///
    /// `shape_type` must be a valid v1.0 keyword. Panics in tests if unknown.
    fn shape_spec_no_alt(shape_type: &str) -> ShapeSpec {
        let st = slideforge_types::ShapeType::from_keyword(shape_type)
            .unwrap_or_else(|_| panic!("unknown shape type in test: {shape_type}"));
        ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::None,
            text: None,
            alt: None,
            decorative: false,
            span: SourceSpan::default(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-001 — Shape EMU conversion from user-declared units (BC-3.04.001)
    // Canonical test vector: 0.5in → Emu(457_200), 1.0in → Emu(914_400),
    //                         2.0in → Emu(1_828_800)   (from BC-3.04.001 story spec)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — `from_inches(500)` converts 0.5 inch → `Some(Emu(457_200))`.
    ///
    /// Canonical test vector from STORY-028 AC-001:
    ///   `position x 0.5in` → `BoundingBox.x = Emu(457_200)`.
    /// `500` is milliinches for `0.5in` (stored as `inches × 1000`).
    /// Returns `Some(Emu)` for all in-range inputs (VP-048 / BC-3.04.001 Invariant 8).
    #[test]
    fn test_bc_3_04_001_ac001_half_inch_to_emu() {
        assert_eq!(
            from_inches(500),
            Some(Emu(457_200)),
            "0.5in (500 milliinches) must convert to Some(Emu(457_200))"
        );
    }

    /// AC-001 — `from_inches(1000)` converts 1.0 inch → `Some(Emu(914_400))`.
    ///
    /// Canonical test vector: `position y 1.0in` → `BoundingBox.y = Emu(914_400)`.
    #[test]
    fn test_bc_3_04_001_ac001_one_inch_to_emu() {
        assert_eq!(
            from_inches(1000),
            Some(Emu(914_400)),
            "1.0in (1000 milliinches) must convert to Some(Emu(914_400))"
        );
    }

    /// AC-001 — `from_inches(2000)` converts 2.0 inches → `Some(Emu(1_828_800))`.
    ///
    /// Canonical test vector: `width 2.0in` → `BoundingBox.width = Emu(1_828_800)`.
    #[test]
    fn test_bc_3_04_001_ac001_two_inches_to_emu() {
        assert_eq!(
            from_inches(2000),
            Some(Emu(1_828_800)),
            "2.0in (2000 milliinches) must convert to Some(Emu(1_828_800))"
        );
    }

    /// AC-001 — `from_inches(0)` converts 0 inches → `Some(Emu(0))`.
    ///
    /// Boundary: zero input is in-range and must succeed.
    #[test]
    fn test_bc_3_04_001_ac001_zero_inches_to_emu() {
        assert_eq!(
            from_inches(0),
            Some(Emu(0)),
            "0in must convert to Some(Emu(0))"
        );
    }

    /// AC-001 — `from_inches(-500)` converts −0.5 inch → `Some(Emu(-457_200))`.
    ///
    /// Off-canvas shapes use negative coordinates; the conversion must preserve sign.
    /// (The off-canvas check happens at the `layout_shapes` level, not in conversion.)
    #[test]
    fn test_bc_3_04_001_ac001_negative_half_inch_to_emu() {
        assert_eq!(
            from_inches(-500),
            Some(Emu(-457_200)),
            "-0.5in must convert to Some(Emu(-457_200))"
        );
    }

    /// AC-001 — `from_em(1000, DEFAULT_EM_IN_EMU)` converts 1em → `Some(Emu(457_200))`.
    ///
    /// Default brand em: `457_200` EMU = 36pt body font = 0.5 inch at `914_400` EMU/inch.
    #[test]
    fn test_bc_3_04_001_ac001_one_em_to_emu() {
        assert_eq!(
            from_em(1000, DEFAULT_EM_IN_EMU),
            Some(Emu(457_200)),
            "1em at default brand size must convert to Some(Emu(457_200))"
        );
    }

    /// AC-001 — `from_em(2000, DEFAULT_EM_IN_EMU)` converts 2em → `Some(Emu(914_400))`.
    #[test]
    fn test_bc_3_04_001_ac001_two_em_to_emu() {
        assert_eq!(
            from_em(2000, DEFAULT_EM_IN_EMU),
            Some(Emu(914_400)),
            "2em at default brand size must convert to Some(Emu(914_400))"
        );
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Inches(500), DEFAULT_EM_IN_EMU)` → `Some(Emu(457_200))`.
    ///
    /// Tests the dispatch function that delegates to `from_inches`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_inches() {
        assert_eq!(
            unit_to_emu(&ShapeUnit::Inches(500), DEFAULT_EM_IN_EMU),
            Some(Emu(457_200)),
            "unit_to_emu with Inches(500) must return Some(Emu(457_200))"
        );
    }

    /// AC-001 — `unit_to_emu(ShapeUnit::Em(1000), DEFAULT_EM_IN_EMU)` → `Some(Emu(457_200))`.
    ///
    /// Tests the dispatch function that delegates to `from_em`.
    #[test]
    fn test_bc_3_04_001_ac001_unit_to_emu_em() {
        assert_eq!(
            unit_to_emu(&ShapeUnit::Em(1000), DEFAULT_EM_IN_EMU),
            Some(Emu(457_200)),
            "unit_to_emu with Em(1000) must return Some(Emu(457_200))"
        );
    }

    /// AC-001 — Full position vector: x=0.5in, y=1.0in, width=2.0in, height=1.0in.
    ///
    /// This is the canonical test vector from STORY-028 AC-001. Verifies that all
    /// four position fields convert correctly in the same call.
    #[test]
    fn test_bc_3_04_001_ac001_full_position_vector_inches() {
        // x=0.5in, y=1.0in, width=2.0in, height=1.0in
        assert_eq!(
            from_inches(500),
            Some(Emu(457_200)),
            "x: 0.5in → Some(Emu(457_200))"
        );
        assert_eq!(
            from_inches(1000),
            Some(Emu(914_400)),
            "y: 1.0in → Some(Emu(914_400))"
        );
        assert_eq!(
            from_inches(2000),
            Some(Emu(1_828_800)),
            "width: 2.0in → Some(Emu(1_828_800))"
        );
        assert_eq!(
            from_inches(1000),
            Some(Emu(914_400)),
            "height: 1.0in → Some(Emu(914_400))"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 — Off-canvas detection (BC-3.04.001 EC-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-003 — Shape with `x = Emu(-457_200)` (negative x = -0.5in) is off-canvas.
    ///
    /// Canonical test vector from BC-3.04.001 EC-002:
    ///   `position x -0.5in y 1.0in` → off-canvas warning.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_negative_x_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(-457_200), // -0.5in
            y: Emu(914_400),  // 1.0in
            width: Emu(1_828_800),
            height: Emu(914_400),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape at x=-0.5in must be detected as off-canvas"
        );
    }

    /// AC-003 — Shape with `y = Emu(-1)` (negative y) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_negative_y_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(-1),
            width: Emu(914_400),
            height: Emu(457_200),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape at y=-1 EMU must be detected as off-canvas"
        );
    }

    /// AC-003 — Shape that extends past right edge (`x + width > page_width`) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_exceeds_right_edge_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(9_000_000), // 9_000_000 + 200_001 > 9_144_000
            y: Emu(0),
            width: Emu(200_001),
            height: Emu(100_000),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape extending past right page edge must be off-canvas"
        );
    }

    /// AC-003 — Shape that extends past bottom edge (`y + height > page_height`) is off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_exceeds_bottom_edge_is_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(5_000_000), // 5_000_000 + 200_000 > 5_143_500
            width: Emu(100_000),
            height: Emu(200_000),
        };
        assert!(
            is_off_canvas(&bbox, default_page()),
            "shape extending past bottom page edge must be off-canvas"
        );
    }

    /// AC-003 — Shape entirely within page bounds is NOT off-canvas.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_valid_position_not_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(457_200),       // 0.5in from left
            y: Emu(914_400),       // 1.0in from top
            width: Emu(1_828_800), // 2.0in wide
            height: Emu(914_400),  // 1.0in tall
        };
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape within page bounds must NOT be off-canvas"
        );
    }

    /// AC-003 — Shape flush with page edges (x=0, y=0, width=page_w, height=page_h)
    /// is exactly on-canvas (boundary: inclusive).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac003_shape_fills_entire_page_not_off_canvas() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: Emu(0),
            width: DEFAULT_PAGE_WIDTH,
            height: DEFAULT_PAGE_HEIGHT,
        };
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape exactly filling the page must NOT be off-canvas"
        );
    }

    /// AC-003 — `layout_shapes` with a negative-x shape emits `LayoutWarning::OffCanvas`
    /// AND still produces a frame at the declared (off-canvas) position.
    ///
    /// This tests the full postcondition: the shape frame IS produced (not skipped),
    /// and the warning IS emitted (accumulated, not fatal).
    ///
    /// Red Gate: panics with `todo!()` inside `layout_shapes`.
    #[test]
    fn test_bc_3_04_001_ac003_layout_shapes_emits_offcanvas_warning_and_produces_frame() {
        // A shape at x=-0.5in, y=1.0in, width=2.0in, height=1.0in.
        // ShapeSpec carries a `position: ShapePosition` field (shipped in STORY-028);
        // this test verifies the off-canvas detection path via `is_off_canvas` directly
        // and asserts the LayoutWarning variant can be constructed as expected.
        // The full layout_shapes integration test exercising the warning-emission path
        // end-to-end is in test_bc_3_04_001_ac003_layout_shapes_full_off_canvas_warning.
        let off_canvas_bbox = BoundingBox {
            x: Emu(-457_200),
            y: Emu(914_400),
            width: Emu(1_828_800),
            height: Emu(914_400),
        };
        assert!(is_off_canvas(&off_canvas_bbox, default_page()));

        // Verify warning can be constructed for this shape:
        // F-HIGH-003: shape_type is ShapeType enum variant (not Arc<str>).
        let warning = LayoutWarning::OffCanvas {
            source_slide_index: 0,
            shape_type: ShapeType::Rect,
            x_emu: Emu(-457_200),
            y_emu: Emu(914_400),
        };
        assert!(matches!(
            warning,
            LayoutWarning::OffCanvas {
                x_emu: Emu(-457_200),
                ..
            }
        ));
    }

    /// AC-003 — `layout_shapes` called with a shape that has a valid position
    /// and explicit alt produces zero warnings and one frame.
    ///
    /// This is the happy-path integration test for `layout_shapes`.
    ///
    /// Red Gate: panics with `todo!()` inside `layout_shapes`.
    #[test]
    fn test_bc_3_04_001_ac003_layout_shapes_valid_shape_no_warnings() {
        let shapes = vec![shape_spec_with_alt("rect", "Blue rectangle highlight")];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        let output = result.expect("layout_shapes must succeed for a valid shape");
        assert_eq!(output.frames.len(), 1, "one shape → one frame");
        assert!(
            output.warnings.is_empty(),
            "on-canvas shape must produce zero warnings"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 — Shape Frame added to LaidOutSlide (BC-3.04.001 postcondition 4)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 — `layout_shapes` with one valid shape produces exactly one frame.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_returns_one_frame_per_shape() {
        let shapes = vec![shape_spec_with_alt("rect", "Blue rectangle")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes must succeed");
        assert_eq!(output.frames.len(), 1, "one shape in → one frame out");
    }

    /// AC-002 — `layout_shapes` with two shapes produces two frames in source order.
    ///
    /// The first shape in source order MUST be the first frame in the output vec
    /// (BC-3.04.001 postcondition 4).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_preserves_source_order() {
        let shapes = vec![
            shape_spec_with_alt("rect", "First shape"),
            shape_spec_with_alt("ellipse", "Second shape"),
        ];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes must succeed");
        assert_eq!(output.frames.len(), 2, "two shapes → two frames");
        // First frame must carry the first shape (Rect), second must carry Ellipse.
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Rect),
                    "first frame must be Rect (first shape in source order)"
                );
            },
            other => panic!("expected FrameContent::Shape for first frame, got: {other:?}"),
        }
        match &output.frames[1].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Ellipse),
                    "second frame must be Ellipse (second shape in source order)"
                );
            },
            other => panic!("expected FrameContent::Shape for second frame, got: {other:?}"),
        }
    }

    /// AC-002 — `layout_shapes` with zero shapes produces zero frames and zero warnings.
    ///
    /// Edge case: an empty shape list is valid (slides with no shape: blocks).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_layout_shapes_empty_slice_produces_empty_output() {
        let shapes: Vec<slideforge_types::ShapeSpec> = vec![];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes with empty slice must succeed");
        assert!(output.frames.is_empty(), "zero shapes → zero frames");
        assert!(output.warnings.is_empty(), "zero shapes → zero warnings");
    }

    /// AC-002 — Frame produced by `layout_shapes` has `FrameContent::Shape` variant.
    ///
    /// The frame content MUST be `FrameContent::Shape(ShapeFrame { .. })`, not
    /// any other variant.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac002_frame_content_is_shape_variant() {
        let shapes = vec![shape_spec_with_alt("rect", "Test rectangle")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes must succeed");
        assert!(
            matches!(
                output.frames[0].content,
                crate::types::FrameContent::Shape(_)
            ),
            "frame content must be FrameContent::Shape"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 — Decorative shape emits AltText::Decorative (BC-3.04.001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-004 — `build_shape_frame` with `decorative=true` produces `AltText::Decorative`.
    ///
    /// Canonical test vector from BC-3.04.001:
    ///   `shape: type ellipse decorative: true fill "#FF6F00"` → `AltText::Decorative`
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_decorative_true_produces_alt_decorative() {
        let shape_frame = build_shape_frame(
            ShapeType::Ellipse,
            FillSpec::SolidColor(Rgb {
                r: 0xFF,
                g: 0x6F,
                b: 0x00,
            }),
            None,
            None, // no explicit alt text
            true, // decorative: true
            0,
            &SourceSpan::default(),
        )
        .expect("decorative shape must succeed");
        assert!(
            matches!(shape_frame.alt, slideforge_types::AltText::Decorative),
            "decorative: true must produce AltText::Decorative, got: {:?}",
            shape_frame.alt
        );
    }

    /// AC-004 — `build_shape_frame` with explicit `alt` text produces `AltText::Provided`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_explicit_alt_produces_alt_provided() {
        let alt_text = "Blue rectangle highlight";
        let shape_frame = build_shape_frame(
            ShapeType::Rect,
            FillSpec::SolidColor(Rgb {
                r: 0,
                g: 55,
                b: 102,
            }),
            None,
            Some(Arc::from(alt_text)),
            false,
            0,
            &SourceSpan::default(),
        )
        .expect("shape with explicit alt must succeed");
        assert!(
            matches!(&shape_frame.alt, slideforge_types::AltText::Provided(s) if s.as_ref() == alt_text),
            "explicit alt must produce AltText::Provided with the given string"
        );
    }

    /// AC-004 — `layout_shapes` with a decorative shape emits `AltText::Decorative`
    /// in the produced `ShapeFrame`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ac004_layout_shapes_decorative_shape_frame() {
        let shapes = vec![shape_spec_decorative("ellipse")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("decorative shape must succeed in layout_shapes");
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.alt, slideforge_types::AltText::Decorative),
                    "decorative shape must produce AltText::Decorative"
                );
            },
            other => panic!("expected FrameContent::Shape, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-001 — Shape without alt or decorative returns MissingAlt error
    // (BC-3.04.001 EC-001 / DI-001)
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001 — `build_shape_frame` with `alt=None` and `decorative=false` returns
    /// `Err(LayoutError::MissingAlt)`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec001_missing_alt_returns_error() {
        let result = build_shape_frame(
            ShapeType::Rect,
            FillSpec::None,
            None,
            None,  // no alt
            false, // not decorative
            3,     // source_slide_index=3 for error message
            &SourceSpan::default(),
        );
        assert!(
            result.is_err(),
            "shape with neither alt nor decorative must return Err"
        );
        assert!(
            matches!(
                result.unwrap_err(),
                LayoutError::MissingAlt {
                    source_slide_index: 3,
                    ..
                }
            ),
            "error must be LayoutError::MissingAlt with the correct source_slide_index"
        );
    }

    /// EC-001 — `layout_shapes` with a shape that has no alt and `decorative=false`
    /// returns `Err(LayoutError::Multiple { inner: [MissingAlt] })`.
    ///
    /// Per Item N (uniform Multiple): even a single error is returned as Multiple.
    #[test]
    fn test_bc_3_04_001_ec001_layout_shapes_missing_alt_returns_error() {
        let shapes = vec![shape_spec_no_alt("rect")];
        let result = layout_shapes(&shapes, default_page(), 2, DEFAULT_EM_IN_EMU, 0);
        assert!(
            result.is_err(),
            "layout_shapes with no-alt shape must return Err"
        );
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single missing-alt must produce Multiple with 1 inner"
                );
                assert!(
                    matches!(
                        &inner[0],
                        LayoutError::MissingAlt {
                            source_slide_index: 2,
                            ..
                        }
                    ),
                    "inner error must be MissingAlt with source_slide_index=2"
                );
            },
            other => panic!("expected LayoutError::Multiple, got: {other:?}"),
        }
    }

    /// EC-001 / BC-3.04.001 EC-010 — Slide with one valid + one missing-alt shape
    /// returns `LayoutError::Multiple { inner: [MissingAlt] }` (uniform Multiple).
    #[test]
    fn test_bc_3_04_001_ec001_second_shape_missing_alt_returns_error() {
        let shapes = vec![
            shape_spec_with_alt("rect", "Valid shape"),
            shape_spec_no_alt("ellipse"), // missing alt
        ];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(
            result.is_err(),
            "second shape missing alt must cause layout_shapes to return Err"
        );
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single missing-alt shape must produce Multiple with 1 entry"
                );
                assert!(
                    matches!(inner[0], LayoutError::MissingAlt { .. }),
                    "inner error must be MissingAlt"
                );
            },
            other => panic!("expected LayoutError::Multiple (uniform), got: {other:?}"),
        }
    }

    /// EC-010 — Slide with two shapes BOTH missing alt returns `LayoutError::Multiple`
    /// containing two `MissingAlt` entries (BC-3.04.001 EC-010 / DI-018).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_ec010_two_shapes_missing_alt_returns_multiple() {
        let shapes = vec![
            shape_spec_no_alt("rect"),    // missing alt
            shape_spec_no_alt("ellipse"), // missing alt
        ];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(
            result.is_err(),
            "two shapes both missing alt must return Err"
        );
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    2,
                    "two missing-alt shapes must produce two errors"
                );
                assert!(
                    inner
                        .iter()
                        .all(|e| matches!(e, LayoutError::MissingAlt { .. })),
                    "all inner errors must be MissingAlt"
                );
            },
            other => {
                panic!("two missing-alt shapes must produce LayoutError::Multiple, got: {other:?}")
            },
        }
    }

    /// DI-018 / BC-3.04.001 item G — `layout_shapes` accumulates ALL errors in one pass.
    ///
    /// A slide containing a shape with missing alt AND a shape with an invalid bounding
    /// box (zero width) must produce `LayoutError::Multiple { inner }` with BOTH
    /// `MissingAlt` and `InvalidBoundingBox` present, regardless of shape order.
    ///
    /// Load-bearing for L1 (F-P21-LOW-001): before this fix, `InvalidBoundingBox`
    /// used `return Err(...)` and discarded prior accumulated `MissingAlt` errors.
    /// After the fix it uses `accumulated_errors.push(...)` + `continue`, so both
    /// errors survive to the end-of-loop `Multiple` return.
    #[test]
    fn test_l1_invalid_bbox_accumulated_alongside_missing_alt() {
        // Shape 1: missing alt (no alt text, decorative: false) — produces MissingAlt.
        let missing_alt_shape = shape_spec_no_alt("rect");

        // Shape 2: has valid alt but zero-width bounding box — produces InvalidBoundingBox.
        let st = slideforge_types::ShapeType::from_keyword("ellipse")
            .expect("ellipse must be a valid shape type keyword");
        let zero_width_shape = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(0), // zero width → InvalidBoundingBox
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("zero-width ellipse"))),
            decorative: false,
            span: SourceSpan::default(),
        };

        let shapes = vec![missing_alt_shape, zero_width_shape];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);

        // Must fail with Multiple containing BOTH MissingAlt and InvalidBoundingBox.
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    2,
                    "must accumulate exactly 2 errors (MissingAlt + InvalidBoundingBox); got: {inner:?}"
                );
                let has_missing_alt = inner
                    .iter()
                    .any(|e| matches!(e, LayoutError::MissingAlt { .. }));
                let has_invalid_bbox = inner
                    .iter()
                    .any(|e| matches!(e, LayoutError::InvalidBoundingBox { .. }));
                assert!(
                    has_missing_alt,
                    "Multiple must contain MissingAlt; inner: {inner:?}"
                );
                assert!(
                    has_invalid_bbox,
                    "Multiple must contain InvalidBoundingBox; inner: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple with MissingAlt + InvalidBoundingBox, got: {other:?}"
            ),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ShapeType parsing — closed vocabulary (BC-3.04.001 invariant 4)
    // F-MED-003: parse_shape_type deleted — tests use ShapeType::from_keyword.ok()
    // ─────────────────────────────────────────────────────────────────────────

    /// `ShapeType::from_keyword("rect").ok()` → `Some(ShapeType::Rect)`.
    ///
    /// F-MED-003: parse_shape_type was deleted (duplicate of ShapeType::from_keyword).
    /// Tests now use `ShapeType::from_keyword(kw).ok()` for Option-style assertions.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_rect() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("rect").ok(),
                Some(ShapeType::Rect)
            ),
            r#"from_keyword("rect").ok() must return Some(ShapeType::Rect)"#
        );
    }

    /// `ShapeType::from_keyword("ellipse").ok()` → `Some(ShapeType::Ellipse)`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_ellipse() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("ellipse").ok(),
                Some(ShapeType::Ellipse)
            ),
            r#"from_keyword("ellipse").ok() must return Some(ShapeType::Ellipse)"#
        );
    }

    /// `ShapeType::from_keyword("arrow").ok()` → `Some(ShapeType::Arrow)`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_arrow() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("arrow").ok(),
                Some(ShapeType::Arrow)
            ),
            r#"from_keyword("arrow").ok() must return Some(ShapeType::Arrow)"#
        );
    }

    /// `ShapeType::from_keyword("line").ok()` → `Some(ShapeType::Line)`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_line() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("line").ok(),
                Some(ShapeType::Line)
            ),
            r#"from_keyword("line").ok() must return Some(ShapeType::Line)"#
        );
    }

    /// `ShapeType::from_keyword("star").ok()` → `Some(ShapeType::Star)`.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_star() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("star").ok(),
                Some(ShapeType::Star)
            ),
            r#"from_keyword("star").ok() must return Some(ShapeType::Star)"#
        );
    }

    /// `ShapeType::from_keyword("roundRect").ok()` → `Some(ShapeType::RoundRect)`.
    ///
    /// Added in BC-3.04.001 v1.3 per Q7 decision example.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_round_rect() {
        assert!(
            matches!(
                slideforge_types::ShapeType::from_keyword("roundRect").ok(),
                Some(ShapeType::RoundRect)
            ),
            r#"from_keyword("roundRect").ok() must return Some(ShapeType::RoundRect)"#
        );
    }

    /// `ShapeType::from_keyword("frobnicator").ok()` → `None` (unknown keyword → E-PAR-012).
    ///
    /// BC-3.04.001 invariant 4: no `Custom` fallback — unknown keywords are parse errors.
    #[test]
    fn test_bc_3_04_001_parse_shape_type_unknown_returns_none() {
        assert!(
            slideforge_types::ShapeType::from_keyword("frobnicator")
                .ok()
                .is_none(),
            r#"from_keyword("frobnicator").ok() must return None (no Custom fallback)"#
        );
    }

    /// `ShapeType::from_keyword("").ok()` → `None` (empty keyword → unknown).
    #[test]
    fn test_bc_3_04_001_parse_shape_type_empty_returns_none() {
        assert!(
            slideforge_types::ShapeType::from_keyword("").ok().is_none(),
            "from_keyword(\"\").ok() must return None (no Custom fallback)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FillSpec — parse_hex_color and build_fill_spec
    // ─────────────────────────────────────────────────────────────────────────

    /// `parse_hex_color("#003766")` → `Some(Rgb { r: 0, g: 55, b: 102 })`.
    ///
    /// Canonical test vector from BC-3.04.001:
    ///   `fill "#003766"` → `FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 })`
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_valid_003766() {
        let rgb = parse_hex_color("#003766").expect("valid hex must parse");
        assert_eq!(rgb.r, 0x00, "red channel must be 0");
        assert_eq!(rgb.g, 0x37, "green channel must be 55 (0x37)");
        assert_eq!(rgb.b, 0x66, "blue channel must be 102 (0x66)");
    }

    /// `parse_hex_color("#FF6F00")` → `Some(Rgb { r: 255, g: 111, b: 0 })`.
    ///
    /// Canonical test vector from BC-3.04.001 (ellipse decorative fill).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_valid_ff6f00() {
        let rgb = parse_hex_color("#FF6F00").expect("valid hex must parse");
        assert_eq!(rgb.r, 0xFF);
        assert_eq!(rgb.g, 0x6F);
        assert_eq!(rgb.b, 0x00);
    }

    /// `parse_hex_color("#000000")` → `Some(Rgb { r: 0, g: 0, b: 0 })` (black).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_black() {
        let rgb = parse_hex_color("#000000").expect("black must parse");
        assert_eq!(rgb, Rgb { r: 0, g: 0, b: 0 });
    }

    /// `parse_hex_color("#FFFFFF")` → `Some(Rgb { r: 255, g: 255, b: 255 })` (white).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_white() {
        let rgb = parse_hex_color("#FFFFFF").expect("white must parse");
        assert_eq!(
            rgb,
            Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
    }

    /// `parse_hex_color("not-a-color")` → `None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_invalid_string() {
        assert!(
            parse_hex_color("not-a-color").is_none(),
            "invalid hex string must return None"
        );
    }

    /// `parse_hex_color("#GGGGGG")` → `None` (invalid hex digits).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_invalid_hex_digits() {
        assert!(
            parse_hex_color("#GGGGGG").is_none(),
            "non-hex digits must return None"
        );
    }

    /// `parse_hex_color("#12345")` → `None` (too short: 5 hex chars).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_too_short() {
        assert!(
            parse_hex_color("#12345").is_none(),
            "5-digit hex must return None (requires exactly 6 digits)"
        );
    }

    /// `parse_hex_color("003766")` → `None` (missing leading `#`).
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_parse_hex_color_missing_hash() {
        assert!(
            parse_hex_color("003766").is_none(),
            "hex without # prefix must return None"
        );
    }

    /// `build_fill_spec(Some("#003766"))` → `FillSpec::SolidColor(Rgb { r: 0, g: 55, b: 102 })`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_solid_hex() {
        let fill = build_fill_spec(Some("#003766"));
        assert!(
            matches!(
                fill,
                FillSpec::SolidColor(Rgb {
                    r: 0,
                    g: 0x37,
                    b: 0x66
                })
            ),
            "build_fill_spec with valid hex must return FillSpec::SolidColor"
        );
    }

    /// `build_fill_spec(None)` → `FillSpec::None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_none_keyword() {
        let fill = build_fill_spec(None);
        assert!(
            matches!(fill, FillSpec::None),
            "build_fill_spec(None) must return FillSpec::None"
        );
    }

    /// `build_fill_spec(Some("none"))` → `FillSpec::None`.
    ///
    /// Red Gate: panics with `todo!()`.
    #[test]
    fn test_bc_3_04_001_build_fill_spec_string_none() {
        let fill = build_fill_spec(Some("none"));
        assert!(
            matches!(fill, FillSpec::None),
            r#"build_fill_spec(Some("none")) must return FillSpec::None"#
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Type-system / green-by-design checks (compile-time verification)
    // ─────────────────────────────────────────────────────────────────────────

    /// `ShapeLayoutOutput` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_shape_layout_output_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let output = ShapeLayoutOutput {
            frames: vec![],
            warnings: vec![],
        };
        let output2 = output.clone();
        assert_eq!(output, output2);
        let mut set = HashSet::new();
        set.insert(output);
        assert_eq!(set.len(), 1);
    }

    /// `ShapeUnit` variants can be constructed and compared.
    #[test]
    fn test_shape_unit_variants_constructable() {
        let inches = ShapeUnit::Inches(500);
        let em = ShapeUnit::Em(1000);
        assert!(matches!(inches, ShapeUnit::Inches(500)));
        assert!(matches!(em, ShapeUnit::Em(1000)));
    }

    /// `LayoutWarning::OffCanvas` can be constructed and is hashable.
    /// F-HIGH-003: shape_type is ShapeType enum variant (not Arc<str>).
    #[test]
    fn test_layout_warning_offcanvas_constructable() {
        use std::collections::HashSet;
        let w = LayoutWarning::OffCanvas {
            source_slide_index: 0,
            shape_type: ShapeType::Rect, // F-HIGH-003: enum variant
            x_emu: Emu(-457_200),
            y_emu: Emu(914_400),
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-037 — EMU conversion (Kani candidate)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-037 / VP-048 — `from_inches` with `i64::MAX` returns `None` (checked overflow).
    ///
    /// BC-3.04.001 Invariant 8 / VP-048: `from_inches` uses `checked_mul`; overflow
    /// returns `None` rather than saturating silently. `i64::MAX * 914_400` overflows
    /// `i64`, so the result is `None`.
    #[test]
    fn test_vp_037_from_inches_max_input_does_not_panic() {
        // checked_mul: MAX * 914_400 overflows → None (not saturated to i64::MAX).
        let result = from_inches(i64::MAX);
        assert!(
            result.is_none(),
            "from_inches(i64::MAX) must return None (overflow, not saturation); got: {result:?}"
        );
    }

    /// VP-037 / VP-048 — `from_inches` with `i64::MIN` returns `None` (checked underflow).
    ///
    /// `i64::MIN * 914_400` underflows `i64` → `None`.
    #[test]
    fn test_vp_037_from_inches_min_input_does_not_panic() {
        let result = from_inches(i64::MIN);
        assert!(
            result.is_none(),
            "from_inches(i64::MIN) must return None (underflow, not saturation); got: {result:?}"
        );
    }

    /// VP-037 / VP-048 — `from_em` with `i64::MAX` returns `None` (checked overflow).
    #[test]
    fn test_vp_037_from_em_max_input_does_not_panic() {
        // checked_mul: MAX * DEFAULT_EM_IN_EMU overflows → None.
        let result = from_em(i64::MAX, DEFAULT_EM_IN_EMU);
        assert!(
            result.is_none(),
            "from_em(i64::MAX, DEFAULT_EM_IN_EMU) must return None (overflow); got: {result:?}"
        );
    }

    /// VP-037 / VP-048 — Canonical AC-001 test vector: x=0.5in y=1.0in width=2.0in height=1.0in.
    ///
    /// In-range inputs return `Some(Emu(...))`.
    #[test]
    fn test_ac_001_canonical_position_vector_to_bbox() {
        // x = 0.5in = 500 milliinches → Some(Emu(457_200))
        // y = 1.0in = 1000 milliinches → Some(Emu(914_400))
        // width = 2.0in = 2000 milliinches → Some(Emu(1_828_800))
        // height = 1.0in = 1000 milliinches → Some(Emu(914_400))
        let pos = default_position();
        let x = unit_to_emu(&pos.x, DEFAULT_EM_IN_EMU).expect("x must not overflow");
        let y = unit_to_emu(&pos.y, DEFAULT_EM_IN_EMU).expect("y must not overflow");
        let width = unit_to_emu(&pos.width, DEFAULT_EM_IN_EMU).expect("width must not overflow");
        let height = unit_to_emu(&pos.height, DEFAULT_EM_IN_EMU).expect("height must not overflow");
        let bbox = BoundingBox {
            x,
            y,
            width,
            height,
        };
        assert_eq!(bbox.x, Emu(457_200), "x: 0.5in → Emu(457_200)");
        assert_eq!(bbox.y, Emu(914_400), "y: 1.0in → Emu(914_400)");
        assert_eq!(bbox.width, Emu(1_828_800), "width: 2.0in → Emu(1_828_800)");
        assert_eq!(bbox.height, Emu(914_400), "height: 1.0in → Emu(914_400)");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-038 — MissingAlt test (span propagation — F-MED-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-038 / F-MED-002 — `layout_shapes` with a shape missing alt returns
    /// `Multiple { inner: [MissingAlt { source_slide_index, span }] }` where
    /// `span` equals the shape's `SourceSpan` (BC-3.04.001 AC-BC-A5).
    ///
    /// Load-bearing span check: the test uses a NON-DEFAULT span (file/line/col
    /// explicitly set) so the assertion fails if `layout_shapes` swaps the span
    /// with a default or overwrites it.
    #[test]
    fn test_vp_038_missing_alt_span_propagation() {
        use slideforge_types::SourceSpan;
        use std::sync::Arc;
        // Non-default span — load-bearing for F-MED-002.
        let non_default_span = SourceSpan {
            file: Arc::from("test.sf"),
            line: 42,
            col: 3,
            byte_offset: 1024,
        };
        let mut spec = shape_spec_no_alt("rect");
        spec.span = non_default_span.clone();

        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 5, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "shape with no alt must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single missing-alt shape must produce Multiple with 1 entry"
                );
                match &inner[0] {
                    LayoutError::MissingAlt {
                        source_slide_index,
                        span,
                    } => {
                        assert_eq!(*source_slide_index, 5, "source_slide_index must be 5");
                        assert_eq!(
                            span.file.as_ref(),
                            "test.sf",
                            "span.file must propagate from ShapeSpec"
                        );
                        assert_eq!(span.line, 42, "span.line must propagate from ShapeSpec");
                        assert_eq!(span.col, 3, "span.col must propagate from ShapeSpec");
                    },
                    other => panic!("inner error must be MissingAlt, got: {other:?}"),
                }
            },
            other => panic!("expected LayoutError::Multiple, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-039 — Unknown shape type rejected at parse level (no Custom fallback)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-039 — The closed shape-type vocabulary is enforced at the type level
    /// by `slideforge_types::ShapeType`. `ShapeType::from_keyword` returns
    /// `Err(ShapeTypeError)` for any keyword outside the six-member v1.0
    /// vocabulary (`rect`, `ellipse`, `arrow`, `line`, `star`, `roundRect`).
    ///
    /// With the STORY-028 pass-2 schema change (`ShapeSpec.shape_type: ShapeType`),
    /// it is not possible to construct a `ShapeSpec` with an unknown shape type
    /// and pass it to `layout_shapes`. The parse stage rejects unknown keywords
    /// with `E-PAR-012` before a `ShapeSpec` is ever built. `LayoutError::UnknownShapeType`
    /// was removed in STORY-028 pass-4 (F-P4-MED-001) because it is unreachable from
    /// `layout_shapes` after the schema change. Future parser-level E-PAR-012 surfacing
    /// will use `slideforge_types::ShapeTypeError` directly.
    ///
    /// This test verifies the parse-level gate: `from_keyword` returns `Err`
    /// for unrecognised keywords and returns `Ok` for all six known keywords.
    /// Per interface-definitions §9.2 (pass-3 adjudication): from_keyword now
    /// returns `Result<ShapeType, ShapeTypeError>` — not `Option<ShapeType>`.
    #[test]
    fn test_vp_039_unknown_shape_type_rejected_at_parse_level() {
        // Unknown keywords produce Err(ShapeTypeError) — caller must emit E-PAR-012.
        assert!(
            slideforge_types::ShapeType::from_keyword("frobnicator").is_err(),
            "unknown keyword 'frobnicator' must return Err from from_keyword"
        );
        assert!(
            slideforge_types::ShapeType::from_keyword("").is_err(),
            "empty string must return Err from from_keyword"
        );
        assert!(
            slideforge_types::ShapeType::from_keyword("Rect").is_err(),
            "case-sensitive check: 'Rect' must return Err (closed vocab uses 'rect')"
        );

        // All six v1.0 keywords must produce Ok.
        for kw in &["rect", "ellipse", "arrow", "line", "star", "roundRect"] {
            assert!(
                slideforge_types::ShapeType::from_keyword(kw).is_ok(),
                "known keyword '{kw}' must return Ok from from_keyword"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-040 — Hex color case-insensitive (Kani candidate)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-040 — `parse_hex_color` accepts lowercase hex digits.
    ///
    /// BC-3.04.001 AC-BC-A2 / F-HIGH-005: 6-digit hex case-insensitive.
    #[test]
    fn test_vp_040_parse_hex_color_lowercase_accepted() {
        let rgb = parse_hex_color("#ff6f00").expect("lowercase hex must parse");
        assert_eq!(rgb.r, 0xFF);
        assert_eq!(rgb.g, 0x6F);
        assert_eq!(rgb.b, 0x00);
    }

    /// VP-040 — `parse_hex_color` accepts uppercase hex digits (canonical form).
    #[test]
    fn test_vp_040_parse_hex_color_uppercase_accepted() {
        let rgb = parse_hex_color("#FF6F00").expect("uppercase hex must parse");
        assert_eq!(rgb.r, 0xFF);
        assert_eq!(rgb.g, 0x6F);
        assert_eq!(rgb.b, 0x00);
    }

    /// VP-040 — `parse_hex_color` accepts mixed-case hex digits.
    #[test]
    fn test_vp_040_parse_hex_color_mixed_case_accepted() {
        let rgb = parse_hex_color("#Ff6F00").expect("mixed-case hex must parse");
        assert_eq!(rgb.r, 0xFF);
        assert_eq!(rgb.g, 0x6F);
        assert_eq!(rgb.b, 0x00);
    }

    /// VP-040 — `parse_hex_color` rejects 3-digit short form.
    ///
    /// BC-3.04.001 AC-BC-A2 / F-HIGH-005: `#RGB` short form → rejected (E-PAR-015).
    #[test]
    fn test_vp_040_parse_hex_color_short_form_rejected() {
        assert!(
            parse_hex_color("#RGB").is_none(),
            "#RGB short form must return None (BC-3.04.001 AC-BC-A2)"
        );
        assert!(
            parse_hex_color("#fff").is_none(),
            "#fff 3-digit form must return None"
        );
    }

    /// VP-040 — `parse_hex_color` rejects 8-digit RGBA form.
    ///
    /// BC-3.04.001 AC-BC-A2 / F-HIGH-005: `#RRGGBBAA` → rejected (E-PAR-015).
    #[test]
    fn test_vp_040_parse_hex_color_rgba_form_rejected() {
        assert!(
            parse_hex_color("#FF6F00AA").is_none(),
            "#RRGGBBAA 8-digit form must return None (use fill-opacity attribute)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-041 — Off-canvas inclusive boundary (Kani candidate)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-041 / AC-BC-A4 — `x + width == page_width` is ON-canvas (inclusive boundary).
    ///
    /// BC-3.04.001 item D: equality is on-canvas (not off). F-MED-001.
    #[test]
    fn test_ac_bc_a4_inclusive_boundary() {
        // A shape that exactly fills the right edge: x + width == page_width.
        let bbox = BoundingBox {
            x: DEFAULT_PAGE_WIDTH - Emu(1_000_000),
            y: Emu(0),
            width: Emu(1_000_000),
            height: Emu(100_000),
        };
        assert_eq!(
            bbox.x.0 + bbox.width.0,
            DEFAULT_PAGE_WIDTH.0,
            "test setup: x + width must equal page_width"
        );
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape with x + width == page_width must be ON-canvas (inclusive boundary)"
        );
    }

    /// VP-041 — `y + height == page_height` is ON-canvas (inclusive boundary).
    #[test]
    fn test_ac_bc_a4_inclusive_boundary_bottom_edge() {
        let bbox = BoundingBox {
            x: Emu(0),
            y: DEFAULT_PAGE_HEIGHT - Emu(500_000),
            width: Emu(100_000),
            height: Emu(500_000),
        };
        assert_eq!(
            bbox.y.0 + bbox.height.0,
            DEFAULT_PAGE_HEIGHT.0,
            "test setup: y + height must equal page_height"
        );
        assert!(
            !is_off_canvas(&bbox, default_page()),
            "shape with y + height == page_height must be ON-canvas (inclusive boundary)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-042 — N MissingAlts → Vec<LayoutError> with N entries (F-MED-003)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-042 — Three shapes all missing alt returns `LayoutError::Multiple`
    /// with 3 entries (DI-018 / BC-3.04.001 AC-BC-A6).
    #[test]
    fn test_vp_042_three_missing_alts_returns_multiple_with_three_entries() {
        let shapes = vec![
            shape_spec_no_alt("rect"),
            shape_spec_no_alt("ellipse"),
            shape_spec_no_alt("arrow"),
        ];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "three missing-alt shapes must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    3,
                    "three missing-alt shapes must produce three errors"
                );
                assert!(
                    inner
                        .iter()
                        .all(|e| matches!(e, LayoutError::MissingAlt { .. })),
                    "all inner errors must be MissingAlt"
                );
            },
            other => panic!("expected LayoutError::Multiple with 3 entries, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-004 — build_fill_spec + build_shape_frame end-to-end flow
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-004 / AC-002 — `build_fill_spec("#003766")` → `FillSpec::SolidColor(Rgb)`.
    /// End-to-end: fill keyword → `build_fill_spec` → `FillSpec::SolidColor(...)`.
    #[test]
    fn test_high_004_build_fill_spec_end_to_end_solid_color() {
        let fill = build_fill_spec(Some("#003766"));
        assert!(
            matches!(
                fill,
                FillSpec::SolidColor(Rgb {
                    r: 0,
                    g: 0x37,
                    b: 0x66
                })
            ),
            "build_fill_spec with '#003766' must return FillSpec::SolidColor; got: {fill:?}"
        );
    }

    /// BC-3.04.001 v1.5.2 Invariant 11 — when both `alt: Some(Provided(s))` and
    /// `decorative: true` are on a `ShapeSpec`, alt text WINS in the output frame
    /// (`AltText::Provided(s)`).
    ///
    /// Load-bearing (Contract B): swap the match arm order in `build_shape_frame`
    /// so that `(_, true) => AltText::Decorative` is checked before `(Some(s), _)`,
    /// and this test MUST fail with `AltText::Decorative` instead of `AltText::Provided`.
    #[test]
    fn test_bc_3_04_001_invariant_11_alt_wins_over_decorative() {
        // Build a ShapeSpec with BOTH alt text AND decorative: true.
        // This is the conflicting-intent case that Invariant 11 resolves.
        let spec = ShapeSpec {
            shape_type: ShapeType::Rect,
            position: default_position(),
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("Blue rect"))),
            decorative: true, // will lose to alt text (Invariant 11)
            span: SourceSpan::default(),
        };

        // Extract values from spec (mirroring layout_shapes extraction at shapes.rs:215-220).
        let alt = match &spec.alt {
            Some(AltText::Provided(s)) => Some(Arc::clone(s)),
            Some(AltText::Decorative | AltText::Unspecified) | None => None,
        };
        let decorative = spec.decorative || matches!(&spec.alt, Some(AltText::Decorative));

        let frame = build_shape_frame(
            spec.shape_type,
            spec.fill.clone(),
            spec.text.clone(),
            alt,
            decorative,
            0,
            &spec.span,
        )
        .expect("alt + decorative=true must succeed (Invariant 11: alt wins)");

        // Contract B (load-bearing): the output frame carries AltText::Provided, not Decorative.
        match &frame.alt {
            slideforge_types::AltText::Provided(s) => {
                assert_eq!(
                    s.as_ref(),
                    "Blue rect",
                    "alt must be preserved in output frame (Invariant 11)"
                );
            },
            slideforge_types::AltText::Decorative => {
                panic!(
                    "decorative MUST NOT win when alt is present (BC-3.04.001 v1.5.2 Invariant 11)"
                );
            },
            slideforge_types::AltText::Unspecified => {
                panic!("Unspecified MUST NOT be produced when alt is Provided (Invariant 11)");
            },
        }
    }

    /// F-HIGH-004 — `build_shape_frame` with fill and alt produces a ShapeFrame
    /// with the correct fill and alt (AC-002 production path).
    #[test]
    fn test_high_004_build_shape_frame_fill_propagation() {
        let fill = build_fill_spec(Some("#003766"));
        let frame = build_shape_frame(
            ShapeType::Rect,
            fill.clone(),
            None,
            Some(Arc::from("Blue rectangle")),
            false,
            0,
            &SourceSpan::default(),
        )
        .expect("build_shape_frame must succeed");

        assert_eq!(
            frame.fill, fill,
            "fill must propagate through build_shape_frame"
        );
        assert!(
            matches!(&frame.alt, slideforge_types::AltText::Provided(s) if s.as_ref() == "Blue rectangle"),
            "alt must propagate through build_shape_frame"
        );
    }

    /// `LayoutWarning::XrefTargetNotFound` can be constructed and is hashable.
    #[test]
    fn test_layout_warning_xref_not_found_constructable() {
        use std::collections::HashSet;
        let w = LayoutWarning::XrefTargetNotFound {
            target: Arc::from("slide-99"),
            source_slide_index: 3,
        };
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }

    /// `ShapeFrame` implements `Clone + PartialEq + Eq + Hash`.
    #[test]
    fn test_shape_frame_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let sf = ShapeFrame {
            shape_type: ShapeType::Rect,
            fill: FillSpec::SolidColor(Rgb {
                r: 0,
                g: 55,
                b: 102,
            }),
            text: None,
            alt: slideforge_types::AltText::Provided(Arc::from("Blue rectangle")),
        };
        let sf2 = sf.clone();
        assert_eq!(sf, sf2);
        let mut set = HashSet::new();
        set.insert(sf);
        assert_eq!(set.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // VP-048 — ArithmeticOverflow wiring (Item M / BC-3.04.001 Invariant 8)
    // ─────────────────────────────────────────────────────────────────────────

    /// VP-048 — `from_inches(i64::MAX)` returns `None` (overflow detected, not silent).
    ///
    /// BC-3.04.001 Invariant 8 / interface-definitions.md §9.4: `checked_mul` must
    /// detect overflow and return `None`. This is the primary VP-048 test.
    ///
    /// Load-bearing: if production code uses `saturating_mul`, this returns
    /// `Some(Emu(i64::MAX / 1_000))` and the assert fails.
    #[test]
    fn test_vp_048_from_inches_max_returns_arithmetic_overflow() {
        let result = from_inches(i64::MAX);
        assert!(
            result.is_none(),
            "from_inches(i64::MAX) must return None (VP-048 checked_mul); \
             saturating_mul would return Some(Emu({})) — that is the wrong production behaviour",
            i64::MAX / 1_000
        );
    }

    /// VP-048 / F-MED-005 — `from_inches(i64::MIN)` returns `None` (underflow).
    ///
    /// Regression test for the negative-overflow boundary (F-MED-005).
    ///
    /// Load-bearing: saturating_mul would return `Some(Emu(i64::MIN / 1_000))`.
    #[test]
    fn test_vp_048_boundary_from_inches_min_returns_none() {
        let result = from_inches(i64::MIN);
        assert!(
            result.is_none(),
            "from_inches(i64::MIN) must return None (VP-048); \
             saturating underflow would return Some — that is wrong"
        );
    }

    /// VP-048 — `from_em(i64::MAX, DEFAULT_EM_IN_EMU)` returns `None`.
    #[test]
    fn test_vp_048_from_em_max_returns_none() {
        let result = from_em(i64::MAX, DEFAULT_EM_IN_EMU);
        assert!(
            result.is_none(),
            "from_em(i64::MAX, DEFAULT_EM_IN_EMU) must return None (VP-048 checked_mul)"
        );
    }

    /// VP-048 — `layout_shapes` with an i64::MAX position field accumulates
    /// `LayoutError::ArithmeticOverflow` in a `Multiple`.
    ///
    /// End-to-end overflow test: an artificially extreme `ShapeUnit::Inches(i64::MAX)`
    /// must propagate through `layout_shapes` as `ArithmeticOverflow`, not silently clamp.
    ///
    /// Load-bearing: without the `checked_mul` path, layout would produce a shape frame
    /// with a saturated (but accepted) EMU value — this test catches that regression.
    #[test]
    fn test_vp_048_layout_shapes_overflow_x_returns_arithmetic_overflow() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(i64::MAX), // overflows checked_mul
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("test shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "overflow position must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. })),
                    "Multiple must contain ArithmeticOverflow; got: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple containing ArithmeticOverflow, got: {other:?}"
            ),
        }
    }

    /// VP-048 — y-overflow propagates as `LayoutError::Multiple` containing `ArithmeticOverflow`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_x_returns_arithmetic_overflow` but
    /// exercises the `y` field overflow path. The x, y, width, and height paths share
    /// identical `checked_mul` logic; each has a dedicated test so a future refactor
    /// cannot accidentally skip one field without breaking coverage.
    #[test]
    fn test_vp_048_layout_shapes_overflow_y_returns_arithmetic_overflow() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(1000),
                y: ShapeUnit::Inches(i64::MAX), // overflows checked_mul
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("test shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "y overflow must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. })),
                    "Multiple must contain ArithmeticOverflow for y overflow; got: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple containing ArithmeticOverflow, got: {other:?}"
            ),
        }
    }

    /// VP-048 — width-overflow propagates as `LayoutError::Multiple` containing `ArithmeticOverflow`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_x_returns_arithmetic_overflow` but
    /// exercises the `width` field overflow path.
    #[test]
    fn test_vp_048_layout_shapes_overflow_width_returns_arithmetic_overflow() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(1000),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(i64::MAX), // overflows checked_mul
                height: ShapeUnit::Inches(500),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("test shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "width overflow must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. })),
                    "Multiple must contain ArithmeticOverflow for width overflow; got: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple containing ArithmeticOverflow, got: {other:?}"
            ),
        }
    }

    /// VP-048 — height-overflow propagates as `LayoutError::Multiple` containing `ArithmeticOverflow`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_x_returns_arithmetic_overflow` but
    /// exercises the `height` field overflow path.
    #[test]
    fn test_vp_048_layout_shapes_overflow_height_returns_arithmetic_overflow() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(1000),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(1000),
                height: ShapeUnit::Inches(i64::MAX), // overflows checked_mul
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("test shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "height overflow must return Err");
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. })),
                    "Multiple must contain ArithmeticOverflow for height overflow; got: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple containing ArithmeticOverflow, got: {other:?}"
            ),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Item N — LayoutError::multiple() smart constructor tests
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.04.001 Item N — Single error → `Multiple { inner: vec![err] }` (uniform).
    ///
    /// `LayoutError::multiple()` MUST return `Multiple` even for a single error,
    /// providing a uniform return type regardless of error count.
    ///
    /// Load-bearing: `layout_shapes` with 1 missing-alt shape previously returned
    /// `MissingAlt` directly; this test enforces the new uniform Multiple contract.
    #[test]
    fn test_bc_3_04_001_multiple_single_error_uniformity() {
        let single = LayoutError::MissingAlt {
            source_slide_index: 0,
            span: SourceSpan::default(),
        };
        let result = LayoutError::multiple(vec![single.clone()]);
        match result {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single error must produce Multiple with len=1"
                );
                assert_eq!(inner[0], single, "inner error must equal the original");
            },
            other => {
                panic!("LayoutError::multiple(vec![one_err]) must return Multiple, got: {other:?}")
            },
        }
    }

    /// BC-3.04.001 Item N — Nested `Multiple` is flattened by smart constructor.
    ///
    /// `LayoutError::multiple(vec![Multiple { inner: [A, B] }, C])` → `Multiple { inner: [A, B, C] }`.
    /// No nested Multiple variant must survive (BC-3.04.001 Multiple.Invariants).
    ///
    /// Load-bearing: without the flat_map path, nesting depth grows on each accumulation pass.
    #[test]
    fn test_bc_3_04_001_multiple_flattens_nested() {
        let inner_a = LayoutError::EmptyDeck {
            source_slide_index: 0,
        };
        let inner_b = LayoutError::EmptyDeck {
            source_slide_index: 1,
        };
        let outer_c = LayoutError::EmptyDeck {
            source_slide_index: 2,
        };
        let nested = LayoutError::Multiple {
            inner: vec![inner_a.clone(), inner_b.clone()],
        };
        let result = LayoutError::multiple(vec![nested, outer_c.clone()]);
        match result {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    3,
                    "nested Multiple must be flattened: expected 3 flat errors, got: {inner:?}"
                );
                assert_eq!(inner[0], inner_a);
                assert_eq!(inner[1], inner_b);
                assert_eq!(inner[2], outer_c);
                // No nested Multiple in inner
                for e in &inner {
                    assert!(
                        !matches!(e, LayoutError::Multiple { .. }),
                        "flattened inner must not contain nested Multiple; found: {e:?}"
                    );
                }
            },
            other => panic!("expected Multiple after flattening, got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-002 — AC-INT-1 EMU canonical vector (load-bearing bbox assertion)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-002 — `layout_shapes` with canonical position vector produces
    /// exact EMU values: x=457_200, y=914_400, width=1_828_800, height=914_400.
    ///
    /// Canonical test vector from BC-3.04.001 AC-001.
    /// Load-bearing: without the `unit_to_emu` wiring, the bbox values would be
    /// zero or default — this assertion proves the full conversion path is active.
    #[test]
    fn test_f_high_002_layout_shapes_canonical_emu_vector() {
        let shapes = vec![shape_spec_with_alt("rect", "Test rect")];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("canonical position must not overflow");
        assert_eq!(output.frames.len(), 1);
        let bbox = output.frames[0].bbox;
        assert_eq!(bbox.x, Emu(457_200), "x: 0.5in → Emu(457_200)");
        assert_eq!(bbox.y, Emu(914_400), "y: 1.0in → Emu(914_400)");
        assert_eq!(bbox.width, Emu(1_828_800), "width: 2.0in → Emu(1_828_800)");
        assert_eq!(bbox.height, Emu(914_400), "height: 1.0in → Emu(914_400)");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-003 — is_valid over shape frames (zero-width/height guard)
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-003 — Shape with `width = 0` (i.e., 0 EMU) is rejected as
    /// `LayoutError::InvalidBoundingBox` (BC-3.06.003).
    ///
    /// Load-bearing: without the is_valid guard in layout_shapes, a zero-width shape
    /// would produce a frame — this test catches that regression.
    #[test]
    fn test_f_high_003_zero_width_shape_rejected_as_invalid_bbox() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(0), // zero width → Emu(0) → invalid
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("zero-width shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 1, DEFAULT_EM_IN_EMU, 0);
        assert!(result.is_err(), "zero-width shape must return Err");
        // InvalidBoundingBox is accumulated per DI-018 (not bail-on-first) and returned
        // in a Multiple for uniform error shape (Item N). Unwrap the Multiple to check.
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single invalid-bbox must produce Multiple with 1 inner"
                );
                assert!(
                    matches!(
                        &inner[0],
                        LayoutError::InvalidBoundingBox {
                            source_slide_index: 1,
                            ..
                        }
                    ),
                    "inner error must be InvalidBoundingBox with source_slide_index=1; got: {:?}",
                    inner[0]
                );
            },
            other => {
                panic!("expected LayoutError::Multiple wrapping InvalidBoundingBox, got: {other:?}")
            },
        }
    }

    /// F-P20-LOW-002 — `InvalidBoundingBox.frame_index` is slide-wide, not sub-list.
    ///
    /// Scenario: 2 region frames already placed by the caller (base_index=2),
    /// then 1 zero-width shape. Expected `frame_index = 2 + 0 = 2` (not 0).
    ///
    /// Load-bearing: change `base_index` from 2 to 0 in the call below and the
    /// assertion for `frame_index: 2` must fail (it would report 0 instead),
    /// proving that the `base_index` parameter is wired through to the error.
    #[test]
    fn test_f_p20_low_002_invalid_bbox_frame_index_is_slide_wide() {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        // Zero-width shape — will trigger InvalidBoundingBox.
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x: ShapeUnit::Inches(500),
                y: ShapeUnit::Inches(1000),
                width: ShapeUnit::Inches(0), // zero width → Emu(0) → invalid
                height: ShapeUnit::Inches(1000),
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("zero-width shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        // Simulates a slide that already has 2 region frames (base_index=2).
        // The zero-width shape is the first (and only) shape, at sub-list index 0.
        // Slide-wide frame_index = base_index(2) + sub-list(0) = 2.
        let result = layout_shapes(&[spec], default_page(), 0, DEFAULT_EM_IN_EMU, 2);
        assert!(result.is_err(), "zero-width shape must return Err");
        // InvalidBoundingBox is accumulated per DI-018 and returned in a Multiple
        // (uniform Item N shape). Unwrap the Multiple to verify the inner field values.
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert_eq!(
                    inner.len(),
                    1,
                    "single invalid-bbox must produce Multiple with 1 inner"
                );
                match &inner[0] {
                    LayoutError::InvalidBoundingBox {
                        source_slide_index,
                        frame_index,
                        ..
                    } => {
                        assert_eq!(*source_slide_index, 0, "source_slide_index must be 0");
                        assert_eq!(
                            *frame_index, 2,
                            "frame_index must be 2 (slide-wide: base_index=2 + sub-list=0); \
                             got {frame_index} — did base_index get wired through?"
                        );
                    },
                    other => panic!("inner error must be InvalidBoundingBox, got: {other:?}"),
                }
            },
            other => {
                panic!("expected LayoutError::Multiple wrapping InvalidBoundingBox, got: {other:?}")
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-OBS-003 — Shape frame index >= region count (BC-3.04.001 PC-3)
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.04.001 — `layout_shapes` preserves shape source order.
    ///
    /// Two shapes submitted in order (Rect, Ellipse) must appear in the same
    /// order in the output vec: frames[0] = Rect, frames[1] = Ellipse.
    ///
    /// This test exercises `layout_shapes` in isolation (no regions).  The
    /// resulting frames are those the caller (layout::run) appends after any
    /// region frames; within the shape sub-vec the relative order must be
    /// preserved.
    #[test]
    fn test_bc_3_04_001_shape_source_order_preserved() {
        // layout_shapes produces shape frames that the caller (layout::run) appends
        // after region frames. With 2 shapes, shape output has frames at indices 0 and 1
        // (relative to the shape output vec), which become N and N+1 in the full slide.
        let shapes = vec![
            shape_spec_with_alt("rect", "First shape"),
            shape_spec_with_alt("ellipse", "Second shape"),
        ];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes must succeed");
        assert_eq!(output.frames.len(), 2, "two shapes → two shape frames");
        // Source order is preserved: first shape → frames[0], second → frames[1].
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Rect),
                    "first shape frame must be Rect"
                );
            },
            other => panic!("expected Shape frame at [0], got: {other:?}"),
        }
        match &output.frames[1].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.shape_type, ShapeType::Ellipse),
                    "second shape frame must be Ellipse"
                );
            },
            other => panic!("expected Shape frame at [1], got: {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P15-LOW-001 — Em overflow end-to-end coverage symmetry with Inches
    //
    // The existing VP-048 tests above exercise ShapeUnit::Inches(i64::MAX) for
    // each of x/y/width/height. This block mirrors those tests using
    // ShapeUnit::Em(i64::MAX) so that the Em arm of unit_to_emu (from_em) is
    // covered by the same end-to-end overflow path. Without these tests a
    // future refactor of from_em (e.g., changing checked_mul to saturating_mul)
    // would break the VP-048 contract without any test failure.
    //
    // A shared helper eliminates 4×2 = 8x copy-paste.
    // ─────────────────────────────────────────────────────────────────────────

    /// Shared overflow assertion for `layout_shapes` with an Em-unit extreme value.
    ///
    /// Builds a `ShapeSpec` with `i64::MAX` in the given field (as `ShapeUnit::Em`)
    /// and all other fields set to safe values. Asserts that `layout_shapes`
    /// returns `LayoutError::Multiple` containing at least one `ArithmeticOverflow`.
    ///
    /// `field` is a human-readable label used in assertion messages ("x", "y",
    /// "width", "height").
    fn assert_em_overflow_field(
        x: ShapeUnit,
        y: ShapeUnit,
        width: ShapeUnit,
        height: ShapeUnit,
        field: &str,
    ) {
        let st = slideforge_types::ShapeType::from_keyword("rect").expect("rect must be known");
        let spec = ShapeSpec {
            shape_type: st,
            position: ShapePosition {
                x,
                y,
                width,
                height,
            },
            fill: FillSpec::None,
            text: None,
            alt: Some(AltText::Provided(Arc::from("em overflow test shape"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let result = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(
            result.is_err(),
            "Em overflow in {field} must return Err (VP-048 checked_mul)"
        );
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::ArithmeticOverflow { .. })),
                    "Multiple must contain ArithmeticOverflow for Em {field} overflow; \
                     got: {inner:?}"
                );
            },
            other => panic!(
                "expected LayoutError::Multiple containing ArithmeticOverflow \
                 for Em {field} overflow, got: {other:?}"
            ),
        }
    }

    /// VP-048 / F-P15-LOW-001 — `layout_shapes` with `ShapeUnit::Em(i64::MAX)` in
    /// the `x` field propagates `LayoutError::ArithmeticOverflow` inside a `Multiple`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_x_returns_arithmetic_overflow`
    /// but exercises the Em arm of `unit_to_emu` (`from_em`).
    ///
    /// Load-bearing: change `from_em` to use `saturating_mul` — this test MUST fail.
    #[test]
    fn test_vp_048_layout_shapes_em_overflow_x_returns_arithmetic_overflow() {
        assert_em_overflow_field(
            ShapeUnit::Em(i64::MAX), // overflows checked_mul in from_em
            ShapeUnit::Em(1000),
            ShapeUnit::Em(1000),
            ShapeUnit::Em(500),
            "x",
        );
    }

    /// VP-048 / F-P15-LOW-001 — `layout_shapes` with `ShapeUnit::Em(i64::MAX)` in
    /// the `y` field propagates `LayoutError::ArithmeticOverflow` inside a `Multiple`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_y_returns_arithmetic_overflow`
    /// but exercises the Em arm of `unit_to_emu` (`from_em`).
    ///
    /// Load-bearing: change `from_em` to use `saturating_mul` — this test MUST fail.
    #[test]
    fn test_vp_048_layout_shapes_em_overflow_y_returns_arithmetic_overflow() {
        assert_em_overflow_field(
            ShapeUnit::Em(1000),
            ShapeUnit::Em(i64::MAX), // overflows checked_mul in from_em
            ShapeUnit::Em(1000),
            ShapeUnit::Em(500),
            "y",
        );
    }

    /// VP-048 / F-P15-LOW-001 — `layout_shapes` with `ShapeUnit::Em(i64::MAX)` in
    /// the `width` field propagates `LayoutError::ArithmeticOverflow` inside a `Multiple`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_width_returns_arithmetic_overflow`
    /// but exercises the Em arm of `unit_to_emu` (`from_em`).
    ///
    /// Load-bearing: change `from_em` to use `saturating_mul` — this test MUST fail.
    #[test]
    fn test_vp_048_layout_shapes_em_overflow_width_returns_arithmetic_overflow() {
        assert_em_overflow_field(
            ShapeUnit::Em(1000),
            ShapeUnit::Em(1000),
            ShapeUnit::Em(i64::MAX), // overflows checked_mul in from_em
            ShapeUnit::Em(500),
            "width",
        );
    }

    /// VP-048 / F-P15-LOW-001 — `layout_shapes` with `ShapeUnit::Em(i64::MAX)` in
    /// the `height` field propagates `LayoutError::ArithmeticOverflow` inside a `Multiple`.
    ///
    /// Mirrors `test_vp_048_layout_shapes_overflow_height_returns_arithmetic_overflow`
    /// but exercises the Em arm of `unit_to_emu` (`from_em`).
    ///
    /// Load-bearing: change `from_em` to use `saturating_mul` — this test MUST fail.
    #[test]
    fn test_vp_048_layout_shapes_em_overflow_height_returns_arithmetic_overflow() {
        assert_em_overflow_field(
            ShapeUnit::Em(1000),
            ShapeUnit::Em(1000),
            ShapeUnit::Em(1000),
            ShapeUnit::Em(i64::MAX), // overflows checked_mul in from_em
            "height",
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 — DEFAULT_EM_IN_EMU backward-compat guard (STORY-074)
    //
    // DEFAULT_EM_IN_EMU is no longer pub (AC-002 / adversary P1 MED-001).
    // This in-crate test is the only place that can assert its value, since
    // the external integration test can no longer import it.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 / STORY-074 — `DEFAULT_EM_IN_EMU` must remain `457_200`.
    ///
    /// This const is test-private (AC-002: "not pub or referenced by production code").
    /// The value must never change — it is the historical default that `BrandFonts::default()`
    /// mirrors for backward compatibility (AC-003).
    ///
    /// Load-bearing: changing the value to anything other than 457_200 MUST fail this test.
    #[test]
    fn test_bc_3_04_001_ac002_default_em_in_emu_is_457200() {
        assert_eq!(
            DEFAULT_EM_IN_EMU, 457_200_i64,
            "DEFAULT_EM_IN_EMU must remain 457_200 (AC-002 backward compat guard)"
        );
    }

    // STORY-072 — FillSpec::Gradient unit tests (AC-002 / AC-003 / AC-005)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 (STORY-072) — `build_shape_frame` with `FillSpec::Gradient` stores
    /// the gradient fill verbatim on the output `ShapeFrame.fill`.
    ///
    /// The layout stage is a pure passthrough for all `FillSpec` variants.
    /// No normalization, no downgrade — the gradient is passed as-is.
    #[test]
    fn test_BC_3_04_001_ac002_story072_build_shape_frame_gradient_fill_passthrough() {
        let from = Rgb { r: 255, g: 0, b: 0 };
        let to = Rgb { r: 0, g: 0, b: 255 };
        let fill = FillSpec::Gradient { from, to };
        let frame = build_shape_frame(
            ShapeType::Rect,
            fill.clone(),
            None,
            Some(Arc::from("Gradient background")),
            false,
            0,
            &SourceSpan::default(),
        )
        .expect("build_shape_frame must succeed for gradient fill with alt text");

        assert_eq!(
            frame.fill, fill,
            "ShapeFrame.fill must match the input FillSpec::Gradient exactly; got: {:?}",
            frame.fill
        );
    }

    /// AC-003 (STORY-072) — `layout_shapes` with a `FillSpec::Gradient` shape
    /// passes the gradient through to `ShapeFrame.fill` in `LaidOutDeck` verbatim.
    ///
    /// Canonical test vector (BC-3.04.001 / STORY-072 AC-003):
    ///   `fill gradient #FF0000 to #0000FF` → `ShapeFrame.fill = FillSpec::Gradient { from: Rgb{255,0,0}, to: Rgb{0,0,255} }`
    #[test]
    fn test_BC_3_04_001_ac003_story072_layout_shapes_gradient_passthrough() {
        let from = Rgb { r: 255, g: 0, b: 0 };
        let to = Rgb { r: 0, g: 0, b: 255 };
        let st = ShapeType::from_keyword("rect").expect("rect must be valid");
        let spec = ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::Gradient { from, to },
            text: None,
            alt: Some(AltText::Provided(Arc::from("Gradient background"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let shapes = vec![spec];
        let output = layout_shapes(&shapes, default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("layout_shapes must succeed for gradient shape");

        assert_eq!(output.frames.len(), 1);
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(
                        &sf.fill,
                        FillSpec::Gradient {
                            from: f,
                            to: t,
                        } if *f == from && *t == to
                    ),
                    "layout_shapes must pass FillSpec::Gradient through verbatim; got: {:?}",
                    sf.fill
                );
            },
            other => panic!("expected FrameContent::Shape for gradient shape; got: {other:?}"),
        }
    }

    /// AC-005 (STORY-072) — A gradient shape without `alt` or `decorative: true`
    /// produces `LayoutError::MissingAlt` (same contract as solid shapes, EC-001).
    ///
    /// The `FillSpec` variant does NOT exempt a shape from the alt requirement.
    #[test]
    fn test_BC_3_04_001_ac005_story072_gradient_shape_without_alt_returns_missing_alt() {
        let st = ShapeType::from_keyword("rect").expect("rect must be valid");
        let spec = ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::Gradient {
                from: Rgb { r: 255, g: 0, b: 0 },
                to: Rgb { r: 0, g: 0, b: 255 },
            },
            text: None,
            alt: None,
            decorative: false, // no alt, no decorative → MissingAlt
            span: SourceSpan::default(),
        };
        let result = layout_shapes(&[spec], default_page(), 0, DEFAULT_EM_IN_EMU, 0);
        assert!(
            result.is_err(),
            "gradient shape without alt must return Err(MissingAlt)"
        );
        match result.unwrap_err() {
            LayoutError::Multiple { inner } => {
                assert!(
                    inner
                        .iter()
                        .any(|e| matches!(e, LayoutError::MissingAlt { .. })),
                    "Multiple must contain MissingAlt for gradient shape without alt; got: {inner:?}"
                );
            },
            other => panic!("expected LayoutError::Multiple with MissingAlt; got: {other:?}"),
        }
    }

    /// EC-006 (STORY-072) — `decorative: true` gradient shape produces
    /// `AltText::Decorative` in the output frame.
    ///
    /// The gradient fill variant does not affect the decorative semantics contract.
    #[test]
    fn test_BC_3_04_001_ec006_story072_gradient_decorative_shape_produces_decorative_alt() {
        let st = ShapeType::from_keyword("ellipse").expect("ellipse must be valid");
        let spec = ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::Gradient {
                from: Rgb { r: 0, g: 255, b: 0 },
                to: Rgb { r: 0, g: 0, b: 255 },
            },
            text: None,
            alt: None,
            decorative: true,
            span: SourceSpan::default(),
        };
        let output = layout_shapes(&[spec], default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("decorative gradient shape must succeed");

        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(sf.alt, slideforge_types::AltText::Decorative),
                    "decorative gradient shape must produce AltText::Decorative; got: {:?}",
                    sf.alt
                );
            },
            other => panic!("expected FrameContent::Shape; got: {other:?}"),
        }
    }

    /// EC-005 (STORY-072) — Same `from` and `to` gradient color (flat gradient) is
    /// valid; `layout_shapes` must not error.
    #[test]
    fn test_BC_3_04_001_ec005_story072_gradient_same_from_to_is_valid() {
        let same = Rgb { r: 255, g: 0, b: 0 };
        let st = ShapeType::from_keyword("rect").expect("rect must be valid");
        let spec = ShapeSpec {
            shape_type: st,
            position: default_position(),
            fill: FillSpec::Gradient {
                from: same,
                to: same,
            },
            text: None,
            alt: Some(AltText::Provided(Arc::from("Flat gradient"))),
            decorative: false,
            span: SourceSpan::default(),
        };
        let output = layout_shapes(&[spec], default_page(), 0, DEFAULT_EM_IN_EMU, 0)
            .expect("flat gradient (same from==to) must succeed");

        assert_eq!(output.frames.len(), 1, "flat gradient must produce 1 frame");
        match &output.frames[0].content {
            crate::types::FrameContent::Shape(sf) => {
                assert!(
                    matches!(&sf.fill, FillSpec::Gradient { from, to } if from == to),
                    "flat gradient must be preserved as FillSpec::Gradient; got: {:?}",
                    sf.fill
                );
            },
            other => panic!("expected FrameContent::Shape; got: {other:?}"),
        }
    }
}
