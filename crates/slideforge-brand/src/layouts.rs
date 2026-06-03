//! Slide layout definitions for the 31-layout brand template (BC-2.01.005).
//!
//! [`generate_all_layouts`] produces all 31 [`SlideLayoutDef`] entries in index
//! order: 11 standard OOXML layouts (SL-01 through SL-11) followed by 20 custom
//! `SF *` layouts (CL-01 through CL-20) as specified in Spike S5.
//!
//! ## Layout Taxonomy
//!
//! | Index | ID | Name | OOXML type |
//! |-------|----|------|-----------|
//! | 1 | SL-01 | Title Slide | `title` |
//! | 2 | SL-02 | Title and Content | `obj` |
//! | 3 | SL-03 | Section Header | `secHead` |
//! | 4 | SL-04 | Two Content | `twoObj` |
//! | 5 | SL-05 | Comparison | `twoColTx` |
//! | 6 | SL-06 | Title Only | `titleOnly` |
//! | 7 | SL-07 | Blank | `blank` |
//! | 8 | SL-08 | Content with Caption | `objTx` |
//! | 9 | SL-09 | Picture with Caption | `picTx` |
//! | 10 | SL-10 | Vertical Title and Text | `vertTitleAndTx` |
//! | 11 | SL-11 | Vertical Text | `vertTx` |
//! | 12 | CL-01 | SF Section Divider | — (dark layout) |
//! | … | … | … | … |
//! | 31 | CL-20 | SF Appendix | — |

use std::sync::Arc;

use crate::toml_schema::BrandConfig;

// ─── Types ────────────────────────────────────────────────────────────────────

/// A single slide layout definition.
///
/// Produced by [`generate_all_layouts`] and stored in [`crate::template::BrandTemplate`].
/// The PPTX exporter (STORY-037) serializes each `SlideLayoutDef` to a
/// `slideLayoutN.xml` file inside the OOXML ZIP package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlideLayoutDef {
    /// 1-based layout index (1–31).
    ///
    /// Used to determine the OOXML file name `slideLayout{index}.xml` and
    /// the OOXML layout ID (= `layout_id_start + index - 1`).
    pub index: usize,

    /// Human-readable layout name (e.g., `"Title Slide"` or `"SF Section Divider"`).
    ///
    /// Standard layouts use Office display names.
    /// Custom layouts always start with `"SF "` (two letters + space).
    pub name: Arc<str>,

    /// OOXML `<p:sldLayout type="...">` attribute value.
    ///
    /// `Some("title")`, `Some("obj")`, etc. for standard layouts.
    /// `None` for custom `SF *` layouts (they use `type="custom"`).
    pub ooxml_type: Option<Arc<str>>,

    /// Placeholder definitions for this layout.
    ///
    /// The Blank layout (SL-07) has an empty `Vec`.
    /// All other layouts have at least one placeholder (AC-010 invariant).
    pub placeholders: Vec<LayoutPlaceholder>,

    /// DSL slide-type keyword for this layout (e.g., `"section_divider"`, `"end"`).
    ///
    /// `Some(keyword)` for the 20 SF custom layouts; the keyword matches the
    /// canonical Q2 DSL keyword for that layout type (ADR-015 §A.4).
    ///
    /// `None` for the 11 standard OOXML layouts (SL-01..SL-11) — these are
    /// looked up by `ooxml_type`, not by DSL keyword.
    pub slide_type_keyword: Option<Arc<str>>,

    /// Whether this layout has a dark color map override.
    ///
    /// `true` for CL-01 ("SF Section Divider") and CL-11 ("SF End Slide").
    /// These layouts carry `<p:clrMapOvr>` with `bg1="dk2" tx1="lt1"` (AC-009).
    pub has_color_override: bool,

    /// Background color token for the color map override.
    ///
    /// `Some("dk2")` for dark layouts; `None` for all other layouts.
    pub color_override_bg: Option<Arc<str>>,

    /// Text color token for the color map override.
    ///
    /// `Some("lt1")` for dark layouts; `None` for all other layouts.
    pub color_override_tx: Option<Arc<str>>,
}

/// A placeholder shape within a slide layout.
///
/// Each placeholder carries a semantic type, an OOXML index, a semantic
/// accessibility name, and position/size in EMU (English Metric Units).
///
/// Used by the PPTX exporter (STORY-037) to generate `<p:sp>` placeholder
/// elements inside `<p:spTree>` for each layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutPlaceholder {
    /// OOXML placeholder type string (e.g., `"ctrTitle"`, `"body"`, `"pic"`).
    ///
    /// `"title"` or `"ctrTitle"` for title placeholders; `"body"` for content.
    pub ph_type: Arc<str>,

    /// OOXML placeholder index (`idx` attribute).
    ///
    /// The title/center-title placeholder has `idx = 0`.
    /// Body/content placeholders start at `idx = 1`.
    pub idx: u32,

    /// Semantic accessibility name for `<p:cNvPr name="...">`.
    ///
    /// Must be a descriptive label, NOT `"Shape N"` (AC-010 invariant,
    /// DI-001 accessibility requirement).
    pub accessibility_name: Arc<str>,

    /// X position in EMU.
    pub x: i64,

    /// Y position in EMU.
    pub y: i64,

    /// Width in EMU.
    pub cx: i64,

    /// Height in EMU.
    pub cy: i64,
}

