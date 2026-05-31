# Evidence Report: STORY-025 — Per-Slide brand_overlay: No Master Switch Invariant

**Story ID:** STORY-025
**Epic:** EPIC-06
**Crate:** `slideforge-brand` + `slideforge-types`
**BC Trace:** BC-2.02.001, BC-2.02.002
**HEAD SHA verified:** `4ce9a7acfee68bbf643f8dfb76a7d409b818e767`
**Workspace test count:** 455 tests run: 455 passed, 0 skipped
**LOCAL adversary convergence:** 3/3 clean passes
**Recording tool:** VHS (terminal library-harness demos via `cargo nextest run`)
**Font:** FiraCode Nerd Font Mono

---

## Library Story Note

STORY-025 is a **library story** — `resolve_overlay()` and the overlay types have no
production CLI caller yet. STORY-037 (PPTX Core Serialization) wires them into the pipeline.
All demos use the test harness (`cargo nextest run`) to drive the library code directly,
following the same pattern as prior library stories (STORY-020, STORY-021, STORY-024).

---

## Coverage Summary

| AC | Description | Demo File | Tests Covered | Status |
|----|-------------|-----------|--------------|--------|
| AC-001 | BrandOverlay + SlideOverlay + LogoOverride struct fields | `AC-001-brand-overlay-structs` | `test_bc_2_02_001_brand_overlay_fields_present`, `test_bc_2_02_001_logo_override_fields_present`, `test_bc_2_02_001_slide_overlay_fields_present`, `test_bc_2_02_001_brand_overlay_hash_clone` | PASSED |
| AC-002 | `resolve_overlay` loads logo bytes from path; media_type inferred | `AC-002-logo-bytes-loaded` | `test_bc_2_02_001_resolve_overlay_logo_path_loads_bytes` | PASSED |
| AC-003 | `footer_text: Some("")` clears text; `None` means no change | `AC-003-footer-some-empty-vs-none` | `test_bc_2_02_001_resolve_overlay_footer_text_some_empty_distinct_from_none`, `test_bc_2_02_001_resolve_overlay_footer_text_none`, `test_bc_2_02_001_invariant_footer_some_empty_distinct_from_none` | PASSED |
| AC-004 | `confidentiality` text passes through to `BrandOverlay` | `AC-004-confidentiality-passthrough` | `test_bc_2_02_001_resolve_overlay_confidentiality_passthrough` | PASSED |
| AC-005 | Missing logo path → `BrandError::FileNotFound` (E-BRD-001) | `AC-005-missing-logo-file-not-found` | `test_bc_2_02_001_resolve_overlay_missing_logo_file_not_found` | PASSED |
| AC-005 (error path) | Path-traversal containment: `../` escape + symlink escape → `LogoOutsideBrandDir`; empty logo_path → `LogoRequired` | `AC-005-path-traversal-guard` | `test_f025_001_resolve_overlay_dotdot_escape_rejected`, `test_f025_001_resolve_overlay_symlink_escape_rejected`, `test_f025_001_resolve_overlay_in_dir_logo_accepted`, `test_f025_a_empty_logo_path_rejected_with_logo_required` | PASSED |
| AC-005 (error path) | Unknown logo extension → `application/octet-stream` + `tracing::warn!` (not a hard error at overlay layer) | `AC-005-unknown-ext-warn` | `test_f025_002_unknown_ext_overlay_resolves_with_warn`, `test_f025_003_media_type_gif`, `test_f025_003_media_type_svg`, `test_f025_003_media_type_extensionless`, `test_f025_003_media_type_unknown_extension`, `test_f025_003_media_type_wmf`, `test_f025_003_media_type_emf` | PASSED |
| AC-006 | Empty `brand_overlay:` block → `resolve_overlay` returns `Ok(None)` | `AC-006-empty-overlay-noop` | `test_bc_2_02_001_resolve_overlay_empty_returns_none`, `test_bc_2_02_001_empty_overlay_is_empty` | PASSED |
| AC-007 | Duplicate `brand_overlay:` blocks rejected at parse time (E-PAR-002) | — | DEFERRED — parser-level rejection (STORY-008/009) | DEFERRED |
| AC-008 | No Master Switch Invariant: `BrandOverlay` and `SlideOverlay` have NO master-switching field (compile-time proof) | `AC-008-no-master-switch-invariant` | `test_bc_2_02_002_invariant_brand_overlay_no_master_field`, `test_bc_2_02_002_invariant_no_second_master_api`, `test_bc_2_02_002_invariant_no_master_path_field`, `test_bc_2_02_002_slide_overlay_type_has_no_master_path` | PASSED |
| AC-009 | `brand_overlay: template "..."` rejected at parse time | — | DEFERRED — parser-level rejection (STORY-008/009) | DEFERRED |
| AC-010 | 10-slide deck with 10 overlays resolves all independently; single-master data-layer proof | `AC-010-all-slides-resolve-independently` | `test_bc_2_02_002_all_slides_have_overlays_resolve_independently` | PASSED |
| AC-011 | `Slide` IR gains `overlay: Option<SlideOverlay>` field; `Hash + Clone`; `None` default | `AC-011-slide-ir-overlay-field` | `test_bc_2_02_001_slide_overlay_none_is_default`, `test_bc_2_02_002_slide_overlay_type_has_no_master_path`, `test_bc_2_02_001_slide_has_overlay_field_hash_clone` | PASSED |
| AC-012 | Duplicate `brand:` declarations rejected at parse time (E-PAR-002) | — | DEFERRED — parser-level rejection (STORY-008/009) | DEFERRED |
| AC-013 | `#![forbid(unsafe_code)]` + clippy pedantic clean + `=` version pinning | `AC-013-forbid-unsafe-clippy-clean` | grep + `cargo clippy -D warnings` | PASSED |

