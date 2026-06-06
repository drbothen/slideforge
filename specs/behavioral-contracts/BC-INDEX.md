---
document_type: behavioral-contract-index
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
traces_to: domain-spec/L2-INDEX.md
---

# Behavioral Contract Index — slideforge

> Master registry for all BC-S.SS.NNN contracts.
> Subsystem IDs are `SS-TBD` until the architect produces ARCH-INDEX.md in Phase 1b.
> BCs are grouped by L2 bounded context (not by implementation module).
> Filename slugs are immutable per append_only_numbering policy.

---

## Section 1: Authoring Bounded Context

> Capabilities: CAP-001 through CAP-009, CAP-012, CAP-013, CAP-014, CAP-028, CAP-029, CAP-030

### 1.01 — DSL Source Parsing (CAP-001)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.01.001 | Parse valid .sf source into typed AST with error accumulation | P0 | active | [BC-1.01.001.md](BC-1.01.001.md) |
| BC-1.01.002 | Reject indentation inconsistency with file:line:col span | P0 | active | [BC-1.01.002.md](BC-1.01.002.md) |
| BC-1.01.003 | Reject tab indentation with hard error | P0 | active | [BC-1.01.003.md](BC-1.01.003.md) |
| BC-1.01.004 | Resolve @include directives and detect cycles (Fail-Closed) | P0 | active | [BC-1.01.004.md](BC-1.01.004.md) |
| BC-1.01.005 | Reject reserved keywords with named-feature error message | P0 | active | [BC-1.01.005.md](BC-1.01.005.md) |
| BC-1.01.006 | Reject variable names colliding with slide type keywords | P0 | active | [BC-1.01.006.md](BC-1.01.006.md) |

### 1.02 — Variable Interpolation and Expression Evaluation (CAP-002)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.02.001 | Evaluate {{ expr }} interpolation with arithmetic and pipe filters | P0 | active | [BC-1.02.001.md](BC-1.02.001.md) |
| BC-1.02.002 | Reject undefined variable reference with scope path | P0 | active | [BC-1.02.002.md](BC-1.02.002.md) |
| BC-1.02.003 | No implicit type coercion — values are never silently converted | P0 | active | [BC-1.02.003.md](BC-1.02.003.md) |
| BC-1.02.004 | Handle ${{ seq }} as text interpolation, not math delimiter | P0 | active | [BC-1.02.004.md](BC-1.02.004.md) |
| BC-1.02.005 | Lexical scoping in nested @for loops — outer vars remain in scope | P0 | active | [BC-1.02.005.md](BC-1.02.005.md) |

### 1.03 — Data Binding from External Sources (CAP-003)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.03.001 | Load @data from JSON/CSV/YAML/TOML file at compile time | P0 | active | [BC-1.03.001.md](BC-1.03.001.md) |
| BC-1.03.002 | Load @data from HTTP/HTTPS URL at compile time | P0 | active | [BC-1.03.002.md](BC-1.03.002.md) |
| BC-1.03.003 | Fail with structured error on missing field access | P0 | active | [BC-1.03.003.md](BC-1.03.003.md) |
| BC-1.03.004 | Support --offline flag to skip HTTP sources | P1 | active | [BC-1.03.004.md](BC-1.03.004.md) |
| BC-1.03.005 | HTTP Domain Allowlist Enforcement (SSRF Prevention) | P1 | active | [BC-1.03.005.md](BC-1.03.005.md) |
| BC-1.03.006 | Load @data from Excel (.xlsx) spreadsheet at compile time | P1 | active | [BC-1.03.006.md](BC-1.03.006.md) |
| BC-1.03.007 | Load @data from SQLite database via parameterized query at compile time | P1 | active | [BC-1.03.007.md](BC-1.03.007.md) |

### 1.04 — Iteration over Data Collections (CAP-004)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.04.001 | @for over finite collection generates slides/blocks per item | P0 | active | [BC-1.04.001.md](BC-1.04.001.md) |
| BC-1.04.002 | @for over empty collection generates zero items without error | P0 | active | [BC-1.04.002.md](BC-1.04.002.md) |
| BC-1.04.003 | Reject any construct that could produce non-termination | P0 | active | [BC-1.04.003.md](BC-1.04.003.md) |

