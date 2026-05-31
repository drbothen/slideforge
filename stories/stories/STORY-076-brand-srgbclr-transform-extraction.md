---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-076
title: "Brand Loader: Transform-Aware Theme Color Extraction (srgbClr lumMod/tint/shade)"
epic: EPIC-06
wave: 4
points: 3
priority: P1
tdd_mode: strict
status: ready
crate: slideforge-brand
subsystems: [SS-04]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.001, BC-2.01.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-024, NFR-025]
depends_on:
  - STORY-022
blocks: []
estimated_days: 1
---

# STORY-076: Brand Loader: Transform-Aware Theme Color Extraction (srgbClr lumMod/tint/shade)

## Summary

`parse_theme_colors` in `crates/slideforge-brand/src/color.rs` currently extracts the
`val` attribute of `<a:srgbClr>` elements and silently drops any child transform
elements (`<a:lumMod>`, `<a:lumOff>`, `<a:tint>`, `<a:shade>`). A real OOXML theme
slot such as:

```xml
<a:dk2>
  <a:srgbClr val="003087">
    <a:lumMod val="75000"/>
  </a:srgbClr>
</a:dk2>
```

is extracted as `dk2 = "#003087"` with no indication that the rendered color differs
from the raw value (the effective luminance-modulated color would be approximately
`#002465`). This silent drop means brand.toml represents a color that diverges from
what a viewer actually sees in the .pptx.

BC-2.01.001 now carries a new edge case EC-006 covering `srgbClr` elements with
transform children (loading side): store the base `val` hex verbatim, set
`ColorSlot.is_derived = true`, emit `tracing::warn!` naming the slot and transform
type(s). No HSL resolution (deferred v2). No new error code is introduced.

BC-2.01.003 EC-003 is widened on the extraction side from "schemeClr with
lumMod/lumOff/tint/shade" to "schemeClr OR srgbClr with lumMod/lumOff/tint/shade";
the extractor conditions on `ColorSlot.is_derived` regardless of element origin.

Note: these are TWO distinct BCs covering the same physical transform: BC-2.01.001
EC-006 governs the loading behavior (what `parse_theme_colors` stores and warns);
BC-2.01.003 EC-003 governs the extraction behavior (what `BrandExtractor` writes to
brand.toml). Do not conflate them — AC-001 and AC-003 trace to BC-2.01.001 EC-006;
AC-002 traces to BC-2.01.003 EC-003.

**Resolution:** The PO has now landed BC-2.01.001 EC-006 (loading side) and widened
BC-2.01.003 EC-003 (extraction side) in v1.9. Option B is the v1.0 choice: store
base hex + set is_derived flag + emit tracing::warn!; defer HSL resolution to v2.
No new error code is introduced (error-taxonomy v2.3 documents this decision).
This story is now `ready` for Wave 4 dispatch.

## Behavioral Contracts

| BC | Title | Version | Covered ACs |
|----|-------|---------|-------------|
| BC-2.01.001 | Brand Loader: .pptx/.docx Template Extraction | v1.2 | AC-001 (EC-006 loading side), AC-003 (EC-006 warn!), AC-004 (EC-006 clean srgbClr regression) |
| BC-2.01.003 | Brand Extraction CLI | v1.9 | AC-002 (EC-003 widened — extractor emits comment for is_derived regardless of origin) |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2,000 |
| `crates/slideforge-brand/src/color.rs` | ~1,500 |
| `crates/slideforge-brand/src/extractor.rs` (for EC-003 comment behavior) | ~1,000 |
| BC-2.01.001 (loader BC, EC-006 — loading side) | ~800 |
| BC-2.01.003 EC-003 widened (extraction side) | ~400 |
| Test code | ~1,500 |
| **Total** | **~7,100** |

Agent context budget: 200k tokens. This story is ~3.6% of budget — well within limit.

## Acceptance Criteria

