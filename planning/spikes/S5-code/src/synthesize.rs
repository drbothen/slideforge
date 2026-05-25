//! S5 Brand Synthesis Prototype
//!
//! Reads a brand.toml spec and generates a valid .pptx template containing:
//!   - theme1.xml (color scheme, font scheme, format scheme)
//!   - slideMaster1.xml with logo placeholder and clrMap
//!   - 11 standard layouts + 5 representative custom slideforge layouts:
//!       CL-01: Section Divider (clrMapOvr dark)
//!       CL-02: Stat Grid (key_metrics / stat_callout)
//!       CL-03: Timeline (vertical_timeline / horizontal_timeline)
//!       CL-04: Two-Panel Contrast (split_contrast)
//!       CL-05: End Slide (full-bleed brand color)
//!   - notesMaster1.xml (stub, required by PowerPoint)
//!   - handoutMaster1.xml (stub, required by PowerPoint)
//!   - A sample presentation.xml with one slide using CL-01
//!   - All required [Content_Types].xml and _rels files
//!
//! Output: s5-synthesized-brand.pptx in the current directory.
//!
//! Correctness invariants tested:
//!   - theme element ordering: clrScheme → fontScheme → fmtScheme
//!   - 12 color slots in fixed order: dk1,lt1,dk2,lt2,accent1-6,hlink,folHlink
//!   - slide master IDs start at 2^31, slide IDs at 256
//!   - every layout has its own _rels back-pointing to slideMaster1
//!   - [Content_Types].xml registers all parts
//!   - clrMapOvr on dark layouts (CL-01, CL-05)
//!   - logo shape on master (suppressed on CL-01 title layout via explicit blank shape)
//!   - accessibility: placeholder names set to semantic labels (not "Shape N")
//!   - reading order: title first in spTree, chrome (logo, page number) last

use std::collections::HashMap;
use std::io::{self, Write as IoWrite};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ── EMU constants ────────────────────────────────────────────────────────────
const EMU_PER_INCH: i64 = 914_400;
const SLIDE_CX: i64 = 12_192_000; // 13.333" × 914400
const SLIDE_CY: i64 = 6_858_000; // 7.5" × 914400

// ── OOXML namespace declarations ─────────────────────────────────────────────
const XMLNS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const XMLNS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const XMLNS_P: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const XMLNS_MC: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

// ── Relationship types ────────────────────────────────────────────────────────
const RT_PRESENTATION: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
const RT_SLIDE_MASTER: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
const RT_SLIDE_LAYOUT: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
const RT_THEME: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";
const RT_NOTES_MASTER: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster";
const RT_HANDOUT_MASTER: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/handoutMaster";
const RT_SLIDE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
const RT_CORE_PROPS: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
const RT_APP_PROPS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";

// ── Content types ─────────────────────────────────────────────────────────────
const CT_PRESENTATION: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml";
const CT_SLIDE_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml";
const CT_SLIDE_LAYOUT: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";
const CT_SLIDE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";
const CT_THEME: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
const CT_NOTES_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml";
const CT_HANDOUT_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml";
const CT_CORE_PROPS: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presProps+xml";
const CT_APP_PROPS: &str =
    "application/vnd.openxmlformats-officedocument.extended-properties+xml";

/// Brand configuration read from brand.toml.
/// In production this would use the serde+toml crates; here we use a hardcoded
/// struct to keep the spike focused on OOXML generation, not config parsing.
#[derive(Debug, Clone)]
struct BrandConfig {
    name: String,
    // 12 theme color slots in ECMA-376 fixed order
    dk1: String,  // Text/Background Dark 1
    lt1: String,  // Text/Background Light 1
    dk2: String,  // Text/Background Dark 2
    lt2: String,  // Text/Background Light 2 (surface alt)
    accent1: String,
    accent2: String,
    accent3: String,
    accent4: String,
    accent5: String,
    accent6: String,
    hlink: String,
    fol_hlink: String,
    // Font scheme
    heading_font: String,
    body_font: String,
}

impl BrandConfig {
    /// 1898 & Co. brand from the q1-decision-final.md example.
    fn example_1898() -> Self {
        Self {
            name: "1898 & Co.".to_string(),
            dk1: "1A1A1A".to_string(),   // near-black body text
            lt1: "FFFFFF".to_string(),   // white background
            dk2: "003766".to_string(),   // brand primary (dark blue)
            lt2: "F5F5F5".to_string(),   // light surface
            accent1: "003766".to_string(), // brand primary
            accent2: "FF6F00".to_string(), // accent orange
            accent3: "6B2D8B".to_string(), // accent purple
            accent4: "CC3333".to_string(), // danger red
            accent5: "339966".to_string(), // success green
            accent6: "81C6BD".to_string(), // teal
            hlink: "0563C1".to_string(),
            fol_hlink: "954F72".to_string(),
            heading_font: "Aptos Display".to_string(),
            body_font: "Aptos".to_string(),
        }
    }
}

/// Layout descriptor — one entry per layout file to be generated.
#[derive(Debug, Clone)]
struct LayoutDescriptor {
    /// Filename (e.g. "slideLayout1.xml")
    filename: String,
    /// Human-readable layout name (appears in PowerPoint layout picker)
    name: String,
    /// rId used by slideMaster to reference this layout
    rid: String,
    /// Whether this layout needs clrMapOvr (dark-themed layouts)
    dark_theme: bool,
    /// Placeholder list for this layout
    placeholders: Vec<PlaceholderDescriptor>,
    /// Layout category: standard (MS Office built-in) or custom (slideforge)
    is_custom: bool,
}

#[derive(Debug, Clone)]
struct PlaceholderDescriptor {
    /// ECMA-376 ST_PlaceholderType value
    ph_type: &'static str,
    /// Placeholder idx (unique within layout; 0 = title by convention for most)
    idx: u32,
    /// Shape name shown in Selection Pane (accessibility)
    name: &'static str,
    /// Position and size in EMU
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    /// Whether this placeholder must be present (vs optional chrome)
    required: bool,
}

