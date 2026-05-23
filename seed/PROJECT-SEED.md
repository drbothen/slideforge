---
title: Project Seed — Rust + DSL PowerPoint Generator
subtitle: All-in-one specification for an AI agent developer factory
version: 1.0
created: 2026-05-23
status: SEED — ready for agent factory ingest
license: MIT OR Apache-2.0 (dual)
---

# Project Seed: Rust + DSL PowerPoint Generator

> This is an all-in-one seed file. An AI agent developer factory should be able to read this document, ingest the attached reference materials in `./reference/`, and produce a functional v0.1 of the project without needing additional clarification.
>
> Where decisions remain open, they are explicitly called out in **Section 11: Open Questions**.

---

## Section 1: Project Identity

### Name candidates

Pick one. Each is available as a crate name on crates.io as of 2026-05-23 (verify before commitment). Rationale included for each.

| Candidate | Rationale | Why it fits |
|-----------|-----------|-------------|
| **Slideforge** | "Forging slides" — evokes deliberate craftsmanship, technical tooling | Strongest: memorable, evocative, suggests both precision and power |
| **Slidesmith** | Smithing slides — artisan register | Similar to Slideforge, slightly softer tone |
| **Decktype** | Echoes Typst (whose architecture we borrow heavily) and "deck typing" | Honest about the architectural debt; might confuse users who know Typst |
| **Brandeck** | "Branded deck" — emphasizes brand-template-first design | Distinguishes from Marp/Slidev (markdown-first, generic style) |
| **Forgedeck** | Variant of Slideforge focused on the deck artifact | Stronger noun than Slideforge but loses "craft" connotation |

**Recommendation:** `slideforge`. The single-word name reads naturally as a CLI binary (`slideforge build my-deck.sf`), works as a library crate name, and clearly signals "tool for building slides." The rest of this document uses `slideforge` as a placeholder — substitute the chosen name throughout.

### Mission

Build a presentation generator that:

1. **Reads structured slide specifications** in a purpose-built DSL (not Python dicts, not YAML, not Markdown)
2. **Renders branded `.pptx` output** using a corporate brand template as the visual foundation
3. **Supports composable, opinionated slide types** (not freeform — slide types are well-defined patterns like `metric_tree`, `formula`, `weighted_composite`)
4. **Compiles fast and incrementally** so an interactive preview loop is feasible
5. **Reports errors helpfully** so non-developers can author slide files
6. **Distributes as a single binary** (no Python runtime, no LaTeX, no Office) plus optional library and FFI bindings

### Tagline

> *"Branded presentations from a DSL, not a deck of dicts."*

### License

MIT OR Apache-2.0 dual-license. Permissive, compatible with corporate use, standard for the Rust ecosystem.

### Target users

- Engineering teams generating regular branded reports / briefs from data
- Incident response teams producing executive briefs from structured event data
- Internal tools that need deterministic, version-controllable presentation output
- Anyone who currently builds slides programmatically via `python-pptx` and wants better tooling

**Non-goal:** Replacing PowerPoint as a manual authoring tool. We generate slides from specifications; we don't compete with WYSIWYG editors.

---

## Section 2: Why This Exists

### The reference implementation

A working Python implementation lives in `./reference/`:

- `build-incident-brief.py` (~1900 lines) — the builder engine with 23 slide type implementations
- `presentation-system.md` (~28 KB) — human-readable specification of all slide types, design rules, and conventions
- `_template.py` — data module template
- `mss_metrics_leadership.py` — full real-world example data module (25 slides)

The Python tool is **functionally complete** for current internal use. We are not rewriting because it's broken — we are rewriting because the Python data-module-as-DSL approach has limitations:

1. **No error recovery.** A malformed data module dies on the first exception; you can't preview partial decks.
2. **Python syntax is not the right DSL.** Authors fight Python's grammar (commas, dict syntax, indentation) instead of expressing slide content.
3. **No incremental compilation.** Every build re-renders the entire deck.
4. **No type safety.** Slide dicts accept arbitrary keys; typos silently produce wrong output.
5. **Single output target.** `.pptx` only. No PDF, HTML, preview formats.
6. **Distribution requires Python runtime + dependencies.** Awkward for users who don't have Python set up.
7. **No real editor support.** No syntax highlighting, autocomplete, or inline error feedback for slide files.

