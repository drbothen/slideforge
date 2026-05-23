---
title: WCAG AA Accessibility Bar for slideforge
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: architect (ADR-011 input) + product-owner (NFR definition)
---

# WCAG AA — slideforge Accessibility Bar

## Executive Summary

slideforge produces and serves four user-facing surfaces, and each one has a distinct accessibility contract. There is **no single tool** that covers all of them, so the v1.0 accessibility NFR must be expressed per-surface.

**The contract for v1.0:**

1. **Web preview (browser UI + canvas-rendered slides)** — MUST satisfy **WCAG 2.2 Level AA**, verified by axe-core (recommended via Playwright) in CI on every PR touching the preview. Zero violations on rule severities `serious` and `critical`; `moderate` and `minor` reported but not gating.
2. **HTML exporter output** — MUST satisfy **WCAG 2.2 Level AA**, verified by the same axe-core configuration applied to the static export, with the additional constraint that the output is consumable by a screen reader without JavaScript.
3. **PDF exporter output** — MUST be a **tagged PDF** conforming to **PDF/UA-1 (ISO 14289-1)** with WCAG 2.2 AA contrast verified. Verified via veraPDF (CI / headless) for spec-level conformance and PAC 2024 (manual gate) for UX-oriented validation.
4. **PPTX exporter output** — MUST pass **Microsoft PowerPoint Accessibility Checker** with zero "Errors"; "Warnings" reviewed and triaged. Verified via a combination of OOXML-level invariants (we own the emitter — we enforce them at generation time) and a per-release manual run of the Microsoft Accessibility Checker on the holdout deck set.

**Underlying DSL contract:** slideforge must surface accessibility primitives in its DSL (alt text, language tags, color-independence affordances) so authors can comply without leaving the DSL. The accessibility bar cannot be satisfied by export-time hacks alone — it must be a first-class concept in the IR.

---

## Scope by Surface

| Surface | Accessibility Bar | Tooling (recommended) | Test Surface |
|---|---|---|---|
| Web preview (browser canvas + UI controls) | WCAG 2.2 AA | Playwright + axe-core (`@axe-core/playwright`) | per-PR a11y CI |
| HTML exporter output | WCAG 2.2 AA | Playwright + axe-core (same config); fallback validator: pa11y with axe runner | per-build a11y CI |
| PDF exporter output | PDF/UA-1 (ISO 14289-1) + WCAG 2.2 AA contrast | veraPDF (CI), PAC 2024 (manual / release gate) | per-build PDF a11y CI + per-release manual |
| PPTX exporter output | Microsoft Accessibility Checker (zero errors) + emitter-enforced OOXML invariants | Custom OOXML linter (CI) + Microsoft Accessibility Checker (manual) | per-build PPTX lint + per-release manual |

> **Rationale for axe-core over pa11y or Lighthouse on web surfaces:** Lighthouse runs only ~50 of axe-core's ~96 rules and is tuned for Chrome DevTools, so it materially under-reports issues. pa11y is solid but is a CLI runner — for a Rust project that already needs Playwright for E2E web-preview testing (ADR-005 / ADR-008 imply this), reusing the Playwright runtime with `@axe-core/playwright` minimizes the Node.js surface area we must own.

---

## WCAG 2.2 AA Criteria Most Relevant to Presentations

The full WCAG 2.2 specification is at https://www.w3.org/TR/WCAG22/ . The following table is the set we have determined to be **load-bearing for slideforge's surfaces**. Each row lists a criterion, its canonical W3C URL, why it matters for slideforge, and how slideforge complies.