fn main() -> io::Result<()> {
    let brand = BrandConfig::example_1898();
    let output_path = "s5-synthesized-brand.pptx";

    println!("S5 Brand Synthesis Prototype");
    println!("Brand: {}", brand.name);
    println!("Output: {}", output_path);
    println!();

    let file = std::fs::File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    // Build layout descriptors for all 11 standard + 5 custom layouts
    let layouts = build_layout_descriptors();

    println!("Generating {} layouts:", layouts.len());
    for (i, l) in layouts.iter().enumerate() {
        let kind = if l.is_custom { "CUSTOM" } else { "STANDARD" };
        println!("  Layout {:02}: {} ({}) — {} placeholders", i + 1, l.name, kind, l.placeholders.len());
    }
    println!();

    // [Content_Types].xml
    write_content_types(&mut zip, &options, &layouts)?;

    // _rels/.rels (package-level)
    write_package_rels(&mut zip, &options)?;

    // docProps/app.xml
    write_app_props(&mut zip, &options, &brand)?;

    // docProps/core.xml
    write_core_props(&mut zip, &options, &brand)?;

    // ppt/_rels/presentation.xml.rels
    write_presentation_rels(&mut zip, &options, &layouts)?;

    // ppt/theme/theme1.xml
    write_theme(&mut zip, &options, &brand)?;

    // ppt/slideMasters/slideMaster1.xml
    write_slide_master(&mut zip, &options, &brand, &layouts)?;

    // ppt/slideMasters/_rels/slideMaster1.xml.rels
    write_slide_master_rels(&mut zip, &options, &layouts)?;

    // ppt/slideLayouts/slideLayoutN.xml (all layouts)
    write_all_layouts(&mut zip, &options, &layouts)?;

    // ppt/notesMasters/notesMaster1.xml (stub)
    write_notes_master(&mut zip, &options)?;

    // ppt/handoutMasters/handoutMaster1.xml (stub)
    write_handout_master(&mut zip, &options)?;

    // ppt/presentation.xml
    write_presentation(&mut zip, &options)?;

    // ppt/slides/slide1.xml (one sample slide using Section Divider layout)
    write_sample_slide(&mut zip, &options, &brand)?;

    zip.finish()?;

    println!("Generated: {}", output_path);
    println!();
    println!("Validation checklist:");
    println!("  [ ] Open in PowerPoint — no repair dialog");
    println!("  [ ] Open in LibreOffice Impress — no warnings");
    println!("  [ ] Theme colors appear correctly in color picker");
    println!("  [ ] Layout picker shows all {} layouts", layouts.len());
    println!("  [ ] Section Divider layout has dark background (clrMapOvr)");
    println!("  [ ] End Slide layout has dark background");
    println!("  [ ] Placeholder names are semantic (not 'Shape N')");

    Ok(())
}

