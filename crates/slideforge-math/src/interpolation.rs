//! `@{var}` interpolation for math expressions.
//!
//! Before the LaTeX parser runs, slideforge resolves `@{var_name}` references
//! within math source strings. This is distinct from `{{ var }}` text-mode
//! interpolation: `{{ }}` is disabled inside `$...$` and `$$...$$` blocks
//! (R3 finding), while `@{...}` is specifically for math-mode variable injection.
//!
//! ## Semantics
//!
//! - `@{x}` is replaced by the string representation of the variable `x` from
//!   the current evaluation context.
//! - `{{ arr }}` double-brace syntax is left as-is (literal) inside math mode
//!   and must NOT be treated as interpolation.
//! - If `@{missing_var}` references a variable not in scope, an
//!   [`MathRendererError::UndefinedVariable`] diagnostic is produced and the
//!   placeholder is replaced with a `?` sentinel for parse continuity.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::Arc;

use slideforge_types::SourceSpan;

use crate::error::{MathDiagnostic, MathRendererError};

/// Substitute all `@{var_name}` occurrences in `latex` with their values from
/// `vars`.
///
/// # Arguments
///
/// * `latex` — Raw LaTeX source potentially containing `@{var}` references.
/// * `vars`  — Map from variable name to string value (from evaluator context).
/// * `span`  — Source span of the enclosing math expression (for diagnostics).
///
/// # Returns
///
/// A tuple of `(substituted_source, Vec<MathDiagnostic>)`:
/// - The first element is the LaTeX string with all `@{...}` references replaced.
/// - The second element contains diagnostics for undefined variable references.
#[must_use]
pub fn substitute<S: BuildHasher>(
    latex: &str,
    vars: &HashMap<Arc<str>, Arc<str>, S>,
    span: &SourceSpan,
) -> (String, Vec<MathDiagnostic>) {
    let mut result = String::with_capacity(latex.len());
    let mut diags: Vec<MathDiagnostic> = Vec::new();
    let mut chars = latex.char_indices().peekable();

    while let Some((_i, ch)) = chars.next() {
        // Detect `@{` — start of math-mode interpolation.
        if ch == '@'
            && let Some((_, '{')) = chars.peek().copied()
        {
            // Consume the `{`
            chars.next();
            // Collect the variable name up to the matching `}`
            let mut var_name = String::new();
            let mut closed = false;
            for (_, vc) in chars.by_ref() {
                if vc == '}' {
                    closed = true;
                    break;
                }
                var_name.push(vc);
            }
            if closed {
                let key: Arc<str> = Arc::from(var_name.as_str());
                if let Some(value) = vars.get(&key) {
                    result.push_str(value);
                } else {
                    // Replace with `?` sentinel so parsing can continue
                    result.push('?');
                    diags.push(MathDiagnostic::new(
                        MathRendererError::UndefinedVariable {
                            var_name: key,
                            span: span.clone(),
                            error_code: "E-EVL-002",
                        },
                        span.clone(),
                    ));
                }
            } else {
                // Unclosed `@{` — emit as-is for best-effort recovery and
                // push a diagnostic so the user knows what went wrong.
                result.push('@');
                result.push('{');
                result.push_str(&var_name);
                diags.push(MathDiagnostic::new(
                    MathRendererError::ParseError {
                        message: Arc::from("unclosed `@{` interpolation expression"),
                        span: span.clone(),
                    },
                    span.clone(),
                ));
            }
            continue;
        }

        // Detect `{{` — double-brace text-mode syntax; leave as literal.
        if ch == '{'
            && let Some((_, '{')) = chars.peek().copied()
        {
            // Two consecutive `{` chars — pass them through verbatim.
            result.push('{');
            // We already peeked the second `{`. Consume it.
            chars.next();
            result.push('{');
            // Copy everything up to and including `}}`
            let mut inner = String::new();
            let mut found_close = false;
            while let Some((_, ic)) = chars.next() {
                if ic == '}' {
                    if let Some((_, '}')) = chars.peek().copied() {
                        chars.next();
                        result.push_str(&inner);
                        result.push('}');
                        result.push('}');
                        found_close = true;
                        break;
                    }
                    inner.push(ic);
                } else {
                    inner.push(ic);
                }
            }
            if !found_close {
                // Unclosed `{{` — emit collected inner verbatim
                result.push_str(&inner);
            }
            continue;
        }

        result.push(ch);
    }

    (result, diags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MathRendererError;

    fn vars_from(pairs: &[(&str, &str)]) -> HashMap<Arc<str>, Arc<str>> {
        pairs
            .iter()
            .map(|(k, v)| (Arc::from(*k), Arc::from(*v)))
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.001 — @{var} substitution in math expressions
    // ─────────────────────────────────────────────────────────────────────────

    /// A defined variable is substituted correctly.
    #[test]
    fn test_bc_1_10_001_at_var_substitution() {
        let vars = vars_from(&[("mean", "4.2")]);
        let (result, diags) = substitute(
            r"\bar{x} = @{mean}",
            &vars,
            &SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        assert_eq!(result, r"\bar{x} = 4.2");
    }

    /// A reference to an undefined variable produces an `UndefinedVariable` diagnostic.
    #[test]
    fn test_bc_1_10_001_at_var_undefined() {
        let vars = HashMap::new();
        let (_, diags) = substitute("@{missing}", &vars, &SourceSpan::default());
        assert!(!diags.is_empty(), "expected UndefinedVariable diagnostic");
        let has_undef = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::UndefinedVariable { var_name, .. }
                if var_name.as_ref() == "missing")
        });
        assert!(has_undef, "expected UndefinedVariable for 'missing', got: {diags:?}");
    }

    /// `{{ arr }}` double-brace syntax is NOT treated as interpolation in math mode.
    #[test]
    fn test_bc_1_10_001_double_brace_not_substituted() {
        let vars = vars_from(&[("arr", "replaced")]);
        let (result, diags) = substitute("{{ arr }}", &vars, &SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        // Double-brace must be left as literal — NOT replaced
        assert_eq!(result, "{{ arr }}", "double-brace must not be substituted in math mode");
    }

    /// Multiple `@{var}` references in one expression are all substituted.
    #[test]
    fn test_bc_1_10_001_multiple_at_vars() {
        let vars = vars_from(&[("a", "3"), ("b", "4")]);
        let (result, diags) = substitute(
            r"\sqrt{@{a}^2 + @{b}^2}",
            &vars,
            &SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        assert_eq!(result, r"\sqrt{3^2 + 4^2}");
    }

    /// An expression with no `@{...}` references is returned unchanged.
    #[test]
    fn test_bc_1_10_001_no_vars_unchanged() {
        let vars = HashMap::new();
        let latex = r"\frac{1}{2} + \sqrt{x}";
        let (result, diags) = substitute(latex, &vars, &SourceSpan::default());
        assert!(diags.is_empty());
        assert_eq!(result, latex);
    }

    /// An empty string is returned unchanged with no diagnostics.
    #[test]
    fn test_bc_1_10_001_empty_string() {
        let vars = HashMap::new();
        let (result, diags) = substitute("", &vars, &SourceSpan::default());
        assert!(diags.is_empty());
        assert_eq!(result, "");
    }

    /// An unclosed `@{` interpolation must emit a ParseError diagnostic
    /// whose message contains "unclosed".
    #[test]
    fn test_unclosed_at_brace_diagnostic() {
        let vars = HashMap::new();
        let (_, diags) = substitute("@{unclosed", &vars, &SourceSpan::default());
        assert!(!diags.is_empty(), "expected a diagnostic for unclosed @{{");
        let has_unclosed_diag = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::ParseError { message, .. }
                if message.contains("unclosed"))
        });
        assert!(
            has_unclosed_diag,
            "expected ParseError with 'unclosed' in message, got: {diags:?}"
        );
    }
}