### 1.05 — Conditional Rendering (CAP-005)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.05.001 | @if/@elif/@else at slide, element, field, and section scope | P0 | active | [BC-1.05.001.md](BC-1.05.001.md) |
| BC-1.05.002 | Conditional expression evaluates with same type rules as {{ expr }} | P0 | active | [BC-1.05.002.md](BC-1.05.002.md) |

### 1.06 — Multi-File Composition (CAP-006)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.06.001 | @include resolves local .sf file and inlines at parse time | P0 | active | [BC-1.06.001.md](BC-1.06.001.md) |
| BC-1.06.002 | Detect and reject circular @include chains (Fail-Closed) | P0 | active | [BC-1.06.002.md](BC-1.06.002.md) |
| BC-1.06.003 | @include with variable path reports resolved path on error | P0 | active | [BC-1.06.003.md](BC-1.06.003.md) |
| BC-1.06.004 | @import resolves to installed package or fails with install hint | P1 | active | [BC-1.06.004.md](BC-1.06.004.md) |

### 1.07 — Variant-Based Deck Segmentation (CAP-007)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.07.001 | variants: block with include_tags/exclude_tags produces filtered deck | P0 | active | [BC-1.07.001.md](BC-1.07.001.md) |
| BC-1.07.002 | Variant vars: overrides deck-level vars per 11-level precedence chain | P0 | active | [BC-1.07.002.md](BC-1.07.002.md) |
| BC-1.07.003 | Detect and reject cyclic variant inheritance graph (Fail-Closed) | P0 | active | [BC-1.07.003.md](BC-1.07.003.md) |
| BC-1.07.004 | Reject --variant flag referencing undefined variant with list of defined | P0 | active | [BC-1.07.004.md](BC-1.07.004.md) |
| BC-1.07.005 | Variant that excludes all slides produces warning in warn-only mode | P0 | active | [BC-1.07.005.md](BC-1.07.005.md) |

### 1.08 — Set Rules for Slide-Type Defaults (CAP-008)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.08.001 | set <type>: <field> <value> applies as default to all instances of type | P0 | active | [BC-1.08.001.md](BC-1.08.001.md) |
| BC-1.08.002 | set rules support {{ }} interpolation and brand.* references | P0 | active | [BC-1.08.002.md](BC-1.08.002.md) |
| BC-1.08.003 | brand.* references in set rules are evaluated after brand loading | P0 | active | [BC-1.08.003.md](BC-1.08.003.md) |

### 1.09 — Parametric Slide-Type Aliases (CAP-009)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.09.001 | alias <name> = <type>: defines named preset that resolves at parse time | P1 | active | [BC-1.09.001.md](BC-1.09.001.md) |
| BC-1.09.002 | Alias cannot add new fields — only preset existing fields | P1 | active | [BC-1.09.002.md](BC-1.09.002.md) |

### 1.10 — Math and LaTeX Rendering (CAP-012)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.10.001 | $...$ and $$...$$ toggle math mode with @{var} interpolation | P1 | active | [BC-1.10.001.md](BC-1.10.001.md) |
| BC-1.10.002 | Unsupported LaTeX command produces error with source span and hint | P1 | active | [BC-1.10.002.md](BC-1.10.002.md) |
| BC-1.10.003 | Math renders to OMML for PPTX/DOCX, MathML for HTML, paths for PDF | P1 | active | [BC-1.10.003.md](BC-1.10.003.md) |

### 1.11 — Chart Rendering from Data (CAP-013)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.11.001 | slide chart: renders bar/line/pie/scatter/area/histogram/stacked-bar as SVG | P1 | active | [BC-1.11.001.md](BC-1.11.001.md) |
| BC-1.11.002 | Chart with empty data produces error-slide placeholder not a crash | P1 | active | [BC-1.11.002.md](BC-1.11.002.md) |

### 1.12 — Diagram Rendering / Mermaid (CAP-014)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.12.001 | slide diagram: renders Mermaid source to PPTX-safe SVG via mermaid-rs-renderer | P1 | active | [BC-1.12.001.md](BC-1.12.001.md) |
| BC-1.12.002 | Invalid Mermaid syntax produces compile error with line number within source | P1 | active | [BC-1.12.002.md](BC-1.12.002.md) |
| BC-1.12.003 | SVG normalized via usvg before PPTX embedding (no foreignObject, absolute dims) | P1 | active | [BC-1.12.003.md](BC-1.12.003.md) |

