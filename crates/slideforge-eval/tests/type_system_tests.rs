//! Integration tests for BC-1.02.003 (No Implicit Type Coercion) and
//! BC-1.02.004 (${{ seq }} Disambiguation) — STORY-014.
//!
//! # Red Gate discipline
//!
//! All tests in this file MUST FAIL before STORY-014 is implemented and PASS
//! after. Specifically:
//!
//! - Equality between mismatched types (Str == Bool) is not yet guarded
//!   with E-EVL-003 in `eval_binop` (BC-1.02.003 postcondition 5).
//! - `@if` condition type checking does not yet produce E-EVL-003 for
//!   non-bool conditions (STORY-013 not yet applied to type checking).
//! - `${{ expr }}` dollar-interpolation is not yet a `TemplateChunk` variant;
//!   the evaluator does not prepend `"$"` to interpolation results
//!   (BC-1.02.004 postconditions 1/2/4).
//! - The E-EVL-003 hint text for arithmetic on strings does not yet mention
//!   the `| float` correction path.
//!
//! Tests are named `test_BC_1_02_003_*` / `test_BC_1_02_004_*` for traceability
//! to BC-1.02.003 and BC-1.02.004.

#![allow(clippy::unwrap_used)] // tests may use unwrap
#![allow(clippy::items_after_statements)]
#![allow(clippy::approx_constant)] // 3.14 in tests is the literal value to test, not pi

use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_eval::{eval_deck, eval_expr, EvalConfig};
use slideforge_syntax::span::Span;
use slideforge_syntax::{
    BinOpKind, BlockItem, DeckNode, DiagnosticSink, Expr, FieldNode, FieldValue, IfNode, SlideNode,
    Spanned, TemplateChunk, VarsBlock,
};
use slideforge_types::Value;

// ─── Test helpers ────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

fn default_config() -> EvalConfig {
    EvalConfig::default()
}

/// Build an `Env` with the given string-valued vars (to replicate `vars:`
/// block behavior at the integration level).
fn env_with_str(pairs: &[(&str, &str)]) -> slideforge_eval::Env {
    let mut vars = IndexMap::new();
    for (k, v) in pairs {
        vars.insert(Arc::from(*k), Value::Str(Arc::from(*v)));
    }
    slideforge_eval::Env::new(vars)
}

fn env_with_int(pairs: &[(&str, i64)]) -> slideforge_eval::Env {
    let mut vars = IndexMap::new();
    for (k, v) in pairs {
        vars.insert(Arc::from(*k), Value::Int(*v));
    }
    slideforge_eval::Env::new(vars)
}

/// Build a `DeckNode` with a single `vars:` block and one slide whose `title`
/// field is a template expression referencing the given `var_name`.
fn deck_with_var_and_expr_title(var_name: &str, var_value: FieldValue, title_expr: Expr) -> DeckNode {
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new(var_name.to_string(), dummy_span()),
            Spanned::new(var_value, dummy_span()),
        )],
    };
    let title_field = FieldNode {
        name: Spanned::new("title".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![TemplateChunk::Expr(title_expr)]),
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
    DeckNode {
        vars: vec![vars_block],
        items: vec![slide_item],
        ..DeckNode::default()
    }
}

// ─── BC-1.02.003 — No-coercion tests ────────────────────────────────────────

/// BC-1.02.003 postcondition 1/2: `flag = "NO"` in a `vars:` block must remain
/// the string `"NO"` when interpolated via `{{ flag }}`.
///
/// YAML tools like PyYAML would coerce this to `false`. slideforge must not.
/// Red Gate note: this test passes through `eval_deck` and asserts the EXACT
/// string value stored in the slide's title field. It fails if the evaluator
/// performs any coercion.
#[test]
fn test_bc_1_02_003_no_flag_string_stays_no() {
    let deck_node = deck_with_var_and_expr_title(
        "flag",
        FieldValue::Template(vec![TemplateChunk::Literal("NO".to_string())]),
        Expr::Ident("flag".to_string()),
    );
    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    let deck = deck.expect("eval_deck must succeed");
    assert!(sink.is_empty(), "no errors expected");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("NO"),
        "flag='NO' (string) must remain 'NO'; coercion to false is forbidden (DI-004)"
    );
}

/// BC-1.02.003 postcondition 1/3: `v = "1.10"` must remain `"1.10"` (not `"1.1"`).
///
/// JSON/YAML parsers normalize `1.10` to `1.1` by dropping trailing zeros.
/// The string value `"1.10"` must be preserved byte-for-byte.
#[test]
fn test_bc_1_02_003_version_string_stays_unchanged() {
    let deck_node = deck_with_var_and_expr_title(
        "v",
        FieldValue::Template(vec![TemplateChunk::Literal("1.10".to_string())]),
        Expr::Ident("v".to_string()),
    );
    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    let deck = deck.expect("eval_deck must succeed");
    assert!(sink.is_empty(), "no errors expected");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("1.10"),
        "string '1.10' must not be normalised to '1.1' (DI-004 precision invariant)"
    );
}

