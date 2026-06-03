#![allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
#![allow(non_snake_case)] // test_BC_S_SS_NNN_xxx naming convention per TDD traceability
#![allow(clippy::doc_markdown)] // test function names in doc comments do not need backticks
#![allow(clippy::redundant_closure_for_method_calls)]
// These lints fire in the test-writer-generated test bodies (Red Gate + Green Gate):
// collapsible_if: nested if-let chains are clearer when kept separate in tests.
// used_underscore_binding: comemo/Hash derive test uses _italic, _code, etc. intentionally.
#![allow(clippy::collapsible_if, clippy::used_underscore_binding)]
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
        panic!(
            "detail field must be Template; got: {:?}",
            field.value.value()
        );
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
            assert!(!content.is_empty(), "Code span content must not be empty");
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

// ─── Test 13: Unclosed bold error is strict-build-fatal (EC-007 / F-077-P14-001) ─

/// EC-007 / BC-3.02.002 AC-002 / error-taxonomy.md:24 (E-PAR always fatal):
/// `"**unclosed"` (no closing `**`) must:
/// - Return `Err(errors)` from `parse()` in strict (default) mode — E-PAR-019 is
///   ALWAYS FATAL per error-taxonomy.md ("Parse Errors (E-PAR) — Always fatal.
///   Build halts with accumulated errors. No output produced." exit 1).
/// - Accumulate the error (not fail-on-first) — error ACCUMULATION is preserved.
/// - The E-PAR-019 error code must appear in the accumulated errors.
/// - Error message must be CLEAN (no SLIDEFORGE_INLINE_ROUTE / Custom( sentinel leak).
///
/// Note: error accumulation (continue parsing, collect all errors) is PRESERVED.
/// The conflation that was wrong: "accumulation" ≠ "non-fatal". The build is fatal
/// in strict mode (default) even after accumulating all errors. (F-077-P14-001)
///
/// REWROTE from the original non-fatal assertion (which encoded the spec violation).
#[test]
fn test_BC_3_02_002_unclosed_bold_error_accumulated() {
    use crate::parser::parse;
    use miette::Diagnostic as _;

    let src = "slide content:\n  detail \"**unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    // E-PAR-019 is ALWAYS FATAL — parse() MUST return Err in strict (default) mode.
    // (error-taxonomy.md:24 + error-taxonomy.md:55 + STORY-077 EC-007)
    let Err(errors) = result else {
        panic!(
            "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
             parse() returned Ok for '**unclosed' — E-PAR-019 must be strict-build-fatal \
             (exit 1) in default mode. error-taxonomy.md:24: E-PAR errors are Always fatal. \
             (F-077-P14-001)"
        )
    };

    // The errors vec must contain at least 1 entry (error accumulation is preserved).
    assert!(
        !errors.is_empty(),
        "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
         Err was returned but with an empty errors vec — error accumulation broken."
    );

    // The first error must carry code E-PAR-019 (not E-PAR-002 / UnexpectedToken).
    let first = &errors[0];
    let code = first
        .code()
        .expect("E-PAR-019 error must carry a diagnostic code");
    let code_str = code.to_string();
    assert!(
        code_str.contains("E-PAR-019"),
        "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
         expected code E-PAR-019 for unclosed bold; got: {code_str}. \
         (F-077-P14-001)"
    );

    // The error variant must be UnclosedInlineMarkup (not UnexpectedToken).
    assert!(
        matches!(
            first,
            crate::error::SyntaxError::UnclosedInlineMarkup { .. }
        ),
        "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
         expected SyntaxError::UnclosedInlineMarkup variant; got: {first:?}"
    );

    // The rendered message must be CLEAN — no routing sentinel leak.
    let rendered = first.to_string();
    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
         rendered message contains routing sentinel. Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_BC_3_02_002_unclosed_bold_error_accumulated FAIL: \
         rendered message contains 'Custom(' wrapper. Rendered: {rendered:?}"
    );
}

// ─── Test 14: Empty bold error is strict-build-fatal (EC-008 / F-077-P14-001) ──

/// EC-008 / BC-3.02.002 AC-002 / error-taxonomy.md:24 (E-PAR always fatal):
/// `"****"` (empty bold span) must:
/// - Return `Err(errors)` from `parse()` in strict (default) mode — E-PAR-020 is
///   ALWAYS FATAL (error-taxonomy.md:24, exit 1, note 57).
/// - The E-PAR-020 error code must appear in the accumulated errors.
/// - Error message must be CLEAN (no routing sentinel leak).
///
/// REWROTE from the original non-fatal assertion (which encoded the spec violation).
/// (F-077-P14-001)
#[test]
fn test_BC_3_02_002_empty_bold_error_accumulated() {
    use crate::parser::parse;
    use miette::Diagnostic as _;

    let src = "slide content:\n  detail \"****\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    // E-PAR-020 is ALWAYS FATAL — parse() MUST return Err in strict (default) mode.
    let Err(errors) = result else {
        panic!(
            "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
             parse() returned Ok for '****' — E-PAR-020 must be strict-build-fatal \
             (exit 1) in default mode. error-taxonomy.md:24: E-PAR errors are Always fatal. \
             (F-077-P14-001)"
        )
    };

    assert!(
        !errors.is_empty(),
        "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
         Err returned but errors vec is empty — accumulation broken."
    );

    // The first error must carry code E-PAR-020 (not E-PAR-002 / UnexpectedToken).
    let first = &errors[0];
    let code = first
        .code()
        .expect("E-PAR-020 error must carry a diagnostic code");
    let code_str = code.to_string();
    assert!(
        code_str.contains("E-PAR-020"),
        "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
         expected code E-PAR-020 for empty bold span; got: {code_str}. \
         (F-077-P14-001)"
    );

    // The error variant must be EmptyInlineMarkupSpan.
    assert!(
        matches!(
            first,
            crate::error::SyntaxError::EmptyInlineMarkupSpan { .. }
        ),
        "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
         expected SyntaxError::EmptyInlineMarkupSpan variant; got: {first:?}"
    );

    // The rendered message must be CLEAN.
    let rendered = first.to_string();
    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
         rendered message contains routing sentinel. Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_BC_3_02_002_empty_bold_error_accumulated FAIL: \
         rendered message contains 'Custom(' wrapper. Rendered: {rendered:?}"
    );
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
    assert_eq!(
        bold, bold2,
        "Bold must implement PartialEq (clone equality)"
    );

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

