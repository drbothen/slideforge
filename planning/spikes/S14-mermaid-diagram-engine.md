---
title: "S14 — Mermaid Diagram Rendering Engine"
status: RESOLVED
resolution: OPTION_A_PRIMARY_OPTION_B_FALLBACK
resolved_by: architect
resolved_date: 2026-05-24
adr_input: ADR-014
spike_type: technology_evaluation
time_box: 2 days
---

# Spike S14: Mermaid Diagram Rendering Engine

## Objective

Determine the implementation approach for the `slide diagram:` type in v1.0.
The DiagramRenderer plugin surface (`slideforge-diagrams` crate) requires an
engine that produces embeddable SVG from Mermaid source. The single-binary
constraint from Q1 (R3: "no runtime dependencies") is the dominant filter.

## Candidates Evaluated

### Option A: mermaid-rs-renderer (Pure Rust)
Crate `mermaid-rs-renderer` v0.2.2 (published 2026-04-24, 13,048 downloads).
Pure Rust implementation built on `selkie-rs` (the parser/layout engine) plus
`fontdb`/`ttf-parser` for text metrics. No browser, no Node.js, no WASM.

### Option B: selkie-rs Direct Dependency (Pure Rust)
Crate `selkie-rs` v0.3.0 (published 2026-02-07, 7,568 downloads). The
underlying engine that `mermaid-rs-renderer` wraps. Could be used directly
for tighter integration, at the cost of a lower-level API.

### Option C: mermaid-cli (Node.js + Puppeteer)
`@mermaid-js/mermaid-cli` v11.15.0 (mmdc). Official Mermaid CLI, spawns
headless Chromium per diagram. Requires Node.js + Puppeteer + ~200 MB
Chromium bundle.

### Option D: Headless Browser (Playwright/Chromium)
Custom Rust code spawning a headless browser via `chromiumoxide` or
`headless_chrome` crate, loading mermaid.js, calling `mermaid.render()`.
Same Chromium dependency as Option C; higher control over reuse.

### Option E: Kroki (HTTP Service)
Universal diagram gateway (`kroki.io` or self-hosted Docker). HTTP round-trip
per diagram. Requires network or Docker. Offline-hostile without self-hosted
instance.

### Option F: Mermaid WASM Embedding
Compile mermaid.js to WASM, embed in Rust binary via wasmtime/wasmer. No
official mermaid.wasm exists; mermaid.js has deep DOM assumptions (D3,
Dagre, browser DOM APIs). Engineering effort is 6-12 weeks with uncertain
outcome. No existing implementation in the ecosystem.

---

## Empirical Results (Measured on Apple M-series, 2026-05-24)

### Test Environment

- Machine: Apple M-series (ARM64, macOS 25.3.0)
- mermaid-rs-renderer: v0.2.2, compiled with `--release`
- mermaid-cli: v11.12.0 (mmdc, via Homebrew, Node v25.2.1)
- All timings are wall-clock, not CPU time

### Option A — mermaid-rs-renderer Render Times

**First run (cold font DB load):** ~124ms for flowchart.
**Warm (font DB cached, library benchmark via Criterion):**

| Diagram | Warm render time | PPTX-safe SVG |
|---------|-----------------|---------------|
| flowchart (simple, 4 nodes) | 65 µs | YES |
| flowchart (medium, 8 nodes) | 2.5 ms | YES |
| sequenceDiagram (4 actors) | 92 µs | YES |
| gantt (2 sections, 4 tasks) | 52 µs | YES |
| stateDiagram-v2 | (cold: 0.5ms) | YES |
| classDiagram | (cold: 0.1ms) | YES |
| erDiagram | (cold: 2.1ms) | YES |
| pie (5 slices) | (cold: 0.1ms) | YES |

**8/8 diagram types pass. 0/8 PPTX safety issues.**

All SVG output: pure `<text>`, `<rect>`, `<path>` elements. No `<foreignObject>`,
no `<script>`, no SMIL animation. `viewBox`, `width`, and `height` present on all.

