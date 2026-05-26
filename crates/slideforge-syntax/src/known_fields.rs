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
//! Per the 25 Q&A planning decisions (q2-decision-final.md, BINDING):
//! - 23 original seed types
//! - 8 additional types: `chart`, `toc`, `agenda`, `quote`, `grid`, `bio`,
//!   `diagram`, `team`
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
/// assert!(known_fields("content").unwrap().contains(&"body"));
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
            "byline",
            "date",
            "background",
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
            "body",
            "subtitle",
            "background",
        ]),
        "section" => Some(&[
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
            "background",
        ]),
        "bullets" => Some(&[
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
            "bullet_style",
        ]),
        "numbered" => Some(&[
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
            "start",
        ]),
        "two_column" => Some(&[
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
        "image" => Some(&[
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
            "width",
            "height",
            "align",
        ]),
        "full_image" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "src",
            "caption",
            "overlay_title",
        ]),
        "statement" => Some(&[
            "tags",
            "notes",
            "report",
            "detail",
            "alt",
            "lang",
            "decorative",
            "footer",
            "logo",
            "text",
            "attribution",
            "background",
        ]),
        "stat" => Some(&[
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
            "stat",
            "label",
            "context",
        ]),
        "three_stats" => Some(&[
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
            "stat1",
            "label1",
            "stat2",
            "label2",
            "stat3",
            "label3",
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
            "left_label",
            "right_label",
            "left_items",
            "right_items",
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
            "events",
        ]),
        "process" => Some(&[
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
        "code" => Some(&[
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
            "source",
            "language",
            "highlight_lines",
        ]),
        "table" => Some(&[
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
            "headers",
            "rows",
        ]),
        "map" => Some(&[
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
            "region",
            "highlights",
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
            "subtitle",
            "cta",
            "contact",
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
            "background",
        ]),
        "divider" => Some(&[
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
            "background",
        ]),
        "callout" => Some(&[
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
            "body",
            "kind",
        ]),
        "agenda" | "toc" => Some(&[
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
        // ── Additional types (q2-decision-final.md) ───────────────────────────
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
            "data",
            "kind",
            "x_label",
            "y_label",
            "caption",
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
            "text",
            "attribution",
            "role",
            "background",
        ]),
        "grid" => Some(&[
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
            "columns",
        ]),
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
            "role",
            "body",
            "photo",
            "links",
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
            "source",
            "caption",
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
        // ── Additional types commonly referenced in DSL docs ──────────────────
        "warning" | "info" => Some(&[
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
            "body",
        ]),
        _ => None,
    }
}

/// Return the list of all 31 built-in slide type names.
///
/// This is used for error messages (e.g., "Did you mean X?") and for
/// iterating over all known types.
#[must_use]
pub fn all_slide_types() -> &'static [&'static str] {
    &[
        "title",
        "content",
        "section",
        "bullets",
        "numbered",
        "two_column",
        "image",
        "full_image",
        "statement",
        "stat",
        "three_stats",
        "comparison",
        "timeline",
        "process",
        "code",
        "table",
        "map",
        "closing",
        "blank",
        "divider",
        "callout",
        "agenda",
        "toc",
        "chart",
        "quote",
        "grid",
        "bio",
        "diagram",
        "team",
        "warning",
        "info",
    ]
}

/// Suggest the closest known slide type name to `given` (Levenshtein distance ≤ 3).
///
/// Returns `None` if no type is close enough to be a useful suggestion.
#[must_use]
pub fn suggest_type(given: &str) -> Option<&'static str> {
    all_slide_types()
        .iter()
        .filter_map(|&candidate| {
            let dist = edit_distance(given, candidate);
            if dist <= 3 {
                Some((dist, candidate))
            } else {
                None
            }
        })
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, name)| name)
}

/// Compute the Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    #[allow(clippy::needless_range_loop)] // 2-D DP: row index required for cross-cell lookups
    for i in 0..=m {
        dp[i][0] = i;
    }
    #[allow(clippy::needless_range_loop)] // 2-D DP: column index required for cross-cell lookups
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
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
    fn test_bc_1_09_002_known_fields_content_includes_body() {
        let fields = known_fields("content").expect("content must be a known slide type");
        assert!(
            fields.contains(&"body"),
            "content type must have a 'body' field"
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
    fn test_known_fields_edit_distance_correctness() {
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "ab"), 1);
        assert_eq!(edit_distance("abc", "axc"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
    }
}