### 1.13 — DSL Versioning (CAP-028)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.13.001 | slideforge_version "1" required in deck metadata; reject forward-incompatible | P1 | active | [BC-1.13.001.md](BC-1.13.001.md) |

### 1.14 — Writing Register Support (CAP-029)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.14.001 | notes register routes to presenter notes in PPTX/DOCX/HTML only | P0 | active | [BC-1.14.001.md](BC-1.14.001.md) |
| BC-1.14.002 | report register routes to DOCX body; excluded from PPTX slide content | P0 | active | [BC-1.14.002.md](BC-1.14.002.md) |
| BC-1.14.003 | detail register routes to DOCX/PDF only; excluded from PPTX and web preview | P0 | active | [BC-1.14.003.md](BC-1.14.003.md) | v1.3 |
| BC-1.14.004 | No register content bleeds to wrong format | P0 | active | [BC-1.14.004.md](BC-1.14.004.md) | v1.2 (DI-2 hygiene; SS-02 + story anchors filled; stories: STORY-035, STORY-036) |

### 1.15 — Diagnostic Reporting with Source Spans (CAP-030)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-1.15.001 | All errors carry file:line:col span and correction hint | P0 | active | [BC-1.15.001.md](BC-1.15.001.md) |
| BC-1.15.002 | All errors accumulated in single pass — no fail-on-first | P0 | active | [BC-1.15.002.md](BC-1.15.002.md) |
| BC-1.15.003 | Parse errors are always fatal; validation errors fatal in strict mode only | P0 | active | [BC-1.15.003.md](BC-1.15.003.md) |

---

## Section 2: Branding Bounded Context

> Capabilities: CAP-018, CAP-019

### 2.01 — Brand Template Loading and Synthesis (CAP-018)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-2.01.001 | Load brand from existing .pptx or .docx template file | P0 | active | [BC-2.01.001.md](BC-2.01.001.md) |
| BC-2.01.002 | Synthesize complete brand template from brand.toml (all 12 OOXML slots) | P0 | active | [BC-2.01.002.md](BC-2.01.002.md) |
| BC-2.01.003 | Extract brand.toml from existing .pptx via slideforge extract-brand | P0 | active | [BC-2.01.003.md](BC-2.01.003.md) |
| BC-2.01.004 | brand.toml missing color slots — synthesize with derived defaults and warn | P0 | active | [BC-2.01.004.md](BC-2.01.004.md) |
| BC-2.01.005 | Brand synthesizer produces all 31 layouts (11 standard + 20 custom) | P0 | active | [BC-2.01.005.md](BC-2.01.005.md) |
| BC-2.01.006 | Font unavailable on build host produces warning and uses fallback | P0 | active | [BC-2.01.006.md](BC-2.01.006.md) |

### 2.02 — Per-Slide Brand Overlay (CAP-019)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-2.02.001 | brand_overlay: overrides logo/footer/confidentiality on individual slide | P1 | active | [BC-2.02.001.md](BC-2.02.001.md) |
| BC-2.02.002 | brand_overlay: does not switch slide master (single-master architecture) | P1 | active | [BC-2.02.002.md](BC-2.02.002.md) |

---

## Section 3: Layout Bounded Context

> Capabilities: CAP-010, CAP-011, CAP-022, CAP-023, CAP-024

### 3.01 — 31 Built-in Slide Types (CAP-010)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.01.001 | Each slide type enforces required fields via SlideType trait | P0 | active | [BC-3.01.001.md](BC-3.01.001.md) |
| BC-3.01.002 | Missing required field on slide type produces compile error with field name | P0 | active | [BC-3.01.002.md](BC-3.01.002.md) |
| BC-3.01.003 | Unknown slide type keyword produces compile error with suggestion | P0 | active | [BC-3.01.003.md](BC-3.01.003.md) |

### 3.02 — Document Section Generation (CAP-011)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.02.001 | Auto-generated DOCX sections derived from slide data (executive_summary, risk_register) | P0 | active | [BC-3.02.001.md](BC-3.02.001.md) |
| BC-3.02.002 | Manually authored section blocks (section methodology:) appear in DOCX/PDF | P0 | active | [BC-3.02.002.md](BC-3.02.002.md) | v1.5 |

