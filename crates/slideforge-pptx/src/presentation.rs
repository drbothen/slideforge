//! `presentation.xml` serializer.
//!
//! [`PresentationSerializer`] produces the `ppt/presentation.xml` part using
//! the `ooxmlsdk` typed API. This is the root `PresentationML` part; it
//! declares the slide ID list, slide master ID list, and (when present) the
//! section list.
//!
//! ## Slide ID assignment (BC-4.01.005)
//!
//! PPTX slide IDs must start at `256`. Some renderers corrupt the file when
//! slide IDs are below this threshold. Slide IDs are assigned sequentially:
//! `256`, `257`, `258`, …
//!
//! ## Master ID assignment (BC-4.01.005)
//!
//! The slide master ID is `2^31` (`2_147_483_648`). This value is required by
//! several renderers.
//!
//! ## Section list
//!
//! `PresentationSerializer::build` itself emits no `<p:extLst>` or
//! `<p14:sectionLst>` — it relies on the typed `Presentation` default, which
//! omits `sectionLst` (correct for decks with no section groups).
//! Section-list injection is performed downstream by
//! `SectionListBuilder::inject` (see `slideforge_pptx::sections`) after
//! `PresentationSerializer::build` produces the baseline bytes.

use ooxmlsdk::schemas::p::{
    HandoutMasterId, HandoutMasterIdList, NotesMasterId, NotesMasterIdList, Presentation, SlideId,
    SlideIdList, SlideMasterId, SlideMasterIdList, SlideSize,
};

use slideforge_layout::LaidOutDeck;
use slideforge_types::Brand;

use crate::error::PptxError;
use crate::slide_ids::{MASTER_ID, SlideIdAssigner};

/// Serializes `presentation.xml` from a `LaidOutDeck` and `Brand`.
///
/// Produces the raw bytes for `ppt/presentation.xml` using `ooxmlsdk`
/// typed builders. No string concatenation.
pub struct PresentationSerializer;

