//! `@for` loop evaluation for the slideforge DSL.
//!
//! This module implements the deck-level `@for item in collection:` iteration
//! block (BC-2.04.001 through BC-2.04.010). For each element in the evaluated
//! collection:
//!
//! 1. A new inner scope is pushed into [`Env`] with the binding variable set
//!    to the current element value.
//! 2. The `@for` body (a `Vec<BlockItem>`) is evaluated, producing zero or more
//!    [`Slide`]s.
//! 3. The inner scope is popped; the binding variable is no longer accessible.
//!
//! Scoping rules:
//! - The binding variable shadows any same-named variable in outer scope for
//!   the duration of the loop body.
//! - Outer variables remain accessible throughout the loop body.
//! - After the loop exits, the binding variable is not accessible (inner frame
//!   is popped).
//! - Nested `@for` loops have their own independent scopes.
//!
//! # Error accumulation
//!
//! All errors are pushed into the [`DiagnosticSink`] rather than returned.
//! Evaluation continues through all iterations even if one iteration produces
//! errors.
//!
//! # Large-deck warning
//!
//! When a `@for` loop generates more slides than
//! [`EvalConfig::large_deck_warn_threshold`](crate::EvalConfig), a
//! `ParseSeverity::Warning` is pushed into the sink. Evaluation continues
//! normally.

use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_syntax::{BlockItem, DiagnosticSink, Expr, FieldValue, SlideNode, TemplateChunk};
use slideforge_syntax::error::ParseSeverity;
use slideforge_types::{OrderedMap, Slide, SourceSpan, Value};

use crate::config::EvalConfig;
use crate::env::Env;
use crate::error::EvalError;
use crate::eval::eval_expr_to_string;
use crate::expr::eval_expr;

// ─── eval_for_block ──────────────────────────────────────────────────────────

/// Evaluate one `@for item in collection:` block, returning the slides it
/// generates.
///
/// The function:
/// 1. Evaluates `collection_expr` in the current `env`.
/// 2. Verifies the result is a [`Value::List`]; pushes
///    [`EvalError::NotIterable`](crate::EvalError) and returns `vec![]` if not.
/// 3. Iterates over the list, pushing an inner scope frame for each element.
/// 4. Evaluates the `body` block items in the inner scope, collecting slides.
/// 5. Pops the inner scope frame after each iteration.
/// 6. Emits a large-deck warning if the generated count exceeds
///    [`EvalConfig::large_deck_warn_threshold`].
///
/// # Parameters
///
/// - `env`: The mutable variable environment. Modified (push/pop) transiently
///   during evaluation; restored to its pre-call state when this function
///   returns.
/// - `var_name`: The loop binding variable name (e.g. `"item"` in
///   `@for item in items:`).
/// - `collection_expr`: The expression whose value is the collection to iterate.
/// - `body`: The body block items to evaluate for each element.
/// - `config`: Evaluation configuration (thresholds, caps).
/// - `sink`: Accumulates all diagnostics produced during evaluation.
///
/// # Returns
///
/// A `Vec<Slide>` containing all slides produced by iterating over the
/// collection and evaluating the body. Returns an empty `Vec` if the
/// collection is empty or not iterable.
pub fn eval_for_block(
    env: &mut Env,
    var_name: &str,
    collection_expr: &Expr,
    body: &[BlockItem],
    config: &EvalConfig,
    sink: &mut DiagnosticSink,
) -> Vec<Slide> {
    // Evaluate the collection expression.
    let Some(collection_val) = eval_expr(env, collection_expr, sink) else {
        return vec![];
    };

    // Verify the collection is a list.
    let list = match collection_val {
        Value::List(items) => items,
        other => {
            sink.push_with_severity(
                EvalError::NotIterable {
                    value_type: Arc::from(other.type_name()),
                    span: SourceSpan::default(),
                },
                ParseSeverity::Error,
            );
            return vec![];
        },
    };

    let mut slides: Vec<Slide> = Vec::new();

    for item in list {
        // Check max_total_slides cap before generating more slides.
        if let Some(max) = config.max_total_slides
            && slides.len() >= max
        {
            sink.push_with_severity(
                EvalError::TooManySlides {
                    count: slides.len() + 1,
                    max,
                    span: SourceSpan::default(),
                },
                ParseSeverity::Error,
            );
            break;
        }

        // Push an inner scope with the loop binding variable.
        let mut bindings: IndexMap<Arc<str>, Value> = IndexMap::new();
        bindings.insert(Arc::from(var_name), item);
        env.push_scope(bindings);

        // Evaluate the body items in the inner scope.
        let body_slides = eval_block_items(env, body, config, sink);
        slides.extend(body_slides);

        // Restore outer scope.
        env.pop_scope();
    }

    // Emit a large-deck warning if the generated count exceeds the threshold.
    if slides.len() > config.large_deck_warn_threshold {
        sink.push_with_severity(
            EvalError::TooManySlides {
                count: slides.len(),
                max: config.large_deck_warn_threshold,
                span: SourceSpan::default(),
            },
            ParseSeverity::Warning,
        );
    }

    slides
}

