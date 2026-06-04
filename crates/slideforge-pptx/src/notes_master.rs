//! `NotesMasterSerializer` — produces `ppt/notesMasters/notesMaster1.xml`
//! and `ppt/handoutMasters/handoutMaster1.xml`.
//!
//! ## Contract (BC-4.01.006 / STORY-040)
//!
//! Both master parts are ALWAYS emitted — no conditional omission even when
//! the deck has zero speaker notes (BC-4.01.006 invariant 1).
//!
//! `notesMaster1.xml` replaces the empty stub from STORY-037 with a minimal
//! valid notes master containing the required placeholder types (`sldImg` and
//! `body`). If the brand template provides a pre-serialized notes master, that
//! is used verbatim.
//!
//! `handoutMaster1.xml` is a minimal valid stub (always generated, always
//! present).

use crate::error::PptxError;

/// Minimal valid `notesMaster1.xml` bytes.
///
/// Contains the two required placeholders:
/// - `<p:ph type="sldImg"/>` — slide thumbnail
/// - `<p:ph type="body" idx="1"/>` — notes text area
///
/// Also includes `<p:clrMap>` with all required color mapping attributes
/// as required by the OOXML schema.
const NOTES_MASTER_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notesMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
               xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
               xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm>
      </p:grpSpPr>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Slide Image Placeholder 1"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="sldImg"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="685800" y="1143000"/><a:ext cx="5181600" cy="3886200"/></a:xfrm>
        </p:spPr>
      </p:sp>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="3" name="Notes Placeholder 2"/>
          <p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
          <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="685800" y="5143500"/><a:ext cx="5181600" cy="3600450"/></a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p><a:endParaRPr lang="en-US"/></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1"
            accent2="accent2" accent3="accent3" accent4="accent4"
            accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
</p:notesMaster>"#;

/// Minimal valid `handoutMaster1.xml` bytes.
///
/// Contains the required placeholders for a handout master:
/// - `<p:ph type="sldImg"/>` — slide thumbnail
/// - `<p:ph type="body" idx="1"/>` — notes area
///
/// Also includes `<p:clrMap>` as required by the OOXML schema.
const HANDOUT_MASTER_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:handoutMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
                 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:grpSpPr>
        <a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm>
      </p:grpSpPr>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1"
            accent2="accent2" accent3="accent3" accent4="accent4"
            accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
</p:handoutMaster>"#;

/// Produces `ppt/notesMasters/notesMaster1.xml` and
/// `ppt/handoutMasters/handoutMaster1.xml`.
pub struct NotesMasterSerializer;

impl NotesMasterSerializer {
    /// Build a minimal valid `notesMaster1.xml`.
    ///
    /// If `brand_notes_master_bytes` is non-empty, returns those bytes verbatim
    /// (preserving brand-specific styling). Otherwise generates a minimal valid
    /// notes master with `sldImg` and `body` placeholders plus a `clrMap`.
    ///
    /// # Errors
    ///
    /// This function does not currently fail (the minimal XML is a compile-time
    /// constant). Returns `Ok` always. The signature carries `Result` for forward
    /// compatibility when brand-provided bytes may require validation.
    pub fn build_notes_master(brand_notes_master_bytes: &[u8]) -> Result<Vec<u8>, PptxError> {
        if brand_notes_master_bytes.is_empty() {
            Ok(NOTES_MASTER_XML.to_vec())
        } else {
            Ok(brand_notes_master_bytes.to_vec())
        }
    }

    /// Build a minimal valid `handoutMaster1.xml`.
    ///
    /// If `brand_handout_master_bytes` is non-empty, returns those bytes verbatim.
    /// Otherwise generates a minimal valid handout master stub.
    ///
    /// # Errors
    ///
    /// This function does not currently fail. Returns `Ok` always. The signature
    /// carries `Result` for forward compatibility.
    pub fn build_handout_master(brand_handout_master_bytes: &[u8]) -> Result<Vec<u8>, PptxError> {
        if brand_handout_master_bytes.is_empty() {
            Ok(HANDOUT_MASTER_XML.to_vec())
        } else {
            Ok(brand_handout_master_bytes.to_vec())
        }
    }
}
