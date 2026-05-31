---
document_type: wave-schedule
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-25T00:00:00
phase: 2
traces_to:
  - .factory/stories/dependency-graph.md
  - .factory/stories/epics.md
total_waves: 6
total_stories: 71
---

# Wave Schedule — slideforge v1.0

> Stories within a wave execute in parallel (each delivered by an independent
> subagent). A wave is complete when ALL stories in it reach `status: merged`.
> A wave gate check runs before launching the next wave.
>
> Wave gates enforce the quality bar: all stories merged, all CI green, no
> open P0 findings, no regression in benchmarks.

---

## Wave Overview

| Wave | Epics | Stories | Can Parallelize | Gate Condition |
|------|-------|---------|----------------|----------------|
| Wave 1 | EPIC-01, EPIC-02, EPIC-19 | 14 | Partial — see internal sequencing note below | All stubs compile; CI runs green on all platforms |
| Wave 2 | EPIC-03, EPIC-04 | 7 | Partial — STORY-011→012→013 chain; STORY-011+012→014 (fork, not sequential after 013); STORY-015→016→017 chain | Wave 1 gate PASS |
| Wave 3 | EPIC-05, EPIC-06, EPIC-07, EPIC-10, EPIC-11, EPIC-12 | 17 | Partial — multiple sub-chains within epics (EPIC-05: 018→019/020→021; EPIC-06: 022→023→024/025; EPIC-07: 026→027/028; EPIC-10: 029→030; EPIC-11: 031→032; EPIC-12: 033→034) | Wave 2 gate PASS |
| Wave 4 | EPIC-06, EPIC-07, EPIC-08, EPIC-09, EPIC-13, EPIC-18, EPIC-21 | 16 | Partial — Batch A parallel: STORY-035→036, STORY-043→044→045, STORY-073, STORY-075, STORY-076; Batch B parallel: STORY-037→038→039→040, STORY-041→042; Batch C: STORY-049→050 | Wave 3 gate PASS; Phase 4 crates added to workspace |
| Wave 5 | EPIC-07, EPIC-14, EPIC-15, EPIC-16, EPIC-17 | 16 | Partial — EPIC-16 and EPIC-17 independent of EPIC-14/15; EPIC-15 depends on EPIC-14 (STORY-056 requires STORY-047 for live reload). Chains: EPIC-14: 046→047→048; EPIC-15: 055→056→059 (056 also needs 047); EPIC-16: 060→061→062/063; EPIC-17: 064→065; STORY-072, STORY-074 independent (deferred P2 surfaces) | Wave 4 gate PASS |
| Wave 6 | EPIC-20 (Phase 6) | 6 | Partial — STORY-066/067/068 independent; STORY-071 depends on 066+067; STORY-069/070 independent | Wave 5 gate PASS; Kani + cargo-fuzz available on CI |

**Total: 76 stories, 454 points across 6 waves.**
(Wave 4: 16 stories / 96 pts; Wave 5: 16 stories / 90 pts — updated 2026-05-31 per human approval)

---

## Wave 1: Foundation (14 stories)

**Theme:** Pure-type crates and DSL parser. No I/O, no dependencies on Wave 2+.
The CI/CD infrastructure goes in Wave 1 so every subsequent story runs on a
production-grade pipeline from the first merge.

**Prerequisite:** None.
**Gate:** All 14 stories merged; `cargo build --workspace` compiles; `cargo test
--workspace` passes; CI matrix green on all 5 platforms.

**Internal sequencing:** STORY-001 starts first; STORY-002 begins after STORY-001
completes (STORY-002 depends on STORY-001 for types). STORY-003 starts after
STORY-001 + STORY-002. STORY-004 + STORY-005 start after STORY-001 only (they do
not need STORY-002). STORY-006-010 after STORY-005.
STORY-051-054 are independent CI stories that run in parallel with all code stories.

### STORY-001 — IR Core Types
- **Epic:** EPIC-01
- **Crate:** slideforge-types (SS-15)
- **BCs:** (structure only — types defined here, contract behavior in EPIC-02+)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Defines `Deck`, `Slide`, `ContentBlock`, `Value`, `Block`, `InlineNode` with
  `Hash + Eq + Clone` on all. Integer EMU type (`Emu(i64)`). `Arc<str>` for
  string fields. `#![forbid(unsafe_code)]`. `#![warn(missing_docs)]`.

### STORY-002 — Plugin Trait API
- **Epic:** EPIC-01
- **Crate:** slideforge-plugin-api (SS-14)
- **BCs:** BC-5.02.001, BC-5.02.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- All 10 plugin trait surfaces with sealed trait pattern. Zero workspace deps.
  Trait signatures: `DataSource`, `Exporter`, `ChartRenderer`, `DiagramRenderer`,
  `Validator`, `MathRenderer`, `BrandProvider`, `SlideType`, `SectionType`,
  `InlineFormat`.

### STORY-003 — 31 SlideType Implementations
- **Epic:** EPIC-01
- **Crate:** slideforge-types (module within SS-15)
- **BCs:** BC-3.01.001, BC-3.01.002, BC-3.01.003
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- All 31 slide types as `SlideType` trait implementations. Required field
  enforcement per type. Unknown slide type keyword produces compile error with
  suggestion (via `strsim` or edit-distance). Missing required field error
  includes field name.

### STORY-004 — Value System + EMU Types
- **Epic:** EPIC-01
- **Crate:** slideforge-types
- **BCs:** BC-1.02.003 (no coercion invariant starts here in type design)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- `Value` enum with variants: String, Integer, Float, Bool, List, Map, Null.
  No From/Into coercions between variants. EMU arithmetic helpers
  (`Emu::from_inches`, `Emu::from_points`). 11-level precedence enum.

### STORY-005 — Lexer + Mode-Based Tokenization
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax (SS-01)
- **BCs:** BC-1.01.003
- **VPs:** VP-001 (tab byte span — proptest setup), VP-014 (fuzz target skeleton)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Byte-level lexer modes: text, math (`$...$`), raw (internal). Tab detection
  with byte span. Indentation token stream. `@` directive tokens. `{{ }}` and
  `@{ }` token types.

