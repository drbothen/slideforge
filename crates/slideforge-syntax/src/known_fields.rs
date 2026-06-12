//! Compile-time known-field registry for all 34 slideforge slide types.
//!
//! [`known_fields`] returns the set of valid field names for a given slide type.
//! This is used by the alias parser (STORY-008) to validate that alias bodies
//! only preset fields that actually exist on the base type.
//!
//! # Design
//!
//! Rather than a `phf::Map` (which would require an additional dependency),
//! the mapping is implemented as a `match` expression over hardcoded
//! `&'static [&'static str]` slices — zero allocation, zero runtime overhead,
//! and no new crate dependency. The values are compile-time constants.
//!
//! # All 34 Slide Types
//!
//! Per the `SlideTypeRegistry` in `slideforge-plugin-api` (single source of truth):
//! - Core: `title`, `section_break`, `content`, `two_col`, `image`, `blank`
//! - Navigation: `agenda`, `toc`
//! - People/quotes: `quote`, `team`, `bio`
//! - Analysis/strategy: `executive_summary`, `problem_statement`, `recommendation`,
//!   `risk_register`, `timeline`, `stat_callout`, `comparison`, `process_flow`,
//!   `matrix`
//! - Financial/metrics: `financials`, `kpi_dashboard`
//! - Data visualization: `chart`, `diagram`
//! - Media/technical: `screenshot`, `code_sample`, `video`
//! - Research/org: `survey_results`, `org_chart`, `roadmap`
//! - Color-coded status (STORY-087): `status`, `progress_bar`, `weighted_composite`
//! - Closing: `closing`
//!
//! Common fields shared by all slide types: `tags`, `notes`, `report`, `detail`,
//! `alt`, `lang`, `decorative`, `footer`, `logo`.

// ─── Public API ───────────────────────────────────────────────────────────────