- [ ] **AC-001:** When `parse_theme_colors` encounters an `<a:srgbClr>` element with
  one or more child transform elements (`<a:lumMod>`, `<a:lumOff>`, `<a:tint>`,
  `<a:shade>`), it stores the base `val` hex verbatim as `ColorSlot.hex` and sets
  `ColorSlot.is_derived = true`. No HSL resolution or effective color computation is
  performed (deferred to v2). A `tracing::warn!` is emitted identifying the specific
  slot name and each transform type found. Example:
  `"slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with is_derived=true"`.
  `srgbClr` elements WITHOUT transform children set `is_derived = false` with no warning.
  (traces to BC-2.01.001 EC-006 — srgbClr with transform children: store base hex,
  set is_derived, emit tracing::warn!; Option B chosen as v1.0 decision; HSL resolution deferred v2)

- [ ] **AC-002:** When `BrandExtractor` serializes a `ColorSlot` that has the
  `is_derived` flag set (from either schemeClr or srgbClr transform detection), it
  emits the same EC-003-style inline TOML comment:
  `# derived via tint/shade; may not match exact color`
  This unifies the comment behavior for both color element types. BC-2.01.003 EC-003
  is widened from "schemeClr with lumMod/lumOff/tint/shade" to "schemeClr OR srgbClr
  with lumMod/lumOff/tint/shade"; the extractor conditions on `ColorSlot.is_derived`
  regardless of the originating element type (schemeClr or srgbClr).
  (traces to BC-2.01.003 EC-003 widened — extractor emits inline comment whenever
  ColorSlot.is_derived is true, regardless of whether origin was schemeClr or srgbClr)

- [ ] **AC-003:** When `parse_theme_colors` encounters a srgbClr with transforms and
  emits a lint warning via `tracing::warn!`, the warning message identifies the
  specific slot name and each transform type found. Example:
  `"slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with is_derived=true"`
  (traces to BC-2.01.001 EC-006 — observability requirement: warn! names slot + transform type(s))

- [ ] **AC-004:** `srgbClr` elements WITHOUT transform children continue to be
  extracted exactly as before — `val` attribute as hex, no derived flag, no comment.
  Regression test verifies existing STORY-022 test fixtures still pass.
  (traces to BC-2.01.001 postcondition 1 — no regression to clean srgbClr extraction)

- [ ] **AC-005:** `#![forbid(unsafe_code)]` (NFR-024), clippy clean (NFR-022),
  `=` version pinning (NFR-025) maintained.

## Scope Boundary

This story is EXCLUSIVELY a change to `crates/slideforge-brand/src/color.rs`
(`parse_theme_colors`) and the `ColorSlot` struct (adding an `is_derived: bool` field
if that is the chosen approach). The extractor change in AC-002 is a one-line
condition update using the existing `is_derived` flag.

**Out of scope for this story:**
- Computing the mathematically correct resolved color from lumMod/tint/shade formulas
  (requires a separate story with Kani proof obligations — deferred to v2 per
  Option B decision; no new story ID assigned yet).
- Any further BC changes — the PO has already landed BC-2.01.001 EC-006 (loading)
  and the BC-2.01.003 EC-003 widening (extraction). This story implements against
  those updated contracts.

## Previous Story Intelligence

STORY-022 (brand loading) introduced `parse_theme_colors` in `color.rs`. The function
handles `srgbClr`, `sysClr`, and `schemeClr` element types. The schemeClr transform
case (EC-003) is handled via the `is_derived` path that emits the inline comment during
extraction. The srgbClr case currently only reads `.val` and discards child nodes.

## Architecture Compliance Rules

1. **Pure function:** `parse_theme_colors` is a pure function (takes XML bytes, returns
   `Vec<ColorSlot>`). It must remain pure — no I/O, no global state, Kani-amenable.
2. **`ColorSlot` struct change:** If adding `is_derived: bool`, update the struct in
   `crates/slideforge-brand/src/color.rs` and all construction sites (sysClr path,
   srgbClr path, schemeClr path). The schemeClr path already sets this field; the
   srgbClr path currently does not.
3. **Forbidden dependencies:** Same as STORY-022 (no xml-rs, only quick-xml; no
   dependencies on slideforge-layout or slideforge-pptx from slideforge-brand).

## Library and Framework Requirements

No new libraries. Uses existing `quick-xml = "=0.36"` for XML parsing and
the `ColorSlot` / `Hex` types already defined in `color.rs`.

## File Structure Requirements

Files to modify:

