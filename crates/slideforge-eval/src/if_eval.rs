//! `@if/@elif/@else` conditional block evaluator.
//!
//! # Behavioral Contracts
//!
//! This module implements BC-1.05.001 and BC-1.05.002:
//!
//! - **BC-1.05.001** — `@if/@elif/@else` at slide, element, field, and section scope.
//! - **BC-1.05.002** — Conditional expression evaluates with the same type rules
//!   as `{{ expr }}` (no implicit truthiness; only `Value::Bool` is accepted as a
//!   condition; any other type is E-EVL-003).
//!
//! # AC-003: Scope Support Status
//!
//! AC-003 requires `@if` at four scopes. Current implementation status:
//!
//! | Scope   | Status    | Notes |
//! |---------|-----------|-------|
//! | **Slide** | Implemented | `BlockItem::If` at top level, dispatched by `eval_block_items` |
//! | **Element** | Implemented | `BlockItem::If` in `slide_node.inline_items`, dispatched by `eval_block_items` |
//! | **Section** | Implemented | `BlockItem::If` as a sibling of `BlockItem::Section`, same dispatch path as slide-scope |
//! | **Field** | Blocked on parser | Requires `ExprNode::IfExpr { condition, then_val, else_val }` in the AST (STORY-013 spec §"@if at Field Scope"). The slideforge-syntax parser (STORY-007 / STORY-008) does not yet emit `ExprNode::IfExpr` for `title @if is_draft: "DRAFT: ..." @else: "..."`. Field-scope `@if` evaluator code will be added when the parser ships the new AST variant — tracked in the STORY-007 parser control-flow story scope. |
//!
//! # Lazy Evaluation Invariant
//!
//! The evaluator MUST implement lazy branch evaluation: once a truthy branch is
//! found, all subsequent `@elif` conditions and the `@else` body are **never
//! evaluated**. This is required by BC-1.05.001 invariant 2 and AC-004 — an
//! undefined variable in an unevaluated branch must NOT produce E-EVL-001.
//!
//! # Type Check Rule
//!
//! After `eval_expr` returns `Some(value)`, the condition is checked:
//!
//! ```text
//! match value {
//!     Value::Bool(b) => Ok(b),          // Only Bool is a valid condition
//!     other => Err(E-EVL-003),          // All other types are a type error
//! }
//! ```
//!
//! `Value::Int(0)`, `Value::Int(1)`, `Value::Str("true")`, `Value::Null` are
//! all type errors — there is no implicit truthiness.

use std::collections::HashMap;
use std::sync::Arc;

use slideforge_syntax::error::ParseSeverity;
use slideforge_syntax::{DiagnosticSink, IfNode};
use slideforge_types::{Slide, SourceSpan, Value};

// ─── span_to_source_span ────────────────────────────────────────────────────

/// Convert a `slideforge_syntax::Span` to a `SourceSpan` carrying the byte
/// offset of the condition expression.
///
/// At eval time, the evaluator operates on the merged AST without a live
/// `SourceMap`. We can extract the `byte_offset` from the syntax span's
/// `start` field, but we cannot resolve the file path or line/col without
/// the `SourceMap`. We mark the file as `"<span:byte>"` with the byte offset
/// embedded so that error messages carry at least partial location info.
///
/// Full `<file>:<line>:<col>` resolution is deferred to when the evaluator
/// receives `SourceMap` context (a future story that threads `SourceMap` through
/// the pipeline). Until then this is better than a zero-origin default.
fn span_to_source_span(syntax_span: slideforge_syntax::span::Span) -> SourceSpan {
    if syntax_span.start == 0 && syntax_span.end == 0 {
        // Synthetic span from test helpers — return default (unknown).
        SourceSpan::default()
    } else {
        // Carry the byte offset; mark file as "<byte:N>" for triage.
        SourceSpan {
            file: Arc::from(format!("<byte:{}>", syntax_span.start).as_str()),
            line: 0,
            col: 0,
            byte_offset: syntax_span.start,
        }
    }
}

