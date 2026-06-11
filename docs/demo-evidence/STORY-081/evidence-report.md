# STORY-081 Demo Evidence Report

**Story:** STORY-081 — Slide-Level Inline Markup: eval + layout + all-exporter structural formatting
**Crates:** `slideforge-eval`, `slideforge-layout`, `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html`, `slideforge` (e2e)
**Branch:** `feature/STORY-081`
**Recorded:** 2026-06-10
**Recording tool:** VHS (terminal capture of `cargo nextest` tests)
**HEAD at recording:** `e20e6732`

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-006-eval-title-constraint.gif` | GIF (PR embed) | AC-001 (eval→Inlines), AC-006 (E-EVL-015 warning + strip) |
| `AC-001-006-eval-title-constraint.webm` | WEBM (archival) | AC-001, AC-006 |
| `AC-001-006-eval-title-constraint.tape` | VHS script | AC-001, AC-006 |
| `AC-002-pptx-inline-markup.gif` | GIF (PR embed) | AC-002 (all 8 PPTX forms + EC-004/008/009), AC-006 (title constraint) |
| `AC-002-pptx-inline-markup.webm` | WEBM (archival) | AC-002, AC-006 (PPTX path) |
| `AC-002-pptx-inline-markup.tape` | VHS script | AC-002, AC-006 (PPTX path) |
| `AC-003-005-docx-html-exporters.gif` | GIF (PR embed) | AC-003 (DOCX 8 forms), AC-005 (HTML semantic elements), combined-form e2e |
| `AC-003-005-docx-html-exporters.webm` | WEBM (archival) | AC-003, AC-005 |
| `AC-003-005-docx-html-exporters.tape` | VHS script | AC-003, AC-005 |
| `AC-004-006-pdf-e2e-strict.gif` | GIF (PR embed) | AC-004 (PDF font dispatch + super/sub), AC-006 (strict fatal + warn-only) |
| `AC-004-006-pdf-e2e-strict.webm` | WEBM (archival) | AC-004, AC-006 |
| `AC-004-006-pdf-e2e-strict.tape` | VHS script | AC-004, AC-006 |

---

## Coverage Map

### AC-001 — Eval converts slide-level inline markup fields to FieldValue::Inlines
(BC-3.05.001 precondition 5)

**Demonstrated in recording:** `AC-001-006-eval-title-constraint` — Sections 1–4
**Tests run:**

- `test_BC_3_05_001_ac001_bullet_bold_produces_field_value_inlines` — bold bullet → `FieldValue::Inlines` containing `InlineNode::Bold`, NOT `FieldValue::Str("**Key finding**...")`
- `test_BC_3_05_001_ac001_all_8_inline_forms_in_body` — body with all 8 forms (Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough, Highlight) → `FieldValue::Inlines` with all 8 `InlineNode` variants confirmed present
- `test_BC_3_05_001_ac001_body_inline_markup_produces_field_value_inlines` — body field with markup → Inlines
- `test_BC_3_05_001_ac001_caption_inline_markup_produces_inlines` — caption field with markup → Inlines
- `test_BC_3_05_001_ac001_description_inline_markup_produces_inlines` — description field with markup → Inlines
- `test_BC_3_05_001_ac001_subtitle_inline_markup_produces_inlines` — subtitle field with markup → Inlines
- `test_BC_3_05_001_ac001_rejects_literal_asterisks_in_bullet` — R1 anti-pattern closed: no `**` in any `FieldValue` output

**Command:** `cargo nextest run -p slideforge-eval -E 'test(ac001_*)' --no-fail-fast`
**Observed result:** 18/18 PASS (0.030s total). All slide-level inline fields produce `FieldValue::Inlines` with structural `InlineNode` variants — no literal `**` or `_` present in any output value.

---

### AC-002 — PPTX exporter renders all 8 inline markup forms natively
(BC-3.05.001 PC-1)

**Demonstrated in recording:** `AC-002-pptx-inline-markup` — Sections 1–8
**Tests run:**

- `test_BC_3_05_001_ac002_pptx_textrun_frame_bold_produces_rpr_b1` — `InlineNode::Bold` → `b="1"` in `slide1.xml`
- `test_BC_3_05_001_ac002_pptx_italic_produces_rpr_i1` — `InlineNode::Italic` → `i="1"` in `slide1.xml`
- `test_BC_3_05_001_ac002_pptx_strikethrough_produces_sng_strike` (EC-009) — `InlineNode::Strikethrough` → `strike="sngStrike"` in `a:rPr`
- `test_adv_p10_high_001_highlight_uses_a_highlight_not_solid_fill` (EC-008) — `InlineNode::Highlight` → `<a:highlight><a:srgbClr val="FFFF00"/>` (NOT `solidFill` glyph recolor)
- `test_adv_p11_med_001_body_path_code_emits_courier_new_latin` — `InlineNode::Code` → `<a:latin typeface="Courier New">` in `a:rPr`
- `test_adv_p11_med_001_body_path_superscript_emits_baseline_30000` — `InlineNode::Superscript` → `baseline="30000"` in `a:rPr`
- `test_adv_p11_med_001_body_path_subscript_emits_baseline_neg25000` — `InlineNode::Subscript` → `baseline="-25000"` in `a:rPr`
- `test_adv_p11_med_001_body_path_link_display_text_present` (EC-004) — `InlineNode::Link` → `<a:hlinkClick>` + `TargetMode="External"` rel + display text in `<a:t>`
- `test_BC_3_05_001_ac006_pptx_plain_title_no_rpr_b1` — plain title → zero `b="1"` (PPTX title constraint holds)

**Command:** `cargo nextest run -p slideforge-pptx -E 'test(inline_markup|adv_p10|adv_p11)' --no-fail-fast`
**Observed result:** 23/23 PASS. PPTX `slide1.xml` contains `b="1"` for Bold, `i="1"` for Italic, `strike="sngStrike"` for Strikethrough, `<a:highlight>` with `FFFF00` for Highlight, `<a:latin typeface="Courier New">` for Code, `baseline="30000"` for Superscript, `baseline="-25000"` for Subscript, `<a:hlinkClick>` + External rel for Link. No literal `**` in any `<a:t>` element.

---

### AC-003 — DOCX exporter renders all 8 inline markup forms structurally
(BC-3.05.001 PC-3)

**Demonstrated in recording:** `AC-003-005-docx-html-exporters` — Sections 1–3
**Tests run:**

- `test_BC_3_05_001_ac003_docx_slide_body_inline_markup_wired` (snapshot test) — slide body with all 8 forms → DOCX XML contains `<w:b/>` (Bold), `<w:i/>` (Italic), `<w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>` (Code — RunFonts mechanism, NOT `<w:rStyle w:val="CodeSpan"/>`), `<w:hyperlink>` (Link), `<w:vertAlign w:val="superscript"/>` (Superscript), `<w:vertAlign w:val="subscript"/>` (Subscript), `<w:strike/>` (Strikethrough), `<w:highlight w:val="yellow"/>` (Highlight)
- `test_story_081_c4_docx_body_bold_produces_wb_run_property` (e2e) — full pipeline: `story-081-inline-markup.sf` → DOCX → `<w:b/>` present in output XML
- `test_f_p13_e2e_docx_bold_link_has_wb_and_hyperlink` — combined `Bold([Link])` form → both `<w:b/>` and `<w:hyperlink>` present
- `test_f_p13_e2e_docx_nested_bold_italic_has_wb_and_wi` — nested `Bold([Italic])` form → both `<w:b/>` and `<w:i/>` present

**Command:** `cargo nextest run -p slideforge-docx -E 'test(ac003)' --no-fail-fast && cargo nextest run -p slideforge -E 'test(c4_docx|f_p13_e2e_docx)' --no-fail-fast`
**Observed result:** All PASS. DOCX XML contains structurally correct `w:rPr` elements for all 8 inline markup forms. No literal `**`, `_`, `` ` ``, or `~~` characters in any `<w:t>` element.

