//! The `executive_summary` section type — auto-generated from slides with a
//! `takeaway:` field.
//!
//! [`ExecutiveSummarySectionType`] scans a slide sequence and produces one
//! [`SectionBlock`] per slide whose fields map contains a `"takeaway"` entry.
//! Slides without a `takeaway` field are skipped.
//!
//! If the DSL author also provides an explicit `section executive_summary:` block,
//! the layout engine applies the supersession rule (BC-3.02.001 EC-002) and uses
//! the manual block instead of this plugin's output.

use std::sync::Arc;

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
/// Slides with `register: Some(Register::Notes)` are excluded per
/// BC-3.02.001 invariant 4 — the notes register is speaker-facing content
/// and must not leak into formal document output.
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
    /// Each qualifying slide must:
    /// - Contain a `"takeaway"` key in its `fields` map, AND
    /// - NOT have `register == Some(Register::Notes)` (BC-3.02.001 invariant 4).
    ///
    /// The emitted `SectionBlock` carries:
    /// - `title`: the slide's resolved title string (empty string if absent)
    /// - `subtitle`: `None`
    /// - `level`: `1`
    /// - `start_slide_index`: the slide's position in `slides`
    /// - `end_slide_index`: `None`
    /// - `include_in_toc`: `true`
    fn generate(&self, slides: &[Slide]) -> Vec<SectionBlock> {
        use slideforge_types::Register;

        slides
            .iter()
            .enumerate()
            .filter(|(_, slide)| {
                // Must have a takeaway field.
                slide.fields.contains_key("takeaway")
                    // Must NOT be a notes-register slide (BC-3.02.001 invariant 4).
                    && slide.register != Some(Register::Notes)
            })
            .map(|(index, slide)| SectionBlock {
                title: Arc::from(slide.title_str().unwrap_or("")),
                subtitle: None,
                level: 1,
                start_slide_index: index,
                end_slide_index: None,
                include_in_toc: true,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slideforge_types::{FieldValue, OrderedMap, Register, SourceSpan, Value};
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

    /// Build a slide that carries a `takeaway` field and a title, with an
    /// optional register override.
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

    // ─────────────────────────────────────────────────────────────────────────
    // id() — implemented in the stub; must pass at Red Gate
    // ─────────────────────────────────────────────────────────────────────────

    /// `id()` returns `"executive_summary"` — implemented in the stub, must pass.
    #[test]
    fn test_bc_5_02_001_executive_summary_id() {
        assert_eq!(ExecutiveSummarySectionType.id(), "executive_summary");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 + EC-001: generate() scanning — RED GATE (panic at todo!())
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001: empty slide slice → `generate()` must return `vec![]`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_executive_summary_ec001_empty_slice_returns_empty() {
        let result = ExecutiveSummarySectionType.generate(&[]);
        assert!(
            result.is_empty(),
            "empty slide slice must produce no SectionBlocks; got {} block(s)",
            result.len()
        );
    }

    /// Deck with no slides that have a `takeaway` field → returns `vec![]`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_executive_summary_no_takeaway_slides_returns_empty() {
        let slides = vec![blank_slide("title"), blank_slide("bullets")];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert!(
            result.is_empty(),
            "slides without takeaway field must produce no SectionBlocks; got {} block(s)",
            result.len()
        );
    }

    /// AC-002: mixed deck — 3 slides, 2 have `takeaway:`, 1 does not → exactly 2 `SectionBlock`s.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_executive_summary_mixed_deck_returns_exact_count_2() {
        let slides = vec![
            slide_with_takeaway("Q1 Results", "Revenue up 12%", None),
            blank_slide("bullets"),
            slide_with_takeaway("Market Share", "Grew 3 pts YoY", None),
        ];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(
            result.len(),
            2,
            "deck with 2 takeaway slides must produce exactly 2 SectionBlocks; got {}",
            result.len()
        );
    }

    /// AC-002: each emitted `SectionBlock` carries `level == 1` and `include_in_toc == true`.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_executive_summary_block_has_level_1_and_include_in_toc_true() {
        let slides = vec![slide_with_takeaway("My Slide", "Key insight", None)];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "one takeaway slide must produce exactly 1 SectionBlock"
        );
        let block = &result[0];
        assert_eq!(
            block.level, 1,
            "SectionBlock.level must be 1; got {}",
            block.level
        );
        assert!(
            block.include_in_toc,
            "SectionBlock.include_in_toc must be true"
        );
    }

    /// AC-002: `SectionBlock.title` is derived from the contributing slide's title field.
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_executive_summary_block_title_matches_slide_title() {
        let slides = vec![slide_with_takeaway("Revenue Summary", "Up 12%", None)];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "one takeaway slide must produce exactly 1 SectionBlock"
        );
        assert_eq!(
            result[0].title.as_ref(),
            "Revenue Summary",
            "SectionBlock.title must match the slide's title field"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 invariant 4: Notes-register slides EXCLUDED — RED GATE
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.02.001 inv-4: a slide with a `takeaway` field but `register:
    /// Some(Register::Notes)` is EXCLUDED from the `executive_summary` output.
    ///
    /// Deck: 2 takeaway slides where 1 is register:notes → returns exactly 1 block
    /// (only the non-notes slide contributes).
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_3_02_001_inv4_notes_register_slide_excluded_from_executive_summary() {
        let slides = vec![
            // This slide has takeaway + notes register → must be EXCLUDED
            slide_with_takeaway(
                "Speaker Notes Slide",
                "Internal takeaway",
                Some(Register::Notes),
            ),
            // This slide has takeaway + no register → must be INCLUDED
            slide_with_takeaway("Revenue Summary", "Revenue up 12%", None),
        ];
        let result = ExecutiveSummarySectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "notes-register slide must be excluded; expected 1 block but got {}",
            result.len()
        );
        assert_eq!(
            result[0].title.as_ref(),
            "Revenue Summary",
            "the included block must be the non-notes slide"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-002: slide that is BOTH takeaway AND severity_cards — RED GATE
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-002: a slide that has both a `takeaway` field AND `slide_type ==
    /// "severity_cards"` is counted by `ExecutiveSummarySectionType` (because this
    /// plugin scans for `takeaway` fields independently of slide type).
    ///
    /// RED GATE: panics at `todo!()` until implemented.
    #[test]
    fn test_bc_5_02_001_ec002_severity_cards_with_takeaway_counted_by_executive_summary() {
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
        let result = ExecutiveSummarySectionType.generate(&[dual_slide]);
        assert_eq!(
            result.len(),
            1,
            "severity_cards slide with a takeaway field must be included by \
             ExecutiveSummarySectionType; got {} block(s)",
            result.len()
        );
        assert_eq!(result[0].level, 1, "SectionBlock.level must be 1");
        assert!(
            result[0].include_in_toc,
            "SectionBlock.include_in_toc must be true"
        );
    }
}