### STORY-006 — Parser Core: Deck, Slide, Fields, Indentation
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax
- **BCs:** BC-1.01.001, BC-1.01.002
- **VPs:** VP-009 (proptest — valid .sf → non-empty AST)
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- chumsky 0.10.1 parser. Deck metadata block. Slide type keyword + field
  key/value. Indentation-significant parsing (spaces only, tab → hard error
  BC-1.01.003 delegated to lexer). Error accumulation with `chumsky::error::Rich`.

### STORY-007 — Parser: @for, @if/@elif/@else, {{ expr }}
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax
- **BCs:** BC-1.04.001 (AST node), BC-1.05.001 (AST node)
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- AST nodes for `@for`, `@if`, `@elif`, `@else`. Expression grammar for
  `{{ expr }}` — arithmetic, boolean, comparison operators, pipe `|` for filters.
  No evaluation here — pure parse into `Expr` AST nodes.

### STORY-008 — Parser: @include, variants:, set rules, aliases
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax
- **BCs:** BC-1.06.001, BC-1.06.003, BC-1.07.001-005, BC-1.08.001-003, BC-1.09.001-002
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- `@include "path"` with variable-path support. `variants:` block with
  `include_tags`, `exclude_tags`. `set <type>: <field> <value>`. `alias <name> = <type>:`.
  All produce typed AST nodes. Cyclic variant detection in AST (DAG check).
  Undefined variant reference error with list of defined variants.

### STORY-009 — Parser: math delimiters, shape:, DSL versioning, diagnostics
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax
- **BCs:** BC-1.01.005, BC-1.01.006, BC-1.09.001 (resolve), BC-1.13.001, BC-3.04.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- `$...$` / `$$...$$` math mode delimiters in lexer. `@{var}` inside math mode.
  `shape:` block AST node. `raw` keyword rejection at parse time (BC-3.04.002).
  `slideforge_version "1"` validation in deck metadata. Reserved keyword enforcement
  (BC-1.01.005). Var name vs slide type keyword collision detection (BC-1.01.006).

### STORY-010 — Error Accumulation + Diagnostic Infrastructure
- **Epic:** EPIC-02
- **Crate:** slideforge-syntax
- **BCs:** BC-1.15.001, BC-1.15.002, BC-1.15.003
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- `Diagnostic` struct: `file`, `line`, `col`, `range`, `message`, `hint`,
  `error_code` (E-CAT-NNN). `DiagnosticSink` accumulates all errors in one pass.
  Parse errors always fatal. Validation errors fatal in strict mode only.
  `miette`-compatible span conversion for CLI rendering.

### STORY-051 — CI: fmt + clippy + nextest (5-platform matrix)
- **Epic:** EPIC-19
- **Crate:** cross-cutting (devops)
- **BCs:** (no BC — NFR enforcement)
- **NFRs:** NFR-021, NFR-022, NFR-023, NFR-024, NFR-025, NFR-026-030
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- GitHub Actions workflows: `ci.yml` with 5-platform matrix (macos-14, macos-13,
  ubuntu-latest, ubuntu-24.04-arm, windows-latest). Jobs: `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo nextest run
  --workspace`. `RUSTDOCFLAGS="-D warnings" cargo doc`. `#![forbid(unsafe_code)]`
  lint check. `=`-pinned deps grep check.

### STORY-052 — CI: Visual Regression (LibreOffice + SSIM/PSNR)
- **Epic:** EPIC-19
- **Crate:** cross-cutting (devops)
- **BCs:** BC-4.01.002
- **NFRs:** NFR-007, NFR-008, NFR-009, NFR-010, NFR-011
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- `visual-regression.yml` workflow. LibreOffice Still 25.8.7 headless PPTX→PDF→PNG
  at 300 DPI. `visual-diff.py` scikit-image SSIM + PSNR computation. Fixture: all
  31 slide types. PR gate: any slide below SSIM 0.99 OR PSNR 35dB → fail.
  `divergence-log.md` for LibreOffice vs PowerPoint informational delta.

### STORY-053 — CI: Supply-Chain Audit (cargo audit + deny + SBOM)
- **Epic:** EPIC-19
- **Crate:** cross-cutting (devops)
- **BCs:** (no BC — NFR enforcement)
- **NFRs:** NFR-016, NFR-017, NFR-020
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- `cargo audit` on every PR (0 high/critical CVEs). `cargo deny check all`
  (license allowlist enforcement). `deny.toml` with bans, advisories, licenses,
  sources sections. SBOM generation in release job (cargo-sbom or cyclonedx).

### STORY-054 — CI: Release Pipeline + Signed Artifacts + Reproducible Builds
- **Epic:** EPIC-19
- **Crate:** cross-cutting (devops)
- **NFRs:** NFR-020, NFR-031
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** facade
- `release.yml` workflow triggered on semver tags. Cross-compile matrix
  (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu,
  x86_64-unknown-linux-musl, x86_64-pc-windows-msvc). Binary signing (sigstore
  or GPG). SBOM in release artifacts. Reproducible build verification job:
  two independent builds → binary hash equality.

---

## Wave 2: Evaluation + Validation (7 stories)

**Theme:** Pure-core evaluation and compile-time validation. Depends on Wave 1
types and AST. No I/O.

**Prerequisite:** Wave 1 gate PASS.
**Gate:** `cargo test --workspace` passes all unit tests. proptest suites for VP-009
and VP-010 pass (first proptest instances).

### STORY-011 — Expression Evaluator Core
- **Epic:** EPIC-03
- **Crate:** slideforge-eval (SS-02)
- **BCs:** BC-1.02.001, BC-1.02.002
- **VPs:** VP-005 (Kani — no overflow, skeleton)
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- Evaluate `{{ expr }}`: arithmetic (+, -, *, /, %), boolean (and/or/not),
  comparison (==, !=, <, >, <=, >=), pipe filters (|upper, |lower, |round,
  |len, ~15 built-ins). Variable lookup with scope path in error. Symbol table.
  Undefined variable error with full scope path (BC-1.02.002).

