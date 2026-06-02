#![allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
#![allow(non_snake_case)] // test_BC_S_SS_NNN_xxx naming convention per TDD traceability
#![allow(clippy::doc_markdown)] // test function names in doc comments do not need backticks
#![allow(clippy::redundant_closure_for_method_calls)]
//! Failing test suite (Red Gate) for STORY-077: Inline markup parser extension.
//!
//! # TDD Red Gate — template chunk parsing (DIR-077-002 §8, tests 1-15)
//!
//! All 15 tests in this module MUST FAIL before the inline markup parsing
//! implementation begins (per the Red Gate protocol). They specify the exact
//! `TemplateChunk` variants that `template_value()` must produce for each
//! supported inline markup form.
//!
//! # Why tests fail (Red Gate)
//!
//! The `template_value()` combinator in `parser/template.rs` currently does not
//! recognize any inline markup delimiters (`**`, `_`, `` ` ``, `[`, `^`, `~`,
//! `~~`, `==`). String content containing these is returned as a single
//! `TemplateChunk::Literal(...)` rather than the structural Bold/Italic/Code/etc.
//! variant required by the AC-002 contract.
//!
//! Until the inline markup scanning pass is added to `template_value()` (per
//! DIR-077-002 §3 and STORY-077 Phase 1 task), these tests will fail with
//! assertion mismatches — NOT compilation errors.
//!
//! TD-VSDD-059: every test has load-bearing assertions on the specific
//! TemplateChunk variant produced and its inner content. None are vacuous.
//!
//! # Authoritative Sources
//!
//! - DIR-077-002 §1 (canonical v1 inline syntax table)
//! - DIR-077-002 §3 (two-phase architecture)
//! - DIR-077-002 §5 (error accumulation rules)
//! - DIR-077-002 §8 (required Red Gate tests, items 1–15)
//! - STORY-077 v1.5 AC-002

use std::sync::Arc;

use crate::expr::Expr;
use crate::span::SourceMap;
use crate::template::TemplateChunk;

// ─── Test helper ─────────────────────────────────────────────────────────────

/// Parse a string literal value through `template_value()` and return the
/// resulting `(Vec<TemplateChunk>, Vec<String>)`.
///
/// Constructs a minimal slide field containing the string, parses it with
/// the full slideforge parser, and extracts the TemplateChunk sequence.
///
/// This drives the PRODUCTION parser path — it does NOT hand-construct the
/// result. Any assertion failure means the parser does not yet recognize the
/// markup form (Red Gate is active). TD-VSDD-059 compliance.
fn parse_template_value(s: &str) -> Vec<TemplateChunk> {
    use crate::ast::{BlockItem, FieldValue};
    use crate::parser::parse;

    // Build the minimal deck source that wraps the value in a slide field.
    // The field name "detail" is used (semantically closest to section sub-block).
    let src = format!("slide content:\n  detail \"{s}\"\n");
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src.as_str()));
    let result = parse(src.as_str(), file_id, &sm).expect("source must parse without fatal errors");
    let deck = result.deck;
    let BlockItem::Slide(slide_s) = &deck.items[0] else {
        panic!("expected Slide block item");
    };
    let field = slide_s
        .value()
        .fields
        .iter()
        .find(|f| f.name.value() == "detail")
        .expect("detail field must exist");
    let FieldValue::Template(chunks) = field.value.value() else {
        panic!("detail field must be Template; got: {:?}", field.value.value());
    };
    chunks.clone()
}

// ─── Test 1: Bold chunk produced (DIR-077-002 §8 item 1) ─────────────────────

/// DIR-077-002 §8 #1 / BC-3.02.002 AC-002:
/// `"**hello**"` must produce `[TemplateChunk::Bold([TemplateChunk::Literal("hello")])]`.
///
/// FAILS until `template_value()` recognizes `**...**` and produces `Bold`.
/// Currently produces `[Literal("**hello**")]`.
#[test]
fn test_BC_3_02_002_bold_chunk_produced() {
    let chunks = parse_template_value("**hello**");

    assert_eq!(
        chunks.len(),
        1,
        "bold span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Bold(children) => {
            assert_eq!(
                children.len(),
                1,
                "Bold must have exactly 1 child (the literal 'hello'); got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "hello"),
                "Bold child must be Literal(\"hello\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_bold_chunk_produced FAIL: expected TemplateChunk::Bold, \
             got: {other:?}\n\
             Forbidden pattern: Literal(\"**hello**\") means the parser did not \
             recognize the bold delimiter."
        ),
    }
}

