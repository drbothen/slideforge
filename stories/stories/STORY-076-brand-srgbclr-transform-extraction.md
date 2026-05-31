---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-076
title: "Brand Loader: Transform-Aware Theme Color Extraction (srgbClr lumMod/tint/shade)"
epic: EPIC-06
wave: TBD
points: 3
priority: P1
tdd_mode: strict
status: draft
crate: slideforge-brand
subsystems: [SS-04]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.001]
# BC status: pending PO authorship — BC-2.01.001 EC-003 widening and possible new EC-006
# are required before this story can move to ready. See Scope section for details.
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

The existing BC-2.01.003 EC-003 covers this case for `schemeClr` elements only.
`srgbClr` elements with transform children are outside that scope and are silently
dropped today with no comment emitted. This story widens the transform-awareness to
cover `srgbClr` as well.

**Deferral justification:** This is a spec-gap follow-up, not a v1.0 blocker. The
current behavior is consistent with the written spec (no BC clause covers srgbClr
transforms). Deferral is legitimate under the canonical principle because: (1) it
requires widening BC-2.01.001 (loader BC) and BC-2.01.003 EC-003 — a product-owner
decision, not an AI-defaulted shortcut; (2) the workaround (inline comment on schemeClr
transforms) is already in place for the analogous schemeClr case; (3) the divergence
is observable only for corporate templates that use luminance-modulated srgbClr slots,
which are uncommon in practice.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2,000 |
| `crates/slideforge-brand/src/color.rs` | ~1,500 |
| `crates/slideforge-brand/src/extractor.rs` (for EC-003 comment behavior) | ~1,000 |
| BC-2.01.001 (loader BC, widening target) | ~800 |
| BC-2.01.003 EC-003 (widening reference) | ~300 |
| Test code | ~1,500 |
| **Total** | **~7,100** |

Agent context budget: 200k tokens. This story is ~3.6% of budget — well within limit.

## Acceptance Criteria

- [ ] **AC-001:** When `parse_theme_colors` encounters an `<a:srgbClr>` element with
  one or more child transform elements (`<a:lumMod>`, `<a:lumOff>`, `<a:tint>`,
  `<a:shade>`), it detects the transform children and either: (a) resolves the
  effective transformed color and stores the result, OR (b) stores the base `val`
  hex value with an `is_derived` flag set on the `ColorSlot` (analogous to schemeClr
  treatment). The choice between (a) and (b) is the product-owner decision that
  must be documented in the BC widening before this story moves to `ready`.
  (traces to BC-2.01.001 invariant 1 — all 12 slots represented with accurate values;
  awaiting BC-2.01.001 EC widening for srgbClr transforms)

- [ ] **AC-002:** When `BrandExtractor` serializes a `ColorSlot` that has the
  `is_derived` flag set (from either schemeClr or srgbClr transform detection), it
  emits the same EC-003-style inline TOML comment:
  `# derived via tint/shade; may not match exact color`
  This unifies the comment behavior for both color element types.
  (traces to BC-2.01.003 EC-003 — widened to cover srgbClr transforms in addition
  to schemeClr transforms)

- [ ] **AC-003:** When `parse_theme_colors` encounters a srgbClr with transforms and
  emits a lint warning via `tracing::warn!`, the warning message identifies the
  specific slot name and the transform type found. Example:
  `"slot dk2: srgbClr has lumMod child (val=75000); base color #003087 stored with derived flag"`
  (traces to BC-2.01.001 — observability of lossy extraction)

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
  (that would require a separate story with Kani proof obligations — defer to v2 if
  exact resolution is desired).
- Any changes to BC files (the product-owner must widen BC-2.01.001 and BC-2.01.003
  EC-003 independently before this story reaches `ready`).

## BC Widening Prerequisite

Before this story moves from `draft` to `ready`, the product-owner must:

1. Add a new edge case to BC-2.01.001 (Brand Loader BC) covering:
   "srgbClr with lumMod/lumOff/tint/shade children → base val stored with derived
   flag; EC-003-style warning emitted; tracing::warn identifies slot and transform."

2. Widen BC-2.01.003 EC-003 from "schemeClr with modifiers" to
   "schemeClr OR srgbClr with lumMod/tint/shade modifiers" — same behavior:
   write as-is with inline TOML comment.

These are product-owner decisions because they change the observable contract of the
loader output. The story cannot be dispatched until the BCs are updated.

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

If the product-owner decides that exact resolved color computation is required
(Option A in AC-001), the implementation becomes:

```
effective = apply_lummod(base_hex, lummod_val)
           // lummod: multiply luminance by val/100000
           // e.g., val=75000 → 75% of base luminance in HSL space
```

This is non-trivial (requires HSL conversion) and should be scoped to a separate
story if chosen, with a Kani proof obligation for the arithmetic. The `is_derived`
flag + comment approach (Option B) is strongly preferred for v1.0 because it is
simpler, provably correct (no new arithmetic), and transparent to the user.
