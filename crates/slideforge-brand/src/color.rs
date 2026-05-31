//! OOXML theme color extraction.
//!
//! The [`parse_theme_colors`] function parses the `theme1.xml` content from a
//! `.pptx` or `.docx` archive and returns all 12 OOXML scheme color slots in
//! ECMA-376 sequential order.
//!
//! ## ECMA-376 Color Slot Order
//!
//! The 12 slots must appear in this exact order:
//! `dk1`, `lt1`, `dk2`, `lt2`, `acc1`, `acc2`, `acc3`, `acc4`, `acc5`, `acc6`,
//! `hlink`, `folHlink`.
//!
//! ## Color Element Forms
//!
//! - `<a:srgbClr val="RRGGBB">` — hex color value (no `#` prefix in OOXML).
//! - `<a:sysClr lastClr="RRGGBB">` — system color; use `lastClr` attribute.
//! - `<a:schemeClr>` — relative scheme color; extract raw and emit warning.

use std::collections::HashMap;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::BrandError;
use crate::template::{COLOR_SLOT_NAMES, ColorSlot, ColorValue};

/// Default fallback hex colors for missing slots, keyed by OOXML slot name.
///
/// `dk1` / `dk2` → dark gray, `lt1` / `lt2` → light gray,
/// accent slots → placeholder gray, link slots → placeholder blue.
///
/// This is the **single source of truth** for per-slot default colors.
/// `extractor.rs` delegates to this function for unresolvable `SchemeRef`
/// fallbacks (F-024A-MED-2 — eliminates sibling-drift risk per TD-VSDD-060).
pub(crate) fn default_color_for_slot(slot_name: &str) -> Arc<str> {
    match slot_name {
        "dk1" | "dk2" => Arc::from("#404040"),
        "lt1" => Arc::from("#F0F0F0"),
        "lt2" => Arc::from("#D0D0D0"),
        "hlink" | "folHlink" => Arc::from("#0000EE"),
        _ => Arc::from("#808080"), // acc1-acc6 fallback
    }
}

/// Default fallback [`ColorValue::Hex`] for a missing slot.
fn default_color_value_for_slot(slot_name: &str) -> ColorValue {
    ColorValue::Hex(default_color_for_slot(slot_name))
}

