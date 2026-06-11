//! Shape IR types — `ShapeType`, `FillSpec`, `Rgb`, `LayoutWarning`.
//!
//! These types are part of the public layout output contract and live in
//! `slideforge-types` (the leaf crate with no workspace crate dependencies) so
//! that both `slideforge-syntax` (the parser), `slideforge-layout` (the producer),
//! and exporter crates can reference them without circular dependencies.
//!
//! ## Design Invariants
//!
//! - All types implement `Debug + Clone + PartialEq + Eq + Hash` for comemo
//!   compatibility (AC-010) and Kani bounded-model checking (Phase 6).
//! - No `f64` anywhere — integer-only fields throughout.
//! - `Arc<str>` for string fields (cheap clone, `Hash + Eq` compatible).
//!
//! ## Relocations (STORY-028 pass-2 schema)
//!
//! `ShapeType`, `FillSpec`, `Rgb`, and `LayoutWarning` were relocated from
//! `slideforge-layout/src/types.rs` to this module so that `slideforge-types`
//! (which carries `ShapeSpec`) can reference the resolved enum type directly.
//! `slideforge-layout` re-exports them for backward compatibility.

use std::sync::Arc;

use crate::emu::Emu;

// ─────────────────────────────────────────────────────────────────────────────
// Rgb
// ─────────────────────────────────────────────────────────────────────────────

/// An sRGB color value.
///
/// Used in [`FillSpec::SolidColor`] to carry per-channel color data.
/// Integer channels (0–255); no floating-point.
///
/// Implements `Hash + Eq + Clone + Debug` for comemo compatibility (AC-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// Red channel (0–255).
    pub r: u8,
    /// Green channel (0–255).
    pub g: u8,
    /// Blue channel (0–255).
    pub b: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// FillSpec
// ─────────────────────────────────────────────────────────────────────────────

/// The fill specification for a shape or shape frame.
///
/// Exporters translate `FillSpec` into format-specific fill markup (PPTX
/// `<a:solidFill>`, HTML `background-color`, PDF fill ops).
///
/// ## v1.0 scope
///
/// Implements `Hash + Eq + Clone + Debug` for comemo compatibility (AC-010).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FillSpec {
    /// A flat solid color fill.
    SolidColor(Rgb),
    /// No fill (transparent background).
    None,
    /// A linear gradient fill (top-to-bottom, vertical, v1.0).
    ///
    /// Added by STORY-072. `from` is the top color, `to` is the bottom color.
    /// Both colors follow the same hex-parsing rules as `SolidColor`.
    /// Gradient direction in v1.0 is fixed as top-to-bottom (vertical linear);
    /// arbitrary-angle gradients are deferred to v2.
    ///
    /// PPTX: `a:gradFill` / `a:gsLst` / `a:lin ang="5400000"` (ooxmlsdk typed builders).
    /// PDF: krilla `paint::LinearGradient` + `Surface::set_fill` + `draw_path`.
    /// HTML: SVG-native `<defs><linearGradient>` def + `fill="url(#sf-grad-...)"` on the shape
    /// `<rect>` (CSS `background` does NOT paint SVG geometry — see `render_shape_svg`).
    /// DOCX: solid fallback to `from` color with lint warning.
    Gradient {
        /// Top (start) color of the linear gradient.
        from: Rgb,
        /// Bottom (end) color of the linear gradient.
        to: Rgb,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// ShapeTypeError
// ─────────────────────────────────────────────────────────────────────────────

/// Error returned by [`ShapeType::from_keyword`] when an unknown keyword is
/// supplied (interface-definitions.md §9.2 / BC-3.04.001 invariant 4).
///
/// ## Design rationale (§9.2)
///
/// `ShapeTypeError` is a minimal self-contained error type in `slideforge-types`
/// (the leaf crate with no workspace crate deps). This avoids an upward
/// dependency from `slideforge-types` into `slideforge-layout`. The parse
/// stage surfaces this as `E-PAR-012`; the layout crate no longer exposes a
/// separate `LayoutError::UnknownShapeType` (removed in STORY-028 pass-4,
/// F-P4-MED-001 — unreachable after `ShapeSpec.shape_type` became `ShapeType`).
///
/// The keyword is stored as `Arc<str>` to avoid unnecessary allocation on the
/// caller side — callers that forward the keyword into a `LayoutError` can move
/// it directly.
///
/// Implements `Debug + Clone + PartialEq + Eq + Hash` for comemo compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeTypeError {
    /// The unrecognised keyword that was supplied to `from_keyword`.
    pub keyword: Arc<str>,
}

