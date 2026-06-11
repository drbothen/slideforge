//! Tests for slide source-span threading.
//!
//! ## Background
//!
//! `eval_block_items_with_sections` threads the span from `spanned_slide.span()`
//! via `span_to_source_span` (in `if_eval.rs`) into each resulting `Slide.source_span`.
//! `span_to_source_span` resolves byte offsets to real `file:line:col` when a
//! `SourceMap` is provided via `EvalConfig::source_map` (F-094-P4-004 fix).
//!
//! ## Strengthened per F-094-P4-005
//!
//! Tests now supply a real `SourceMap` and assert actual `file:line:col` values
//! rather than only checking that the span is non-default. The old masking pattern
//! (`span != SourceSpan::default()`) was insufficient because `<byte:N>` sentinels
//! also satisfied that predicate while being fake file names.
//!
//! ## Traceability
//!
//! PR-A Finding 2 (diag-span); F-094-P4-004; F-094-P4-005; `for_eval.rs`
//! `eval_block_items_with_sections`; `if_eval.rs::span_to_source_span`.

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
mod tests {
    use std::sync::Arc;

    use slideforge_syntax::DiagnosticSink;
    use slideforge_syntax::parse;
    use slideforge_syntax::span::SourceMap;
    use slideforge_types::SourceSpan;

    use crate::config::EvalConfig;
    use crate::eval::eval_deck;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Parse `src` as file `file_name` and evaluate it with `eval_deck`,
    /// threading the `SourceMap` into `EvalConfig` so that `span_to_source_span`
    /// resolves byte offsets to real `file:line:col`.
    ///
    /// Panics on any parse or eval error. Returns the resulting [`Deck`].
    fn parse_and_eval_with_map(src: &str, file_name: &str) -> (slideforge_types::Deck, SourceMap) {
        let mut sm = SourceMap::new();
        let file_id = sm.add_file(Arc::from(file_name), Arc::from(src));

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

        // F-094-P4-004: thread the source map into EvalConfig so span resolution
        // produces real file:line:col (not SourceSpan::default() or <byte:N>).
        let config = EvalConfig {
            source_map: Some(Arc::new(sm)),
            ..EvalConfig::default()
        };
        let mut sink = DiagnosticSink::new();
        let deck = eval_deck(&parse_result.deck, &config, &mut sink)
            .expect("eval_deck must return Some for valid source");
        assert!(
            sink.is_empty(),
            "no eval diagnostics expected; got: {:?}",
            sink.errors()
        );
        // Return a reconstructed source map for assertions.
        // (We consumed sm into Arc above; re-create for validation.)
        let mut sm2 = SourceMap::new();
        sm2.add_file(Arc::from(file_name), Arc::from(src));
        (deck, sm2)
    }

    // ── Span threading tests ──────────────────────────────────────────────────

    /// A single slide's `source_span` must carry the real file name, a non-zero
    /// line number, and a non-zero column (F-094-P4-004 / F-094-P4-005).
    ///
    /// With `SourceMap` threaded via `EvalConfig`, `span_to_source_span` resolves
    /// the byte offset to real `file:line:col`. Previously only checked != default;
    /// now strengthened to assert actual values (F-094-P4-005).
    #[test]
    fn test_BC_diag_span_slide_source_span_not_default() {
        // "slideforge_version \"1\"\n" = 24 bytes
        // "lang \"en-US\"\n"           = 13 bytes
        // "\n"                          =  1 byte
        // → preamble 38 bytes; "slide title:\n" starts at byte 38, line 4
        let src = concat!(
            "slideforge_version \"1\"\n", // line 1
            "lang \"en-US\"\n",           // line 2
            "\n",                         // line 3
            "slide title:\n",             // line 4 col 1
            "  title \"Hello\"\n",
        );

        let (deck, _sm) = parse_and_eval_with_map(src, "test.sf");

        assert_eq!(
            deck.slides.len(),
            1,
            "expected 1 slide; got {}",
            deck.slides.len()
        );

        let span = &deck.slides[0].source_span;

        // F-094-P4-005: must be a real resolved span, not default or sentinel.
        assert!(
            !span.is_unknown(),
            "slide[0].source_span must not be unknown after span threading; got: {span:?}"
        );
        assert_eq!(
            span.file.as_ref(),
            "test.sf",
            "slide[0].source_span.file must be 'test.sf'; got: {:?}",
            span.file
        );
        assert_eq!(
            span.line, 4,
            "slide[0].source_span.line must be 4 (line of 'slide title:'); got: {}",
            span.line
        );
        assert!(
            span.col >= 1,
            "slide[0].source_span.col must be >= 1 (1-based); got: {}",
            span.col
        );
        assert_ne!(
            *span,
            SourceSpan::default(),
            "slide[0].source_span must NOT be SourceSpan::default() after span threading"
        );
    }