impl PresentationSerializer {
    /// Generate `ppt/presentation.xml` bytes.
    ///
    /// `slide_rel_ids` is the ordered list of `rId` strings assigned to each
    /// slide in `ppt/_rels/presentation.xml.rels`. The slide ID list in the
    /// output XML uses the same ordering: `slide_rel_ids[0]` gets ID 256,
    /// `slide_rel_ids[1]` gets ID 257, and so on.
    ///
    /// `master_rel_id` is the `rId` assigned to `slideMaster1.xml` in
    /// `ppt/_rels/presentation.xml.rels`.
    ///
    /// `notes_master_rel_id` is the `rId` for `notesMaster1.xml`.
    ///
    /// `handout_master_rel_id` is the `rId` for `handoutMaster1.xml`.
    ///
    /// # Errors
    ///
    /// Returns [`PptxError::InvalidEmu`] if `page_size.width.0` or
    /// `page_size.height.0` exceeds `i32::MAX` (slide size out of range).
    /// Silent clamping is forbidden (BC-4.01.005 invariant 1; CLAUDE.md
    /// Forbidden Patterns).
    ///
    /// Returns [`PptxError::OoxmlElement`] if `ooxmlsdk` fails to serialise
    /// any required element.
    ///
    /// # Panics
    ///
    /// Panics if `slide_rel_ids` has more than `u32::MAX` entries (impossible in practice:
    /// a deck cannot have 4 billion slides). Documented infallible path.
    pub fn build(
        deck: &LaidOutDeck,
        _brand: &Brand,
        slide_rel_ids: &[String],
        master_rel_id: &str,
        notes_master_rel_id: &str,
        handout_master_rel_id: &str,
    ) -> Result<Vec<u8>, PptxError> {
        // Validate page size: i32 range check (AC-012 / BC-4.01.005 invariant 1).
        // Silent clamping is forbidden (BC-4.01.005 invariant 1; CLAUDE.md Forbidden Patterns).
        let cx = i32::try_from(deck.page_size.width.0).map_err(|_| PptxError::InvalidEmu {
            slide_index: 0,
            frame_index: 0,
            detail: format!(
                "slide size width EMU {} exceeds i32::MAX ({}); \
                 cannot represent as OOXML cx attribute",
                deck.page_size.width.0,
                i32::MAX
            ),
        })?;
        let cy = i32::try_from(deck.page_size.height.0).map_err(|_| PptxError::InvalidEmu {
            slide_index: 0,
            frame_index: 0,
            detail: format!(
                "slide size height EMU {} exceeds i32::MAX ({}); \
                 cannot represent as OOXML cy attribute",
                deck.page_size.height.0,
                i32::MAX
            ),
        })?;

        // ooxmlsdk-generated types have many optional fields; the most readable
        // approach is to start from `default()` and set only the fields we need.
        #[allow(clippy::field_reassign_with_default)]
        let mut prs = {
            let mut p = Presentation::default();
            // Set required namespaces for presentation.xml.
            p.xmlns = vec![
                ooxmlsdk::common::XmlNamespaceDecl::new(
                    "a",
                    "http://schemas.openxmlformats.org/drawingml/2006/main",
                ),
                ooxmlsdk::common::XmlNamespaceDecl::new(
                    "r",
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                ),
                ooxmlsdk::common::XmlNamespaceDecl::new(
                    "p",
                    "http://schemas.openxmlformats.org/presentationml/2006/main",
                ),
            ];
            p
        };

        // Slide master ID list (always exactly one master, ID = 2^31).
        let master_id_list = SlideMasterIdList {
            p_sld_master_id: vec![SlideMasterId {
                id: Some(MASTER_ID),
                relationship_id: master_rel_id.to_string(),
                extension_list: None,
            }],
        };
        prs.slide_master_id_list = Some(master_id_list);

        // Notes master ID list (required — BC-4.01.006).
        prs.notes_master_id_list = Some(Box::new(NotesMasterIdList {
            notes_master_id: Some(Box::new(NotesMasterId {
                id: notes_master_rel_id.to_string(),
                extension_list: None,
            })),
        }));

        // Handout master ID list (required — BC-4.01.006).
        prs.handout_master_id_list = Some(Box::new(HandoutMasterIdList {
            handout_master_id: Some(Box::new(HandoutMasterId {
                id: handout_master_rel_id.to_string(),
                extension_list: None,
            })),
        }));

        // Slide ID list: IDs start at 256, one per slide.
        //
        // Routes through SlideIdAssigner::assign — the single authoritative
        // code path for slide ID generation (BC-4.01.005 invariant 1;
        // F-038-P1-M1). SlideIdAssigner encapsulates the SLIDE_ID_START offset
        // and the u32-fit precondition, keeping this caller free of ad-hoc
        // ID arithmetic.
        if !slide_rel_ids.is_empty() {
            let ids = SlideIdAssigner::assign(slide_rel_ids.len());
            let sld_id_list = SlideIdList {
                p_sld_id: slide_rel_ids
                    .iter()
                    .zip(ids)
                    .map(|(rid, id)| SlideId {
                        id,
                        relationship_id: rid.clone(),
                        extension_list: None,
                    })
                    .collect(),
            };
            prs.slide_id_list = Some(sld_id_list);
        }

        // Slide size: use the page_size from the LaidOutDeck.
        // cx and cy are validated above — i32 range check guarantees no silent clamp.
        prs.slide_size = Some(SlideSize {
            cx,
            cy,
            r#type: None,
        });

        // Notes size: use a standard 6858000 x 9144000 (portrait letter).
        *prs.notes_size = ooxmlsdk::schemas::p::NotesSize {
            cx: 6_858_000,
            cy: 9_144_000,
        };

        prs.to_xml_bytes().map_err(|e| PptxError::OoxmlElement {
            part: "ppt/presentation.xml".to_string(),
            detail: e.to_string(),
        })
    }
}
