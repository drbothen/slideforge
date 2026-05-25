---
document_type: architecture-section
section: purity-boundary-map
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Purity Boundary Map

## Definition

**Pure Core:** Deterministic functions with no side effects. Input data in, result
out. No I/O, no database, no network, no global mutable state. Formal verification
(Kani) operates here.

**Effectful Shell:** I/O, network, file system, process lifecycle, mutable global
state. Tested via integration and snapshot tests; not formally proven.

## Boundary Classification

| Crate | Classification | Boundary Note |
|-------|---------------|--------------|
| slideforge-plugin-api | Pure types | Trait definitions only; no implementation |
| slideforge-types | Pure types | Deck, LaidOutDeck, Brand, Value; no I/O |
| slideforge-syntax | Pure core | Lexer + parser: `&str` in, `Ast` out. Kani-amenable. |
| slideforge-eval | Pure core | Expressions evaluated from Deck data structures. No I/O. |
| slideforge-validate | Pure core | Deck validation: `&Deck` + `&Brand` in, `Vec<Diagnostic>` out. |
| slideforge-layout | Pure core | `(&Deck, &Brand)` in, `LaidOutDeck` out. Pure function. |
| slideforge-charts | Pure core | `ChartSpec` in, `SvgData` out. plotters is pure. |
| slideforge-math | Pure core | `&str` (LaTeX) in, `String` (MathML/OMML) out. |
| slideforge-brand | Effectful shell | `brand.toml` file read is I/O; `BrandConfig::synthesize()` is isolatable pure function |
| slideforge-pptx | Effectful shell | Serialization to bytes is pure; ZIP writing to disk is effectful |
| slideforge-docx | Effectful shell | Same pattern as slideforge-pptx |
| slideforge-pdf | Effectful shell | Font file reads are I/O; coordinate mapping and structure tree are pure |
| slideforge-html | Effectful shell | HTML page generation, file writing — effectful |
| slideforge-preview | Effectful shell | axum TCP server, WebSocket, file serving — fully effectful |
| slideforge-data | Effectful shell | All data sources: file reads, HTTP requests — fully effectful |
| slideforge-diagrams | Effectful shell | fontdb system font scan (mmap, I/O on cold start) |
| slideforge-package | Effectful shell | Git operations, sf.lock file writes — fully effectful |
| slideforge-config | Effectful shell | File reads for config cascade — effectful |
| slideforge | Effectful shell | Registry assembly, pipeline orchestration — effectful |
| slideforge-cli | Effectful shell | Process lifecycle, stdout/stderr, watch mode — fully effectful |

## Pure Core Isolation Rules

1. Pure core crates MUST NOT import `std::fs`, `std::net`, `std::io::BufWriter`,
   `tokio`, `reqwest`, `notify`, or any crate whose primary purpose is I/O.
2. All I/O must be performed in effectful crates and the result passed as a
   value into pure core functions.
3. The pattern for brand synthesis: effectful code reads `brand.toml` → pure
   `BrandConfig::synthesize(&config)` → effectful code writes template bytes.

## Isolatable Pure Subfunctions in Effectful Crates

Some effectful crates contain pure subsets eligible for Kani proof:

| Crate | Pure Subfunction | Kani Candidate |
|-------|-----------------|---------------|
| slideforge-pdf | `emu_to_pt()`, `ir_y_to_pdf_y()` | Yes (VP-006) |
| slideforge-pptx | `opc_postprocess::inject_default_entries()` | Proptest |
| slideforge-diagrams | `normalize_svg_for_pptx()` (usvg, pure after font load) | Proptest |