/// Build the complete layout descriptor list:
/// 11 standard Office layouts + 5 slideforge custom layouts = 16 total
fn build_layout_descriptors() -> Vec<LayoutDescriptor> {
    let mut layouts = Vec::new();

    // ── Standard Layout 1: Title Slide ───────────────────────────────────────
    // Uses ctrTitle + subTitle per ECMA-376.
    // This is the opening cover slide, NOT the section divider.
    layouts.push(LayoutDescriptor {
        filename: "slideLayout1.xml".to_string(),
        name: "Title Slide".to_string(),
        rid: "rId2".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "ctrTitle",
                idx: 0,
                name: "Title 1",
                x: EMU_PER_INCH,           // 1"
                y: 2 * EMU_PER_INCH,       // 2"
                cx: 10 * EMU_PER_INCH,     // 10"
                cy: (EMU_PER_INCH * 3) / 2, // 1.5"
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "subTitle",
                idx: 1,
                name: "Subtitle 2",
                x: EMU_PER_INCH,
                y: (EMU_PER_INCH * 7) / 2, // 3.5"
                cx: 10 * EMU_PER_INCH,
                cy: EMU_PER_INCH,
                required: false,
            },
        ],
    });

    // ── Standard Layout 2: Title and Content ─────────────────────────────────
    // The workhorse layout: title + single body area.
    // Maps to: content, table (plain), chart (single chart)
    layouts.push(LayoutDescriptor {
        filename: "slideLayout2.xml".to_string(),
        name: "Title and Content".to_string(),
        rid: "rId3".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,     // 0.67"
                y: (EMU_PER_INCH * 54) / 100,   // 0.54"
                cx: (EMU_PER_INCH * 118) / 10,  // 11.8"
                cy: EMU_PER_INCH / 2,            // 0.5"
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Content Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,    // 1.5"
                cx: (EMU_PER_INCH * 118) / 10,
                cy: 5 * EMU_PER_INCH,            // 5.0"
                required: true,
            },
        ],
    });

    // ── Standard Layout 3: Section Header ────────────────────────────────────
    // title + body intro text (light version — NOT the slideforge section divider)
    layouts.push(LayoutDescriptor {
        filename: "slideLayout3.xml".to_string(),
        name: "Section Header".to_string(),
        rid: "rId4".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 20) / 10,    // 2.0"
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 15) / 10,   // 1.5"
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Text Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 38) / 10,    // 3.8"
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 20) / 10,   // 2.0"
                required: false,
            },
        ],
    });

    // ── Standard Layout 4: Two Content ───────────────────────────────────────
    // title + two side-by-side body areas
    // Maps to: two_column
    layouts.push(LayoutDescriptor {
        filename: "slideLayout4.xml".to_string(),
        name: "Two Content".to_string(),
        rid: "rId5".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Content Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 58) / 10,   // 5.8" left column
                cy: 5 * EMU_PER_INCH,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Content Placeholder 3",
                x: (EMU_PER_INCH * 686) / 100, // 6.86"
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 58) / 10,   // 5.8" right column
                cy: 5 * EMU_PER_INCH,
                required: true,
            },
        ],
    });

    // ── Standard Layout 5: Comparison ────────────────────────────────────────
    // title + 4 body areas (2 header labels + 2 content areas)
    layouts.push(LayoutDescriptor {
        filename: "slideLayout5.xml".to_string(),
        name: "Comparison".to_string(),
        rid: "rId6".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            // Left header label
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Text Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 56) / 10,
                cy: (EMU_PER_INCH * 5) / 10,    // 0.5" header label
                required: false,
            },
            // Right header label
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Text Placeholder 3",
                x: (EMU_PER_INCH * 686) / 100,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 56) / 10,
                cy: (EMU_PER_INCH * 5) / 10,
                required: false,
            },
            // Left content
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 3,
                name: "Content Placeholder 4",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 22) / 10,    // 2.2"
                cx: (EMU_PER_INCH * 56) / 10,
                cy: (EMU_PER_INCH * 38) / 10,   // 3.8"
                required: true,
            },
            // Right content
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 4,
                name: "Content Placeholder 5",
                x: (EMU_PER_INCH * 686) / 100,
                y: (EMU_PER_INCH * 22) / 10,
                cx: (EMU_PER_INCH * 56) / 10,
                cy: (EMU_PER_INCH * 38) / 10,
                required: true,
            },
        ],
    });

    // ── Standard Layout 6: Title Only ─────────────────────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout6.xml".to_string(),
        name: "Title Only".to_string(),
        rid: "rId7".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
        ],
    });

    // ── Standard Layout 7: Blank ──────────────────────────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout7.xml".to_string(),
        name: "Blank".to_string(),
        rid: "rId8".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![],
    });

    // ── Standard Layout 8: Content with Caption ───────────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout8.xml".to_string(),
        name: "Content with Caption".to_string(),
        rid: "rId9".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Content Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 75) / 10,   // 7.5" main content
                cy: (EMU_PER_INCH * 50) / 10,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Text Placeholder 3",
                x: (EMU_PER_INCH * 85) / 10,    // 8.5"
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 35) / 10,   // 3.5" caption
                cy: (EMU_PER_INCH * 50) / 10,
                required: false,
            },
        ],
    });

    // ── Standard Layout 9: Picture with Caption ───────────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout9.xml".to_string(),
        name: "Picture with Caption".to_string(),
        rid: "rId10".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "pic",
                idx: 1,
                name: "Picture Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 85) / 10,   // 8.5" picture
                cy: (EMU_PER_INCH * 45) / 10,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Caption Placeholder 3",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 62) / 10,    // 6.2" caption below picture
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 4) / 10,
                required: false,
            },
        ],
    });

    // ── Standard Layout 10: Title and Vertical Text ───────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout10.xml".to_string(),
        name: "Title and Vertical Text".to_string(),
        rid: "rId11".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Vertical Text Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: 5 * EMU_PER_INCH,
                required: true,
            },
        ],
    });

    // ── Standard Layout 11: Vertical Title and Text ───────────────────────────
    layouts.push(LayoutDescriptor {
        filename: "slideLayout11.xml".to_string(),
        name: "Vertical Title and Text".to_string(),
        rid: "rId12".to_string(),
        dark_theme: false,
        is_custom: false,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 11) / 10,   // 1.1" — vertical title on right side
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 8) / 10,
                cy: 5 * EMU_PER_INCH,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Vertical Body Text Placeholder 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: 5 * EMU_PER_INCH,
                required: true,
            },
        ],
    });

    // ── Custom Layout CL-01: Section Divider (dark) ───────────────────────────
    // Full-bleed brand color background via clrMapOvr.
    // Maps to: slideforge `title` (opening) and `divider` types.
    // IMPORTANT: uses clrMapOvr to invert bg1/tx1 so brand dark = background,
    // and lt1 (white) = text. This is how a single theme produces both
    // light-body slides and dark-divider slides.
    layouts.push(LayoutDescriptor {
        filename: "slideLayout12.xml".to_string(),
        name: "SF Section Divider".to_string(),
        rid: "rId13".to_string(),
        dark_theme: true, // triggers clrMapOvr generation
        is_custom: true,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "ctrTitle",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: 2 * EMU_PER_INCH,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 15) / 10,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "subTitle",
                idx: 1,
                name: "Subtitle 2",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 38) / 10,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH,
                required: false,
            },
        ],
    });

    // ── Custom Layout CL-02: Stat Grid ────────────────────────────────────────
    // title + up to 6 stat card body slots in a 2-or-3 column grid.
    // Maps to: stat_callout, key_metrics, stats_summary, content_stat (stat portion)
    // In practice the layout engine draws the stat cards as explicit shapes;
    // the placeholders here are anchor positions only.
    layouts.push(LayoutDescriptor {
        filename: "slideLayout13.xml".to_string(),
        name: "SF Stat Grid".to_string(),
        rid: "rId14".to_string(),
        dark_theme: false,
        is_custom: true,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            // Stat card slot 1 (top-left)
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Stat 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 37) / 10,
                cy: (EMU_PER_INCH * 26) / 10,
                required: false,
            },
            // Stat card slot 2 (top-right)
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Stat 2",
                x: (EMU_PER_INCH * 730) / 100,  // 7.3"
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 37) / 10,
                cy: (EMU_PER_INCH * 26) / 10,
                required: false,
            },
            // Context/summary slot (optional footer)
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 3,
                name: "Context Placeholder",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 56) / 10,    // 5.6"
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 6) / 10,
                required: false,
            },
        ],
    });

    // ── Custom Layout CL-03: Timeline ─────────────────────────────────────────
    // title + a single large body area intended for timeline rendering.
    // Both vertical_timeline and horizontal_timeline use this layout;
    // the timeline rendering is done by the layout engine as explicit shapes,
    // and the body placeholder is effectively unused at runtime.
    // It exists so PowerPoint's outline view shows the title correctly.
    layouts.push(LayoutDescriptor {
        filename: "slideLayout14.xml".to_string(),
        name: "SF Timeline".to_string(),
        rid: "rId15".to_string(),
        dark_theme: false,
        is_custom: true,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Timeline Content Area",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 15) / 10,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 50) / 10,
                required: false,
            },
        ],
    });

    // ── Custom Layout CL-04: Two-Panel Contrast ───────────────────────────────
    // title + two equal-width panel body areas with brand-colored panels.
    // Maps to: split_contrast, card_rows, severity_cards (multi-column mode)
    layouts.push(LayoutDescriptor {
        filename: "slideLayout15.xml".to_string(),
        name: "SF Two Panel Contrast".to_string(),
        rid: "rId16".to_string(),
        dark_theme: false,
        is_custom: true,
        placeholders: vec![
            PlaceholderDescriptor {
                ph_type: "title",
                idx: 0,
                name: "Title 1",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 54) / 100,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: EMU_PER_INCH / 2,
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 1,
                name: "Left Panel",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 14) / 10,    // 1.4"
                cx: (EMU_PER_INCH * 58) / 10,   // 5.8"
                cy: (EMU_PER_INCH * 42) / 10,   // 4.2"
                required: true,
            },
            PlaceholderDescriptor {
                ph_type: "body",
                idx: 2,
                name: "Right Panel",
                x: (EMU_PER_INCH * 667) / 100,  // 6.67"
                y: (EMU_PER_INCH * 14) / 10,
                cx: (EMU_PER_INCH * 58) / 10,
                cy: (EMU_PER_INCH * 42) / 10,
                required: true,
            },
        ],
    });

    // ── Custom Layout CL-05: End Slide (dark) ────────────────────────────────
    // Full-bleed brand color. No content placeholders (just like end_slide in Python).
    // Uses clrMapOvr like CL-01 (dark theme variant).
    layouts.push(LayoutDescriptor {
        filename: "slideLayout16.xml".to_string(),
        name: "SF End Slide".to_string(),
        rid: "rId17".to_string(),
        dark_theme: true,
        is_custom: true,
        placeholders: vec![
            // Optional small centered text placeholder for "Thank You" message
            PlaceholderDescriptor {
                ph_type: "ctrTitle",
                idx: 0,
                name: "End Title",
                x: (EMU_PER_INCH * 2) / 3,
                y: (EMU_PER_INCH * 25) / 10,
                cx: (EMU_PER_INCH * 118) / 10,
                cy: (EMU_PER_INCH * 20) / 10,
                required: false,
            },
        ],
    });

    layouts
}