// ═══════════════════════════════════════════════════════════════════════════════
// F-077-P5-001: Parser produces Expr::Call for ref("id"), figref(n), footnote("t")
//
// These tests prove the REAL DSL path: `{{ ref("slide-1") }}` in a field value
// is parsed by `parse_inner_expr` (via `expr()`) and produces `Expr::Call`, not
// `Expr::Error`. Before `Expr::Call` was added to the grammar, these produced
// `Expr::Error` because the parser saw `ref` (identifier) then `("id")` (trailing
// unconsumed tokens → parse failure).
// ═══════════════════════════════════════════════════════════════════════════════

/// F-077-P5-001: `ref("slide-1")` as an expression must produce
/// `Expr::Call { func: "ref", args: [Str("slide-1")] }`.
///
/// This tests `parse_inner_expr_for_test` directly — the function called by
/// `scan_template_chunks` to parse the content inside `{{ ... }}`.
///
/// We test through `parse_inner_expr_for_test` rather than `parse_template_value`
/// because the DSL string-literal lexer stores StringLit tokens verbatim (raw
/// bytes), so a DSL field value `"{{ ref(\"id\") }}"` would have the literal
/// backslashes present when `parse_inner_expr` re-lexes the content — causing
/// parse failure. The direct test bypasses that round-trip and tests the `expr()`
/// combinator with the already-unquoted content `ref("slide-1")`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_ref_call_parses_to_expr_call() {
    // Call parse_inner_expr directly with the unquoted expression content.
    // This is what scan_template_chunks passes to parse_inner_expr after
    // extracting the content between {{ and }}.
    let result = crate::parser::template::parse_inner_expr_for_test(r#"ref("slide-1")"#);

    match result {
        Ok(Expr::Call { func, args }) => {
            assert_eq!(
                func.as_str(),
                "ref",
                "F-077-P5-001: Call func must be 'ref'; got: {func:?}"
            );
            assert_eq!(
                args.len(),
                1,
                "F-077-P5-001: Call must have 1 arg; got {args:?}"
            );
            assert!(
                matches!(&args[0], Expr::Str(s) if s == "slide-1"),
                "F-077-P5-001: arg must be Str(\"slide-1\"); got: {:?}",
                args[0]
            );
        },
        Ok(Expr::Error) => panic!(
            "F-077-P5-001 FAIL: ref(\"slide-1\") produced Expr::Error — \
             the parser did not recognize the call syntax. \
             Expr::Call must be added to the expression grammar."
        ),
        Ok(other) => panic!("F-077-P5-001 FAIL: expected Expr::Call; got: {other:?}"),
        Err(()) => panic!(
            "F-077-P5-001 FAIL: parse_inner_expr returned Err for ref(\"slide-1\") — \
             lex or parse failure"
        ),
    }
}

/// F-077-P5-001: `footnote("see appendix")` as an expression must produce
/// `Expr::Call { func: "footnote", args: [Str("see appendix")] }`.
///
/// Same direct `parse_inner_expr_for_test` approach as the ref() test.
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_footnote_call_parses_to_expr_call() {
    let result = crate::parser::template::parse_inner_expr_for_test(r#"footnote("see appendix")"#);

    match result {
        Ok(Expr::Call { func, args }) => {
            assert_eq!(func.as_str(), "footnote", "Call func must be 'footnote'");
            assert_eq!(args.len(), 1, "footnote must have 1 arg");
            assert!(
                matches!(&args[0], Expr::Str(s) if s == "see appendix"),
                "arg must be Str(\"see appendix\"); got: {:?}",
                args[0]
            );
        },
        Ok(Expr::Error) => {
            panic!("F-077-P5-001 FAIL: footnote(\"see appendix\") produced Expr::Error")
        },
        Ok(other) => panic!("F-077-P5-001 FAIL: expected Expr::Call for footnote; got: {other:?}"),
        Err(()) => panic!("F-077-P5-001 FAIL: parse_inner_expr returned Err for footnote(...)"),
    }
}

/// F-077-P5-001: `{{ figref(3) }}` must produce
/// `TemplateChunk::Expr(Expr::Call { func: "figref", args: [Num(3)] })`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_figref_call_parses_to_expr_call() {
    let chunks = parse_template_value("{{ figref(3) }}");

    assert_eq!(
        chunks.len(),
        1,
        "figref(3) must produce 1 chunk; got {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Expr(Expr::Call { func, args }) => {
            assert_eq!(func.as_str(), "figref", "Call func must be 'figref'");
            assert_eq!(args.len(), 1, "figref must have 1 arg");
            assert!(
                matches!(&args[0], Expr::Num(3)),
                "figref arg must be Num(3); got: {:?}",
                args[0]
            );
        },
        TemplateChunk::Expr(Expr::Error) => {
            panic!("F-077-P5-001 FAIL: figref(3) produced Expr::Error")
        },
        other => panic!("F-077-P5-001 FAIL: expected Expr::Call for figref; got: {other:?}"),
    }
}

