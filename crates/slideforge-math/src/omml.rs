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
use crate::symbols::{
    greek_to_unicode_char, operator_to_unicode_char, symbol_to_unicode_char, unescape_delimiter,
};

/// The XML namespace URI for OMML.
pub const OMML_NAMESPACE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

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
            out.push_str(
                r#"<m:oMathPara xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><m:oMath>"#,
            );
            render_nodes(&ast.nodes, &mut out)?;
            out.push_str("</m:oMath></m:oMathPara>");
        },
        MathMode::Inline => {
            out.push_str(
                r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">"#,
            );
            render_nodes(&ast.nodes, &mut out)?;
            out.push_str("</m:oMath>");
        },
    }

    Ok(out.into_bytes())
}

/// Render a slice of [`MathNode`]s into `out`.
fn render_nodes(nodes: &[MathNode], out: &mut String) -> Result<(), MathRendererError> {
    for node in nodes {
        render_node(node, out)?;
    }
    Ok(())
}

/// Render a single [`MathNode`] into `out`.
///
/// Returns `Err(MathRendererError)` for empty command names or unrecognised
/// symbols; the public [`render`] function propagates these as
/// [`slideforge_plugin_api::MathError`].
fn render_node(node: &MathNode, out: &mut String) -> Result<(), MathRendererError> {
    match node {
        // ── Plain text / identifiers / digits (italic/math style) ─────────
        MathNode::Text(s) => {
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(s));
            out.push_str("</m:t></m:r>");
        },

        // ── Upright text run from \text{...} (plain/roman style) ──────────
        MathNode::TextRun(s) => {
            out.push_str(r#"<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t>"#);
            out.push_str(&xml_escape(s));
            out.push_str("</m:t></m:r>");
        },

        // ── Superscript: x^{2} ────────────────────────────────────────────
        MathNode::Superscript { base, sup } => {
            out.push_str("<m:sSup><m:e>");
            render_node(base, out)?;
            out.push_str("</m:e><m:sup>");
            render_node(sup, out)?;
            out.push_str("</m:sup></m:sSup>");
        },

        // ── Subscript: x_{i} ──────────────────────────────────────────────
        MathNode::Subscript { base, sub } => {
            out.push_str("<m:sSub><m:e>");
            render_node(base, out)?;
            out.push_str("</m:e><m:sub>");
            render_node(sub, out)?;
            out.push_str("</m:sub></m:sSub>");
        },

        // ── Fraction: \frac{a}{b} ─────────────────────────────────────────
        MathNode::Fraction { num, denom } => {
            out.push_str("<m:f><m:num>");
            render_node(num, out)?;
            out.push_str("</m:num><m:den>");
            render_node(denom, out)?;
            out.push_str("</m:den></m:f>");
        },

        // ── Square root / nth root ────────────────────────────────────────
        MathNode::Sqrt { index, radicand } => {
            out.push_str("<m:rad>");
            match index {
                None => {
                    out.push_str(r#"<m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/>"#);
                },
                Some(idx) => {
                    out.push_str("<m:radPr/><m:deg>");
                    render_node(idx, out)?;
                    out.push_str("</m:deg>");
                },
            }
            out.push_str("<m:e>");
            render_node(radicand, out)?;
            out.push_str("</m:e></m:rad>");
        },

        // ── Greek letters → Unicode run via canonical symbols table ──────
        //
        // All Greek glyphs are resolved through `symbols::greek_to_unicode_char`
        // to guarantee cross-renderer Unicode equivalence (BC-1.10.003 inv. 6).
        // An empty name or an unrecognised name returns an error rather than a
        // silent "?" fallback.
        MathNode::Greek(name) => {
            if name.is_empty() {
                return Err(MathRendererError::EmptyCommandName);
            }
            let ch = greek_to_unicode_char(name).ok_or_else(|| {
                MathRendererError::UnsupportedSymbol {
                    name: name.to_string().into(),
                }
            })?;
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(&ch.to_string()));
            out.push_str("</m:t></m:r>");
        },

        // ── Large operators → Unicode run via canonical symbols table ─────
        //
        // Text-based operators (lim, max, min, etc.) must render in upright
        // (plain) style using <m:rPr><m:sty m:val="p"/></m:rPr> so they appear
        // in roman rather than italic — matching standard mathematical typography.
        // Symbol operators (∑, ∏, ∫) do not carry this property.
        // Unknown operators that are not in the text-operator list return an error.
        MathNode::Operator(name) => {
            if name.is_empty() {
                return Err(MathRendererError::EmptyCommandName);
            }
            if is_text_operator(name) {
                out.push_str(r#"<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t>"#);
                out.push_str(&xml_escape(name));
                out.push_str("</m:t></m:r>");
            } else if let Some(ch) = operator_to_unicode_char(name) {
                out.push_str("<m:r><m:t>");
                out.push_str(&xml_escape(&ch.to_string()));
                out.push_str("</m:t></m:r>");
            } else {
                return Err(MathRendererError::UnsupportedSymbol {
                    name: name.to_string().into(),
                });
            }
        },

        // ── Math symbols → Unicode run via canonical symbols table ────────
        //
        // All symbols are resolved through `symbols::symbol_to_unicode_char`.
        // An empty name or an unrecognised name returns an error.
        MathNode::Symbol(name) => {
            if name.is_empty() {
                return Err(MathRendererError::EmptyCommandName);
            }
            let ch = symbol_to_unicode_char(name).ok_or_else(|| {
                MathRendererError::UnsupportedSymbol {
                    name: name.to_string().into(),
                }
            })?;
            out.push_str("<m:r><m:t>");
            out.push_str(&xml_escape(&ch.to_string()));
            out.push_str("</m:t></m:r>");
        },

        // ── Accent: \hat, \bar, \vec, etc. ───────────────────────────────
        MathNode::Accent { kind, inner } => {
            let chr = accent_char(kind);
            out.push_str("<m:acc><m:accPr><m:chr m:val=\"");
            out.push_str(chr);
            out.push_str("\"/></m:accPr><m:e>");
            render_node(inner, out)?;
            out.push_str("</m:e></m:acc>");
        },

        // ── Braced group: {a b c} → flatten ──────────────────────────────
        MathNode::Group(nodes) => {
            render_nodes(nodes, out)?;
        },

        // ── Delimiter: \left( ... \right) ────────────────────────────────
        MathNode::Delimiter { left, right, inner } => {
            out.push_str("<m:d><m:dPr>");
            out.push_str("<m:begChr m:val=\"");
            out.push_str(&xml_escape(unescape_delimiter(left)));
            out.push_str("\"/>");
            out.push_str("<m:endChr m:val=\"");
            out.push_str(&xml_escape(unescape_delimiter(right)));
            out.push_str("\"/>");
            out.push_str("</m:dPr><m:e>");
            render_nodes(inner, out)?;
            out.push_str("</m:e></m:d>");
        },

        // ── Align environment ─────────────────────────────────────────────
        MathNode::Align(rows) => {
            out.push_str("<m:eqArr>");
            for row in rows {
                out.push_str("<m:e>");
                render_nodes(row, out)?;
                out.push_str("</m:e>");
            }
            out.push_str("</m:eqArr>");
        },

        // ── Cases environment ─────────────────────────────────────────────
        MathNode::Cases(cases) => {
            out.push_str(
                r#"<m:d><m:dPr><m:begChr m:val="{"/><m:endChr m:val=""/></m:dPr><m:e><m:eqArr>"#,
            );
            for (lhs, rhs) in cases {
                out.push_str("<m:e>");
                render_nodes(lhs, out)?;
                if !rhs.is_empty() {
                    out.push_str("<m:r><m:t> </m:t></m:r>");
                    render_nodes(rhs, out)?;
                }
                out.push_str("</m:e>");
            }
            out.push_str("</m:eqArr></m:e></m:d>");
        },

        // ── Space ─────────────────────────────────────────────────────────
        MathNode::Space => {
            out.push_str(r#"<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t> </m:t></m:r>"#);
        },
    }
    Ok(())
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

/// Return `true` if `name` is a text-based operator that must render upright.
///
/// Text operators (lim, max, min, sin, cos, tan, log, ln, exp, det, sup, inf,
/// gcd, dim, ker, deg, hom, mod) are typeset in roman (non-italic) style in
/// standard mathematical notation. OMML achieves this with
/// `<m:rPr><m:sty m:val="p"/></m:rPr>`.
///
/// NOTE: The parser currently only produces `Operator` nodes for `lim`, `max`,
/// and `min`. The remaining entries are forward-looking for when more text
/// operators are added to the parser's supported command set.
fn is_text_operator(name: &str) -> bool {
    matches!(
        name,
        "lim"
            | "max"
            | "min"
            | "sin"
            | "cos"
            | "tan"
            | "log"
            | "ln"
            | "exp"
            | "det"
            | "sup"
            | "inf"
            | "gcd"
            | "dim"
            | "ker"
            | "deg"
            | "hom"
            | "mod"
    )
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ast::{AccentKind, MathAst, MathMode, MathNode};
    use std::sync::Arc;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 — OMML output for MathAst nodes
    // ─────────────────────────────────────────────────────────────────────────

    /// A superscript node renders to OMML with `<m:sSup>`.
    #[test]
    fn test_bc_1_10_003_omml_superscript() {
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
    fn test_bc_1_10_003_omml_fraction() {
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
    fn test_bc_1_10_003_omml_display_wraps_in_para() {
        let ast = MathAst::new(MathMode::Display, vec![MathNode::Text(Arc::from("x"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Root element is <m:oMathPara> with namespace attribute, so check
        // for the tag prefix rather than the bare tag without attributes.
        assert!(
            xml.contains("<m:oMathPara"),
            "display mode must use <m:oMathPara>, got: {xml}"
        );
    }

    /// Inline-mode output uses `<m:oMath>` but NOT `<m:oMathPara>`.
    #[test]
    fn test_bc_1_10_003_omml_inline_no_para() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Text(Arc::from("y"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Root element is <m:oMath> with namespace attribute.
        assert!(
            xml.contains("<m:oMath"),
            "inline mode must use <m:oMath>, got: {xml}"
        );
        assert!(
            !xml.contains("<m:oMathPara"),
            "inline mode must NOT use <m:oMathPara>, got: {xml}"
        );
    }

    /// OMML output carries the correct namespace URI.
    #[test]
    fn test_bc_1_10_003_omml_namespace() {
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
    fn test_bc_1_10_003_omml_sqrt() {
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
    fn test_bc_1_10_003_omml_greek_letter() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Greek(Arc::from("alpha"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains("<m:r>"),
            "expected <m:r> run element in: {xml}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-006 — lim/max/min render upright, not italic
    // ─────────────────────────────────────────────────────────────────────────

    /// `lim` renders with `<m:rPr><m:sty m:val="p"/></m:rPr>` for upright style.
    #[test]
    fn test_finding_006_lim_renders_upright() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Operator(Arc::from("lim"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains(r#"<m:sty m:val="p"/>"#),
            "lim must render with upright style m:sty p; got: {xml}"
        );
        assert!(
            xml.contains("lim"),
            "must still contain the text 'lim'; got: {xml}"
        );
    }

    /// Symbol operators (∑, ∏, ∫) do NOT carry the upright style property.
    #[test]
    fn test_finding_006_sum_does_not_get_upright_style() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Operator(Arc::from("sum"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // ∑ is a symbol, not a text operator — no rPr style needed
        assert!(
            !xml.contains(r#"<m:sty m:val="p"/>"#),
            "∑ must not carry upright style; got: {xml}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-005 — delimiter escapes must be stripped before OMML output
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left\{ ... \right\}` must render with bare `{` and `}` in OMML
    /// `m:val` attributes, not with the LaTeX escape sequences `\{` / `\}`.
    #[test]
    fn test_finding_005_delimiter_escapes_stripped() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Delimiter {
                left: Arc::from("\\{"),
                right: Arc::from("\\}"),
                inner: vec![MathNode::Text(Arc::from("x"))],
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Must contain the bare character, not the LaTeX escape
        assert!(
            xml.contains(r#"m:val="{""#),
            "expected bare '{{' in m:val attribute; got: {xml}"
        );
        assert!(
            xml.contains(r#"m:val="}""#),
            "expected bare '}}' in m:val attribute; got: {xml}"
        );
        // Must NOT contain the LaTeX escape sequence in the attribute
        assert!(
            !xml.contains(r#"m:val="\{""#),
            "must not contain LaTeX escape \\{{ in m:val; got: {xml}"
        );
    }

    /// `\left. ... \right.` (null delimiters) must produce empty `m:val` attributes.
    #[test]
    fn test_finding_005_null_delimiter_dot_becomes_empty() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Delimiter {
                left: Arc::from("\\."),
                right: Arc::from("\\."),
                inner: vec![MathNode::Text(Arc::from("x"))],
            }],
        );
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Empty string delimiter → m:val=""
        assert!(
            xml.contains(r#"m:val="""#),
            "expected empty m:val for null delimiter; got: {xml}"
        );
    }

    /// Round-trip: `\left. x \right|` parsed then rendered — null delimiter
    /// must produce `m:val=""` (not `m:val="."`).
    #[test]
    fn test_finding_100_null_delimiter_roundtrip() {
        use crate::parser;
        use slideforge_types::SourceSpan;

        let span = SourceSpan::new(Arc::from("test.sf"), 1, 1, 0);
        let (ast, diags) = parser::parse(r"\left. x \right|", MathMode::Inline, span);
        assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
        let ast = ast.expect("parser must return Some for valid input");
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains(r#"m:val="""#),
            "null delimiter \\left. must produce empty m:val; got: {xml}"
        );
        assert!(
            xml.contains(r#"m:val="|""#),
            "\\right| must produce m:val=\"|\"; got: {xml}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-004 — \div and \mp must render to correct Unicode in OMML
    // ─────────────────────────────────────────────────────────────────────────

    /// `\div` renders to the Unicode division sign ÷ in OMML.
    #[test]
    fn test_finding_004_div_renders_unicode() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Symbol(Arc::from("div"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains('÷'),
            "expected ÷ (U+00F7) in OMML for \\div; got: {xml}"
        );
    }

    /// `\mp` renders to the Unicode minus-or-plus sign ∓ in OMML.
    #[test]
    fn test_finding_004_mp_renders_unicode() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Symbol(Arc::from("mp"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains('∓'),
            "expected ∓ (U+2213) in OMML for \\mp; got: {xml}"
        );
    }

    /// An accent node renders to OMML with `<m:acc>`.
    #[test]
    fn test_bc_1_10_003_omml_accent() {
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
    fn test_bc_1_10_003_snapshot_superscript_x2() {
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
    fn test_bc_1_10_003_snapshot_fraction_ab() {
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
    fn test_bc_1_10_003_snapshot_display_sum() {
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
    fn test_bc_1_10_003_snapshot_sqrt_pythagorean() {
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

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-016 — \sqrt[3]{x} OMML snapshot for nth root
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot: `\sqrt[3]{x}` inline OMML (nth root with index).
    #[test]
    fn test_finding_016_snapshot_sqrt_cube_root() {
        use crate::parser::parse;
        let (ast, diags) = parse(
            r"\sqrt[3]{x}",
            crate::ast::MathMode::Inline,
            slideforge_types::SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Must contain <m:rad> with an actual <m:deg> element (not degHide)
        assert!(xml.contains("<m:rad>"), "expected <m:rad>; got: {xml}");
        assert!(
            !xml.contains(r#"m:val="1""#),
            "nth root must NOT have degHide; got: {xml}"
        );
        insta::assert_yaml_snapshot!("omml_sqrt_cube_root", xml);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-014 — \text{...} must render upright (plain style), not italic
    // ─────────────────────────────────────────────────────────────────────────

    /// `\text{if}` must render with `<m:sty m:val="p"/>` (upright/plain style).
    #[test]
    fn test_finding_014_text_run_renders_upright() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::TextRun(Arc::from("if"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains(r#"<m:sty m:val="p"/>"#),
            "\\text{{}} must render with upright style m:sty p; got: {xml}"
        );
        assert!(
            xml.contains("if"),
            "must still contain the text 'if'; got: {xml}"
        );
    }

    /// Plain `Text` nodes (math identifiers) must NOT carry the upright style.
    #[test]
    fn test_finding_014_plain_text_node_is_italic() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Text(Arc::from("x"))]);
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        // Plain text (math variable) must not have the upright style property
        assert!(
            !xml.contains(r#"<m:sty m:val="p"/>"#),
            "plain Text node must not carry upright style; got: {xml}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-012 — OMML snapshot tests for align and cases environments
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot: `\begin{align} a &= b \\ c &= d \end{align}` display OMML.
    #[test]
    fn test_finding_012_snapshot_align_two_rows() {
        use crate::parser::parse;
        let (ast, diags) = parse(
            r"\begin{align} a &= b \\ c &= d \end{align}",
            crate::ast::MathMode::Display,
            slideforge_types::SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_align_two_rows", xml);
    }

    /// Snapshot: `\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}` inline OMML.
    #[test]
    fn test_finding_012_snapshot_cases_two_entries() {
        use crate::parser::parse;
        let (ast, diags) = parse(
            r"\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}",
            crate::ast::MathMode::Inline,
            slideforge_types::SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let bytes = render(&ast).expect("render should succeed");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        insta::assert_yaml_snapshot!("omml_cases_two_entries", xml);
    }

    /// Snapshot: `\alpha + \beta` inline OMML.
    #[test]
    fn test_bc_1_10_003_snapshot_greek_alpha_beta() {
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

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.003 invariant 6 — Cross-renderer Unicode equivalence
    // ─────────────────────────────────────────────────────────────────────────

    /// `\varphi` produces U+03C6 (φ) in OMML, consistent with `MathML` and PDF.
    ///
    /// Previously OMML emitted U+03D5 (ϕ) — a distinct Unicode codepoint —
    /// while the other two renderers emitted U+03C6 (φ). This test asserts
    /// the fix: all renderers route through `symbols::greek_to_unicode_char`
    /// which maps `"varphi"` → U+03C6.
    #[test]
    fn test_bc_1_10_003_omml_varphi_is_u03c6() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Greek(Arc::from("varphi"))]);
        let bytes = render(&ast).expect("render must succeed for \\varphi");
        let xml = String::from_utf8(bytes).expect("valid UTF-8");
        assert!(
            xml.contains('\u{03C6}'),
            "OMML for \\varphi must contain φ (U+03C6), not ϕ (U+03D5); got: {xml}"
        );
        assert!(
            !xml.contains('\u{03D5}'),
            "OMML for \\varphi must NOT contain ϕ (U+03D5); got: {xml}"
        );
    }

    /// OMML `\varphi` empty-name returns `Err(EmptyCommandName)`.
    #[test]
    fn test_bc_1_10_003_omml_empty_greek_returns_error() {
        let ast = MathAst::new(MathMode::Inline, vec![MathNode::Greek(Arc::from(""))]);
        let result = render(&ast);
        assert!(
            result.is_err(),
            "OMML render with empty Greek name must return Err; got Ok"
        );
    }

    /// OMML `\xyzunknown` operator (not in symbols table) returns error.
    #[test]
    fn test_bc_1_10_003_omml_unknown_operator_returns_error() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Operator(Arc::from("xyzunknown"))],
        );
        let result = render(&ast);
        assert!(
            result.is_err(),
            "OMML render with unknown non-text operator must return Err; got Ok"
        );
    }

    /// OMML `\xyzunknown` symbol (not in symbols table) returns error.
    #[test]
    fn test_bc_1_10_003_omml_unknown_symbol_returns_error() {
        let ast = MathAst::new(
            MathMode::Inline,
            vec![MathNode::Symbol(Arc::from("xyzunknown"))],
        );
        let result = render(&ast);
        assert!(
            result.is_err(),
            "OMML render with unknown symbol must return Err; got Ok"
        );
    }
}
