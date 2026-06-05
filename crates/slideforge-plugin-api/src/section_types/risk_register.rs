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
    // BC-5.02.001: AC-003 — RiskRegisterSectionType scanning tests
    // ─────────────────────────────────────────────────────────────────────────

    /// `id()` returns `"risk_register"`.
    #[test]
    fn test_bc_5_02_001_risk_register_id() {
        assert_eq!(RiskRegisterSectionType.id(), "risk_register");
    }

    /// RED GATE: empty slice returns empty Vec (EC-001).
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_generate_empty_slice() {
        let result = RiskRegisterSectionType.generate(&[]);
        assert!(result.is_empty());
    }

    /// RED GATE: deck with no `severity_cards` slides produces no `SectionBlock`s.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_generate_no_severity_cards() {
        let slides = vec![
            make_slide("title", Some("Intro")),
            make_slide("bullets", Some("Key Points")),
            make_slide("chart", Some("Revenue")),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert!(result.is_empty(), "expected empty; got {result:?}");
    }

    /// RED GATE: 5-slide deck, 2 are `severity_cards` → 2 `SectionBlock`s.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_generate_two_severity_cards() {
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
            "expected 2 SectionBlocks for 2 severity_cards slides; got {result:?}"
        );
    }

    /// RED GATE: all slides are `severity_cards` → one `SectionBlock` each.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_generate_all_severity_cards() {
        let slides = vec![
            make_slide("severity_cards", Some("Risk A")),
            make_slide("severity_cards", Some("Risk B")),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(result.len(), 2);
    }

    /// RED GATE: emitted `SectionBlock` has `level=1`, `include_in_toc=true`.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_section_block_fields_level_and_toc() {
        let slides = vec![make_slide("severity_cards", Some("Supply-Chain Risks"))];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(result.len(), 1);
        let block = &result[0];
        assert_eq!(block.level, 1, "level must be 1");
        assert!(block.include_in_toc, "include_in_toc must be true");
    }

    /// RED GATE: `SectionBlock.title` is the slide's title field value.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_section_block_title_from_slide_title() {
        let slides = vec![make_slide("severity_cards", Some("Vendor Risks"))];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title.as_ref(), "Vendor Risks");
    }

    /// RED GATE: EC-002 — a slide that is BOTH `severity_cards` AND has a takeaway
    /// field is included by this plugin (plugins are independent).
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_risk_register_ec002_severity_cards_with_takeaway_included() {
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
            "severity_cards slide must be included regardless of takeaway field"
        );
    }
}
