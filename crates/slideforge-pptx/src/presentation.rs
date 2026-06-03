//! `presentation.xml` serializer.
//!
//! [`PresentationSerializer`] produces the `ppt/presentation.xml` part using
//! the `ooxmlsdk` typed API. This is the root PresentationML part; it
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
//! an empty `<p:sectionLst/>` when sections are absent (stub behaviour
//! consistent with the BC).

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
    /// # Errors
    ///
    /// Returns [`PptxError::OoxmlElement`] if `ooxmlsdk` fails to serialise
    /// any required element.
    pub fn build(
        deck: &LaidOutDeck,
        brand: &Brand,
        slide_rel_ids: &[String],
        master_rel_id: &str,
    ) -> Result<Vec<u8>, PptxError> {
        todo!(
            "PresentationSerializer::build — construct CT_Presentation via ooxmlsdk, \
             populate sldIdLst (IDs from 256), sldMasterIdLst (ID=2^31), \
             page size from deck.page_size, lang from deck metadata"
        )
    }
}