### STORY-012 — Variable Scoping + @for Evaluation + Termination
- **Epic:** EPIC-03
- **Crate:** slideforge-eval
- **BCs:** BC-1.02.005, BC-1.04.001, BC-1.04.002, BC-1.04.003
- **VPs:** VP-003 (Kani — @for bounded termination), VP-010 (proptest — outer vars visible)
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- Lexical scoping: outer `@for` variables remain in scope in nested `@for`.
  `@for` generates slides/blocks per collection item. Empty collection → zero
  items, no error (DEC-001). Termination guarantee: `@for` must iterate over
  a bound collection — no unbounded loops. Reject any construct that could
  produce non-termination.

### STORY-013 — @if/@elif/@else Evaluation + @include Cycle Detection
- **Epic:** EPIC-03
- **Crate:** slideforge-eval
- **BCs:** BC-1.05.001, BC-1.05.002, BC-1.06.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Evaluate `@if/@elif/@else` using same type rules as `{{ expr }}` expressions.
  `@include` resolution: load and parse included `.sf` file, inline AST, detect
  and reject circular chains (DEC-003 — Fail-Closed). Reports cycle path in error.
  @include with variable path reports resolved path on error (DEC-006).

### STORY-014 — Type System: No Implicit Coercion + ${{ seq }} Disambiguation
- **Epic:** EPIC-03
- **Crate:** slideforge-eval
- **BCs:** BC-1.02.003, BC-1.02.004
- **VPs:** VP-004 (Kani — "1.10" stays string)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- No implicit type coercion: `"NO"` stays `String("NO")`, `"1.10"` stays
  `String("1.10")`. Type mismatch in expressions → structured error. `${{ seq }}`
  treated as text interpolation context, not math delimiter (DEC-009 — `$` followed
  by `{{` is text, not `$...$ math`).

### STORY-015 — Alt Text Enforcement
- **Epic:** EPIC-04
- **Crate:** slideforge-validate (SS-03)
- **BCs:** BC-5.01.001, BC-5.01.002
- **VPs:** VP-002 (Kani skeleton), VP-008 (Kani skeleton)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Missing `alt` on visual element → compile error with element location. `decorative:
  true` opts out of alt requirement (emits empty alt in all formats, DEC-008). Runs
  in validation stage on completed Deck IR (after eval, before layout).

### STORY-016 — Canvas Overflow + Zero-Slide + Strict/Warn-Only Mode
- **Epic:** EPIC-04
- **Crate:** slideforge-validate
- **BCs:** BC-3.03.001, BC-3.03.002, BC-3.03.003, BC-3.03.004
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Canvas overflow → `CanvasOverflow` warning with EMU estimate (DEC-013). Strict
  mode: no output produced on any validation error (DI-017). Warn-only mode: renders
  error-slide placeholders and continues. Zero-slide deck → validation error (DEC-011).
  `BC-1.15.003` enforcement: parse errors always fatal; validation errors fatal in
  strict mode only.

### STORY-017 — Color-Coded Label + WCAG Contrast Enforcement
- **Epic:** EPIC-04
- **Crate:** slideforge-validate
- **BCs:** BC-5.01.003, BC-5.01.004, BC-5.01.005
- **VPs:** VP-007 (Kani skeleton — WCAG contrast formula)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Missing `label` on color-coded element → compile error (DI-002). Missing `lang`
  declaration → lint warning; default `"en"` used. `lang` propagates to PPTX Core
  Properties, PDF /Lang, HTML lang attribute. WCAG contrast formula:
  `BrandValidator::check_theme_pairs()` using correct luminance linearization
  (0.04045 threshold per VP-007).

---

## Wave 3: Core Services (17 stories)

**Theme:** All subsystem crates that feed into exporters. Runs fully in parallel
across data sources, brand, layout, math, charts, and diagrams.

**Prerequisite:** Wave 2 gate PASS.
**Gate:** All 17 stories merged; brand synthesis ZIP structure test validates 31 layout XML files are well-formed; brand
synthesis produces 31 layouts; proptest VP-011, VP-012 pass.

### STORY-018 — DataSource: JSON/CSV/YAML/TOML File Loading
- **Epic:** EPIC-05
- **Crate:** slideforge-data (SS-10)
- **BCs:** BC-1.03.001, BC-1.03.003
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict

### STORY-019 — DataSource: HTTP/HTTPS + SSRF Allowlist
- **Epic:** EPIC-05
- **Crate:** slideforge-data
- **BCs:** BC-1.03.002, BC-1.03.005
- **NFRs:** NFR-019
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- reqwest 0.12 HTTP client. Domain allowlist enforcement (SSRF prevention). Non-
  allowlisted domains → structured error when allowlist is configured.

### STORY-020 — DataSource: Excel (.xlsx) + SQLite
- **Epic:** EPIC-05
- **Crate:** slideforge-data
- **BCs:** BC-1.03.006, BC-1.03.007
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- calamine 0.26.1 for Excel. rusqlite 0.32.0 with parameterized queries only.

### STORY-021 — DataSource: --offline Flag + Error Handling
- **Epic:** EPIC-05
- **Crate:** slideforge-data
- **BCs:** BC-1.03.004
- **Points:** 3
- **Priority:** P0
- **tdd_mode:** strict
- `--offline` flag skips HTTP data sources. HTTP source in offline mode → structured
  error (not silent skip).

### STORY-022 — Brand Loading: .pptx/.docx Template Extraction
- **Epic:** EPIC-06
- **Crate:** slideforge-brand (SS-04)
- **BCs:** BC-2.01.001, BC-2.01.006
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- quick-xml 0.36 extraction from .pptx ZIP parts. Font unavailability → warning +
  fallback (BC-2.01.006). Extracts: theme colors, font names, master structure.

