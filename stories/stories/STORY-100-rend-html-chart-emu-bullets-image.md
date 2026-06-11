---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-100
title: "REND-008-html: HTML chart SVG EMU/px fix + image src + bullet list semantics"
epic: EPIC-14
wave: 5
points: 5
priority: P0
tdd_mode: strict
status: draft
spec_version: "1.0"
created: "2026-06-11"
source_findings: [REND-008]
behavioral_contracts: [BC-4.03.003, BC-3.05.001]
# BC status: BC-4.03.003 (HTML WCAG AA — bullet items as <p> instead of <ul>/<li> is a
# WCAG 1.3.1 Info and Relationships failure; axe-core flags this). BC-3.05.001 (inline
# formats correct for HTML surface — EMU/px mismatch in chart SVG unit conversion is a
# rendering defect in the HTML exporter's SVG emission path).
# Image src empty: no BC covers image embedding in HTML yet (see STORY-101 for PPTX;
# HTML image src is a gap in the same area). PO action: confirm whether image src
# should reference the embedded file path, a data URI, or a relative asset path.
verification_properties: []
nfr_refs: []
closes_findings: [REND-008]
depends_on:
  - STORY-046
  - STORY-031
  - STORY-073
blocks: []
target_module: slideforge-html
subsystems: [SS-09]
estimated_days: 3
---

# STORY-100: REND-008-html — HTML Chart EMU/px Unit + Image src + Bullet List Semantics

## BC Gap Notice (image src)

No existing BC covers HTML image `src` attribute population. The HTML exporter emits an
empty `<img src="">` outline for image frames. Before AC-004 (image src) can be
implemented, the PO must confirm the intended v1.0 behavior:
(a) relative asset path referencing a co-located file, or
(b) inline data URI (base64-encoded image data).

Until PO confirms, AC-004 is stubbed as "emit `src=""` with a `data-deferred="true"`
attribute and a `tracing::warn!`" — this makes the deferral observable rather than
invisible. The story status remains `draft` pending that confirmation; all other ACs
(chart EMU fix, bullet list semantics) can proceed without it.

## Subsystem Anchor Justification

SS-09 (HTML Export) owns all three defects: SVG unit conversion coefficient, image src
population, and bullet list HTML semantics. All fixes are in `slideforge-html`.
EPIC-14 is the owning epic.

## Dependency Anchor Justifications

- `depends_on: [STORY-046]` — HTML WCAG AA exporter established here; this story fixes
  defects in the merged code.
- `depends_on: [STORY-031]` — Chart renderer produces SVG; the EMU/px coefficient error
  is in how the HTML exporter wraps that SVG in a viewport.
- `depends_on: [STORY-073]` — ContentBlock::Bullets; the HTML exporter must emit
  `<ul>/<li>` for these frames.

## Narrative

As a slideforge user, I want HTML output to render charts at the correct scale, display
bullet lists as semantic `<ul>/<li>` elements for screen reader accessibility, and
provide a clear observable placeholder for images rather than an invisible empty outline.

## Previous Story Intelligence

STORY-046 implemented the static HTML exporter and WCAG AA. STORY-031 produced the chart
SVG renderer. Post-merge defects in `slideforge-html`:

1. Chart SVG `viewBox` or container `width`/`height` uses EMU values directly instead of
   converting to pixels — rendering at ~0.004% scale (invisible).
2. Image `<img>` element emits with empty `src=""` — renders as a broken outline.
3. Bullet items emit as positioned `<p>` elements — semantic lists (`<ul>/<li>`) are
   required for WCAG 1.3.1.

## Architecture Compliance Rules

- Per BC-4.03.003 (HTML WCAG AA): bullet items as `<p>` violates WCAG 1.3.1 (lists must
  use `<ul>/<li>` or `<ol>/<li>`). axe-core will flag this in CI.
- EMU-to-pixel: 1 EMU = 1/914400 inches; at 96 DPI, 1 inch = 96px, therefore
  `px = emu / (914400 / 96) = emu / 9525`. Verify this coefficient against the
  EMU-to-PDF conversion in STORY-044 for consistency.
- Per ADR-008 (P4 Composite Rendering Model for HTML): SVG layers use
  `role="presentation"` not `aria-hidden`; non-decorative frames use
  `<g role="img" aria-labelledby>` with `<title>` children.
- No `<canvas>` in HTML output; no `<foreignObject>`.
- Image src deferral: if v1.0 behavior is data-URI, implement that. If relative path,
  implement that. No invisible empty `src=""`.

## Library & Framework Requirements

- No new dependencies.
- EMU conversion constant: `const EMU_TO_PX: f64 = 9525.0;` (verify against Cargo.lock
  units crate if one exists).

## File Structure Requirements

Files to modify:
- `crates/slideforge-html/src/render.rs` — fix EMU/px conversion in chart SVG wrapper;
  change bullet frame emission from `<p>` to `<ul>/<li>`; fix image src.
