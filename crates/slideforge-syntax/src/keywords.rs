//! Reserved keyword registry for the slideforge DSL.
//!
//! This module provides compile-time lookup tables for three categories of
//! reserved names:
//!
//! 1. **Reserved structural keywords** — names used as DSL keywords at the
//!    deck / slide level (`vars`, `set`, `alias`, `variants`).
//!    Note: `section` was un-reserved in STORY-078 and is now an active
//!    block-level introducer handled by `parser/section.rs`.
//! 2. **Reserved directive keywords** — `@`-prefixed names (`@for`, `@if`,
//!    `@elif`, `@else`, `@include`, `@data`, `@fn`, `@mixin`, `@while`).
//! 3. **Slide-type keywords** — the 35 DSL keywords that must not be used as
//!    variable names (e.g. `title`, `chart`, `content`). This is a deliberate
//!    superset of the 34 registered `SlideType` implementations: `severity_cards`
//!    is a color-coded scan target (region frames + `LabelCheck`/`COLOR_CODED`
//!    coverage) but has no standalone `SlideType` registration.
//!
//! # Usage
//!
//! The parser calls these functions when processing `vars:` entries and
//! bare identifiers at positions where a keyword would be ambiguous:
//!
//! - [`classify_keyword`] — returns `(error_code, description)` if `kw` is
//!   reserved in any category, or `None` if it is a safe user-defined name.
//! - [`is_slide_type_keyword`] — `true` if `name` is one of the 35 slide-type
//!   keywords (34 registered `SlideType` impls + `severity_cards`).
//! - [`is_directive_keyword`] — `true` if `name` is a `@`-prefixed directive.
//! - [`is_reserved_bare_keyword`] — `true` if `name` is any non-`@`-prefixed reserved identifier.
//!
//! # STORY-009
//!
//! This module is introduced in STORY-009 to support:
//! - E-PAR-006: reserved keyword used as identifier (`@fn`, `@mixin`, …)
//! - E-PAR-008: variable name collides with slide type keyword (`chart`, …)
//! - E-PAR-009: `raw` or `raw*` variants rejected in user files

use phf::phf_map;

// ─── Reserved keyword table ──────────────────────────────────────────────────