### STORY-023 — Brand Synthesis: brand.toml → 31 Layouts + 12 OOXML Slots
- **Epic:** EPIC-06
- **Crate:** slideforge-brand
- **BCs:** BC-2.01.002, BC-2.01.004, BC-2.01.005
- **VPs:** VP-012 (proptest — 12-slot palette round-trip)
- **Points:** 13
- **Priority:** P0
- **tdd_mode:** strict
- Synthesize complete brand template from `brand.toml`. All 12 OOXML color slots.
  31 layouts: 11 standard + 20 custom (per brand-architecture.md). Missing color
  slots → derived defaults with warning (DEC-016). `clrMapOvr` for dark-themed
  layouts. Single-master architecture enforced.

### STORY-024 — Brand Extraction CLI (slideforge extract-brand)
- **Epic:** EPIC-06
- **Crate:** slideforge-brand + slideforge-cli (stub command)
- **BCs:** BC-2.01.003
- **Points:** 3
- **Priority:** P0
- **tdd_mode:** strict
- `slideforge extract-brand --input deck.pptx --output brand.toml` command.
  Extracts brand.toml from .pptx or .docx template.

### STORY-025 — Per-Slide brand_overlay: (No Master Switch Invariant)
- **Epic:** EPIC-06
- **Crate:** slideforge-brand
- **BCs:** BC-2.02.001, BC-2.02.002
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- `brand_overlay:` block overrides logo/footer/confidentiality on individual slide.
  Does NOT switch slide master (DI-016 — single-master architecture).

### STORY-026 — Core Layout: Deck → LaidOutDeck, EMU System
- **Epic:** EPIC-07
- **Crate:** slideforge-layout (SS-05)
- **BCs:** (structural — all exporters consume LaidOutDeck)
- **VPs:** VP-011 (proptest — slide count invariant)
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- `layout::run(deck: &Deck, brand: &Brand) → LaidOutDeck`. Integer EMU coordinates
  throughout. `Frame` + `BoundingBox` + `TextFlow` types. `LaidOutSlide` with
  positioned shapes. Slide count in LaidOutDeck must equal Deck slide count (DI-009).

### STORY-027 — Layout: Document Section Generation (DOCX)
- **Epic:** EPIC-07
- **Crate:** slideforge-layout
- **BCs:** BC-3.02.001, BC-3.02.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Auto-generated DOCX sections derived from slide data (`executive_summary`,
  `risk_register`). Manually authored `section methodology:` blocks appear in
  DOCX/PDF. Section type implementations via `SectionType` plugin trait.

### STORY-028 — Layout: shape: Block + Rich Inline Formatting
- **Epic:** EPIC-07
- **Crate:** slideforge-layout
- **BCs:** BC-3.04.001, BC-3.05.001
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- `shape:` block layout: type, position, fill, text, alt fields positioned in
  LaidOutDeck. 11 inline format types: bold, italic, code, link, math (inline),
  footnote, xref, super, sub, strike, highlight. Each maps to `InlineNode` variant
  with layout coordinates.

### STORY-029 — Math Parser: $...$ / $$...$$ + @{var} + OMML Output
- **Epic:** EPIC-10
- **Crate:** slideforge-math (SS-13)
- **BCs:** BC-1.10.001, BC-1.10.002, BC-1.10.003 (OMML)
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- Mode-based math parsing. `@{var}` interpolation in math context (not `{{ }}`).
  Unsupported LaTeX command → error with source span and suggestion hint. OMML
  (Office Math Markup Language) output for PPTX/DOCX.

### STORY-030 — Math: MathML (HTML) + Path-Based (PDF) Output
- **Epic:** EPIC-10
- **Crate:** slideforge-math
- **BCs:** BC-1.10.003 (HTML + PDF paths)
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- MathML output for HTML export. Path-based (glyph outline) output for PDF.
  `MathRenderer` plugin trait implementation.

### STORY-031 — Chart Renderer: bar/line/pie/scatter/area/histogram/stacked-bar
- **Epic:** EPIC-11
- **Crate:** slideforge-charts (SS-12)
- **BCs:** BC-1.11.001
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- plotters 0.3.7 SVG generation for all 7 chart types. `ChartRenderer` plugin
  trait implementation. Brand-aware color application. SVG output usable in all
  export formats.

### STORY-032 — Chart: Empty Data Error-Slide Placeholder
- **Epic:** EPIC-11
- **Crate:** slideforge-charts
- **BCs:** BC-1.11.002
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- Chart with empty dataset → error-slide placeholder (not crash). Error-slide
  includes error message text and source location. Consistent with `BC-3.03.003`
  warn-only mode behavior.

### STORY-033 — Diagram Renderer: Mermaid → PPTX-Safe SVG
- **Epic:** EPIC-12
- **Crate:** slideforge-diagrams (SS-11)
- **BCs:** BC-1.12.001, BC-1.12.002
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- mermaid-rs-renderer 0.2.2. `DiagramRenderer` plugin trait implementation.
  Invalid Mermaid syntax → compile error with line number within Mermaid source
  (DEC-015). Output SVG is pre-normalization.

### STORY-034 — SVG Normalization via usvg + Performance Gate
- **Epic:** EPIC-12
- **Crate:** slideforge-diagrams
- **BCs:** BC-1.12.003
- **NFRs:** NFR-003, NFR-004
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- usvg 0.47.0 normalization: removes `foreignObject`, resolves relative paths,
  sets absolute dimensions. Criterion benchmark: cold font DB scan < 200ms
  (NFR-003); warm render < 10ms (NFR-004).

---

## Wave 4: Exporters + Registry (16 stories)

**Theme:** All output format exporters and plugin registry assembly, plus three
pulled-in P1 follow-ups (STORY-073, STORY-075, STORY-076). Depends on brand,
layout, and all renderers being complete.

**Prerequisite:** Wave 3 gate PASS. Phase 4 crates (`slideforge-pdf`, `slideforge-html`)
added to `[workspace] members` at start of wave (previously in `exclude`).
**Gate:** All 16 stories merged; `.pptx` output validated by LibreOffice headless;
`veraPDF` passes; `cargo deny` green.