/// F-077-P5-001: `{{ figref(3) }}` in a template value must produce
/// `[TemplateChunk::Expr(Expr::Call { func: "figref", args: [Num(3)] })]`.
///
/// This uses `parse_template_value` (not `parse_inner_expr_for_test`) because
/// `figref(3)` has no inner quotes and works through the full DSL round-trip.
/// It tests the composition of Call with the template scanner.
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_ref_call_composes_with_surrounding_text() {
    // figref(3) has no inner quotes, so the round-trip through parse_template_value works.
    // DSL source: detail "Count: {{ figref(3) }}."
    let chunks = parse_template_value("Count: {{ figref(3) }}.");

    assert_eq!(
        chunks.len(),
        3,
        "mixed literal+call+literal must produce 3 chunks; got {chunks:?}"
    );
    assert!(
        matches!(&chunks[0], TemplateChunk::Literal(s) if s == "Count: "),
        "first chunk must be Literal(\"Count: \"); got: {:?}",
        chunks[0]
    );
    match &chunks[1] {
        TemplateChunk::Expr(Expr::Call { func, args }) => {
            assert_eq!(
                func.as_str(),
                "figref",
                "F-077-P5-001: func must be 'figref'"
            );
            assert_eq!(args.len(), 1, "figref must have 1 arg");
            assert!(
                matches!(&args[0], Expr::Num(3)),
                "figref arg must be Num(3); got: {:?}",
                args[0]
            );
        },
        TemplateChunk::Expr(Expr::Error) => {
            panic!("F-077-P5-001 FAIL: figref(3) produced Expr::Error — call syntax not parsed")
        },
        other => {
            panic!("F-077-P5-001 FAIL: second chunk must be Expr::Call for figref; got: {other:?}")
        },
    }
    assert!(
        matches!(&chunks[2], TemplateChunk::Literal(s) if s == "."),
        "third chunk must be Literal(\".\"); got: {:?}",
        chunks[2]
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// E-PAR-019 / E-PAR-020 code-assertion tests (anti-collision guard for STORY-077)
//
// These tests assert that:
//   - unclosed inline markup (`**foo`) produces a warning with code E-PAR-019
//   - empty inline markup span (`****`) produces a warning with code E-PAR-020
//
// This prevents regression of the error-code collision found during adversarial
// review (STORY-077 fix pass): E-PAR-015 and E-PAR-016 are SHAPE codes; the
// inline-markup parser must use DEDICATED codes E-PAR-019 and E-PAR-020.
//
// The tests assert via miette::Diagnostic::code() — a load-bearing check on the
// actual diagnostic code attribute, not on the message string.
// ═══════════════════════════════════════════════════════════════════════════════

/// E-PAR-019 code assertion: unclosed `**foo` must return `Err` from `parse()` in
/// strict (default) mode, and the first error must carry code `E-PAR-019`.
///
/// This test guards against the E-PAR-015 collision: the unclosed-inline-markup
/// error must NOT reuse E-PAR-015 (which belongs to SHAPE parsing).
///
/// REWROTE: original asserted Ok(pr.warnings) — wrong, E-PAR-019 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_E_PAR_019_unclosed_inline_markup_code_assertion() {
    use crate::parser::parse;
    use crate::span::SourceMap;
    use miette::Diagnostic;

    let src = "slide content:\n  detail \"**unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    // E-PAR-019 is ALWAYS FATAL — must return Err in strict (default) mode.
    let Err(errors) = result else {
        panic!(
            "test_E_PAR_019_unclosed_inline_markup_code_assertion FAIL: \
             parse() returned Ok — E-PAR-019 must be strict-build-fatal (exit 1). \
             error-taxonomy.md:24: E-PAR errors are Always fatal. (F-077-P14-001)"
        )
    };

    assert!(
        !errors.is_empty(),
        "test_E_PAR_019_unclosed_inline_markup_code_assertion FAIL: \
         unclosed '**foo' must produce at least 1 fatal error; got 0"
    );

    // The FIRST error must carry code E-PAR-019, not E-PAR-002 or E-PAR-015.
    let first = &errors[0];
    let code = first
        .code()
        .expect("error must have a diagnostic code (E-PAR-019)");
    let code_str = code.to_string();
    assert!(
        code_str.contains("E-PAR-019"),
        "test_E_PAR_019_unclosed_inline_markup_code_assertion FAIL: \
         expected code E-PAR-019 for unclosed inline markup, got: {code_str}\n\
         If this shows E-PAR-002: the error is being emitted via UnexpectedToken (wrong variant).\n\
         If this shows E-PAR-015: the E-PAR-015/019 code collision is not fixed."
    );
}

/// E-PAR-020 code assertion: empty `****` must return `Err` from `parse()` in
/// strict (default) mode, and the first error must carry code `E-PAR-020`.
///
/// This test guards against the E-PAR-016 collision: the empty-inline-markup-span
/// error must NOT reuse E-PAR-016 (which belongs to SHAPE parsing).
///
/// REWROTE: original asserted Ok(pr.warnings) — wrong, E-PAR-020 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_E_PAR_020_empty_inline_markup_span_code_assertion() {
    use crate::parser::parse;
    use crate::span::SourceMap;
    use miette::Diagnostic;

    let src = "slide content:\n  detail \"****\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    // E-PAR-020 is ALWAYS FATAL — must return Err in strict (default) mode.
    let Err(errors) = result else {
        panic!(
            "test_E_PAR_020_empty_inline_markup_span_code_assertion FAIL: \
             parse() returned Ok — E-PAR-020 must be strict-build-fatal (exit 1). \
             error-taxonomy.md:24: E-PAR errors are Always fatal. (F-077-P14-001)"
        )
    };

    assert!(
        !errors.is_empty(),
        "test_E_PAR_020_empty_inline_markup_span_code_assertion FAIL: \
         empty '****' must produce at least 1 fatal error; got 0"
    );

    // The FIRST error must carry code E-PAR-020, not E-PAR-002 or E-PAR-016.
    let first = &errors[0];
    let code = first
        .code()
        .expect("error must have a diagnostic code (E-PAR-020)");
    let code_str = code.to_string();
    assert!(
        code_str.contains("E-PAR-020"),
        "test_E_PAR_020_empty_inline_markup_span_code_assertion FAIL: \
         expected code E-PAR-020 for empty inline markup span, got: {code_str}\n\
         If this shows E-PAR-002: the error is being emitted via UnexpectedToken (wrong variant).\n\
         If this shows E-PAR-016: the E-PAR-016/020 code collision is not fixed."
    );
}

