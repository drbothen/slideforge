//! Source location spans — carried by all IR nodes for error reporting.
//!
//! Every parse error, validation warning, and evaluation error in slideforge
//! carries a [`SourceSpan`] so that `miette` can render colored source
//! pointers in the terminal.

use std::sync::Arc;

/// A location range within a source file.
///
/// All parser-produced IR nodes carry a `SourceSpan` so that downstream
/// errors (layout overflow, validation failures, type errors) can point back
/// to the exact location in the `.sf` source file.
///
/// ## Defaults
///
/// `SourceSpan::default()` represents an unknown / synthetic location. Use it
/// for IR nodes generated programmatically (e.g., auto-generated TOC slides).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SourceSpan {
    /// The source file path, or `"<unknown>"` for synthetic nodes.
    pub file: Arc<str>,

    /// One-based line number of the start of the span.
    pub line: u32,

    /// One-based column number (UTF-8 character offset) of the start.
    pub col: u32,

    /// Byte offset from the start of the file to the beginning of the span.
    pub byte_offset: usize,
}

impl SourceSpan {
    /// Construct a new `SourceSpan` at the given file / line / col / byte offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::SourceSpan;
    /// use std::sync::Arc;
    /// let span = SourceSpan::new(Arc::from("deck.sf"), 10, 3, 150);
    /// assert_eq!(span.line, 10);
    /// assert_eq!(span.col, 3);
    /// ```
    #[must_use]
    pub fn new(file: Arc<str>, line: u32, col: u32, byte_offset: usize) -> Self {
        SourceSpan {
            file,
            line,
            col,
            byte_offset,
        }
    }

    /// Return `true` if this span represents an unknown / synthetic / unresolved location.
    ///
    /// Returns `true` for three cases:
    /// 1. Empty file string — the default `SourceSpan::default()` state.
    /// 2. `"<unknown>"` — the conventional explicit unknown sentinel.
    /// 3. `"<byte:N>"` — the unresolved byte-offset sentinel produced by
    ///    `span_to_source_span` in `slideforge-eval` when no `SourceMap` is
    ///    available. These strings look like real file names but are NOT —
    ///    any diagnostic renderer that would display them to the user MUST
    ///    treat them as unresolved (F-094-P4-006 invariant: no fake filenames
    ///    in user-facing output).
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::SourceSpan;
    /// use std::sync::Arc;
    /// assert!(SourceSpan::default().is_unknown());
    /// assert!(SourceSpan { file: Arc::from("<byte:42>"), line: 0, col: 0, byte_offset: 42 }.is_unknown());
    /// assert!(!SourceSpan::new(Arc::from("deck.sf"), 4, 3, 42).is_unknown());
    /// ```
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.file.is_empty() || self.file.as_ref() == "<unknown>" || self.file.starts_with("<byte:")
    }
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_unknown() {
            write!(f, "<unknown>")
        } else {
            write!(f, "{}:{}:{}", self.file, self.line, self.col)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // AC-009 — SourceSpan fields correct, implements Hash+Eq+Clone+Debug+Default
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_01_009_source_span_default() {
        let span = SourceSpan::default();
        assert_eq!(span.line, 0);
        assert_eq!(span.col, 0);
        assert_eq!(span.byte_offset, 0);
    }

    #[test]
    fn test_bc_1_01_009_source_span_default_is_unknown() {
        assert!(SourceSpan::default().is_unknown());
    }

    #[test]
    fn test_bc_1_01_009_source_span_new() {
        let span = SourceSpan::new(Arc::from("deck.sf"), 10, 3, 150);
        assert_eq!(span.file.as_ref(), "deck.sf");
        assert_eq!(span.line, 10);
        assert_eq!(span.col, 3);
        assert_eq!(span.byte_offset, 150);
    }

    #[test]
    fn test_bc_1_01_009_source_span_clone() {
        let span = SourceSpan::new(Arc::from("a.sf"), 1, 1, 0);
        let span2 = span.clone();
        assert_eq!(span, span2);
    }

    #[test]
    fn test_bc_1_01_009_source_span_hash() {
        use std::collections::HashMap;
        let span = SourceSpan::new(Arc::from("x.sf"), 1, 1, 0);
        let mut map: HashMap<SourceSpan, &str> = HashMap::new();
        map.insert(span.clone(), "span");
        assert_eq!(map[&span], "span");
    }

    #[test]
    fn test_bc_1_01_009_source_span_eq() {
        let a = SourceSpan::new(Arc::from("a.sf"), 5, 2, 100);
        let b = SourceSpan::new(Arc::from("a.sf"), 5, 2, 100);
        assert_eq!(a, b);
    }

    #[test]
    fn test_bc_1_01_009_source_span_ne() {
        let a = SourceSpan::new(Arc::from("a.sf"), 5, 2, 100);
        let b = SourceSpan::new(Arc::from("a.sf"), 5, 3, 101);
        assert_ne!(a, b);
    }

    #[test]
    fn test_bc_1_01_009_source_span_debug() {
        let span = SourceSpan::new(Arc::from("deck.sf"), 1, 1, 0);
        let s = format!("{span:?}");
        assert!(s.contains("SourceSpan"));
    }

    #[test]
    fn test_bc_1_01_009_source_span_display_unknown() {
        let span = SourceSpan::default();
        assert_eq!(span.to_string(), "<unknown>");
    }

    #[test]
    fn test_bc_1_01_009_source_span_display_known() {
        let span = SourceSpan::new(Arc::from("deck.sf"), 10, 3, 150);
        assert_eq!(span.to_string(), "deck.sf:10:3");
    }

    // ── F-094-P4-006 — `<byte:N>` sentinel treated as unresolved by is_unknown() ──

    /// F-094-P4-006: A span with `file = "<byte:42>"` is an UNRESOLVED sentinel
    /// (produced by `span_to_source_span` when `SourceMap` is not available).
    /// `is_unknown()` must return `true` for these sentinels so no future diagnostic
    /// renderer can display a fake filename to the user.
    ///
    /// RED: before the fix `is_unknown()` only checks for `""` and `"<unknown>"`,
    /// so `"<byte:42>"` is treated as a real filename → `is_unknown()` returns `false`.
    #[test]
    fn test_f094_p4_006_byte_sentinel_is_unknown() {
        // Sentinel produced by old span_to_source_span — must be treated as unresolved.
        let byte_sentinel = SourceSpan {
            file: Arc::from("<byte:42>"),
            line: 0,
            col: 0,
            byte_offset: 42,
        };
        assert!(
            byte_sentinel.is_unknown(),
            "F-094-P4-006: span with file='<byte:42>' must be is_unknown()=true; \
             got false — the byte-sentinel is not a real file name"
        );

        // Variant with a different byte offset — same expectation.
        let byte_zero = SourceSpan {
            file: Arc::from("<byte:0>"),
            line: 0,
            col: 0,
            byte_offset: 0,
        };
        assert!(
            byte_zero.is_unknown(),
            "F-094-P4-006: span with file='<byte:0>' must be is_unknown()=true"
        );

        // A real file name must still be NOT unknown.
        let real = SourceSpan::new(Arc::from("deck.sf"), 4, 3, 42);
        assert!(
            !real.is_unknown(),
            "F-094-P4-006: span with file='deck.sf' must be is_unknown()=false"
        );
    }
}
