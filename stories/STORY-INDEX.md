---
document_type: story-index
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-25T00:00:00
phase: 2
traces_to:
  - .factory/stories/epics.md
  - .factory/stories/dependency-graph.md
  - .factory/stories/wave-schedule.md
total_stories: 77
stories_written: 77
stories_ready: 4
stories_in_progress: 0
stories_merged: 42
---

# STORY-INDEX — slideforge v1.0

> Authoritative index of all 77 stories across 6 waves and 21 epics.
> Update `status` and `merged_sha` fields as stories progress.
> Story files live in `.factory/stories/stories/STORY-NNN-[short].md`.
>
> Status values: `draft` | `ready` | `in-progress` | `merged` | `blocked`

---

## Progress Summary

| Wave | Total | Draft | Ready | In-Progress | Merged | Blocked |
|------|-------|-------|-------|-------------|--------|---------|
| Wave 1 | 14 | 0 | 0 | 0 | 14 | 0 |
| Wave 2 | 7 | 0 | 0 | 0 | 7 | 0 |
| Wave 3 | 17 | 0 | 0 | 0 | 17 | 0 |
| Wave 4 | 17 | 9 | 4 | 0 | 4 | 0 |
| Wave 5 | 16 | 16 | 0 | 0 | 0 | 0 |
| Wave 6 | 6 | 6 | 0 | 0 | 0 | 0 |
| **Total** | **77** | **25** | **4** | **0** | **42** | **0** |

---

## Wave 1: Foundation (14 stories)

| Story ID | Epic | Title | BCs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-001](stories/STORY-001-ir-core-types.md) | EPIC-01 | IR Core Types (Deck, Slide, Value, ContentBlock) | — | P0 | 5 | strict | merged |
| [STORY-002](stories/STORY-002-plugin-trait-api.md) | EPIC-01 | Plugin Trait API (all 10 surfaces) | BC-5.02.001, BC-5.02.002 | P0 | 5 | strict | merged |
| [STORY-003](stories/STORY-003-slide-type-impls.md) | EPIC-01 | 31 SlideType Implementations | BC-3.01.001-003 | P0 | 8 | strict | merged |
| [STORY-004](stories/STORY-004-value-system-emu.md) | EPIC-01 | Value System + EMU Types | BC-1.02.003 | P0 | 5 | strict | merged |
| [STORY-005](stories/STORY-005-lexer-tokenization.md) | EPIC-02 | Lexer + Mode-Based Tokenization | BC-1.01.003 | P0 | 5 | strict | merged |
| [STORY-006](stories/STORY-006-parser-core.md) | EPIC-02 | Parser Core: Deck, Slide, Fields, Indentation | BC-1.01.001, BC-1.01.002 | P0 | 8 | strict | merged |
| [STORY-007](stories/STORY-007-parser-control-flow.md) | EPIC-02 | Parser: @for, @if/@elif/@else, {{ expr }} | BC-1.04.001, BC-1.05.001 | P0 | 8 | strict | merged |
| [STORY-008](stories/STORY-008-parser-includes-variants.md) | EPIC-02 | Parser: @include, variants:, set rules, aliases | BC-1.01.004, BC-1.06.001+003, BC-1.07.001-005, BC-1.08.001-003, BC-1.09.001-002 | P0 | 8 | strict | merged |
| [STORY-009](stories/STORY-009-parser-math-shape-version.md) | EPIC-02 | Parser: math delimiters, shape:, DSL versioning, diagnostics | BC-1.01.005-006, BC-1.09.001, BC-1.13.001, BC-3.04.002 | P0 | 5 | strict | merged |
| [STORY-010](stories/STORY-010-error-accumulation-infra.md) | EPIC-02 | Error Accumulation + Diagnostic Infrastructure | BC-1.15.001, BC-1.15.002, BC-1.15.003 | P0 | 5 | strict | merged |
| [STORY-051](stories/STORY-051-ci-matrix-lint.md) | EPIC-19 | CI: fmt + clippy + nextest (5-platform matrix) | — | P0 | 5 | facade | merged |
| [STORY-052](stories/STORY-052-ci-visual-regression.md) | EPIC-19 | CI: Visual Regression (LibreOffice + SSIM/PSNR) | BC-4.01.002 | P0 | 5 | facade | merged |
| [STORY-053](stories/STORY-053-ci-supply-chain-audit.md) | EPIC-19 | CI: Supply-Chain Audit (cargo audit + deny + SBOM) | — | P0 | 5 | facade | merged |
| [STORY-054](stories/STORY-054-ci-release-pipeline.md) | EPIC-19 | CI: Release Pipeline + Signed Artifacts + Reproducible Builds | — | P0 | 8 | facade | merged |

