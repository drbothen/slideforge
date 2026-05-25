---
document_type: architecture-section
section: tooling-selection
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Tooling Selection

## Core Toolchain

| Component | Selected Tool | Version | Spike | Rationale |
|-----------|--------------|---------|-------|-----------|
| Language | Rust stable | per rust-toolchain.toml | — | Single-binary, safe, performance |
| Parser combinator | chumsky | =0.10.1 | S4 | Error accumulation is first-class design goal; 14/14 tests pass; 89µs for 25 slides |
| PPTX generation | ooxmlsdk | =0.6.1 | S1 | 55/57 capability checks pass; typed API; actively maintained |
| PDF generation | pdf-writer + krilla | =0.14.0 / =0.6.0 | S2 | Only path to PDF/UA-1 in single-binary constraint |
| SVG charts | plotters | =0.3.7 | — | Pure Rust, all-formats SVG output |
| Diagram rendering | mermaid-rs-renderer | =0.2.2 | S14 | Single binary, PPTX-safe SVG, < 3ms warm, 23 diagram types |
| SVG normalization | usvg | =0.47.0 | S14, S2 | PPTX embedding normalization; by Typst team |
| Font subsetting | subsetter | =0.2.0 | S2 | PDF font embedding; by Typst team, Apache-2.0 |
| Web server | axum | =0.8.1 | — | Ergonomic, tokio-native |
| Error display | miette | =7.2 | — | Source-pointer rendering with `fancy` feature |
| CLI framework | clap | =4.5 | — | Derive macros, help generation |
| Serialization | serde | =1.0 | — | Universal; derive macros |
| File watching | notify | =6.1 | — | Cross-platform fs events |
| Tracing | tracing | =0.1 | — | Structured, async-safe |

## Test Tooling

| Tool | Purpose | Phase |
|------|---------|-------|
| cargo-nextest | Test runner (parallel, per-test timing) | 3+ |
| insta | Snapshot tests (AST, IR, XML, SVG output) | 3+ |
| proptest | Property-based tests | 3+ |
| cargo-fuzz | Fuzz harness (parser, eval) | 6 |
| cargo-mutants | Mutation testing | 6 |
| Kani | Formal model checking (pure-core proofs) | 6 |

## CI Tooling

| Tool | Trigger | Gate |
|------|---------|------|
| cargo fmt --check | Every PR | Blocking |
| cargo clippy::pedantic | Every PR | Blocking (0 violations) |
| cargo deny | Every PR | Blocking (CVE + license) |
| cargo audit | Every PR | Blocking (0 high/critical CVE) |
| RUSTDOCFLAGS=-D warnings cargo doc | Every PR | Blocking |
| insta (snapshot tests) | Every PR | Blocking |
| visual-diff.py (SSIM + PSNR) | Every PR | Blocking (0.99/35dB) |
| @axe-core/playwright | Every PR touching preview/HTML | Blocking (0 critical) |
| veraPDF --flavour ua1 | Every PR touching PDF | Blocking (0 violations) |
| criterion benchmarks | Every PR | Blocking if NFR-001/002 exceeded |

## Rejected Alternatives (Summary)

| Tool | Reason Rejected | Spike |
|------|----------------|-------|
| winnow | Error accumulation is opt-in, not first-class | S4 |
| pest | No built-in error recovery; single-error output | S4 |
| headless Chromium | 150-250 MB binary; single-binary-hostile | S2, S14 |
| printpdf | No PDF/UA-1 structure tree support | S2 |
| mermaid-cli (mmdc) | Node.js + 200 MB Chromium; foreignObject in SVG output | S14 |
| pa11y | Duplicate of axe-core; SVG ARIA false positives | S3 |
| lopdf | Too low-level for writing; no structural support | S2 |
| openxml (Rust) | Abandoned (2020); incomplete PML coverage | S1 |
