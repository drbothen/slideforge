//! Brand adapter: construct a [`slideforge_brand::BrandTemplate`] from a
//! [`slideforge_types::Brand`] for use by the PPTX exporter.
//!
//! The `Exporter` trait hands the PPTX exporter a `&Brand` (the slim semantic-IR
//! brand type from `slideforge-types`). The XML serializers in `slideforge-brand`
//! operate on the richer `BrandTemplate` type. This module bridges the two.
//!
//! ## Approach (ADR-015 §scope verdict)
//!
//! `BrandTemplate` is synthesized on-the-fly from `&Brand` by:
//! 1. Building a 12-entry owned `BrandConfig`-style declaration from the brand palette.
//! 2. Inferring all 12 OOXML color slots via
//!    `slideforge_brand::inference::infer_missing_slots`.
//! 3. Generating 31 slide layout definitions via
//!    `slideforge_brand::layouts::generate_all_layouts`.
//! 4. Populating font names from `brand.fonts`.
//!
//! No I/O is performed. The synthesized `BrandTemplate` is suitable for all
//! PPTX XML serialization calls: layout XML, master XML, and theme XML.
//!
//! ## Performance
//!
//! The synthesis is O(1) — 31 layout geometries + 12 color inferences — and
//! completes in well under 1ms for any practical brand. It is called once per
//! `export` invocation.

use std::sync::Arc;

use slideforge_brand::error::BrandError;
use slideforge_brand::inference;
use slideforge_brand::layout_xml::{
    HANDOUT_MASTER_STUB, NOTES_MASTER_STUB, generate_content_types_layout_entries,
};
use slideforge_brand::layouts::generate_all_layouts;
use slideforge_brand::template::{
    BrandFonts, BrandTemplate, COLOR_SLOT_NAMES, ColorSlot, ColorValue, MasterIds,
};
use slideforge_brand::toml_schema::BrandConfig;
use slideforge_types::Brand;

/// Construct a `BrandTemplate` from a `&Brand` for use by the PPTX exporter.
///
/// The returned template has:
/// - 12 OOXML color slots inferred from the brand palette.
/// - 31 slide layout definitions (same as synthesized brands).
/// - Notes/handout master stub bytes.
/// - Correct `MasterIds` defaults.
///
/// Cosmetic inference warnings are discarded via `tracing::debug!` — they are
/// non-fatal and do not affect export correctness.
///
/// # Design note
///
/// This function is called once per `export` invocation. Any inference warnings
/// that would normally surface in the CLI are discarded here because the exporter
/// has no user-facing warning channel. The brand palette validity is guaranteed
/// upstream — by the time `export` is called, the brand has been validated.
#[must_use]
pub fn brand_template_from_brand(brand: &Brand) -> BrandTemplate {
    // Step 1: extract the 4 palette hex values as owned Strings.
    // We'll map them to specific OOXML slots:
    //   dk2 = primary, lt2 = neutral, acc1 = primary, acc2 = secondary, acc3 = accent
    // Remaining slots (dk1, lt1, acc4-acc6, hlink, folHlink) are inferred.
    let primary = strip_hash(&brand.palette.primary);
    let secondary = strip_hash(&brand.palette.secondary);
    let accent = strip_hash(&brand.palette.accent);
    let neutral = strip_hash(&brand.palette.neutral);

    // Build owned strings so we can take `&str` references that outlive the function call.
    // `infer_missing_slots` accepts `[Option<&str>; 12]` — we need `&str` with sufficient lifetime.
    let palette_slots: [Option<String>; 12] = [
        None,                    // dk1: inferred
        None,                    // lt1: always #FFFFFF
        Some(primary.clone()),   // dk2: primary
        Some(neutral.clone()),   // lt2: neutral
        Some(primary.clone()),   // acc1: primary
        Some(secondary.clone()), // acc2: secondary
        Some(accent.clone()),    // acc3: accent
        None,                    // acc4: inferred
        None,                    // acc5: inferred
        None,                    // acc6: inferred
        None,                    // hlink: inferred
        None,                    // folHlink: inferred
    ];

    // Build the `[Option<&str>; 12]` array by borrowing from the owned Strings.
    let slot_refs: [Option<&str>; 12] = std::array::from_fn(|i| palette_slots[i].as_deref());

    let mut warnings: Vec<BrandError> = Vec::new();
    let hex_slots = inference::infer_missing_slots(slot_refs, &mut warnings);

    if !warnings.is_empty() {
        tracing::debug!(
            warning_count = warnings.len(),
            "brand_template_from_brand: color inference warnings (non-fatal)"
        );
    }

    // Step 2: build ColorSlot array from inferred hex strings.
    let colors = std::array::from_fn(|i| ColorSlot {
        name: Arc::from(COLOR_SLOT_NAMES[i]),
        value: ColorValue::Hex(hex_slots[i].clone()),
        is_derived: false,
    });

    // Step 3: generate 31 layout definitions (same as BrandSynthesizer).
    // BrandConfig is not used by generate_all_layouts (_config is ignored),
    // so we pass a default instance.
    let config = BrandConfig::default_minimal();
    let layouts = generate_all_layouts(&config);

    // Step 4: assemble BrandTemplate.
    BrandTemplate {
        colors,
        fonts: BrandFonts {
            heading: brand.fonts.heading.clone(),
            body: brand.fonts.body.clone(),
        },
        logo: None, // logo not required for XML serialization in the PPTX exporter
        footer_text: None,
        footer_flags: slideforge_brand::FooterFlags::default(),
        layout_names: vec![],
        layouts,
        notes_master_stub: NOTES_MASTER_STUB.to_vec(),
        handout_master_stub: HANDOUT_MASTER_STUB.to_vec(),
        master_ids: MasterIds::default(),
        content_types_layout_entries: Arc::from(generate_content_types_layout_entries(31).as_str()),
    }
}

/// Strip the `#` prefix from a hex color string, returning an owned `String`.
///
/// `"#003087"` → `"003087"`. If no `#` is present, returns the full string as-is.
fn strip_hash(s: &Arc<str>) -> String {
    s.strip_prefix('#').unwrap_or(s.as_ref()).to_owned()
}
