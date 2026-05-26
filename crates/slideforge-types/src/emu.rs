//! English Metric Units — the canonical unit of measurement in OOXML.
//!
//! OOXML uses EMUs (English Metric Units) for all dimensions. One inch equals
//! 914,400 EMU. One point equals 12,700 EMU. Using integers avoids floating-
//! point non-determinism and enables exact arithmetic proofs in Phase 6 (Kani).

use std::ops::{Add, Div, Mul, Neg, Sub};

/// English Metric Unit — the canonical distance unit for OOXML.
///
/// One inch = 914,400 EMU. One point = 12,700 EMU.
///
/// Use [`Emu::from_inches`] and [`Emu::from_points`] for construction from
/// human-readable measurements. Use the integer value directly when working
/// with raw OOXML attribute values.
///
/// ## Design invariant
///
/// `Emu` is always an `i64` internally so that arithmetic never overflows for
/// any plausible slide dimension (a 100,000-inch canvas is `9.14 × 10¹⁰` EMU,
/// well within `i64::MAX = 9.22 × 10¹⁸`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Emu(pub i64);

/// Standard widescreen slide width (13.33 inches = 12,192,000 EMU).
///
/// This is the PPTX default for widescreen (16:9) presentations.
pub const SLIDE_WIDTH: Emu = Emu(12_192_000);

/// Standard widescreen slide height (7.5 inches = 6,858,000 EMU).
///
/// This is the PPTX default for widescreen (16:9) presentations.
pub const SLIDE_HEIGHT: Emu = Emu(6_858_000);

/// Standard letter-width canvas (10 inches = 9,144,000 EMU).
///
/// Used as a reference canvas width for layout calculations.
pub const CANVAS_WIDTH: Emu = Emu(9_144_000);

/// Standard letter-height canvas (7.5 inches = 6,858,000 EMU).
///
/// Used as a reference canvas height for layout calculations.
pub const CANVAS_HEIGHT: Emu = Emu(6_858_000);

impl Emu {
    /// English Metric Units per inch (914,400).
    pub const EMU_PER_INCH: i64 = 914_400;

    /// English Metric Units per point (12,700).
    pub const EMU_PER_POINT: i64 = 12_700;

    /// Construct an `Emu` from a measurement in inches.
    ///
    /// The conversion is exact when `inches` is a simple fraction whose
    /// denominator divides evenly into 914,400.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Emu;
    /// assert_eq!(Emu::from_inches(1.0), Emu(914_400));
    /// assert_eq!(Emu::from_inches(0.5), Emu(457_200));
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn from_inches(inches: f64) -> Self {
        Emu((inches * (Self::EMU_PER_INCH as f64)).round() as i64)
    }

    /// Construct an `Emu` from a measurement in typographic points.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Emu;
    /// assert_eq!(Emu::from_points(1.0), Emu(12_700));
    /// assert_eq!(Emu::from_points(12.0), Emu(152_400));
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn from_points(points: f64) -> Self {
        Emu((points * (Self::EMU_PER_POINT as f64)).round() as i64)
    }

    /// Return the raw `i64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Emu;
    /// assert_eq!(Emu(914_400).as_i64(), 914_400_i64);
    /// ```
    #[must_use]
    pub fn as_i64(self) -> i64 {
        self.0
    }

    /// Convert to inches as an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Emu;
    /// assert!((Emu(914_400).to_inches() - 1.0_f64).abs() < 1e-10);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_inches(self) -> f64 {
        (self.0 as f64) / (Self::EMU_PER_INCH as f64)
    }

    /// Convert to typographic points as an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::Emu;
    /// assert!((Emu(12_700).to_points() - 1.0_f64).abs() < 1e-10);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn to_points(self) -> f64 {
        (self.0 as f64) / (Self::EMU_PER_POINT as f64)
    }
}

impl Add for Emu {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Emu(self.0 + rhs.0)
    }
}

impl Sub for Emu {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Emu(self.0 - rhs.0)
    }
}

impl Mul<i64> for Emu {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self {
        Emu(self.0 * rhs)
    }
}

impl Mul<Emu> for i64 {
    type Output = Emu;

    fn mul(self, rhs: Emu) -> Emu {
        Emu(self * rhs.0)
    }
}

impl Div<i64> for Emu {
    type Output = Self;

    /// # Panics
    ///
    /// Panics if `rhs` is zero (standard integer division behavior).
    fn div(self, rhs: i64) -> Self {
        Emu(self.0 / rhs)
    }
}

impl Neg for Emu {
    type Output = Self;

    fn neg(self) -> Self {
        Emu(-self.0)
    }
}

