//! `slideforge-pptx` — PPTX exporter plugin for the slideforge pipeline.
//!
//! This crate implements the [`slideforge_plugin_api::Exporter`] trait for the
//! `.pptx` format. It takes a [`slideforge_types::Deck`] (semantic IR) and a
//! [`slideforge_layout::LaidOutDeck`] (geometric IR) together with a resolved
//! [`slideforge_types::Brand`] and produces a valid PPTX ZIP archive as
//! `Vec<u8>`.
//!
//! ## Architecture (STORY-037 / BC-4.01.001)
//!
//! ```text
//! PptxExporter::export(deck, laid_out, brand, opts)
//!   │
//!   ├─ SlideSerializer::build()     → ppt/slides/slide{n}.xml
//!   ├─ PresentationSerializer::build() → ppt/presentation.xml
//!   ├─ ContentTypesBuilder::build() → [Content_Types].xml
//!   ├─ RelsBuilder::build()         → _rels/.rels, ppt/_rels/…, etc.
//!   └─ ZipAssembler::assemble()     → final ZIP bytes
//! ```
//!
//! ## Constraints
//!
//! - All OOXML element construction goes through `ooxmlsdk` typed builders
//!   (ADR-001). No XML string concatenation.
//! - All coordinates are integer [`slideforge_types::Emu`] (`i64`). No `f64`.
//! - Report and detail register content MUST NOT appear in slide XML bodies.
//! - `notesMaster1.xml` and `handoutMaster1.xml` are always written.
//! - Slide IDs start at 256; master ID is 2^31.
//! - Output is deterministic: same inputs → byte-identical output.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Canonical number of slide layouts always embedded in every PPTX archive.
///
/// Both `build_master_parts` (master `.rels` relationship count) and
/// [`crate::layout_embedder::LayoutEmbedder::embed`] (layout XML part generation)
/// derive their loop bound from this single constant so master rels can never
/// reference a non-existent `slideLayout{N}.xml` part (cheap hardening,
/// F-038-P2 follow-up).
pub(crate) const LAYOUT_COUNT: usize = 31;

/// Default BCP-47 language tag used when `deck.metadata.lang` is `None`.
///
/// Re-exported from [`slideforge_types::DEFAULT_DECK_LANG`] — the canonical
/// shared source of truth (BC-5.01.004 / BC-5.01.005 / TD-VSDD-060).
///
/// The value is `"en"`. All surfaces (`<a:rPr lang>` in slide XML and
/// `<dc:language>` in `docProps/core.xml`) derive their no-lang fallback from
/// this single constant so they cannot diverge.
pub(crate) use slideforge_types::DEFAULT_DECK_LANG;

pub mod a11y;
pub mod brand_adapter;
pub mod clrmapovr;
pub mod content_types;
pub mod error;
pub mod layout_embedder;
pub mod link_safety;
pub mod notes_master;
pub mod notes_slide;
pub mod presentation;
pub mod rels;
pub mod sections;
pub mod slide_ids;
pub mod slide_serializer;
pub mod xml_escape;
pub mod zip_assembler;

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    dead_code,
    clippy::option_if_let_else,
    clippy::map_unwrap_or
)]
mod tests {
    mod a11y_tests;
    mod core_tests;
    mod inline_markup_tests;
    mod layout_tests;
    mod notes_tests;
    mod sections_tests;
    mod story_072_gradient_tests;
    mod story_096_tests;
}

use content_types::ContentTypesBuilder;
use notes_master::NotesMasterSerializer;
use notes_slide::NotesSlideSerializer;
use presentation::PresentationSerializer;
use rels::{RelsBuilder, rel_types};
use slide_serializer::SlideSerializer;
use slideforge_brand::BrandTemplate;
use slideforge_brand::layout_xml::{serialize_master_to_xml, serialize_theme_to_xml};
use slideforge_layout::{FrameContent, LaidOutDeck, LaidOutSlide};
use slideforge_plugin_api::{ExportError, ExportOptions, Exporter};
use slideforge_types::{Brand, Deck, Register};
use zip_assembler::{ZipAssembler, ZipPart};

use crate::brand_adapter::brand_template_from_brand;
use crate::error::PptxError;

/// The built-in PPTX exporter plugin.
///
/// `PptxExporter` implements the [`Exporter`] plugin trait for the `.pptx`
/// output format. It is registered with the `PluginRegistry`
/// under the id `"pptx"`.
///
/// ## Thread safety
///
/// `PptxExporter` holds no mutable state; it is `Send + Sync` by construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct PptxExporter;

impl PptxExporter {
    /// Create a new `PptxExporter` instance.
    ///
    /// The exporter is stateless; this function is equivalent to
    /// `PptxExporter::default()`.
    #[must_use]
    pub fn new() -> Self {
        PptxExporter
    }