/// Return the known field names for a given slide type.
///
/// Returns `Some(&[...])` with the type's valid field names, or `None` if
/// `slide_type` is not a recognised built-in type.
///
/// The returned slice is a `'static` reference — no allocation occurs.
///
/// # Examples
///
/// ```rust
/// use slideforge_syntax::known_fields::known_fields;
///
/// assert!(known_fields("title").is_some());
/// assert!(known_fields("unknown_type").is_none());
/// assert!(known_fields("content").unwrap().contains(&"bullets"));
/// ```
#[must_use]
#[allow(clippy::too_many_lines)] // 34 slide types × ~10 fields each — exhaustive lookup table, not logic
pub fn known_fields(slide_type: &str) -> Option<&'static [&'static str]> {
    // Common fields that all slide types share.
    // These appear in every type's list below.
    //
    // - tags: slide filter tags (variant filtering)
    // - notes: presenter register
    // - report: reader register
    // - detail: document-only register
    // - alt: accessibility label for the slide
    // - lang: per-slide language override
    // - decorative: mark slide as decorative (no alt required)
    // - footer: footer override
    // - logo: logo override
    match slide_type {
        "title" => Some(&[
            // Common
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            // Type-specific
            "title",
            "subtitle",
            "author",
            "date",
        ]),
        "section_break" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "subtitle",
        ]),
        "content" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            // body: prose paragraph text in the content area (BC-4.01.001 v1.2 PC-11;
            // STORY-098 F-098-P1-002: declared so body threading is not gated out for content).
            "body",
            "bullets",
            "takeaway",
        ]),
        "two_col" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "left",
            "right",
        ]),
        // Both `image` and `screenshot` use: title, src (media-source path), caption, plus common fields.
        // Canonical media-source keyword is `src` per BC-1.16.001 PC-10/EC-006;
        // field_to_block.rs reads "src" to construct ContentBlock::Image.
        // Human decision 2026-06-07: `image` is NOT the source-field keyword; `src` is.
        "image" | "screenshot" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "src",
            "caption",
        ]),
        "blank" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
        ]),
        "agenda" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "items",
        ]),
        "toc" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
        ]),
        "quote" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "quote",
            "attribution",
        ]),
        "team" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "members",
        ]),
        // `bio` uses: name, title, bio, src (optional photo), plus common fields.
        // Canonical media-source keyword is `src` per BC-1.16.001 PC-10/EC-006;
        // field_to_block.rs reads "src" to construct ContentBlock::Image for the photo.
        // Human decision 2026-06-07: `image` is NOT the source-field keyword; `src` is.
        "bio" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "name",
            "title",
            "bio",
            "src",
        ]),
        "executive_summary" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "summary",
            "bullets",
        ]),
        "problem_statement" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "problem",
            "impact",
        ]),
        "recommendation" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "recommendation",
            "rationale",
            "risk",
        ]),
        "risk_register" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "risks",
        ]),
        "timeline" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "milestones",
        ]),
        "stat_callout" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "stat_1",
            "label_1",
            "stat_2",
            "label_2",
            "stat_3",
            "label_3",
        ]),
        "comparison" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "option_a",
            "option_b",
            "criteria",
        ]),
        "process_flow" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "steps",
        ]),
        "matrix" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "cells",
        ]),
        "financials" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "rows",
        ]),
        "kpi_dashboard" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "kpis",
        ]),
        "chart" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "chart_type",
            "data",
        ]),
        "diagram" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "diagram",
        ]),
        "code_sample" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "code",
            "language",
        ]),
        "video" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "video_url",
        ]),
        "survey_results" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "results",
        ]),
        "org_chart" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "nodes",
        ]),
        "roadmap" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "phases",
        ]),
        // Color-coded status slide types (STORY-087 — BC-1.17.001/002/003).
        // None of these three declare `body` — body is NOT in their field schemas.
        // A `body` field on any of these types is an unknown field and will trigger
        // W-VAL-103 (promoted to Error severity in strict mode because body is a
        // CONTENT_DROP_KEY per BC-3.03.002 v1.2 Invariant 4).
        "status" => Some(&[
            // Common
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            // Type-specific (BC-1.17.001)
            // Required: title, label (WCAG 1.4.1 co-encoding of color state)
            "title",
            "label",
        ]),
        "progress_bar" => Some(&[
            // Common
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            // Type-specific (BC-1.17.002)
            // Required: title, label, value (integer 0–100)
            "title",
            "label",
            "value",
        ]),
        "weighted_composite" => Some(&[
            // Common
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            // Type-specific (BC-1.17.003)
            // Required: title, label (aggregate), components (list of component maps)
            "title",
            "label",
            "components",
        ]),
        "closing" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "title",
            "call_to_action",
            "contact",
        ]),
        _ => None,
    }
}

/// Return the list of all 34 built-in slide type names.
///
/// These match exactly the keywords registered in `SlideTypeRegistry::default()`
/// in `slideforge-plugin-api`. This function is the compile-time counterpart of
/// the runtime registry's `all_keywords()` method.
///
/// This is used for error messages (e.g., "Did you mean X?") and for
/// iterating over all known types.
///
/// Count: 31 original types (STORY-003) + `status`, `progress_bar`,
/// `weighted_composite` (color-coded types added in STORY-087).
#[must_use]
pub fn all_slide_types() -> &'static [&'static str] {
    &[
        // Core presentation structure
        "title",
        "section_break",
        "content",
        "two_col",
        "image",
        "blank",
        // Navigation and overview
        "agenda",
        "toc",
        // People and quotes
        "quote",
        "team",
        "bio",
        // Analysis and strategy
        "executive_summary",
        "problem_statement",
        "recommendation",
        "risk_register",
        "timeline",
        "stat_callout",
        "comparison",
        "process_flow",
        "matrix",
        // Financial and metrics
        "financials",
        "kpi_dashboard",
        // Data visualization
        "chart",
        "diagram",
        // Media and technical
        "screenshot",
        "code_sample",
        "video",
        // Research and organizational
        "survey_results",
        "org_chart",
        "roadmap",
        // Color-coded status (STORY-087 — BC-1.17.001/002/003)
        "status",
        "progress_bar",
        "weighted_composite",
        // Closing
        "closing",
    ]
}

