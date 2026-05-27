//! Region maps for the 31 built-in slide types.
//!
//! A region map defines the canonical [`BoundingBox`] for each content frame
//! within a slide type, computed relative to the default 16:9 widescreen page
//! (9,144,000 × 5,143,500 EMU = 10 × 5.625 inches).
//!
//! ## Coordinate System
//!
//! All values are in EMU (1 inch = 914,400 EMU). The origin is the top-left
//! corner of the slide canvas. X increases rightward; Y increases downward.
//!
//! ## Reference
//!
//! Region values are derived from the Python reference implementation in
//! `.factory/seed/reference/`. Pixel-to-inch ratios from the reference's 960×540
//! canvas (96 DPI equivalent) are converted: `px / 96 * 914_400`.
//!
//! ## Invariant
//!
//! Every `BoundingBox` returned by this module satisfies AC-014 / BC-3.06.003:
//! `x >= 0`, `y >= 0`, `width > 0`, `height > 0`,
//! `x + width <= page_width`, `y + height <= page_height`.

use crate::types::{BoundingBox, Emu, Frame, FrameContent};

/// Produce the canonical [`Frame`] list for a given slide type keyword on the
/// given page dimensions.
///
/// Returns an empty `Vec` when the `slide_type_keyword` is not recognised.
/// Callers in `layout::run` should check for this and return
/// [`crate::error::LayoutError::UnknownSlideType`] if the keyword is unknown.
///
/// # Arguments
///
/// * `slide_type_keyword` — the DSL keyword (e.g., `"title"`, `"content"`).
/// * `page_width` — the slide canvas width in EMU.
/// * `page_height` — the slide canvas height in EMU.
///
/// # Returns
///
/// A `Vec<Frame>` with `FrameContent::Empty` placeholders, ordered
/// semantically (title before subtitle/body). The implementer will fill
/// `FrameContent` variants with resolved content in `layout::run`.
// This function is a data-driven region map for 31 built-in slide types.
// Its length exceeds the clippy::too_many_lines limit by design — a match
// over 31 named slide types is inherently long and is the correct structure
// for this data. Each arm is a named slide type with its canonical geometry.
//
// Several slide types share the standard two-region layout (title header +
// full-width body) and are merged into a single arm to satisfy
// clippy::match_same_arms.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn region_frames_for(
    slide_type_keyword: &str,
    page_width: Emu,
    page_height: Emu,
) -> Option<Vec<Frame>> {
    // Compute scaling factors relative to the default 16:9 dimensions.
    // Region values are authored against the default page size.
    // For non-default page sizes, scale proportionally.
    let sx = |raw_emu: i64| -> Emu { Emu(raw_emu * page_width.0 / 9_144_000) };
    let sy = |raw_emu: i64| -> Emu { Emu(raw_emu * page_height.0 / 5_143_500) };

    let bbox = |x: i64, y: i64, w: i64, h: i64| -> BoundingBox {
        BoundingBox {
            x: sx(x),
            y: sy(y),
            width: sx(w),
            height: sy(h),
        }
    };

    let frames: Vec<Frame> = match slide_type_keyword {
        // ── title | closing ───────────────────────────────────────────────
        // Both use the same centered-title layout:
        // Title region: 0.5in, 1.75in, 9.0in, 1.25in
        // Subtitle/CTA region: 0.5in, 3.0in, 9.0in, 1.0in
        "title" | "closing" => vec![
            Frame {
                bbox: bbox(457_200, 1_600_200, 8_229_600, 1_143_000),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 2_743_200, 8_229_600, 914_400),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── Standard two-region layout (title header + full-width body) ───
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Body:  0.5in, 1.3in, 9.0in, 4.0in
        //
        // Slide types sharing this exact geometry:
        //   content, agenda, toc, team, executive_summary, problem_statement,
        //   recommendation, risk_register, timeline, process_flow, matrix,
        //   financials, kpi_dashboard, code_sample, survey_results, org_chart,
        //   roadmap
        "content" | "agenda" | "toc" | "team" | "executive_summary" | "problem_statement"
        | "recommendation" | "risk_register" | "timeline" | "process_flow" | "matrix"
        | "financials" | "kpi_dashboard" | "code_sample" | "survey_results" | "org_chart"
        | "roadmap" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_188_720, 8_229_600, 3_657_600),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── section_break ─────────────────────────────────────────────────
        // Section title: centered vertically, full width
        // Title: 0.5in, 1.9in, 9.0in, 1.5in
        // Subtitle: 0.5in, 3.5in, 9.0in, 1.0in
        "section_break" => vec![
            Frame {
                bbox: bbox(457_200, 1_737_360, 8_229_600, 1_371_600),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 3_200_400, 8_229_600, 914_400),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── two_col | comparison ───────────────────────────────────────────
        // Both use the same 3-region layout:
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Left body:  0.5in,  1.3in, 4.25in, 4.0in
        // Right body: 5.25in, 1.3in, 4.25in, 4.0in
        "two_col" | "comparison" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_188_720, 3_886_200, 3_657_600),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(4_800_600, 1_188_720, 3_886_200, 3_657_600),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── image ──────────────────────────────────────────────────────────
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Image: 1.5in, 1.3in, 7.0in, 4.0in
        "image" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(1_371_600, 1_188_720, 6_400_800, 3_657_600),
                content: FrameContent::Image {
                    alt: std::sync::Arc::from(""),
                },
                text_flow: None,
            },
        ],

        // ── blank ──────────────────────────────────────────────────────────
        // No frames — blank slide has zero content regions
        "blank" => vec![],

        // ── quote ─────────────────────────────────────────────────────────
        // Quote text: centered
        // Quote: 1.0in, 1.3in, 8.0in, 2.5in
        // Attribution: 1.0in, 4.0in, 8.0in, 0.75in
        "quote" => vec![
            Frame {
                bbox: bbox(914_400, 1_188_720, 7_315_200, 2_286_000),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(914_400, 3_657_600, 7_315_200, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── bio ───────────────────────────────────────────────────────────
        // Photo: 0.5in, 0.5in, 3.0in, 4.5in
        // Name+content: 4.0in, 0.5in, 5.5in, 4.5in
        "bio" => vec![
            Frame {
                bbox: bbox(457_200, 457_200, 2_743_200, 4_114_800),
                content: FrameContent::Image {
                    alt: std::sync::Arc::from(""),
                },
                text_flow: None,
            },
            Frame {
                bbox: bbox(3_657_600, 457_200, 5_029_200, 4_114_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── stat_callout ──────────────────────────────────────────────────
        // Title (optional): 0.5in, 0.3in, 9.0in, 0.75in
        // Left stat: 0.5in, 1.5in, 4.0in, 2.5in
        // Right stat: 4.7in, 1.5in, 4.0in, 2.5in
        // Left label: 0.5in, 4.0in, 4.0in, 0.75in
        // Right label: 4.7in, 4.0in, 4.0in, 0.75in
        "stat_callout" => vec![
            Frame {
                bbox: bbox(457_200, 274_320, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_371_600, 3_657_600, 2_286_000),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(4_297_680, 1_371_600, 3_657_600, 2_286_000),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 3_657_600, 3_657_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(4_297_680, 3_657_600, 3_657_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // ── chart ─────────────────────────────────────────────────────────
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Chart: 0.5in, 1.3in, 9.0in, 4.0in
        "chart" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_188_720, 8_229_600, 3_657_600),
                content: FrameContent::Chart,
                text_flow: None,
            },
        ],

        // ── diagram ───────────────────────────────────────────────────────
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Diagram: 0.5in, 1.3in, 9.0in, 4.0in
        "diagram" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_188_720, 8_229_600, 3_657_600),
                content: FrameContent::Diagram,
                text_flow: None,
            },
        ],

        // ── screenshot ────────────────────────────────────────────────────
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Image: 0.5in, 1.3in, 9.0in, 4.0in
        "screenshot" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(457_200, 1_188_720, 8_229_600, 3_657_600),
                content: FrameContent::Image {
                    alt: std::sync::Arc::from(""),
                },
                text_flow: None,
            },
        ],

        // ── video ─────────────────────────────────────────────────────────
        // Title: 0.5in, 0.4in, 9.0in, 0.75in
        // Video placeholder: 1.5in, 1.3in, 7.0in, 4.0in
        "video" => vec![
            Frame {
                bbox: bbox(457_200, 365_760, 8_229_600, 685_800),
                content: FrameContent::Empty,
                text_flow: None,
            },
            Frame {
                bbox: bbox(1_371_600, 1_188_720, 6_400_800, 3_657_600),
                content: FrameContent::Empty,
                text_flow: None,
            },
        ],

        // Unknown keyword — caller should return LayoutError::UnknownSlideType
        _ => return None,
    };

    Some(frames)
}

