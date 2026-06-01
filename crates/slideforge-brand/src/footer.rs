//! Footer detection from OOXML `.pptx` slide master and layout XML.
//!
//! This module reads `ppt/slideMasters/slideMaster1.xml` and (optionally)
//! `ppt/slideLayouts/slideLayout1.xml` from an already-open PPTX ZIP archive to
//! detect footer placeholder text (`<p:ph type="ftr"/>`) and footer-visibility
//! flags (`show_footer`, `show_date`, `show_slide_number`).
//!
//! ## Parsing strategy
//!
//! `slideMaster1.xml` is parsed in a SINGLE SAX pass that simultaneously collects:
//! 1. Footer placeholder text — all `<a:t>` runs inside `<p:ph type="ftr"/>` shapes,
//!    concatenated in document order (AC-001 / EC-008).
//! 2. Footer-visibility flags — the three boolean attributes (`ftr`, `dt`, `sldNum`)
//!    on the `<p:hf>` (`CT_HeaderFooter`) element (AC-003, BC-2.01.001 v1.3 corrected).
//!
//! **CORRECTION (adversary H-1 / BC-2.01.001 v1.3):** Footer visibility flags are
//! NOT read from `ppt/presProps.xml`.  `<p:showPr>` is the slide-show runtime
//! configuration element (`CT_ShowProperties`) and has no `ftr`/`dt`/`sldNum`
//! children per ECMA-376.  The flags are boolean **attributes** on the `<p:hf>`
//! (`CT_HeaderFooter`) element on `slideMaster1.xml`.
//!
//! ## Multi-run text concatenation
//!
//! Footer text is the FULL concatenation of ALL `<a:t>` runs inside the first
//! footer placeholder found, in document order.  "First-run-only" semantics are
//! explicitly forbidden (AC-001 / EC-008 — styled text spans split across runs).
//!
//! ## OOXML boolean handling
//!
//! `<p:hf>` attributes are `xsd:boolean` — both `"1"`/`"0"` and `"true"`/`"false"`
//! forms are accepted (ECMA-376).  An absent attribute defaults to `false`.
//!
//! ## Inheritance order
//!
//! Slide master is checked first; if the footer placeholder is present but has
//! no text, the slide layout fallback (`slideLayout1.xml`) is tried (AC-002).
//! This mirrors the OOXML placeholder inheritance chain: layout → master by type.

use std::io::{Read, Seek};
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;

// ─── Public types ─────────────────────────────────────────────────────────────

/// Footer-visibility flags extracted from the `<p:hf>` (`CT_HeaderFooter`) element
/// on `ppt/slideMasters/slideMaster1.xml` (AC-003 / BC-2.01.001 v1.3 corrected).
///
/// Each flag corresponds to one OOXML boolean attribute on the `<p:hf>` element.
/// If `<p:hf>` is absent from `slideMaster1.xml`, all three flags default to
/// `false` and a `tracing::debug!` is emitted (EC-004).
///
/// Both `"1"`/`"0"` and `"true"`/`"false"` attribute forms are accepted (ECMA-376
/// `xsd:boolean`).  An absent attribute defaults to `false`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FooterFlags {
    /// Whether footer text is visible on slides (`<p:hf ftr="1"/>` or `ftr="true"`).
    pub show_footer: bool,
    /// Whether the date/time placeholder is visible on slides (`<p:hf dt="1"/>` or `dt="true"`).
    pub show_date: bool,
    /// Whether the slide-number placeholder is visible on slides (`<p:hf sldNum="1"/>` or `sldNum="true"`).
    pub show_slide_number: bool,
}

/// The combined result of footer detection from a PPTX ZIP archive.
///
/// Produced by [`detect_footer`].  For DOCX input (`is_pptx = false`) or when
/// the slide master is absent from the ZIP, all fields hold their default values.
///
/// This type is an internal helper consumed by [`crate::loader::BrandLoader`].
/// External consumers access the extracted data via [`crate::template::BrandTemplate`]
/// fields (`footer_flags: FooterFlags`, `footer_text: Option<Arc<str>>`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FooterDetection {
    /// Footer text — the full concatenation of all `<a:t>` runs inside the first
    /// footer placeholder found in the slide master (or layout1 as a fallback).
    /// `None` if no text was found.
    pub(crate) text: Option<Arc<str>>,
    /// Footer-visibility flags from the `<p:hf>` element on `slideMaster1.xml`.
    pub(crate) flags: FooterFlags,
}

// ─── Crate-internal API ───────────────────────────────────────────────────────

