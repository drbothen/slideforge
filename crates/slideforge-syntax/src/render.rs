//! [`DiagnosticRenderer`] — renders accumulated diagnostics to a writer.
//!
//! Wraps [`miette`]'s diagnostic rendering to produce human-readable output
//! for CLI consumers. The renderer respects a `use_color` flag so that
//! terminal ANSI escape codes are suppressed in environments that do not
//! support them (e.g., CI logs, JSON output mode).
//!
//! # Source Order
//!
//! [`DiagnosticRenderer::render_all`] renders diagnostics in the order they
//! appear in the [`crate::DiagnosticSink`], which is source order when the
//! parser pushes errors in source order (AC-010).
//!
//! # Color
//!
//! When `use_color` is `false`, no ANSI escape sequences are emitted.
//! When `use_color` is `true`, [`miette`]'s default graphical handler
//! highlights source snippets with color (AC-009).

use crate::sink::DiagnosticSink;

// ─── DiagnosticRenderer ──────────────────────────────────────────────────────

/// A renderer that writes human-readable diagnostics from a [`DiagnosticSink`]
/// to any [`std::io::Write`] implementation.
///
/// # Example
///
/// ```no_run
/// use slideforge_syntax::{DiagnosticRenderer, DiagnosticSink};
///
/// let sink = DiagnosticSink::new();
/// let renderer = DiagnosticRenderer::new(false);
/// renderer.render_all(&sink, &mut std::io::stderr()).unwrap();
/// ```
pub struct DiagnosticRenderer {
    /// Whether to emit ANSI color escape codes in the output.
    #[allow(dead_code)] // stub field — read by render_all() once implemented
    use_color: bool,
}

impl DiagnosticRenderer {
    /// Construct a new renderer.
    ///
    /// When `use_color` is `false`, all ANSI color escape sequences are
    /// suppressed in the output. When `true`, miette's graphical handler
    /// renders colored source snippets.
    #[must_use]
    pub fn new(_use_color: bool) -> Self {
        todo!("STORY-010: DiagnosticRenderer::new()")
    }

    /// Render all diagnostics from `sink` to `writer`, in source order.
    ///
    /// Each diagnostic is rendered as a complete miette-formatted block
    /// followed by a newline separator. If the sink is empty, nothing is
    /// written to `writer`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any write to `writer` fails.
    pub fn render_all(
        &self,
        _sink: &DiagnosticSink,
        _writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        todo!("STORY-010: DiagnosticRenderer::render_all()")
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagnosticSink, SyntaxError};

    /// Helper: build a cheap `SyntaxError` for renderer testing.
    fn make_error(line: u32, col: u32) -> SyntaxError {
        SyntaxError::indent_error(
            "test.sf".to_string(),
            line,
            col,
            2,
            3,
            "  bad\n".to_string(),
            0,
        )
    }

    // ── AC-009: no ANSI escape codes when use_color=false ────────────────────

    /// AC-009: When `use_color=false`, the rendered output must not contain any
    /// ANSI escape sequences (i.e., no `\x1b[` bytes).
    #[test]
    fn test_ac009_no_ansi_without_color() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let renderer = DiagnosticRenderer::new(false);
        let mut buf = Vec::new();
        renderer
            .render_all(&sink, &mut buf)
            .expect("render_all must not fail");
        let output = String::from_utf8_lossy(&buf);
        assert!(
            !output.contains("\x1b["),
            "use_color=false must produce no ANSI escape codes; got:\n{output}"
        );
    }

    // ── AC-010: render_all writes diagnostics in source order ────────────────

    /// AC-010: `render_all` renders diagnostics in the order they were pushed
    /// into the sink, which must be source-file order. Verify by checking that
    /// the output contains line 1 text before line 3 text.
    #[test]
    fn test_ac010_render_all_source_order() {
        let mut sink = DiagnosticSink::new();
        // Push two errors: line 1 first, then line 3.
        sink.push(make_error(1, 1));
        sink.push(make_error(3, 5));
        let renderer = DiagnosticRenderer::new(false);
        let mut buf = Vec::new();
        renderer
            .render_all(&sink, &mut buf)
            .expect("render_all must not fail");
        let output = String::from_utf8_lossy(&buf);
        // The output for line-1 error must appear before the output for line-3.
        // Both errors contain "test.sf" — check relative order by finding the
        // two occurrences and asserting the first has line 1.
        let first_occurrence = output.find("1:1").unwrap_or(usize::MAX);
        let second_occurrence = output.find("3:5").unwrap_or(usize::MAX);
        assert!(
            first_occurrence < second_occurrence,
            "line 1 error must appear before line 3 error in rendered output;\ngot:\n{output}"
        );
    }

    // ── AC-009: empty sink produces no output ────────────────────────────────

    /// Rendering an empty sink must produce no output.
    #[test]
    fn test_ac009_empty_sink_no_output() {
        let sink = DiagnosticSink::new();
        let renderer = DiagnosticRenderer::new(false);
        let mut buf = Vec::new();
        renderer
            .render_all(&sink, &mut buf)
            .expect("render_all on empty sink must not fail");
        assert!(
            buf.is_empty(),
            "empty sink must produce no output; got {} bytes",
            buf.len()
        );
    }
}