    /// Core serialisation logic: build all PPTX parts and assemble the ZIP.
    ///
    /// Separated from the trait method so it can return [`PptxError`] directly
    /// before the trait boundary converts it to [`ExportError`].
    fn export_inner(
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        _opts: &ExportOptions,
    ) -> Result<Vec<u8>, PptxError> {
        tracing::debug!(
            slides = laid_out.slides.len(),
            "PptxExporter::export_inner starting"
        );

        // ADR-015: synthesize BrandTemplate from &Brand once for all serializers.
        let brand_template = brand_template_from_brand(brand);

        let mut parts: Vec<ZipPart> = Vec::new();

        // Build all parts, collecting into `parts`.
        //
        // F-037-010 fix: build presentation rels ONCE and thread the resulting rIds
        // to both `build_presentation_xml` and `build_presentation_rels_part`.
        // Previously rels were rebuilt twice (once for slide rIds, once for XML).
        let (slide_rel_ids, prs_rels_bytes) = build_presentation_rels_bytes(laid_out)?;

        // F-040-P1-001: compute the set of slides that HAVE non-empty notes once,
        // so build_slide_parts and build_notes_slide_parts use the SAME decision.
        // A slide "has notes" iff it has at least one non-whitespace-only
        // Register::Notes entry in register_content.
        let slides_with_notes: std::collections::HashSet<usize> = laid_out
            .slides
            .iter()
            .enumerate()
            .filter_map(|(i, slide)| {
                if slide_has_notes(slide) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // Extract page size once for master and slide serializers.
        // build_master_parts uses page_size_emu for placeholder geometry; the slide
        // serializer (lang on rPr) derives its lang from the deck at this point.
        let page_size_emu = (laid_out.page_size.width.0, laid_out.page_size.height.0);

        // SEC-096-001 / CWE-116 defense-in-depth: validate the raw lang value at
        // export_inner entry — BEFORE constructing deck_lang and BEFORE any slide XML
        // is generated.  This is the first layer; build_doc_props carries the second.
        // A valid BCP-47 tag (ASCII alphanumeric + hyphen) always passes unchanged
        // (lossless per BC-5.01.005 invariant 1). Values containing XML-1.0-illegal
        // control characters (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F, U+FFFE,
        // U+FFFF) are rejected here with PptxError::InvalidLanguageTag before any
        // ZIP part assembly begins.
        let lang_raw = deck.metadata.lang.as_deref().unwrap_or(DEFAULT_DECK_LANG);
        validate_lang_for_xml(lang_raw)?;
        let deck_lang: std::sync::Arc<str> = std::sync::Arc::from(lang_raw);

        build_slide_parts(
            laid_out,
            &brand_template,
            &slides_with_notes,
            &deck_lang,
            &mut parts,
        )?;
        build_presentation_xml(laid_out, brand, &slide_rel_ids, &mut parts)?;
        parts.push(ZipPart {
            path: "ppt/_rels/presentation.xml.rels".to_string(),
            bytes: prs_rels_bytes,
        });
        build_master_parts(&brand_template, page_size_emu, &mut parts)?;
        build_layout_parts(&brand_template, &mut parts)?;
        build_theme_part(&brand_template, &mut parts);
        build_notes_handout_masters(&brand_template, &mut parts)?;
        let notes_slide_count = build_notes_slide_parts(laid_out, &slides_with_notes, &mut parts)?;
        build_doc_props(deck, &mut parts)?;
        build_root_rels(&mut parts)?;

        // Content types are built last so all media parts are visible in `parts`.
        let mut ct = ContentTypesBuilder::new();
        for _ in 0..laid_out.slides.len() {
            ct.add_slide();
        }
        for _ in 0..LAYOUT_COUNT {
            ct.add_layout();
        }
        // Register notesSlide content-type overrides. Scan `parts` for notesSlide
        // paths (written by `build_notes_slide_parts`) to get the exact sparse
        // indices (e.g., notesSlide1.xml and notesSlide3.xml for a 3-slide deck
        // where only slides 1 and 3 have notes).
        let _ = notes_slide_count; // count verified via the path scan below
        for part in &parts {
            if part.path.starts_with("ppt/notesSlides/notesSlide")
                && !part.path.contains("/_rels/")
                && std::path::Path::new(&part.path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
            {
                ct.add_notes_slide(&part.path);
            }
        }
        for part in &parts {
            if part.path.starts_with("ppt/media/") {
                let ct_mime = content_type_for_media_path(&part.path);
                ct.add_media(&part.path, ct_mime);
            }
        }
        parts.push(ZipPart {
            path: "[Content_Types].xml".to_string(),
            bytes: ct.build()?,
        });

        ZipAssembler::assemble(parts)
    }
}

/// Build all slide XML parts and slide `.rels` files, including media parts.
///
/// ## Layout index wiring (F-037-011)
///
/// `LaidOutSlide.slide_type_keyword` is used to look up the matching layout
/// index in `brand_template.layouts` via `find_layout_index`. Phase 1 matches
/// the `slide_type_keyword` field on `SlideLayoutDef` (SF custom layouts).
/// Phase 2 matches by `ooxml_type` for the 5 standard DSL keywords. Falls back
/// to layout index 1 (Title and Content) with a `tracing::warn!` if no match.
///
/// ## Dark layout wiring (F-037-004)
///
/// The `is_dark_layout` flag is read from the layout's `has_color_override`
/// field (not hardcoded to `false`).
///
/// ## Diagram media wiring (F-037-005)
///
/// For `FrameContent::Diagram` frames, the SVG is written to `ppt/media/` and
/// an IMAGE relationship is added. A `<p:pic>` shape is also emitted in the
/// slide XML that references the media `rId` via `r:embed`, so the relationship
/// is never dangling.
///
/// ## Notes slide back-relationship (F-040-P1-001)
///
/// For each slide in `slides_with_notes`, a `rel_types::NOTES_SLIDE` relationship
/// is added to the slide's `.rels` file so `PowerPoint` can discover the slide's
/// notesSlide part. Without this the notesSlide is orphaned even if it exists.
///
/// ## Run language threading (STORY-096 AC-003)
///
/// `deck_lang` is the BCP-47 language tag from `deck.metadata.lang` (defaulting to
/// [`DEFAULT_DECK_LANG`] = `"en"` when absent). It is threaded into `SlideSerializer`
/// so every `<a:rPr>` element in every slide XML carries `lang="..."` (BC-5.01.005
/// postcondition 1).
fn build_slide_parts(
    laid_out: &LaidOutDeck,
    brand_template: &BrandTemplate,
    slides_with_notes: &std::collections::HashSet<usize>,
    deck_lang: &std::sync::Arc<str>,
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    let mut media_idx = 1_usize;

    for (i, slide) in laid_out.slides.iter().enumerate() {
        let slide_path = format!("ppt/slides/slide{}.xml", i + 1);
        let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", i + 1);

        // F-037-011: look up layout index from slide_type_keyword.
        // Layouts are 1-indexed in the ZIP (slideLayout1.xml = layouts[0]).
        let layout_index = find_layout_index(brand_template, slide.slide_type_keyword.as_ref());
        let layout_num = layout_index + 1; // 1-based ZIP name

        // F-037-004: read dark layout flag via ClrMapOvrInjector (F-038-P1-M1).
        // Routes through the single authoritative code path for dark-layout
        // detection — no inline has_color_override check outside that module.
        let is_dark_layout =
            crate::clrmapovr::ClrMapOvrInjector::needs_clr_map_ovr(brand_template, layout_index);

        let mut slide_rels = RelsBuilder::new();
        let layout_rel_id = slide_rels.add(
            rel_types::SLIDE_LAYOUT,
            format!("../slideLayouts/slideLayout{layout_num}.xml"),
        );

        // F-037-005: collect diagram frames and emit both media and <p:pic> shapes.
        // The diagram rIds must be collected before calling SlideSerializer so
        // the serializer can reference them in the slide XML.
        let mut diagram_rids: Vec<(usize, String)> = Vec::new(); // (frame_idx, rId)

        for (frame_idx, frame) in slide.frames.iter().enumerate() {
            // STORY-039 IR reshape: Diagram is now struct with svg + alt fields.
            if let FrameContent::Diagram {
                svg: normalized_svg,
                ..
            } = &frame.content
            {
                let media_filename = format!("image{media_idx}.svg");
                let media_path = format!("ppt/media/{media_filename}");
                let rid = slide_rels.add(rel_types::IMAGE, format!("../media/{media_filename}"));
                parts.push(ZipPart {
                    path: media_path,
                    bytes: normalized_svg.as_str().as_bytes().to_vec(),
                });
                diagram_rids.push((frame_idx, rid));
                media_idx += 1;
            }
        }

        // EC-004: collect body-path hyperlink URLs and register External rels BEFORE
        // building the slide XML. The rIds returned by add_external_hyperlink are
        // sequential (rId{N+1}, rId{N+2}, ...) and passed to SlideSerializer::build
        // via hlink_map so the serializer can wire <a:hlinkClick r:id="..."/>.
        let hlink_urls = slide_serializer::collect_hyperlink_urls_from_slide(slide);
        let hlink_map: Vec<(String, String)> = hlink_urls
            .into_iter()
            .map(|url| {
                let rid = slide_rels.add_external_hyperlink(url.as_str());
                (url, rid)
            })
            .collect();

        // Build the slide XML. SlideSerializer handles text frames AND diagram
        // <p:pic> shapes via typed ooxmlsdk builders (ADR-001, F-037-005).
        // AC-011: thread the resolved layout's placeholder info into the serializer
        // so it can perform idx-chain verification (ADR-015 §7).
        // STORY-096 AC-003: thread deck_lang so every <a:rPr> carries lang="...".
        let serializer = if let Some(layout) = brand_template.layouts.get(layout_index) {
            SlideSerializer::new(is_dark_layout, layout_index)
                .with_layout(layout)
                .with_lang(deck_lang)
        } else {
            SlideSerializer::new(is_dark_layout, layout_index).with_lang(deck_lang)
        };
        let (slide_xml_bytes, _warnings) =
            serializer.build(slide, i, &layout_rel_id, &diagram_rids, &hlink_map)?;

        parts.push(ZipPart {
            path: slide_path,
            bytes: slide_xml_bytes,
        });

        // F-040-P1-001: add notesSlide relationship to slide's rels if this slide
        // has notes. This is the SLIDE-side relationship that PowerPoint uses to
        // discover the slide's notes part. The notesSlide→slide back-rel alone
        // is insufficient for PowerPoint to find the notes.
        if slides_with_notes.contains(&i) {
            slide_rels.add(
                rel_types::NOTES_SLIDE,
                format!("../notesSlides/notesSlide{}.xml", i + 1),
            );
        }

        parts.push(ZipPart {
            path: rels_path,
            bytes: slide_rels.build()?,
        });
    }
    Ok(())
}

/// Find the 0-based layout index for a given `slide_type_keyword`.
///
/// ## Two-phase lookup (ADR-015 §A.2, closes F-PASS2-C1 and F-PASS2-M2)
///
/// **Phase 1 — custom layout keyword match:**
/// Search `brand_template.layouts` for a layout whose `slide_type_keyword` field
/// equals `slide_type_keyword`. This covers the 20 SF custom layouts (CL-01..CL-20).
///
/// **Phase 2 — standard layout OOXML type match:**
/// For the 5 standard layouts that correspond to DSL keywords (`title`, `content`,
/// `two_column`, `table`, blank), map the DSL keyword to its OOXML type string and
/// find the layout whose `ooxml_type` matches.
///
/// **Fallback:**
/// If no match is found in either phase, returns layout index 1 (0-based), which is
/// "Title and Content" (a generic content layout). Index 0 ("Title Slide") is NOT
/// the fallback — it is reserved for explicit `title` keyword slides. A
/// `tracing::warn!` is emitted naming the unmatched keyword.
///
/// ## Index semantics
///
/// Returns a 0-based index into `brand_template.layouts`. The ZIP file name is
/// `slideLayout{index + 1}.xml` (1-based).
fn find_layout_index(brand_template: &BrandTemplate, slide_type_keyword: &str) -> usize {
    // Phase 1: match by slide_type_keyword field (SF custom layouts).
    for (idx, layout) in brand_template.layouts.iter().enumerate() {
        if layout
            .slide_type_keyword
            .as_deref()
            .is_some_and(|kw| kw == slide_type_keyword)
        {
            return idx;
        }
    }

    // Phase 2: match by ooxml_type for standard DSL keywords.
    // Maps the 5 DSL keywords that correspond to standard OOXML layout types.
    let ooxml_type_for_keyword = match slide_type_keyword {
        "title" => Some("title"),
        "content" => Some("obj"),
        "two_column" => Some("twoObj"),
        "table" => Some("objTx"),
        "blank" => Some("blank"),
        _ => None,
    };

    if let Some(ooxml_type) = ooxml_type_for_keyword {
        for (idx, layout) in brand_template.layouts.iter().enumerate() {
            if layout.ooxml_type.as_deref() == Some(ooxml_type) {
                return idx;
            }
        }
    }

    // Fallback: "Title and Content" layout (index 1, 0-based).
    // Index 0 ("Title Slide") is NOT the fallback — it is reserved for explicit
    // `title` keyword slides. This matches the no-silent-fallback rule (ADR-015 §A.4).
    tracing::warn!(
        slide_type = slide_type_keyword,
        fallback_index = 1usize,
        "no matching layout found for slide_type_keyword; \
         falling back to layout index 1 (Title and Content, 0-based). \
         Index 0 (Title Slide) is intentionally NOT the generic fallback."
    );
    1
}

#[cfg(test)]
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::expect_used
)]
mod layout_index_tests {
    use super::*;