### 3.03 — Compile-Time Content Validation (CAP-022)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.03.001 | Canvas overflow produces CanvasOverflow warning with EMU estimate | P0 | active | [BC-3.03.001.md](BC-3.03.001.md) |
| BC-3.03.002 | Strict mode produces no output on validation error | P0 | active | [BC-3.03.002.md](BC-3.03.002.md) |
| BC-3.03.003 | Warn-only mode renders error-slide placeholders and continues | P0 | active | [BC-3.03.003.md](BC-3.03.003.md) |
| BC-3.03.004 | Zero-slide deck produces validation error | P0 | active | [BC-3.03.004.md](BC-3.03.004.md) |

### 3.04 — Structured Shape DSL (CAP-023)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.04.001 | shape: block declares custom shape with type/position/fill/text/alt | P1 | active | [BC-3.04.001.md](BC-3.04.001.md) |
| BC-3.04.002 | raw keyword in user .sf files is rejected at parse time | P1 | active | [BC-3.04.002.md](BC-3.04.002.md) |

### 3.05 — Rich Inline Formatting (CAP-024)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.05.001 | All 12 inline format types render to correct output per format (plain, bold, italic, code, link, math, footnote, xref, super, sub, strike, highlight) | P1 | active | [BC-3.05.001.md](BC-3.05.001.md) |

### 3.06 — Core Layout Transformation: Deck → LaidOutDeck (CAP-010)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-3.06.001 | Layout Transformation Preserves Slide Count | P0 | active | [BC-3.06.001.md](BC-3.06.001.md) |
| BC-3.06.002 | Layout Transformation Is Deterministic (Same Inputs Produce Identical LaidOutDeck) | P0 | active | [BC-3.06.002.md](BC-3.06.002.md) |
| BC-3.06.003 | All Positioned Elements Have Valid Non-Negative EMU Coordinates Within Slide Bounds | P0 | active | [BC-3.06.003.md](BC-3.06.003.md) |

---

## Section 4: Export Bounded Context

> Capabilities: CAP-015, CAP-016, CAP-017

### 4.01 — PPTX Export (CAP-015)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-4.01.001 | Serialize LaidOutDeck to valid .pptx with correct placeholder inheritance | P0 | active | [BC-4.01.001.md](BC-4.01.001.md) |
| BC-4.01.002 | PPTX output passes multi-renderer fidelity: SSIM≥0.99 AND PSNR≥35dB vs reference | P0 | active | [BC-4.01.002.md](BC-4.01.002.md) |
| BC-4.01.003 | PPTX Contains Speaker Notes and Master/Layout/Theme System; Slide Sections Deferred to Follow-Up Story | P0 | active | [BC-4.01.003.md](BC-4.01.003.md) |
| BC-4.01.004 | PPTX contains accessibility metadata (alt text, lang, and WCAG contrast) | P0 | active | [BC-4.01.004.md](BC-4.01.004.md) |
| BC-4.01.005 | Slide IDs start at 256; master IDs at 2^31; all 11 standard + 20 custom layouts present | P0 | active | [BC-4.01.005.md](BC-4.01.005.md) |
| BC-4.01.006 | PPTX contains notesMaster1.xml and handoutMaster1.xml even if empty | P0 | active | [BC-4.01.006.md](BC-4.01.006.md) |

### 4.02 — DOCX Export (CAP-016)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-4.02.001 | Serialize deck to .docx with report register as body paragraphs | P0 | active | [BC-4.02.001.md](BC-4.02.001.md) |
| BC-4.02.002 | Auto-generated document sections present in DOCX from slide data | P0 | active | [BC-4.02.002.md](BC-4.02.002.md) |

