//! Source span types for the slideforge DSL parser.
//!
//! Spans represent byte-offset ranges into a source file. They are used
//! throughout the AST to provide precise error locations for diagnostic
//! messages rendered via `miette`.
//!
//! # Design
//!
//! [`Span`] stores a `file_id: u32` (index into a [`SourceMap`]), plus
//! `start` and `end` byte offsets (exclusive) into the source string.
//! Byte offsets rather than `(line, col)` pairs are stored because:
//!
//! 1. Byte offsets are compact — two `usize` fields.
//! 2. Line/col can be computed on demand in O(log n) from a line-starts
//!    table; storing them redundantly would double the storage.
//! 3. `usize` avoids `f64` (forbidden by ADR-013) and stays `Hash`-safe.

use std::sync::Arc;

// ─── Span ────────────────────────────────────────────────────────────────────

/// A half-open byte-offset range `[start, end)` into a source file.
///
/// `file_id` is an index into the [`SourceMap`] that owns the file text.
/// `start` and `end` are byte offsets (not character or line indices).
///
/// All fields are `usize` — no `f64` per ADR-013.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Index of the source file in the owning [`SourceMap`].
    pub file_id: u32,
    /// Byte offset of the first byte of the span (inclusive).
    pub start: usize,
    /// Byte offset of the first byte *after* the span (exclusive).
    pub end: usize,
}

impl Span {
    /// Construct a new `Span`.
    #[must_use]
    pub fn new(file_id: u32, start: usize, end: usize) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }

    /// Return the byte length of this span.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Return `true` if the span covers zero bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

// ─── Spanned<T> ──────────────────────────────────────────────────────────────

/// A value `T` paired with the source [`Span`] that produced it.
///
/// All AST node types are wrapped in `Spanned<T>` so that error messages can
/// reference the exact source location of each parsed construct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T>(pub T, pub Span);

impl<T> Spanned<T> {
    /// Construct a new `Spanned` value.
    #[must_use]
    pub fn new(value: T, span: Span) -> Self {
        Self(value, span)
    }

    /// Return a reference to the inner value.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Return the span.
    #[must_use]
    pub fn span(&self) -> Span {
        self.1
    }

    /// Consume `self` and return `(value, span)`.
    #[must_use]
    pub fn into_parts(self) -> (T, Span) {
        (self.0, self.1)
    }
}

// ─── SourceFile ───────────────────────────────────────────────────────────────

/// A single source file: its path and raw text.
///
/// `SourceFile` is owned by a [`SourceMap`] and indexed by a `u32` file ID.
/// The `file_id` stored in each [`Span`] is the index of the owning
/// `SourceFile` within the map's internal vector.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Human-readable file path (e.g. `"deck.sf"` or `"<unknown>"`).
    pub path: Arc<str>,
    /// Full text of the source file.
    pub text: Arc<str>,
}

impl SourceFile {
    /// Construct a new `SourceFile`.
    #[must_use]
    pub fn new(path: Arc<str>, text: Arc<str>) -> Self {
        Self { path, text }
    }

    /// Convert a byte offset to a 1-based `(line, col)` pair.
    ///
    /// Uses a binary search over a pre-computed line-starts table — O(log n)
    /// per call, consistent with the lexer's implementation.
    #[must_use]
    pub fn offset_to_line_col(&self, offset: usize) -> (u32, u32) {
        let line_starts = self.compute_line_starts();
        let line_idx = line_starts
            .partition_point(|&s| s <= offset)
            .saturating_sub(1);
        let col = offset - line_starts[line_idx];
        #[allow(clippy::cast_possible_truncation)]
        let line = line_idx as u32 + 1;
        #[allow(clippy::cast_possible_truncation)]
        let col_u32 = col as u32 + 1;
        (line, col_u32)
    }

