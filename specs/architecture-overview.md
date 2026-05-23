---
title: Slideforge Architecture Overview
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Slideforge Architecture Overview

> Source: §3, §4, and §6 (Phased Implementation Plan) from PROJECT-SEED.md.
> NOTE: §6 phased plan is superseded by the scope expansion in decisions-applied.md §11.A.
> The PRD (authored by product-owner in Phase 1) is the authoritative phase plan.
> The phase descriptions below are preserved verbatim for context, but the product-owner
> MUST re-scope them to reflect the Q4/Q5/Q7 scope expansions before they become stories.

---

## Section 3: Architectural Overview

### Pipeline (borrowed from Typst)

```
┌─────────┐    ┌──────────┐    ┌─────────┐    ┌─────────────────┐
│  Parse  │ -> │ Evaluate │ -> │ Layout  │ -> │ Export (PPTX/   │
│  (DSL → │    │  (AST →  │    │  (IR →  │    │  PDF/HTML)      │
│   AST)  │    │   IR)    │    │  slide  │    │                 │
└─────────┘    └──────────┘    │ frames) │    └─────────────────┘
                               └─────────┘
```

Each phase is a separate crate. Exporters are pluggable — adding a new output format means adding a new exporter crate, not modifying the layout engine.

This is the same architecture Typst uses, and the `office2pdf` project demonstrates it produces valid Office-format output.

### Crate layout

```
slideforge/
├── Cargo.toml                    # Workspace root
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── crates/
│   ├── slideforge/               # Main library crate
│   ├── slideforge-cli/           # Command-line binary
│   ├── slideforge-syntax/        # Parser + AST + lexer (chumsky-based)
│   ├── slideforge-eval/          # AST → IR evaluator
│   ├── slideforge-layout/        # IR → slide frames (positioning, sizing)
│   ├── slideforge-pptx/          # PPTX exporter (built on ooxmlsdk)
│   ├── slideforge-pdf/           # PDF exporter (Phase 4, may use Typst backend)
│   ├── slideforge-html/          # HTML exporter (Phase 4)
│   ├── slideforge-validate/      # Content validation rules
│   └── slideforge-ffi/           # C ABI + Python bindings (Phase 5)
├── examples/
│   ├── minimal.sf                # Simplest valid input
│   ├── all-slide-types.sf        # Reference deck with all 23 types
│   ├── incident-brief.sf         # Port of mss_metrics_leadership.py
│   └── templates/                # Brand templates (.pptx)
├── docs/
│   ├── dsl-reference.md          # Generated DSL syntax reference
│   ├── slide-types.md            # Each slide type with examples
│   ├── architecture.md           # This document, distilled
│   └── style-guide.md            # Voice and design conventions
└── tests/
    ├── snapshot/                 # Snapshot tests (rendered PPTX → fixture)
    ├── fixtures/                 # Expected output PPTX files
    └── integration/              # End-to-end CLI tests
```

### Pluggable exporters

The IR (intermediate representation) produced by the layout phase is exporter-agnostic. Each exporter consumes the IR and produces its output format. This means:

- The PPTX exporter is one crate (`slideforge-pptx`)
- Adding a PDF exporter is a new crate, not a fork
- The IR is the stable internal contract

See ADR-006 (IR shape) and ADR-010 (plugin/extension mechanism) for decisions on IR stability.

### Brand template handling

PowerPoint brand templates (`.pptx` files with master layouts) are loaded as the rendering foundation. The PPTX exporter:

