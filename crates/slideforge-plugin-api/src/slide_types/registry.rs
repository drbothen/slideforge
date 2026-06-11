//! [`SlideTypeRegistry`] — the runtime registry of all 34 built-in slide types.
//!
//! The registry maps DSL keywords (e.g., `"title"`, `"content"`) to their
//! [`SlideType`] implementations. It is the single source of truth for:
//!
//! - Keyword → type dispatch (used by the evaluator and layout engine)
//! - Typo suggestions (Levenshtein distance ≤ 3, used in error messages)
//! - Field validation ([`validate_fields`] — accumulates all diagnostics)
//!
//! ## Default registry
//!
//! [`SlideTypeRegistry::default`] pre-registers all 34 built-in slide types
//! (31 original + `status`, `progress_bar`, `weighted_composite` added in STORY-087).
//!
//! Note: the parser keyword set (`SLIDE_TYPE_KEYWORDS` in `slideforge-syntax`) contains
//! 35 entries — the 34 registered types plus `severity_cards`, which is a color-coded
//! scan target without a standalone `SlideType` registration.
//!
//! ## Thread safety
//!
//! The registry itself is not `Sync` (it holds `Box<dyn SlideType + Send + Sync>`
//! behind a mutable interface). Wrap in `Arc<RwLock<SlideTypeRegistry>>` when
//! sharing across threads, or use the `SLIDE_TYPE_REGISTRY` lazy-lock singleton.

use std::collections::HashMap;
use std::sync::Arc;

use slideforge_types::Slide;

use crate::traits::{
    Diagnostic, DiagnosticSeverity, FieldType, SlideType, type_matches, value_type_name,
};
use slideforge_types::{FieldValue, Value};

use super::{
    agenda::AgendaSlideType, bio::BioSlideType, blank::BlankSlideType, chart::ChartSlideType,
    closing::ClosingSlideType, code_sample::CodeSampleSlideType, comparison::ComparisonSlideType,
    content::ContentSlideType, diagram::DiagramSlideType,
    executive_summary::ExecutiveSummarySlideType, financials::FinancialsSlideType,
    image::ImageSlideType, kpi_dashboard::KpiDashboardSlideType, matrix::MatrixSlideType,
    org_chart::OrgChartSlideType, problem_statement::ProblemStatementSlideType,
    process_flow::ProcessFlowSlideType, progress_bar::ProgressBarSlideType, quote::QuoteSlideType,
    recommendation::RecommendationSlideType, risk_register::RiskRegisterSlideType,
    roadmap::RoadmapSlideType, screenshot::ScreenshotSlideType,
    section_break::SectionBreakSlideType, stat_callout::StatCalloutSlideType,
    status::StatusSlideType, survey_results::SurveyResultsSlideType, team::TeamSlideType,
    timeline::TimelineSlideType, title::TitleSlideType, toc::TocSlideType,
    two_col::TwoColSlideType, video::VideoSlideType,
    weighted_composite::WeightedCompositeSlideType,
};

/// The runtime registry of all registered slide types.
///
/// `SlideTypeRegistry` holds every registered [`SlideType`] implementation
/// and exposes keyword-based dispatch and typo suggestions.
pub struct SlideTypeRegistry {
    /// Map from type keyword to boxed implementation.
    types: HashMap<Arc<str>, Box<dyn SlideType + Send + Sync>>,
    /// Ordered list of all registered keywords (for `all_keywords()`).
    keywords: Vec<Arc<str>>,
}

impl SlideTypeRegistry {
    /// Create an empty registry with no registered types.
    ///
    /// Call [`register`](Self::register) to add types, or use
    /// [`SlideTypeRegistry::default`] to get the full built-in set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            keywords: Vec::new(),
        }
    }

    /// Register a slide type implementation.
    ///
    /// If a type with the same keyword is already registered, it is replaced.
    /// The keyword is taken from [`SlideType::id`].
    pub fn register(&mut self, slide_type: Box<dyn SlideType + Send + Sync>) {
        let keyword: Arc<str> = Arc::from(slide_type.id());
        // Remove existing entry for this keyword if present.
        if !self.types.contains_key(&keyword) {
            self.keywords.push(Arc::clone(&keyword));
        }
        self.types.insert(keyword, slide_type);
    }

    /// Look up a slide type by its DSL keyword.
    ///
    /// Returns `None` when no type is registered for `keyword`.
    #[must_use]
    pub fn lookup_by_keyword(&self, keyword: &str) -> Option<&dyn SlideType> {
        self.types
            .get(keyword)
            .map(|b| b.as_ref() as &dyn SlideType)
    }

    /// Suggest the closest registered keyword when `unknown` is not found.
    ///
    /// Uses Levenshtein distance (via `strsim`). Returns `Some(keyword)` when
    /// the closest match has distance ≤ 3; returns `None` when all registered
    /// keywords are too far away.
    ///
    /// # Examples
    ///
    /// ```text
    /// "conetnt"         → Some("content")  (distance 2)
    /// "flibbertigibbet" → None             (distance > 3)
    /// ```
    #[must_use]
    pub fn suggest(&self, unknown: &str) -> Option<&str> {
        // Normalize to lowercase before distance comparison so that "CONTENT"
        // and "Title" match the same as their lowercase equivalents (F-008).
        let unknown_lower = unknown.to_lowercase();
        let mut best: Option<(&str, usize)> = None;
        for kw in &self.keywords {
            let dist = strsim::levenshtein(&unknown_lower, kw.as_ref());
            if dist <= 3 {
                let is_better = match best {
                    None => true,
                    Some((best_kw, best_dist)) => {
                        dist < best_dist || (dist == best_dist && kw.as_ref() < best_kw)
                    },
                };
                if is_better {
                    best = Some((kw.as_ref(), dist));
                }
            }
        }
        best.map(|(kw, _)| kw)
    }

    /// Return the complete list of registered type keywords.
    ///
    /// Keywords are returned in registration order (insertion order).
    #[must_use]
    pub fn all_keywords(&self) -> &[Arc<str>] {
        &self.keywords
    }
}