// ─── eval_slide_node ─────────────────────────────────────────────────────────

/// Evaluate a single [`SlideNode`] in the current `env`, returning the
/// resulting [`Slide`].
///
/// Returns `None` if evaluation fails (error pushed to `sink`).
///
/// This function is the bridge from the AST `SlideNode` to the semantic IR
/// `Slide`. It resolves all field values via the expression evaluator and
/// constructs the output `Slide`.
///
/// Field resolution:
/// - [`slideforge_syntax::FieldValue::Template`]: each chunk is evaluated
///   (literals are kept as-is; `Expr` chunks go through `eval_expr_to_string`).
/// - [`slideforge_syntax::FieldValue::Num`]: stored as `Value::Int`.
/// - [`slideforge_syntax::FieldValue::Float`]: stored as `Value::Float`.
/// - [`slideforge_syntax::FieldValue::Bool`]: stored as `Value::Bool`.
/// - [`slideforge_syntax::FieldValue::Ident`]: looked up in env.
/// - [`slideforge_syntax::FieldValue::Error`]: skipped (error already in sink).
/// - [`slideforge_syntax::FieldValue::Shape`]: stored as a block (future story).
pub fn eval_slide_node(
    env: &Env,
    slide_node: &SlideNode,
    sink: &mut DiagnosticSink,
) -> Option<Slide> {
    let slide_type: Arc<str> = Arc::from(slide_node.kind.value().as_str());

    let mut fields: OrderedMap<Arc<str>, slideforge_types::FieldValue> = OrderedMap::new();

    for field_node in &slide_node.fields {
        let field_name: Arc<str> = Arc::from(field_node.name.value().as_str());
        let field_value = match field_node.value.value() {
            FieldValue::Template(chunks) => {
                // Evaluate each chunk and concatenate into a string.
                let mut result = String::new();
                let mut had_error = false;
                for chunk in chunks {
                    match chunk {
                        TemplateChunk::Literal(s) => result.push_str(s),
                        TemplateChunk::Expr(expr) => {
                            match eval_expr_to_string(env, expr, sink) {
                                Some(s) => result.push_str(s.as_ref()),
                                None => {
                                    had_error = true;
                                },
                            }
                        },
                        TemplateChunk::MathInline(_)
                        | TemplateChunk::MathDisplay(_)
                        | TemplateChunk::MathInterp(_) => {
                            // Math chunks are stored as-is for now (future story).
                        },
                    }
                }
                if had_error {
                    // Continue with partial result — error already in sink.
                    slideforge_types::FieldValue::Literal(Value::Str(Arc::from(result.as_str())))
                } else {
                    slideforge_types::FieldValue::Literal(Value::Str(Arc::from(result.as_str())))
                }
            },
            FieldValue::Num(n) => {
                slideforge_types::FieldValue::Literal(Value::Int(*n))
            },
            FieldValue::Float(f) => {
                slideforge_types::FieldValue::Literal(Value::Float(*f))
            },
            FieldValue::Bool(b) => {
                slideforge_types::FieldValue::Literal(Value::Bool(*b))
            },
            FieldValue::Ident(name) => {
                // Look up the identifier in env.
                if let Some(v) = env.lookup(name) {
                    slideforge_types::FieldValue::Literal(v.clone())
                } else {
                    let scope_list = env
                        .all_names()
                        .iter()
                        .map(|n| n.as_ref().to_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    sink.push_with_severity(
                        EvalError::UndefinedVariable {
                            name: Arc::from(name.as_str()),
                            scope_list,
                            span: SourceSpan::default(),
                        },
                        ParseSeverity::Error,
                    );
                    // Use Null as a fallback so we can continue.
                    slideforge_types::FieldValue::Literal(Value::Null)
                }
            },
            FieldValue::Shape(shape_node) => {
                // Shape blocks are passed through for future story implementation.
                let _ = shape_node;
                slideforge_types::FieldValue::Literal(Value::Null)
            },
            FieldValue::Error => {
                // Error sentinel — already in sink, skip this field.
                continue;
            },
        };
        fields.insert(field_name, field_value);
    }

    let tags: Vec<Arc<str>> = slide_node
        .tags
        .iter()
        .map(|t| Arc::from(t.value().as_str()))
        .collect();

    Some(Slide {
        slide_type,
        fields,
        blocks: vec![],
        register: None,
        tags,
        source_span: SourceSpan::default(),
    })
}

// ─── eval_block_items ────────────────────────────────────────────────────────

/// Evaluate a slice of [`BlockItem`]s, collecting all generated slides.
///
/// Each item is dispatched based on its variant:
/// - [`BlockItem::Slide`] → [`eval_slide_node`]
/// - [`BlockItem::For`] → [`eval_for_block`] (recursive)
/// - [`BlockItem::If`] → condition evaluated and matching branch selected
///   (delegated to future `if_eval` module in STORY-013)
/// - [`BlockItem::Section`] → ignored for now (generates no slides)
///
/// Errors are accumulated into `sink` without short-circuiting.
pub fn eval_block_items(
    env: &mut Env,
    items: &[BlockItem],
    config: &EvalConfig,
    sink: &mut DiagnosticSink,
) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();

    for item in items {
        match item {
            BlockItem::Slide(spanned_slide) => {
                if let Some(slide) = eval_slide_node(env, spanned_slide.value(), sink) {
                    slides.push(slide);
                }
            },
            BlockItem::For(spanned_for) => {
                let for_node = spanned_for.value();
                let generated = eval_for_block(
                    env,
                    for_node.binding.value(),
                    for_node.collection.value(),
                    &for_node.body,
                    config,
                    sink,
                );
                slides.extend(generated);
            },
            BlockItem::If(_spanned_if) => {
                // @if evaluation is delegated to STORY-013.
                // For now: do nothing (no slides generated).
            },
            BlockItem::Section(_spanned_section) => {
                // Section blocks generate no slides at the eval level.
            },
        }
    }

    slides
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use slideforge_syntax::{BlockItem, Expr, ForNode, SlideNode, Spanned};
    use slideforge_syntax::span::Span;
    use slideforge_types::Value;

    use super::*;
    use crate::config::EvalConfig;
    use crate::env::Env;

    // ─── Test helpers ──────────────────────────────────────────────────────────

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

    /// Build a minimal SlideNode with no fields and no inline items.
    fn minimal_slide_node(kind: &str) -> SlideNode {
        SlideNode {
            kind: Spanned::new(kind.to_string(), dummy_span()),
            tags: vec![],
            fields: vec![],
            inline_items: vec![],
        }
    }

    /// Build a ForNode with a given binding and collection expression.
    fn make_for_node(binding: &str, collection: Expr, body: Vec<BlockItem>) -> ForNode {
        ForNode {
            binding: Spanned::new(binding.to_string(), dummy_span()),
            collection: Spanned::new(collection, dummy_span()),
            body,
        }
    }

    /// Build a single-slide body block item.
    fn slide_block_item(kind: &str) -> BlockItem {
        BlockItem::Slide(Spanned::new(minimal_slide_node(kind), dummy_span()))
    }

    /// Build a collection expression from a Vec of i64 integers.
    fn int_list_expr(items: &[i64]) -> Expr {
        Expr::List(items.iter().map(|&n| Expr::Num(n)).collect())
    }

    // ─── BC-2.04.001: @for generates correct slide count ────────────────────

    /// BC-2.04.001 postcondition: @for over a 3-item list → exactly 3 slides.
    #[test]
    fn test_bc_2_04_001_for_generates_correct_count() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // @for x in [1, 2, 3]:
        //   slide content:
        let body = vec![slide_block_item("content")];
        let collection = int_list_expr(&[1, 2, 3]);

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(
            slides.len(),
            3,
            "@for over 3-item list must produce exactly 3 slides"
        );
        assert!(sink.is_empty(), "no errors expected for valid @for loop");
    }

    // ─── BC-2.04.002: iteration variable holds the current element ───────────

    /// BC-2.04.002: @for x in [1, 2, 3], body references x → each slide's
    /// title field resolves to the corresponding element value.
    ///
    /// The test uses `eval_slide_node` indirectly through `eval_for_block`.
    /// We verify that for each iteration i, the binding `x` is set to i+1.
    ///
    /// This test MUST fail until eval_for_block is implemented.
    #[test]
    fn test_bc_2_04_002_for_item_value_per_slide() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // Build a slide body that references {{ x }} in its title field.
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident("x".to_string()))]),
                dummy_span(),
            ),
        };
        let slide_node = SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![title_field],
            inline_items: vec![],
        };
        let body = vec![BlockItem::Slide(Spanned::new(slide_node, dummy_span()))];
        let collection = int_list_expr(&[10, 20, 30]);

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 3, "must produce 3 slides");
        // Each slide's title must equal the corresponding element value.
        assert_eq!(
            slides[0].title_str(),
            Some("10"),
            "first slide title must be '10'"
        );
        assert_eq!(
            slides[1].title_str(),
            Some("20"),
            "second slide title must be '20'"
        );
        assert_eq!(
            slides[2].title_str(),
            Some("30"),
            "third slide title must be '30'"
        );
    }

    // ─── BC-2.04.003: @for over empty collection ─────────────────────────────

    /// BC-2.04.003: @for x in [] → 0 slides produced, 0 errors pushed.
    #[test]
    fn test_bc_2_04_003_for_empty_collection() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let body = vec![slide_block_item("content")];
        let collection = Expr::List(vec![]); // empty list

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(
            slides.len(),
            0,
            "@for over empty list must produce 0 slides"
        );
        assert!(sink.is_empty(), "no errors expected for empty @for");
    }

    // ─── BC-2.04.004: scope exit — binding not accessible after loop ─────────

    /// BC-2.04.004: after eval_for_block returns, the binding variable must
    /// not be visible in the outer env.
    #[test]
    fn test_bc_2_04_004_for_scope_exit() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // @for item in [1]:  — run one iteration
        let body = vec![slide_block_item("content")];
        let collection = int_list_expr(&[1]);

        eval_for_block(&mut env, "item", &collection, &body, &config, &mut sink);

        // After the loop, "item" must not be in scope.
        assert!(
            env.lookup("item").is_none(),
            "loop binding 'item' must not be visible after the loop exits"
        );
    }

    // ─── BC-2.04.005: outer variables accessible inside loop body ─────────────

    /// BC-2.04.005: variables declared before the loop are accessible inside
    /// the loop body.
    ///
    /// Verified by running the loop and checking that eval_slide_node can
    /// resolve both the outer var and the iteration var.
    #[test]
    fn test_bc_2_04_005_nested_for_outer_var_accessible() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        // Outer env has `prefix = "slide"`.
        let mut env = env_with(&[("prefix", Value::Str(Arc::from("slide")))]);
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // Body references the outer var `prefix`.
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident(
                    "prefix".to_string(),
                ))]),
                dummy_span(),
            ),
        };
        let slide_node = SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![title_field],
            inline_items: vec![],
        };
        let body = vec![BlockItem::Slide(Spanned::new(slide_node, dummy_span()))];

        // @for x in [1, 2]: — body uses outer `prefix`, not x
        let collection = int_list_expr(&[1, 2]);
        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 2, "must produce 2 slides");
        // Both slides must resolve `prefix` from the outer scope.
        assert_eq!(slides[0].title_str(), Some("slide"));
        assert_eq!(slides[1].title_str(), Some("slide"));
        assert!(sink.is_empty(), "no errors expected; outer var must be visible");
    }

    // ─── BC-2.04.006: inner var shadows outer same-name var ─────────────────

    /// BC-2.04.006: if the loop binding has the same name as an outer variable,
    /// the inner binding shadows it inside the loop; after the loop the outer
    /// value is restored.
    #[test]
    fn test_bc_2_04_006_nested_for_shadowing() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        // Outer env has `x = 99`.
        let mut env = env_with(&[("x", Value::Int(99))]);
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // Build a body that reads `x` in the title.
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident("x".to_string()))]),
                dummy_span(),
            ),
        };
        let slide_node = SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![title_field],
            inline_items: vec![],
        };
        let body = vec![BlockItem::Slide(Spanned::new(slide_node, dummy_span()))];

        // @for x in [7]: — inner x = 7 shadows outer x = 99
        let collection = int_list_expr(&[7]);
        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 1, "must produce 1 slide");
        // Inside the loop, x = 7 (the iteration variable, not the outer x = 99).
        assert_eq!(
            slides[0].title_str(),
            Some("7"),
            "inner x must shadow outer x inside the loop body"
        );

        // After the loop, outer x = 99 must be restored.
        assert_eq!(
            env.lookup("x"),
            Some(&Value::Int(99)),
            "outer x must be restored to 99 after the loop exits"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    // ─── BC-2.04.007: three-level nesting — all vars accessible ─────────────

    /// BC-2.04.007: three nested `@for` loops — each level's binding is
    /// accessible within its body and the inner loops can access outer vars.
    #[test]
    fn test_bc_2_04_007_for_three_levels_nesting() {
        // We construct a 3-level nesting by calling eval_for_block from within
        // a body that itself calls eval_for_block (simulating the recursive
        // eval_block_items dispatch). Since the function is stubbed, this test
        // will fail (panic with todo!) — which is the desired Red Gate state.
        //
        // When implemented, this should produce outer_count × mid_count × inner_count slides.

        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // Build innermost: @for z in [1]:  slide content:
        let inner_body = vec![slide_block_item("content")];
        let inner_for = ForNode {
            binding: Spanned::new("z".to_string(), dummy_span()),
            collection: Spanned::new(int_list_expr(&[1]), dummy_span()),
            body: inner_body,
        };

        // Build middle: @for y in [1]: @for z in [1]: ...
        let mid_body = vec![BlockItem::For(Spanned::new(inner_for, dummy_span()))];
        let mid_for = ForNode {
            binding: Spanned::new("y".to_string(), dummy_span()),
            collection: Spanned::new(int_list_expr(&[1]), dummy_span()),
            body: mid_body,
        };

        // Build outer: @for x in [1, 2]: @for y in [1]: @for z in [1]: ...
        let outer_body = vec![BlockItem::For(Spanned::new(mid_for, dummy_span()))];
        let collection = int_list_expr(&[1, 2]); // outer = 2 elements

        let slides =
            eval_for_block(&mut env, "x", &collection, &outer_body, &config, &mut sink);

        // 2 outer × 1 middle × 1 inner = 2 slides
        assert_eq!(
            slides.len(),
            2,
            "3-level nesting with 2×1×1 iterations must produce 2 slides"
        );
        assert!(sink.is_empty(), "no errors expected for valid 3-level nesting");
    }

    // ─── BC-2.04.008: large collection warning ───────────────────────────────

    /// BC-2.04.008: when the generated slide count exceeds
    /// `large_deck_warn_threshold`, a Warning is pushed to the sink.
    #[test]
    fn test_bc_2_04_008_large_collection_warning() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        // Use a low threshold so we can trigger it with a small list.
        let config = EvalConfig {
            large_deck_warn_threshold: 3,
            max_total_slides: None,
        };

        // @for x in [1..=5]: slide content: — 5 slides, threshold is 3
        let body = vec![slide_block_item("content")];
        let collection = int_list_expr(&[1, 2, 3, 4, 5]);

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 5, "must still produce all 5 slides");
        // A warning diagnostic must be in the sink.
        assert!(
            !sink.is_empty(),
            "large collection must push a warning into the sink"
        );
        // The warning must NOT be fatal — evaluation continues.
        assert!(
            !sink.has_fatal(),
            "large-collection warning must not be fatal"
        );
    }

    // ─── BC-2.04.009: non-iterable collection → NotIterable error ─────────────

    /// BC-2.04.009: @for x in "not-a-list": → E-EVL-008 error, 0 slides.
    #[test]
    fn test_bc_2_04_009_for_non_iterable_collection() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        // collection is a string literal, not a list
        let collection = Expr::Str("not-a-list".to_string());
        let body = vec![slide_block_item("content")];

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 0, "non-iterable must produce 0 slides");
        assert!(
            !sink.is_empty(),
            "non-iterable collection must push an error"
        );
    }

    // ─── BC-2.04.010: @for over a null collection → NotIterable error ────────

    /// BC-2.04.010: @for x in null_var: where null_var = Null → E-EVL-008.
    #[test]
    fn test_bc_2_04_010_for_null_collection() {
        let mut env = env_with(&[("null_var", Value::Null)]);
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let collection = Expr::Ident("null_var".to_string());
        let body = vec![slide_block_item("content")];

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 0, "null collection must produce 0 slides");
        assert!(!sink.is_empty(), "null collection must push an error");
    }

    // ─── BC-2.04.011: undefined collection var → UndefinedVariable error ─────

    /// BC-2.04.011: @for x in no_such_var: → E-EVL-001 error, 0 slides.
    #[test]
    fn test_bc_2_04_011_for_undefined_collection_var() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let collection = Expr::Ident("no_such_var".to_string());
        let body = vec![slide_block_item("content")];

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        assert_eq!(slides.len(), 0, "undefined var must produce 0 slides");
        assert!(
            !sink.is_empty(),
            "undefined collection variable must push an error"
        );
    }

    // ─── BC-2.04.012: @for with max_total_slides cap ─────────────────────────

    /// BC-2.04.012: when max_total_slides is Some(2) and the loop would generate
    /// 5 slides, evaluation stops and pushes E-EVL-007.
    #[test]
    fn test_bc_2_04_012_for_max_slides_cap() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = EvalConfig {
            large_deck_warn_threshold: 500,
            max_total_slides: Some(2),
        };

        let body = vec![slide_block_item("content")];
        let collection = int_list_expr(&[1, 2, 3, 4, 5]); // would generate 5

        let slides = eval_for_block(&mut env, "x", &collection, &body, &config, &mut sink);

        // Must have stopped at 2 slides.
        assert!(
            slides.len() <= 2,
            "max_total_slides=2 cap must limit to at most 2 slides; got {}",
            slides.len()
        );
        assert!(!sink.is_empty(), "TooManySlides error must be pushed");
    }

    // ─── eval_slide_node tests ────────────────────────────────────────────────

    /// BC-2.03.001: eval_slide_node produces a Slide with the correct slide_type.
    #[test]
    fn test_bc_2_03_001_eval_slide_node_slide_type() {
        let env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();

        let node = minimal_slide_node("bullets");
        let slide = eval_slide_node(&env, &node, &mut sink);

        let slide = slide.expect("eval_slide_node must return Some for valid input");
        assert_eq!(
            slide.slide_type.as_ref(),
            "bullets",
            "slide_type must match the SlideNode kind"
        );
        assert!(sink.is_empty(), "no errors expected for minimal valid slide");
    }

    /// BC-2.03.002: eval_slide_node with a title field containing a literal string.
    #[test]
    fn test_bc_2_03_002_eval_slide_node_literal_title() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        let env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();

        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal("Hello World".to_string())]),
                dummy_span(),
            ),
        };
        let slide_node = SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![title_field],
            inline_items: vec![],
        };

        let slide = eval_slide_node(&env, &slide_node, &mut sink)
            .expect("eval_slide_node must return Some for valid input");
        assert_eq!(
            slide.title_str(),
            Some("Hello World"),
            "title must resolve to the literal string"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.03.003: eval_slide_node with an expression title field.
    #[test]
    fn test_bc_2_03_003_eval_slide_node_expr_title() {
        use slideforge_syntax::{FieldNode, FieldValue, TemplateChunk};

        let env = env_with(&[("name", Value::Str(Arc::from("World")))]);
        let mut sink = slideforge_syntax::DiagnosticSink::new();

        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident(
                    "name".to_string(),
                ))]),
                dummy_span(),
            ),
        };
        let slide_node = SlideNode {
            kind: Spanned::new("content".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![title_field],
            inline_items: vec![],
        };

        let slide = eval_slide_node(&env, &slide_node, &mut sink)
            .expect("eval_slide_node must return Some");
        assert_eq!(
            slide.title_str(),
            Some("World"),
            "title must resolve the {{ name }} interpolation"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    // ─── eval_block_items tests ────────────────────────────────────────────────

    /// BC-2.05.001: eval_block_items with a single slide produces 1 slide.
    #[test]
    fn test_bc_2_05_001_eval_block_items_single_slide() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let items = vec![slide_block_item("title")];
        let slides = eval_block_items(&mut env, &items, &config, &mut sink);

        assert_eq!(slides.len(), 1, "single slide block item must produce 1 slide");
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.05.002: eval_block_items with a @for block generates the expected slides.
    #[test]
    fn test_bc_2_05_002_eval_block_items_with_for() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let for_node = make_for_node(
            "x",
            int_list_expr(&[1, 2, 3]),
            vec![slide_block_item("content")],
        );
        let items = vec![BlockItem::For(Spanned::new(for_node, dummy_span()))];

        let slides = eval_block_items(&mut env, &items, &config, &mut sink);

        assert_eq!(
            slides.len(),
            3,
            "@for block inside block_items must produce 3 slides"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.05.003: eval_block_items with mixed slides and @for.
    #[test]
    fn test_bc_2_05_003_eval_block_items_mixed() {
        let mut env = empty_env();
        let mut sink = slideforge_syntax::DiagnosticSink::new();
        let config = default_config();

        let for_node = make_for_node(
            "x",
            int_list_expr(&[1, 2]),
            vec![slide_block_item("content")],
        );
        let items = vec![
            slide_block_item("title"),                                   // 1 slide
            BlockItem::For(Spanned::new(for_node, dummy_span())),        // 2 slides
            slide_block_item("bullets"),                                 // 1 slide
        ];

        let slides = eval_block_items(&mut env, &items, &config, &mut sink);

        assert_eq!(
            slides.len(),
            4,
            "1 slide + @for(2) + 1 slide = 4 slides total"
        );
        assert!(sink.is_empty(), "no errors expected");
    }
}