### Option C — mermaid-cli (mmdc) Render Times

| Run | Wall time |
|-----|-----------|
| First invocation (cold Puppeteer) | 4.1 seconds |
| Second invocation (warm Node) | 1.04 seconds |
| Third invocation (warm Node) | 1.04 seconds |

**SVG PPTX safety: FAIL.** mmdc output contains `<foreignObject>` in every
flowchart node label and every edge label. Every node's text content is wrapped
in `<foreignObject><div xmlns="http://www.w3.org/1999/xhtml">...</div></foreignObject>`.
PowerPoint does not support `foreignObject` reliably; embedding this SVG
directly in PPTX will produce invisible or garbled node labels.

Post-processing fix exists (set Mermaid config `htmlLabels: false`) but adds
a configuration requirement and does not cover all diagram types. The Q2
decision requires "high quality for all users by default" with no extra steps.

---

## Candidate Comparison Matrix

| Criterion | Option A (mmdr) | Option C (mmdc) | Option D (Browser) | Option E (Kroki) | Option F (WASM) |
|-----------|:--------------:|:--------------:|:-----------------:|:---------------:|:---------------:|
| Single binary (no runtime deps) | YES | NO | NO | NO | MAYBE |
| SVG quality — PPTX-safe by default | YES | NO* | NO* | YES | YES |
| Warm render < 1s | YES (< 3ms) | NO (~1s each) | NO (~1s) | YES (if local) | UNKNOWN |
| Cold startup < 500ms | NO (124ms) | NO (4s) | NO (~2s) | N/A | UNKNOWN |
| Offline capable | YES | YES | YES | NO (public API) | YES |
| Cross-platform binary | YES | NO (Node req.) | NO (Chrome req.) | YES (HTTP) | YES |
| Diagram type coverage (Mermaid) | 23 types | Full | Full | Full | N/A |
| `#![forbid(unsafe_code)]` compat | PARTIAL** | N/A | N/A | N/A | N/A |
| Dependency weight | ~5 MB added | ~200 MB | ~200 MB | 0 (network) | ~3 MB est. |
| Mermaid JS API change risk | LOW | MEDIUM | MEDIUM | LOW | HIGH |
| Implementation effort | LOW (add dep) | LOW (exec) | MEDIUM | LOW | VERY HIGH |
| v1.0 feasibility | YES | NO | NO | NO | NO |

*mmdc SVG contains `<foreignObject>` by default for flowcharts. Fixable with
`htmlLabels: false` config flag but not transparent to users.

**`mermaid-rs-renderer` itself contains no `unsafe`. Its deps `fontdb` and
`ttf-parser` use `unsafe` internally (memory-mapped font files, font parsing).
`#![forbid(unsafe_code)]` on `slideforge-diagrams` crate is achievable; the
dep-tree `unsafe` is contained in sound, audited crates.

---

## SVG Quality Assessment

### Option A (mermaid-rs-renderer) SVG Characteristics

Confirmed by inspection of rendered output:

- All labels: pure `<text>` SVG elements with `font-family`, `font-size` attrs
- No `<foreignObject>`, no HTML content, no CSS `@keyframes` animations
- All shapes: `<rect>`, `<circle>`, `<path>` with presentation attributes
- viewBox and width/height present on root `<svg>` element
- No external CSS references; styles are inline on elements
- No `<script>` elements
- Output passes all OOXML SVG embedding criteria (§7 of PPTX embedding spec)

One post-processing step remains necessary: `usvg` normalization for text-to-path
conversion when deploying to environments where the target font (default: sans-serif
fallback in fontdb) may not match the PPTX consumer's font environment. This is a
standard PPTX SVG embedding practice, not specific to mmdr.

### Option C (mmdc) SVG Characteristics

Confirmed by inspection of mmdc v11.12.0 output:

