//! OOXML theme font extraction and host font availability checks.
//!
//! [`parse_theme_fonts`] extracts the heading (`<a:majorFont>`) and body
//! (`<a:minorFont>`) font names from `theme1.xml` bytes.
//!
//! [`font_available`] checks whether a given font is installed on the build host
//! using a platform-specific directory scan (BC-2.01.006).

use std::path::PathBuf;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::BrandError;
use crate::template::BrandFonts;

/// The deterministic fallback font chain (BC-2.01.006 invariant 3).
///
/// Resolution order: first candidate that is available on the build host wins.
/// If none are found, `"Arial"` (the final fallback) is returned unconditionally.
pub const FALLBACK_CHAIN: &[&str] = &["Aptos", "Calibri", "Arial"];

/// Parse heading and body font names from `theme1.xml` bytes.
///
/// Extracts `typeface` attributes from `<a:majorFont>` (heading) and
/// `<a:minorFont>` (body). Falls back to `"Calibri"` and emits a
/// `tracing::warn!` for any missing element (BC-2.01.001 AC-004).
///
/// # Errors
///
/// Returns [`BrandError::ParseError`] only if the XML is completely unparseable.
/// A missing font element is not an error — it triggers a fallback.
pub fn parse_theme_fonts(xml_bytes: &[u8]) -> Result<BrandFonts, BrandError> {
    let mut heading: Option<Arc<str>> = None;
    let mut body: Option<Arc<str>> = None;

    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    // Track which font context we are in: "major" or "minor".
    let mut current_font_context: Option<&'static str> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                match name_str {
                    "majorFont" => {
                        current_font_context = Some("major");
                    },
                    "minorFont" => {
                        current_font_context = Some("minor");
                    },
                    "latin" => {
                        if let Some(ctx) = current_font_context {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"typeface"
                                    && let Ok(val) = std::str::from_utf8(&attr.value)
                                {
                                    let typeface: Arc<str> = Arc::from(val);
                                    match ctx {
                                        "major" => heading = Some(typeface),
                                        "minor" => body = Some(typeface),
                                        _ => {},
                                    }
                                }
                            }
                        }
                    },
                    _ => {},
                }
            },
            Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                if name_str == "latin"
                    && let Some(ctx) = current_font_context
                {
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"typeface"
                            && let Ok(val) = std::str::from_utf8(&attr.value)
                        {
                            let typeface: Arc<str> = Arc::from(val);
                            match ctx {
                                "major" => heading = Some(typeface),
                                "minor" => body = Some(typeface),
                                _ => {},
                            }
                        }
                    }
                }
            },
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                match name_str {
                    "majorFont" | "minorFont" => {
                        current_font_context = None;
                    },
                    _ => {},
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!(error = %e, "XML parse error in theme1.xml font section; partial font data may be incomplete");
                break;
            },
            _ => {},
        }
        buf.clear();
    }

    let fallback: Arc<str> = Arc::from("Calibri");

    if heading.is_none() {
        tracing::warn!("OOXML theme majorFont missing; falling back to Calibri");
    }
    if body.is_none() {
        tracing::warn!("OOXML theme minorFont missing; falling back to Calibri");
    }

    Ok(BrandFonts {
        heading: heading.unwrap_or_else(|| Arc::clone(&fallback)),
        body: body.unwrap_or_else(|| Arc::clone(&fallback)),
    })
}