#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod tests {
    use super::*;
    use crate::types::{DEFAULT_PAGE_HEIGHT, DEFAULT_PAGE_WIDTH};

    // ─────────────────────────────────────────────────────────────────────────
    // AC-006 — SlideType-driven region layout
    // AC-014 — Valid EMU coordinates for all frames
    // ─────────────────────────────────────────────────────────────────────────

    fn assert_all_valid(slide_type_keyword: &str) {
        let frames = region_frames_for(slide_type_keyword, DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .unwrap_or_else(|| {
                panic!("region_frames_for returned None for '{slide_type_keyword}'")
            });
        for (i, frame) in frames.iter().enumerate() {
            assert!(
                frame.bbox.is_valid(DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT),
                "frame {i} of slide type '{slide_type_keyword}' has invalid bbox: {:?}",
                frame.bbox
            );
        }
    }

    /// AC-006 — title slide produces title + subtitle frames with EMU values.
    #[test]
    fn test_bc_3_06_002_title_slide_regions() {
        let frames = region_frames_for("title", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .expect("title slide must have region frames");
        // title has 2 frames: title + subtitle
        assert_eq!(
            frames.len(),
            2,
            "title slide must produce 2 frames (title, subtitle)"
        );
        // Title frame: x=457200, y=1600200, width=8229600, height=1143000
        assert_eq!(frames[0].bbox.x, Emu(457_200));
        assert_eq!(frames[0].bbox.y, Emu(1_600_200));
        assert_eq!(frames[0].bbox.width, Emu(8_229_600));
        assert_eq!(frames[0].bbox.height, Emu(1_143_000));
        // Subtitle frame
        assert_eq!(frames[1].bbox.x, Emu(457_200));
        assert_eq!(frames[1].bbox.y, Emu(2_743_200));
        assert_eq!(frames[1].bbox.width, Emu(8_229_600));
        assert_eq!(frames[1].bbox.height, Emu(914_400));
    }

    /// AC-006 — content slide produces title + body frames.
    #[test]
    fn test_bc_3_06_002_content_slide_regions() {
        let frames = region_frames_for("content", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .expect("content slide must have region frames");
        assert_eq!(
            frames.len(),
            2,
            "content slide must produce 2 frames (title, body)"
        );
        assert_eq!(frames[0].bbox.x, Emu(457_200));
        assert_eq!(frames[1].bbox.x, Emu(457_200));
        // Body is taller
        assert!(
            frames[1].bbox.height > frames[0].bbox.height,
            "body frame must be taller than title frame"
        );
    }

    /// AC-006 — blank slide produces zero frames.
    #[test]
    fn test_bc_3_06_002_blank_slide_regions_empty() {
        let frames = region_frames_for("blank", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .expect("blank slide must return Some(vec![])");
        assert!(frames.is_empty(), "blank slide must produce zero frames");
    }

    /// AC-006 — `stat_callout` produces 5 frames.
    #[test]
    fn test_bc_3_06_002_stat_callout_slide_regions() {
        let frames = region_frames_for("stat_callout", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .expect("stat_callout slide must have region frames");
        assert_eq!(
            frames.len(),
            5,
            "stat_callout must produce 5 frames (title + 2 stats + 2 labels)"
        );
        assert_all_valid("stat_callout");
    }

    /// AC-006 — `two_col` produces 3 frames.
    #[test]
    fn test_bc_3_06_002_two_col_slide_regions() {
        let frames = region_frames_for("two_col", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
            .expect("two_col slide must have region frames");
        assert_eq!(
            frames.len(),
            3,
            "two_col must produce 3 frames (title + 2 columns)"
        );
        assert_all_valid("two_col");
    }

    /// EC-002 — Unknown slide type keyword returns None.
    #[test]
    fn test_bc_3_06_002_unknown_slide_type_returns_none() {
        let result = region_frames_for("not_a_real_type", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT);
        assert!(
            result.is_none(),
            "region_frames_for must return None for unknown keyword"
        );
    }

    /// AC-014 — All 31 built-in slide types produce valid `BoundingBox`es.
    #[test]
    fn test_bc_3_06_003_all_slide_types_valid_bounding_boxes() {
        let known_types = [
            "title",
            "content",
            "section_break",
            "two_col",
            "image",
            "blank",
            "agenda",
            "toc",
            "quote",
            "team",
            "bio",
            "executive_summary",
            "problem_statement",
            "recommendation",
            "risk_register",
            "timeline",
            "stat_callout",
            "comparison",
            "process_flow",
            "matrix",
            "financials",
            "kpi_dashboard",
            "chart",
            "diagram",
            "screenshot",
            "code_sample",
            "video",
            "survey_results",
            "org_chart",
            "roadmap",
            "closing",
        ];
        assert_eq!(known_types.len(), 31, "must cover all 31 built-in types");
        for kw in known_types {
            assert_all_valid(kw);
        }
    }

    /// AC-006 — snapshot test for title slide region map (insta yaml).
    #[test]
    fn test_bc_3_06_002_title_slide_regions_snapshot() {
        let frames = region_frames_for("title", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        // Snapshot using insta (will auto-create on first run in review mode)
        insta::assert_debug_snapshot!("title_slide_regions", frames);
    }

    /// AC-006 — snapshot test for content slide region map.
    #[test]
    fn test_bc_3_06_002_content_slide_regions_snapshot() {
        let frames = region_frames_for("content", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("content_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `section_break` slide region map.
    #[test]
    fn test_bc_3_06_002_section_break_regions_snapshot() {
        let frames =
            region_frames_for("section_break", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("section_break_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `stat_callout` slide region map.
    #[test]
    fn test_bc_3_06_002_stat_callout_regions_snapshot() {
        let frames =
            region_frames_for("stat_callout", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("stat_callout_slide_regions", frames);
    }

    /// AC-006 — snapshot test for chart slide region map.
    #[test]
    fn test_bc_3_06_002_chart_regions_snapshot() {
        let frames = region_frames_for("chart", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("chart_slide_regions", frames);
    }

    /// AC-006 — snapshot test for closing slide region map.
    #[test]
    fn test_bc_3_06_002_closing_regions_snapshot() {
        let frames = region_frames_for("closing", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("closing_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `two_col` slide region map.
    #[test]
    fn test_bc_3_06_002_two_col_regions_snapshot() {
        let frames = region_frames_for("two_col", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("two_col_slide_regions", frames);
    }

    /// AC-006 — snapshot test for comparison slide region map.
    #[test]
    fn test_bc_3_06_002_comparison_regions_snapshot() {
        let frames =
            region_frames_for("comparison", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("comparison_slide_regions", frames);
    }

    /// AC-006 — snapshot test for image slide region map.
    #[test]
    fn test_bc_3_06_002_image_regions_snapshot() {
        let frames = region_frames_for("image", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("image_slide_regions", frames);
    }

    /// AC-006 — snapshot test for blank slide region map.
    #[test]
    fn test_bc_3_06_002_blank_regions_snapshot() {
        let frames = region_frames_for("blank", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("blank_slide_regions", frames);
    }

    /// AC-006 — snapshot test for quote slide region map.
    #[test]
    fn test_bc_3_06_002_quote_regions_snapshot() {
        let frames = region_frames_for("quote", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("quote_slide_regions", frames);
    }

    /// AC-006 — snapshot test for bio slide region map.
    #[test]
    fn test_bc_3_06_002_bio_regions_snapshot() {
        let frames = region_frames_for("bio", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("bio_slide_regions", frames);
    }

    /// AC-006 — snapshot test for agenda slide region map.
    #[test]
    fn test_bc_3_06_002_agenda_regions_snapshot() {
        let frames = region_frames_for("agenda", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("agenda_slide_regions", frames);
    }

    /// AC-006 — snapshot test for toc slide region map.
    #[test]
    fn test_bc_3_06_002_toc_regions_snapshot() {
        let frames = region_frames_for("toc", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("toc_slide_regions", frames);
    }

    /// AC-006 — snapshot test for team slide region map.
    #[test]
    fn test_bc_3_06_002_team_regions_snapshot() {
        let frames = region_frames_for("team", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("team_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `executive_summary` slide region map.
    #[test]
    fn test_bc_3_06_002_executive_summary_regions_snapshot() {
        let frames =
            region_frames_for("executive_summary", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
                .unwrap();
        insta::assert_debug_snapshot!("executive_summary_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `problem_statement` slide region map.
    #[test]
    fn test_bc_3_06_002_problem_statement_regions_snapshot() {
        let frames =
            region_frames_for("problem_statement", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT)
                .unwrap();
        insta::assert_debug_snapshot!("problem_statement_slide_regions", frames);
    }

    /// AC-006 — snapshot test for recommendation slide region map.
    #[test]
    fn test_bc_3_06_002_recommendation_regions_snapshot() {
        let frames =
            region_frames_for("recommendation", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("recommendation_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `risk_register` slide region map.
    #[test]
    fn test_bc_3_06_002_risk_register_regions_snapshot() {
        let frames =
            region_frames_for("risk_register", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("risk_register_slide_regions", frames);
    }

    /// AC-006 — snapshot test for timeline slide region map.
    #[test]
    fn test_bc_3_06_002_timeline_regions_snapshot() {
        let frames =
            region_frames_for("timeline", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("timeline_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `process_flow` slide region map.
    #[test]
    fn test_bc_3_06_002_process_flow_regions_snapshot() {
        let frames =
            region_frames_for("process_flow", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("process_flow_slide_regions", frames);
    }

    /// AC-006 — snapshot test for matrix slide region map.
    #[test]
    fn test_bc_3_06_002_matrix_regions_snapshot() {
        let frames = region_frames_for("matrix", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("matrix_slide_regions", frames);
    }

    /// AC-006 — snapshot test for financials slide region map.
    #[test]
    fn test_bc_3_06_002_financials_regions_snapshot() {
        let frames =
            region_frames_for("financials", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("financials_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `kpi_dashboard` slide region map.
    #[test]
    fn test_bc_3_06_002_kpi_dashboard_regions_snapshot() {
        let frames =
            region_frames_for("kpi_dashboard", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("kpi_dashboard_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `code_sample` slide region map.
    #[test]
    fn test_bc_3_06_002_code_sample_regions_snapshot() {
        let frames =
            region_frames_for("code_sample", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("code_sample_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `survey_results` slide region map.
    #[test]
    fn test_bc_3_06_002_survey_results_regions_snapshot() {
        let frames =
            region_frames_for("survey_results", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("survey_results_slide_regions", frames);
    }

    /// AC-006 — snapshot test for `org_chart` slide region map.
    #[test]
    fn test_bc_3_06_002_org_chart_regions_snapshot() {
        let frames =
            region_frames_for("org_chart", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("org_chart_slide_regions", frames);
    }

    /// AC-006 — snapshot test for roadmap slide region map.
    #[test]
    fn test_bc_3_06_002_roadmap_regions_snapshot() {
        let frames = region_frames_for("roadmap", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("roadmap_slide_regions", frames);
    }

    /// AC-006 — snapshot test for diagram slide region map.
    #[test]
    fn test_bc_3_06_002_diagram_regions_snapshot() {
        let frames = region_frames_for("diagram", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("diagram_slide_regions", frames);
    }

    /// AC-006 — snapshot test for screenshot slide region map.
    #[test]
    fn test_bc_3_06_002_screenshot_regions_snapshot() {
        let frames =
            region_frames_for("screenshot", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("screenshot_slide_regions", frames);
    }

    /// AC-006 — snapshot test for video slide region map.
    #[test]
    fn test_bc_3_06_002_video_regions_snapshot() {
        let frames = region_frames_for("video", DEFAULT_PAGE_WIDTH, DEFAULT_PAGE_HEIGHT).unwrap();
        insta::assert_debug_snapshot!("video_slide_regions", frames);
    }

    /// EC-003 — Non-default page size scales proportionally.
    #[test]
    fn test_bc_3_06_002_non_default_page_size_scales_proportionally() {
        // Use 4:3 aspect ratio — same width, taller height
        let custom_height = Emu(6_858_000);
        let frames = region_frames_for("title", DEFAULT_PAGE_WIDTH, custom_height)
            .expect("title must work with non-default height");
        for (i, frame) in frames.iter().enumerate() {
            assert!(
                frame.bbox.is_valid(DEFAULT_PAGE_WIDTH, custom_height),
                "frame {i} must be valid at 4:3 page size"
            );
        }
    }
}