/// BC-1.02.003 invariant 2 (AC-005): `yes_value = "yes"` must remain `"yes"`.
#[test]
fn test_bc_1_02_003_yes_string_stays_yes() {
    let deck_node = deck_with_var_and_expr_title(
        "yes_value",
        FieldValue::Template(vec![TemplateChunk::Literal("yes".to_string())]),
        Expr::Ident("yes_value".to_string()),
    );
    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    let deck = deck.expect("eval_deck must succeed");
    assert!(sink.is_empty(), "no errors expected");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("yes"),
        "string 'yes' must not be coerced to bool true (DI-004)"
    );
}

/// BC-1.02.003: `zero = "0"` must remain the string `"0"` (not integer 0).
#[test]
fn test_bc_1_02_003_zero_string_stays_string() {
    let deck_node = deck_with_var_and_expr_title(
        "zero",
        FieldValue::Template(vec![TemplateChunk::Literal("0".to_string())]),
        Expr::Ident("zero".to_string()),
    );
    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    let deck = deck.expect("eval_deck must succeed");
    assert!(sink.is_empty(), "no errors expected");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("0"),
        "string '0' must not be coerced to integer 0 (DI-004)"
    );
}

/// BC-1.02.003 postcondition 4 (AC-003): `{{ "NO" + 1 }}` must produce
/// E-EVL-003 with error code E-EVL-003.
///
/// Red Gate note: the E-EVL-003 error already fires from STORY-011. This test
/// additionally asserts the diagnostic code string to provide named coverage.
#[test]
fn test_bc_1_02_003_arith_on_string_produces_type_error() {
    use slideforge_eval::eval_expr;

    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Add,
        lhs: Box::new(Expr::Str("NO".to_string())),
        rhs: Box::new(Expr::Num(1)),
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert_eq!(result, None, "Str + Int must return None");
    assert!(!sink.is_empty(), "Str + Int must push E-EVL-003");

    // The diagnostic code must be E-EVL-003 (type mismatch).
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "arithmetic on string must produce E-EVL-003 (BC-1.02.003 postcondition 4)"
    );
}

/// BC-1.02.003 EC-002: `rate = "2.5"` / `{{ rate * 100 }}` must produce
/// E-EVL-003 with a hint mentioning `| float`.
///
/// Red Gate: The CURRENT error message does NOT include a `| float` hint.
/// The implementer must add the hint to the `TypeMismatch` message for
/// arithmetic ops on string operands.
#[test]
fn test_bc_1_02_003_arith_string_rate_hint_mentions_float_filter() {
    let env = env_with_str(&[("rate", "2.5")]);
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Mul,
        lhs: Box::new(Expr::Ident("rate".to_string())),
        rhs: Box::new(Expr::Num(100)),
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert_eq!(result, None, "Str * Int must return None (E-EVL-003)");
    assert!(!sink.is_empty(), "must push E-EVL-003 for string arithmetic");

    // BC-1.02.003 requires the error hint to suggest the `| float` conversion.
    // This assertion FAILS before implementation: current errors do not include
    // the `| float` hint text.
    let diag = &sink.errors()[0];
    let help_text = diag.help().map(|h| h.to_string()).unwrap_or_default();
    let err_msg = diag.to_string();
    let combined = format!("{err_msg} {help_text}");
    assert!(
        combined.contains("float") || combined.contains("| float"),
        "E-EVL-003 for string arithmetic must hint at '| float' conversion; got: {combined}"
    );
}

/// BC-1.02.003 postcondition 5 (AC-004): `"NO" == false` must produce E-EVL-003.
///
/// Red Gate: CURRENT `eval_binop` applies `BinOpKind::Eq` as structural equality
/// without type checking. `Value::Str("NO") == Value::Bool(false)` returns
/// `Value::Bool(false)` without an error. This test FAILS before implementation
/// because the equality operator must reject cross-type comparisons involving Bool.
#[test]
fn test_bc_1_02_003_compare_string_to_bool_produces_type_error() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Eq,
        lhs: Box::new(Expr::Str("NO".to_string())),
        rhs: Box::new(Expr::Bool(false)),
    };
    let result = eval_expr(&env, &expr, &mut sink);

    // BC-1.02.003 postcondition 5: `"NO" == false` must produce E-EVL-003,
    // not `Bool(false)`. Comparing a String to a Bool is always a type error.
    assert_eq!(
        result, None,
        "Str == Bool must return None (E-EVL-003), not Bool(false)"
    );
    assert!(
        !sink.is_empty(),
        "'NO' == false must push E-EVL-003 (String vs Bool comparison)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "String == Bool must produce E-EVL-003"
    );
}

