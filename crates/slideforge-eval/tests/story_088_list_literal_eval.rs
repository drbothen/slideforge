//! STORY-088 Evaluator tests: `FieldValue::List` → `Value::List` dispatch.
//!
//! ## Acceptance Criteria covered
//!
//! - **AC-001 / AC-003**: `FieldValue::List([Template("Item A"), ...])` stored in a
//!   vars block evaluates to `Value::List([Value::Str("Item A"), ...])`.
//! - **AC-002**: `FieldValue::List([])` evaluates to `Value::List(vec![])`.
//!
//! ## Red Gate discipline
//!
//! All tests MUST FAIL until the implementer adds the `FieldValue::List` arm to
//! `eval_field_value_to_value` in `eval.rs`. The stub arm returns `None`, which
//! means the variable is not stored in the env → `FieldValue::Ident("items")`
//! lookup produces an `UndefinedVariable` error → `sink.is_empty()` is false →
//! the assertions FAIL.
//!
//! ## Traceability
//!
//! BC-1.01.002 (field-value parser — STORY-088 scope);
//! BC-1.16.001 PC-7 (Value::List → ContentBlock::Bullets, via Stage 2b).

#![allow(clippy::unwrap_used)] // test helpers — explicit panic on failure is correct

use std::sync::Arc;

use slideforge_eval::{EvalConfig, eval_deck};
use slideforge_syntax::{
    BlockItem, DiagnosticSink, DeckNode, FieldNode, FieldValue, SlideNode, Spanned, TemplateChunk,
    VarsBlock,
};
use slideforge_syntax::span::Span;
use slideforge_types::Value;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    Span::new(0, 0, 0)
}

fn make_string_template(s: &str) -> FieldValue {
    FieldValue::Template(vec![TemplateChunk::Literal(s.to_string())])
}

fn make_field(name: &str, value: FieldValue) -> FieldNode {
    FieldNode {
        name: Spanned::new(name.to_string(), dummy_span()),
        value: Spanned::new(value, dummy_span()),
    }
}

/// Build a minimal `DeckNode` with a vars block containing one entry.
///
/// The vars block entry has `name = var_name` and `value = var_value`.
/// The deck also contains one `slide content:` block with a `bullets: items`
/// field referencing the variable.
fn make_deck_with_var(var_name: &str, var_value: FieldValue) -> DeckNode {
    let mut deck = DeckNode::default();

    // Add a vars block with the given variable.
    let vars_block = VarsBlock {
        entries: vec![(
            Spanned::new(var_name.to_string(), dummy_span()),
            Spanned::new(var_value, dummy_span()),
        )],
    };
    deck.vars.push(vars_block);

    // Add a `slide content:` block with a `bullets: <var_name>` field.
    let slide_node = SlideNode {
        kind: Spanned::new("content".to_string(), dummy_span()),
        tags: vec![],
        fields: vec![
            make_field("title", make_string_template("Test Slide")),
            // Reference the variable by ident.
            make_field("bullets", FieldValue::Ident(var_name.to_string())),
        ],
        inline_items: vec![],
    };
    deck.items.push(BlockItem::Slide(Spanned::new(slide_node, dummy_span())));
    deck.lang = Some(Spanned::new("en-US".to_string(), dummy_span()));

    deck
}

// ─── AC-001/003: FieldValue::List → Value::List via vars block ───────────────