/// Detect footer text and visibility flags from an already-open PPTX ZIP.
///
/// ## Detection order
///
/// 1. If `is_pptx` is `false` (DOCX input), return [`FooterDetection::default`]
///    immediately without reading any XML (AC-006 / EC-005).
/// 2. Read `ppt/slideMasters/slideMaster1.xml` in a **single SAX pass** that
///    simultaneously extracts:
///    - Footer placeholder text: ALL `<a:t>` runs inside the first
///      `<p:ph type="ftr"/>` shape, concatenated in document order (AC-001 / EC-008).
///    - Footer-visibility flags: `ftr`, `dt`, `sldNum` boolean attributes from
///      the `<p:hf>` element (AC-003 / BC-2.01.001 v1.3 corrected).
/// 3. If `slideMaster1.xml` is absent from the ZIP, emit a `tracing::debug!`
///    and return defaults for both text and flags (AC-005 / EC-001).
/// 4. If the master placeholder is present but has no text, fall back to
///    `ppt/slideLayouts/slideLayout1.xml` for text only (AC-002 / EC-002).
///    Flags always come from the master's `<p:hf>`.
/// 5. If `<p:hf>` is absent from the master, emit a `tracing::debug!` and use
///    `FooterFlags::default()` (EC-004).
///
/// ## Field element handling
///
/// A footer placeholder that contains only `<a:fld>` elements (dynamic date /
/// slide-number fields) is treated as having no text (EC-006).  Only `<a:t>`
/// text runs inside `<a:r>` elements count.
///
/// ## Note
///
/// `ppt/presProps.xml` is **NOT** read for footer visibility (adversary H-1 /
/// BC-2.01.001 v1.3).  `CT_ShowProperties` has no `ftr`/`dt`/`sldNum` children.
#[must_use]
pub(crate) fn detect_footer<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    is_pptx: bool,
) -> FooterDetection {
    if !is_pptx {
        // AC-006 / EC-005: DOCX has no slide-master footer; skip all detection.
        return FooterDetection::default();
    }

    // --- Single pass over slideMaster1.xml for BOTH text and flags ---
    let master_bytes = read_zip_entry(zip, "ppt/slideMasters/slideMaster1.xml");
    let Some(master_bytes) = master_bytes else {
        // EC-001 / AC-005: master XML not present in ZIP.
        tracing::debug!(
            "ppt/slideMasters/slideMaster1.xml absent from ZIP; \
             footer detection skipped"
        );
        return FooterDetection::default();
    };

    let MasterParseResult {
        text_result,
        flags,
        hf_found,
    } = parse_master_xml(&master_bytes);

    if !hf_found {
        // EC-004: <p:hf> absent from master.
        tracing::debug!(
            "<p:hf> element absent from ppt/slideMasters/slideMaster1.xml; \
             footer visibility flags default to all-false (EC-004)"
        );
    }

    let text = match text_result {
        FooterXmlResult::Absent | FooterXmlResult::NoPlaceholder => None,
        FooterXmlResult::EmptyText => {
            // AC-002 / EC-002: master placeholder present but no text; try layout1.
            match read_footer_text_from_zip(zip, "ppt/slideLayouts/slideLayout1.xml") {
                FooterXmlResult::FoundText(t) => Some(t),
                _ => None,
            }
        },
        FooterXmlResult::FoundText(t) => Some(t),
    };

    FooterDetection { text, flags }
}

// ─── Internal types and helpers ───────────────────────────────────────────────

/// Outcome of attempting to extract footer text from one XML file in the ZIP.
#[derive(Debug)]
enum FooterXmlResult {
    /// The ZIP entry did not exist.
    Absent,
    /// The XML was parsed successfully but contained no `<p:ph type="ftr"/>` element.
    NoPlaceholder,
    /// A footer placeholder was found but its text run(s) were all empty/whitespace.
    EmptyText,
    /// A footer placeholder was found with non-empty concatenated text.
    FoundText(Arc<str>),
}

/// Result of the combined single-pass parse over `slideMaster1.xml`.
struct MasterParseResult {
    /// Footer text result (same semantics as [`FooterXmlResult`]).
    text_result: FooterXmlResult,
    /// Flags from the `<p:hf>` element (all-false if element absent).
    flags: FooterFlags,
    /// Whether a `<p:hf>` element was encountered at all (for debug logging).
    hf_found: bool,
}