- `crates/slideforge-html/src/tests/` — add failing tests before fixes.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `crates/slideforge-html/src/render.rs` | ~4,000 |
| ADR-008 (Composite Rendering Model) reference | ~1,500 |
| Test files | ~2,000 |
| **Total** | **~9,500** |

## Acceptance Criteria

### AC-001: Chart SVG renders at correct scale in HTML output
(traces to BC-3.05.001 postcondition for HTML surface — chart rendered correctly)

A chart slide built with `slideforge build --format html` produces an HTML file where the
chart SVG container has `width` and `height` attributes expressed in CSS pixels (not EMUs).
A 9144000 EMU-wide frame maps to 960px (`9144000 / 9525 = 960`). The chart SVG is
visible at a reasonable viewport scale.

Verified by: unit test building a chart slide; parse the HTML; assert the SVG container
`width`/`height` attributes are in pixel units (numeric value ≤ 4000 for any standard
slide width, not millions).

### AC-002: EMU-to-pixel conversion coefficient is correct
(traces to BC-3.05.001 postcondition — rendering fidelity for HTML surface)

The conversion constant used in the HTML exporter for all EMU → CSS pixel conversions is
`emu / 9525`. A unit test explicitly validates this coefficient:
`assert_eq!(emu_to_px(Emu(9525)), 1.0)` and `assert_eq!(emu_to_px(Emu(914400)), 96.0)`.

Verified by: dedicated unit test for the conversion constant.

### AC-003: Bullet items emitted as semantic `<ul>/<li>` HTML structure
(traces to BC-4.03.003 — WCAG AA; WCAG 1.3.1 Info and Relationships)

A slide with bullet content produces HTML where bullets are wrapped in `<ul class="slide-bullets">`,
each bullet item in an `<li>` element — NOT in positioned `<p>` elements. The
`@axe-core/playwright` scan on the CI axe integration test continues to report zero WCAG
AA violations.

Verified by: unit test building a content slide with bullets; parse HTML; assert
`<ul>` wrapper and `<li>` items. Existing axe-core CI test must continue to pass.

### AC-004: Image frames emit observable placeholder (not silent empty src)
(traces to BC-4.03.003 — WCAG AA requires `alt` on img; empty src is an axe failure)

Image frames in HTML output emit `<img alt="<alt_text>" src="" data-image-embedding="pending-STORY-101">`
with a `data-*` attribute making the deferral machine-readable. The `alt` attribute is
populated from the frame's alt text (never empty for non-decorative). This is NOT a
silent drop — the element is visible and attributable to a specific story.

Verified by: unit test asserting `data-image-embedding` attribute present with story ID;
assert `alt` is non-empty.

### AC-005: axe-core CI integration test passes with zero new violations
(traces to BC-4.03.003 postcondition — zero WCAG AA violations)

After AC-001 through AC-004, the existing `@axe-core/playwright` CI test on the HTML
output reports zero WCAG AA violations. The bullet `<ul>/<li>` fix closes the 1.3.1
violation; the image alt fix closes the img alt violation.

Verified by: CI integration test green.

## Tasks

- [ ] **T-001 (RED):** Write `test_chart_svg_pixel_scale()` in `slideforge-html`.
- [ ] **T-002 (RED):** Write `test_emu_to_px_coefficient()`.
- [ ] **T-003 (RED):** Write `test_bullets_use_ul_li()`.
- [ ] **T-004 (RED):** Write `test_image_has_data_deferral_attribute()`.
- [ ] **T-005 (GREEN):** Fix EMU/px conversion in chart SVG container emission.
- [ ] **T-006 (GREEN):** Change bullet frame emission from `<p>` to `<ul>/<li>`.
- [ ] **T-007 (GREEN):** Fix image src emission to include `data-image-embedding` attribute
  citing STORY-101.
- [ ] **T-008:** Run `cargo nextest run -p slideforge-html --no-fail-fast`.
- [ ] **T-009:** Run `just check` before declaring done.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Slide with no bullets | No `<ul>` emitted; no regression |
| EC-002 | Nested bullet items | Nested `<ul>` inside `<li>` — proper nesting |
| EC-003 | Chart with 1 data point | Renders at correct scale; not invisible |
| EC-004 | Image with decorative: true | `alt=""` on img element; accessible opt-out per BC-5.01.002 |

## Behavioral Contracts Table

| BC ID | Title | Covering ACs |
|-------|-------|-------------|
| BC-4.03.003 | HTML WCAG AA | AC-003, AC-004, AC-005 |
| BC-3.05.001 | Inline formats correct per format | AC-001, AC-002 |

## Test Strategy

TDD strict mode. Four failing tests first. The EMU/px fix and bullet fix are independent
and straightforward. Image src deferral is an explicit observable placeholder, not a
silent drop.
