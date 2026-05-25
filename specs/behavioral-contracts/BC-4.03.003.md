---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-017
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-4.03.003: Static HTML Output Passes WCAG AA via axe-core on CI

## Description

The static HTML exporter produces accessible HTML pages for each slide, passed through
`@axe-core/playwright` (or equivalent) on every CI run. The output must achieve zero
WCAG AA violations. All visual elements have alt text (from DSL), the HTML document has
a `lang` attribute from the deck declaration, headings are in correct hierarchical order,
and color contrasts meet 4.5:1 for normal text and 3:1 for large text. Charts and
diagrams are embedded as `<svg>` with `role="img"` and appropriate `<title>` elements.

## Preconditions

1. A valid `LaidOutDeck` IR exists with all accessibility fields populated (alt text,
   lang, color-coded labels).
2. Static HTML export is invoked (e.g., `slideforge build --format html`).
3. `@axe-core/playwright` is available in the CI environment.
4. Playwright is installed for headless browser accessibility scanning.

## Postconditions

1. A valid HTML file (or directory of per-slide HTML files) is produced.
2. `@axe-core/playwright` runs against the output and reports zero WCAG AA violations.
3. `<html lang="...">` attribute is set to the deck's declared language.
4. Every non-decorative image has a non-empty `alt` attribute.
5. Every decorative image has `alt=""` and `role="presentation"`.
6. Chart SVG elements have `role="img"` and a `<title>` element containing the chart's
   alt text.
7. Heading hierarchy (h1 → h2 → ...) is correct and non-skipped.
8. All body text meets 4.5:1 contrast ratio against background; large text meets 3:1.

## Invariants

1. axe-core is run in CI on every PR touching export code or templates — not just on
   release.
2. The HTML output uses SVG-based canvas rendering (not `<canvas>`) for slide visuals,
   preserving semantic structure for ARIA. (CLAUDE.md — "SVG-based canvas with ARIA")
3. The lang attribute is never hardcoded — it always derives from the deck `lang` field.
4. Zero WCAG AA violations is the hard gate; warnings (WCAG AAA suggestions) are
   informational only.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with only decorative images | All images have `alt="" role="presentation"`; axe-core: zero violations |
| EC-002 | Chart with complex nested SVG structure | `role="img"` on outer SVG; inner SVG elements are `aria-hidden="true"` |
| EC-003 | Deck with non-English lang "de" | `<html lang="de">`; axe-core detects no lang mismatch |
| EC-004 | Low-contrast brand color pair | E-A11-004 emitted at compile time (BC-3.03.001 area); blocked before HTML export |
| EC-005 | HTML output on Windows (CRLF line endings) | axe-core still passes; line endings are cosmetic |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3-slide deck with all alts, lang "en-US", passing contrast | axe-core exit 0; zero WCAG AA violations | happy-path (TV-12.1) |
| Chart slide with `alt "Revenue by quarter"` | SVG `<title>Revenue by quarter</title>` + `role="img"` | happy-path |
| Deck `lang "ja"` | `<html lang="ja">`; axe-core: no lang violation | edge-case |
| Deck with all-decorative slide | All images `alt="" role="presentation"`; axe-core passes | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | axe-core WCAG AA: zero violations on all fixture decks | CI: Playwright + axe-core on HTML output; assert violations.length === 0 |
| VP-TBD | All non-decorative images have non-empty alt attributes | integration test: parse HTML, count img[alt=""] vs img[alt!=""] |
| VP-TBD | html[lang] attribute matches deck lang declaration | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — "static HTML (WCAG AA via axe-core)" is verbatim from CAP-017 |
| L2 Domain Invariants | DI-003 (lang declaration propagates to HTML lang attr), DI-014 (referenced for PDF; HTML has analogous WCAG AA requirement) |
| Architecture Module | slideforge-html crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.004 — composes with (web preview is a live version of the same HTML renderer)
- BC-5.01.001 — depends on (alt text compile enforcement ensures no missing alt reaches HTML export)
- BC-5.01.005 — depends on (lang propagation to HTML lang attr is enforced by this BC)
- BC-4.03.001 — related to (PDF has analogous accessibility requirement via veraPDF)

## Architecture Anchors

- `architecture/export-subsystem.md#html` — static HTML export design
- `architecture/cross-cutting.md#accessibility-validation` — WCAG AA enforcement pipeline

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
