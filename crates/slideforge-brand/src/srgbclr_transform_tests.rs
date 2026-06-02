//! STORY-076 Red Gate tests: Transform-Aware Theme Color Extraction (srgbClr lumMod/tint/shade).
//!
//! ## Expected-but-missing symbols (compile errors until implementation)
//!
//! All tests below reference `ColorSlot::is_derived`, which does NOT yet exist on the
//! `ColorSlot` struct in `template.rs`. Adding an `is_derived: bool` field to `ColorSlot`
//! (and threading it through all construction sites in `color.rs`) is the primary
//! implementation task for STORY-076.
//!
//! The `parse_theme_colors_with_derived_info` helper used in some tests below is a
//! conceptual alias for `parse_theme_colors` — the function already exists but currently
//! returns `ColorSlot` values that lack the `is_derived` field. Tests access `.is_derived`
//! on the returned slots; this fails to compile until the field is added.
//!
//! ## Test coverage map
//!
//! | Test name | AC | BC | Edge Case |
//! |-----------|----|----|-----------|
//! | `test_BC_2_01_001_EC006_srgbclr_lummod_sets_derived_flag` | AC-001 | BC-2.01.001 EC-006 | — |
//! | `test_BC_2_01_001_EC006_srgbclr_tint_sets_derived_flag` | AC-001 | BC-2.01.001 EC-006 | — |
//! | `test_BC_2_01_001_EC006_srgbclr_shade_sets_derived_flag` | AC-001 | BC-2.01.001 EC-006 | — |
//! | `test_BC_2_01_001_EC006_srgbclr_lumoff_sets_derived_flag` | AC-001 | BC-2.01.001 EC-006 | EC-002 |
//! | `test_BC_2_01_001_EC006_srgbclr_lummod_preserves_base_hex` | AC-001 | BC-2.01.001 EC-006 | — |
//! | `test_BC_2_01_001_EC006_srgbclr_multiple_transforms_single_warn` | AC-001/AC-003 | EC-006 | EC-001 |
//! | `test_BC_2_01_001_EC006_srgbclr_without_transforms_not_derived` | AC-004 | BC-2.01.001 post-1 | regression |
//! | `test_BC_2_01_001_EC006_srgbclr_clean_val_regression` | AC-004 | BC-2.01.001 post-1 | regression |
//! | `test_BC_2_01_001_EC006_derived_flag_not_set_on_sysclr` | AC-004 | BC-2.01.001 post-1 | regression |
//! | `test_BC_2_01_001_EC006_srgbclr_no_hsl_resolution_performed` | AC-001 | BC-2.01.001 EC-006 | Option B |
//! | `test_BC_2_01_001_EC006_srgbclr_warn_names_slot_and_transform` | AC-003 | BC-2.01.001 EC-006 | observability |
//! | `test_BC_2_01_003_EC003_extractor_derived_srgbclr_emits_comment` | AC-002 | BC-2.01.003 EC-003 | widened |
//! | `test_BC_2_01_003_EC003_extractor_clean_srgbclr_no_comment` | AC-004 | BC-2.01.003 invariant | regression |
//! | `test_BC_2_01_003_EC003_extractor_unified_flag_drives_comment` | AC-002 | BC-2.01.003 EC-003 | unified path |

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(non_snake_case)] // test names use BC-ID naming convention (e.g. test_BC_2_01_001_...)
#[allow(clippy::doc_markdown)] // test doc comments use prose-style type names without backticks
#[allow(clippy::items_after_statements)] // static COUNTER after let-bindings in test helpers
#[allow(unused_mut)] // `let mut colors: [ColorSlot; 12]` pattern in tests
mod tests {
    use std::sync::Arc;

    use crate::color::parse_theme_colors;
    use crate::template::{ColorSlot, ColorValue};