// ─── EMU constants ────────────────────────────────────────────────────────────

/// Standard title placeholder X origin in EMUs (`PowerPoint` defaults, 16:9).
const TITLE_X: i64 = 457_200;
/// Standard title placeholder Y origin in EMUs.
const TITLE_Y: i64 = 274_638;
/// Standard title placeholder width in EMUs.
const TITLE_CX: i64 = 8_229_600;
/// Standard title placeholder height in EMUs.
const TITLE_CY: i64 = 1_143_000;

/// Standard body/content placeholder X origin in EMUs.
const BODY_X: i64 = 457_200;
/// Standard body/content placeholder Y origin in EMUs.
const BODY_Y: i64 = 1_600_200;
/// Standard body/content placeholder width in EMUs.
const BODY_CX: i64 = 8_229_600;
/// Standard body/content placeholder height in EMUs.
const BODY_CY: i64 = 3_543_300;

/// Left half content X origin in EMUs (two-column layouts).
const LEFT_X: i64 = 457_200;
/// Left half content width in EMUs.
const LEFT_CX: i64 = 3_962_400;

/// Right half content X origin in EMUs (two-column layouts).
const RIGHT_X: i64 = 4_724_400;
/// Right half content width in EMUs.
const RIGHT_CX: i64 = 3_962_400;

/// Two-column body Y origin in EMUs.
const TWO_COL_Y: i64 = 1_600_200;
/// Two-column body height in EMUs.
const TWO_COL_CY: i64 = 3_543_300;

// ─── Builder helpers ──────────────────────────────────────────────────────────

/// Build a title placeholder.
fn title_ph(ph_type: &str, name: &str) -> LayoutPlaceholder {
    LayoutPlaceholder {
        ph_type: Arc::from(ph_type),
        idx: 0,
        accessibility_name: Arc::from(name),
        x: TITLE_X,
        y: TITLE_Y,
        cx: TITLE_CX,
        cy: TITLE_CY,
    }
}

/// Build a body placeholder.
fn body_ph(idx: u32, name: &str) -> LayoutPlaceholder {
    LayoutPlaceholder {
        ph_type: Arc::from("body"),
        idx,
        accessibility_name: Arc::from(name),
        x: BODY_X,
        y: BODY_Y,
        cx: BODY_CX,
        cy: BODY_CY,
    }
}

/// Build a light (no color override) standard layout.
///
/// Standard layouts (SL-01..SL-11) are referenced by `ooxml_type`, not by
/// DSL keyword. They receive `slide_type_keyword = None`.
fn light_layout(
    index: usize,
    name: &str,
    ooxml_type: &str,
    placeholders: Vec<LayoutPlaceholder>,
) -> SlideLayoutDef {
    SlideLayoutDef {
        index,
        name: Arc::from(name),
        ooxml_type: Some(Arc::from(ooxml_type)),
        slide_type_keyword: None,
        placeholders,
        has_color_override: false,
        color_override_bg: None,
        color_override_tx: None,
    }
}

/// Build a custom (no OOXML type) light layout with a canonical DSL keyword.
///
/// Custom layouts (CL-01..CL-20) receive a `slide_type_keyword` from the
/// canonical Q2 DSL keyword table (ADR-015 §A.4). This keyword is the
/// source of truth for `find_layout_index` in `slideforge-pptx`.
fn custom_layout(
    index: usize,
    name: &str,
    keyword: &str,
    placeholders: Vec<LayoutPlaceholder>,
) -> SlideLayoutDef {
    SlideLayoutDef {
        index,
        name: Arc::from(name),
        ooxml_type: None,
        slide_type_keyword: Some(Arc::from(keyword)),
        placeholders,
        has_color_override: false,
        color_override_bg: None,
        color_override_tx: None,
    }
}