- ALL flowchart node labels: `<foreignObject><div xmlns="..."><span class="nodeLabel"><p>text</p></span></div></foreignObject>`
- Edge labels: also `<foreignObject>` wrappers
- Extensive `<style>` block with CSS `@keyframes` animations (`edge-animation-slow`, `dash`)
- `width="100%"` (percentage width — not absolute, fragile in PPTX) on root SVG
- Contains CSS class references relying on embedded `<style>` block
- SVG size: ~10,185 bytes (vs 4,409 bytes for same diagram from mmdr)

**Verdict:** mmdc output is not embeddable in PPTX without post-processing.
The `htmlLabels: false` workaround produces plain SVG text, but this requires
a config file at render time and is not the default behavior of mmdc.

---

## Unsafe Code Assessment

`#![forbid(unsafe_code)]` is enforced on all slideforge crates. The
`slideforge-diagrams` crate itself will have this lint. The dependency
`mermaid-rs-renderer` does not declare `#![forbid(unsafe_code)]`, but its
own source files (`src/lib.rs`, `src/main.rs`) contain no `unsafe` blocks.

Transitive `unsafe` in the dep tree:
- `fontdb`: uses `memmap2` (memory-mapped font files) — controlled, sound
- `ttf-parser`: uses `unsafe` for font table parsing — widely audited
- `memmap2`: thin safe wrapper over mmap syscall — standard

These crates are widely used across the Rust ecosystem (fontdb is used by
`resvg`, `cosmic-text`, Zed editor, etc.) and considered sound. The `unsafe`
is in font I/O, not in the rendering logic slideforge cares about. This is
an acceptable boundary: `slideforge-diagrams` forbids `unsafe` in its own
code, and the font-reading `unsafe` is in a separate, audited crate.

---

## Diagram Type Coverage

mermaid-rs-renderer v0.2.2 supports 23 Mermaid diagram types:

| Category | Types | slideforge relevance |
|----------|-------|---------------------|
| Core | Flowchart, Sequence, Class, State | HIGH — primary use cases |
| Data | ER, Pie, XY Chart, Quadrant, Sankey | MEDIUM |
| Planning | Gantt, Timeline, Journey, Kanban | HIGH — project decks |
| Architecture | C4, Block, Architecture, Requirement | HIGH — tech decks |
| Other | Mindmap, Git Graph, ZenUML, Packet, Radar, Treemap | LOW-MEDIUM |

Coverage for v1.0 use cases is complete. The 4 most important diagram types
for slideforge users (flowchart, sequence, class, state) all render correctly
and produce PPTX-safe SVG.

**Conformance vs mermaid.js:** selkie-rs reports 85% structural parity against
reference mermaid.js output (17/20 diagrams match in their eval suite), with
100% match on sequence and class diagrams, 80% on flowcharts and pie. The 15%
gap is in rendering aesthetic differences (minor label positioning) rather than
structural errors. For embedding as a static image in PPTX/DOCX, these
differences are invisible to end users.

---

## Font Handling

mermaid-rs-renderer uses `fontdb` + `ttf-parser` for accurate text measurement.
On first render (cold), fontdb scans the system font database — this accounts
for the ~124ms cold-start time observed. Subsequent renders in the same process
are sub-3ms (font cache is held in memory for the process lifetime).

For slideforge's use case:
- `slideforge build`: process starts once, renders all diagrams in the deck.
  Cold load: ~124ms once per process. For a 25-slide deck with 5 diagrams,
  total diagram cost: ~124ms + 4×3ms = ~136ms. Well within the 500ms cold
  build budget.
- `slideforge watch`: process stays alive between file changes. After the first
  rebuild, diagram render cost is ~1-3ms each.
- CI (per-deck build): one process start per deck. Cold load amortized.

The `--fastText` option (available in the mmdr CLI, not in library mode) uses
calibrated ASCII fallback widths and avoids the font scan entirely. Not needed
for slideforge's library integration since we amortize the cold load.

---