impl Default for SlideTypeRegistry {
    /// Create a registry pre-populated with all 34 built-in slide types.
    ///
    /// Registration order matches the canonical slide type table. All types
    /// use underscore-separated keywords (e.g., `section_break`, `stat_callout`).
    /// Types are accessible by keyword via [`Self::lookup_by_keyword`].
    ///
    /// Count: 31 original types + `status`, `progress_bar`, `weighted_composite`
    /// (color-coded types added in STORY-087). `severity_cards` is NOT registered
    /// here — it is a color-coded scan target handled by region frames; its keyword
    /// is reserved in `slideforge-syntax::keywords::SLIDE_TYPE_KEYWORDS`.
    fn default() -> Self {
        let mut r = Self::new();
        // Core presentation structure
        r.register(Box::new(TitleSlideType::new()));
        r.register(Box::new(SectionBreakSlideType::new()));
        r.register(Box::new(ContentSlideType::new()));
        r.register(Box::new(TwoColSlideType::new()));
        r.register(Box::new(ImageSlideType::new()));
        r.register(Box::new(BlankSlideType::new()));
        // Navigation and overview
        r.register(Box::new(AgendaSlideType::new()));
        r.register(Box::new(TocSlideType::new()));
        // People and quotes
        r.register(Box::new(QuoteSlideType::new()));
        r.register(Box::new(TeamSlideType::new()));
        r.register(Box::new(BioSlideType::new()));
        // Analysis and strategy
        r.register(Box::new(ExecutiveSummarySlideType::new()));
        r.register(Box::new(ProblemStatementSlideType::new()));
        r.register(Box::new(RecommendationSlideType::new()));
        r.register(Box::new(RiskRegisterSlideType::new()));
        r.register(Box::new(TimelineSlideType::new()));
        r.register(Box::new(StatCalloutSlideType::new()));
        r.register(Box::new(ComparisonSlideType::new()));
        r.register(Box::new(ProcessFlowSlideType::new()));
        r.register(Box::new(MatrixSlideType::new()));
        // Financial and metrics
        r.register(Box::new(FinancialsSlideType::new()));
        r.register(Box::new(KpiDashboardSlideType::new()));
        // Data visualization
        r.register(Box::new(ChartSlideType::new()));
        r.register(Box::new(DiagramSlideType::new()));
        // Media and technical
        r.register(Box::new(ScreenshotSlideType::new()));
        r.register(Box::new(CodeSampleSlideType::new()));
        r.register(Box::new(VideoSlideType::new()));
        // Research and organizational
        r.register(Box::new(SurveyResultsSlideType::new()));
        r.register(Box::new(OrgChartSlideType::new()));
        r.register(Box::new(RoadmapSlideType::new()));
        // Color-coded status (STORY-087 — BC-1.17.001/002/003)
        r.register(Box::new(StatusSlideType::new()));
        r.register(Box::new(ProgressBarSlideType::new()));
        r.register(Box::new(WeightedCompositeSlideType::new()));
        // Closing
        r.register(Box::new(ClosingSlideType::new()));
        r
    }
}

/// Validate a slide's fields against a slide type's schema.
///
/// Accumulates **all** diagnostics before returning — does not stop at the
/// first error. This ensures the user receives a complete picture of every
/// issue in one compiler pass.
///
/// # Diagnostics produced
///
/// | Condition | Severity | Code |
/// |-----------|----------|------|
/// | Required field absent | `Error` | `"E-VAL-101"` |
/// | Required field is empty string | `Error` | `"E-VAL-102"` |
/// | Unknown field (not in required or optional) | `Warning` | `"W-VAL-103"` |
/// | Field value type mismatch or `OneOf` violation | `Error` | `"E-VAL-104"` |
///
/// # Arguments
///
/// * `slide` — The semantic slide to validate.
/// * `slide_type` — The registered slide type to validate against.
///
/// # Diagnostics produced
///
/// See the table above. All four error codes are fully implemented.
#[must_use]
pub fn validate_fields(slide: &Slide, slide_type: &dyn SlideType) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let known = known_field_names(slide_type);

    // Check all required fields: missing → E-VAL-101, empty string → E-VAL-102.
    // Build the required field list once for use in diagnostic hints (F-010).
    let required_list: Vec<&str> = slide_type
        .required_fields()
        .iter()
        .map(|f| f.name.as_ref())
        .collect();
    let required_list_str = required_list.join(", ");
    let type_id = slide_type.id();

    for field_def in slide_type.required_fields() {
        match slide.fields.get(field_def.name.as_ref()) {
            None => {
                let field_name = &field_def.name;
                diags.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: Arc::from("E-VAL-101"),
                    message: Arc::from(format!(
                        "Required field '{field_name}' missing on {type_id} slide. \
                         Required fields for '{type_id}': [{required_list_str}]."
                    )),
                    span: slide.source_span.clone(),
                    hint: Some(Arc::from(format!(
                        "add '{field_name}: <value>' to this slide block. \
                         Required fields for '{type_id}': [{required_list_str}]."
                    ))),
                });
            },
            Some(FieldValue::Literal(Value::Str(s))) if s.is_empty() => {
                let field_name = &field_def.name;
                diags.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: Arc::from("E-VAL-102"),
                    message: Arc::from(format!(
                        "Required field '{field_name}' is empty on {type_id} slide. \
                         Required fields for '{type_id}': [{required_list_str}]."
                    )),
                    span: slide.source_span.clone(),
                    hint: Some(Arc::from(format!(
                        "provide a non-empty value for '{field_name}'. \
                         Required fields for '{type_id}': [{required_list_str}]."
                    ))),
                });
            },
            _ => {},
        }
    }

    // ── E-VAL-104: type-mismatch / OneOf violation ───────────────────────────
    // BC-1.18.001 postconditions 2 and 3.
    //
    // Iterates ALL fields (required + optional). Accumulates ALL E-VAL-104
    // diagnostics without halting (DI-018 / BC-1.18.001 invariant 7).
    //
    // Two message branches:
    //   T1 — value variant wrong for expected type (including non-Str on OneOf):
    //     "Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>.
    //      See the DSL reference for valid field types."
    //   T2 — value IS Str but not in the OneOf allowlist:
    //     "Field '<name>' on <type> slide has disallowed value \"<val>\":
    //      allowed values are [<list>]."
    for field_def in slide_type
        .required_fields()
        .iter()
        .chain(slide_type.optional_fields().iter())
    {
        if let Some(expected) = &field_def.expected_type {
            // Skip FieldType::Any — no type constraint.
            if matches!(expected, FieldType::Any) {
                continue;
            }
            if let Some(FieldValue::Literal(v)) = slide.fields.get(field_def.name.as_ref())
                && !type_matches(v, expected)
            {
                let field_name = &field_def.name;
                let message: Arc<str> = if let FieldType::OneOf(allowed) = expected {
                    if let Value::Str(s) = v {
                        // T2: Str value not in the allowlist.
                        let allowed_list: Vec<&str> =
                            allowed.iter().map(std::convert::AsRef::as_ref).collect();
                        let allowed_str = allowed_list.join(", ");
                        Arc::from(format!(
                            "Field '{field_name}' on {type_id} slide has disallowed value \
                             \"{s}\": allowed values are [{allowed_str}]."
                        ))
                    } else {
                        // T1: non-Str value on a OneOf field.
                        let expected_name = expected.display_name();
                        let actual_name = value_type_name(v);
                        Arc::from(format!(
                            "Field '{field_name}' on {type_id} slide has wrong type: \
                             expected {expected_name}, got {actual_name}. \
                             See the DSL reference for valid field types."
                        ))
                    }
                } else {
                    // T1: value variant wrong for expected type.
                    let expected_name = expected.display_name();
                    let actual_name = value_type_name(v);
                    Arc::from(format!(
                        "Field '{field_name}' on {type_id} slide has wrong type: \
                         expected {expected_name}, got {actual_name}. \
                         See the DSL reference for valid field types."
                    ))
                };
                diags.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: Arc::from("E-VAL-104"),
                    message,
                    span: slide.source_span.clone(),
                    hint: None,
                });
            }
            // FieldValue::Inlines / Expr / Interpolated / absent: skip.
            // Inlines are untypeable at Stage 5 (BC-1.18.001 postcondition 5).
            // Absence is the E-VAL-101 concern.
        }
    }

    // Check for unknown fields: not in required ∪ optional → W-VAL-103.
    // Build known-field list once for the diagnostic message (F-P2-001 / AC-008).
    let mut known_list: Vec<&str> = known.iter().map(std::convert::AsRef::as_ref).collect();
    known_list.sort_unstable();
    let known_list_str = known_list.join(", ");
    for key in slide.fields.keys() {
        if !known.contains(key.as_ref()) {
            diags.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: Arc::from("W-VAL-103"),
                message: Arc::from(format!(
                    "Unknown field '{key}' for slide type '{type_id}'. \
                     Known fields: [{known_list_str}]."
                )),
                span: slide.source_span.clone(),
                hint: Some(Arc::from(format!(
                    "Valid fields for '{type_id}' are: {known_list_str}"
                ))),
            });
        }
    }

    diags
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helper — build a known-field set from required + optional
// ─────────────────────────────────────────────────────────────────────────────