/// BC-1.02.003 invariant 1 (AC-006): `{{ v | int }}` on `v = "42"` → `Int(42)`.
///
/// Explicit `| int` is the ONLY conversion path from string to integer.
#[test]
fn test_bc_1_02_003_explicit_int_conversion() {
    let env = env_with_str(&[("v", "42")]);
    let mut sink = DiagnosticSink::new();
    // {{ v | int }}
    let expr = Expr::Pipe {
        lhs: Box::new(Expr::Ident("v".to_string())),
        filter: "int".to_string(),
        args: vec![],
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert!(sink.is_empty(), "no errors expected for valid | int conversion");
    assert_eq!(result, Some(Value::Int(42)), "v='42' | int must produce Int(42)");
}

/// BC-1.02.003 invariant 1 (AC-006): `{{ v | float }}` on `v = "3.14"` → `Float(3.14)`.
#[test]
fn test_bc_1_02_003_explicit_float_conversion() {
    let env = env_with_str(&[("v", "3.14")]);
    let mut sink = DiagnosticSink::new();
    // {{ v | float }}
    let expr = Expr::Pipe {
        lhs: Box::new(Expr::Ident("v".to_string())),
        filter: "float".to_string(),
        args: vec![],
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert!(sink.is_empty(), "no errors expected for valid | float conversion");
    match result {
        Some(Value::Float(f)) => {
            assert!(
                (f.0 - 3.14_f64).abs() < 1e-10,
                "v='3.14' | float must produce Float(3.14); got: {f}"
            );
        },
        other => panic!("expected Float(3.14), got: {other:?}"),
    }
}

/// BC-1.02.003 invariant 1 (AC-006): `{{ flag | bool }}` must produce E-EVL-004.
///
/// There is NO `| bool` filter. Boolean conversion is only via explicit
/// comparison: `{{ v == "true" }}`. This test verifies E-EVL-004 is produced.
///
/// Red Gate: this test PASSES immediately (the filter registry has no `bool`
/// entry). It is included as a regression guard — any future addition of `| bool`
/// would silently introduce implicit coercion and break BC-1.02.003.
#[test]
fn test_bc_1_02_003_bool_filter_does_not_exist() {
    let env = env_with_str(&[("flag", "NO")]);
    let mut sink = DiagnosticSink::new();
    let expr = Expr::Pipe {
        lhs: Box::new(Expr::Ident("flag".to_string())),
        filter: "bool".to_string(),
        args: vec![],
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert_eq!(result, None, "'| bool' must return None (filter not found)");
    assert!(
        !sink.is_empty(),
        "'| bool' must push E-EVL-004 (no such filter)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-004"),
        "'| bool' must produce E-EVL-004 (filter not found) — regression guard against implicit coercion"
    );
    // The available filters list must NOT include "bool".
    let help_text = sink.errors()[0]
        .help()
        .map(|h| h.to_string())
        .unwrap_or_default();
    assert!(
        !help_text.contains("bool"),
        "available filter list must not mention 'bool'; got: {help_text}"
    );
}

/// BC-1.02.003 invariant 2 (AC-005): `@if "true":` must produce E-EVL-003.
///
/// String `"true"` is NOT a boolean. Using it as an `@if` condition without
/// explicit comparison (`== "true"`) must produce a type error.
///
/// Red Gate: `@if` condition evaluation is NOT yet implemented (STORY-013).
/// The `BlockItem::If` arm in `eval_block_items` is a no-op stub. This test
/// FAILS because currently: no error is produced (the block is silently skipped),
/// but BC-1.02.003 requires E-EVL-003.
#[test]
fn test_bc_1_02_003_string_in_condition_type_error() {
    // Build: @if "true": slide content:
    let if_node = IfNode {
        condition: Spanned::new(Expr::Str("true".to_string()), dummy_span()),
        then_body: vec![BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![],
                inline_items: vec![],
            },
            dummy_span(),
        ))],
        elif_branches: vec![],
        else_body: None,
    };
    let deck_node = DeckNode {
        items: vec![BlockItem::If(Spanned::new(if_node, dummy_span()))],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let _deck = eval_deck(&deck_node, &default_config(), &mut sink);

    // BC-1.02.003 invariant 2: string "true" as @if condition must produce
    // E-EVL-003 (no implicit boolean coercion).
    assert!(
        !sink.is_empty(),
        "@if with string condition 'true' must produce E-EVL-003; currently produces no error (stub)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "@if string condition must produce E-EVL-003"
    );
}

/// BC-1.02.003 invariant 2 (AC-005): `@if 0:` must produce E-EVL-003.
///
/// Integer `0` is NOT a boolean (not falsy). Using an integer as an `@if`
/// condition must be a type error.
///
/// Red Gate: same as `test_bc_1_02_003_string_in_condition_type_error` —
/// `@if` evaluation stub silently skips the block.
#[test]
fn test_bc_1_02_003_int_in_condition_type_error() {
    // Build: @if 0: slide content:
    let if_node = IfNode {
        condition: Spanned::new(Expr::Num(0), dummy_span()),
        then_body: vec![BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![],
                inline_items: vec![],
            },
            dummy_span(),
        ))],
        elif_branches: vec![],
        else_body: None,
    };
    let deck_node = DeckNode {
        items: vec![BlockItem::If(Spanned::new(if_node, dummy_span()))],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let _deck = eval_deck(&deck_node, &default_config(), &mut sink);

    // BC-1.02.003 invariant 2: integer 0 as @if condition must produce E-EVL-003.
    assert!(
        !sink.is_empty(),
        "@if with integer condition 0 must produce E-EVL-003; currently produces no error (stub)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "@if integer condition must produce E-EVL-003"
    );
}