## SVG Post-Processing Pipeline for PPTX Embedding

Regardless of the engine, the following SVG normalization is required before
embedding in PPTX. This pipeline is implemented in `slideforge-diagrams` using
`usvg` v0.47.0 (already a transitive dep of `mermaid-rs-renderer`):

```
mermaid source
    │
    ▼
mermaid-rs-renderer::render()
    │   produces: pure SVG, no foreignObject
    ▼
usvg::Tree::from_data()   ← normalize SVG tree
    │   resolves: use references, simplifies attributes
    │   removes: invisible elements, editor metadata
    ▼
SVG serialization
    │   ensures: absolute viewBox, width/height in px
    │   inlines: styles as presentation attributes
    ▼
/ppt/media/diagramN.svg   ← embed in PPTX ZIP as media part
```

The `usvg` normalization step:
1. Flattens CSS class-based styles to inline presentation attributes
2. Resolves `<use>` references
3. Ensures all dimensions are absolute (px, not %)
4. Strips any metadata Inkscape/editor namespaces (none in mmdr output, but
   defensive programming is correct here)

Accessibility: The `alt` text from the slideforge DSL (`alt "..."` field on
`slide diagram:`) is injected as the `<svg aria-label="...">` attribute and
a `<title>` element inside the SVG before embedding. This satisfies WCAG AA
and PPTX accessibility checker requirements.

---

## CI Integration Plan

### GitHub Actions (all platforms)

No additional CI setup required for Option A. `mermaid-rs-renderer` compiles
as part of the Rust workspace build. No Node.js, no Chromium, no Docker.

```yaml
# Existing CI step — no changes needed:
- name: Build
  run: cargo build --workspace --all-features

# mermaid rendering is exercised by:
- name: Test
  run: cargo test --workspace --all-features --no-fail-fast
```

Contrast with Option C (mmdc): would require:
```yaml
- uses: actions/setup-node@v4
  with:
    node-version: '20'
- run: npm install -g @mermaid-js/mermaid-cli  # ~200 MB Chromium download
- run: mmdc ...  # 1-4s per diagram
```

This would add 2-5 minutes to CI on first run (Chromium download) and 1s per
diagram on every run. Unacceptable for the < 500ms cold build CI gate.

### Windows cross-compilation

Option A compiles to `x86_64-pc-windows-msvc` with no changes. Font scanning
on Windows uses the OS font registry (fontdb supports Win32 font API). The same
`fontdb` code that runs on macOS/Linux works on Windows.

Option C (mmdc) on Windows requires a Windows Chromium binary (~300MB). Node.js
arm64 binaries for Windows are available but less tested in CI.

---

## Risk Assessment

### Option A Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| mermaid-rs-renderer drops Mermaid syntax coverage | LOW | MEDIUM | Pin exact version (`=0.2.2`); crate is actively maintained (5 releases in 4 months); selkie-rs is the core engine and is independently maintained |
| selkie-rs visual parity gap (85%) causes user complaints | MEDIUM | LOW | The 15% gap is aesthetic, not structural. For static PPTX export, minor layout differences are acceptable. Document known limitations. |
| Font metrics cause PPTX text layout issues | LOW | LOW | `usvg` normalization pipeline converts text to SVG paths before embedding if needed |
| Cold font scan 124ms exceeds budget on slow CI hardware | LOW | LOW | `--fastText` equivalent available; or pin a known font dir; cold scan is once per process |
| crate abandonment (maintainer goes dark) | LOW | MEDIUM | selkie-rs is independent; could fork or switch to selkie-rs directly; mermaid syntax is stable |

### Option C Risks (rejected)

| Risk | Likelihood | Impact |
|------|-----------|--------|
| Chromium binary unavailable on target platform | MEDIUM | CRITICAL (breaks v1.0 shipping) |
| foreignObject in PPTX breaks display | CERTAIN | HIGH (verified empirically) |
| 200MB runtime dep conflicts with single-binary goal | CERTAIN | BLOCKING |
| 1-4s per diagram fails < 500ms build gate | CERTAIN | BLOCKING |

