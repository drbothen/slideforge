---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-075
title: "Brand Loader: Footer Detection from .pptx Slide Master/Layout Placeholders"
epic: EPIC-06
wave: 4
points: 3
priority: P1
tdd_mode: strict
status: ready
crate: slideforge-brand
subsystems: [SS-04]
target_module: slideforge-brand
behavioral_contracts: [BC-2.01.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on:
  - STORY-022
blocks:
  - STORY-024
  # STORY-024's BrandExtractor already writes a [footer] section when
  # BrandTemplate.footer_text is Some(). That branch is currently permanently dead
  # because this story's detection capability does not exist yet. This story
  # unblocks STORY-024's [footer] writer (AC-001 / AC-009 traces to BC-2.01.003
  # postcondition 1: "if detected").
estimated_days: 1
created_by: story-writer
created_reason: "Adversary finding F-024A-OBS-1 — loader.rs:214 hardcodes footer_text: None; STORY-022 has no AC for footer detection; BC-2.01.001 postcondition 1 promises footer_text in BrandTemplate."
---

# STORY-075: Brand Loader: Footer Detection from .pptx Slide Master/Layout Placeholders

## Summary

`loader.rs` line 214 currently hardcodes `footer_text: None` in every `BrandTemplate`
returned from `BrandLoader::load()`. This makes the `[footer]` writer in `BrandExtractor`
(STORY-024) permanently unreachable on real `.pptx` input — any `.pptx` with a footer
placeholder will silently produce a `brand.toml` without a `[footer]` section, violating
BC-2.01.001 postcondition 1's promise that `BrandTemplate` includes footer text "(if
detected)".

This story adds footer detection to the load path: read the slide master XML
(`ppt/slideMasters/slideMaster1.xml`) and its corresponding slide layout XMLs, find
`<p:ph type="ftr">` placeholder elements, and extract the full concatenated text of all
`<a:t>` runs. The detected text is stored in `BrandTemplate.footer_text: Option<Arc<str>>`.
A secondary parse of the same `slideMaster1.xml` reads the `<p:hf>` (CT_HeaderFooter)
element and extracts the `ftr`, `dt`, and `sldNum` boolean attributes as footer visibility
flags. **Note:** visibility flags are on `<p:hf>` in the master/layout, NOT in
`presProps.xml` (see adversary H-1 correction in BC-2.01.001 v1.3).

The `BrandExtractor` in STORY-024 already contains the `[footer]` serialization
path — this story makes it reachable.

**Scope boundary:** Detection only. Writing `[footer]` to `brand.toml` is STORY-024.
Writing footer placeholders into exported slides is STORY-037/STORY-038.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| `crates/slideforge-brand/src/loader.rs` (current + additions) | ~4,000 |
| `crates/slideforge-brand/src/footer.rs` (new) | ~2,000 |
| Test fixtures + test code | ~2,500 |
| BC file consulted (BC-2.01.001) | ~1,500 |
| **Total** | **~13,500** |

Agent context budget: 200k tokens. This story is ~7% of budget — within limit.

## Acceptance Criteria

### Footer Detection — BC-2.01.001

- [ ] **AC-001:** `BrandLoader::load()` reads `ppt/slideMasters/slideMaster1.xml` from
  the PPTX ZIP and searches for `<p:sp>` elements containing `<p:ph type="ftr"/>`.
  When at least one footer placeholder is found and has one or more `<a:t>` children
  (across any number of `<a:r>` runs in the paragraph), the **full concatenation** of
  all `<a:t>` text runs in document order is stored as `BrandTemplate.footer_text = Some(...)`.
  "First run only" semantics are explicitly forbidden — footer text is frequently split
  across multiple runs (e.g., styled spans), and discarding all but the first run silently
  loses content. When no footer placeholder is present, or the placeholder exists but all
  `<a:t>` text runs are empty or whitespace-only (after trimming), `BrandTemplate.footer_text`
  is `None` (not an error).
  (traces to BC-2.01.001 postcondition 1 v1.3 — BrandTemplate includes full footer text
  if detected; adversary M-1 clarification applied)