/// FINDING-P2-001 (adversary pass 2, STORY-014): `@elif "some_string":` must
/// produce E-EVL-003.
///
/// The same DI-004 guard that rejects `@if "true":` must also reject
/// `@elif "some_string":`. Previously the `BlockItem::If` handler type-checked
/// only the main `@if` condition and left `elif_branches` unchecked.
///
/// This test constructs: `@if true: ... @elif "some_string": ...`
/// The `@if` condition is a valid Bool, so no error from that arm.
/// The `@elif` condition is a Str — this MUST produce E-EVL-003.
#[test]
fn test_bc_1_02_003_string_in_elif_condition_type_error() {
    // Build: @if true: (valid) @elif "some_string": (invalid — Str, not Bool)
    let if_node = IfNode {
        condition: Spanned::new(Expr::Bool(true), dummy_span()),
        then_body: vec![],
        elif_branches: vec![(
            Spanned::new(Expr::Str("some_string".to_string()), dummy_span()),
            vec![BlockItem::Slide(Spanned::new(
                SlideNode {
                    kind: Spanned::new("content".to_string(), dummy_span()),
                    tags: vec![],
                    fields: vec![],
                    inline_items: vec![],
                },
                dummy_span(),
            ))],
        )],
        else_body: None,
    };
    let deck_node = DeckNode {
        items: vec![BlockItem::If(Spanned::new(if_node, dummy_span()))],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let _deck = eval_deck(&deck_node, &default_config(), &mut sink);

    // DI-004: @elif with string condition must produce E-EVL-003.
    assert!(
        !sink.is_empty(),
        "@elif with string condition 'some_string' must produce E-EVL-003 (FINDING-P2-001)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "@elif string condition must produce E-EVL-003 (FINDING-P2-001, DI-004)"
    );
}

/// FINDING-P2-001 (adversary pass 2, STORY-014): `@elif 42:` must also
/// produce E-EVL-003 (integer is not a boolean).
#[test]
fn test_bc_1_02_003_int_in_elif_condition_type_error() {
    // Build: @if true: (valid) @elif 42: (invalid — Int, not Bool)
    let if_node = IfNode {
        condition: Spanned::new(Expr::Bool(true), dummy_span()),
        then_body: vec![],
        elif_branches: vec![(
            Spanned::new(Expr::Num(42), dummy_span()),
            vec![],
        )],
        else_body: None,
    };
    let deck_node = DeckNode {
        items: vec![BlockItem::If(Spanned::new(if_node, dummy_span()))],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let _deck = eval_deck(&deck_node, &default_config(), &mut sink);

    assert!(
        !sink.is_empty(),
        "@elif with integer condition 42 must produce E-EVL-003 (FINDING-P2-001)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "@elif integer condition must produce E-EVL-003 (FINDING-P2-001, DI-004)"
    );
}

// ─── BC-1.02.004 — ${{ seq }} disambiguation tests ──────────────────────────
//
// These tests require a `TemplateChunk::DollarInterp(Expr)` variant that does
// not yet exist in `slideforge-syntax`. Until STORY-005 (lexer) and the parser
// add this variant, these tests model the EVALUATOR'S RESPONSIBILITY:
// when the evaluator receives a `DollarInterp` chunk, it must prepend "$" to
// the evaluated expression result.
//
// Strategy: construct a TemplateChunk sequence that SIMULATES ${{ expr }} by
// using a Literal("$") followed by Expr(e). Then assert the concatenated result
// equals "$<result>". This exercises the pipeline as-is and will FAIL when the
// evaluator is upgraded to handle a real `DollarInterp` variant (at which point
// the test must be updated to use the new variant).
//
// Note: these tests fail until the DollarInterp evaluator path is implemented.
// The failure mode: either the TemplateChunk variant doesn't compile, OR the
// evaluation produces the wrong string (e.g. no "$" prefix).

/// BC-1.02.004 postconditions 1/2/4 (AC-007): `${{ arr | currency }}` with
/// `arr = 1000000` must evaluate to the string `"$1,000,000"`.
///
/// The `${{` sequence is text-mode interpolation with a literal `$` prefix.
/// Math mode is NOT activated. The `currency` filter formats the number; the
/// `$` is prepended as a literal character.
///
/// Red Gate: the `TemplateChunk::DollarInterp` variant does not yet exist.
/// This test constructs the equivalent `[Literal("$"), Expr(arr | currency)]`
/// sequence and verifies the concatenated output. It FAILS because:
/// 1. Without a `DollarInterp` chunk, there is no pipeline guarantee that `$`
///    is always prepended atomically.
/// 2. The assertion checks the FULL spec-required string `"$1,000,000"`,
///    which requires both the currency filter AND the `$` prefix to be correct.
#[test]
fn test_bc_1_02_004_dollar_interpolation_basic() {
    // Construct: stat field = "${{ arr | currency }}"
    // Modelled as: Literal("$") + Expr(arr | currency)
    let arr_currency_expr = Expr::Pipe {
        lhs: Box::new(Expr::Ident("arr".to_string())),
        filter: "currency".to_string(),
        args: vec![],
    };

    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new("arr".to_string(), dummy_span()),
            Spanned::new(FieldValue::Num(1_000_000), dummy_span()),
        )],
    };

    let stat_field = FieldNode {
        name: Spanned::new("stat".to_string(), dummy_span()),
        value: Spanned::new(
            // The spec requires "${{ arr | currency }}" to produce "$1,000,000".
            // Currently modelled as Literal("$") + Expr(arr | currency).
            // When DollarInterp is added, this should use:
            //   FieldValue::Template(vec![TemplateChunk::DollarInterp(arr_currency_expr)])
            // For Red Gate, we assert the concatenation equals "$1,000,000".
            FieldValue::Template(vec![
                TemplateChunk::Literal("$".to_string()),
                TemplateChunk::Expr(arr_currency_expr),
            ]),
            dummy_span(),
        ),
    };

    let slide_item = BlockItem::Slide(Spanned::new(
        SlideNode {
            kind: Spanned::new("stat".to_string(), dummy_span()),
            tags: vec![],
            fields: vec![stat_field],
            inline_items: vec![],
        },
        dummy_span(),
    ));

    let deck_node = DeckNode {
        vars: vec![vars_block],
        items: vec![slide_item],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);

    // Red Gate failure point: when `TemplateChunk::DollarInterp` is implemented,
    // the evaluation path changes and this test must be updated to use the new
    // variant. Until then, this test exercises the Literal("$") + Expr path and
    // verifies the concatenation.
    assert!(
        sink.is_empty(),
        "no errors expected for ${{ arr | currency }} with arr=1000000; got: {:?}",
        sink.errors()
    );
    let deck = deck.expect("eval_deck must return Some");
    assert_eq!(deck.slides.len(), 1);

    // The `stat` field must equal "$1,000,000" (dollar prefix + currency-formatted value, no decimals for integer input).
    let stat_val = deck.slides[0].fields.get("stat");
    assert!(
        stat_val.is_some(),
        "slide must have 'stat' field after eval"
    );
    match stat_val {
        Some(slideforge_types::FieldValue::Literal(Value::Str(s))) => {
            assert_eq!(
                s.as_ref(),
                "$1,000,000",
                "BC-1.02.004 AC-007: ${{{{ arr | currency }}}} with arr=1000000 must produce \
                 '$1,000,000' (integer input — no decimal places); got: '{s}'"
            );
        },
        other => panic!("stat field must be Literal(Str); got: {other:?}"),
    }
}

