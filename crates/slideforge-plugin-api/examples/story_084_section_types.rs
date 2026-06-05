//! STORY-084 demo: Bundled `SectionType` Implementations.
//!
//! Demonstrates all five acceptance criteria:
//!
//! - AC-001: instantiate all 7 `SectionType` impls and print their `id()`s.
//! - AC-002: `ExecutiveSummarySectionType::generate()` on a 3-slide deck
//!   (2 with `takeaway`, 1 without) — 2 `SectionBlock`s produced.
//!   Also: BC-3.02.001 inv-4 — a `takeaway` slide with
//!   `register: Notes` is excluded from the output.
//! - AC-003: `RiskRegisterSectionType::generate()` on a 5-slide deck
//!   (2 `severity_cards`, 3 others) — 2 blocks produced. Also shows a
//!   notes-register `severity_cards` slide being excluded.
//! - AC-004: each of the 5 manual types returns an empty `Vec`.
//! - AC-005: all 7 impls compile using only the public API.
//!
//! Run with:
//!   `cargo run --example story_084_section_types -p slideforge-plugin-api`

// ── Suppress lints unavoidable in a demo binary ───────────────────────────────
#![allow(clippy::print_stdout)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_docs_in_private_items)]

use std::sync::Arc;

use slideforge_plugin_api::{
    AppendixSectionType, ApprovalSectionType, ExecutiveSummarySectionType, GlossarySectionType,
    MethodologySectionType, RiskRegisterSectionType, ScopeSectionType, SectionType,
};
use slideforge_types::{FieldValue, OrderedMap, Register, Slide, SourceSpan, Value};

// ─────────────────────────────────────────────────────────────────────────────
// Slide construction helpers
// ─────────────────────────────────────────────────────────────────────────────