/// Check whether a font is installed on the build host.
///
/// Uses a platform-specific directory scan:
/// - macOS: `/System/Library/Fonts`, `/Library/Fonts`, `~/Library/Fonts`
/// - Windows: `C:\Windows\Fonts`
/// - Linux: `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts` (legacy),
///   `~/.local/share/fonts` (XDG Base Directory standard, modern)
///
/// Returns `true` if any file matching the font name (case-insensitive exact
/// stem match or name-hyphen/space prefix) is found. Returns `false` if the
/// font is not found or if the directories cannot be read.
///
/// Matching rules (FINDING-013 — tighter than prefix):
/// - Stem equals the font name exactly (e.g., `"arial"` == `"arial"`).
/// - Stem starts with `"{name}-"` (e.g., `"calibri-bold"`).
/// - Stem starts with `"{name} "` (e.g., `"calibri light"`).
///
/// This is a best-effort check. A full implementation would use `font-kit`
/// or system font enumeration APIs. The name-based scan is acceptable for v1.0
/// (documented limitation in STORY-022 implementation notes).
#[must_use]
pub fn font_available(name: &str) -> bool {
    let lower_name = name.to_lowercase();
    for dir in font_search_dirs() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    subdirs.push(path);
                    continue;
                }
                let file_name = entry.file_name();
                let fname_lower = file_name.to_string_lossy().to_lowercase();
                let stem = fname_lower
                    .rsplit_once('.')
                    .map_or(fname_lower.as_ref(), |(s, _)| s);
                if font_stem_matches(stem, &lower_name) {
                    return true;
                }
            }
            // Recurse one level into subdirectories (common on Linux).
            for sub_dir in subdirs {
                if let Ok(sub_entries) = std::fs::read_dir(&sub_dir) {
                    for sub_entry in sub_entries.flatten() {
                        let file_name = sub_entry.file_name();
                        let fname_lower = file_name.to_string_lossy().to_lowercase();
                        let stem = fname_lower
                            .rsplit_once('.')
                            .map_or(fname_lower.as_ref(), |(s, _)| s);
                        if font_stem_matches(stem, &lower_name) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Returns `true` if `stem` matches the `lower_name` font name.
///
/// Matching rules (FINDING-013):
/// - Exact equality (`"arial"` matches `"arial"`).
/// - Starts with `"{name}-"` (style suffix, e.g., `"calibri-bold"`).
/// - Starts with `"{name} "` (space-separated variant, e.g., `"calibri light"`).
#[must_use]
fn font_stem_matches(stem: &str, lower_name: &str) -> bool {
    stem == lower_name
        || stem.starts_with(&format!("{lower_name}-"))
        || stem.starts_with(&format!("{lower_name} "))
}

/// Resolve the first available fallback font from [`FALLBACK_CHAIN`].
///
/// If no font in the chain is available, returns `"Arial"` unconditionally
/// (the chain's final entry is always `"Arial"` which is ubiquitous).
#[must_use]
pub fn resolve_fallback() -> Arc<str> {
    for &candidate in FALLBACK_CHAIN {
        if font_available(candidate) {
            return Arc::from(candidate);
        }
    }
    // Final unconditional fallback — Arial is the last entry in FALLBACK_CHAIN.
    Arc::from("Arial")
}

/// Returns platform-specific font search directories.
///
/// Used by [`font_available`] to scope the directory scan.
///
/// Platform paths:
/// - macOS: `/System/Library/Fonts`, `/Library/Fonts`, `~/Library/Fonts`
/// - Windows: `C:\Windows\Fonts`
/// - Linux: `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts` (legacy),
///   `~/.local/share/fonts` (XDG Base Directory standard, modern)
#[must_use]
pub fn font_search_dirs() -> Vec<PathBuf> {
    if cfg!(target_os = "macos") {
        let mut dirs = vec![
            PathBuf::from("/System/Library/Fonts"),
            PathBuf::from("/Library/Fonts"),
        ];
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/Fonts"));
        }
        dirs
    } else if cfg!(target_os = "windows") {
        vec![PathBuf::from("C:/Windows/Fonts")]
    } else {
        // Linux and other Unix-like systems.
        let mut dirs = vec![
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
        ];
        if let Some(home) = dirs::home_dir() {
            // ~/.fonts is the legacy XDG per-user font directory.
            dirs.push(home.join(".fonts"));
        }
        // ~/.local/share/fonts/ is the modern XDG Base Directory standard path
        // (freedesktop.org spec) for per-user fonts on Linux.
        if let Some(data_dir) = dirs::data_dir() {
            dirs.push(data_dir.join("fonts"));
        }
        dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Theme1.xml with `<a:majorFont>` and `<a:minorFont>` elements.
    const THEME_XML_WITH_FONTS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="FontTheme">
  <a:themeElements>
    <a:fontScheme name="FontScheme">
      <a:majorFont>
        <a:latin typeface="Calibri Light" panose="020F0302020204030204"/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="Calibri" panose="020F0502020204030204"/>
      </a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    /// Theme1.xml with no font elements.
    const THEME_XML_NO_FONTS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="NoFontTheme">
  <a:themeElements>
    <a:fontScheme name="EmptyFontScheme">
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    /// Theme1.xml with Arial as both heading and body.
    const THEME_XML_ARIAL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="ArialTheme">
  <a:themeElements>
    <a:fontScheme name="ArialScheme">
      <a:majorFont>
        <a:latin typeface="Arial" panose="020B0604020202020204"/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="Arial" panose="020B0604020202020204"/>
      </a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

    /// BC-2.01.001 AC-004 — extracts majorFont and minorFont typeface attributes.
    ///
    /// Test vector: `THEME_XML_WITH_FONTS` → `BrandFonts` { heading: "Calibri Light", body: "Calibri" }.
    #[test]
    fn test_bc_2_01_001_parse_major_minor_fonts() {
        let fonts =
            parse_theme_fonts(THEME_XML_WITH_FONTS.as_bytes()).expect("font XML must parse");
        assert_eq!(
            fonts.heading.as_ref(),
            "Calibri Light",
            "heading font must be Calibri Light"
        );
        assert_eq!(fonts.body.as_ref(), "Calibri", "body font must be Calibri");
    }

    /// BC-2.01.001 AC-004 — missing font elements fall back to "Calibri".
    ///
    /// Test vector: `THEME_XML_NO_FONTS` → `BrandFonts` { heading: "Calibri", body: "Calibri" }.
    #[test]
    fn test_bc_2_01_001_missing_fonts_fallback() {
        let fonts = parse_theme_fonts(THEME_XML_NO_FONTS.as_bytes())
            .expect("empty font section must parse");
        assert_eq!(
            fonts.heading.as_ref(),
            "Calibri",
            "missing heading font must fall back to Calibri"
        );
        assert_eq!(
            fonts.body.as_ref(),
            "Calibri",
            "missing body font must fall back to Calibri"
        );
    }

    /// BC-2.01.001 AC-004 — Arial font extracted correctly when present.
    #[test]
    fn test_bc_2_01_001_parse_arial_font() {
        let fonts =
            parse_theme_fonts(THEME_XML_ARIAL.as_bytes()).expect("Arial font XML must parse");
        assert_eq!(fonts.heading.as_ref(), "Arial");
        assert_eq!(fonts.body.as_ref(), "Arial");
    }

    /// BC-2.01.006 invariant 3 — `FALLBACK_CHAIN` is deterministic and non-empty.
    #[test]
    fn test_bc_2_01_006_fallback_chain_non_empty() {
        assert!(
            !FALLBACK_CHAIN.is_empty(),
            "FALLBACK_CHAIN must not be empty"
        );
        // Arial must be present as the last-resort fallback.
        assert!(
            FALLBACK_CHAIN.contains(&"Arial"),
            "Arial must be in the fallback chain as last resort"
        );
    }

    /// BC-2.01.006 invariant 3 — fallback chain order is Aptos → Calibri → Arial.
    #[test]
    fn test_bc_2_01_006_fallback_chain_order() {
        assert_eq!(FALLBACK_CHAIN[0], "Aptos", "first fallback must be Aptos");
        assert_eq!(
            FALLBACK_CHAIN[1], "Calibri",
            "second fallback must be Calibri"
        );
        assert_eq!(FALLBACK_CHAIN[2], "Arial", "last fallback must be Arial");
    }

    /// BC-2.01.006 — `font_search_dirs` returns a non-empty list.
    #[test]
    fn test_bc_2_01_006_font_search_dirs_non_empty() {
        let dirs = font_search_dirs();
        assert!(
            !dirs.is_empty(),
            "font_search_dirs must return at least one directory"
        );
    }

    /// BC-2.01.006 — `font_available` returns a bool (does not panic).
    ///
    /// We cannot assert the return value as it depends on build host font installation.
    /// The test verifies the function does not panic and returns a valid bool.
    #[test]
    fn test_bc_2_01_006_font_available_does_not_panic() {
        // "Arial" is universally installed — but we don't assert true here
        // because CI environments may lack fonts. We verify it returns a bool.
        let _result: bool = font_available("Arial");
    }

    /// FINDING-013 — `font_stem_matches` uses exact or well-delimited prefix matching.
    ///
    /// Ensures that a broad font name like "Calibri" does not accidentally match
    /// "`CalibriBody`" (no separator) while still matching "Calibri-Bold" and "Calibri Light".
    #[test]
    fn test_bc_2_01_006_font_stem_matches_exact() {
        // Exact match.
        assert!(
            font_stem_matches("arial", "arial"),
            "exact match must return true"
        );
        // Hyphen-separated style suffix.
        assert!(
            font_stem_matches("calibri-bold", "calibri"),
            "calibri-bold must match 'calibri'"
        );
        // Space-separated variant.
        assert!(
            font_stem_matches("calibri light", "calibri"),
            "calibri light must match 'calibri'"
        );
        // No separator — must NOT match (FINDING-013 anti-pattern fix).
        assert!(
            !font_stem_matches("calibribody", "calibri"),
            "calibribody must NOT match 'calibri' (no separator)"
        );
        assert!(
            !font_stem_matches("arialnarrow", "arial"),
            "arialnarrow must NOT match 'arial' (no separator)"
        );
        // Completely different name.
        assert!(
            !font_stem_matches("helvetica", "arial"),
            "helvetica must not match 'arial'"
        );
    }

    /// FINDING-004 — truncated/malformed XML in font parsing does not panic.
    ///
    /// When the XML stream hits a parse error, the font parser logs via `tracing::warn!`
    /// and falls back to Calibri for any missing font elements.
    #[test]
    fn test_finding_004_malformed_font_xml_returns_fallback() {
        // Abruptly truncated XML that cuts off before the font elements.
        let truncated_xml =
            b"<?xml version=\"1.0\"?><a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:themeElements><a:fontScheme name=\"T\"><a:majorFont><a:latin typeface=\"Calibri Light\"/><!-- abrupt";
        let result = parse_theme_fonts(truncated_xml);
        // Must not panic — falls back to defaults for any missing context.
        assert!(
            result.is_ok(),
            "malformed XML must not produce a hard error in font parsing"
        );
        // Whatever was parsed (or fallback Calibri) must be returned.
        let fonts = result.unwrap();
        // We extracted majorFont before truncation, so heading might be Calibri Light.
        // minorFont was not reached, so body falls back to Calibri.
        assert_eq!(
            fonts.body.as_ref(),
            "Calibri",
            "body font falls back to Calibri after truncation"
        );
    }
}