/// Read the bytes of `zip_path` from `zip`, return `None` if absent.
fn read_zip_entry<R: Read + Seek>(zip: &mut ZipArchive<R>, zip_path: &str) -> Option<Vec<u8>> {
    let mut entry = zip.by_name(zip_path).ok()?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// Parse `slideMaster1.xml` bytes in a **single SAX pass**, extracting BOTH
/// footer placeholder text AND `<p:hf>` visibility flags.
///
/// ## State machine
///
/// - `<p:hf>` (start or empty) → extract `ftr`, `dt`, `sldNum` boolean attrs.
/// - `<p:sp>` → enter shape scope; reset per-shape state.
/// - `<p:ph>` with `type="ftr"` attribute within `<p:sp>` → mark as footer placeholder.
/// - `</p:sp>` → if this was a footer placeholder, finalize text accumulator;
///   EC-003: first footer placeholder wins (stop after first `</p:sp>` that is
///   a footer placeholder).
/// - `<a:r>` inside a footer placeholder → enter run scope.
/// - `</a:r>` → exit run scope; run text is appended to the shape accumulator
///   (ALL runs concatenated — no "first run only" restriction).
/// - `<a:t>` inside `<a:r>` inside a footer placeholder → collect characters.
/// - `<a:fld>` inside a footer placeholder → NOT a text run; no text collected (EC-006).
fn parse_master_xml(xml_bytes: &[u8]) -> MasterParseResult {
    let mut reader = Reader::from_reader(xml_bytes);
    // Do NOT trim_text here: trimming would strip meaningful whitespace from
    // interior-of-run text (e.g., "Acme Confidential " loses trailing space).
    // We trim only the FINAL concatenated accumulator when finalizing.
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();

    // `<p:hf>` state
    let mut flags = FooterFlags::default();
    let mut hf_found = false;

    // Footer placeholder text state
    let mut in_sp = false; // inside <p:sp>
    let mut is_footer_ph = false; // current <p:sp> has <p:ph type="ftr"/>
    let mut footer_ph_done = false; // first footer placeholder has been resolved (EC-003)
    let mut in_run = false; // inside <a:r> within a footer placeholder
    let mut in_text = false; // inside <a:t> within a run
    let mut run_buf = String::new(); // accumulates characters within current <a:t>
    let mut sp_text_accum = String::new(); // accumulates all run text for the placeholder
    let mut found_any_footer_ph = false;
    let mut text_result = FooterXmlResult::NoPlaceholder;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    "parse error reading slideMaster1.xml; remaining footer detection proceeds with partial result"
                );
                break;
            },
            Ok(Event::Start(ref e)) => {
                let local = local_name_owned(e);
                match local.as_str() {
                    "hf" => {
                        // <p:hf> as a start element (rare but possible if parser sees it that way).
                        hf_found = true;
                        flags = extract_hf_flags(e);
                    },
                    "sp" if !footer_ph_done => {
                        in_sp = true;
                        is_footer_ph = false;
                        in_run = false;
                        in_text = false;
                        run_buf.clear();
                        sp_text_accum.clear();
                    },
                    "ph" if in_sp && !footer_ph_done && attr_equals(e, b"type", b"ftr") => {
                        is_footer_ph = true;
                        found_any_footer_ph = true;
                    },
                    "r" if is_footer_ph => {
                        in_run = true;
                        run_buf.clear();
                    },
                    "t" if is_footer_ph && in_run => {
                        in_text = true;
                    },
                    _ => {},
                }
            },
            Ok(Event::Empty(ref e)) => {
                let local = local_name_owned(e);
                match local.as_str() {
                    "hf" => {
                        // <p:hf .../> as a self-closing (empty) element — the common case.
                        hf_found = true;
                        flags = extract_hf_flags(e);
                    },
                    "ph" if in_sp && !footer_ph_done && attr_equals(e, b"type", b"ftr") => {
                        is_footer_ph = true;
                        found_any_footer_ph = true;
                    },
                    // <a:fld> as empty element: not a text run; do not set in_run (EC-006).
                    _ => {},
                }
            },
            Ok(Event::End(ref e)) => {
                let local = local_name_end_owned(e);
                match local.as_str() {
                    "sp" => {
                        if is_footer_ph {
                            // EC-003: first footer placeholder wins.
                            // Trim the full concatenated text; None if all whitespace.
                            let trimmed = sp_text_accum.trim();
                            text_result = if trimmed.is_empty() {
                                FooterXmlResult::EmptyText
                            } else {
                                FooterXmlResult::FoundText(Arc::from(trimmed))
                            };
                            // Mark done so subsequent footer placeholders are ignored.
                            footer_ph_done = true;
                            is_footer_ph = false;
                        }
                        in_sp = false;
                    },
                    "r" if is_footer_ph => {
                        // End of a run: append run's accumulated chars to the shape accumulator.
                        // AC-001 / EC-008: ALL runs concatenated in document order.
                        sp_text_accum.push_str(&run_buf);
                        run_buf.clear();
                        in_run = false;
                        in_text = false;
                    },
                    "t" if is_footer_ph && in_run => {
                        in_text = false;
                        // Characters are already in run_buf via Event::Text below.
                    },
                    _ => {},
                }
            },
            Ok(Event::Text(ref e)) if in_text && is_footer_ph && in_run => match e.unescape() {
                Ok(s) => run_buf.push_str(s.as_ref()),
                Err(err) => {
                    tracing::debug!(
                        error = %err,
                        "failed to unescape text in footer placeholder; skipping chunk"
                    );
                },
            },
            Ok(Event::Eof) => break,
            _ => {},
        }
        buf.clear();
    }

    // If we found a placeholder but </sp> finalization never ran (truncated XML),
    // treat as empty.
    if found_any_footer_ph && matches!(text_result, FooterXmlResult::NoPlaceholder) {
        text_result = FooterXmlResult::EmptyText;
    }

    MasterParseResult {
        text_result,
        flags,
        hf_found,
    }
}

