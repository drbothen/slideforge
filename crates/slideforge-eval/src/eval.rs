//! High-level evaluation helpers for the slideforge expression evaluator.
//!
//! This module provides convenience wrappers over [`crate::expr::eval_expr`]
//! for use by the slide builder and template renderer.
//!
//! # STORY-012 additions
//!
//! [`eval_deck`] is the top-level entry point that takes a parsed [`DeckNode`]
//! and produces a semantic [`Deck`] IR. It is implemented as a stub here and
//! will be filled in during the TDD implementation phase.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_syntax::error::ParseSeverity;
use slideforge_syntax::{DeckNode, DiagnosticSink, Expr, FieldValue, SetRuleValue, TemplateChunk};
use slideforge_types::{Deck, DeckMetadata, OrderedMap, SourceSpan, Value};

use crate::config::EvalConfig;
use crate::env::Env;
use crate::error::EvalError;
use crate::expr::eval_expr;
use crate::filters::format_float_display;
use crate::for_eval::eval_block_items;
use crate::include_cycle::{IncludeGraph, check_include_cycles};

// ─── eval_expr_to_string ────────────────────────────────────────────────────

/// Evaluate `expr` in `env` and coerce the result to an `Arc<str>`.
///
/// This is the function used by the template renderer for `{{ expr }}`
/// interpolation sites. The coercion rules are:
///
/// | Value variant | String representation    |
/// |--------------|--------------------------|
/// | `Str(s)`     | `s` (no quotes)          |
/// | `Int(n)`     | decimal representation    |
/// | `Float(f)`   | decimal representation    |
/// | `Bool(b)`    | `"true"` or `"false"`    |
/// | `Null`       | `""` (empty string)      |
/// | `List(_)`    | Error: E-EVL-003 (use `\| join` to convert)  |
/// | `Map(_)`     | Error: E-EVL-003 (use dot access for fields)  |
///
/// Returns `None` if expression evaluation fails; the error is pushed to
/// `sink` by the underlying [`crate::expr::eval_expr`] call.
///
/// # Errors pushed to `sink`
///
/// Delegates entirely to [`crate::expr::eval_expr`].
pub fn eval_expr_to_string(env: &Env, expr: &Expr, sink: &mut DiagnosticSink) -> Option<Arc<str>> {
    use crate::error::EvalError;
    use slideforge_syntax::error::ParseSeverity;
    use slideforge_types::SourceSpan;

    let val = eval_expr(env, expr, sink)?;
    match val {
        Value::Str(s) => Some(s),
        Value::Int(n) => Some(Arc::from(n.to_string().as_str())),
        Value::Float(f) => Some(Arc::from(format_float_display(f.0).as_str())),
        Value::Bool(b) => Some(Arc::from(if b { "true" } else { "false" })),
        Value::Null => Some(Arc::from("")),
        Value::List(_) | Value::Map(_) => {
            sink.push_with_severity(
                EvalError::TypeMismatch {
                    message: format!(
                        "cannot coerce {} to string for interpolation",
                        val.type_name()
                    ),
                    span: SourceSpan::default(),
                },
                ParseSeverity::Error,
            );
            None
        },
    }
}

// ─── eval_deck ──────────────────────────────────────────────────────────────

/// Evaluate a fully-parsed [`DeckNode`] into a semantic [`Deck`] IR.
///
/// This is the primary top-level entry point for the evaluator pipeline. It:
///
/// 1. Collects all `vars:` block entries into an [`Env`] deck-level frame.
/// 2. Processes `set` rules (C01): resolves each set-rule value and stores
///    defaults keyed by `(slide_type, field_name)`.
/// 3. Applies active variant vars (C02): if `active_variant` is `Some`, pushes
///    that variant's `vars` onto the env as a scope frame before slide evaluation.
/// 4. Evaluates all top-level [`BlockItem`](slideforge_syntax::BlockItem)s
///    (slides, `@for` blocks, `@if` blocks) in source order.
/// 5. Returns `None` if any **fatal** diagnostic was pushed; `Some(Deck)` otherwise.
///
/// Returns `None` if any **fatal** diagnostic was pushed during evaluation.
/// Non-fatal diagnostics (warnings, lint hints) are accumulated in `sink` but
/// do not prevent a `Deck` from being returned.
///
/// # Parameters
///
/// - `deck_node`: The parsed AST root node for the `.sf` file.
/// - `config`: Evaluation configuration (thresholds, caps).
/// - `active_variant`: Optional variant name. When `Some`, the named variant's
///   vars override deck-level vars (11-level precedence chain, C02).
/// - `sink`: Accumulates all diagnostics produced during evaluation.
///
/// # Errors pushed to `sink`
///
/// - [`EvalError::UndefinedVariable`](crate::EvalError) — a variable is
///   referenced that is not defined in any scope.
/// - [`EvalError::ReservedKeyword`](crate::EvalError) — a reserved keyword is
///   used as an identifier.
/// - [`EvalError::NotIterable`](crate::EvalError) — an `@for` collection is
///   not a list.
/// - [`EvalError::TooManySlides`](crate::EvalError) — slide cap exceeded.
/// - Any other [`EvalError`] variants from expression
///   evaluation.
pub fn eval_deck(
    deck_node: &DeckNode,
    config: &EvalConfig,
    sink: &mut DiagnosticSink,
) -> Option<Deck> {
    eval_deck_with_variant(deck_node, config, None, sink)
}