**Wave 1 total points: 85**

---

## Wave 2: Evaluation + Validation (7 stories)

| Story ID | Epic | Title | BCs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-011](stories/STORY-011-expr-evaluator-core.md) | EPIC-03 | Expression Evaluator Core | BC-1.02.001, BC-1.02.002 | P0 | 8 | strict | merged |
| [STORY-012](stories/STORY-012-scoping-for-iteration.md) | EPIC-03 | Variable Scoping + @for Evaluation + Termination | BC-1.02.005, BC-1.04.001-003, BC-1.07.002, BC-1.08.001-003 | P0 | 8 | strict | merged |
| [STORY-013](stories/STORY-013-conditional-include-cycle.md) | EPIC-03 | @if/@elif/@else Evaluation + @include Cycle Detection | BC-1.05.001-002, BC-1.06.002 | P0 | 5 | strict | merged |
| [STORY-014](stories/STORY-014-type-system-no-coercion.md) | EPIC-03 | Type System: No Implicit Coercion + ${{ seq }} Disambiguation | BC-1.02.003, BC-1.02.004 | P0 | 5 | strict | merged |
| [STORY-015](stories/STORY-015-alt-text-enforcement.md) | EPIC-04 | Alt Text Enforcement | BC-5.01.001, BC-5.01.002 | P0 | 5 | strict | merged |
| [STORY-016](stories/STORY-016-canvas-overflow-validation.md) | EPIC-04 | Canvas Overflow + Zero-Slide + Strict/Warn-Only Mode | BC-3.03.001-004 | P0 | 5 | strict | merged |
| [STORY-017](stories/STORY-017-wcag-contrast-label-check.md) | EPIC-04 | Color-Coded Label + WCAG Contrast Enforcement | BC-5.01.003-005 | P0 | 5 | strict | merged |

**Wave 2 total points: 41**

---

## Wave 3: Core Services (17 stories)