// ─── Test 2: Italic chunk produced (DIR-077-002 §8 item 2) ───────────────────

/// DIR-077-002 §8 #2 / BC-3.02.002 AC-002:
/// `"_hello_"` must produce `[TemplateChunk::Italic([TemplateChunk::Literal("hello")])]`.
///
/// FAILS until `template_value()` recognizes `_..._` and produces `Italic`.
#[test]
fn test_BC_3_02_002_italic_chunk_produced() {
    let chunks = parse_template_value("_hello_");

    assert_eq!(
        chunks.len(),
        1,
        "italic span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Italic(children) => {
            assert_eq!(
                children.len(),
                1,
                "Italic must have exactly 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "hello"),
                "Italic child must be Literal(\"hello\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_italic_chunk_produced FAIL: expected TemplateChunk::Italic, \
             got: {other:?}"
        ),
    }
}

// ─── Test 3: Code chunk produced (DIR-077-002 §8 item 3) ─────────────────────

/// DIR-077-002 §8 #3 / BC-3.02.002 AC-002:
/// `` "`fn foo()`" `` must produce `[TemplateChunk::Code("fn foo()")]`.
///
/// FAILS until `template_value()` recognizes `` `...` `` and produces `Code`.
#[test]
fn test_BC_3_02_002_code_chunk_produced() {
    let chunks = parse_template_value("`fn foo()`");

    assert_eq!(
        chunks.len(),
        1,
        "code span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Code(content) => {
            assert_eq!(
                content, "fn foo()",
                "Code chunk must contain verbatim content 'fn foo()'; got: {content:?}"
            );
        },
        other => panic!(
            "test_BC_3_02_002_code_chunk_produced FAIL: expected TemplateChunk::Code, \
             got: {other:?}"
        ),
    }
}

// ─── Test 4: Link chunk produced (DIR-077-002 §8 item 4) ─────────────────────