/// Build the complete set of known field names (required ∪ optional) for a slide type.
///
/// Used by [`validate_fields`] to identify fields not declared by the type.
fn known_field_names(slide_type: &dyn SlideType) -> std::collections::HashSet<Arc<str>> {
    slide_type
        .required_fields()
        .iter()
        .chain(slide_type.optional_fields().iter())
        .map(|f| Arc::clone(&f.name))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — BC-1.03 and BC-1.18.001 (validate_fields, SlideTypeRegistry)
//
// Test naming: test_BC_1_03_NNN_xxx  (BC-1.03 = STORY-003 slide type registry)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
mod tests {
    use super::*;
    use slideforge_types::{FieldValue, OrderedMap, Slide, SourceSpan, Value};
    use std::sync::Arc;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_slide(slide_type: &str, fields: Vec<(&str, &str)>) -> Slide {
        let mut field_map = OrderedMap::new();
        for (k, v) in fields {
            field_map.insert(Arc::from(k), FieldValue::Literal(Value::Str(Arc::from(v))));
        }
        Slide {
            slide_type: Arc::from(slide_type),
            fields: field_map,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],

            field_spans: OrderedMap::new(),
        }
    }

    fn make_slide_with_empty_field(slide_type: &str, field_name: &str) -> Slide {
        let mut field_map = OrderedMap::new();
        field_map.insert(
            Arc::from(field_name),
            FieldValue::Literal(Value::Str(Arc::from(""))),
        );
        Slide {
            slide_type: Arc::from(slide_type),
            fields: field_map,
            blocks: vec![],
            register: None,
            tags: vec![],
            source_span: SourceSpan::default(),
            overlay: None,
            register_content: vec![],

            field_spans: OrderedMap::new(),
        }
    }

    // ── AC-001: each implemented type has non-empty required_fields (except blank) ─

    /// Exercises BC-1.03.001: `title` type has at least one required field.
    #[test]
    fn test_bc_1_03_001_title_has_required_fields() {
        let t = TitleSlideType::new();
        // This assertion drives the contract: title must have ≥1 required field.
        // FAILS until impl makes required non-empty (currently fails at todo!).
        // The type IS implemented — this test should pass. But validate_fields
        // is todo!() — AC-005 tests will fail there.
        assert!(!t.required_fields().is_empty());
    }

    /// Exercises BC-1.03.001: `content` type has at least one required field.
    #[test]
    fn test_bc_1_03_001_content_has_required_fields() {
        let t = ContentSlideType::new();
        assert!(!t.required_fields().is_empty());
    }

    /// Exercises BC-1.03.001: `blank` type has zero required fields.
    #[test]
    fn test_bc_1_03_001_blank_has_no_required_fields() {
        let t = BlankSlideType::new();
        assert!(t.required_fields().is_empty());
    }

    /// Exercises BC-1.03.001: `stat_callout` type has exactly 4 required fields.
    #[test]
    fn test_bc_1_03_001_stat_callout_has_four_required_fields() {
        let t = StatCalloutSlideType::new();
        assert_eq!(t.required_fields().len(), 4);
    }

    // ── AC-002: lookup_by_keyword ─────────────────────────────────────────────

    /// Exercises BC-1.03.002: registered types are found by keyword.
    #[test]
    fn test_bc_1_03_002_lookup_registered_type() {
        let reg = SlideTypeRegistry::default();
        assert!(reg.lookup_by_keyword("title").is_some());
        assert!(reg.lookup_by_keyword("content").is_some());
        assert!(reg.lookup_by_keyword("blank").is_some());
        assert!(reg.lookup_by_keyword("stat_callout").is_some());
    }

    /// Exercises BC-1.03.002: unknown keyword returns None.
    #[test]
    fn test_bc_1_03_002_lookup_unknown_returns_none() {
        let reg = SlideTypeRegistry::default();
        assert!(reg.lookup_by_keyword("not-a-type").is_none());
        assert!(reg.lookup_by_keyword("").is_none());
        assert!(reg.lookup_by_keyword("TITLE").is_none()); // case-sensitive
    }

    // ── AC-003 / AC-013: suggest Levenshtein ≤ 3 ────────────────────────────

    /// Exercises BC-1.03.003: "conetnt" → Some("content") (distance 2).
    ///
    /// FAILS at Red Gate: `suggest` is `todo!()`.
    #[test]
    fn test_bc_1_03_003_suggest_conetnt_returns_content() {
        let reg = SlideTypeRegistry::default();
        let suggestion = reg.suggest("conetnt");
        assert_eq!(suggestion, Some("content"));
    }

    /// Exercises BC-1.03.013: identical to AC-003 — canonical test vector.
    ///
    /// FAILS at Red Gate: `suggest` is `todo!()`.
    #[test]
    fn test_bc_1_03_013_suggest_typo_returns_closest() {
        let reg = SlideTypeRegistry::default();
        // "ttile" → "title" (distance 2)
        assert_eq!(reg.suggest("ttile"), Some("title"));
    }

    // ── AC-014: suggest far-miss returns None ────────────────────────────────

    /// Exercises BC-1.03.014: `suggest` returns None when distance > 3.
    ///
    /// FAILS at Red Gate: `suggest` is `todo!()`.
    #[test]
    fn test_bc_1_03_014_suggest_far_miss_returns_none() {
        let reg = SlideTypeRegistry::default();
        assert_eq!(reg.suggest("flibbertigibbet"), None);
        assert_eq!(reg.suggest("zzzzzzzzzzz"), None);
    }

    // ── AC-004: all_keywords ─────────────────────────────────────────────────

    /// Exercises BC-1.03.004: `all_keywords()` contains all registered types.
    #[test]
    fn test_bc_1_03_004_all_keywords_contains_registered_types() {
        let reg = SlideTypeRegistry::default();
        let keywords: Vec<&str> = reg
            .all_keywords()
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();
        assert!(keywords.contains(&"title"));
        assert!(keywords.contains(&"content"));
        assert!(keywords.contains(&"blank"));
        assert!(keywords.contains(&"stat_callout"));
    }

    // ── AC-005: validate_fields accumulates all missing field diagnostics ─────

    /// Exercises BC-1.03.005: `validate_fields` emits one Error per missing
    /// required field (accumulates all, not just first).
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_005_validate_fields_missing_required_emits_error() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // Slide has no fields → "title" required field is missing.
        let slide = make_slide("title", vec![]);
        let diagnostics = validate_fields(&slide, slide_type);
        assert!(
            !diagnostics.is_empty(),
            "expected at least one diagnostic for missing required field"
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error)
        );
    }

    // ── AC-006: validate_fields returns ALL errors, not just first ───────────

    /// Exercises BC-1.03.006: `validate_fields` on `stat_callout` with zero
    /// fields returns 4 Error diagnostics (one per required field).
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_006_validate_fields_accumulates_all_errors() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("stat_callout").unwrap();
        // Slide has no fields → all 4 required fields are missing.
        let slide = make_slide("stat_callout", vec![]);
        let diagnostics = validate_fields(&slide, slide_type);
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count();
        assert_eq!(
            error_count, 4,
            "expected 4 error diagnostics (one per missing required field), got {error_count}"
        );
    }

    // ── AC-007: empty string for required field triggers diagnostic ───────────

    /// Exercises BC-1.03.007: a required field present but set to empty string
    /// triggers an Error diagnostic.
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_007_validate_fields_empty_required_field_is_error() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // "title" field is present but empty.
        let slide = make_slide_with_empty_field("title", "title");
        let diagnostics = validate_fields(&slide, slide_type);
        assert!(
            !diagnostics.is_empty(),
            "expected a diagnostic for empty required field"
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error),
            "expected an Error-severity diagnostic for empty required field"
        );
    }

    // ── AC-008: unknown field produces Warning diagnostic ─────────────────────

    /// Exercises BC-1.03.008: a field not in required or optional produces a
    /// Warning diagnostic with the AC-008 message format.
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_008_validate_fields_unknown_field_is_warning() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // Valid required field + one unknown field.
        let slide = make_slide("title", vec![("title", "Hello"), ("zzz_unknown", "value")]);
        let diagnostics = validate_fields(&slide, slide_type);
        let warning = diagnostics
            .iter()
            .find(|d| d.severity == DiagnosticSeverity::Warning)
            .expect("expected a Warning-severity diagnostic for unknown field 'zzz_unknown'");
        // AC-008: message format must include "Known fields:"
        assert!(
            warning.message.contains("Known fields:"),
            "unknown-field message must list known fields; got: {}",
            warning.message
        );
        // AC-008: message must name the unknown field
        assert!(
            warning.message.contains("zzz_unknown"),
            "unknown-field message must name the unknown field; got: {}",
            warning.message
        );
    }

    // ── AC-009: content type has title as required ────────────────────────────

    /// Exercises BC-1.03.009: `content` type's first required field is `title`.
    #[test]
    fn test_bc_1_03_009_content_type_required_field_is_title() {
        let t = ContentSlideType::new();
        let req: Vec<&str> = t
            .required_fields()
            .iter()
            .map(|f| f.name.as_ref())
            .collect();
        assert!(
            req.contains(&"title"),
            "content type must have 'title' as a required field"
        );
    }

    // ── AC-010: stat_callout has 4 required fields ───────────────────────────

    /// Exercises BC-1.03.010: `stat_callout` has exactly 4 required fields
    /// with the canonical names `stat_1`, `label_1`, `stat_2`, `label_2`.
    #[test]
    fn test_bc_1_03_010_stat_callout_required_field_names() {
        let t = StatCalloutSlideType::new();
        let req: Vec<&str> = t
            .required_fields()
            .iter()
            .map(|f| f.name.as_ref())
            .collect();
        assert_eq!(req.len(), 4);
        assert!(req.contains(&"stat_1"));
        assert!(req.contains(&"label_1"));
        assert!(req.contains(&"stat_2"));
        assert!(req.contains(&"label_2"));
    }

    // ── AC-011: title type has title as only required field ───────────────────

    /// Exercises BC-1.03.011: `title` type has exactly one required field: `title`.
    #[test]
    fn test_bc_1_03_011_title_type_has_only_title_required() {
        let t = TitleSlideType::new();
        let req: Vec<&str> = t
            .required_fields()
            .iter()
            .map(|f| f.name.as_ref())
            .collect();
        assert_eq!(
            req.len(),
            1,
            "title type must have exactly 1 required field"
        );
        assert_eq!(req[0], "title");
    }

    // ── AC-012: lay_out returns Ok (not todo!()) ──────────────────────────────

    /// Exercises BC-1.03.012: `lay_out` returns `Ok(LaidOutSlide)` and does
    /// not panic with `todo!()`.
    #[test]
    fn test_bc_1_03_012_lay_out_returns_ok_for_title() {
        use crate::traits::Canvas;

        let t = TitleSlideType::new();
        let slide = make_slide("title", vec![("title", "Hello")]);
        let brand = make_stub_brand();
        let canvas = Canvas::default();
        let result = t.lay_out(&slide, &brand, canvas);
        assert!(result.is_ok(), "lay_out must return Ok, got {result:?}");
    }

    /// Exercises BC-1.03.012: `lay_out` returns `Ok` for `blank` (no required fields).
    #[test]
    fn test_bc_1_03_012_lay_out_returns_ok_for_blank() {
        use crate::traits::Canvas;

        let t = BlankSlideType::new();
        let slide = make_slide("blank", vec![]);
        let brand = make_stub_brand();
        let canvas = Canvas::default();
        let result = t.lay_out(&slide, &brand, canvas);
        assert!(result.is_ok(), "blank lay_out must return Ok");
    }

    /// Exercises BC-1.03.012: `lay_out` returns `Ok` for `content`.
    #[test]
    fn test_bc_1_03_012_lay_out_returns_ok_for_content() {
        use crate::traits::Canvas;

        let t = ContentSlideType::new();
        let slide = make_slide("content", vec![("title", "My Slide")]);
        let brand = make_stub_brand();
        let canvas = Canvas::default();
        let result = t.lay_out(&slide, &brand, canvas);
        assert!(result.is_ok(), "content lay_out must return Ok");
    }

    /// Exercises BC-1.03.012: `lay_out` returns `Ok` for `stat_callout`.
    #[test]
    fn test_bc_1_03_012_lay_out_returns_ok_for_stat_callout() {
        use crate::traits::Canvas;

        let t = StatCalloutSlideType::new();
        let slide = make_slide(
            "stat_callout",
            vec![
                ("stat_1", "93%"),
                ("label_1", "Customer satisfaction"),
                ("stat_2", "$4.2B"),
                ("label_2", "Annual revenue"),
            ],
        );
        let brand = make_stub_brand();
        let canvas = Canvas::default();
        let result = t.lay_out(&slide, &brand, canvas);
        assert!(result.is_ok(), "stat_callout lay_out must return Ok");
    }

    // ── AC-017: all_keywords().len() == N ────────────────────────────────────

    /// Exercises BC-1.03.017: `all_keywords()` length matches registration count.
    ///
    /// STORY-087 added `status`, `progress_bar`, `weighted_composite` — count is now 34.
    #[test]
    fn test_bc_1_03_017_all_keywords_len_equals_34() {
        let reg = SlideTypeRegistry::default();
        assert_eq!(
            reg.all_keywords().len(),
            34,
            "SlideTypeRegistry::default must register all 34 built-in slide types \
             (31 original + status + progress_bar + weighted_composite); \
             currently registers {}",
            reg.all_keywords().len()
        );
    }

    // ── F-008: suggest() case-insensitive matching ────────────────────────────

    /// Exercises F-008: `suggest()` matches case-insensitively.
    #[test]
    fn test_suggest_case_insensitive() {
        let reg = SlideTypeRegistry::default();
        assert_eq!(reg.suggest("CONTENT"), Some("content"));
        assert_eq!(reg.suggest("Title"), Some("title"));
        assert_eq!(reg.suggest("BLANK"), Some("blank"));
    }

    // ── F-011: parameterized lay_out test for all 34 types ────────────────────

    /// Exercises BC-1.03.012 for all 34 registered types: `lay_out` returns Ok for every
    /// registered slide type (not just the 4 representative ones).
    ///
    /// STORY-087: The 3 color-coded types (`status`, `progress_bar`,
    /// `weighted_composite`) perform field validation in `lay_out()`. For these
    /// types, a slide with the minimum required fields is used so `lay_out()`
    /// returns `Ok`. The other 31 types are called with empty fields (their
    /// `lay_out()` stubs return `Ok` regardless of fields).
    #[test]
    fn test_all_34_types_lay_out_returns_ok() {
        use crate::traits::Canvas;
        use slideforge_types::{Brand, BrandFonts, BrandPalette};

        let brand = Brand {
            name: Arc::from("stub"),
            palette: BrandPalette {
                primary: Arc::from("#000000"),
                secondary: Arc::from("#ffffff"),
                accent: Arc::from("#0000ff"),
                neutral: Arc::from("#f5f5f5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };

        let reg = SlideTypeRegistry::default();
        for kw in reg.all_keywords() {
            let slide_type = reg.lookup_by_keyword(kw.as_ref()).unwrap();

            // The 3 STORY-087 color-coded types validate fields in lay_out().
            // Provide minimum required fields so they return Ok.
            let slide = match kw.as_ref() {
                "status" => {
                    let mut fields = OrderedMap::new();
                    fields.insert(
                        Arc::from("title"),
                        FieldValue::Literal(Value::Str(Arc::from("Test"))),
                    );
                    fields.insert(
                        Arc::from("label"),
                        FieldValue::Literal(Value::Str(Arc::from("On Track"))),
                    );
                    Slide {
                        slide_type: Arc::clone(kw),
                        fields,
                        blocks: vec![],
                        register: None,
                        tags: vec![],
                        source_span: SourceSpan::default(),
                        overlay: None,
                        register_content: vec![],

                        field_spans: OrderedMap::new(),
                    }
                },
                "progress_bar" => {
                    let mut fields = OrderedMap::new();
                    fields.insert(
                        Arc::from("title"),
                        FieldValue::Literal(Value::Str(Arc::from("Progress"))),
                    );
                    fields.insert(
                        Arc::from("label"),
                        FieldValue::Literal(Value::Str(Arc::from("50% done"))),
                    );
                    fields.insert(Arc::from("value"), FieldValue::Literal(Value::Int(50)));
                    Slide {
                        slide_type: Arc::clone(kw),
                        fields,
                        blocks: vec![],
                        register: None,
                        tags: vec![],
                        source_span: SourceSpan::default(),
                        overlay: None,
                        register_content: vec![],

                        field_spans: OrderedMap::new(),
                    }
                },
                "weighted_composite" => {
                    let mut comp = OrderedMap::new();
                    comp.insert(Arc::from("name"), Value::Str(Arc::from("Quality")));
                    comp.insert(
                        Arc::from("weight"),
                        Value::Float(ordered_float::OrderedFloat(0.5)),
                    );
                    comp.insert(Arc::from("score"), Value::Int(80));
                    comp.insert(Arc::from("label"), Value::Str(Arc::from("Good")));
                    let mut fields = OrderedMap::new();
                    fields.insert(
                        Arc::from("title"),
                        FieldValue::Literal(Value::Str(Arc::from("Scorecard"))),
                    );
                    fields.insert(
                        Arc::from("label"),
                        FieldValue::Literal(Value::Str(Arc::from("Overall: Good"))),
                    );
                    fields.insert(
                        Arc::from("components"),
                        FieldValue::Literal(Value::List(vec![Value::Map(comp)])),
                    );
                    Slide {
                        slide_type: Arc::clone(kw),
                        fields,
                        blocks: vec![],
                        register: None,
                        tags: vec![],
                        source_span: SourceSpan::default(),
                        overlay: None,
                        register_content: vec![],

                        field_spans: OrderedMap::new(),
                    }
                },
                _ => make_slide(kw.as_ref(), vec![]),
            };

            let result = slide_type.lay_out(&slide, &brand, Canvas::default());
            assert!(
                result.is_ok(),
                "lay_out() for '{kw}' must return Ok, got: {result:?}"
            );
        }
    }

    // ── EC-005: stat_callout with partial fields ──────────────────────────────

    /// Exercises EC-005: `validate_fields` on `stat_callout` with only 2 of 4
    /// required fields provided emits exactly 2 Error diagnostics — one for
    /// each missing required field (`stat_2`, `label_2`).
    #[test]
    fn test_ec_005_stat_callout_partial_fields() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("stat_callout").unwrap();
        // Provide only stat_1 and label_1; stat_2 and label_2 are missing.
        let slide = make_slide("stat_callout", vec![("stat_1", "93%"), ("label_1", "CSAT")]);
        let diags = validate_fields(&slide, slide_type);
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert_eq!(
            errors.len(),
            2,
            "exactly 2 missing required fields (stat_2, label_2); got {} errors",
            errors.len()
        );
        // Each error identifies exactly one missing field.
        // The messages should collectively name stat_2 and label_2 as the
        // missing fields, not stat_1 or label_1 (which are present).
        let missing_names: Vec<&str> = errors
            .iter()
            .filter_map(|e| {
                // Extract the field named in "Required field 'X' missing"
                e.message
                    .strip_prefix("Required field '")
                    .and_then(|s| s.split('\'').next())
            })
            .collect();
        assert!(
            missing_names.contains(&"stat_2"),
            "error must identify 'stat_2' as missing; got missing_names={missing_names:?}"
        );
        assert!(
            missing_names.contains(&"label_2"),
            "error must identify 'label_2' as missing; got missing_names={missing_names:?}"
        );
        assert!(
            !missing_names.contains(&"stat_1"),
            "present field 'stat_1' must not be identified as missing; got missing_names={missing_names:?}"
        );
        assert!(
            !missing_names.contains(&"label_1"),
            "present field 'label_1' must not be identified as missing; got missing_names={missing_names:?}"
        );
    }

    // ── Validate fields: clean slide produces no diagnostics ──────────────────

    /// Exercises BC-1.03.005 (positive path): a fully valid `title` slide with
    /// required fields present produces zero Error diagnostics.
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_005_validate_fields_valid_slide_no_errors() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        let slide = make_slide("title", vec![("title", "Hello World")]);
        let diagnostics = validate_fields(&slide, slide_type);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "valid slide should produce no Error diagnostics, got: {errors:?}"
        );
    }

    /// Exercises BC-1.03.005 (positive path): a blank slide with no required
    /// fields and no provided fields produces zero diagnostics.
    ///
    /// FAILS at Red Gate: `validate_fields` is `todo!()`.
    #[test]
    fn test_bc_1_03_005_validate_fields_blank_slide_no_diagnostics() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("blank").unwrap();
        let slide = make_slide("blank", vec![]);
        let diagnostics = validate_fields(&slide, slide_type);
        assert!(
            diagnostics.is_empty(),
            "blank slide with no fields should produce no diagnostics, got: {diagnostics:?}"
        );
    }

    // ── Helper: stub Brand ────────────────────────────────────────────────────

    fn make_stub_brand() -> slideforge_types::Brand {
        use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};
        use std::sync::Arc;
        Brand {
            name: Arc::from("stub"),
            palette: BrandPalette {
                primary: Arc::from("#000000"),
                secondary: Arc::from("#ffffff"),
                accent: Arc::from("#0000ff"),
                neutral: Arc::from("#f5f5f5"),
            },
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
                mono: Arc::from("Courier New"),
                font_size_emu: 457_200,
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    // ── STORY-098: W-VAL-103 content-drop severity promotion (BC-3.03.002 v1.2) ─
    //
    // FU-DIAGNOSTIC-FIELD-PINNING lesson: assert message TEXT and distinguishing
    // struct fields (severity, code, span presence), not just error-code presence.
    //
    // These tests exercise Route A (context-sensitive severity at accumulation time
    // in validate_fields): field key ∈ {"shape", "body"} AND strict mode → Error
    // severity (broken/exit-2). Current code always emits Warning — tests FAIL at
    // Red Gate. After implementation these tests must pass.
    //
    // Test naming: test_BC_3_03_002_xxx (BC-3.03.002 = STORY-098 content-drop BC)

    /// BC-3.03.002 v1.2 Invariant 4 / AC-001 (T-003 RED):
    ///
    /// `shape:` on a `title` slide (which does not support `shape:`) in strict mode
    /// must produce a W-VAL-103 diagnostic with `DiagnosticSeverity::Error` (broken),
    /// not `DiagnosticSeverity::Warning` (cosmetic).
    ///
    /// RED: Currently `validate_fields` emits `Warning` for ALL unknown fields including
    /// `shape`. This test fails until the implementer adds the content-drop key set check
    /// `{"shape", "body"}` in `validate_fields` (registry.rs).
    ///
    /// Message format is UNCHANGED (per postcondition 3 / Route A):
    /// `"Unknown field 'shape' for slide type 'title'. Known fields: [...]."
    #[test]
    fn test_BC_3_03_002_shape_content_drop_strict_exits_2() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // title slide has required `title` field + known optional fields.
        // `shape:` is NOT in the schema — it would cause authored content to be silently dropped.
        let slide = make_slide("title", vec![("title", "My Title"), ("shape", "some shape spec")]);
        let diags = validate_fields(&slide, slide_type);

        // There must be a W-VAL-103 diagnostic for the unknown `shape` field.
        let w_val_103: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "W-VAL-103")
            .collect();
        assert!(
            !w_val_103.is_empty(),
            "STORY-098 AC-001: `shape` on `title` slide must emit W-VAL-103; got diags: {diags:?}"
        );

        // RED GATE ASSERTION: content-drop W-VAL-103 must be Error severity in strict mode.
        // Current code emits Warning — this assertion FAILS before implementation.
        let shape_diag = w_val_103
            .iter()
            .find(|d| d.message.contains("shape"))
            .expect("W-VAL-103 must name the dropped field 'shape' in its message");
        assert_eq!(
            shape_diag.severity,
            DiagnosticSeverity::Error,
            "BC-3.03.002 v1.2 Invariant 4 / AC-001: W-VAL-103 for 'shape' on unsupported slide \
             type MUST be Error severity (broken/exit-2 in strict mode); \
             currently emits Warning — RED GATE: this assertion fails before implementation. \
             Got severity: {:?}. Message: {}",
            shape_diag.severity,
            shape_diag.message
        );

        // FU-DIAGNOSTIC-FIELD-PINNING: assert message format is UNCHANGED (Route A).
        // The message must still use the canonical W-VAL-103 format.
        assert!(
            shape_diag.message.contains("Unknown field 'shape'"),
            "BC-3.03.002 postcondition 3: W-VAL-103 message format is UNCHANGED; \
             must contain \"Unknown field 'shape'\"; got: {}",
            shape_diag.message
        );
        assert!(
            shape_diag.message.contains("title"),
            "W-VAL-103 message must name the slide type 'title'; got: {}",
            shape_diag.message
        );
        assert!(
            shape_diag.message.contains("Known fields:"),
            "W-VAL-103 message must list known fields (canonical format); got: {}",
            shape_diag.message
        );
        // Code must remain W-VAL-103 (no new E-VAL-105 introduced — Route A).
        assert_eq!(
            shape_diag.code.as_ref(),
            "W-VAL-103",
            "BC-3.03.002 Route A: no new error code — code must remain W-VAL-103; \
             got: {}",
            shape_diag.code
        );
    }

    /// BC-3.03.002 v1.2 Invariant 4 / EC-007 / AC-002 (T-004 RED):
    ///
    /// `body` on a `content` slide (schema-invalid per content.rs — `body` is NOT
    /// in content's optional or required fields) in strict mode must produce
    /// W-VAL-103 with `DiagnosticSeverity::Error` (broken/exit-2).
    ///
    /// RED: Currently `validate_fields` emits `Warning` for `body` on `content`. This
    /// test fails until the implementer adds the content-drop key set check.
    ///
    /// This also closes the body/content schema drift (AC-002/AC-003): after this story,
    /// `body` on `content` is consistently rejected by BOTH `validate_fields` AND the
    /// threading pass (field_to_block.rs:135 fix).
    #[test]
    fn test_BC_3_03_002_body_on_content_strict_exits_2() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("content").unwrap();
        // `content` slide: required field is `title`; optional include `bullets`, `takeaway`,
        // and common optional fields. `body` is NOT in the schema.
        // BC-3.03.002 EC-007: body on content type is schema-invalid.
        let slide = make_slide(
            "content",
            vec![("title", "My Content Slide"), ("body", "some body text")],
        );
        let diags = validate_fields(&slide, slide_type);

        // There must be a W-VAL-103 diagnostic for the unknown `body` field.
        let w_val_103: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "W-VAL-103" && d.message.contains("body"))
            .collect();
        assert!(
            !w_val_103.is_empty(),
            "STORY-098 AC-002: `body` on `content` slide must emit W-VAL-103 naming 'body'; \
             got diags: {diags:?}"
        );

        let body_diag = &w_val_103[0];

        // RED GATE ASSERTION: content-drop W-VAL-103 for `body` must be Error in strict mode.
        // Currently emits Warning — FAILS before implementation.
        assert_eq!(
            body_diag.severity,
            DiagnosticSeverity::Error,
            "BC-3.03.002 v1.2 Invariant 4 / EC-007 / AC-002: W-VAL-103 for 'body' on 'content' \
             slide MUST be Error severity (broken/exit-2 in strict mode); \
             currently emits Warning — RED GATE: this assertion fails before implementation. \
             Got severity: {:?}. Message: {}",
            body_diag.severity,
            body_diag.message
        );

        // FU-DIAGNOSTIC-FIELD-PINNING: message format unchanged (Route A).
        assert!(
            body_diag.message.contains("Unknown field 'body'"),
            "W-VAL-103 message must contain \"Unknown field 'body'\"; got: {}",
            body_diag.message
        );
        assert!(
            body_diag.message.contains("content"),
            "W-VAL-103 message must name the slide type 'content'; got: {}",
            body_diag.message
        );
        assert_eq!(
            body_diag.code.as_ref(),
            "W-VAL-103",
            "Route A: code must remain W-VAL-103 (no E-VAL-105); got: {}",
            body_diag.code
        );
    }

    /// BC-3.03.002 Invariant 4 — EC-001 (non-content-drop unknown field stays cosmetic, exit 0).
    ///
    /// Regression guard: an unknown field that is NOT in the content-drop set
    /// `{"shape", "body"}` must retain `Warning` severity (cosmetic/exit-0).
    /// This ensures the implementer's change is strictly scoped to the content-drop key set.
    ///
    /// This test MAY PASS at Red Gate (current behavior: Warning for all unknown fields).
    /// It is a regression guard that must CONTINUE PASSING after implementation.
    #[test]
    fn test_BC_3_03_002_non_content_drop_field_stays_cosmetic() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // `zzz_metadata_annotation` is unknown but not a content-bearing field.
        // Must remain Warning (cosmetic/exit-0) — not promoted to Error.
        let slide = make_slide(
            "title",
            vec![
                ("title", "My Title"),
                ("zzz_metadata_annotation", "some-value"),
            ],
        );
        let diags = validate_fields(&slide, slide_type);
        let zzz_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "W-VAL-103" && d.message.contains("zzz_metadata"))
            .collect();
        assert!(
            !zzz_diags.is_empty(),
            "EC-001 guard: unknown non-content-drop field must still produce W-VAL-103; \
             got diags: {diags:?}"
        );
        // Must stay Warning — NOT promoted to Error.
        assert_eq!(
            zzz_diags[0].severity,
            DiagnosticSeverity::Warning,
            "EC-001 guard: non-content-drop unknown field must remain Warning (cosmetic/exit-0); \
             got: {:?}. This would regress EC-001 behavior if it changes to Error.",
            zzz_diags[0].severity
        );
    }

    /// BC-3.03.002 Invariant 4 — EC-002 (shape on supporting slide type → no warning).
    ///
    /// Regression guard: a `shape:` field on a slide type that explicitly supports shapes
    /// must NOT produce W-VAL-103. This confirms the key-set check is conditional on
    /// the field being UNKNOWN (not in the schema), not unconditional.
    ///
    /// Uses `blank` slide type — it accepts common optional fields. `shape` is not
    /// in `common_optional_fields` either, but this test documents the invariant that
    /// a slide type DECLARING `shape` as optional/required must not trigger W-VAL-103.
    ///
    /// NOTE: No built-in slide type currently declares `shape` as a known optional field.
    /// The implementer must create or register a slide type that does to satisfy this
    /// fully. For now this test asserts the semantic invariant: if a field IS in the
    /// known set, no W-VAL-103 is emitted.
    ///
    /// This test exercises the negative path — using a known field `title` on `title` type.
    #[test]
    fn test_BC_3_03_002_known_field_on_supporting_type_no_warning() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // `title` IS a known field for `title` slide type. Must produce zero W-VAL-103.
        let slide = make_slide("title", vec![("title", "Hello World")]);
        let diags = validate_fields(&slide, slide_type);
        let w_val_103: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "W-VAL-103")
            .collect();
        assert!(
            w_val_103.is_empty(),
            "EC-002 guard: known field 'title' on 'title' slide type must NOT produce W-VAL-103; \
             got: {w_val_103:?}"
        );
    }

    /// BC-3.03.002 Invariant 1 (DI-017) + DI-018 / EC-004 (multiple errors incl. content-drop):
    ///
    /// When a deck has multiple W-VAL-103 content-drop errors AND other validation errors,
    /// ALL must be reported (error accumulation, DI-018) and the content-drop W-VAL-103
    /// must be Error severity.
    ///
    /// RED: Currently `shape` W-VAL-103 is Warning. The Error assertion on `shape` fails.
    #[test]
    fn test_BC_3_03_002_multiple_errors_all_reported_content_drop_is_error() {
        let reg = SlideTypeRegistry::default();
        let slide_type = reg.lookup_by_keyword("title").unwrap();
        // Slide has: required `title` field (present), plus TWO unknown fields:
        //   - `shape` (content-drop set → must be Error in strict mode)
        //   - `zzz_unknown_meta` (non-content-drop → must remain Warning)
        let slide = make_slide(
            "title",
            vec![
                ("title", "Deck Title"),
                ("shape", "some_shape_spec"),
                ("zzz_unknown_meta", "ignored-value"),
            ],
        );
        let diags = validate_fields(&slide, slide_type);

        // DI-018: both W-VAL-103 diagnostics must be emitted (accumulation, not bail-on-first).
        let w_val_103: Vec<_> = diags
            .iter()
            .filter(|d| d.code.as_ref() == "W-VAL-103")
            .collect();
        assert!(
            w_val_103.len() >= 2,
            "EC-004 / DI-018: both unknown fields must produce W-VAL-103 diagnostics (no bail-on-first); \
             got {} W-VAL-103 diags — all diags: {diags:?}",
            w_val_103.len()
        );

        // `shape` must be Error (content-drop set).
        let shape_diag = w_val_103
            .iter()
            .find(|d| d.message.contains("'shape'"))
            .expect("must have a W-VAL-103 for 'shape'");
        assert_eq!(
            shape_diag.severity,
            DiagnosticSeverity::Error,
            "EC-004: W-VAL-103 for 'shape' must be Error (content-drop, broken); \
             got: {:?}. RED GATE: fails before implementation.",
            shape_diag.severity
        );

        // `zzz_unknown_meta` must remain Warning (non-content-drop).
        let meta_diag = w_val_103
            .iter()
            .find(|d| d.message.contains("zzz_unknown_meta"))
            .expect("must have a W-VAL-103 for 'zzz_unknown_meta'");
        assert_eq!(
            meta_diag.severity,
            DiagnosticSeverity::Warning,
            "EC-004: non-content-drop W-VAL-103 must remain Warning; got: {:?}",
            meta_diag.severity
        );
    }
}
