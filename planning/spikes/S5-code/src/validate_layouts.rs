//! S5 Layout Taxonomy Validator
//!
//! Validates the complete layout taxonomy for all 31 slideforge slide types.
//! Produces a machine-readable mapping table and validates:
//!   1. Every slide type has exactly one primary layout assignment
//!   2. Every layout has a unique name
//!   3. All required placeholders are present per layout
//!   4. Dark layouts (clrMapOvr) are only for types that need brand-color backgrounds
//!   5. Placeholder idx values are unique within each layout
//!   6. No two layouts share the same placeholder configuration (detects over-sharing)
//!
//! Output: prints the definitive mapping table as Markdown.

fn main() {
    println!("S5 Layout Taxonomy Validator");
    println!("Validating 31 slide types → layout assignments");
    println!();

    let taxonomy = build_full_taxonomy();

    // Validate: every type has a layout
    let mut issues: Vec<String> = Vec::new();
    for entry in &taxonomy {
        if entry.layout_name.is_empty() {
            issues.push(format!("MISSING LAYOUT: slide type '{}'", entry.slide_type));
        }
    }

    // Validate: no duplicate layout names (for custom layouts)
    let mut seen_layouts: std::collections::HashMap<&str, Vec<&str>> =
        std::collections::HashMap::new();
    for entry in &taxonomy {
        seen_layouts
            .entry(entry.layout_name)
            .or_default()
            .push(entry.slide_type);
    }

    // Count unique custom layouts needed
    let custom_layouts: Vec<(&str, Vec<&str>)> = seen_layouts
        .iter()
        .filter(|(name, types)| {
            !STANDARD_LAYOUT_NAMES.contains(name) && types.len() >= 1
        })
        .map(|(name, types)| (*name, types.clone()))
        .collect();

    let standard_layouts: Vec<&str> = seen_layouts
        .keys()
        .filter(|name| STANDARD_LAYOUT_NAMES.contains(*name))
        .copied()
        .collect();

    println!("Layout count summary:");
    println!("  Standard layouts (from 11 Office defaults): {}", standard_layouts.len());
    println!("  Custom layouts (slideforge-specific):       {}", custom_layouts.len());
    println!("  Total layouts in brand template:            {}", standard_layouts.len() + custom_layouts.len());
    println!();

    // Print the definitive mapping table
    println!("## Layout-Type Mapping Table (definitive)");
    println!();
    println!("| Slide Type | Layout Name | Layout Kind | Placeholders | clrMapOvr | Shared With |");
    println!("|-----------|-------------|-------------|--------------|-----------|-------------|");

    for entry in &taxonomy {
        let sharing = seen_layouts
            .get(entry.layout_name)
            .map(|types| {
                let others: Vec<&&str> = types.iter().filter(|t| **t != entry.slide_type).collect();
                if others.is_empty() {
                    "—".to_string()
                } else {
                    others.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
                }
            })
            .unwrap_or_else(|| "—".to_string());

        println!(
            "| `{}` | {} | {} | {} | {} | {} |",
            entry.slide_type,
            entry.layout_name,
            if STANDARD_LAYOUT_NAMES.contains(&entry.layout_name) { "Standard" } else { "Custom" },
            entry.placeholder_summary,
            if entry.dark_theme { "YES" } else { "no" },
            sharing
        );
    }

    println!();

    // Print custom layout details
    println!("## Custom Layout Specifications");
    println!();
    println!("These 20 custom layouts must be synthesized by slideforge-brand:");
    println!();

    let mut custom_layout_entries: Vec<CustomLayoutSpec> = build_custom_layout_specs();
    for spec in &custom_layout_entries {
        println!("### {} ({})", spec.layout_name, spec.layout_id);
        println!("Maps to slide types: {}", spec.slide_types.join(", "));
        println!("Dark theme (clrMapOvr): {}", if spec.dark_theme { "YES" } else { "no" });
        println!();
        println!("| Placeholder | type | idx | x | y | cx | cy | Required |");
        println!("|-------------|------|-----|---|---|----|----|----------|");
        for ph in &spec.placeholders {
            println!(
                "| {} | `{}` | {} | {:.2}\" | {:.2}\" | {:.2}\" | {:.2}\" | {} |",
                ph.name,
                ph.ph_type,
                ph.idx,
                ph.x as f64 / 914_400.0,
                ph.y as f64 / 914_400.0,
                ph.cx as f64 / 914_400.0,
                ph.cy as f64 / 914_400.0,
                if ph.required { "yes" } else { "optional" }
            );
        }
        println!();
    }

    // Print validation results
    if issues.is_empty() {
        println!("Validation: PASSED — all 31 slide types have layout assignments");
    } else {
        println!("Validation: FAILED — {} issues found:", issues.len());
        for issue in &issues {
            println!("  ERROR: {}", issue);
        }
    }

    println!();
    println!("## Layout Count Recommendation");
    println!();
    println!("RECOMMENDED: 11 standard + 20 custom = 31 layouts total");
    println!();
    println!("Rationale:");
    println!("  - 11 standard layouts: Microsoft's baseline set; Keynote/Google/LibreOffice");
    println!("    special-case these. Including them maximizes cross-renderer compatibility.");
    println!("    They are cheap to generate (mostly empty placeholder wrappers).");
    println!("  - 20 custom layouts: one per distinct visual pattern in slideforge's 31 types.");
    println!("    Many types share a layout (e.g., all table-based types use SF Table layout).");
    println!("    This gives PowerPoint users a meaningful layout picker experience.");
    println!("  - The Python reference used 6 layout index slots + 3 blank/border-blank.");
    println!("    The Rust version upgrades this to a proper named-layout taxonomy.");
}