| Criterion | Level | Canonical URL | Relevance to slideforge | How slideforge complies |
|---|---|---|---|---|
| **1.1.1 Non-text Content** | A | https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html | Every image, chart, icon, decorative graphic in a slide needs a text alternative | DSL provides `alt "..."` on image/chart/icon blocks; emitter propagates to OOXML `cNvPr@descr`, HTML `<img alt>`, PDF `/Alt` on `<Figure>` tag; decorative items get explicit `decorative: true` → empty alt / Artifact |
| **1.3.1 Info and Relationships** | A | https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html | Slide titles, section structure, table headers, list semantics must be programmatically determinable | DSL preserves semantic types (title, section_header, list, table); emitter outputs `<H1>..<Hn>` in HTML, heading-tagged `<H>` in PDF, real slide title placeholders in PPTX (not "text boxes that look like titles"); tables emit `<thead>`/`<th scope>` |
| **1.3.2 Meaningful Sequence** | A | https://www.w3.org/WAI/WCAG22/Understanding/meaningful-sequence.html | Reading order must match visual order across all 23 slide types | IR carries explicit reading order; PPTX emitter orders `<p:spTree>` children in reading order; HTML uses document order; PDF structure tree is built from IR order, not from visual positioning |
| **1.4.1 Use of Color** | A | https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html | severity_cards, status, progress_bar, weighted_composite, formula all encode meaning via color | DSL requires text labels alongside color cues (e.g., `severity: high` always renders both color and the word "High" + a shape glyph); brand validator emits a hard error if a color-coded slide type lacks a text label |
| **1.4.3 Contrast (Minimum)** | AA | https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html | All slide text on brand backgrounds; 4.5:1 normal, 3:1 large (>= 18pt or 14pt bold) | Brand validator (S-spike) computes contrast for every `(fg, bg)` pair in the theme using the WCAG luminance formula; rejects themes that fail; reports per-pair ratios |
| **1.4.10 Reflow** | AA | https://www.w3.org/WAI/WCAG22/Understanding/reflow.html | Web preview + HTML export must reflow at 400% zoom without horizontal scroll | HTML exporter emits responsive CSS (no fixed-width layout); web preview canvas uses CSS scaling. PPTX is fixed-layout by format (exempt per WCAG note: "spatial layout to convey information" exemption applies for chart-like content, but only where layout is the meaning) |
| **1.4.11 Non-text Contrast** | AA | https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html | Chart strokes, severity badges, progress fill, focus outlines must hit 3:1 vs adjacent colors | Brand validator extends contrast check to graphical elements; chart renderer enforces 3:1 between adjacent series colors AND between series fill and background |
| **1.4.12 Text Spacing** | AA | https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html | User-applied line-height 1.5, paragraph 2x, letter 0.12, word 0.16 must not cut off content | Web preview + HTML output use relative units, no `overflow: hidden` on text containers, no fixed text-box heights. PPTX is exempt by format. |
| **1.4.13 Content on Hover or Focus** | AA | https://www.w3.org/WAI/WCAG22/Understanding/content-on-hover-or-focus.html | Web preview tooltips (e.g., source span info on hover) must be dismissable, hoverable, persistent | Web preview UI conformance |
| **2.1.1 Keyboard** | A | https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html | All web preview UI must be operable by keyboard alone | Web preview UI: every control has a keyboard handler; no mouse-only interactions |
| **2.1.2 No Keyboard Trap** | A | https://www.w3.org/WAI/WCAG22/Understanding/no-keyboard-trap.html | Web preview must not trap focus in any panel | UI implementation; verified by axe |
| **2.2.2 Pause, Stop, Hide** | A | https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html | Web preview live-reload (websocket-triggered re-render) is a moving/updating affordance | Provide a "pause live-reload" toggle in the preview UI; document keyboard shortcut. |
| **2.3.1 Three Flashes or Below Threshold** | A | https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html | Any transitions/animations slideforge emits must not flash > 3 Hz | slideforge v1.0 does NOT emit slide transitions or build animations — this criterion is trivially satisfied. If post-v1.0 adds animations, they must be rate-limited to <= 3 flashes/sec and tested |
| **2.4.2 Page Titled** | A | https://www.w3.org/WAI/WCAG22/Understanding/page-titled.html | HTML export and web preview need a `<title>` reflecting deck name | Emitter populates `<title>` from deck-level `title` field in DSL |
| **2.4.3 Focus Order** | A | https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html | Web preview tab order matches visual order | Web preview UI |
| **2.4.6 Headings and Labels** | AA | https://www.w3.org/WAI/WCAG22/Understanding/headings-and-labels.html | Slide titles must be descriptive (not "Slide 5"); section headings labeled | DSL requires titles to be non-empty; lint warns on duplicate adjacent titles |
| **2.4.7 Focus Visible** | AA | https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html | Web preview focus indicator must be visible (>= 3:1 contrast) | UI implementation; tested by axe |
| **2.4.11 Focus Not Obscured (Minimum)** | AA | https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html | New in WCAG 2.2 — focused element not fully hidden by overlays | Web preview UI design constraint |
| **2.5.7 Dragging Movements** | AA | https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html | New in 2.2 — any drag-to-resize / drag-to-reorder in preview needs single-pointer alternative | Web preview UI design constraint |
| **2.5.8 Target Size (Minimum)** | AA | https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html | New in 2.2 — interactive targets >= 24x24 CSS px | Web preview UI design constraint |
| **3.1.1 Language of Page** | A | https://www.w3.org/WAI/WCAG22/Understanding/language-of-page.html | HTML root `<html lang>`, PDF `/Lang`, PPTX presentation-level + run-level `lang` | DSL has top-level `lang: "en-US"` (default `"en"`); emitter propagates to all three formats |
| **3.1.2 Language of Parts** | AA | https://www.w3.org/WAI/WCAG22/Understanding/language-of-parts.html | Mixed-language text runs need per-run language tags | DSL supports inline `{:lang "fr"}foo bar{/}` style annotation (post-v1.0 candidate; v1.0 requires only page-level) |
| **4.1.2 Name, Role, Value** | A | https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html | Web preview UI controls have correct names/roles | UI implementation; verified by axe |
| **4.1.3 Status Messages** | AA | https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html | Web preview parse-error notifications, "deck reloaded" toasts must use `aria-live` | UI implementation |

