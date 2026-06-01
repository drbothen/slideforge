//! Footer detection from OOXML `.pptx` slide master and layout XML.
//!
//! ## STORY-075 — Red Gate (test-only file, no production implementation)
//!
//! This file declares the **public API surface** that STORY-075 must implement.
//! Every symbol here is referenced by tests in `#[cfg(test)] mod tests` below.
//! The tests are the only authoritative specification at the code level —
//! no implementation exists yet. All tests must fail at the Red Gate.
//!
//! ## Expected API after implementation
//!
//! ```text
//! // Missing symbols that tests reference — listed for implementer orientation:
//!
//! pub struct FooterFlags {
//!     pub show_footer: bool,
//!     pub show_date: bool,
//!     pub show_slide_number: bool,
//! }
//! impl Default for FooterFlags { ... }  // all false
//!
//! pub struct FooterDetection {
//!     pub text: Option<Arc<str>>,
//!     pub flags: FooterFlags,
//! }
//! impl Default for FooterDetection { ... }
//!
//! pub fn detect_footer<R: Read + Seek>(
//!     zip: &mut ZipArchive<R>,
//!     is_pptx: bool,
//! ) -> FooterDetection
//! ```
//!
//! ## OOXML structures detected
//!
//! Footer placeholder in `ppt/slideMasters/slideMaster1.xml`:
//! ```xml
//! <p:sp>
//!   <p:nvSpPr><p:nvPr><p:ph type="ftr" sz="quarter" idx="11"/></p:nvPr></p:nvSpPr>
//!   <p:txBody><a:bodyPr/><a:lstStyle/>
//!     <a:p><a:r><a:t>Footer text content here</a:t></a:r></a:p>
//!   </p:txBody>
//! </p:sp>
//! ```
//!
//! Footer visibility flags in `ppt/presProps.xml`:
//! ```xml
//! <p:presentationPr>
//!   <p:showPr>
//!     <p:ftr val="1"/>
//!     <p:dt val="1"/>
//!     <p:sldNum val="1"/>
//!   </p:showPr>
//! </p:presentationPr>
//! ```

// NOTE: No production symbols are defined here yet.
// The test module below references `FooterFlags`, `FooterDetection`, and
// `detect_footer` — which do not exist. The Rust compiler will reject
// this file with "cannot find type/function in this scope" errors, proving
// Red Gate without requiring runtime execution.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    // ─────────────────────────────────────────────────────────────────────────
    // MISSING SYMBOLS: the following types and functions are referenced below
    // but are NOT defined anywhere in the codebase yet. This file will fail
    // to compile until the implementer provides them. That compile failure IS
    // the Red Gate for STORY-075.
    //
    //   FooterFlags              — in this module (footer.rs) or template.rs
    //   FooterDetection          — in this module (footer.rs)
    //   detect_footer()          — in this module (footer.rs)
    //   BrandTemplate.footer_flags — in template.rs (struct field extension)
    // ─────────────────────────────────────────────────────────────────────────

    use std::io::{Cursor, Write as _};
    use std::sync::Arc;

    use zip::CompressionMethod;
    use zip::write::{SimpleFileOptions, ZipWriter};

    // The crate re-exports FooterFlags from footer.rs (story task: lib.rs pub use).
    // Until STORY-075 is implemented, this import itself is the Red Gate:
    // "error[E0432]: unresolved import `crate::footer::FooterFlags`"
    use crate::footer::{FooterDetection, FooterFlags, detect_footer};
    use crate::template::BrandTemplate;

    // ─── ZIP fixture builders ─────────────────────────────────────────────────

    /// Minimal `theme1.xml` embedded in test ZIPs so PPTX detection succeeds.
    const MINIMAL_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="TestTheme">
  <a:themeElements>
    <a:clrScheme name="TestScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"/></a:dk2>
      <a:lt2><a:srgbClr val="F5F5F5"/></a:lt2>
      <a:acc1><a:srgbClr val="0066CC"/></a:acc1>
      <a:acc2><a:srgbClr val="FF6B35"/></a:acc2>
      <a:acc3><a:srgbClr val="28A745"/></a:acc3>
      <a:acc4><a:srgbClr val="FFC107"/></a:acc4>
      <a:acc5><a:srgbClr val="6F42C1"/></a:acc5>
      <a:acc6><a:srgbClr val="17A2B8"/></a:acc6>
      <a:hlink><a:srgbClr val="0000EE"/></a:hlink>
      <a:folHlink><a:srgbClr val="551A8B"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="TestFontScheme">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    /// Minimal slide master XML with a populated footer placeholder.
    ///
    /// Contains `<p:ph type="ftr"/>` with `<a:t>Confidential</a:t>`.
    const MASTER_WITH_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" sz="quarter" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p>
          <a:r><a:t>Confidential</a:t></a:r>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with a footer placeholder but EMPTY text run.
    const MASTER_WITH_EMPTY_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" sz="quarter" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p>
          <a:r><a:t></a:t></a:r>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with NO footer placeholder at all.
    const MASTER_NO_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="title"/>
        </p:nvPr>
      </p:nvSpPr>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide layout XML with a populated footer placeholder — used as fallback.
    const LAYOUT_WITH_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" sz="quarter" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p>
          <a:r><a:t>Q4 Report</a:t></a:r>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldLayout>"#;

    /// Slide layout XML with an empty footer placeholder (no usable text).
    const LAYOUT_EMPTY_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p>
          <a:r><a:t></a:t></a:r>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldLayout>"#;

    /// `presProps.xml` with footer visible (val="1"), slide number visible, date hidden.
    const PRESPROPS_FOOTER_AND_SLDNUM_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentationPr xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:showPr>
    <p:ftr val="1"/>
    <p:sldNum val="1"/>
  </p:showPr>