    /// The slide's `source_span.byte_offset` must correspond to its actual
    /// position in the source string — non-zero when the slide appears after
    /// a non-empty preamble.
    #[test]
    fn test_BC_diag_span_slide_byte_offset_nonzero_after_preamble() {
        let src = concat!(
            "slideforge_version \"1\"\n", // line 1 — 24 bytes
            "lang \"en-US\"\n",           // line 2 — 13 bytes
            "\n",                         // line 3 —  1 byte  → total 38 before slide
            "slide title:\n",             // line 4
            "  title \"First slide\"\n",
        );

        let (deck, _sm) = parse_and_eval_with_map(src, "test.sf");

        assert_eq!(deck.slides.len(), 1);
        let span = &deck.slides[0].source_span;
        let byte_offset = span.byte_offset;
        assert!(
            byte_offset > 0,
            "slide[0].source_span.byte_offset must be > 0 for a slide that starts \
             after a non-empty preamble; got byte_offset = {byte_offset}"
        );
        // F-094-P4-005: also verify real file name and line.
        assert_eq!(
            span.file.as_ref(),
            "test.sf",
            "slide[0].source_span.file must be 'test.sf'; got: {:?}",
            span.file
        );
        assert_eq!(
            span.line, 4,
            "slide[0].source_span.line must be 4 (preamble is 3 lines); got: {}",
            span.line
        );
    }

    /// For a multi-slide deck, each slide's `source_span` must have a real file
    /// name and monotonically increasing `byte_offset` values.
    #[test]
    fn test_BC_diag_span_multi_slide_spans_are_monotone() {
        let src = concat!(
            "slideforge_version \"1\"\n", // line 1
            "lang \"en-US\"\n",           // line 2
            "\n",                         // line 3
            "slide title:\n",             // line 4 — first slide
            "  title \"Slide One\"\n",
            "\n",               // blank separator
            "slide content:\n", // line 7 — second slide
            "  title \"Slide Two\"\n",
        );

        let (deck, _sm) = parse_and_eval_with_map(src, "test.sf");

        assert_eq!(
            deck.slides.len(),
            2,
            "expected 2 slides; got {}",
            deck.slides.len()
        );

        let span0 = &deck.slides[0].source_span;
        let span1 = &deck.slides[1].source_span;

        // Both must have the real file name (F-094-P4-005).
        assert_eq!(
            span0.file.as_ref(),
            "test.sf",
            "slide[0].source_span.file must be 'test.sf'; got: {:?}",
            span0.file
        );
        assert_eq!(
            span1.file.as_ref(),
            "test.sf",
            "slide[1].source_span.file must be 'test.sf'; got: {:?}",
            span1.file
        );

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

        // Line numbers must be monotone (slide 1 is on an earlier line).
        assert!(
            span1.line > span0.line,
            "slide[1].line ({}) must be > slide[0].line ({}) — later slides start later",
            span1.line,
            span0.line
        );

        // Byte offsets must also be monotone.
        assert!(
            span1.byte_offset > span0.byte_offset,
            "slide[1].byte_offset ({}) must be > slide[0].byte_offset ({}) — \
             later slides start later in the source.",
            span1.byte_offset,
            span0.byte_offset
        );
    }

    /// The `source_span.byte_offset` for the first slide must be at or beyond
    /// the byte position of the preamble (>= `preamble.len()`).
    #[test]
    fn test_BC_diag_span_byte_offset_not_less_than_preamble_length() {
        // preamble = "slideforge_version \"1\"\nlang \"en-US\"\n\n" = 24+13+1 = 38 bytes
        let preamble = "slideforge_version \"1\"\nlang \"en-US\"\n\n";
        let slide_src = "slide title:\n  title \"Positioned\"\n";
        let src = format!("{preamble}{slide_src}");

        let expected_min_offset = preamble.len(); // 38

        let (deck, _sm) = parse_and_eval_with_map(&src, "test.sf");

        assert_eq!(deck.slides.len(), 1);
        let span = &deck.slides[0].source_span;
        let byte_offset = span.byte_offset;

        assert!(
            byte_offset >= expected_min_offset,
            "slide[0].source_span.byte_offset ({byte_offset}) must be >= preamble length \
             ({expected_min_offset}) — the slide starts after the preamble."
        );

        // Belt-and-suspenders: must be within source bounds.
        assert!(
            byte_offset < src.len(),
            "slide[0].source_span.byte_offset ({byte_offset}) must be within source bounds \
             (src.len() = {})",
            src.len()
        );

        // F-094-P4-005: must have real file name, not sentinel.
        assert_eq!(
            span.file.as_ref(),
            "test.sf",
            "slide[0].source_span.file must be 'test.sf'; got: {:?}",
            span.file
        );
        assert!(
            !span.file.starts_with("<byte:"),
            "slide[0].source_span.file must NOT be a '<byte:N>' sentinel; got: {:?}",
            span.file
        );
    }
}