**Human-Approved Batch Plan (Wave 4):**
- **Batch A (parallel):** STORY-035→036, STORY-043→044→045, STORY-073 (pulled-in P1),
  STORY-075 (pulled-in P1), STORY-076 (pulled-in P1)
- **Batch B (parallel, after Batch A):** STORY-037→038→039→040, STORY-041→042
- **Batch C (after Batch B):** STORY-049→050

**Pulled-in P1 follow-up stories (human-approved, 2026-05-31):**
- STORY-073 (5 pts) — ContentBlock::Bullets → FrameContent::TextRun frame generation
- STORY-075 (3 pts) — Brand Loader: Footer Detection from .pptx Slide Master
- STORY-076 (3 pts) — Brand Loader: Transform-Aware Theme Color Extraction (srgbClr)

**Deferred to Wave 5 (P2, 2026-05-31):**
- STORY-072 (3 pts) — shape: Gradient Fills (FillSpec::Gradient) — P2
- STORY-074 (3 pts) — Brand-aware Em conversion (font_size_emu) — P2

### STORY-035 — Writing Register Routing in Evaluator
- **Epic:** EPIC-18
- **Crate:** slideforge-eval
- **BCs:** BC-1.14.001, BC-1.14.002, BC-1.14.003
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Register routing: `notes` → presenter notes in PPTX/DOCX/HTML. `report` →
  DOCX body; excluded from PPTX slide content. `detail` → DOCX/PDF only; excluded
  from PPTX and web preview. Tags each `ContentBlock` with its `Register` enum
  variant in the Deck IR.

### STORY-036 — Register-Aware Rendering: No Content Bleed Invariant
- **Epic:** EPIC-18
- **Crate:** slideforge-eval + per-exporter validation
- **BCs:** BC-1.14.004
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Invariant: no register content appears in wrong output format (DI-012 style).
  Enforced in each exporter by filtering `ContentBlock` by `Register` before
  rendering. Integration test: build same source with all 5 exporters, assert
  `notes` content absent from PPTX slide XML, `detail` content absent from PPTX
  and HTML preview, etc.

### STORY-037 — PPTX Core Serialization: ooxmlsdk 0.6.1 + ZIP Assembly
- **Epic:** EPIC-08
- **Crate:** slideforge-pptx (SS-06)
- **BCs:** BC-4.01.001
- **VPs:** VP-013 (proptest skeleton — valid ZIP with Content_Types.xml)
- **Points:** 13
- **Priority:** P0
- **tdd_mode:** strict
- ooxmlsdk 0.6.1. `Exporter` plugin trait implementation. Serialize `LaidOutDeck`
  to `.pptx` ZIP. `[Content_Types].xml` comprehensive registration of all parts.
  Element ordering schema-significant throughout. zip 4.2.0 for OPC postprocessing.

### STORY-038 — PPTX Layout Compliance: Placeholder Inheritance + Slide IDs + Layouts
- **Epic:** EPIC-08
- **Crate:** slideforge-pptx
- **BCs:** BC-4.01.005
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- Placeholder inheritance: layout→master by `type`, slide→layout by `idx`. Slide
  IDs start at 256. Master IDs start at 2^31. All 11 standard + 20 custom layouts
  present in synthesized `.pptx`. `clrMapOvr` for dark-themed layouts.

### STORY-039 — PPTX Accessibility Metadata
- **Epic:** EPIC-08
- **Crate:** slideforge-pptx
- **BCs:** BC-4.01.004, BC-5.01.005 (PPTX lang propagation)
- **NFRs:** NFR-014, NFR-015
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Alt text for all non-decorative elements (`<p:pic>` → `<p:nvPicPr><p:cNvPr descr="..."/>`).
  `lang` in Core Properties (`<cp:defaultLocale>`). WCAG contrast metadata.
  Custom PPTX linter test verifies 100% alt coverage on fixture.

### STORY-040 — PPTX: Speaker Notes + Slide Sections + notesMaster1.xml
- **Epic:** EPIC-08
- **Crate:** slideforge-pptx
- **BCs:** BC-4.01.003, BC-4.01.006
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Speaker notes via `notes` register → `<p:notes>` part. Slide sections from
  variant/section markers. `notesMaster1.xml` and `handoutMaster1.xml` always
  present even if empty (R2 finding). Visual regression fixture: all 31 slide
  types.

### STORY-041 — DOCX Core Serialization: report register + ooxmlsdk
- **Epic:** EPIC-09
- **Crate:** slideforge-docx (SS-08)
- **BCs:** BC-4.02.001
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- ooxmlsdk 0.6.1. `Exporter` plugin trait implementation for DOCX. `report`
  register content → body paragraphs. Document structure: styles, theme, settings.

### STORY-042 — DOCX: Auto-Generated Document Sections
- **Epic:** EPIC-09
- **Crate:** slideforge-docx
- **BCs:** BC-4.02.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Auto-generated sections from slide data (per `SectionType` plugin). Manually
  authored `section methodology:` blocks. Section ordering matches slide order.

### STORY-043 — PDF Core: pdf-writer + krilla + SlideTagEngine
- **Epic:** EPIC-13
- **Crate:** slideforge-pdf (SS-07)
- **BCs:** BC-4.03.002
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- pdf-writer 0.14.0 + krilla 0.6.0. No Chrome/headless dependency. `SlideTagEngine`
  for tagged PDF structure. subsetter 0.2.0 for font subsetting. `Exporter` plugin
  trait implementation.

### STORY-044 — PDF: EMU-to-PDF Coordinate Mapping + Y-Axis Flip
- **Epic:** EPIC-13
- **Crate:** slideforge-pdf
- **BCs:** BC-4.03.005
- **VPs:** VP-006 (Kani skeleton — coordinate mapping + no overflow)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- EMU → PDF user units: 914400 EMU/inch → 72 points/inch. Y-axis flip: PDF
  origin is bottom-left, slideforge origin is top-left. Integer arithmetic only
  (no `f64` overflow risk). Kani-amenable pure function.

