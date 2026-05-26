---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-033
title: "Diagram Renderer: Mermaid → PPTX-Safe SVG"
epic: EPIC-12
wave: 3
points: 8
priority: P1
tdd_mode: strict
status: draft
behavioral_contracts: [BC-1.12.001, BC-1.12.002]
verification_properties: []
nfr_refs: [NFR-003, NFR-004, NFR-021, NFR-022, NFR-023, NFR-024]
crate: slideforge-diagrams
target_module: slideforge-diagrams
subsystems: [SS-11]
depends_on: [STORY-001, STORY-002]
blocks:
  - STORY-034
  - STORY-037
  - STORY-043
  - STORY-046
estimated_days: 4
---

# STORY-033: Diagram Renderer — Mermaid → PPTX-Safe SVG

## Subsystem Anchor Justification

SS-11 (Diagrams) owns this story's scope because `slideforge-diagrams` is the
exclusive owner of Mermaid → SVG rendering per ARCH-INDEX Subsystem Registry.
ADR-014 selected `mermaid-rs-renderer` as the rendering engine (resolving Spike S14).
No exporter implements diagram rendering.

## Dependency Anchor Justifications

- Depends on STORY-001: `DiagramSlide`, `AltText`, `Brand` types from
  `slideforge-types` are the inputs to diagram rendering.
- Depends on STORY-002: `DiagramRenderer` plugin trait declared in
  `slideforge-plugin-api`.
- Blocks STORY-034: SVG normalization (usvg) operates on the raw SVG this story
  produces. STORY-034 extends this crate with the normalization step.
- Blocks exporters (037, 043, 046): All consume the normalized `DiagramSvg`.

## Summary

Implement `slideforge-diagrams` as the `DiagramRenderer` plugin. Using
`mermaid-rs-renderer` 0.2.2 (pure Rust, no Node.js), render Mermaid diagram source
to an SVG string. Handle the happy path (BC-1.12.001) and the error path for invalid
Mermaid syntax (BC-1.12.002). Maintain a per-process font database cache to meet
the cold (< 200ms) and warm (< 10ms) performance requirements (NFR-003, NFR-004).

The SVG produced by this story is the RAW output from `mermaid-rs-renderer`. STORY-034
adds the usvg normalization step. Together they form the complete diagram pipeline.

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-1.12.001 | slide diagram: renders Mermaid source to PPTX-safe SVG | AC-001, AC-002, AC-003, AC-004, AC-005 |
| BC-1.12.002 | Invalid Mermaid syntax produces compile error with line number within source | AC-006, AC-007 |

## Acceptance Criteria

### AC-001: DiagramRenderer plugin trait implementation
(traces to BC-1.12.001 postcondition 1 — valid SVG produced)

`slideforge_diagrams::DiagramRendererImpl` implements
`slideforge_plugin_api::DiagramRenderer`:
```rust
pub trait DiagramRenderer: Send + Sync {
    fn render(
        &self,
        source: &str,
        lang: DiagramLang,
        alt: &AltText,
        brand: &Brand,
    ) -> Result<RawDiagramSvg, DiagramError>;
}
```
`DiagramLang::Mermaid` is the only supported language in v1.0. `lang: DiagramLang`
is an enum with `Mermaid` variant. `RawDiagramSvg(String)` is a newtype.
`DiagramError` uses `thiserror`.

### AC-002: Mermaid rendered via mermaid-rs-renderer 0.2.2
(traces to BC-1.12.001 invariant 1 — no Node.js spawned)

`mermaid_rs_renderer::render(source: &str) -> Result<String, MermaidError>` is
called synchronously. No subprocess is spawned. The font database is initialized
once per process using a `std::sync::OnceLock<FontDb>` and reused on subsequent
calls (warm path).

### AC-003: Cold font DB scan < 200ms (NFR-003)
(traces to BC-1.12.001 postcondition 9 — cold render time < 200ms)

The first call to `DiagramRendererImpl::render()` in a fresh process initializes
the font database. A Criterion benchmark in `benches/cold_render.rs` measures this
initialization time. The benchmark must pass < 200ms on the CI Linux x86_64 runner.
The empirical measurement from Spike S14: 124ms on M-series Mac; budget allows
200ms headroom for CI runners.