/// Mapping from reserved keyword name → `(error_code, feature_description)`.
///
/// This table covers:
/// - `@`-prefixed future directives not yet implemented (`@fn`, `@mixin`, …)
/// - The `raw` keyword (always rejected in user files → E-PAR-009)
/// - Structural deck-level keywords that cannot be used as variable names
///
/// Slide-type keywords are handled separately by [`is_slide_type_keyword`]
/// because there are 35 of them and they all share the same error code
/// (E-PAR-008 via [`classify_keyword`]).
static RESERVED_KEYWORDS: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    // ── Future directive keywords (E-PAR-006) ─────────────────────────────
    "@fn"       => ("E-PAR-006", "user-defined functions (planned for slideforge v2)"),
    "@mixin"    => ("E-PAR-006", "mixin declarations (planned for slideforge v2)"),
    "@while"    => ("E-PAR-006", "while loops (planned for slideforge v2)"),
    "@match"    => ("E-PAR-006", "pattern matching (planned for slideforge v2)"),
    "@let"      => ("E-PAR-006", "lexical variable bindings (planned for slideforge v2)"),
    "@macro"    => ("E-PAR-006", "macro declarations (planned for slideforge v2)"),
    "@import"   => ("E-PAR-006", "package imports (use @include for file includes)"),
    "@export"   => ("E-PAR-006", "export declarations (planned for slideforge v2)"),
    "@type"     => ("E-PAR-006", "type declarations (planned for slideforge v2)"),
    "@schema"   => ("E-PAR-006", "schema declarations (planned for slideforge v2)"),
    "@yield"    => ("E-PAR-006", "generator-style yield (planned for slideforge v3)"),
    // ── Raw escape hatch variants (E-PAR-009 — forbidden in user .sf files) ─
    "raw"       => ("E-PAR-009", "raw escape hatch (not available in user .sf files)"),
    "raw_pptx"  => ("E-PAR-009", "raw PPTX escape hatch (not available in user .sf files)"),
    "raw_html"  => ("E-PAR-009", "raw HTML escape hatch (not available in user .sf files)"),
    "raw_xml"   => ("E-PAR-009", "raw XML escape hatch (not available in user .sf files)"),
    "raw_docx"  => ("E-PAR-009", "raw DOCX escape hatch (not available in user .sf files)"),
    // ── Structural deck-level keywords (E-PAR-006 when used as identifier) ─
    "slideforge_version" => ("E-PAR-006", "deck version declaration"),
    "lang"      => ("E-PAR-006", "deck language declaration"),
    "brand"     => ("E-PAR-006", "brand template declaration"),
    "vars"      => ("E-PAR-006", "variable block keyword"),
    "set"       => ("E-PAR-006", "default-value rule keyword"),
    "alias"     => ("E-PAR-006", "type alias declaration keyword"),
    "variants"  => ("E-PAR-006", "audience variants block keyword"),
    "slide"     => ("E-PAR-006", "slide block keyword"),
    // NOTE: "section" was removed from this table in STORY-078.
    // "section" is now an active block-level introducer (same tier as "slide"),
    // not a reserved-to-error keyword. classify_keyword("section") returns None.
    "shape"     => ("E-PAR-006", "shape block keyword"),
    // ── Reserved bare identifiers (E-PAR-006 — reserved for future use) ──
    "component" => ("E-PAR-006", "reusable components (planned for slideforge v2)"),
    "extends"   => ("E-PAR-006", "type extension (planned for slideforge v2)"),
    "import_as" => ("E-PAR-006", "aliased imports (planned for slideforge v2)"),
    "macro"     => ("E-PAR-006", "macro system (planned for slideforge v2)"),
    "template"  => ("E-PAR-006", "template system (planned for slideforge v2)"),
    "override"  => ("E-PAR-006", "override mechanism (planned for slideforge v2)"),
    "abstract"  => ("E-PAR-006", "reserved; unused"),
    "interface" => ("E-PAR-006", "reserved; unused"),
    "module"    => ("E-PAR-006", "module system (planned for slideforge v2)"),
    "namespace" => ("E-PAR-006", "reserved; unused"),
    "type"      => ("E-PAR-006", "type definition (non-alias use) (planned for slideforge v2)"),
    "enum"      => ("E-PAR-006", "reserved; unused"),
    "struct"    => ("E-PAR-006", "reserved; unused"),
    "impl"      => ("E-PAR-006", "reserved; unused"),
    "trait"     => ("E-PAR-006", "reserved; unused"),
    "use"       => ("E-PAR-006", "reserved; unused"),
    "match"     => ("E-PAR-006", "pattern matching (planned for slideforge v2)"),
    "let"       => ("E-PAR-006", "reserved; unused"),
    "mut"       => ("E-PAR-006", "reserved; unused"),
    "ref"       => ("E-PAR-006", "reserved; unused"),
    "const"     => ("E-PAR-006", "constants (planned for slideforge v2)"),
};

// ─── Slide-type keyword table ─────────────────────────────────────────────────