impl std::fmt::Display for ShapeTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown shape type keyword '{}': expected one of [rect, ellipse, arrow, line, star, roundRect]",
            self.keyword
        )
    }
}

impl std::error::Error for ShapeTypeError {}

// ─────────────────────────────────────────────────────────────────────────────
// ShapeType
// ─────────────────────────────────────────────────────────────────────────────

/// The geometric shape type for a shape block.
///
/// Corresponds to the `type` field in the DSL `shape:` block. This is a
/// **closed vocabulary** in v1.0 — exactly 6 keywords are accepted. Any
/// unknown keyword produces `E-PAR-012` at parse time (BC-3.04.001 invariant 4).
/// There is NO `Custom` variant; the type system enforces the closed vocabulary.
///
/// ## Placement in slideforge-types
///
/// `ShapeType` lives in `slideforge-types` (the leaf IR crate) so that
/// `ShapeSpec` (the pre-layout semantic type) can carry a resolved `ShapeType`
/// directly, rather than an unvalidated `Arc<str>`. This eliminates a whole
/// class of "unknown shape type" bugs that previously could only be detected at
/// layout time. `slideforge-layout` re-exports `ShapeType` for back-compat.
///
/// Implements `Hash + Eq + Clone + Debug` for comemo compatibility (AC-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeType {
    /// Rectangular shape (`type rect`).
    Rect,
    /// Ellipse / circle shape (`type ellipse`).
    Ellipse,
    /// Single-headed arrow (`type arrow`).
    Arrow,
    /// Line segment (`type line`).
    Line,
    /// Star / burst shape (`type star`).
    Star,
    /// Rounded-corner rectangle (`type roundRect`).
    ///
    /// Added in BC-3.04.001 v1.3 (per Q7 decision example).
    RoundRect,
    // NOTE: No Custom variant. Unknown keywords are parse errors (E-PAR-012).
    // See BC-3.04.001 invariant 4 and CLAUDE.md "no silent fallback" rule.
}

impl ShapeType {
    /// Parse a shape type keyword string into a [`ShapeType`] enum variant.
    ///
    /// Known keywords (closed v1.0 vocabulary per BC-3.04.001 invariant 4):
    /// `"rect"`, `"ellipse"`, `"arrow"`, `"line"`, `"star"`, `"roundRect"`.
    ///
    /// Keywords are **case-sensitive** per the DSL spec (BC-3.04.001 invariant 4).
    ///
    /// Returns `Err(ShapeTypeError { keyword })` for any unknown keyword.
    /// There is NO `Custom` fallback — the type system enforces the closed
    /// vocabulary. The parse stage surfaces this as `E-PAR-012`
    /// (interface-definitions.md §9.2).
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::ShapeType;
    ///
    /// assert_eq!(ShapeType::from_keyword("rect"), Ok(ShapeType::Rect));
    /// assert_eq!(ShapeType::from_keyword("roundRect"), Ok(ShapeType::RoundRect));
    /// assert!(ShapeType::from_keyword("unknown").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ShapeTypeError`] when `kw` is not in the closed v1.0 vocabulary.
    pub fn from_keyword(kw: &str) -> Result<Self, ShapeTypeError> {
        match kw {
            "rect" => Ok(Self::Rect),
            "ellipse" => Ok(Self::Ellipse),
            "arrow" => Ok(Self::Arrow),
            "line" => Ok(Self::Line),
            "star" => Ok(Self::Star),
            "roundRect" => Ok(Self::RoundRect),
            _ => Err(ShapeTypeError {
                keyword: Arc::from(kw),
            }),
        }
    }

