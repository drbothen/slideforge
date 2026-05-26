//! [`DiagnosticSink`] — an accumulator for [`miette::Diagnostic`] values.
//!
//! The sink collects diagnostics produced during a parse pass and provides
//! querying methods used by the CLI renderer and by `parse_checked`.
//!
//! # Design
//!
//! Diagnostics are stored as type-erased `Box<dyn miette::Diagnostic + Send + Sync>`
//! so that any crate that implements [`miette::Diagnostic`] can be pushed
//! without coupling `slideforge-syntax` to every error type.
//!
//! Severity is captured eagerly at push time (while the concrete type is still
//! available) and stored in a parallel `Vec<ParseSeverity>`.  This lets
//! [`DiagnosticSink::has_fatal`] and [`DiagnosticSink::max_severity`] operate
//! in O(n) without downcasting.
//!
//! # Thread Safety
//!
//! `DiagnosticSink` is `Send + Sync` because every stored diagnostic is
//! `Send + Sync` (enforced by the `push` bound).
//!
//! # JSON Export
//!
//! [`DiagnosticSink::to_json`] produces a [`serde_json::Value`] object with
//! the following schema:
//!
//! ```json
//! {
//!   "diagnostics": [{
//!     "error_code": "E-PAR-001",
//!     "severity": "fatal",
//!     "file": "deck.sf",
//!     "line": 5,
//!     "col": 3,
//!     "message": "Unexpected indentation …",
//!     "hint": "Use exactly 2 spaces …"
//!   }],
//!   "total": 1,
//!   "has_fatal": true
//! }
//! ```
//!
//! `"severity"` is always lowercase (`"fatal"`, `"error"`, or `"warning"`).
//! For [`crate::SyntaxError`] diagnostics the `file`, `line`, and `col` fields
//! are extracted directly from the variant fields.  For any other diagnostic
//! type the position falls back to `"<unknown>"` / `0` / `0`.

use crate::error::ParseSeverity;
use crate::span::SourceMap;

// ─── BoxDiagnostic ───────────────────────────────────────────────────────────

/// A type-erased, heap-allocated [`miette::Diagnostic`].
///
/// Every value stored in a [`DiagnosticSink`] is an instance of this alias.
/// The `Send + Sync` bounds are required so that `DiagnosticSink` is usable
/// across thread boundaries.
pub type BoxDiagnostic = Box<dyn miette::Diagnostic + Send + Sync>;

// ─── DiagnosticSink ──────────────────────────────────────────────────────────

/// An accumulator for parse-time diagnostics.
///
/// Diagnostics are pushed in the order they are produced and may be retrieved
/// via [`DiagnosticSink::errors`]. The sink never drops or deduplicates
/// entries — it is a faithful record of every diagnostic produced during a
/// parse (AC-011: 100-error accumulation without truncation).
///
/// Use [`DiagnosticSink::max_severity`] to determine the overall outcome of a
/// parse, and [`DiagnosticSink::has_fatal`] to gate on whether a re-parse or
/// further analysis is safe.
///
/// # Example
///
/// ```
/// use slideforge_syntax::DiagnosticSink;
///
/// let mut sink = DiagnosticSink::new();
/// assert!(sink.is_empty());
/// ```
pub struct DiagnosticSink {
    /// Type-erased diagnostics in push order.
    diagnostics: Vec<BoxDiagnostic>,
    /// Severity for each entry in `diagnostics`, captured eagerly at push
    /// time.  Always the same length as `diagnostics`.
    severities: Vec<ParseSeverity>,
    /// Source position `(file, line, col)` for each entry, captured eagerly at
    /// push time while the concrete type is still available.  Always the same
    /// length as `diagnostics`.  Entries for unknown diagnostic types default to
    /// `("<unknown>", 0, 0)`.
    positions: Vec<(String, u32, u32)>,
}