### STORY-045 — PDF: PDF/UA-1 Tagging + veraPDF CI Gate
- **Epic:** EPIC-13
- **Crate:** slideforge-pdf + CI
- **BCs:** BC-4.03.001
- **NFRs:** NFR-012
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Complete PDF/UA-1 tagging via `SlideTagEngine`. `veraPDF --flavour ua1` gate in
  CI: 0 violations. Tagging includes: document structure, figure alt text, language.
  Integration test: generate PDF from fixture, run veraPDF, assert 0 errors.

### STORY-049 — Plugin Registry Assembly
- **Epic:** EPIC-21
- **Crate:** slideforge (root)
- **BCs:** BC-5.02.001, BC-5.02.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- Root `slideforge` crate assembles `PluginRegistry` from all bundled plugin crates.
  All 10 trait surfaces have at least one registered implementation. Dog-fooding
  guarantee: every bundled plugin registered via the public `slideforge-plugin-api`
  trait — zero internal type bypasses.

### STORY-050 — End-to-End Integration Test Suite
- **Epic:** EPIC-21
- **Crate:** slideforge (root) + slideforge-cli (integration tests)
- **BCs:** BC-5.02.001, BC-5.02.002
- **NFRs:** NFR-034, NFR-035
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- End-to-end tests: compile `.sf` fixture → each of 5 output formats. Assert:
  correct slide count, alt text present, lang set, register routing correct. Holdout
  scenario stubs (NFR-034, NFR-035 targets defined here; evaluated in Phase 4).
  `insta` snapshot tests for IR structures.

### STORY-073 — Layout: ContentBlock::Bullets → FrameContent::TextRun (Pulled-in P1)
- **Epic:** EPIC-07
- **Crate:** slideforge-layout (SS-05)
- **BCs:** BC-3.05.001
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- **Batch:** A (parallel with STORY-035→036, STORY-043→044→045, STORY-075, STORY-076)
- Extends `layout::run()` to generate `FrameContent::TextRun` frames from
  `ContentBlock::Bullets` items; extends `run_inline_validation` to traverse bullet
  inline content (xref validation, depth bounds). Pulled into Wave 4 from Wave TBD
  because STORY-037/041/046 exporter stories depend on bullets being in LaidOutDeck.

### STORY-075 — Brand Loader: Footer Detection (Pulled-in P1)
- **Epic:** EPIC-06
- **Crate:** slideforge-brand (SS-04)
- **BCs:** BC-2.01.001
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- **Batch:** A (parallel with STORY-035→036, STORY-043→044→045, STORY-073, STORY-076)
- Adds footer detection to `BrandLoader::load()`: reads `ppt/slideMasters/slideMaster1.xml`
  and `presProps.xml` to populate `BrandTemplate.footer_text` and `footer_flags`.
  Makes STORY-024's `[footer]` writer reachable. Pulled into Wave 4 as P1 follow-up
  (closes adversary finding F-024A-OBS-1).

### STORY-076 — Brand Loader: srgbClr Transform Detection (Pulled-in P1)
- **Epic:** EPIC-06
- **Crate:** slideforge-brand (SS-04)
- **BCs:** BC-2.01.001, BC-2.01.003
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- **Batch:** A (parallel with STORY-035→036, STORY-043→044→045, STORY-073, STORY-075)
- Extends `parse_theme_colors` to detect `srgbClr` elements with transform children
  (lumMod/lumOff/tint/shade): stores base hex verbatim, sets `is_derived = true`,
  emits `tracing::warn!`. Implements BC-2.01.001 EC-006 (loading) and BC-2.01.003
  EC-003 widened (extraction). Pulled into Wave 4 as P1 follow-up (PO BC delta landed
  2026-05-31; Option B chosen for v1.0; no new error code).

---

## Wave 5: CLI + User-Facing Features + Deferred Surfaces (16 stories)

**Theme:** CLI binary, web preview, package management, workspace configuration.
The complete user-facing product. Depends on all exporters being complete.
Also includes two P2 deferred-surface stories (STORY-072, STORY-074) moved here
from Wave TBD per human approval on 2026-05-31.

**Prerequisite:** Wave 4 gate PASS.
**Gate:** `slideforge build deck.sf` end-to-end works. `slideforge watch` live
reloads. All CI gates pass including performance benchmarks (NFR-001 < 500ms).

**Added to Wave 5 (deferred P2 stories from Wave TBD, 2026-05-31):**
- STORY-072 (3 pts) — shape: Gradient Fills (FillSpec::Gradient) — P2
- STORY-074 (3 pts) — Brand-aware Em conversion (font_size_emu) — P2

### STORY-055 — CLI: build command + miette error rendering
- **Epic:** EPIC-15
- **Crate:** slideforge-cli (SS-18)
- **BCs:** BC-1.15.001, BC-1.15.002, BC-1.15.003
- **NFRs:** NFR-032, NFR-033
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- clap 4.5 CLI. `slideforge build <input.sf> [--output <dir>] [--format pptx|docx|pdf|html]`.
  miette 7.2 error rendering: file:line:col, source excerpt, caret, hint, error code.
  tracing-subscriber structured logging. All 6 pipeline stage spans (NFR-032).
  `--json` output with structured diagnostics (NFR-033).

### STORY-056 — CLI: watch mode + notify integration + incremental rebuild
- **Epic:** EPIC-15
- **Crate:** slideforge-cli
- **BCs:** BC-5.05.001, BC-5.05.002, BC-5.05.003, BC-5.05.004
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- `slideforge watch <input.sf>`. notify 8.0 file watcher. Incremental rebuild on
  `.sf` change. HTTP data source failure in watch mode → error-slide, no crash
  (DEC-007). HTTP schema change → undefined-var errors surfaced (DEC-018). Event
  loss warning → force-rebuild available (BC-5.05.004).

