//! Document section collection — assembles [`GeneratedSection`] data for DOCX
//! and PDF output from the semantic [`slideforge_types::Deck`] IR.
//!
//! This module implements the layout stage's responsibility under SS-05: after
//! the evaluator has resolved all `{{ expr }}` and `@data` bindings, the
//! section collection pass walks the `Deck` and produces a
//! `Vec<GeneratedSection>` that is stored on [`crate::types::LaidOutDeck`].
//!
//! ## Architecture (DI-008 / DI-012)
//!
//! - **Pure collection only.** No Word XML generation happens here. DOCX/PDF
//!   rendering is delegated to `SectionType` plugin implementations
//!   (STORY-041/042).
//! - **Single source for all document formats.** Both DOCX and PDF exporters
//!   consume `LaidOutDeck.sections`. No format-specific branching occurs in
//!   the layout stage.
//! - **Exporter filtering by `target_formats`.** `GeneratedSection` carries a
//!   `target_formats` bitset (AC-005). PPTX and HTML exporters filter out
//!   sections where their format is not listed.
//!
//! ## Auto-generated sections (BC-3.02.001)
//!
//! | Source | Produced section |
//! |--------|-----------------|
//! | Slides with `takeaway:` field | [`SectionKind::ExecutiveSummary`] |
//! | `severity_cards` slides | [`SectionKind::RiskRegister`] |
//!
//! ## Manually authored sections (BC-3.02.002)
//!
//! `section <type>:` blocks in the `Deck` IR are collected into
//! [`SectionSource::ManuallyAuthored`] entries. An unrecognised section type
//! returns [`crate::error::LayoutError::UnknownSectionType`].

use std::sync::Arc;

use slideforge_types::{Deck, FieldValue, OrderedMap, Register, Value};
use tracing::warn;

use crate::error::LayoutError;

/// The set of section type names supported by manually authored sections
/// (BC-3.02.002 AC-004).
///
/// `executive_summary` and `risk_register` are included here because they
/// can be manually authored to supersede the auto-generated equivalents
/// (BC-3.02.001 EC-002 / AC-006).  When a manual block with one of these
/// names is present, `collect_sections` fires the supersession path and
/// suppresses the auto-generated section of the same kind.
const SUPPORTED_MANUAL_SECTION_TYPES: &[&str] = &[
    "executive_summary",
    "risk_register",
    "methodology",
    "scope",
    "approval",
    "appendix",
    "glossary",
];

/// The output format a section should be included in.
///
/// `OutputFormat` is used in [`GeneratedSection::target_formats`] to signal
/// which exporters should render a section. PPTX and HTML exporters skip
/// sections where their format is absent from this list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OutputFormat {
    /// DOCX (Word) export.
    Docx,
    /// HTML export.
    Html,
    /// PDF export.
    Pdf,
    /// PPTX (`PowerPoint`) export.
    Pptx,
    /// Web preview export.
    Preview,
}

/// The kind of a generated document section.
///
/// `SectionKind` identifies the semantics of a section and controls how a
/// `SectionType` plugin renders it. Auto-generated sections have a fixed
/// semantic kind; manually authored sections carry their type name as a
/// `ManualSection` variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionKind {
    /// An auto-generated executive summary built from slide `takeaway:` fields.
    ///
    /// Heading: `"Executive Summary"`. Items: one
    /// [`SectionItem::TakeawayBullet`] per contributing slide, in slide order.
    ExecutiveSummary,

    /// An auto-generated risk register built from `severity_cards` slides.
    ///
    /// Heading: `"Risk Register"`. Items: one [`SectionItem::RiskRow`] per
    /// card entry, in slide order. No de-duplication is applied (BC-3.02.001
    /// EC-003).
    RiskRegister,

    /// A manually authored section block from `section <type>:` DSL syntax.
    ///
    /// Supported types: `executive_summary`, `risk_register`, `methodology`,
    /// `scope`, `approval`, `appendix`, `glossary`. The type name is stored
    /// here for plugin dispatch.
    ///
    /// `executive_summary` and `risk_register` are special: when a manual block
    /// with one of those names is present, `collect_sections` fires the
    /// supersession rule (AC-006 / BC-3.02.001 EC-002) and suppresses the
    /// auto-generated equivalent.
    ///
    /// ## Future extension (MED-002 / STORY-041 / STORY-042)
    ///
    /// A `Custom` variant for third-party plugin-registered section types will
    /// be added when the `SectionType` plugin surface is activated. At that
    /// point, `ManualSection` will carry a plugin-registered type ID alongside
    /// the author-declared name. No breaking change is needed until then.
    ManualSection(Arc<str>),
}

impl SectionKind {
    /// Return the canonical ordering key used when sorting by `section_order:`.
    ///
    /// Returns the section type name as a string slice:
    /// - `"executive_summary"` for [`SectionKind::ExecutiveSummary`]
    /// - `"risk_register"` for [`SectionKind::RiskRegister`]
    /// - the type name for [`SectionKind::ManualSection`]
    fn order_key(&self) -> &str {
        match self {
            SectionKind::ExecutiveSummary => "executive_summary",
            SectionKind::RiskRegister => "risk_register",
            SectionKind::ManualSection(name) => name.as_ref(),
        }
    }
}

/// Whether a section was auto-generated from slide data or manually authored.
///
/// This distinction is preserved in the layout IR so that:
/// - The supersession rule (AC-006) can suppress auto-generated sections when a
///   manual section of the same kind is present.
/// - The DOCX/PDF exporters can apply different rendering policies per source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionSource {
    /// Assembled by the layout engine from slide field data.
    AutoGenerated,
    /// Parsed from an explicit `section <type>:` block in the `.sf` source.
    ManuallyAuthored,
}

/// A single item within a [`GeneratedSection`].
///
/// The variant determines what a `SectionType` plugin renders:
/// - [`SectionItem::TakeawayBullet`] renders as a bullet point in an executive
///   summary.
/// - [`SectionItem::RiskRow`] renders as a table row in a risk register.
/// - [`SectionItem::Custom`] is a catch-all for manually authored section
///   content with arbitrary key-value data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionItem {
    /// A single bullet derived from a slide's `takeaway:` field.
    TakeawayBullet(Arc<str>),

    /// A risk entry derived from a single card within a `severity_cards` slide.
    RiskRow {
        /// The risk item title.
        title: Arc<str>,
        /// Severity level (e.g., `"High"`, `"Medium"`, `"Low"`).
        severity: Arc<str>,
        /// Human-readable description of the risk.
        description: Arc<str>,
        /// The risk owner (name or role).
        owner: Arc<str>,
    },

    /// A custom section item from a manually authored `section:` block.
    ///
    /// The map keys are field names declared in the section block; values are
    /// the resolved [`slideforge_types::Value`]s.
    ///
    /// Uses [`OrderedMap`] (not `BTreeMap`) to preserve insertion order and
    /// satisfy the `Hash` requirement on all layout IR types (AC-010).
    Custom(OrderedMap<Arc<str>, Value>),
}

/// A collected document section ready for exporter consumption.
///
/// `GeneratedSection` is the primary output of this module and is stored as a
/// `Vec<GeneratedSection>` on [`crate::types::LaidOutDeck`].
///
/// ## Ordering
///
/// Within `LaidOutDeck.sections`, sections appear in the order determined by
/// [`collect_sections`]:
/// 1. Manually authored sections first (in DSL source order).
/// 2. Auto-generated sections after (in default kind order: executive summary
///    before risk register).
///
/// If `section_order:` is declared in the deck metadata, [`collect_sections`]
/// sorts the sections accordingly (AC-007).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneratedSection {
    /// The semantic kind of this section.
    pub kind: SectionKind,

    /// Whether this section was auto-generated or manually authored.
    pub source: SectionSource,

    /// The ordered list of items in this section.
    pub items: Vec<SectionItem>,

    /// The section heading text.
    ///
    /// For auto-generated sections this is a canonical default
    /// (`"Executive Summary"`, `"Risk Register"`). For manually authored
    /// sections this is the heading declared in the `section:` block, or the
    /// default for that section type if none was declared.
    pub heading: Arc<str>,

    /// The output formats that should render this section.
    ///
    /// Set by the layout stage so that exporters can filter without
    /// re-examining section kind. Default for most sections: `[Docx, Pdf]`.
    /// PPTX and HTML exporters skip sections where their format is absent
    /// (AC-005).
    ///
    /// **Invariant:** always sorted by `OutputFormat` discriminant order.
    pub target_formats: Vec<OutputFormat>,
}

/// Produce a canonical sorted `target_formats` vec for DOCX + PDF sections.
///
/// Returns `[OutputFormat::Docx, OutputFormat::Pdf]` (already in sorted order
/// because `Docx < Pdf` in the `OutputFormat` definition). HIGH-004: canonical
/// sort enforced on construction.
fn docx_pdf_formats() -> Vec<OutputFormat> {
    let mut formats = vec![OutputFormat::Docx, OutputFormat::Pdf];
    formats.sort();
    formats
}