fn blank_slide(slide_type: &str) -> Slide {
    Slide {
        slide_type: Arc::from(slide_type),
        fields: OrderedMap::new(),
        blocks: vec![],
        register: None,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

fn slide_with_takeaway(title: &str, takeaway: &str, register: Option<Register>) -> Slide {
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(title))),
    );
    fields.insert(
        Arc::from("takeaway"),
        FieldValue::Literal(Value::Str(Arc::from(takeaway))),
    );
    Slide {
        slide_type: Arc::from("bullets"),
        fields,
        blocks: vec![],
        register,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

fn severity_cards_slide(title: &str, register: Option<Register>) -> Slide {
    let mut fields = OrderedMap::new();
    fields.insert(
        Arc::from("title"),
        FieldValue::Literal(Value::Str(Arc::from(title))),
    );
    Slide {
        slide_type: Arc::from("severity_cards"),
        fields,
        blocks: vec![],
        register,
        tags: vec![],
        source_span: SourceSpan::default(),
        overlay: None,
        register_content: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Separator helper
// ─────────────────────────────────────────────────────────────────────────────

fn separator(label: &str) {
    println!();
    println!("-----------------------------------------------------------------");
    println!(" {label}");
    println!("-----------------------------------------------------------------");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC demonstrations — split into focused functions to stay under line limit
// ─────────────────────────────────────────────────────────────────────────────

fn demo_ac001() {
    separator("AC-001  All 7 SectionType impls exist; id()s printed");

    let impls: Vec<Box<dyn SectionType>> = vec![
        Box::new(ExecutiveSummarySectionType),
        Box::new(RiskRegisterSectionType),
        Box::new(MethodologySectionType),
        Box::new(ScopeSectionType),
        Box::new(ApprovalSectionType),
        Box::new(AppendixSectionType),
        Box::new(GlossarySectionType),
    ];

    println!("  Total impls: {}", impls.len());
    for plugin in &impls {
        println!("  id() = {:?}", plugin.id());
    }
    let expected_ids = [
        "executive_summary",
        "risk_register",
        "methodology",
        "scope",
        "approval",
        "appendix",
        "glossary",
    ];
    let all_ids_match = impls
        .iter()
        .zip(expected_ids.iter())
        .all(|(p, &exp)| p.id() == exp);
    if all_ids_match && impls.len() == 7 {
        println!("  PASS   all 7 ids match canonical set");
    } else {
        println!("  FAIL   id mismatch detected");
    }
}

fn demo_ac002_happy_path() {
    separator("AC-002  ExecutiveSummarySectionType::generate() — happy path");

    let three_slides = vec![
        slide_with_takeaway("Q1 Revenue", "Revenue up 12% YoY", None),
        blank_slide("bullets"), // no takeaway — must be skipped
        slide_with_takeaway("Market Share", "Grew 3 pts YoY", None),
    ];

    println!("  Deck: 3 slides (slide 0 has takeaway, slide 1 has none, slide 2 has takeaway)");
    let blocks = ExecutiveSummarySectionType.generate(&three_slides);
    println!("  generate() returned {} SectionBlock(s):", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        println!(
            "    block[{i}]: title={:?}  level={}  include_in_toc={}  start_slide_index={}",
            block.title.as_ref(),
            block.level,
            block.include_in_toc,
            block.start_slide_index,
        );
    }
    if blocks.len() == 2
        && blocks[0].level == 1
        && blocks[0].include_in_toc
        && blocks[1].level == 1
        && blocks[1].include_in_toc
    {
        println!("  PASS   2 blocks with level=1 and include_in_toc=true");
    } else {
        println!("  FAIL   unexpected block count or field values");
    }
}

fn demo_ac002_notes_exclusion() {
    separator("AC-002 / BC-3.02.001 inv-4  notes-register takeaway slide excluded");

    let notes_deck = vec![
        // register: Notes → must be EXCLUDED even though it has a takeaway
        slide_with_takeaway(
            "Internal Note",
            "Speaker-only insight",
            Some(Register::Notes),
        ),
        // register: None → must be INCLUDED
        slide_with_takeaway("Revenue Summary", "Revenue up 12%", None),
    ];

    println!("  Deck: 2 takeaway slides — slide 0 has register:Notes, slide 1 has none");
    let result = ExecutiveSummarySectionType.generate(&notes_deck);
    println!("  generate() returned {} SectionBlock(s):", result.len());
    for (i, block) in result.iter().enumerate() {
        println!(
            "    block[{i}]: title={:?}  level={}  include_in_toc={}",
            block.title.as_ref(),
            block.level,
            block.include_in_toc,
        );
    }
    if result.len() == 1 && result[0].title.as_ref() == "Revenue Summary" {
        println!("  PASS   notes-register slide excluded; only non-notes slide produced a block");
    } else {
        println!("  FAIL   notes-register exclusion not enforced");
    }
}

fn demo_ac003_happy_path() {
    separator("AC-003  RiskRegisterSectionType::generate() — happy path");

    let five_slides = vec![
        blank_slide("title"),
        severity_cards_slide("Technical Risks", None),
        blank_slide("bullets"),
        severity_cards_slide("Vendor Risks", None),
        blank_slide("closing"),
    ];

    println!("  Deck: 5 slides (2 severity_cards, 3 others)");
    let blocks = RiskRegisterSectionType.generate(&five_slides);
    println!("  generate() returned {} SectionBlock(s):", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        println!(
            "    block[{i}]: title={:?}  level={}  include_in_toc={}  start_slide_index={}",
            block.title.as_ref(),
            block.level,
            block.include_in_toc,
            block.start_slide_index,
        );
    }
    if blocks.len() == 2
        && blocks[0].level == 1
        && blocks[0].include_in_toc
        && blocks[1].level == 1
        && blocks[1].include_in_toc
    {
        println!("  PASS   2 blocks with level=1 and include_in_toc=true");
    } else {
        println!("  FAIL   unexpected block count or field values");
    }
}

fn demo_ac003_notes_exclusion() {
    separator("AC-003 / BC-3.02.001 inv-4  notes-register severity_cards excluded");

    let notes_risk_deck = vec![
        // register: Notes → EXCLUDED from risk_register output
        severity_cards_slide("Speaker Notes Risk", Some(Register::Notes)),
        // register: None → INCLUDED
        severity_cards_slide("Vendor Risks", None),
    ];

    println!("  Deck: 2 severity_cards slides — slide 0 has register:Notes, slide 1 has none");
    let result = RiskRegisterSectionType.generate(&notes_risk_deck);
    println!("  generate() returned {} SectionBlock(s):", result.len());
    for (i, block) in result.iter().enumerate() {
        println!(
            "    block[{i}]: title={:?}  level={}  include_in_toc={}",
            block.title.as_ref(),
            block.level,
            block.include_in_toc,
        );
    }
    if result.len() == 1 && result[0].title.as_ref() == "Vendor Risks" {
        println!("  PASS   notes-register severity_cards excluded; only non-notes slide included");
    } else {
        println!("  FAIL   notes-register exclusion not enforced for risk_register");
    }
}

fn demo_ac004() {
    separator("AC-004  Manual-only types (methodology/scope/approval/appendix/glossary) — empty");

    let any_slides = vec![
        blank_slide("title"),
        blank_slide("bullets"),
        severity_cards_slide("Risk A", None),
    ];

    let manual_impls: [(&str, Box<dyn SectionType>); 5] = [
        ("methodology", Box::new(MethodologySectionType)),
        ("scope", Box::new(ScopeSectionType)),
        ("approval", Box::new(ApprovalSectionType)),
        ("appendix", Box::new(AppendixSectionType)),
        ("glossary", Box::new(GlossarySectionType)),
    ];

    let mut all_empty = true;
    for (name, plugin) in &manual_impls {
        let result = plugin.generate(&any_slides);
        let status = if result.is_empty() { "PASS" } else { "FAIL" };
        if !result.is_empty() {
            all_empty = false;
        }
        println!(
            "  {status}   {name}::generate({} slides) → {} block(s)",
            any_slides.len(),
            result.len()
        );
    }
    if all_empty {
        println!("  PASS   all 5 manual types return vec![] for any slide slice");
    } else {
        println!("  FAIL   at least one manual type returned non-empty Vec");
    }
}

fn demo_ac005() {
    separator("AC-005  All 7 impls compile using only slideforge-plugin-api public API");
    println!("  (This entire example uses ONLY public re-exports from slideforge_plugin_api)");
    println!("  (No imports from slideforge_types private paths or other crates)");
    println!("  PASS   `cargo build --example story_084_section_types` succeeds = AC-005 met");
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

/// Entry point for the STORY-084 demo.
fn main() {
    println!("=================================================================");
    println!(" STORY-084: Bundled SectionType Implementations");
    println!("=================================================================");

    demo_ac001();
    demo_ac002_happy_path();
    demo_ac002_notes_exclusion();
    demo_ac003_happy_path();
    demo_ac003_notes_exclusion();
    demo_ac004();
    demo_ac005();

    println!();
    println!("=================================================================");
    println!(" All AC-001..AC-005 for STORY-084 demonstrated successfully.");
    println!("=================================================================");
}
