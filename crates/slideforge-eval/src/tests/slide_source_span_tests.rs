//! Failing tests for the slide source-span threading gap (PR-A Finding 2).
//!
//! ## Root cause (architect-confirmed)
//!
//! `for_eval.rs::eval_slide_node` hardcodes `source_span: SourceSpan::default()`
//! (~line 346). The real span on the `Spanned<SlideNode>` wrapper — accessible as
//! `spanned_slide.span()` in `eval_block_items` (~line 398) — is discarded.
//!
//! The bridge `span_to_source_span(Span) -> SourceSpan` in `if_eval.rs` (~line 65)
//! populates `SourceSpan { file: "<byte:N>", line: 0, col: 0, byte_offset: N }`
//! where N is the syntax `Span.start` byte offset. The same bridge must be used in
//! `eval_block_items` to thread the `Spanned<SlideNode>.span()` into each
//! produced `Slide.source_span`.
//!
//! ## Red Gate contract
//!
//! All tests in this module MUST FAIL before the implementer's fix. After the fix
//! (`eval_block_items` threads the span from `spanned_slide.span()` into each
//! resulting slide via `span_to_source_span`), all tests must pass.
//!
//! ## DSL syntax note
//!
//! The slideforge DSL uses `slide <type>:` as the slide keyword, with fields
//! written as `  <field> "<value>"` (no colon on field lines). For example:
//!
//! ```text
//! slideforge_version "1"
//! lang "en-US"
//!
//! slide title:
//!   title "Hello"
//! ```
//!
//! ## Traceability
//!
//! PR-A Finding 2 (diag-span); `for_eval.rs` ~line 346 and ~line 398;
//! `if_eval.rs::span_to_source_span` ~line 65.

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;

    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::parse;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::SourceSpan;

    use crate::config::EvalConfig;
    use crate::eval::eval_deck;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Parse `src` and evaluate it with `eval_deck`, panicking on any parse or
    /// eval error. Returns the resulting [`Deck`].
    fn parse_and_eval(src: &str) -> slideforge_types::Deck {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from("test.sf"), Arc::from(src));

        let parse_result =
            parse(src, file_id, &sm).expect("source must parse without fatal errors");

        // Allow only version warnings and W-PAR- advisory warnings.
        for w in &parse_result.warnings {
            let msg = format!("{w:?}");
            assert!(
                msg.contains("version") || msg.contains("Version") || msg.contains("W-PAR-"),
                "unexpected parse warning: {msg}"
            );
        }

        let config = EvalConfig::default();
        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &config, &mut sink)
            .expect("eval_deck must return Some for valid source");
        assert!(
            sink.is_empty(),
            "no eval diagnostics expected; got: {:?}",
            sink.errors()
        );
        deck
    }

    // ── Finding 2 tests ───────────────────────────────────────────────────────

    /// A single slide's `source_span` must NOT be `SourceSpan::default()`.
    ///
    /// After `eval_block_items` threads the span from `spanned_slide.span()`
    /// via `span_to_source_span`, the byte offset must be non-zero for a slide
    /// that appears after the deck preamble.
    ///
    /// Red Gate: `eval_slide_node` currently hardcodes `SourceSpan::default()`
    /// → `slide.source_span == SourceSpan::default()` → assertion FAILS.
    #[test]
    fn test_BC_diag_span_slide_source_span_not_default() {
        // Place a slide after a non-trivial preamble so its byte offset is > 0.
        // "slideforge_version \"1\"\n" = 24 bytes; "lang \"en-US\"\n" = 13 bytes;
        // "\n" = 1 byte → total preamble ≥ 38 bytes before the slide keyword.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "\n",
            "slide title:\n",
            "  title \"Hello\"\n",
        );

        let deck = parse_and_eval(src);

        assert_eq!(
            deck.slides.len(),
            1,
            "expected 1 slide; got {}",
            deck.slides.len()
        );

        let span = &deck.slides[0].source_span;
        assert_ne!(
            *span,
            SourceSpan::default(),
            "slide[0].source_span must NOT be SourceSpan::default() after span threading; \
             got: {span:?}. \
             Root cause: eval_slide_node hardcodes SourceSpan::default() (for_eval.rs ~line 346). \
             Fix: thread spanned_slide.span() via span_to_source_span in eval_block_items."
        );
    }

    /// The slide's `source_span.byte_offset` must correspond to its actual
    /// position in the source string — i.e., it must be non-zero when the slide
    /// appears after a non-empty preamble.
    ///
    /// `span_to_source_span` in `if_eval.rs` sets:
    ///   `byte_offset = syntax_span.start`
    ///
    /// For the first slide after a preamble, `syntax_span.start > 0`.
    ///
    /// Red Gate: `source_span == SourceSpan::default()` → `byte_offset == 0`
    /// → `assert!(byte_offset > 0)` FAILS.
    #[test]
    fn test_BC_diag_span_slide_byte_offset_nonzero_after_preamble() {
        // "slideforge_version \"1\"\n" = 24 bytes
        // "lang \"en-US\"\n"           = 13 bytes  → total 37 bytes before "\n"
        // "\n"                           =  1 byte  → total 38 bytes before "slide title:\n"
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "\n",
            "slide title:\n",
            "  title \"First slide\"\n",
        );

        let deck = parse_and_eval(src);

        assert_eq!(deck.slides.len(), 1);
        let byte_offset = deck.slides[0].source_span.byte_offset;
        assert!(
            byte_offset > 0,
            "slide[0].source_span.byte_offset must be > 0 for a slide that starts \
             after a non-empty preamble; got byte_offset = {byte_offset}. \
             Root cause: eval_slide_node hardcodes SourceSpan::default() \
             (for_eval.rs ~line 346)."
        );
    }

    /// For a multi-slide deck, each slide's `source_span` must be distinct and
    /// monotonically increasing by `byte_offset` — i.e., later slides in source
    /// must have larger byte offsets than earlier ones.
    ///
    /// Red Gate: all slides have `SourceSpan::default()` → all byte_offsets == 0
    /// → the strict-inequality assertion FAILS (0 is not strictly less than 0).
    #[test]
    fn test_BC_diag_span_multi_slide_spans_are_monotone() {
        // Two slides separated by a blank line; second slide's byte offset > first's.
        let src = concat!(
            "slideforge_version \"1\"\n",
            "lang \"en-US\"\n",
            "\n",
            "slide title:\n",
            "  title \"Slide One\"\n",
            "\n",
            "slide content:\n",
            "  title \"Slide Two\"\n",
        );

        let deck = parse_and_eval(src);

        assert_eq!(
            deck.slides.len(),
            2,
            "expected 2 slides; got {}",
            deck.slides.len()
        );

        let span0 = &deck.slides[0].source_span;
        let span1 = &deck.slides[1].source_span;

        // Both must differ from default.
        assert_ne!(
            *span0,
            SourceSpan::default(),
            "slide[0].source_span must not be default; got: {span0:?}"
        );
        assert_ne!(
            *span1,
            SourceSpan::default(),
            "slide[1].source_span must not be default; got: {span1:?}"
        );

        // Second slide must appear after first in the source.
        assert!(
            span1.byte_offset > span0.byte_offset,
            "slide[1].byte_offset ({}) must be > slide[0].byte_offset ({}) — \
             later slides start later in the source. \
             Root cause: all source_spans are SourceSpan::default() (byte_offset 0).",
            span1.byte_offset,
            span0.byte_offset
        );
    }

    /// The `source_span.byte_offset` for the first slide must be at or beyond
    /// the byte position of the preamble — i.e., ≥ `preamble.len()`.
    ///
    /// `span_to_source_span` stores `byte_offset = syntax_span.start`.
    /// Since the slide keyword appears after the preamble, `syntax_span.start`
    /// must be at least `preamble.len()`.
    ///
    /// Red Gate: `byte_offset == 0` (default) → assertion `>= preamble.len()` FAILS.
    #[test]
    fn test_BC_diag_span_byte_offset_not_less_than_preamble_length() {
        // Build a source where the exact byte position of the first slide is known.
        // preamble = "slideforge_version \"1\"\nlang \"en-US\"\n\n"
        //          = 24 + 13 + 1 = 38 bytes
        let preamble = "slideforge_version \"1\"\nlang \"en-US\"\n\n";
        let slide_src = "slide title:\n  title \"Positioned\"\n";
        let src = format!("{preamble}{slide_src}");

        let expected_min_offset = preamble.len(); // 38

        let deck = parse_and_eval(&src);

        assert_eq!(deck.slides.len(), 1);
        let byte_offset = deck.slides[0].source_span.byte_offset;

        // The implementer threads `spanned_slide.span().start` into `byte_offset`.
        // The slide keyword starts at exactly `preamble.len()` bytes into the source.
        assert!(
            byte_offset >= expected_min_offset,
            "slide[0].source_span.byte_offset ({byte_offset}) must be ≥ preamble length \
             ({expected_min_offset}) — the slide starts after the preamble. \
             Root cause: eval_slide_node hardcodes SourceSpan::default()."
        );

        // Belt-and-suspenders: it must also be within the length of the full source.
        assert!(
            byte_offset < src.len(),
            "slide[0].source_span.byte_offset ({byte_offset}) must be within source bounds \
             (src.len() = {})",
            src.len()
        );
    }
}