/// The slide-type keywords reserved in the DSL parser (35 total).
///
/// Each of these is a valid `slide <type>:` introducer. Using any of them as a
/// `vars:` entry name produces E-PAR-008 ([`SyntaxError::VarNameCollision`]).
///
/// ## Relationship to `SlideTypeRegistry`
///
/// This compile-time set is a **deliberate superset** of `SlideTypeRegistry::default()`
/// (34 registered `SlideType` implementations). The one extra entry is `severity_cards`:
/// it is a color-coded type covered by region frames and `LabelCheck`/`COLOR_CODED`
/// validation, but it has no standalone `SlideType` registration and is therefore absent
/// from the runtime registry. Reserving its keyword here prevents user variables from
/// colliding with it if a `SlideType` impl is added in a future story.
///
/// Summary of counts (post-STORY-087):
/// - `SlideTypeRegistry::default()` — 34 registered types (31 original + `status`,
///   `progress_bar`, `weighted_composite`)
/// - `SLIDE_TYPE_KEYWORDS` — 35 keywords (the 34 above + `severity_cards`)
///
/// Keywords use underscore separators (e.g., `section_break`, not `section-break`).
static SLIDE_TYPE_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {
    // Core presentation structure
    "title",
    "section_break",
    "content",
    "two_col",
    "image",
    "blank",
    // Navigation and overview
    "agenda",
    "toc",
    // People and quotes
    "quote",
    "team",
    "bio",
    // Analysis and strategy
    "executive_summary",
    "problem_statement",
    "recommendation",
    "risk_register",
    "timeline",
    "stat_callout",
    "comparison",
    "process_flow",
    "matrix",
    // Financial and metrics
    "financials",
    "kpi_dashboard",
    // Data visualization
    "chart",
    "diagram",
    // Media and technical
    "screenshot",
    "code_sample",
    "video",
    // Research and organizational
    "survey_results",
    "org_chart",
    "roadmap",
    // Closing
    "closing",
    // Color-coded status types (STORY-087 — BC-1.17.001/002/003)
    "status",
    "progress_bar",
    "weighted_composite",
    // severity_cards: was in COLOR_CODED_TYPES but absent from this set (D4 gap fix)
    "severity_cards",
};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Return `(error_code, feature_description)` if `kw` is a reserved keyword,
/// or `None` if it is a safe user-defined identifier.
///
/// Slide-type keywords are also checked here — they return `("E-PAR-008",
/// "slide type keyword")` so the caller can distinguish them.
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::keywords::classify_keyword;
///
/// assert_eq!(classify_keyword("@fn"), Some(("E-PAR-006", "user-defined functions (planned for slideforge v2)")));
/// assert_eq!(classify_keyword("chart"), Some(("E-PAR-008", "slide type keyword")));
/// assert_eq!(classify_keyword("my_var"), None);
/// ```
#[must_use]
pub fn classify_keyword(kw: &str) -> Option<(&'static str, &'static str)> {
    // First check the explicit reserved-keyword table (covers @fn, raw, structural keywords, etc.)
    if let Some(&(code, desc)) = RESERVED_KEYWORDS.get(kw) {
        return Some((code, desc));
    }
    // Then check slide-type keywords (E-PAR-008).
    if SLIDE_TYPE_KEYWORDS.contains(kw) {
        return Some(("E-PAR-008", "slide type keyword"));
    }
    None
}

/// Return `true` if `name` exactly matches one of the 35 reserved slide-type keywords.
///
/// This covers the 34 registered `SlideType` implementations plus `severity_cards`
/// (a color-coded scan target without a standalone `SlideType` registration).
/// Variable names that collide with any of these must produce E-PAR-008.
/// Note: suffix matches are allowed (`chart_data` is NOT a collision).
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::keywords::is_slide_type_keyword;
///
/// assert!(is_slide_type_keyword("chart"));
/// assert!(is_slide_type_keyword("title"));
/// assert!(!is_slide_type_keyword("chart_data"));  // suffix — NOT a collision
/// assert!(!is_slide_type_keyword("my_title"));    // prefix — NOT a collision
/// ```
#[must_use]
pub fn is_slide_type_keyword(name: &str) -> bool {
    SLIDE_TYPE_KEYWORDS.contains(name)
}

/// Return `true` if `name` matches an `@`-prefixed directive keyword.
///
/// These are checked without the leading `@` because the lexer strips it.
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::keywords::is_directive_keyword;
///
/// assert!(is_directive_keyword("fn"));
/// assert!(is_directive_keyword("mixin"));
/// assert!(!is_directive_keyword("for"));   // implemented directive — not reserved
/// assert!(!is_directive_keyword("title")); // slide type — not a directive
/// ```
#[must_use]
pub fn is_directive_keyword(name: &str) -> bool {
    // Directive keywords are stored in RESERVED_KEYWORDS with an `@` prefix.
    // The caller passes the name WITHOUT the `@`, so we reconstruct it.
    let with_at = format!("@{name}");
    RESERVED_KEYWORDS
        .get(with_at.as_str())
        .is_some_and(|&(code, _)| code == "E-PAR-006")
}

