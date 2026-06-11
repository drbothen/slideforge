---
review_id: DEMO-REVIEW-2026-06-11
date: 2026-06-11
reviewer: fable-model (deep review)
artifact: sample deck built with merged CLI on develop f3502c50
trigger: human-directed post-Wave-5 demo artifact review
classification:
  b_product_defects: 10  # REND-001..010
  c_expected_gaps: 2     # media embedding (images), chart pipeline
---

# Demo Deep-Review — 2026-06-11

Sample deck generated via `slideforge build` on develop `f3502c50` (81 merged PRs, PPTX + HTML + PDF output).
Reviewed by fable-model at human direction. All findings classified as **(b) product defects in merged code** unless noted.

---

## Findings

### REND-001 [CRIT] — Bullets stacked at (0,0) / placeholder bbox never finalized

**Severity:** CRITICAL
**Formats affected:** PPTX, HTML, PDF
**Root cause:** `slideforge-layout/src/layout.rs:1039-1064` — placeholder bounding-box never finalized at the STORY-073/STORY-088 seam. All bullet shapes are emitted at position (0,0), stacking on top of the slide title and rendering illegible.
**Secondary defect:** PPTX bullet shapes emit duplicate `ph type="body" idx="1"` placeholder descriptor.
**Routing crates:** `slideforge-layout`, `slideforge-pptx`
**Required fix:** Finalize placeholder bbox in layout pass before export; deduplicate ph descriptor.

---

### REND-002 [CRIT] — PDF has no line-wrapping engine

**Severity:** CRITICAL
**Formats affected:** PDF
**Root cause:** `slideforge-pdf` — no line-wrapping engine implemented. Text runs past the page edge and clips; no word-wrap or character-wrap boundary is applied.
**Routing crates:** `slideforge-pdf`
**Required fix:** Implement word-wrap (break at word boundaries) and hard-wrap fallback at page margin boundary before text emission.

---

### REND-003 [CRIT] — `<p:nvGrpSpPr>` missing from slide/notes `<p:spTree>`

**Severity:** CRITICAL
**Formats affected:** PPTX
**Root cause:** `slideforge-pptx` slide serializer and notes serializer — `<p:spTree>` emitted without the required `<p:nvGrpSpPr>` child (CT_GroupShape schema violation). Masters and layouts are correct; only slideN.xml and notesSlideN.xml are defective. PowerPoint will issue a repair prompt on open.
**Routing crates:** `slideforge-pptx`
**Required fix:** Add `<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>` as the mandatory first child of every `<p:spTree>` in slide and notes serializers.

---

### REND-004 [HIGH] — `takeaway` field never renders on-slide

**Severity:** HIGH
**Formats affected:** PPTX, HTML, PDF (all formats)
**Root cause:** The `takeaway` field is only aggregated into a trailing Executive Summary section in DOCX output. The spec mandates a visible takeaway bar on each slide in all presentation formats.
**Routing crates:** `slideforge-eval`, `slideforge-layout`, `slideforge-pptx`, `slideforge-html`, `slideforge-pdf`
**Required fix:** Thread `takeaway` through the layout pass as a designated shape/region and emit it in all renderers. Story-writer must verify which story owns this contract and whether a new story is needed.

---

### REND-005 [HIGH] — Strict-mode exits 0 while silently dropping authored content

**Severity:** HIGH
**Formats affected:** All
**Root cause (primary):** `slideforge-validate` / `slideforge-eval` / `slideforge-plugin-api` — W-VAL-103 warnings for `shape:` and `body` fields do not trigger a non-zero exit in strict mode. Authored content is silently dropped with only a warning.
**Root cause (secondary — RC-4):** `body` field is schema-invalid for the `content` slide type yet is rendered anyway. Validator and eval are inconsistent: `slideforge-validate/src/field_to_block.rs:135` passes `body` through while `content.rs` schema rejects it. This is a validator/eval drift.
**Routing crates:** `slideforge-validate`, `slideforge-eval`, `slideforge-plugin-api`
**Required fix:** Strict-mode must exit non-zero on W-VAL-103 (silent content drop is a strict-default violation). Reconcile `body` field handling between validator and eval for `content` type.

---

### REND-006 [HIGH] — DOCX: bullets entirely dropped; no sectPr; lang dropped

**Severity:** HIGH
**Formats affected:** DOCX
**Root cause:** `slideforge-docx` — bullet list items are emitted as empty `<w:p/>` elements (no `<w:r>` run content). `numbering.xml` is a stub with no actual numbering definitions. `sectPr` (section properties / page size) is absent. `lang` attribute is dropped from all runs.
**Routing crates:** `slideforge-docx`
**Required fix:** Implement bullet run content emission; populate `numbering.xml` with actual abstract/concrete numbering definitions; add `sectPr` with correct page dimensions; propagate `lang` to `<w:lang>` on runs.

---

### REND-007 [MED] — PPTX slideMaster geometry mismatch; progress_bar layout fallback; lang dropped