/// E-PAR-019 variant check: the fatal error variant must be
/// `SyntaxError::UnclosedInlineMarkup`, not `SyntaxError::UnexpectedToken`.
///
/// This structural check ensures the routing in mod.rs produces the correct
/// variant (TD-VSDD-059: load-bearing assertion, not just a doc comment).
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-019 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_E_PAR_019_warning_is_unclosed_inline_markup_variant() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    let src = "slide content:\n  detail \"**unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-019 must be fatal in strict mode — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_E_PAR_019_warning_is_unclosed_inline_markup_variant FAIL: \
         parse() returned Ok — E-PAR-019 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_E_PAR_019_warning_is_unclosed_inline_markup_variant FAIL: \
         no error produced"
    );

    let first = &errors[0];
    assert!(
        matches!(first, SyntaxError::UnclosedInlineMarkup { .. }),
        "test_E_PAR_019_warning_is_unclosed_inline_markup_variant FAIL: \
         expected SyntaxError::UnclosedInlineMarkup variant; got: {first:?}"
    );
}

/// E-PAR-020 variant check: the fatal error variant must be
/// `SyntaxError::EmptyInlineMarkupSpan`, not `SyntaxError::UnexpectedToken`.
///
/// This structural check ensures the routing in mod.rs produces the correct
/// variant (TD-VSDD-059: load-bearing assertion, not just a doc comment).
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-020 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_E_PAR_020_warning_is_empty_inline_markup_span_variant() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    let src = "slide content:\n  detail \"****\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-020 must be fatal in strict mode — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_E_PAR_020_warning_is_empty_inline_markup_span_variant FAIL: \
         parse() returned Ok — E-PAR-020 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_E_PAR_020_warning_is_empty_inline_markup_span_variant FAIL: \
         no error produced"
    );

    let first = &errors[0];
    assert!(
        matches!(first, SyntaxError::EmptyInlineMarkupSpan { .. }),
        "test_E_PAR_020_warning_is_empty_inline_markup_span_variant FAIL: \
         expected SyntaxError::EmptyInlineMarkupSpan variant; got: {first:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-077-P4-002 (backtick delimiter preservation)
//
// BUG: when the delimiter IS a backtick (`` ` ``), the E-PAR-019 / E-PAR-020
// message embeds the delimiter as three consecutive backticks, so
// `extract_backtick_name` returns "" → delimiter field is "" / "?" in the
// produced SyntaxError.
//
// These tests assert that the CORRECT delimiter `` ` `` (backtick) is preserved
// in the UnclosedInlineMarkup / EmptyInlineMarkupSpan variants.  They FAIL until
// routing is kind-based (not message-string-based).
// ═══════════════════════════════════════════════════════════════════════════════

/// F-077-P4-002: unclosed backtick code span must produce a FATAL E-PAR-019 error
/// with `SyntaxError::UnclosedInlineMarkup { delimiter: "`" }` — delimiter is `` ` ``,
/// NOT empty string or "?".
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-019 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P4_002_backtick_unclosed_delimiter_preserved() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // Detail field with an unclosed backtick code span: `unclosed
    let src = "slide content:\n  detail \"`unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-019 must be fatal — parse() must return Err in strict mode.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P4_002_backtick_unclosed_delimiter_preserved FAIL: \
         parse() returned Ok — E-PAR-019 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P4_002_backtick_unclosed_delimiter_preserved FAIL: \
         no error produced for unclosed backtick span"
    );

    let first = &errors[0];
    match first {
        SyntaxError::UnclosedInlineMarkup { delimiter, .. } => {
            assert_eq!(
                delimiter, "`",
                "test_F077_P4_002_backtick_unclosed_delimiter_preserved FAIL: \
                 delimiter must be \"`\" (backtick); got: {delimiter:?}\n\
                 If empty: extract_backtick_name lost the delimiter when the message \
                 embedded ` inside backtick pairs. Fix: route via TemplateErrorKind."
            );
        },
        other => panic!(
            "test_F077_P4_002_backtick_unclosed_delimiter_preserved FAIL: \
             expected SyntaxError::UnclosedInlineMarkup; got: {other:?}"
        ),
    }
}

/// F-077-P4-002: empty backtick code span (two adjacent backticks `` `` ``) must
/// produce a FATAL E-PAR-020 error with
/// `SyntaxError::EmptyInlineMarkupSpan { delimiter: "`" }` — delimiter is `` ` ``,
/// NOT empty string or "?".
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-020 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P4_002_backtick_empty_span_delimiter_preserved() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // Detail field with an empty backtick code span: ``
    let src = "slide content:\n  detail \"``\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-020 must be fatal — parse() must return Err in strict mode.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P4_002_backtick_empty_span_delimiter_preserved FAIL: \
         parse() returned Ok — E-PAR-020 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P4_002_backtick_empty_span_delimiter_preserved FAIL: \
         no error produced for empty backtick span"
    );

    let first = &errors[0];
    match first {
        SyntaxError::EmptyInlineMarkupSpan { delimiter, .. } => {
            assert_eq!(
                delimiter, "`",
                "test_F077_P4_002_backtick_empty_span_delimiter_preserved FAIL: \
                 delimiter must be \"`\" (backtick); got: {delimiter:?}\n\
                 If empty: extract_backtick_name lost the delimiter when the message \
                 embedded ` inside backtick pairs. Fix: route via TemplateErrorKind."
            );
        },
        other => panic!(
            "test_F077_P4_002_backtick_empty_span_delimiter_preserved FAIL: \
             expected SyntaxError::EmptyInlineMarkupSpan; got: {other:?}"
        ),
    }
}

