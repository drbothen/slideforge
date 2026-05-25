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

# BC-4.03.002: PDF Produced via pdf-writer + krilla + SlideTagEngine (No Chrome/Headless)

## Description

The slideforge PDF exporter must use the pure-Rust pdf-writer + krilla stack with a
custom SlideTagEngine for PDF/UA-1 structure tree generation. Chrome headless, Puppeteer,
wkhtmltopdf, and any other browser-based pipeline are explicitly rejected. This decision
is binding (Spike S2 resolution): browser-based pipelines produce unreliable PDF/UA-1
output for absolute-positioned layouts and would make single-binary distribution
infeasible. This BC specifies the implementation constraint, not just the output.

## Preconditions

1. The `slideforge-pdf` crate is compiled against pdf-writer and krilla (not a headless
   browser library).
2. The `Cargo.toml` of `slideforge-pdf` does NOT list chromium, headless-chrome,
   puppeteer-rs, or wkhtmltopdf as dependencies.
3. A valid `LaidOutDeck` IR is available for export.

## Postconditions

1. A valid PDF file is produced without invoking any external browser process.
2. The PDF is produced as a single-pass operation within the `slideforge` binary
   (no subprocess spawning of chrome, headless, or wkhtmltopdf).
3. The produced PDF contains a complete PDF/UA-1 structure tree (verified by BC-4.03.001).
4. Charts (SVG from plotters) are embedded as vector paths after usvg normalization —
   not rasterized.
5. The `slideforge-pdf` crate compiles on the CI target matrix (Linux, macOS, Windows)
   without requiring a browser installation.

## Invariants

1. Chrome/Chromium is NEVER a runtime dependency of the PDF export path. (Spike S2
   explicit rejection)
2. The `SlideTagEngine` is the sole mechanism for PDF/UA-1 structure tree generation.
3. Font subsetting uses the `subsetter` crate (same as used by Typst) — no system font
   tooling dependency.
4. The PDF export stack is fully contained within the Rust workspace — no FFI to
   C-based PDF libraries (e.g., libharu, Cairo) is required in v1.0.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CI environment has no Chrome installed | Build succeeds; PDF export succeeds; no PATH-based chrome lookup |
| EC-002 | Chart SVG has complex clip paths | usvg normalizes clip paths to compatible paths before embedding |
| EC-003 | PDF export on Windows (target: x86_64-pc-windows-msvc) | Compilation succeeds; pdf-writer is pure Rust; no Windows-specific code |
| EC-004 | Concurrent PDF exports (thread-safety) | pdf-writer is thread-safe within one export; no global state |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `cargo build --target x86_64-unknown-linux-gnu -p slideforge-pdf` | Compilation succeeds; no chrome dependency in Cargo.lock | happy-path |
| 3-slide deck exported to PDF | PDF produced; no subprocess spawned (verified via process tracing) | happy-path |
| Chart slide exported to PDF | PDF embeds SVG as vector paths; not rasterized (verify with `pdfimages -list`) | happy-path |
| `Cargo.lock` of slideforge-pdf | No `chromium`, `headless-chrome`, or `wkhtmltopdf` entries | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | pdf-writer and krilla are the only PDF-writing crates in Cargo.lock | unit test: parse Cargo.lock, assert no browser-PDF deps |
| VP-TBD | PDF export does not spawn child processes | integration test: run with strace/dtrace; assert zero exec syscalls during export |
| VP-TBD | Chart embedded as vector paths (not raster image) | integration test: pdfimages -list; assert zero raster images for chart-only deck |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 |
| Capability Anchor Justification | CAP-017 ("PDF, HTML, and Web Preview Export") per capabilities.md §CAP-017 — "tagged, PDF/UA-1 compliant via [direct backend]" (Spike S2 resolved the backend as pdf-writer + krilla, not Chrome) |
| L2 Domain Invariants | DI-014 (PDF output must be tagged PDF/UA-1 compliant — browser-based pipelines cannot guarantee this for absolute-positioned layouts) |
| Architecture Module | slideforge-pdf crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.001 — composes with (this BC specifies the stack; BC-4.03.001 specifies the compliance target)
- BC-4.03.005 — composes with (coordinate mapping is part of the pdf-writer + krilla implementation)
- BC-5.02.002 — related to (pdf exporter is a bundled plugin; must use plugin trait API)

## Architecture Anchors

- `architecture/export-architecture.md` — pdf-writer + krilla + SlideTagEngine design (Spike S2)

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