use crate::config::EvalConfig;
use crate::env::Env;
use crate::error::EvalError;
use crate::expr::eval_expr;
use crate::for_eval::eval_block_items;

// ─── eval_if_chain ───────────────────────────────────────────────────────────

/// Evaluate one `@if/@elif/@else` chain, returning the slides it generates.
///
/// # Algorithm
///
/// 1. Evaluate the `@if` condition using [`eval_expr`].
/// 2. If `None` (error already in `sink`) — return `vec![]`.
/// 3. If `Some(value)`: verify `value` is `Value::Bool` — if not, push
///    E-EVL-003 (`EvalError::TypeMismatch`) and return `vec![]`.
/// 4. If `Bool(true)`: evaluate the `@if` body and return the slides.
/// 5. If `Bool(false)`: try `@elif` branches in order (lazy — stop at
///    first truthy); if none are truthy, evaluate the `@else` body if present.
///
/// # Laziness guarantee
///
/// This function MUST NOT evaluate conditions of branches that are never
/// reached. In particular: if the `@if` condition is `true`, no `@elif`
/// or `@else` bodies are evaluated and no `@elif` conditions are tested.
/// If the first `@elif` is `true`, no further `@elif` or `@else` bodies
/// are evaluated.
///
/// # Parameters
///
/// - `env`: The current variable environment (read-only for condition eval;
///   `@if` does NOT push a new scope — unlike `@for`).
/// - `if_node`: The `@if/@elif/@else` AST node.
/// - `set_rule_defaults`: Set-rule defaults threaded through to `eval_slide_node`.
/// - `config`: Evaluation configuration.
/// - `sink`: Accumulates all diagnostics.
///
/// # Returns
///
/// A `Vec<Slide>` from the first truthy branch, or from the `@else` branch if
/// all conditions are false, or an empty `Vec` if there is no matching branch.
pub fn eval_if_chain<S: std::hash::BuildHasher>(
    env: &mut Env,
    if_node: &IfNode,
    set_rule_defaults: &HashMap<(Arc<str>, Arc<str>), Value, S>,
    config: &EvalConfig,
    sink: &mut DiagnosticSink,
) -> Vec<Slide> {
    // Step 1: evaluate the @if condition.
    // Extract the condition's syntax span for error reporting (FINDING-006).
    let if_span = span_to_source_span(if_node.condition.span());
    match eval_bool_condition(env, if_node.condition.value(), if_span, sink) {
        None => {
            // Evaluation failed or type error — error already in sink; return empty.
            vec![]
        },
        Some(true) => {
            // @if branch is truthy: evaluate and return its body. Lazy — skip all
            // @elif and @else.
            eval_block_items(env, &if_node.then_body, set_rule_defaults, config, sink)
        },
        Some(false) => {
            // @if branch is false: try @elif branches in order (lazy).
            for (elif_condition_spanned, elif_body) in &if_node.elif_branches {
                let elif_span = span_to_source_span(elif_condition_spanned.span());
                match eval_bool_condition(env, elif_condition_spanned.value(), elif_span, sink) {
                    None => {
                        // Error in this elif condition — already in sink.
                        // Per lazy evaluation: stop here (don't evaluate further branches).
                        return vec![];
                    },
                    Some(true) => {
                        // This @elif branch is truthy. Lazy — evaluate its body and return.
                        return eval_block_items(env, elif_body, set_rule_defaults, config, sink);
                    },
                    Some(false) => {
                        // This @elif was false — continue to next @elif (lazy).
                    },
                }
            }
            // All conditions were false: evaluate @else body if present, else empty.
            if let Some(else_body) = &if_node.else_body {
                eval_block_items(env, else_body, set_rule_defaults, config, sink)
            } else {
                vec![]
            }
        },
    }
}

// ─── eval_bool_condition ─────────────────────────────────────────────────────

