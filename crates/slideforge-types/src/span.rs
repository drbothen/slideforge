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

    /// Return `true` if this span represents an unknown / synthetic location.
    ///
    /// # Examples
    ///
    /// ```
    /// use slideforge_types::SourceSpan;
    /// assert!(SourceSpan::default().is_unknown());
    /// ```
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.file.is_empty() || self.file.as_ref() == "<unknown>"
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
}
