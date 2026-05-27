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
//! | E-DAT-005 | `FieldAccessFailed`  |

use std::sync::Arc;

use miette::Diagnostic;
use slideforge_types::SourceSpan;
use thiserror::Error;

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
    #[diagnostic(
        code("E-EVL-001"),
        help("Defined variables in scope: {scope_list}")
    )]
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
    #[diagnostic(
        code("E-EVL-004"),
        help("Available filters: {available}")
    )]
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
        help("Check that the field name is spelled correctly and that the variable holds a map value")
    )]
    FieldAccessFailed {
        /// The field name that was not found.
        field: Arc<str>,
        /// The type of the value on which field access was attempted.
        parent_type: Arc<str>,
        /// Source location of the field access expression.
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
        assert!(msg.contains("foo"), "error message must mention the variable name");
    }

    #[test]
    fn test_bc_2_01_001_eval_error_type_mismatch_constructible() {
        let e = EvalError::TypeMismatch {
            message: "expected int, got string".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("type mismatch"), "error message must say 'type mismatch'");
    }

    #[test]
    fn test_bc_2_01_001_eval_error_filter_not_found_constructible() {
        let e = EvalError::FilterNotFound {
            name: Arc::from("bogus"),
            available: "upper, lower".to_string(),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("bogus"), "error message must mention the filter name");
    }

    #[test]
    fn test_bc_2_01_001_eval_error_division_by_zero_constructible() {
        let e = EvalError::DivisionByZero { span: test_span() };
        let msg = format!("{e}");
        assert!(msg.contains("division by zero"), "error message must mention division by zero");
    }

    #[test]
    fn test_bc_2_01_001_eval_error_field_access_failed_constructible() {
        let e = EvalError::FieldAccessFailed {
            field: Arc::from("price"),
            parent_type: Arc::from("string"),
            span: test_span(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("price"), "error message must mention the field name");
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
        assert_eq!(code, "E-EVL-001", "UndefinedVariable must have code E-EVL-001");

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
        assert_eq!(code, "E-EVL-003", "DivisionByZero must have code E-EVL-003 (type error in expression)");

        let e_dat005 = EvalError::FieldAccessFailed {
            field: Arc::from("x"),
            parent_type: Arc::from("string"),
            span: test_span(),
        };
        let code = e_dat005.code().unwrap().to_string();
        assert_eq!(code, "E-DAT-005", "FieldAccessFailed must have code E-DAT-005");
    }
}