| Story ID | Epic | Title | BCs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-018](stories/STORY-018-datasource-file-formats.md) | EPIC-05 | DataSource: JSON/CSV/YAML/TOML File Loading | BC-1.03.001, BC-1.03.003 | P0 | 5 | strict | merged |
| [STORY-019](stories/STORY-019-datasource-http-ssrf.md) | EPIC-05 | DataSource: HTTP/HTTPS + SSRF Allowlist | BC-1.03.002, BC-1.03.005 | P0 | 8 | strict | merged |
| [STORY-020](stories/STORY-020-datasource-excel-sqlite.md) | EPIC-05 | DataSource: Excel (.xlsx) + SQLite | BC-1.03.006, BC-1.03.007 | P0 | 5 | strict | merged |
| [STORY-021](stories/STORY-021-datasource-offline-errors.md) | EPIC-05 | DataSource: --offline Flag + Error Handling | BC-1.03.004 | P0 | 3 | strict | merged |
| [STORY-022](stories/STORY-022-brand-load-extraction.md) | EPIC-06 | Brand Loading: .pptx/.docx Template Extraction | BC-2.01.001, BC-2.01.006 | P0 | 5 | strict | merged |
| [STORY-023](stories/STORY-023-brand-synthesis-layouts.md) | EPIC-06 | Brand Synthesis: brand.toml → 31 Layouts + 12 OOXML Slots | BC-2.01.002, BC-2.01.004-005 | P0 | 13 | strict | merged |
| [STORY-024](stories/STORY-024-brand-extract-cli.md) | EPIC-06 | Brand Extraction CLI (slideforge extract-brand) | BC-2.01.003 | P0 | 3 | strict | merged |
| [STORY-025](stories/STORY-025-brand-overlay-per-slide.md) | EPIC-06 | Per-Slide brand_overlay: (No Master Switch Invariant) | BC-2.02.001-002 | P1 | 3 | strict | merged |
| [STORY-026](stories/STORY-026-layout-core-emu.md) | EPIC-07 | Core Layout: Deck → LaidOutDeck, EMU System | BC-3.06.001, BC-3.06.002, BC-3.06.003 | P0 | 8 | strict | merged |
| [STORY-027](stories/STORY-027-layout-docx-sections.md) | EPIC-07 | Layout: Document Section Generation (DOCX) | BC-3.02.001-002 | P0 | 5 | strict | merged |
| [STORY-028](stories/STORY-028-layout-shape-inline.md) | EPIC-07 | Layout: shape: Block + Rich Inline Formatting | BC-3.04.001, BC-3.05.001 | P1 | 5 | strict | merged |
| [STORY-029](stories/STORY-029-math-parser-omml.md) | EPIC-10 | Math Parser: $...$ / $$...$$ + @{var} + OMML Output | BC-1.10.001-003 | P1 | 8 | strict | merged |
| [STORY-030](stories/STORY-030-math-mathml-pdf.md) | EPIC-10 | Math: MathML (HTML) + Path-Based (PDF) Output | BC-1.10.003 | P1 | 5 | strict | merged |
| [STORY-031](stories/STORY-031-chart-renderer-core.md) | EPIC-11 | Chart Renderer: bar/line/pie/scatter/area/histogram/stacked-bar | BC-1.11.001 | P1 | 8 | strict | merged |
| [STORY-032](stories/STORY-032-chart-empty-data.md) | EPIC-11 | Chart: Empty Data Error-Slide Placeholder | BC-1.11.002 | P1 | 3 | strict | merged |
| [STORY-033](stories/STORY-033-diagram-mermaid-render.md) | EPIC-12 | Diagram Renderer: Mermaid → PPTX-Safe SVG | BC-1.12.001-002 | P1 | 8 | strict | merged |
| [STORY-034](stories/STORY-034-diagram-svg-normalize-perf.md) | EPIC-12 | SVG Normalization via usvg + Performance Gate | BC-1.12.003 | P1 | 5 | strict | merged |

**Wave 3 total points: 100**

---

## Wave 4: Exporters + Registry (16 stories)

