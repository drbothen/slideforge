//! [`SlideTypeRegistry`] — the runtime registry of all 31 built-in slide types.
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
//! [`SlideTypeRegistry::default`] pre-registers all 31 built-in slide types.
//!
//! ## Thread safety
//!
//! The registry itself is not `Sync` (it holds `Box<dyn SlideType + Send + Sync>`
//! behind a mutable interface). Wrap in `Arc<RwLock<SlideTypeRegistry>>` when
//! sharing across threads, or use the `SLIDE_TYPE_REGISTRY` lazy-lock singleton.

use std::collections::HashMap;
use std::sync::Arc;

use slideforge_types::Slide;

use crate::traits::{Diagnostic, DiagnosticSeverity, SlideType};
use slideforge_types::{FieldValue, Value};

use super::{
    agenda::AgendaSlideType, bio::BioSlideType, blank::BlankSlideType, chart::ChartSlideType,
    closing::ClosingSlideType, code_sample::CodeSampleSlideType, comparison::ComparisonSlideType,
    content::ContentSlideType, diagram::DiagramSlideType,
    executive_summary::ExecutiveSummarySlideType, financials::FinancialsSlideType,
    image::ImageSlideType, kpi_dashboard::KpiDashboardSlideType, matrix::MatrixSlideType,
    org_chart::OrgChartSlideType, problem_statement::ProblemStatementSlideType,
    process_flow::ProcessFlowSlideType, quote::QuoteSlideType,
    recommendation::RecommendationSlideType, risk_register::RiskRegisterSlideType,
    roadmap::RoadmapSlideType, screenshot::ScreenshotSlideType,
    section_break::SectionBreakSlideType, stat_callout::StatCalloutSlideType,
    survey_results::SurveyResultsSlideType, team::TeamSlideType, timeline::TimelineSlideType,
    title::TitleSlideType, toc::TocSlideType, two_col::TwoColSlideType, video::VideoSlideType,
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
    /// Create a registry pre-populated with all 31 built-in slide types.
    ///
    /// Registration order matches the canonical slide type table. All types
    /// use underscore-separated keywords (e.g., `section_break`, `stat_callout`).
    /// Types are accessible by keyword via [`Self::lookup_by_keyword`].
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
///
/// # Arguments
///
/// * `slide` — The semantic slide to validate.
/// * `slide_type` — The registered slide type to validate against.
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
// Red Gate tests — ALL tests MUST FAIL before implementation begins.
//
// Test naming: test_BC_1_03_NNN_xxx  (BC-1.03 = STORY-003 slide type registry)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
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
    #[test]
    fn test_bc_1_03_017_all_keywords_len_equals_31() {
        let reg = SlideTypeRegistry::default();
        assert_eq!(
            reg.all_keywords().len(),
            31,
            "SlideTypeRegistry::default must register all 31 built-in slide types; \
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

    // ── F-011: parameterized lay_out test for all 31 types ────────────────────

    /// Exercises BC-1.03.012 for all 31 types: `lay_out` returns Ok for every
    /// registered slide type (not just the 4 representative ones).
    #[test]
    fn test_all_31_types_lay_out_returns_ok() {
        use crate::traits::Canvas;
        use slideforge_types::{Brand, BrandFonts, BrandPalette, SourceSpan};

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
            },
            layouts: vec![],
            span: SourceSpan::default(),
        };

        let reg = SlideTypeRegistry::default();
        for kw in reg.all_keywords() {
            let slide_type = reg.lookup_by_keyword(kw.as_ref()).unwrap();
            let slide = make_slide(kw.as_ref(), vec![]);
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
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }
}