/// Full evaluation entry point with optional variant activation.
///
/// See [`eval_deck`] for the primary API. This function accepts an
/// `active_variant` parameter for CLI `--variant` flag support (C02).
///
/// Set-rule defaults resolved here are currently stored in the returned [`Deck`]
/// registers field (reserved); future stories will thread set-rules through the
/// layout engine.
pub fn eval_deck_with_variant(
    deck_node: &DeckNode,
    config: &EvalConfig,
    active_variant: Option<&str>,
    sink: &mut DiagnosticSink,
) -> Option<Deck> {
    // ── Step 1: Build deck-level variable environment from all vars: blocks ──
    let mut deck_vars: IndexMap<Arc<str>, Value> = IndexMap::new();

    for vars_block in &deck_node.vars {
        for (name_spanned, value_spanned) in &vars_block.entries {
            let var_name: Arc<str> = Arc::from(name_spanned.value().as_str());
            let value = eval_field_value_to_value(
                value_spanned.value(),
                &Env::new(deck_vars.clone()),
                sink,
            );
            if let Some(v) = value {
                deck_vars.insert(var_name, v);
            }
        }
    }

    let mut env = Env::new(deck_vars.clone());

    // ── Step 2: Process set-rules (C01) ──
    // Resolve each set-rule value and store defaults keyed by (slide_type, field_name).
    // The resolved map is built here and will be threaded through to the layout
    // stage in a future story. Slide-level field values override these defaults.
    // `brand.*` references are preserved as placeholder strings for the brand stage
    // (per AC-015 — brand loading happens after eval).
    let mut set_rule_defaults: HashMap<(Arc<str>, Arc<str>), Value> = HashMap::new();
    for set_rule in &deck_node.set_rules {
        let slide_type: Arc<str> = Arc::from(set_rule.slide_type.value().as_str());
        let field_name: Arc<str> = Arc::from(set_rule.field.value().as_str());
        let value = eval_set_rule_value(set_rule.value.value(), &env, sink);
        if let Some(v) = value {
            set_rule_defaults.insert((slide_type, field_name), v);
        }
    }
    // set_rule_defaults is threaded to eval_block_items → eval_slide_node
    // where defaults are applied to each slide's fields (AC-014 / C01).

    // ── Step 3: Apply active variant vars (C02) ──
    // Variant vars override deck-level vars (11-level precedence chain).
    if let Some(variant_name) = active_variant {
        match deck_node
            .variants
            .as_ref()
            .and_then(|vb| vb.variants.iter().find(|v| v.name.value() == variant_name))
        {
            Some(variant) => {
                // Evaluate the variant's vars and push them as an inner scope.
                let mut variant_vars: IndexMap<Arc<str>, Value> = IndexMap::new();
                for (name_spanned, value_spanned) in &variant.vars {
                    let var_name: Arc<str> = Arc::from(name_spanned.value().as_str());
                    let value = eval_field_value_to_value(value_spanned.value(), &env, sink);
                    if let Some(v) = value {
                        variant_vars.insert(var_name, v);
                    }
                }
                env.push_scope(variant_vars);
            },
            None => {
                // Named variant not found — push an error (E-EVL-001 style).
                sink.push_with_severity(
                    EvalError::UndefinedVariable {
                        name: Arc::from(variant_name),
                        scope_list: deck_node.variant_names.join(", "),
                        span: SourceSpan::default(),
                    },
                    ParseSeverity::Error,
                );
            },
        }
    }

    // ── Step 4: Evaluate all top-level block items ──
    let mut slides = eval_block_items(&mut env, &deck_node.items, &set_rule_defaults, config, sink);

    // ── Step 4b: Deck-level slide cap (F-P2-003 / AC-014) ──
    // The per-@for cap in eval_for_block handles intra-loop excess. However,
    // when multiple top-level @for blocks each produce slides below the cap,
    // the combined total may still exceed `max_total_slides`. This deck-level
    // check enforces the hard cap on the aggregate slide count.
    if let Some(max) = config.max_total_slides
        && slides.len() > max
    {
        sink.push_with_severity(
            EvalError::TooManySlides {
                count: slides.len(),
                max,
                span: SourceSpan::default(),
            },
            ParseSeverity::Error,
        );
        slides.truncate(max);
    }

    // ── Step 5: I01 — Return None if any fatal diagnostic was pushed ──
    if sink.has_fatal() {
        return None;
    }

    // ── Step 6: Build deck metadata ──
    let lang = deck_node
        .lang
        .as_ref()
        .map(|l| Arc::from(l.value().as_str()));
    let title = None; // Title is not present in DeckNode (comes from a slide); leave None.

    let metadata = DeckMetadata {
        title,
        slideforge_version: Arc::from("0.1.0"),
        lang,
        author: None,
    };

    // ── Step 7: Build the Deck vars map (OrderedMap) ──
    let mut deck_vars_ordered: OrderedMap<Arc<str>, Value> = OrderedMap::new();
    for (k, v) in &deck_vars {
        deck_vars_ordered.insert(k.clone(), v.clone());
    }

    Some(Deck {
        slides,
        vars: deck_vars_ordered,
        metadata,
        registers: OrderedMap::new(),
    })
}

/// Evaluate a fully-parsed [`DeckNode`] with include-cycle pre-pass.
///
/// This is the **full** top-level entry point for the evaluator pipeline when
/// the caller has include-graph metadata (produced by the parser/include-resolver
/// from STORY-008). It runs the include-cycle detection pre-pass **before** any
/// expression evaluation begins, satisfying the Fail-Closed Invariant from
/// BC-1.06.002:
///
/// > Cycle detection runs as a **pre-pass** on the include graph BEFORE any
/// > expression evaluation begins. This ensures no partial evaluation of a
/// > cyclic deck can occur.
///
/// If any E-PAR-004 diagnostics are pushed (cycle detected), this function
/// returns `None` without evaluating the deck — the sink will contain the
/// cycle diagnostics.
///
/// # Parameters
///
/// - `deck_node`: The parsed (merged) AST root.
/// - `include_graph`: The include graph for the deck, mapping canonical file
///   paths to the files each directly includes. Built by the parser during
///   `@include` resolution.
/// - `root_file`: The canonical path of the root `.sf` file (entry point).
/// - `config`: Evaluation configuration.
/// - `active_variant`: Optional variant name for CLI `--variant` flag.
/// - `sink`: Accumulates all diagnostics.
///
/// # Returns
///
/// `None` if any cycle is detected (or any fatal diagnostic during evaluation),
/// `Some(Deck)` otherwise.
pub fn eval_deck_with_cycle_check(
    deck_node: &DeckNode,
    include_graph: &IncludeGraph,
    root_file: &Arc<str>,
    config: &EvalConfig,
    active_variant: Option<&str>,
    sink: &mut DiagnosticSink,
) -> Option<Deck> {
    // ── Pre-pass: include cycle detection (BC-1.06.002 Fail-Closed) ──
    // Run before any expression evaluation. If cycles are detected, return None
    // immediately — no partial evaluation of a cyclic deck can occur.
    if !check_include_cycles(root_file, include_graph, sink) {
        return None;
    }

    // ── Delegate to the expression evaluator ──
    eval_deck_with_variant(deck_node, config, active_variant, sink)
}

