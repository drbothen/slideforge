//! `MergePrecedence` — the 11-level precedence chain for value merging.
//!
//! When the same field is set by multiple sources (e.g., a deck-level `vars:`
//! block AND a per-slide override AND a CLI flag), the value with the highest
//! `MergePrecedence` wins.  This is the "last-wins scalars" merge strategy
//! from the DSL design decisions (q1-decision-final.md §merge-semantics and
//! q16-q25-decisions.md).

/// Precedence levels for the 11-level merge chain.
///
/// Variants are declared in lowest-to-highest order so that the derived `Ord`
/// implementation gives `Default < SlideTypeDefault < … < Internal`.
///
/// ## Level ordering (lowest → highest)
///
/// | Level | Variant | Source |
/// |-------|---------|--------|
/// | 0 | [`Default`][MergePrecedence::Default] | Hardcoded defaults in slide type definitions |
/// | 1 | [`SlideTypeDefault`][MergePrecedence::SlideTypeDefault] | Defaults declared via `default:` in `optional_fields()` |
/// | 2 | [`SetRule`][MergePrecedence::SetRule] | `set <type>: <field> <value>` rules |
/// | 3 | [`DataSource`][MergePrecedence::DataSource] | Values from `@data` bindings |
/// | 4 | [`DeckVar`][MergePrecedence::DeckVar] | `vars:` block in deck metadata |
/// | 5 | [`IncludeVar`][MergePrecedence::IncludeVar] | Vars passed through `@include` |
/// | 6 | [`VariantVar`][MergePrecedence::VariantVar] | `variants: { vars: ... }` overrides |
/// | 7 | [`CliFlag`][MergePrecedence::CliFlag] | `--variant` and other CLI overrides |
/// | 8 | [`PerSlideOverride`][MergePrecedence::PerSlideOverride] | Explicit field values on individual slides |
/// | 9 | [`BrandOverlay`][MergePrecedence::BrandOverlay] | `brand_overlay:` per-slide overrides |
/// | 10 | [`Internal`][MergePrecedence::Internal] | System-internal values (e.g., generated slide numbers) |
///
/// ## No coercion
///
/// `MergePrecedence` is the mechanism by which one value REPLACES another —
/// it is not a coercion mechanism. The winning `Value` is used as-is;
/// its type is never changed during merge (BC-1.02.003 invariant 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MergePrecedence {
    /// Level 0 — hardcoded defaults baked into slide type definitions.
    Default,

    /// Level 1 — defaults declared via `default:` in a slide type's
    /// `optional_fields()` implementation.
    SlideTypeDefault,

    /// Level 2 — values applied by `set <type>: <field> <value>` rules.
    SetRule,

    /// Level 3 — values from `@data` source bindings.
    DataSource,

    /// Level 4 — variables declared in the deck `vars:` block.
    DeckVar,

    /// Level 5 — variables passed through `@include` calls.
    IncludeVar,

    /// Level 6 — variables from `variants: { vars: ... }` overrides.
    VariantVar,

    /// Level 7 — CLI-flag overrides (`--variant` and similar).
    CliFlag,

    /// Level 8 — explicit field values declared on individual slides.
    PerSlideOverride,

    /// Level 9 — `brand_overlay:` per-slide brand overrides.
    BrandOverlay,

    /// Level 10 — system-internal values set by the slideforge engine
    /// (e.g., auto-generated slide numbers).
    ///
    /// This variant is NEVER set by user DSL; it is set exclusively by
    /// system code.
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-007 — MergePrecedence ordering (Default < ... < Internal)
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_02_003_precedence_default_is_lowest() {
        assert!(MergePrecedence::Default < MergePrecedence::SlideTypeDefault);
    }

    #[test]
    fn test_bc_1_02_003_precedence_slide_type_default_lt_set_rule() {
        assert!(MergePrecedence::SlideTypeDefault < MergePrecedence::SetRule);
    }

    #[test]
    fn test_bc_1_02_003_precedence_set_rule_lt_data_source() {
        assert!(MergePrecedence::SetRule < MergePrecedence::DataSource);
    }

    #[test]
    fn test_bc_1_02_003_precedence_data_source_lt_deck_var() {
        assert!(MergePrecedence::DataSource < MergePrecedence::DeckVar);
    }

    #[test]
    fn test_bc_1_02_003_precedence_deck_var_lt_include_var() {
        assert!(MergePrecedence::DeckVar < MergePrecedence::IncludeVar);
    }

    #[test]
    fn test_bc_1_02_003_precedence_include_var_lt_variant_var() {
        assert!(MergePrecedence::IncludeVar < MergePrecedence::VariantVar);
    }

    #[test]
    fn test_bc_1_02_003_precedence_variant_var_lt_cli_flag() {
        assert!(MergePrecedence::VariantVar < MergePrecedence::CliFlag);
    }

    #[test]
    fn test_bc_1_02_003_precedence_cli_flag_lt_per_slide_override() {
        assert!(MergePrecedence::CliFlag < MergePrecedence::PerSlideOverride);
    }

    #[test]
    fn test_bc_1_02_003_precedence_per_slide_lt_brand_overlay() {
        assert!(MergePrecedence::PerSlideOverride < MergePrecedence::BrandOverlay);
    }

    #[test]
    fn test_bc_1_02_003_precedence_brand_overlay_lt_internal() {
        assert!(MergePrecedence::BrandOverlay < MergePrecedence::Internal);
    }

    /// Complete chain: Default is the minimum, Internal is the maximum.
    #[test]
    fn test_bc_1_02_003_precedence_full_chain() {
        use MergePrecedence::*;
        let ordered = [
            Default,
            SlideTypeDefault,
            SetRule,
            DataSource,
            DeckVar,
            IncludeVar,
            VariantVar,
            CliFlag,
            PerSlideOverride,
            BrandOverlay,
            Internal,
        ];
        for window in ordered.windows(2) {
            assert!(
                window[0] < window[1],
                "{:?} must be < {:?}",
                window[0],
                window[1]
            );
        }
    }

    /// Equality — a variant equals itself.
    #[test]
    fn test_bc_1_02_003_precedence_equality() {
        assert_eq!(MergePrecedence::CliFlag, MergePrecedence::CliFlag);
    }

    /// Hash — usable as a map key.
    #[test]
    fn test_bc_1_02_003_precedence_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(MergePrecedence::Default);
        set.insert(MergePrecedence::Internal);
        set.insert(MergePrecedence::Default); // duplicate
        assert_eq!(set.len(), 2);
    }

    /// Clone + Copy.
    #[test]
    fn test_bc_1_02_003_precedence_copy() {
        let p = MergePrecedence::DeckVar;
        let p2 = p; // Copy
        let p3 = p; // still valid
        assert_eq!(p2, MergePrecedence::DeckVar);
        assert_eq!(p3, MergePrecedence::DeckVar);
    }

    /// `max()` helper — higher-precedence value wins in a merge.
    #[test]
    fn test_bc_1_02_003_precedence_max_wins() {
        let a = MergePrecedence::DeckVar;
        let b = MergePrecedence::CliFlag;
        assert_eq!(a.max(b), MergePrecedence::CliFlag);
    }
}