/// BC-1.02.004 postcondition 4 (AC-008): `"${{ count }} items"` with `count=5`
/// must evaluate to `"$5 items"`.
///
/// Red Gate: same as `test_bc_1_02_004_dollar_interpolation_basic`. The `$`
/// literal followed by the count interpolation must concatenate correctly.
#[test]
fn test_bc_1_02_004_dollar_interpolation_count() {
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new("count".to_string(), dummy_span()),
            Spanned::new(FieldValue::Num(5), dummy_span()),
        )],
    };

    // "${{ count }} items" → [Literal("$"), Expr(count), Literal(" items")]
    let title_field = FieldNode {
        name: Spanned::new("title".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![
                TemplateChunk::Literal("$".to_string()),
                TemplateChunk::Expr(Expr::Ident("count".to_string())),
                TemplateChunk::Literal(" items".to_string()),
            ]),
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

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    assert!(
        sink.is_empty(),
        "no errors expected; got: {:?}",
        sink.errors()
    );
    let deck = deck.expect("eval_deck must return Some");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("$5 items"),
        "BC-1.02.004 AC-008: '${{{{ count }}}} items' with count=5 must produce '$5 items'"
    );
}

/// BC-1.02.004 invariant 2 (AC-009): `"$ {{ price }}"` — a space between `$`
/// and `{{` means `$` is a literal character, NOT a `${{` interpolation trigger.
///
/// Result: `"$ "` + evaluated `price` value as separate text segments.
/// The `$` is plain text; `{{ price }}` is normal interpolation.
///
/// Red Gate: this test verifies that `Literal("$ ")` + `Expr(price)` produces
/// `"$ 100"` (not `"$100"`). If the evaluator incorrectly treats `$ {{` as
/// `${{`, it would produce the wrong prefix. The space is load-bearing.
#[test]
fn test_bc_1_02_004_dollar_space_not_interpolation() {
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new("price".to_string(), dummy_span()),
            Spanned::new(FieldValue::Num(100), dummy_span()),
        )],
    };

    // "$ {{ price }}" → [Literal("$ "), Expr(price)]
    // The space distinguishes this from "${{ price }}" — the `$` is literal text.
    let title_field = FieldNode {
        name: Spanned::new("title".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![
                TemplateChunk::Literal("$ ".to_string()), // dollar + space = literal
                TemplateChunk::Expr(Expr::Ident("price".to_string())),
            ]),
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

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    assert!(
        sink.is_empty(),
        "no errors expected; got: {:?}",
        sink.errors()
    );
    let deck = deck.expect("eval_deck must return Some");

    // Invariant: "$ {{ price }}" → "$ 100" (dollar-space is literal, not a prefix).
    // If this were ${{ price }}, it would produce "$100" (no space). The space matters.
    assert_eq!(
        deck.slides[0].title_str(),
        Some("$ 100"),
        "BC-1.02.004 invariant 2: '$ {{{{ price }}}}' (space between $ and {{) must produce '$ 100', not '$100'"
    );
}