`OnceLock` initialization:
```rust
static FONT_DB: OnceLock<Arc<FontDatabase>> = OnceLock::new();

fn get_font_db() -> &'static Arc<FontDatabase> {
    FONT_DB.get_or_init(|| Arc::new(FontDatabase::load_system_fonts()))
}
```

### AC-004: Warm render < 10ms (NFR-004)
(traces to BC-1.12.001 postcondition 8 — warm render time < 10ms)

After the font DB is initialized (second and subsequent calls), `render()` returns
within 10ms for typical diagrams. A Criterion benchmark in `benches/warm_render.rs`
measures per-call render time after DB initialization. The benchmark must pass
< 10ms. Empirical from S14: < 3ms typical for flowcharts.

### AC-005: Accessibility attributes in rendered SVG
(traces to BC-1.12.001 postcondition 7)

Before returning `RawDiagramSvg`, inject:
- `aria-label="{alt text}"` on the root `<svg>` element
- `<title>{alt text}</title>` as the first child element

If `AltText::Decorative`, use `aria-label=""` and `<title></title>`.

### AC-006: Invalid Mermaid syntax → E-EXP-008 with source line number
(traces to BC-1.12.002 postcondition 1)

When `mermaid_rs_renderer::render()` returns an error, the error message from
`mermaid-rs-renderer` is parsed to extract the line number within the Mermaid source
block. The structured diagnostic emitted is:
```
E-EXP-008: Diagram render failed for slide 'Architecture Overview': 
           Undefined node 'B' at source line 3
  --> deck.sf:45:5
  hint: Check the Mermaid diagram source starting at line 3.
```
If the `mermaid-rs-renderer` error does not include a line number, `source line 1`
is used as a conservative fallback.

### AC-007: Error-slide placeholder in warn-only mode
(traces to BC-1.12.002 postcondition 4)

In `--warn-only` mode, `DiagramError::MermaidSyntaxError` causes a
`FrameContent::ErrorSlidePlaceholder` to be produced for the diagram slide's
position. Processing continues for other slides (DI-018).

### AC-008: All 23 Mermaid diagram types available
(traces to BC-1.12.001 invariant 3)

The 23 diagram types supported by `mermaid-rs-renderer` 0.2.2 are all available
without configuration. A test confirms that the following types parse and render
without error: `flowchart`, `sequenceDiagram`, `gantt`, `classDiagram`,
`stateDiagram`, `erDiagram`, `pie`, `journey`, `gitGraph`.
(Full 23-type list is in the mermaid-rs-renderer crate documentation.)

## Tasks

- [ ] Create `crates/slideforge-diagrams/` with `Cargo.toml`
- [ ] Add to workspace members
- [ ] Define `DiagramLang`, `RawDiagramSvg`, `DiagramError`, `DiagramDiagnostic` types in `src/types.rs`
- [ ] Add `DiagramRenderer` trait to `slideforge-plugin-api`
- [ ] Implement `DiagramRendererImpl` with `OnceLock<Arc<FontDatabase>>` in `src/lib.rs`
- [ ] Implement `render()` calling `mermaid_rs_renderer::render()`
- [ ] Implement `OnceLock` font DB initialization in `src/font_db.rs`
- [ ] Implement `E-EXP-008` error construction with line number extraction in `src/error.rs`
- [ ] Implement `inject_aria_attributes` post-processor (shared pattern with charts)
- [ ] Implement warn-only mode: produce `FrameContent::ErrorSlidePlaceholder` on error
- [ ] Write Criterion benchmark `benches/cold_render.rs` (measures first-call font DB init)
- [ ] Write Criterion benchmark `benches/warm_render.rs` (measures per-call after init)
- [ ] Write unit tests:
  - valid flowchart → non-empty SVG, aria-label present
  - valid sequenceDiagram → non-empty SVG
  - invalid Mermaid → `DiagramError::MermaidSyntaxError` with source line
  - empty source (`""`) → `DiagramError` with "at source line 1"
  - two diagram slides: one valid, one invalid → valid renders, invalid → placeholder

## Previous Story Intelligence