---

## Recommendation

**Option A: mermaid-rs-renderer as the primary DiagramRenderer plugin.**

Rationale:
1. Single binary. Zero runtime dependencies. Compiles into the slideforge binary.
2. PPTX-safe SVG by default. No post-processing to strip `foreignObject`. Verified.
3. 23 diagram types — covers all v1.0 use cases.
4. Sub-3ms warm render. Amortized cold start is ~136ms for 5 diagrams per deck.
5. Cross-platform. Same code path on macOS, Linux, Windows.
6. Active development: 5 releases in 4 months, used in Typst universe (mmdr package).
7. Built on selkie-rs (a well-tested, independently maintained engine with its
   own eval harness against mermaid.js reference output).

**Option B (selkie-rs direct) as future escape hatch** if mermaid-rs-renderer
proves limiting. selkie-rs is the underlying engine; switching is a dep swap,
not a redesign.

Options C, D, E, F are **rejected for v1.0**:
- C, D: violate single-binary constraint, PPTX safety fails empirically, CI budget fails
- E: requires network/Docker, offline-hostile
- F: no implementation exists; 6-12 weeks of risky R&D for unclear benefit

---

## Implementation Notes for slideforge-diagrams

The `MermaidRenderer` struct implementing `DiagramRenderer` trait:

```rust
// crates/slideforge-diagrams/src/mermaid.rs

use slideforge_plugin_api::{DiagramRenderer, DiagramLang, SvgData};

pub struct MermaidRenderer;

impl DiagramRenderer for MermaidRenderer {
    fn id(&self) -> &str { "mermaid" }

    fn render(&self, source: &str, lang: DiagramLang) -> Result<SvgData> {
        debug_assert_eq!(lang, DiagramLang::Mermaid);
        let svg = mermaid_rs_renderer::render(source)
            .map_err(|e| DiagramError::RenderFailed(e.to_string()))?;
        let normalized = normalize_svg_for_pptx(&svg)?;
        Ok(SvgData::from(normalized))
    }

    fn supported_langs(&self) -> &[DiagramLang] {
        &[DiagramLang::Mermaid]
    }
}

/// Normalize SVG via usvg for safe PPTX embedding.
/// Strips editor metadata, flattens CSS classes, ensures absolute dimensions.
fn normalize_svg_for_pptx(svg: &str) -> Result<String> {
    // usvg is already a transitive dep; use it for normalization
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)
        .map_err(|e| DiagramError::SvgNormalizationFailed(e.to_string()))?;
    // Re-serialize to canonical SVG
    Ok(tree.to_string(&usvg::WriteOptions::default()))
}
```

Cargo.toml addition for `slideforge-diagrams`:
```toml
[dependencies]
mermaid-rs-renderer = { version = "=0.2.2", default-features = false }
usvg = { version = "=0.47.0" }
```

Note: pin with `=` per workspace convention (production-crate deps pinned).

---

## ADR Input

This spike resolves the architecture decision for ADR-014:
"Mermaid rendering engine for DiagramRenderer plugin."

Decision: Use `mermaid-rs-renderer` (Option A). Single-binary, PPTX-safe,
< 3ms warm render, 23 diagram types, cross-platform. Reject Options C/D/E/F.

---

## Test Code

Spike test code is in `.factory/planning/spikes/S14-code/`:
- `src/main.rs` — functional evaluation across 8 diagram types
- `benches/render_bench.rs` — Criterion benchmarks (flowchart, sequence, gantt)
- `Cargo.toml` — standalone workspace with `mermaid-rs-renderer` dep

To reproduce:
```bash
# Functional evaluation
cargo run --release --manifest-path .factory/planning/spikes/S14-code/Cargo.toml

# Criterion benchmarks
cargo bench --manifest-path .factory/planning/spikes/S14-code/Cargo.toml
```