/// F-077-P4-001 + F-077-P4-002 combined: verify delimiter field is non-empty and
/// correct for `**`, `_`, and `` ` `` delimiters (all three must preserve their
/// delimiter string unmodified in the SyntaxError variant).
///
/// REWROTE: original used unwrap_or_else(|_| panic!("...not fatal")) — E-PAR-019
/// is ALWAYS FATAL per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P4_002_delimiter_preserved_for_all_markup_types() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    let cases: &[(&str, &str, &str)] = &[
        (
            "slide content:\n  detail \"**unclosed\"\n",
            "**",
            "unclosed bold",
        ),
        (
            "slide content:\n  detail \"_unclosed\"\n",
            "_",
            "unclosed italic",
        ),
        (
            "slide content:\n  detail \"`unclosed\"\n",
            "`",
            "unclosed code span",
        ),
    ];

    for (src, expected_delim, label) in cases {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(*src));

        // E-PAR-019 is ALWAYS FATAL — parse() must return Err in strict mode.
        let Err(errors) = parse(src, file_id, &sm) else {
            panic!(
                "test_F077_P4_002_delimiter_preserved_for_all_markup_types FAIL ({label}): \
                 parse() returned Ok — E-PAR-019 must be strict-build-fatal. (F-077-P14-001)"
            )
        };

        assert!(
            !errors.is_empty(),
            "test_F077_P4_002_delimiter_preserved_for_all_markup_types FAIL ({label}): \
             no error produced"
        );

        let first = &errors[0];
        match first {
            SyntaxError::UnclosedInlineMarkup { delimiter, .. } => {
                assert_eq!(
                    delimiter, expected_delim,
                    "test_F077_P4_002_delimiter_preserved_for_all_markup_types FAIL ({label}): \
                     delimiter must be {expected_delim:?}; got: {delimiter:?}"
                );
            },
            other => panic!(
                "test_F077_P4_002_delimiter_preserved_for_all_markup_types FAIL ({label}): \
                 expected UnclosedInlineMarkup; got: {other:?}"
            ),
        }
    }
}

// ─── Tests for F-077-P5-001: sentinel must not appear in rendered message ────

/// F-077-P5-001 [HIGH]: `SyntaxError::to_string()` for UnclosedInlineMarkup
/// with a backtick delimiter MUST NOT contain the routing sentinel, hex payload,
/// or `Custom(` wrapper.
///
/// The error is FATAL (Err return) in strict mode — we inspect the error in the
/// Err vec. The clean-message assertion still holds: the fatal E-PAR-019 error
/// must have a clean rendered message.
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-019 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_unclosed_backtick_message_is_clean() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // Unclosed backtick inline markup span.
    let src = "slide content:\n  detail \"`unclosed\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-019 is ALWAYS FATAL — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
         parse() returned Ok — E-PAR-019 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: no error produced"
    );

    let rendered = errors[0].to_string();

    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
         rendered message contains routing sentinel — user-facing diagnostic \
         must not expose internal routing tags.\nRendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
         rendered message contains chumsky Debug wrapper 'Custom(' — \
         internal representation must not appear in user diagnostics.\nRendered: {rendered:?}"
    );
    // Sentinel format uses `|` as separator — assert none of the sentinel's
    // pipe-separated fields appear.
    assert!(
        !rendered.contains('|'),
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
         rendered message contains '|' which indicates the routing sentinel payload \
         leaked into the user-facing message.\nRendered: {rendered:?}"
    );
    // The human-readable content must be present: the error code and delimiter.
    assert!(
        rendered.contains("E-PAR-019"),
        "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
         rendered message must contain the E-PAR-019 code.\nRendered: {rendered:?}"
    );

    // The backtick delimiter must be visible in the rendered output.
    match &errors[0] {
        SyntaxError::UnclosedInlineMarkup {
            delimiter, message, ..
        } => {
            assert_eq!(
                delimiter, "`",
                "delimiter field must be backtick; got: {delimiter:?}"
            );
            assert!(
                !message.contains("SLIDEFORGE_INLINE_ROUTE"),
                "message field must not contain routing sentinel; got: {message:?}"
            );
            assert!(
                !message.contains("Custom("),
                "message field must not contain chumsky Debug wrapper; got: {message:?}"
            );
            assert!(
                message.contains("E-PAR-019"),
                "message field must contain E-PAR-019; got: {message:?}"
            );
        },
        other => panic!(
            "test_F077_P5_001_unclosed_backtick_message_is_clean FAIL: \
             expected UnclosedInlineMarkup; got: {other:?}"
        ),
    }
}

/// F-077-P5-001 [HIGH]: same sentinel-leak test for `**` (multi-char delimiter).
///
/// Exercises the `UnclosedInlineMarkup` path with a two-byte delimiter to
/// confirm the hex encoding/decoding round-trip does not corrupt the clean message.
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-019 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_unclosed_bold_message_is_clean() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    let src = "slide content:\n  detail \"**unclosed bold\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-019 is ALWAYS FATAL — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: \
         parse() returned Ok — E-PAR-019 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: no error produced"
    );

    let rendered = errors[0].to_string();

    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: sentinel in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: Custom( in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains('|'),
        "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: '|' in rendered message.\n\
         Rendered: {rendered:?}"
    );

    match &errors[0] {
        SyntaxError::UnclosedInlineMarkup {
            delimiter, message, ..
        } => {
            assert_eq!(
                delimiter, "**",
                "delimiter must be '**'; got: {delimiter:?}"
            );
            assert!(
                !message.contains("SLIDEFORGE_INLINE_ROUTE"),
                "message field must not contain sentinel; got: {message:?}"
            );
            assert!(
                !message.contains("Custom("),
                "message field must not contain Custom(; got: {message:?}"
            );
            assert!(
                message.contains("E-PAR-019"),
                "message field must contain E-PAR-019; got: {message:?}"
            );
            assert!(
                message.contains("**"),
                "message field must contain the delimiter '**'; got: {message:?}"
            );
        },
        other => panic!(
            "test_F077_P5_001_unclosed_bold_message_is_clean FAIL: \
             expected UnclosedInlineMarkup; got: {other:?}"
        ),
    }
}

/// F-077-P5-001 [HIGH]: sentinel-leak test for `EmptyInlineMarkupSpan` with
/// backtick delimiter.
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-020 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_empty_backtick_span_message_is_clean() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // Empty backtick code span: ``
    let src = "slide content:\n  detail \"``\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-020 is ALWAYS FATAL — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: \
         parse() returned Ok — E-PAR-020 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: no error produced"
    );

    let rendered = errors[0].to_string();

    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: sentinel in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: Custom( in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains('|'),
        "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: '|' in rendered message.\n\
         Rendered: {rendered:?}"
    );

    match &errors[0] {
        SyntaxError::EmptyInlineMarkupSpan {
            delimiter, message, ..
        } => {
            assert_eq!(
                delimiter, "`",
                "delimiter must be backtick; got: {delimiter:?}"
            );
            assert!(
                !message.contains("SLIDEFORGE_INLINE_ROUTE"),
                "message field must not contain sentinel; got: {message:?}"
            );
            assert!(
                !message.contains("Custom("),
                "message field must not contain Custom(; got: {message:?}"
            );
            assert!(
                message.contains("E-PAR-020"),
                "message field must contain E-PAR-020; got: {message:?}"
            );
        },
        other => panic!(
            "test_F077_P5_001_empty_backtick_span_message_is_clean FAIL: \
             expected EmptyInlineMarkupSpan; got: {other:?}"
        ),
    }
}