    /// Return the canonical DSL keyword for this shape type.
    ///
    /// The returned string is always a member of the closed v1.0 vocabulary:
    /// `"rect"`, `"ellipse"`, `"arrow"`, `"line"`, `"star"`, `"roundRect"`.
    ///
    /// Useful for error messages and display purposes.
    #[must_use]
    pub fn as_keyword(&self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Line => "line",
            Self::Star => "star",
            Self::RoundRect => "roundRect",
        }
    }
}

impl std::fmt::Display for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_keyword())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LayoutWarning
// ─────────────────────────────────────────────────────────────────────────────

/// A non-fatal diagnostic produced by the layout engine.
///
/// `LayoutWarning`s are accumulated during layout and do not halt processing.
/// They are stored on `LaidOutDeck::warnings` in `slideforge-layout`.
///
/// ## Placement in slideforge-types
///
/// `LayoutWarning` lives in `slideforge-types` so that the `warnings` field on
/// `LaidOutDeck` is part of the public IR contract shared across crates, and so
/// that exporter crates can inspect warnings without depending on the layout
/// implementation crate.
///
/// `slideforge-layout` re-exports `LayoutWarning` for backward compatibility.
///
/// ## Field-naming convention (interface-definitions.md §8 / BC-3.04.001)
///
/// All variants use `source_slide_index: usize` (canonical field name) — not
/// `slide_index`, `idx`, or any other spelling.
///
/// Implements `Hash + Eq + Clone + Debug` for comemo compatibility (AC-010).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayoutWarning {
    /// A shape's declared position extends outside the slide canvas (AC-003 /
    /// BC-3.04.001 EC-002). Negative x or y, or x+width > `page_width`, etc.
    ///
    /// `source_slide_index` follows the canonical field-naming convention from
    /// interface-definitions.md §8 (F-HIGH-001 / full symmetry with `LayoutError`).
    OffCanvas {
        /// Zero-based index of the slide containing the off-canvas shape.
        /// Canonical field name per interface-definitions.md §8 (F-HIGH-001).
        source_slide_index: usize,
        /// The resolved shape type enum variant (interface-definitions.md §9.3 /
        /// F-HIGH-003 binding). MUST be the enum variant, not `Arc<str>`.
        shape_type: ShapeType,
        /// The declared x position in EMU (may be negative).
        x_emu: Emu,
        /// The declared y position in EMU (may be negative).
        y_emu: Emu,
    },
    /// An `Xref` inline node references a slide title that does not exist in
    /// the deck (AC-007 / BC-3.05.001 EC-002).
    ///
    /// `source_slide_index` follows the canonical field-naming convention from
    /// interface-definitions.md §8 (F-HIGH-001 / full symmetry with `LayoutError`).
    XrefTargetNotFound {
        /// The target identifier that was not found.
        target: Arc<str>,
        /// Zero-based index of the slide containing the unresolved xref.
        /// Canonical field name per interface-definitions.md §8 (F-HIGH-001).
        source_slide_index: usize,
    },
    // NOTE: InlineMarkupInTitle was previously defined here but was DEAD CODE.
    // The live diagnostic is EvalError::InlineMarkupInTitle (E-EVL-015) in
    // slideforge-eval (crates/slideforge-eval/src/error.rs). That variant is
    // emitted by eval_slide_node in for_eval.rs and surfaces through the strict
    // gate in slideforge::compile_inner as BuildError::EvalFailed. No production
    // code ever constructed LayoutWarning::InlineMarkupInTitle; the variant was
    // removed by F-P25-MED-001. (STORY-081 Pass-25 adversary finding.)
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, non_snake_case)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ─────────────────────────────────────────────────────────────────────────
    // ShapeType
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_shape_type_from_keyword_all_valid() {
        assert_eq!(ShapeType::from_keyword("rect"), Ok(ShapeType::Rect));
        assert_eq!(ShapeType::from_keyword("ellipse"), Ok(ShapeType::Ellipse));
        assert_eq!(ShapeType::from_keyword("arrow"), Ok(ShapeType::Arrow));
        assert_eq!(ShapeType::from_keyword("line"), Ok(ShapeType::Line));
        assert_eq!(ShapeType::from_keyword("star"), Ok(ShapeType::Star));
        assert_eq!(
            ShapeType::from_keyword("roundRect"),
            Ok(ShapeType::RoundRect)
        );
    }

    #[test]
    fn test_shape_type_from_keyword_unknown_returns_err() {
        // Unknown keywords return Err(ShapeTypeError) — not Option::None.
        assert!(ShapeType::from_keyword("frobnicator").is_err());
        assert!(ShapeType::from_keyword("").is_err());
        assert!(ShapeType::from_keyword("Rect").is_err()); // case-sensitive
        assert!(ShapeType::from_keyword("RECT").is_err());
        // The error carries the original keyword for diagnostics.
        // Use expect_err to satisfy clippy::unwrap_used in test code.
        let err = ShapeType::from_keyword("frobnicator")
            .expect_err("frobnicator is not a valid ShapeType keyword");
        assert_eq!(err.keyword.as_ref(), "frobnicator");
    }

    #[test]
    fn test_shape_type_as_keyword_round_trips() {
        let variants = [
            ShapeType::Rect,
            ShapeType::Ellipse,
            ShapeType::Arrow,
            ShapeType::Line,
            ShapeType::Star,
            ShapeType::RoundRect,
        ];
        for v in variants {
            let kw = v.as_keyword();
            assert_eq!(
                ShapeType::from_keyword(kw),
                Ok(v),
                "as_keyword + from_keyword must round-trip for {v:?}"
            );
        }
    }

    /// `ShapeTypeError` carries the failing keyword and displays a human-readable message.
    #[test]
    fn test_shape_type_error_carries_keyword_and_displays() {
        let err = ShapeTypeError {
            keyword: Arc::from("unknownType"),
        };
        assert_eq!(err.keyword.as_ref(), "unknownType");
        let msg = err.to_string();
        assert!(
            msg.contains("unknownType"),
            "ShapeTypeError display must include the keyword; got: {msg}"
        );
        assert!(
            msg.contains("rect"),
            "ShapeTypeError display must list valid keywords; got: {msg}"
        );
    }

    /// `ShapeTypeError` implements `Clone + PartialEq + Eq + Hash` (comemo AC-010).
    #[test]
    fn test_shape_type_error_implements_hash_eq_clone() {
        use std::collections::HashSet;
        let e1 = ShapeTypeError {
            keyword: Arc::from("bad"),
        };
        let e2 = e1.clone();
        assert_eq!(e1, e2);
        let mut set = HashSet::new();
        set.insert(e1);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_shape_type_display() {
        assert_eq!(format!("{}", ShapeType::Rect), "rect");
        assert_eq!(format!("{}", ShapeType::RoundRect), "roundRect");
    }

    #[test]
    fn test_shape_type_implements_hash_eq_clone() {
        let mut set = HashSet::new();
        set.insert(ShapeType::Rect);
        set.insert(ShapeType::Rect); // duplicate
        assert_eq!(set.len(), 1);
        let v = ShapeType::Ellipse;
        let v2 = v;
        assert_eq!(v, v2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FillSpec
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_fill_spec_variants_constructable() {
        let solid = FillSpec::SolidColor(Rgb {
            r: 0,
            g: 55,
            b: 102,
        });
        let none = FillSpec::None;
        assert!(matches!(solid, FillSpec::SolidColor(_)));
        assert!(matches!(none, FillSpec::None));
    }

    #[test]
    fn test_fill_spec_implements_hash_eq_clone() {
        let mut set = HashSet::new();
        set.insert(FillSpec::SolidColor(Rgb {
            r: 0,
            g: 55,
            b: 102,
        }));
        set.insert(FillSpec::SolidColor(Rgb {
            r: 0,
            g: 55,
            b: 102,
        }));
        assert_eq!(set.len(), 1);
        let v = FillSpec::None;
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // STORY-072: FillSpec::Gradient — AC-002
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 (BC-3.04.001 postcondition 1 — STORY-072): `FillSpec::Gradient` variant
    /// is constructable with `from` and `to` `Rgb` fields.
    ///
    /// Canonical test vector: `fill gradient #FF0000 to #0000FF` →
    /// `FillSpec::Gradient { from: Rgb { r: 255, g: 0, b: 0 }, to: Rgb { r: 0, g: 0, b: 255 } }`
    #[test]
    fn test_BC_3_04_001_ac002_gradient_variant_constructable() {
        let grad = FillSpec::Gradient {
            from: Rgb { r: 255, g: 0, b: 0 },
            to: Rgb { r: 0, g: 0, b: 255 },
        };
        // Verify both field accessors.
        let (from_color, to_color) = match &grad {
            FillSpec::Gradient { from, to } => (*from, *to),
            _ => panic!("expected FillSpec::Gradient"),
        };
        assert_eq!(from_color, Rgb { r: 255, g: 0, b: 0 }, "from must be red");
        assert_eq!(to_color, Rgb { r: 0, g: 0, b: 255 }, "to must be blue");
    }

    /// AC-002: `FillSpec::Gradient` implements `Debug + Clone + PartialEq + Eq + Hash`
    /// for comemo compatibility (DI-010 / BC-3.04.001 postcondition 1).
    #[test]
    fn test_BC_3_04_001_ac002_gradient_implements_hash_eq_clone() {
        let g1 = FillSpec::Gradient {
            from: Rgb { r: 255, g: 0, b: 0 },
            to: Rgb { r: 0, g: 0, b: 255 },
        };
        let g2 = g1.clone();
        assert_eq!(g1, g2, "cloned Gradient must equal original");

        let mut set = HashSet::new();
        set.insert(g1.clone());
        set.insert(g1);
        assert_eq!(
            set.len(),
            1,
            "duplicate Gradient must deduplicate in HashSet"
        );
    }

    /// AC-002: Two `FillSpec::Gradient` with different colors are NOT equal.
    #[test]
    fn test_BC_3_04_001_ac002_gradient_inequality() {
        let g1 = FillSpec::Gradient {
            from: Rgb { r: 255, g: 0, b: 0 },
            to: Rgb { r: 0, g: 0, b: 255 },
        };
        let g2 = FillSpec::Gradient {
            from: Rgb { r: 0, g: 255, b: 0 },
            to: Rgb { r: 0, g: 0, b: 255 },
        };
        assert_ne!(
            g1, g2,
            "Gradients with different from-colors must not be equal"
        );
    }

    /// AC-002: `FillSpec::Gradient` is distinct from `FillSpec::SolidColor` and
    /// `FillSpec::None` in the type system.
    #[test]
    fn test_BC_3_04_001_ac002_gradient_distinct_from_solid_and_none() {
        let grad = FillSpec::Gradient {
            from: Rgb { r: 255, g: 0, b: 0 },
            to: Rgb { r: 0, g: 0, b: 255 },
        };
        let solid = FillSpec::SolidColor(Rgb { r: 255, g: 0, b: 0 });
        let none = FillSpec::None;
        assert_ne!(grad, solid, "Gradient must not equal SolidColor");
        assert_ne!(grad, none, "Gradient must not equal None");
    }

    /// EC-005 (STORY-072): Same `from` and `to` color is valid — equivalent to a flat
    /// gradient (visually solid). No error or panic.
    #[test]
    fn test_BC_3_04_001_ec005_same_from_and_to_is_valid() {
        let same_color = Rgb { r: 255, g: 0, b: 0 };
        let grad = FillSpec::Gradient {
            from: same_color,
            to: same_color,
        };
        // Matching both fields must succeed.
        assert!(
            matches!(grad, FillSpec::Gradient { from, to } if from == to),
            "same from==to gradient must be constructable and match-able"
        );
    }

    /// AC-002 invariant: `FillSpec::Gradient` `from` and `to` carry integer `u8` channels —
    /// no `f64` anywhere (DI-010 / CLAUDE.md "no f64 in IR" rule).
    ///
    /// This is a compile-time test: if `Rgb.r/g/b` were `f64`, the literal `255_u8`
    /// assignment below would fail to compile.
    #[test]
    fn test_BC_3_04_001_ac002_gradient_rgb_channels_are_u8_not_f64() {
        let r: u8 = 255;
        let g: u8 = 0;
        let b: u8 = 0;
        let _ = FillSpec::Gradient {
            from: Rgb { r, g, b },
            to: Rgb { r: 0, g: 0, b },
        };
        // If this compiles, channels are u8 (not f64). Test passes vacuously on compile success.
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Rgb
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_rgb_fields() {
        let rgb = Rgb {
            r: 255,
            g: 128,
            b: 0,
        };
        assert_eq!(rgb.r, 255);
        assert_eq!(rgb.g, 128);
        assert_eq!(rgb.b, 0);
    }

    #[test]
    fn test_rgb_implements_hash_eq_clone() {
        let mut set = HashSet::new();
        set.insert(Rgb { r: 0, g: 0, b: 0 });
        set.insert(Rgb { r: 0, g: 0, b: 0 });
        assert_eq!(set.len(), 1);
        let v = Rgb { r: 1, g: 2, b: 3 };
        let v2 = v;
        assert_eq!(v, v2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LayoutWarning
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_layout_warning_offcanvas_constructable() {
        // F-HIGH-003: shape_type is ShapeType (enum variant), not Arc<str>.
        let w = LayoutWarning::OffCanvas {
            source_slide_index: 0,
            shape_type: ShapeType::Rect,
            x_emu: Emu(-457_200),
            y_emu: Emu(914_400),
        };
        assert!(matches!(w, LayoutWarning::OffCanvas { .. }));
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }

    /// F-HIGH-003 compile test: `OffCanvas.shape_type` must be `ShapeType` enum, not `Arc<str>`.
    ///
    /// If this compiles with `shape_type: ShapeType::Ellipse`, the binding is correct.
    #[test]
    fn test_layout_warning_offcanvas_shape_type_is_enum_not_string() {
        // Construct the warning — if shape_type were Arc<str>, this would not compile.
        let w = LayoutWarning::OffCanvas {
            source_slide_index: 1,
            shape_type: ShapeType::Ellipse,
            x_emu: Emu(0),
            y_emu: Emu(0),
        };
        // Assert the field is accessible as ShapeType, not Arc<str>.
        assert!(matches!(
            w,
            LayoutWarning::OffCanvas {
                shape_type: ShapeType::Ellipse,
                ..
            }
        ));
    }

    #[test]
    fn test_layout_warning_xref_not_found_constructable() {
        let w = LayoutWarning::XrefTargetNotFound {
            target: Arc::from("slide-99"),
            source_slide_index: 3,
        };
        assert!(matches!(w, LayoutWarning::XrefTargetNotFound { .. }));
        let w2 = w.clone();
        assert_eq!(w, w2);
        let mut set = HashSet::new();
        set.insert(w);
        assert_eq!(set.len(), 1);
    }

    /// Verify canonical field name `source_slide_index` (not `slide_index`) is
    /// present on both variants (interface-definitions.md §8 / BC-3.04.001).
    #[test]
    fn test_layout_warning_canonical_field_name_source_slide_index() {
        // Compile-time proof: if the field were named differently, these would not compile.
        assert!(matches!(
            LayoutWarning::OffCanvas {
                source_slide_index: 0,
                shape_type: ShapeType::Rect,
                x_emu: Emu(0),
                y_emu: Emu(0),
            },
            LayoutWarning::OffCanvas {
                source_slide_index: 0,
                ..
            }
        ));
        assert!(matches!(
            LayoutWarning::XrefTargetNotFound {
                target: Arc::from("t"),
                source_slide_index: 1,
            },
            LayoutWarning::XrefTargetNotFound {
                source_slide_index: 1,
                ..
            }
        ));
    }
}