```
crates/slideforge-brand/src/
├── color.rs          # parse_theme_colors: detect srgbClr transform children; set is_derived flag
└── extractor.rs      # serialize is_derived flag for srgbClr slots (unified with schemeClr path)
```

No new files. No changes outside `slideforge-brand`.

## Tasks

1. **Read `color.rs` `parse_theme_colors`** — locate the `srgbClr` branch and identify
   where child element iteration is absent. (5 min)
2. **Add child-element scan to srgbClr branch** — iterate children, detect lumMod/
   lumOff/tint/shade presence, set `is_derived = true` and emit `tracing::warn!`. (20 min)
3. **Update `ColorSlot::new_srgbclr` constructor** (or equivalent) to thread the
   derived flag through. (10 min)
4. **Verify extractor.rs** already conditions on `is_derived` for the TOML comment — if
   it uses a shared path, no change needed; if it has a separate srgbClr path, add the
   condition. (10 min)
5. **Write unit tests** — see Test Strategy. (20 min)
6. **Run `cargo clippy -p slideforge-brand -- -D warnings`** — fix. (5 min)
7. **Run `cargo test -p slideforge-brand`** — all tests pass including STORY-022
   regression tests. (5 min)

## Test Strategy

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_srgbclr_with_lummod_sets_derived_flag` | XML snippet: `<a:srgbClr val="003087"><a:lumMod val="75000"/></a:srgbClr>` | `ColorSlot.is_derived == true`; `ColorSlot.hex == "#003087"` |
| `test_srgbclr_without_transforms_not_derived` | XML snippet: `<a:srgbClr val="FF0000"/>` | `ColorSlot.is_derived == false`; `ColorSlot.hex == "#FF0000"` |
| `test_srgbclr_tint_sets_derived_flag` | XML snippet with `<a:tint val="50000"/>` child | `is_derived == true` |
| `test_srgbclr_shade_sets_derived_flag` | XML snippet with `<a:shade val="60000"/>` child | `is_derived == true` |
| `test_extractor_derived_srgbclr_emits_comment` | Parse theme with derived srgbClr slot; run extractor | `brand.toml` contains inline comment `# derived via tint/shade; may not match exact color` on that slot |
| `test_srgbclr_regression_clean_val` | Re-run existing STORY-022 fixture with clean srgbClr slots | All slots extracted correctly; no derived flags; no comments in brand.toml |

## Dependencies

**Depends on:**
- STORY-022 (Brand Loading) — `parse_theme_colors`, `ColorSlot`, `BrandLoader` are
  all defined there. This story modifies `parse_theme_colors` behavior.

Dependency justification: STORY-076 is a targeted enhancement to STORY-022's
`parse_theme_colors` function. It cannot be implemented without STORY-022's
`ColorSlot` struct definition and the existing srgbClr parsing branch.

**Blocks:** None currently. If the product-owner decides to add a formal Kani proof
for the transform detection (VP-NNN), that proof story would depend on this one.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | srgbClr has lumMod AND tint (multiple transforms) | derived flag set once; single warning emitted listing all found transform types |
| EC-002 | srgbClr has lumOff child (not lumMod) | derived flag set; same warning pattern |
| EC-003 | srgbClr val attribute is missing (malformed XML) | Existing E-BRD-002 / parse error path — no change from STORY-022 behavior |
| EC-004 | schemeClr with transforms — existing behavior | Unchanged; is_derived already set by STORY-022 schemeClr branch |

## Implementation Notes

The minimal correct fix is: in the srgbClr branch of `parse_theme_colors`, after
reading the `val` attribute, peek at child elements. If any child element has a
local name in `{"lumMod", "lumOff", "tint", "shade"}`, set `is_derived = true` and
emit the `tracing::warn!`. This requires iterating child elements with `quick-xml`
event reader before returning — an O(N) scan where N is the number of child elements
(typically 0–2).

The v1.0 decision (Option B, confirmed by PO, BC-2.01.001 EC-006) is: store base
hex verbatim + set `is_derived = true` + emit `tracing::warn!`. No HSL resolution.
If exact color computation is required in a future version, it would need a separate
story with a Kani proof obligation for the HSL arithmetic.