**Criteria covered: 23 success criteria explicitly addressed across surfaces** (counting both Level A and Level AA, since Level AA presumes A). Several Level AAA criteria (e.g., 1.4.6 Contrast Enhanced 7:1) are out of scope for v1.0.

---

## OOXML Accessibility Encoding

The PPTX exporter must encode the following accessibility metadata into OOXML. (Sources: ISO/IEC 29500 PresentationML; Microsoft Office Open XML documentation; cross-referenced via PowerPoint accessibility behavior.)

### Alt text

Two encoding paths exist and **both must be populated** because different consumers (PowerPoint, Keynote, Google Slides, LibreOffice) read different fields:

- **Primary:** `<p:cNvPr id="N" name="..." descr="ALT TEXT HERE" title="SHORT TITLE"/>` — the `descr` attribute is what PowerPoint's Accessibility Checker validates.
- **Secondary for images:** `<a:blip>` with `<a:extLst>` carrying an `<a14:useLocalDpi>` extension is NOT the alt-text path. The image's alt text comes from the parent `cNvPr@descr`. (Note: there is no `<a:descr>` element in standard schema — the earlier survey conflates this with `cNvPr@descr`. Implementer to verify against ISO/IEC 29500 directly.)
- **For grouped shapes:** alt text must be on EACH constituent shape's `cNvPr@descr`, NOT only on the `<p:grpSp>` parent. Grouping in PowerPoint strips child alt text — slideforge's emitter must not do this.

### Slide titles

- Each slide MUST have a title placeholder of `type="title"` or `type="ctrTitle"` on `<p:ph>` inside a `<p:sp>` — NOT a text box that looks like a title.
- The Accessibility Checker considers a slide "missing title" if there is no shape with `<p:ph type="title">` or `<p:ph type="ctrTitle">`.

### Reading order

OOXML reading order is determined by the order of `<p:sp>` children inside `<p:spTree>`. There is no separate `tab-index` field in the standard schema for shapes (despite older folklore). slideforge's emitter must therefore emit shapes in **logical reading order**, not z-order. Where visual layering differs from reading order, use `<p:nvSpPr>` z-order properties for layering, but keep XML document order = reading order.

### Language

- Presentation-level: `<p:defaultTextStyle>` / `<a:defRPr lang="en-US"/>` on the master.
- Run-level: every `<a:r><a:rPr lang="en-US"/>` should be populated. Mixed-language runs use different `lang` attribute values per run.

### Table headers

- Tables: set `<a:tblPr firstRow="1"/>` on `<a:tbl>` to indicate the first row is a header. This is what Accessibility Checker reads.

### Notes pages

Speaker notes live in a separate `notesSlide` part. Critical caveat: **Speaker notes are NOT read by screen readers during presentation playback**, and in many PDF export configurations they are dropped entirely. Therefore:

- **slideforge MUST treat speaker notes as supplementary only.** The lint shall warn if a slide has critical content (alt-text-bearing elements, content referenced by reading-order semantics) ONLY in notes.
- Notes pages themselves should still be authored accessibly (titles, language tags) for the case where someone navigates a deck in editor view.

---

## PDF/UA Requirements