- [ ] **AC-002:** If `slideMaster1.xml` contains a footer placeholder (`<p:ph type="ftr"/>`)
  but no text run, `BrandLoader` also searches `ppt/slideLayouts/slideLayout1.xml` for a
  footer placeholder with a non-empty text run and uses that as the fallback. If neither
  master nor layout has a populated footer, `footer_text` is `None`.
  (traces to BC-2.01.001 postcondition 1 — best-effort detection covers master + layout)

- [ ] **AC-003:** `BrandLoader` reads the `<p:hf>` (CT_HeaderFooter) element from
  `ppt/slideMasters/slideMaster1.xml` (the same file already opened for footer text
  detection in AC-001) and extracts the three boolean visibility attributes:
  - `ftr="1"` (or `true`) → footer text is visible on slides (`show_footer: true`)
  - `dt="1"` (or `true`) → date/time placeholder is visible on slides (`show_date: true`)
  - `sldNum="1"` (or `true`) → slide number placeholder is visible on slides (`show_slide_number: true`)
  These three flags are stored in `BrandTemplate.footer_flags: FooterFlags` (a new struct
  with fields `show_footer: bool`, `show_date: bool`, `show_slide_number: bool`).
  If the `<p:hf>` element is absent from `slideMaster1.xml`, all three flags default to
  `false` and a `tracing::debug!` is emitted.
  **CORRECTION from prior spec (adversary H-1, BC-2.01.001 v1.3):** the prior spec
  incorrectly directed parsing `ppt/presProps.xml` and reading `<p:ftr>`, `<p:dt>`,
  `<p:sldNum>` child elements of `<p:showPr>`. Those child elements do NOT exist in
  ECMA-376 CT_ShowProperties — `<p:showPr>` is the slide-show runtime configuration
  element (present/browse/kiosk, penClr, timings). Footer/date/slide-number visibility
  is carried exclusively by the `<p:hf>` (CT_HeaderFooter) element's boolean attributes
  on slide masters, layouts, and individual slides. Reading `presProps.xml` for footer
  flags would always produce all-false results on real .pptx files, making the feature
  permanently inert. Do NOT read `presProps.xml` for footer visibility.
  (traces to BC-2.01.001 postcondition 1 v1.3 — footer visibility flags from <p:hf>
  attributes on slideMaster1.xml)

- [ ] **AC-004:** `footer_text` and `footer_flags` are populated BEFORE `BrandTemplate` is
  returned from `BrandLoader::load()`. The hardcoded `footer_text: None` at `loader.rs:214`
  is replaced by the detected value. No caller needs to call a separate "detect footer"
  function.
  (traces to BC-2.01.001 postcondition 1 — BrandTemplate is complete on return)

- [ ] **AC-005:** If `ppt/slideMasters/slideMaster1.xml` is absent from the ZIP (malformed
  PPTX that somehow passed the earlier load checks), footer detection is skipped silently
  with `footer_text: None` and `footer_flags: FooterFlags::default()`. A `tracing::debug!`
  message is emitted. This is NOT a fatal error.
  (traces to BC-2.01.001 invariant 2 — loading is read-only; does not affect build exit code)

- [ ] **AC-006:** DOCX format (`word/theme/theme1.xml` path) — DOCX files do not have slide
  master footer placeholders. For DOCX input, `footer_text` and `footer_flags` are always
  `None` / `FooterFlags::default()`. No XML read is attempted for DOCX footer detection.
  (traces to BC-2.01.001 postcondition 1 — DOCX has no footer in brand loading scope)

- [ ] **AC-007:** `BrandTemplate.footer_text` populated by this story flows through to
  `BrandExtractor::extract()` (STORY-024): when `footer_text` is `Some(...)`, the
  `[footer]` section IS written to `brand.toml`. Integration test verifies the
  end-to-end path: load PPTX fixture with footer placeholder → extract → `brand.toml`
  contains `[footer] text = "..."`.
  (traces to BC-2.01.001 postcondition 1 cross-story — activates STORY-024's dormant
  [footer] writer, closing F-024A-OBS-1)

- [ ] **AC-008:** `#![forbid(unsafe_code)]` (NFR-024), `#![warn(missing_docs)]` (NFR-023),
  clippy pedantic clean (NFR-022), `=` version pinning (NFR-025) maintained.

## Previous Story Intelligence