/// Collect all document sections from a fully evaluated [`Deck`].
///
/// `collect_sections` is the main entry point called by [`crate::layout::run`].
/// It applies the supersession rule (AC-006): if both a manual and an
/// auto-generated section of the same kind are present, the manual section is
/// retained and the auto-generated one is suppressed.
///
/// The returned `Vec<GeneratedSection>` is stored in
/// [`crate::types::LaidOutDeck::sections`]. Ordering follows AC-007:
/// manual sections first (in DSL source order), then auto-generated sections
/// (executive summary before risk register), unless `section_order:` in the
/// deck metadata overrides this.
///
/// # Errors
///
/// Returns [`LayoutError::UnknownSectionType`] if the deck contains a manually
/// authored `section <type>:` block with an unrecognised type name.
///
/// Returns [`LayoutError::UnresolvedTakeaway`] if a slide's `takeaway:` field
/// is not a resolved `Literal(Str)` value (indicates an evaluator bug).
///
/// Returns [`LayoutError::MissingRiskCardField`] if a card entry in a
/// `severity_cards` slide is missing a required field.
///
/// # Panics
///
/// Does not panic. All error paths return `Err`.
pub fn collect_sections(deck: &Deck) -> Result<Vec<GeneratedSection>, LayoutError> {
    // ── Step 1: Collect manual sections from deck.section_blocks ──────────────
    // Validates section type names — returns Err for unknown types.
    let manual_sections = collect_manual_sections(deck)?;

    // Build a set of manually-authored section kinds so auto-generated sections
    // of the same kind can be suppressed (supersession rule, AC-006).
    let manual_section_names: std::collections::HashSet<String> = manual_sections
        .iter()
        .map(|s| s.kind.order_key().to_owned())
        .collect();

    // ── Step 2: Collect auto-generated sections (unless superseded) ────────────
    let mut auto_sections: Vec<GeneratedSection> = Vec::new();

    // ExecutiveSummary from takeaway fields — suppressed if a manual section of
    // the same kind is present (AC-006).
    if manual_section_names.contains("executive_summary") {
        // AC-006 / BC-3.02.001 EC-002: manual executive_summary supersedes auto.
        // Only warn if there are slides with resolved Literal(Str) takeaway values
        // that would have produced a real auto-generated section. Checking only
        // `contains_key("takeaway")` was a false-positive: an unresolved
        // `FieldValue::Expr` takeaway would have caused collect_executive_summary to
        // return Err(UnresolvedTakeaway), not a section — so there would be nothing
        // to supersede and the warning would be misleading.
        let has_resolved_takeaways = deck.slides.iter().any(|s| {
            use slideforge_types::FieldValue;
            s.register != Some(Register::Notes)
                && matches!(
                    s.fields.get("takeaway"),
                    Some(FieldValue::Literal(slideforge_types::Value::Str(_)))
                )
        });
        if has_resolved_takeaways {
            warn!(
                "BC-3.02.001 EC-002: Auto-generated executive_summary overridden by explicit \
                 section block"
            );
        }
    } else if let Some(exec_summary) = collect_executive_summary(deck)? {
        auto_sections.push(exec_summary);
    }

    // RiskRegister from severity_cards slides — suppressed if a manual section
    // of the same kind is present (AC-006).
    if manual_section_names.contains("risk_register") {
        // AC-006 / BC-3.02.001 EC-002: manual risk_register supersedes auto.
        // OBS-A (Pass 5): exclude Notes-register slides from this check.
        // A deck whose only severity_cards slides are Notes-register would not
        // have produced an auto-generated risk_register (collect_risk_register
        // already skips Notes slides). Firing the supersession warning for
        // Notes-only severity_cards would be misleading — there was nothing to
        // supersede. Mirror the same Notes filter used in the takeaway check above.
        let has_severity_cards = deck.slides.iter().any(|s| {
            s.register != Some(Register::Notes) && s.slide_type.as_ref() == "severity_cards"
        });
        if has_severity_cards {
            warn!(
                "BC-3.02.001 EC-002: Auto-generated risk_register overridden by explicit section \
                 block"
            );
        }
    } else if let Some(risk_register) = collect_risk_register(deck)? {
        auto_sections.push(risk_register);
    }

    // ── Step 3: Merge and apply section_order if present ──────────────────────
    // Default order: manual sections first, then auto-generated.
    let mut result = manual_sections;
    result.extend(auto_sections);

    // AC-007: if section_order is declared in deck metadata, sort accordingly.
    if let Some(order) = &deck.metadata.section_order {
        let order_map: std::collections::HashMap<&str, usize> = order
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_ref(), i))
            .collect();

        // OBS-005: warn if section_order names a section that wasn't collected.
        let collected_keys: std::collections::HashSet<&str> =
            result.iter().map(|s| s.kind.order_key()).collect();
        for name in order {
            if !collected_keys.contains(name.as_ref()) {
                warn!(
                    section_name = %name,
                    "section_order names '{}' but no such section was collected — \
                     check for missing slides or typo in section_order",
                    name
                );
            }
        }

        result.sort_by_key(|s| {
            order_map
                .get(s.kind.order_key())
                .copied()
                .unwrap_or(usize::MAX)
        });
    }

    Ok(result)
}

/// Collect manually authored sections from `deck.section_blocks` (BC-3.02.002).
///
/// Returns `Err(LayoutError::UnknownSectionType)` for any block whose type
/// name is not in [`SUPPORTED_MANUAL_SECTION_TYPES`].
fn collect_manual_sections(deck: &Deck) -> Result<Vec<GeneratedSection>, LayoutError> {
    let mut sections = Vec::new();
    // OBS-D (Pass 5): track section names seen so far to warn on duplicates.
    // The spec (BC-3.02.002) does not treat duplicate manual section names as an
    // error — both blocks are collected and included in the result. However, a
    // duplicate is almost certainly a DSL authoring mistake (the author likely
    // intended a single section), so a warning is emitted for each repeated name.
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for block in &deck.section_blocks {
        let name: &str = block.name.as_ref();

        // AC-004: only recognised type names are allowed (BC-3.02.002 EC-001).
        // The span from the section block is included in the error so that
        // miette can render a colored source pointer to the offending line.
        if !SUPPORTED_MANUAL_SECTION_TYPES.contains(&name) {
            return Err(LayoutError::UnknownSectionType {
                name: name.to_owned(),
                span: block.span.clone(),
            });
        }

        // OBS-D (Pass 5): warn on duplicate manual section names.
        // Both blocks are still collected — the spec does not prohibit duplicates.
        if !seen_names.insert(name.to_owned()) {
            warn!(
                section_name = %name,
                "section '{}' is declared more than once — \
                 both blocks will be included in the output; \
                 this is usually a DSL authoring mistake",
                name
            );
        }

        // OBS-003 / BC-3.02.002 EC-003: warn if the section body is empty.
        // An empty body is valid (no compile error) but likely a DSL authoring
        // mistake — the section block was declared with no content fields. The
        // warning gives the author an actionable signal without failing the build.
        if block.body.is_empty() {
            warn!(
                section_name = %name,
                "BC-3.02.002 EC-003: section '{}' has empty body — \
                 no content fields declared in the section block",
                name
            );
        }

        // Build ONE SectionItem::Custom containing all fields from the block body.
        // HIGH-005: each manual section block produces exactly one Custom item
        // whose map holds all key-value pairs (insertion order preserved via
        // OrderedMap).  Previous code emitted one item per key — semantically
        // wrong because it fragmented the section body into unrelated atoms.
        //
        // F-002 (Pass 4): The "heading" key is extracted above to set the section
        // heading, so it must NOT also appear in the Custom map — it is a layout
        // directive, not a content field.  Consumers of `SectionItem::Custom`
        // (DOCX/PDF exporters) must not receive the same "heading" key twice.
        //
        // OBS-C (Pass 5 — DEFERRED): the spec is silent on whether unrecognised
        // field keys in a manual section body should emit a warning. The BC
        // (BC-3.02.002) specifies that the section body is an open-ended
        // key-value map with no prescribed fields (section type plugins define
        // their own schema). Adding a key-allowlist or an "unknown field" warning
        // here would therefore be premature and could break author-defined content
        // whose keys are not yet known at layout time. Deferred until the
        // SectionType plugin surface (STORY-041/042) defines per-section schemas;
        // at that point each plugin can validate its own field set.
        // STORY-077: SectionBlock.body is now OrderedMap<Arc<str>, FieldValue>.
        // For backwards compatibility with SectionItem::Custom (which expects
        // OrderedMap<Arc<str>, Value>), extract the Value from Literal variants.
        // Non-Literal variants (Inlines, Template, etc.) are coerced to a Null
        // placeholder — the DOCX/PDF exporters read register_content for rich
        // section content, not this legacy Custom map.
        let custom_map: OrderedMap<Arc<str>, Value> = block
            .body
            .iter()
            .filter(|(k, _)| k.as_ref() != "heading")
            .map(|(k, fv)| {
                let v = match fv {
                    FieldValue::Literal(val) => val.clone(),
                    // Rich or template content is handled via register_content;
                    // the legacy Custom map gets a Null placeholder.
                    _ => Value::Null,
                };
                (Arc::clone(k), v)
            })
            .collect();
        let items = vec![SectionItem::Custom(custom_map)];

        // OBS-002: if the section body has a "heading" key with a plain string
        // value, use it as the section heading; otherwise fall back to the
        // humanized section type name.
        // OBS-B (Pass 5): when a "heading" key is present but holds a non-Str
        // value (e.g., a number, list, or map), the fallback is applied silently.
        // Emit a warning so DSL authors learn that their heading declaration was
        // ignored. This is NOT an error — the build continues with the fallback.
        // STORY-077: body values are now FieldValue; extract Literal(Str) for heading.
        let heading: Arc<str> = match block.body.get("heading") {
            Some(FieldValue::Literal(Value::Str(s))) => Arc::clone(s),
            Some(_non_str) => {
                warn!(
                    section_name = %name,
                    "section '{}' has a 'heading' key with a non-string value — \
                     falling back to humanized section name; \
                     use heading: \"...\" with a plain string value",
                    name
                );
                humanize_section_name(name)
            },
            None => humanize_section_name(name),
        };

        sections.push(GeneratedSection {
            kind: SectionKind::ManualSection(Arc::clone(&block.name)),
            source: SectionSource::ManuallyAuthored,
            items,
            heading,
            target_formats: docx_pdf_formats(),
        });
    }

    Ok(sections)
}

/// Produce a human-readable heading from a section type name.
///
/// Converts `"methodology"` → `"Methodology"`, `"executive_summary"` →
/// `"Executive Summary"`, etc. Used for manually authored sections.
fn humanize_section_name(name: &str) -> Arc<str> {
    let words: Vec<String> = name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect();
    Arc::from(words.join(" ").as_str())
}

/// Collect the auto-generated `executive_summary` section from slide `takeaway:` fields.
///
/// Walks `deck.slides` in order; for each slide that:
/// - has `register != Notes` (MED-004: Notes-register slides are excluded)
/// - has a `takeaway:` field resolving to `FieldValue::Literal(Value::Str(_))`
///
/// appends a [`SectionItem::TakeawayBullet`] to the result.
///
/// Returns `Ok(None)` if no slides contribute a takeaway (AC-003).
///
/// # Errors
///
/// Returns [`LayoutError::UnresolvedTakeaway`] if a slide has a `takeaway:`
/// field that is present but not a resolved `Literal(Str)` — i.e., it is a
/// `FieldValue::Expr`, `FieldValue::Interpolated`, or
/// `FieldValue::Inlines` variant. This indicates an evaluator bug: all
/// expressions should be resolved before layout runs.
///
/// # Contract
///
/// - Slide order is preserved (BC-3.02.001 postcondition 2).
/// - Notes-register slides do not contribute (MED-004).
pub(crate) fn collect_executive_summary(
    deck: &Deck,
) -> Result<Option<GeneratedSection>, LayoutError> {
    use slideforge_types::FieldValue;

    let mut items: Vec<SectionItem> = Vec::new();

    for (slide_index, slide) in deck.slides.iter().enumerate() {
        // OBS-002: The Notes-register filter comes FIRST — before the takeaway
        // field check. This ordering is intentional: Notes-register slides are
        // excluded from all document sections regardless of whether they have
        // unresolved fields. If the Notes check came after the takeaway match,
        // a Notes-register slide with an Expr takeaway would return
        // UnresolvedTakeaway instead of being silently skipped — masking an
        // evaluator bug rather than surfacing it. The current ordering is
        // correct: Notes-register exclusion takes precedence over all other
        // checks. MED-004 defines this rule.
        if slide.register == Some(Register::Notes) {
            continue;
        }

        match slide.fields.get("takeaway") {
            // Happy path: resolved string.
            Some(FieldValue::Literal(Value::Str(s))) => {
                items.push(SectionItem::TakeawayBullet(Arc::clone(s)));
            },
            // Field is present but not a Literal(Str): unresolved Expr/Interpolated/
            // Inlines variants (evaluator bug) or a non-string Literal (type error).
            // In all cases return an error (HIGH-002).
            Some(
                FieldValue::Expr(_)
                | FieldValue::Interpolated(_)
                | FieldValue::Inlines(_)
                | FieldValue::Literal(_),
            ) => {
                return Err(LayoutError::UnresolvedTakeaway {
                    source_slide_index: slide_index,
                });
            },
            // No takeaway field — skip this slide.
            None => {},
        }
    }

    if items.is_empty() {
        return Ok(None);
    }

    Ok(Some(GeneratedSection {
        kind: SectionKind::ExecutiveSummary,
        source: SectionSource::AutoGenerated,
        items,
        heading: Arc::from("Executive Summary"),
        target_formats: docx_pdf_formats(),
    }))
}