| Story ID | Epic | Title | BCs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-035](stories/STORY-035-register-routing-eval.md) | EPIC-18 | Writing Register Routing in Evaluator | BC-1.14.001-004 | P0 | 5 | strict | merged |
| [STORY-036](stories/STORY-036-register-no-bleed-invariant.md) | EPIC-18 | Register-Aware Rendering: No Content Bleed Invariant | BC-1.14.004 | P0 | 5 | strict | merged |
| [STORY-037](stories/STORY-037-pptx-core-serialization.md) | EPIC-08 | PPTX Core Serialization: ooxmlsdk 0.6.1 + ZIP Assembly | BC-4.01.001 | P0 | 13 | strict | draft |
| [STORY-038](stories/STORY-038-pptx-layout-compliance.md) | EPIC-08 | PPTX Layout Compliance: Placeholder Inheritance + Slide IDs + Layouts | BC-4.01.005 | P0 | 8 | strict | draft |
| [STORY-039](stories/STORY-039-pptx-a11y-metadata.md) | EPIC-08 | PPTX Accessibility Metadata | BC-4.01.004, BC-5.01.005 | P0 | 5 | strict | draft |
| [STORY-040](stories/STORY-040-pptx-notes-sections-masters.md) | EPIC-08 | PPTX: Speaker Notes + Slide Sections + notesMaster1.xml | BC-4.01.003, BC-4.01.006 | P0 | 5 | strict | draft |
| [STORY-041](stories/STORY-041-docx-core-serialization.md) | EPIC-09 | DOCX Core Serialization: report register + ooxmlsdk | BC-4.02.001 | P0 | 8 | strict | draft |
| [STORY-042](stories/STORY-042-docx-auto-sections.md) | EPIC-09 | DOCX: Auto-Generated Document Sections | BC-4.02.002 | P0 | 5 | strict | draft |
| [STORY-043](stories/STORY-043-pdf-core-backend.md) | EPIC-13 | PDF Core: pdf-writer + krilla + SlideTagEngine | BC-4.03.002 | P0 | 8 | strict | merged |
| [STORY-044](stories/STORY-044-pdf-coordinate-mapping.md) | EPIC-13 | PDF: EMU-to-PDF Coordinate Mapping + Y-Axis Flip | BC-4.03.005 | P0 | 5 | strict | merged |
| [STORY-045](stories/STORY-045-pdf-ua1-tagging-verapdf.md) | EPIC-13 | PDF: PDF/UA-1 Tagging + veraPDF CI Gate | BC-4.03.001 | P0 | 5 | strict | draft |
| [STORY-049](stories/STORY-049-plugin-registry-assembly.md) | EPIC-21 | Plugin Registry Assembly (root crate) | BC-5.02.001-002 | P0 | 5 | strict | draft |
| [STORY-050](stories/STORY-050-e2e-integration-tests.md) | EPIC-21 | End-to-End Integration Test Suite | BC-5.02.001-002 | P0 | 8 | strict | draft |
| [STORY-073](stories/STORY-073-bullets-layout.md) | EPIC-07 | Layout: ContentBlock::Bullets → FrameContent::TextRun frame generation | BC-3.05.001 | P1 | 5 | strict | ready |
| [STORY-075](stories/STORY-075-brand-loader-footer-detection.md) | EPIC-06 | Brand Loader: Footer Detection from .pptx Slide Master/Layout Placeholders | BC-2.01.001 | P1 | 3 | strict | ready |
| [STORY-076](stories/STORY-076-brand-srgbclr-transform-extraction.md) | EPIC-06 | Brand Loader: Transform-Aware Theme Color Extraction (srgbClr lumMod/tint/shade) | BC-2.01.001, BC-2.01.003 | P1 | 3 | strict | ready |
| [STORY-077](stories/STORY-077-section-block-ir-extension.md) | EPIC-18 | SectionBlock IR Extension: FieldValue body + section-level register routing | BC-3.02.002, BC-1.14.003 | P0 | 8 | strict | ready |

**Wave 4 total points: 104**

---

## Wave 5: CLI + User-Facing Features + Deferred Surfaces (16 stories)

| Story ID | Epic | Title | BCs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-046](stories/STORY-046-html-exporter-wcag.md) | EPIC-14 | Static HTML Exporter: WCAG AA via axe-core | BC-4.03.003 | P0 | 8 | strict | draft |
| [STORY-047](stories/STORY-047-preview-server-websocket.md) | EPIC-14 | Web Preview Server: axum + WebSocket + SVG Canvas | BC-4.03.004 | P1 | 8 | strict | draft |
| [STORY-048](stories/STORY-048-preview-live-reload-a11y.md) | EPIC-14 | Web Preview: Live Reload + Accessibility | BC-5.05.005 | P1 | 8 | strict | draft |
| [STORY-055](stories/STORY-055-cli-build-command.md) | EPIC-15 | CLI: build command + miette error rendering | BC-1.15.001-003 | P0 | 8 | strict | draft |
| [STORY-056](stories/STORY-056-cli-watch-mode.md) | EPIC-15 | CLI: watch mode + notify integration + incremental rebuild | BC-5.05.001-004 | P0 | 8 | strict | draft |
| [STORY-057](stories/STORY-057-cli-init-extract-brand.md) | EPIC-15 | CLI: init scaffolding + extract-brand command | BC-5.06.001-002 | P0 | 5 | strict | draft |
| [STORY-058](stories/STORY-058-cli-flags-nocolor.md) | EPIC-15 | CLI: --json/--quiet/--verbose flags + NO_COLOR + non-TTY | BC-1.15.001 | P0 | 5 | strict | draft |
| [STORY-059](stories/STORY-059-cli-criterion-benchmarks.md) | EPIC-15 | CLI: Criterion performance benchmarks | — | P0 | 5 | facade | draft |
| [STORY-060](stories/STORY-060-pkg-install-lock.md) | EPIC-16 | Package: install + sf.lock + SHA-256 integrity | BC-5.03.001, BC-5.03.003 | P1 | 8 | strict | draft |
| [STORY-061](stories/STORY-061-pkg-import-resolution.md) | EPIC-16 | Package: @import resolution in evaluator | BC-5.03.002, BC-1.06.004 | P1 | 5 | strict | draft |
| [STORY-062](stories/STORY-062-pkg-remove-list.md) | EPIC-16 | Package: remove + list | BC-5.03.004-005 | P1 | 3 | strict | draft |
| [STORY-063](stories/STORY-063-pkg-verify-integrity.md) | EPIC-16 | Package: verify (SHA-256 checksum audit) | BC-5.03.006 | P1 | 3 | strict | draft |
| [STORY-064](stories/STORY-064-workspace-build.md) | EPIC-17 | Workspace: slideforge.toml [workspace] + build --workspace | BC-5.04.001 | P1 | 5 | strict | draft |
| [STORY-065](stories/STORY-065-workspace-sfconfig-explain.md) | EPIC-17 | Workspace: .sfconfig cascade + config explain provenance | BC-5.04.002-003 | P1 | 5 | strict | draft |
| [STORY-072](stories/STORY-072-shape-gradient-fills.md) | EPIC-07 | shape: Gradient Fills (FillSpec::Gradient) | BC-3.04.001 | P2 | 3 | strict | draft |
| [STORY-074](stories/STORY-074-brand-em-sizing.md) | EPIC-07 | Brand-aware Em conversion (font_size_emu) | BC-3.04.001 | P2 | 3 | strict | draft |