Continues from STORY-022 (Brand Loading: .pptx/.docx Template Extraction).

- STORY-022 already reads `ppt/slideMasters/slideMaster1.xml.rels` for logo detection.
  The same ZIP file handle can be reused to read `slideMaster1.xml` for footer detection.
  Do NOT re-open the ZIP archive — pass the `ZipArchive` reference into the new
  `detect_footer()` function.
- `BrandTemplate` already has a `footer_text: Option<Arc<str>>` field (see `template.rs`).
  This story ADDS a `footer_flags: FooterFlags` field to that struct.
- The `quick-xml` SAX parsing pattern is established in `color.rs` and `font.rs`.
  Follow the same event-loop pattern for `slideMaster1.xml` parsing.
- STORY-023 (Brand Synthesis) does NOT interact with `footer_text` — the synthesizer
  reads `brand.toml` (not raw PPTX), so STORY-023 is unaffected by this story.

## Architecture Compliance Rules

1. **SS-04 (Brand) — Effectful (file I/O):** No change to crate classification.
2. **SAX parsing required:** `slideMaster1.xml` (and `slideLayout1.xml` for the fallback
   text path) must use `quick-xml::Reader` event-loop (no serde). Element ordering is
   schema-significant. `presProps.xml` is NOT read for footer detection.