### STORY-057 — CLI: init scaffolding + extract-brand command
- **Epic:** EPIC-15
- **Crate:** slideforge-cli
- **BCs:** BC-5.06.001, BC-5.06.002
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- `slideforge init [--target <dir>]` creates buildable starter project. Rejects
  existing project without `--force` (E-CFG-008). `slideforge extract-brand` wired
  to slideforge-brand extraction (complements STORY-024).

### STORY-058 — CLI: --json/--quiet/--verbose flags + NO_COLOR + non-TTY
- **Epic:** EPIC-15
- **Crate:** slideforge-cli
- **BCs:** BC-1.15.001 (diagnostic rendering modes)
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** strict
- anstyle/anstream crate for TTY detection and NO_COLOR compliance. Never raw
  `\x1b[` codes. `--quiet`: errors only to stderr. `--verbose`: per-stage timing.
  `--json`: all output as JSON to stdout. Terminal color palette per UX-INDEX.md.

### STORY-059 — CLI: Criterion performance benchmarks
- **Epic:** EPIC-15
- **Crate:** slideforge-cli
- **BCs:** (no BC — NFR enforcement)
- **NFRs:** NFR-001, NFR-002, NFR-005, NFR-006
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- Criterion benchmark in `crates/slideforge-cli/benches/build_bench.rs`. Fixture:
  25 slides, 10+ slide types, brand.toml synthesis, 2 file-based @data sources.
  Gate: NFR-001 cold build < 500ms; NFR-002 incremental < 50ms; NFR-005 PPTX/DOCX
  serialization < 200ms; NFR-006 peak memory < 256MB.

### STORY-046 — HTML Exporter: WCAG AA via axe-core
- **Epic:** EPIC-14
- **Crate:** slideforge-html (SS-09 partial)
- **BCs:** BC-4.03.003
- **NFRs:** NFR-013
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** strict
- Static HTML `Exporter`. SVG-based canvas (not `<canvas>` — per S3 spike). WCAG AA
  via @axe-core/playwright on every PR touching HTML/preview output. 0 axe-core
  critical violations gate.

### STORY-047 — Web Preview Server: axum + WebSocket + SVG Canvas
- **Epic:** EPIC-14
- **Crate:** slideforge-preview (SS-09)
- **BCs:** BC-4.03.004
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- axum 0.8.1 HTTP server (local only, not a deployed service). tokio 1.38 async
  runtime. WebSocket for live reload. SVG-based slide canvas. `prefers-reduced-motion`
  suppresses animation. Design tokens from UX-INDEX.md.

### STORY-048 — Web Preview: Live Reload + Accessibility (ARIA, keyboard nav)
- **Epic:** EPIC-14
- **Crate:** slideforge-preview
- **BCs:** BC-5.05.005
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- WebSocket reconnect on drop (BC-5.05.005). `aria-live="polite"` for connection
  status. Keyboard: arrow keys for slide prev/next, Tab for controls. Error overlays
  `role="alert"`. 48px min touch targets. All 11 contextual variants from UX-INDEX.md.

### STORY-060 — Package: install + sf.lock + SHA-256 integrity
- **Epic:** EPIC-16
- **Crate:** slideforge-package (SS-16)
- **BCs:** BC-5.03.001, BC-5.03.003
- **NFRs:** NFR-018
- **Points:** 8
- **Priority:** P1
- **tdd_mode:** strict
- `slideforge package install <url>` — adds to `slideforge.toml` + `sf.lock` with
  SHA-256 checksum. Git-based package fetch. `sf.lock` missing for project with deps
  → build warning (DEC-019 complement, BC-5.03.003). sha2 0.10 for checksums.

### STORY-061 — Package: @import resolution in evaluator
- **Epic:** EPIC-16
- **Crate:** slideforge-package + slideforge-eval
- **BCs:** BC-5.03.002, BC-1.06.004
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- `@import` of package not in `sf.lock` fails with install hint (DEC-019).
  `@import` resolves at compile time to installed package location. Wires
  slideforge-eval's `@include` resolution path to package install location.

### STORY-062 — Package: remove + list
- **Epic:** EPIC-16
- **Crate:** slideforge-package
- **BCs:** BC-5.03.004, BC-5.03.005
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- `slideforge package remove <name>` removes from `slideforge.toml` and `sf.lock`.
  `slideforge package list` shows all installed packages with versions and lock status.

### STORY-063 — Package: verify (SHA-256 checksum audit)
- **Epic:** EPIC-16
- **Crate:** slideforge-package
- **BCs:** BC-5.03.006
- **NFRs:** NFR-018
- **Points:** 3
- **Priority:** P1
- **tdd_mode:** strict
- `slideforge package verify` checks all packages against `sf.lock` SHA-256
  checksums. Reports any mismatches with package name and expected vs actual hash.

### STORY-064 — Workspace: slideforge.toml [workspace] + build --workspace
- **Epic:** EPIC-17
- **Crate:** slideforge-config (SS-17)
- **BCs:** BC-5.04.001
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- `slideforge.toml [workspace]` with `members = [...]`. `slideforge build --workspace`
  builds all member decks. toml 0.8 for config parsing.

### STORY-065 — Workspace: .sfconfig cascade + config explain provenance
- **Epic:** EPIC-17
- **Crate:** slideforge-config
- **BCs:** BC-5.04.002, BC-5.04.003
- **Points:** 5
- **Priority:** P1
- **tdd_mode:** strict
- `.sfconfig` cascade: max 3 levels, family-specific overrides. `slideforge config
  explain <setting>` shows full provenance: which `.sfconfig` file at which level
  provided the value.

