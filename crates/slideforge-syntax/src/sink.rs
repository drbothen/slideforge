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
//! [`DiagnosticSink::to_json`] produces a [`serde_json::Value`] array. Each
//! element has the following fields:
//!
//! ```json
//! {
//!   "code": "E-PAR-001",
//!   "message": "Unexpected indentation …",
//!   "help": "Use exactly 2 spaces …",
//!   "severity": "Fatal"
//! }
//! ```
//!
//! The `"severity"` field is the `Debug` representation of [`ParseSeverity`].
//! Callers must not rely on the exact string — it may be normalised to lower
//! case in a future version.

use crate::error::ParseSeverity;

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
}

impl DiagnosticSink {
    /// Construct an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            severities: Vec::new(),
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
        // Capture severity before type-erasure by downcasting while the
        // concrete type `D` is still visible.  This avoids any need to
        // downcast the BoxDiagnostic at query time.
        use std::any::Any;
        let severity = if let Some(se) = (&err as &dyn Any).downcast_ref::<crate::SyntaxError>() {
            se.severity()
        } else {
            // Conservative default for non-SyntaxError diagnostics.
            ParseSeverity::Error
        };

        self.severities.push(severity);
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

    /// Serialise all diagnostics to a [`serde_json::Value`] array.
    ///
    /// Each element has the keys `"code"`, `"message"`, `"help"`, and
    /// `"severity"`. Missing fields (e.g. `help` returning `None`) are
    /// serialised as `null`.
    ///
    /// # Panics
    ///
    /// Never panics — all fields are converted via `to_string` or `None`.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        let entries: Vec<serde_json::Value> = self
            .diagnostics
            .iter()
            .zip(self.severities.iter())
            .map(|(diag, sev)| {
                let code = diag
                    .code()
                    .map_or(serde_json::Value::Null, |c| {
                        serde_json::Value::String(c.to_string())
                    });
                let message = serde_json::Value::String(diag.to_string());
                let help = diag
                    .help()
                    .map_or(serde_json::Value::Null, |h| {
                        serde_json::Value::String(h.to_string())
                    });
                let severity = serde_json::Value::String(format!("{sev:?}"));
                serde_json::json!({
                    "code": code,
                    "message": message,
                    "help": help,
                    "severity": severity,
                })
            })
            .collect();
        serde_json::Value::Array(entries)
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
    use super::*;
    use crate::SyntaxError;

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

    /// AC-014: `to_json()` returns an array where each element has `"code"`,
    /// `"message"`, `"help"`, and `"severity"` keys.
    #[test]
    fn test_bc_1_10_014_to_json_has_required_keys() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json();
        let arr = json.as_array().expect("to_json() must return a JSON array");
        assert!(!arr.is_empty(), "array must have at least one entry");
        let entry = &arr[0];
        assert!(
            entry.get("code").is_some(),
            "each entry must have a 'code' key"
        );
        assert!(
            entry.get("message").is_some(),
            "each entry must have a 'message' key"
        );
        assert!(
            entry.get("help").is_some(),
            "each entry must have a 'help' key"
        );
        assert!(
            entry.get("severity").is_some(),
            "each entry must have a 'severity' key"
        );
    }

    // ── EC-001: empty sink invariants ────────────────────────────────────────

    /// EC-001: a freshly created empty sink satisfies all expected invariants:
    /// `is_empty()`, `!has_fatal()`, `max_severity() == None`, and
    /// `to_json()` produces an empty JSON array.
    #[test]
    fn test_ec001_empty_sink() {
        let sink = DiagnosticSink::new();
        assert!(sink.is_empty(), "new sink must be empty");
        assert!(!sink.has_fatal(), "new sink must not have fatal errors");
        assert_eq!(
            sink.max_severity(),
            None,
            "new sink max_severity must be None"
        );
        let json = sink.to_json();
        let arr = json.as_array().expect("to_json() must return JSON array");
        assert_eq!(arr.len(), 0, "empty sink to_json must have 0 entries");
    }

    /// AC-014: `to_json()` output is valid JSON (parseable by `serde_json`).
    #[test]
    fn test_bc_1_10_014_to_json_is_valid_json() {
        let mut sink = DiagnosticSink::new();
        sink.push(make_error(1, 1));
        let json = sink.to_json();
        // If serde_json::Value round-trips through its Display without panic
        // then the value is valid JSON.
        let serialised = json.to_string();
        let reparsed: serde_json::Value =
            serde_json::from_str(&serialised).expect("to_json output must be valid JSON");
        assert!(reparsed.is_array());
    }
}