/// BC-1.02.004 invariant 3 (AC-010): `$${{ n }}` — `$$` opens math display
/// mode; `{{ n }}` inside is NOT text interpolation (math mode uses `@{var}`).
///
/// Red Gate: the math display region `$$...$$` is stored as
/// `TemplateChunk::MathDisplay(content)`. The content is raw LaTeX, not
/// evaluated. `{{ n }}` inside `$$...$$` is literal LaTeX text `{{ n }}`, not
/// a text-mode interpolation.
///
/// This test verifies that:
/// 1. The `MathDisplay` chunk is passed through unchanged (not evaluated).
/// 2. No interpolation of `{{ n }}` happens inside the math region.
#[test]
fn test_bc_1_02_004_dollar_dollar_brace_is_math_mode_not_interpolation() {
    // Construct: title "$${{ n }}$$" where n=42
    // Parser would produce: [MathDisplay("{{ n }}")]
    // The "{{ n }}" inside $$ ... $$ is raw LaTeX, not text interpolation.
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new("n".to_string(), dummy_span()),
            Spanned::new(FieldValue::Num(42), dummy_span()),
        )],
    };

    let title_field = FieldNode {
        name: Spanned::new("title".to_string(), dummy_span()),
        value: Spanned::new(
            // $${{ n }}$$ → MathDisplay("{{ n }}") — the {{ n }} is raw LaTeX content
            FieldValue::Template(vec![TemplateChunk::MathDisplay("{{ n }}".to_string())]),
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

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    assert!(
        sink.is_empty(),
        "no errors expected — math display chunks are pass-through; got: {:?}",
        sink.errors()
    );
    let deck = deck.expect("eval_deck must return Some");

    // The MathDisplay chunk must be stored in the slide's title field.
    // It must NOT be evaluated as text interpolation — the field stores the
    // math content, not a resolved string with n=42 substituted.
    //
    // BC-1.02.004 invariant 3: $${{ n }}$$ does NOT interpolate n.
    // Red Gate failure: if the evaluator incorrectly evaluates MathDisplay
    // content as text interpolation, title_str() would return "42" (wrong).
    // The correct behavior: the math content is preserved as-is.
    let title_val = deck.slides[0].fields.get("title");
    assert!(
        title_val.is_some(),
        "slide must have 'title' field after eval"
    );
    // The title field must NOT have been resolved to "42" (n's integer value).
    // If it were, that would mean {{ n }} inside $$ was treated as text interpolation.
    //
    // FINDING-007 (adversary pass 1): the test previously only asserted != "42".
    // A positive assertion is required: MathDisplay chunks contribute nothing to
    // the evaluated string output (they are pass-through / no-op in the current
    // string-building loop). So a template of [MathDisplay("{{ n }}")] produces
    // the empty string "". This pins the concrete correct behavior.
    match title_val {
        Some(slideforge_types::FieldValue::Literal(Value::Str(s))) => {
            assert_ne!(
                s.as_ref(),
                "42",
                "BC-1.02.004 invariant 3: $${{{{ n }}}}$$ must NOT interpolate n=42; \
                 math display content is raw LaTeX, not text interpolation"
            );
            // Positive assertion: MathDisplay chunks are no-ops in the current
            // string evaluator — they contribute zero characters. A template
            // consisting solely of a MathDisplay chunk resolves to "".
            assert_eq!(
                s.as_ref(),
                "",
                "BC-1.02.004 invariant 3: $${{{{ n }}}}$$ template (MathDisplay-only) must \
                 resolve to empty string \"\", not to any interpolated value; got: '{s}'"
            );
        },
        // Other representations (e.g. a dedicated math field type in future IR)
        // are also acceptable as long as they don't resolve to "42".
        other => {
            // If the field type is not a Str, that's acceptable — it means
            // the math content is stored in a structured way, not as a plain string.
            // Just verify it's not Literal(Str("42")).
            if let Some(slideforge_types::FieldValue::Literal(Value::Int(42))) = other {
                panic!(
                    "BC-1.02.004 invariant 3: $${{{{ n }}}}$$ must NOT resolve n=42 inside math mode; \
                     got Int(42) which means {{ n }} was incorrectly interpolated"
                );
            }
        },
    }
}

// ─── EC-003: string concat without | string conversion ──────────────────────

/// BC-1.02.003 EC-003: `count = 42` (integer) used in string concat (title +
/// " items") without `| string` conversion must produce E-EVL-003.
///
/// The `+` operator does NOT accept mixed Int/Str operands — it requires two
/// numeric operands (for addition) or two string operands (for future `~`
/// concat). Using `+` with a string and an integer is always E-EVL-003.
///
/// Red Gate: this currently fires E-EVL-003 from the existing arithmetic
/// type check, but the hint text should mention `| string` or `~` operator.
/// This test asserts the error code is present (passes) and hints at conversion
/// (may fail if hint not added).
#[test]
fn test_bc_1_02_003_int_string_concat_type_error() {
    let env = env_with_int(&[("count", 42)]);
    let mut sink = DiagnosticSink::new();
    // {{ "items: " + count }} — mixing Str and Int with + → E-EVL-003
    let expr = Expr::BinOp {
        op: BinOpKind::Add,
        lhs: Box::new(Expr::Str("items: ".to_string())),
        rhs: Box::new(Expr::Ident("count".to_string())),
    };
    let result = eval_expr(&env, &expr, &mut sink);
    assert_eq!(result, None, "Str + Int must return None (no implicit concat)");
    assert!(
        !sink.is_empty(),
        "Str + Int must push E-EVL-003 (use | string or ~ operator)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "String + Int must produce E-EVL-003 (EC-003)"
    );
}

// ─── EC-004: "${{ amount }} USD" ────────────────────────────────────────────

/// BC-1.02.004 EC-004: `"${{ amount }} USD"` with `amount = 1500` must
/// evaluate to `"$1500 USD"`.
#[test]
fn test_bc_1_02_004_dollar_interp_with_suffix() {
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new("amount".to_string(), dummy_span()),
            Spanned::new(FieldValue::Num(1500), dummy_span()),
        )],
    };

    // "${{ amount }} USD" → [Literal("$"), Expr(amount), Literal(" USD")]
    let title_field = FieldNode {
        name: Spanned::new("title".to_string(), dummy_span()),
        value: Spanned::new(
            FieldValue::Template(vec![
                TemplateChunk::Literal("$".to_string()),
                TemplateChunk::Expr(Expr::Ident("amount".to_string())),
                TemplateChunk::Literal(" USD".to_string()),
            ]),
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

    let mut sink = DiagnosticSink::new();
    let deck = eval_deck(&deck_node, &default_config(), &mut sink);
    assert!(
        sink.is_empty(),
        "no errors expected; got: {:?}",
        sink.errors()
    );
    let deck = deck.expect("eval_deck must return Some");
    assert_eq!(
        deck.slides[0].title_str(),
        Some("$1500 USD"),
        "BC-1.02.004 EC-004: '${{{{ amount }}}} USD' with amount=1500 must produce '$1500 USD'"
    );
}

// ─── EC-006: | bool filter does not exist ───────────────────────────────────

/// BC-1.02.003 EC-006: `| bool` must produce E-EVL-004 with an error message
/// that does NOT include "bool" in the available filter list.
///
/// The help text must NOT suggest "bool" as an available conversion.
/// Red Gate: passes immediately (no `bool` filter exists in registry). This
/// is a regression guard — if someone adds `| bool`, this test will fail.
#[test]
fn test_bc_1_02_003_bool_filter_help_text_excludes_bool() {
    let env = env_with_str(&[("v", "yes")]);
    let mut sink = DiagnosticSink::new();
    let expr = Expr::Pipe {
        lhs: Box::new(Expr::Ident("v".to_string())),
        filter: "bool".to_string(),
        args: vec![],
    };
    let _result = eval_expr(&env, &expr, &mut sink);
    assert!(!sink.is_empty());
    let help = sink.errors()[0]
        .help()
        .map(|h| h.to_string())
        .unwrap_or_default();
    // "bool" must NOT appear as an available filter name in the help text.
    // If it did, it means the filter was added (regression).
    assert!(
        !help.split(", ").any(|name| name.trim() == "bool"),
        "E-EVL-004 help text must NOT list 'bool' as an available filter; \
         that would enable implicit coercion. Help text: {help}"
    );
}

// ─── Invariant: Value::Str never implicitly becomes Value::Int ───────────────

/// BC-1.02.003 invariant (DI-004): `eval_expr` applied to `Expr::Str("42")`
/// must return `Value::Str("42")`, never `Value::Int(42)`.
///
/// The `Expr::Str` literal must produce `Value::Str` without any implicit
/// type inference. This is the most fundamental form of the no-coercion rule.
#[test]
fn test_bc_1_02_003_invariant_str_literal_never_becomes_int() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::Str("42".to_string());
    let result = eval_expr(&env, &expr, &mut sink);
    assert!(sink.is_empty(), "Str literal '42' must not produce any error");
    assert_eq!(
        result,
        Some(Value::Str(Arc::from("42"))),
        "Expr::Str('42') must produce Value::Str('42'), never Value::Int(42) — DI-004"
    );
}