/// Suggest the closest known slide type name to `given` (Levenshtein distance ≤ 3).
///
/// Returns `None` if no type is close enough to be a useful suggestion.
/// Uses `strsim::levenshtein` for accurate edit distance computation.
#[must_use]
pub fn suggest_type(given: &str) -> Option<&'static str> {
    all_slide_types()
        .iter()
        .filter_map(|&candidate| {
            let dist = strsim::levenshtein(given, candidate);
            if dist <= 3 {
                Some((dist, candidate))
            } else {
                None
            }
        })
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, name)| name)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc_1_09_002_known_fields_title_includes_title() {
        let fields = known_fields("title").expect("title must be a known slide type");
        assert!(
            fields.contains(&"title"),
            "title type must have a 'title' field"
        );
        assert!(
            fields.contains(&"footer"),
            "title type must have a 'footer' field (common)"
        );
    }

    #[test]
    fn test_bc_1_09_002_known_fields_content_includes_bullets() {
        let fields = known_fields("content").expect("content must be a known slide type");
        assert!(
            fields.contains(&"bullets"),
            "content type must have a 'bullets' field"
        );
        assert!(
            fields.contains(&"takeaway"),
            "content type must have a 'takeaway' field"
        );
    }

    #[test]
    fn test_bc_1_09_002_known_fields_unknown_returns_none() {
        assert!(
            known_fields("not_a_real_type").is_none(),
            "unknown type must return None"
        );
    }

    #[test]
    fn test_bc_1_08_001_all_34_types_registered() {
        // All 34 built-in slide types must have known fields.
        // 31 original types (STORY-003) + status, progress_bar, weighted_composite (STORY-087).
        for &ty in all_slide_types() {
            assert!(
                known_fields(ty).is_some(),
                "slide type '{ty}' must be in known_fields"
            );
        }
        assert_eq!(
            all_slide_types().len(),
            34,
            "must have exactly 34 built-in slide types (31 original + 3 color-coded from STORY-087)"
        );
    }

    /// F-098-P2-001: color-coded types must be in `known_fields` with their correct field sets.
    ///
    /// - `status`: title + label (NO body)
    /// - `progress_bar`: title + label + value (NO body)
    /// - `weighted_composite`: title + label + components (NO body)
    ///
    /// The absence of `body` is load-bearing: the body-threading gate in
    /// `field_to_block.rs` uses `known_fields().contains("body")` to decide
    /// whether to thread a body block. Body on these types is a W-VAL-103
    /// `CONTENT_DROP_KEY` → Error in strict mode.
    #[test]
    fn test_f098_p2_001_color_coded_types_in_known_fields() {
        for ty in &["status", "progress_bar", "weighted_composite"] {
            assert!(
                known_fields(ty).is_some(),
                "color-coded type '{ty}' must be in known_fields (F-098-P2-001)"
            );
        }
    }

    /// F-098-P2-001: none of the 3 color-coded types declare `body`.
    ///
    /// Body on these types is an unknown `CONTENT_DROP_KEY` field. The body-threading
    /// gate must NOT thread body blocks for these types in warn-only mode.
    #[test]
    fn test_f098_p2_001_color_coded_types_have_no_body() {
        for ty in &["status", "progress_bar", "weighted_composite"] {
            let fields = known_fields(ty)
                .unwrap_or_else(|| panic!("color-coded type '{ty}' must be in known_fields"));
            assert!(
                !fields.contains(&"body"),
                "color-coded type '{ty}' must NOT declare 'body' in known_fields — \
                 body is unsupported for this layout; use label instead (F-098-P2-001)"
            );
        }
    }

    /// F-098-P2-001: `status` type declares exactly title + label + common fields.
    #[test]
    fn test_f098_p2_001_status_required_fields_in_known_fields() {
        let fields = known_fields("status").expect("status must be in known_fields");
        assert!(fields.contains(&"title"), "status must have 'title'");
        assert!(fields.contains(&"label"), "status must have 'label'");
        // Verify common fields
        assert!(fields.contains(&"tags"), "status must have common 'tags'");
        assert!(fields.contains(&"notes"), "status must have common 'notes'");
        assert!(fields.contains(&"alt"), "status must have common 'alt'");
    }

    /// F-098-P2-001: `progress_bar` type declares exactly title + label + value + common fields.
    #[test]
    fn test_f098_p2_001_progress_bar_required_fields_in_known_fields() {
        let fields = known_fields("progress_bar").expect("progress_bar must be in known_fields");
        assert!(fields.contains(&"title"), "progress_bar must have 'title'");
        assert!(fields.contains(&"label"), "progress_bar must have 'label'");
        assert!(fields.contains(&"value"), "progress_bar must have 'value'");
        assert!(
            !fields.contains(&"body"),
            "progress_bar must NOT have 'body'"
        );
    }

    /// F-098-P2-001: `weighted_composite` type declares exactly title + label + components + common fields.
    #[test]
    fn test_f098_p2_001_weighted_composite_required_fields_in_known_fields() {
        let fields =
            known_fields("weighted_composite").expect("weighted_composite must be in known_fields");
        assert!(
            fields.contains(&"title"),
            "weighted_composite must have 'title'"
        );
        assert!(
            fields.contains(&"label"),
            "weighted_composite must have 'label'"
        );
        assert!(
            fields.contains(&"components"),
            "weighted_composite must have 'components'"
        );
        assert!(
            !fields.contains(&"body"),
            "weighted_composite must NOT have 'body'"
        );
    }

    #[test]
    fn test_bc_1_09_002_all_types_have_common_fields() {
        // Every slide type must have the universal common fields.
        let common = ["tags", "notes", "report", "detail", "alt", "lang", "footer"];
        for &ty in all_slide_types() {
            let fields = known_fields(ty).unwrap_or_else(|| panic!("type '{ty}' not in registry"));
            for &common_field in &common {
                assert!(
                    fields.contains(&common_field),
                    "type '{ty}' is missing common field '{common_field}'"
                );
            }
        }
    }

    #[test]
    fn test_bc_1_08_001_suggest_type_close_match() {
        // "conten" is close to "content" (edit distance 1).
        let suggestion = suggest_type("conten");
        assert_eq!(suggestion, Some("content"), "should suggest 'content'");
    }

    #[test]
    fn test_bc_1_08_001_suggest_type_no_match_for_gibberish() {
        // A completely unrelated string should return None.
        let suggestion = suggest_type("xyzzy_not_a_type");
        assert!(
            suggestion.is_none(),
            "gibberish should not match any type; got: {suggestion:?}"
        );
    }

    #[test]
    fn test_bc_1_08_001_suggest_type_exact_match() {
        // An exact type name returns itself (edit distance 0).
        let suggestion = suggest_type("title");
        assert_eq!(suggestion, Some("title"));
    }

    #[test]
    fn test_known_fields_registry_types_present() {
        // Verify the specific registry types called out in F-WG-001 are present.
        for expected in &[
            "section_break",
            "two_col",
            "stat_callout",
            "process_flow",
            "executive_summary",
            "problem_statement",
            "recommendation",
            "risk_register",
            "matrix",
            "financials",
            "kpi_dashboard",
            "screenshot",
            "code_sample",
            "survey_results",
            "org_chart",
            "roadmap",
        ] {
            assert!(
                known_fields(expected).is_some(),
                "registry type '{expected}' must be in known_fields"
            );
        }
    }
}
