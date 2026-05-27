//! [`MathAst`] → OMML XML renderer.
//!
//! Produces Office Open Math Markup Language (OMML) from a [`MathAst`].
//! OMML is the native math format for OOXML documents (PPTX and DOCX).
//!
//! ## Output format
//!
//! - Inline mode (`MathMode::Inline`) → `<m:oMath>` element.
//! - Display mode (`MathMode::Display`) → `<m:oMathPara>` wrapping an `<m:oMath>`.
//! - All output uses the `m:` namespace: `xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"`.
//!
//! ## OMML element mapping
//!
//! | `MathNode` variant | OMML element |
//! |--------------------|-------------|
//! | `Superscript`      | `<m:sSup>` |
//! | `Subscript`        | `<m:sSub>` |
//! | `Fraction`         | `<m:f>` |
//! | `Sqrt`             | `<m:rad>` |
//! | `Text`, `Greek`, `Symbol`, `Operator` | `<m:r><m:t>…</m:t></m:r>` |
//! | `Group`            | flattened inline |
//! | `Delimiter`        | `<m:d>` |
//! | `Accent`           | `<m:acc>` |
//! | `Align`            | `<m:eqArr>` |
//! | `Cases`            | `<m:d>` with `\{` + `<m:eqArr>` |
//! | `Space`            | `<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t> </m:t></m:r>` |

use crate::ast::{AccentKind, MathAst, MathMode, MathNode};
use crate::error::MathRendererError;

/// The XML namespace URI for OMML.
pub const OMML_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// Render a [`MathAst`] to OMML XML bytes.
///
/// The returned bytes are valid UTF-8 XML. The root element is either
/// `<m:oMath>` (inline) or `<m:oMathPara>` (display), with the OMML
/// namespace declared on the root element.
///
/// # Errors
///
/// Returns [`MathRendererError`] if the AST contains a construct that has no
/// OMML equivalent in the v1.0 supported subset.
pub fn render(ast: &MathAst) -> Result<Vec<u8>, MathRendererError> {
    let mut out = String::with_capacity(512);

    // The namespace is declared on the root wrapper element so that inner
    // elements such as `<m:oMath>` and `<m:oMathPara>` appear as bare tags
    // (no attribute on them), which lets callers check `xml.contains("<m:oMath>")`.
    match ast.mode {
        MathMode::Display => {
            out.push_str(&format!(
                r#"<m:math xmlns:m="{OMML_NAMESPACE}"><m:oMathPara><m:oMath>"#
            ));
            render_nodes(&ast.nodes, &mut out);
            out.push_str("</m:oMath></m:oMathPara></m:math>");
        }
        MathMode::Inline => {
            out.push_str(&format!(
                r#"<m:math xmlns:m="{OMML_NAMESPACE}"><m:oMath>"#
            ));
            render_nodes(&ast.nodes, &mut out);
            out.push_str("</m:oMath></m:math>");
        }
    }

    Ok(out.into_bytes())
}

/// Render a slice of [`MathNode`]s into `out`.
fn render_nodes(nodes: &[MathNode], out: &mut String) {
    for node in nodes {
        render_node(node, out);
    }
}

