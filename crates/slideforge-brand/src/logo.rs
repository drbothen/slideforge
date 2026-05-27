//! Logo extraction from PPTX slide master relationships.
//!
//! [`extract_logo`] reads `ppt/slideMasters/_rels/slideMaster1.xml.rels` from
//! a PPTX ZIP archive, finds the first relationship with a `Type` attribute
//! ending in `/image`, and reads the image bytes from the target file.
//!
//! If no image relationship is found, returns `None` (EC-005 — not an error).

use std::io::Read;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;

use crate::template::LogoAsset;

/// ZIP-internal path to the slide master 1 relationships file (PPTX only).
pub const SLIDE_MASTER_RELS_PATH: &str =
    "ppt/slideMasters/_rels/slideMaster1.xml.rels";

/// Extract a logo [`LogoAsset`] from a PPTX ZIP archive.
///
/// Reads `ppt/slideMasters/_rels/slideMaster1.xml.rels`, finds the first
/// relationship entry whose `Type` attribute ends in `/image`, then reads the
/// target image file from the ZIP.
///
/// # Returns
///
/// - `Some(LogoAsset)` — if an image relationship and its target file are found.
/// - `None` — if no image relationship exists (BC-2.01.001 AC-005, EC-005).
///
/// # Note
///
/// This function only operates on PPTX archives. For DOCX, no logo extraction
/// is performed (DOCX does not use the slide master concept).
pub fn extract_logo<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
) -> Option<LogoAsset> {
    // Read the rels file from the ZIP.
    let rels_xml = {
        let mut entry = zip.by_name(SLIDE_MASTER_RELS_PATH).ok()?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).ok()?;
        buf
    };

    // Parse the relationships XML to find an image relationship.
    let image_target = find_image_target(&rels_xml)?;

    // Resolve the image path relative to the slide masters directory.
    // Target may be relative like `../media/image1.png` or absolute like `ppt/media/image1.png`.
    let resolved_path = resolve_rels_target(&image_target);

    // Read the image bytes from the ZIP.
    let image_bytes = {
        let mut entry = zip.by_name(&resolved_path).ok()?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).ok()?;
        buf
    };

    let ext = resolved_path
        .rsplit_once('.')
        .map_or("", |(_, e)| e);
    let media_type = media_type_from_extension(ext);

    Some(LogoAsset {
        bytes: image_bytes,
        media_type,
        original_path: Arc::from(resolved_path.as_str()),
    })
}

/// Parse the .rels XML and return the `Target` attribute of the first
/// relationship whose `Type` ends in `/image`.
fn find_image_target(rels_xml: &[u8]) -> Option<String> {
    let mut reader = Reader::from_reader(rels_xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e) | Event::Start(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                if name_str == "Relationship" {
                    let mut rel_type: Option<String> = None;
                    let mut target: Option<String> = None;
                    for attr in e.attributes().flatten() {
                        let key = attr.key.local_name();
                        match std::str::from_utf8(key.as_ref()).unwrap_or("") {
                            "Type" => {
                                if let Ok(v) = std::str::from_utf8(&attr.value) {
                                    rel_type = Some(v.to_owned());
                                }
                            }
                            "Target" => {
                                if let Ok(v) = std::str::from_utf8(&attr.value) {
                                    target = Some(v.to_owned());
                                }
                            }
                            _ => {}
                        }
                    }
                    if let (Some(rt), Some(tgt)) = (rel_type, target)
                        && rt.ends_with("/image")
                    {
                        return Some(tgt);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Resolve a relationship `Target` value to an absolute ZIP-internal path.
///
/// The `.rels` file is at `ppt/slideMasters/_rels/slideMaster1.xml.rels`.
/// Its base for resolution is `ppt/slideMasters/`.
/// Targets like `../media/image1.png` resolve to `ppt/media/image1.png`.
fn resolve_rels_target(target: &str) -> String {
    // If the target is already an absolute path (starts with `/`), strip the leading `/`.
    if let Some(stripped) = target.strip_prefix('/') {
        return stripped.to_owned();
    }

    // Base directory is the directory of the rels file's owning part.
    // The rels file is at `ppt/slideMasters/_rels/slideMaster1.xml.rels`.
    // The owning part is `ppt/slideMasters/slideMaster1.xml`.
    // So the base for resolution is `ppt/slideMasters/`.
    let base = "ppt/slideMasters/";
    let combined = format!("{base}{target}");
    normalize_path(&combined)
}

/// Normalize a slash-separated path by resolving `..` components.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Infer the MIME type from a file extension.
///
/// Used when building a [`LogoAsset`] to set the `media_type` field.
#[must_use]
pub fn media_type_from_extension(ext: &str) -> Arc<str> {
    match ext.to_lowercase().as_str() {
        "png" => Arc::from("image/png"),
        "jpg" | "jpeg" => Arc::from("image/jpeg"),
        "gif" => Arc::from("image/gif"),
        "svg" => Arc::from("image/svg+xml"),
        "wmf" => Arc::from("image/x-wmf"),
        "emf" => Arc::from("image/x-emf"),
        _ => Arc::from("application/octet-stream"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BC-2.01.001 AC-005 — SLIDE_MASTER_RELS_PATH is correct OOXML path.
    #[test]
    fn test_bc_2_01_001_slide_master_rels_path() {
        assert_eq!(
            SLIDE_MASTER_RELS_PATH,
            "ppt/slideMasters/_rels/slideMaster1.xml.rels"
        );
    }

    /// BC-2.01.001 AC-005 — media_type_from_extension returns correct MIME types.
    #[test]
    fn test_bc_2_01_001_media_type_from_extension() {
        assert_eq!(media_type_from_extension("png").as_ref(), "image/png");
        assert_eq!(media_type_from_extension("jpg").as_ref(), "image/jpeg");
        assert_eq!(media_type_from_extension("jpeg").as_ref(), "image/jpeg");
        assert_eq!(media_type_from_extension("gif").as_ref(), "image/gif");
        assert_eq!(media_type_from_extension("svg").as_ref(), "image/svg+xml");
        assert_eq!(media_type_from_extension("wmf").as_ref(), "image/x-wmf");
        assert_eq!(media_type_from_extension("emf").as_ref(), "image/x-emf");
        // Unknown extension falls back to octet-stream.
        assert_eq!(
            media_type_from_extension("xyz").as_ref(),
            "application/octet-stream"
        );
    }

    /// BC-2.01.001 AC-005 — media_type_from_extension is case-insensitive.
    #[test]
    fn test_bc_2_01_001_media_type_case_insensitive() {
        assert_eq!(media_type_from_extension("PNG").as_ref(), "image/png");
        assert_eq!(media_type_from_extension("JPG").as_ref(), "image/jpeg");
    }
}