/// BC-1.01.002 AC-001 (eval side) — a `FieldValue::List([Template("Item A"),
/// Template("Item B"), Template("Item C")])` stored in a vars block entry
/// must evaluate to `Value::List([Value::Str("Item A"), ...])` so that the
/// `bullets: items` ident reference resolves correctly.
///
/// RED GATE: `eval_field_value_to_value` returns `None` for `FieldValue::List`
/// (stub arm in eval.rs) → the variable is not stored in env →
/// `FieldValue::Ident("items")` lookup produces `UndefinedVariable` →
/// `sink.is_empty()` is false → assertion FAILS.
#[test]
fn test_bc_1_01_002_eval_field_value_list_to_value_list_three_items() {
    let var_value = FieldValue::List(vec![
        make_string_template("Item A"),
        make_string_template("Item B"),
        make_string_template("Item C"),
    ]);

    let deck_node = make_deck_with_var("items", var_value);
    let config = EvalConfig::default();
    let mut sink = DiagnosticSink::new();

    let deck_opt = eval_deck(&deck_node, &config, &mut sink);

    // RED GATE: stub returns None → variable not stored → UndefinedVariable error
    // → sink is not empty.
    assert!(
        sink.is_empty(),
        "AC-001 eval RED GATE: eval_deck must NOT produce errors when vars block \
         contains FieldValue::List(3 items). Got errors: {:?}. \
         The stub FieldValue::List arm returns None → variable not stored → \
         'bullets: items' ident lookup → UndefinedVariable.",
        sink.errors()
    );

    let deck = deck_opt.expect(
        "AC-001 eval RED GATE: eval_deck must return Some(Deck) when vars block \
         contains FieldValue::List([...]) → after implementation Value::List is \
         stored in env and the slide evaluates successfully."
    );

    // The slide's `bullets` field must resolve to a Value::List with 3 items.
    assert_eq!(deck.slides.len(), 1, "AC-001 eval: deck must have 1 slide");
    let slide = &deck.slides[0];

    let bullets_field = slide.fields.get("bullets").expect(
        "AC-001 eval: slide must have a 'bullets' field after eval. \
         The FieldValue::Ident('items') must resolve to Value::List."
    );

    // The resolved field must be FieldValue::Literal(Value::List([...])).
    let slideforge_types::FieldValue::Literal(value) = bullets_field else {
        panic!(
            "AC-001 eval: bullets field must be FieldValue::Literal(Value::List); \
             got: {bullets_field:?}"
        );
    };
    let Value::List(items) = value else {
        panic!(
            "AC-001 eval: bullets field literal must be Value::List; got: {value:?}. \
             After implementation, Value::List([Str(\"Item A\"), ...]) is expected."
        );
    };
    assert_eq!(items.len(), 3, "AC-001 eval: Value::List must have 3 items");
    assert_eq!(items[0], Value::Str(Arc::from("Item A")), "AC-001 eval: item 0");
    assert_eq!(items[1], Value::Str(Arc::from("Item B")), "AC-001 eval: item 1");
    assert_eq!(items[2], Value::Str(Arc::from("Item C")), "AC-001 eval: item 2");
}

/// BC-1.01.002 AC-002 (eval side) — `FieldValue::List([])` (empty list)
/// evaluates to `Value::List(vec![])`.
///
/// RED GATE: stub returns `None` for `FieldValue::List` → variable not stored
/// → UndefinedVariable error → sink not empty.
#[test]
fn test_bc_1_01_002_eval_field_value_list_empty_to_value_list_empty() {
    let var_value = FieldValue::List(vec![]);
    let deck_node = make_deck_with_var("items", var_value);
    let config = EvalConfig::default();
    let mut sink = DiagnosticSink::new();

    let deck_opt = eval_deck(&deck_node, &config, &mut sink);

    assert!(
        sink.is_empty(),
        "AC-002 eval RED GATE: empty FieldValue::List must evaluate without errors. \
         Got: {:?}",
        sink.errors()
    );
    let deck = deck_opt.expect(
        "AC-002 eval: eval_deck must return Some(Deck) for empty list variable"
    );

    assert_eq!(deck.slides.len(), 1, "AC-002 eval: deck must have 1 slide");
    let slide = &deck.slides[0];
    let bullets_field = slide
        .fields
        .get("bullets")
        .expect("AC-002 eval: bullets field must exist");

    let slideforge_types::FieldValue::Literal(Value::List(items)) = bullets_field else {
        panic!(
            "AC-002 eval: bullets field must be FieldValue::Literal(Value::List([])); \
             got: {bullets_field:?}"
        );
    };
    assert!(
        items.is_empty(),
        "AC-002 eval: Value::List from empty FieldValue::List must be empty"
    );
}

/// BC-1.01.002 AC-003 (eval side) — single-item `FieldValue::List([Template("Only")])`
/// evaluates to `Value::List([Value::Str("Only")])`.
///
/// RED GATE: stub returns `None` → variable not in env → errors.
#[test]
fn test_bc_1_01_002_eval_field_value_list_single_item_to_value_list() {
    let var_value = FieldValue::List(vec![make_string_template("Only")]);
    let deck_node = make_deck_with_var("items", var_value);
    let config = EvalConfig::default();
    let mut sink = DiagnosticSink::new();

    let deck_opt = eval_deck(&deck_node, &config, &mut sink);

    assert!(
        sink.is_empty(),
        "AC-003 eval RED GATE: single-item FieldValue::List must evaluate without errors. \
         Got: {:?}",
        sink.errors()
    );
    let deck = deck_opt.expect("AC-003 eval: eval_deck must return Some");
    let slide = &deck.slides[0];
    let bullets_field = slide
        .fields
        .get("bullets")
        .expect("AC-003 eval: bullets field must exist");

    let slideforge_types::FieldValue::Literal(Value::List(items)) = bullets_field else {
        panic!(
            "AC-003 eval: single-item list must produce Value::List([Str(\"Only\")]); \
             got: {bullets_field:?}"
        );
    };
    assert_eq!(items.len(), 1, "AC-003 eval: single-item list must have 1 item");
    assert_eq!(
        items[0],
        Value::Str(Arc::from("Only")),
        "AC-003 eval: item text"
    );
}