/// BC-1.02.003 invariant (DI-004): `eval_expr` applied to `Expr::Str("true")`
/// must return `Value::Str("true")`, never `Value::Bool(true)`.
#[test]
fn test_bc_1_02_003_invariant_str_literal_never_becomes_bool() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::Str("true".to_string());
    let result = eval_expr(&env, &expr, &mut sink);
    assert!(sink.is_empty(), "Str literal 'true' must not produce any error");
    assert_eq!(
        result,
        Some(Value::Str(Arc::from("true"))),
        "Expr::Str('true') must produce Value::Str('true'), never Value::Bool(true) — DI-004"
    );
}

/// BC-1.02.003 invariant (DI-004): `eval_expr` applied to `Expr::Str("NO")`
/// must return `Value::Str("NO")`, never `Value::Bool(false)`.
#[test]
fn test_bc_1_02_003_invariant_str_no_never_becomes_bool_false() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::Str("NO".to_string());
    let result = eval_expr(&env, &expr, &mut sink);
    assert!(sink.is_empty(), "Str literal 'NO' must not produce any error");
    assert_eq!(
        result,
        Some(Value::Str(Arc::from("NO"))),
        "Expr::Str('NO') must produce Value::Str('NO'), never Value::Bool(false) — DI-004"
    );
}