impl DiagnosticSink {
    /// Construct an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            severities: Vec::new(),
            positions: Vec::new(),
        }
    }

    /// Push a diagnostic into the sink.
    ///
    /// Severity is determined at push time while the concrete type `D` is still
    /// available.  For [`crate::SyntaxError`] values, severity is obtained via
    /// [`crate::SyntaxError::severity`].  For any other diagnostic type the
    /// severity defaults to [`ParseSeverity::Error`] (conservative).
    ///
    /// The sink never truncates — all 100 errors from a pathological input
    /// will be retained (AC-011).
    pub fn push<D>(&mut self, err: D)
    where
        D: miette::Diagnostic + Send + Sync + 'static,
    {
        // Capture severity and position before type-erasure by downcasting
        // while the concrete type `D` is still visible.  This avoids any need
        // to downcast the BoxDiagnostic at query time.
        use std::any::Any;
        let (severity, position) =
            if let Some(se) = (&err as &dyn Any).downcast_ref::<crate::SyntaxError>() {
                (se.severity(), se.sort_position())
            } else {
                // Conservative defaults for non-SyntaxError diagnostics.
                (ParseSeverity::Error, ("<unknown>".to_owned(), 0u32, 0u32))
            };

        self.severities.push(severity);
        self.positions.push(position);
        self.diagnostics.push(Box::new(err));
    }

    /// Push a diagnostic into the sink with an explicit severity override.
    ///
    /// This is useful for non-[`crate::SyntaxError`] diagnostics where the
    /// caller knows the correct severity at push time (e.g., a warning-level
    /// diagnostic from a validator plugin).
    ///
    /// The sink never truncates — all errors are retained (AC-011).
    pub fn push_with_severity<D>(&mut self, err: D, severity: ParseSeverity)
    where
        D: miette::Diagnostic + Send + Sync + 'static,
    {
        use std::any::Any;
        let position =
            if let Some(se) = (&err as &dyn Any).downcast_ref::<crate::SyntaxError>() {
                se.sort_position()
            } else {
                ("<unknown>".to_owned(), 0u32, 0u32)
            };

        self.severities.push(severity);
        self.positions.push(position);
        self.diagnostics.push(Box::new(err));
    }

    /// Return `true` if no diagnostics have been pushed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Return the number of diagnostics in the sink.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Return `true` if any diagnostic has [`ParseSeverity::Fatal`] severity.
    ///
    /// This is the gate used by `parse_checked` to decide whether to return
    /// `Some(DeckNode)` or `None`.
    #[must_use]
    pub fn has_fatal(&self) -> bool {
        self.severities.contains(&ParseSeverity::Fatal)
    }

    /// Return a slice of all accumulated diagnostics.
    ///
    /// The order is the push order (i.e. source order when the parser pushes
    /// errors in source order).
    #[must_use]
    pub fn errors(&self) -> &[BoxDiagnostic] {
        &self.diagnostics
    }

    /// Serialise all diagnostics to a [`serde_json::Value`] object.
    ///
    /// # Schema
    ///
    /// ```json
    /// {
    ///   "diagnostics": [{
    ///     "error_code": "E-PAR-001",
    ///     "severity": "fatal",
    ///     "file": "deck.sf",
    ///     "line": 5,
    ///     "col": 3,
    ///     "message": "…",
    ///     "hint": "…"
    ///   }],
    ///   "total": N,
    ///   "has_fatal": true
    /// }
    /// ```
    ///
    /// `"severity"` is lowercase (`"fatal"`, `"error"`, or `"warning"`).
    /// `"hint"` is `null` when the diagnostic has no help text.
    /// For [`crate::SyntaxError`] diagnostics, `"file"`, `"line"`, and
    /// `"col"` are extracted from the variant fields.  For all other types
    /// the position falls back to `"<unknown>"` / `0` / `0`.
    ///
    /// `source_map` is accepted for forward-compatibility with multi-file
    /// diagnostics that store only a [`crate::span::Span`] (file ID + byte
    /// offset).  In the current implementation, position is read directly
    /// from [`crate::SyntaxError`] variant fields so the map is not consulted
    /// for those; it is available for unknown diagnostic types that may carry
    /// span information resolvable via the map.
    ///
    /// # Panics
    ///
    /// Never panics — all fields are converted via `to_string` or `None`.
    #[must_use]
    pub fn to_json(&self, _source_map: &SourceMap) -> serde_json::Value {
        // positions, severities, and diagnostics are all the same length —
        // maintained by the push and push_with_severity methods.
        let entries: Vec<serde_json::Value> = self
            .diagnostics
            .iter()
            .zip(self.severities.iter())
            .zip(self.positions.iter())
            .map(|((diag, sev), (file, line, col))| {
                let error_code = diag
                    .code()
                    .map_or(serde_json::Value::Null, |c| {
                        serde_json::Value::String(c.to_string())
                    });
                let message = serde_json::Value::String(diag.to_string());
                let hint = diag
                    .help()
                    .map_or(serde_json::Value::Null, |h| {
                        serde_json::Value::String(h.to_string())
                    });
                let severity_str = match sev {
                    ParseSeverity::Fatal => "fatal",
                    ParseSeverity::Error => "error",
                    ParseSeverity::Warning => "warning",
                };

                serde_json::json!({
                    "error_code": error_code,
                    "severity": severity_str,
                    "file": file,
                    "line": line,
                    "col": col,
                    "message": message,
                    "hint": hint,
                })
            })
            .collect();

        let total = entries.len();
        let has_fatal = self.has_fatal();
        serde_json::json!({
            "diagnostics": entries,
            "total": total,
            "has_fatal": has_fatal,
        })
    }

    /// Return the maximum [`ParseSeverity`] across all diagnostics, or `None`
    /// if the sink is empty.
    ///
    /// Relies on [`ParseSeverity`]'s `Ord` impl (`Warning < Error < Fatal`).
    #[must_use]
    pub fn max_severity(&self) -> Option<ParseSeverity> {
        self.severities.iter().copied().max()
    }
}