---

## Deferred Acceptance Criteria

Three ACs are deferred per **Architecture Compliance Rule 4** (STORY-025 spec, section "Architecture Compliance Rules"):
parser-level rejections require `slideforge-syntax` (STORY-008/009), which does not yet exist.

| AC | Reason | Blocking Story |
|----|--------|---------------|
| AC-007 | Duplicate `brand_overlay:` block rejection is parse-time (E-PAR-002); requires the `.sf` parser | STORY-008/009 |
| AC-009 | `brand_overlay: template "..."` syntax rejection is parse-time; requires the `.sf` parser | STORY-008/009 |
| AC-012 | Duplicate `brand:` declaration rejection is parse-time; requires the `.sf` parser | STORY-008/009 |

Production wiring of `resolve_overlay()` into the PPTX pipeline (AC-002 final output verification) is deferred to **STORY-037** (PPTX Core Serialization).

---

## AC-001: BrandOverlay + SlideOverlay + LogoOverride Struct Fields

**File:** `AC-001-brand-overlay-structs.{tape,gif,webm}`
**BC trace:** BC-2.02.001 precondition 2 (overlay declares logo, footer_text, confidentiality)

Demonstrates that `BrandOverlay` (in `slideforge-brand`) and `SlideOverlay` (in
`slideforge-types`) have the correct fields with the correct types, and that `LogoOverride`
has `path`, `bytes`, and `media_type`. All three types implement `Hash + Clone + Eq + Debug`
for comemo compatibility.

Tests: `test_bc_2_02_001_brand_overlay_fields_present`, `test_bc_2_02_001_logo_override_fields_present`,
`test_bc_2_02_001_slide_overlay_fields_present`, `test_bc_2_02_001_brand_overlay_hash_clone`

---

## AC-002: Logo Bytes Loaded from Path

**File:** `AC-002-logo-bytes-loaded.{tape,gif,webm}`
**BC trace:** BC-2.02.001 postcondition 1 (overlay slide logo contains override bytes)

Demonstrates that `resolve_overlay()` with a valid PNG logo path reads the file bytes and
returns a `BrandOverlay` with `logo = Some(LogoOverride { bytes, media_type: "image/png" })`.
The data-layer contract (logo bytes loaded from disk) is proven here; the PPTX exporter
wiring (replacing the logo placeholder relationship) is demonstrated in STORY-037.

Tests: `test_bc_2_02_001_resolve_overlay_logo_path_loads_bytes`