/// Collect the auto-generated `risk_register` section from `severity_cards` slides.
///
/// Walks `deck.slides` in order; for each slide with
/// `slide_type == "severity_cards"`, reads its `cards:` field as a
/// `Value::List`. Each entry in the list must be a `Value::Map` with
/// `title`, `severity`, `description`, and `owner` string keys. One
/// [`SectionItem::RiskRow`] is produced per card entry (CRIT-001 fix).
///
/// Returns `Ok(None)` if no `severity_cards` slides exist in the deck
/// (AC-003).
///
/// # Errors
///
/// Returns [`LayoutError::MissingRiskCardField`] if a card entry is missing a
/// required field or its value is not a plain string (HIGH-001).
///
/// # Contract
///
/// - Multiple `severity_cards` slides each contribute rows to the same section
///   (BC-3.02.001 postcondition 3).
/// - No de-duplication is applied — duplicate rows from an `@for` loop are all
///   included (BC-3.02.001 EC-003).
/// - If `@if` suppresses all `severity_cards` slides, this function returns
///   `Ok(None)` (BC-3.02.001 EC-004).
pub(crate) fn collect_risk_register(deck: &Deck) -> Result<Option<GeneratedSection>, LayoutError> {
    use slideforge_types::FieldValue;

    let mut items: Vec<SectionItem> = Vec::new();

    for (slide_index, slide) in deck.slides.iter().enumerate() {
        if slide.slide_type.as_ref() != "severity_cards" {
            continue;
        }

        // HIGH-002: Notes-register slides are excluded from the risk register.
        // BC-3.02.001 Invariant 4: the Notes-register exclusion rule applies to
        // ALL auto-generated sections (executive_summary AND risk_register).
        // Notes-register slides are presenter-only content and must not appear
        // in document sections that are rendered to DOCX/PDF.
        if slide.register == Some(Register::Notes) {
            continue;
        }

        // Extract the `cards:` field, which must be a Value::List of Value::Maps.
        // HIGH-003: silent fallback removed — wrong types propagate as errors.
        let cards: &[Value] = match slide.fields.get("cards") {
            Some(FieldValue::Literal(Value::List(list))) => list.as_slice(),
            Some(FieldValue::Literal(_)) => {
                return Err(LayoutError::MalformedSeverityCards {
                    source_slide_index: slide_index,
                    reason: "expected List value for 'cards' field, found a non-List Literal"
                        .to_owned(),
                });
            },
            Some(_) => {
                return Err(LayoutError::UnresolvedSeverityCards {
                    source_slide_index: slide_index,
                });
            },
            None => {
                return Err(LayoutError::MissingRiskCardField {
                    source_slide_index: slide_index,
                    card_index: 0,
                    field: "cards".to_owned(),
                });
            },
        };

        for (card_index, card) in cards.iter().enumerate() {
            let Value::Map(map) = card else {
                return Err(LayoutError::MissingRiskCardField {
                    source_slide_index: slide_index,
                    card_index,
                    field: "cards[n]".to_owned(),
                });
            };

            let title = extract_card_str(map, "title", slide_index, card_index)?;
            let severity = extract_card_str(map, "severity", slide_index, card_index)?;
            let description = extract_card_str(map, "description", slide_index, card_index)?;
            let owner = extract_card_str(map, "owner", slide_index, card_index)?;

            items.push(SectionItem::RiskRow {
                title,
                severity,
                description,
                owner,
            });
        }
    }

    if items.is_empty() {
        return Ok(None);
    }

    Ok(Some(GeneratedSection {
        kind: SectionKind::RiskRegister,
        source: SectionSource::AutoGenerated,
        items,
        heading: Arc::from("Risk Register"),
        target_formats: docx_pdf_formats(),
    }))
}

/// Extract a required string field from a risk card map.
///
/// Returns `Err(LayoutError::MissingRiskCardField)` if the field is absent or
/// not a `Value::Str`.
fn extract_card_str(
    map: &OrderedMap<Arc<str>, Value>,
    field: &str,
    source_slide_index: usize,
    card_index: usize,
) -> Result<Arc<str>, LayoutError> {
    match map.get(field) {
        Some(Value::Str(s)) => Ok(Arc::clone(s)),
        _ => Err(LayoutError::MissingRiskCardField {
            source_slide_index,
            card_index,
            field: field.to_owned(),
        }),
    }
}

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown
)]
mod tests {
    use std::sync::Arc;

    use slideforge_types::{
        Deck, DeckMetadata, FieldValue, OrderedMap, Register, SectionBlock, Slide, SourceSpan,
        Value,
    };

    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Test helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_metadata() -> DeckMetadata {
        DeckMetadata {
            title: Some(Arc::from("Test Deck")),
            slideforge_version: Arc::from("0.1.0"),
            lang: Some(Arc::from("en-US")),
            author: None,
            section_order: None,
        }
    }