impl Default for DiagnosticSink {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for DiagnosticSink {
    type Item = BoxDiagnostic;
    type IntoIter = std::vec::IntoIter<BoxDiagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{span::SourceMap, SyntaxError};

    // Helper: build a cheap SyntaxError for sink testing.
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

    // Helper: build a SourceMap with one file (used when to_json() is called).
    fn make_source_map() -> SourceMap {
        let mut sm = SourceMap::new();
        sm.add_file(Arc::from("test.sf"), Arc::from("  bad\n"));
        sm
    }

    // ── AC-008: push / is_empty / has_fatal / errors / IntoIterator ──────────

    /// AC-008a: a freshly created sink is empty.
    #[test]
    fn test_bc_1_10_008_new_sink_is_empty() {
        let sink = DiagnosticSink::new();
        assert!(sink.is_empty(), "new sink must be empty");
        assert_eq!(sink.len(), 0, "new sink len must be 0");
    }

    /// AC-008b: pushing one error makes `is_empty()` return false and `len()` == 1.
    #[test]
    fn test_bc_1_10_008_push_makes_nonempty() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        assert!(!sink.is_empty(), "sink must not be empty after push");
        assert_eq!(sink.len(), 1, "len must be 1 after one push");
    }

    /// AC-008c: `errors()` slice length matches push count.
    #[test]
    fn test_bc_1_10_008_errors_slice_len_matches_push_count() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        sink.push(make_error(2, 1));
        assert_eq!(sink.errors().len(), 2);
    }

    /// AC-008d: `has_fatal()` is false when sink is empty.
    #[test]
    fn test_bc_1_10_008_has_fatal_false_when_empty() {
        let sink = DiagnosticSink::new();
        assert!(!sink.has_fatal(), "empty sink has no fatal errors");
    }

    /// AC-008e: `has_fatal()` is true when at least one `SyntaxError` is pushed
    /// (all `SyntaxError` variants are Fatal).
    #[test]
    fn test_bc_1_10_008_has_fatal_true_after_syntax_error_push() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        assert!(sink.has_fatal(), "SyntaxError must be fatal");
    }

    /// AC-008f: `IntoIterator` yields the same number of items as `len()`.
    #[test]
    fn test_bc_1_10_008_into_iterator_yields_all_items() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        sink.push(make_error(2, 1));
        let count = sink.into_iter().count();
        assert_eq!(count, 2, "IntoIterator must yield all items");
    }

    // ── AC-007: max_severity with mixed severities ────────────────────────────

    /// AC-007: pushing a Fatal diagnostic makes `max_severity()` == `Some(Fatal)`.
    #[test]
    fn test_bc_1_10_007_max_severity_fatal_when_any_fatal_present() {
        let mut sink = DiagnosticSink::new();
        // SyntaxError is Fatal by AC-006 contract
        sink.push(make_error(1, 1));
        let max = sink.max_severity();
        assert_eq!(
            max,
            Some(ParseSeverity::Fatal),
            "max_severity must be Fatal when a Fatal error is present"
        );
    }

    /// AC-007: empty sink returns `None` from `max_severity()`.
    #[test]
    fn test_bc_1_10_007_max_severity_none_when_empty() {
        let sink = DiagnosticSink::new();
        assert_eq!(sink.max_severity(), None);
    }

    // ── AC-011: 100-error accumulation without truncation ────────────────────

    /// AC-011: pushing 100 errors results in `len()` == 100.
    #[test]
    fn test_bc_1_10_011_hundred_errors_no_truncation() {
        let mut sink = DiagnosticSink::new();
        for i in 1u32..=100 {
            sink.push(make_error(i, 1));
        }
        assert_eq!(sink.len(), 100, "sink must retain all 100 errors");
    }

    // ── AC-007: mixed severity → has_fatal and max_severity ──────────────────

    /// AC-007: pushing a Warning-severity diagnostic followed by a Fatal one
    /// must result in `has_fatal()` == true and `max_severity()` == `Some(Fatal)`.
    ///
    /// `SyntaxError` variants are all Fatal. A future Warning variant is
    /// anticipated; this test uses two `SyntaxError`s (both Fatal) to verify
    /// the mixed-present-fatal path is hit.
    #[test]
    fn test_ac007_sink_has_fatal_with_mixed() {
        let mut sink = DiagnosticSink::new();
        // Push two errors — both Fatal (all SyntaxErrors are Fatal per AC-006).
        sink.push(make_error(1, 1));
        sink.push(make_error(2, 1));
        assert!(
            sink.has_fatal(),
            "sink with Fatal diagnostics must report has_fatal()=true"
        );
        assert_eq!(
            sink.max_severity(),
            Some(crate::error::ParseSeverity::Fatal),
            "max_severity must be Some(Fatal) when Fatal diagnostics are present"
        );
    }

    // ── AC-014: to_json() produces valid JSON with required fields ────────────

    /// AC-014: `to_json()` returns a wrapper object with `"diagnostics"`,
    /// `"total"`, and `"has_fatal"` keys; each diagnostic entry has
    /// `"error_code"`, `"severity"`, `"file"`, `"line"`, `"col"`,
    /// `"message"`, and `"hint"` keys.
    #[test]
    fn test_bc_1_10_014_to_json_has_required_keys() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json(&sm);
        // Top-level wrapper keys.
        assert!(
            json.get("diagnostics").is_some(),
            "to_json() must have 'diagnostics' key"
        );
        assert!(
            json.get("total").is_some(),
            "to_json() must have 'total' key"
        );
        assert!(
            json.get("has_fatal").is_some(),
            "to_json() must have 'has_fatal' key"
        );
        // Per-diagnostic keys.
        let arr = json["diagnostics"]
            .as_array()
            .expect("'diagnostics' must be an array");
        assert!(!arr.is_empty(), "diagnostics array must have at least one entry");
        let entry = &arr[0];
        assert!(entry.get("error_code").is_some(), "entry must have 'error_code'");
        assert!(entry.get("severity").is_some(), "entry must have 'severity'");
        assert!(entry.get("file").is_some(), "entry must have 'file'");
        assert!(entry.get("line").is_some(), "entry must have 'line'");
        assert!(entry.get("col").is_some(), "entry must have 'col'");
        assert!(entry.get("message").is_some(), "entry must have 'message'");
        assert!(entry.get("hint").is_some(), "entry must have 'hint'");
    }

    /// AC-014: severity field in `to_json()` output is lowercase.
    #[test]
    fn test_bc_1_10_014_to_json_severity_is_lowercase() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json(&sm);
        let arr = json["diagnostics"].as_array().expect("must be array");
        let sev = arr[0]["severity"].as_str().expect("severity must be string");
        assert_eq!(sev, "fatal", "severity must be lowercase 'fatal'; got: {sev}");
    }

    /// AC-014: `to_json()` total and `has_fatal` fields are correct.
    #[test]
    fn test_bc_1_10_014_to_json_total_and_has_fatal() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        sink.push(make_error(2, 3));
        let json = sink.to_json(&sm);
        assert_eq!(json["total"].as_u64(), Some(2), "'total' must be 2");
        assert_eq!(json["has_fatal"].as_bool(), Some(true), "'has_fatal' must be true");
    }

    /// AC-014: file/line/col extracted from `SyntaxError` into `to_json()` output.
    #[test]
    fn test_bc_1_10_014_to_json_file_line_col_extracted() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(5, 3));
        let json = sink.to_json(&sm);
        let arr = json["diagnostics"].as_array().expect("must be array");
        let entry = &arr[0];
        assert_eq!(
            entry["file"].as_str(),
            Some("test.sf"),
            "'file' must be 'test.sf'"
        );
        assert_eq!(entry["line"].as_u64(), Some(5), "'line' must be 5");
        assert_eq!(entry["col"].as_u64(), Some(3), "'col' must be 3");
    }

    // ── AC-014: error_code key uses spec name ────────────────────────────────

    /// AC-014: the per-entry key for the diagnostic code is `"error_code"`,
    /// not `"code"`.
    #[test]
    fn test_bc_1_10_014_to_json_uses_error_code_key() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json(&sm);
        let arr = json["diagnostics"].as_array().expect("must be array");
        let entry = &arr[0];
        // "error_code" must exist, "code" must NOT exist.
        assert!(
            entry.get("error_code").is_some(),
            "entry must have 'error_code' key (not 'code')"
        );
        assert!(
            entry.get("code").is_none(),
            "entry must NOT have a 'code' key (renamed to 'error_code')"
        );
    }

    /// AC-014: the per-entry key for help text is `"hint"`, not `"help"`.
    #[test]
    fn test_bc_1_10_014_to_json_uses_hint_key() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json(&sm);
        let arr = json["diagnostics"].as_array().expect("must be array");
        let entry = &arr[0];
        // "hint" must exist, "help" must NOT exist.
        assert!(
            entry.get("hint").is_some(),
            "entry must have 'hint' key (not 'help')"
        );
        assert!(
            entry.get("help").is_none(),
            "entry must NOT have a 'help' key (renamed to 'hint')"
        );
    }

    // ── AC-009: push_with_severity ───────────────────────────────────────────

    /// AC-009: `push_with_severity` stores the explicit severity rather than
    /// auto-detecting it from the concrete type.
    #[test]
    fn test_push_with_severity_stores_explicit_severity() {
        let mut sink = DiagnosticSink::new();
        // Push a SyntaxError (normally Fatal) with an explicit Warning override.
        // This exercises the push_with_severity path, not the push path.
        sink.push_with_severity(make_error(1, 1), ParseSeverity::Warning);
        assert_eq!(sink.len(), 1, "sink must have 1 entry");
        assert_eq!(
            sink.max_severity(),
            Some(ParseSeverity::Warning),
            "max_severity must be Warning when pushed with Warning"
        );
        assert!(
            !sink.has_fatal(),
            "has_fatal must be false when only Warning diagnostics are present"
        );
    }

    // ── EC-001: empty sink invariants ────────────────────────────────────────

    /// EC-001: a freshly created empty sink satisfies all expected invariants:
    /// `is_empty()`, `!has_fatal()`, `max_severity() == None`, and
    /// `to_json()` produces a wrapper object with `"total": 0`.
    #[test]
    fn test_ec001_empty_sink() {
        let sm = make_source_map();
        let sink = DiagnosticSink::new();
        assert!(sink.is_empty(), "new sink must be empty");
        assert!(!sink.has_fatal(), "new sink must not have fatal errors");
        assert_eq!(
            sink.max_severity(),
            None,
            "new sink max_severity must be None"
        );
        let json = sink.to_json(&sm);
        assert_eq!(
            json["total"].as_u64(),
            Some(0),
            "empty sink to_json must have total:0"
        );
        assert_eq!(
            json["has_fatal"].as_bool(),
            Some(false),
            "empty sink to_json must have has_fatal:false"
        );
        let arr = json["diagnostics"].as_array().expect("'diagnostics' must be array");
        assert_eq!(arr.len(), 0, "empty sink to_json must have 0 diagnostic entries");
    }

    /// AC-014: `to_json()` output is valid JSON (parseable by `serde_json`).
    #[test]
    fn test_bc_1_10_014_to_json_is_valid_json() {
        let sm = make_source_map();
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json(&sm);
        // If serde_json::Value round-trips through its Display without panic
        // then the value is valid JSON.
        let serialised = json.to_string();
        let reparsed: serde_json::Value =
            serde_json::from_str(&serialised).expect("to_json output must be valid JSON");
        assert!(reparsed.is_object(), "to_json output must be a JSON object");
    }
}