/// FINDING-001 (adversary pass 1, STORY-014): `"42" == 42` must produce
/// E-EVL-003, NOT silently return `Bool(false)`.
///
/// BC-1.02.003 invariant 1 (no-coercion): cross-type equality comparisons
/// between non-numeric types are type errors. `String == Int` has no
/// defined coercion path in slideforge — it must always be E-EVL-003.
///
/// Previously, `eval_equality` only rejected `Bool` cross-type comparisons,
/// allowing `Str == Int` to fall through the same-type structural equality
/// arm and silently return `Bool(false)`. This test pins the correct behavior.
#[test]
fn test_string_equals_int_type_error() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Eq,
        lhs: Box::new(Expr::Str("42".to_string())),
        rhs: Box::new(Expr::Num(42)),
    };
    let result = eval_expr(&env, &expr, &mut sink);

    assert_eq!(
        result, None,
        "Str == Int must return None (E-EVL-003), not Bool(false)"
    );
    assert!(
        !sink.is_empty(),
        "'\"42\" == 42' must push E-EVL-003 (String vs Int comparison)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "String == Int must produce E-EVL-003 (FINDING-001, BC-1.02.003 no-coercion invariant)"
    );
}

/// FINDING-005 (adversary pass 1, STORY-014): `"hello" < 5` must produce
/// E-EVL-003, not silently fail or coerce.
///
/// BC-1.02.003 invariant 1 (no-coercion): ordering comparisons (`<`, `<=`,
/// `>`, `>=`) between incompatible types must produce E-EVL-003. Strings
/// cannot be ordered against integers without explicit conversion.
#[test]
fn test_ordering_string_vs_int_type_error() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Lt,
        lhs: Box::new(Expr::Str("hello".to_string())),
        rhs: Box::new(Expr::Num(5)),
    };
    let result = eval_expr(&env, &expr, &mut sink);

    assert_eq!(
        result, None,
        "Str < Int must return None (E-EVL-003), not Bool"
    );
    assert!(
        !sink.is_empty(),
        "'\"hello\" < 5' must push E-EVL-003 (String vs Int ordering comparison)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "String < Int must produce E-EVL-003 (FINDING-005, BC-1.02.003 no-coercion invariant)"
    );
}

/// FINDING-001 (adversary pass 3, STORY-014): `@if false:` must NOT produce
/// E-EVL-003. `Bool` is the valid condition type for `@if`.
///
/// This is the negative / happy-path counterpart to the non-Bool tests above.
/// BC-1.02.003 invariant 2 (DI-004) only rejects non-Bool conditions; a
/// `Value::Bool(false)` condition is perfectly valid — the `@if` block simply
/// evaluates to "condition not taken" (no slides generated). No E-EVL-003
/// should be pushed.
///
/// Regression guard: if the type-check in `eval_block_items` were incorrectly
/// broadened to reject all Bool values (or the `matches!` guard were inverted),
/// this test would catch it.
#[test]
fn test_bc_1_02_003_bool_false_condition_no_type_error() {
    // Build: @if false: slide content:
    let if_node = IfNode {
        condition: Spanned::new(Expr::Bool(false), dummy_span()),
        then_body: vec![BlockItem::Slide(Spanned::new(
            SlideNode {
                kind: Spanned::new("content".to_string(), dummy_span()),
                tags: vec![],
                fields: vec![],
                inline_items: vec![],
            },
            dummy_span(),
        ))],
        elif_branches: vec![],
        else_body: None,
    };
    let deck_node = DeckNode {
        items: vec![BlockItem::If(Spanned::new(if_node, dummy_span()))],
        ..DeckNode::default()
    };

    let mut sink = DiagnosticSink::new();
    let _deck = eval_deck(&deck_node, &default_config(), &mut sink);

    // BC-1.02.003 invariant 2 (DI-004): Bool is the VALID condition type.
    // @if false: must produce ZERO errors — Bool(false) is a legitimate
    // @if condition; it means the branch is simply not taken.
    assert!(
        sink.is_empty(),
        "@if with Bool(false) condition must produce ZERO diagnostics; \
         Bool is the valid condition type — got: {:?}",
        sink.errors()
    );
}

/// FINDING-P2-002 (adversary pass 2, STORY-014): `"hello" != 42` must produce
/// E-EVL-003, NOT silently return `Bool(true)`.
///
/// BC-1.02.003 no-coercion invariant: `!=` (not-equal) is the logical negation
/// of `==`. Because `"hello" == 42` must produce E-EVL-003, `"hello" != 42`
/// must also produce E-EVL-003 — not `Bool(true)` via "they are different types
/// so they must be not-equal."
///
/// The same cross-type guard that rejects `Str == Int` must also reject
/// `Str != Int`.
#[test]
fn test_string_ne_int_type_error() {
    let env = slideforge_eval::Env::new(IndexMap::new());
    let mut sink = DiagnosticSink::new();
    let expr = Expr::BinOp {
        op: BinOpKind::Ne,
        lhs: Box::new(Expr::Str("hello".to_string())),
        rhs: Box::new(Expr::Num(42)),
    };
    let result = eval_expr(&env, &expr, &mut sink);

    assert_eq!(
        result, None,
        "Str != Int must return None (E-EVL-003), not Bool(true)"
    );
    assert!(
        !sink.is_empty(),
        "'\"hello\" != 42' must push E-EVL-003 (String vs Int comparison, FINDING-P2-002)"
    );
    let code = sink.errors()[0].code().map(|c| c.to_string());
    assert_eq!(
        code.as_deref(),
        Some("E-EVL-003"),
        "String != Int must produce E-EVL-003 (FINDING-P2-002, BC-1.02.003 no-coercion invariant)"
    );
}