3. **Read-only invariant:** `BrandLoader` MUST NOT modify the source ZIP. Footer detection
   reads-only. Validated by existing integration test (file hash before/after, inherited
   from STORY-022's `test_load_valid_pptx`).
4. **Forbidden dependencies:** Same as STORY-022. `slideforge-brand` MUST NOT depend on
   `slideforge-eval`, `slideforge-syntax`, `slideforge-validate`, `slideforge-layout`,
   `slideforge-pptx`, `slideforge-pdf`, `slideforge-html`, or `slideforge-cli`.
5. **Struct extension is additive:** Adding `footer_flags: FooterFlags` to `BrandTemplate`
   is a non-breaking change for all existing callsites because `BrandTemplate` is
   constructed only in `loader.rs` and in test fixtures. No external callers construct
   `BrandTemplate` directly.

## Library and Framework Requirements

No new library dependencies required. All needed libraries are already declared in
STORY-022:

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `quick-xml` | `=0.36.2` | SAX parsing of `slideMaster1.xml` (and `slideLayout1.xml` for fallback) |
| `zip` | `=2.6.1` | Reading additional files from the already-open ZIP archive |
| `tracing` | `=0.1` | Debug logging for absent slideMaster1.xml |
| `slideforge-types` | workspace | `Arc<str>` |

Dev dependencies (unchanged from STORY-022):
- `insta = "=1.39.0"` — snapshot tests on extended `BrandTemplate`

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/src/
└── footer.rs    # detect_footer(zip: &mut ZipArchive<File>, is_pptx: bool) -> FooterDetection
                 #   single-pass SAX over slideMaster1.xml:
                 #     (1) <p:ph type="ftr"/> → concat all <a:t> runs → text
                 #     (2) <p:hf ftr=".." dt=".." sldNum=".."/> → FooterFlags
                 # FooterDetection { text: Option<Arc<str>>, flags: FooterFlags }
                 # FooterFlags { show_footer: bool, show_date: bool, show_slide_number: bool }
                 # NOTE: no parse_presProps_flags function — presProps.xml is NOT the
                 #       source for footer visibility flags (adversary H-1 correction)
```

Files to modify:

```
crates/slideforge-brand/src/
├── template.rs   # Add: FooterFlags struct; add footer_flags: FooterFlags to BrandTemplate
├── loader.rs     # Replace: footer_text: None (line 214) with detected value;
                  # call detect_footer() before constructing BrandTemplate;
                  # assign footer_text: footer.text, footer_flags: footer.flags
└── lib.rs        # Add: pub mod footer; pub use footer::FooterFlags;
```

## Tasks

1. **Write `src/footer.rs`:** (35 min)
   - `pub struct FooterFlags { pub show_footer: bool, pub show_date: bool, pub show_slide_number: bool }`
   - `impl Default for FooterFlags` — all `false`
   - `pub struct FooterDetection { pub text: Option<Arc<str>>, pub flags: FooterFlags }`
   - `pub fn detect_footer(zip: &mut ZipArchive<impl Read + Seek>, is_pptx: bool) -> FooterDetection`
     - If `!is_pptx` → return `FooterDetection::default()` immediately (DOCX has no footer detection)
     - Read `ppt/slideMasters/slideMaster1.xml` ONCE — SAX-parse in a single pass for:
       1. `<p:ph type="ftr"/>` placeholder → collect ALL `<a:t>` text runs (concatenate in
          document order) → `text: Option<Arc<str>>`
       2. `<p:hf>` element → read `ftr`, `dt`, `sldNum` boolean attributes → `FooterFlags`
          (attribute absent or `"0"` or `"false"` → `false`; `"1"` or `"true"` → `true`)
          If `<p:hf>` element absent → emit `tracing::debug!`; use `FooterFlags::default()`
     - If master has no footer placeholder text (None) → also try
       `ppt/slideLayouts/slideLayout1.xml` for placeholder text fallback (AC-002)
     - **Do NOT read `ppt/presProps.xml`** for footer flags — CT_ShowProperties has no
       ftr/dt/sldNum children per ECMA-376 (adversary H-1 correction)
     - Return `FooterDetection { text, flags }`
2. **Extend `src/template.rs`:** (10 min)
   - Add `FooterFlags` struct (or re-export from `footer.rs`)
   - Add `pub footer_flags: FooterFlags` field to `BrandTemplate`
3. **Update `src/loader.rs`:** (15 min)
   - Import `footer::detect_footer`
   - After `logo` extraction, call `let footer = detect_footer(&mut zip, is_pptx);`
   - In `BrandTemplate { ... }` constructor: replace `footer_text: None` with
     `footer_text: footer.text`, add `footer_flags: footer.flags`
4. **Update `src/lib.rs`:** (5 min)
   - Add `pub mod footer;` and `pub use footer::FooterFlags;`
5. **Write unit tests in `footer.rs`:** (25 min)
   - See Test Strategy below
6. **Run `cargo clippy -p slideforge-brand -- -D warnings`** — fix. (10 min)
7. **Run `cargo test -p slideforge-brand`** — all tests pass. (10 min)

## Test Strategy

### Unit tests in `footer.rs`

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_detect_footer_text_from_master` | Minimal ZIP with `ppt/slideMasters/slideMaster1.xml` containing `<p:ph type="ftr"/>` with `<a:t>Confidential</a:t>` | `FooterDetection.text = Some("Confidential")` |
| `test_detect_footer_text_multirun` | ZIP with footer placeholder containing two runs: `<a:r><a:t>Q4 </a:t></a:r><a:r><a:t>Report</a:t></a:r>` | `FooterDetection.text = Some("Q4 Report")` (all runs concatenated) |
| `test_detect_footer_empty_master_fallback_layout` | ZIP with footer placeholder in master (empty `<a:t>`) + layout1.xml with `<a:t>Q4 Report</a:t>` | `FooterDetection.text = Some("Q4 Report")` |
| `test_detect_footer_absent_placeholder` | ZIP with slideMaster1.xml that has NO `<p:ph type="ftr"/>` | `FooterDetection.text = None` |
| `test_detect_footer_absent_master_xml` | ZIP with no `ppt/slideMasters/slideMaster1.xml` entry | `FooterDetection.text = None`; no panic; `tracing::debug!` emitted |
| `test_detect_footer_flags_from_hf_element` | ZIP with slideMaster1.xml containing `<p:hf ftr="1" sldNum="1"/>` (no `dt` attribute) | `flags = FooterFlags { show_footer: true, show_date: false, show_slide_number: true }` |
| `test_detect_footer_flags_absent_hf_element` | ZIP with slideMaster1.xml that has NO `<p:hf>` element | `flags = FooterFlags::default()` (all false); `tracing::debug!` emitted; no error |
| `test_detect_footer_docx_returns_default` | `is_pptx = false` | `FooterDetection::default()` immediately; no XML reads |

### Integration tests in `loader.rs` (extending STORY-022 test helpers)

| Test Name | Setup | Expected |
|-----------|-------|----------|
| `test_load_pptx_with_footer` | PPTX fixture (ZIP) with master footer placeholder text "Acme Corp Confidential" | `BrandTemplate.footer_text = Some("Acme Corp Confidential")` |
| `test_load_pptx_without_footer` | Existing STORY-022 minimal PPTX fixture (no footer placeholder) | `BrandTemplate.footer_text = None` (unchanged from before) |
| `test_extract_brand_toml_includes_footer_section` | PPTX fixture with footer text → `BrandExtractor::extract()` | `brand.toml` contains `[footer]` section with `text = "Acme Corp Confidential"` |

The third integration test (`test_extract_brand_toml_includes_footer_section`) is the
end-to-end closure test for adversary finding F-024A-OBS-1. It demonstrates that the
STORY-024 `[footer]` writer is now reachable.

## Dependencies

**Depends on:**
- STORY-022 (Brand Loading: .pptx/.docx Template Extraction) — this story modifies
  `loader.rs` which was created in STORY-022. Cannot exist without STORY-022 merged.

Dependency justification: STORY-075 depends on STORY-022 because it extends
`BrandLoader::load()` and `BrandTemplate` both of which are owned by STORY-022.

**Blocks:**
- STORY-024 (Brand Extraction CLI) — STORY-024's `BrandExtractor` already contains the
  `[footer]` serialization path (see `brand_template_to_config()` in STORY-024's
  implementation notes). That path is permanently dead until this story populates
  `BrandTemplate.footer_text`. After this story merges, STORY-024's AC-001 / AC-009
  become fully exercisable on real PPTX input with footer placeholders.

Blocks justification: STORY-075 blocks STORY-024 because STORY-024's `[footer]` writer
requires a non-None `footer_text` to be reachable, which only this story provides.
(Note: STORY-022 is already merged so the formal blocks relationship is informational
for wave scheduling — STORY-024 is still draft and has not been delivered yet.)

## Implementation Notes

### OOXML Footer Placeholder XPath (informational)

Footer placeholders in slide masters follow this OOXML structure:

```xml
<!-- ppt/slideMasters/slideMaster1.xml -->
<p:sp>
  <p:nvSpPr>
    <p:nvPr>
      <p:ph type="ftr" sz="quarter" idx="11"/>
    </p:nvPr>
  </p:nvSpPr>
  <p:txBody>
    <a:bodyPr/>
    <a:lstStyle/>
    <a:p>
      <a:r><a:t>Footer text content here</a:t></a:r>
    </a:p>
  </p:txBody>
</p:sp>
```

The SAX parser must track: (1) entering a `<p:sp>` element, (2) finding `<p:ph>` with
`type="ftr"` attribute within it, (3) then collecting `<a:t>` text content from **ALL**
`<a:r>` runs in the `<p:txBody>` paragraph (concatenate in document order) — do NOT stop
after the first run. "First run only" silently discards styled text spans. Use a
`String` accumulator and push each `<a:t>` text chunk; convert to `Arc<str>` at the end.
Use a state machine pattern consistent with `color.rs`.

### Footer Visibility Flags: `<p:hf>` on slideMaster1.xml (CORRECTED)

**CORRECTION (adversary H-1 / BC-2.01.001 v1.3):** Footer visibility flags are NOT in
`presProps.xml`. The prior spec was wrong — `<p:showPr>` (CT_ShowProperties) is the
slide-show runtime configuration element with no ftr/dt/sldNum children.

Footer, date, and slide-number visibility are boolean **attributes** on the `<p:hf>`
(CT_HeaderFooter) element, which lives directly under the master/layout/slide root element:

```xml
<!-- ppt/slideMasters/slideMaster1.xml — master-level visibility declaration -->
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ...>
  ...
  <p:hf sldNum="0" hdr="0" ftr="1" dt="1"/>
  ...
</p:sldMaster>
```

Attribute semantics (ECMA-376 CT_HeaderFooter, all optional `xsd:boolean`):
- `ftr="1"` or `ftr="true"` → footer text placeholder is visible (`show_footer: true`)
- `dt="1"` or `dt="true"` → date/time placeholder is visible (`show_date: true`)
- `sldNum="1"` or `sldNum="true"` → slide number placeholder is visible (`show_slide_number: true`)
- `hdr` — header visibility (notes/handout context only; ignored for slide brand loading)
- Attribute absent → `false` (optional boolean, default-absent = off)

The `<p:hf>` element is parsed from the same `slideMaster1.xml` already opened for
placeholder text detection — no additional file read is required. The SAX state machine
must be extended to also emit `<p:hf>` start-element events and extract the three
boolean attributes in the same pass.

**Inheritance note (v1.0 scope):** Slide layouts and individual slides can carry their
own `<p:hf>` that overrides the master. In v1.0 the brand loader reads ONLY the
`slideMaster1.xml` `<p:hf>` element as the baseline `FooterFlags` and does **not**
inspect any `slideLayoutN.xml` for `<p:hf>` at all. Because layout-level `<p:hf>` is
never read in v1, no `tracing::debug!` is emitted for a layout override — layout-override
detection and the corresponding debug log are both **v2-deferred** (together with full
per-layout `FooterFlags` merge). Do NOT add any layout-override detection or debug
logging in the v1 implementation.

### Why No New BC is Needed

BC-2.01.001 postcondition 1 already reads: "A `BrandTemplate` struct is returned
containing [...] and footer text." The "if detected" hedge in STORY-024 AC-001 traces
to this postcondition. The capability was always promised — it simply was not implemented
in STORY-022's scope. This story closes the implementation gap against an existing BC;
no new behavioral contract is required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideMaster1.xml` absent from ZIP | `footer_text: None`; `debug!` log; no build failure |
| EC-002 | Footer placeholder present but `<a:t>` is empty string | Try slideLayout1.xml; if also empty → `None` |
| EC-003 | Multiple `<p:ph type="ftr"/>` in master | Use text from the first one found (document order) |
| EC-004 | `<p:hf>` element absent from `slideMaster1.xml` (master declares no header/footer settings) | `FooterFlags::default()` (all false); `tracing::debug!` emitted; no error |
| EC-005 | DOCX input (`is_pptx = false`) | Skip all detection; return defaults immediately |
| EC-006 | Footer placeholder has `<a:fld>` field element (date field) instead of `<a:r>` | Treat as no text; `footer_text: None` for that placeholder |
| EC-007 | `<p:hf>` element present in `slideMaster1.xml` but all three attributes (`ftr`, `dt`, `sldNum`) are absent | All three flags are `false` (absence = false per ECMA-376 xsd:boolean optional attr default); no error; no warning |
| EC-008 | Footer placeholder text is split across multiple `<a:r>` runs (e.g., styled spans) | All `<a:t>` text content is concatenated in document order; no truncation at first run |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-31 | story-writer | Initial story created from adversary finding F-024A-OBS-1 |
| 1.1 | 2026-06-01 | product-owner | **SPEC DEFECT correction (adversary H-1 / BC-2.01.001 v1.3):** AC-003 rewritten — footer visibility flags are read from `<p:hf>` boolean attributes (`ftr`, `dt`, `sldNum`) on `slideMaster1.xml`, NOT from `<p:showPr>` children of `presProps.xml`. Prior spec would produce permanently all-false `FooterFlags` on real .pptx files. Task 1, File Structure Requirements, Test Strategy (test cases for presProps replaced with `<p:hf>` tests), Implementation Notes (presProps block replaced with corrected `<p:hf>` block), and EC-004/EC-007 all updated consistently. **Adversary M-1 clarification applied:** AC-001 rewritten to specify full concatenation of all `<a:t>` runs (not first-run-only). EC-008 added for multi-run edge case. Summary and SAX parser notes updated to reflect concatenation requirement. |
| 1.2 | 2026-06-01 | product-owner | **v1/v2 scope reconciliation (STORY-075 adversary pass 2 OBS-3 — EC-007 scope correction):** Implementation Notes "Inheritance note" rewritten. Previous text stated per-layout `<p:hf>` overrides "are noted via tracing::debug! but not merged," implying v1 inspects layouts and emits a debug log. This is inconsistent with the actual v1 scope: the v1 brand loader reads ONLY `slideMaster1.xml` `<p:hf>` and does not inspect `slideLayoutN.xml` at all. Layout-override detection and the corresponding `tracing::debug!` are both v2-deferred. No AC changes required — no AC promised the layout-override debug. Aligns with BC-2.01.001 v1.4 (EC-007 reconciliation). |