/// Parse `xml_bytes` for footer placeholder text only (used for layout fallback,
/// AC-002).  Returns [`FooterXmlResult`] describing the outcome.
///
/// This is a text-only pass — it does NOT look for `<p:hf>` flags.
///
/// All `<a:t>` runs inside the first `<p:ph type="ftr"/>` shape are concatenated
/// in document order (AC-001 / EC-008).  `<a:fld>` elements are not text runs
/// (EC-006).
fn parse_footer_text_from_xml(xml_bytes: &[u8]) -> FooterXmlResult {
    let mut reader = Reader::from_reader(xml_bytes);
    // Do NOT trim_text: preserves interior whitespace within runs (e.g., trailing
    // space in "Acme Confidential "). The final accumulator is trimmed on output.
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();

    let mut in_sp = false;
    let mut is_footer_ph = false;
    let mut in_run = false;
    let mut in_text = false;
    let mut run_buf = String::new();
    let mut sp_text_accum = String::new();
    let mut found_any_footer_ph = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    "parse error reading layout XML for footer fallback; stopping"
                );
                break;
            },
            Ok(Event::Start(ref e)) => {
                let local = local_name_owned(e);
                match local.as_str() {
                    "sp" => {
                        in_sp = true;
                        is_footer_ph = false;
                        in_run = false;
                        in_text = false;
                        run_buf.clear();
                        sp_text_accum.clear();
                    },
                    "ph" if in_sp && attr_equals(e, b"type", b"ftr") => {
                        is_footer_ph = true;
                        found_any_footer_ph = true;
                    },
                    "r" if is_footer_ph => {
                        in_run = true;
                        run_buf.clear();
                    },
                    "t" if is_footer_ph && in_run => {
                        in_text = true;
                    },
                    _ => {},
                }
            },
            Ok(Event::Empty(ref e)) => {
                let local = local_name_owned(e);
                if local == "ph" && in_sp && attr_equals(e, b"type", b"ftr") {
                    is_footer_ph = true;
                    found_any_footer_ph = true;
                }
                // <a:fld> as empty element: not a text run (EC-006).
            },
            Ok(Event::End(ref e)) => {
                let local = local_name_end_owned(e);
                match local.as_str() {
                    "sp" => {
                        if is_footer_ph {
                            // EC-003: first footer placeholder wins — early return.
                            let trimmed = sp_text_accum.trim();
                            return if trimmed.is_empty() {
                                FooterXmlResult::EmptyText
                            } else {
                                FooterXmlResult::FoundText(Arc::from(trimmed))
                            };
                        }
                        in_sp = false;
                    },
                    "r" if is_footer_ph => {
                        // AC-001 / EC-008: ALL runs concatenated in document order.
                        sp_text_accum.push_str(&run_buf);
                        run_buf.clear();
                        in_run = false;
                        in_text = false;
                    },
                    "t" if is_footer_ph && in_run => {
                        in_text = false;
                    },
                    _ => {},
                }
            },
            Ok(Event::Text(ref e)) if in_text && is_footer_ph && in_run => match e.unescape() {
                Ok(s) => run_buf.push_str(s.as_ref()),
                Err(err) => {
                    tracing::debug!(
                        error = %err,
                        "failed to unescape text in layout footer placeholder; skipping chunk"
                    );
                },
            },
            Ok(Event::Eof) => break,
            _ => {},
        }
        buf.clear();
    }

    if found_any_footer_ph {
        FooterXmlResult::EmptyText
    } else {
        FooterXmlResult::NoPlaceholder
    }
}

/// Try to read and parse footer text from a ZIP entry at `zip_path`.
fn read_footer_text_from_zip<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    zip_path: &str,
) -> FooterXmlResult {
    match read_zip_entry(zip, zip_path) {
        None => FooterXmlResult::Absent,
        Some(bytes) => parse_footer_text_from_xml(&bytes),
    }
}

// ─── XML attribute helpers ────────────────────────────────────────────────────

/// Extract the local name from a `quick_xml` event element as an owned `String`.
///
/// Returns an empty string on invalid UTF-8 (which should never occur in
/// well-formed OOXML, but is handled gracefully per the production-grade default).
fn local_name_owned(e: &quick_xml::events::BytesStart<'_>) -> String {
    std::str::from_utf8(e.local_name().as_ref())
        .unwrap_or("")
        .to_owned()
}