    /// Build a minimal `BrandTemplate` with the 31 standard layouts for testing
    /// `find_layout_index` without going through the full export path.
    fn test_brand_template() -> BrandTemplate {
        use std::sync::Arc;

        use slideforge_brand::layout_xml::{
            HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
        };
        use slideforge_brand::layouts::generate_all_layouts;
        use slideforge_brand::template::{BrandFonts, ColorSlot, ColorValue, MasterIds};
        use slideforge_brand::toml_schema::BrandConfig;

        let config = BrandConfig::default_minimal();
        let layouts = generate_all_layouts(&config);
        BrandTemplate {
            colors: std::array::from_fn(|i| ColorSlot {
                name: Arc::from(slideforge_brand::template::COLOR_SLOT_NAMES[i]),
                value: ColorValue::Hex(Arc::from("003087")),
                is_derived: false,
            }),
            fonts: BrandFonts {
                heading: Arc::from("Calibri"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            footer_flags: slideforge_brand::FooterFlags::default(),
            layout_names: vec![],
            layouts,
            notes_master_stub: NOTES_MASTER_STUB.to_vec(),
            handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(
                generate_content_types_layout_entries(LAYOUT_COUNT).as_str(),
            ),
        }
    }

    /// ADR-015 §A.4 — `"section_divider"` must map to 0-based index 11 (CL-01).
    ///
    /// Layout 12 (1-based) is "SF Section Divider". 0-based index = 11.
    #[test]
    fn test_find_layout_index_section_divider_maps_to_11() {
        let template = test_brand_template();
        let idx = find_layout_index(&template, "section_divider");
        assert_eq!(
            idx, 11,
            "\"section_divider\" must map to 0-based index 11 (SF Section Divider, layout 12) \
             (ADR-015 §A.4)"
        );
    }

    /// ADR-015 §A.4 — "end" must map to 0-based index 21 (CL-11, SF End Slide).
    ///
    /// Layout 22 (1-based) is "SF End Slide". 0-based index = 21.
    #[test]
    fn test_find_layout_index_end_maps_to_21() {
        let template = test_brand_template();
        let idx = find_layout_index(&template, "end");
        assert_eq!(
            idx, 21,
            "\"end\" must map to 0-based index 21 (SF End Slide, layout 22) (ADR-015 §A.4)"
        );
    }

    /// ADR-015 §A.4 — "title" must map to 0-based index 0 (SL-01, Title Slide).
    ///
    /// Layout 1 (1-based) is "Title Slide" with `ooxml_type` = `"title"`. 0-based index = 0.
    #[test]
    fn test_find_layout_index_title_maps_to_0() {
        let template = test_brand_template();
        let idx = find_layout_index(&template, "title");
        assert_eq!(
            idx, 0,
            "\"title\" must map to 0-based index 0 (Title Slide) (ADR-015 §A.4)"
        );
    }

    /// ADR-015 §A.4 — unknown keyword must fall back to index 1 (NOT index 0).
    ///
    /// Index 1 is "Title and Content" — the generic content fallback.
    /// Index 0 ("Title Slide") must NOT be used as a generic fallback.
    #[test]
    fn test_find_layout_index_unknown_falls_back_to_1() {
        let template = test_brand_template();
        let idx = find_layout_index(&template, "totally_unknown_keyword_xyz");
        assert_eq!(
            idx, 1,
            "an unknown keyword must fall back to 0-based index 1 (Title and Content), \
             NOT index 0 (Title Slide) — index 0 is reserved for explicit 'title' slides \
             (ADR-015 §A.4)"
        );
    }

    /// Verify `"content"` maps to 0-based index 1 (SL-02, Title and Content, `ooxml_type` = `"obj"`).
    #[test]
    fn test_find_layout_index_content_maps_to_1() {
        let template = test_brand_template();
        let idx = find_layout_index(&template, "content");
        assert_eq!(
            idx, 1,
            "\"content\" must map to 0-based index 1 (Title and Content, ooxml_type obj)"
        );
    }
}

/// Build `ppt/_rels/presentation.xml.rels` and return the slide `rId` list AND the bytes.
///
/// F-037-010: Previously presentation.xml.rels was built twice — once in
/// `build_presentation_rels` (to get rIds for `presentation.xml`) and again in
/// `build_presentation_xml` via a second `RelsBuilder`. The second build assigned
/// rIds independently, risking desynchronization. Now we build the rels ONCE and
/// return both the serialized bytes (to write as the part) and the rId list (to pass
/// into `PresentationSerializer`).
fn build_presentation_rels_bytes(
    laid_out: &LaidOutDeck,
) -> Result<(Vec<String>, Vec<u8>), PptxError> {
    let mut prs_rels = RelsBuilder::new();
    let _master_rel_id = prs_rels.add(rel_types::SLIDE_MASTER, "slideMasters/slideMaster1.xml");
    let _notes_master_rel_id =
        prs_rels.add(rel_types::NOTES_MASTER, "notesMasters/notesMaster1.xml");
    let _handout_master_rel_id = prs_rels.add(
        rel_types::HANDOUT_MASTER,
        "handoutMasters/handoutMaster1.xml",
    );

    let mut slide_rel_ids: Vec<String> = Vec::new();
    for i in 0..laid_out.slides.len() {
        let rid = prs_rels.add(rel_types::SLIDE, format!("slides/slide{}.xml", i + 1));
        slide_rel_ids.push(rid);
    }

    let bytes = prs_rels.build()?;
    Ok((slide_rel_ids, bytes))
}

/// Build `ppt/presentation.xml`.
///
/// F-037-010: rIds are now derived from the same `RelsBuilder` that produced
/// `presentation.xml.rels` (via `build_presentation_rels_bytes`). rId1 = master,
/// rId2 = notes master, rId3 = handout master, rId4..=rId{N+3} = slides.
/// These are the canonical rIds — hardcoded by convention to match the
/// `build_presentation_rels_bytes` assignment order.
fn build_presentation_xml(
    laid_out: &LaidOutDeck,
    brand: &Brand,
    slide_rel_ids: &[String],
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    // These rId values MUST match the order in `build_presentation_rels_bytes`.
    // rId1 = slide master, rId2 = notes master, rId3 = handout master.
    // slide_rel_ids[i] = rId{4+i}.
    let master_rel_id = "rId1";
    let notes_master_rel_id = "rId2";
    let handout_master_rel_id = "rId3";

    let prs_xml = PresentationSerializer::build(
        laid_out,
        brand,
        slide_rel_ids,
        master_rel_id,
        notes_master_rel_id,
        handout_master_rel_id,
    )?;
    // STORY-082 CRIT-1: inject p14:sectionLst into presentation.xml when
    // slide_sections is non-empty. If empty, returns bytes unchanged (AC-004).
    let prs_xml = crate::sections::SectionListBuilder::inject(prs_xml, &laid_out.slide_sections)?;
    parts.push(ZipPart {
        path: "ppt/presentation.xml".to_string(),
        bytes: prs_xml,
    });
    Ok(())
}

/// Build `slideMaster1.xml` and its `.rels`.
///
/// ADR-015 §2: uses `serialize_master_to_xml` from `slideforge-brand` to
/// produce a schema-valid master with `<a:clrMap>`, `<p:sldLayoutIdLst>`,
/// `<p:txStyles>`, 5 master placeholder shapes, and `<p:hf>` flags.
/// `CT_SlideMaster` does not include `<p:sldSz>` (ECMA-376 §19.3.1.42);
/// slide size is carried exclusively by `presentation.xml` (STORY-096 AC-001 v1.2).
///
/// ## Page size threading
///
/// `page_size_emu` is `(width_emu, height_emu)` sourced from `LaidOutDeck.page_size`
/// in `export_inner`. It is threaded here so that footer and slide-number placeholder
/// positions remain within the declared page bounds (BC-4.01.001 postcondition 4 / AC-004).
///
/// The master `.rels` file references:
/// - rId1: the theme
/// - rId2..=rId32: the 31 slide layouts
///
/// These rId values match the `sldLayoutIdLst` entries in master XML
/// (which use r:id="rId2".."rId32").
fn build_master_parts(
    brand_template: &BrandTemplate,
    page_size_emu: (i64, i64),
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    parts.push(ZipPart {
        path: "ppt/slideMasters/slideMaster1.xml".to_string(),
        bytes: serialize_master_to_xml(brand_template, page_size_emu),
    });

    let mut master_rels = RelsBuilder::new();
    // rId1 = theme (matches the rId used in serialize_master_to_xml for theme ref if any)
    master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    // rId2..=rId32 = layouts (always exactly LAYOUT_COUNT = 31 entries,
    // matching the parts written by LayoutEmbedder::embed).
    for n in 1..=LAYOUT_COUNT {
        master_rels.add(
            rel_types::SLIDE_LAYOUT,
            format!("../slideLayouts/slideLayout{n}.xml"),
        );
    }
    parts.push(ZipPart {
        path: "ppt/slideMasters/_rels/slideMaster1.xml.rels".to_string(),
        bytes: master_rels.build()?,
    });
    Ok(())
}

/// Build all 31 slide layout XML files and their `.rels`.
///
/// Routes through [`LayoutEmbedder::embed`] — the single authoritative code
/// path for layout part generation (BC-4.01.005 invariant 3; F-038-P1-M1).
/// No inline loop or duplicate XML generation here.
fn build_layout_parts(
    brand_template: &BrandTemplate,
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    crate::layout_embedder::LayoutEmbedder::embed(brand_template, parts)
}

/// Build `theme1.xml` from brand data and push to parts.
///
/// ADR-015 §3: uses `serialize_theme_to_xml` from `slideforge-brand` to
/// produce a `theme1.xml` from the brand's 12 color slots and font names.
/// Closes F-037-007 (hardcoded theme bytes removed).
fn build_theme_part(brand_template: &BrandTemplate, parts: &mut Vec<ZipPart>) {
    parts.push(ZipPart {
        path: "ppt/theme/theme1.xml".to_string(),
        bytes: serialize_theme_to_xml(brand_template),
    });
}

/// Build `notesSlide{N}.xml` and `_rels/notesSlide{N}.xml.rels` for each slide
/// with non-empty speaker notes (BC-4.01.003 / STORY-040).
///
/// Returns the count of notesSlide parts written (for content-type registration).
///
/// ## Naming convention
///
/// The notesSlide index (N) matches the corresponding slide's 1-based index in
/// the ZIP (`slide1.xml` → `notesSlide1.xml`, `slide3.xml` → `notesSlide3.xml`).
/// Slides without notes get no notesSlide part (BC-4.01.003 postcondition 2 / EC-001).
///
/// ## Relationship targets
///
/// Each notesSlide's `.rels` file uses relative paths:
/// - `rId1` → `../slides/slide{N}.xml`  (the owning slide)
/// - `rId2` → `../notesMasters/notesMaster1.xml`  (notes master)
/// - `rId3..` → external hyperlinks (if any in notes content)
///
/// ## Notes sourcing (F-040-P1-003)
///
/// Sources notes from `register_content` filtered to `Register::Notes`,
/// covering ALL Notes entries (not just the first). The `slides_with_notes`
/// set is pre-computed in `export_inner` so the decision is consistent with
/// `build_slide_parts` (F-040-P1-001 same-set invariant).
fn build_notes_slide_parts(
    laid_out: &LaidOutDeck,
    slides_with_notes: &std::collections::HashSet<usize>,
    parts: &mut Vec<ZipPart>,
) -> Result<usize, PptxError> {
    let mut count = 0_usize;

    for (i, slide) in laid_out.slides.iter().enumerate() {
        // Only process slides that are in the precomputed notes set.
        if !slides_with_notes.contains(&i) {
            continue;
        }

        let slide_num = i + 1; // 1-based

        tracing::debug!(
            slide = slide_num,
            notes_entries = slide
                .register_content
                .iter()
                .filter(|rc| rc.register == Register::Notes)
                .count(),
            "building notesSlide part"
        );

        // ADR-024: NotesSlideSerializer::build uses the unified
        // render_inline_nodes_to_runs engine internally (no InlineFormat parameter).
        let output =
            NotesSlideSerializer::build(slide_num, &slide.register_content).map_err(|e| {
                PptxError::OoxmlElement {
                    part: format!("ppt/notesSlides/notesSlide{slide_num}.xml"),
                    detail: format!("NotesSlideSerializer::build failed: {e}"),
                }
            })?;

        parts.push(ZipPart {
            path: format!("ppt/notesSlides/notesSlide{slide_num}.xml"),
            bytes: output.xml_bytes,
        });
        parts.push(ZipPart {
            path: format!("ppt/notesSlides/_rels/notesSlide{slide_num}.xml.rels"),
            bytes: output.rels_bytes,
        });

        count += 1;
    }

    Ok(count)
}

/// Build `notesMaster1.xml` and `handoutMaster1.xml` (always present — BC-4.01.006).
///
/// ## Notes master sourcing (F-040-P1-002, F-040-P1-005)
///
/// The emitted `notesMaster1.xml` is ALWAYS a valid master containing `<p:clrMap>`,
/// `<p:ph type="sldImg"/>`, and `<p:ph type="body" idx="1"/>`. This is produced by
/// `NotesMasterSerializer::build_notes_master` — the single authoritative source.
///
/// A brand-provided notes master is only used if it is non-empty (i.e., the brand
/// extracted a real notes master from a .pptx template). The empty `NOTES_MASTER_STUB`
/// from `slideforge_brand::layout_xml` is NEVER emitted as the final part; it is
/// only used as a sentinel for "brand has no custom notes master".
///
/// ## Error handling (SEC-002 / CWE-755)
///
/// Both rels builds are `?`-propagated. The previous `unwrap_or_else(|_| b"".to_vec())`
/// pattern violated the no-silent-fallback Forbidden Pattern (CLAUDE.md): an empty rels
/// byte sequence produces a structurally-invalid PPTX. `RelsBuilder::build()` only fails
/// when no entries have been added; since we always add one entry before calling `build()`,
/// a failure here indicates a bug in `RelsBuilder` itself, which must surface as an error
/// rather than silently producing a malformed archive.
fn build_notes_handout_masters(
    brand_template: &BrandTemplate,
    parts: &mut Vec<ZipPart>,
) -> Result<(), PptxError> {
    // F-040-P1-002: route through NotesMasterSerializer — the single authoritative
    // source for a valid notesMaster1.xml. The brand stub is used ONLY if it is a
    // non-empty brand-extracted master (not the empty default stub).
    // The empty NOTES_MASTER_STUB is never the final emitted bytes.
    let brand_notes = brand_template.notes_master_stub.as_slice();
    let is_brand_stub_empty =
        brand_notes == slideforge_brand::layout_xml::NOTES_MASTER_STUB || brand_notes.is_empty();
    let notes_bytes = if is_brand_stub_empty {
        // Brand has no custom notes master: emit the valid default from NotesMasterSerializer.
        NotesMasterSerializer::build_notes_master(b"")?
    } else {
        // Brand has a real extracted notes master: use it verbatim.
        NotesMasterSerializer::build_notes_master(brand_notes)?
    };
    parts.push(ZipPart {
        path: "ppt/notesMasters/notesMaster1.xml".to_string(),
        bytes: notes_bytes,
    });

    // notesMaster.rels — references theme.  Propagate rels-build error with `?`
    // instead of silently substituting empty bytes (SEC-002 / CWE-755).
    let mut notes_master_rels = RelsBuilder::new();
    notes_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    let notes_rels_bytes = notes_master_rels
        .build()
        .map_err(|e| PptxError::OoxmlElement {
            part: "ppt/notesMasters/_rels/notesMaster1.xml.rels".to_string(),
            detail: format!("RelsBuilder::build failed: {e}"),
        })?;
    parts.push(ZipPart {
        path: "ppt/notesMasters/_rels/notesMaster1.xml.rels".to_string(),
        bytes: notes_rels_bytes,
    });

    // F-040-P1-002 (handout): same pattern as notesMaster — route through
    // NotesMasterSerializer::build_handout_master for the authoritative valid bytes.
    let brand_handout = brand_template.handout_master_stub.as_slice();
    let is_brand_handout_stub_empty = brand_handout
        == slideforge_brand::layout_xml::HANDOUT_MASTER_STUB
        || brand_handout.is_empty();
    let handout_bytes = if is_brand_handout_stub_empty {
        NotesMasterSerializer::build_handout_master(b"")?
    } else {
        NotesMasterSerializer::build_handout_master(brand_handout)?
    };
    parts.push(ZipPart {
        path: "ppt/handoutMasters/handoutMaster1.xml".to_string(),
        bytes: handout_bytes,
    });

    // handoutMaster.rels — same propagation pattern as notesMaster.rels.
    let mut handout_master_rels = RelsBuilder::new();
    handout_master_rels.add(rel_types::THEME, "../theme/theme1.xml");
    let handout_rels_bytes = handout_master_rels
        .build()
        .map_err(|e| PptxError::OoxmlElement {
            part: "ppt/handoutMasters/_rels/handoutMaster1.xml.rels".to_string(),
            detail: format!("RelsBuilder::build failed: {e}"),
        })?;
    parts.push(ZipPart {
        path: "ppt/handoutMasters/_rels/handoutMaster1.xml.rels".to_string(),
        bytes: handout_rels_bytes,
    });

    Ok(())
}

/// Validate that `lang` contains no XML-1.0-illegal control characters.
///
/// XML 1.0 §2.2 defines legal characters as:
/// `#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]`
///
/// The illegal ranges relevant here are: U+0000–U+0008, U+000B, U+000C,
/// U+000E–U+001F, U+FFFE, U+FFFF. A valid BCP-47 tag (ASCII alphanumeric + `-`)
/// always passes unchanged (lossless per BC-5.01.005 invariant 1).
///
/// On success the original `lang` string is returned unmodified (lossless).
/// On failure a [`PptxError::InvalidLanguageTag`] is returned (SEC-039-001 / CWE-116).
fn validate_lang_for_xml(lang: &str) -> Result<(), PptxError> {
    for ch in lang.chars() {
        let code = ch as u32;
        let illegal = matches!(
            code,
            0x0000..=0x0008 | 0x000B | 0x000C | 0x000E..=0x001F | 0xFFFE | 0xFFFF
        );
        if illegal {
            return Err(PptxError::InvalidLanguageTag {
                lang: lang.to_owned(),
                reason: format!("contains XML-1.0-illegal control character U+{code:04X}"),
            });
        }
    }
    Ok(())
}

/// Build `docProps/core.xml` and `docProps/app.xml`.
///
/// These use pre-formed XML strings — the OPC core properties namespace is
/// outside the `ooxmlsdk` schema module scope for this story.
///
/// ## XML escaping (F-037-006)
///
/// The `dc:language` value is XML-escaped before interpolation. A user-supplied
/// lang value like `"en-US<script>"` must not produce malformed XML.
/// Characters escaped: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`,
/// `"` → `&quot;`, `'` → `&apos;`.
///
/// ## XML-1.0 control-character validation (SEC-039-001 / CWE-116)
///
/// Before escaping, the raw lang value is validated against the XML-1.0 legal
/// character set. A lang containing U+0000–U+0008, U+000B, U+000C, U+000E–U+001F,
/// U+FFFE, or U+FFFF returns [`PptxError::InvalidLanguageTag`] instead of emitting
/// malformed XML. Valid BCP-47 tags (ASCII alphanumeric + hyphen) pass through
/// unchanged (lossless — BC-5.01.005 invariant 1).
///
/// This is the bounded exception for string-built XML (referenced in ADR-001):
/// the OPC core properties namespace is not covered by `ooxmlsdk` schemas in
/// this story's scope. Escaping + validation is the correct mitigation.
fn build_doc_props(deck: &Deck, parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let lang_raw = deck.metadata.lang.as_deref().unwrap_or(DEFAULT_DECK_LANG);
    // SEC-039-001: validate before escaping — fail safe on XML-1.0-illegal chars.
    validate_lang_for_xml(lang_raw)?;
    // F-037-006: XML-escape the lang value before interpolating into the XML body.
    let lang = xml_escape(lang_raw);

    let core_xml = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
            "<cp:coreProperties",
            " xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\"",
            " xmlns:dc=\"http://purl.org/dc/elements/1.1/\"",
            " xmlns:dcterms=\"http://purl.org/dc/terms/\"",
            " xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n",
            "  <dc:creator>slideforge</dc:creator>\n",
            "  <dc:language>{lang}</dc:language>\n",
            "  <dcterms:created xsi:type=\"dcterms:W3CDTF\">1980-01-01T00:00:00Z</dcterms:created>\n",
            "</cp:coreProperties>"
        ),
        lang = lang,
    );
    parts.push(ZipPart {
        path: "docProps/core.xml".to_string(),
        bytes: core_xml.into_bytes(),
    });