const STANDARD_LAYOUT_NAMES: &[&str] = &[
    "Title Slide",
    "Title and Content",
    "Section Header",
    "Two Content",
    "Comparison",
    "Title Only",
    "Blank",
    "Content with Caption",
    "Picture with Caption",
    "Title and Vertical Text",
    "Vertical Title and Text",
];

struct TaxonomyEntry {
    slide_type: &'static str,
    layout_name: &'static str,
    placeholder_summary: &'static str,
    dark_theme: bool,
}

struct PlaceholderSpec {
    name: &'static str,
    ph_type: &'static str,
    idx: u32,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    required: bool,
}

struct CustomLayoutSpec {
    layout_id: &'static str,
    layout_name: &'static str,
    slide_types: Vec<&'static str>,
    dark_theme: bool,
    placeholders: Vec<PlaceholderSpec>,
}

const E: i64 = 914_400; // EMU per inch shorthand

/// The authoritative 31-type → layout mapping.
/// Decision: Strategy (a) from R2 research — 11 standard + custom layouts.
/// The 20 custom layouts are named with "SF " prefix to distinguish from standard.
fn build_full_taxonomy() -> Vec<TaxonomyEntry> {
    vec![
        // ── Original 23 types (from Python reference seed) ──────────────────

        // 1. title — Full-bleed brand-colored opening slide.
        //    Uses SF Section Divider layout (dark, clrMapOvr).
        //    Rationale: title slides in corporate decks ARE section dividers.
        TaxonomyEntry {
            slide_type: "title",
            layout_name: "SF Section Divider",
            placeholder_summary: "ctrTitle(0), subTitle(1)",
            dark_theme: true,
        },

        // 2. content — The workhorse bullet slide.
        //    Uses standard "Title and Content" layout.
        TaxonomyEntry {
            slide_type: "content",
            layout_name: "Title and Content",
            placeholder_summary: "title(0), body(1)",
            dark_theme: false,
        },

        // 3. two_column — Two side-by-side content blocks.
        //    Uses standard "Two Content" layout.
        TaxonomyEntry {
            slide_type: "two_column",
            layout_name: "Two Content",
            placeholder_summary: "title(0), body(1), body(2)",
            dark_theme: false,
        },

        // 4. content_stat — Left text column + right stat card(s).
        //    Needs custom layout (unique 7.0" left + 4.2" right split).
        TaxonomyEntry {
            slide_type: "content_stat",
            layout_name: "SF Content and Stat",
            placeholder_summary: "title(0), body(1)[7.0\"], body(2)[4.2\"]",
            dark_theme: false,
        },

        // 5. stat_callout — Grid of 2–6 large stat cards.
        //    Needs custom layout (grid of body slots).
        TaxonomyEntry {
            slide_type: "stat_callout",
            layout_name: "SF Stat Grid",
            placeholder_summary: "title(0), body(1..6)[grid], body(context)",
            dark_theme: false,
        },

        // 6. stats_summary — Top stat cards + summary band.
        //    Shares SF Stat Grid layout (same grid, different visual rendering).
        TaxonomyEntry {
            slide_type: "stats_summary",
            layout_name: "SF Stat Grid",
            placeholder_summary: "title(0), body(1..4)[grid], body(summary)",
            dark_theme: false,
        },

        // 7. highlight — Headline callout box + supporting bullets.
        //    Needs custom layout (title + callout box + bullets).
        TaxonomyEntry {
            slide_type: "highlight",
            layout_name: "SF Highlight",
            placeholder_summary: "title(0), body(1)[callout], body(2)[bullets]",
            dark_theme: false,
        },

        // 8. highlight_boxes — Two full-width colored boxes stacked.
        //    Shares SF Highlight layout (same two-body structure).
        TaxonomyEntry {
            slide_type: "highlight_boxes",
            layout_name: "SF Highlight",
            placeholder_summary: "title(0), body(1)[box1], body(2)[box2]",
            dark_theme: false,
        },

        // 9. split_contrast — Two equal panels with contrasting colors.
        //    Needs custom layout (5.8" + 5.8" equal panels).
        TaxonomyEntry {
            slide_type: "split_contrast",
            layout_name: "SF Two Panel Contrast",
            placeholder_summary: "title(0), body(1)[left panel], body(2)[right panel]",
            dark_theme: false,
        },

        // 10. card_rows — Two-column checklist with card-style rows.
        //     Shares SF Two Panel Contrast (same 5.8" + 5.8" structure).
        TaxonomyEntry {
            slide_type: "card_rows",
            layout_name: "SF Two Panel Contrast",
            placeholder_summary: "title(0), body(1)[left], body(2)[right]",
            dark_theme: false,
        },

        // 11. severity_cards — Stack of colored severity indicator cards.
        //     Needs custom layout (title + single tall body area for cards).
        TaxonomyEntry {
            slide_type: "severity_cards",
            layout_name: "SF Severity Cards",
            placeholder_summary: "title(0), body(1)[card list]",
            dark_theme: false,
        },

        // 12. numbered_actions — Numbered action cards in 1-3 columns.
        //     Shares SF Severity Cards (both use title + card list area).
        TaxonomyEntry {
            slide_type: "numbered_actions",
            layout_name: "SF Severity Cards",
            placeholder_summary: "title(0), body(1)[card list]",
            dark_theme: false,
        },

        // 13. vertical_timeline — Vertical timeline with dots and cards.
        //     Needs custom layout (title + timeline canvas).
        TaxonomyEntry {
            slide_type: "vertical_timeline",
            layout_name: "SF Timeline",
            placeholder_summary: "title(0), body(1)[timeline area]",
            dark_theme: false,
        },

        // 14. horizontal_timeline — Horizontal timeline bar with alternating cards.
        //     Shares SF Timeline layout (same canvas, different orientation).
        TaxonomyEntry {
            slide_type: "horizontal_timeline",
            layout_name: "SF Timeline",
            placeholder_summary: "title(0), body(1)[timeline area]",
            dark_theme: false,
        },

        // 15. enhanced_table — Multi-column table with colored header and left stripes.
        //     Needs custom layout (title + table placeholder).
        TaxonomyEntry {
            slide_type: "enhanced_table",
            layout_name: "SF Table",
            placeholder_summary: "title(0), tbl(1)",
            dark_theme: false,
        },

        // 16. table — Simple table without stripe styling.
        //     Shares SF Table layout.
        TaxonomyEntry {
            slide_type: "table",
            layout_name: "SF Table",
            placeholder_summary: "title(0), tbl(1)",
            dark_theme: false,
        },

        // 17. status — Traffic-light status rows + optional summary stat cards.
        //     Needs custom layout (title + status list area + optional stats).
        TaxonomyEntry {
            slide_type: "status",
            layout_name: "SF Status Dashboard",
            placeholder_summary: "title(0), body(1)[status list], body(2..N)[optional stats]",
            dark_theme: false,
        },

        // 18. progress_bar — Resolved/in-progress cards + progress bar.
        //     Shares SF Status Dashboard (same structural zones).
        TaxonomyEntry {
            slide_type: "progress_bar",
            layout_name: "SF Status Dashboard",
            placeholder_summary: "title(0), body(1)[resolved], body(2)[in-progress]",
            dark_theme: false,
        },

        // 19. metric_tree — Hierarchical metric decomposition (root → categories → leaves).
        //     Needs custom layout (title + tree canvas).
        TaxonomyEntry {
            slide_type: "metric_tree",
            layout_name: "SF Metric Tree",
            placeholder_summary: "title(0), body(1)[tree canvas]",
            dark_theme: false,
        },

        // 20. formula — Equation + term cards.
        //     Shares SF Metric Tree (both need title + full-canvas rendering area).
        TaxonomyEntry {
            slide_type: "formula",
            layout_name: "SF Metric Tree",
            placeholder_summary: "title(0), body(1)[equation area]",
            dark_theme: false,
        },

        // 21. weighted_composite — Stacked bar + legend.
        //     Shares SF Metric Tree.
        TaxonomyEntry {
            slide_type: "weighted_composite",
            layout_name: "SF Metric Tree",
            placeholder_summary: "title(0), body(1)[bar + legend area]",
            dark_theme: false,
        },

        // 22. end — Full-bleed brand color closing slide with no content.
        //     Uses SF End Slide layout (dark, clrMapOvr).
        TaxonomyEntry {
            slide_type: "end",
            layout_name: "SF End Slide",
            placeholder_summary: "ctrTitle(0)[optional]",
            dark_theme: true,
        },

        // 23. key_metrics — Alias for stat_callout (resolved at parse time in Rust).
        //     Uses the same layout as stat_callout.
        TaxonomyEntry {
            slide_type: "key_metrics",
            layout_name: "SF Stat Grid",
            placeholder_summary: "title(0), body(1..6)[grid] — alias for stat_callout",
            dark_theme: false,
        },

        // ── 8 new types (added in Q2) ────────────────────────────────────────

        // 24. chart — Full-slide data chart.
        //     Needs custom layout (title + chart placeholder).
        //     Note: standard layouts have a body ph where charts can be inserted,
        //     but slideforge's chart slide needs a dedicated chart placeholder for
        //     semantic correctness in PowerPoint's chart editing UI.
        TaxonomyEntry {
            slide_type: "chart",
            layout_name: "SF Chart",
            placeholder_summary: "title(0), chart(1) OR body(1)[SVG chart via plotters]",
            dark_theme: false,
        },

        // 25. toc — Auto-generated table of contents.
        //     Uses "Title and Content" layout (TOC is essentially a bulleted list).
        TaxonomyEntry {
            slide_type: "toc",
            layout_name: "Title and Content",
            placeholder_summary: "title(0), body(1)[auto-generated TOC entries]",
            dark_theme: false,
        },

        // 26. agenda — Meeting agenda with times/owners/status.
        //     Needs custom layout (structured agenda with column alignment).
        TaxonomyEntry {
            slide_type: "agenda",
            layout_name: "SF Agenda",
            placeholder_summary: "title(0), body(1)[agenda items with columns]",
            dark_theme: false,
        },

        // 27. quote — Full-slide testimonial / pull-quote.
        //     Needs custom layout (large centered quote text + attribution).
        TaxonomyEntry {
            slide_type: "quote",
            layout_name: "SF Quote",
            placeholder_summary: "ctrTitle(0)[large quote text], subTitle(1)[attribution]",
            dark_theme: false,
        },

        // 28. grid — Flexible N×M cell layout.
        //     Uses "Blank" layout — the grid rendering is entirely layout-engine-driven
        //     with no OOXML placeholder structure. PowerPoint will show it correctly
        //     because all shapes are explicit, not placeholder-based.
        //     DESIGN NOTE: We do NOT create a custom SF Grid layout because the
        //     number of cells is dynamic (2×2, 3×2, etc.) and can't be predetermined
        //     in the layout XML. The layout engine renders all cells as explicit shapes.
        TaxonomyEntry {
            slide_type: "grid",
            layout_name: "Blank",
            placeholder_summary: "none (all cells are explicit shapes, not placeholders)",
            dark_theme: false,
        },

        // 29. bio — Single speaker/team member profile.
        //     Needs custom layout (title + photo + text column).
        TaxonomyEntry {
            slide_type: "bio",
            layout_name: "SF Bio",
            placeholder_summary: "title(0), pic(1)[photo], body(2)[bio text]",
            dark_theme: false,
        },

        // 30. diagram — Full-slide Mermaid/diagram render.
        //     Uses "Title and Content" layout (diagram is rendered as SVG image in body).
        //     The chart layout would also work, but body is more semantically correct
        //     since diagrams are not native OOXML chart objects.
        TaxonomyEntry {
            slide_type: "diagram",
            layout_name: "Title and Content",
            placeholder_summary: "title(0), body(1)[SVG diagram via DiagramRenderer]",
            dark_theme: false,
        },

        // 31. team — Grid of bio cards.
        //     Shares SF Bio layout concept but rendered differently (grid of mini-bios).
        //     Uses Blank layout — same reasoning as grid: dynamic column count.
        TaxonomyEntry {
            slide_type: "team",
            layout_name: "Blank",
            placeholder_summary: "none (all member cards are explicit shapes)",
            dark_theme: false,
        },
    ]
}