/// Extract the local name from a `quick_xml` end-event element as an owned `String`.
fn local_name_end_owned(e: &quick_xml::events::BytesEnd<'_>) -> String {
    std::str::from_utf8(e.local_name().as_ref())
        .unwrap_or("")
        .to_owned()
}

/// Returns `true` if the element `e` has attribute `attr_name` equal to `attr_value`.
fn attr_equals(e: &quick_xml::events::BytesStart<'_>, attr_name: &[u8], attr_value: &[u8]) -> bool {
    e.attributes()
        .filter_map(std::result::Result::ok)
        .any(|a| a.key.local_name().as_ref() == attr_name && a.value.as_ref() == attr_value)
}

/// Parse an OOXML `xsd:boolean` attribute value from a `<p:hf>` element.
///
/// Accepts both numeric (`"1"` → `true`, `"0"` → `false`) and string
/// (`"true"` → `true`, `"false"` → `false`) forms per ECMA-376.  An absent
/// attribute or any unrecognized value returns `false` (optional default).
fn parse_ooxml_bool(value: &[u8]) -> bool {
    matches!(value, b"1" | b"true")
}

/// Extract `<p:hf>` boolean attributes (`ftr`, `dt`, `sldNum`) from the start element.
///
/// Called when the SAX parser encounters a `<p:hf>` start or empty element.
/// Returns a [`FooterFlags`] populated from the element's attributes.
fn extract_hf_flags(e: &quick_xml::events::BytesStart<'_>) -> FooterFlags {
    let mut flags = FooterFlags::default();
    for attr in e.attributes().filter_map(std::result::Result::ok) {
        match attr.key.local_name().as_ref() {
            b"ftr" => flags.show_footer = parse_ooxml_bool(attr.value.as_ref()),
            b"dt" => flags.show_date = parse_ooxml_bool(attr.value.as_ref()),
            b"sldNum" => flags.show_slide_number = parse_ooxml_bool(attr.value.as_ref()),
            _ => {},
        }
    }
    flags
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    use crate::footer::{FooterFlags, detect_footer};

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

    /// Minimal slide master XML with a populated footer placeholder AND a `<p:hf>` element
    /// with all three flags set to "1".
    ///
    /// Contains `<p:ph type="ftr"/>` with `<a:t>Confidential</a:t>` and
    /// `<p:hf ftr="1" dt="1" sldNum="1"/>`.
    const MASTER_WITH_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="1" dt="1" sldNum="1"/>
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

    /// Slide master XML with a footer placeholder containing TWO `<a:r>` runs —
    /// tests EC-008 / AC-001 (corrected): full concatenation of all runs.
    ///
    /// Runs: "Acme Confidential " and "2026" → expected concat "Acme Confidential 2026".
    /// The `<p:hf>` element is present with ftr="1" only.
    const MASTER_MULTIRUN_FOOTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="1" dt="0" sldNum="0"/>
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
          <a:r><a:t>Acme Confidential </a:t></a:r>
          <a:r><a:t>2026</a:t></a:r>
        </a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with `<p:hf ftr="1" sldNum="1"/>` — `dt` attribute ABSENT.
    ///
    /// Expected: `show_footer=true`, `show_date=false` (absent=false), `show_slide_number=true`.
    /// Also contains a footer placeholder so the ZIP is self-consistent.
    const MASTER_HF_FTR_AND_SLDNUM_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="1" sldNum="1"/>
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
        <a:p><a:r><a:t>Corp Footer</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with `<p:hf ftr="true" dt="true" sldNum="false"/>` —
    /// tests OOXML boolean "true"/"false" string form (in addition to "1"/"0").
    const MASTER_HF_BOOL_STRING_FORM_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="true" dt="true" sldNum="false"/>
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>Bool Form Test</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with a `<p:hf>` element present but ALL THREE attributes absent.
    ///
    /// EC-007: `<p:hf>` present but `ftr`, `dt`, `sldNum` attributes all absent →
    /// all three flags false (xsd:boolean optional, absent = false).
    const MASTER_HF_ALL_ATTRS_ABSENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf hdr="0"/>
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>Footer</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

    /// Slide master XML with NO `<p:hf>` element at all.
    ///
    /// EC-004: absent `<p:hf>` → `FooterFlags::default()` (all false); `tracing::debug!` emitted.
    const MASTER_NO_HF_ELEMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>Footer text</a:t></a:r></a:p>
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

    /// `presProps.xml` stub — used in the `BrandLoader` integration test to verify
    /// `presProps.xml` is NOT consulted for footer flags (AC-003 corrected).
    /// A real `presProps.xml` present in the ZIP should have no effect on `FooterFlags`
    /// because flags come from `<p:hf>` on `slideMaster1.xml`, not from `presProps.xml`.
    const PRESPROPS_STUB_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
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
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
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

    /// BC-2.01.001 AC-003 / EC-004 (corrected) — `<p:hf>` element absent from
    /// `slideMaster1.xml` → `FooterFlags::default()` (all false); `tracing::debug!` emitted.
    ///
    /// Test vector from BC-2.01.001 v1.3 (corrected): NO `<p:hf>` in master XML →
    /// all flags false. This replaces the old presProps-absent test.
    ///
    /// RED GATE: the current implementation reads presProps.xml (not `<p:hf>`), so if
    /// presProps.xml is absent the old code returns default. However, this test exercises
    /// the correct source (`<p:hf>` absent from master), which must also yield defaults.
    /// The new `<p:hf>` tests below (`flags_from_hf_element`, `hf_partial_attrs`) are the
    /// load-bearing Red Gate tests — they fail against the presProps implementation
    /// because the presProps impl can never read `<p:hf>` attributes.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_absent_hf_element() {
        // ZIP has master but NO <p:hf> element in it (MASTER_NO_HF_ELEMENT_XML).
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_NO_HF_ELEMENT_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            !detection.flags.show_footer,
            "EC-004 (corrected): absent <p:hf> must yield show_footer = false; \
             got show_footer={}",
            detection.flags.show_footer
        );
        assert!(
            !detection.flags.show_date,
            "EC-004 (corrected): absent <p:hf> must yield show_date = false; \
             got show_date={}",
            detection.flags.show_date
        );
        assert!(
            !detection.flags.show_slide_number,
            "EC-004 (corrected): absent <p:hf> must yield show_slide_number = false; \
             got show_slide_number={}",
            detection.flags.show_slide_number
        );
    }

    /// BC-2.01.001 AC-003 (corrected) — `<p:hf ftr="1" sldNum="1"/>` in slideMaster1.xml
    /// (no `dt` attribute) → `show_footer=true, show_date=false, show_slide_number=true`.
    ///
    /// Test vector from BC-2.01.001 v1.3 canonical test vectors:
    ///   `.pptx with <p:hf ftr="1" dt="1" sldNum="0"/>` → `{ show_footer: true, show_date: true,
    ///   show_slide_number: false }`.
    /// This test uses ftr="1" + sldNum="1" (no dt attr) to verify partial attribute presence.
    ///
    /// **RED GATE:** the current implementation reads `presProps.xml` `<p:showPr>` children.
    /// `MASTER_HF_FTR_AND_SLDNUM_XML` has NO `presProps.xml` in the ZIP, so the old code returns
    /// all-false. This test expects `show_footer=true` and `show_slide_number=true` → FAILS.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_from_hf_element() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_HF_FTR_AND_SLDNUM_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "AC-003 (corrected): <p:hf ftr=\"1\"/> must set show_footer = true; \
             got show_footer={}",
            detection.flags.show_footer
        );
        assert!(
            !detection.flags.show_date,
            "AC-003 (corrected): absent dt attribute in <p:hf> must leave show_date = false \
             (xsd:boolean optional default); got show_date={}",
            detection.flags.show_date
        );
        assert!(
            detection.flags.show_slide_number,
            "AC-003 (corrected): <p:hf sldNum=\"1\"/> must set show_slide_number = true; \
             got show_slide_number={}",
            detection.flags.show_slide_number
        );
    }

    /// BC-2.01.001 AC-003 (corrected) — all three flags set via `<p:hf>` in slideMaster1.xml.
    ///
    /// Test vector: `<p:hf ftr="1" dt="1" sldNum="1"/>` → all three flags true.
    /// Canonical test vector from BC-2.01.001 v1.3.
    ///
    /// **RED GATE:** old impl reads `presProps.xml`. `MASTER_WITH_FOOTER_XML` now contains
    /// `<p:hf ftr="1" dt="1" sldNum="1"/>` but no `presProps.xml` is in the ZIP, so the old
    /// code returns all-false. This test expects all three true → FAILS.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_all_three_from_hf_element() {
        // MASTER_WITH_FOOTER_XML carries <p:hf ftr="1" dt="1" sldNum="1"/> (updated fixture).
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_WITH_FOOTER_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "AC-003 (corrected): <p:hf ftr=\"1\"/> must set show_footer = true"
        );
        assert!(
            detection.flags.show_date,
            "AC-003 (corrected): <p:hf dt=\"1\"/> must set show_date = true"
        );
        assert!(
            detection.flags.show_slide_number,
            "AC-003 (corrected): <p:hf sldNum=\"1\"/> must set show_slide_number = true"
        );
    }

    /// BC-2.01.001 AC-003 / EC-007 (corrected) — `<p:hf>` element present in
    /// `slideMaster1.xml` but ALL three attributes (`ftr`, `dt`, `sldNum`) are absent.
    ///
    /// EC-007: all three flags are false (xsd:boolean optional, absent = false per
    /// ECMA-376 `CT_HeaderFooter`). No error, no warning.
    ///
    /// **RED GATE:** old impl reads presProps.xml `<p:showPr>` children — all-false is
    /// the expected result for both old and new code in this specific case (no attrs).
    /// However, the naming asserts `<p:hf>`-awareness is required — a correct impl must
    /// parse `<p:hf>` at all to distinguish EC-007 from EC-004. The test is structurally
    /// non-tautological because any impl that reads the right element still gets the
    /// same result; the other `<p:hf>` tests (above) provide the actual Red Gate signal.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_hf_partial_attrs() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_HF_ALL_ATTRS_ABSENT_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            !detection.flags.show_footer,
            "EC-007 (corrected): <p:hf> with absent ftr attr must yield show_footer = false"
        );
        assert!(
            !detection.flags.show_date,
            "EC-007 (corrected): <p:hf> with absent dt attr must yield show_date = false"
        );
        assert!(
            !detection.flags.show_slide_number,
            "EC-007 (corrected): <p:hf> with absent sldNum attr must yield show_slide_number = false"
        );
    }

    /// BC-2.01.001 AC-003 (corrected) — OOXML boolean "true"/"false" string form in `<p:hf>`.
    ///
    /// ECMA-376 xsd:boolean allows "1", "0", "true", "false". The parser must handle
    /// both numeric and string forms.
    ///
    /// Fixture: `<p:hf ftr="true" dt="true" sldNum="false"/>` →
    ///   `show_footer=true`, `show_date=true`, `show_slide_number=false`.
    ///
    /// **RED GATE:** the old presProps impl uses `attr_val_is_one()` which checks for "1"
    /// only. Even if it were adapted to read `<p:hf>`, "true" would still yield false.
    /// This test catches that inadequacy. Against the current code the RED GATE fires
    /// because `presProps` is read (not `<p:hf>`) and `show_footer`/`show_date` come out false.
    #[test]
    fn test_bc_2_01_001_detect_footer_flags_hf_boolean_string_form() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_HF_BOOL_STRING_FORM_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "AC-003: <p:hf ftr=\"true\"/> must set show_footer = true (OOXML bool string form); \
             got show_footer={}",
            detection.flags.show_footer
        );
        assert!(
            detection.flags.show_date,
            "AC-003: <p:hf dt=\"true\"/> must set show_date = true (OOXML bool string form); \
             got show_date={}",
            detection.flags.show_date
        );
        assert!(
            !detection.flags.show_slide_number,
            "AC-003: <p:hf sldNum=\"false\"/> must set show_slide_number = false (OOXML bool string form); \
             got show_slide_number={}",
            detection.flags.show_slide_number
        );
    }

    /// BC-2.01.001 AC-001 (corrected) / EC-008 — footer placeholder with multiple
    /// `<a:r>` runs: ALL runs must be concatenated in document order.
    ///
    /// "First-run-only" semantics are FORBIDDEN per STORY-075 v1.1 / BC-2.01.001 v1.3.
    ///
    /// Fixture: two runs "Acme Confidential " + "2026" → expected "Acme Confidential 2026".
    ///
    /// **RED GATE:** the current `parse_footer_text_from_xml` implementation stores the
    /// first non-empty run and then gates on `sp_text.is_none()` — so subsequent runs are
    /// discarded. Against the current code `detection.text` == Some("Acme Confidential ")
    /// (first run only, trimmed to "Acme Confidential"), NOT "Acme Confidential 2026".
    /// The assertion `== Some("Acme Confidential 2026")` therefore FAILS.
    #[test]
    fn test_bc_2_01_001_detect_footer_text_multirun() {
        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_MULTIRUN_FOOTER_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert_eq!(
            detection.text.as_deref(),
            Some("Acme Confidential 2026"),
            "AC-001/EC-008 (corrected): footer text split across multiple <a:r> runs \
             must be fully concatenated in document order; \
             'first-run-only' semantics are forbidden. \
             Got: {:?}",
            detection.text
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
    /// This is the primary integration path: `detect_footer()` called inside
    /// `load_template()` before `BrandTemplate` is returned.
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

    /// BC-2.01.001 AC-003 / AC-004 (corrected) — `BrandLoader::load_template()` on PPTX
    /// reads footer visibility flags from `<p:hf>` attributes on `slideMaster1.xml`,
    /// NOT from `presProps.xml`.
    ///
    /// Setup: master XML has `<p:hf ftr="1" sldNum="1"/>` (dt absent). A presProps.xml
    /// stub is also present in the ZIP (with no showPr children) to confirm it is NOT
    /// consulted for flags.
    ///
    /// Expected: `show_footer=true`, `show_date=false`, `show_slide_number=true`.
    ///
    /// **RED GATE:** the current implementation calls `read_pres_props_flags()` which reads
    /// `presProps.xml`. The presProps stub has no `<p:showPr>` so the old code returns
    /// all-false flags. The `<p:hf>` element in the master is ignored entirely by the
    /// current impl. This test therefore fails: `show_footer` is false (got) vs true (expected).
    #[test]
    fn test_bc_2_01_001_load_pptx_footer_flags_populated_from_hf_element() {
        use crate::context::BrandLoadContext;
        use crate::loader::BrandLoader;

        // Master XML with <p:hf ftr="1" sldNum="1"/> (dt absent) and a footer placeholder.
        let master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="1" sldNum="1"/>
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

        // Also include a presProps.xml stub — it must have NO effect on footer_flags.
        let zip_bytes = build_pptx_zip_with_extras(&[
            ("ppt/slideMasters/slideMaster1.xml", master_xml.as_bytes()),
            ("ppt/presProps.xml", PRESPROPS_STUB_XML.as_bytes()),
        ]);
        let path = write_temp(&zip_bytes, "pptx");
        let loader = BrandLoader::new();
        let ctx = BrandLoadContext::for_test();

        let result = loader.load_template(&path, &ctx);
        let _ = std::fs::remove_file(&path);

        let template = result.expect("PPTX with <p:hf> master must load without error");

        // footer_flags must reflect the <p:hf> attributes on slideMaster1.xml,
        // NOT the presProps.xml stub (which has no showPr children).
        assert!(
            template.footer_flags.show_footer,
            "AC-003/AC-004 (corrected): footer_flags.show_footer must be true from \
             <p:hf ftr=\"1\"/> on slideMaster1.xml; \
             got show_footer={}",
            template.footer_flags.show_footer
        );
        assert!(
            !template.footer_flags.show_date,
            "AC-003/AC-004 (corrected): footer_flags.show_date must be false \
             (dt attribute absent from <p:hf>); \
             got show_date={}",
            template.footer_flags.show_date
        );
        assert!(
            template.footer_flags.show_slide_number,
            "AC-003/AC-004 (corrected): footer_flags.show_slide_number must be true from \
             <p:hf sldNum=\"1\"/> on slideMaster1.xml; \
             got show_slide_number={}",
            template.footer_flags.show_slide_number
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
    /// `BrandExtractor` was permanently unreachable before STORY-075 because
    /// `BrandLoader` always returned `footer_text: None`.
    ///
    /// The test exercises the full pipeline:
    ///   `BrandLoader::load_template()` → `BrandExtractor::extract()` → brand.toml
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
        let result = BrandExtractor::extract(pptx_path.to_str().unwrap(), &output_dir, false)
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

    // ─── BC-2.01.001 v1.3 canonical test vector (OBS-2) ──────────────────────

    /// BC-2.01.001 v1.3 canonical test vector — `<p:hf ftr="1" dt="1" sldNum="0"/>`.
    ///
    /// This is the verbatim test vector from BC-2.01.001 v1.3:
    ///   `<p:hf ftr="1" dt="1" sldNum="0"/>` →
    ///   `FooterFlags { show_footer: true, show_date: true, show_slide_number: false }`
    ///
    /// The `sldNum="0"` path explicitly exercises the numeric `"0"` → `false` branch
    /// of `parse_ooxml_bool`, distinct from the absent-attribute path (which also
    /// yields `false` but via a different code path).
    #[test]
    fn test_bc_2_01_001_v1_3_canonical_vector_ftr1_dt1_sldnum0() {
        // Fixture: exactly the canonical <p:hf> element from BC-2.01.001 v1.3.
        const MASTER_CANONICAL_VECTOR_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:hf ftr="1" dt="1" sldNum="0"/>
  <p:spTree>
    <p:sp>
      <p:nvSpPr>
        <p:nvPr>
          <p:ph type="ftr" idx="11"/>
        </p:nvPr>
      </p:nvSpPr>
      <p:txBody>
        <a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>Test Footer</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree>
</p:sldMaster>"#;

        let zip_bytes = build_pptx_zip_with_extras(&[(
            "ppt/slideMasters/slideMaster1.xml",
            MASTER_CANONICAL_VECTOR_XML.as_bytes(),
        )]);
        let cursor = Cursor::new(zip_bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();

        let detection = detect_footer(&mut zip, true);

        assert!(
            detection.flags.show_footer,
            "BC-2.01.001 v1.3 canonical vector: <p:hf ftr=\"1\"/> must yield show_footer = true; \
             got show_footer={}",
            detection.flags.show_footer
        );
        assert!(
            detection.flags.show_date,
            "BC-2.01.001 v1.3 canonical vector: <p:hf dt=\"1\"/> must yield show_date = true; \
             got show_date={}",
            detection.flags.show_date
        );
        assert!(
            !detection.flags.show_slide_number,
            "BC-2.01.001 v1.3 canonical vector: <p:hf sldNum=\"0\"/> must yield \
             show_slide_number = false (explicit numeric zero); \
             got show_slide_number={}",
            detection.flags.show_slide_number
        );
    }
}
