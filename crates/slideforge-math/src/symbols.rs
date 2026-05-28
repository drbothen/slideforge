//! Canonical command-name → Unicode character mappings shared across all
//! three math renderers (OMML, `MathML`, and the PDF path renderer).
//!
//! Every renderer that needs to translate a LaTeX command name (e.g. `"gamma"`,
//! `"sum"`) to a Unicode scalar value should call these functions.  This
//! ensures that all three renderers produce semantically identical characters
//! for the same command name — a requirement for glyph-distinctness in the PDF
//! path renderer (findings C1–C5, STORY-030).
//!
//! ## Design
//!
//! Functions return `Option<char>` rather than a fallback so that callers can
//! decide the appropriate error behaviour for unknown names:
//! - The PDF path renderer returns `MathError::UnsupportedSymbol`.
//! - The `MathML` / OMML renderers fall back to emitting the command name as
//!   literal text (preserving backwards-compatible behaviour).

/// Map a Greek-letter command name to its canonical Unicode scalar.
///
/// Returns `None` for unrecognised names.
///
/// # Examples
///
/// ```
/// use slideforge_math::symbols::greek_to_unicode_char;
/// assert_eq!(greek_to_unicode_char("gamma"), Some('\u{03B3}'));  // γ
/// assert_eq!(greek_to_unicode_char("Sigma"), Some('\u{03A3}'));  // Σ
/// assert_eq!(greek_to_unicode_char("unknown"), None);
/// ```
#[must_use]
pub fn greek_to_unicode_char(name: &str) -> Option<char> {
    match name {
        "alpha" => Some('\u{03B1}'),                  // α
        "beta" => Some('\u{03B2}'),                   // β
        "gamma" => Some('\u{03B3}'),                  // γ
        "delta" => Some('\u{03B4}'),                  // δ
        "epsilon" | "varepsilon" => Some('\u{03B5}'), // ε
        "zeta" => Some('\u{03B6}'),                   // ζ
        "eta" => Some('\u{03B7}'),                    // η
        "theta" => Some('\u{03B8}'),                  // θ
        "vartheta" => Some('\u{03D1}'),               // ϑ
        "iota" => Some('\u{03B9}'),                   // ι
        "kappa" => Some('\u{03BA}'),                  // κ
        "lambda" => Some('\u{03BB}'),                 // λ
        "mu" => Some('\u{03BC}'),                     // μ
        "nu" => Some('\u{03BD}'),                     // ν
        "xi" => Some('\u{03BE}'),                     // ξ
        "pi" => Some('\u{03C0}'),                     // π
        "varpi" => Some('\u{03D6}'),                  // ϖ
        "rho" => Some('\u{03C1}'),                    // ρ
        "varrho" => Some('\u{03F1}'),                 // ϱ
        "sigma" => Some('\u{03C3}'),                  // σ
        "varsigma" => Some('\u{03C2}'),               // ς
        "tau" => Some('\u{03C4}'),                    // τ
        "upsilon" => Some('\u{03C5}'),                // υ
        "phi" | "varphi" => Some('\u{03C6}'),         // φ
        "chi" => Some('\u{03C7}'),                    // χ
        "psi" => Some('\u{03C8}'),                    // ψ
        "omega" => Some('\u{03C9}'),                  // ω
        // Uppercase
        "Alpha" => Some('\u{0391}'),   // Α
        "Beta" => Some('\u{0392}'),    // Β
        "Gamma" => Some('\u{0393}'),   // Γ
        "Delta" => Some('\u{0394}'),   // Δ
        "Epsilon" => Some('\u{0395}'), // Ε
        "Zeta" => Some('\u{0396}'),    // Ζ
        "Eta" => Some('\u{0397}'),     // Η
        "Theta" => Some('\u{0398}'),   // Θ
        "Iota" => Some('\u{0399}'),    // Ι
        "Kappa" => Some('\u{039A}'),   // Κ
        "Lambda" => Some('\u{039B}'),  // Λ
        "Mu" => Some('\u{039C}'),      // Μ
        "Nu" => Some('\u{039D}'),      // Ν
        "Xi" => Some('\u{039E}'),      // Ξ
        "Pi" => Some('\u{03A0}'),      // Π
        "Rho" => Some('\u{03A1}'),     // Ρ
        "Sigma" => Some('\u{03A3}'),   // Σ
        "Tau" => Some('\u{03A4}'),     // Τ
        "Upsilon" => Some('\u{03A5}'), // Υ
        "Phi" => Some('\u{03A6}'),     // Φ
        "Chi" => Some('\u{03A7}'),     // Χ
        "Psi" => Some('\u{03A8}'),     // Ψ
        "Omega" => Some('\u{03A9}'),   // Ω
        _ => None,
    }
}