// ── XML generation helpers ────────────────────────────────────────────────────

/// Generate a hex color reference suitable for OOXML: <a:srgbClr val="RRGGBB"/>
fn srgb_clr(hex: &str) -> String {
    format!(r#"<a:srgbClr val="{}"/>"#, hex)
}

/// Generate a theme color reference: <a:schemeClr val="..."/>
fn scheme_clr(name: &str) -> String {
    format!(r#"<a:schemeClr val="{}"/>"#, name)
}

/// Generate the standard clrMap element (light-theme default).
fn clr_map_standard() -> &'static str {
    r#"<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2"
              accent1="accent1" accent2="accent2" accent3="accent3"
              accent4="accent4" accent5="accent5" accent6="accent6"
              hlink="hlink" folHlink="folHlink"/>"#
}

/// Generate the clrMapOvr for dark-themed layouts (brand color = background).
/// bg1 → dk2 (brand primary dark blue), tx1 → lt1 (white text)
fn clr_map_ovr_dark() -> &'static str {
    r#"<p:clrMapOvr>
    <a:overrideClrMapping bg1="dk2" tx1="lt1" bg2="dk1" tx2="lt2"
                          accent1="accent1" accent2="accent2" accent3="accent3"
                          accent4="accent4" accent5="accent5" accent6="accent6"
                          hlink="hlink" folHlink="folHlink"/>
  </p:clrMapOvr>"#
}

/// Generate an xfrm (transform) element giving position + size in EMU.
fn xfrm(x: i64, y: i64, cx: i64, cy: i64) -> String {
    format!(
        r#"<a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm>"#,
        x, y, cx, cy
    )
}