### 4.03 — PDF, HTML, and Web Preview Export (CAP-017)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-4.03.001 | PDF output passes veraPDF --flavour ua1 (PDF/UA-1 compliant, tagged) | P0 | active | [BC-4.03.001.md](BC-4.03.001.md) | v1.2 (DI-3 hygiene; SS-07 + story/VP anchors filled; story: STORY-045) |
| BC-4.03.002 | PDF produced via pdf-writer + krilla + SlideTagEngine (no Chrome/headless) | P0 | active | [BC-4.03.002.md](BC-4.03.002.md) | v1.2 (DI-3 hygiene; SS-07 + story/VP anchors filled; stories: STORY-043, STORY-044) |
| BC-4.03.003 | Static HTML output passes WCAG AA via axe-core on CI | P0 | active | [BC-4.03.003.md](BC-4.03.003.md) |
| BC-4.03.004 | Web preview served via axum+websocket+SVG canvas; updates on save | P1 | active | [BC-4.03.004.md](BC-4.03.004.md) |
| BC-4.03.005 | PDF coordinate mapping: EMU to PDF user units with Y-axis flip | P0 | active | [BC-4.03.005.md](BC-4.03.005.md) | v1.2 |

---

## Section 5: Cross-Cutting Capabilities

> Capabilities: CAP-020, CAP-021, CAP-025, CAP-026, CAP-027

### 5.01 — Accessibility Validation (CAP-020)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.01.001 | Missing alt on Visual Element Is Compile Error with Element Location (Post-Layout Enforcement via Stage 6b) | P0 | active | [BC-5.01.001.md](BC-5.01.001.md) |
| BC-5.01.002 | decorative: true opts out of alt requirement; emits empty alt in all formats | P0 | active | [BC-5.01.002.md](BC-5.01.002.md) |
| BC-5.01.003 | Missing label on color-coded element is compile error | P0 | active | [BC-5.01.003.md](BC-5.01.003.md) |
| BC-5.01.004 | Missing deck lang declaration produces lint warning; default is "en" | P0 | active | [BC-5.01.004.md](BC-5.01.004.md) |
| BC-5.01.005 | lang declaration propagates to PPTX Core Properties, PDF /Lang, HTML lang attr | P0 | active | [BC-5.01.005.md](BC-5.01.005.md) |

### 5.02 — Plugin Architecture (CAP-021)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API (Validator Has Pre- and Post-Layout Dispatch) | P0 | active | [BC-5.02.001.md](BC-5.02.001.md) |
| BC-5.02.002 | No bundled plugin bypasses the registered trait interface (dog-fooding guarantee) | P0 | active | [BC-5.02.002.md](BC-5.02.002.md) |

### 5.03 — Package Management (CAP-025)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.03.001 | slideforge package install adds dep to slideforge.toml and sf.lock with sha256 | P1 | active | [BC-5.03.001.md](BC-5.03.001.md) |
| BC-5.03.002 | @import of package not in sf.lock fails with install hint | P1 | active | [BC-5.03.002.md](BC-5.03.002.md) |
| BC-5.03.003 | sf.lock missing for project with deps produces build warning | P1 | active | [BC-5.03.003.md](BC-5.03.003.md) |
| BC-5.03.004 | slideforge package list shows all installed packages with versions and lock status | P1 | active | [BC-5.03.004.md](BC-5.03.004.md) |
| BC-5.03.005 | slideforge package remove removes package from slideforge.toml and sf.lock | P1 | active | [BC-5.03.005.md](BC-5.03.005.md) |
| BC-5.03.006 | slideforge package verify checks all packages against sf.lock SHA-256 checksums | P1 | active | [BC-5.03.006.md](BC-5.03.006.md) |

### 5.04 — Workspace Configuration (CAP-026)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.04.001 | slideforge.toml [workspace] declares multi-deck members; build --workspace builds all | P1 | active | [BC-5.04.001.md](BC-5.04.001.md) |
| BC-5.04.002 | .sfconfig cascade (max 3 levels) applies family-specific overrides | P1 | active | [BC-5.04.002.md](BC-5.04.002.md) |
| BC-5.04.003 | slideforge config explain shows configuration provenance for any setting | P1 | active | [BC-5.04.003.md](BC-5.04.003.md) |

### 5.05 — Watch Mode (CAP-027)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.05.001 | slideforge watch polls .sf files and data sources; re-evaluates on change | P1 | active | [BC-5.05.001.md](BC-5.05.001.md) |
| BC-5.05.002 | HTTP data source failure in watch mode shows error-slide; does not crash | P1 | active | [BC-5.05.002.md](BC-5.05.002.md) |
| BC-5.05.003 | Watch mode HTTP schema change (field rename) surfaces undefined-var errors | P1 | active | [BC-5.05.003.md](BC-5.05.003.md) |
| BC-5.05.004 | File watcher event loss produces CLI warning; force-rebuild available | P1 | active | [BC-5.05.004.md](BC-5.05.004.md) |
| BC-5.05.005 | WebSocket connection drop from web preview reconnects automatically | P1 | active | [BC-5.05.005.md](BC-5.05.005.md) |

