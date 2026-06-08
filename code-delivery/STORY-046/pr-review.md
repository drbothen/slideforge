# PR Review — PR #70 (STORY-046 Static HTML Exporter, WCAG AA)

**Verdict: APPROVE** (no blocking findings)

Fresh-eyes pre-merge review against the diff, PR description, and test evidence only.
Target: `develop`. ~7,690 insertions across the new `slideforge-html` crate plus
workspace wiring.

---

## Checklist results

| # | Item | Result |
|---|------|--------|
| 1 | Diff coherence | PASS — every change serves STORY-046 (new crate, workspace member move out of `exclude`, 2 shared deps + 2 dev-deps, registry registration + regression test). No unrelated edits. |
| 2 | Description accuracy | PASS — P4 model, ARIA `role="presentation"` outer SVG + `<g role="img">` children, 4-step heading chain, URL allowlist, SVG sanitizer, registry wiring all match the code. |
| 3 | Test coverage | PASS — 131 unit + 1 integration + 1 doctest. Tests assert CONTENT (escaped entity forms, exact h1/h2/h3 counts via `scraper`, role/title presence, exactly-one-warn counting), not mere presence. |
| 4 | Demo evidence | PASS — `evidence-report.md` + per-AC `.txt`/`.png` + axe-core JSON + 2 `.webm` browser sessions. (Behind info wall; presence verified from file list.) |
| 5 | Commit quality | N/A at review time (squash on merge). |
| 6 | Diff size | PASS WITH NOTE — large but cohesive: single crate, one story. ~80% of the volume is the in-module test suite (render.rs 4,435 lines, exporter.rs 2,905 lines, both dominated by `#[cfg(test)]`). No split warranted. |
| 7 | Missing changes | PASS — all AC-001..AC-010 paths present and tested incl. edge cases (empty deck, chart-only, table-only, empty-body, negative bbox, multi-title). |
| 8 | Dependency status | PASS — STORY-026/034/049 confirmed merged; HtmlExporter registered in `default_registry`. |

## What I verified in depth

- **P4 DOM model:** `render_slide_to_html` emits `<article position:relative>` with a real
  HTML text layer (`<h1>/<h2>/<p>/<ul>`) and a sibling `<svg role="presentation">` graphics
  layer. No `<canvas>`, no `<foreignObject>` — both asserted via `scraper` selectors.
- **ARIA model:** outer SVG carries `role="presentation"` (NOT `aria-hidden`), non-decorative
  frames wrap in `<g role="img" aria-labelledby>` + `<title>`, decoratives get per-`<g>`
  `aria-hidden="true"`. The WAI-ARIA ancestor-hiding rationale is correct and tested
  (`test_B2_outer_svg_has_role_presentation_not_aria_hidden`).
- **Heading chain (AC-008):** 4-step pre-pass in `compute_heading_levels` — title-type slide
  → first Title frame → first promotable text frame (warn) → synthetic visually-hidden h1
  from deck title (warn), with `""`/whitespace → "Presentation" fallback and empty-deck
  exception (zero h1). Subtitle = `title_level+1` capped at 6; subtitle-without-title → `<p>`.
  The non-degenerate-bbox guard is shared (`is_non_degenerate_bbox`, single source of truth,
  TD-VSDD-060) between the pre-pass and the render loop so the pre-pass never assigns h1 to a
  frame the loop would skip. This is the subtle bug class the 23-pass convergence closed, and
  it is correctly handled.
- **URL allowlist (AC-010 / CWE-601):** `is_safe_link_scheme` allows http/https/mailto/tel +
  relative paths, rejects javascript:/data:/vbscript:/protocol-relative `//`/backslash, with
  exactly one warn per rejection (asserted by count, not `>=1`). Xref routed through the same
  single enforcement point.