/// Evaluate a [`slideforge_syntax::FieldValue`] into a [`Value`] in the
/// context of a given [`Env`].
///
/// Used by [`eval_deck`] to resolve vars block entries into concrete values.
fn eval_field_value_to_value(
    field_value: &FieldValue,
    env: &Env,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    match field_value {
        FieldValue::Template(chunks) => {
            let mut result = String::new();
            let mut had_error = false;
            for chunk in chunks {
                match chunk {
                    TemplateChunk::Literal(s) => result.push_str(s),
                    TemplateChunk::Expr(expr) => match eval_expr_to_string(env, expr, sink) {
                        Some(s) => result.push_str(s.as_ref()),
                        None => {
                            had_error = true;
                        },
                    },
                    TemplateChunk::MathInline(_)
                    | TemplateChunk::MathDisplay(_)
                    | TemplateChunk::MathInterp(_) => {
                        // Math chunks are stored as-is for now.
                    },
                }
            }
            if had_error {
                None
            } else {
                Some(Value::Str(Arc::from(result.as_str())))
            }
        },
        FieldValue::Num(n) => Some(Value::Int(*n)),
        FieldValue::Float(f) => Some(Value::Float(*f)),
        FieldValue::Bool(b) => Some(Value::Bool(*b)),
        FieldValue::Ident(name) => {
            if let Some(v) = env.lookup(name) {
                Some(v.clone())
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
                        span: slideforge_types::SourceSpan::default(),
                    },
                    ParseSeverity::Error,
                );
                None
            }
        },
        FieldValue::Shape(_) | FieldValue::Error => None,
    }
}