---

### AC-004 — PDF exporter renders Bold/Italic/Code via font switching + super/sub positioning
(BC-3.05.001 PC-5)

**Demonstrated in recording:** `AC-004-006-pdf-e2e-strict` — Sections 1–5
**Tests run:**

- `test_BC_3_05_001_ac004_pdf_bold_span_uses_distinct_font_resource` (C2 distinctness) — two distinct OTF fixture font resources embedded in PDF output when Bold is used; PostScript font names for both regular and bold faces appear in uncompressed PDF bytes
- `test_BC_3_05_001_ac004_pdf_bold_uses_font_face_dispatch` — `InlineNode::Bold` dispatches to `ResolvedFontSet.bold` face
- `test_BC_3_05_001_ac004_pdf_italic_uses_font_face_dispatch` — `InlineNode::Italic` dispatches to `ResolvedFontSet.italic` face
- `test_BC_3_05_001_ac004_pc5_superscript_baseline_raised_subscript_lowered` — Superscript: `font_size * 0.583` (smaller) + `baseline_y - (font_size * 0.333)` (raised); Subscript: `font_size * 0.583` (smaller) + `baseline_y + (font_size * 0.333)` (lowered); NOT full parent font size
- `test_BC_3_05_001_ac004_pdf_code_uses_mono_font_face` — `InlineNode::Code` dispatches to `ResolvedFontSet.mono` face
- `test_story_081_c4_pdf_body_bold_text_present_no_asterisks` (e2e) — full pipeline: fixture → PDF → bold text content present, no literal `**` in output