    /// Pre-compute the byte offset of the first character on each line.
    fn compute_line_starts(&self) -> Vec<usize> {
        std::iter::once(0)
            .chain(self.text.match_indices('\n').map(|(i, _)| i + 1))
            .collect()
    }
}

// ─── SourceMap ────────────────────────────────────────────────────────────────

/// A registry of all source files participating in a parse.
///
/// Each file is assigned a monotonically increasing `u32` ID. [`Span`] values
/// store this ID so they can be resolved back to file path + text on demand.
///
/// In the STORY-006 scope, a parse is always single-file, so the map usually
/// contains exactly one entry. Multi-file support (via `@import`) is a future
/// concern (STORY-008).
#[derive(Debug, Default)]
pub struct SourceMap {
    /// The registered source files, indexed by their assigned `file_id`.
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// Construct an empty `SourceMap`.
    #[must_use]
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add a source file and return its assigned `file_id`.
    pub fn add_file(&mut self, path: Arc<str>, text: Arc<str>) -> u32 {
        let id = self.files.len();
        self.files.push(SourceFile::new(path, text));
        #[allow(clippy::cast_possible_truncation)]
        {
            id as u32
        }
    }

    /// Look up a [`SourceFile`] by ID.
    ///
    /// Returns `None` if the ID is out of range.
    #[must_use]
    pub fn get(&self, file_id: u32) -> Option<&SourceFile> {
        self.files.get(file_id as usize)
    }

    /// Return the number of registered source files.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Return `true` if no files have been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_01_001_span_new_and_len() {
        let s = Span::new(0, 10, 20);
        assert_eq!(s.file_id, 0);
        assert_eq!(s.start, 10);
        assert_eq!(s.end, 20);
        assert_eq!(s.len(), 10);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_bc_1_01_001_span_empty() {
        let s = Span::new(0, 5, 5);
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_bc_1_01_001_spanned_derives() {
        let s = Spanned::new("hello", Span::new(0, 0, 5));
        let s2 = s.clone();
        assert_eq!(s, s2);
        let _ = format!("{s:?}");
        assert_eq!(s.value(), &"hello");
        assert_eq!(s.span(), Span::new(0, 0, 5));
    }

    #[test]
    fn test_bc_1_01_001_span_hash_eq() {
        use std::collections::HashSet;
        let a = Span::new(0, 0, 5);
        let b = Span::new(0, 0, 5);
        assert_eq!(a, b);
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn test_bc_1_01_001_source_map_add_and_get() {
        let mut sm = SourceMap::new();
        assert!(sm.is_empty());
        let id = sm.add_file(
            Arc::from("deck.sf"),
            Arc::from("slideforge_version \"1\"\n"),
        );
        assert_eq!(id, 0);
        assert_eq!(sm.len(), 1);
        let sf = sm.get(0).expect("file must exist");
        assert_eq!(sf.path.as_ref(), "deck.sf");
    }

    #[test]
    fn test_bc_1_01_001_source_file_offset_to_line_col() {
        let sf = SourceFile::new(Arc::from("x.sf"), Arc::from("hello\nworld\n"));
        // offset 0 → line 1, col 1
        assert_eq!(sf.offset_to_line_col(0), (1, 1));
        // offset 5 is '\n' → line 1, col 6
        assert_eq!(sf.offset_to_line_col(5), (1, 6));
        // offset 6 is 'w' (first char of "world") → line 2, col 1
        assert_eq!(sf.offset_to_line_col(6), (2, 1));
    }

    #[test]
    fn test_bc_1_01_001_source_map_multiple_files() {
        let mut sm = SourceMap::new();
        let id0 = sm.add_file(Arc::from("a.sf"), Arc::from("a"));
        let id1 = sm.add_file(Arc::from("b.sf"), Arc::from("b"));
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(sm.get(0).map(|f| f.path.as_ref()), Some("a.sf"));
        assert_eq!(sm.get(1).map(|f| f.path.as_ref()), Some("b.sf"));
        assert!(sm.get(2).is_none());
    }
}
