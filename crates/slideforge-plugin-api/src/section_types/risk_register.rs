//! The `risk_register` section type — auto-generated from `severity_cards` slides.
//!
//! [`RiskRegisterSectionType`] scans a slide sequence and produces one
//! [`SectionBlock`] per slide whose `slide_type` field is `"severity_cards"`.
//! All other slide types are skipped.
//!
//! If the DSL author also provides an explicit `section risk_register:` block,
//! the layout engine applies the supersession rule (BC-3.02.001 EC-002) and uses
//! the manual block instead of this plugin's output.

use std::sync::Arc;

use slideforge_types::{Register, Slide};

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
/// `slide_type` field equals `"severity_cards"` AND whose `register` is NOT
/// `Some(Register::Notes)`, the plugin emits one [`SectionBlock`] at level 1
/// with `include_in_toc: true`. The block title is the slide's resolved title
/// string, falling back to the empty string when no `title` field is present.
///
/// Slides with `register: Some(Register::Notes)` are excluded per
/// BC-3.02.001 invariant 4 — the notes register is speaker-facing content
/// and must not leak into formal document output.
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
    /// Each qualifying slide must:
    /// - Have `slide_type == "severity_cards"`, AND
    /// - NOT have `register == Some(Register::Notes)` (BC-3.02.001 invariant 4 —
    ///   the notes register is speaker-facing content and must not appear in formal
    ///   document output; this mirrors the same exclusion in `executive_summary`).
    ///
    /// The emitted `SectionBlock` carries:
    /// - `title`: the slide's resolved title string (empty string if absent)
    /// - `subtitle`: `None`
    /// - `level`: `1`
    /// - `start_slide_index`: the slide's position in `slides`
    /// - `end_slide_index`: `None`
    /// - `include_in_toc`: `true`
    ///
    /// ## Empty-title behavior (intentional)
    ///
    /// The [`SectionBlock`] type carries only a `title` string, not the richer
    /// `severity_cards` card content that the layout pipeline's `GeneratedSection`
    /// carries. When a contributing slide has no resolvable `title` field, the
    /// emitted block has `title == ""`. The block is **still emitted** — it is a
    /// genuine contributing slide and dropping it would lose a real section entry.
    /// This is an intentional trait-shape limitation, locked by
    /// `test_bc_3_02_001_med084_003_missing_title_field_emits_block_with_empty_title`.
    fn generate(&self, slides: &[Slide]) -> Vec<SectionBlock> {
        slides
            .iter()
            .enumerate()
            .filter(|(_, slide)| {
                // Must be a severity_cards slide.
                slide.slide_type.as_ref() == "severity_cards"
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
            field_spans: OrderedMap::new(),
        }
    }

    /// Build a `severity_cards` slide with an optional title and an optional register.
    fn make_severity_cards(title: Option<&str>, register: Option<Register>) -> Slide {
        let mut fields = OrderedMap::new();
        if let Some(t) = title {
            fields.insert(
                Arc::from("title"),
                FieldValue::Literal(Value::Str(Arc::from(t))),
            );
        }
        Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
            field_spans: OrderedMap::new(),
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
    // AC-003 + EC-001: generate() scanning
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001: empty slide slice → `generate()` must return `vec![]`.
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

    /// AC-003: `SectionBlock.title` is derived from the contributing slide's title field.
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
    // EC-002: slide that is BOTH severity_cards AND has takeaway
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-002: a slide that is `severity_cards` AND has a `takeaway` field is
    /// counted by `RiskRegisterSectionType` (this plugin scans on `slide_type`,
    /// not on field presence; the two plugins are independent).
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
            field_spans: OrderedMap::new(),
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

    // ─────────────────────────────────────────────────────────────────────────
    // CRIT-084-001: BC-3.02.001 invariant 4 — Notes-register severity_cards EXCLUDED
    // RED GATE (closed): failed prior to the `register != Notes` filter; now
    // asserts the exclusion is enforced by RiskRegisterSectionType::generate.
    // ─────────────────────────────────────────────────────────────────────────

    /// CRIT-084-001 / BC-3.02.001 inv-4: a `severity_cards` slide with
    /// `register: Some(Register::Notes)` MUST be excluded from `risk_register`
    /// output — the same rule that `executive_summary` already enforces.
    ///
    /// Deck: 2 `severity_cards` slides where 1 has `register: Notes` → exactly
    /// 1 block (only the non-notes slide contributes).
    ///
    /// RED GATE (closed): failed prior to the `register != Notes` filter being
    /// added to `RiskRegisterSectionType::generate`; now asserts the exclusion.
    #[test]
    fn test_bc_3_02_001_inv4_notes_register_severity_cards_excluded_from_risk_register() {
        let slides = vec![
            // This slide is severity_cards + notes register → must be EXCLUDED.
            make_severity_cards(Some("Speaker Notes Risk"), Some(Register::Notes)),
            // This slide is severity_cards + no register → must be INCLUDED.
            make_severity_cards(Some("Vendor Risks"), None),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "notes-register severity_cards slide must be excluded; \
             expected 1 block but got {}",
            result.len()
        );
        assert_eq!(
            result[0].title.as_ref(),
            "Vendor Risks",
            "the surviving block must be the non-notes slide"
        );
    }

    /// CRIT-084-001 / BC-3.02.001 inv-4: when ALL `severity_cards` slides have
    /// `register: Some(Register::Notes)`, `generate()` must return `vec![]`.
    ///
    /// RED GATE (closed): failed prior to the notes-exclusion filter; now
    /// asserts that all-notes-register decks produce an empty result.
    #[test]
    fn test_bc_3_02_001_inv4_all_severity_cards_notes_register_returns_empty() {
        let slides = vec![
            make_slide("title", Some("Intro")),
            make_severity_cards(Some("Internal Risk A"), Some(Register::Notes)),
            make_severity_cards(Some("Internal Risk B"), Some(Register::Notes)),
            make_slide("bullets", Some("Summary")),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert!(
            result.is_empty(),
            "all severity_cards slides are notes-register; \
             generate() must return vec![]; got {} block(s)",
            result.len()
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-084-002: absolute start_slide_index contract + subtitle/end_slide_index
    // These tests PASS against the current impl (coverage backfill).
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-084-002: in a 3-slide deck where contributing slides are at ABSOLUTE
    /// indices 0 and 2 (non-contiguous, with a non-contributing slide at index 1),
    /// the second block's `start_slide_index` must be 2.
    ///
    /// This locks the `.enumerate()`-before-`.filter()` absolute-index contract —
    /// the index reflects the slide's position in the full input slice, not its
    /// position among filtered slides.
    #[test]
    fn test_bc_3_02_001_med084_002_absolute_start_slide_index_non_contiguous() {
        let slides = vec![
            // index 0 — severity_cards → INCLUDED
            make_severity_cards(Some("Technical Risks"), None),
            // index 1 — not severity_cards → EXCLUDED
            make_slide("bullets", Some("Mitigations")),
            // index 2 — severity_cards → INCLUDED
            make_severity_cards(Some("Vendor Risks"), None),
        ];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            2,
            "expected 2 blocks for 2 severity_cards slides; got {}",
            result.len()
        );
        assert_eq!(
            result[0].start_slide_index, 0,
            "first block must have start_slide_index == 0 (absolute index)"
        );
        assert_eq!(
            result[1].start_slide_index, 2,
            "second block must have start_slide_index == 2 (absolute index, \
             not 1 after filtering)"
        );
    }

    /// MED-084-002: each emitted block has `end_slide_index == None` and
    /// `subtitle == None` — both fields are `None` per the generation spec.
    #[test]
    fn test_bc_3_02_001_med084_002_block_end_slide_index_and_subtitle_are_none() {
        let slides = vec![make_severity_cards(Some("Supply-Chain Risks"), None)];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(result.len(), 1, "expected 1 block");
        assert!(
            result[0].end_slide_index.is_none(),
            "SectionBlock.end_slide_index must be None; \
             got {:?}",
            result[0].end_slide_index
        );
        assert!(
            result[0].subtitle.is_none(),
            "SectionBlock.subtitle must be None; got {:?}",
            result[0].subtitle
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-084-003: empty-title fallback — document intentional behavior
    // This test PASSES against the current impl (coverage backfill).
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-084-003: a `severity_cards` slide that has NO `title` field emits a
    /// `SectionBlock` with `title == ""` (the intentional fallback from
    /// `title_str().unwrap_or("")`). The block IS still emitted — it is NOT
    /// dropped due to the absent title.
    ///
    /// This locks the current `SectionBlock` shape contract: title is the slide's
    /// resolved title string or the empty string; the trait carries only a title,
    /// not the full `severity_cards` card content.
    #[test]
    fn test_bc_3_02_001_med084_003_missing_title_field_emits_block_with_empty_title() {
        // severity_cards slide with NO title field.
        let slides = vec![make_severity_cards(None, None)];
        let result = RiskRegisterSectionType.generate(&slides);
        assert_eq!(
            result.len(),
            1,
            "severity_cards slide without a title must still emit 1 SectionBlock; \
             got {} block(s)",
            result.len()
        );
        assert_eq!(
            result[0].title.as_ref(),
            "",
            "SectionBlock.title must be \"\" when the slide has no title field; \
             got {:?}",
            result[0].title
        );
    }
}