/// F-077-P5-001 [HIGH]: sentinel-leak test for `EmptyInlineMarkupSpan` with
/// `**` delimiter.
///
/// REWROTE: original used .expect("...not fatal") — E-PAR-020 is ALWAYS FATAL
/// per error-taxonomy.md:24. (F-077-P14-001)
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_empty_bold_span_message_is_clean() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // Empty bold span: ****
    let src = "slide content:\n  detail \"****\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

    // E-PAR-020 is ALWAYS FATAL — parse() must return Err.
    let errors = parse(src, file_id, &sm).expect_err(
        "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: \
         parse() returned Ok — E-PAR-020 must be strict-build-fatal. (F-077-P14-001)",
    );

    assert!(
        !errors.is_empty(),
        "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: no error produced"
    );

    let rendered = errors[0].to_string();

    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: sentinel in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: Custom( in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains('|'),
        "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: '|' in rendered message.\n\
         Rendered: {rendered:?}"
    );

    match &errors[0] {
        SyntaxError::EmptyInlineMarkupSpan {
            delimiter, message, ..
        } => {
            assert_eq!(
                delimiter, "**",
                "delimiter must be '**'; got: {delimiter:?}"
            );
            assert!(
                !message.contains("SLIDEFORGE_INLINE_ROUTE"),
                "message field must not contain sentinel; got: {message:?}"
            );
            assert!(
                !message.contains("Custom("),
                "message field must not contain Custom(; got: {message:?}"
            );
            assert!(
                message.contains("E-PAR-020"),
                "message field must contain E-PAR-020; got: {message:?}"
            );
            assert!(
                message.contains("**"),
                "message field must contain the delimiter '**'; got: {message:?}"
            );
        },
        other => panic!(
            "test_F077_P5_001_empty_bold_span_message_is_clean FAIL: \
             expected EmptyInlineMarkupSpan; got: {other:?}"
        ),
    }
}

/// F-077-P5-001 SCRUTINY-2b: non-inline template errors (UnterminatedInterpolation,
/// EmptyInterpolation, UnterminatedMath) must render clean messages with no sentinel
/// contamination (they pass through `into_routing_message` but receive no routing-tag
/// prefix, so they render clean — this asserts that the fix does not regress them).
#[test]
#[allow(non_snake_case)]
fn test_F077_P5_001_scrutiny_2b_non_inline_errors_render_clean() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use crate::span::SourceMap;

    // E-PAR-012: unterminated {{ interpolation
    let src_012 = "slide content:\n  detail \"{{ unclosed\"\n";
    // E-PAR-013: empty {{ }} interpolation
    let src_013 = "slide content:\n  detail \"{{  }}\"\n";
    // E-PAR-014: unterminated $ math block
    let src_014 = "slide content:\n  detail \"$unclosed\"\n";

    for (src, label) in &[
        (src_012, "E-PAR-012 unterminated interpolation"),
        (src_013, "E-PAR-013 empty interpolation"),
        (src_014, "E-PAR-014 unterminated math"),
    ] {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(*src));

        // These may be fatal errors (Err) or produce warnings — check both paths.
        let warnings: Vec<SyntaxError> = match parse(src, file_id, &sm) {
            Ok(pr) => pr.warnings,
            Err(errs) => errs,
        };

        assert!(
            !warnings.is_empty(),
            "test_F077_P5_001_scrutiny_2b FAIL ({label}): no diagnostic produced"
        );

        for warning in &warnings {
            let rendered = warning.to_string();
            assert!(
                !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
                "test_F077_P5_001_scrutiny_2b FAIL ({label}): routing sentinel found in rendered message.\n\
                 Rendered: {rendered:?}"
            );
            assert!(
                !rendered.contains("Custom("),
                "test_F077_P5_001_scrutiny_2b FAIL ({label}): Custom( found in rendered message.\n\
                 Rendered: {rendered:?}"
            );
        }
    }
}

// ─── F-077-P7-001: UTF-8 char-boundary safety (Red Gate) ─────────────────────

/// F-077-P7-001: `**café**` must parse without panic and produce a `Bold` chunk
/// wrapping a `Literal("café")` child.  A single-byte advance at a multi-byte
/// UTF-8 char-lead byte panics with "byte index N is not a char boundary"
/// before the fix.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_bold_with_multibyte_char_no_panic() {
    // parse_template_value panics on fatal errors; a clean valid string must
    // not panic and must return the correct chunk structure.
    let chunks = parse_template_value("**café**");
    assert_eq!(
        chunks.len(),
        1,
        "test_F077_P7_001 FAIL: expected 1 Bold chunk, got: {chunks:?}"
    );
    match &chunks[0] {
        TemplateChunk::Bold(children) => {
            assert_eq!(
                children.len(),
                1,
                "test_F077_P7_001 FAIL: Bold must have 1 child, got: {children:?}"
            );
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "café"),
                "test_F077_P7_001 FAIL: Bold child must be Literal(\"café\"), got: {ch:?}",
                ch = &children[0]
            );
        },
        other => panic!("test_F077_P7_001 FAIL: expected Bold chunk, got: {other:?}"),
    }
}

/// F-077-P7-001: `_naïve_` must parse without panic to `Italic([Literal("naïve")])`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_italic_with_multibyte_char_no_panic() {
    let chunks = parse_template_value("_naïve_");
    assert_eq!(chunks.len(), 1, "expected 1 chunk, got: {chunks:?}");
    match &chunks[0] {
        TemplateChunk::Italic(children) => {
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "naïve"),
                "Italic child must be Literal(\"naïve\"), got: {children:?}"
            );
        },
        other => panic!("expected Italic, got: {other:?}"),
    }
}

