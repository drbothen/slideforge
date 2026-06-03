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
//! ## Section list (STORY-040 deferred)
//!
//! Full section list population is deferred to STORY-040. This module emits
//! no section list when sections are absent (empty list is the default).

use ooxmlsdk::schemas::p::{
    HandoutMasterId, HandoutMasterIdList, NotesMasterId, NotesMasterIdList, Presentation, SlideId,
    SlideIdList, SlideMasterId, SlideMasterIdList, SlideSize,
};

use slideforge_layout::LaidOutDeck;
use slideforge_types::Brand;

use crate::error::PptxError;

/// The minimum slide ID permitted by the PPTX spec (BC-4.01.005).
pub const SLIDE_ID_START: u32 = 256;

/// The slide master ID required by several renderers (BC-4.01.005 / BUG-006).
pub const MASTER_ID: u32 = 2_147_483_648;

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
    /// Returns [`PptxError::OoxmlElement`] if `ooxmlsdk` fails to serialise
    /// any required element.
    pub fn build(
        deck: &LaidOutDeck,
        _brand: &Brand,
        slide_rel_ids: &[String],
        master_rel_id: &str,
        notes_master_rel_id: &str,
        handout_master_rel_id: &str,
    ) -> Result<Vec<u8>, PptxError> {
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
        if !slide_rel_ids.is_empty() {
            let sld_id_list = SlideIdList {
                p_sld_id: slide_rel_ids
                    .iter()
                    .enumerate()
                    .map(|(i, rid)| SlideId {
                        id: SLIDE_ID_START + u32::try_from(i).unwrap_or(0),
                        relationship_id: rid.clone(),
                        extension_list: None,
                    })
                    .collect(),
            };
            prs.slide_id_list = Some(sld_id_list);
        }

        // Slide size: use the page_size from the LaidOutDeck.
        // The PPTX exporter emits the layout-engine canvas size here.
        // Future stories may apply a scaling factor for exact PPTX native dims.
        let cx = i32::try_from(deck.page_size.width.0).unwrap_or(9_144_000_i32);
        let cy = i32::try_from(deck.page_size.height.0).unwrap_or(5_143_500_i32);
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