/// Parse all 12 OOXML theme color slots from `theme1.xml` bytes.
///
/// Returns the 12-element array on success. Any missing slots produce
/// [`BrandError::MissingColorSlot`] entries in the returned `Vec<BrandError>`.
///
/// # Errors
///
/// Does not return a hard error — missing slots are reported via the second
/// return value. Returns an [`Err`] only if the XML is so malformed that no
/// useful parsing can proceed.
///
/// # Panics
///
/// This function cannot panic in practice. The `expect` on `try_into` is
/// infallible because `COLOR_SLOT_NAMES` has exactly 12 entries and we produce
/// exactly 12 `ColorSlot` values from it.
pub fn parse_theme_colors(
    xml_bytes: &[u8],
) -> Result<([ColorSlot; 12], Vec<BrandError>), BrandError> {
    // Map from slot name → ColorValue (Hex or SchemeRef).
    let mut found: HashMap<&'static str, ColorValue> = HashMap::new();

    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    // Track which color slot element we are currently inside.
    // e.g. when we see `<a:dk1>` we set current_slot = Some("dk1")
    let mut current_slot: Option<&'static str> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                // Check if this tag is a known color slot wrapper.
                if let Some(&slot) = COLOR_SLOT_NAMES.iter().find(|&&n| n == name_str) {
                    current_slot = Some(slot);
                    buf.clear();
                    continue;
                }

                // Check for color value elements inside a slot.
                if let Some(slot) = current_slot {
                    match name_str {
                        "srgbClr" => {
                            // Extract the `val` attribute (the base hex color).
                            //
                            // Child transform elements (lumMod, lumOff, tint, shade) are
                            // intentionally NOT applied here — the raw hex is preserved as-is.
                            // BC-2.01.003 EC-003 scopes transforms to schemeClr only; srgbClr
                            // transform-aware extraction (widening EC-003) is tracked as STORY-076
                            // (Brand Loader: Transform-Aware Theme Color Extraction).
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val"
                                    && let Ok(val) = std::str::from_utf8(&attr.value)
                                {
                                    if val.len() == 6 && val.chars().all(|c| c.is_ascii_hexdigit())
                                    {
                                        let hex =
                                            Arc::from(format!("#{}", val.to_uppercase()).as_str());
                                        found.insert(slot, ColorValue::Hex(hex));
                                    } else {
                                        tracing::warn!(
                                            slot,
                                            val,
                                            "srgbClr val is not a 6-character hex string; \
                                             using default color for slot"
                                        );
                                    }
                                }
                            }
                        },
                        "sysClr" => {
                            // Use `lastClr` attribute as the resolved hex value.
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"lastClr"
                                    && let Ok(val) = std::str::from_utf8(&attr.value)
                                {
                                    if val.len() == 6 && val.chars().all(|c| c.is_ascii_hexdigit())
                                    {
                                        let hex =
                                            Arc::from(format!("#{}", val.to_uppercase()).as_str());
                                        found.insert(slot, ColorValue::Hex(hex));
                                    } else {
                                        tracing::warn!(
                                            slot,
                                            val,
                                            "sysClr lastClr is not a 6-character hex string; \
                                             using default color for slot"
                                        );
                                    }
                                }
                            }
                        },
                        "schemeClr" => {
                            // Relative scheme color reference (e.g., val="dk1", val="accent1").
                            // We cannot resolve the absolute hex without a rendering context,
                            // so we store a ColorValue::SchemeRef with a warning.
                            // AC-002: schemeClr with lumMod/tint/shade → extracted value
                            // with inline warning.
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val"
                                    && let Ok(val) = std::str::from_utf8(&attr.value)
                                {
                                    tracing::warn!(
                                        slot,
                                        scheme_ref = val,
                                        "schemeClr in theme1.xml color slot; \
                                         storing scheme reference — actual hex may differ \
                                         depending on the active theme"
                                    );
                                    let scheme_ref = Arc::from(val.to_lowercase().as_str());
                                    found.insert(slot, ColorValue::SchemeRef(scheme_ref));
                                }
                            }
                        },
                        _ => {},
                    }
                }
            },
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name_str = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                // If we close the current slot element, clear tracking.
                if let Some(slot) = current_slot
                    && name_str == slot
                {
                    current_slot = None;
                }
            },
            // XML errors: log the error with context then treat as EOF.
            // We accumulate whatever data was extracted before the error.
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!(error = %e, "XML parse error in theme1.xml; partial color data may be incomplete");
                break;
            },
            _ => {},
        }
        buf.clear();
    }

    // Build the 12-element output array and collect warnings for missing slots.
    let mut warnings = Vec::new();
    let slots: Vec<ColorSlot> = COLOR_SLOT_NAMES
        .iter()
        .map(|&name| {
            if let Some(color_value) = found.remove(name) {
                ColorSlot {
                    name: Arc::from(name),
                    value: color_value,
                }
            } else {
                let inferred_hex = default_color_for_slot(name);
                warnings.push(BrandError::MissingColorSlot {
                    slot_name: Arc::from(name),
                    inferred_hex: Arc::clone(&inferred_hex),
                    derivation: Arc::from("default inference"),
                });
                tracing::warn!(
                    slot = name,
                    "OOXML color slot missing; using default inference"
                );
                ColorSlot {
                    name: Arc::from(name),
                    value: default_color_value_for_slot(name),
                }
            }
        })
        .collect();

    // SAFETY: We always produce exactly 12 elements from COLOR_SLOT_NAMES (len == 12).
    let array: [ColorSlot; 12] = slots
        .try_into()
        .expect("COLOR_SLOT_NAMES has exactly 12 entries");

    Ok((array, warnings))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::template::COLOR_SLOT_NAMES;

    /// Minimal theme1.xml with 12 `<a:srgbClr>` color elements.
    ///
    /// Colors are in ECMA-376 order: dk1, lt1, dk2, lt2, acc1-6, hlink, folHlink.
    const THEME_XML_12_SRGB: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
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
  </a:themeElements>
</a:theme>"#;

    /// Theme1.xml where dk1 uses `<a:sysClr lastClr="FFFFFF">`.
    const THEME_XML_SYSCLR: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="SysTheme">
  <a:themeElements>
    <a:clrScheme name="SysScheme">
      <a:dk1><a:sysClr lastClr="FFFFFF" val="windowText"/></a:dk1>
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
  </a:themeElements>