/// F-077-P7-001: `==über==` must parse without panic to `Highlight([Literal("über")])`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_highlight_with_multibyte_char_no_panic() {
    let chunks = parse_template_value("==über==");
    assert_eq!(chunks.len(), 1, "expected 1 chunk, got: {chunks:?}");
    match &chunks[0] {
        TemplateChunk::Highlight(children) => {
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "über"),
                "Highlight child must be Literal(\"über\"), got: {children:?}"
            );
        },
        other => panic!("expected Highlight, got: {other:?}"),
    }
}

/// F-077-P7-001: `~~déjà~~` must parse without panic to `Strikethrough([Literal("déjà")])`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_strikethrough_with_multibyte_char_no_panic() {
    let chunks = parse_template_value("~~déjà~~");
    assert_eq!(chunks.len(), 1, "expected 1 chunk, got: {chunks:?}");
    match &chunks[0] {
        TemplateChunk::Strikethrough(children) => {
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "déjà"),
                "Strikethrough child must be Literal(\"déjà\"), got: {children:?}"
            );
        },
        other => panic!("expected Strikethrough, got: {other:?}"),
    }
}

/// F-077-P7-001: `[café](u)` must parse without panic to a Link with `Literal("café")`.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_link_with_multibyte_text_no_panic() {
    let chunks = parse_template_value("[café](https://example.com)");
    assert_eq!(chunks.len(), 1, "expected 1 Link chunk, got: {chunks:?}");
    match &chunks[0] {
        TemplateChunk::Link { text, url } => {
            assert_eq!(url, "https://example.com", "url must be preserved");
            assert!(
                matches!(&text[0], TemplateChunk::Literal(s) if s == "café"),
                "link text must be Literal(\"café\"), got: {text:?}"
            );
        },
        other => panic!("expected Link, got: {other:?}"),
    }
}

/// F-077-P7-001: emoji in bold span must parse without panic.
/// `**🎉**` → `Bold([Literal("🎉")])` — emoji is a 4-byte UTF-8 sequence.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_001_bold_with_emoji_no_panic() {
    let chunks = parse_template_value("**🎉**");
    assert_eq!(chunks.len(), 1, "expected 1 Bold chunk, got: {chunks:?}");
    match &chunks[0] {
        TemplateChunk::Bold(children) => {
            assert!(
                matches!(&children[0], TemplateChunk::Literal(s) if s == "🎉"),
                "Bold child must be Literal(\"🎉\"), got: {children:?}"
            );
        },
        other => panic!("expected Bold, got: {other:?}"),
    }
}

// ─── F-077-P7-002: Nesting depth cap E-PAR-021 (Red Gate) ────────────────────