---

## AC-003: footer_text Some("") vs None Distinction

**File:** `AC-003-footer-some-empty-vs-none.{tape,gif,webm}`
**BC trace:** BC-2.02.001 postcondition 2, invariant 3 (Some("") clears text; None = no change)

Demonstrates the critical `Some("") ≠ None` semantic. `SlideOverlay { footer_text: Some("") }`
produces `BrandOverlay { footer_text: Some("") }` — NOT `None`. `is_empty()` returns `false`
for `Some("")` (it has explicit intent: clear the footer). This distinction drives different
PPTX export behavior in STORY-037.

Tests: `test_bc_2_02_001_resolve_overlay_footer_text_some_empty_distinct_from_none`,
`test_bc_2_02_001_resolve_overlay_footer_text_none`,
`test_bc_2_02_001_invariant_footer_some_empty_distinct_from_none`

---

## AC-004: Confidentiality Label Passthrough

**File:** `AC-004-confidentiality-passthrough.{tape,gif,webm}`
**BC trace:** BC-2.02.001 postcondition 3 (confidentiality label added/updated)

Demonstrates that `resolve_overlay()` passes `confidentiality: Some("CONFIDENTIAL — DO NOT DISTRIBUTE")`
through unchanged to `BrandOverlay.confidentiality`. Shape positioning at the slide
bottom-right is deferred to the PPTX exporter (STORY-037).

Tests: `test_bc_2_02_001_resolve_overlay_confidentiality_passthrough`

---

## AC-005: Error Paths — Missing Logo, Path Traversal, Empty Path, Unknown Extension

### Missing Logo (E-BRD-001)

**File:** `AC-005-missing-logo-file-not-found.{tape,gif,webm}`
**BC trace:** BC-2.02.001 edge case EC-001 (missing logo path → E-BRD-001; fatal, exit 4)

Demonstrates that a `SlideOverlay` with a non-existent logo path produces
`BrandError::FileNotFound { path }` — the `path` field contains the exact logo path string
declared by the user, not an OS-derived message.

Tests: `test_bc_2_02_001_resolve_overlay_missing_logo_file_not_found`

### Path-Traversal Containment Guard (F-025-001 / E-BRD-007)

**File:** `AC-005-path-traversal-guard.{tape,gif,webm}`
**BC trace:** Security guard — mirrors `synthesizer::load_from_toml` guard (F-PASS13-HIGH-2)

Demonstrates three containment behaviors:
1. `../secret.png` (dotdot escape) → `BrandError::LogoOutsideBrandDir`
2. Symlink inside `brand_dir` pointing outside → `BrandError::LogoOutsideBrandDir` (Unix only)
3. Legitimate in-directory logo → `Ok(Some(overlay))` (positive path)
4. Empty `logo_path: Some("")` → `BrandError::LogoRequired` (mirrors synthesizer guard, F-025-A)

Tests: `test_f025_001_resolve_overlay_dotdot_escape_rejected`,
`test_f025_001_resolve_overlay_symlink_escape_rejected`,
`test_f025_001_resolve_overlay_in_dir_logo_accepted`,
`test_f025_a_empty_logo_path_rejected_with_logo_required`

### Unknown Extension Warning (F-025-002/003)

**File:** `AC-005-unknown-ext-warn.{tape,gif,webm}`
**BC trace:** BC-2.02.001 EC-007 (unknown extension → warn, still resolves)

Demonstrates that a logo with an unrecognized file extension (`.xyz`) resolves `Ok` with
`media_type = "application/octet-stream"` and emits a `tracing::warn!` whose message
contains `"unrecognized extension"`, the logo path, `"media_type set to"`, and
`"may not render in all viewers"`. Hard-error rejection is deferred to the parser/validator
(STORY-008/009). Also covers `.gif`, `.svg`, `.wmf`, `.emf` MIME type coverage via F-025-003.

Tests: `test_f025_002_unknown_ext_overlay_resolves_with_warn`, `test_f025_003_media_type_gif`,
`test_f025_003_media_type_svg`, `test_f025_003_media_type_extensionless`,
`test_f025_003_media_type_unknown_extension`, `test_f025_003_media_type_wmf`,
`test_f025_003_media_type_emf`