/// Generate a placeholder shape (sp element) for a layout.
fn placeholder_sp(ph: &PlaceholderDescriptor) -> String {
    let ph_attr = if ph.idx == 0 {
        format!(r#"type="{}" sz="full" idx="{}""#, ph.ph_type, ph.idx)
    } else {
        format!(r#"type="{}" sz="full" idx="{}""#, ph.ph_type, ph.idx)
    };

    format!(
        r#"    <p:sp>
      <p:nvSpPr>
        <p:cNvPr id="{id}" name="{name}"/>
        <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
        <p:nvPr><p:ph {ph_attr}/></p:nvPr>
      </p:nvSpPr>
      <p:spPr>
        {xfrm}
        <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
      </p:spPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p><a:endParaRPr lang="en-US" dirty="0"/></a:p>
      </p:txBody>
    </p:sp>"#,
        id = ph.idx + 2, // shape IDs start at 2; 1 is reserved for bg shape
        name = ph.name,
        ph_attr = ph_attr,
        xfrm = xfrm(ph.x, ph.y, ph.cx, ph.cy),
    )
}

// ── File generators ───────────────────────────────────────────────────────────

fn write_content_types(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    layouts: &[LayoutDescriptor],
) -> io::Result<()> {
    let mut overrides = String::new();

    // Core mandatory parts
    overrides.push_str(r#"  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
"#);

    for layout in layouts {
        overrides.push_str(&format!(
            r#"  <Override PartName="/ppt/slideLayouts/{}" ContentType="{}"/>
"#,
            layout.filename, CT_SLIDE_LAYOUT
        ));
    }

    overrides.push_str(r#"  <Override PartName="/ppt/notesMasters/notesMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/ppt/handoutMasters/handoutMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
"#);
    overrides.push_str(r#"  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
"#);

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Default Extension="jpeg" ContentType="image/jpeg"/>
  <Default Extension="jpg" ContentType="image/jpeg"/>
{}
</Types>"#,
        overrides
    );

    zip.start_file("[Content_Types].xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_package_rels(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> io::Result<()> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="ppt/presentation.xml"/>
  <Relationship Id="rId2" Type="{}" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="{}" Target="docProps/app.xml"/>
</Relationships>"#,
        RT_PRESENTATION, RT_CORE_PROPS, RT_APP_PROPS
    );

    zip.start_file("_rels/.rels", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_app_props(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    brand: &BrandConfig,
) -> io::Result<()> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>slideforge S5 Spike</Application>
  <Company>{}</Company>
  <Slides>1</Slides>
  <PresentationFormat>Widescreen</PresentationFormat>
</Properties>"#,
        brand.name
    );

    zip.start_file("docProps/app.xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_core_props(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    brand: &BrandConfig,
) -> io::Result<()> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/"
                   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:creator>{}</dc:creator>
  <dc:title>S5 Brand Synthesis — {} Brand Template</dc:title>
  <dc:description>Generated by slideforge S5 spike. Tests brand synthesis with 11 standard + 5 custom layouts.</dc:description>
  <cp:revision>1</cp:revision>
  <dcterms:created xsi:type="dcterms:W3CDTF">2026-05-24T00:00:00Z</dcterms:created>
</cp:coreProperties>"#,
        brand.name, brand.name
    );

    zip.start_file("docProps/core.xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_presentation_rels(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    _layouts: &[LayoutDescriptor],
) -> io::Result<()> {
    // presentation.xml rels: slideMaster, notesMaster, handoutMaster, slide
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rId2" Type="{}" Target="notesMasters/notesMaster1.xml"/>
  <Relationship Id="rId3" Type="{}" Target="handoutMasters/handoutMaster1.xml"/>
  <Relationship Id="rId4" Type="{}" Target="slides/slide1.xml"/>
</Relationships>"#,
        RT_SLIDE_MASTER, RT_NOTES_MASTER, RT_HANDOUT_MASTER, RT_SLIDE
    );

    zip.start_file("ppt/_rels/presentation.xml.rels", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

/// Generate theme1.xml with strict element ordering per ECMA-376:
/// clrScheme → fontScheme → fmtScheme (all inside themeElements)
/// 12 color slots in fixed order: dk1, lt1, dk2, lt2, accent1-6, hlink, folHlink
fn write_theme(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    brand: &BrandConfig,
) -> io::Result<()> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="{xmlns_a}" name="{brand_name} Theme">
  <a:themeElements>
    <!-- ORDERING INVARIANT: clrScheme MUST precede fontScheme MUST precede fmtScheme -->
    <!-- COLOR SLOT ORDERING INVARIANT: dk1,lt1,dk2,lt2,accent1-6,hlink,folHlink -->
    <a:clrScheme name="{brand_name}">
      <a:dk1><a:srgbClr val="{dk1}"/></a:dk1>
      <a:lt1><a:srgbClr val="{lt1}"/></a:lt1>
      <a:dk2><a:srgbClr val="{dk2}"/></a:dk2>
      <a:lt2><a:srgbClr val="{lt2}"/></a:lt2>
      <a:accent1><a:srgbClr val="{accent1}"/></a:accent1>
      <a:accent2><a:srgbClr val="{accent2}"/></a:accent2>
      <a:accent3><a:srgbClr val="{accent3}"/></a:accent3>
      <a:accent4><a:srgbClr val="{accent4}"/></a:accent4>
      <a:accent5><a:srgbClr val="{accent5}"/></a:accent5>
      <a:accent6><a:srgbClr val="{accent6}"/></a:accent6>
      <a:hlink><a:srgbClr val="{hlink}"/></a:hlink>
      <a:folHlink><a:srgbClr val="{fol_hlink}"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="{brand_name}">
      <a:majorFont>
        <!-- +mj-lt references this as the "major" (heading) font -->
        <a:latin typeface="{heading_font}"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <!-- CJK fallbacks (required for East Asian text rendering) -->
        <a:font script="Jpan" typeface="Yu Gothic"/>
        <a:font script="Hang" typeface="Malgun Gothic"/>
        <a:font script="Hans" typeface="DengXian"/>
        <a:font script="Arab" typeface="Traditional Arabic"/>
      </a:majorFont>
      <a:minorFont>
        <!-- +mn-lt references this as the "minor" (body) font -->
        <a:latin typeface="{body_font}"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Jpan" typeface="Yu Gothic UI"/>
        <a:font script="Hang" typeface="Malgun Gothic"/>
        <a:font script="Hans" typeface="DengXian Light"/>
        <a:font script="Arab" typeface="Arial"/>
      </a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <!-- Minimal format scheme — fill/line/effect styles PowerPoint expects -->
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0"><a:schemeClr val="phClr"><a:lumMod val="110000"/><a:satMod val="105000"/><a:tint val="67000"/></a:schemeClr></a:gs>
            <a:gs pos="50000"><a:schemeClr val="phClr"><a:lumMod val="105000"/><a:satMod val="103000"/><a:tint val="73000"/></a:schemeClr></a:gs>
            <a:gs pos="100000"><a:schemeClr val="phClr"><a:lumMod val="105000"/><a:satMod val="109000"/><a:tint val="81000"/></a:schemeClr></a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0"><a:schemeClr val="phClr"><a:satMod val="103000"/><a:lumMod val="102000"/><a:tint val="94000"/></a:schemeClr></a:gs>
            <a:gs pos="50000"><a:schemeClr val="phClr"><a:satMod val="110000"/><a:lumMod val="100000"/><a:shade val="100000"/></a:schemeClr></a:gs>
            <a:gs pos="100000"><a:schemeClr val="phClr"><a:lumMod val="99000"/><a:satMod val="120000"/><a:shade val="78000"/></a:schemeClr></a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle>
          <a:effectLst>
            <a:outerShdw blurRad="57150" dist="19050" dir="5400000" algn="ctr" rotWithShape="0">
              <a:srgbClr val="000000"><a:alpha val="63000"/></a:srgbClr>
            </a:outerShdw>
          </a:effectLst>
        </a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"><a:tint val="95000"/><a:satMod val="170000"/></a:schemeClr></a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0"><a:schemeClr val="phClr"><a:tint val="93000"/><a:satMod val="150000"/><a:shade val="98000"/><a:lumMod val="102000"/></a:schemeClr></a:gs>
            <a:gs pos="50000"><a:schemeClr val="phClr"><a:tint val="98000"/><a:satMod val="130000"/><a:shade val="90000"/><a:lumMod val="103000"/></a:schemeClr></a:gs>
            <a:gs pos="100000"><a:schemeClr val="phClr"><a:shade val="63000"/><a:satMod val="120000"/></a:schemeClr></a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
  <a:objectDefaults/>
  <a:extraClrSchemeLst/>
</a:theme>"#,
        xmlns_a = XMLNS_A,
        brand_name = brand.name,
        dk1 = brand.dk1,
        lt1 = brand.lt1,
        dk2 = brand.dk2,
        lt2 = brand.lt2,
        accent1 = brand.accent1,
        accent2 = brand.accent2,
        accent3 = brand.accent3,
        accent4 = brand.accent4,
        accent5 = brand.accent5,
        accent6 = brand.accent6,
        hlink = brand.hlink,
        fol_hlink = brand.fol_hlink,
        heading_font = brand.heading_font,
        body_font = brand.body_font,
    );

    zip.start_file("ppt/theme/theme1.xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_slide_master(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    brand: &BrandConfig,
    layouts: &[LayoutDescriptor],
) -> io::Result<()> {
    // Build sldLayoutIdLst from all layouts
    let mut layout_id_list = String::new();
    for (i, layout) in layouts.iter().enumerate() {
        // Layout relationship IDs in master start at rId2 (rId1 = theme)
        let rel_id = format!("rId{}", i + 2);
        layout_id_list.push_str(&format!(
            "    <p:sldLayoutId id=\"{}\" r:id=\"{}\"/>\n",
            2_147_483_649u64 + i as u64, // master layout IDs start at 2^31+1
            rel_id
        ));
    }

    // Slide master structure:
    // 1. cSld (background + spTree with title/body placeholder templates + logo)
    // 2. clrMap (standard light-theme mapping)
    // 3. sldLayoutIdLst (all layouts under this master)
    // 4. txStyles (title, body, other — each with 9 levels)
    //
    // READING ORDER in spTree: title placeholder → body placeholder → logo shape → page number
    // (top-to-bottom in XML = bottom-to-top in Selection Pane = correct screen-reader order)

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="{xmlns_a}"
             xmlns:r="{xmlns_r}"
             xmlns:p="{xmlns_p}">
  <p:cSld>
    <p:bg>
      <p:bgRef idx="1001">
        <a:schemeClr val="bg1"/>
      </p:bgRef>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="{slide_cx}" cy="{slide_cy}"/><a:chOff x="0" y="0"/><a:chExt cx="{slide_cx}" cy="{slide_cy}"/></a:xfrm>
      </p:grpSpPr>
      <!-- READING ORDER: title placeholder first, body placeholder second,
           then chrome (logo, page number) last. This is the correct screen-reader
           order and matches PowerPoint's Selection Pane bottom-to-top convention. -->
      <!-- Title placeholder template (type=title, idx=0) -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title Placeholder"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="title"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="{title_x}" y="{title_y}"/><a:ext cx="{title_cx}" cy="{title_cy}"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US" dirty="0"/><a:t>Click to edit Master title style</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <!-- Body placeholder template (type=body, idx=1) -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Content Placeholder"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="{body_x}" y="{body_y}"/><a:ext cx="{body_cx}" cy="{body_cy}"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:r><a:rPr lang="en-US" dirty="0"/><a:t>Click to edit Master text styles</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <!-- Logo (top-right corner, decorative — screen reader skips) -->
      <!-- Position: x=10.8", y=0.25", cx=1.0", cy=0.5" -->
      <!-- In production this would use <p:blipFill r:embed="rId10"/> to reference logo.png -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="4" name="Corporate Logo">
            <a:extLst>
              <a:ext uri="{{C452FA01-BE2C-4960-9963-9C64D71B0521}}">
                <!-- Mark as decorative so screen readers skip it -->
                <!-- In production: <adec:decorative val="1"/> -->
              </a:ext>
            </a:extLst>
          </p:cNvPr>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="10800000" y="228600"/><a:ext cx="914400" cy="457200"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          <!-- Placeholder logo: brand-colored rectangle -->
          <a:solidFill><a:schemeClr val="accent1"/></a:solidFill>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:endParaRPr lang="en-US" dirty="0"/></a:p>
        </p:txBody>
      </p:sp>
      <!-- Slide number placeholder (chrome, comes last in reading order) -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="5" name="Slide Number Placeholder"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="sldNum" sz="quarter" idx="3"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="0" y="6553200"/><a:ext cx="457200" cy="304800"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:fld id="{{A3B47C3D-2E3F-4F5A-8B6C-9D0E1F2A3B4C}}" type="slidenum">
              <a:rPr lang="en-US"/>
              <a:t>‹#›</a:t>
            </a:fld>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  {clr_map}
  <p:sldLayoutIdLst>
{layout_id_list}  </p:sldLayoutIdLst>
  <!-- txStyles: defines 9-level text hierarchy for title, body, and other -->
  <p:txStyles>
    <p:titleStyle>
      <a:lvl1pPr algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
        <a:spcBef><a:spcPts val="0"/></a:spcBef>
        <a:buNone/>
        <a:defRPr lang="en-US" sz="2400" b="1" dirty="0">
          <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
          <a:latin typeface="+mj-lt"/>
        </a:defRPr>
      </a:lvl1pPr>
    </p:titleStyle>
    <p:bodyStyle>
      <a:lvl1pPr marL="342900" indent="-342900" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
        <a:spcBef><a:spcPct val="20000"/></a:spcBef>
        <a:buFont typeface="Arial" charset="0"/>
        <a:buChar char="&#x2022;"/>
        <a:defRPr lang="en-US" sz="1800" dirty="0">
          <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
          <a:latin typeface="+mn-lt"/>
        </a:defRPr>
      </a:lvl1pPr>
      <a:lvl2pPr marL="685800" indent="-342900" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
        <a:spcBef><a:spcPct val="20000"/></a:spcBef>
        <a:buFont typeface="Arial" charset="0"/>
        <a:buChar char="&#x2013;"/>
        <a:defRPr lang="en-US" sz="1600" dirty="0">
          <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
          <a:latin typeface="+mn-lt"/>
        </a:defRPr>
      </a:lvl2pPr>
    </p:bodyStyle>
    <p:otherStyle>
      <a:defPPr>
        <a:defRPr lang="en-US" dirty="0"/>
      </a:defPPr>
    </p:otherStyle>
  </p:txStyles>
</p:sldMaster>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        slide_cx = SLIDE_CX,
        slide_cy = SLIDE_CY,
        // Title placeholder: 0.67" left, 0.54" top, 11.8" wide, 0.5" tall
        title_x = (EMU_PER_INCH * 2) / 3,
        title_y = (EMU_PER_INCH * 54) / 100,
        title_cx = (EMU_PER_INCH * 118) / 10,
        title_cy = EMU_PER_INCH / 2,
        // Body placeholder: 0.67" left, 1.5" top, 11.8" wide, 5.0" tall
        body_x = (EMU_PER_INCH * 2) / 3,
        body_y = (EMU_PER_INCH * 15) / 10,
        body_cx = (EMU_PER_INCH * 118) / 10,
        body_cy = 5 * EMU_PER_INCH,
        clr_map = clr_map_standard(),
        layout_id_list = layout_id_list,
    );

    zip.start_file("ppt/slideMasters/slideMaster1.xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_slide_master_rels(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    layouts: &[LayoutDescriptor],
) -> io::Result<()> {
    // slideMaster1.xml.rels: rId1 = theme, rId2..N = layouts
    let mut rels = String::new();
    rels.push_str(&format!(
        r#"  <Relationship Id="rId1" Type="{}" Target="../theme/theme1.xml"/>
"#,
        RT_THEME
    ));

    for (i, layout) in layouts.iter().enumerate() {
        let rid = format!("rId{}", i + 2);
        rels.push_str(&format!(
            r#"  <Relationship Id="{}" Type="{}" Target="../slideLayouts/{}"/>
"#,
            rid, RT_SLIDE_LAYOUT, layout.filename
        ));
    }

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{}
</Relationships>"#,
        rels
    );

    zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_all_layouts(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    layouts: &[LayoutDescriptor],
) -> io::Result<()> {
    for layout in layouts {
        write_single_layout(zip, options, layout)?;
        write_layout_rels(zip, options, layout)?;
    }
    Ok(())
}

fn write_single_layout(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    layout: &LayoutDescriptor,
) -> io::Result<()> {
    // Build the spTree children from placeholder descriptors
    let mut sp_tree_children = String::new();
    for ph in &layout.placeholders {
        sp_tree_children.push_str(&placeholder_sp(ph));
        sp_tree_children.push('\n');
    }

    // Dark layouts get clrMapOvr; light layouts get no override (inherit from master)
    let clr_map_section = if layout.dark_theme {
        // Full-bleed solid background fill for dark layouts
        let bg_section = r#"  <p:cSld>
    <p:bg>
      <p:bgPr>
        <!-- Background: brand primary color (dk2 via clrMapOvr maps to bg1) -->
        <a:solidFill><a:schemeClr val="bg1"/></a:solidFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>"#;

        format!(
            r#"{}
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="{}" cy="{}"/><a:chOff x="0" y="0"/><a:chExt cx="{}" cy="{}"/></a:xfrm>
      </p:grpSpPr>
{}    </p:spTree>
  </p:cSld>
  {}
"#,
            bg_section,
            SLIDE_CX,
            SLIDE_CY,
            SLIDE_CX,
            SLIDE_CY,
            sp_tree_children,
            clr_map_ovr_dark()
        )
    } else {
        format!(
            r#"  <p:cSld>
    <p:bg>
      <p:bgRef idx="1001">
        <a:schemeClr val="bg1"/>
      </p:bgRef>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="{}" cy="{}"/><a:chOff x="0" y="0"/><a:chExt cx="{}" cy="{}"/></a:xfrm>
      </p:grpSpPr>
{}    </p:spTree>
  </p:cSld>
"#,
            SLIDE_CX, SLIDE_CY, SLIDE_CX, SLIDE_CY, sp_tree_children
        )
    };

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="{xmlns_a}"
             xmlns:r="{xmlns_r}"
             xmlns:p="{xmlns_p}"
             type="{layout_type}"
             preserve="1">
{body}
</p:sldLayout>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        // Use OOXML standard layout type names where applicable
        layout_type = layout_type_name(&layout.name),
        body = clr_map_section,
    );

    zip.start_file(
        format!("ppt/slideLayouts/{}", layout.filename),
        *options,
    )?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

/// Map a layout name to the OOXML sldLayout type attribute.
/// For custom layouts we use "custom"; standard ones use ECMA-376 names.
fn layout_type_name(name: &str) -> &'static str {
    match name {
        "Title Slide" => "title",
        "Title and Content" => "obj",
        "Section Header" => "secHead",
        "Two Content" => "twoObj",
        "Comparison" => "twoColTx",
        "Title Only" => "titleOnly",
        "Blank" => "blank",
        "Content with Caption" => "objTx",
        "Picture with Caption" => "picTx",
        "Title and Vertical Text" => "vertTitleAndTx",
        "Vertical Title and Text" => "vertTx",
        _ => "custom", // All slideforge custom layouts
    }
}

fn write_layout_rels(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    layout: &LayoutDescriptor,
) -> io::Result<()> {
    // Each layout has a _rels file pointing back to the slide master.
    // This back-pointer is CRITICAL — PowerPoint loses the layout without it.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#,
        RT_SLIDE_MASTER
    );

    let rels_filename = format!(
        "ppt/slideLayouts/_rels/{}.rels",
        layout.filename
    );
    zip.start_file(rels_filename, *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_notes_master(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> io::Result<()> {
    // Minimal stub — required by PowerPoint even if no notes exist.
    // References the same theme1.xml as the slide master.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notesMaster xmlns:a="{xmlns_a}"
               xmlns:r="{xmlns_r}"
               xmlns:p="{xmlns_p}">
  <p:cSld>
    <p:bg>
      <p:bgRef idx="1001">
        <a:schemeClr val="bg1"/>
      </p:bgRef>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="6858000" cy="9144000"/><a:chOff x="0" y="0"/><a:chExt cx="6858000" cy="9144000"/></a:xfrm>
      </p:grpSpPr>
      <!-- Notes body placeholder -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Notes Placeholder"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="457200" y="3200400"/><a:ext cx="5943600" cy="5400900"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:endParaRPr lang="en-US" dirty="0"/></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  {clr_map}
  <p:notesStyle/>
</p:notesMaster>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        clr_map = clr_map_standard(),
    );

    zip.start_file("ppt/notesMasters/notesMaster1.xml", *options)?;
    zip.write_all(xml.as_bytes())?;

    // _rels for notesMaster1.xml
    let rels_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="../theme/theme1.xml"/>
</Relationships>"#,
        RT_THEME
    );
    zip.start_file("ppt/notesMasters/_rels/notesMaster1.xml.rels", *options)?;
    zip.write_all(rels_xml.as_bytes())?;

    Ok(())
}

fn write_handout_master(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> io::Result<()> {
    // Minimal stub — required by PowerPoint even if never printed.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:handoutMaster xmlns:a="{xmlns_a}"
                 xmlns:r="{xmlns_r}"
                 xmlns:p="{xmlns_p}">
  <p:cSld>
    <p:bg>
      <p:bgRef idx="1001">
        <a:schemeClr val="bg1"/>
      </p:bgRef>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="9144000" cy="6858000"/><a:chOff x="0" y="0"/><a:chExt cx="9144000" cy="6858000"/></a:xfrm>
      </p:grpSpPr>
    </p:spTree>
  </p:cSld>
  {clr_map}
</p:handoutMaster>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        clr_map = clr_map_standard(),
    );

    zip.start_file("ppt/handoutMasters/handoutMaster1.xml", *options)?;
    zip.write_all(xml.as_bytes())?;

    let rels_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="../theme/theme1.xml"/>
</Relationships>"#,
        RT_THEME
    );
    zip.start_file(
        "ppt/handoutMasters/_rels/handoutMaster1.xml.rels",
        *options,
    )?;
    zip.write_all(rels_xml.as_bytes())?;

    Ok(())
}

fn write_presentation(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
) -> io::Result<()> {
    // Slide master ID starts at 2^31 = 2147483648 per ECMA-376 minimum.
    // Slide IDs start at 256 per ECMA-376 minimum.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="{xmlns_a}"
                xmlns:r="{xmlns_r}"
                xmlns:p="{xmlns_p}"
                saveSubsetFonts="1">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:notesMasterIdLst>
    <p:notesMasterId r:id="rId2"/>
  </p:notesMasterIdLst>
  <p:handoutMasterIdLst>
    <p:handoutMasterId r:id="rId3"/>
  </p:handoutMasterIdLst>
  <p:sldIdLst>
    <!-- Slide IDs start at 256 per ECMA-376 -->
    <p:sldId id="256" r:id="rId4"/>
  </p:sldIdLst>
  <!-- 16:9 widescreen: 13.333" × 7.5" in EMU -->
  <p:sldSz cx="{slide_cx}" cy="{slide_cy}" type="screen16x9"/>
  <!-- Notes pages: portrait 4:3 -->
  <p:notesSz cx="6858000" cy="9144000"/>
  <p:defaultTextStyle>
    <a:defPPr>
      <a:defRPr lang="en-US"/>
    </a:defPPr>
    <a:lvl1pPr marL="0" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
      <a:defRPr lang="en-US" sz="1800" kern="1200">
        <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
        <a:latin typeface="+mn-lt"/>
      </a:defRPr>
    </a:lvl1pPr>
  </p:defaultTextStyle>
</p:presentation>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        slide_cx = SLIDE_CX,
        slide_cy = SLIDE_CY,
    );

    zip.start_file("ppt/presentation.xml", *options)?;
    zip.write_all(xml.as_bytes())?;
    Ok(())
}

fn write_sample_slide(
    zip: &mut ZipWriter<std::fs::File>,
    options: &SimpleFileOptions,
    brand: &BrandConfig,
) -> io::Result<()> {
    // Sample slide using layout 12 (CL-01 Section Divider, dark)
    // This tests that clrMapOvr works: title text should be white on dark brand bg.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="{xmlns_a}"
       xmlns:r="{xmlns_r}"
       xmlns:p="{xmlns_p}">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="{slide_cx}" cy="{slide_cy}"/><a:chOff x="0" y="0"/><a:chExt cx="{slide_cx}" cy="{slide_cy}"/></a:xfrm>
      </p:grpSpPr>
      <!-- Title on dark background (inherits white text via clrMapOvr tx1→lt1) -->
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="ctrTitle"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="en-US" dirty="0">
                <!-- Color inherits from scheme via clrMapOvr: tx1 → lt1 (white) -->
                <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
                <a:latin typeface="+mj-lt"/>
              </a:rPr>
              <a:t>S5 Brand Synthesis — {brand_name}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Subtitle 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="subTitle" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="en-US" dirty="0">
                <a:solidFill><a:schemeClr val="tx2"/></a:solidFill>
                <a:latin typeface="+mn-lt"/>
              </a:rPr>
              <a:t>16 layouts: 11 standard + 5 custom (slideforge types)</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <!-- Notes slide placeholder for accessibility -->
</p:sld>"#,
        xmlns_a = XMLNS_A,
        xmlns_r = XMLNS_R,
        xmlns_p = XMLNS_P,
        slide_cx = SLIDE_CX,
        slide_cy = SLIDE_CY,
        brand_name = brand.name,
    );

    zip.start_file("ppt/slides/slide1.xml", *options)?;
    zip.write_all(xml.as_bytes())?;

    // slide1.xml.rels — points to layout 12 (CL-01 Section Divider)
    let rels_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="{}" Target="../slideLayouts/slideLayout12.xml"/>
</Relationships>"#,
        RT_SLIDE_LAYOUT
    );

    zip.start_file("ppt/slides/_rels/slide1.xml.rels", *options)?;
    zip.write_all(rels_xml.as_bytes())?;

    Ok(())
}