/// F-077-P7-002: deeply-nested alternating delimiters must NOT overflow the
/// stack and must produce an E-PAR-021 diagnostic.
///
/// Input: 200 alternating `^_` pairs (400 opening delimiters).  Before the
/// nesting-depth cap this would recurse until a stack overflow.
///
/// Uses `parse()` directly (not `parse_template_value`). E-PAR-021 is ALWAYS
/// FATAL (error-taxonomy.md + DIR-077-002 §5), so `parse()` MUST return `Err`
/// in strict (default) mode. (F-077-P14-001, F-077-P15-002)
///
/// Two load-bearing properties are verified:
/// 1. **No stack overflow** — reaching the assertions proves the depth cap fired.
/// 2. **Strict-build-fatal** — `parse()` must return `Err`; `Ok` is a regression.
#[test]
#[allow(non_snake_case)]
fn test_F077_P7_002_deep_nesting_produces_E_PAR_021_no_stack_overflow() {
    use crate::parser::parse;
    use miette::Diagnostic as _;

    // Build a pathological string: `^_^_^_…` (200 pairs = 400 chars).
    // Each `^` or `_` tries to open a new markup span, triggering recursion.
    let deep_content: String = "^_".repeat(200);
    let src = format!("slide content:\n  detail \"{deep_content}\"\n");
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src.as_str()));

    // Must not panic — reaching here means no stack overflow.
    // E-PAR-021 is strict-build-fatal: parse() MUST return Err, never Ok.
    // If E-PAR-021 were routed to ParseResult::warnings, parse() would return
    // Ok and this expect_err call would fail — that is the regression this test
    // is designed to catch. (F-077-P15-002)
    let errors = parse(src.as_str(), file_id, &sm).expect_err(
        "E-PAR-021 must be strict-build-fatal: parse() must return Err (F-077-P14-001)",
    );

    // At least one E-PAR-021 diagnostic must appear in the ERRORS vec (not warnings).
    // Guards the anti-regression contract: must NOT be E-PAR-002 (UnexpectedToken).
    let has_021_by_code = errors.iter().any(|e| {
        e.code()
            .is_some_and(|c| c.to_string().contains("E-PAR-021"))
    });
    assert!(
        has_021_by_code,
        "test_F077_P7_002 FAIL: no InlineNestingDepthExceeded (E-PAR-021) diagnostic \
         in the fatal errors vec for deeply-nested input (by structured .code()).\n\
         errors: {errors:?}"
    );

    // Belt-and-suspenders: message text must also contain E-PAR-021.
    let has_021_by_msg = errors.iter().any(|e| e.to_string().contains("E-PAR-021"));
    assert!(
        has_021_by_msg,
        "test_F077_P7_002 FAIL: E-PAR-021 not found in diagnostic message text.\n\
         errors: {errors:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// F-077-P21-001 (MED): unclosed `[text](url` link must emit E-PAR-019
//
// BUG: when `](` IS found (clear link intent) but `)` is MISSING, control
// falls through to literal — NO error is pushed. Every OTHER delimiter emits
// E-PAR-019 on its unclosed path; the link delimiter is the sole omission.
//
// SPEC BASIS: DIR-077-002 §5 classifies unclosed `[link` as FATAL — same
// class and recovery pattern as unclosed bold.
//
// RED GATE: these tests FAIL before the fix (parse() returns Ok, silent literal).
// After the fix they must PASS (parse() returns Err, E-PAR-019, clean message).
// ═══════════════════════════════════════════════════════════════════════════════

/// F-077-P21-001 (MED) RED GATE: unclosed `[text](url` link (no closing `)`)
/// MUST return `Err` from `parse()` in strict (default) mode, accumulate
/// exactly one E-PAR-019 error, and the first error must be
/// `SyntaxError::UnclosedInlineMarkup` with delimiter `"["`.
///
/// The rendered message must be clean — no SLIDEFORGE_INLINE_ROUTE sentinel,
/// no `Custom(` wrapper, no `|` pipe character.
///
/// FAILS before fix (parse() returns Ok, silent literal — F-077-P21-001).
/// PASSES after fix (link unclosed path pushes UnclosedInlineMarkup("[")).
#[test]
#[allow(non_snake_case)]
fn test_unclosed_link_error_accumulated() {
    use crate::error::SyntaxError;
    use crate::parser::parse;
    use miette::Diagnostic as _;

    let src = "slide content:\n  detail \"[click here](https://example.com\"\n";
    let mut sm = SourceMap::new();
    let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));
    let result = parse(src, file_id, &sm);

    // E-PAR-019 must be FATAL — parse() MUST return Err in strict (default) mode.
    // Before fix: returns Ok (silent literal). After fix: returns Err.
    let Err(errors) = result else {
        panic!(
            "test_unclosed_link_error_accumulated FAIL (F-077-P21-001 RED GATE): \
             parse() returned Ok for '[click here](https://example.com' — \
             unclosed link must be strict-build-fatal E-PAR-019 per DIR-077-002 §5. \
             (This is the pre-fix silent-literal behavior — the bug is that NO error was emitted.)"
        )
    };

    // Error accumulation: at least 1 error (accumulate all, not fail-on-first).
    assert!(
        !errors.is_empty(),
        "test_unclosed_link_error_accumulated FAIL: Err returned but errors vec is empty."
    );

    // Load-bearing: first error must carry code E-PAR-019 (not E-PAR-002 or other).
    let first = &errors[0];
    let code = first
        .code()
        .expect("E-PAR-019 error must carry a diagnostic code");
    let code_str = code.to_string();
    assert!(
        code_str.contains("E-PAR-019"),
        "test_unclosed_link_error_accumulated FAIL (F-077-P21-001): \
         expected code E-PAR-019 for unclosed link; got: {code_str}. \
         The link delimiter must use UnclosedInlineMarkup (E-PAR-019) like all other delimiters."
    );

    // Load-bearing: error variant must be UnclosedInlineMarkup with delimiter "[".
    match first {
        SyntaxError::UnclosedInlineMarkup { delimiter, .. } => {
            assert_eq!(
                delimiter, "[",
                "test_unclosed_link_error_accumulated FAIL (F-077-P21-001): \
                 UnclosedInlineMarkup delimiter must be \"[\"; got: {delimiter:?}"
            );
        },
        other => panic!(
            "test_unclosed_link_error_accumulated FAIL (F-077-P21-001): \
             expected SyntaxError::UnclosedInlineMarkup; got: {other:?}"
        ),
    }

    // Load-bearing: rendered message must be clean — no routing sentinel leak.
    let rendered = first.to_string();
    assert!(
        !rendered.contains("SLIDEFORGE_INLINE_ROUTE"),
        "test_unclosed_link_error_accumulated FAIL: routing sentinel in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains("Custom("),
        "test_unclosed_link_error_accumulated FAIL: 'Custom(' wrapper in rendered message.\n\
         Rendered: {rendered:?}"
    );
    assert!(
        !rendered.contains('|'),
        "test_unclosed_link_error_accumulated FAIL: '|' pipe in rendered message (sentinel leak).\n\
         Rendered: {rendered:?}"
    );
}

/// F-077-P21-001 (MED) REGRESSION GUARD: happy-path `[text](url)` link must
/// still parse to `TemplateChunk::Link` with NO error after the fix.
///
/// This guard must PASS both before and after the fix (it tests the success path).
#[test]
#[allow(non_snake_case)]
fn test_complete_link_still_parses_ok_regression_guard() {
    // parse_template_value panics on fatal errors — success path must not panic.
    let chunks = parse_template_value("[click here](https://example.com)");

    assert_eq!(
        chunks.len(),
        1,
        "test_complete_link_still_parses_ok_regression_guard FAIL (F-077-P21-001): \
         happy-path link must produce exactly 1 chunk; got {chunks:?}"
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
            "test_complete_link_still_parses_ok_regression_guard FAIL (F-077-P21-001): \
             happy-path `[click here](https://example.com)` must produce TemplateChunk::Link; \
             got: {other:?}"
        ),
    }
}

/// F-077-P21-001 (MED) REGRESSION GUARD: bare `[text]` (no `](`) must remain
/// a literal with NO error.
///
/// Only the clear-link-intent case (`[…](…` with no closing `)`) emits E-PAR-019.
/// A bare `[text]` with no `](` is CommonMark-aligned benign prose text.
///
/// This guard must PASS both before and after the fix.
#[test]
#[allow(non_snake_case)]
fn test_bare_bracket_stays_literal_no_error_regression_guard() {
    // parse_template_value panics on fatal errors — bare [text] must not panic.
    let chunks = parse_template_value("[just text]");

    assert!(
        !chunks.is_empty(),
        "test_bare_bracket_stays_literal_no_error_regression_guard FAIL (F-077-P21-001): \
         bare [text] must produce at least 1 chunk (literal)"
    );
    // All chunks must be Literal — NO Link, NO error.
    for chunk in &chunks {
        assert!(
            matches!(chunk, TemplateChunk::Literal(_)),
            "test_bare_bracket_stays_literal_no_error_regression_guard FAIL (F-077-P21-001): \
             bare [text] must produce only Literal chunks; got: {chunk:?}\n\
             A bare '[' with no '](' must NOT be treated as a link opener."
        );
    }
    // The concatenated literal content must contain the bracket text.
    let all_text: String = chunks
        .iter()
        .filter_map(|c| {
            if let TemplateChunk::Literal(s) = c {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        all_text.contains("just text"),
        "test_bare_bracket_stays_literal_no_error_regression_guard FAIL: \
         literal content must contain 'just text'; got all_text={all_text:?}"
    );
}