    let app_xml: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>slideforge</Application>
  <AppVersion>0.1.0</AppVersion>
</Properties>"#;
    parts.push(ZipPart {
        path: "docProps/app.xml".to_string(),
        bytes: app_xml.to_vec(),
    });
    Ok(())
}

/// XML-escape a string for safe embedding in XML element text content.
///
/// Replaces the 5 XML-reserved characters:
/// - `&` → `&amp;` (must be first to avoid double-escaping)
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `"` → `&quot;`
/// - `'` → `&apos;`
///
/// Used by [`build_doc_props`] to escape the `dc:language` value (F-037-006).
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Build `_rels/.rels` (root relationships).
fn build_root_rels(parts: &mut Vec<ZipPart>) -> Result<(), PptxError> {
    let mut root_rels = RelsBuilder::new();
    root_rels.add(rel_types::OFFICE_DOCUMENT, "ppt/presentation.xml");
    root_rels.add(rel_types::CORE_PROPERTIES, "docProps/core.xml");
    root_rels.add(rel_types::EXTENDED_PROPERTIES, "docProps/app.xml");
    parts.push(ZipPart {
        path: "_rels/.rels".to_string(),
        bytes: root_rels.build()?,
    });
    Ok(())
}

/// Return `true` if a slide has at least one non-empty `Register::Notes` entry.
///
/// A slide "has notes" iff its `register_content` contains at least one entry
/// with `register == Register::Notes` whose plain-text content is non-empty
/// after whitespace trimming.
///
/// This is the single authoritative definition used by both `build_slide_parts`
/// (adds the notesSlide→slide relationship) and `build_notes_slide_parts`
/// (decides whether to emit a notesSlide part) — F-040-P1-001 same-set invariant.
fn slide_has_notes(slide: &LaidOutSlide) -> bool {
    slide.register_content.iter().any(|rc| {
        if rc.register != Register::Notes {
            return false;
        }
        // Check that the content is non-empty after whitespace trimming.
        // We flatten the inline tree to plain text for this check.
        let text = inline_nodes_to_plain_text(&rc.content);
        !text.trim().is_empty()
    })
}

/// Extract plain text from a slice of `InlineNode`s for whitespace-trimming checks.
fn inline_nodes_to_plain_text(nodes: &[slideforge_types::InlineNode]) -> String {
    use slideforge_types::InlineNode;
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Plain(s) | InlineNode::Code(s) | InlineNode::Xref(s) => {
                out.push_str(s);
            },
            InlineNode::Bold(c)
            | InlineNode::Italic(c)
            | InlineNode::Footnote(c)
            | InlineNode::Superscript(c)
            | InlineNode::Subscript(c)
            | InlineNode::Strikethrough(c)
            | InlineNode::Highlight(c) => {
                out.push_str(&inline_nodes_to_plain_text(c));
            },
            InlineNode::Link { text, .. } => {
                out.push_str(&inline_nodes_to_plain_text(text));
            },
            InlineNode::Math(m) => {
                out.push_str(m.latex.as_ref());
            },
        }
    }
    out
}

/// Return the MIME type for a media file based on its path extension.
fn content_type_for_media_path(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext.to_lowercase().as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

impl Exporter for PptxExporter {
    // The trait declares `&str`; returning a `'static` literal is compatible.
    // clippy::unnecessary_literal_bound is suppressed here because changing the
    // trait signature is out of scope for this story.
    #[allow(clippy::unnecessary_literal_bound)]
    fn id(&self) -> &str {
        "pptx"
    }

    #[allow(clippy::unnecessary_literal_bound)]
    fn extension(&self) -> &str {
        "pptx"
    }

    fn export(
        &self,
        deck: &Deck,
        laid_out: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        Self::export_inner(deck, laid_out, brand, opts).map_err(|e| ExportError::RenderError {
            message: e.to_string(),
        })
    }
}