**Note on live rendering:** LibreOffice headless rendering verification is handled by CI's `just ci-visual` job (headless-render + SSIM comparison against committed fixtures). The byte-level evidence above (distinct font resource embedding, SUPER_SUB_SCALE constants) provides CI-safe deterministic proof per AC-004's specified PASS condition.

**Command:** `cargo nextest run -p slideforge-pdf -E 'test(ac004)' --no-fail-fast && cargo nextest run -p slideforge -E 'test(c4_pdf)' --no-fail-fast`
**Observed result:** All PASS. Bold and italic runs use distinct font face instances (separate PostScript font names in PDF). Superscript uses `font_size * 0.583` + raised baseline; subscript uses same scale + lowered baseline — neither at full parent font size.

---

### AC-005 — HTML exporter renders all 8 inline markup forms as semantic HTML elements
(BC-3.05.001 PC-4)

**Demonstrated in recording:** `AC-003-005-docx-html-exporters` — Sections 4–6
**Tests run:**

- `test_BC_3_05_001_ac005_html_bullet_inline_markup_wired` (snapshot test) — slide body with all 8 forms → HTML contains `<strong>` (Bold), `<em>` (Italic), `<code>` (Code), `<a href="...">` (Link), `<sup>` (Superscript), `<sub>` (Subscript), `<del>` (Strikethrough), `<mark>` (Highlight)
- `test_story_081_c4_html_body_bold_produces_strong_element` (e2e) — full pipeline: fixture → HTML → `<strong>` present
- `test_f_p13_e2e_html_bold_link_has_strong_and_anchor` — combined `Bold([Link])` → both `<strong>` and `<a href>` present
- `test_f_p13_e2e_html_nested_bold_italic_has_strong_and_em` — nested `Bold([Italic])` → both `<strong>` and `<em>` present

**Note on axe-core scan:** `@axe-core/playwright` WCAG AA accessibility scan on the HTML preview is run by CI's `just ci-a11y` job per PR. The `<a>` elements have non-empty text content (satisfied by link DSL source text), meeting the accessible-name requirement.

