# Slideforge

> *Branded presentations from a DSL, not a deck of dicts.*

[![crates.io](https://img.shields.io/crates/v/slideforge.svg)](https://crates.io/crates/slideforge)
[![docs.rs](https://docs.rs/slideforge/badge.svg)](https://docs.rs/slideforge)
[![CI](https://github.com/drbothen/slideforge/workflows/CI/badge.svg)](https://github.com/drbothen/slideforge/actions)
[![License](https://img.shields.io/crates/l/slideforge.svg)](LICENSE-MIT)

Slideforge compiles structured slide specifications into branded PowerPoint presentations. Write your decks in a purpose-built DSL with helpful errors, version-control the source, and let Slideforge handle the OOXML.

```
$ slideforge build incident-brief.sf --template brand.pptx
✓ Parsed 25 slides in 12ms
✓ Rendered to incident-brief.pptx (87 KB)
```

## Why Slideforge

Programmatically-generated slides usually go one of two ways:

1. **`python-pptx`**: powerful but verbose, ships a Python runtime, and your slide source is buried in dictionaries.
2. **Markdown tools** (Marp, Slidev): clean syntax but limited slide vocabulary and no native PowerPoint output.

Slideforge takes a third path: a **purpose-built DSL** for the slide types you actually use, **native `.pptx` output** that respects your brand template, and a **single binary** with no runtime dependencies.

### What it gives you

- **Helpful error messages** with source spans, not stack traces
- **Real-time preview** (`slideforge watch`) — sub-second incremental rebuilds
- **23 opinionated slide types** including metric trees, formulas, weighted composites, severity cards, timelines
- **Brand template integration** — start from your `.pptx` template, generate slides into it
- **Speaker notes** as first-class content (talk tracks become PowerPoint notes)
- **Version-controllable** — your decks become diffable plain text

### What it does NOT try to be

- A replacement for PowerPoint as a manual authoring tool
- A general-purpose freeform shape editor
- A Markdown-to-slides converter (your DSL is purpose-built; Markdown imports are out of scope)

## Quick example

`example.sf`:

```
metadata:
  title "Q2 MSSP Health Review"
  output "q2-mssp-health.pptx"
  template "templates/brand.pptx"

slide title:
  color blue
  title "Q2 MSSP Health"
  subtitle "Leadership Brief · Quarterly Review"
  talk_track """
    Today we walk through Q2 MSSP health. Three things to land:
    the North Star is trending up, two dimensions need attention,
    and here's what we're doing about it.
  """

slide formula:
  title "Our North Star"
  result "Retained Protected Revenue at Target Margin"
  operator "×"
  accent orange
  term:
    name "ARR"
    color blue
    definition "Annual recurring revenue from retained clients"
  term:
    name "Gross Margin %"
    color teal
    definition "Profitability after delivery cost"
  term:
    name "Service Health Score"
    color purple
    definition "Composite of six operational measures"
  takeaway "Three things, multiplicatively. Any one weakens, the metric does."

slide end:
  color blue
```

Build it:

```sh
slideforge build example.sf
```

Output: a `.pptx` file with three slides, branded to your template, with speaker notes intact.

## Installation

### Homebrew (macOS, Linux)

```sh
brew install drbothen/tap/slideforge
```

### Cargo

```sh
cargo install slideforge-cli
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/drbothen/slideforge/releases) — macOS (arm64, x86_64), Linux (x86_64), Windows (x86_64).

### From source

```sh
git clone https://github.com/drbothen/slideforge
cd slideforge
cargo install --path crates/slideforge-cli
```

## Quick start

1. **Create a slide file** (`my-deck.sf`) — see the example above or `docs/dsl-reference.md` for the full syntax.
2. **Add a brand template** — any `.pptx` file works; Slideforge uses its master slides for fonts, colors, and chrome.
3. **Build it**:
   ```sh
   slideforge build my-deck.sf --template brand.pptx
   ```
4. **Iterate** with watch mode:
   ```sh
   slideforge watch my-deck.sf --template brand.pptx
   ```

## Slide types

Slideforge ships with 23 slide types covering the patterns most decks need:

**Structural** — `title` (section divider) · `end` (closing slide)

**Narrative + bullets** — `content` · `two_column` · `highlight` · `content_stat`

**Metrics** — `stat_callout` · `stats_summary` · `weighted_composite`

**Comparison** — `highlight_boxes` · `split_contrast` · `card_rows`

**Hierarchy & formulas** — `metric_tree` · `formula`

**Risk, actions, status** — `severity_cards` · `numbered_actions` · `status` · `progress_bar`

**Timelines & tables** — `vertical_timeline` · `horizontal_timeline` · `enhanced_table` · `table`

Each has documented fields, layout rules, and validation. See [`docs/slide-types.md`](docs/slide-types.md) for the full catalog with rendered examples.

## Documentation

- **[DSL Reference](docs/dsl-reference.md)** — full syntax with grammar notes
- **[Slide Types Catalog](docs/slide-types.md)** — every slide type with examples
- **[Brand Templates](docs/brand-templates.md)** — how Slideforge uses your `.pptx` template
- **[Architecture](docs/architecture.md)** — internals (parser, eval, layout, exporters)
- **[Migration from python-pptx](docs/migration-python-pptx.md)** — converting existing decks

## Comparison to alternatives

| Feature | Slideforge | python-pptx | Marp / Slidev |
|---------|-----------|-------------|---------------|
| Native PPTX output | ✓ | ✓ | (Marp via conversion) |
| Single-binary distribution | ✓ | (needs Python) | (needs Node) |
| Purpose-built DSL | ✓ | (Python code) | Markdown |
| Brand template integration | ✓ | ✓ | (CSS-only) |
| Helpful error messages | ✓ | (tracebacks) | (varies) |
| Incremental rebuilds | ✓ | ✗ | (partial) |
| Speaker notes | ✓ | ✓ | ✓ |
| Composed slide types | 23 built-in | (freeform) | (freeform via CSS) |
| Validation | ✓ | ✗ | (basic) |

## Status

Slideforge is **pre-1.0**. The DSL syntax may shift between minor versions until v1.0. Track the [CHANGELOG](CHANGELOG.md) for breaking changes.

**Roadmap:**
- v0.1 — Core pipeline, 5 slide types, PPTX output
- v0.2 — All 23 slide types
- v0.3 — Brand template loading, validation
- v0.4 — Watch mode, incremental compilation, formatter
- v1.0 — DSL syntax frozen, stable API
- v1.x — PDF and HTML exporters, FFI bindings (Python, Node)

## Contributing

Bug reports and feature proposals welcome. For new slide types, see [`docs/proposing-slide-types.md`](docs/proposing-slide-types.md) — we keep the catalog opinionated and small, so new types require design review.

```sh
git clone https://github.com/drbothen/slideforge
cd slideforge
cargo build --workspace
cargo test --workspace
```

## License

Slideforge is dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.

## Acknowledgments

Slideforge's pipeline architecture borrows from [Typst](https://typst.app/). OOXML serialization is powered by [ooxmlsdk](https://github.com/KaiserY/ooxmlsdk). DSL parsing uses [chumsky](https://github.com/zesterer/chumsky).