---

## AC-006: Empty Overlay Block is a No-op

**File:** `AC-006-empty-overlay-noop.{tape,gif,webm}`
**BC trace:** BC-2.02.001 edge case EC-004 (empty block → warning, no overlay)

Demonstrates that `resolve_overlay(&SlideOverlay { all_none })` returns `Ok(None)` — the
caller (PPTX exporter) sees `None` and skips overlay application. The parse warning for
an empty `brand_overlay:` block is emitted by the parser (STORY-008/009). `SlideOverlay::is_empty()`
correctly returns `true` only when all three fields are `None`.

Tests: `test_bc_2_02_001_resolve_overlay_empty_returns_none`, `test_bc_2_02_001_empty_overlay_is_empty`

---

## AC-008: No Master Switch Invariant (BC-2.02.002)

**File:** `AC-008-no-master-switch-invariant.{tape,gif,webm}`
**BC trace:** BC-2.02.002 postconditions 1, 2, 3 (single sldMaster; all layouts reference same master)

Demonstrates the structural (compile-time) enforcement of DI-016. Both `BrandOverlay` and
`SlideOverlay` are constructed exhaustively with their full field lists — no `template_path`,
`master_path`, or `layout_idx` field. The fact that Rust compiles these exhaustive struct
initializations proves, at the type level, that no master-switch API exists.

PPTX XML verification (every `<p:sld>` references the single `slideMaster1.xml`) is
deferred to STORY-037 where the XML structure is built.

Tests: `test_bc_2_02_002_invariant_brand_overlay_no_master_field`,
`test_bc_2_02_002_invariant_no_second_master_api`,
`test_bc_2_02_002_invariant_no_master_path_field`,
`test_bc_2_02_002_slide_overlay_type_has_no_master_path`

---

## AC-010: All Slides Resolve Independently (10-Slide Deck)

**File:** `AC-010-all-slides-resolve-independently.{tape,gif,webm}`
**BC trace:** BC-2.02.002 edge case EC-002 (overlay on every slide still single master)

Demonstrates that `resolve_overlay()` is pure and stateless: calling it 10 times for a
10-slide deck (each with a distinct `footer_text: Some("Slide N Footer")`) returns 10
independent `BrandOverlay` values with no cross-slide contamination. The single-master
constraint at the PPTX XML level (one `<p:sldMaster>`) is enforced in STORY-037; this
test proves the data-layer side.

Tests: `test_bc_2_02_002_all_slides_have_overlays_resolve_independently`

---

## AC-011: Slide IR Gains overlay Field

**File:** `AC-011-slide-ir-overlay-field.{tape,gif,webm}`
**BC trace:** BC-2.02.001 invariant 2 (overlay stored in Deck IR as metadata on slide node)

Demonstrates that `slideforge-types::Slide` has an `overlay: Option<SlideOverlay>` field
that defaults to `None`, supports `Hash + Clone + Eq`, and does not contain a `master_path`
field. This confirms the no-circular-dependency architecture: `SlideOverlay` lives in
`slideforge-types` (leaf crate), not in `slideforge-brand`.

Tests: `test_bc_2_02_001_slide_overlay_none_is_default`, `test_bc_2_02_002_slide_overlay_type_has_no_master_path`,
`test_bc_2_02_001_slide_has_overlay_field_hash_clone`

---

## AC-013: NFR Compliance (forbid(unsafe_code) + clippy + pinning)

**File:** `AC-013-forbid-unsafe-clippy-clean.{tape,gif,webm}`
**BC trace:** NFR-022 (clippy::pedantic), NFR-024 (forbid unsafe), NFR-025 (= version pinning)

Demonstrates:
1. `grep -r 'forbid(unsafe_code)'` confirms the attribute in both `slideforge-brand/src/lib.rs`
   and `slideforge-types/src/lib.rs`
2. `cargo clippy -p slideforge-brand -p slideforge-types -- -D warnings` exits clean (0 warnings)

Version pinning (`=` prefix on all production deps) is visible in `Cargo.toml` for both crates.

Tests: `grep` + `cargo clippy`