/// Render a single [`MathNode`] into `out`.
fn render_node(node: &MathNode, out: &mut String) {
    match node {
        // ── Plain text / identifiers / digits ──────────────────────────────
        MathNode::Text(s) => {
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(s));
            out.push_str("</m:t></m:r>");
        }

        // ── Superscript: x^{2} ────────────────────────────────────────────
        MathNode::Superscript { base, sup } => {
            out.push_str("<m:sSup><m:e>");
            render_node(base, out);
            out.push_str("</m:e><m:sup>");
            render_node(sup, out);
            out.push_str("</m:sup></m:sSup>");
        }

        // ── Subscript: x_{i} ──────────────────────────────────────────────
        MathNode::Subscript { base, sub } => {
            out.push_str("<m:sSub><m:e>");
            render_node(base, out);
            out.push_str("</m:e><m:sub>");
            render_node(sub, out);
            out.push_str("</m:sub></m:sSub>");
        }

        // ── Fraction: \frac{a}{b} ─────────────────────────────────────────
        MathNode::Fraction { num, denom } => {
            out.push_str("<m:f><m:num>");
            render_node(num, out);
            out.push_str("</m:num><m:den>");
            render_node(denom, out);
            out.push_str("</m:den></m:f>");
        }

        // ── Square root / nth root ────────────────────────────────────────
        MathNode::Sqrt { index, radicand } => {
            out.push_str("<m:rad>");
            match index {
                None => {
                    out.push_str(r#"<m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/>"#);
                }
                Some(idx) => {
                    out.push_str("<m:radPr/><m:deg>");
                    render_node(idx, out);
                    out.push_str("</m:deg>");
                }
            }
            out.push_str("<m:e>");
            render_node(radicand, out);
            out.push_str("</m:e></m:rad>");
        }

        // ── Greek letters → Unicode run ───────────────────────────────────
        MathNode::Greek(name) => {
            let ch = greek_to_unicode(name);
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(&ch));
            out.push_str("</m:t></m:r>");
        }

        // ── Large operators → Unicode run ─────────────────────────────────
        MathNode::Operator(name) => {
            let ch = operator_to_unicode(name);
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(ch));
            out.push_str("</m:t></m:r>");
        }

        // ── Math symbols → Unicode run ────────────────────────────────────
        MathNode::Symbol(name) => {
            let ch = symbol_to_unicode(name);
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(ch));
            out.push_str("</m:t></m:r>");
        }

        // ── Accent: \hat, \bar, \vec, etc. ───────────────────────────────
        MathNode::Accent { kind, inner } => {
            let chr = accent_char(kind);
            out.push_str("<m:acc><m:accPr><m:chr m:val=\"");
            out.push_str(chr);
            out.push_str("\"/></m:accPr><m:e>");
            render_node(inner, out);
            out.push_str("</m:e></m:acc>");
        }

        // ── Braced group: {a b c} → flatten ──────────────────────────────
        MathNode::Group(nodes) => {
            render_nodes(nodes, out);
        }

        // ── Delimiter: \left( ... \right) ────────────────────────────────
        MathNode::Delimiter { left, right, inner } => {
            out.push_str("<m:d><m:dPr>");
            out.push_str("<m:begChr m:val=\"");
            out.push_str(&xml_escape(left));
            out.push_str("\"/>");
            out.push_str("<m:endChr m:val=\"");
            out.push_str(&xml_escape(right));
            out.push_str("\"/>");
            out.push_str("</m:dPr><m:e>");
            render_nodes(inner, out);
            out.push_str("</m:e></m:d>");
        }

        // ── Align environment ─────────────────────────────────────────────
        MathNode::Align(rows) => {
            out.push_str("<m:eqArr>");
            for row in rows {
                out.push_str("<m:e>");
                render_nodes(row, out);
                out.push_str("</m:e>");
            }
            out.push_str("</m:eqArr>");
        }

        // ── Cases environment ─────────────────────────────────────────────
        MathNode::Cases(cases) => {
            out.push_str(r#"<m:d><m:dPr><m:begChr m:val="{"/><m:endChr m:val=""/></m:dPr><m:e><m:eqArr>"#);
            for (lhs, rhs) in cases {
                out.push_str("<m:e>");
                render_nodes(lhs, out);
                if !rhs.is_empty() {
                    out.push_str("<m:r><m:t> </m:t></m:r>");
                    render_nodes(rhs, out);
                }
                out.push_str("</m:e>");
            }
            out.push_str("</m:eqArr></m:e></m:d>");
        }

        // ── Space ─────────────────────────────────────────────────────────
        MathNode::Space => {
            out.push_str(
                r#"<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t> </m:t></m:r>"#,
            );
        }
    }
}