### What we're keeping

- **The 23 slide types and their visual designs.** Each is a well-considered pattern. The Rust implementation must achieve visual parity with the Python tool's output.
- **The brand template integration.** Slides load from a `.pptx` template that defines fonts, colors, and master layouts. Our brand identity lives in that template.
- **The data-driven philosophy.** Slide content stays separate from rendering logic.
- **The opinionated slide-type vocabulary** (not freeform shapes — composed patterns with semantic meaning).

### What we're changing

- **DSL replaces Python dicts.** Purpose-built syntax with helpful errors and editor support.
- **Compilation pipeline replaces direct rendering.** Parse → Evaluate → Layout → Export.
- **Incremental.** Re-render only changed slides.
- **Multi-target output.** PPTX first, PDF and HTML next.
- **Single-binary distribution.**

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

### Brand template handling

PowerPoint brand templates (`.pptx` files with master layouts) are loaded as the rendering foundation. The PPTX exporter:

1. Opens the template `.pptx`
2. Removes any default slides from the template
3. For each IR slide, dispatches to the slide-type renderer (using template's `slide_layouts[n]` for backgrounds and chrome)
4. Saves the resulting `.pptx`

This mirrors the Python tool's approach exactly. The template file is a hard input to the build — separate from the DSL source.

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

## Section 5: DSL Specification

### Syntax goals

1. **Readable to non-developers.** A program manager should be able to author a slide file without learning Rust.
2. **Composable.** Common patterns (colors, accents, takeaways) work the same across slide types.
3. **Editor-friendly.** Grammar amenable to syntax highlighting, autocomplete, jump-to-definition.
4. **Error-tolerant.** Typos and incomplete sections produce useful errors AND a partial AST.
5. **No mandatory escape hatches into Rust.** The DSL is sufficient by itself.

### Proposed syntax: indented blocks with explicit keywords

Indentation-significant (Python-like / YAML-like), with explicit slide-type keywords. Strings are double-quoted. Lists use repeated keys or bracketed shorthand.

```
metadata:
  incident_id "INC-2026-0320"
  title "Trivy Supply Chain Compromise"
  date "2026-03-19"
  author "Joshua Magady"
  output "incidents/.../INC-2026-0320-Brief.pptx"
  template "templates/1898-pptx/1898-Presentation-Template-V3.0-2026.pptx"

# A title slide
slide title:
  color blue
  title "Trivy Supply Chain Compromise"
  subtitle "INC-2026-0320  |  SEV-1  |  March 19-23, 2026"
  talk_track """
    Here's what happened. On March 19th, TeamPCP compromised the
    Trivy scanner used widely in CI/CD pipelines...
  """

# A content slide
slide content:
  title "Five Things Every MSSP Metric System Must Answer"
  bullets:
    - "Can we sell it?"
    - "Can we onboard it?"
    - bold "Can we detect accurately?"        # inline formatting
    - "Can we operate within SLA?"
    - "Can we do all this profitably?"
  takeaway "If our metrics can't answer all five, we're measuring the wrong things."

# A metric_tree slide (complex nested structure)
slide metric_tree:
  title "Executive Metric Tree"
  root:
    label "Retained Protected Revenue at Target Margin"
    color blue
  category:
    header "Growth Health"
    color blue
    items:
      - "ARR"
      - "MRR"
      - "Qualified Pipeline"
      - "Win Rate"
      - "Avg Deal Size"
      - "Profitable Bookings"
  category:
    header "Retention Health"
    color green
    items:
      - "Logo Retention"
      - "Net Revenue Retention"
      # ...
  footnote "Each leaf rolls up to one dimension."
  takeaway "Four dimensions. One North Star. No single metric carries the business."

# Section divider
slide divider:
  color purple
  title "Executive Health Dimensions"
  subtitle "Four rollups beneath the North Star"

# End slide
slide end:
  color blue
```

### Syntax features

- **Indentation-significant** for block structure (chumsky supports semantic indentation natively)
- **Triple-quoted strings** for multi-line content (talk tracks especially)
- **Repeated keyword blocks** for lists of complex items (multiple `category:` blocks)
- **Bracketed shorthand** for simple lists (`["one", "two", "three"]`)
- **Inline modifiers** (`bold "text"`, `italic "text"`)
- **Color names** as bareword keywords (no quotes needed): `blue`, `orange`, `purple`, `green`, `red`, `teal`, `gray`, `white`, `light_blue`, `light_gray`, `dark_gray`
- **Comments** with `#`
- **No imports** — single-file or multi-file slide projects use a `slideforge.toml` config

### File extension

`.sf` for slide files. `.sf.toml` for project config.

### Reserved keywords

```
metadata, slide, takeaway, talk_track, footnote,
title, content, two_column, content_stat, stat_callout,
stats_summary, highlight, highlight_boxes, split_contrast,
card_rows, severity_cards, numbered_actions, vertical_timeline,
horizontal_timeline, enhanced_table, table, status, progress_bar,
metric_tree, formula, weighted_composite, end, key_metrics,
divider,
true, false, none,
bold, italic, code,
```

### Color vocabulary

Brand colors are named identifiers, not hex codes (hex is reserved for advanced use):

```
blue, orange, purple, green, red, teal, dark_gray, gray, white,
light_blue, light_gray
```

The actual hex values are defined in the brand configuration, not in user DSL code. This is critical: users say "color blue," not "color #003766." The DSL stays brand-portable.

### Slide types (all 23 from Python reference)

Each slide type from the Python implementation must be supported with visual parity. The full slide type catalog and field reference is in `./reference/presentation-system.md`.

For each slide type, the agent factory should:

1. Read the Python builder function in `./reference/build-incident-brief.py` (e.g., `build_metric_tree_slide`)
2. Understand the visual output (consult `./reference/mss_metrics_leadership.py` for real usage examples)
3. Define the DSL syntax for that slide type (consistent with the patterns above)
4. Implement the IR representation
5. Implement the PPTX renderer

The 23 slide types:

| Type | DSL Keyword | Visual Pattern |
|------|-------------|----------------|
| Section divider | `title` (with `color`) or `divider` | Full-bleed blue/purple slide |
| Bullet content | `content` | Title + bullets |
| Two columns | `two_column` | Two columns of bullets |
| Content + stat | `content_stat` | Blocks left, stat card right |
| Stat callout | `stat_callout` | 2-6 stat cards |
| Stats + summary | `stats_summary` | Effort top + zero-result band |
| Single highlight | `highlight` | Callout box + supporting bullets |
| Two boxes | `highlight_boxes` | Two stacked full-width bands |
| Split contrast | `split_contrast` | Two panels with mini-timelines |
| Card rows | `card_rows` | Two-column checklist |
| Severity cards | `severity_cards` | Risk register with badges |
| Numbered actions | `numbered_actions` | Numbered cards with categories |
| Vertical timeline | `vertical_timeline` | Cascade timeline cards |
| Horizontal timeline | `horizontal_timeline` | Time bar with alternating cards |
| Enhanced table | `enhanced_table` | Table with colored row borders |
| Plain table | `table` | Standard branded table |
| Status dashboard | `status` | Label + badge pairs |
| Progress bar | `progress_bar` | Horizontal bar with items |
| Metric tree | `metric_tree` | Root → categories → leaves |
| Formula | `formula` | Equation + term cards |
| Weighted composite | `weighted_composite` | Stacked bar + legend |
| End slide | `end` (with `color`) | Closing slide |
| Stat callout alias | `key_metrics` | Alias for `stat_callout` |

### Validation rules

Same as the Python implementation:

- **Bullet length** > 85 characters at 18pt → warning
- **Bullet count** > 6 in a `content` slide → warning
- **Consecutive `content` slides** → warning
- **Cross-slide consistency** (counts in summary slides match content) → warning

All warnings include source spans pointing to the exact line/column in the DSL.

---

## Section 6: Implementation Plan (Phased)

Each phase produces a working artifact. The factory should pause after each phase for review.

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

---

## Section 7: Reference Materials

The `./reference/` directory contains the existing Python implementation and its specification. The agent factory should consult these for:

| Question | Reference file |
|----------|----------------|
| What does slide type X look like visually? | `presentation-system.md` (Section 3 has per-type descriptions) |
| How is slide type X rendered programmatically? | `build-incident-brief.py` (search for `def build_X_slide`) |
| What does a real complex deck look like in the existing tool? | `mss_metrics_leadership.py` (25 slides, all major types exercised) |
| What's the minimum data shape for a deck? | `_template.py` (shows METADATA + SLIDES structure) |
| What are the design rules (colors, sizes, layout)? | `presentation-system.md` Section 5 |
| What validation rules apply? | `build-incident-brief.py` (`validate_slides` function) and `presentation-system.md` Section 8 |

### How to use the reference

The Python code is the authoritative behavior reference, but **do not literally port it line-by-line**. The Python implementation has accumulated stylistic and structural choices that don't translate well to Rust. Specifically:

- **Do** match the visual output of each slide type
- **Do** preserve the brand color vocabulary and font sizes
- **Do** keep the same slide types and their core semantic meaning
- **Don't** carry forward Python's mutable-dict patterns (use Rust enums + structs)
- **Don't** mirror the Python builder's procedural style (use the Typst-style pipeline)
- **Don't** preserve Python-specific quirks like the `_parse_bullets` helper's string-prefix-based formatting (`"**bold**"`); use proper inline-formatting syntax in the DSL

### External references

These are not in `./reference/` but are essential reading for the implementation:

- **Typst architecture:** <https://github.com/typst/typst/blob/main/docs/dev/architecture.md>
- **chumsky documentation:** <https://docs.rs/chumsky/latest/chumsky/>
- **ooxmlsdk documentation:** <https://docs.rs/ooxmlsdk>
- **OOXML format specification:** <https://learn.microsoft.com/en-us/office/open-xml/open-xml-sdk> (Microsoft Learn covers the package concepts)
- **office2pdf as Typst-backend example:** <https://github.com/developer0hye/office2pdf>
- **rsslide for reference DSL-to-PPTX patterns:** <https://github.com/veltzer/rsslide>

---

## Section 8: Conventions

### Code style

- **`rustfmt`** with project config (`rustfmt.toml`): `edition = "2024"`, `max_width = 100`, `tab_spaces = 4`
- **`clippy::pedantic`** enabled with documented exceptions
- **Public APIs** documented with rustdoc; `#![warn(missing_docs)]` on public modules
- **No `unsafe`** in any crate (FFI bindings are the only exception, in `slideforge-ffi`)
- **No `unwrap()`** outside of tests and clearly-infallible main paths; use `?` and proper error types

### Error handling

- Crate-level error enums using `thiserror`
- Display impls for human-readable messages
- All parser/eval errors carry source spans (line, column, range)
- CLI errors render via `miette` for nice colored output with source pointers

### Testing

- **Unit tests** in each crate's `src/` (next to the code, in `#[cfg(test)] mod tests`)
- **Snapshot tests** for parser AST output and IR (use `insta` crate)
- **Integration tests** in `tests/` for end-to-end CLI behavior
- **Fixture tests** in `tests/fixtures/` — expected `.pptx` output files for diff comparison
- **CI runs all tests** on push/PR via GitHub Actions

### Documentation

- **`README.md`** — project overview, quick start, link to full docs
- **`docs/`** — multi-file documentation including DSL reference, slide type catalog, architecture deep-dive
- **Rustdoc** — published to `docs.rs/slideforge` automatically on crate publish
- **`CHANGELOG.md`** — Keep-a-Changelog format

### Release process

- **Semver:** `0.x` for pre-stable. `1.0` when DSL syntax is frozen and major slide types are stable.
- **Tags:** `vX.Y.Z` (git tags)
- **GitHub Releases** with auto-built binaries
- **crates.io** publishing for library crates
- **Homebrew tap** for the CLI binary

### Commit conventions

- Conventional Commits format: `<type>(<scope>): <description>`
- Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`, `ci`
- Scopes: `syntax`, `eval`, `layout`, `pptx`, `cli`, `validate`, `docs`, `ci`
- No AI attribution lines in commits (no `Co-Authored-By: Claude` etc.)

### Repository conventions

- Default branch: `main`
- Protected: PRs required for changes to `main`, CI must pass
- Issues use templates for: bug report, feature request, slide type proposal
- Discussions enabled for DSL syntax debates

---

## Section 9: Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| `ooxmlsdk` API changes during development | Medium | Low | Pin to a specific minor version; vendor a fork if needed |
| OOXML edge cases break brand template loading | High | Medium | Snapshot-test against real brand template; manual QA in PowerPoint and Keynote |
| DSL syntax churn frustrates early users | Medium | Medium | Mark v0.x as pre-stable; freeze syntax at v1.0; offer `slideforge fmt` to migrate |
| chumsky 0.10 has rough edges (it's a recent rewrite) | Medium | Low | Use chumsky 0.10+ patterns from Tao and other reference projects; fall back to nom if blockers emerge |
| Visual output doesn't match Python tool | High | Low | Snapshot-test rendered XML; manual visual review at each phase |
| Incremental compilation produces stale output | High | Low | Use `comemo` only after Phase 4; test thoroughly; provide `--no-incremental` escape hatch |
| Brand template is a moving target | Medium | Medium | Template is a separate input file; users update independently of slideforge |
| FFI bindings (Python, Node) drift from Rust API | Low | Medium | Generate bindings from a shared spec where possible; CI tests for each binding |

---

## Section 10: Success Metrics

The project is "done with v1.0" when:

1. **Visual parity**: The reference deck (`mss_metrics_leadership.py` ported to `.sf`) produces a `.pptx` indistinguishable from the Python tool's output to a reviewer who didn't know it was rebuilt.
2. **Performance**: A 25-slide deck builds in < 500ms cold, < 50ms incremental.
3. **Distribution**: A user can `brew install slideforge` and run a build without installing Rust, Python, or anything else.
4. **Documentation**: A developer with no prior context can author a working `.sf` file using only `docs/dsl-reference.md`.
5. **Editor support**: At least one editor (VS Code) has syntax highlighting for `.sf` files. Bonus: a basic LSP for autocomplete.

---

## Section 11: Open Questions

These decisions are NOT made by the agent factory. They require human input before or during implementation.

### Q1: Project name

The seed proposes `slideforge` but lists alternatives. The factory should NOT pick a name unilaterally. Confirm before scaffolding crates.

### Q2: Indentation-significant vs. brace-delimited DSL

The seed proposes indentation (YAML/Python-like). A brace-based syntax is also viable (and easier for some parsers). Confirm the choice before designing the grammar.

**Indented variant:**
```
slide title:
  color blue
  title "Some Title"
```

**Braced variant:**
```
slide title {
  color: blue,
  title: "Some Title",
}
```

### Q3: Multi-file projects?

Single `.sf` file per deck (Python tool's pattern) or composed across multiple files (`@include "slides/intro.sf"`)?

### Q4: Brand template as input vs. embedded?

The Python tool requires a `.pptx` template file. Should `slideforge` also accept just a `.toml` of brand colors and synthesize the template, or always require a `.pptx`?

### Q5: PDF and HTML exporters in v1.0 or post-v1.0?

Phase 4 lists these as optional. Confirm whether they're in scope for v1.0 or deferred to v1.x.

### Q6: Python binding parity?

If we provide Python bindings via `pyo3`, should they expose a Python-dict-based API for ease of migration from the existing `python-pptx`-based tool, or should they require the DSL?

### Q7: Live preview architecture?

The `slideforge watch` mode rebuilds on file change. Should it:
- Just rebuild the `.pptx` (user opens in PowerPoint)
- Render an HTML preview in a browser tab
- Embed a small GUI preview window
- Use the same approach Typst uses (web-based preview)

---

## Section 12: Agent Factory Instructions

This section is meta — instructions for how the agent factory should approach this seed.

### Recommended agent assignment

| Phase | Suggested agent team |
|-------|---------------------|
| Phase 0 | 1 scaffolding agent |
| Phase 1 | 1 parser agent + 1 PPTX exporter agent + 1 integration agent |
| Phase 2 | 1 agent per slide type (23 parallel), 1 review agent |
| Phase 3 | 1 brand template agent + 1 validator agent + 1 diagnostic-rendering agent |
| Phase 4 | 1 incremental compilation agent + 1 formatter agent + 1 doc-generation agent |
| Phase 5 | 1 release-engineering agent + 1 FFI agent per language |

Each agent should:
1. Read this seed in full before starting
2. Read the relevant reference files in `./reference/`
3. Produce a self-contained PR that passes CI
4. Include tests with their implementation
5. Update `CHANGELOG.md`

### Things the factory must NOT do

- **Do not invent slide types.** The 23 types are the contract. New types require human approval and design review.
- **Do not redesign the visual style.** Brand colors and font sizes are fixed.
- **Do not change the DSL syntax mid-phase.** If syntax issues emerge, raise to a human and pause.
- **Do not skip tests.** Every public-API change needs a test.
- **Do not modify `./reference/`.** It's the source of truth for visual behavior.
- **Do not pick the project name.** Phase 0 starts with a placeholder until confirmed.

### Things the factory must DO

- Ask for clarification when an Open Question (Section 11) is reached
- Snapshot every visual output decision (each slide type rendered to XML, committed)
- Maintain visual parity with the Python tool
- Optimize for clarity in DSL syntax over implementation cleverness
- Document every public API with rustdoc

---

## Appendix A: Visual Parity Test Plan

To verify visual parity with the Python reference, the factory should:

1. **Build the reference deck via Python:** `uv run python scripts/build-incident-brief.py scripts/incident_data/mss_metrics_leadership.py` → `MSS-Metrics-Leadership-Brief.pptx`
2. **Build the equivalent deck via slideforge:** `slideforge build examples/mss-metrics.sf --template templates/1898.pptx` → `slideforge-output.pptx`
3. **Compare:**
   - Open both in PowerPoint side by side
   - Verify slide-by-slide visual equivalence
   - Verify slide count matches (25 slides)
   - Verify takeaway bars render
   - Verify talk tracks appear in speaker notes
   - Verify brand chrome (logo, page numbers, confidential markers) appears correctly

Pixel-perfect equivalence is NOT required. Brand-consistent and semantically-equivalent output IS required.

## Appendix B: Sample DSL Files

### Minimal valid input (`minimal.sf`)

```
metadata:
  title "Minimal Example"
  output "minimal.pptx"

slide title:
  color blue
  title "Hello, World"
  subtitle "A minimal slideforge example"

slide end:
  color blue
```

### Realistic content slide

```
slide content:
  title "Five Things Every MSSP Metric System Must Answer"
  bullets:
    - "Can we sell it?"
    - "Can we onboard it?"
    - "Can we detect accurately?"
    - "Can we operate within SLA?"
    - "Can we do all this profitably?"
  takeaway "If our metrics can't answer all five, we're measuring the wrong things."
  talk_track """
    Five questions an MSSP metric system has to answer. Can we sell it.
    Can we onboard it. Can we detect accurately. Can we operate it within
    SLA. And can we do all of that profitably and retain the client.
    Today our metric system answers question four well. The other four
    are partial or implicit. That's the gap.
  """
```

### Complex composite (formula slide)

```
slide formula:
  title "North Star Formula"
  result "Retained Protected Revenue at Target Margin"
  operator "×"
  accent orange
  stack true   # default
  term:
    name "ARR"
    color blue
    definition "Annual recurring revenue from retained MSS clients"
  term:
    name "Gross Margin %"
    color teal
    definition "Profitability of the service after delivery cost"
  term:
    name "Service Health Score"
    color purple
    definition "Composite of SLA, detection, onboarding, response, satisfaction, platform"
  takeaway "Revenue alone isn't health. Margin and delivery health both have to hold."
```

---

**End of seed document.**

Reference materials follow in `./reference/`.