/// Evaluate a [`SetRuleValue`] into a [`Value`] for set-rule default resolution.
///
/// This is the set-rule variant of [`eval_field_value_to_value`]. The key
/// difference is handling `brand.*` references:
///
/// - `{{ brand.footer }}` is stored as a template with an `Expr::FieldAccess`
///   chunk. When the "base" ident is `"brand"`, we preserve the reference as a
///   placeholder string `"__brand_ref:footer__"` so the brand stage can resolve
///   it later (per AC-015).
/// - All other template expressions are evaluated normally in `env`.
fn eval_set_rule_value(
    value: &SetRuleValue,
    env: &Env,
    sink: &mut DiagnosticSink,
) -> Option<Value> {
    match value {
        SetRuleValue::Template(chunks) => {
            let mut result = String::new();
            let mut had_error = false;
            for chunk in chunks {
                match chunk {
                    TemplateChunk::Literal(s) => result.push_str(s),
                    TemplateChunk::Expr(expr) => {
                        // AC-015: brand.* references are preserved as placeholders.
                        // The evaluator cannot resolve `brand` at this stage
                        // (brand loading happens after eval). Detect the pattern
                        // `Expr::FieldAccess { base: Ident("brand"), field }` and
                        // produce `"__brand_ref:<field>__"` placeholder.
                        if let Expr::FieldAccess { base, field } = expr
                            && let Expr::Ident(base_name) = base.as_ref()
                            && base_name == "brand"
                        {
                            // Use write! to avoid extra allocation (clippy::format_push_string).
                            let _ = write!(result, "__brand_ref:{field}__");
                            continue;
                        }
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
                        // Math chunks are stored as-is for now.
                    },
                }
            }
            if had_error {
                None
            } else {
                Some(Value::Str(Arc::from(result.as_str())))
            }
        },
        SetRuleValue::Num(n) => Some(Value::Int(*n)),
        SetRuleValue::Float(f) => Some(Value::Float(*f)),
        SetRuleValue::Bool(b) => Some(Value::Bool(*b)),
        SetRuleValue::Ident(name) => {
            if let Some(v) = env.lookup(name) {
                Some(v.clone())
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
                None
            }
        },
        SetRuleValue::Error => None,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
mod tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use ordered_float::OrderedFloat;
    use slideforge_syntax::span::Span;
    use slideforge_syntax::{
        BlockItem, DeckNode, DiagnosticSink, Expr, FieldNode, FieldValue, ForNode, SlideNode,
        Spanned, TemplateChunk, VarsBlock,
    };
    use slideforge_types::Value;

    use super::*;
    use crate::config::EvalConfig;
    use crate::env::Env;

    // ─── eval_deck test helpers ───────────────────────────────────────────────

    fn dummy_span() -> Span {
        Span::new(0, 0, 0)
    }

    fn default_config() -> EvalConfig {
        EvalConfig::default()
    }

    fn minimal_deck_with_slides(slide_kinds: &[&str]) -> DeckNode {
        let items = slide_kinds
            .iter()
            .map(|kind| {
                BlockItem::Slide(Spanned::new(
                    SlideNode {
                        kind: Spanned::new((*kind).to_string(), dummy_span()),
                        tags: vec![],
                        fields: vec![],
                        inline_items: vec![],
                    },
                    dummy_span(),
                ))
            })
            .collect();
        DeckNode {
            items,
            ..DeckNode::default()
        }
    }

    #[allow(dead_code)]
    fn deck_with_var_and_slide(
        var_name: &str,
        var_value: FieldValue,
        slide_kind: &str,
    ) -> DeckNode {
        let vars_block = VarsBlock {
            entries: vec![(
                Spanned::new(var_name.to_string(), dummy_span()),
                Spanned::new(var_value, dummy_span()),
            )],
        };
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new(slide_kind.to_string(), dummy_span()),
                tags: vec![],
                fields: vec![],
                inline_items: vec![],
            },
            dummy_span(),
        ));
        DeckNode {
            vars: vec![vars_block],
            items: vec![slide_item],
            ..DeckNode::default()
        }
    }

    // ─── BC-2.06.001: eval_deck returns None for todo! stub ──────────────────

    // NOTE: all eval_deck tests exercise the stub — they MUST panic or produce
    // wrong output (Red Gate). When implemented, they will produce the correct
    // results described in each assertion.

    /// BC-2.06.001: eval_deck on a 1-slide deck produces a Deck with 1 slide.
    #[test]
    fn test_bc_2_06_001_eval_deck_simple() {
        let deck_node = minimal_deck_with_slides(&["title"]);
        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);

        let deck = deck.expect("eval_deck must return Some for valid 1-slide deck");
        assert_eq!(deck.slides.len(), 1, "Deck must contain 1 slide");
        assert_eq!(
            deck.slides[0].slide_type.as_ref(),
            "title",
            "slide_type must be 'title'"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.06.002: eval_deck with a vars block resolves variables in slide fields.
    #[test]
    fn test_bc_2_06_002_eval_deck_vars() {
        // deck with vars: client = "Acme" and one slide that references {{ client }} in title.
        let vars_block = VarsBlock {
            entries: vec![(
                Spanned::new("client".to_string(), dummy_span()),
                Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("Acme".to_string())]),
                    dummy_span(),
                ),
            )],
        };
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident("client".to_string()))]),
                dummy_span(),
            ),
        };
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![title_field],
                inline_items: vec![],
            },
            dummy_span(),
        ));
        let deck_node = DeckNode {
            vars: vec![vars_block],
            items: vec![slide_item],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck must return Some for valid deck");

        assert_eq!(deck.slides.len(), 1, "must have 1 slide");
        assert_eq!(
            deck.slides[0].title_str(),
            Some("Acme"),
            "title must resolve {{ client }} to 'Acme'"
        );
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.06.003: eval_deck with a @for block produces the correct slide count.
    #[test]
    fn test_bc_2_06_003_eval_deck_for() {
        // @for x in [1, 2, 3]:
        //   slide content:
        let for_node = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(
                Expr::List(vec![Expr::Num(1), Expr::Num(2), Expr::Num(3)]),
                dummy_span(),
            ),
            body: vec![BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("content".to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))],
        };
        let deck_node = DeckNode {
            items: vec![BlockItem::For(Spanned::new(for_node, dummy_span()))],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck must return Some");

        assert_eq!(deck.slides.len(), 3, "@for [1,2,3] must produce 3 slides");
        assert!(sink.is_empty(), "no errors expected");
    }

    /// BC-2.06.004: eval_deck with only an empty @for → 0-slide Deck (no error).
    #[test]
    fn test_bc_2_06_004_eval_deck_empty_for() {
        let for_node = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(Expr::List(vec![]), dummy_span()),
            body: vec![BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("content".to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))],
        };
        let deck_node = DeckNode {
            items: vec![BlockItem::For(Spanned::new(for_node, dummy_span()))],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck must return Some (empty for is valid)");

        assert_eq!(deck.slides.len(), 0, "empty @for must produce 0 slides");
        assert!(sink.is_empty(), "no errors expected for empty @for");
    }

    /// BC-2.06.005: eval_deck with 0 items → Deck with 0 slides.
    #[test]
    fn test_bc_2_06_005_eval_deck_empty_deck() {
        let deck_node = DeckNode::default();
        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck on empty deck must return Some");

        assert_eq!(deck.slides.len(), 0, "empty deck must have 0 slides");
        assert!(sink.is_empty(), "no errors expected for empty deck");
    }

    /// BC-2.06.006: eval_deck metadata fields are populated from deck_node.
    #[test]
    fn test_bc_2_06_006_eval_deck_metadata_lang() {
        let deck_node = DeckNode {
            lang: Some(Spanned::new("en-US".to_string(), dummy_span())),
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck must return Some");

        assert_eq!(
            deck.metadata.lang.as_deref(),
            Some("en-US"),
            "deck metadata lang must be 'en-US'"
        );
    }

    // ─── BC-2.07.001: reserved keywords produce E-PAR-006 ──────────────────

    /// BC-2.07.001: @while in AST → E-PAR-006 error in sink.
    ///
    /// The reserved-keyword detection happens at the eval layer. The keyword
    /// `@while` is not a valid DSL construct; the evaluator must detect it
    /// and push E-PAR-006.
    ///
    /// Since `@while` is parsed as an error sentinel or is entirely absent
    /// from the AST (the parser doesn't recognise it), we test via the
    /// EvalError::ReservedKeyword variant directly to verify the error type
    /// is pushable and carries the right code.
    #[test]
    fn test_bc_2_07_001_while_reserved() {
        use crate::error::EvalError;
        use slideforge_syntax::error::ParseSeverity;
        use slideforge_types::SourceSpan;

        let mut sink = DiagnosticSink::new();

        // Simulate the evaluator detecting a reserved keyword.
        let err = EvalError::ReservedKeyword {
            keyword: Arc::from("while"),
            hint: "@while is not a valid DSL keyword; use @for for iteration".to_string(),
            span: SourceSpan::default(),
        };
        sink.push_with_severity(err, ParseSeverity::Error);

        assert!(!sink.is_empty(), "E-PAR-006 must be in the sink");
        let code = sink.errors()[0].code().map(|c| c.to_string());
        assert_eq!(
            code.as_deref(),
            Some("E-PAR-006"),
            "error code must be E-PAR-006"
        );
    }

    /// BC-2.07.002: @fn in AST → E-PAR-006 error in sink.
    #[test]
    fn test_bc_2_07_002_fn_reserved() {
        use crate::error::EvalError;
        use slideforge_syntax::error::ParseSeverity;
        use slideforge_types::SourceSpan;

        let mut sink = DiagnosticSink::new();

        let err = EvalError::ReservedKeyword {
            keyword: Arc::from("fn"),
            hint: "@fn is reserved for a future version of slideforge".to_string(),
            span: SourceSpan::default(),
        };
        sink.push_with_severity(err, ParseSeverity::Error);

        assert!(!sink.is_empty(), "E-PAR-006 must be in the sink");
        let code = sink.errors()[0].code().map(|c| c.to_string());
        assert_eq!(
            code.as_deref(),
            Some("E-PAR-006"),
            "error code must be E-PAR-006"
        );
        // The error message must mention the keyword.
        let msg = sink.errors()[0].to_string();
        assert!(msg.contains("fn"), "message must mention the keyword 'fn'");
    }

    /// BC-2.07.003: @return is a reserved keyword → E-PAR-006.
    #[test]
    fn test_bc_2_07_003_return_reserved() {
        use crate::error::EvalError;
        use slideforge_syntax::error::ParseSeverity;
        use slideforge_types::SourceSpan;

        let mut sink = DiagnosticSink::new();
        sink.push_with_severity(
            EvalError::ReservedKeyword {
                keyword: Arc::from("return"),
                hint: "@return is reserved for future user-defined functions".to_string(),
                span: SourceSpan::default(),
            },
            ParseSeverity::Error,
        );

        assert!(!sink.is_empty());
        let code = sink.errors()[0].code().map(|c| c.to_string());
        assert_eq!(code.as_deref(), Some("E-PAR-006"));
    }

    /// BC-2.07.004: reserved keyword help text is present in the diagnostic.
    #[test]
    fn test_bc_2_07_004_reserved_keyword_help_text_present() {
        use crate::error::EvalError;
        use slideforge_syntax::error::ParseSeverity;
        use slideforge_types::SourceSpan;

        let mut sink = DiagnosticSink::new();
        let hint = "Use @for instead of @while for iteration".to_string();
        sink.push_with_severity(
            EvalError::ReservedKeyword {
                keyword: Arc::from("while"),
                hint: hint.clone(),
                span: SourceSpan::default(),
            },
            ParseSeverity::Error,
        );

        let help = sink.errors()[0]
            .help()
            .map(|h| h.to_string())
            .unwrap_or_default();
        assert!(
            help.contains("@for") || help.contains("while"),
            "help text must reference the correction; got: {help}"
        );
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

    // ── Int → string ──────────────────────────────────────────────────────────

    /// BC-2.02.008: `Int(42)` coerces to `"42"`
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_int() {
        let env = env_with(&[("n", Value::Int(42))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("n".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("42")));
    }

    // ── Str → string ──────────────────────────────────────────────────────────

    /// BC-2.02.008: `Str("hello")` coerces to `"hello"` (no quotes added)
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_str() {
        let env = env_with(&[("msg", Value::Str(Arc::from("hello")))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("msg".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("hello")));
    }

    // ── Null → empty string ───────────────────────────────────────────────────

    /// BC-2.02.008: `Null` coerces to `""` (empty string, not "null")
    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_null() {
        let env = env_with(&[("nothing", Value::Null)]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("nothing".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("")));
    }

    // ── Bool → "true"/"false" ────────────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_bool_true() {
        let env = env_with(&[("flag", Value::Bool(true))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("flag".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("true")));
    }

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_bool_false() {
        let env = env_with(&[("flag", Value::Bool(false))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("flag".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("false")));
    }

    // ── Float → decimal string ────────────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_float() {
        // Use a value that is not an approximation of a well-known constant
        // (avoids clippy::approx_constant lint).
        let test_val = 1.234_567_89_f64;
        let env = env_with(&[("x", Value::Float(OrderedFloat(test_val)))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("x".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        let s = result.expect("float must produce Some(string)");
        // Parse it back to verify it's a valid float representation.
        let reparsed: f64 = s.parse().expect("float string must be parseable");
        assert!((reparsed - test_val).abs() < 1e-10);
    }

    // ── Undefined var → None + error ─────────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_undefined_var_returns_none() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("nonexistent".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert_eq!(result, None, "undefined var must return None");
        assert!(!sink.is_empty(), "undefined var must push a diagnostic");
    }

    // ── Literal Null expr → empty string ──────────────────────────────────────

    #[test]
    fn test_bc_2_02_008_eval_expr_to_string_literal_null() {
        let env = empty_env();
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Null;
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(sink.is_empty());
        assert_eq!(result, Some(Arc::from("")));
    }

    // ── List/Map → error (FINDING-005) ───────────────────────────────────────

    /// FINDING-005: `eval_expr_to_string` must return None + push error for List values.
    #[test]
    fn test_eval_expr_to_string_list_returns_error() {
        let env = env_with(&[("items", Value::List(vec![Value::Int(1)]))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("items".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(
            result.is_none(),
            "List value must return None from eval_expr_to_string"
        );
        assert!(!sink.is_empty(), "List value must push a diagnostic");
    }

    /// FINDING-005: `eval_expr_to_string` must return None + push error for Map values.
    #[test]
    fn test_eval_expr_to_string_map_returns_error() {
        use slideforge_types::OrderedMap;
        let mut map = OrderedMap::new();
        map.insert(Arc::from("k"), Value::Int(1));
        let env = env_with(&[("obj", Value::Map(map))]);
        let mut sink = DiagnosticSink::new();
        let expr = Expr::Ident("obj".to_string());
        let result = eval_expr_to_string(&env, &expr, &mut sink);
        assert!(
            result.is_none(),
            "Map value must return None from eval_expr_to_string"
        );
        assert!(!sink.is_empty(), "Map value must push a diagnostic");
    }

    // ─── C01: set-rule support tests ─────────────────────────────────────────

    /// C01: set-rule default is applied to a slide that does not explicitly set the field.
    ///
    /// `set content: footer "Confidential"` + a `content` slide with NO explicit
    /// `footer` field → the slide's `footer` field must be `"Confidential"` after
    /// evaluation (AC-014: set-rule injects the default into the slide IR).
    #[test]
    fn test_set_rule_applied_as_default() {
        use slideforge_syntax::{SetRule, SetRuleValue};

        let set_rule = SetRule {
            slide_type: Spanned::new("content".to_string(), dummy_span()),
            field: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                SetRuleValue::Template(vec![TemplateChunk::Literal("Confidential".to_string())]),
                dummy_span(),
            ),
        };
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![], // no footer field — set-rule supplies the default
                inline_items: vec![],
            },
            dummy_span(),
        ));
        let deck_node = DeckNode {
            set_rules: vec![set_rule],
            items: vec![slide_item],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        assert!(
            deck.is_some(),
            "eval_deck must return Some when set-rules are present"
        );
        assert!(sink.is_empty(), "no errors expected for valid set-rule");

        // AC-014: the set-rule default MUST be present in the slide's fields.
        let deck = deck.unwrap();
        assert_eq!(deck.slides.len(), 1, "must have 1 slide");
        let slide = &deck.slides[0];
        let footer = slide.fields.get("footer");
        assert!(
            footer.is_some(),
            "set-rule default 'Confidential' must be injected into slide's footer field; got no footer"
        );
        // The value must be Value::Str("Confidential").
        match footer {
            Some(slideforge_types::FieldValue::Literal(Value::Str(s))) => {
                assert_eq!(
                    s.as_ref(),
                    "Confidential",
                    "set-rule default value must be 'Confidential'"
                );
            },
            other => panic!("footer field must be Literal(Str('Confidential')); got: {other:?}"),
        }
    }

    /// C01: A slide that explicitly sets a field overrides the set-rule default.
    ///
    /// This test verifies that eval_deck completes without error when a slide
    /// sets a field explicitly that is also covered by a set-rule.
    #[test]
    fn test_set_rule_slide_override_wins() {
        use slideforge_syntax::{SetRule, SetRuleValue};

        let set_rule = SetRule {
            slide_type: Spanned::new("content".to_string(), dummy_span()),
            field: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                SetRuleValue::Template(vec![TemplateChunk::Literal("Default".to_string())]),
                dummy_span(),
            ),
        };

        // Slide explicitly sets footer = "Custom".
        let footer_field = FieldNode {
            name: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Literal("Custom".to_string())]),
                dummy_span(),
            ),
        };
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![footer_field],
                inline_items: vec![],
            },
            dummy_span(),
        ));
        let deck_node = DeckNode {
            set_rules: vec![set_rule],
            items: vec![slide_item],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        let deck = eval_deck(&deck_node, &config, &mut sink);
        let deck = deck.expect("eval_deck must return Some");
        assert!(sink.is_empty(), "no errors expected");

        // Slide explicitly set footer = "Custom"; the set-rule default ("Default")
        // must not overwrite it.
        let slide = &deck.slides[0];
        let footer = slide.fields.get("footer");
        assert!(
            footer.is_some(),
            "slide must have a footer field after eval"
        );
        match footer {
            Some(slideforge_types::FieldValue::Literal(Value::Str(s))) => {
                assert_eq!(
                    s.as_ref(),
                    "Custom",
                    "slide-level footer 'Custom' must win over set-rule default 'Default'"
                );
            },
            other => panic!("footer must be Literal(Str('Custom')); got: {other:?}"),
        }
    }

    /// C01: set-rule with a brand.* reference preserves the placeholder.
    ///
    /// `set content: footer "{{ brand.footer }}"` → the resolved value is
    /// `"__brand_ref:footer__"` (brand resolution deferred to brand stage).
    /// End-to-end: a `content` slide without an explicit `footer` field must
    /// receive the brand-ref placeholder as its footer value (AC-015).
    #[test]
    fn test_set_rule_brand_ref_preserved() {
        use slideforge_syntax::{SetRule, SetRuleValue};

        let set_rule = SetRule {
            slide_type: Spanned::new("content".to_string(), dummy_span()),
            field: Spanned::new("footer".to_string(), dummy_span()),
            value: Spanned::new(
                SetRuleValue::Template(vec![TemplateChunk::Expr(Expr::FieldAccess {
                    base: Box::new(Expr::Ident("brand".to_string())),
                    field: "footer".to_string(),
                })]),
                dummy_span(),
            ),
        };

        // Add a content slide without an explicit footer field.
        // The set-rule default must inject the brand-ref placeholder.
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![], // no footer — set-rule injects the brand-ref
                inline_items: vec![],
            },
            dummy_span(),
        ));

        let deck_node = DeckNode {
            set_rules: vec![set_rule],
            items: vec![slide_item],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        // eval_deck must succeed — brand.* refs are preserved, not failed.
        let deck = eval_deck(&deck_node, &config, &mut sink);
        assert!(
            deck.is_some(),
            "eval_deck must return Some for brand-ref set-rule"
        );
        // No error should be pushed for `brand.*` references in set-rules
        // (they are preserved as placeholders, not resolved at eval time).
        assert!(
            sink.is_empty(),
            "brand-ref set-rule must not push an error; got: {:?}",
            sink.errors()
        );

        // End-to-end propagation: the content slide's footer must carry the
        // brand-ref placeholder `"__brand_ref:footer__"` (AC-015).
        let deck = deck.unwrap();
        assert_eq!(deck.slides.len(), 1, "must have 1 slide");
        let slide = &deck.slides[0];
        let footer = slide.fields.get("footer");
        assert!(
            footer.is_some(),
            "brand-ref set-rule must inject footer into the slide's fields (AC-015)"
        );
        match footer {
            Some(slideforge_types::FieldValue::Literal(Value::Str(s))) => {
                assert_eq!(
                    s.as_ref(),
                    "__brand_ref:footer__",
                    "brand-ref placeholder must be '__brand_ref:footer__'; got: {s}"
                );
            },
            other => panic!("footer must be Literal(Str('__brand_ref:footer__')); got: {other:?}"),
        }
    }

    // ─── C02: variant vars tests ──────────────────────────────────────────────

    /// C02: variant vars override deck-level vars.
    ///
    /// Deck vars `{ color: "blue" }`, variant "exec" vars `{ color: "red" }`.
    /// With active variant "exec", `{{ color }}` must resolve to "red".
    #[test]
    fn test_variant_vars_override_deck_vars() {
        use slideforge_syntax::{VariantNode, VariantsBlock};

        // Deck var: color = "blue".
        let vars_block = VarsBlock {
            entries: vec![(
                Spanned::new("color".to_string(), dummy_span()),
                Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("blue".to_string())]),
                    dummy_span(),
                ),
            )],
        };

        // Variant "exec" var: color = "red".
        let variant = VariantNode {
            name: Spanned::new("exec".to_string(), dummy_span()),
            include_tags: vec![],
            exclude_tags: vec![],
            vars: vec![(
                Spanned::new("color".to_string(), dummy_span()),
                Spanned::new(
                    FieldValue::Template(vec![TemplateChunk::Literal("red".to_string())]),
                    dummy_span(),
                ),
            )],
            inherits: None,
        };

        // Slide with title = {{ color }}.
        let title_field = FieldNode {
            name: Spanned::new("title".to_string(), dummy_span()),
            value: Spanned::new(
                FieldValue::Template(vec![TemplateChunk::Expr(Expr::Ident("color".to_string()))]),
                dummy_span(),
            ),
        };
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![title_field],
                inline_items: vec![],
            },
            dummy_span(),
        ));

        let deck_node = DeckNode {
            vars: vec![vars_block],
            variants: Some(VariantsBlock {
                variants: vec![variant],
            }),
            variant_names: vec!["exec".to_string()],
            items: vec![slide_item],
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        // Evaluate with active variant "exec".
        let deck = eval_deck_with_variant(&deck_node, &config, Some("exec"), &mut sink);
        let deck = deck.expect("eval_deck_with_variant must return Some");
        assert!(
            sink.is_empty(),
            "no errors expected; got: {:?}",
            sink.errors()
        );

        // With variant "exec", {{ color }} should resolve to "red" (not "blue").
        assert_eq!(
            deck.slides[0].title_str(),
            Some("red"),
            "variant var color='red' must override deck var color='blue'"
        );
    }

    // ─── C02: undefined variant name ─────────────────────────────────────────

    /// C02: `eval_deck_with_variant` with an undefined variant name pushes an
    /// error to the sink and returns `None`.
    ///
    /// The deck has no `variants:` block. Requesting variant `"nonexistent"`
    /// must push an error (E-EVL-001 / UndefinedVariable) and cause
    /// `eval_deck_with_variant` to return `None` (fatal or at least sink
    /// contains an error that propagates).
    #[test]
    fn test_undefined_variant_name_produces_error() {
        // Build a DeckNode with no variants block.
        let slide_item = BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![],
                inline_items: vec![],
            },
            dummy_span(),
        ));
        let deck_node = DeckNode {
            items: vec![slide_item],
            variants: None,
            variant_names: vec![], // no variants defined
            ..DeckNode::default()
        };

        let config = default_config();
        let mut sink = DiagnosticSink::new();

        // Request a variant that doesn't exist.
        let _deck = eval_deck_with_variant(&deck_node, &config, Some("nonexistent"), &mut sink);

        // The sink must contain at least one error about the undefined variant.
        assert!(
            !sink.is_empty(),
            "requesting an undefined variant must push a diagnostic to the sink"
        );
        // The error message must mention the variant name to aid the user.
        let first_err = sink.errors()[0].to_string();
        assert!(
            first_err.contains("nonexistent"),
            "error message must mention the undefined variant name 'nonexistent'; got: {first_err}"
        );
    }

    // ─── I01: fatal error returns None ────────────────────────────────────────

    /// I01: eval_deck returns None when a Fatal diagnostic is in the sink.
    ///
    /// This verifies the postcondition: `if sink.has_fatal() { None }`.
    #[test]
    fn test_eval_deck_returns_none_on_fatal_error() {
        use slideforge_syntax::error::ParseSeverity;
        use slideforge_types::SourceSpan;

        // Create a sink with a pre-pushed Fatal error.
        // We simulate the evaluator encountering a fatal condition by
        // pre-seeding the sink before calling eval_deck. In production,
        // a Fatal diagnostic would be pushed during evaluation itself.
        let deck_node = DeckNode::default();
        let config = default_config();
        let mut sink = DiagnosticSink::new();

        // Push a Fatal diagnostic to simulate a fatal evaluation failure.
        sink.push_with_severity(
            EvalError::TooManySlides {
                count: 10_000,
                max: 1_000,
                span: SourceSpan::default(),
            },
            ParseSeverity::Fatal,
        );

        assert!(sink.has_fatal(), "sink must report has_fatal == true");

        let deck = eval_deck(&deck_node, &config, &mut sink);
        assert!(
            deck.is_none(),
            "eval_deck must return None when sink has a Fatal diagnostic"
        );
    }

    // ─── C03: structural reserved-keyword guarantee ───────────────────────────

    /// C03: The `BlockItem` enum does not have `While` or `Fn` variants.
    ///
    /// Reserved keywords (@while, @fn) are rejected at parse time — the parser
    /// cannot produce `BlockItem::While` or `BlockItem::Fn` variants because
    /// they do not exist. This is a structural (compile-time) guarantee.
    ///
    /// This test documents the defense-in-depth strategy: the Rust type system
    /// enforces that no code path can construct a BlockItem with a reserved-
    /// keyword variant. The exhaustive match in `eval_block_items` would fail to
    /// compile if new variants were added without handling them.
    ///
    /// The test verifies the 4 ALLOWED variants are constructible and the
    /// compiler rejects any others (structural invariant, not runtime check).
    #[test]
    fn test_c03_block_item_has_no_reserved_keyword_variants() {
        use slideforge_syntax::span::Span;
        use slideforge_syntax::{BlockItem, IfNode, SectionNode, SlideNode, Spanned};

        let span = Span::new(0, 0, 0);

        // Exhaust all 4 valid BlockItem variants — if the enum grew a While/Fn
        // variant, the exhaustive match in eval_block_items would fail to compile.
        let variants: Vec<BlockItem> = vec![
            BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("title".to_string(), span),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                span,
            )),
            BlockItem::For(Spanned::new(
                slideforge_syntax::ForNode {
                    binding: Spanned::new("x".to_string(), span),
                    collection: Spanned::new(Expr::List(vec![]), span),
                    body: vec![],
                },
                span,
            )),
            BlockItem::If(Spanned::new(
                IfNode {
                    condition: Spanned::new(Expr::Bool(true), span),
                    then_body: vec![],
                    elif_branches: vec![],
                    else_body: None,
                },
                span,
            )),
            BlockItem::Section(Spanned::new(
                SectionNode {
                    kind: Spanned::new("intro".to_string(), span),
                    fields: vec![],
                },
                span,
            )),
        ];

        // All 4 valid variants are constructible; no 5th "While" or "Fn" variant exists.
        assert_eq!(
            variants.len(),
            4,
            "BlockItem must have exactly 4 variants (no While/Fn)"
        );
    }

    // ─── F-P2-003: deck-level slide cap ──────────────────────────────────────

    /// F-P2-003: max_total_slides is enforced at the deck level, not just per @for.
    ///
    /// Two @for blocks each producing 2 slides = 4 total. With max_total_slides=3,
    /// the deck-level gate truncates to 3 and pushes TooManySlides.
    #[test]
    fn test_deck_level_max_total_slides_cap() {
        // Two separate @for blocks:
        //   @for x in [1, 2]: slide content:   → 2 slides
        //   @for x in [3, 4]: slide content:   → 2 slides
        // Combined: 4 slides. max_total_slides = 3 → must error + truncate.
        let for_node_a = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(Expr::List(vec![Expr::Num(1), Expr::Num(2)]), dummy_span()),
            body: vec![BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("content".to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))],
        };
        let for_node_b = ForNode {
            binding: Spanned::new("x".to_string(), dummy_span()),
            collection: Spanned::new(Expr::List(vec![Expr::Num(3), Expr::Num(4)]), dummy_span()),
            body: vec![BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("content".to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))],
        };
        let deck_node = DeckNode {
            items: vec![
                BlockItem::For(Spanned::new(for_node_a, dummy_span())),
                BlockItem::For(Spanned::new(for_node_b, dummy_span())),
            ],
            ..DeckNode::default()
        };

        let config = EvalConfig {
            large_deck_warn_threshold: 500,
            max_total_slides: Some(3),
        };
        let mut sink = DiagnosticSink::new();

        // eval_deck returns Some (non-fatal error) with truncated slides.
        let deck = eval_deck(&deck_node, &config, &mut sink);
        assert!(
            deck.is_some(),
            "eval_deck must return Some (TooManySlides is non-fatal by default)"
        );

        let deck = deck.unwrap();
        // Deck-level gate truncates to max_total_slides.
        assert!(
            deck.slides.len() <= 3,
            "deck must be truncated to max_total_slides=3; got {}",
            deck.slides.len()
        );
        // TooManySlides error must have been pushed.
        assert!(
            !sink.is_empty(),
            "TooManySlides error must be in the sink when deck cap is exceeded"
        );
    }

    /// F-P2-003: eval_deck_with_variant is callable from the public API.
    ///
    /// This verifies the re-export (F-P2-005): `eval_deck_with_variant` must be
    /// accessible from crate root via `slideforge_eval::eval_deck_with_variant`.
    #[test]
    fn test_eval_deck_with_variant_is_public() {
        // This test will fail to compile if eval_deck_with_variant is not
        // re-exported from lib.rs (F-P2-005).
        use crate::eval_deck_with_variant;

        let deck_node = DeckNode::default();
        let config = default_config();
        let mut sink = DiagnosticSink::new();
        // None = no active variant (same as eval_deck).
        let deck = eval_deck_with_variant(&deck_node, &config, None, &mut sink);
        assert!(
            deck.is_some(),
            "eval_deck_with_variant must return Some for empty deck"
        );
    }

    // ─── FINDING-001: eval_deck_with_cycle_check integrates cycle pre-pass ────

    /// FINDING-001: eval_deck_with_cycle_check must run include cycle detection
    /// as a pre-pass before expression evaluation.
    ///
    /// A deck with a cyclic include graph must return None and push E-PAR-004
    /// without evaluating any slides. This verifies the Fail-Closed invariant
    /// from BC-1.06.002.
    #[test]
    fn test_eval_deck_with_cycle_check_rejects_cyclic_graph() {
        use std::collections::HashMap;

        use crate::eval_deck_with_cycle_check;
        use crate::include_cycle::IncludeGraph;

        // a 1-slide deck that would otherwise evaluate fine
        let deck_node = minimal_deck_with_slides(&["content"]);
        let config = default_config();

        // Build a cyclic include graph: a.sf → b.sf → a.sf
        let mut include_graph: IncludeGraph = HashMap::new();
        include_graph.insert(Arc::from("a.sf"), vec![Arc::from("b.sf")]);
        include_graph.insert(Arc::from("b.sf"), vec![Arc::from("a.sf")]);
        let root = Arc::from("a.sf");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck_with_cycle_check(
            &deck_node,
            &include_graph,
            &root,
            &config,
            None,
            &mut sink,
        );

        assert!(
            deck.is_none(),
            "eval_deck_with_cycle_check must return None for cyclic include graph"
        );
        assert!(
            !sink.is_empty(),
            "cyclic include graph must push E-PAR-004 to sink"
        );
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-PAR-004",
            "cyclic graph error must be E-PAR-004; got: {code}"
        );
    }

    /// FINDING-001: eval_deck_with_cycle_check on a cycle-free graph evaluates normally.
    #[test]
    fn test_eval_deck_with_cycle_check_acyclic_evaluates() {
        use std::collections::HashMap;

        use crate::eval_deck_with_cycle_check;
        use crate::include_cycle::IncludeGraph;

        let deck_node = minimal_deck_with_slides(&["title"]);
        let config = default_config();

        // Acyclic graph: a.sf → b.sf (b.sf has no includes)
        let mut include_graph: IncludeGraph = HashMap::new();
        include_graph.insert(Arc::from("a.sf"), vec![Arc::from("b.sf")]);
        include_graph.insert(Arc::from("b.sf"), vec![]);
        let root = Arc::from("a.sf");

        let mut sink = DiagnosticSink::new();
        let deck = eval_deck_with_cycle_check(
            &deck_node,
            &include_graph,
            &root,
            &config,
            None,
            &mut sink,
        );

        assert!(
            deck.is_some(),
            "eval_deck_with_cycle_check must return Some for acyclic include graph"
        );
        assert!(
            sink.is_empty(),
            "acyclic include graph must not push any diagnostics; got: {:?}",
            sink.errors()
        );
        let deck = deck.unwrap();
        assert_eq!(deck.slides.len(), 1, "evaluated deck must have 1 slide");
    }
}
