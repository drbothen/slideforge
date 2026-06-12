//! Compile-time known-field registry for all 31 slideforge slide types.
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
//! # All 31 Slide Types
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
#[allow(clippy::too_many_lines)] // 31 slide types × ~10 fields each — exhaustive lookup table, not logic
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

/// Return the list of all 31 built-in slide type names.
///
/// These match exactly the keywords registered in `SlideTypeRegistry::default()`
/// in `slideforge-plugin-api`. This function is the compile-time counterpart of
/// the runtime registry's `all_keywords()` method.
///
/// This is used for error messages (e.g., "Did you mean X?") and for
/// iterating over all known types.
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
    fn test_bc_1_08_001_all_31_types_registered() {
        // All 31 built-in slide types must have known fields.
        for &ty in all_slide_types() {
            assert!(
                known_fields(ty).is_some(),
                "slide type '{ty}' must be in known_fields"
            );
        }
        assert_eq!(
            all_slide_types().len(),
            31,
            "must have exactly 31 built-in slide types"
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