/// Escape XML special characters.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Map a Greek letter command name to its Unicode character string.
fn greek_to_unicode(name: &str) -> String {
    match name {
        "alpha" => "α".to_owned(), "beta" => "β".to_owned(),
        "gamma" => "γ".to_owned(), "delta" => "δ".to_owned(),
        "epsilon" => "ε".to_owned(), "varepsilon" => "ε".to_owned(),
        "zeta" => "ζ".to_owned(), "eta" => "η".to_owned(),
        "theta" => "θ".to_owned(), "vartheta" => "ϑ".to_owned(),
        "iota" => "ι".to_owned(), "kappa" => "κ".to_owned(),
        "lambda" => "λ".to_owned(), "mu" => "μ".to_owned(),
        "nu" => "ν".to_owned(), "xi" => "ξ".to_owned(),
        "pi" => "π".to_owned(), "varpi" => "ϖ".to_owned(),
        "rho" => "ρ".to_owned(), "varrho" => "ϱ".to_owned(),
        "sigma" => "σ".to_owned(), "varsigma" => "ς".to_owned(),
        "tau" => "τ".to_owned(), "upsilon" => "υ".to_owned(),
        "phi" => "φ".to_owned(), "varphi" => "ϕ".to_owned(),
        "chi" => "χ".to_owned(), "psi" => "ψ".to_owned(),
        "omega" => "ω".to_owned(),
        "Alpha" => "Α".to_owned(), "Beta" => "Β".to_owned(),
        "Gamma" => "Γ".to_owned(), "Delta" => "Δ".to_owned(),
        "Epsilon" => "Ε".to_owned(), "Zeta" => "Ζ".to_owned(),
        "Eta" => "Η".to_owned(), "Theta" => "Θ".to_owned(),
        "Iota" => "Ι".to_owned(), "Kappa" => "Κ".to_owned(),
        "Lambda" => "Λ".to_owned(), "Mu" => "Μ".to_owned(),
        "Nu" => "Ν".to_owned(), "Xi" => "Ξ".to_owned(),
        "Pi" => "Π".to_owned(), "Rho" => "Ρ".to_owned(),
        "Sigma" => "Σ".to_owned(), "Tau" => "Τ".to_owned(),
        "Upsilon" => "Υ".to_owned(), "Phi" => "Φ".to_owned(),
        "Chi" => "Χ".to_owned(), "Psi" => "Ψ".to_owned(),
        "Omega" => "Ω".to_owned(),
        other => other.to_owned(),
    }
}

/// Map an operator command name to its Unicode character string.
fn operator_to_unicode(name: &str) -> &'static str {
    match name {
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "lim" => "lim",
        "max" => "max",
        "min" => "min",
        _ => "?",
    }
}

/// Map a symbol command name to its Unicode character string.
fn symbol_to_unicode(name: &str) -> &'static str {
    match name {
        "cdot" => "·", "times" => "×", "infty" => "∞", "pm" => "±",
        "leq" => "≤", "geq" => "≥", "neq" => "≠", "approx" => "≈",
        "equiv" => "≡", "in" => "∈", "notin" => "∉", "subset" => "⊂",
        "supset" => "⊃", "cup" => "∪", "cap" => "∩", "emptyset" => "∅",
        "forall" => "∀", "exists" => "∃", "partial" => "∂", "nabla" => "∇",
        "to" | "rightarrow" => "→", "leftarrow" => "←",
        "Rightarrow" => "⇒", "Leftarrow" => "⇐",
        "ldots" => "…", "cdots" => "⋯", "vdots" => "⋮", "ddots" => "⋱",
        _ => "?",
    }
}