impl std::fmt::Display for Emu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}emu", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_01_001_emu_from_inches_one_inch() {
        assert_eq!(Emu::from_inches(1.0), Emu(914_400));
    }

    #[test]
    fn test_bc_1_01_001_emu_from_inches_half_inch() {
        assert_eq!(Emu::from_inches(0.5), Emu(457_200));
    }

    #[test]
    fn test_bc_1_01_001_emu_from_points_one_point() {
        assert_eq!(Emu::from_points(1.0), Emu(12_700));
    }

    #[test]
    fn test_bc_1_01_001_emu_from_points_twelve_points() {
        assert_eq!(Emu::from_points(12.0), Emu(152_400));
    }

    #[test]
    fn test_bc_1_01_001_emu_add() {
        assert_eq!(Emu(100) + Emu(200), Emu(300));
    }

    #[test]
    fn test_bc_1_01_001_emu_sub() {
        assert_eq!(Emu(300) - Emu(100), Emu(200));
    }

    #[test]
    fn test_bc_1_01_001_emu_mul_scalar() {
        assert_eq!(Emu(100) * 3, Emu(300));
    }

    #[test]
    fn test_bc_1_01_001_emu_mul_scalar_commutative() {
        assert_eq!(3 * Emu(100), Emu(300));
    }

    #[test]
    fn test_bc_1_01_001_emu_canvas_width_constant() {
        // Canvas width referenced in the spec: 10 inches = 9,144,000 EMU.
        assert_eq!(CANVAS_WIDTH, Emu(9_144_000));
    }

    #[test]
    fn test_bc_1_01_001_emu_slide_width_constant() {
        assert_eq!(SLIDE_WIDTH, Emu(12_192_000));
    }

    #[test]
    fn test_bc_1_01_001_emu_implements_copy() {
        let a = Emu(42);
        let b = a; // Copy — no move
        let c = a; // Still usable (Copy trait)
        assert_eq!(b, Emu(42));
        assert_eq!(c, Emu(42));
    }

    #[test]
    fn test_bc_1_01_001_emu_implements_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<Emu, &str> = HashMap::new();
        map.insert(Emu(914_400), "one inch");
        assert_eq!(map[&Emu(914_400)], "one inch");
    }

    #[test]
    fn test_bc_1_01_001_emu_default_is_zero() {
        assert_eq!(Emu::default(), Emu(0));
    }

    #[test]
    fn test_bc_1_01_001_emu_per_inch_constant() {
        assert_eq!(Emu::EMU_PER_INCH, 914_400_i64);
    }

    #[test]
    fn test_bc_1_01_001_emu_per_point_constant() {
        assert_eq!(Emu::EMU_PER_POINT, 12_700_i64);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn test_bc_1_01_001_emu_roundtrip_inches() {
        let original = Emu::from_inches(2.5);
        let roundtripped = (original.to_inches() * (Emu::EMU_PER_INCH as f64)).round() as i64;
        assert_eq!(original.0, roundtripped);
    }

    #[test]
    fn test_bc_1_01_001_emu_ordering() {
        assert!(Emu(100) < Emu(200));
        assert!(Emu(200) > Emu(100));
        assert_eq!(Emu(100), Emu(100));
    }

    // ──────────────────────────────────────────────────────────────────────────
    // STORY-004 AC-010 — Div<i64> and Neg
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_emu_div_scalar() {
        assert_eq!(Emu(300) / 3, Emu(100));
    }

    #[test]
    fn test_bc_1_02_003_emu_div_by_one() {
        assert_eq!(Emu(914_400) / 1, Emu(914_400));
    }

    #[test]
    fn test_bc_1_02_003_emu_neg() {
        assert_eq!(-Emu(100), Emu(-100));
    }

    #[test]
    fn test_bc_1_02_003_emu_neg_zero() {
        assert_eq!(-Emu(0), Emu(0));
    }

    #[test]
    fn test_bc_1_02_003_emu_neg_negative() {
        assert_eq!(-Emu(-50), Emu(50));
    }

    /// AC-010: Standard 16:9 slide dimensions using arithmetic (10in × 5.625in
    /// in terms of the canvas reference).
    #[test]
    fn test_bc_1_02_003_emu_canvas_width_ten_inches() {
        // 10 inches = 9,144,000 EMU
        assert_eq!(Emu::from_inches(10.0), Emu(9_144_000));
        assert_eq!(Emu::from_inches(10.0), CANVAS_WIDTH);
    }

    /// `to_points` roundtrip — AC-011.
    #[test]
    fn test_bc_1_02_003_emu_to_points_roundtrip() {
        let original = Emu::from_points(72.0); // 72pt = 1 inch
        let back = original.to_points();
        assert!(
            (back - 72.0_f64).abs() < 0.001,
            "to_points roundtrip failed: got {back}"
        );
    }

    /// `to_inches` roundtrip — AC-011.
    #[test]
    fn test_bc_1_02_003_emu_to_inches_roundtrip() {
        let original = Emu::from_inches(5.0);
        let back = original.to_inches();
        assert!(
            (back - 5.0_f64).abs() < 1e-10,
            "to_inches roundtrip failed: got {back}"
        );
    }

    /// EC-004: `from_inches(0.0)` = Emu(0).
    #[test]
    fn test_bc_1_02_003_emu_from_inches_zero() {
        assert_eq!(Emu::from_inches(0.0), Emu(0));
    }

    /// Division by zero panics with standard Rust integer behavior.
    #[test]
    #[should_panic(expected = "divide by zero")]
    fn emu_div_by_zero_panics() {
        let _ = Emu(100) / 0;
    }
}
