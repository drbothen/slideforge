//! Reserved keyword registry for the slideforge DSL.
//!
//! This module provides compile-time lookup tables for three categories of
//! reserved names:
//!
//! 1. **Reserved structural keywords** — names used as DSL keywords at the
//!    deck / slide level (`vars`, `set`, `alias`, `variants`, `section`).
//! 2. **Reserved directive keywords** — `@`-prefixed names (`@for`, `@if`,
//!    `@elif`, `@else`, `@include`, `@data`, `@fn`, `@mixin`, `@while`).
//! 3. **Slide-type keywords** — the 31 built-in slide types that must not be
//!    used as variable names (e.g. `title`, `chart`, `content`).
//!
//! # Usage
//!
//! The parser calls these functions when processing `vars:` entries and
//! bare identifiers at positions where a keyword would be ambiguous:
//!
//! - [`classify_keyword`] — returns `(error_code, description)` if `kw` is
//!   reserved in any category, or `None` if it is a safe user-defined name.
//! - [`is_slide_type_keyword`] — `true` if `name` is one of the 31 built-in
//!   slide types.
//! - [`is_directive_keyword`] — `true` if `name` is a `@`-prefixed directive.
//! - [`is_structural_keyword`] — `true` if `name` is a top-level DSL keyword.
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
/// because there are 31 of them and they all share the same error code
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
    "section"   => ("E-PAR-006", "section block keyword"),
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

/// The 31 built-in slide type keywords.
///
/// Each of these is a valid `slide <type>:` introducer. Using any of them as a
/// `vars:` entry name produces E-PAR-008 ([`SyntaxError::VarNameCollision`]).
///
/// Source: q2-decision-final.md (BINDING — 23 original + 8 additional types).
static SLIDE_TYPE_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {
    // 23 original seed types
    "title",
    "content",
    "bullets",
    "numbered",
    "two_column",
    "image",
    "big_number",
    "comparison",
    "timeline",
    "process",
    "statement",
    "blank",
    "cover",
    "divider",
    "end",
    "video",
    "table",
    "code",
    "form",
    "map",
    "icons",
    "profile",
    "closing",
    // 8 additional types (q2-decision-final.md)
    "chart",
    "toc",
    "agenda",
    "quote",
    "grid",
    "bio",
    "diagram",
    "team",
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

/// Return `true` if `name` exactly matches one of the 31 built-in slide types.
///
/// Variable names that collide with slide types must produce E-PAR-008.
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

/// Return `true` if `name` is a structural deck-level keyword.
///
/// These keywords (`vars`, `set`, `alias`, `variants`, `section`, etc.) are
/// used at the deck body level and must not be used as user identifiers.
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::keywords::is_structural_keyword;
///
/// assert!(is_structural_keyword("vars"));
/// assert!(is_structural_keyword("set"));
/// assert!(!is_structural_keyword("title")); // slide type — not structural
/// assert!(!is_structural_keyword("my_var")); // user identifier — safe
/// ```
#[must_use]
pub fn is_structural_keyword(name: &str) -> bool {
    // Structural keywords are in RESERVED_KEYWORDS with E-PAR-006 and NO `@` prefix.
    RESERVED_KEYWORDS
        .get(name)
        .is_some_and(|&(code, _)| code == "E-PAR-006")
        && !name.starts_with('@')
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
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

    // is_slide_type_keyword: all 31 types recognized
    #[test]
    fn test_bc_1_09_008_is_slide_type_keyword_all_31_types() {
        let all_types = [
            "title",
            "content",
            "bullets",
            "numbered",
            "two_column",
            "image",
            "big_number",
            "comparison",
            "timeline",
            "process",
            "statement",
            "blank",
            "cover",
            "divider",
            "end",
            "video",
            "table",
            "code",
            "form",
            "map",
            "icons",
            "profile",
            "closing",
            "chart",
            "toc",
            "agenda",
            "quote",
            "grid",
            "bio",
            "diagram",
            "team",
        ];
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

    // is_structural_keyword: deck-level keywords recognized
    #[test]
    fn test_bc_1_09_006_is_structural_keyword_vars_set_recognized() {
        assert!(
            is_structural_keyword("vars"),
            "vars must be a structural keyword"
        );
        assert!(
            is_structural_keyword("set"),
            "set must be a structural keyword"
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
            !is_structural_keyword("my_variable"),
            "my_variable is not structural"
        );
    }
}
