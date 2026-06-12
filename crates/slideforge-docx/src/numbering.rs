//! DOCX `word/numbering.xml` builder.
//!
//! Produces a [`Numbering`] document with one abstract numbering definition
//! covering bullet list levels 0–2 and one concrete numbering instance
//! referencing it. Bullet paragraphs in `word/document.xml` use
//! `numId=BULLET_NUM_ID` to reference this concrete definition.
//!
//! ## OOXML numbering structure
//!
//! ```text
//! <w:numbering>
//!   <w:abstractNum w:abstractNumId="0">
//!     <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/>…</w:lvl>
//!     <w:lvl w:ilvl="1">…</w:lvl>
//!     <w:lvl w:ilvl="2">…</w:lvl>
//!   </w:abstractNum>
//!   <w:num w:numId="1">
//!     <w:abstractNumId w:val="0"/>
//!   </w:num>
//! </w:numbering>
//! ```
//!
//! Bullet paragraphs in `document.xml` reference `numId=BULLET_NUM_ID` via
//! `<w:numPr><w:ilvl w:val="N"/><w:numId w:val="1"/></w:numPr>`.

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::schemas_openxmlformats_org_wordprocessingml_2006_main::{
    AbstractNum, AbstractNumId, Indentation, Level, LevelText, NumberFormatValues, Numbering,
    NumberingFormat, NumberingInstance, PreviousParagraphProperties, StartNumberingValue,
};
use ooxmlsdk::sdk::SdkType;

use crate::error::ExportError;

/// W namespace URI for Word processing ML.
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// Abstract numbering definition ID used for the built-in bullet list.
///
/// All DOCX bullet paragraphs reference concrete `numId=BULLET_NUM_ID` which
/// maps back to this abstract definition.
pub const BULLET_ABSTRACT_NUM_ID: i32 = 0;

/// Concrete numbering instance ID for bullet lists.
///
/// Bullet paragraphs use `<w:numId w:val="1"/>` in their `<w:numPr>`. The
/// value is small and non-zero; `numId=0` is reserved by Word to mean
/// "remove all list formatting" and MUST NOT be used for active lists.
pub const BULLET_NUM_ID: i32 = 1;

/// Unicode bullet symbols for list levels 0, 1, and 2.
///
/// - Level 0: U+2022 BULLET (•)
/// - Level 1: U+25E6 WHITE BULLET (◦)
/// - Level 2: U+25AA BLACK SMALL SQUARE (▪)
const BULLET_CHARS: [&str; 3] = ["\u{2022}", "\u{25E6}", "\u{25AA}"];

/// Left indent in twips per bullet level (720 twips ≈ 0.5 inch).
const LEVEL_LEFT_TWIPS: [i32; 3] = [720, 1440, 2160];

/// Hanging indent in twips applied at every bullet level.
const LEVEL_HANGING_TWIPS: i32 = 360;

/// Build `word/numbering.xml` bytes using ooxmlsdk typed constructors.
///
/// Produces one abstract numbering definition with bullet levels 0–2
/// and one concrete numbering instance that bullet paragraphs reference.
///
/// # Errors
///
/// Returns [`ExportError::OoxmlError`] if the XML cannot be serialized.
pub fn build_numbering_xml() -> Result<Vec<u8>, ExportError> {
    let mut levels: Vec<Level> = Vec::with_capacity(3);

    for (idx, (&bullet_char, &left_twips)) in
        BULLET_CHARS.iter().zip(LEVEL_LEFT_TWIPS.iter()).enumerate()
    {
        // `level_index` is `i32` in the OOXML schema (w:ilvl attribute).
        // The loop is bounded to 0..3 so the conversion never truncates.
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let ilvl = idx as i32;

        // Paragraph properties for this level: left indent + hanging indent.
        let level_pp = PreviousParagraphProperties {
            indentation: Some(Indentation {
                left: Some(left_twips.to_string()),
                hanging: Some(LEVEL_HANGING_TWIPS.to_string()),
                ..Indentation::default()
            }),
            ..PreviousParagraphProperties::default()
        };

        levels.push(Level {
            level_index: ilvl,
            start_numbering_value: Some(StartNumberingValue { val: 1 }),
            numbering_format: Some(NumberingFormat {
                val: NumberFormatValues::Bullet,
                ..NumberingFormat::default()
            }),
            level_text: Some(LevelText {
                val: Some(bullet_char.to_owned()),
                ..LevelText::default()
            }),
            previous_paragraph_properties: Some(Box::new(level_pp)),
            ..Level::default()
        });
    }

    let abstract_num = AbstractNum {
        xmlns: vec![XmlNamespaceDecl::new("w", W_NS)],
        abstract_number_id: BULLET_ABSTRACT_NUM_ID,
        w_lvl: levels,
        ..AbstractNum::default()
    };

    let num_instance = NumberingInstance {
        number_id: BULLET_NUM_ID,
        abstract_num_id: Box::new(AbstractNumId {
            val: BULLET_ABSTRACT_NUM_ID,
        }),
        ..NumberingInstance::default()
    };

    let numbering = Numbering {
        xmlns: vec![XmlNamespaceDecl::new("w", W_NS)],
        xml_header: ooxmlsdk::common::XmlHeaderType::Standalone,
        w_abstract_num: vec![abstract_num],
        w_num: vec![num_instance],
        ..Numbering::default()
    };

    let mut buf: Vec<u8> = Vec::new();
    numbering
        .write_type_xml(&mut buf, "")
        .map_err(|e| ExportError::OoxmlError {
            message: format!("numbering.xml write error: {e}"),
        })?;

    Ok(buf)
}