**Wave 5 total points: 90**

---

## Wave 6: Formal Verification (6 stories)

| Story ID | Epic | Title | VPs | Priority | Points | tdd_mode | Status |
|----------|------|-------|-----|---------|--------|---------|--------|
| [STORY-066](stories/STORY-066-kani-syntax.md) | EPIC-20 | Kani Proofs: slideforge-syntax (VP-001, VP-003, VP-009, VP-014) | VP-001, VP-003, VP-009, VP-014 | P0 | 8 | facade | draft |
| [STORY-067](stories/STORY-067-kani-eval.md) | EPIC-20 | Kani Proofs: slideforge-eval (VP-004, VP-005, VP-010, VP-015) | VP-004, VP-005, VP-010, VP-015 | P0 | 8 | facade | draft |
| [STORY-068](stories/STORY-068-kani-validate-pdf.md) | EPIC-20 | Kani Proofs: slideforge-validate + slideforge-pdf (VP-002, VP-007, VP-008) | VP-002, VP-007, VP-008 | P0 | 8 | facade | draft |
| [STORY-069](stories/STORY-069-proptest-brand-pptx.md) | EPIC-20 | Proptest Suites: brand (VP-012) + pptx (VP-013) + layout (VP-011) | VP-011, VP-012, VP-013 | P0 | 5 | facade | draft |
| [STORY-070](stories/STORY-070-kani-layout-pdf.md) | EPIC-20 | Kani Proofs: slideforge-pdf (VP-006) | VP-006 | P0 | 5 | facade | draft |
| [STORY-071](stories/STORY-071-fuzz-mutants.md) | EPIC-20 | Fuzz Harnesses + cargo-mutants Integration | VP-014, VP-015 | P0 | 8 | facade | draft |

**Wave 6 total points: 42**

---

## Wave TBD: Deferred Surfaces (0 stories)

All previously Wave TBD stories have been assigned waves per human approval 2026-05-31:
- STORY-073, STORY-075, STORY-076 → Wave 4 (pulled in as P1 follow-ups)
- STORY-072, STORY-074 → Wave 5 (deferred P2 surfaces)

**Wave TBD total points: 0**

---

## Story Points Summary

| Wave | Stories | Total Points | Avg Points/Story |
|------|---------|-------------|-----------------|
| Wave 1 | 14 | 85 | 6.1 |
| Wave 2 | 7 | 41 | 5.9 |
| Wave 3 | 17 | 100 | 5.9 |
| Wave 4 | 17 | 104 | 6.1 |
| Wave 5 | 16 | 90 | 5.6 |
| Wave 6 | 6 | 42 | 7.0 |
| **Total** | **77** | **462** | **6.0** |