N/A — first story in EPIC-12. Key lesson from Spike S14: `mermaid-rs-renderer`
requires a font database for text rendering in diagrams. The font database scan is
the source of cold-start latency (124ms). Use `OnceLock` to initialize once per
process — this is the standard pattern for lazy static initialization in Rust.

## Architecture Compliance Rules

1. **SS-11 is Effectful shell (font DB scan)**: `slideforge-diagrams` is classified
   as effectful because of the font database scan. However, subsequent renders are
   pure (font DB is read-only after initialization). The effectful boundary is the
   `OnceLock::get_or_init` call only.
2. **No Node.js (BC-1.12.001 invariant 1)**: `mermaid-rs-renderer` is a pure Rust
   crate. No subprocess spawning. Verify this in the dependency tree.
3. **No HTTP during rendering (BC-1.12.001 invariant 2)**: Font loading uses system
   fonts only. No network calls in the render path.
4. **DiagramRenderer trait compliance (DI-008)**: `DiagramRendererImpl` uses only
   the `slideforge-plugin-api::DiagramRenderer` public trait.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace | `Brand`, `AltText`, `DiagramSlide` types |
| `slideforge-plugin-api` | workspace | `DiagramRenderer` trait |
| `mermaid-rs-renderer` | `=0.2.2` | Mermaid → SVG rendering (pure Rust) |
| `thiserror` | `=2.0.18` | `DiagramError` |
| `criterion` | `=0.5.1` | Performance benchmarks (NFR-003, NFR-004) |
| `quick-xml` | `=0.36.0` | Accessibility attribute injection |

Note: `mermaid-rs-renderer` 0.2.2 is the version validated in Spike S14. Do NOT
upgrade without re-running the performance benchmarks and documenting the delta.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-diagrams/Cargo.toml` | Create | Crate manifest |
| `crates/slideforge-diagrams/src/lib.rs` | Create | `DiagramRendererImpl` |
| `crates/slideforge-diagrams/src/types.rs` | Create | `DiagramLang`, `RawDiagramSvg`, `DiagramError` |
| `crates/slideforge-diagrams/src/font_db.rs` | Create | `OnceLock<FontDatabase>` initialization |
| `crates/slideforge-diagrams/src/error.rs` | Create | E-EXP-008 construction with line extraction |
| `crates/slideforge-diagrams/src/accessibility.rs` | Create | aria-label + title injection |
| `crates/slideforge-diagrams/benches/cold_render.rs` | Create | Cold font DB Criterion benchmark |
| `crates/slideforge-diagrams/benches/warm_render.rs` | Create | Warm render Criterion benchmark |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~2,800 |
| BC-1.12.001 + BC-1.12.002 | ~3,500 |
| mermaid-rs-renderer 0.2.2 API | ~1,500 |
| `slideforge-types` diagram types | ~1,000 |
| Test + benchmark files to write | ~4,000 |
| **Total** | **~12,800** |

## Test Strategy

- **Unit tests**: valid flowchart → SVG; invalid Mermaid → E-EXP-008 with line N;
  empty source → E-EXP-008 at line 1; aria-label present; no panic on any string input.
- **Benchmark gates**: `cold_render` < 200ms; `warm_render` < 10ms. These must be
  Criterion benchmarks with `--bench` flag. CI will run them and check thresholds.
- **Supported type coverage**: Test 9+ Mermaid diagram types produce non-empty SVG.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flowchart with 4 nodes | SVG produced; warm render < 3ms |
| EC-002 | ER diagram with 10 entities | SVG produced; warm render < 5ms |
| EC-003 | Gantt chart with 5 tasks | SVG produced; warm render < 1ms |
| EC-004 (DEC-015) | Arrow to undefined node | E-EXP-008 with source line; placeholder in warn-only |
| EC-005 | Empty `source:` field | E-EXP-008: "empty Mermaid source" at line 1 |
| EC-006 | Valid + invalid diagram in same build | Valid renders; invalid → E-EXP-008; DI-018 |

## Forbidden Dependencies

`slideforge-diagrams` MUST NOT depend on:
- Any exporter crate
- `slideforge-data`, `slideforge-brand`, `slideforge-layout`, `slideforge-cli`
- Any Node.js runtime, headless browser, or subprocess-based renderer