**Command:** `cargo nextest run -p slideforge-html -E 'test(ac005)' --no-fail-fast && cargo nextest run -p slideforge -E 'test(c4_html|f_p13_e2e_html)' --no-fail-fast`
**Observed result:** All PASS. HTML output contains semantic elements `<strong>`, `<em>`, `<code>`, `<a href>`, `<sup>`, `<sub>`, `<del>`, `<mark>` for their respective `InlineNode` variants. No literal `**`, `_`, `` ` `` characters leak into output.

---

### AC-006 — PPTX title with inline markup triggers E-EVL-015 and strips to plain text
(BC-3.05.001 Slide-Level Title Constraint / EC-011)

**Demonstrated in recording:** `AC-001-006-eval-title-constraint` (eval path, Sections 5–7) and `AC-004-006-pdf-e2e-strict` (e2e, Sections 6–8)

**Eval-path tests:**

- `test_BC_3_05_001_ac006_title_with_bold_emits_warning` — `title: "**Bold Title**"` → `EvalError::InlineMarkupInTitle` (E-EVL-015) pushed to diagnostic sink; rendered message contains `**` in the `slide_title` portion and `"Stripped to: 'Bold Title'"` (no asterisks) in the stripped portion — confirming F-P27-MED-001 distinguishing fields
- `test_BC_3_05_001_ac006_title_with_bold_strips_to_plain_str` — title field evaluates to `FieldValue::Str("Bold Title")` (plain, no asterisks) for PPTX path
- `test_BC_3_05_001_ac006_plain_title_no_warning` — plain title → zero E-EVL-015 diagnostics (no false positive)

**E2E strict/warn-only tests:**

- `test_fp25_high_001_strict_mode_title_markup_fails_build` — strict mode (default): `title: "**Bold Title**"` → `BuildError::EvalFailed` returned; `build()` returns `Err`; no output file produced. Confirms: non-zero exit code, E-EVL-015 fatal escalation.
- `test_fp25_high_001_warn_only_title_markup_succeeds` — `--warn-only` mode: same input → `build()` returns `Ok`; PPTX output produced with plain-text title `"Bold Title"` (no `b="1"` on title run); E-EVL-015 warning present in diagnostic sink (non-fatal).
- `test_story_081_c4_pptx_body_no_underscore_literals` — full pipeline PPTX output → zero `**` or `_` in any `<a:t>` text run element across all slides.

**Command:**
```
cargo nextest run -p slideforge-eval -E 'test(ac006)' --no-fail-fast
cargo nextest run -p slideforge -E 'test(fp25_high_001|c4_pptx_body)' --no-fail-fast
```
**Observed result:** All PASS. Strict mode build returns `Err` (non-zero exit, E-EVL-015, no output). Warn-only mode returns `Ok` (output produced, plain title, warning in sink). Zero literal markup characters in any `<a:t>` element in the PPTX pipeline output.

---

## Error Path Coverage Summary

| AC | Error input | Mechanism | Observed result |
|----|-------------|-----------|-----------------|
| AC-001 | Bullet with `**bold**` (pre-STORY-081 behavior) | `eval_slide_node` detects `TemplateChunk::Bold` → calls `chunks_to_inline_nodes` | `FieldValue::Inlines([Bold(...)])` — no literal `**` in any variant |
| AC-002 | `InlineNode::Highlight` — prior impl used `solidFill` (wrong, glyph recolor) | `ooxml_run_to_ooxmlsdk` uses `rpr.a_highlight` (CT_Color, yellow) not `run_properties_choice1` | `<a:highlight><a:srgbClr val="FFFF00"/>` in `a:rPr`; no `<a:solidFill>` |
| AC-002 | `InlineNode::Link` with `javascript:` scheme | Unsafe-scheme guard in `collect_link_urls_from_node` + run emission | Plain text run; no `<a:hlinkClick>`; no External rel |
| AC-004 | `InlineNode::Superscript` at full parent font size (pre-fix) | `SUPER_SUB_SCALE = 0.583` constant + `baseline_y - (font_size * 0.333)` | `draw_text()` called at `font_size * 0.583` — confirmed smaller than parent |
| AC-006 strict | `title: "**Bold Title**"` in default (strict) mode | `EvalError::InlineMarkupInTitle` at `ParseSeverity::Error` → `compile_inner` strict gate | `build()` returns `Err(BuildError::EvalFailed)`; no output produced |
| AC-006 warn-only | Same title input with `strict: false` | E-EVL-015 pushed as non-fatal warning | `build()` returns `Ok`; PPTX title `"Bold Title"` (plain); warning in sink |

---

## Gate Confirmation

All CI gates ran clean before recording (3/3 strict-CLEAN adversary convergence confirmed on feature/STORY-081):

| Gate | Command | Result |
|------|---------|--------|
| Format | `cargo fmt --all -- --check` | CLEAN |
| Clippy (canonical gate) | `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::unwrap_used -W clippy::missing_docs_in_private_items` | CLEAN |
| slideforge-eval inline suite | `cargo nextest run -p slideforge-eval -E 'test(slide_inline_markup)'` | CLEAN (18/18 PASS) |
| slideforge-pptx inline suite | `cargo nextest run -p slideforge-pptx -E 'test(inline_markup)'` | CLEAN (23/23 PASS) |
| slideforge-docx inline suite | `cargo nextest run -p slideforge-docx -E 'test(inline_markup)'` | CLEAN (1/1 PASS) |
| slideforge-html inline suite | `cargo nextest run -p slideforge-html -E 'test(inline_markup)'` | CLEAN (1/1 PASS) |
| slideforge-pdf AC-004 suite | `cargo nextest run -p slideforge-pdf -E 'test(ac004)'` | CLEAN (8/8 PASS) |
| slideforge e2e story-081 | `cargo nextest run -p slideforge -E 'test(story_081)'` | CLEAN (13/13 PASS) |