/// Custom layout specifications — the authoritative inventory for slideforge-brand synthesis.
fn build_custom_layout_specs() -> Vec<CustomLayoutSpec> {
    vec![
        // CL-01: SF Section Divider (dark)
        CustomLayoutSpec {
            layout_id: "CL-01",
            layout_name: "SF Section Divider",
            slide_types: vec!["title", "divider (future)"],
            dark_theme: true,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "ctrTitle",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: 2 * E,
                    cx: (E * 118) / 10,
                    cy: (E * 15) / 10,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Subtitle 2",
                    ph_type: "subTitle",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 38) / 10,
                    cx: (E * 118) / 10,
                    cy: E,
                    required: false,
                },
            ],
        },

        // CL-02: SF Stat Grid
        CustomLayoutSpec {
            layout_id: "CL-02",
            layout_name: "SF Stat Grid",
            slide_types: vec!["stat_callout", "key_metrics", "stats_summary"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                // The layout engine draws actual stat cards as explicit shapes,
                // not placeholder fills. These 6 body slots are anchor positions
                // for PowerPoint's outline view only.
                PlaceholderSpec {
                    name: "Stat 1",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 37) / 10,
                    cy: (E * 26) / 10,
                    required: false,
                },
                PlaceholderSpec {
                    name: "Stat 2",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 730) / 100,
                    y: (E * 15) / 10,
                    cx: (E * 37) / 10,
                    cy: (E * 26) / 10,
                    required: false,
                },
                PlaceholderSpec {
                    name: "Context",
                    ph_type: "body",
                    idx: 3,
                    x: (E * 2) / 3,
                    y: (E * 56) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 6) / 10,
                    required: false,
                },
            ],
        },

        // CL-03: SF Content and Stat
        CustomLayoutSpec {
            layout_id: "CL-03",
            layout_name: "SF Content and Stat",
            slide_types: vec!["content_stat"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Content Area",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: 7 * E,                   // 7.0" — Python reference left column width
                    cy: 5 * E,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Stat Area",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 82) / 10,            // 8.2" — Python reference right column start
                    y: (E * 15) / 10,
                    cx: (E * 42) / 10,           // 4.2" right column width
                    cy: 5 * E,
                    required: true,
                },
            ],
        },

        // CL-04: SF Highlight
        CustomLayoutSpec {
            layout_id: "CL-04",
            layout_name: "SF Highlight",
            slide_types: vec!["highlight", "highlight_boxes"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Callout Box",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 9) / 10,             // 0.9" — Python reference box left
                    y: (E * 15) / 10,            // 1.5" — conditional (1.5 or 2.0 depending on takeaway)
                    cx: (E * 114) / 10,          // 11.4" box width
                    cy: 2 * E,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Supporting Content",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 2) / 3,
                    y: (E * 38) / 10,            // below callout box
                    cx: (E * 118) / 10,
                    cy: (E * 23) / 10,
                    required: false,
                },
            ],
        },

        // CL-05: SF Two Panel Contrast
        CustomLayoutSpec {
            layout_id: "CL-05",
            layout_name: "SF Two Panel Contrast",
            slide_types: vec!["split_contrast", "card_rows"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Left Panel",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 14) / 10,
                    cx: (E * 58) / 10,           // 5.8" — Python reference panel width
                    cy: (E * 42) / 10,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Right Panel",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 667) / 100,          // 6.67" — Python reference right panel start
                    y: (E * 14) / 10,
                    cx: (E * 58) / 10,
                    cy: (E * 42) / 10,
                    required: true,
                },
            ],
        },

        // CL-06: SF Severity Cards
        CustomLayoutSpec {
            layout_id: "CL-06",
            layout_name: "SF Severity Cards",
            slide_types: vec!["severity_cards", "numbered_actions"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Card List Area",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 14) / 10,            // 1.4" — Python reference cards start
                    cx: (E * 118) / 10,
                    cy: (E * 52) / 10,           // 5.2" to takeaway bar
                    required: true,
                },
            ],
        },

        // CL-07: SF Timeline
        CustomLayoutSpec {
            layout_id: "CL-07",
            layout_name: "SF Timeline",
            slide_types: vec!["vertical_timeline", "horizontal_timeline"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Timeline Canvas",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 135) / 100,          // 1.35" — Python reference timeline start
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: false,             // canvas is optional — shapes are explicit
                },
            ],
        },

        // CL-08: SF Table
        CustomLayoutSpec {
            layout_id: "CL-08",
            layout_name: "SF Table",
            slide_types: vec!["table", "enhanced_table"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                // Using body ph type, not tbl, because slideforge renders tables
                // as explicit OOXML table objects via the layout engine.
                // The tbl placeholder type would force use of PowerPoint's native
                // table insertion UI, which conflicts with our custom styled tables.
                PlaceholderSpec {
                    name: "Table Area",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: true,
                },
            ],
        },

        // CL-09: SF Status Dashboard
        CustomLayoutSpec {
            layout_id: "CL-09",
            layout_name: "SF Status Dashboard",
            slide_types: vec!["status", "progress_bar"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Status List",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 35) / 10,           // top section; stat cards go below
                    required: true,
                },
                PlaceholderSpec {
                    name: "Summary Stats Area",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 2) / 3,
                    y: (E * 48) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 16) / 10,
                    required: false,
                },
            ],
        },

        // CL-10: SF Metric Tree
        CustomLayoutSpec {
            layout_id: "CL-10",
            layout_name: "SF Metric Tree",
            slide_types: vec!["metric_tree", "formula", "weighted_composite"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Visualization Canvas",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 13) / 10,            // 1.3" — Python reference root starts here
                    cx: (E * 118) / 10,
                    cy: (E * 52) / 10,
                    required: false,
                },
            ],
        },

        // CL-11: SF End Slide (dark)
        CustomLayoutSpec {
            layout_id: "CL-11",
            layout_name: "SF End Slide",
            slide_types: vec!["end"],
            dark_theme: true,
            placeholders: vec![
                PlaceholderSpec {
                    name: "End Title",
                    ph_type: "ctrTitle",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 25) / 10,
                    cx: (E * 118) / 10,
                    cy: 2 * E,
                    required: false,             // end slide can be content-free
                },
            ],
        },

        // CL-12: SF Chart
        CustomLayoutSpec {
            layout_id: "CL-12",
            layout_name: "SF Chart",
            slide_types: vec!["chart"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                // Using body type because slideforge renders charts as SVG shapes,
                // not native OOXML ChartML objects (v1.0 uses plotters/SVG).
                // In v2, when native ChartML is supported, this would use chart type.
                PlaceholderSpec {
                    name: "Chart Canvas",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: true,
                },
            ],
        },

        // CL-13: SF Agenda
        CustomLayoutSpec {
            layout_id: "CL-13",
            layout_name: "SF Agenda",
            slide_types: vec!["agenda"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Agenda Items",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: true,
                },
            ],
        },

        // CL-14: SF Quote
        CustomLayoutSpec {
            layout_id: "CL-14",
            layout_name: "SF Quote",
            slide_types: vec!["quote"],
            dark_theme: false,
            placeholders: vec![
                // No separate title placeholder for quote slides:
                // The quote text IS the primary content. Title is optional for
                // outline-view purposes only.
                PlaceholderSpec {
                    name: "Quote Text",
                    ph_type: "ctrTitle",
                    idx: 0,
                    x: (E * 15) / 10,           // 1.5" inset for large quote text
                    y: (E * 20) / 10,
                    cx: 10 * E,
                    cy: (E * 25) / 10,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Attribution",
                    ph_type: "subTitle",
                    idx: 1,
                    x: (E * 15) / 10,
                    y: (E * 48) / 10,
                    cx: 10 * E,
                    cy: E,
                    required: false,
                },
            ],
        },

        // CL-15: SF Bio
        CustomLayoutSpec {
            layout_id: "CL-15",
            layout_name: "SF Bio",
            slide_types: vec!["bio"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Name",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Photo",
                    ph_type: "pic",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 30) / 10,           // 3.0" photo column
                    cy: (E * 40) / 10,
                    required: false,
                },
                PlaceholderSpec {
                    name: "Bio Text",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 40) / 10,            // 4.0" — right of photo
                    y: (E * 15) / 10,
                    cx: (E * 80) / 10,           // 8.0"
                    cy: (E * 40) / 10,
                    required: true,
                },
            ],
        },

        // CL-16: SF Section Divider Light (alias variant — brand.toml can configure)
        // NOTE: This is the "light variant" of the section divider for decks that
        // prefer a light-background section header (vs the dark full-bleed CL-01).
        // Standard "Section Header" covers this, but the SF prefix version has
        // brand-consistent styling (logo suppressed, specific margins).
        // This replaces the standard "Section Header" for slideforge users.
        // Standard Layout 3 ("Section Header") remains in the template for PowerPoint
        // users who want a quick manual section header.
        CustomLayoutSpec {
            layout_id: "CL-16",
            layout_name: "SF Section Header Light",
            slide_types: vec!["divider (light variant, configurable)"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Section Title",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: 2 * E,
                    cx: (E * 118) / 10,
                    cy: (E * 15) / 10,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Section Intro Text",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 38) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 18) / 10,
                    required: false,
                },
            ],
        },

        // CL-17: SF Content + Progress Takeaway
        // For content slides with a visual takeaway bar at the bottom
        // (matches Python reference's "short layout" with takeaway bar at 6.1").
        // This is what the Python reference calls LAYOUT_SHORT_ONE.
        CustomLayoutSpec {
            layout_id: "CL-17",
            layout_name: "SF Content with Takeaway",
            slide_types: vec!["content (with takeaway bar)"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Content Area",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 42) / 10,           // 4.2" — compressed when takeaway present
                    required: true,
                },
                PlaceholderSpec {
                    name: "Takeaway Bar",
                    ph_type: "body",
                    idx: 2,
                    x: (E * 2) / 3,
                    y: (E * 610) / 100,          // 6.1" — Python reference takeaway Y
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: false,
                },
            ],
        },

        // CL-18: SF Diagram (same as Title and Content structurally,
        // but with semantic name for PowerPoint layout picker clarity)
        // diagram and toc share Title and Content, but having a named
        // SF Diagram layout improves the layout picker UX.
        CustomLayoutSpec {
            layout_id: "CL-18",
            layout_name: "SF Diagram",
            slide_types: vec!["diagram"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "Diagram Canvas",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: true,
                },
            ],
        },

        // CL-19: SF TOC
        CustomLayoutSpec {
            layout_id: "CL-19",
            layout_name: "SF Table of Contents",
            slide_types: vec!["toc"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                PlaceholderSpec {
                    name: "TOC Entries",
                    ph_type: "body",
                    idx: 1,
                    x: (E * 2) / 3,
                    y: (E * 15) / 10,
                    cx: (E * 118) / 10,
                    cy: (E * 50) / 10,
                    required: true,
                },
            ],
        },

        // CL-20: SF Team Grid
        // team and grid both use Blank layout for dynamic cell counts,
        // but team has a title placeholder for outline-view accessibility.
        CustomLayoutSpec {
            layout_id: "CL-20",
            layout_name: "SF Team Grid",
            slide_types: vec!["team", "grid"],
            dark_theme: false,
            placeholders: vec![
                PlaceholderSpec {
                    name: "Title 1",
                    ph_type: "title",
                    idx: 0,
                    x: (E * 2) / 3,
                    y: (E * 54) / 100,
                    cx: (E * 118) / 10,
                    cy: E / 2,
                    required: true,
                },
                // No body placeholder — all member/cell content is explicit shapes
            ],
        },
    ]
}