</a:theme>"#;

    /// Theme1.xml with only 8 color slots (missing acc3–acc6).
    const THEME_XML_8_COLORS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="PartialTheme">
  <a:themeElements>
    <a:clrScheme name="PartialScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"/></a:dk2>
      <a:lt2><a:srgbClr val="F5F5F5"/></a:lt2>
      <a:acc1><a:srgbClr val="0066CC"/></a:acc1>
      <a:acc2><a:srgbClr val="FF6B35"/></a:acc2>
      <a:hlink><a:srgbClr val="0000EE"/></a:hlink>
      <a:folHlink><a:srgbClr val="551A8B"/></a:folHlink>
    </a:clrScheme>
  </a:themeElements>
</a:theme>"#;

    /// Theme1.xml with no color elements at all.
    const THEME_XML_EMPTY: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="EmptyTheme">
  <a:themeElements>
    <a:clrScheme name="EmptyScheme">
    </a:clrScheme>
  </a:themeElements>
</a:theme>"#;

    /// BC-2.01.001 postcondition 1 — parse 12 sRGB colors → 12 `ColorSlots`, no warnings.
    ///
    /// Test vector: `THEME_XML_12_SRGB` → 12 slots, 0 warnings.
    #[test]
    fn test_bc_2_01_001_parse_12_srgb_colors() {
        let result = parse_theme_colors(THEME_XML_12_SRGB.as_bytes());
        let (slots, warnings) = result.expect("valid theme XML must parse without hard error");
        assert_eq!(
            slots.len(),
            12,
            "must return exactly 12 color slots, DI-015"
        );
        assert!(
            warnings.is_empty(),
            "no missing slot warnings expected for complete theme, got: {warnings:?}"
        );
        // Spot-check specific values.
        assert_eq!(slots[0].name.as_ref(), "dk1");
        assert_eq!(slots[0].hex(), Some("#000000"));
        assert_eq!(slots[1].name.as_ref(), "lt1");
        assert_eq!(slots[1].hex(), Some("#FFFFFF"));
        assert_eq!(slots[4].name.as_ref(), "acc1");
        assert_eq!(slots[4].hex(), Some("#0066CC"));
        assert_eq!(slots[11].name.as_ref(), "folHlink");
        assert_eq!(slots[11].hex(), Some("#551A8B"));
    }

    /// BC-2.01.001 AC-002 — sysClr uses `lastClr` attribute as hex value.
    ///
    /// Test vector: `THEME_XML_SYSCLR` → dk1.hex == "#FFFFFF" (from lastClr="FFFFFF").
    #[test]
    fn test_bc_2_01_001_parse_sysclr_uses_lastclr() {
        let result = parse_theme_colors(THEME_XML_SYSCLR.as_bytes());
        let (slots, _warnings) = result.expect("sysClr theme XML must parse without hard error");
        assert_eq!(slots[0].name.as_ref(), "dk1");
        assert_eq!(
            slots[0].hex(),
            Some("#FFFFFF"),
            "sysClr dk1 lastClr=FFFFFF must produce #FFFFFF"
        );
    }

    /// BC-2.01.001 EC-003 — template with only 8 colors produces 4 `MissingColorSlot` warnings.
    ///
    /// Test vector: `THEME_XML_8_COLORS` → 4 `BrandError::MissingColorSlot` warnings.
    #[test]
    fn test_bc_2_01_001_missing_slot_emits_warning() {
        let result = parse_theme_colors(THEME_XML_8_COLORS.as_bytes());
        let (slots, warnings) = result.expect("partial theme XML must parse without hard error");
        assert_eq!(
            slots.len(),
            12,
            "invariant DI-015: must always return 12 slots even when some are missing"
        );
        assert_eq!(
            warnings.len(),
            4,
            "8-color template must emit exactly 4 MissingColorSlot warnings, got: {warnings:?}"
        );
        // All warnings must be MissingColorSlot variants.
        for w in &warnings {
            assert!(
                matches!(w, BrandError::MissingColorSlot { .. }),
                "expected MissingColorSlot warning, got: {w:?}"
            );
        }
    }

    /// BC-2.01.001 AC-002 / invariant 1 — colors returned in ECMA-376 sequential order.
    ///
    /// Test vector: `THEME_XML_12_SRGB` → slots at positions 0-11 match `COLOR_SLOT_NAMES`.
    #[test]
    fn test_bc_2_01_001_color_order_preserved() {
        let result = parse_theme_colors(THEME_XML_12_SRGB.as_bytes());
        let (slots, _warnings) = result.expect("12-color theme must parse cleanly");
        let expected_names = COLOR_SLOT_NAMES;
        for (i, expected_name) in expected_names.iter().enumerate() {
            assert_eq!(
                slots[i].name.as_ref(),
                *expected_name,
                "ECMA-376 order violation: position {i} must be '{expected_name}', \
                 got '{}'",
                slots[i].name
            );
        }
    }

    /// BC-2.01.001 EC-003 — empty theme produces 12 `MissingColorSlot` warnings.
    ///
    /// Test vector: `THEME_XML_EMPTY` → 12 `BrandError::MissingColorSlot` warnings.
    #[test]
    fn test_bc_2_01_001_empty_theme_all_missing() {
        let result = parse_theme_colors(THEME_XML_EMPTY.as_bytes());
        let (slots, warnings) = result.expect("empty theme XML must parse without hard error");
        assert_eq!(
            slots.len(),
            12,
            "invariant DI-015: even an empty theme must return 12 slots"
        );
        assert_eq!(
            warnings.len(),
            12,
            "empty theme must emit 12 MissingColorSlot warnings, got: {warnings:?}"
        );
    }

    /// BC-2.01.001 — hex values are normalized to uppercase #RRGGBB format.
    #[test]
    fn test_bc_2_01_001_hex_values_uppercase_normalized() {
        let result = parse_theme_colors(THEME_XML_12_SRGB.as_bytes());
        let (slots, _warnings) = result.expect("12-color theme must parse cleanly");
        for slot in &slots {
            let hex = slot
                .hex()
                .expect("all slots in THEME_XML_12_SRGB must be Hex variants");
            assert!(
                hex.starts_with('#'),
                "hex value must start with '#', got: {hex}"
            );
            assert_eq!(
                hex.len(),
                7,
                "hex value must be 7 chars (#RRGGBB), got: {hex}"
            );
            // Verify hex digits are uppercase.
            let digits = &hex[1..];
            assert_eq!(
                digits,
                digits.to_uppercase(),
                "hex digits must be uppercase, got: {hex}"
            );
        }
    }

    /// BC-2.01.001 AC-002 — schemeClr elements store a `ColorValue::SchemeRef` with a warning.
    ///
    /// When a slot uses `<a:schemeClr val="dk1">` (a self-referential scheme color),
    /// the parser stores `ColorValue::SchemeRef("dk1")` and emits a `tracing::warn`.
    /// No `MissingColorSlot` warning is emitted — the slot IS present, just unresolved.
    #[test]
    fn test_bc_2_01_001_schemeclr_stored_as_reference() {
        use crate::template::ColorValue;

        // A theme where dk1 uses schemeClr self-reference.
        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="SchemeTheme">
  <a:themeElements>
    <a:clrScheme name="SchemeScheme">
      <a:dk1><a:schemeClr val="dk1"/></a:dk1>
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
  </a:themeElements>
</a:theme>"#;
        let result = parse_theme_colors(theme_xml.as_bytes());
        let (slots, warnings) = result.expect("schemeClr theme must parse without hard error");
        // dk1 uses schemeClr so it is present — no MissingColorSlot warning for dk1.
        assert_eq!(
            warnings.len(),
            0,
            "schemeClr is present (not missing), so no MissingColorSlot warnings expected"
        );
        // dk1 must be a SchemeRef variant (AC-002 — type contract enforced).
        assert!(
            matches!(&slots[0].value, ColorValue::SchemeRef(_)),
            "schemeClr slot must have ColorValue::SchemeRef variant, got: {:?}",
            slots[0].value
        );
        assert!(
            !slots[0].is_resolved(),
            "schemeClr slot must not be resolved"
        );
        assert_eq!(
            slots[0].hex(),
            None,
            "schemeClr slot must return None from hex()"
        );
        // The scheme reference string must be the lowercase val attribute.
        assert_eq!(
            slots[0].value.as_scheme_ref(),
            Some("dk1"),
            "schemeClr val='dk1' must produce SchemeRef(\"dk1\")"
        );
    }

    /// FINDING-003 — srgbClr with an invalid (too-short) val uses default color and emits a warning.
    ///
    /// When `<a:srgbClr val="00FF"/>` (4 chars, not 6), the parser must NOT use
    /// the invalid value. The slot must fall through to the default inference path,
    /// producing a `MissingColorSlot` warning.
    #[test]
    fn test_finding_003_srgb_clr_invalid_short_hex_uses_default() {
        // dk1 has val="00FF" — 4 hex chars, not 6; invalid.
        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="InvalidHexTheme">
  <a:themeElements>
    <a:clrScheme name="InvalidHexScheme">
      <a:dk1><a:srgbClr val="00FF"/></a:dk1>
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
  </a:themeElements>
</a:theme>"#;
        let (slots, warnings) = parse_theme_colors(theme_xml.as_bytes())
            .expect("invalid hex theme must parse without hard error");
        // dk1 had invalid val — it must fall back to a MissingColorSlot warning + default.
        assert_eq!(
            warnings.len(),
            1,
            "invalid srgbClr val must produce exactly 1 MissingColorSlot warning, got: {warnings:?}"
        );
        assert!(
            matches!(&warnings[0], BrandError::MissingColorSlot { slot_name, .. } if slot_name.as_ref() == "dk1"),
            "MissingColorSlot warning must be for 'dk1', got: {:?}",
            warnings[0]
        );
        // dk1 must use the default dark-gray fallback (not the 4-char invalid value).
        assert_ne!(
            slots[0].hex(),
            Some("#00FF"),
            "invalid 4-char hex must NOT be stored as hex value"
        );
        assert!(
            slots[0].hex().is_some(),
            "dk1 must still have a default hex color after invalid val"
        );
    }

    /// FINDING-003 — srgbClr with non-hex characters (e.g. "ZZZZZZ") uses default color.
    #[test]
    fn test_finding_003_srgb_clr_invalid_non_hex_chars_uses_default() {
        // dk1 has val="ZZZZZZ" — 6 chars but not hex digits; invalid.
        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="NonHexTheme">
  <a:themeElements>
    <a:clrScheme name="NonHexScheme">
      <a:dk1><a:srgbClr val="ZZZZZZ"/></a:dk1>
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
  </a:themeElements>
</a:theme>"#;
        let (slots, warnings) = parse_theme_colors(theme_xml.as_bytes())
            .expect("non-hex theme must parse without hard error");
        // dk1 had "ZZZZZZ" (non-hex) — must produce a MissingColorSlot warning.
        assert_eq!(
            warnings.len(),
            1,
            "non-hex srgbClr val must produce exactly 1 MissingColorSlot warning, got: {warnings:?}"
        );
        // dk1 must not store "ZZZZZZ".
        assert_ne!(
            slots[0].hex().map(str::to_uppercase),
            Some("#ZZZZZZ".to_owned()),
            "invalid non-hex val must NOT be stored as hex value"
        );
        // dk1 must have the dark-gray default fallback.
        assert!(
            slots[0].hex().is_some(),
            "dk1 must still have a default hex color after invalid val"
        );
    }

    /// FINDING-004 — truncated/malformed XML does not panic; returns partial data.
    ///
    /// When the XML stream hits a parse error mid-document, the parser logs the error
    /// via `tracing::warn!` and returns whatever color slots were successfully extracted
    /// before the error.
    #[test]
    fn test_finding_004_malformed_xml_returns_partial_data() {
        // XML that is syntactically valid for the first few slots then abruptly cut off.
        let truncated_xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
  <a:themeElements><a:clrScheme name=\"T\">\
    <a:dk1><a:srgbClr val=\"000000\"/></a:dk1>\
    <a:lt1><a:srgbClr val=\"FFFFFF\"/></a:lt1>\
    <!-- abrupt end with invalid bytes";
        let result = parse_theme_colors(truncated_xml);
        // Must not panic — must return partial data.
        assert!(
            result.is_ok(),
            "malformed XML must not produce a hard error"
        );
        let (slots, _warnings) = result.unwrap();
        assert_eq!(
            slots.len(),
            12,
            "invariant: always 12 slots even after XML error"
        );
        // dk1 and lt1 were extracted before the error.
        assert_eq!(
            slots[0].hex(),
            Some("#000000"),
            "dk1 must be extracted before truncation"
        );
        assert_eq!(
            slots[1].hex(),
            Some("#FFFFFF"),
            "lt1 must be extracted before truncation"
        );
    }
}
