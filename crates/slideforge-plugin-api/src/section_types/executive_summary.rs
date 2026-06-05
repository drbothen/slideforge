//! The `executive_summary` section type — auto-generated from slides with a
//! `takeaway:` field.
//!
//! [`ExecutiveSummarySectionType`] scans a slide sequence and produces one
//! [`SectionBlock`](crate::traits::SectionBlock) per slide whose fields map
//! contains a `"takeaway"` entry. Slides without a `takeaway` field are skipped.
//!
//! If the DSL author also provides an explicit `section executive_summary:` block,
//! the layout engine applies the supersession rule (BC-3.02.001 EC-002) and uses
//! the manual block instead of this plugin's output.

use slideforge_types::Slide;

use crate::traits::section_type::{SectionBlock, SectionType};

/// A [`SectionType`] plugin that auto-generates an executive-summary section
/// by scanning slides for a `takeaway` field.
///
/// ## Plugin identity
///
/// `id()` returns `"executive_summary"`.
///
/// ## Generation logic
///
/// `generate(slides)` iterates the slide sequence. For each slide that contains
/// a `takeaway` key in its `fields` map, the plugin emits one
/// [`SectionBlock`] at level 1 with `include_in_toc: true`. The block title is
/// the slide's resolved title string, falling back to the empty string when no
/// `title` field is present.
///
/// Registered by the `PluginRegistryBuilder` (STORY-049).
#[derive(Debug, Default)]
pub struct ExecutiveSummarySectionType;

impl SectionType for ExecutiveSummarySectionType {
    /// Returns the unique plugin identifier `"executive_summary"`.
    fn id(&self) -> &'static str {
        "executive_summary"
    }

    /// Scan `slides` and produce one [`SectionBlock`] per slide that has a
    /// `takeaway` field.
    ///
    /// # Implementation note
    ///
    /// The full scanning logic is implemented by STORY-084's TDD green phase.
    /// This stub body satisfies the compiler without revealing the algorithm.
    fn generate(&self, slides: &[Slide]) -> Vec<SectionBlock> {
        todo!(
            "STORY-084: scan slides for takeaway fields and emit SectionBlocks; \
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

    /// Build a minimal slide with no fields.
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

    /// Build a slide that carries a `takeaway` field and an optional title.
    fn slide_with_takeaway(title: &str, takeaway: &str) -> Slide {
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
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-5.02.001: AC-002 — ExecutiveSummarySectionType scanning tests
    // ─────────────────────────────────────────────────────────────────────────

    /// RED GATE: `id()` returns `"executive_summary"`.
    #[test]
    fn test_bc_5_02_001_executive_summary_id() {
        assert_eq!(ExecutiveSummarySectionType.id(), "executive_summary");
    }

    /// RED GATE: empty slide slice returns empty Vec (EC-001).
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_generate_empty_slice() {
        let result = ExecutiveSummarySectionType.generate(&[]);
        assert!(result.is_empty());
    }

    /// RED GATE: slides with no takeaway field produce no `SectionBlock`s.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_generate_no_takeaway_slides() {
        let slides = vec![blank_slide("title"), blank_slide("bullets")];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert!(result.is_empty(), "expected empty; got {result:?}");
    }

    /// RED GATE: mixed deck — 2 takeaway slides, 1 without → 2 `SectionBlock`s.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_generate_mixed_deck_two_takeaway() {
        let slides = vec![
            slide_with_takeaway("Q1 Results", "Revenue up 12%"),
            blank_slide("bullets"),
            slide_with_takeaway("Market Share", "Grew 3 pts YoY"),
        ];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(
            result.len(),
            2,
            "expected 2 SectionBlocks for 2 takeaway slides; got {result:?}"
        );
    }

    /// RED GATE: all slides have takeaway → one `SectionBlock` per slide.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_generate_all_takeaway_slides() {
        let slides = vec![
            slide_with_takeaway("Slide A", "Takeaway A"),
            slide_with_takeaway("Slide B", "Takeaway B"),
        ];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(result.len(), 2);
    }

    /// RED GATE: each emitted `SectionBlock` has `level=1`, `include_in_toc=true`.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_section_block_fields_level_and_toc() {
        let slides = vec![slide_with_takeaway("My Slide", "Key insight")];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(result.len(), 1);
        let block = &result[0];
        assert_eq!(block.level, 1, "level must be 1");
        assert!(block.include_in_toc, "include_in_toc must be true");
    }

    /// RED GATE: `SectionBlock.title` comes from the slide's title field.
    #[test]
    #[should_panic(expected = "STORY-084")]
    fn test_bc_5_02_001_executive_summary_section_block_title_from_slide_title() {
        let slides = vec![slide_with_takeaway("Revenue Summary", "Up 12%")];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title.as_ref(), "Revenue Summary");
    }
}