- **SVG sanitizer (F-005):** case-insensitive strip of `<script>`/`<foreignObject>` (with
  depth tracking for nested blocked subtrees), `on*` handlers, and `javascript:` href values;
  inner `<svg>` roots (incl. self-closing, F-007) get `aria-hidden`. On any parse/encode error
  the original string is returned with a `tracing::warn!` — no panic, no silent swallow.
- **Error handling:** zero production `.unwrap()`/`.expect()`/`println!`/`panic!`; `?` +
  structured `ExportError::RenderError`; `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`,
  `clippy::pedantic` all set.
- **Template:** `page.html.jinja` autoescapes via `.html` suffix; `synthetic_h1` is
  Rust-side `html_escape::encode_text`-escaped before the `| safe` filter, pinned by
  `test_B3_P5_synthetic_h1_xss_escaped` (CWE-79 regression guard).
- **CI workflow `html-wcag.yml`:** correct. Path-filtered to the crate; actions pinned to full
  SHAs; `html-wcag-check` gated on `html-unit-tests`; uses the correct `@axe-core/playwright`
  `AxeBuilder(...).withTags(['wcag2a','wcag2aa']).analyze()` API (NOT the axe-playwright
  injectAxe/checkA11y); fixture generated via `--include-ignored` at the exact path
  (`/tmp/slideforge-html-wcag-fixture.html`) the node script reads; success log prints a
  runtime-computed pass count so an inert run is greppably distinguishable.
- **Ignored tests (SID-1):** the two `#[ignore]`'d integration tests cite the blocking CI
  dependency (`@axe-core/playwright` + headless browser) in their attribute message, and the
  Rust-verifiable invariants are covered by non-ignored unit tests + the non-ignored
  `test_CRIT1_fixture_heading_order_no_level_skip`.

---

## Non-blocking suggestions

### [SUGGESTION] Dead `usvg` dependency — declared but never imported or invoked
`crates/slideforge-html/Cargo.toml:24` declares `usvg = { workspace = true }` (=0.47.0), and
the doc comments in `render.rs:77` and `:955-960` state "usvg is used for geometry
normalization and validation ONLY." However, `usvg` is **never imported or called anywhere**
in the crate — `grep -rn 'use usvg|usvg::'` returns nothing. All SVG handling is done with
`quick-xml`. The dependency is dead weight: it adds compile time and supply-chain surface
(against the project's pinned/minimal-deps bar) for zero functional benefit, and the doc
comments describe a usvg processing step that does not exist in this crate.

This is not a merge blocker (`unused_crate_dependencies` is not in the enforced clippy set, so
CI stays green), but it should be cleaned up: either remove the `usvg` dep and the doc
sentences that imply a usvg geometry pass, OR — if the intent is that STORY-047/048 will route
real chart/diagram SVG through usvg before `render_chart_frame` — drop it now and re-add it in
the story that actually calls it (the existing comments already say "WHEN real chart SVG is
added (STORY-047/048) it MUST route through render_chart_frame"). Suggest removing in-scope.

### [NIT] `render_svg_chart` is a legacy/secondary API kept for tests
`render.rs:1390` `render_svg_chart` is documented as the "legacy chart-SVG injection API
preserved for backward compatibility with tests that call it directly." It duplicates most of
the sanitizer logic in `sanitize_svg_for_graphics_layer`. It is `pub` and well-tested, so this
is fine for v1, but the duplication is a future maintenance cost — consider having
`render_svg_chart` delegate to the shared sanitizer in a follow-up to keep the two XSS-strip
paths from drifting.

---

## Conclusion
The implementation faithfully matches the PR description and AC-001..AC-010, including the
hard edge cases around the heading chain and the ARIA ancestor-hiding rule. Error handling is
sound, tests assert real content and counts, the CI gate is correctly wired, and the diff —
while large — is cohesive single-crate, single-story work whose bulk is its own test suite.
The only substantive item is a dead `usvg` dependency, which is non-blocking. **APPROVE.**
