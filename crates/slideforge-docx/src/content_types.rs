//! DOCX `[Content_Types].xml` generator.
//!
//! Produces the `[Content_Types].xml` part required by the OOXML packaging
//! spec. Registers every mandatory DOCX part type so that Word and `LibreOffice`
//! can locate each part within the ZIP.
//!
//! Required content types (STORY-041 Task 3):
//! - `word/document.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml`
//! - `word/styles.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml`
//! - `word/numbering.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml`
//! - `word/settings.xml` → `application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml`

use ooxmlsdk::common::XmlNamespaceDecl;
use ooxmlsdk::schemas::opc_content_types::{Default, Override, Types, TypesChoice};
use ooxmlsdk::sdk::SdkType;

use crate::error::ExportError;

/// Namespace URI for the content types schema.
const CONTENT_TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";

/// Builds the `[Content_Types].xml` byte buffer for a `.docx` archive.
///
/// # Errors
///
/// Returns [`ExportError::OoxmlError`] if the XML cannot be constructed.
pub fn build_content_types() -> Result<Vec<u8>, ExportError> {
    let types = Types {
        xmlns: vec![XmlNamespaceDecl::new("", CONTENT_TYPES_NS)],
        xml_header: ooxmlsdk::common::XmlHeaderType::Standalone,
        types_choice: vec![
            // Default types for common file extensions.
            TypesChoice::Default(Box::new(Default {
                extension: "rels".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-package.relationships+xml".to_owned(),
            })),
            TypesChoice::Default(Box::new(Default {
                extension: "xml".to_owned(),
                content_type: "application/xml".to_owned(),
            })),
            // Override: main document body.
            TypesChoice::Override(Box::new(Override {
                part_name: "/word/document.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
                        .to_owned(),
            })),
            // Override: styles part.
            TypesChoice::Override(Box::new(Override {
                part_name: "/word/styles.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"
                        .to_owned(),
            })),
            // Override: numbering definitions.
            TypesChoice::Override(Box::new(Override {
                part_name: "/word/numbering.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"
                        .to_owned(),
            })),
            // Override: document settings.
            TypesChoice::Override(Box::new(Override {
                part_name: "/word/settings.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"
                        .to_owned(),
            })),
            // Override: core document properties.
            TypesChoice::Override(Box::new(Override {
                part_name: "/docProps/core.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-package.core-properties+xml".to_owned(),
            })),
            // Override: extended document properties.
            TypesChoice::Override(Box::new(Override {
                part_name: "/docProps/app.xml".to_owned(),
                content_type:
                    "application/vnd.openxmlformats-officedocument.extended-properties+xml"
                        .to_owned(),
            })),
        ],
    };

    let mut buf: Vec<u8> = Vec::new();
    types
        .write_type_xml(&mut buf, "")
        .map_err(|e| ExportError::OoxmlError {
            message: format!("[Content_Types].xml write error: {e}"),
        })?;

    Ok(buf)
}