/// Return `true` if `name` is any non-`@`-prefixed reserved identifier.
///
/// This covers ALL reserved bare identifiers including:
/// - Structural deck-level keywords (`vars`, `set`, `alias`, `variants`, `section`, etc.)
/// - Reserved future-feature identifiers (`component`, `extends`, `macro`, etc.)
/// - The `raw` escape-hatch variants (`raw`, `raw_pptx`, `raw_html`, etc.)
///
/// Unlike [`is_slide_type_keyword`], slide-type keywords are not included.
/// Unlike [`is_directive_keyword`], `@`-prefixed directives are not included.
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::keywords::is_reserved_bare_keyword;
///
/// assert!(is_reserved_bare_keyword("vars"));
/// assert!(is_reserved_bare_keyword("set"));
/// assert!(is_reserved_bare_keyword("raw"));
/// assert!(is_reserved_bare_keyword("component"));
/// assert!(!is_reserved_bare_keyword("title")); // slide type — not a reserved bare keyword
/// assert!(!is_reserved_bare_keyword("my_var")); // user identifier — safe
/// ```
#[must_use]
pub fn is_reserved_bare_keyword(name: &str) -> bool {
    // Reserved bare keywords are in RESERVED_KEYWORDS without an `@` prefix.
    RESERVED_KEYWORDS
        .get(name)
        .is_some_and(|&(code, _)| code == "E-PAR-006" || code == "E-PAR-009")
        && !name.starts_with('@')
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    use super::*;

    // AC-012 related: @fn produces E-PAR-006
    #[test]
    fn test_bc_1_09_006_classify_keyword_at_fn_is_reserved() {
        let result = classify_keyword("@fn");
        assert!(
            result.is_some(),
            "@fn must be classified as a reserved keyword"
        );
        let (code, _desc) = result.unwrap();
        assert_eq!(code, "E-PAR-006", "@fn must map to E-PAR-006");
    }

    // AC-013: vars: { chart: "data" } → E-PAR-008
    #[test]
    fn test_bc_1_09_008_classify_keyword_chart_is_slide_type() {
        let result = classify_keyword("chart");
        assert!(
            result.is_some(),
            "chart must be classified (slide type collision)"
        );
        let (code, _desc) = result.unwrap();
        assert_eq!(
            code, "E-PAR-008",
            "slide type collision must map to E-PAR-008"
        );
    }

    // AC-014: vars: { chart_data: "x" } → no error (suffix, not exact)
    #[test]
    fn test_bc_1_09_008_classify_keyword_chart_data_is_safe() {
        let result = classify_keyword("chart_data");
        assert!(
            result.is_none(),
            "chart_data must NOT be reserved (suffix match is not a collision)"
        );
    }

    // AC-012: @mixin is reserved
    #[test]
    fn test_bc_1_09_006_classify_keyword_at_mixin_is_reserved() {
        let result = classify_keyword("@mixin");
        assert!(result.is_some(), "@mixin must be classified as reserved");
        let (code, _) = result.unwrap();
        assert_eq!(code, "E-PAR-006");
    }

    // AC-010: `raw` maps to E-PAR-009
    #[test]
    fn test_bc_1_09_009_classify_keyword_raw_maps_to_e_par_009() {
        let result = classify_keyword("raw");
        assert!(result.is_some(), "raw must be classified");
        let (code, _) = result.unwrap();
        assert_eq!(code, "E-PAR-009", "raw must map to E-PAR-009 (RawKeyword)");
    }

    // is_slide_type_keyword: all slide type keywords recognized
    //
    // STORY-087 added status, progress_bar, weighted_composite (new color-coded types)
    // and severity_cards (D4 gap fix — was in COLOR_CODED_TYPES but absent from this set).
    // Total is now 35 keywords (31 original + 3 new + severity_cards).
    #[test]
    fn test_bc_1_09_008_is_slide_type_keyword_all_35_types() {
        // All 35 keywords in SLIDE_TYPE_KEYWORDS: the 34 registered in
        // SlideTypeRegistry::default() plus severity_cards (COLOR_CODED_TYPES
        // scan target without a SlideType registration — STORY-087 D4 gap fix).
        let all_types = [
            // Core presentation structure
            "title",
            "section_break",
            "content",
            "two_col",
            "image",
            "blank",
            // Navigation and overview
            "agenda",
            "toc",
            // People and quotes
            "quote",
            "team",
            "bio",
            // Analysis and strategy
            "executive_summary",
            "problem_statement",
            "recommendation",
            "risk_register",
            "timeline",
            "stat_callout",
            "comparison",
            "process_flow",
            "matrix",
            // Financial and metrics
            "financials",
            "kpi_dashboard",
            // Data visualization
            "chart",
            "diagram",
            // Media and technical
            "screenshot",
            "code_sample",
            "video",
            // Research and organizational
            "survey_results",
            "org_chart",
            "roadmap",
            // Closing
            "closing",
            // Color-coded status types (STORY-087 — BC-1.17.001/002/003)
            "status",
            "progress_bar",
            "weighted_composite",
            // severity_cards: D4 gap fix — keyword now matches COLOR_CODED_TYPES
            "severity_cards",
        ];
        assert_eq!(
            all_types.len(),
            35,
            "test vector must have exactly 35 keywords (31 original + 3 new + severity_cards)"
        );
        for t in all_types {
            assert!(
                is_slide_type_keyword(t),
                "'{t}' must be recognized as a slide type keyword"
            );
        }
    }

    // is_slide_type_keyword: suffix is NOT a collision
    #[test]
    fn test_bc_1_09_008_is_slide_type_keyword_suffix_not_collision() {
        assert!(
            !is_slide_type_keyword("chart_data"),
            "chart_data is NOT a slide type keyword (AC-014)"
        );
        assert!(
            !is_slide_type_keyword("my_title"),
            "my_title is NOT a slide type keyword"
        );
    }

    // is_directive_keyword: known future directives recognized
    #[test]
    fn test_bc_1_09_006_is_directive_keyword_fn_mixin_recognized() {
        assert!(
            is_directive_keyword("fn"),
            "fn (from @fn) must be recognized as a reserved directive"
        );
        assert!(
            is_directive_keyword("mixin"),
            "mixin (from @mixin) must be recognized"
        );
    }

    // is_reserved_bare_keyword: all non-@-prefixed reserved identifiers recognized
    #[test]
    fn test_bc_1_09_006_is_reserved_bare_keyword_vars_set_recognized() {
        assert!(
            is_reserved_bare_keyword("vars"),
            "vars must be a reserved bare keyword"
        );
        assert!(
            is_reserved_bare_keyword("set"),
            "set must be a reserved bare keyword"
        );
        // raw variants are also reserved bare keywords (E-PAR-009)
        assert!(
            is_reserved_bare_keyword("raw"),
            "raw must be a reserved bare keyword"
        );
        // future-feature reserved identifiers
        assert!(
            is_reserved_bare_keyword("component"),
            "component must be a reserved bare keyword"
        );
    }

    // safe user identifier: none of the checks fire
    #[test]
    fn test_bc_1_09_008_safe_identifier_not_classified() {
        assert!(
            classify_keyword("my_variable").is_none(),
            "my_variable must not be classified as reserved"
        );
        assert!(
            !is_slide_type_keyword("my_variable"),
            "my_variable is not a slide type"
        );
        assert!(
            !is_directive_keyword("my_variable"),
            "my_variable is not a directive"
        );
        assert!(
            !is_reserved_bare_keyword("my_variable"),
            "my_variable is not a reserved bare keyword"
        );
    }
}