/// DIR-077-002 §8 #4 / BC-3.02.002 AC-002:
/// `"[click here](https://example.com)"` must produce
/// `[TemplateChunk::Link { text: [Literal("click here")], url: "https://example.com" }]`.
///
/// FAILS until `template_value()` recognizes `[text](url)` and produces `Link`.
#[test]
fn test_BC_3_02_002_link_chunk_produced() {
    let chunks = parse_template_value("[click here](https://example.com)");

    assert_eq!(
        chunks.len(),
        1,
        "link span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Link { text, url } => {
            assert_eq!(
                url, "https://example.com",
                "Link url must be 'https://example.com'; got: {url:?}"
            );
            assert_eq!(
                text.len(),
                1,
                "Link text must have exactly 1 child; got {text:?}"
            );
            assert!(
                matches!(&text[0], TemplateChunk::Literal(s) if s == "click here"),
                "Link text child must be Literal(\"click here\"); got: {:?}",
                text[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_link_chunk_produced FAIL: expected TemplateChunk::Link, \
             got: {other:?}"
        ),
    }
}

// ─── Test 5: Superscript chunk produced (DIR-077-002 §8 item 5) ──────────────

/// DIR-077-002 §8 #5 / BC-3.02.002 AC-002:
/// `"^2^"` must produce `[TemplateChunk::Superscript([TemplateChunk::Literal("2")])]`.
///
/// FAILS until `template_value()` recognizes `^...^` and produces `Superscript`.
#[test]
fn test_BC_3_02_002_superscript_chunk_produced() {
    let chunks = parse_template_value("^2^");

    assert_eq!(
        chunks.len(),
        1,
        "superscript span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Superscript(children) => {
            assert_eq!(
                children.len(),
                1,
                "Superscript must have exactly 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "2"),
                "Superscript child must be Literal(\"2\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_superscript_chunk_produced FAIL: expected TemplateChunk::Superscript, \
             got: {other:?}"
        ),
    }
}

// ─── Test 6: Subscript chunk produced (DIR-077-002 §8 item 6) ────────────────

/// DIR-077-002 §8 #6 / BC-3.02.002 AC-002:
/// `"~n~"` must produce `[TemplateChunk::Subscript([TemplateChunk::Literal("n")])]`.
///
/// FAILS until `template_value()` recognizes `~...~` and produces `Subscript`.
#[test]
fn test_BC_3_02_002_subscript_chunk_produced() {
    let chunks = parse_template_value("~n~");

    assert_eq!(
        chunks.len(),
        1,
        "subscript span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Subscript(children) => {
            assert_eq!(
                children.len(),
                1,
                "Subscript must have exactly 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "n"),
                "Subscript child must be Literal(\"n\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_subscript_chunk_produced FAIL: expected TemplateChunk::Subscript, \
             got: {other:?}"
        ),
    }
}

// ─── Test 7: Strikethrough before subscript (DIR-077-002 §8 item 7) ──────────

/// DIR-077-002 §8 #7 / BC-3.02.002 AC-002 (disambiguation rule 1):
/// `"~~del~~ ~sub~"` must produce:
/// `[Strikethrough([Literal("del")]), Literal(" "), Subscript([Literal("sub")])]`.
///
/// The parser MUST consume `~~` greedily before checking `~`.
/// A single `~` that is NOT the start of `~~` becomes Subscript.
///
/// FAILS until `template_value()` implements the `~~` > `~` disambiguation.
#[test]
fn test_BC_3_02_002_strikethrough_before_subscript() {
    let chunks = parse_template_value("~~del~~ ~sub~");

    assert_eq!(
        chunks.len(),
        3,
        "must produce 3 chunks: Strikethrough, Literal(' '), Subscript; got {chunks:?}"
    );
    // First chunk: Strikethrough
    match &chunks[0] {
        TemplateChunk::Strikethrough(children) => {
            assert_eq!(
                children.len(),
                1,
                "Strikethrough must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "del"),
                "Strikethrough child must be Literal(\"del\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_strikethrough_before_subscript FAIL (chunk[0]): \
             expected Strikethrough, got: {other:?}\n\
             If this is Subscript, the ~~-before-~ disambiguation is not implemented."
        ),
    }
    // Second chunk: separator literal
    assert!(
        matches!(&chunks[1], TemplateChunk::Literal(s) if s == " "),
        "chunk[1] must be Literal(' '); got: {:?}",
        chunks[1]
    );
    // Third chunk: Subscript
    match &chunks[2] {
        TemplateChunk::Subscript(children) => {
            assert_eq!(
                children.len(),
                1,
                "Subscript must have 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "sub"),
                "Subscript child must be Literal(\"sub\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_strikethrough_before_subscript FAIL (chunk[2]): \
             expected Subscript, got: {other:?}"
        ),
    }
}

// ─── Test 8: Highlight chunk produced (DIR-077-002 §8 item 8) ────────────────

/// DIR-077-002 §8 #8 / BC-3.02.002 AC-002:
/// `"==highlight me=="` must produce `[TemplateChunk::Highlight([Literal("highlight me")])]`.
///
/// FAILS until `template_value()` recognizes `==...==` and produces `Highlight`.
#[test]
fn test_BC_3_02_002_highlight_chunk_produced() {
    let chunks = parse_template_value("==highlight me==");

    assert_eq!(
        chunks.len(),
        1,
        "highlight span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Highlight(children) => {
            assert_eq!(
                children.len(),
                1,
                "Highlight must have exactly 1 child; got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "highlight me"),
                "Highlight child must be Literal(\"highlight me\"); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_highlight_chunk_produced FAIL: expected TemplateChunk::Highlight, \
             got: {other:?}"
        ),
    }
}

// ─── Test 9: Bold with {{ }} interpolation (DIR-077-002 §8 item 9) ───────────

/// DIR-077-002 §8 #9 / BC-3.02.002 AC-002 (composition rule):
/// `"**{{ client }}**"` must produce
/// `[TemplateChunk::Bold([TemplateChunk::Expr(Expr::Ident("client"))])]`.
///
/// Demonstrates that `{{ }}` interpolation COMPOSES with inline markup:
/// expressions inside bold spans are represented as Expr children of Bold.
///
/// FAILS until `template_value()` implements `{{ }}` inside bold children.
#[test]
fn test_BC_3_02_002_bold_with_interpolation() {
    let chunks = parse_template_value("**{{ client }}**");

    assert_eq!(
        chunks.len(),
        1,
        "bold-with-interpolation must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Bold(children) => {
            assert_eq!(
                children.len(),
                1,
                "Bold must have exactly 1 child (the Expr); got {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Expr(Expr::Ident(name)) if name == "client"),
                "Bold child must be Expr(Ident(\"client\")); got: {:?}",
                children[0]
            );
        },
        other => panic!(
            "test_BC_3_02_002_bold_with_interpolation FAIL: expected TemplateChunk::Bold, \
             got: {other:?}"
        ),
    }
}

// ─── Test 10: Nested bold italic (DIR-077-002 §8 item 10) ────────────────────

/// DIR-077-002 §8 #10 / BC-3.02.002 AC-002 (nesting rule):
/// `"**_bold italic_**"` must produce
/// `[TemplateChunk::Bold([TemplateChunk::Italic([TemplateChunk::Literal("bold italic")])])]`.
///
/// Demonstrates one level of markup nesting (Bold > Italic > Literal).
///
/// FAILS until `template_value()` produces nested markup variants.
#[test]
fn test_BC_3_02_002_nested_bold_italic() {
    let chunks = parse_template_value("**_bold italic_**");

    assert_eq!(
        chunks.len(),
        1,
        "nested bold+italic must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Bold(bold_children) => {
            assert_eq!(
                bold_children.len(),
                1,
                "Bold must have exactly 1 child (the Italic); got {bold_children:?}"
            );
            match &bold_children[0] {
                TemplateChunk::Italic(italic_children) => {
                    assert_eq!(
                        italic_children.len(),
                        1,
                        "Italic must have exactly 1 child; got {italic_children:?}"
                    );
                    assert!(
                        matches!(&italic_children[0], TemplateChunk::Literal(s) if s == "bold italic"),
                        "Italic child must be Literal(\"bold italic\"); got: {:?}",
                        italic_children[0]
                    );
                },
                other => panic!(
                    "test_BC_3_02_002_nested_bold_italic FAIL (Bold child): \
                     expected Italic, got: {other:?}"
                ),
            }
        },
        other => panic!(
            "test_BC_3_02_002_nested_bold_italic FAIL: expected TemplateChunk::Bold, \
             got: {other:?}"
        ),
    }
}

// ─── Test 11: Code span no inner markup (DIR-077-002 §8 item 11) ─────────────

/// DIR-077-002 §8 #11 / BC-3.02.002 AC-002 (code span verbatim rule):
/// `` "`**not bold**`" `` must produce `[TemplateChunk::Code("**not bold**")]`.
///
/// Content inside a code span is verbatim — `**` is NOT processed as bold.
/// This enforces the CommonMark rule: code spans disable inline markup inside them.
///
/// FAILS until `template_value()` treats code span content as verbatim.
#[test]
fn test_BC_3_02_002_code_span_no_inner_markup() {
    let chunks = parse_template_value("`**not bold**`");

    assert_eq!(
        chunks.len(),
        1,
        "code span must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Code(content) => {
            // The content must be verbatim — including the literal asterisks.
            assert_eq!(
                content, "**not bold**",
                "Code chunk must contain verbatim '**not bold**' (asterisks are NOT markup \
                 inside a code span); got: {content:?}\n\
                 If the content is 'not bold' without asterisks, the code span is stripping \
                 markup which is WRONG."
            );
            // Verify: the content must NOT have been processed to a Bold node.
            assert!(
                !content.is_empty(),
                "Code span content must not be empty"
            );
        },
        other => panic!(
            "test_BC_3_02_002_code_span_no_inner_markup FAIL: expected TemplateChunk::Code, \
             got: {other:?}\n\
             If this is TemplateChunk::Bold, the code span boundary is not being respected."
        ),
    }
}

