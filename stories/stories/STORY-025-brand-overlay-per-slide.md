---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-025
title: "Per-Slide brand_overlay: (No Master Switch Invariant)"
epic: EPIC-06
wave: 3
points: 3
priority: P1
tdd_mode: strict
status: draft
crate: slideforge-brand
subsystems: [SS-04, SS-06]
target_module: slideforge-brand
behavioral_contracts: [BC-2.02.001, BC-2.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-022
  - STORY-023
blocks:
  - STORY-037
  - STORY-038
estimated_days: 1
---

# STORY-025: Per-Slide brand_overlay: (No Master Switch Invariant)

## Summary

Implement the `BrandOverlay` type and per-slide brand overlay logic in `slideforge-brand`.
A `brand_overlay:` block on a DSL slide overrides specific brand chrome elements
(logo, footer_text, confidentiality_label) on that individual slide without switching the
slide master. The overlay is stored as metadata on the `Slide` IR node (defined in
`slideforge-types` STORY-001). The PPTX exporter (STORY-037) applies it as slide-level
`<p:sp>` shape overrides — NOT as layout or master modifications. The single-master
architecture (DI-016) is absolute in v1.0: the PPTX output always has exactly one
`<p:sldMaster>` regardless of how many slides have `brand_overlay:`.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,000 |
| `crates/slideforge-brand/src/overlay.rs` | ~1,500 |
| `slideforge-types` `Slide` extension | ~500 |
| Test code | ~2,000 |
| BC files consulted (BC-2.02.001, BC-2.02.002) | ~2,000 |
| **Total** | **~9,000** |

Agent context budget: 200k tokens. This story is ~4.5% of budget — within limit.

## Acceptance Criteria

### Brand Overlay Fields — BC-2.02.001

- [ ] **AC-001:** `BrandOverlay` struct is defined in `slideforge-brand::overlay` (and re-exported from `slideforge-types` via a re-export or a separate `SlideOverlay` type that mirrors it). Fields:
  ```rust
  pub struct BrandOverlay {
      pub logo: Option<LogoOverride>,        // None = use deck-level brand logo
      pub footer_text: Option<Arc<str>>,     // None = use deck-level footer; Some("") = clear footer
      pub confidentiality: Option<Arc<str>>, // None = no confidentiality label change
  }
  pub struct LogoOverride {
      pub path: Arc<str>,  // resolved at build time; must exist or BrandError::FileNotFound
      pub bytes: Vec<u8>,  // loaded at brand-apply time
      pub media_type: Arc<str>,
  }
  ```
  (traces to BC-2.02.001 precondition 2 — overlay declares at least one of: logo, footer_text, confidentiality)

- [ ] **AC-002:** In the PPTX output, an overlay logo on a slide is applied by replacing the logo placeholder's image relationship on that specific slide only. Adjacent slides retain the base brand logo. This is implemented in STORY-037 (PPTX exporter); this story defines the `BrandOverlay` type and the `resolve_overlay` function that loads logo bytes from the path.
  (traces to BC-2.02.001 postcondition 1 — overlay slide's logo contains override bytes)

- [ ] **AC-003:** `BrandOverlay.footer_text = Some("CONFIDENTIAL")` replaces the footer placeholder text on the targeted slide. `Some("")` clears the footer text (but NOT the placeholder structure — the `<p:ph type="ftr"/>` element remains). `None` means no change (use deck-level footer). This distinction between `Some("")` and `None` is critical.
  (traces to BC-2.02.001 postcondition 2 and invariant 3 — `Some("")` clears text, not placeholder structure)

- [ ] **AC-004:** `BrandOverlay.confidentiality = Some("CONFIDENTIAL — DO NOT DISTRIBUTE")` adds or updates a confidentiality label shape on the affected slide. The shape is positioned at the bottom-right of the slide (implementation detail left to PPTX exporter in STORY-037). `None` means no confidentiality shape on this slide. In the `Slide` IR, this is represented as `slide.brand_overlay.confidentiality`.
  (traces to BC-2.02.001 postcondition 3 — confidentiality label added/updated)

- [ ] **AC-005:** A `brand_overlay:` block with an invalid logo path (file does not exist) produces `BrandError::FileNotFound { path, span }` mapping to `E-BRD-001` (fatal, exit 4).
  (traces to BC-2.02.001 edge case EC-001 — missing logo path → E-BRD-001)

- [ ] **AC-006:** A `brand_overlay:` block with no fields (empty) is a parse-time warning: `"brand_overlay block has no effect (all fields absent). Remove it or add logo/footer_text/confidentiality."` The build continues with no overlay applied.
  (traces to BC-2.02.001 edge case EC-004 — empty block → warning, no overlay)

- [ ] **AC-007:** Multiple `brand_overlay:` blocks on the same slide are rejected at parse time with `E-PAR-002: duplicate brand_overlay block on slide at <file>:<line>`.
  (traces to BC-2.02.001 edge case EC-005 — duplicate block → E-PAR-002)

### No Master Switch Invariant — BC-2.02.002

- [ ] **AC-008:** The single-master invariant is enforced at the IR level in this story: an integration test verifies that `BrandOverlay` data structures contain only field-level overrides (no master references, no `master_path` field). PPTX XML relationship-graph verification (that every `<p:sld>` references a `slideLayout` pointing to the single `slideMaster1.xml`) is deferred to STORY-037.
  (traces to BC-2.02.002 postconditions 1, 2, 3 — single sldMaster; all layouts reference same master)

- [ ] **AC-009:** A `brand_overlay:` block that attempts to set a template path (e.g., `brand_overlay: template "other.pptx"`) is rejected at parse time with: `E-PAR-NNN: brand_overlay does not accept a template path. Use brand: at deck level for the master brand. Per-slide overlay supports: logo, footer_text, confidentiality.`
  (traces to BC-2.02.002 edge case EC-001 — template path in overlay → parse error)

- [ ] **AC-010:** `brand_overlay:` on every slide in the deck is supported. The synthesized PPTX still has exactly 1 `<p:sldMaster>` element. All N slides have per-slide shape overrides. Integration test uses a 10-slide deck with 10 overlays.
  (traces to BC-2.02.002 edge case EC-002 — overlay on every slide still single master)

- [ ] **AC-011:** The `Slide` IR type (from `slideforge-types`, STORY-001) gains an `overlay: Option<BrandOverlay>` field. This field is set by the parser (STORY-006/008) when it encounters a `brand_overlay:` block. The overlay is stored in the `Slide` before evaluation — it is metadata, not evaluated content.
  (traces to BC-2.02.001 invariant 2 — overlay stored in Deck IR as metadata on slide node)

- [ ] **AC-012:** Two brand declarations in the same `.sf` file produce `E-PAR-002: duplicate brand declaration; only one brand allowed per deck`. `@include` fragments that also have `brand:` are merged (last-wins per DI-020), maintaining single master.
  (traces to BC-2.02.002 edge case EC-003 and EC-004 — two brand: decls → parse error; @include brand merges)

- [ ] **AC-013:** `#![forbid(unsafe_code)]` (NFR-024), clippy clean (NFR-022), `=` version pinning (NFR-025) maintained.

## Previous Story Intelligence

Continues from STORY-022 and STORY-023. Key lessons:
- `BrandTemplate` and `LogoAsset` structs are already defined in STORY-022. `LogoOverride` in this story mirrors `LogoAsset` but represents a per-slide override rather than the master brand logo.
- The `BrandOverlay` type must be usable from `slideforge-types` (for the `Slide` IR) without creating a circular dependency. Options: (a) define `BrandOverlay` in `slideforge-types` with minimal fields (no `Vec<u8>` logo bytes, just the path), and keep `LogoOverride` with loaded bytes in `slideforge-brand`; (b) define `BrandOverlay` in `slideforge-brand` and have `slideforge-types` reference it via a re-export. Option (a) is preferred to avoid circular deps.

## Architecture Compliance Rules

1. **No circular dependency:** `slideforge-types` → `slideforge-brand` would be circular (brand depends on types). The `Slide.overlay` field must use a type defined in `slideforge-types`, NOT in `slideforge-brand`. Solution: define a thin `SlideOverlay` in `slideforge-types` (path + text + confidentiality string fields, no logo bytes), and have `slideforge-brand` interpret `SlideOverlay` into `BrandOverlay` (with loaded bytes) at export time.
2. **Single-master invariant is enforced at PPTX serialization (STORY-037):** This story defines the overlay data type and loading logic. The invariant enforcement (verifying one `sldMaster` in output) is tested here but enforced during PPTX serialization.
3. **`brand_overlay:` is parse-time metadata:** The overlay block is parsed by `slideforge-syntax` (STORY-008/009) and stored on `Slide` as `SlideOverlay`. The data is not evaluated (no `{{ }}` interpolation in overlay field values in v1.0).
4. **Forbidden dependencies:** `slideforge-brand` still cannot depend on `slideforge-eval`, `slideforge-syntax`, `slideforge-pptx`, etc.

## Library and Framework Requirements

No new libraries required. All deps from STORY-022 are sufficient.

One minor extension: `slideforge-types` gains a new `slide_overlay` module. This requires a one-line modification to `crates/slideforge-types/Cargo.toml` — no new deps.

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/src/
└── overlay.rs             # BrandOverlay, LogoOverride, resolve_overlay()
```

Files to modify:

```
crates/slideforge-types/src/
├── slide_overlay.rs       # NEW: SlideOverlay struct (thin, path-only, lives in types crate)
└── slide.rs               # Add: overlay: Option<SlideOverlay> field to Slide struct

crates/slideforge-brand/src/
├── lib.rs                 # Add: pub mod overlay;
└── error.rs               # No changes needed (BrandError::FileNotFound already exists)
```

## Tasks

1. **Write `crates/slideforge-types/src/slide_overlay.rs`** — `SlideOverlay` struct:
   ```rust
   pub struct SlideOverlay {
       pub logo_path: Option<Arc<str>>,
       pub footer_text: Option<Arc<str>>,
       pub confidentiality: Option<Arc<str>>,
       pub span: SourceSpan,
   }
   ```
   Implement `Hash + Eq + Clone + Debug`. (15 min)
2. **Modify `crates/slideforge-types/src/slide.rs`** — add `pub overlay: Option<SlideOverlay>` field to `Slide`. (5 min)
3. **Modify `crates/slideforge-types/src/lib.rs`** — add `pub mod slide_overlay;`. (5 min)
4. **Write `crates/slideforge-brand/src/overlay.rs`:**
   - `BrandOverlay { logo: Option<LogoOverride>, footer_text: Option<Arc<str>>, confidentiality: Option<Arc<str>> }`.
   - `LogoOverride { bytes: Vec<u8>, media_type: Arc<str>, path: Arc<str> }`.
   - `fn resolve_overlay(raw: &SlideOverlay, root_dir: &Path) -> Result<Option<BrandOverlay>, BrandError>`: reads logo bytes from `raw.logo_path` if set; validates file exists; constructs `BrandOverlay`. Returns `Ok(None)` if all fields are `None` (empty overlay, no-op).
   (30 min)
5. **Write unit tests for `overlay.rs`** — see Test Strategy. (20 min)
6. **Write integration test for single-master invariant** — create a 3-slide `BrandTemplate` + 2 `SlideOverlay` entries; call `resolve_overlay` for each; verify the resulting `BrandOverlay` structures (this verifies the data layer; the single-master XML verification is in STORY-037). (15 min)
7. **Run `cargo clippy -p slideforge-brand -- -D warnings`** and `cargo clippy -p slideforge-types -- -D warnings` — fix. (10 min)
8. **Run `cargo test -p slideforge-brand && cargo test -p slideforge-types`** — all pass. (10 min)

## Test Strategy

### Unit tests for `overlay.rs`

| Test | Setup | Expected |
|------|-------|----------|
| `test_resolve_overlay_logo_path_loads_bytes` | Create temp PNG file; `SlideOverlay { logo_path: Some(path), .. }` | `BrandOverlay.logo = Some(LogoOverride { bytes: file_bytes, .. })` |
| `test_resolve_overlay_missing_logo` | `SlideOverlay { logo_path: Some("missing.png"), .. }` | `BrandError::FileNotFound` |
| `test_resolve_overlay_empty_returns_none` | `SlideOverlay { logo_path: None, footer_text: None, confidentiality: None }` | `Ok(None)` |
| `test_resolve_overlay_footer_text_some_empty` | `SlideOverlay { footer_text: Some(""), .. }` | `BrandOverlay.footer_text = Some("")` (NOT `None` — distinction matters) |
| `test_resolve_overlay_footer_text_none` | `SlideOverlay { footer_text: None, .. }` | `BrandOverlay.footer_text = None` |
| `test_resolve_overlay_confidentiality` | `SlideOverlay { confidentiality: Some("CONFIDENTIAL"), .. }` | `BrandOverlay.confidentiality = Some("CONFIDENTIAL")` |

### Slide IR modification tests (in `slideforge-types`)

- `test_slide_has_overlay_field` — construct a `Slide` with `overlay: Some(SlideOverlay { ... })` and verify `Hash + Clone` work correctly.
- `test_slide_overlay_none_is_default` — verify `Slide { overlay: None, .. }` compiles and hashes consistently.

## Dependencies

**Depends on:**
- STORY-022 (Brand Loading) — `BrandError`, `LogoAsset` pattern, `BrandLoadContext`.
- STORY-023 (Brand Synthesis) — `BrandTemplate` must exist before per-slide overlays can be tested against it.

Dependency justification: STORY-025 depends on STORY-022 because the overlay type (`LogoOverride`) mirrors `LogoAsset` and shares the same `BrandError::FileNotFound` error contract. It depends on STORY-023 because the overlay tests verify that overlays do not modify the single-master structure established during brand synthesis.

**Blocks:**
- STORY-037 (PPTX Core Serialization) — the PPTX exporter reads `Slide.overlay` and applies `BrandOverlay` at the slide level during serialization.
- STORY-038 (PPTX Layout Compliance) — the layout compliance story verifies that the single-master invariant holds for all slides including those with overlays.

## Implementation Notes

### `Some("")` vs `None` for footer_text

This is the most critical semantic distinction in this story. BC-2.02.001 invariant 3 states:
- `footer_text: Some("")` → clears the footer placeholder TEXT (but placeholder structure remains in XML).
- `footer_text: None` → no change to footer (deck-level brand footer text is used).

The PPTX exporter (STORY-037) must distinguish these two cases. The `Option<Arc<str>>` type naturally represents this: `None` = no change, `Some("")` = explicitly empty.

At the `SlideOverlay` level (in `slideforge-types`), `footer_text: Option<Arc<str>>` carries the same semantics. The parser (STORY-008) sets `footer_text = Some(Arc::from(""))` when `footer_text: ""` is written in the DSL, and `footer_text = None` when the field is absent entirely.

### Single-Master Invariant Test Strategy

The full single-master test (verifying PPTX XML has exactly one `<p:sldMaster>` element)
is an integration test that runs in STORY-037. However, this story can validate the
DATA-LAYER constraint: `resolve_overlay()` produces `BrandOverlay` objects that contain
only shape-override data (logo bytes, footer text, confidentiality string) — NEVER
a master reference or layout change. Asserting this at the type level (via `BrandOverlay`
having no `master_path` or `layout_idx` fields) provides compile-time evidence of the
constraint.

### DSL Parse Rejection for `brand_overlay: template "..."

This rejection is in the parser (STORY-008/009 — `slideforge-syntax` crate). At this
story's level, we need only ensure that `SlideOverlay` in `slideforge-types` has no
`template_path` field — making it structurally impossible to represent an overlay that
switches masters.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `brand_overlay:` logo path does not exist | `BrandError::FileNotFound` → E-BRD-001; fatal, exit 4 |
| EC-002 | `brand_overlay:` on a `slide end:` type | Overlay applied; CL-11 layout still used; logo/footer overrides are slide-level shapes |
| EC-003 | `brand_overlay:` in `@for` loop | Each iteration's `Slide` gets its own `SlideOverlay`; `resolve_overlay()` called per slide |
| EC-004 | Empty `brand_overlay:` block | `resolve_overlay()` returns `Ok(None)`; parse warning emitted by parser |
| EC-005 | Multiple `brand_overlay:` blocks on same slide | Rejected at parse time with `E-PAR-002` |
| EC-006 | All slides have `brand_overlay:` | All overlays resolved; PPTX still has 1 `<p:sldMaster>` |
| EC-007 | `footer_text: ""` (explicitly empty) | `Some(Arc::from(""))` — distinct from `None`; clears footer text content only |
| EC-008 | `footer_text:` absent entirely | `None` — deck-level footer text is used unchanged |
