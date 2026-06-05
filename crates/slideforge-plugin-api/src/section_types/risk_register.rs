//! The `risk_register` section type — auto-generated from `severity_cards` slides.
//!
//! [`RiskRegisterSectionType`] scans a slide sequence and produces one
//! [`SectionBlock`](crate::traits::SectionBlock) per slide whose
//! `slide_type` field is `"severity_cards"`. All other slide types are skipped.
//!
//! If the DSL author also provides an explicit `section risk_register:` block,
//! the layout engine applies the supersession rule (BC-3.02.001 EC-002) and uses
//! the manual block instead of this plugin's output.

use slideforge_types::Slide;

use crate::traits::section_type::{SectionBlock, SectionType};

/// A [`SectionType`] plugin that auto-generates a risk-register section by
/// scanning slides whose `slide_type` is `"severity_cards"`.
///
/// ## Plugin identity
///
/// `id()` returns `"risk_register"`.
///
/// ## Generation logic
///
/// `generate(slides)` iterates the slide sequence. For each slide whose
/// `slide_type` field equals `"severity_cards"`, the plugin emits one
/// [`SectionBlock`] at level 1 with `include_in_toc: true`. The block title is
/// the slide's resolved title string, falling back to the empty string when no
/// `title` field is present.
///
/// Registered by the `PluginRegistryBuilder` (STORY-049).
#[derive(Debug, Default)]
pub struct RiskRegisterSectionType;

impl SectionType for RiskRegisterSectionType {
    /// Returns the unique plugin identifier `"risk_register"`.
    fn id(&self) -> &'static str {
        "risk_register"
    }

    /// Scan `slides` and produce one [`SectionBlock`] per slide whose type is
    /// `"severity_cards"`.
    ///
    /// # Implementation note
    ///
    /// The full scanning logic is implemented by STORY-084's TDD green phase.
    /// This stub body satisfies the compiler without revealing the algorithm.
    fn generate(&self, slides: &[Slide]) -> Vec<SectionBlock> {
        todo!(
            "STORY-084: scan slides for severity_cards type and emit SectionBlocks; \
             received {} slide(s)",
            slides.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_types::{FieldValue, OrderedMap, SourceSpan, Value};
    use std::sync::Arc;

    /// Build a minimal slide of the given slide type, with an optional title.
    fn make_slide(slide_type: &str, title: Option<&str>) -> Slide {
        let mut fields = OrderedMap::new();
        if let Some(t) = title {
            fields.insert(
                Arc::from("title"),
                FieldValue::Literal(Value::Str(Arc::from(t))),
            );
        }
        Slide {
            slide_type: Arc::from(slide_type),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // id() — implemented in the stub; must pass at Red Gate
    // ─────────────────────────────────────────────────────────────────────────

    /// `id()` returns `"risk_register"` — implemented in the stub, must pass.
    #[test]
    fn test_bc_5_02_001_risk_register_id() {
        assert_eq!(RiskRegisterSectionType.id(), "risk_register");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 + EC-001: generate() scanning — RED GATE (panic at todo!())
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001: empty slide slice → `generate()` must return `vec![]`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_risk_register_ec001_empty_slice_returns_empty() {
        let result = RiskRegisterSectionType.generate(&[]);
        assert!(
            result.is_empty(),
            "empty slide slice must produce no SectionBlocks; got {} block(s)",
            result.len()
        );
    }

    /// AC-003: deck with no `severity_cards` slides → returns `vec![]`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_risk_register_no_severity_cards_returns_empty() {
        let slides = vec![
            make_slide("title", Some("Intro")),
            make_slide("bullets", Some("Key Points")),
            make_slide("chart", Some("Revenue")),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert!(
            result.is_empty(),
            "slides without severity_cards type must produce no SectionBlocks; \
             got {} block(s)",
            result.len()
        );
    }

    /// AC-003: 5-slide deck with 2 `severity_cards` slides and 3 others → exactly 2 `SectionBlock`s.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_risk_register_mixed_deck_returns_exact_count_2() {
        let slides = vec![
            make_slide("title", Some("Overview")),
            make_slide("severity_cards", Some("Technical Risks")),
            make_slide("bullets", Some("Mitigations")),
            make_slide("severity_cards", Some("Vendor Risks")),
            make_slide("closing", Some("Thank You")),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            2,
            "deck with 2 severity_cards slides must produce exactly 2 SectionBlocks; \
             got {}",
            result.len()
        );
    }

    /// AC-003: each emitted `SectionBlock` carries `level == 1` and `include_in_toc == true`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_risk_register_block_has_level_1_and_include_in_toc_true() {
        let slides = vec![make_slide("severity_cards", Some("Supply-Chain Risks"))];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "one severity_cards slide must produce exactly 1 SectionBlock"
        );
        let block = &result[0];
        assert_eq!(block.level, 1, "SectionBlock.level must be 1; got {}", block.level);
        assert!(
            block.include_in_toc,
            "SectionBlock.include_in_toc must be true"
        );
    }

    /// AC-003: `SectionBlock.title` is derived from the contributing slide's title field.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_risk_register_block_title_matches_slide_title() {
        let slides = vec![make_slide("severity_cards", Some("Vendor Risks"))];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "one severity_cards slide must produce exactly 1 SectionBlock"
        );
        assert_eq!(
            result[0].title.as_ref(),
            "Vendor Risks",
            "SectionBlock.title must match the slide's title field"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-002: slide that is BOTH severity_cards AND has takeaway — RED GATE
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-002: a slide that is `severity_cards` AND has a `takeaway` field is
    /// counted by `RiskRegisterSectionType` (this plugin scans on `slide_type`,
    /// not on field presence; the two plugins are independent).
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_ec002_severity_cards_with_takeaway_counted_by_risk_register() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("title"),
            FieldValue::Literal(Value::Str(Arc::from("Dual Slide"))),
        );
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Literal(Value::Str(Arc::from("High impact"))),
        );
        let dual_slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let result = RiskRegisterSectionType.generate(&[dual_slide]);
        assert_eq!(
            result.len(),
            1,
            "severity_cards slide with a takeaway field must still be included by \
             RiskRegisterSectionType; got {} block(s)",
            result.len()
        );
        assert_eq!(
            result[0].title.as_ref(),
            "Dual Slide",
            "SectionBlock.title must match the slide's title field"
        );
        assert_eq!(result[0].level, 1, "SectionBlock.level must be 1");
        assert!(
            result[0].include_in_toc,
            "SectionBlock.include_in_toc must be true"
        );
    }
}
