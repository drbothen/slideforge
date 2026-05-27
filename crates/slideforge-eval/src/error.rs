//! Evaluation error types for `slideforge-eval`.
//!
//! Every variant carries a [`slideforge_types::SourceSpan`] so that `miette`
//! can render colored source pointers in the terminal. Error codes follow the
//! taxonomy established in the project error taxonomy supplement:
//!
//! | Code       | Variant              |
//! |-----------|----------------------|
//! | E-EVL-001 | `UndefinedVariable`  |
//! | E-EVL-002 | (reserved — math context undefined) |
//! | E-EVL-003 | `TypeMismatch`       |
//! | E-EVL-004 | `FilterNotFound`     |
//! | E-PAR-004 | `IncludeCycle`       |
//! | E-DAT-005 | `FieldAccessFailed`  |
//! | E-PAR-006 | `ReservedKeyword`    |
//! | E-EVL-007 | `TooManySlides`      |
//! | E-EVL-008 | `NotIterable`        |
//! | E-EVL-009 | `LargeDeckWarning`   |

use std::sync::Arc;

use miette::Diagnostic;
use slideforge_types::SourceSpan;
use thiserror::Error;

// ─── Format helpers ──────────────────────────────────────────────────────────

/// Format a cycle path as `"a.sf → b.sf → a.sf"` for error messages.
///
/// This is used in the `#[error(...)]` attribute of [`EvalError::IncludeCycle`].
/// It must be a free function (not a method) because `thiserror`'s `#[error]`
/// macro can only call free functions in format expressions.
#[must_use]
pub fn format_cycle_path(cycle_path: &[Arc<str>]) -> String {
    cycle_path
        .iter()
        .map(std::convert::AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(" → ")
}

// ─── EvalError ───────────────────────────────────────────────────────────────

/// An error produced by the slideforge expression evaluator.
///
/// Each variant carries a [`SourceSpan`] for diagnostic rendering and
/// implements [`miette::Diagnostic`] with a stable error code.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum EvalError {
    /// E-EVL-001: A variable was referenced but is not defined in any
    /// scope frame visible from the current evaluation context.
    ///
    /// `scope_list` is a comma-separated summary of variables that ARE
    /// defined, to aid the user in finding typos.
    #[error("undefined variable `{name}` at {span}")]
    #[diagnostic(code("E-EVL-001"), help("Defined variables in scope: {scope_list}"))]
    UndefinedVariable {
        /// The name of the variable that was not found.
        name: Arc<str>,
        /// Comma-separated list of variables available in scope, for the
        /// diagnostic help text.
        scope_list: String,
        /// Source location of the variable reference.
        span: SourceSpan,
    },

    /// E-EVL-003: An operation received an operand of the wrong type.
    ///
    /// The `message` field contains the full human-readable description
    /// (e.g. `"expected int or float, got string"`).
    #[error("type mismatch at {span}: {message}")]
    #[diagnostic(code("E-EVL-003"))]
    TypeMismatch {
        /// Full description of the type conflict.
        message: String,
        /// Source location of the offending expression.
        span: SourceSpan,
    },

    /// E-EVL-004: A pipe expression referenced a filter function that does
    /// not exist in the built-in filter registry.
    ///
    /// `available` is a comma-separated list of the registered filter names.
    #[error("unknown filter `{name}` at {span}")]
    #[diagnostic(code("E-EVL-004"), help("Available filters: {available}"))]
    FilterNotFound {
        /// The filter name that was not found.
        name: Arc<str>,
        /// Comma-separated list of available filter names.
        available: String,
        /// Source location of the pipe expression.
        span: SourceSpan,
    },

    /// E-EVL-003 (division by zero): Integer or float division by zero was
    /// attempted. Classified under type error because the divisor has an
    /// invalid value for the requested operation.
    #[error("division by zero at {span}")]
    #[diagnostic(code("E-EVL-003"))]
    DivisionByZero {
        /// Source location of the division expression.
        span: SourceSpan,
    },

    /// E-DAT-005: A field access expression (`a.b`) failed because the field
    /// `field` does not exist on the value of type `parent_type`.
    #[error("field `{field}` not found on `{parent_type}` at {span}")]
    #[diagnostic(
        code("E-DAT-005"),
        help(
            "Check that the field name is spelled correctly and that the variable holds a map value"
        )
    )]
    FieldAccessFailed {
        /// The field name that was not found.
        field: Arc<str>,
        /// The type of the value on which field access was attempted.
        parent_type: Arc<str>,
        /// Source location of the field access expression.
        span: SourceSpan,
    },

    /// E-PAR-006: A reserved DSL keyword was used where a user-defined
    /// identifier is expected.
    ///
    /// Keywords such as `@while`, `@fn`, `@return`, `@class`, and `@import`
    /// are reserved for future DSL versions (Q1 decision). Using them in the
    /// current DSL produces this error.
    ///
    /// `hint` carries a correction suggestion (e.g., "Use @for instead of @while").
    #[error("reserved keyword `{keyword}` used at {span}")]
    #[diagnostic(code("E-PAR-006"), help("{hint}"))]
    ReservedKeyword {
        /// The reserved keyword that was encountered.
        keyword: Arc<str>,
        /// A human-readable correction hint.
        hint: String,
        /// Source location of the reserved keyword.
        span: SourceSpan,
    },

    /// E-EVL-007: The `@for` loop (or the overall deck) would produce more
    /// slides than the configured
    /// [`EvalConfig::max_total_slides`](crate::EvalConfig) hard cap.
    ///
    /// This error is only produced when `max_total_slides` is `Some(n)` and
    /// the limit is exceeded. It is NOT produced when the limit is `None`.
    #[error("slide count {count} exceeds the configured maximum of {max} at {span}")]
    #[diagnostic(
        code("E-EVL-007"),
        help("Reduce the collection size, or increase max_total_slides in EvalConfig")
    )]
    TooManySlides {
        /// The number of slides that would have been produced.
        count: usize,
        /// The configured maximum.
        max: usize,
        /// Source location of the `@for` expression or deck node that triggered the cap.
        span: SourceSpan,
    },

    /// E-EVL-008: The expression in `@for x in <expr>` evaluated to a value
    /// that is not iterable.
    ///
    /// Per BC-2.04.001: `List` and `Map` values are iterable. Scalars (`Int`,
    /// `Float`, `Bool`, `Str`) and `Null` are rejected with this error. For
    /// maps, each entry is bound as a two-field map `{ key, value }` inside
    /// the loop body.
    #[error("cannot iterate over `{value_type}` value at {span}: @for requires a list")]
    #[diagnostic(
        code("E-EVL-008"),
        help(
            "Wrap the value in a list literal `[{value_type}]` or ensure the variable holds a list"
        )
    )]
    NotIterable {
        /// The type name of the value that was not a list.
        value_type: Arc<str>,
        /// Source location of the collection expression.
        span: SourceSpan,
    },

    /// E-EVL-009: The generated slide count exceeds the lint warning threshold.
    ///
    /// This is a **warning**, not an error — evaluation continues normally. It
    /// is emitted when the deck produces more slides than
    /// [`EvalConfig::large_deck_warn_threshold`](crate::EvalConfig).
    ///
    /// Distinguished from [`EvalError::TooManySlides`] (which is a hard error)
    /// by using a friendlier message and `ParseSeverity::Warning` severity.
    #[error(
        "Large iteration: @for over {count} items may produce a very large deck. Consider filtering data at source."
    )]
    #[diagnostic(
        code("E-EVL-009"),
        help(
            "Consider splitting the deck into multiple files, or increase large_deck_warn_threshold in EvalConfig"
        )
    )]
    LargeDeckWarning {
        /// The actual number of slides generated.
        count: usize,
        /// The warning threshold that was exceeded.
        threshold: usize,
        /// Source location of the deck or block that triggered the warning.
        span: SourceSpan,
    },

    /// E-PAR-004: A circular `@include` chain was detected in the merged AST.
    ///
    /// The evaluator runs a DFS over the include graph (built from `@include`
    /// metadata preserved in the merged deck node) as a pre-pass before any
    /// expression evaluation begins (fail-closed: no partial evaluation of a
    /// cyclic deck).
    ///
    /// `cycle_path` contains the canonical file paths that form the cycle, in
    /// order: `["a.sf", "b.sf", "a.sf"]`. The last element repeats the first
    /// to make the cycle explicit in the error message.
    ///
    /// Format: `Include cycle detected: a.sf → b.sf → a.sf`
    #[error("Include cycle detected: {}", format_cycle_path(cycle_path))]
    #[diagnostic(
        code("E-PAR-004"),
        help("Remove the circular @include to break the cycle")
    )]
    IncludeCycle {
        /// The ordered list of file paths forming the cycle.
        ///
        /// The last element is the same as the first to make the cycle
        /// explicit: `["a.sf", "b.sf", "a.sf"]`.
        cycle_path: Vec<Arc<str>>,
        /// Source location of the `@include` directive that closed the cycle.
        span: SourceSpan,
    },
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_span() -> SourceSpan {
        SourceSpan::new(Arc::from("test.sf"), 1, 1, 0)
    }

    #[test]
    fn test_bc_2_01_001_eval_error_undefined_variable_constructible() {
        let e = EvalError::UndefinedVariable {
            name: Arc::from("foo"),
            scope_list: "a, b".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("foo"),
            "error message must mention the variable name"
        );
    }

    #[test]
    fn test_bc_2_01_001_eval_error_type_mismatch_constructible() {
        let e = EvalError::TypeMismatch {
            message: "expected int, got string".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("type mismatch"),
            "error message must say 'type mismatch'"
        );
    }

    #[test]
    fn test_bc_2_01_001_eval_error_filter_not_found_constructible() {
        let e = EvalError::FilterNotFound {
            name: Arc::from("bogus"),
            available: "upper, lower".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("bogus"),
            "error message must mention the filter name"
        );
    }

    #[test]
    fn test_bc_2_01_001_eval_error_division_by_zero_constructible() {
        let e = EvalError::DivisionByZero { span: test_span() };
        let msg = format!("{e}");
        assert!(
            msg.contains("division by zero"),
            "error message must mention division by zero"
        );
    }

    #[test]
    fn test_bc_2_01_001_eval_error_field_access_failed_constructible() {
        let e = EvalError::FieldAccessFailed {
            field: Arc::from("price"),
            parent_type: Arc::from("string"),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("price"),
            "error message must mention the field name"
        );
    }

    #[test]
    fn test_bc_2_01_001_eval_error_diagnostic_codes() {
        use miette::Diagnostic;

        let e001 = EvalError::UndefinedVariable {
            name: Arc::from("x"),
            scope_list: String::new(),
            span: test_span(),
        };
        let code = e001.code().unwrap().to_string();
        assert_eq!(
            code, "E-EVL-001",
            "UndefinedVariable must have code E-EVL-001"
        );

        let e003 = EvalError::TypeMismatch {
            message: String::new(),
            span: test_span(),
        };
        let code = e003.code().unwrap().to_string();
        assert_eq!(code, "E-EVL-003", "TypeMismatch must have code E-EVL-003");

        let e004 = EvalError::FilterNotFound {
            name: Arc::from("x"),
            available: String::new(),
            span: test_span(),
        };
        let code = e004.code().unwrap().to_string();
        assert_eq!(code, "E-EVL-004", "FilterNotFound must have code E-EVL-004");

        let e003_div = EvalError::DivisionByZero { span: test_span() };
        let code = e003_div.code().unwrap().to_string();
        assert_eq!(
            code, "E-EVL-003",
            "DivisionByZero must have code E-EVL-003 (type error in expression)"
        );

        let e_dat005 = EvalError::FieldAccessFailed {
            field: Arc::from("x"),
            parent_type: Arc::from("string"),
            span: test_span(),
        };
        let code = e_dat005.code().unwrap().to_string();
        assert_eq!(
            code, "E-DAT-005",
            "FieldAccessFailed must have code E-DAT-005"
        );

        let e_par006 = EvalError::ReservedKeyword {
            keyword: Arc::from("while"),
            hint: "Use @for instead".to_string(),
            span: test_span(),
        };
        let code = e_par006.code().unwrap().to_string();
        assert_eq!(
            code, "E-PAR-006",
            "ReservedKeyword must have code E-PAR-006"
        );

        let e_evl007 = EvalError::TooManySlides {
            count: 1001,
            max: 1000,
            span: test_span(),
        };
        let code = e_evl007.code().unwrap().to_string();
        assert_eq!(code, "E-EVL-007", "TooManySlides must have code E-EVL-007");

        let e_evl008 = EvalError::NotIterable {
            value_type: Arc::from("string"),
            span: test_span(),
        };
        let code = e_evl008.code().unwrap().to_string();
        assert_eq!(code, "E-EVL-008", "NotIterable must have code E-EVL-008");

        let e_par004 = EvalError::IncludeCycle {
            cycle_path: vec![Arc::from("a.sf"), Arc::from("b.sf"), Arc::from("a.sf")],
            span: test_span(),
        };
        let code = e_par004.code().unwrap().to_string();
        assert_eq!(code, "E-PAR-004", "IncludeCycle must have code E-PAR-004");
    }

    #[test]
    fn test_bc_2_12_001_reserved_keyword_message_contains_keyword() {
        let e = EvalError::ReservedKeyword {
            keyword: Arc::from("while"),
            hint: "Use @for instead of @while".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("while"),
            "ReservedKeyword message must mention the keyword; got: {msg}"
        );
    }

    #[test]
    fn test_bc_2_12_001_not_iterable_message_contains_type() {
        let e = EvalError::NotIterable {
            value_type: Arc::from("string"),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("string"),
            "NotIterable message must mention the type name; got: {msg}"
        );
    }

    #[test]
    fn test_bc_2_12_001_too_many_slides_message_contains_counts() {
        let e = EvalError::TooManySlides {
            count: 999,
            max: 500,
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("999"),
            "TooManySlides message must mention count; got: {msg}"
        );
        assert!(
            msg.contains("500"),
            "TooManySlides message must mention max; got: {msg}"
        );
    }

    /// I04: `LargeDeckWarning` is a distinct variant from `TooManySlides`.
    ///
    /// `TooManySlides` = hard error (cap exceeded), `LargeDeckWarning` = lint warning.
    #[test]
    fn test_i04_large_deck_warning_distinct_from_too_many_slides() {
        use miette::Diagnostic;

        let warn = EvalError::LargeDeckWarning {
            count: 600,
            threshold: 500,
            span: test_span(),
        };
        let code = warn.code().unwrap().to_string();
        assert_eq!(
            code, "E-EVL-009",
            "LargeDeckWarning must have code E-EVL-009"
        );

        let msg = format!("{warn}");
        assert!(
            msg.contains("600"),
            "LargeDeckWarning message must mention count; got: {msg}"
        );
        assert!(
            msg.contains("Large iteration"),
            "LargeDeckWarning message must contain 'Large iteration' per AC-011; got: {msg}"
        );
        assert!(
            msg.contains("filtering data at source"),
            "LargeDeckWarning message must contain AC-011 guidance text; got: {msg}"
        );
    }

    #[test]
    fn test_i04_large_deck_warning_constructible() {
        let e = EvalError::LargeDeckWarning {
            count: 600,
            threshold: 500,
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(
            !msg.is_empty(),
            "LargeDeckWarning must produce a non-empty message"
        );
    }
}