### STORY-072 — shape: Gradient Fills (FillSpec::Gradient) (Deferred P2)
- **Epic:** EPIC-07
- **Crate:** slideforge-layout + exporters (SS-05, SS-06, SS-07, SS-08, SS-09)
- **BCs:** BC-3.04.001
- **Points:** 3
- **Priority:** P2
- **tdd_mode:** strict
- Adds `FillSpec::Gradient { from: Rgb, to: Rgb }` to `slideforge-types` and wires
  DSL parser → IR → layout passthrough → all four exporters (PPTX native `<a:gradFill>`;
  PDF `ShadingPattern`; HTML CSS `linear-gradient`; DOCX solid fallback + lint warning).
  Deferred from Wave TBD to Wave 5 per human approval 2026-05-31 (P2, after exporters exist).

### STORY-074 — Brand-aware Em conversion (font_size_emu) (Deferred P2)
- **Epic:** EPIC-07
- **Crate:** slideforge-types + slideforge-layout (SS-01, SS-05)
- **BCs:** BC-3.04.001
- **Points:** 3
- **Priority:** P2
- **tdd_mode:** strict
- Adds `font_size_emu: Emu` to `BrandFonts` (default 457_200 = 36pt). Threads real
  brand body font size through `layout::run` → `layout_shapes()` replacing
  `DEFAULT_EM_IN_EMU` constant. Closes structural deferral from STORY-028.
  Deferred from Wave TBD to Wave 5 per human approval 2026-05-31 (P2).

---

## Wave 6: Formal Verification (6 stories)

**Theme:** Phase 6 formal hardening. Kani proofs, proptest suites, fuzz harnesses,
mutation testing. Linux/macOS only for Kani (Windows uses concrete tests).

**Prerequisite:** Wave 5 gate PASS. Kani and cargo-fuzz available on CI runners.
**Gate:** All 6 stories merged; all Kani proofs bounded-model-check pass; proptest
suites find no counterexamples; fuzz harnesses run 24h without crash; cargo-mutants
kill rate meets documented budget.

### STORY-066 — Kani Proofs: slideforge-syntax (VP-001, VP-003, VP-009)
- **Epic:** EPIC-20
- **Crate:** `crates/slideforge-syntax/src/proofs/`
- **VPs:** VP-001, VP-003, VP-009, VP-014
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** facade
- Kani proof: tab detection byte span accuracy (VP-001). Kani proof: @for over
  bounded collection always terminates (VP-003). proptest: valid .sf → non-empty
  AST (VP-009). cargo-fuzz harness: any input terminates and produces errors or
  AST (VP-014).

### STORY-067 — Kani Proofs: slideforge-eval (VP-004, VP-005, VP-010)
- **Epic:** EPIC-20
- **Crate:** `crates/slideforge-eval/src/proofs/`
- **VPs:** VP-004, VP-005, VP-010, VP-015
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** facade
- Kani proof: no implicit coercion — "1.10" stays string (VP-004). Kani proof:
  integer arithmetic no overflow (VP-005). proptest: outer @for vars visible in
  inner @for scope (VP-010). cargo-fuzz: any valid AST terminates eval (VP-015).

### STORY-068 — Kani Proofs: slideforge-validate (VP-002, VP-007, VP-008)
- **Epic:** EPIC-20
- **Crate:** `crates/slideforge-validate/src/proofs/`
- **VPs:** VP-002, VP-007, VP-008
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** facade
- Kani proof: alt-missing produces error before layout (VP-002). Kani proof: WCAG
  contrast formula correct luminance linearization 0.04045 (VP-007). Kani proof:
  image with empty alt always produces diagnostic (VP-008). All three are P0.

### STORY-069 — Proptest Suites: brand (VP-012) + pptx (VP-013) + layout (VP-011)
- **Epic:** EPIC-20
- **Crate:** multiple
- **VPs:** VP-011, VP-012, VP-013
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- proptest: 12-slot palette round-trip synthesize→extract→same colors (VP-012).
  proptest: every synthesized PPTX is valid ZIP with [Content_Types].xml (VP-013).
  proptest: LaidOutDeck slide count equals Deck slide count (VP-011).

### STORY-070 — Kani Proofs: slideforge-pdf (VP-006)
- **Epic:** EPIC-20
- **Crate:** `crates/slideforge-pdf/src/proofs/`
- **VPs:** VP-006
- **Points:** 5
- **Priority:** P0
- **tdd_mode:** facade
- Kani proof: EMU-to-PDF coordinate mapping — correct Y-axis flip, no overflow
  (VP-006). Pure function: `emu_to_pdf_units(emu: Emu, page_height: Emu) → PdfUnit`.
  Bounded model checking verifies no integer overflow within OOXML coordinate range.

### STORY-071 — Fuzz Harnesses + cargo-mutants Integration
- **Epic:** EPIC-20
- **Crate:** cross-cutting
- **VPs:** VP-014, VP-015
- **Points:** 8
- **Priority:** P0
- **tdd_mode:** facade
- cargo-fuzz harnesses for VP-014 (syntax parser) and VP-015 (eval). cargo-mutants
  integration: mutation testing with documented kill-rate budget per crate. CI job:
  `just mutants` with pass/fail threshold. `just kani-local` and `just fuzz-local`
  Justfile targets.

---

## Wave Gate Checklist

### Wave Gate Template (applies to each wave)

Before launching wave N+1, verify:

- [ ] All `status: merged` in STORY-INDEX.md for wave N stories
- [ ] `cargo build --workspace` green on `develop` branch
- [ ] `cargo test --workspace --all-features --no-fail-fast` green on all 5 platforms
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` zero violations
- [ ] `cargo fmt --all -- --check` clean
- [ ] All adversarial review passes resolved (3-CLEAN per BC-5.39.001)
- [ ] No open P0 findings in tally tracker
- [ ] Visual regression CI green (Wave 4+ only)
- [ ] Performance benchmarks within threshold (Wave 5+ only)

### Wave 6 Additional Gates (Phase 6 formal verification)

- [ ] All Kani proofs pass (`just kani-local` on Linux/macOS)
- [ ] All proptest suites find zero counterexamples
- [ ] cargo-fuzz 24h run without crash on CI
- [ ] cargo-mutants kill rate meets documented budget
- [ ] Full 7-dimension convergence check PASS (per CLAUDE.md quality bar)