    fn make_deck(slides: Vec<Slide>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn make_deck_with_section_blocks(slides: Vec<Slide>, blocks: Vec<SectionBlock>) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: make_metadata(),
            registers: OrderedMap::new(),
            section_blocks: blocks,
        }
    }

    fn make_deck_with_section_order(
        slides: Vec<Slide>,
        blocks: Vec<SectionBlock>,
        order: &[&str],
    ) -> Deck {
        Deck {
            slides,
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: Some(Arc::from("Test Deck")),
                slideforge_version: Arc::from("0.1.0"),
                lang: Some(Arc::from("en-US")),
                author: None,
                section_order: Some(order.iter().map(|&s| Arc::from(s)).collect()),
            },
            registers: OrderedMap::new(),
            section_blocks: blocks,
        }
    }

    fn make_slide(slide_type: &str) -> Slide {
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

    fn make_slide_with_takeaway(slide_type: &str, takeaway: &str) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Literal(Value::Str(Arc::from(takeaway))),
        );
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

    fn make_slide_with_takeaway_and_register(
        slide_type: &str,
        takeaway: &str,
        register: Register,
    ) -> Slide {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Literal(Value::Str(Arc::from(takeaway))),
        );
        Slide {
            slide_type: Arc::from(slide_type),
            fields,
            blocks: vec![],
            register: Some(register),
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Make a severity_cards slide whose cards are in a Value::List — the
    /// correct structure per CRIT-001 fix.
    fn make_severity_cards_slide_with_cards(
        cards: Vec<(&str, &str, &str, &str)>, // (title, severity, description, owner)
    ) -> Slide {
        let card_values: Vec<Value> = cards
            .into_iter()
            .map(|(title, severity, description, owner)| {
                let mut m = OrderedMap::new();
                m.insert(Arc::from("title"), Value::Str(Arc::from(title)));
                m.insert(Arc::from("severity"), Value::Str(Arc::from(severity)));
                m.insert(Arc::from("description"), Value::Str(Arc::from(description)));
                m.insert(Arc::from("owner"), Value::Str(Arc::from(owner)));
                Value::Map(m)
            })
            .collect();

        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("cards"),
            FieldValue::Literal(Value::List(card_values)),
        );
        Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// Legacy helper for tests that use a single flat severity_cards slide
    /// (the old structure with top-level title/severity/description/owner fields).
    /// These tests now use the new `make_severity_cards_slide_with_cards` helper.
    fn make_severity_card_slide(
        title: &str,
        severity: &str,
        description: &str,
        owner: &str,
    ) -> Slide {
        make_severity_cards_slide_with_cards(vec![(title, severity, description, owner)])
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-001 — executive_summary from takeaway fields
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — deck with 3 takeaway slides produces executive_summary with 3
    /// items in slide order.
    #[test]
    fn test_bc_3_02_001_executive_summary_three_takeaways() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "Takeaway from slide 1"),
            make_slide("blank"),
            make_slide_with_takeaway("bullets", "Takeaway from slide 3"),
            make_slide_with_takeaway("quote", "Takeaway from slide 4"),
        ]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("deck with takeaway slides must produce an executive_summary section");
        assert_eq!(
            section.kind,
            SectionKind::ExecutiveSummary,
            "section kind must be ExecutiveSummary"
        );
        assert_eq!(
            section.source,
            SectionSource::AutoGenerated,
            "auto-collected section must have source AutoGenerated"
        );
        assert_eq!(
            section.items.len(),
            3,
            "must collect exactly 3 TakeawayBullet items (slide without takeaway is skipped)"
        );
        assert_eq!(
            section.items[0],
            SectionItem::TakeawayBullet(Arc::from("Takeaway from slide 1")),
        );
        assert_eq!(
            section.items[1],
            SectionItem::TakeawayBullet(Arc::from("Takeaway from slide 3")),
        );
        assert_eq!(
            section.items[2],
            SectionItem::TakeawayBullet(Arc::from("Takeaway from slide 4")),
        );
    }

    /// AC-001 — heading must be "Executive Summary".
    #[test]
    fn test_bc_3_02_001_executive_summary_heading() {
        let deck = make_deck(vec![make_slide_with_takeaway("content", "Key point")]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.heading.as_ref(),
            "Executive Summary",
            "heading must be exactly 'Executive Summary'"
        );
    }

    /// AC-001 — target_formats must include Docx and Pdf.
    #[test]
    fn test_bc_3_02_001_executive_summary_target_formats() {
        let deck = make_deck(vec![make_slide_with_takeaway("content", "Key point")]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("must produce section");
        assert!(
            section.target_formats.contains(&OutputFormat::Docx),
            "executive_summary must target Docx"
        );
        assert!(
            section.target_formats.contains(&OutputFormat::Pdf),
            "executive_summary must target Pdf"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-003 — section omitted when no contributing slides
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001 / AC-003 — deck with no takeaway fields produces no executive_summary.
    #[test]
    fn test_bc_3_02_001_executive_summary_absent_when_no_takeaways() {
        let deck = make_deck(vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("blank"),
        ]);
        let result = collect_executive_summary(&deck).expect("no error");
        assert!(
            result.is_none(),
            "collect_executive_summary must return None when no slides have a takeaway field"
        );
    }

    /// AC-003 — empty deck produces no executive_summary.
    #[test]
    fn test_bc_3_02_001_executive_summary_absent_for_empty_deck() {
        let deck = make_deck(vec![]);
        let result = collect_executive_summary(&deck).expect("no error");
        assert!(
            result.is_none(),
            "collect_executive_summary must return None for a deck with no slides"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-002 — risk_register from severity_cards slides (CRIT-001)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 — deck with severity_cards slides produces a risk_register section.
    /// CRIT-001 fix: cards are in Value::List, one RiskRow per card.
    #[test]
    fn test_bc_3_02_001_risk_register_from_severity_cards() {
        let deck = make_deck(vec![
            make_slide("title"),
            make_severity_card_slide("Budget Risk", "High", "May exceed by 20%", "CFO"),
            make_severity_card_slide("Timeline Risk", "Medium", "Delay possible in Q3", "PM"),
        ]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("deck with severity_cards slides must produce a risk_register section");
        assert_eq!(
            section.kind,
            SectionKind::RiskRegister,
            "section kind must be RiskRegister"
        );
        assert_eq!(
            section.source,
            SectionSource::AutoGenerated,
            "auto-collected section must have source AutoGenerated"
        );
        assert_eq!(
            section.items.len(),
            2,
            "must collect exactly 2 RiskRow items"
        );
        assert_eq!(
            section.items[0],
            SectionItem::RiskRow {
                title: Arc::from("Budget Risk"),
                severity: Arc::from("High"),
                description: Arc::from("May exceed by 20%"),
                owner: Arc::from("CFO"),
            }
        );
    }

    /// CRIT-001 — ONE severity_cards slide with 3 cards produces 3 RiskRow items.
    #[test]
    fn test_bc_3_02_001_risk_register_one_slide_three_cards() {
        let slide = make_severity_cards_slide_with_cards(vec![
            ("Risk A", "High", "Description A", "Owner A"),
            ("Risk B", "Medium", "Description B", "Owner B"),
            ("Risk C", "Low", "Description C", "Owner C"),
        ]);
        let deck = make_deck(vec![slide]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.items.len(),
            3,
            "one severity_cards slide with 3 cards must produce 3 RiskRow items (CRIT-001)"
        );
        assert_eq!(
            section.items[0],
            SectionItem::RiskRow {
                title: Arc::from("Risk A"),
                severity: Arc::from("High"),
                description: Arc::from("Description A"),
                owner: Arc::from("Owner A"),
            }
        );
        assert_eq!(
            section.items[2],
            SectionItem::RiskRow {
                title: Arc::from("Risk C"),
                severity: Arc::from("Low"),
                description: Arc::from("Description C"),
                owner: Arc::from("Owner C"),
            }
        );
    }

    /// CRIT-001 — TWO severity_cards slides, 3 cards + 2 cards = 5 RiskRow items.
    #[test]
    fn test_bc_3_02_001_risk_register_two_slides_five_cards_total() {
        let slide1 = make_severity_cards_slide_with_cards(vec![
            ("Risk A", "High", "Desc A", "Owner A"),
            ("Risk B", "Medium", "Desc B", "Owner B"),
            ("Risk C", "Low", "Desc C", "Owner C"),
        ]);
        let slide2 = make_severity_cards_slide_with_cards(vec![
            ("Risk D", "High", "Desc D", "Owner D"),
            ("Risk E", "Critical", "Desc E", "Owner E"),
        ]);
        let deck = make_deck(vec![slide1, slide2]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.items.len(),
            5,
            "3 cards + 2 cards across two slides must yield 5 RiskRow items (CRIT-001)"
        );
    }

    /// AC-002 — heading for risk_register must be "Risk Register".
    #[test]
    fn test_bc_3_02_001_risk_register_heading() {
        let deck = make_deck(vec![make_severity_card_slide(
            "Risk A",
            "Low",
            "Minor issue",
            "Team Lead",
        )]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.heading.as_ref(),
            "Risk Register",
            "heading must be exactly 'Risk Register'"
        );
    }

    /// AC-003 — deck with no severity_cards slides produces no risk_register.
    #[test]
    fn test_bc_3_02_001_risk_register_absent_when_no_severity_cards() {
        let deck = make_deck(vec![make_slide("title"), make_slide("content")]);
        let result = collect_risk_register(&deck).expect("no error");
        assert!(
            result.is_none(),
            "collect_risk_register must return None when no severity_cards slides exist"
        );
    }

    /// EC-003 — multiple severity_cards slides produce rows in a single section, no dedup.
    #[test]
    fn test_bc_3_02_001_risk_register_multiple_slides_no_dedup() {
        // Two slides with identical risk data — both rows must appear.
        let deck = make_deck(vec![
            make_severity_card_slide("Same Risk", "High", "Repeated", "Owner"),
            make_severity_card_slide("Same Risk", "High", "Repeated", "Owner"),
        ]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.items.len(),
            2,
            "risk rows must not be de-duplicated (EC-003)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Type structure tests — SectionKind, SectionSource, SectionItem, GeneratedSection
    // ─────────────────────────────────────────────────────────────────────────

    /// All SectionKind variants exist and are pattern-matchable.
    #[test]
    fn test_bc_3_02_001_section_kind_variants_exist() {
        let exec = SectionKind::ExecutiveSummary;
        let risk = SectionKind::RiskRegister;
        let manual = SectionKind::ManualSection(Arc::from("methodology"));
        assert!(matches!(exec, SectionKind::ExecutiveSummary));
        assert!(matches!(risk, SectionKind::RiskRegister));
        assert!(matches!(manual, SectionKind::ManualSection(_)));
    }

    /// SectionSource variants exist.
    #[test]
    fn test_bc_3_02_001_section_source_variants_exist() {
        let auto = SectionSource::AutoGenerated;
        let manual = SectionSource::ManuallyAuthored;
        assert!(matches!(auto, SectionSource::AutoGenerated));
        assert!(matches!(manual, SectionSource::ManuallyAuthored));
    }

    /// SectionItem variants exist with expected fields.
    #[test]
    fn test_bc_3_02_001_section_item_variants_exist() {
        let bullet = SectionItem::TakeawayBullet(Arc::from("Key insight"));
        let row = SectionItem::RiskRow {
            title: Arc::from("Risk A"),
            severity: Arc::from("High"),
            description: Arc::from("Description"),
            owner: Arc::from("Owner"),
        };
        let custom = SectionItem::Custom(OrderedMap::new());
        assert!(matches!(bullet, SectionItem::TakeawayBullet(_)));
        assert!(matches!(row, SectionItem::RiskRow { .. }));
        assert!(matches!(custom, SectionItem::Custom(_)));
    }

    /// OutputFormat variants exist.
    #[test]
    fn test_bc_3_02_001_output_format_variants_exist() {
        let docx = OutputFormat::Docx;
        let pdf = OutputFormat::Pdf;
        let pptx = OutputFormat::Pptx;
        let html = OutputFormat::Html;
        let preview = OutputFormat::Preview;
        assert!(matches!(docx, OutputFormat::Docx));
        assert!(matches!(pdf, OutputFormat::Pdf));
        assert!(matches!(pptx, OutputFormat::Pptx));
        assert!(matches!(html, OutputFormat::Html));
        assert!(matches!(preview, OutputFormat::Preview));
    }

    /// GeneratedSection implements Hash + Eq + Clone (AC-010).
    #[test]
    fn test_bc_3_02_001_generated_section_implements_hash_eq_clone() {
        use std::collections::HashSet;

        let section = GeneratedSection {
            kind: SectionKind::ExecutiveSummary,
            source: SectionSource::AutoGenerated,
            items: vec![SectionItem::TakeawayBullet(Arc::from("Key point"))],
            heading: Arc::from("Executive Summary"),
            target_formats: docx_pdf_formats(),
        };
        let section2 = section.clone();
        assert_eq!(section, section2);
        let mut set = HashSet::new();
        set.insert(section);
        assert_eq!(set.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-001 — skips slides without takeaway (mixed deck)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 — deck with 5 slides where only slides 1 and 3 have a takeaway
    /// must produce a section with exactly 2 items in slide order.
    #[test]
    fn test_bc_3_02_001_executive_summary_skips_slides_without_takeaway() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "First key point"),
            make_slide("title"),
            make_slide_with_takeaway("bullets", "Second key point"),
            make_slide("stat_callout"),
            make_slide("section_break"),
        ]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("deck with 2 takeaway slides must produce an executive_summary section");
        assert_eq!(
            section.items.len(),
            2,
            "only slides with takeaway: fields contribute — 3 slides without takeaway must be skipped"
        );
        assert_eq!(
            section.items[0],
            SectionItem::TakeawayBullet(Arc::from("First key point")),
        );
        assert_eq!(
            section.items[1],
            SectionItem::TakeawayBullet(Arc::from("Second key point")),
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-002 — RiskRow field preservation
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-002 — each RiskRow preserves all four source card fields.
    #[test]
    fn test_bc_3_02_001_risk_row_preserves_card_fields() {
        let deck = make_deck(vec![make_severity_card_slide(
            "Integration Failure",
            "Critical",
            "Third-party API may be deprecated in Q2",
            "Platform Lead",
        )]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(section.items.len(), 1);
        match &section.items[0] {
            SectionItem::RiskRow {
                title,
                severity,
                description,
                owner,
            } => {
                assert_eq!(title.as_ref(), "Integration Failure");
                assert_eq!(severity.as_ref(), "Critical");
                assert_eq!(
                    description.as_ref(),
                    "Third-party API may be deprecated in Q2"
                );
                assert_eq!(owner.as_ref(), "Platform Lead");
            },
            other => panic!("expected SectionItem::RiskRow, got {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 — collect_sections integration (combines auto-generated)
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-001 + AC-002 — collect_sections on a deck with both takeaway
    /// slides and severity_cards slides returns a Vec containing both.
    /// Strict ordering: ExecutiveSummary before RiskRegister (AC-007 default).
    #[test]
    fn test_bc_3_02_001_collect_sections_combines_auto_generated() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "Strategic point"),
            make_severity_card_slide("Budget Overrun", "High", "10% over plan", "CFO"),
        ]);
        let sections = collect_sections(&deck).expect("no error");
        assert!(
            !sections.is_empty(),
            "collect_sections must return at least one section"
        );
        let has_exec_summary = sections
            .iter()
            .any(|s| s.kind == SectionKind::ExecutiveSummary);
        let has_risk_register = sections.iter().any(|s| s.kind == SectionKind::RiskRegister);
        assert!(has_exec_summary, "must include ExecutiveSummary");
        assert!(has_risk_register, "must include RiskRegister");
        // Strict ordering: ExecutiveSummary before RiskRegister (AC-007 default, LOW-003)
        assert_eq!(sections[0].kind, SectionKind::ExecutiveSummary);
        assert_eq!(sections[1].kind, SectionKind::RiskRegister);
    }

    /// AC-003 / EC-001 — collect_sections on a deck with no contributing slides
    /// returns an empty Vec.
    #[test]
    fn test_bc_3_02_001_collect_sections_empty_when_no_contributing_slides() {
        let deck = make_deck(vec![
            make_slide("title"),
            make_slide("content"),
            make_slide("section_break"),
        ]);
        let sections = collect_sections(&deck).expect("no error");
        assert!(
            sections.is_empty(),
            "collect_sections must return an empty Vec when no slides contribute sections"
        );
    }

    /// AC-003 / EC-001 — empty deck produces empty Vec.
    #[test]
    fn test_bc_3_02_001_collect_sections_empty_deck_produces_empty_vec() {
        let deck = make_deck(vec![]);
        let sections = collect_sections(&deck).expect("no error");
        assert!(sections.is_empty());
    }

    /// BC-3.02.001 postcondition 4 — deterministic.
    #[test]
    fn test_bc_3_02_001_collect_sections_is_deterministic() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "Finding A"),
            make_severity_card_slide("Risk X", "Medium", "Some risk", "Owner"),
            make_slide_with_takeaway("bullets", "Finding B"),
        ]);
        let first = collect_sections(&deck).expect("no error");
        let second = collect_sections(&deck).expect("no error");
        assert_eq!(first, second);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.002 — manual section collection (CRIT-002)
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.02.002 / AC-004 — deck with methodology SectionBlock produces
    /// ManualSection("methodology") with ManuallyAuthored source.
    #[test]
    fn test_bc_3_02_002_manual_methodology_section_passed_through() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1, "one manual section expected");
        assert_eq!(
            manual[0].kind,
            SectionKind::ManualSection(Arc::from("methodology")),
        );
    }

    /// BC-3.02.002 / AC-004 — ManualSection has ManuallyAuthored source.
    #[test]
    fn test_bc_3_02_002_manual_section_source_is_manually_authored() {
        let block = SectionBlock {
            name: Arc::from("scope"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        for section in &sections {
            if matches!(section.kind, SectionKind::ManualSection(_)) {
                assert_eq!(
                    section.source,
                    SectionSource::ManuallyAuthored,
                    "every ManualSection must have SectionSource::ManuallyAuthored"
                );
            }
        }
        assert!(!sections.is_empty(), "at least one section expected");
    }

    /// BC-3.02.002 / AC-004 — unknown section type returns LayoutError::UnknownSectionType.
    #[test]
    fn test_bc_3_02_002_unknown_section_type_errors() {
        let block = SectionBlock {
            name: Arc::from("unknown_type"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let result = collect_sections(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnknownSectionType { ref name, .. }) if name == "unknown_type"
            ),
            "expected UnknownSectionType error for unrecognised section type"
        );
    }

    /// BC-3.02.002 — all five supported section types are accepted without error.
    #[test]
    fn test_bc_3_02_002_all_supported_section_types_accepted() {
        for &type_name in &["methodology", "scope", "approval", "appendix", "glossary"] {
            let block = SectionBlock {
                name: Arc::from(type_name),
                body: OrderedMap::new(),
                register_content: vec![],
                span: SourceSpan::default(),
            };
            let deck = make_deck_with_section_blocks(vec![], vec![block]);
            assert!(
                collect_sections(&deck).is_ok(),
                "section type '{type_name}' must be accepted without error"
            );
        }
    }

    /// BC-3.02.002 — deck with no section blocks produces no ManualSection entries.
    #[test]
    fn test_bc_3_02_002_manual_section_block_passed_through() {
        let deck = make_deck(vec![make_slide("title")]);
        let sections = collect_sections(&deck).expect("no error");
        let has_manual = sections
            .iter()
            .any(|s| matches!(s.kind, SectionKind::ManualSection(_)));
        assert!(
            !has_manual,
            "a deck with no manual section blocks must not produce any ManualSection entries"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-005 — PPTX/HTML sections excluded via target_formats tag
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-005 — auto-generated sections must NOT include Pptx or Html.
    #[test]
    fn test_bc_3_02_001_executive_summary_excludes_pptx_and_html_from_target_formats() {
        let deck = make_deck(vec![make_slide_with_takeaway("content", "Key point")]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("must produce section");
        assert!(!section.target_formats.contains(&OutputFormat::Pptx));
        assert!(!section.target_formats.contains(&OutputFormat::Html));
    }

    /// AC-005 — auto-generated risk_register must NOT include Pptx or Html.
    #[test]
    fn test_bc_3_02_001_risk_register_excludes_pptx_and_html_from_target_formats() {
        let deck = make_deck(vec![make_severity_card_slide(
            "Risk", "Low", "Minor", "Owner",
        )]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        assert!(!section.target_formats.contains(&OutputFormat::Pptx));
        assert!(!section.target_formats.contains(&OutputFormat::Html));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-004 — target_formats is always sorted
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-004 — target_formats on auto-generated sections is sorted.
    #[test]
    fn test_high_004_target_formats_sorted() {
        let deck = make_deck(vec![make_slide_with_takeaway("content", "Key point")]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("must produce section");
        let sorted = {
            let mut v = section.target_formats.clone();
            v.sort();
            v
        };
        assert_eq!(
            section.target_formats, sorted,
            "target_formats must be sorted (HIGH-004)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.001 / AC-005 — UnknownSectionType error variant reachability
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-005 — `LayoutError::UnknownSectionType` can be constructed and
    /// displays a human-readable message containing the unknown name and known
    /// types list (BC-3.02.002 EC-001).
    ///
    /// Also verifies that the span is rendered via `Display` (compact
    /// `file:line:col` / `<unknown>` form), NOT via `Debug` (verbose struct
    /// dump). The `Display` impl on `SourceSpan` was added specifically for
    /// clean error rendering; the error format string must use `{span}` not
    /// `{span:?}`.
    #[test]
    fn test_bc_3_02_002_unknown_section_type_error_variant_exists() {
        use crate::error::LayoutError;
        let err = LayoutError::UnknownSectionType {
            name: "frobnicator".to_owned(),
            span: SourceSpan::default(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("frobnicator"),
            "UnknownSectionType error must include the unknown name; got: {msg}"
        );
        assert!(
            msg.contains("Known types"),
            "UnknownSectionType error must include 'Known types' list (BC-3.02.002 EC-001); got: {msg}"
        );
        // Verify span uses Display (compact form), NOT Debug (struct dump).
        // SourceSpan::default() has an empty file so Display renders as "<unknown>".
        // Debug would render as "SourceSpan { file: \"\", line: 0, col: 0, byte_offset: 0 }".
        assert!(
            msg.contains("<unknown>"),
            "span must render via Display ('<unknown>' for default span), not Debug struct dump; got: {msg}"
        );
        assert!(
            !msg.contains("SourceSpan {"),
            "span must NOT render as a Debug struct dump; got: {msg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-002 — UnresolvedTakeaway error for non-Literal(Str) takeaway values
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-002 — Expr takeaway variant returns UnresolvedTakeaway error.
    #[test]
    fn test_high_002_expr_takeaway_returns_error() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Expr(Arc::from("some_expr")),
        );
        let slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_executive_summary(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnresolvedTakeaway {
                    source_slide_index: 0
                })
            ),
            "Expr takeaway must return UnresolvedTakeaway at slide index 0"
        );
    }

    /// HIGH-002 — Interpolated takeaway variant returns UnresolvedTakeaway error.
    #[test]
    fn test_high_002_interpolated_takeaway_returns_error() {
        use slideforge_types::StringPart;
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Interpolated(vec![StringPart::Literal(Arc::from("hello"))]),
        );
        let slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_executive_summary(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnresolvedTakeaway {
                    source_slide_index: 0
                })
            ),
            "Interpolated takeaway must return UnresolvedTakeaway"
        );
    }

    /// HIGH-002 — Inlines takeaway variant returns UnresolvedTakeaway error.
    #[test]
    fn test_high_002_inlines_takeaway_returns_error() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("takeaway"), FieldValue::Inlines(vec![]));
        let slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_executive_summary(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnresolvedTakeaway {
                    source_slide_index: 0
                })
            ),
            "Inlines takeaway must return UnresolvedTakeaway"
        );
    }

    /// HIGH-002 — Non-string Literal (e.g., Bool) takeaway returns UnresolvedTakeaway.
    #[test]
    fn test_high_002_non_str_literal_takeaway_returns_error() {
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Literal(Value::Bool(true)),
        );
        let slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_executive_summary(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnresolvedTakeaway {
                    source_slide_index: 0
                })
            ),
            "Non-Str Literal takeaway must return UnresolvedTakeaway"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-001 — MissingRiskCardField error for missing required fields
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-001 — severity_cards slide where a card map is missing 'title' returns error.
    #[test]
    fn test_high_001_missing_risk_card_field_returns_error() {
        let mut card = OrderedMap::new();
        card.insert(Arc::from("severity"), Value::Str(Arc::from("High")));
        card.insert(Arc::from("description"), Value::Str(Arc::from("Desc")));
        card.insert(Arc::from("owner"), Value::Str(Arc::from("Owner")));
        // 'title' is absent — must error.

        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("cards"),
            FieldValue::Literal(Value::List(vec![Value::Map(card)])),
        );
        let slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_risk_register(&deck);
        assert!(
            matches!(result, Err(LayoutError::MissingRiskCardField { field, .. }) if field == "title"),
            "missing 'title' field must return MissingRiskCardField"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-002 — Notes-register severity_cards slides excluded from risk_register
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-002 — severity_cards slide with register: Notes must NOT contribute
    /// to the risk_register section. BC-3.02.001 Invariant 4: the Notes-register
    /// exclusion applies to ALL auto-generated sections.
    #[test]
    fn test_bc_3_02_001_notes_register_severity_cards_excluded_from_risk_register() {
        // One Notes-register severity_cards slide + one non-Notes severity_cards slide.
        let notes_slide = {
            let card_values = vec![{
                let mut m = OrderedMap::new();
                m.insert(Arc::from("title"), Value::Str(Arc::from("Notes-only risk")));
                m.insert(Arc::from("severity"), Value::Str(Arc::from("Low")));
                m.insert(
                    Arc::from("description"),
                    Value::Str(Arc::from("Must not appear")),
                );
                m.insert(Arc::from("owner"), Value::Str(Arc::from("Nobody")));
                Value::Map(m)
            }];
            let mut fields = OrderedMap::new();
            fields.insert(
                Arc::from("cards"),
                FieldValue::Literal(Value::List(card_values)),
            );
            Slide {
                slide_type: Arc::from("severity_cards"),
                fields,
                blocks: vec![],
                register: Some(Register::Notes),
                tags: vec![],
                source_span: SourceSpan::default(),
                overlay: None,
                register_content: vec![],
            }
        };
        let normal_slide =
            make_severity_cards_slide_with_cards(vec![("SQL Injection", "High", "DB risk", "DBA")]);

        let deck = make_deck(vec![notes_slide, normal_slide]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("deck with one non-Notes severity_cards slide must produce risk_register");

        // Only the non-Notes card must appear.
        assert_eq!(
            section.items.len(),
            1,
            "Notes-register severity_cards slide must be excluded; only 1 non-Notes card expected"
        );
        match &section.items[0] {
            SectionItem::RiskRow { title, .. } => {
                assert_eq!(
                    title.as_ref(),
                    "SQL Injection",
                    "wrong card in risk register"
                );
            },
            other => panic!("expected RiskRow, got {other:?}"),
        }
    }

    /// HIGH-002 — deck with ONLY Notes-register severity_cards slides produces no
    /// risk_register section (returns Ok(None)).
    #[test]
    fn test_bc_3_02_001_all_notes_severity_cards_returns_none() {
        let notes_slide = {
            let card_values = vec![{
                let mut m = OrderedMap::new();
                m.insert(Arc::from("title"), Value::Str(Arc::from("Notes risk")));
                m.insert(Arc::from("severity"), Value::Str(Arc::from("Low")));
                m.insert(
                    Arc::from("description"),
                    Value::Str(Arc::from("Notes only")),
                );
                m.insert(Arc::from("owner"), Value::Str(Arc::from("Nobody")));
                Value::Map(m)
            }];
            let mut fields = OrderedMap::new();
            fields.insert(
                Arc::from("cards"),
                FieldValue::Literal(Value::List(card_values)),
            );
            Slide {
                slide_type: Arc::from("severity_cards"),
                fields,
                blocks: vec![],
                register: Some(Register::Notes),
                tags: vec![],
                source_span: SourceSpan::default(),
                overlay: None,
                register_content: vec![],
            }
        };

        let deck = make_deck(vec![notes_slide]);
        let result = collect_risk_register(&deck).expect("no error");
        assert!(
            result.is_none(),
            "deck with only Notes-register severity_cards must return Ok(None)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006 — supersession: manual section replaces auto-generated
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-006 — when a manual section with a DIFFERENT key than the auto-generated
    /// executive_summary is present, both appear in the output (no suppression).
    ///
    /// This test verifies the coexistence case: methodology (manual) and
    /// executive_summary (auto) have different order_keys, so both survive.
    /// The supersession rule only fires when manual and auto sections share the
    /// same order_key (e.g., a manual "executive_summary" block suppresses the
    /// auto-generated executive_summary). That path is covered by
    /// `test_ac_006_manual_executive_summary_rewrite_supersedes`.
    #[test]
    fn test_ac_006_different_key_manual_does_not_suppress_auto() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // methodology (manual) + takeaway slide (auto executive_summary):
        // different order_keys so both must appear.
        let deck = make_deck_with_section_blocks(
            vec![make_slide_with_takeaway("content", "Auto takeaway")],
            vec![block],
        );
        let sections = collect_sections(&deck).expect("no error");
        // Both methodology (manual) and executive_summary (auto) should be present.
        let has_manual = sections
            .iter()
            .any(|s| s.kind == SectionKind::ManualSection(Arc::from("methodology")));
        let has_auto = sections
            .iter()
            .any(|s| s.kind == SectionKind::ExecutiveSummary);
        assert!(has_manual, "manual methodology section must be present");
        assert!(
            has_auto,
            "auto executive summary must also be present (different key)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-007 — section_order sorting
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-007 — section_order in metadata reorders sections accordingly.
    #[test]
    fn test_ac_007_section_order_reorders_sections() {
        // Without section_order: executive_summary first (default auto order).
        // With section_order: ["risk_register", "executive_summary"] — risk first.
        let deck = make_deck_with_section_order(
            vec![
                make_slide_with_takeaway("content", "Key point"),
                make_severity_card_slide("Risk A", "High", "Desc A", "Owner A"),
            ],
            vec![],
            &["risk_register", "executive_summary"],
        );
        let sections = collect_sections(&deck).expect("no error");
        assert_eq!(sections.len(), 2);
        assert_eq!(
            sections[0].kind,
            SectionKind::RiskRegister,
            "risk_register must come first"
        );
        assert_eq!(
            sections[1].kind,
            SectionKind::ExecutiveSummary,
            "executive_summary must come second"
        );
    }

    /// AC-007 — sections not listed in section_order appear at the end.
    #[test]
    fn test_ac_007_unlisted_sections_appear_at_end() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_order(
            vec![
                make_slide_with_takeaway("content", "Key point"),
                make_severity_card_slide("Risk A", "High", "Desc A", "Owner A"),
            ],
            vec![block],
            // Only list risk_register — methodology and executive_summary go to end.
            &["risk_register"],
        );
        let sections = collect_sections(&deck).expect("no error");
        assert_eq!(sections.len(), 3);
        assert_eq!(
            sections[0].kind,
            SectionKind::RiskRegister,
            "risk_register listed first"
        );
        // methodology and executive_summary are at the end in whatever order (both have
        // sort key usize::MAX).
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MED-004 — Notes-register slides excluded from executive_summary
    // ─────────────────────────────────────────────────────────────────────────

    /// MED-004 — slides with register: Notes are excluded from executive_summary.
    #[test]
    fn test_med_004_notes_register_slide_excluded_from_exec_summary() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "Public takeaway"),
            make_slide_with_takeaway_and_register(
                "content",
                "Notes-only takeaway",
                Register::Notes,
            ),
        ]);
        let section = collect_executive_summary(&deck)
            .expect("no error")
            .expect("must produce section");
        assert_eq!(
            section.items.len(),
            1,
            "Notes-register slide must be excluded (MED-004)"
        );
        assert_eq!(
            section.items[0],
            SectionItem::TakeawayBullet(Arc::from("Public takeaway"))
        );
    }

    /// MED-004 — deck with only Notes-register takeaway slides produces None.
    #[test]
    fn test_med_004_all_notes_register_slides_produces_none() {
        let deck = make_deck(vec![make_slide_with_takeaway_and_register(
            "content",
            "Notes-only",
            Register::Notes,
        )]);
        let result = collect_executive_summary(&deck).expect("no error");
        assert!(result.is_none(), "all-Notes deck must produce None");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // New error variant reachability tests
    // ─────────────────────────────────────────────────────────────────────────

    /// LayoutError::UnresolvedTakeaway can be constructed.
    #[test]
    fn test_error_unresolved_takeaway_variant_exists() {
        use crate::error::LayoutError;
        let err = LayoutError::UnresolvedTakeaway {
            source_slide_index: 3,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("takeaway") || msg.contains("unresolved"),
            "error message must mention takeaway; got: {msg}"
        );
    }

    /// LayoutError::MissingRiskCardField can be constructed.
    #[test]
    fn test_error_missing_risk_card_field_variant_exists() {
        use crate::error::LayoutError;
        let err = LayoutError::MissingRiskCardField {
            source_slide_index: 1,
            card_index: 0,
            field: "severity".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("severity"),
            "error message must mention field name; got: {msg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CRIT-001 — executive_summary and risk_register as manual section types
    // ─────────────────────────────────────────────────────────────────────────

    /// CRIT-001 — `executive_summary` is now a supported manual section type.
    /// A `section executive_summary:` block must NOT return UnknownSectionType.
    #[test]
    fn test_crit_001_executive_summary_accepted_as_manual_section_type() {
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let result = collect_sections(&deck);
        assert!(
            result.is_ok(),
            "executive_summary must be accepted as a manual section type (CRIT-001); \
             got: {:?}",
            result.err()
        );
        let sections = result.unwrap();
        assert!(
            sections
                .iter()
                .any(|s| s.kind == SectionKind::ManualSection(Arc::from("executive_summary"))),
            "sections must contain ManualSection(\"executive_summary\")"
        );
    }

    /// CRIT-001 — `risk_register` is now a supported manual section type.
    #[test]
    fn test_crit_001_risk_register_accepted_as_manual_section_type() {
        let block = SectionBlock {
            name: Arc::from("risk_register"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let result = collect_sections(&deck);
        assert!(
            result.is_ok(),
            "risk_register must be accepted as a manual section type (CRIT-001); \
             got: {:?}",
            result.err()
        );
    }

    /// AC-006 (rewrite, CRIT-001) — when a manual `executive_summary` section
    /// block exists alongside takeaway slides, the manual section is retained
    /// and the auto-generated ExecutiveSummary is suppressed.
    #[test]
    fn test_ac_006_manual_executive_summary_supersedes_auto_generated_rewrite() {
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(
            vec![make_slide_with_takeaway("content", "Auto takeaway")],
            vec![block],
        );
        let sections = collect_sections(&deck).expect("no error");
        // Must have exactly one executive_summary — the manual one.
        let exec_sections: Vec<_> = sections
            .iter()
            .filter(|s| {
                s.kind == SectionKind::ExecutiveSummary
                    || s.kind == SectionKind::ManualSection(Arc::from("executive_summary"))
            })
            .collect();
        assert_eq!(
            exec_sections.len(),
            1,
            "exactly one executive_summary must appear (manual supersedes auto); \
             got: {:?}",
            sections.iter().map(|s| &s.kind).collect::<Vec<_>>()
        );
        assert_eq!(
            exec_sections[0].source,
            SectionSource::ManuallyAuthored,
            "the retained executive_summary must be the manually authored one"
        );
        assert!(
            matches!(
                &exec_sections[0].kind,
                SectionKind::ManualSection(name) if name.as_ref() == "executive_summary"
            ),
            "retained section kind must be ManualSection(\"executive_summary\")"
        );
        // Auto-generated ExecutiveSummary variant must be absent.
        assert!(
            !sections
                .iter()
                .any(|s| s.kind == SectionKind::ExecutiveSummary),
            "auto-generated ExecutiveSummary must be suppressed when manual override present"
        );
    }

    /// AC-006 (F-008) — when a manual `executive_summary` block with a non-empty
    /// body supersedes the auto-generated executive summary (takeaway slides
    /// present), the supersession fires AND the manual body content is preserved
    /// in the resulting section's items.
    ///
    /// This verifies the full supersession path from AC-006: the manual block's
    /// body content reaches the exporter via `items`, not the auto-generated
    /// `TakeawayBullet` items from the slide `takeaway:` fields.
    #[test]
    fn test_ac_006_supersession_preserves_manual_body_content() {
        // Manual executive_summary block with non-empty body content.
        let mut body = OrderedMap::new();
        body.insert(
            Arc::from("summary"),
            FieldValue::Literal(Value::Str(Arc::from("Manual executive summary text"))),
        );
        body.insert(
            Arc::from("author"),
            FieldValue::Literal(Value::Str(Arc::from("Strategy Team"))),
        );
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body,
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // Two takeaway slides that would normally produce auto-generated bullets.
        let deck = make_deck_with_section_blocks(
            vec![
                make_slide_with_takeaway("content", "Auto takeaway 1"),
                make_slide_with_takeaway("bullets", "Auto takeaway 2"),
            ],
            vec![block],
        );
        let sections = collect_sections(&deck).expect("no error");

        // Supersession must fire: exactly one executive_summary section, the manual one.
        let exec_sections: Vec<_> = sections
            .iter()
            .filter(|s| {
                s.kind == SectionKind::ExecutiveSummary
                    || s.kind == SectionKind::ManualSection(Arc::from("executive_summary"))
            })
            .collect();
        assert_eq!(
            exec_sections.len(),
            1,
            "supersession must produce exactly one executive_summary section; got: {:?}",
            sections.iter().map(|s| &s.kind).collect::<Vec<_>>()
        );
        assert_eq!(
            exec_sections[0].source,
            SectionSource::ManuallyAuthored,
            "retained section must be ManuallyAuthored"
        );

        // The manual body content must be in the items, not the auto-generated bullets.
        assert_eq!(
            exec_sections[0].items.len(),
            1,
            "manual executive_summary body must produce exactly one Custom item"
        );
        match &exec_sections[0].items[0] {
            SectionItem::Custom(map) => {
                assert!(
                    map.contains_key("summary"),
                    "Custom map must contain 'summary' key from manual body"
                );
                assert!(
                    map.contains_key("author"),
                    "Custom map must contain 'author' key from manual body"
                );
                // Auto-generated takeaway text must NOT appear in the items.
                let has_takeaway_bullet = exec_sections[0]
                    .items
                    .iter()
                    .any(|item| matches!(item, SectionItem::TakeawayBullet(_)));
                assert!(
                    !has_takeaway_bullet,
                    "manual supersession must NOT include auto-generated TakeawayBullet items"
                );
            },
            other => panic!("expected SectionItem::Custom, got {other:?}"),
        }

        // The auto-generated ExecutiveSummary must be absent (superseded).
        assert!(
            !sections
                .iter()
                .any(|s| s.kind == SectionKind::ExecutiveSummary),
            "auto-generated ExecutiveSummary must be suppressed"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CRIT-002 — supersession warning via tracing
    // ─────────────────────────────────────────────────────────────────────────

    /// CRIT-002 — when a manual executive_summary block supersedes the auto-
    /// generated one, a `tracing::warn!` is emitted (BC-3.02.001 EC-002).
    #[tracing_test::traced_test]
    #[test]
    fn test_crit_002_supersession_warning_emitted_for_executive_summary() {
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(
            vec![make_slide_with_takeaway("content", "Takeaway")],
            vec![block],
        );
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            logs_contain("executive_summary overridden"),
            "supersession warning must mention 'executive_summary overridden'"
        );
    }

    /// CRIT-002 — warning is NOT emitted when manual executive_summary exists
    /// but there are no takeaway slides (nothing to suppress).
    #[tracing_test::traced_test]
    #[test]
    fn test_crit_002_no_supersession_warning_when_no_takeaways() {
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // No takeaway slides — no auto-generated section would have been produced.
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("executive_summary overridden"),
            "supersession warning must NOT fire when no takeaway slides exist"
        );
    }

    /// CRIT-002 / PR-review finding 2 — supersession warning must NOT fire
    /// when the only `takeaway` fields are unresolved `FieldValue::Expr`
    /// values. An unresolved takeaway would have caused `collect_executive_summary`
    /// to return `Err(UnresolvedTakeaway)`, NOT a section — so there is nothing
    /// to supersede, and the warning would be a false positive.
    #[tracing_test::traced_test]
    #[test]
    fn test_crit_002_no_supersession_warning_for_unresolved_expr_takeaway() {
        let block = SectionBlock {
            name: Arc::from("executive_summary"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // A slide whose `takeaway:` is an unresolved Expr — NOT a Literal(Str).
        // collect_executive_summary would have returned Err for this slide, not a section.
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("takeaway"),
            FieldValue::Expr(Arc::from("{{ some_unresolved_expr }}")),
        );
        let expr_takeaway_slide = Slide {
            slide_type: Arc::from("content"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck_with_section_blocks(vec![expr_takeaway_slide], vec![block]);
        // collect_sections must succeed (the manual section is collected; the
        // unresolved takeaway is in a superseded branch, not evaluated).
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("executive_summary overridden"),
            "supersession warning must NOT fire when the only takeaway fields are unresolved \
             Expr values — there would be no auto-generated section to supersede"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-003 — tracing::warn! for empty section body (BC-3.02.002 EC-003)
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-003 — a manual section block with an empty body emits a
    /// `tracing::warn!` diagnostic (BC-3.02.002 EC-003).
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_003_empty_section_body_emits_warn() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(), // empty body
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let _ =
            collect_sections(&deck).expect("collect_sections must succeed even with empty body");
        assert!(
            logs_contain("empty body"),
            "tracing::warn! must be emitted for section with empty body (BC-3.02.002 EC-003)"
        );
    }

    /// OBS-003 — a manual section block with a non-empty body does NOT emit
    /// the empty-body warning.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_003_non_empty_section_body_no_warn() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: {
                let mut m = OrderedMap::new();
                m.insert(
                    Arc::from("approach"),
                    FieldValue::Literal(Value::Str(Arc::from("Agile"))),
                );
                m
            },
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("empty body"),
            "empty-body warning must NOT fire when section has content"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-002 — insta snapshot test for collect_sections
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-002 — insta debug snapshot of collect_sections output for a
    /// representative deck with both takeaway and severity_cards slides.
    ///
    /// Uses `assert_debug_snapshot!` because `GeneratedSection` does not yet
    /// derive `serde::Serialize` (that derives for doc-format serialization is
    /// deferred to the exporter stories where JSON-serializable IR is needed).
    #[test]
    fn test_high_002_collect_sections_snapshot() {
        let deck = make_deck(vec![
            make_slide_with_takeaway("content", "Key finding A"),
            make_slide_with_takeaway("bullets", "Key finding B"),
            make_severity_card_slide("Budget Risk", "High", "May exceed by 15%", "CFO"),
        ]);
        let sections = collect_sections(&deck).expect("no error");
        insta::assert_debug_snapshot!(sections);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-003 — error propagation for malformed severity_cards fields
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-003 — severity_cards slide with a non-List Literal `cards:` field
    /// returns `LayoutError::MalformedSeverityCards`.
    #[test]
    fn test_high_003_malformed_severity_cards_wrong_type_returns_error() {
        let mut fields = OrderedMap::new();
        // cards: "not a list" — wrong type (a scalar string instead of list).
        fields.insert(
            Arc::from("cards"),
            FieldValue::Literal(Value::Str(Arc::from("not a list"))),
        );
        let slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_risk_register(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::MalformedSeverityCards {
                    source_slide_index: 0,
                    ..
                })
            ),
            "wrong-typed Literal 'cards' field must return MalformedSeverityCards; got: {result:?}"
        );
    }

    /// HIGH-003 — severity_cards slide with an unresolved (Expr) `cards:` field
    /// returns `LayoutError::UnresolvedSeverityCards`.
    #[test]
    fn test_high_003_unresolved_severity_cards_expr_returns_error() {
        let mut fields = OrderedMap::new();
        fields.insert(Arc::from("cards"), FieldValue::Expr(Arc::from("some_expr")));
        let slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_risk_register(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::UnresolvedSeverityCards {
                    source_slide_index: 0
                })
            ),
            "Expr 'cards' field must return UnresolvedSeverityCards; got: {result:?}"
        );
    }

    /// HIGH-003 — severity_cards slide with no `cards:` field at all returns
    /// `LayoutError::MissingRiskCardField` for the "cards" field.
    #[test]
    fn test_high_003_missing_cards_field_returns_error() {
        // severity_cards slide with no `cards:` field at all.
        let slide = Slide {
            slide_type: Arc::from("severity_cards"),
            fields: OrderedMap::new(),
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        };
        let deck = make_deck(vec![slide]);
        let result = collect_risk_register(&deck);
        assert!(
            matches!(
                result,
                Err(LayoutError::MissingRiskCardField { ref field, .. })
                if field == "cards"
            ),
            "missing 'cards' field must return MissingRiskCardField(field=cards); got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-005 — manual section body produces ONE Custom item with full map
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-005 — a manual section block with multiple body fields produces
    /// exactly ONE `SectionItem::Custom` containing all fields.
    #[test]
    fn test_high_005_manual_section_body_single_custom_item_with_full_map() {
        let mut body = OrderedMap::new();
        body.insert(
            Arc::from("author"),
            FieldValue::Literal(Value::Str(Arc::from("Alice"))),
        );
        body.insert(
            Arc::from("version"),
            FieldValue::Literal(Value::Str(Arc::from("1.0"))),
        );
        body.insert(
            Arc::from("date"),
            FieldValue::Literal(Value::Str(Arc::from("2026-01-01"))),
        );
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body,
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1, "one manual section expected");
        // Must have exactly ONE Custom item.
        assert_eq!(
            manual[0].items.len(),
            1,
            "manual section with 3 body fields must produce exactly 1 SectionItem::Custom \
             (HIGH-005 fix: one item with full map, not one item per key)"
        );
        // That item must be a Custom with all 3 keys.
        match &manual[0].items[0] {
            SectionItem::Custom(map) => {
                assert_eq!(
                    map.len(),
                    3,
                    "Custom item map must contain all 3 body fields"
                );
                assert!(map.contains_key("author"), "map must contain 'author'");
                assert!(map.contains_key("version"), "map must contain 'version'");
                assert!(map.contains_key("date"), "map must contain 'date'");
            },
            other => panic!("expected SectionItem::Custom, got {other:?}"),
        }
    }

    /// HIGH-005 — an empty section block body produces ONE Custom item with an
    /// empty map (not zero items).
    #[test]
    fn test_high_005_empty_manual_section_body_produces_single_empty_custom() {
        let block = SectionBlock {
            name: Arc::from("scope"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1, "one manual section expected");
        assert_eq!(
            manual[0].items.len(),
            1,
            "empty body must still produce 1 SectionItem::Custom (with empty map)"
        );
        match &manual[0].items[0] {
            SectionItem::Custom(map) => {
                assert!(
                    map.is_empty(),
                    "Custom item map must be empty for empty body"
                );
            },
            other => panic!("expected SectionItem::Custom, got {other:?}"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-006 — target_formats invariant: ManualSection + RiskRegister sorted
    // ─────────────────────────────────────────────────────────────────────────

    /// HIGH-006 — ManualSection sections have target_formats in canonical order.
    #[test]
    fn test_manual_section_target_formats_sorted() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1);
        let mut sorted = manual[0].target_formats.clone();
        sorted.sort();
        assert_eq!(
            manual[0].target_formats, sorted,
            "ManualSection target_formats must be sorted (HIGH-006)"
        );
    }

    /// HIGH-006 — RiskRegister sections have target_formats in canonical order.
    #[test]
    fn test_risk_register_target_formats_sorted() {
        let deck = make_deck(vec![make_severity_card_slide(
            "Risk A", "High", "Desc", "Owner",
        )]);
        let section = collect_risk_register(&deck)
            .expect("no error")
            .expect("must produce section");
        let mut sorted = section.target_formats.clone();
        sorted.sort();
        assert_eq!(
            section.target_formats, sorted,
            "RiskRegister target_formats must be sorted (HIGH-006)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-002 — manual section heading override from "heading" body key
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-002 — a manual section block with a "heading" key in the body uses
    /// that string as the section heading instead of the humanized type name.
    /// The "heading" key must NOT appear in the Custom map — it is a layout
    /// directive consumed by the heading field, not a content field (F-002).
    #[test]
    fn test_obs_002_manual_section_heading_override() {
        let mut body = OrderedMap::new();
        body.insert(
            Arc::from("heading"),
            FieldValue::Literal(Value::Str(Arc::from("Our Research Methodology"))),
        );
        body.insert(
            Arc::from("content"),
            FieldValue::Literal(Value::Str(Arc::from("Details here"))),
        );
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body,
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1);
        assert_eq!(
            manual[0].heading.as_ref(),
            "Our Research Methodology",
            "heading must use the 'heading' body key when present (OBS-002)"
        );
        // F-002: "heading" key must NOT appear in the Custom map — it is a layout
        // directive, not a content field for DOCX/PDF exporters.
        assert_eq!(
            manual[0].items.len(),
            1,
            "must have exactly one Custom item"
        );
        match &manual[0].items[0] {
            SectionItem::Custom(map) => {
                assert!(
                    !map.contains_key("heading"),
                    "custom_map must NOT contain 'heading' key — it is consumed as the section \
                     heading, not passed to exporters as content (F-002)"
                );
                assert!(
                    map.contains_key("content"),
                    "custom_map must still contain the 'content' key"
                );
            },
            other => panic!("expected SectionItem::Custom, got {other:?}"),
        }
    }

    /// OBS-002 — a manual section block without a "heading" key falls back to
    /// the humanized type name.
    #[test]
    fn test_obs_002_manual_section_heading_fallback() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![], vec![block]);
        let sections = collect_sections(&deck).expect("no error");
        let manual: Vec<_> = sections
            .iter()
            .filter(|s| matches!(&s.kind, SectionKind::ManualSection(_)))
            .collect();
        assert_eq!(manual.len(), 1);
        assert_eq!(
            manual[0].heading.as_ref(),
            "Methodology",
            "heading must fall back to humanized type name when no 'heading' key (OBS-002)"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-003 — section_order stable sort produces specific order
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-003 — when `section_order:` names only some sections, the unlisted
    /// sections appear at the end in their default order (manual before auto).
    /// Stable sort preserves relative order: methodology (manual) before
    /// executive_summary (auto) because manual sections are prepended first.
    #[test]
    fn test_obs_003_unlisted_sections_retain_default_order_after_section_order_sort() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_order(
            vec![
                make_slide_with_takeaway("content", "Key point"),
                make_severity_card_slide("Risk A", "High", "Desc A", "Owner A"),
            ],
            vec![block],
            // Only list risk_register — methodology and executive_summary go to end.
            &["risk_register"],
        );
        let sections = collect_sections(&deck).expect("no error");
        assert_eq!(sections.len(), 3, "must have 3 sections total");
        // Listed: risk_register first.
        assert_eq!(
            sections[0].kind,
            SectionKind::RiskRegister,
            "risk_register (listed) must be first"
        );
        // Unlisted in default order: manual sections first (methodology), then auto (exec summary).
        assert_eq!(
            sections[1].kind,
            SectionKind::ManualSection(Arc::from("methodology")),
            "methodology (manual, unlisted) must be second — manual before auto in default order"
        );
        assert_eq!(
            sections[2].kind,
            SectionKind::ExecutiveSummary,
            "executive_summary (auto, unlisted) must be third"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-005 — section_order unknown name warning
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-005 — section_order containing a name not in collected sections
    /// emits a `tracing::warn!` diagnostic.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_005_section_order_unknown_name_warns() {
        // section_order references "nonexistent_section" — not in the deck.
        let deck = make_deck_with_section_order(
            vec![make_slide_with_takeaway("content", "Key point")],
            vec![],
            &["executive_summary", "nonexistent_section"],
        );
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            logs_contain("nonexistent_section"),
            "warn must mention the unknown section name 'nonexistent_section'"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BC-3.02.002 — all 7 supported section types (expanded list) accepted
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-3.02.002 / CRIT-001 — all 7 supported section types (including
    /// executive_summary and risk_register) are accepted without error.
    #[test]
    fn test_bc_3_02_002_all_seven_supported_section_types_accepted() {
        for &type_name in &[
            "executive_summary",
            "risk_register",
            "methodology",
            "scope",
            "approval",
            "appendix",
            "glossary",
        ] {
            let block = SectionBlock {
                name: Arc::from(type_name),
                body: OrderedMap::new(),
                register_content: vec![],
                span: SourceSpan::default(),
            };
            let deck = make_deck_with_section_blocks(vec![], vec![block]);
            assert!(
                collect_sections(&deck).is_ok(),
                "section type '{type_name}' must be accepted without error (CRIT-001)"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LayoutError new variants reachability
    // ─────────────────────────────────────────────────────────────────────────

    /// LayoutError::MalformedSeverityCards can be constructed and displays a
    /// meaningful message.
    #[test]
    fn test_error_malformed_severity_cards_variant_exists() {
        use crate::error::LayoutError;
        let err = LayoutError::MalformedSeverityCards {
            source_slide_index: 2,
            reason: "expected List, found String".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("wrong type") || msg.contains("expected List"),
            "MalformedSeverityCards message must mention type mismatch; got: {msg}"
        );
        assert!(
            msg.contains('2'),
            "MalformedSeverityCards message must include slide index; got: {msg}"
        );
    }

    /// LayoutError::UnresolvedSeverityCards can be constructed and displays a
    /// meaningful message.
    #[test]
    fn test_error_unresolved_severity_cards_variant_exists() {
        use crate::error::LayoutError;
        let err = LayoutError::UnresolvedSeverityCards {
            source_slide_index: 3,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("unresolved") || msg.contains("FieldValue"),
            "UnresolvedSeverityCards message must mention unresolved; got: {msg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-A (Pass 5) — Notes-only severity_cards must NOT trigger the
    // risk_register supersession warning
    // ─────────────────────────────────────────────────────────────────────────

    /// Helper: severity_cards slide with an explicit register.
    fn make_severity_cards_slide_with_register(register: Register) -> Slide {
        let card_values = vec![{
            let mut m = OrderedMap::new();
            m.insert(Arc::from("title"), Value::Str(Arc::from("Risk A")));
            m.insert(Arc::from("severity"), Value::Str(Arc::from("High")));
            m.insert(Arc::from("description"), Value::Str(Arc::from("Desc")));
            m.insert(Arc::from("owner"), Value::Str(Arc::from("Owner")));
            Value::Map(m)
        }];
        let mut fields = OrderedMap::new();
        fields.insert(
            Arc::from("cards"),
            FieldValue::Literal(Value::List(card_values)),
        );
        Slide {
            slide_type: Arc::from("severity_cards"),
            fields,
            blocks: vec![],
            register: Some(register),
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],
        }
    }

    /// OBS-A (Pass 5) — when a manual risk_register block supersedes, the
    /// supersession warning must NOT fire if all severity_cards slides are
    /// Notes-register (they would not have contributed to the auto-generated
    /// risk_register anyway).
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_a_risk_register_supersession_no_warn_for_notes_only_severity_cards() {
        let manual_block = SectionBlock {
            name: Arc::from("risk_register"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // Only severity_cards slide is Notes-register — it would not have
        // contributed to the auto-generated risk_register.
        let deck = make_deck_with_section_blocks(
            vec![make_severity_cards_slide_with_register(Register::Notes)],
            vec![manual_block],
        );
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("risk_register overridden"),
            "supersession warning must NOT fire when all severity_cards slides are Notes-register"
        );
    }

    /// OBS-A (Pass 5) — supersession warning DOES fire when at least one
    /// non-Notes severity_cards slide exists alongside a manual risk_register.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_a_risk_register_supersession_warns_for_non_notes_severity_cards() {
        let manual_block = SectionBlock {
            name: Arc::from("risk_register"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        // Non-Notes severity_cards slide — the auto-generated risk_register
        // would have been produced, so the supersession warning must fire.
        let deck = make_deck_with_section_blocks(
            vec![make_severity_card_slide(
                "Budget Risk",
                "High",
                "Desc",
                "CFO",
            )],
            vec![manual_block],
        );
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            logs_contain("risk_register overridden"),
            "supersession warning must fire when non-Notes severity_cards slides exist"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-B (Pass 5) — non-Str heading value emits tracing::warn!
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-B (Pass 5) — when a section block has a "heading" key with a
    /// non-string value, a `tracing::warn!` is emitted and the humanized
    /// fallback is used as the heading.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_b_non_str_heading_emits_warn_and_uses_fallback() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: {
                let mut m = OrderedMap::new();
                // heading key is a number, not a string (triggers OBS-B warning)
                m.insert(
                    Arc::from("heading"),
                    FieldValue::Literal(Value::Int(42)),
                );
                m
            },
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let sections = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            logs_contain("non-string value"),
            "warn must mention 'non-string value' for non-Str heading"
        );
        // The fallback heading must be the humanized section name.
        assert_eq!(
            sections[0].heading.as_ref(),
            "Methodology",
            "heading must fall back to humanized section name when heading value is non-Str"
        );
    }

    /// OBS-B (Pass 5) — a plain-string heading does NOT emit the non-Str warning.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_b_str_heading_no_warn() {
        let block = SectionBlock {
            name: Arc::from("methodology"),
            body: {
                let mut m = OrderedMap::new();
                m.insert(
                    Arc::from("heading"),
                    FieldValue::Literal(Value::Str(Arc::from("Our Approach"))),
                );
                m
            },
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block]);
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("non-string value"),
            "non-Str heading warn must NOT fire for a valid string heading"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OBS-D (Pass 5) — duplicate manual section names emit tracing::warn!
    // ─────────────────────────────────────────────────────────────────────────

    /// OBS-D (Pass 5) — when two manual section blocks share the same type name,
    /// a `tracing::warn!` is emitted and both blocks are still collected.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_d_duplicate_section_name_emits_warn_and_both_collected() {
        let block_a = SectionBlock {
            name: Arc::from("methodology"),
            body: {
                let mut m = OrderedMap::new();
                m.insert(
                    Arc::from("approach"),
                    FieldValue::Literal(Value::Str(Arc::from("Agile"))),
                );
                m
            },
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let block_b = SectionBlock {
            name: Arc::from("methodology"),
            body: {
                let mut m = OrderedMap::new();
                m.insert(
                    Arc::from("approach"),
                    FieldValue::Literal(Value::Str(Arc::from("Waterfall"))),
                );
                m
            },
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block_a, block_b]);
        let sections =
            collect_sections(&deck).expect("collect_sections must succeed with duplicates");
        assert!(
            logs_contain("declared more than once"),
            "duplicate section warning must mention 'declared more than once'"
        );
        // Both methodology blocks must be present in the result.
        let methodology_count = sections
            .iter()
            .filter(|s| s.kind == SectionKind::ManualSection(Arc::from("methodology")))
            .count();
        assert_eq!(
            methodology_count, 2,
            "both duplicate methodology blocks must be collected (2 expected, got {methodology_count})"
        );
    }

    /// OBS-D (Pass 5) — a section with a unique name does NOT emit the
    /// duplicate warning.
    #[tracing_test::traced_test]
    #[test]
    fn test_obs_d_unique_section_names_no_warn() {
        let block_a = SectionBlock {
            name: Arc::from("methodology"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let block_b = SectionBlock {
            name: Arc::from("scope"),
            body: OrderedMap::new(),
            register_content: vec![],
            span: SourceSpan::default(),
        };
        let deck = make_deck_with_section_blocks(vec![make_slide("title")], vec![block_a, block_b]);
        let _ = collect_sections(&deck).expect("collect_sections must succeed");
        assert!(
            !logs_contain("declared more than once"),
            "duplicate warning must NOT fire when all section names are unique"
        );
    }
}