/// Build a dark custom layout (with clrMapOvr) and a canonical DSL keyword.
///
/// Dark layouts carry `<p:clrMapOvr>` with `bg1="dk2" tx1="lt1"` (AC-009)
/// and a `slide_type_keyword` from the canonical Q2 DSL keyword table (ADR-015 §A.4).
fn dark_layout(
    index: usize,
    name: &str,
    keyword: &str,
    placeholders: Vec<LayoutPlaceholder>,
) -> SlideLayoutDef {
    SlideLayoutDef {
        index,
        name: Arc::from(name),
        ooxml_type: None,
        slide_type_keyword: Some(Arc::from(keyword)),
        placeholders,
        has_color_override: true,
        color_override_bg: Some(Arc::from("dk2")),
        color_override_tx: Some(Arc::from("lt1")),
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Generate all 31 slide layout definitions.
///
/// Returns a `Vec<SlideLayoutDef>` with exactly 31 entries in index order
/// (1-based). The first 11 are standard OOXML layouts; the remaining 20
/// are custom `SF *` layouts.
///
/// `config` is used to parameterize layout geometry or color choices in the
/// future; the current implementation ignores it (layouts use standard
/// `PowerPoint` default positions).
///
/// # Contract
///
/// - `result.len() == 31` always (BC-2.01.005 postcondition 1)
/// - `result[0..11]` all have `ooxml_type = Some(...)` (BC-2.01.005 postcondition 2)
/// - `result[11..31]` all have names starting with `"SF "` (BC-2.01.005 postcondition 3)
/// - `result[6].placeholders.is_empty()` (Blank layout — AC-010)
/// - `result[11].has_color_override && result[21].has_color_override` (AC-009)
// The body is a declarative data initialiser for 31 layout definitions.
// Splitting it into sub-functions would add artificial indirection without
// improving readability. The line-count lint is suppressed here deliberately.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn generate_all_layouts(_config: &BrandConfig) -> Vec<SlideLayoutDef> {
    vec![
        // ── Standard layouts (SL-01..SL-11) ──────────────────────────────────

        // SL-01: Title Slide
        light_layout(
            1,
            "Title Slide",
            "title",
            vec![
                title_ph("ctrTitle", "Title Placeholder"),
                LayoutPlaceholder {
                    ph_type: Arc::from("subTitle"),
                    idx: 1,
                    accessibility_name: Arc::from("Subtitle Placeholder"),
                    x: TITLE_X,
                    y: 1_600_200,
                    cx: TITLE_CX,
                    cy: 914_400,
                },
            ],
        ),
        // SL-02: Title and Content
        light_layout(
            2,
            "Title and Content",
            "obj",
            vec![
                title_ph("title", "Title"),
                body_ph(1, "Content Placeholder"),
            ],
        ),
        // SL-03: Section Header
        light_layout(
            3,
            "Section Header",
            "secHead",
            vec![
                title_ph("title", "Section Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Section Description"),
                    x: TITLE_X,
                    y: 1_600_200,
                    cx: TITLE_CX,
                    cy: 914_400,
                },
            ],
        ),
        // SL-04: Two Content
        light_layout(
            4,
            "Two Content",
            "twoObj",
            vec![
                title_ph("title", "Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Left Content"),
                    x: LEFT_X,
                    y: TWO_COL_Y,
                    cx: LEFT_CX,
                    cy: TWO_COL_CY,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Right Content"),
                    x: RIGHT_X,
                    y: TWO_COL_Y,
                    cx: RIGHT_CX,
                    cy: TWO_COL_CY,
                },
            ],
        ),
        // SL-05: Comparison
        light_layout(
            5,
            "Comparison",
            "twoColTx",
            vec![
                title_ph("title", "Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Left Header"),
                    x: LEFT_X,
                    y: TWO_COL_Y,
                    cx: LEFT_CX,
                    cy: 571_500,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Left Content"),
                    x: LEFT_X,
                    y: 2_286_000,
                    cx: LEFT_CX,
                    cy: 2_743_200,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 3,
                    accessibility_name: Arc::from("Right Header"),
                    x: RIGHT_X,
                    y: TWO_COL_Y,
                    cx: RIGHT_CX,
                    cy: 571_500,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 4,
                    accessibility_name: Arc::from("Right Content"),
                    x: RIGHT_X,
                    y: 2_286_000,
                    cx: RIGHT_CX,
                    cy: 2_743_200,
                },
            ],
        ),
        // SL-06: Title Only
        light_layout(
            6,
            "Title Only",
            "titleOnly",
            vec![title_ph("title", "Title")],
        ),
        // SL-07: Blank — zero placeholders (AC-010)
        light_layout(7, "Blank", "blank", vec![]),
        // SL-08: Content with Caption
        light_layout(
            8,
            "Content with Caption",
            "objTx",
            vec![
                title_ph("title", "Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Content Area"),
                    x: LEFT_X,
                    y: BODY_Y,
                    cx: 5_486_400,
                    cy: BODY_CY,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Caption"),
                    x: 6_096_000,
                    y: BODY_Y,
                    cx: 2_590_800,
                    cy: BODY_CY,
                },
            ],
        ),
        // SL-09: Picture with Caption
        light_layout(
            9,
            "Picture with Caption",
            "picTx",
            vec![
                title_ph("title", "Picture Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("pic"),
                    idx: 1,
                    accessibility_name: Arc::from("Picture Placeholder"),
                    x: TITLE_X,
                    y: 1_371_600,
                    cx: TITLE_CX,
                    cy: 2_743_200,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Caption Text"),
                    x: TITLE_X,
                    y: 4_343_400,
                    cx: TITLE_CX,
                    cy: 571_500,
                },
            ],
        ),
        // SL-10: Vertical Title and Text
        light_layout(
            10,
            "Vertical Title and Text",
            "vertTitleAndTx",
            vec![
                LayoutPlaceholder {
                    ph_type: Arc::from("title"),
                    idx: 0,
                    accessibility_name: Arc::from("Vertical Title"),
                    x: 7_315_200,
                    y: TITLE_Y,
                    cx: 1_371_600,
                    cy: BODY_CY,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Vertical Content"),
                    x: TITLE_X,
                    y: TITLE_Y,
                    cx: 6_553_200,
                    cy: BODY_CY,
                },
            ],
        ),
        // SL-11: Vertical Text
        // AC-010 requires a title-type placeholder. For vertTx layout the heading
        // sits at the right edge (OOXML vertTx convention): narrow column on the
        // right, body spans the remaining left width.
        light_layout(
            11,
            "Vertical Text",
            "vertTx",
            vec![
                LayoutPlaceholder {
                    ph_type: Arc::from("title"),
                    idx: 0,
                    accessibility_name: Arc::from("Vertical Text Heading"),
                    // Right edge vertical title: x = slide_width - narrow_col, narrow col = 1371600 EMU
                    x: 7_315_200,
                    y: TITLE_Y,
                    cx: 1_371_600,
                    cy: BODY_CY,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Vertical Text Area"),
                    x: TITLE_X,
                    y: TITLE_Y,
                    cx: 6_553_200,
                    cy: BODY_CY,
                },
            ],
        ),
        // ── Custom layouts (CL-01..CL-20) ─────────────────────────────────────

        // CL-01: SF Section Divider (DARK LAYOUT) — DSL keyword: section_divider
        dark_layout(
            12,
            "SF Section Divider",
            "section_divider",
            vec![
                title_ph("title", "Section Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Section Subtitle"),
                    x: TITLE_X,
                    y: 1_600_200,
                    cx: TITLE_CX,
                    cy: 914_400,
                },
            ],
        ),
        // CL-02: SF Stat Grid — DSL keyword: stat_callout
        custom_layout(
            13,
            "SF Stat Grid",
            "stat_callout",
            vec![
                title_ph("title", "Stats Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Statistics Grid"),
                    x: BODY_X,
                    y: BODY_Y,
                    cx: BODY_CX,
                    cy: BODY_CY,
                },
            ],
        ),
        // CL-03: SF Quote — DSL keyword: quote
        // AC-010 requires a title-type placeholder. The quote title sits at the top
        // of the slide (standard title position), followed by the pullquote body and
        // attribution body below.
        custom_layout(
            14,
            "SF Quote",
            "quote",
            vec![
                title_ph("title", "Quote Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Quote Text"),
                    x: TITLE_X,
                    y: 1_371_600,
                    cx: TITLE_CX,
                    cy: 2_286_000,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Quote Attribution"),
                    x: TITLE_X,
                    y: 3_886_200,
                    cx: TITLE_CX,
                    cy: 571_500,
                },
            ],
        ),
        // CL-04: SF Timeline — DSL keyword: vertical_timeline
        custom_layout(
            15,
            "SF Timeline",
            "vertical_timeline",
            vec![
                title_ph("title", "Timeline Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Timeline Content"),
                    x: BODY_X,
                    y: BODY_Y,
                    cx: BODY_CX,
                    cy: BODY_CY,
                },
            ],
        ),
        // CL-05: SF Agenda — DSL keyword: agenda
        custom_layout(
            16,
            "SF Agenda",
            "agenda",
            vec![
                title_ph("title", "Agenda Title"),
                body_ph(1, "Agenda Items"),
            ],
        ),
        // CL-06: SF TOC — DSL keyword: toc
        custom_layout(
            17,
            "SF TOC",
            "toc",
            vec![
                title_ph("title", "Table of Contents Title"),
                body_ph(1, "TOC Items"),
            ],
        ),
        // CL-07: SF Bio — DSL keyword: bio
        custom_layout(
            18,
            "SF Bio",
            "bio",
            vec![
                title_ph("title", "Name"),
                LayoutPlaceholder {
                    ph_type: Arc::from("pic"),
                    idx: 1,
                    accessibility_name: Arc::from("Profile Photo"),
                    x: LEFT_X,
                    y: BODY_Y,
                    cx: LEFT_CX,
                    cy: LEFT_CX, // square crop
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Biography Text"),
                    x: RIGHT_X,
                    y: BODY_Y,
                    cx: RIGHT_CX,
                    cy: TWO_COL_CY,
                },
            ],
        ),
        // CL-08: SF Team Grid — DSL keyword: team
        custom_layout(
            19,
            "SF Team Grid",
            "team",
            vec![title_ph("title", "Team Title"), body_ph(1, "Team Members")],
        ),
        // CL-09: SF Comparison Table — DSL keyword: enhanced_table
        custom_layout(
            20,
            "SF Comparison Table",
            "enhanced_table",
            vec![
                title_ph("title", "Comparison Title"),
                body_ph(1, "Comparison Table"),
            ],
        ),
        // CL-10: SF Full-Bleed Image — DSL keyword: image
        custom_layout(
            21,
            "SF Full-Bleed Image",
            "image",
            vec![
                LayoutPlaceholder {
                    ph_type: Arc::from("pic"),
                    idx: 1,
                    accessibility_name: Arc::from("Full-Bleed Image"),
                    x: 0,
                    y: 0,
                    cx: 9_144_000,
                    cy: 5_143_500,
                },
                title_ph("title", "Image Caption"),
            ],
        ),
        // CL-11: SF End Slide (DARK LAYOUT) — DSL keyword: end
        dark_layout(
            22,
            "SF End Slide",
            "end",
            vec![
                title_ph("ctrTitle", "Closing Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Closing Message"),
                    x: TITLE_X,
                    y: 1_828_800,
                    cx: TITLE_CX,
                    cy: 1_143_000,
                },
            ],
        ),
        // CL-12: SF Data — DSL keyword: content_stat
        custom_layout(
            23,
            "SF Data",
            "content_stat",
            vec![title_ph("title", "Data Title"), body_ph(1, "Data Content")],
        ),
        // CL-13: SF Diagram — DSL keyword: diagram
        custom_layout(
            24,
            "SF Diagram",
            "diagram",
            vec![
                title_ph("title", "Diagram Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Diagram Area"),
                    x: BODY_X,
                    y: BODY_Y,
                    cx: BODY_CX,
                    cy: BODY_CY,
                },
            ],
        ),
        // CL-14: SF Chart — DSL keyword: chart
        custom_layout(
            25,
            "SF Chart",
            "chart",
            vec![
                title_ph("title", "Chart Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Chart Area"),
                    x: BODY_X,
                    y: BODY_Y,
                    cx: BODY_CX,
                    cy: BODY_CY,
                },
            ],
        ),
        // CL-15: SF Map — DSL keyword: highlight
        custom_layout(
            26,
            "SF Map",
            "highlight",
            vec![
                title_ph("title", "Map Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Map Area"),
                    x: BODY_X,
                    y: BODY_Y,
                    cx: BODY_CX,
                    cy: BODY_CY,
                },
            ],
        ),
        // CL-16: SF Risk Register — DSL keyword: severity_cards
        custom_layout(
            27,
            "SF Risk Register",
            "severity_cards",
            vec![
                title_ph("title", "Risk Register Title"),
                body_ph(1, "Risk Items"),
            ],
        ),
        // CL-17: SF Executive Summary — DSL keyword: highlight_boxes
        custom_layout(
            28,
            "SF Executive Summary",
            "highlight_boxes",
            vec![
                title_ph("title", "Executive Summary Title"),
                body_ph(1, "Summary Content"),
            ],
        ),
        // CL-18: SF Two Column — DSL keyword: stats_summary
        custom_layout(
            29,
            "SF Two Column",
            "stats_summary",
            vec![
                title_ph("title", "Title"),
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 1,
                    accessibility_name: Arc::from("Left Column"),
                    x: LEFT_X,
                    y: TWO_COL_Y,
                    cx: LEFT_CX,
                    cy: TWO_COL_CY,
                },
                LayoutPlaceholder {
                    ph_type: Arc::from("body"),
                    idx: 2,
                    accessibility_name: Arc::from("Right Column"),
                    x: RIGHT_X,
                    y: TWO_COL_Y,
                    cx: RIGHT_CX,
                    cy: TWO_COL_CY,
                },
            ],
        ),
        // CL-19: SF Methodology — DSL keyword: numbered_actions
        custom_layout(
            30,
            "SF Methodology",
            "numbered_actions",
            vec![
                title_ph("title", "Methodology Title"),
                body_ph(1, "Methodology Steps"),
            ],
        ),
        // CL-20: SF Appendix — no dedicated Q2 DSL keyword; uses generic content fallback.
        // This layout slot serves appendix/overflow content. `find_layout_index` in
        // slideforge-pptx falls back to layout index 1 ("Title and Content") for
        // unmapped DSL keywords. If a future story adds an `appendix` keyword, add it here.
        custom_layout(
            31,
            "SF Appendix",
            "appendix",
            vec![
                title_ph("title", "Appendix Title"),
                body_ph(1, "Appendix Content"),
            ],
        ),
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `BrandConfig` directly for layout tests, exercising
    /// `generate_all_layouts()` without filesystem dependencies.
    fn minimal_config_direct() -> BrandConfig {
        BrandConfig {
            colors: crate::toml_schema::ColorConfig {
                acc1: Some("#3B82F6".to_owned()),
                dk1: Some("#1F2937".to_owned()),
                ..Default::default()
            },
            fonts: crate::toml_schema::FontConfig {
                heading: "Calibri".to_owned(),
                body: "Calibri".to_owned(),
            },
            logo: Some(crate::toml_schema::LogoConfig {
                path: "test-logo.png".to_owned(),
            }),
            footer: crate::toml_schema::FooterConfig {
                text: String::new(),
                show_slide_number: true,
                show_date: false,
            },
        }
    }

    // FontConfig::default() and FooterConfig::default() use struct literals above
    // to keep tests self-contained and independent of BrandConfig::default_minimal().

    /// BC-2.01.005 postcondition 1 — exactly 31 layouts generated.
    #[test]
    fn test_bc_2_01_005_generate_31_layouts() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        assert_eq!(layouts.len(), 31, "must generate exactly 31 layouts");
    }

    /// BC-2.01.005 postcondition 2 — SL-01..SL-11 (indices 0–10) have non-None `ooxml_type`.
    #[test]
    fn test_bc_2_01_005_11_standard_layouts_present() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let standard_names = [
            "Title Slide",
            "Title and Content",
            "Section Header",
            "Two Content",
            "Comparison",
            "Title Only",
            "Blank",
            "Content with Caption",
            "Picture with Caption",
            "Vertical Title and Text",
            "Vertical Text",
        ];
        for (i, expected_name) in standard_names.iter().enumerate() {
            assert_eq!(
                layouts[i].name.as_ref(),
                *expected_name,
                "layout index {i} (0-based) must be '{expected_name}'"
            );
        }
    }

    /// BC-2.01.005 postcondition 2 — standard layouts have correct ECMA-376 `ooxml_type` values.
    #[test]
    fn test_bc_2_01_005_standard_layouts_have_ooxml_types() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let expected_types = [
            "title",
            "obj",
            "secHead",
            "twoObj",
            "twoColTx",
            "titleOnly",
            "blank",
            "objTx",
            "picTx",
            "vertTitleAndTx",
            "vertTx",
        ];
        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                layouts[i].ooxml_type.as_deref(),
                Some(*expected_type),
                "layout index {i} must have ooxml_type = Some(\"{expected_type}\")"
            );
        }
    }

    /// BC-2.01.005 postcondition 3 — CL-01..CL-20 (indices 11–30) names start with "SF ".
    #[test]
    fn test_bc_2_01_005_20_sf_custom_layouts_present() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let sf_names = [
            "SF Section Divider",
            "SF Stat Grid",
            "SF Quote",
            "SF Timeline",
            "SF Agenda",
            "SF TOC",
            "SF Bio",
            "SF Team Grid",
            "SF Comparison Table",
            "SF Full-Bleed Image",
            "SF End Slide",
            "SF Data",
            "SF Diagram",
            "SF Chart",
            "SF Map",
            "SF Risk Register",
            "SF Executive Summary",
            "SF Two Column",
            "SF Methodology",
            "SF Appendix",
        ];
        assert_eq!(
            layouts[11..].len(),
            20,
            "must have exactly 20 SF custom layouts"
        );
        for (i, expected_name) in sf_names.iter().enumerate() {
            let layout_idx = i + 11;
            assert_eq!(
                layouts[layout_idx].name.as_ref(),
                *expected_name,
                "layout index {layout_idx} must be '{expected_name}'"
            );
            assert!(
                layouts[layout_idx].name.starts_with("SF "),
                "layout index {layout_idx} name must start with 'SF '"
            );
        }
    }

    /// BC-2.01.005 invariant 2 / AC-010 — Blank layout (SL-07, index 6) has zero placeholders.
    #[test]
    fn test_bc_2_01_005_blank_layout_has_zero_placeholders() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let blank = &layouts[6]; // SL-07 (0-based index 6)
        assert_eq!(blank.name.as_ref(), "Blank", "index 6 must be Blank layout");
        assert_eq!(
            blank.placeholders.len(),
            0,
            "Blank layout must have zero placeholders"
        );
    }

    /// BC-2.01.005 / AC-009 — CL-01 "SF Section Divider" (index 11) has dark color override.
    #[test]
    fn test_bc_2_01_005_cl01_section_divider_dark_layout() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl01 = &layouts[11]; // CL-01 (0-based index 11)
        assert_eq!(
            cl01.name.as_ref(),
            "SF Section Divider",
            "index 11 must be SF Section Divider"
        );
        assert!(
            cl01.has_color_override,
            "SF Section Divider must have has_color_override = true"
        );
        assert_eq!(
            cl01.color_override_bg.as_deref(),
            Some("dk2"),
            "SF Section Divider bg override must be 'dk2'"
        );
        assert_eq!(
            cl01.color_override_tx.as_deref(),
            Some("lt1"),
            "SF Section Divider tx override must be 'lt1'"
        );
    }

    /// BC-2.01.005 / AC-009 — CL-11 "SF End Slide" (index 21) has dark color override.
    #[test]
    fn test_bc_2_01_005_cl11_end_slide_dark_layout() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl11 = &layouts[21]; // CL-11 (0-based index 21)
        assert_eq!(
            cl11.name.as_ref(),
            "SF End Slide",
            "index 21 must be SF End Slide"
        );
        assert!(
            cl11.has_color_override,
            "SF End Slide must have has_color_override = true"
        );
        assert_eq!(
            cl11.color_override_bg.as_deref(),
            Some("dk2"),
            "SF End Slide bg override must be 'dk2'"
        );
        assert_eq!(
            cl11.color_override_tx.as_deref(),
            Some("lt1"),
            "SF End Slide tx override must be 'lt1'"
        );
    }

    /// BC-2.01.005 — layout indices are 1-based and sequential (1..=31).
    #[test]
    fn test_bc_2_01_005_layout_indices_are_sequential() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        assert_eq!(layouts[0].index, 1, "first layout must have index 1");
        assert_eq!(layouts[30].index, 31, "last layout must have index 31");
        for (i, layout) in layouts.iter().enumerate() {
            assert_eq!(
                layout.index,
                i + 1,
                "layout at position {i} must have 1-based index {}",
                i + 1
            );
        }
    }

    /// BC-2.01.005 — non-dark layouts do NOT have color overrides.
    #[test]
    fn test_bc_2_01_005_non_dark_layouts_have_no_color_override() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            // Only indices 11 (CL-01) and 21 (CL-11) should be dark
            if i != 11 && i != 21 {
                assert!(
                    !layout.has_color_override,
                    "layout index {i} ('{}') must NOT have has_color_override = true",
                    layout.name
                );
            }
        }
    }

    /// BC-2.01.005 invariant 2 / AC-010 — every layout except Blank (SL-07) has at
    /// least one placeholder with `ph_type` == `"title"` or `"ctrTitle"`.
    ///
    /// This test replaces `test_bc_2_01_005_non_blank_layouts_have_at_least_one_placeholder`
    /// and adds the stronger constraint that a title-type placeholder must be present.
    #[test]
    fn test_bc_2_01_005_non_blank_layouts_have_title_placeholder() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            if i == 6 {
                // Blank layout (SL-07) — zero placeholders is correct
                continue;
            }
            assert!(
                !layout.placeholders.is_empty(),
                "layout index {i} ('{}') must have at least one placeholder",
                layout.name
            );
            let has_title = layout
                .placeholders
                .iter()
                .any(|ph| ph.ph_type.as_ref() == "title" || ph.ph_type.as_ref() == "ctrTitle");
            assert!(
                has_title,
                "layout index {i} ('{}') must have at least one placeholder with ph_type == 'title' or 'ctrTitle' (AC-010)",
                layout.name
            );
        }
    }

    /// BC-2.01.005 invariant 4 / AC-010 — no placeholder uses "Shape N" as its
    /// accessibility name (DI-001 accessibility requirement).
    #[test]
    fn test_bc_2_01_005_layout_placeholder_names_are_semantic() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for (i, layout) in layouts.iter().enumerate() {
            for ph in &layout.placeholders {
                let name = ph.accessibility_name.as_ref();
                // Must not match the forbidden "Shape N" pattern
                let is_shape_n =
                    name.starts_with("Shape ") && name[6..].chars().all(|c| c.is_ascii_digit());
                assert!(
                    !is_shape_n,
                    "layout index {i} ('{}') has a placeholder with forbidden name '{}' (must not be 'Shape N')",
                    layout.name, name
                );
                // Accessibility name must be non-empty
                assert!(
                    !name.is_empty(),
                    "layout index {i} ('{}') has a placeholder with empty accessibility_name",
                    layout.name
                );
            }
        }
    }

    /// BC-2.01.005 — generation is deterministic (same config → same Vec ordering).
    #[test]
    fn test_bc_2_01_005_generation_deterministic() {
        let config = minimal_config_direct();
        let layouts_a = generate_all_layouts(&config);
        let layouts_b = generate_all_layouts(&config);
        assert_eq!(
            layouts_a.len(),
            layouts_b.len(),
            "generate_all_layouts must be deterministic in length"
        );
        for (i, (a, b)) in layouts_a.iter().zip(layouts_b.iter()).enumerate() {
            assert_eq!(
                a.name.as_ref(),
                b.name.as_ref(),
                "layout index {i}: name must be deterministic"
            );
            assert_eq!(
                a.index, b.index,
                "layout index {i}: index field must be deterministic"
            );
            assert_eq!(
                a.ooxml_type.as_deref(),
                b.ooxml_type.as_deref(),
                "layout index {i}: ooxml_type must be deterministic"
            );
            assert_eq!(
                a.has_color_override, b.has_color_override,
                "layout index {i}: has_color_override must be deterministic"
            );
        }
    }

    // ─── ADR-015 §A.4 — slide_type_keyword field tests ───────────────────────

    /// ADR-015 §A.4 — standard layouts (SL-01..SL-11) have `slide_type_keyword = None`.
    ///
    /// Standard layouts are looked up by `ooxml_type`, not by DSL keyword.
    #[test]
    fn test_adr015_standard_layouts_have_none_slide_type_keyword() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for i in 0..11 {
            assert_eq!(
                layouts[i].slide_type_keyword,
                None,
                "standard layout index {i} ('{}') must have slide_type_keyword = None \
                 (ADR-015 §A.4: standard layouts matched by ooxml_type, not keyword)",
                layouts[i].name
            );
        }
    }

    /// ADR-015 §A.4 — CL-01 "SF Section Divider" (0-based index 11) has
    /// `slide_type_keyword = Some("section_divider")`.
    #[test]
    fn test_adr015_section_divider_has_correct_keyword() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl01 = &layouts[11]; // CL-01 (0-based index 11, 1-based index 12)
        assert_eq!(
            cl01.slide_type_keyword.as_deref(),
            Some("section_divider"),
            "SF Section Divider must have slide_type_keyword = Some(\"section_divider\") \
             (ADR-015 §A.4 canonical mapping table)"
        );
    }

    /// ADR-015 §A.4 — CL-11 "SF End Slide" (0-based index 21) has
    /// `slide_type_keyword = Some("end")`.
    #[test]
    fn test_adr015_end_slide_has_correct_keyword() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        let cl11 = &layouts[21]; // CL-11 (0-based index 21, 1-based index 22)
        assert_eq!(
            cl11.slide_type_keyword.as_deref(),
            Some("end"),
            "SF End Slide must have slide_type_keyword = Some(\"end\") \
             (ADR-015 §A.4 canonical mapping table)"
        );
    }

    /// ADR-015 §A.4 — all 20 SF custom layouts (0-based indices 11..=30) have
    /// `slide_type_keyword = Some(...)` (non-None).
    #[test]
    fn test_adr015_all_custom_layouts_have_keyword() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);
        for i in 11..31 {
            assert!(
                layouts[i].slide_type_keyword.is_some(),
                "custom layout index {i} ('{}') must have slide_type_keyword = Some(...) \
                 (ADR-015 §A.4: all SF custom layouts require a DSL keyword)",
                layouts[i].name
            );
        }
    }

    /// ADR-015 §A.4 — canonical mapping spot-checks for 5 custom layouts.
    ///
    /// Verifies the full §A.4 table for the most critical entries.
    #[test]
    fn test_adr015_canonical_keyword_mapping_spot_checks() {
        let config = minimal_config_direct();
        let layouts = generate_all_layouts(&config);

        // (0-based index, expected keyword, layout name)
        let spot_checks: &[(usize, &str, &str)] = &[
            (11, "section_divider", "SF Section Divider"),
            (12, "stat_callout", "SF Stat Grid"),
            (13, "quote", "SF Quote"),
            (20, "image", "SF Full-Bleed Image"),
            (21, "end", "SF End Slide"),
            (22, "content_stat", "SF Data"),
            (23, "diagram", "SF Diagram"),
            (24, "chart", "SF Chart"),
            (25, "highlight", "SF Map"),
            (26, "severity_cards", "SF Risk Register"),
            (27, "highlight_boxes", "SF Executive Summary"),
            (28, "stats_summary", "SF Two Column"),
            (29, "numbered_actions", "SF Methodology"),
            (30, "appendix", "SF Appendix"),
        ];
        for &(idx, expected_keyword, expected_name) in spot_checks {
            assert_eq!(
                layouts[idx].name.as_ref(),
                expected_name,
                "layout[{idx}] name mismatch"
            );
            assert_eq!(
                layouts[idx].slide_type_keyword.as_deref(),
                Some(expected_keyword),
                "layout[{idx}] ('{}') slide_type_keyword must be Some(\"{expected_keyword}\") \
                 (ADR-015 §A.4 canonical mapping table)",
                layouts[idx].name
            );
        }
    }
}