</p:presentationPr>"#;

    /// `presProps.xml` with all three flags set to "1".
    const PRESPROPS_ALL_FLAGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentationPr xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:showPr>
    <p:ftr val="1"/>
    <p:dt val="1"/>
    <p:sldNum val="1"/>
  </p:showPr>
</p:presentationPr>"#;

    /// `presProps.xml` with no `<p:showPr>` child — flags should default to false.
    const PRESPROPS_NO_SHOWPR_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentationPr xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
</p:presentationPr>"#;

    /// Slide master XML with TWO footer placeholders (EC-003: first wins).
    const MASTER_TWO_FOOTER_PLACEHOLDERS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" idx="11"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p><a:r><a:t>First Footer</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" idx="12"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p><a:r><a:t>Second Footer</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML where the footer placeholder contains a `<a:fld>` field element
    /// instead of `<a:r>` text runs (EC-006: treat as no text).
    const MASTER_FOOTER_FIELD_ELEMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" idx="11"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p>
          <a:fld id="{GUID}" type="datetimeFigureOut">
            <a:t>Date</a:t>
          </a:fld>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    // ─── In-memory ZIP builders ───────────────────────────────────────────────

    /// Build an in-memory ZIP with the given entries.
    ///
    /// Each entry is `(zip_path, content_bytes)`.
    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zw = ZipWriter::new(cursor);
            let opts =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
            for (path, content) in entries {
                zw.start_file(*path, opts).unwrap();
                zw.write_all(content).unwrap();
            }
            zw.finish().unwrap();
        }
        buf
    }

    /// Build a minimal PPTX ZIP (just `ppt/theme/theme1.xml` and optional extras).
    fn build_pptx_zip_with_extras(extras: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8])> = vec![
            ("ppt/theme/theme1.xml", MINIMAL_THEME_XML.as_bytes()),
            (
                "[Content_Types].xml",
                b"<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>",
            ),
        ];
        entries.extend_from_slice(extras);
        build_zip(&entries)
    }

    /// Write bytes to a uniquely-named temp file; return the path.
    fn write_temp(bytes: &[u8], ext: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let seq = CTR.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            "slideforge_s075_test_{}_{}_{}_{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq,
            "footer",
            ext
        );
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        f.sync_all().unwrap();
        path
    }

    // ─── FooterFlags unit tests ───────────────────────────────────────────────

    /// BC-2.01.001 AC-003 — `FooterFlags::default()` has all fields false.
    ///
    /// presProps.xml absent → `FooterFlags { show_footer: false, show_date: false,
    /// show_slide_number: false }` (AC-005 / EC-004).
    #[test]
    fn test_bc_2_01_001_footer_flags_default_all_false() {
        let flags = FooterFlags::default();
        assert!(
            !flags.show_footer,
            "FooterFlags::default().show_footer must be false"
        );
        assert!(
            !flags.show_date,
            "FooterFlags::default().show_date must be false"
        );
        assert!(
            !flags.show_slide_number,
            "FooterFlags::default().show_slide_number must be false"
        );
    }

    /// BC-2.01.001 AC-003 — `FooterFlags` struct has the three required boolean fields.
    ///
    /// Verifies the struct can be constructed with all three fields set to specific values,
    /// and that field reads are correct. Non-tautological: checks each field independently.
    #[test]
    fn test_bc_2_01_001_footer_flags_struct_fields() {
        let flags = FooterFlags {
            show_footer: true,
            show_date: false,
            show_slide_number: true,
        };
        assert!(flags.show_footer, "show_footer must be true as set");
        assert!(!flags.show_date, "show_date must be false as set");
        assert!(
            flags.show_slide_number,
            "show_slide_number must be true as set"
        );

        let all_true = FooterFlags {
            show_footer: true,
            show_date: true,
            show_slide_number: true,
        };
        assert!(all_true.show_footer);
        assert!(all_true.show_date);
        assert!(all_true.show_slide_number);

        let all_false = FooterFlags {
            show_footer: false,
            show_date: false,
            show_slide_number: false,
        };
        assert!(!all_false.show_footer);
        assert!(!all_false.show_date);
        assert!(!all_false.show_slide_number);
    }

    // ─── detect_footer — PPTX master footer text extraction ──────────────────

    /// BC-2.01.001 AC-001 — footer placeholder with populated `<a:t>` in master
    /// produces `FooterDetection.text = Some("Confidential")`.
    ///
    /// Test vector from story Test Strategy table: "Confidential" footer.
    #[test]
    fn test_bc_2_01_001_detect_footer_text_from_master() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_WITH_FOOTER_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert_eq!(
            detection.text.as_deref(),
            Some("Confidential"),
            "AC-001: master footer placeholder text 'Confidential' must be detected; \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 AC-002 — master has empty footer, layout1.xml has "Q4 Report".
    ///
    /// Test vector from story Test Strategy table: empty master → fallback to layout.
    #[test]
    fn test_bc_2_01_001_detect_footer_empty_master_fallback_layout() {
        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_EMPTY_FOOTER_XML.as_bytes(),
            ),
            (
                "ppt/slideLayouts/slideLayout1.xml",
                LAYOUT_WITH_FOOTER_XML.as_bytes(),
            ),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert_eq!(
            detection.text.as_deref(),
            Some("Q4 Report"),
            "AC-002: when master footer is empty, must fall back to layout1.xml text; \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 AC-001 — no `<p:ph type=\"ftr\"/>` in master → `text = None`.
    ///
    /// Test vector from story Test Strategy table: absent placeholder.
    #[test]
    fn test_bc_2_01_001_detect_footer_absent_placeholder() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_NO_FOOTER_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.text.is_none(),
            "AC-001: master with no footer placeholder must return text = None; \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 AC-005 / EC-001 — `slideMaster1.xml` absent from ZIP.
    ///
    /// Must not panic. Returns `text = None` silently.
    #[test]
    fn test_bc_2_01_001_detect_footer_absent_master_xml() {
        // ZIP has no slideMaster1.xml entry at all.
        let zip_bytes = build_pptx_zip_with_extras(&[]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        // Must not panic — AC-005 says skip silently.
        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.text.is_none(),
            "AC-005/EC-001: absent slideMaster1.xml must yield text = None (no panic); \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 AC-003 / EC-004 — `presProps.xml` absent → `FooterFlags::default()`.
    ///
    /// Test vector from story Test Strategy table: absent presProps.xml → all false flags.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_absent_presprops() {
        // ZIP has a master but NO presProps.xml.
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_WITH_FOOTER_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        let default_flags = FooterFlags::default();
        assert_eq!(
            detection.flags.show_footer, default_flags.show_footer,
            "AC-003/EC-004: absent presProps.xml must yield show_footer = false"
        );
        assert_eq!(
            detection.flags.show_date, default_flags.show_date,
            "AC-003/EC-004: absent presProps.xml must yield show_date = false"
        );
        assert_eq!(
            detection.flags.show_slide_number, default_flags.show_slide_number,
            "AC-003/EC-004: absent presProps.xml must yield show_slide_number = false"
        );
    }

    /// BC-2.01.001 AC-003 — `presProps.xml` with `<p:ftr val=\"1\"/>` and `<p:sldNum val=\"1\"/>`.
    ///
    /// Test vector from story Test Strategy table: `show_footer=true, show_date=false,
    /// show_slide_number=true`.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_from_presprops() {
        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_FOOTER_XML.as_bytes(),
            ),
            (
                "ppt/presProps.xml",
                PRESPROPS_FOOTER_AND_SLDNUM_XML.as_bytes(),
            ),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "AC-003: <p:ftr val=\"1\"/> must set show_footer = true; \
             got show_footer={}",
            detection.flags.show_footer
        );
        assert!(
            !detection.flags.show_date,
            "AC-003: no <p:dt> in presProps must leave show_date = false; \
             got show_date={}",
            detection.flags.show_date
        );
        assert!(
            detection.flags.show_slide_number,
            "AC-003: <p:sldNum val=\"1\"/> must set show_slide_number = true; \
             got show_slide_number={}",
            detection.flags.show_slide_number
        );
    }

    /// BC-2.01.001 AC-003 — all three flags parsed from `presProps.xml`.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_all_three_from_presprops() {
        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_FOOTER_XML.as_bytes(),
            ),
            ("ppt/presProps.xml", PRESPROPS_ALL_FLAGS_XML.as_bytes()),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "show_footer must be true (val=1)"
        );
        assert!(detection.flags.show_date, "show_date must be true (val=1)");
        assert!(
            detection.flags.show_slide_number,
            "show_slide_number must be true (val=1)"
        );
    }

    /// BC-2.01.001 AC-003 / EC-007 — `presProps.xml` present but no `<p:showPr>` element.
    ///
    /// All flags must default to false (EC-007).
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_presprops_no_showpr() {
        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_FOOTER_XML.as_bytes(),
            ),
            ("ppt/presProps.xml", PRESPROPS_NO_SHOWPR_XML.as_bytes()),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            !detection.flags.show_footer,
            "EC-007: presProps with no <p:showPr> must yield show_footer = false"
        );
        assert!(
            !detection.flags.show_date,
            "EC-007: presProps with no <p:showPr> must yield show_date = false"
        );
        assert!(
            !detection.flags.show_slide_number,
            "EC-007: presProps with no <p:showPr> must yield show_slide_number = false"
        );
    }

    /// BC-2.01.001 AC-006 / EC-005 — DOCX input: `is_pptx = false` → defaults immediately.
    ///
    /// No XML reads attempted for DOCX. text = None, flags = default.
    #[test]
    fn test_bc_2_01_001_detect_footer_docx_returns_default() {
        // DOCX ZIP (word/theme/theme1.xml) — should be ignored entirely.
        let zip_bytes = build_zip(&[
            ("word/theme/theme1.xml", MINIMAL_THEME_XML.as_bytes()),
            // Include a fake master XML to verify it is NOT read for DOCX.
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_FOOTER_XML.as_bytes(),
            ),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, false); // is_pptx = false

        assert!(
            detection.text.is_none(),
            "AC-006/EC-005: DOCX input must return text = None immediately; \
             got: {:?}",
            detection.text
        );
        let defaults = FooterFlags::default();
        assert_eq!(
            detection.flags.show_footer, defaults.show_footer,
            "AC-006/EC-005: DOCX input must return show_footer = false"
        );
        assert_eq!(
            detection.flags.show_date, defaults.show_date,
            "AC-006/EC-005: DOCX input must return show_date = false"
        );
        assert_eq!(
            detection.flags.show_slide_number, defaults.show_slide_number,
            "AC-006/EC-005: DOCX input must return show_slide_number = false"
        );
    }

    /// BC-2.01.001 EC-003 — multiple footer placeholders in master: first one wins.
    ///
    /// Document-order traversal: "First Footer" before "Second Footer".
    #[test]
    fn test_bc_2_01_001_detect_footer_multiple_placeholders_first_wins() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_TWO_FOOTER_PLACEHOLDERS_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert_eq!(
            detection.text.as_deref(),
            Some("First Footer"),
            "EC-003: when multiple footer placeholders exist, first in document order wins; \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 EC-006 — footer placeholder contains `<a:fld>` field element,
    /// NOT `<a:r>` text run: treat as no text.
    ///
    /// The field element is a date/time or numbering auto-field — no static text
    /// to extract.
    #[test]
    fn test_bc_2_01_001_detect_footer_field_element_treated_as_no_text() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_FOOTER_FIELD_ELEMENT_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.text.is_none(),
            "EC-006: footer placeholder with <a:fld> (no <a:r>) must yield text = None; \
             got: {:?}",
            detection.text
        );
    }

    /// BC-2.01.001 AC-002 — both master and layout1 have empty footers: `text = None`.
    ///
    /// When master footer is empty AND layout1 footer is also empty, the result
    /// must be `None` — not an empty `Some("")`.
    #[test]
    fn test_bc_2_01_001_detect_footer_both_empty_yields_none() {
        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_EMPTY_FOOTER_XML.as_bytes(),
            ),
            (
                "ppt/slideLayouts/slideLayout1.xml",
                LAYOUT_EMPTY_FOOTER_XML.as_bytes(),
            ),
        ]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.text.is_none(),
            "AC-002: when both master and layout footer are empty, text must be None \
             (not Some(\"\")). Got: {:?}",
            detection.text
        );
    }

    /// EC-002 — empty `<a:t>` in master, no layout1.xml in ZIP: `text = None`.
    ///
    /// Fallback is attempted; if layout1 is absent the result is None.
    #[test]
    fn test_bc_2_01_001_detect_footer_empty_master_no_layout_yields_none() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_WITH_EMPTY_FOOTER_XML.as_bytes(),
        )]);
        // No slideLayout1.xml in this ZIP.
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.text.is_none(),
            "EC-002: empty master footer with absent layout1 must yield text = None; \
             got: {:?}",
            detection.text
        );
    }

    // ─── BrandTemplate.footer_flags integration ───────────────────────────────

    /// BC-2.01.001 AC-003 — `BrandTemplate` struct has `footer_flags: FooterFlags` field.
    ///
    /// This test exercises the field on the struct directly. It fails to compile
    /// until `footer_flags` is added to `BrandTemplate` in `template.rs`.
    ///
    /// Anti-paper-fix (TD-VSDD-059): asserts the specific field value, not just
    /// that the struct compiles.
    #[test]
    fn test_bc_2_01_001_brand_template_has_footer_flags_field() {
        use crate::template::{BrandFonts, BrandTemplate, ColorSlot, ColorValue, MasterIds};
        fn make_color_slot(name: &str, hex: &str) -> ColorSlot {
            ColorSlot {
                name: Arc::from(name),
                value: ColorValue::Hex(Arc::from(hex)),
            }
        }

        let template = BrandTemplate {
            colors: [
                make_color_slot("dk1", "#000000"),
                make_color_slot("lt1", "#FFFFFF"),
                make_color_slot("dk2", "#003087"),
                make_color_slot("lt2", "#F5F5F5"),
                make_color_slot("acc1", "#0066CC"),
                make_color_slot("acc2", "#FF6B35"),
                make_color_slot("acc3", "#28A745"),
                make_color_slot("acc4", "#FFC107"),
                make_color_slot("acc5", "#6F42C1"),
                make_color_slot("acc6", "#17A2B8"),
                make_color_slot("hlink", "#0000EE"),
                make_color_slot("folHlink", "#551A8B"),
            ],
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            // This field does NOT exist yet → compile error until STORY-075 is implemented.
            footer_flags: FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        // Verify the field is readable and has the expected default value.
        assert!(
            !template.footer_flags.show_footer,
            "BrandTemplate.footer_flags.show_footer must default to false"
        );
        assert!(
            !template.footer_flags.show_date,
            "BrandTemplate.footer_flags.show_date must default to false"
        );
        assert!(
            !template.footer_flags.show_slide_number,
            "BrandTemplate.footer_flags.show_slide_number must default to false"
        );
    }

    // ─── BrandLoader integration (through-load tests) ─────────────────────────

    /// BC-2.01.001 AC-001 / AC-004 — `BrandLoader::load_template()` on PPTX with
    /// footer placeholder returns `BrandTemplate.footer_text = Some("Acme Corp Confidential")`.
    ///
    /// This is the primary integration path: detect_footer() called inside
    /// load_template() before BrandTemplate is returned.
    ///
    /// Test vector from story Integration Tests table: "Acme Corp Confidential".
    #[test]
    fn test_bc_2_01_001_load_pptx_with_footer() {
        use crate::context::BrandLoadContext;
        use crate::loader::BrandLoader;

        let master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" sz="quarter" idx="11"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p><a:r><a:t>Acme Corp Confidential</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            master_xml.as_bytes(),
        )]);
        let path = write_temp(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("PPTX with footer must load without error");

        assert_eq!(
            template.footer_text.as_deref(),
            Some("Acme Corp Confidential"),
            "AC-001/AC-004: BrandLoader must populate footer_text from master placeholder; \
             got: {:?}",
            template.footer_text
        );
    }

    /// BC-2.01.001 AC-004 — `BrandLoader::load_template()` on minimal PPTX
    /// (no footer placeholder) returns `BrandTemplate.footer_text = None`.
    ///
    /// Verifies the no-footer path is still correct after AC-001 implementation.
    #[test]
    fn test_bc_2_01_001_load_pptx_without_footer() {
        use crate::context::BrandLoadContext;
        use crate::loader::BrandLoader;

        // Minimal PPTX with no slideMaster1.xml at all.
        let zip_bytes = build_pptx_zip_with_extras(&[]);
        let path = write_temp(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("minimal PPTX must load without error");

        assert!(
            template.footer_text.is_none(),
            "AC-004: PPTX without footer placeholder must return footer_text = None; \
             got: {:?}",
            template.footer_text
        );
    }

    /// BC-2.01.001 AC-004 — `BrandLoader::load_template()` on PPTX with footer
    /// also populates `footer_flags` from `presProps.xml`.
    ///
    /// Verifies the flags field is set on the returned BrandTemplate.
    #[test]
    fn test_bc_2_01_001_load_pptx_footer_flags_populated_from_presprops() {
        use crate::context::BrandLoadContext;
        use crate::loader::BrandLoader;

        let master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" idx="11"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>Acme</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

        let zip_bytes = build_pptx_zip_with_extras(&[
            (
                "ppt/slideMasters/slideMaster1.xml",
                master_xml.as_bytes(),
            ),
            (
                "ppt/presProps.xml",
                PRESPROPS_FOOTER_AND_SLDNUM_XML.as_bytes(),
            ),
        ]);
        let path = write_temp(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("PPTX with presProps must load without error");

        // footer_flags should reflect presProps.xml (ftr=1, sldNum=1, dt absent).
        assert!(
            template.footer_flags.show_footer,
            "AC-004: footer_flags.show_footer must be true from presProps.xml <p:ftr val=1/>"
        );
        assert!(
            !template.footer_flags.show_date,
            "AC-004: footer_flags.show_date must be false (no <p:dt> in presProps)"
        );
        assert!(
            template.footer_flags.show_slide_number,
            "AC-004: footer_flags.show_slide_number must be true from presProps.xml <p:sldNum val=1/>"
        );
    }

    /// BC-2.01.001 AC-006 — `BrandLoader::load_template()` on DOCX returns
    /// `footer_text = None` and `footer_flags = FooterFlags::default()`.
    ///
    /// DOCX templates have no slide master footer placeholders.
    #[test]
    fn test_bc_2_01_001_load_docx_footer_always_none() {
        use crate::context::BrandLoadContext;
        use crate::loader::BrandLoader;

        let zip_bytes = build_zip(&[
            ("word/theme/theme1.xml", MINIMAL_THEME_XML.as_bytes()),
            (
                "[Content_Types].xml",
                b"<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>",
            ),
            // Include a master XML to verify it is NOT read for DOCX.
            (
                "ppt/slideMasters/slideMaster1.xml",
                MASTER_WITH_FOOTER_XML.as_bytes(),
            ),
        ]);
        let path = write_temp(&zip_bytes, "docx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("DOCX must load without error");

        assert!(
            template.footer_text.is_none(),
            "AC-006: DOCX must return footer_text = None regardless of any XML; \
             got: {:?}",
            template.footer_text
        );
        assert!(
            !template.footer_flags.show_footer,
            "AC-006: DOCX must return footer_flags.show_footer = false"
        );
        assert!(
            !template.footer_flags.show_date,
            "AC-006: DOCX must return footer_flags.show_date = false"
        );
        assert!(
            !template.footer_flags.show_slide_number,
            "AC-006: DOCX must return footer_flags.show_slide_number = false"
        );
    }

    // ─── BrandExtractor end-to-end (F-024A-OBS-1 closure test) ──────────────

    /// BC-2.01.001 AC-007 — end-to-end: load PPTX with footer → extract →
    /// `brand.toml` contains `[footer]` section with `text = "Acme Corp Confidential"`.
    ///
    /// This test closes adversary finding F-024A-OBS-1: the [footer] writer in
    /// BrandExtractor was permanently unreachable before STORY-075 because
    /// BrandLoader always returned footer_text: None.
    ///
    /// The test exercises the full pipeline:
    ///   BrandLoader::load_template() → BrandExtractor::extract() → brand.toml
    #[test]
    fn test_bc_2_01_001_extract_brand_toml_includes_footer_section() {
        use crate::extractor::BrandExtractor;

        let master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr><p:ph type="ftr" sz="quarter" idx="11"/></p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/>
        <a:lstStyle/>
        <a:p><a:r><a:t>Acme Corp Confidential</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            master_xml.as_bytes(),
        )]);
        let pptx_path = write_temp(&zip_bytes, "pptx");

        // Create a temporary output directory.
        let output_dir = {
            use std::sync::atomic::{AtomicU64, Ordering};
            static CTR2: AtomicU64 = AtomicU64::new(100);
            let seq = CTR2.fetch_add(1, Ordering::Relaxed);
            let d = std::env::temp_dir().join(format!("s075_extract_{seq}"));
            std::fs::create_dir_all(&d).unwrap();
            d
        };

        // Run extractor directly from the PPTX path — BrandExtractor::extract()
        // re-loads the template internally. The PPTX file must still exist when called.
        let result = BrandExtractor::extract(
            pptx_path.to_str().unwrap(),
            &output_dir,
            false,
        )
        .expect("extract must succeed for valid template with footer");
        let _ = std::fs::remove_file(&pptx_path);

        let brand_toml_path = output_dir.join("brand.toml");
        let brand_toml_content =
            std::fs::read_to_string(&brand_toml_path).expect("brand.toml must be written");

        // Verify [footer] section is present and contains the expected text.
        assert!(
            brand_toml_content.contains("[footer]"),
            "AC-007: brand.toml must contain [footer] section when footer_text is populated; \
             content:\n{brand_toml_content}"
        );
        assert!(
            brand_toml_content.contains("Acme Corp Confidential"),
            "AC-007: brand.toml [footer] section must contain 'Acme Corp Confidential'; \
             content:\n{brand_toml_content}"
        );

        // Check the extraction result indicates footer was written.
        let _ = result;

        // Cleanup.
        let _ = std::fs::remove_dir_all(&output_dir);
    }
}