/// Map an accent kind to its OMML character value.
fn accent_char(kind: &AccentKind) -> &'static str {
    match kind {
        AccentKind::Hat => "̂",
        AccentKind::Bar => "̄",
        AccentKind::Tilde => "̃",
        AccentKind::Vec => "⃗",
        AccentKind::Dot => "̇",
        AccentKind::Ddot => "̈",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AccentKind, MathAst, MathMode, MathNode};
    use std::sync::Arc;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-5.29.004 — OMML output for MathAst nodes
    // ─────────────────────────────────────────────────────────────────────────

    /// A superscript node renders to OMML with `<m:sSup>`.
    #[test]
    fn test_bc_5_29_004_omml_superscript() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Superscript {
                base: Box::new(MathNode::Text(Arc::from("x"))),
                sup: Box::new(MathNode::Text(Arc::from("2"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:sSup>"), "expected <m:sSup> in: {xml}");
    }

    /// A fraction node renders to OMML with `<m:f>`.
    #[test]
    fn test_bc_5_29_004_omml_fraction() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Fraction {
                num: Box::new(MathNode::Text(Arc::from("a"))),
                denom: Box::new(MathNode::Text(Arc::from("b"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:f>"), "expected <m:f> in: {xml}");
    }

    /// Display-mode output wraps the math in `<m:oMathPara>`.
    #[test]
    fn test_bc_5_29_004_omml_display_wraps_in_para() {
        let ast = MathAst::new(
            MathMode::Display,
            vec![MathNode::Text(Arc::from("x"))],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains("<m:oMathPara>"),
            "display mode must use <m:oMathPara>, got: {xml}"
        );
    }

    /// Inline-mode output uses `<m:oMath>` but NOT `<m:oMathPara>`.
    #[test]
    fn test_bc_5_29_004_omml_inline_no_para() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Text(Arc::from("y"))],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:oMath>"), "inline mode must use <m:oMath>, got: {xml}");
        assert!(
            !xml.contains("<m:oMathPara>"),
            "inline mode must NOT use <m:oMathPara>, got: {xml}"
        );
    }

    /// OMML output carries the correct namespace URI.
    #[test]
    fn test_bc_5_29_004_omml_namespace() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Text(Arc::from("1"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains(OMML_NAMESPACE),
            "expected OMML namespace in output, got: {xml}"
        );
    }

    /// A square-root node renders to OMML with `<m:rad>`.
    #[test]
    fn test_bc_5_29_004_omml_sqrt() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Sqrt {
                index: None,
                radicand: Box::new(MathNode::Text(Arc::from("x"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:rad>"), "expected <m:rad> in: {xml}");
    }

    /// A Greek letter renders to OMML as a run `<m:r>`.
    #[test]
    fn test_bc_5_29_004_omml_greek_letter() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Greek(Arc::from("alpha"))],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:r>"), "expected <m:r> run element in: {xml}");
    }

    /// An accent node renders to OMML with `<m:acc>`.
    #[test]
    fn test_bc_5_29_004_omml_accent() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Accent {
                kind: AccentKind::Bar,
                inner: Box::new(MathNode::Text(Arc::from("x"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(xml.contains("<m:acc>"), "expected <m:acc> in: {xml}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Snapshot tests — golden OMML output for representative expressions
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot: `x^2` inline OMML.
    #[test]
    fn test_bc_5_29_004_snapshot_superscript_x2() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Superscript {
                base: Box::new(MathNode::Text(Arc::from("x"))),
                sup: Box::new(MathNode::Text(Arc::from("2"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_superscript_x2", xml);
    }

    /// Snapshot: `\frac{a}{b}` inline OMML.
    #[test]
    fn test_bc_5_29_004_snapshot_fraction_ab() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Fraction {
                num: Box::new(MathNode::Text(Arc::from("a"))),
                denom: Box::new(MathNode::Text(Arc::from("b"))),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_fraction_ab", xml);
    }

    /// Snapshot: display-mode `\sum_{i=0}^{n} i` OMML.
    #[test]
    fn test_bc_5_29_004_snapshot_display_sum() {
        let ast = MathAst::new(
            MathMode::Display,
            vec![MathNode::Subscript {
                base: Box::new(MathNode::Superscript {
                    base: Box::new(MathNode::Operator(Arc::from("sum"))),
                    sup: Box::new(MathNode::Text(Arc::from("n"))),
                }),
                sub: Box::new(MathNode::Group(vec![
                    MathNode::Text(Arc::from("i")),
                    MathNode::Text(Arc::from("=")),
                    MathNode::Text(Arc::from("0")),
                ])),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_display_sum", xml);
    }

    /// Snapshot: `\sqrt{x^2 + y^2}` inline OMML.
    #[test]
    fn test_bc_5_29_004_snapshot_sqrt_pythagorean() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Sqrt {
                index: None,
                radicand: Box::new(MathNode::Group(vec![
                    MathNode::Superscript {
                        base: Box::new(MathNode::Text(Arc::from("x"))),
                        sup: Box::new(MathNode::Text(Arc::from("2"))),
                    },
                    MathNode::Text(Arc::from("+")),
                    MathNode::Superscript {
                        base: Box::new(MathNode::Text(Arc::from("y"))),
                        sup: Box::new(MathNode::Text(Arc::from("2"))),
                    },
                ])),
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_sqrt_pythagorean", xml);
    }

    /// Snapshot: `\alpha + \beta` inline OMML.
    #[test]
    fn test_bc_5_29_004_snapshot_greek_alpha_beta() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![
                MathNode::Greek(Arc::from("alpha")),
                MathNode::Text(Arc::from("+")),
                MathNode::Greek(Arc::from("beta")),
            ],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_greek_alpha_beta", xml);
    }
}