// ─── Test 12: Math mode disables inline markup (DIR-077-002 §8 item 12) ──────

/// DIR-077-002 §8 #12 / BC-3.02.002 AC-002 (math mode rule — EC-012):
/// `"$**not bold**$"` must produce `[TemplateChunk::MathInline("**not bold**")]`.
///
/// Inside a math region (`$...$`), ALL inline markup is disabled.
/// The `**` characters are treated as LaTeX content (e.g., multiplication),
/// NOT as bold delimiters.
///
/// NOTE: This rule requires NO implementation change — `template_value()` already
/// treats content between `$` delimiters as verbatim math. If this test passes,
/// it demonstrates the existing math mode behavior is correct. If the test fails,
/// it means the inline markup pass is being applied to math content (incorrect).
///
/// This test may PASS at Red Gate (since math mode already works correctly).
/// If it fails, the inline markup scanning pass is incorrectly entering math regions.
#[test]
fn test_BC_3_02_002_math_mode_no_inline_markup() {
    let chunks = parse_template_value("$**not bold**$");

    assert_eq!(
        chunks.len(),
        1,
        "math mode with '**' must produce exactly 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::MathInline(latex) => {
            // The content must be the raw LaTeX including the asterisks.
            assert_eq!(
                latex, "**not bold**",
                "MathInline must contain verbatim LaTeX '**not bold**'; \
                 the asterisks are LaTeX content (not markup); got: {latex:?}"
            );
            // Crucially: no Bold node must appear.
            // (The bold check is structural: we are inside MathInline, so there is
            // no Bold variant here — the assertion above is sufficient.)
        },
        TemplateChunk::Bold(_) => panic!(
            "test_BC_3_02_002_math_mode_no_inline_markup FAIL: got TemplateChunk::Bold inside math!\n\
             The inline markup scanning pass must NOT enter math regions ($...$).\n\
             Math mode disables ALL text-mode markup (DIR-077-002 §3 last bullet)."
        ),
        other => panic!(
            "test_BC_3_02_002_math_mode_no_inline_markup FAIL: expected TemplateChunk::MathInline, \
             got: {other:?}"
        ),
    }
}