PDF/UA-1 (ISO 14289-1) is the binding standard. Sources: https://en.wikipedia.org/wiki/PDF/UA ; PDF Association Matterhorn Protocol (31 checkpoints, 136 failure conditions).

slideforge's PDF emitter MUST produce a **tagged PDF** satisfying:

1. **Document structure tree** rooted in `<Document>`, with logical hierarchy.
2. **Heading hierarchy** using `<H1>..<H6>` (no level skipping for purely visual reasons).
3. **Figures** wrapped in `<Figure>` with `/Alt` populated (from DSL `alt`).
4. **Tables** with `<Table>` / `<TR>` / `<TH>` / `<TD>` and correct header scope.
5. **Lists** with `<L>` / `<LI>` / `<Lbl>` / `<LBody>`.
6. **Decorative content** marked as **Artifact**, not as untagged content.
7. **Document `/Lang`** set (from DSL `lang`).
8. **Per-run language** changes tagged where the DSL provides them.
9. **Embedded fonts**, Unicode-mapped, text-selectable (no rasterized text).
10. **Display title** in document metadata (`/Title`) populated from deck DSL title.
11. **No security settings** that block AT access (`/Perms` must not disable copy/extract).
12. **Reading order** in the structure tree matches intended visual reading order.

PDF backend implications (ADR-003): of the three candidates (Typst-as-backend, direct printpdf/lopdf, HTML→PDF via headless browser):

- **Typst-as-backend:** Typst v0.13+ has improving PDF/UA support but is NOT yet certifiable for PDF/UA-1 out of the box (as of 2026-05). Requires extension or post-processing.
- **printpdf/lopdf:** Low-level; slideforge would own all tagging itself. Highest control, highest implementation cost.
- **HTML→PDF via headless browser (Chrome `--print-to-pdf` or `wkhtmltopdf`):** Modern Chrome produces tagged PDFs from semantically marked-up HTML. This is the lowest-effort path to PDF/UA-passing output IF the HTML exporter is already a11y-clean.

**Recommendation to ADR-003:** prefer HTML→PDF via headless browser if web preview architecture (ADR-008) already needs a browser runtime; this collapses two surfaces (HTML + PDF) into a single a11y verification problem.

---

## HTML Accessibility Requirements for Slide Content

The HTML exporter and the web preview both produce HTML, but they have different shapes:

- **HTML exporter:** static file, opened directly in a browser. No JS dependency.
- **Web preview:** SPA-ish, websocket-driven, canvas-based slide rendering.

**For both, the following are mandatory:**

