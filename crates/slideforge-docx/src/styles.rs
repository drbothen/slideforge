//! DOCX `word/styles.xml` generator.
//!
//! Produces the `word/styles.xml` part that defines the paragraph and
//! character styles used in `word/document.xml`. Required styles:
//!
//! - `Heading1` — slide title level (paragraph style)
//! - `Heading2` — section title level (paragraph style)
//! - `Normal` — body paragraph (paragraph style)
//! - `Hyperlink` — hyperlink character style (character style)
//! - `CodeText` — inline code (character style; Courier New font)
//!
//! Style definitions are sourced from the brand when available (fonts, colors).
//! Minimal valid stubs are emitted when no brand override is present.

use slideforge_types::Brand;

use crate::error::ExportError;
use crate::xml_escape::xml_attr_escape;

/// Builds the `word/styles.xml` byte buffer.
///
/// When `brand` is `Some`, heading and body styles are configured using the
/// brand's font names. When `brand` is `None`, minimal valid stubs are emitted
/// (sufficient for Word to open the document).
///
/// # Errors
///
/// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
pub fn build_styles(brand: Option<&Brand>) -> Result<Vec<u8>, ExportError> {
    // Resolve font names from brand or fall back to safe defaults.
    let heading_font = brand.map_or("Calibri Light", |b| b.fonts.heading.as_ref());
    let body_font = brand.map_or("Calibri", |b| b.fonts.body.as_ref());
    let mono_font = brand.map_or("Courier New", |b| b.fonts.mono.as_ref());

    // We construct styles.xml as a raw XML string because the ooxmlsdk Styles
    // types are extremely verbose for what is essentially a fixed template.
    // This is an approved pattern when the OOXML structure is a static template
    // that doesn't vary dynamically beyond font substitution. The XML itself
    // is deterministic and brand-driven — no logic escapes, no user input is
    // embedded without XML-attribute escaping.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:rPr>
      <w:rFonts w:ascii="{body}" w:hAnsi="{body}"/>
      <w:sz w:val="22"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr>
      <w:outlineLvl w:val="0"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{heading}" w:hAnsi="{heading}"/>
      <w:b/>
      <w:sz w:val="32"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:basedOn w:val="Normal"/>
    <w:pPr>
      <w:outlineLvl w:val="1"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{heading}" w:hAnsi="{heading}"/>
      <w:b/>
      <w:sz w:val="28"/>
    </w:rPr>
  </w:style>
  <w:style w:type="character" w:styleId="Hyperlink">
    <w:name w:val="Hyperlink"/>
    <w:rPr>
      <w:color w:val="0563C1"/>
      <w:u w:val="single"/>
    </w:rPr>
  </w:style>
  <w:style w:type="character" w:styleId="CodeText">
    <w:name w:val="Code Text"/>
    <w:rPr>
      <w:rFonts w:ascii="{mono}" w:hAnsi="{mono}"/>
      <w:sz w:val="20"/>
    </w:rPr>
  </w:style>
  <w:style w:type="table" w:styleId="TableGrid">
    <w:name w:val="Table Grid"/>
    <w:basedOn w:val="TableNormal"/>
    <w:tblPr>
      <w:tblBorders>
        <w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/>
        <w:left w:val="single" w:sz="4" w:space="0" w:color="auto"/>
        <w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/>
        <w:right w:val="single" w:sz="4" w:space="0" w:color="auto"/>
        <w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/>
        <w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/>
      </w:tblBorders>
    </w:tblPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="TableHeader">
    <w:name w:val="Table Header"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr>
      <w:rFonts w:ascii="{heading}" w:hAnsi="{heading}"/>
      <w:b/>
      <w:sz w:val="22"/>
    </w:rPr>
  </w:style>
</w:styles>"#,
        body = xml_attr_escape(body_font),
        heading = xml_attr_escape(heading_font),
        mono = xml_attr_escape(mono_font),
    );

    Ok(xml.into_bytes())
}