/// Evaluate a condition expression and assert the result is `Value::Bool`.
///
/// Returns `Some(bool)` if the expression evaluates to a boolean, or `None`
/// if evaluation fails or the result is not a boolean (in which case an
/// E-EVL-003 error is pushed to `sink`).
///
/// This is the shared type-checking gate for all branch conditions in
/// `eval_if_chain`. It enforces the BC-1.05.002 rule that conditions must
/// evaluate to `Bool` — no implicit truthiness conversion.
fn eval_bool_condition(
    env: &Env,
    condition: &slideforge_syntax::Expr,
    span: SourceSpan,
    sink: &mut DiagnosticSink,
) -> Option<bool> {
    // Evaluate the expression. Returns None if evaluation itself fails (e.g.,
    // undefined variable — E-EVL-001 already pushed to sink).
    let value = eval_expr(env, condition, sink)?;

    // BC-1.05.002: Only Value::Bool is a valid condition type.
    // No implicit truthiness: Int(0), Int(1), Str("true"), Null are all errors.
    match value {
        Value::Bool(b) => Some(b),
        other => {
            sink.push_with_severity(
                EvalError::TypeMismatch {
                    message: format!(
                        "condition expression evaluated to {}, not Bool; \
                         use a comparison operator (==, !=, <, >, <=, >=) to produce a boolean",
                        other.type_name()
                    ),
                    span,
                },
                ParseSeverity::Error,
            );
            None
        },
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
#[allow(non_snake_case)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use indexmap::IndexMap;
    use slideforge_syntax::span::Span;
    use slideforge_syntax::{
        BlockItem, DiagnosticSink, Expr, FieldNode, FieldValue, IfNode, SlideNode, Spanned,
        TemplateChunk,
    };
    use slideforge_types::Value;

    use super::*;
    use crate::config::EvalConfig;
    use crate::env::Env;

    // ─── Test helpers ─────────────────────────────────────────────────────────

    fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    fn empty_env() -> Env {
        Env::new(IndexMap::new())
    }

    fn env_with(pairs: &[(&str, Value)]) -> Env {
        let mut vars = IndexMap::new();
        for (k, v) in pairs {
            vars.insert(Arc::from(*k), v.clone());
        }
        Env::new(vars)
    }

    fn default_config() -> EvalConfig {
        EvalConfig::default()
    }

    fn empty_defaults() -> HashMap<(Arc<str>, Arc<str>), Value> {
        HashMap::new()
    }

    fn minimal_slide_node(kind: &str) -> SlideNode {
        SlideNode {
            kind: Spanned::new(kind.to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![],
        }
    }

    fn slide_block_item(kind: &str) -> BlockItem {
        BlockItem::Slide(Spanned::new(minimal_slide_node(kind), dummy_span()))
    }

    fn make_slide_with_title(kind: &str, title: &str) -> BlockItem {
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal(title.to_string())]),
                dummy_span(),
            ),
        };
        BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new(kind.to_string(), dummy_span()),
                tags: vec![],
                fields: vec![title_field],
                inline_items: vec![],
            },
            dummy_span(),
        ))
    }

    fn simple_if_node(condition: Expr, then_body: Vec<BlockItem>) -> IfNode {
        IfNode {
            condition: Spanned::new(condition, dummy_span()),
            then_body,
            elif_branches: vec![],
            else_body: None,
        }
    }

    // ─── BC-1.05.001 postcondition 1: @if true → slide included ──────────────

    /// BC-1.05.001 postcondition 1 / AC-001:
    /// `@if true:` with a slide body → the slide is included in output.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_true_renders_block() {
        let mut env = empty_env();
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if true:
        //   slide content:
        let if_node = simple_if_node(Expr::Bool(true), vec![slide_block_item("content")]);

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "@if true: must produce exactly 1 slide; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "@if true: must not produce errors; got: {:?}",
            sink.errors()
        );
    }

    // ─── BC-1.05.001 postcondition: @if false → no output ───────────────────

    /// BC-1.05.001 / AC-002 (false branch):
    /// `@if false:` without an else branch → 0 slides, 0 errors.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_false_no_output() {
        let mut env = empty_env();
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if false:
        //   slide content:
        // (no @else)
        let if_node = simple_if_node(Expr::Bool(false), vec![slide_block_item("content")]);

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "@if false: with no else must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "@if false: with no else must produce 0 errors; got: {:?}",
            sink.errors()
        );
    }

    // ─── BC-1.05.001 postcondition 2: @if/@elif/@else — first branch truthy ──

    /// BC-1.05.001 postcondition 2 + invariant 1 / AC-002:
    /// first branch is true → only first branch rendered, others skipped.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_elif_else_first_truthy() {
        let mut env = env_with(&[("env", Value::Str(Arc::from("prod")))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if env == "prod":    → true (first branch)
        //   slide content: title "prod-slide"
        // @elif env == "staging":
        //   slide content: title "staging-slide"
        // @else:
        //   slide content: title "dev-slide"
        let condition_if = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("prod".to_string())),
        };
        let condition_elif = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("staging".to_string())),
        };
        let if_node = IfNode {
            condition: Spanned::new(condition_if, dummy_span()),
            then_body: vec![make_slide_with_title("content", "prod-slide")],
            elif_branches: vec![(
                Spanned::new(condition_elif, dummy_span()),
                vec![make_slide_with_title("content", "staging-slide")],
            )],
            else_body: Some(vec![make_slide_with_title("content", "dev-slide")]),
        };

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "first truthy branch must produce exactly 1 slide; got {}",
            slides.len()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("prod-slide"),
            "first truthy branch (@if) must render 'prod-slide'; got: {:?}",
            slides[0].title_str()
        );
        assert!(
            sink.is_empty(),
            "@if/@elif/@else with first truthy must produce no errors"
        );
    }

    // ─── BC-1.05.001 postcondition 3: second branch truthy ───────────────────

    /// BC-1.05.001 postcondition 3 / AC-002:
    /// first branch false, second (@elif) true → second branch rendered.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_elif_else_second_truthy() {
        let mut env = env_with(&[("env", Value::Str(Arc::from("staging")))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        let condition_if = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("prod".to_string())),
        };
        let condition_elif = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("staging".to_string())),
        };
        let if_node = IfNode {
            condition: Spanned::new(condition_if, dummy_span()),
            then_body: vec![make_slide_with_title("content", "prod-slide")],
            elif_branches: vec![(
                Spanned::new(condition_elif, dummy_span()),
                vec![make_slide_with_title("content", "staging-slide")],
            )],
            else_body: Some(vec![make_slide_with_title("content", "dev-slide")]),
        };

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "@elif truthy branch must produce 1 slide; got {}",
            slides.len()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("staging-slide"),
            "second truthy branch (@elif) must render 'staging-slide'; got: {:?}",
            slides[0].title_str()
        );
        assert!(
            sink.is_empty(),
            "@elif truthy branch must produce no errors"
        );
    }

    // ─── BC-1.05.001 postcondition: @else fallback ───────────────────────────

    /// BC-1.05.001 postcondition / EC-002:
    /// all conditions false → @else branch rendered.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_else_fallback() {
        let mut env = env_with(&[("env", Value::Str(Arc::from("dev")))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if env == "prod": ... false
        // @elif env == "staging": ... false
        // @else: → rendered
        let condition_if = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("prod".to_string())),
        };
        let condition_elif = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("env".to_string())),
            rhs: Box::new(Expr::Str("staging".to_string())),
        };
        let if_node = IfNode {
            condition: Spanned::new(condition_if, dummy_span()),
            then_body: vec![make_slide_with_title("content", "prod-slide")],
            elif_branches: vec![(
                Spanned::new(condition_elif, dummy_span()),
                vec![make_slide_with_title("content", "staging-slide")],
            )],
            else_body: Some(vec![make_slide_with_title("content", "dev-slide")]),
        };

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "@else fallback must produce 1 slide; got {}",
            slides.len()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("dev-slide"),
            "@else fallback must render 'dev-slide'; got: {:?}",
            slides[0].title_str()
        );
        assert!(sink.is_empty(), "@else fallback must produce no errors");
    }

    // ─── BC-1.05.002 postcondition 2: String condition → E-EVL-003 ──────────

    /// BC-1.05.002 postcondition 2 + postcondition 3 / AC-005:
    /// `flag = "true"` (string), `@if flag:` → E-EVL-003.
    ///
    /// Strings are NOT implicitly truthy. Using a string as a boolean condition
    /// is a type error per BC-1.05.002.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_002_if_string_condition_type_error() {
        let mut env = env_with(&[("flag", Value::Str(Arc::from("true")))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if flag:   (flag is a String, not Bool → type error)
        let if_node = simple_if_node(
            Expr::Ident("flag".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "String condition type error must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "String condition must push E-EVL-003 to sink"
        );
        // Verify it's a TypeMismatch error (E-EVL-003).
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-003",
            "error code must be E-EVL-003; got: {code}"
        );
        // Verify the message mentions the type name so the user knows what went wrong.
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("Str") || msg.contains("string"),
            "E-EVL-003 message for String condition must mention the type name ('Str' or 'string'); got: {msg}"
        );
    }

    // ─── BC-1.05.002 postcondition 2: Int condition → E-EVL-003 ─────────────

    /// BC-1.05.002 postcondition 2 / AC-005 (Int case):
    /// `n = 0` (integer), `@if n:` → E-EVL-003.
    ///
    /// Integers are NOT implicitly truthy. `0` is not `false`; `1` is not `true`.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_002_if_int_condition_type_error() {
        let mut env = env_with(&[("n", Value::Int(0))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if n:   (n is Int, not Bool → type error)
        let if_node = simple_if_node(
            Expr::Ident("n".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "Int condition type error must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "Int condition must push E-EVL-003 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-003",
            "error code must be E-EVL-003 for Int condition; got: {code}"
        );
        // Verify the message mentions the type name so the user knows what went wrong.
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("Int") || msg.contains("int"),
            "E-EVL-003 message for Int condition must mention the type name ('Int' or 'int'); got: {msg}"
        );
    }

    // ─── BC-1.05.002 postcondition 1: Bool condition valid ───────────────────

    /// BC-1.05.002 postcondition 1 / AC-006:
    /// `flag = true` (Bool), `@if flag:` → valid, block rendered.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_002_if_bool_condition_valid() {
        let mut env = env_with(&[("flag", Value::Bool(true))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if flag:   (flag is Bool(true) → valid)
        //   slide content: title "visible"
        let if_node = simple_if_node(
            Expr::Ident("flag".to_string()),
            vec![make_slide_with_title("content", "visible")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "Bool(true) condition must produce 1 slide; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "Bool condition must not produce errors; got: {:?}",
            sink.errors()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("visible"),
            "rendered slide title must be 'visible'"
        );
    }

    // ─── BC-1.05.001 invariant 2: lazy evaluation ────────────────────────────

    /// BC-1.05.001 invariant 2 / AC-004 (CRITICAL):
    /// Second branch condition has an undefined variable, but first branch is
    /// true. The undefined variable in the second (unevaluated) branch must
    /// NOT produce E-EVL-001.
    ///
    /// Lazy evaluation: once a truthy branch is found, all subsequent
    /// conditions must not be evaluated.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_lazy_evaluation() {
        // First condition: true (will match)
        // Second condition: `undefined_var == "x"` — if evaluated, would push E-EVL-001
        let mut env = empty_env(); // no variables defined
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if true:
        //   slide content: title "first"
        // @elif undefined_var == "x":   ← must NEVER be evaluated
        //   slide content: title "second"
        let condition_if = Expr::Bool(true);
        let condition_elif = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Eq,
            lhs: Box::new(Expr::Ident("undefined_var".to_string())),
            rhs: Box::new(Expr::Str("x".to_string())),
        };
        let if_node = IfNode {
            condition: Spanned::new(condition_if, dummy_span()),
            then_body: vec![make_slide_with_title("content", "first")],
            elif_branches: vec![(
                Spanned::new(condition_elif, dummy_span()),
                vec![make_slide_with_title("content", "second")],
            )],
            else_body: None,
        };

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "lazy eval: first truthy branch must produce 1 slide; got {}",
            slides.len()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("first"),
            "lazy eval: must render first branch 'first'"
        );
        assert!(
            sink.is_empty(),
            "lazy eval: unevaluated @elif with undefined_var must NOT push E-EVL-001; \
             got: {:?}",
            sink.errors()
        );
    }

    // ─── BC-1.05.002 postcondition 1: pipe filter in condition ───────────────

    /// BC-1.05.002 postcondition 1 / AC-007:
    /// `@if items | length > 0:` with a non-empty list → block rendered.
    ///
    /// Pipe filters work in conditional expressions. The final value of the
    /// pipe expression must be `Bool` (since `length > 0` is a comparison).
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_002_if_pipe_in_condition() {
        let mut env = env_with(&[(
            "items",
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        )]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if items | length > 0:
        //   slide content: title "has-items"
        //
        // `items | length` applies the `length` filter → Value::Int(3)
        // `3 > 0` → Value::Bool(true)
        let pipe_expr = Expr::Pipe {
            lhs: Box::new(Expr::Ident("items".to_string())),
            filter: "length".to_string(),
            args: vec![],
        };
        let condition = Expr::BinOp {
            op: slideforge_syntax::BinOpKind::Gt,
            lhs: Box::new(pipe_expr),
            rhs: Box::new(Expr::Num(0)),
        };
        let if_node = simple_if_node(
            condition,
            vec![make_slide_with_title("content", "has-items")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            1,
            "pipe condition `items | length > 0` must produce 1 slide; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "pipe condition must not produce errors; got: {:?}",
            sink.errors()
        );
        assert_eq!(
            slides[0].title_str(),
            Some("has-items"),
            "pipe condition slide must have title 'has-items'"
        );
    }

    // ─── BC-1.05.001 EC-005: undefined condition variable → E-EVL-001 ────────

    /// BC-1.05.001 edge case EC-005 / AC-008:
    /// `@if undefined_cond:` → E-EVL-001 (undefined variable), 0 slides.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_undefined_condition_var() {
        let mut env = empty_env();
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if undefined_cond:
        //   slide content:
        let if_node = simple_if_node(
            Expr::Ident("undefined_cond".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "undefined condition variable must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "undefined condition variable must push E-EVL-001 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-001",
            "error code must be E-EVL-001 for undefined variable; got: {code}"
        );
    }

    // ─── EC-001: @if at slide scope — atomic inclusion ────────────────────────

    /// EC-001: `@if` at slide scope wraps an entire slide block; if condition is
    /// false, the slide is excluded atomically (not partially rendered).
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_if_slide_scope_atomic_exclusion() {
        let mut env = empty_env();
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if false:
        //   slide title: title "Should Not Appear"
        let if_node = simple_if_node(
            Expr::Bool(false),
            vec![make_slide_with_title("title", "Should Not Appear")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "@if false: at slide scope must atomically exclude the slide; got {} slides",
            slides.len()
        );
        assert!(sink.is_empty(), "no errors expected for @if false:");
    }

    // ─── FINDING-008: EC-003 — @if nested inside @for ────────────────────────

    /// EC-003 / FINDING-008: `@for` body containing a `BlockItem::If` whose
    /// condition references the loop variable.
    ///
    /// For each iteration of @for, the @if condition is evaluated in the
    /// loop's inner scope (where the binding variable is set). Different
    /// iterations may render different branches.
    ///
    /// Scenario:
    ///   @for x in [1, 2, 3]:
    ///     @if x > 1:
    ///       slide content: title "gt1"
    ///     @else:
    ///       slide content: title "lte1"
    ///
    /// x=1: @if 1>1 → false → @else → "lte1"
    /// x=2: @if 2>1 → true  → @if  → "gt1"
    /// x=3: @if 3>1 → true  → @if  → "gt1"
    ///
    /// Expected output: 3 slides `["lte1", "gt1", "gt1"]`.
    #[test]
    fn test_BC_1_05_001_if_nested_in_for_ec003() {
        use slideforge_syntax::{BinOpKind, Expr, ForNode};

        let mut env = empty_env();
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // Build `@if x > 1: slide content: title "gt1" @else: slide content: title "lte1"`
        let condition = Expr::BinOp {
            op: BinOpKind::Gt,
            lhs: Box::new(Expr::Ident("x".to_string())),
            rhs: Box::new(Expr::Num(1)),
        };

        // Reuse the existing module-level helper: make_slide_with_title("content", title)
        let if_node = IfNode {
            condition: Spanned::new(condition, dummy_span()),
            then_body: vec![make_slide_with_title("content", "gt1")],
            elif_branches: vec![],
            else_body: Some(vec![make_slide_with_title("content", "lte1")]),
        };

        // Build @for x in [1, 2, 3]: containing the @if block
        let for_body = vec![BlockItem::If(Spanned::new(if_node, dummy_span()))];
        let for_node = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(
                Expr::List(vec![Expr::Num(1), Expr::Num(2), Expr::Num(3)]),
                dummy_span(),
            ),
            body: for_body,
        };

        let slides = crate::for_eval::eval_for_block(
            &mut env,
            for_node.binding.value(),
            for_node.collection.value(),
            &for_node.body,
            &empty_defaults(),
            &config,
            &mut sink,
        );

        assert_eq!(
            slides.len(),
            3,
            "@for [1,2,3] with @if x>1 must produce 3 slides; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "no errors expected for @if inside @for; got: {:?}",
            sink.errors()
        );
        // x=1 → @else → "lte1"
        assert_eq!(
            slides[0].title_str(),
            Some("lte1"),
            "x=1: @if(1>1=false) → @else → 'lte1'; got: {:?}",
            slides[0].title_str()
        );
        // x=2 → @if → "gt1"
        assert_eq!(
            slides[1].title_str(),
            Some("gt1"),
            "x=2: @if(2>1=true) → 'gt1'; got: {:?}",
            slides[1].title_str()
        );
        // x=3 → @if → "gt1"
        assert_eq!(
            slides[2].title_str(),
            Some("gt1"),
            "x=3: @if(3>1=true) → 'gt1'; got: {:?}",
            slides[2].title_str()
        );
    }

    // ─── FINDING-007: Value::Null condition → E-EVL-003 ──────────────────────

    /// FINDING-007: `@if v:` where `v = Value::Null` → E-EVL-003 (TypeMismatch).
    ///
    /// Null is not implicitly falsy — it is a type error like Int or Str.
    #[test]
    fn test_if_null_condition_type_error() {
        let mut env = env_with(&[("v", Value::Null)]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        let if_node = simple_if_node(
            Expr::Ident("v".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "Null condition must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "Null condition must push E-EVL-003 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-003",
            "error code must be E-EVL-003 for Null condition; got: {code}"
        );
    }

    // ─── FINDING-005: Value::List and Value::Map as condition → E-EVL-003 ──────

    /// FINDING-005: `@if items:` where `items = Value::List(...)` → E-EVL-003.
    ///
    /// Lists are not implicitly truthy — they are a type error like Int or Str.
    #[test]
    fn test_if_list_condition_type_error() {
        let mut env = env_with(&[("items", Value::List(vec![Value::Int(1), Value::Int(2)]))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if items:   (items is List, not Bool → type error)
        let if_node = simple_if_node(
            Expr::Ident("items".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "List condition must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "List condition must push E-EVL-003 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-003",
            "error code must be E-EVL-003 for List condition; got: {code}"
        );
        // The message must mention 'List' so the user knows which type caused the error.
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("List") || msg.contains("list"),
            "E-EVL-003 message must mention 'List'; got: {msg}"
        );
    }

    /// FINDING-005: `@if obj:` where `obj = Value::Map(...)` → E-EVL-003.
    ///
    /// Maps are not implicitly truthy — they are a type error like Int or Str.
    #[test]
    fn test_if_map_condition_type_error() {
        use slideforge_types::OrderedMap;
        let mut map = OrderedMap::new();
        map.insert(Arc::from("key"), Value::Int(1));
        let mut env = env_with(&[("obj", Value::Map(map))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if obj:   (obj is Map, not Bool → type error)
        let if_node = simple_if_node(
            Expr::Ident("obj".to_string()),
            vec![slide_block_item("content")],
        );

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "Map condition must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            !sink.is_empty(),
            "Map condition must push E-EVL-003 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-EVL-003",
            "error code must be E-EVL-003 for Map condition; got: {code}"
        );
        // The message must mention 'Map' so the user knows which type caused the error.
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("Map") || msg.contains("map"),
            "E-EVL-003 message must mention 'Map'; got: {msg}"
        );
    }

    // ─── FINDING-004: AC-003 element-scope @if ────────────────────────────────

    /// AC-003: @if at element scope — @if as an inline_item inside a slide body.
    ///
    /// A slide's `inline_items` can contain `BlockItem::If`. When the condition
    /// is true, the @if's slides are appended after the primary slide.
    ///
    /// This exercises the element-scope @if path in `eval_block_items`.
    #[test]
    fn test_BC_1_05_001_if_element_scope() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        let mut env = env_with(&[("show", Value::Bool(true))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();
        let defaults = empty_defaults();

        // Build an inline @if block: `@if show: slide content: title "inline-slide"`
        let inline_if = BlockItem::If(Spanned::new(
            IfNode {
                condition: Spanned::new(Expr::Ident("show".to_string()), dummy_span()),
                then_body: vec![{
                    let title_field = FieldNode {
                        name: Spanned::new("title".to_string(), dummy_span()),
                        value: Spanned::new(
                            FieldValue::Template(vec![TemplateChunk::Literal(
                                "inline-slide".to_string(),
                            )]),
                            dummy_span(),
                        ),
                    };
                    BlockItem::Slide(Spanned::new(
                        SlideNode {
                            kind: Spanned::new("content".to_string(), dummy_span()),
                            tags: vec![],
                            fields: vec![title_field],
                            inline_items: vec![],
                        },
                        dummy_span(),
                    ))
                }],
                elif_branches: vec![],
                else_body: None,
            },
            dummy_span(),
        ));

        // Primary slide with an inline @if in its inline_items.
        let primary_slide = SlideNode {
            kind: Spanned::new("title".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![inline_if],
        };

        let items = vec![BlockItem::Slide(Spanned::new(primary_slide, dummy_span()))];
        let slides =
            crate::for_eval::eval_block_items(&mut env, &items, &defaults, &config, &mut sink);

        // 1 primary slide + 1 from inline @if(show=true) = 2 total.
        assert_eq!(
            slides.len(),
            2,
            "element-scope @if(true) must add 1 inline slide; got {} total",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "element-scope @if must not produce errors; got: {:?}",
            sink.errors()
        );
        assert_eq!(
            slides[1].title_str(),
            Some("inline-slide"),
            "inline slide title must be 'inline-slide'"
        );
    }

    // ─── EC-002: all @elif false, no @else → 0 slides, no error ─────────────

    /// EC-002 (BC-1.05.001):
    /// All `@elif` conditions are false and no `@else:` branch → 0 output, no error.
    ///
    /// Red Gate: fails with `todo!()` until eval_if_chain is implemented.
    #[test]
    fn test_BC_1_05_001_all_branches_false_no_else() {
        let mut env = env_with(&[("x", Value::Bool(false))]);
        let mut sink = DiagnosticSink::new();
        let config = default_config();

        // @if x:    (false)
        //   slide content:
        // @elif x:  (also false)
        //   slide content:
        // (no @else)
        let if_node = IfNode {
            condition: Spanned::new(Expr::Ident("x".to_string()), dummy_span()),
            then_body: vec![slide_block_item("content")],
            elif_branches: vec![(
                Spanned::new(Expr::Ident("x".to_string()), dummy_span()),
                vec![slide_block_item("content")],
            )],
            else_body: None, // no @else
        };

        let slides = eval_if_chain(&mut env, &if_node, &empty_defaults(), &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "all branches false with no @else must produce 0 slides; got {}",
            slides.len()
        );
        assert!(
            sink.is_empty(),
            "all branches false with no @else must produce no errors; got: {:?}",
            sink.errors()
        );
    }
}