**Severity:** MEDIUM
**Formats affected:** PPTX
**Root cause (a):** `slideMaster1.xml` declares 4:3 geometry (`cx="6858000" cy="5143500"`) on a deck with 16:9 slide size (`sldSz` `cx="9144000" cy="5143500"`). Footer and date placeholders are positioned off-slide in 16:9.
**Root cause (b):** `progress_bar` slide type has no matching named layout; falls back to layout 1 (generic), losing intended visual behavior.
**Root cause (c):** Run-level `lang` attribute is dropped from all PPTX text runs — required for spell-check and accessibility.
**Routing crates:** `slideforge-pptx`
**Required fix:** Set slide master `sldSz` to match deck aspect ratio; add `progress_bar` layout; propagate `lang` to `<a:rPr lang="..."/>` on all runs.

---

### REND-008 [MED] — HTML chart SVG empty / EMU-px mismatch; PDF progress bar untagged; font issues

**Severity:** MEDIUM
**Formats affected:** HTML, PDF

**HTML defects:**
- Chart inner SVG is empty AND has an EMU/px unit mismatch (would render at ~0.004% scale).
- Image element renders as an empty outline (no `src` populated).
- Bullet items are emitted as positioned `<p>` elements rather than `<ul>/<li>` semantic structure (AT semantics lost; WCAG failure).

**PDF defects:**
- `progress_bar` is tagged `/Artifact` — assistive technology loses the information it conveys.
- Only a single font subset is embedded; no bold variant is included, so bold text renders in the regular weight.

**Routing crates:** `slideforge-html`, `slideforge-pdf`
**Required fix:** Fix EMU-to-px conversion coefficient; populate image src; emit `<ul>/<li>` for bullet lists in HTML; tag progress_bar as `/Figure` with Alt in PDF; embed bold font subset.

---

### REND-009 [HIGH] — CLI: bare relative path fails with empty brand path error

**Severity:** HIGH
**Formats affected:** CLI (all format outputs)
**Root cause:** `slideforge-cli` + brand module — `Path::parent()` of a bare filename (e.g., `deck.sf`) returns `""` (empty string), not `.`. Brand resolution then attempts I/O on an empty path and emits: `"brand I/O error for ''"`. The diagnostic cites the empty path, obscuring the real issue.
**Routing crates:** `slideforge-cli`, brand resolution module
**Required fix:** Normalize bare filenames before `parent()` call: `path.canonicalize()` or `if path.parent() == Some(Path::new("")) { Path::new(".") } else { path.parent() }`. Add regression test for bare-path invocation.

---

### REND-010 [MED] — Chart with no data renders silently empty; unanchored deferral in PPTX chart/image

**Severity:** MEDIUM
**Formats affected:** PPTX (chart/image)

**Defect (a) — silent empty chart:** A chart element with no data source renders as empty with no error or placeholder, suspected violation of STORY-032 error-slide placeholder behavior (BC-1.11.002). Strict mode should surface this as a diagnostic.

**Defect (b) — unanchored deferral [DISCIPLINE VIOLATION]:** PPTX chart serializer and image serializer emit a blip-less `<p:pic>` with a code comment `"deferred to a later story"` but cite NO story ID. This violates CLAUDE.md deferral discipline: *"Adding to the register requires [...] attachment to the specific future story or wave where it will be resolved."* The comment must be replaced with a cited story ID or the behavior must be implemented.

**Routing crates:** `slideforge-pptx`, `slideforge-eval`
**Required fix:** (a) Add error-slide placeholder or diagnostic for chart with no data. (b) Story-writer must assign a story ID to the chart/image PPTX embedding deferral, then update the code comment to cite it.

---

## Expected Gaps (c) — Needing Story-Anchor Verification

These were anticipated deferrals from the current wave but need verification that a concrete story anchor exists.

| Gap | Description | Action needed |
|-----|-------------|---------------|
| PPTX/PDF media embedding | Image binary data not embedded in `.pptx` or `.pdf` output | Verify story ID in STORY-INDEX; if absent, story-writer creates one |
| Chart rendering pipeline | SVG chart generation not wired end-to-end; inner SVG empty | Verify story ID; confirm `plotters` integration story exists |

---

## Human Decision — SEQUENCING-RENDERING-FIX-WAVE (2026-06-11)

**Decision:** Finish CI stabilization first (STORY-092 in delivery, STORY-093 next, then STORY-081 PR #80 merge), THEN run a dedicated **RENDERING-FIX WAVE** for REND-001..010 BEFORE any remaining Wave-5 feature stories.

**Fix-wave prep:** At fix-wave start, story-writer + product-owner create stories with BC anchoring for each REND finding. Verify which gaps (REND-010b, media embedding, chart pipeline) are covered by pending stories vs. need new stories.

**Severity summary:**
- CRIT (3): REND-001, REND-002, REND-003
- HIGH (4): REND-004, REND-005, REND-006, REND-009
- MED (3): REND-007, REND-008, REND-010