1. **Semantic landmarks:** `<main>`, `<nav>` (deck navigation), `<header>` if applicable. Each slide region: `<section role="region" aria-labelledby="slide-N-title">`.
2. **Heading hierarchy:** `<h1>` for deck title (in static HTML) or document title; `<h2>` for slide titles; `<h3>` for in-slide section headers.
3. **Image alt text:** `<img alt="...">` from DSL.
4. **Decorative images:** `<img alt="" role="presentation">` or CSS background images.
5. **Tables:** `<table>` with `<thead><th scope="col">` headers; `<caption>` from slide title where relevant.
6. **Lists:** `<ul>` / `<ol>` / `<li>` — never visually-styled `<div>`s.
7. **Language:** `<html lang="en-US">` from DSL.
8. **Focus management:** logical tab order; `:focus-visible` styling with >= 3:1 outline.
9. **No keyboard trap:** especially in the web preview's modal panels.
10. **Live-region announcements:** parse errors, reload notifications via `aria-live="polite"` regions.
11. **Canvas-rendered slides (web preview):** because `<canvas>` is opaque to AT, the canvas renderer MUST emit a sibling DOM accessibility tree (an off-screen-positioned `<div>` containing the slide's text content in reading order, with headings/lists/etc.), OR render slides as accessible SVG with `<title>` / `<desc>` / `role="img"`. The latter is simpler — recommend SVG over canvas for the preview if AA conformance is non-negotiable.

> **Architectural implication for ADR-008 (canvas renderer choice):** A `<canvas>`-only slide renderer cannot be made WCAG AA compliant without a parallel DOM tree. SVG-based rendering with proper ARIA is the lower-friction path.

---

## Color Contrast Validation

slideforge's brand validator MUST verify every `(foreground, background)` color pair in the theme against WCAG 2.2 thresholds.

### Algorithm (WCAG 2.x relative luminance method)

For each color C with sRGB channels (R, G, B) in [0, 255]:

1. Normalize: `r = R/255`, `g = G/255`, `b = B/255`.
2. Linearize each channel: `c_lin = c / 12.92` if `c <= 0.03928`, else `c_lin = ((c + 0.055) / 1.055)^2.4`.
3. Relative luminance: `L = 0.2126 * R_lin + 0.7152 * G_lin + 0.0722 * B_lin`.
4. Contrast ratio between L1 (lighter) and L2 (darker): `ratio = (L1 + 0.05) / (L2 + 0.05)`.

### Thresholds (1.4.3 + 1.4.11)

- **Normal text:** ratio >= 4.5:1 (text < 18pt regular, < 14pt bold).
- **Large text:** ratio >= 3.0:1 (text >= 18pt regular OR >= 14pt bold).
- **Non-text (graphical objects, UI components):** ratio >= 3.0:1.

### What slideforge checks

| Pair | Threshold | Where |
|---|---|---|
| Body text fg vs slide bg | 4.5:1 | brand validator |
| Title text fg vs slide bg | 3.0:1 (qualifies as large text) | brand validator |
| Subtle text fg vs slide bg | 4.5:1 (unless explicitly tagged large) | brand validator |
| Chart series N color vs slide bg | 3.0:1 | brand validator |
| Chart series N color vs chart series N+1 color (adjacent in legend) | 3.0:1 | chart renderer |
| Severity badge fg vs severity badge bg | 4.5:1 | brand validator |
| Severity badge bg vs slide bg | 3.0:1 | brand validator |
| Progress-bar fill vs progress-bar track | 3.0:1 | brand validator |
| Focus outline vs adjacent colors | 3.0:1 | web preview UI |

The brand validator emits a **hard error** on any failure. It emits a **warning** on ratios within 10% of the threshold (designers should leave headroom).

---

## Color-Independence Rules for slideforge Slide Types

WCAG 1.4.1 prohibits color as the sole carrier of meaning. The following slide types are at risk and must enforce text/shape redundancy:

| Slide type | Color-encoded info | Required co-encoding |
|---|---|---|
| **severity_cards** | red/amber/green levels | text label ("High" / "Medium" / "Low") + shape glyph (square / triangle / circle) |
| **status** | green/yellow/red | text label + icon (check / warning / cross) |
| **progress_bar** | filled vs unfilled | numeric `"45%"` or `"3 of 7"` text |
| **weighted_composite** | category color blocks | category name printed in each block |
| **formula** | red highlight on changed variable | variable name + asterisk or underline |
| **chart slide types (any)** | series color | series direct label OR distinct marker shape per series |

**Lint rule:** if a slide of these types omits the redundancy (e.g., severity card with no `label` field), the linter emits a hard error.

---

## Animation / Transition Rules

WCAG criteria engaged:

- **2.3.1 Three Flashes or Below Threshold** (Level A): no content flashes > 3x/sec.
- **2.2.2 Pause, Stop, Hide** (Level A): user must be able to pause moving/updating content.
- **2.3.3 Animation from Interactions** (AAA — out of scope but tracked).

**Decisions for v1.0:**

1. **No slide transitions** are emitted in PPTX (`<p:transition>` is omitted). Rationale: any transition we ship must be verified non-flashing across PowerPoint, Keynote, Google Slides, LibreOffice — that verification work is post-v1.0.
2. **No build animations** are emitted (no `<p:timing>` with entrance/exit/emphasis effects).
3. **Web preview live-reload** is a moving/updating affordance under 2.2.2. The preview UI MUST include a **"Pause live-reload"** toggle, default off. Documented keyboard shortcut (e.g., `Ctrl+P`).
4. **Web preview slide navigation** (next/prev) is user-initiated only — no auto-advance.
5. **No video / audio embedding** in v1.0; defers media-related criteria (1.2.x).

---

## Tooling Comparison

Verified against tool documentation (cross-referenced from Deque, Pa11y docs, Google Lighthouse docs, PDF Association, axes4 PAC documentation, Microsoft Office documentation; see Sources).

| Tool | Surface | WCAG Coverage | Rule Count | Misses | Toolchain | Runtime cost |
|---|---|---|---|---|---|---|
| **axe-core (Deque)** | HTML | WCAG 2.1/2.2 A+AA (machine-testable subset; ~30-40% of WCAG issues) | ~96 rules | Alt-text quality, real-world keyboard UX, complex widget usability | Node.js / browser; integratable via Playwright (`@axe-core/playwright`), Cypress, Puppeteer | Low; runs per-page |
| **pa11y** | HTML | WCAG 2.x A+AA (depends on engine: axe-core or HTML_CodeSniffer) | Depends on engine; with axe ≈ same as axe-core | Same automated limits as axe | Node.js CLI; designed for CI | Low |
| **Lighthouse** | HTML | WCAG 2.x A+AA (subset of axe-core); also performance/SEO/best-practices | ~50 axe rules / 57 audits | Many axe rules absent → false sense of security; "score 100" possible while real a11y issues remain | Node.js / Chrome DevTools | Low-medium |
| **veraPDF** | PDF | PDF/UA-1 (ISO 14289-1) spec-level conformance | Spec checkpoint coverage | UX-level review (alt-text quality); does not assess "does it work for AT users" | Java; CLI-friendly | Low (CI-suitable) |
| **PAC 2024** | PDF | PDF/UA-1 + WCAG 2.1 AA machine-testable | Implements full Matterhorn Protocol (31 checkpoints, 136 failure conditions) | Manual UX review still required | Windows desktop only | Manual / interactive — NOT CI |
| **Adobe Acrobat Pro a11y checker** | PDF | PDF/UA + WCAG | Proprietary | Manual review required; commercial license | Adobe; manual | Manual |
| **Microsoft PowerPoint Accessibility Checker** | PPTX | Microsoft-defined rules loosely aligned to WCAG 2.x A/AA; covers <30% of relevant WCAG criteria | ~Several dozen rules | Color contrast, text spacing, reflow, animation safety, semantic heading hierarchy, link text quality, language tagging | PowerPoint desktop (Windows / Mac) | Manual — NOT CI-automatable |
| **office2pdf + downstream PDF checker** | PPTX→PDF | Depends on PDF checker run after | n/a | PPTX-specific issues are not validated | LibreOffice headless or similar | CI-suitable but indirect |
| **Custom OOXML linter (slideforge-owned)** | PPTX | Whatever we encode | Designer-defined | Whatever we don't encode | Rust (in-tree) | Low (built into emit) |

### Key overlaps and gaps

- **axe-core ⊇ Lighthouse:** Lighthouse runs a subset of axe rules. Choosing axe-core gives strictly more coverage on the same surface.
- **No tool covers PPTX in CI.** Microsoft Accessibility Checker is interactive desktop only. slideforge's emitter must enforce OOXML-level a11y invariants itself — and we have the ability to do that because we own the emitter. This is a **build-time enforcement** strategy, not a test-time validation strategy.
- **PDF/UA: veraPDF + PAC are complementary, not redundant.** veraPDF is automatable (CI) and spec-strict; PAC adds UX-oriented insight. Use veraPDF in CI gating, PAC as a per-release manual gate.

---

## Recommended Tooling Stack (input to ADR-011)

| Surface | CI Gate (per PR) | Release Gate (manual) | Build-time enforcement |
|---|---|---|---|
| Web preview UI | `@axe-core/playwright` against running preview, zero serious/critical | Manual screen-reader walk-through (NVDA + macOS VoiceOver) per release | Playwright a11y test in `slideforge-preview` crate's test harness |
| HTML exporter | `@axe-core/playwright` against static export of canonical decks | Same screen-reader walk-through | HTML emitter has type-level guards: every image carrier requires `alt`, lang propagation, etc. |
| PDF exporter | `veraPDF` CLI in CI; PDF/UA-1 conformance gate | `PAC 2024` on Windows VM per release on holdout deck | PDF emitter (or Chrome `--print-to-pdf` driver) emits tagged PDF from semantic source |
| PPTX exporter | Custom OOXML linter (Rust, in-tree) checks: every shape has `cNvPr@descr` (or is `decorative`); every slide has a title placeholder; reading order matches IR order; `lang` populated; table headers flagged; etc. | `Microsoft PowerPoint Accessibility Checker` (manual, on canonical holdout decks) | Type-level guards in emitter |

**ADR-011 decision recommendation:** **`@axe-core/playwright`** for web surfaces, **`veraPDF`** + **PAC 2024** for PDF, **custom OOXML linter + Microsoft Accessibility Checker (manual)** for PPTX. Rejected: Lighthouse (subset of axe); pa11y (functional duplicate of axe via Playwright runner we already need); Adobe Acrobat Pro (commercial licensing).

---

## NFRs (input to product-owner PRD)

- **a11y-NFR-1:** Web preview MUST pass `@axe-core/playwright` with zero violations of severity `serious` or `critical`. `moderate` / `minor` reported but not gating. Verified on every PR touching `slideforge-preview` crate or its UI assets.

- **a11y-NFR-2:** HTML exporter output MUST pass `@axe-core/playwright` against the canonical 23-slide-type sample deck with zero violations of severity `serious` or `critical`. Verified on every PR touching `slideforge-html`.

- **a11y-NFR-3:** PDF exporter output MUST be a tagged PDF passing `veraPDF` for PDF/UA-1 conformance on the canonical 23-slide-type sample deck. Verified on every PR touching `slideforge-pdf`. Per-release: PAC 2024 manual review on holdout deck set.

- **a11y-NFR-4:** PPTX exporter output MUST pass slideforge's custom OOXML lint rules (defined in `slideforge-pptx/a11y-lint.rs`) with zero errors. Per-release: Microsoft PowerPoint Accessibility Checker manual run on canonical sample deck, zero "Errors", "Warnings" reviewed and resolved or documented.

- **a11y-NFR-5:** All brand color combinations MUST satisfy WCAG 1.4.3 / 1.4.11 contrast thresholds (4.5:1 normal text, 3.0:1 large text, 3.0:1 non-text). Brand validator enforces at theme load time; rejects non-conformant themes.

- **a11y-NFR-6:** All slide types that encode meaning via color (severity_cards, status, progress_bar, weighted_composite, formula, all chart types) MUST also include text labels and/or distinct shape encoding. DSL linter enforces; chart renderer enforces marker-shape variation per series.

- **a11y-NFR-7:** All emitted output (PPTX, PDF, HTML, web preview) MUST carry a language tag (page-level minimum) propagated from DSL `lang` field (default `"en"`).

- **a11y-NFR-8:** Web preview live-reload MUST be pauseable via UI control and documented keyboard shortcut (WCAG 2.2.2).

- **a11y-NFR-9:** v1.0 MUST NOT emit slide transitions or build animations in any output format (WCAG 2.3.1 conformance via avoidance).

- **a11y-NFR-10:** Canvas-rendered slide content in web preview MUST be paired with a DOM accessibility tree (or use SVG with ARIA `<title>` / `<desc>` / `role="img"`).

---

## DSL Implications

The accessibility bar requires the DSL to expose accessibility primitives directly. Top-3 (and follow-on) requirements:

1. **`alt "..."` block** on every non-decorative visual element (image, icon, chart, diagram). Decorative variant: `decorative: true` → emits empty alt / `Artifact` in PDF. **Without this, the DSL cannot ship a11y-compliant decks.**

2. **`lang: "en-US"` top-level deck field** (default `"en"`) propagated to HTML root, PPTX presentation defaults, PDF `/Lang`. Per-run language tagging via inline annotation deferred to post-v1.0; v1.0 ships page-level lang only.

3. **Color-coded slide types REQUIRE text-label co-encoding.** DSL grammar for `severity_cards`, `status`, `progress_bar`, `weighted_composite`, `formula` requires a `label` (or equivalent) field; lint rejects color-only encodings. Chart blocks support `marker` (shape glyph) per series.

Additional DSL requirements:

4. **Slide titles are mandatory** — every slide block must have a `title` field, or be explicitly marked `subtitle_continuation: true` (continuation of previous slide's logical content). Lint rejects untitled slides.

5. **Tables declare headers explicitly** — first-row-header is the default but must be confirmable via `header_row: true | false`. Multi-header tables: header cells marked explicitly.

6. **Speaker notes lint** — DSL emits a warning if a `notes` block contains content that the lint heuristic flags as "essential" (e.g., the only place a referenced figure is described).

7. **Deck-level title** field (`title: "..."`) populates HTML `<title>`, PDF metadata `/Title`, PPTX `app.xml` Title. Not optional.

---

## Sources

All URLs as of 2026-05-23.

### W3C / WCAG

- WCAG 2.2 Recommendation: https://www.w3.org/TR/WCAG22/
- WCAG 2.2 Understanding documents: https://www.w3.org/WAI/WCAG22/Understanding/
- Techniques for WCAG 2.2: https://www.w3.org/WAI/WCAG22/Techniques/
- Specific Understanding pages cited inline in the criteria table above.
- WCAG 2.0 PDF Techniques (still authoritative for PDF mapping): https://www.w3.org/TR/WCAG20-TECHS/PDF1.html

### PDF/UA

- ISO 14289-1 (PDF/UA-1) overview (Wikipedia summary): https://en.wikipedia.org/wiki/PDF/UA
- PDF Association — Matterhorn Protocol description.
- DAISY Consortium accessible PDF guidance: https://daisy.org/guidance/info-help/guidance-training/content-creation/accessible-pdf/
- Section508.gov common PDF tags: https://www.section508.gov/create/pdfs/common-tags-and-usage/
- Quadient PAC overview: https://www.quadient.com/en/blog/free-pdf-accessibility-checker-pac
- axes4 PAC 2024 announcement: https://www.axes4.com/en/blog/post/2024/how-pac-conquered-the-world-of-accessibility
- veraPDF (open-source PDF/UA-1 validator): https://verapdf.org/

### Tooling

- axe-core (Deque): https://github.com/dequelabs/axe-core
- `@axe-core/playwright`: https://github.com/dequelabs/axe-core-npm/tree/develop/packages/playwright
- pa11y: https://pa11y.org/
- Google Lighthouse accessibility audits: https://developer.chrome.com/docs/lighthouse/accessibility/
- Comparison (axe vs Lighthouse rule counts): https://inclly.com/resources/axe-vs-lighthouse
- Comparison (CI/CD tooling): https://getboostra.com/blog/accessibility-testing-in-ci-cd-lighthouse-vs-axe-vs-pa11y

### Microsoft / OOXML

- Microsoft "Make your PowerPoint presentations accessible": https://support.microsoft.com/en-us/office/make-your-powerpoint-presentations-accessible-to-people-with-disabilities-6f7772b2-2f33-4bd2-8ca7-dae3b2b3ef25
- ISO/IEC 29500 (Office Open XML) — PresentationML schema (canonical reference for `cNvPr@descr`, `<p:spTree>`, `<p:ph type="title">`).

### Color blindness / data viz

- Wikipedia color blindness prevalence: https://en.wikipedia.org/wiki/Color_blindness
- Practical colorblind-friendly visualization patterns: https://mbounthavong.com/blog/2022/4/29/communicating-data-effectively-with-data-visualization-color-blind-friendly-palette

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | (1) WCAG 2.2 AA criteria most relevant to slide presentations + OOXML alt text / reading order / Microsoft Accessibility Checker scope; (2) tooling comparison (axe-core, pa11y, Lighthouse, PAC 2024, veraPDF, MS checker) — second call exceeded token limit and was read from saved output |
| Perplexity perplexity_ask | 3 | (1) pa11y vs axe-core vs Lighthouse with rule counts, plus PAC/veraPDF/MS checker summary; (2) PDF/UA-1 (ISO 14289-1) requirements and Matterhorn Protocol checkpoint structure; (3) WCAG 1.4.1 Use of Color exact requirement + color-blind-friendly viz patterns |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_reason | 0 | — |
| Context7 | 0 | — |
| Tavily tavily_search | 0 | — |
| Tavily tavily_research | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Read (project files) | 2 | Loaded STATE.md and partial output of large research blob |
| Glob | 1 | Confirmed `.factory/planning/` exists with prior planning artifacts |
| Training data | 1 area | OOXML schema details (`cNvPr@descr`, `<p:spTree>` semantics) — cross-validated against multiple Perplexity sources but the precise XML element names rely on Microsoft documentation we did not fetch directly. Implementer should validate against ISO/IEC 29500 PresentationML schema and the Python reference implementation in `.factory/seed/reference/` before encoding. Specifically: the `<a:descr>` element under `<a:blip>` cited in one survey response is suspect — there is no such element in standard schema; alt text on images flows through parent `cNvPr@descr`. Flagged in the OOXML section. |

**Total MCP tool calls:** 5 (2 deep research + 3 quick ask).
**Training data reliance:** medium — WCAG criteria and tooling capabilities are well-sourced from web; precise OOXML element names rely on cross-validation between Perplexity sources and prior knowledge of ISO/IEC 29500. Implementer should consult the schema directly when wiring the PPTX emitter, and one specific claim (`<a:descr>` under `<a:blip>`) is flagged as suspect inline.