1. Opens the template `.pptx`
2. Removes any default slides from the template
3. For each IR slide, dispatches to the slide-type renderer (using template's `slide_layouts[n]` for backgrounds and chrome)
4. Saves the resulting `.pptx`

This mirrors the Python tool's approach exactly. The template file is a hard input to the build — separate from the DSL source.

See ADR-001 for brand synthesis decisions (full `.toml`-to-PPTX generation).

---

## Section 4: Tech Stack

### Required dependencies

| Crate | Version | Purpose | Rationale |
|-------|---------|---------|-----------|
| `chumsky` | `^0.10` | Parser combinator library | Best error recovery in the Rust ecosystem; designed for DSLs and languages. 4.5k+ stars, active maintenance. |
| `ooxmlsdk` | `^0.6` | OOXML serialization | Most mature pure-Rust OOXML library. MIT licensed. Read AND write. Covers Office 2007 baseline through 2024 schema additions. |
| `serde` + `serde_derive` | latest | AST serialization | For tooling (LSP, formatters). |
| `miette` or `ariadne` | latest | Diagnostics rendering | High-quality error messages with source spans. `miette` for stable, `ariadne` is what chumsky integrates with natively. |
| `clap` | `^4` | CLI argument parsing | Standard. |
| `walkdir` | latest | File traversal | For multi-file projects. |
| `notify` | latest | Filesystem watcher | For `--watch` mode in CLI. |
| `tracing` | latest | Structured logging | Standard. |

### Optional / future dependencies

| Crate | Phase | Purpose |
|-------|-------|---------|
| `comemo` | Phase 4 | Incremental compilation (Typst's memoization library) |
| `typst` | Phase 4 | PDF backend (mirror office2pdf's approach if we want PDF) |
| `pyo3` | Phase 5 | Python FFI bindings |
| `napi-rs` | Phase 5 | Node.js bindings (if needed) |

### Rust edition and toolchain

- **Edition:** 2024 (stabilized in Rust 1.85, Feb 2025)
- **MSRV:** Rust 1.85
- **Toolchain config:** `rust-toolchain.toml` pinning stable
- **Lints:** `#![warn(missing_docs)]` on public APIs; `clippy::pedantic` enabled

NOTE: See ADR-007 (proposed) for MSRV and ooxmlsdk version policy. Preflight flags a potential MSRV bump from 1.85 to 1.88 as HIGH severity. Do NOT start any ooxmlsdk integration story until ADR-007 is resolved.

### Why not these alternatives

| Considered | Rejected because |
|-----------|------------------|
| `pest` (parser) | No native error recovery; external grammar files less flexible than chumsky combinators |
| `nom` (parser) | Designed for binary/protocol parsing; weaker for language DSLs |
| `lalrpop` (parser) | LR(1) limits expressiveness; chumsky's error recovery is the differentiator |
| `openxml-office` (OOXML) | AGPL-3.0 license — commercial use requires sponsorship. Disqualifying. |
| `office_oxide` (OOXML) | Extraction/parsing only; no write support |
| `ppt-rs` (OOXML) | Direct port of python-pptx, but immature and limited scope |
| Roll our own OOXML | Premature reinvention; ooxmlsdk has years of generator work behind it |
| Markdown as the DSL | Too constrained for our composed slide types (e.g., `metric_tree` needs hierarchy; markdown lists fight this) |
| Embedded Rust DSL (macros) | Locks users into Rust code authoring; we want non-developers to write slide files |

---

## Section 6: Implementation Plan (Phased) — SUPERSEDED

> WARNING: This phased plan is superseded by the scope expansion in decisions-applied.md §11.A.
> The PRD (authored by product-owner in Phase 1) is the authoritative phase plan.
> The Q4/Q5/Q7 decisions add ~2x scope vs. these original phase descriptions.
> Do NOT transcribe this section verbatim into the PRD.

### Phase 0: Project scaffolding

**Deliverables:**
- Workspace `Cargo.toml`
- All crate skeletons with `lib.rs` stubs
- `LICENSE-MIT`, `LICENSE-APACHE`, `README.md`
- CI workflow (`.github/workflows/ci.yml`): `cargo fmt`, `cargo clippy`, `cargo test`
- `rust-toolchain.toml` pinning Rust 2024 edition

**Acceptance:** `cargo build --workspace` succeeds. `cargo test --workspace` runs (zero tests is OK).

### Phase 1: Minimal end-to-end pipeline

**Deliverables:**
- `slideforge-syntax`: chumsky parser for two slide types only (`title`, `content`)
- `slideforge-eval`: AST → IR for those two types
- `slideforge-layout`: minimal layout (just slide positioning, no fancy shapes)
- `slideforge-pptx`: PPTX exporter using `ooxmlsdk` — emits a valid `.pptx` from IR
- `slideforge-cli`: `slideforge build input.sf` produces `input.pptx`

**Acceptance:**
- A 3-slide `.sf` file (title + content + end) compiles to a valid `.pptx`
- Opening the result in PowerPoint shows recognizable content
- All tests pass (snapshot test of the rendered XML)

### Phase 2: All slide types

**Deliverables:**
- All 23 slide types implemented (parser + IR + layout + exporter)
- `examples/all-slide-types.sf` — reference deck exercising every slide type
- Snapshot tests for each slide type's XML output

**Acceptance:**
- All examples render to visually-equivalent `.pptx` files vs. the Python reference output
- Pixel-perfect parity is not required; semantic and brand-consistent output is required
- Snapshot tests pass

### Phase 3: Brand template loading and validation

**Deliverables:**
- `slideforge-pptx` opens an existing `.pptx` brand template and renders into it (matching Python tool's behavior)
- `slideforge-validate` implements all validation rules from `presentation-system.md`
- Validation warnings include source spans
- Sample brand template in `examples/templates/`

**Acceptance:**
- `slideforge build mss_metrics_leadership.sf --template 1898.pptx` produces output indistinguishable from the Python tool's current output
- Validator catches all known issue patterns from the Python reference
- Diagnostic output uses `miette`/`ariadne` for nice error rendering

### Phase 4: Polish

**Deliverables:**
- Incremental compilation via `comemo` (re-render only changed slides)
- `slideforge watch` for live preview
- `slideforge fmt` for canonical DSL formatting
- Optional: PDF exporter (using Typst backend or direct PDF generation)
- Optional: HTML exporter (for web preview)
- Generated documentation (`docs/dsl-reference.md` generated from chumsky grammar + slide type metadata)

**Acceptance:**
- Watch mode produces sub-second refresh on a 25-slide deck after first build
- Formatter produces deterministic output (round-trips through itself unchanged)
- PDF output (if implemented) is visually consistent with PPTX output

### Phase 5: Distribution and bindings

**Deliverables:**
- Pre-built binaries for macOS (arm64 + x86_64), Linux (x86_64), Windows (x86_64)
- Homebrew tap
- `cargo install slideforge-cli`
- Optional: `pip install slideforge` Python bindings via `pyo3`
- Optional: `npm install @slideforge/cli` wrapper

**Acceptance:**
- Cross-platform binary releases published to GitHub Releases
- Homebrew formula installs and runs the binary
- Python bindings successfully render a deck from a Python script