> No story exceeds 13 points. STORY-023 (brand synthesis — 13 pts) and STORY-037
> (PPTX core serialization — 13 pts) are the largest stories. Both have well-
> defined scope that warrants their size and were not split further to preserve
> coherent implementation units. All other stories are 3-8 points.
>
> Wave 4 expanded from 13 → 16 stories (85 → 96 pts) per human approval 2026-05-31:
> STORY-073 (bullets-layout, 5 pts, P1), STORY-075 (footer detection, 3 pts, P1),
> STORY-076 (srgbClr transform extraction, 3 pts, P1) pulled in from Wave TBD.
> BC-2.01.001 EC-006 + BC-2.01.003 EC-003 widening landed by PO; STORY-076
> implements Option B (store base hex + is_derived flag + tracing::warn!; HSL deferred v2).
>
> Wave 4 further expanded from 16 → 17 stories (96 → 104 pts) per architect directive
> F-002 (2026-05-31): STORY-077 (SectionBlock IR extension + section-level register
> routing, 8 pts, P0) spun out from STORY-035 descope. Required before STORY-042
> renders section-level DOCX content.
>
> Wave 5 expanded from 14 → 16 stories (84 → 90 pts) per same approval:
> STORY-072 (gradient fills, 3 pts, P2) and STORY-074 (brand-em-sizing, 3 pts, P2)
> assigned to Wave 5 (require exporters from Wave 4 to be complete before dispatch).

---

## BC Coverage Verification

| Metric | Count | Status |
|--------|-------|--------|
| Total BCs | 112 | — |
| BCs covered by at least one story | 112 | PASS — 100% (BC-3.02.002 now covered by STORY-027 + STORY-077; BC-1.14.003 now covered by STORY-035 + STORY-077) |
| Orphan BCs (no story) | 0 | PASS |
| Total VPs | 15 | — |
| VPs covered | 15 | PASS — 100% |
| Total NFRs | 35 | — |
| NFRs covered | 35 | PASS — 100% |
| Domain Edge Cases (DEC-NNN) | 20 | — |
| DECs covered | 20 | PASS — 100% |

---

## Quick-Reference: Story by BC

For fast BC-to-story lookup, see `dependency-graph.md §BC-to-Stories Traceability Matrix`.
For VP-to-story lookup, see `dependency-graph.md §VP-to-Stories Traceability Matrix`.
For NFR-to-story lookup, see `dependency-graph.md §NFR-to-Stories Traceability Matrix`.

---

## Dispatch Order for Individual Story Writing

Stories should be written in wave order. Within each wave, they can be written
in any order (all have the same prerequisites). Suggested batch sizes per burst
to stay within context budget:

- **Burst A:** STORY-001 through STORY-005 (Wave 1, EPIC-01 + start EPIC-02)
- **Burst B:** STORY-006 through STORY-010 (Wave 1, EPIC-02 complete)
- **Burst C:** STORY-051 through STORY-054 (Wave 1, EPIC-19 CI stories)
- **Burst D:** STORY-011 through STORY-017 (Wave 2, EPIC-03 + EPIC-04)
- **Burst E:** STORY-018 through STORY-025 (Wave 3, EPIC-05 + EPIC-06)
- **Burst F:** STORY-026 through STORY-034 (Wave 3, EPIC-07 + EPIC-10-12)
- **Burst G:** STORY-035 through STORY-042 (Wave 4, EPIC-18 + EPIC-08 + EPIC-09)
- **Burst H:** STORY-043 through STORY-045, STORY-049, STORY-050 (Wave 4, EPIC-13 + EPIC-21)
- **Burst I:** STORY-046 through STORY-048 + STORY-055 through STORY-059 (Wave 5, EPIC-14 + EPIC-15)
- **Burst J:** STORY-060 through STORY-065 (Wave 5, EPIC-16 + EPIC-17)
- **Burst K:** STORY-066 through STORY-071 (Wave 6, EPIC-20)