/// Map a large-operator command name to its canonical Unicode scalar.
///
/// Returns `None` for unrecognised names.  Text-based operators (`lim`,
/// `max`, etc.) are NOT mapped here — they remain multi-character strings and
/// are handled by the individual renderers as runs of Latin characters.
///
/// # Examples
///
/// ```
/// use slideforge_math::symbols::operator_to_unicode_char;
/// assert_eq!(operator_to_unicode_char("sum"), Some('\u{2211}'));  // ∑
/// assert_eq!(operator_to_unicode_char("int"), Some('\u{222B}'));  // ∫
/// assert_eq!(operator_to_unicode_char("lim"), None);              // text operator
/// ```
#[must_use]
pub fn operator_to_unicode_char(name: &str) -> Option<char> {
    match name {
        "sum" => Some('\u{2211}'),       // ∑
        "prod" => Some('\u{220F}'),      // ∏
        "int" => Some('\u{222B}'),       // ∫
        "oint" => Some('\u{222E}'),      // ∮
        "bigcup" => Some('\u{22C3}'),    // ⋃
        "bigcap" => Some('\u{22C2}'),    // ⋂
        "bigoplus" => Some('\u{2A01}'),  // ⨁
        "bigotimes" => Some('\u{2A02}'), // ⨂
        // Text operators (lim, max, min, sin, …) are NOT mapped here —
        // they render as multi-char Latin strings in all three renderers.
        _ => None,
    }
}

/// Map a miscellaneous symbol command name to its canonical Unicode scalar.
///
/// Returns `None` for unrecognised names.
///
/// # Examples
///
/// ```
/// use slideforge_math::symbols::symbol_to_unicode_char;
/// assert_eq!(symbol_to_unicode_char("times"), Some('\u{00D7}')); // ×
/// assert_eq!(symbol_to_unicode_char("cup"),   Some('\u{222A}')); // ∪
/// assert_eq!(symbol_to_unicode_char("unknown"), None);
/// ```
#[must_use]
pub fn symbol_to_unicode_char(name: &str) -> Option<char> {
    match name {
        "cdot" => Some('\u{22C5}'),              // ⋅
        "times" => Some('\u{00D7}'),             // ×
        "div" => Some('\u{00F7}'),               // ÷
        "infty" => Some('\u{221E}'),             // ∞
        "pm" => Some('\u{00B1}'),                // ±
        "mp" => Some('\u{2213}'),                // ∓
        "leq" | "le" => Some('\u{2264}'),        // ≤
        "geq" | "ge" => Some('\u{2265}'),        // ≥
        "neq" | "ne" => Some('\u{2260}'),        // ≠
        "approx" => Some('\u{2248}'),            // ≈
        "equiv" => Some('\u{2261}'),             // ≡
        "in" => Some('\u{2208}'),                // ∈
        "notin" => Some('\u{2209}'),             // ∉
        "subset" => Some('\u{2282}'),            // ⊂
        "supset" => Some('\u{2283}'),            // ⊃
        "cup" => Some('\u{222A}'),               // ∪
        "cap" => Some('\u{2229}'),               // ∩
        "emptyset" => Some('\u{2205}'),          // ∅
        "forall" => Some('\u{2200}'),            // ∀
        "exists" => Some('\u{2203}'),            // ∃
        "partial" => Some('\u{2202}'),           // ∂
        "nabla" => Some('\u{2207}'),             // ∇
        "angle" => Some('\u{2220}'),             // ∠
        "rightarrow" | "to" => Some('\u{2192}'), // →
        "leftarrow" => Some('\u{2190}'),         // ←
        "Rightarrow" => Some('\u{21D2}'),        // ⇒
        "Leftarrow" => Some('\u{21D0}'),         // ⇐
        "Leftrightarrow" => Some('\u{21D4}'),    // ⇔
        "leftrightarrow" => Some('\u{2194}'),    // ↔
        "uparrow" => Some('\u{2191}'),           // ↑
        "downarrow" => Some('\u{2193}'),         // ↓
        "ldots" => Some('\u{2026}'),             // …
        "cdots" => Some('\u{22EF}'),             // ⋯
        "vdots" => Some('\u{22EE}'),             // ⋮
        "ddots" => Some('\u{22F1}'),             // ⋱
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_gamma_maps_to_unicode_not_latin_g() {
        let ch = greek_to_unicode_char("gamma").expect("gamma must map");
        // Must be γ (U+03B3), NOT Latin 'g'
        assert_eq!(ch, '\u{03B3}', "gamma must map to γ, not Latin g");
        assert_ne!(ch, 'g', "gamma must NOT map to Latin g");
    }

    #[test]
    fn test_operator_sum_maps_to_unicode_not_latin_s() {
        let ch = operator_to_unicode_char("sum").expect("sum must map");
        // Must be ∑ (U+2211), NOT Latin 's'
        assert_eq!(ch, '\u{2211}', "sum must map to ∑, not Latin s");
        assert_ne!(ch, 's', "sum must NOT map to Latin s");
    }

    #[test]
    fn test_symbol_cup_maps_to_unicode_not_latin_u() {
        let ch = symbol_to_unicode_char("cup").expect("cup must map");
        // Must be ∪ (U+222A), NOT Latin 'U'
        assert_eq!(ch, '\u{222A}', "cup must map to ∪, not Latin U");
        assert_ne!(ch, 'U', "cup must NOT map to Latin U");
    }

    #[test]
    fn test_operator_lim_returns_none() {
        // lim is a text operator, not a Unicode-symbol operator
        assert_eq!(operator_to_unicode_char("lim"), None);
    }

    #[test]
    fn test_unknown_greek_returns_none() {
        assert_eq!(greek_to_unicode_char("notAGreekLetter"), None);
    }
}
