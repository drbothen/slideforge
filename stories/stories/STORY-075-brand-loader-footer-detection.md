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
`<p:ph type="ftr">` placeholder elements, and extract their text runs. The detected
text is stored in `BrandTemplate.footer_text: Option<Arc<str>>`. A secondary check
reads the presentation properties (`ppt/presProps.xml`) for `<p:showMasterSp>` and
`<p:dt>` / `<p:ftr>` / `<p:sldNum>` footer visibility flags.

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
  When at least one footer placeholder is found and has a non-empty `<a:t>` text run
  child, the first such text run is stored as `BrandTemplate.footer_text = Some(...)`.
  When no footer placeholder is present or all footer placeholders have empty text runs,
  `BrandTemplate.footer_text` is `None` (not an error).
  (traces to BC-2.01.001 postcondition 1 — BrandTemplate includes footer text if detected)

- [ ] **AC-002:** If `slideMaster1.xml` contains a footer placeholder (`<p:ph type="ftr"/>`)
  but no text run, `BrandLoader` also searches `ppt/slideLayouts/slideLayout1.xml` for a
  footer placeholder with a non-empty text run and uses that as the fallback. If neither
  master nor layout has a populated footer, `footer_text` is `None`.
  (traces to BC-2.01.001 postcondition 1 — best-effort detection covers master + layout)

- [ ] **AC-003:** `BrandLoader` reads `ppt/presProps.xml` (if present in the ZIP) and
  extracts the footer visibility flags from `<p:showPr>` child elements:
  - `<p:ftr val="1"/>` → footer text is visible on slides
  - `<p:dt val="1"/>` → date/time is visible on slides
  - `<p:sldNum val="1"/>` → slide number is visible on slides
  These three flags are stored in `BrandTemplate.footer_flags: FooterFlags` (a new struct
  with fields `show_footer: bool`, `show_date: bool`, `show_slide_number: bool`).
  If `presProps.xml` is absent, all three flags default to `false`.
  (traces to BC-2.01.001 postcondition 1 — footer metadata captured comprehensively)

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
2. **SAX parsing required:** `slideMaster1.xml` and `presProps.xml` must use
   `quick-xml::Reader` event-loop (no serde). Element ordering is schema-significant.
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
| `quick-xml` | `=0.36.2` | SAX parsing of `slideMaster1.xml` and `presProps.xml` |
| `zip` | `=2.6.1` | Reading additional files from the already-open ZIP archive |
| `tracing` | `=0.1` | Debug logging for absent slideMaster1.xml |
| `slideforge-types` | workspace | `Arc<str>` |

Dev dependencies (unchanged from STORY-022):
- `insta = "=1.39.0"` — snapshot tests on extended `BrandTemplate`

## File Structure Requirements

Files to create:

```
crates/slideforge-brand/src/
└── footer.rs    # detect_footer(zip: &mut ZipArchive<File>) -> FooterDetection
                 # parse_presProps_flags(xml_bytes: &[u8]) -> FooterFlags
                 # FooterDetection { text: Option<Arc<str>>, flags: FooterFlags }
                 # FooterFlags { show_footer: bool, show_date: bool, show_slide_number: bool }
```

Files to modify:

```
crates/slideforge-brand/src/
├── template.rs   # Add: FooterFlags struct; add footer_flags: FooterFlags to BrandTemplate
├── loader.rs     # Replace: footer_text: None (line 214) with detected value;
                  # call detect_footer() and presProps_flags() before constructing BrandTemplate
└── lib.rs        # Add: pub mod footer; pub use footer::FooterFlags;
```

## Tasks

1. **Write `src/footer.rs`:** (30 min)
   - `pub struct FooterFlags { pub show_footer: bool, pub show_date: bool, pub show_slide_number: bool }`
   - `impl Default for FooterFlags` — all `false`
   - `pub struct FooterDetection { pub text: Option<Arc<str>>, pub flags: FooterFlags }`
   - `pub fn detect_footer(zip: &mut ZipArchive<impl Read + Seek>, is_pptx: bool) -> FooterDetection`
     - If `!is_pptx` → return `FooterDetection::default()` immediately (DOCX has no footer detection)
     - Read `ppt/slideMasters/slideMaster1.xml` → SAX-parse for `<p:ph type="ftr"/>` placeholder
     - If found but empty text → also try `ppt/slideLayouts/slideLayout1.xml`
     - Read `ppt/presProps.xml` → SAX-parse `<p:ftr val="...">`, `<p:dt val="...">`, `<p:sldNum val="...">`
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
| `test_detect_footer_empty_master_fallback_layout` | ZIP with footer placeholder in master (empty `<a:t>`) + layout1.xml with `<a:t>Q4 Report</a:t>` | `FooterDetection.text = Some("Q4 Report")` |
| `test_detect_footer_absent_placeholder` | ZIP with slideMaster1.xml that has NO `<p:ph type="ftr"/>` | `FooterDetection.text = None` |
| `test_detect_footer_absent_master_xml` | ZIP with no `ppt/slideMasters/slideMaster1.xml` entry | `FooterDetection.text = None`; no panic; `tracing::debug!` emitted |
| `test_detect_footer_flags_from_presprops` | ZIP with `ppt/presProps.xml` containing `<p:ftr val="1"/>`, `<p:sldNum val="1"/>`, no `<p:dt>` | `flags = FooterFlags { show_footer: true, show_date: false, show_slide_number: true }` |
| `test_detect_footer_flags_absent_presprops` | ZIP with no `ppt/presProps.xml` | `flags = FooterFlags::default()` (all false); no error |
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
`type="ftr"` attribute within it, (3) then collecting `<a:t>` text content from the
first `<a:r>` in the first `<a:p>` of the `<p:txBody>`. Use a state machine pattern
consistent with `color.rs`.

### presProps.xml Footer Visibility Flags

```xml
<!-- ppt/presProps.xml -->
<p:presentationPr xmlns:p="...">
  <p:showPr>
    <p:ftr val="1"/>       <!-- footer text visible on slides -->
    <p:dt val="1"/>        <!-- date/time visible on slides -->
    <p:sldNum val="1"/>    <!-- slide number visible on slides -->
  </p:showPr>
</p:presentationPr>
```

Parse: look for `<p:showPr>` element, then for `<p:ftr>`, `<p:dt>`, `<p:sldNum>` children.
Extract `val` attribute. `val="1"` → `true`. Any other value or absent attribute → `false`.

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
| EC-004 | `presProps.xml` absent from ZIP | `FooterFlags::default()` (all false); no error |
| EC-005 | DOCX input (`is_pptx = false`) | Skip all detection; return defaults immediately |
| EC-006 | Footer placeholder has `<a:fld>` field element (date field) instead of `<a:r>` | Treat as no text; `footer_text: None` for that placeholder |
| EC-007 | `ppt/presProps.xml` is present but `<p:showPr>` is absent | `FooterFlags::default()` (all false); no error |