### 5.06 — Project Init / Scaffolding (CAP-026)

| BC ID | Title | Priority | Status | File |
|-------|-------|----------|--------|------|
| BC-5.06.001 | slideforge init scaffolds a buildable starter project in the target directory | P1 | active | [BC-5.06.001.md](BC-5.06.001.md) |
| BC-5.06.002 | slideforge init rejects existing project without --force (E-CFG-008) | P1 | active | [BC-5.06.002.md](BC-5.06.002.md) |

---

## Summary Counts

| Section | Subsection | BCs | P0 | P1 |
|---------|-----------|-----|----|----|
| 1 — Authoring | 1.01–1.15 | 53 | 37 | 16 |
| 2 — Branding | 2.01–2.02 | 8 | 6 | 2 |
| 3 — Layout | 3.01–3.06 | 15 | 12 | 3 |
| 4 — Export | 4.01–4.03 | 13 | 12 | 1 |
| 5 — Cross-cutting | 5.01–5.06 | 23 | 7 | 16 |
| **Total** | | **112** | **74** | **38** |

---

## Invariant Coverage

| DI-NNN | Enforcing BCs |
|--------|--------------|
| DI-001 | BC-5.01.001, BC-5.01.002 |
| DI-002 | BC-5.01.003 |
| DI-003 | BC-5.01.004, BC-5.01.005 |
| DI-004 | BC-1.02.003 |
| DI-005 | BC-1.04.003 |
| DI-006 | BC-1.02.002, BC-1.03.003 |
| DI-007 | BC-1.01.004, BC-1.06.002 |
| DI-008 | BC-5.02.001, BC-5.02.002 |
| DI-009 | BC-3.06.001, BC-3.06.002, BC-3.06.003, BC-4.01.001, BC-4.02.001, BC-4.03.002 |
| DI-010 | BC-3.06.003, BC-4.03.005 |
| DI-011 | BC-3.06.001, BC-3.06.002, BC-3.06.003, BC-4.01.001 |
| DI-012 | BC-4.01.001, BC-4.02.001, BC-4.03.001 |
| DI-013 | BC-4.01.002 |
| DI-014 | BC-4.03.001 |
| DI-015 | BC-2.01.002, BC-2.01.004 |
| DI-016 | BC-2.02.002 |
| DI-017 | BC-3.03.002, BC-3.03.004, BC-1.07.005 |
| DI-018 | BC-1.15.002 |
| DI-019 | BC-5.03.003 |
| DI-020 | BC-1.07.002 |
| DI-021 | BC-1.01.005 |
| DI-022 | BC-1.07.003 |

All 22 domain invariants have at least one enforcing BC. Zero orphan invariants.

---

## Edge Case Coverage

| DEC-NNN | Covering BCs |
|---------|-------------|
| DEC-001 | BC-1.04.002 |
| DEC-002 | BC-1.03.003 |
| DEC-003 | BC-1.06.002, BC-1.01.004 |
| DEC-004 | BC-1.07.002 |
| DEC-005 | BC-1.08.003 |
| DEC-006 | BC-1.06.003 |
| DEC-007 | BC-5.05.002 |
| DEC-008 | BC-5.01.002 |
| DEC-009 | BC-1.02.004 |
| DEC-010 | BC-1.01.006 |
| DEC-011 | BC-3.03.004 |
| DEC-012 | BC-1.07.005 |
| DEC-013 | BC-3.03.001 |
| DEC-014 | BC-1.11.002 |
| DEC-015 | BC-1.12.002 |
| DEC-016 | BC-2.01.004 |
| DEC-017 | BC-1.02.005 |
| DEC-018 | BC-5.05.003 |
| DEC-019 | BC-5.03.002 |
| DEC-020 | BC-1.07.004 |

All 20 domain edge cases have at least one covering BC. Zero orphan edge cases.