    // ─────────────────────────────────────────────────────────────────────────
    // Fixture XML helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Build a minimal theme1.xml with a single specified srgbClr element in the
    /// `dk2` slot, inserting raw inner XML (e.g. transform children) inside the
    /// `<a:srgbClr>` element.
    ///
    /// The remaining 11 slots use clean `<a:srgbClr val="XXXXXX"/>` values so the
    /// test only needs to inspect the `dk2` (index 2) slot.
    fn theme_xml_with_dk2_srgbclr(inner_srgbclr_xml: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="TestTheme">
  <a:themeElements>
    <a:clrScheme name="TestScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2>{inner_srgbclr_xml}</a:dk2>
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
</a:theme>"#,
        )
    }

    /// Build a minimal full-12-slot theme1.xml where ALL slots use clean `srgbClr`
    /// (no transform children). Identical to the STORY-022 regression fixture.
    const CLEAN_12_SRGB_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="CleanTheme">
  <a:themeElements>
    <a:clrScheme name="CleanScheme">
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

    /// Theme1.xml where dk1 uses `<a:sysClr lastClr="1A1A1A">`.
    const SYSCLR_THEME_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="SysClrTheme">
  <a:themeElements>
    <a:clrScheme name="SysClrScheme">
      <a:dk1><a:sysClr val="windowText" lastClr="1A1A1A"/></a:dk1>
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

    // ─────────────────────────────────────────────────────────────────────────
    // AC-001 / BC-2.01.001 EC-006 — srgbClr with transforms sets is_derived flag
    //
    // Each test asserts `slot.is_derived == true` — this field does not yet exist
    // on `ColorSlot` and will cause a compile error (Red Gate).
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-2.01.001 EC-006 / AC-001 — `<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>`
    /// sets `ColorSlot.is_derived = true` on the dk2 slot.
    ///
    /// Canonical test vector (BC-2.01.001): base hex stored verbatim = `#003087`.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot` has no `is_derived` field. This test will NOT compile until the field
    /// is added to `template.rs` and `color.rs` threads it through.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_lummod_sets_derived_flag() {
        // Canonical test vector from BC-2.01.001: dk2 = srgbClr val=003087 + lumMod child.
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>"#,
        );
        let (slots, warnings) = parse_theme_colors(xml.as_bytes())
            .expect("valid theme XML must parse without hard error");

        // No MissingColorSlot warnings — all 12 slots present.
        assert_eq!(
            warnings.len(),
            0,
            "srgbClr with lumMod must not emit MissingColorSlot warnings, got: {warnings:?}"
        );

        let dk2 = &slots[2]; // ECMA-376 index 2 = dk2
        assert_eq!(dk2.name.as_ref(), "dk2", "slot at index 2 must be dk2");

        // AC-001 (BC-2.01.001 EC-006): is_derived must be true when lumMod child is present.
        // RED GATE: `ColorSlot` has no `is_derived` field. Compile error until implemented.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006: srgbClr with lumMod child must set is_derived = true on slot dk2"
        );

        // AC-001 (BC-2.01.001 EC-006): base val hex stored verbatim — no transform applied.
        assert_eq!(
            dk2.hex(),
            Some("#003087"),
            "BC-2.01.001 EC-006: base hex #003087 must be stored verbatim (Option B — no HSL resolution)"
        );
    }

    /// BC-2.01.001 EC-006 / AC-001 — `<a:srgbClr val="C0392B"><a:tint val="50000"/></a:srgbClr>`
    /// sets `ColorSlot.is_derived = true`.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_tint_sets_derived_flag() {
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="C0392B"><a:tint val="50000"/></a:srgbClr>"#,
        );
        let (slots, _) = parse_theme_colors(xml.as_bytes())
            .expect("valid theme XML must parse without hard error");

        let dk2 = &slots[2];

        // AC-001: tint child triggers is_derived flag.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006: srgbClr with tint child must set is_derived = true"
        );

        // Base hex preserved verbatim.
        assert_eq!(
            dk2.hex(),
            Some("#C0392B"),
            "tint child must NOT modify stored hex — base val stored verbatim per Option B"
        );
    }

    /// BC-2.01.001 EC-006 / AC-001 — `<a:srgbClr val="8E44AD"><a:shade val="60000"/></a:srgbClr>`
    /// sets `ColorSlot.is_derived = true`.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_shade_sets_derived_flag() {
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="8E44AD"><a:shade val="60000"/></a:srgbClr>"#,
        );
        let (slots, _) = parse_theme_colors(xml.as_bytes())
            .expect("valid theme XML must parse without hard error");

        let dk2 = &slots[2];

        // AC-001: shade child triggers is_derived flag.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006: srgbClr with shade child must set is_derived = true"
        );

        assert_eq!(
            dk2.hex(),
            Some("#8E44AD"),
            "shade child must NOT modify stored hex — base val stored verbatim per Option B"
        );
    }

    /// BC-2.01.001 EC-006 / AC-001 / EC-002 — `<a:lumOff val="20000"/>` is also
    /// a recognized transform child; sets `is_derived = true`.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_lumoff_sets_derived_flag() {
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="2E86C1"><a:lumOff val="20000"/></a:srgbClr>"#,
        );
        let (slots, _) = parse_theme_colors(xml.as_bytes())
            .expect("valid theme XML must parse without hard error");

        let dk2 = &slots[2];

        // AC-001 / EC-002: lumOff child triggers is_derived flag.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006 EC-002: srgbClr with lumOff child must set is_derived = true"
        );

        assert_eq!(
            dk2.hex(),
            Some("#2E86C1"),
            "lumOff child must NOT modify stored hex — base val stored verbatim per Option B"
        );
    }

    /// BC-2.01.001 EC-006 / AC-001 — base hex stored verbatim (NOT modified by the
    /// lumMod transform). Canonical test vector from BC-2.01.001: dk2 val="003087"
    /// with lumMod val="75000" — the stored hex must be `#003087`, NOT the
    /// luminance-modulated approximation.
    ///
    /// This explicitly tests the Option B decision: NO HSL arithmetic is performed.
    ///
    /// # Red Gate
    ///
    /// Depends on `is_derived` field existing. Compile error until implemented.
    /// Also validates base-hex invariant — real assertion, not a tautology.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_lummod_preserves_base_hex() {
        // BC-2.01.001 canonical test vector: val=003087 + lumMod=75000.
        // 75000 in OOXML percent-thousandths = 75% luminance factor.
        // IF HSL resolution were applied, the resulting hex would be approximately #002465.
        // Option B mandates: base hex stored verbatim = #003087.
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>"#,
        );
        let (slots, _) = parse_theme_colors(xml.as_bytes())
            .expect("valid theme XML must parse without hard error");

        let dk2 = &slots[2];

        // is_derived must be true (this is the load-bearing flag assertion).
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006: lumMod must set is_derived = true"
        );

        // Stored hex must be the RAW base val, not the luminance-modulated result.
        // lumMod=75000 applied to #003087 → approx #002465 in HSL arithmetic.
        // Asserting the stored value is NOT the resolved color proves Option B is honored.
        assert_eq!(
            dk2.hex(),
            Some("#003087"),
            "BC-2.01.001 EC-006 Option B: hex must be verbatim base '003087', NOT the \
             luminance-modulated resolved color (approx #002465 if 75% lumMod were applied)"
        );

        // Confirming that the resolution DID NOT happen by also verifying it's not #002465.
        assert_ne!(
            dk2.hex(),
            Some("#002465"),
            "HSL resolution must NOT be performed in v1.0 (Option B defers to v2)"
        );
    }

    /// BC-2.01.001 EC-006 / AC-001 / EC-001 — multiple transform children on the SAME
    /// srgbClr element (both lumMod and tint) set `is_derived = true` exactly once;
    /// a single tracing::warn! is emitted listing ALL transform types found in that
    /// one message (EC-001 forbids one-warn-per-transform noisy behavior).
    ///
    /// Load-bearing assertions (MED-1 fix):
    /// 1. Exactly ONE warn message captured for the slot (count == 1, not 2).
    /// 2. That single message contains BOTH "lumMod" AND "tint".
    ///
    /// A regression emitting one warn per transform would produce count == 2 and
    /// FAIL assertion (1). A regression listing only the first transform would
    /// omit "tint" and FAIL assertion (2).
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_multiple_transforms_single_warn() {
        use std::sync::{Arc as StdArc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt as _;

        // Reuse the MessageCapturingLayer harness pattern from
        // test_BC_2_01_001_EC006_srgbclr_warn_names_slot_and_transform (lines ~560-637).
        struct MessageCapturingLayer {
            messages: StdArc<Mutex<Vec<String>>>,
        }

        struct MessageVisitor {
            message: Option<String>,
        }

        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_owned());
                }
            }
        }

        impl<S: tracing::Subscriber> Layer<S> for MessageCapturingLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor { message: None };
                    event.record(&mut visitor);
                    if let Some(msg) = visitor.message {
                        let _ = self.messages.lock().map(|mut guard| guard.push(msg));
                    }
                }
            }
        }

        let captured_messages: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));
        let layer = MessageCapturingLayer {
            messages: StdArc::clone(&captured_messages),
        };
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // EC-001 canonical vector: srgbClr with lumMod AND tint — both recognized transforms.
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="003087"><a:lumMod val="75000"/><a:tint val="30000"/></a:srgbClr>"#,
        );
        let (slots, warnings) = parse_theme_colors(xml.as_bytes())
            .expect("multi-transform theme XML must parse without hard error");

        // No MissingColorSlot warnings.
        assert_eq!(
            warnings.len(),
            0,
            "multi-transform srgbClr must not emit MissingColorSlot warnings"
        );

        let dk2 = &slots[2];

        // AC-001 / EC-001: multiple transforms still set is_derived = true (only once).
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            dk2.is_derived,
            "BC-2.01.001 EC-006 EC-001: srgbClr with multiple transform children must set \
             is_derived = true"
        );

        // Base hex verbatim.
        assert_eq!(
            dk2.hex(),
            Some("#003087"),
            "multi-transform srgbClr: base hex must be stored verbatim"
        );

        // ── EC-001 single-warn load-bearing assertions (MED-1 fix) ──────────────
        //
        // Collect only the warn messages that mention the "dk2" slot — isolating
        // the warn(s) emitted by this parse call from any ambient noise.
        let messages = captured_messages.lock().unwrap();
        let dk2_warns: Vec<&String> = messages.iter().filter(|msg| msg.contains("dk2")).collect();

        // Assertion 1: exactly ONE warn for this slot.
        // If the impl emits one warn per transform this count will be 2, not 1.
        assert_eq!(
            dk2_warns.len(),
            1,
            "EC-001: exactly ONE tracing::warn! must be emitted for slot 'dk2' when \
             srgbClr has multiple transform children — got {} warn(s).\n\
             Captured dk2 WARN messages: {dk2_warns:?}",
            dk2_warns.len(),
        );

        // Assertion 2: the single message lists BOTH transforms.
        // If the impl only lists the first transform "tint" would be absent.
        let single_warn = dk2_warns[0];
        assert!(
            single_warn.contains("lumMod"),
            "EC-001: the single warn! for slot 'dk2' must name 'lumMod' transform.\n\
             Actual message: {single_warn:?}"
        );
        assert!(
            single_warn.contains("tint"),
            "EC-001: the single warn! for slot 'dk2' must name 'tint' transform.\n\
             Actual message: {single_warn:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-004 / BC-2.01.001 postcondition 1 — srgbClr WITHOUT transforms: no derived flag
    //
    // Regression: existing STORY-022 behavior must be unaffected.
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-2.01.001 postcondition 1 / AC-004 — `<a:srgbClr val="FF0000"/>` (no transform
    /// children) sets `ColorSlot.is_derived = false` and stores hex verbatim.
    ///
    /// This is the primary regression guard for STORY-022 behavior.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    /// Once the field exists, this test verifies the negative case (is_derived = false).
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_without_transforms_not_derived() {
        // Canonical test vector from BC-2.01.001 boundary row: clean srgbClr, no transforms.
        let xml = theme_xml_with_dk2_srgbclr(r#"<a:srgbClr val="FF0000"/>"#);
        let (slots, warnings) = parse_theme_colors(xml.as_bytes())
            .expect("clean srgbClr theme XML must parse without hard error");

        assert_eq!(
            warnings.len(),
            0,
            "clean srgbClr (no transforms) must not emit any warnings"
        );

        let dk2 = &slots[2];
        assert_eq!(dk2.name.as_ref(), "dk2");

        // AC-004 regression: clean srgbClr MUST NOT have is_derived set.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            !dk2.is_derived,
            "BC-2.01.001 AC-004 regression: srgbClr WITHOUT transform children must have \
             is_derived = false"
        );

        // Value must still be the correct hex.
        assert_eq!(
            dk2.hex(),
            Some("#FF0000"),
            "clean srgbClr val=FF0000 must produce #FF0000"
        );
    }

    /// BC-2.01.001 postcondition 1 / AC-004 — full 12-slot clean theme (STORY-022 fixture):
    /// ALL slots have `is_derived = false` and no warnings are emitted.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_clean_val_regression() {
        let (slots, warnings) = parse_theme_colors(CLEAN_12_SRGB_XML.as_bytes())
            .expect("clean 12-color theme must parse without hard error");

        assert_eq!(
            warnings.len(),
            0,
            "clean 12-srgb theme must produce 0 warnings (STORY-022 regression)"
        );
        assert_eq!(slots.len(), 12);

        // Every slot must have is_derived = false — no transforms present.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        for slot in &slots {
            assert!(
                !slot.is_derived,
                "STORY-022 regression: clean srgbClr slot '{}' must have is_derived = false",
                slot.name
            );
        }

        // Spot-check canonical values from the clean fixture.
        assert_eq!(slots[0].hex(), Some("#000000"), "dk1 regression: #000000");
        assert_eq!(slots[2].hex(), Some("#003087"), "dk2 regression: #003087");
        assert_eq!(slots[4].hex(), Some("#0066CC"), "acc1 regression: #0066CC");
        assert_eq!(
            slots[11].hex(),
            Some("#551A8B"),
            "folHlink regression: #551A8B"
        );
    }

    /// BC-2.01.001 postcondition 1 / AC-004 — `sysClr` slots are NOT derived;
    /// `is_derived` must be `false` for sysClr elements.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_derived_flag_not_set_on_sysclr() {
        let (slots, _) = parse_theme_colors(SYSCLR_THEME_XML.as_bytes())
            .expect("sysClr theme must parse without hard error");

        let dk1 = &slots[0];
        assert_eq!(dk1.name.as_ref(), "dk1");

        // sysClr is NOT a transform; is_derived must be false.
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            !dk1.is_derived,
            "sysClr slot dk1 must NOT set is_derived = true (sysClr is not a transform)"
        );

        // sysClr lastClr value must still be extracted correctly.
        assert_eq!(
            dk1.hex(),
            Some("#1A1A1A"),
            "sysClr lastClr=1A1A1A must produce #1A1A1A"
        );
    }

    /// BC-2.01.001 EC-006 Option B / AC-001 — srgbClr with lumMod does NOT have the
    /// resolved (HSL-transformed) color stored. The base hex and the derived flag are
    /// stored; any downstream consumer that needs the transformed color must compute it.
    ///
    /// This test specifically validates that the stored value is the verbatim base hex,
    /// not any of the plausible wrong answers for a lumMod=75000 transform.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_no_hsl_resolution_performed() {
        // Test three distinct base colors with lumMod to confirm no HSL math.
        // For each: the stored hex must equal the base val, not the transformed result.

        // Base: #003087 (navy blue), lumMod=75000 → naively expected darkened ~ #002465.
        let xml1 = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>"#,
        );
        let (slots1, _) = parse_theme_colors(xml1.as_bytes()).unwrap();
        let dk2_1 = &slots1[2];
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(dk2_1.is_derived, "lumMod navy: is_derived must be true");
        assert_eq!(
            dk2_1.hex(),
            Some("#003087"),
            "lumMod navy: base hex verbatim"
        );
        assert_ne!(
            dk2_1.hex(),
            Some("#002465"),
            "lumMod navy: MUST NOT store resolved color"
        );

        // Base: #FF0000 (red), tint=50000 → naive tint → lighter red ~ #FF8080.
        let xml2 = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="FF0000"><a:tint val="50000"/></a:srgbClr>"#,
        );
        let (slots2, _) = parse_theme_colors(xml2.as_bytes()).unwrap();
        let dk2_2 = &slots2[2];
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(dk2_2.is_derived, "tint red: is_derived must be true");
        assert_eq!(dk2_2.hex(), Some("#FF0000"), "tint red: base hex verbatim");
        assert_ne!(
            dk2_2.hex(),
            Some("#FF8080"),
            "tint red: MUST NOT store resolved color"
        );

        // Base: #000000 (black), shade=60000 → naive shade → dark gray ~ #000000 (no change for black)
        // Use a non-trivial base: #808080, shade=50000 → darkened ~ #404040.
        let xml3 = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="808080"><a:shade val="50000"/></a:srgbClr>"#,
        );
        let (slots3, _) = parse_theme_colors(xml3.as_bytes()).unwrap();
        let dk2_3 = &slots3[2];
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(dk2_3.is_derived, "shade gray: is_derived must be true");
        assert_eq!(
            dk2_3.hex(),
            Some("#808080"),
            "shade gray: base hex verbatim"
        );
        assert_ne!(
            dk2_3.hex(),
            Some("#404040"),
            "shade gray: MUST NOT store resolved color"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-003 / BC-2.01.001 EC-006 observability — tracing::warn! names slot + transforms
    //
    // These tests use a tracing subscriber to capture the emitted warning messages
    // and assert the required content. The warn! is emitted by the implementation;
    // the test verifies it fires with the right content.
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-2.01.001 EC-006 / AC-003 — the `tracing::warn!` emitted for a srgbClr with
    /// lumMod child must identify the slot name (`dk2`) and the transform type (`lumMod`)
    /// in its message.
    ///
    /// AC-003 example: `"slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with is_derived=true"`.
    ///
    /// # Red Gate
    ///
    /// Two failure paths:
    /// 1. `ColorSlot.is_derived` compile error (primary Red Gate).
    /// 2. Even if the field were added, the `tracing::warn!` is not yet emitted — the
    ///    test would fail at runtime because no warning fires.
    #[test]
    fn test_BC_2_01_001_EC006_srgbclr_warn_names_slot_and_transform() {
        use std::sync::{Arc as StdArc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt as _;

        // Visitor that extracts the "message" field from each tracing event.
        struct MessageCapturingLayer {
            messages: StdArc<Mutex<Vec<String>>>,
        }

        struct MessageVisitor {
            message: Option<String>,
        }

        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_owned());
                }
            }
        }

        impl<S: tracing::Subscriber> Layer<S> for MessageCapturingLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor { message: None };
                    event.record(&mut visitor);
                    if let Some(msg) = visitor.message {
                        let _ = self.messages.lock().map(|mut guard| guard.push(msg));
                    }
                }
            }
        }

        let captured_messages: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));
        let layer = MessageCapturingLayer {
            messages: StdArc::clone(&captured_messages),
        };
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Canonical test vector: dk2 = srgbClr val=003087 + lumMod val=75000.
        let xml = theme_xml_with_dk2_srgbclr(
            r#"<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>"#,
        );
        let (slots, _) = parse_theme_colors(xml.as_bytes()).expect("must parse without hard error");

        // Verify is_derived was set (compile-time Red Gate also fires here).
        // RED GATE: compile error — `is_derived` not yet on ColorSlot.
        assert!(
            slots[2].is_derived,
            "is_derived must be true before checking the warning message"
        );

        // AC-003: the warning message must contain the slot name and transform type.
        let messages = captured_messages.lock().unwrap();
        let found_warn = messages.iter().any(|msg| {
            // The warning must identify both "dk2" (slot name) and "lumMod" (transform type).
            msg.contains("dk2") && msg.contains("lumMod")
        });
        assert!(
            found_warn,
            "AC-003: tracing::warn! must be emitted identifying slot 'dk2' and transform \
             type 'lumMod' when an srgbClr has a lumMod child.\n\
             Captured WARN messages: {messages:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-002 / BC-2.01.003 EC-003 (widened) — extractor emits inline TOML comment
    // for is_derived slots regardless of origin (srgbClr or schemeClr)
    //
    // These tests operate on the extractor's TOML serialization path.
    // They use `build_pptx_zip` + `BrandExtractor::extract` to create an end-to-end
    // scenario and assert the written brand.toml content.
    // ─────────────────────────────────────────────────────────────────────────

    /// BC-2.01.003 EC-003 (widened) / AC-002 — when BrandExtractor serializes a
    /// `ColorSlot` with `is_derived = true` (from a srgbClr transform), the written
    /// brand.toml line for that slot contains the inline TOML comment
    /// `# derived via tint/shade; may not match exact color`.
    ///
    /// Canonical test vector from BC-2.01.003: dk2 = srgbClr val=003087 + lumMod →
    /// brand.toml must contain `dk2 = "#003087" # derived via tint/shade; ...`.
    ///
    /// # Red Gate
    ///
    /// Two failure paths:
    /// 1. `ColorSlot.is_derived` compile error.
    /// 2. Even if the field existed, `brand_template_to_toml` in `extractor.rs` does not
    ///    yet branch on `ColorSlot.is_derived` for `ColorValue::Hex` slots — it only
    ///    handles `ColorValue::SchemeRef`. The test would fail at runtime.
    #[test]
    fn test_BC_2_01_003_EC003_extractor_derived_srgbclr_emits_comment() {
        use std::io::Write as _;
        use std::sync::atomic::{AtomicU64, Ordering};

        use zip::CompressionMethod;
        use zip::write::{SimpleFileOptions, ZipWriter};

        use crate::extractor::BrandExtractor;
        use crate::loader::PPTX_THEME_PATH;

        // Theme XML with dk2 srgbClr + lumMod child.
        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="DerivedTheme">
  <a:themeElements>
    <a:clrScheme name="DerivedScheme">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr></a:dk2>
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
    <a:fontScheme name="DerivedFontScheme">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

        // Build a minimal in-memory PPTX ZIP containing this theme XML.
        let zip_bytes = {
            let mut buf = Vec::new();
            {
                let cursor = std::io::Cursor::new(&mut buf);
                let mut zw = ZipWriter::new(cursor);
                let opts =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
                zw.start_file(PPTX_THEME_PATH, opts).unwrap();
                zw.write_all(theme_xml.as_bytes()).unwrap();
                zw.start_file("[Content_Types].xml", opts).unwrap();
                zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
                zw.finish().unwrap();
            }
            buf
        };

        // Write PPTX to a temp file.
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let source_path = std::env::temp_dir().join(format!(
            "story076_test_derived_{}_{}_{}.pptx",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        ));
        {
            let mut f = std::fs::File::create(&source_path).unwrap();
            f.write_all(&zip_bytes).unwrap();
            f.sync_all().unwrap();
        }

        // Create a unique output directory.
        let out_dir = std::env::temp_dir().join(format!(
            "story076_test_derived_out_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        ));
        std::fs::create_dir_all(&out_dir).unwrap();

        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        let extraction = result.expect(
            "BC-2.01.003 EC-003 (widened): extraction from PPTX with derived srgbClr slot \
             must succeed",
        );

        let content = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("brand.toml must be readable after extraction");

        let _ = std::fs::remove_dir_all(&out_dir);

        // AC-002 (BC-2.01.003 EC-003 widened): the dk2 line in [colors] must contain
        // both the base hex AND the inline TOML comment.
        assert!(
            content.contains("003087"),
            "BC-2.01.003 EC-003: brand.toml must contain base hex '003087' for dk2, got:\n{content}"
        );

        // The exact inline comment mandated by BC-2.01.003 EC-003:
        assert!(
            content.contains("# derived via tint/shade; may not match exact color"),
            "BC-2.01.003 EC-003 (widened): brand.toml must contain inline comment \
             '# derived via tint/shade; may not match exact color' for derived srgbClr slot dk2.\n\
             This comment is currently only emitted for SchemeRef slots (not Hex+is_derived slots).\n\
             Actual content:\n{content}"
        );

        // The comment must appear on the same line as the dk2 field assignment.
        let dk2_line = content
            .lines()
            .find(|l| l.contains("dk2"))
            .unwrap_or_default();
        assert!(
            dk2_line.contains("# derived via tint/shade; may not match exact color"),
            "BC-2.01.003 EC-003: inline comment must be on the dk2 = \"...\" line, got: '{dk2_line}'"
        );

        // The written brand.toml must still be valid TOML (inline comment after value is legal).
        let _parsed: crate::toml_schema::BrandConfig = toml::from_str(&content).expect(
            "brand.toml with EC-003 inline comment must be parseable as valid TOML — \
                 inline comments after values are legal TOML syntax",
        );
    }

    /// BC-2.01.003 invariant / AC-004 — when BrandExtractor serializes a `ColorSlot`
    /// with `is_derived = false` (clean srgbClr, no transforms), the brand.toml line
    /// does NOT contain the inline TOML comment.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    /// Once the field exists, this test would fail at runtime IF the extractor
    /// incorrectly emits the comment on non-derived hex slots.
    #[test]
    fn test_BC_2_01_003_EC003_extractor_clean_srgbclr_no_comment() {
        use std::io::Write as _;
        use std::sync::atomic::{AtomicU64, Ordering};

        use zip::CompressionMethod;
        use zip::write::{SimpleFileOptions, ZipWriter};

        use crate::extractor::BrandExtractor;
        use crate::loader::PPTX_THEME_PATH;

        // Minimal theme with CLEAN srgbClr slots (no transforms).
        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="CleanTheme2">
  <a:themeElements>
    <a:clrScheme name="CleanScheme2">
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
    <a:fontScheme name="CleanFontScheme2">
      <a:majorFont><a:latin typeface="Calibri Light"/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/></a:minorFont>
    </a:fontScheme>
  </a:themeElements>
</a:theme>"#;

        let zip_bytes = {
            let mut buf = Vec::new();
            {
                let cursor = std::io::Cursor::new(&mut buf);
                let mut zw = ZipWriter::new(cursor);
                let opts =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
                zw.start_file(PPTX_THEME_PATH, opts).unwrap();
                zw.write_all(theme_xml.as_bytes()).unwrap();
                zw.start_file("[Content_Types].xml", opts).unwrap();
                zw.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>").unwrap();
                zw.finish().unwrap();
            }
            buf
        };

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let source_path = std::env::temp_dir().join(format!(
            "story076_test_clean_{}_{}_{}.pptx",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        ));
        {
            let mut f = std::fs::File::create(&source_path).unwrap();
            f.write_all(&zip_bytes).unwrap();
            f.sync_all().unwrap();
        }

        let out_dir = std::env::temp_dir().join(format!(
            "story076_test_clean_out_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            seq
        ));
        std::fs::create_dir_all(&out_dir).unwrap();

        let result = BrandExtractor::extract(source_path.to_str().unwrap(), &out_dir, false);

        let _ = std::fs::remove_file(&source_path);

        let extraction = result.expect("clean PPTX extraction must succeed");
        let content = std::fs::read_to_string(&extraction.brand_toml_path)
            .expect("brand.toml must be readable");

        let _ = std::fs::remove_dir_all(&out_dir);

        // AC-004 regression: clean srgbClr slots must NOT have the inline comment.
        assert!(
            !content.contains("# derived via tint/shade; may not match exact color"),
            "AC-004 regression: brand.toml for clean srgbClr slots must NOT contain \
             '# derived via tint/shade; may not match exact color', got:\n{content}"
        );

        // All color values must be plain `key = "#RRGGBB"` lines (no trailing comments).
        let color_section_start = content
            .find("[colors]")
            .expect("brand.toml must have [colors] section");
        let fonts_section_start = content
            .find("[fonts]")
            .expect("brand.toml must have [fonts] section");
        let colors_section = &content[color_section_start..fonts_section_start];
        for line in colors_section.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('[') {
                continue;
            }
            // A non-comment, non-empty line in [colors] must not have a trailing comment.
            assert!(
                !trimmed.contains("# derived"),
                "AC-004 regression: clean srgbClr color line must not have '# derived' \
                 comment, got: '{trimmed}'"
            );
        }
    }

    /// BC-2.01.003 EC-003 (widened) / AC-002 — the extractor's `is_derived` branch
    /// is UNIFIED: it fires on ColorSlot.is_derived = true regardless of whether the
    /// underlying ColorValue is SchemeRef (schemeClr origin) or Hex (srgbClr origin).
    ///
    /// This test directly exercises `brand_template_to_toml` with a synthetic
    /// `BrandTemplate` where a Hex-valued slot has `is_derived = true`, verifying
    /// the extractor conditions on the flag rather than on the ColorValue variant.
    ///
    /// # Red Gate
    ///
    /// `ColorSlot.is_derived` does not exist — compile error until implemented.
    /// Once the field exists, `brand_template_to_toml` must be updated to emit the
    /// inline comment for `ColorValue::Hex` slots where `is_derived = true` (currently
    /// it only emits the comment in the `ColorValue::SchemeRef` branch).
    #[test]
    fn test_BC_2_01_003_EC003_extractor_unified_flag_drives_comment() {
        use crate::footer::FooterFlags;
        use crate::template::{BrandFonts, BrandTemplate, MasterIds};

        // Directly construct a BrandTemplate with slot dk2 having:
        //   - ColorValue::Hex("#003087") — a resolved hex value
        //   - is_derived = true          — the new flag added by STORY-076
        //
        // This models the srgbClr + lumMod case where the loader stores the base
        // hex verbatim AND sets is_derived = true.
        //
        // RED GATE: compile error — `ColorSlot` has no `is_derived` field.
        let make_slot = |name: &str, hex: &str, derived: bool| -> ColorSlot {
            ColorSlot {
                name: Arc::from(name),
                value: ColorValue::Hex(Arc::from(hex)),
                is_derived: derived,
            }
        };

        let mut colors: [ColorSlot; 12] = [
            make_slot("dk1", "#000000", false),
            make_slot("lt1", "#FFFFFF", false),
            // dk2 is_derived = true — simulates srgbClr + lumMod extraction.
            make_slot("dk2", "#003087", true),
            make_slot("lt2", "#F5F5F5", false),
            make_slot("acc1", "#0066CC", false),
            make_slot("acc2", "#FF6B35", false),
            make_slot("acc3", "#28A745", false),
            make_slot("acc4", "#FFC107", false),
            make_slot("acc5", "#6F42C1", false),
            make_slot("acc6", "#17A2B8", false),
            make_slot("hlink", "#0000EE", false),
            make_slot("folHlink", "#551A8B", false),
        ];

        let template = BrandTemplate {
            colors,
            fonts: BrandFonts {
                heading: Arc::from("Calibri Light"),
                body: Arc::from("Calibri"),
            },
            logo: None,
            footer_text: None,
            footer_flags: FooterFlags::default(),
            layout_names: vec![],
            layouts: vec![],
            notes_master_stub: vec![],
            handout_master_stub: vec![],
            master_ids: MasterIds::default(),
            content_types_layout_entries: Arc::from(""),
        };

        // Invoke the TOML serializer directly (testing the production code path).
        // `brand_template_to_toml` is pub(crate); expose it via the test module
        // by using the `super::super::` path from the nested test module context.
        // Since this file is in the crate root-level test module, we call through
        // the public extraction path for the TOML string assertion.
        //
        // We write the brand.toml via BrandExtractor using a real PPTX fixture
        // that has been engineered to produce a derived srgbClr slot. Instead of
        // testing the private `brand_template_to_toml` fn directly, we round-trip
        // through `BrandExtractor::extract` using the PPTX built above.
        //
        // However: to test the UNIFIED flag path specifically (i.e., a Hex slot with
        // is_derived = true, vs the current SchemeRef path), we need to reach
        // `brand_template_to_toml` with the synthetic template above. We do this by
        // calling into the internal module via an assembly of the
        // `crate::extractor::brand_template_to_toml` function (pub(crate)).
        //
        // This confirms that AC-002 is satisfied by the Hex+is_derived path, not only
        // by the SchemeRef path.
        //
        // Expected behavior: dk2 line = `dk2 = "#003087" # derived via tint/shade; may not match exact color`

        // Call the pub(crate) function directly from within the crate.
        let (toml_str, _logo_path) = crate::extractor::brand_template_to_toml(&template, None);

        // AC-002: dk2 must have the inline comment because is_derived = true.
        let dk2_line = toml_str
            .lines()
            .find(|l| l.starts_with("dk2 "))
            .unwrap_or_default();

        assert!(
            dk2_line.contains("# derived via tint/shade; may not match exact color"),
            "BC-2.01.003 EC-003 unified path: dk2 (Hex + is_derived=true) must emit \
             inline comment '# derived via tint/shade; may not match exact color'.\n\
             Current implementation only branches on ColorValue::SchemeRef, not on \
             is_derived flag.\n\
             dk2 line: '{dk2_line}'\n\
             Full TOML:\n{toml_str}"
        );

        // Non-derived slots must NOT have the comment.
        let dk1_line = toml_str
            .lines()
            .find(|l| l.starts_with("dk1 "))
            .unwrap_or_default();
        assert!(
            !dk1_line.contains("# derived"),
            "non-derived slot dk1 must NOT have the inline comment, got: '{dk1_line}'"
        );

        // The output must still be valid TOML.
        let _parsed: crate::toml_schema::BrandConfig = toml::from_str(&toml_str)
            .expect("TOML with EC-003 inline comment on Hex slot must be valid TOML");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SEC-002 (CWE-789/400) — unbounded Vec allocation guard
    //
    // A crafted theme1.xml with many transform children must not cause unbounded
    // Vec growth. The accumulator must be capped at MAX_TRANSFORMS_PER_SLOT.
    // ─────────────────────────────────────────────────────────────────────────

    /// SEC-002 / CWE-789 — srgbClr with 100 transform children is capped at
    /// `MAX_TRANSFORMS_PER_SLOT` (8) entries; no excessive allocation occurs.
    ///
    /// The slot must still be parsed correctly: `is_derived = true`, base hex
    /// stored verbatim, and no `MissingColorSlot` warning emitted.
    ///
    /// # RED Gate
    ///
    /// Before the fix, BOTH push sites in `parse_theme_colors` push unconditionally.
    /// This test asserts the capped behavior that requires the guard to be present.
    /// It will FAIL (or exhibit unbounded allocation) until `MAX_TRANSFORMS_PER_SLOT`
    /// and both `transforms.len() < MAX_TRANSFORMS_PER_SLOT` guards are added.
    #[test]
    fn test_sec_002_srgbclr_many_transforms_capped_at_max() {
        use std::sync::{Arc as StdArc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt as _;

        // Capture warn messages to verify the warning is still emitted (capped, not silent).
        struct MessageCapturingLayer {
            messages: StdArc<Mutex<Vec<String>>>,
        }
        struct MessageVisitor {
            message: Option<String>,
        }
        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_owned());
                }
            }
        }
        impl<S: tracing::Subscriber> Layer<S> for MessageCapturingLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor { message: None };
                    event.record(&mut visitor);
                    if let Some(msg) = visitor.message {
                        let _ = self.messages.lock().map(|mut guard| guard.push(msg));
                    }
                }
            }
        }
        let captured: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));
        let layer = MessageCapturingLayer {
            messages: StdArc::clone(&captured),
        };
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Build a dk2 srgbClr with 100 lumMod children (crafted adversarial input).
        // OOXML defines only 4 transform types; 100 is far beyond any legitimate usage.
        use std::fmt::Write as _;
        let mut inner = String::from(r#"<a:srgbClr val="003087">"#);
        for i in 0..100u32 {
            let _ = write!(inner, r#"<a:lumMod val="{i}"/>"#);
        }
        inner.push_str("</a:srgbClr>");

        let xml = theme_xml_with_dk2_srgbclr(&inner);
        let (slots, warnings) = parse_theme_colors(xml.as_bytes())
            .expect("adversarial many-transforms XML must parse without hard error");

        // No MissingColorSlot warnings — the slot is still present.
        assert_eq!(
            warnings.len(),
            0,
            "many-transform srgbClr must not produce MissingColorSlot warnings"
        );

        let dk2 = &slots[2];
        assert_eq!(dk2.name.as_ref(), "dk2");

        // is_derived must be true — transforms were detected (even if capped).
        assert!(
            dk2.is_derived,
            "SEC-002: srgbClr with many transforms must still set is_derived = true"
        );

        // Base hex stored verbatim (Option B).
        assert_eq!(
            dk2.hex(),
            Some("#003087"),
            "SEC-002: base hex must be stored verbatim despite many transforms"
        );

        // The warn message must exist (one warn for the slot) and must NOT contain
        // 100 semicolon-delimited entries — the cap limits the list.
        let messages = captured.lock().unwrap();
        let dk2_warns: Vec<&String> = messages.iter().filter(|m| m.contains("dk2")).collect();
        assert_eq!(
            dk2_warns.len(),
            1,
            "SEC-002: exactly one warn must be emitted for dk2 with many transforms"
        );

        // The warn message must NOT contain more than MAX_TRANSFORMS_PER_SLOT (8)
        // semicolon-separated entries. Count occurrences of " child" as a proxy for
        // the number of transform descriptions joined in the warn.
        let warn_msg = dk2_warns[0];
        let child_count = warn_msg.matches(" child").count();
        assert!(
            child_count <= 8,
            "SEC-002: warn message must list at most MAX_TRANSFORMS_PER_SLOT (8) \
             transform entries, but found {child_count} 'child' occurrences.\n\
             Full warn message: {warn_msg:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SEC-001 (CWE-117) — log injection sanitization guard
    //
    // A transform `val` attribute containing control characters / newlines must
    // NOT flow verbatim into the tracing::warn! message. The sanitized description
    // must omit control chars and be length-capped.
    // ─────────────────────────────────────────────────────────────────────────

    /// SEC-001 / CWE-117 — a transform `val` containing control characters
    /// (e.g. `"\n\u{1b}[31minjected"`) is sanitized before inclusion in the
    /// `tracing::warn!` log message.
    ///
    /// The captured warning message must NOT contain a raw newline (`\n`) or the
    /// ANSI escape sequence (`\x1b[31m`). The sanitized message may still name the
    /// transform type (`lumMod`) so the legitimate diagnostic value is preserved.
    ///
    /// # RED Gate
    ///
    /// Before the fix, `transform_description` embeds `v` verbatim (line ~83 of
    /// color.rs). This test will FAIL until the sanitization (`filter(is_control) +
    /// take(32)`) is applied.
    #[test]
    fn test_sec_001_transform_val_control_chars_sanitized_in_warn() {
        use std::sync::{Arc as StdArc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt as _;

        struct MessageCapturingLayer {
            messages: StdArc<Mutex<Vec<String>>>,
        }
        struct MessageVisitor {
            message: Option<String>,
        }
        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "message" {
                    self.message = Some(value.to_owned());
                }
            }
        }
        impl<S: tracing::Subscriber> Layer<S> for MessageCapturingLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor { message: None };
                    event.record(&mut visitor);
                    if let Some(msg) = visitor.message {
                        let _ = self.messages.lock().map(|mut guard| guard.push(msg));
                    }
                }
            }
        }

        let captured: StdArc<Mutex<Vec<String>>> = StdArc::new(Mutex::new(Vec::new()));
        let layer = MessageCapturingLayer {
            messages: StdArc::clone(&captured),
        };
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        // Adversarial val: newline + ANSI escape to simulate log injection.
        // The XML attribute value contains raw control chars that quick_xml will
        // pass through as-is. The val after attribute parsing will be
        // "\n\x1b[31minjected" — a newline followed by a color-code escape.
        let injected_val = "\n\x1b[31minjected";

        // Build the XML with the injected val embedded in a lumMod child.
        // We use the standard dk2 wrapper fixture.
        let inner =
            format!(r#"<a:srgbClr val="003087"><a:lumMod val="{injected_val}"/></a:srgbClr>"#);
        let xml = theme_xml_with_dk2_srgbclr(&inner);

        let (slots, _warnings) = parse_theme_colors(xml.as_bytes())
            .expect("injection-attempt XML must parse without hard error");

        // is_derived must still be set — the transform was detected.
        assert!(
            slots[2].is_derived,
            "SEC-001: lumMod with injected val must still set is_derived = true"
        );

        // Inspect captured warn messages for the raw control characters.
        let messages = captured.lock().unwrap();
        let dk2_warns: Vec<&String> = messages.iter().filter(|m| m.contains("dk2")).collect();

        // There must be at least one warn for dk2.
        assert!(
            !dk2_warns.is_empty(),
            "SEC-001: a warn! must be emitted for dk2 even with injected val"
        );

        let warn_msg = dk2_warns[0];

        // The raw newline must NOT appear in the logged message.
        assert!(
            !warn_msg.contains('\n'),
            "SEC-001: warn! message must NOT contain raw newline (log injection guard).\n\
             Actual message: {warn_msg:?}"
        );

        // The raw ANSI escape must NOT appear in the logged message.
        assert!(
            !warn_msg.contains('\x1b'),
            "SEC-001: warn! message must NOT contain raw ESC character (log injection guard).\n\
             Actual message: {warn_msg:?}"
        );

        // The injected payload string must NOT appear verbatim.
        assert!(
            !warn_msg.contains("[31minjected"),
            "SEC-001: warn! message must NOT contain ANSI color code payload '[31minjected'.\n\
             Actual message: {warn_msg:?}"
        );
    }
}