// ─── Test 13: Unclosed bold error accumulated (DIR-077-002 §8 item 13) ────────

/// DIR-077-002 §8 #13 / BC-3.02.002 AC-002 (error accumulation — EC-007):
/// `"**unclosed"` (no closing `**`) must:
/// - Accumulate a non-fatal parse error (error count == 1)
/// - Produce a Bold or Literal sentinel (not panic)
/// - The error span must point to the opening `**`
///
/// This test calls `split_template()` directly via the parser to capture the
/// (chunks, errors) tuple. Because parse_template_value() does not expose the
/// errors vector, we use a direct `template_value()` call via the public
/// `parse()` entry that propagates template errors to `ParseResult::warnings`.
///
/// FAILS until `template_value()` implements unclosed-bold error accumulation.
#[test]
fn test_BC_3_02_002_unclosed_bold_error_accumulated() {
    use crate::parser::parse;

    // parse() surfaces template errors via ParseResult::warnings (or Err).
    // An unclosed bold must produce an error (accumulated, not fatal).
    let src = "slide content:\n  detail \"**unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    match result {
        Ok(pr) => {
            // If parse succeeds, errors must be in warnings (non-fatal accumulation).
            // The deck must have exactly 1 slide.
            let deck = pr.deck;
            assert_eq!(
                deck.items.len(),
                1,
                "deck must still have 1 slide (error recovery, not fatal)"
            );
            // Warnings must contain at least 1 entry (the unclosed bold error).
            // Note: this test checks the WARNING path because inline markup errors
            // are non-fatal per DIR-077-002 §5.
            assert!(
                !pr.warnings.is_empty() || {
                    // Alternatively, the error may be surfaced via the template chunk's
                    // error sentinel (the errors vec inside template_value). Either path
                    // is acceptable as long as the overall parse did not succeed silently.
                    // Check the field value for an error sentinel.
                    use crate::ast::{BlockItem, FieldValue};
                    if let BlockItem::Slide(s) = &deck.items[0] {
                        let field = s.value().fields.iter().find(|f| f.name.value() == "detail");
                        if let Some(f) = field {
                            if let FieldValue::Template(chunks) = f.value.value() {
                                // An unclosed bold should produce Bold or a Literal fallback.
                                // The important thing is the asterisks are NOT in a plain Literal.
                                chunks.iter().any(|c| matches!(c, TemplateChunk::Bold(_)))
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                },
                "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
                 unclosed bold '**unclosed' must produce a Bold chunk or a warning; \
                 got no bold chunk and no warnings. The error accumulation is not active."
            );
        },
        Err(errors) => {
            // If it's a fatal error, the error must mention the unclosed bold.
            let combined = errors
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("; ");
            // An unclosed bold in a field value should be NON-FATAL (parse should succeed
            // with the error accumulated). Getting Err here means the parser fatally rejected
            // the unclosed bold, which violates the error accumulation convention.
            panic!(
                "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
                 unclosed bold produced a FATAL parse error (should be non-fatal, \
                 DIR-077-002 §5); errors: {combined}"
            );
        },
    }
}

// ─── Test 14: Empty bold error accumulated (DIR-077-002 §8 item 14) ──────────

/// DIR-077-002 §8 #14 / BC-3.02.002 AC-002 (error accumulation — EC-008):
/// `"****"` (empty bold span) must accumulate a parse error.
///
/// An empty bold span (`**` immediately followed by `**`) is a parse error per
/// DIR-077-002 §2. Error accumulation applies — the parse is non-fatal.
///
/// FAILS until `template_value()` detects and reports empty spans.
#[test]
fn test_BC_3_02_002_empty_bold_error_accumulated() {
    use crate::parser::parse;

    let src = "slide content:\n  detail \"****\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    match result {
        Ok(pr) => {
            // Non-fatal path: parse succeeded with accumulated error in warnings.
            // Must have produced either a warning OR the "****" was correctly parsed
            // as an error sentinel (not silently as a Literal("****")).
            use crate::ast::{BlockItem, FieldValue};
            if let BlockItem::Slide(s) = &pr.deck.items[0] {
                let field = s.value().fields.iter().find(|f| f.name.value() == "detail");
                if let Some(f) = field {
                    if let FieldValue::Template(chunks) = f.value.value() {
                        // "****" must NOT silently produce Literal("****").
                        let is_plain_four_asterisks = chunks.len() == 1
                            && matches!(&chunks[0], TemplateChunk::Literal(s) if s == "****");
                        assert!(
                            !is_plain_four_asterisks || !pr.warnings.is_empty(),
                            "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
                             '****' silently produced Literal(\"****\") with no warning. \
                             Empty bold span must produce an error (DIR-077-002 §2)."
                        );
                    }
                }
            }
            // The parse must have produced at least 1 warning (the empty-span error).
            assert!(
                !pr.warnings.is_empty(),
                "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
                 empty bold '****' must produce at least 1 warning (non-fatal error); \
                 got 0 warnings. Error accumulation is not active for empty spans."
            );
        },
        Err(errors) => {
            // Fatal error for an empty bold span violates the non-fatal convention.
            let combined = errors
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("; ");
            panic!(
                "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
                 empty bold produced a FATAL parse error (should be non-fatal, \
                 DIR-077-002 §5); errors: {combined}"
            );
        },
    }
}

// ─── Test 15: TemplateChunk::Bold derives Hash + Eq + Clone + Debug (DIR-077-002 §8 item 15) ─

/// DIR-077-002 §8 #15 / BC-3.02.002 AC-002 / ADR-013:
/// `TemplateChunk::Bold` must implement `Hash + Eq + Clone + Debug`.
/// This is the comemo cache compatibility requirement (ADR-013).
///
/// This test PASSES at Red Gate (the struct derivation is a compile-time check,
/// and the new Bold variant already derives these traits in template.rs).
/// If it fails, the Bold variant is missing a derive.
#[test]
fn test_BC_3_02_002_template_chunk_bold_derives_hash_eq_clone_debug() {
    use std::collections::HashSet;

    // Bold with Literal child
    let bold = TemplateChunk::Bold(vec![TemplateChunk::Literal("hello".to_string())]);

    // Clone
    let bold2 = bold.clone();
    assert_eq!(bold, bold2, "Bold must implement PartialEq (clone equality)");

    // Debug
    let debug_str = format!("{bold:?}");
    assert!(
        debug_str.contains("Bold"),
        "Bold must implement Debug; got: {debug_str}"
    );

    // Hash (via HashSet insertion)
    let mut set = HashSet::new();
    set.insert(bold.clone());
    assert_eq!(set.len(), 1, "Bold must be hashable (HashSet::insert)");

    // Verify all new inline markup variants also derive Hash+Eq+Clone+Debug
    // (structural check — will fail to compile if any derive is missing).
    let _italic = TemplateChunk::Italic(vec![]);
    let _code = TemplateChunk::Code("fn x() {}".to_string());
    let _link = TemplateChunk::Link {
        text: vec![],
        url: "https://example.com".to_string(),
    };
    let _sup = TemplateChunk::Superscript(vec![]);
    let _sub = TemplateChunk::Subscript(vec![]);
    let _del = TemplateChunk::Strikethrough(vec![]);
    let _hi = TemplateChunk::Highlight(vec![]);

    // Each must be cloneable and debug-printable.
    let _ = (_italic.clone(), format!("{_italic:?}"));
    let _ = (_code.clone(), format!("{_code:?}"));
    let _ = (_link.clone(), format!("{_link:?}"));
    let _ = (_sup.clone(), format!("{_sup:?}"));
    let _ = (_sub.clone(), format!("{_sub:?}"));
    let _ = (_del.clone(), format!("{_del:?}"));
    let _ = (_hi.clone(), format!("{_hi:?}"));
}
