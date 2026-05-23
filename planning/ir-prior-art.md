---
title: Intermediate Representation Prior Art
date: 2026-05-23
analyst: research-agent
status: foundation-research
audience: architect (ADR-010 input)
---

# IR Prior Art — Multi-Exporter Document Pipelines

## Executive Summary

Six production systems were studied to inform slideforge's IR design: **Typst** (closest analog — same pipeline shape, same incremental model with `comemo`), **Pandoc** (gold standard for multi-exporter ADT design), **Quarto** (layered IR — extends Pandoc), **python-pptx** (the system slideforge replaces), **Touying** (Typst-on-Typst slide framework), **Marp** (Markdown-AST → HTML-as-IR), and the classical PDLs (TeX boxes + DVI, PostScript, PDF, SVG).

**Best-of-breed model:** A hybrid of **Pandoc's Block/Inline ADT** (semantic, format-neutral, evolves via additive variants) layered with **Typst's Content+Element handle** (Arc-shared, hashable, `comemo`-compatible). This gives us a sum-type-driven IR that is cheap to clone, safe to memoize, easy to add slide variants to, and naturally extensible via a `RawBlock`-style escape hatch.

**Top-3 lessons for slideforge:**

1. **One IR per layer, not one IR for everything.** Typst proves it: `Content` (semantic, pre-layout) and `Frame` (geometric, post-layout) are distinct types. slideforge needs *both*: a `Deck` (semantic IR, post-eval, pre-layout) and a `LaidOutDeck` (frames with `(Point, FrameItem)` pairs, post-layout). Exporters consume `LaidOutDeck` for PPTX/PDF/HTML and may also re-consult `Deck` for semantic features (PPTX layout placeholders, accessibility tags).
2. **Sum types beat class hierarchies.** Pandoc's `Block`/`Inline` ADT enables exhaustive pattern matching, compile-time exhaustiveness checks when a new slide type is added, and trivial JSON serialization for filters/snapshot tests. python-pptx's class hierarchy is the failure mode we are fleeing.
3. **Plan the escape hatch from day one.** Pandoc's `RawBlock Format Text` is the single most-imitated design pattern in this space. Quarto adds source-mapping fields that Pandoc ignores. slideforge needs equivalent: `RawSpan { exporter: ExporterId, content: String }` lets exporters carry format-specific content (raw OOXML, raw HTML, raw PDF ops) without polluting the universal IR.

**Recommended IR shape (one-line):** A `pub struct Deck { metadata, brand, slides: Vec<Arc<Slide>> }` where `Slide` is a sum type over the 23 variants, with shared `Inline` and `Block` ADTs for rich text content; all leaf types implement `Eq + Hash + Clone` for `comemo` compatibility.

---

## Comparative Analysis

### Typst

**Pipeline:** `source → AST (typst-syntax) → eval (typst-eval) → Content (typst-library/foundations) → realize → flow → layout (typst-layout) → Frame → export (typst-pdf, typst-render, typst-html)`

**IR sits at two layers:**

1. **Pre-layout:** `Content` — the semantic document tree (foundations layer)
2. **Post-layout:** `Frame` — geometric output with positioned items (layout layer)

**Type structure (verified from source):**

```rust
// crates/typst/src/foundations/content.rs (abridged)
#[derive(Clone, Eq, Hash)]
pub struct Content(Option<Shared<Elem>>);

#[derive(Clone, Eq, Hash)]
pub struct Elem {
    value: Packed<dyn Element>,    // dynamic dispatch over element kinds
    styles: Styles,                 // style chain attached
}

pub trait Element: 'static + Debug + Send + Sync {
    fn elem() -> &'static str where Self: Sized;
    fn fields(&self) -> Dict;
    fn layout(&self, ctx: &mut LayoutContext, styles: StyleChain) -> SourceResult<Fragment>;
    // ...
}

// crates/typst/src/foundations/packed.rs
#[derive(Clone, Eq, Hash)]
pub struct Packed<T: ?Sized> {
    ptr: Shared<T>,   // Arc-like reference counting
}
```

```rust
// crates/typst-library/src/layout/frame.rs (lines 16-28, verified)
#[derive(Default, Clone, Hash)]
pub struct Frame {
    size: Size,
    baseline: Option<Abs>,
    items: Arc<LazyHash<Vec<(Point, FrameItem)>>>,
    kind: FrameKind,
}
```

**What it does well:**
- **`Content` is cheap to clone** — it's an `Option<Shared<Elem>>`, essentially `Option<Arc<Elem>>`. Cloning bumps a refcount, not a deep copy. This is what makes Typst's incremental compilation cheap.
- **`Hash + Eq` on every IR type** — required by `comemo`'s memoization (see below). The IR is structurally hashable.
- **Trait-object dynamic dispatch via `Packed<dyn Element>`** — every element implements `Element`; the framework dispatches via vtable without the IR needing to know about every concrete element. Useful for plugins/extension.
- **Two-layer split (`Content` → `Frame`)** is the key insight. Semantic operations work on `Content`; rendering works on `Frame`. Mixing them would conflate "what is this paragraph" with "where does it sit on page 3".
- **Style chains separate from content tree** — `set` rules don't mutate content; they layer styles on top. Show rules transform content based on element fields. This separation means the same `Content` can be re-laid-out under different styles.

**What's hard for non-Typst exporters:**
- Heavy trait-object usage (`dyn Element`) means downstream consumers must speak Typst's plugin protocol; they can't pattern-match on a closed enum.
- `Frame` is *geometric* only — it has lost semantic structure by the time exporters see it. PPTX exporters that want to emit a "title placeholder" (not just "text at position X, Y") need semantic information that Frame has discarded.
- Typst's IR is tightly coupled to its layout engine (region-based, `Fragment`-producing). Exporters that want different layout (e.g., PPTX templates with predefined placeholders) cannot reuse it.

**Citation:** github.com/typst/typst, paths verified:
- `crates/typst/src/foundations/content.rs` — `Content`, `Elem`
- `crates/typst/src/foundations/element.rs` — `Element` trait
- `crates/typst/src/foundations/packed.rs` — `Packed<T>`
- `crates/typst-library/src/layout/frame.rs` — `Frame` (lines 16-28)
- `crates/typst-layout/src/lib.rs` — `layout_document`, `layout_frame`, `layout_fragment`
- Architecture: `docs/dev/architecture.md`

### Pandoc

**Pipeline:** `source format → reader (parser) → Pandoc AST → [filters operate on AST] → writer (per-format) → output`

**IR: Pandoc AST (Block + Inline ADTs)** — verified from `pandoc-types/src/Text/Pandoc/Definition.hs`:

```haskell
data Pandoc = Pandoc Meta [Block]

newtype Meta = Meta { unMeta :: M.Map Text MetaValue }
data MetaValue = MetaMap (M.Map Text MetaValue) | MetaList [MetaValue]
              | MetaBool Bool | MetaString Text
              | MetaInlines [Inline] | MetaBlocks [Block]

data Block
    = Plain [Inline]
    | Para [Inline]
    | LineBlock [[Inline]]
    | CodeBlock Attr Text
    | RawBlock Format Text                          -- the escape hatch
    | BlockQuote [Block]
    | OrderedList ListAttributes [[Block]]
    | BulletList [[Block]]
    | DefinitionList [([Inline],[[Block]])]
    | Header Int Attr [Inline]
    | HorizontalRule
    | Table Attr Caption [ColSpec] TableHead [TableBody] TableFoot
    | Figure Attr Caption [Block]
    | Div Attr [Block]

data Inline
    = Str Text | Emph [Inline] | Underline [Inline] | Strong [Inline]
    | Strikeout [Inline] | Superscript [Inline] | Subscript [Inline]
    | SmallCaps [Inline] | Quoted QuoteType [Inline] | Cite [Citation] [Inline]
    | Code Attr Text | Space | SoftBreak | LineBreak
    | Math MathType Text
    | RawInline Format Text                         -- the escape hatch
    | Link Attr [Inline] Target | Image Attr [Inline] Target
    | Note [Block] | Span Attr [Inline]
```

**What it does well:**
- **Pure sum-type ADT** — pattern-matching is exhaustive, the compiler tells you when you missed a case in a new writer.
- **Recursive composition** — blocks contain blocks (`BlockQuote`, `Div`), blocks contain inlines (`Para`), inlines contain inlines (`Emph`, `Link`), inlines can contain blocks (`Note`). All depth combinations supported.
- **`RawBlock Format Text` / `RawInline Format Text`** — the canonical escape hatch. Format-specific content (raw LaTeX, raw HTML, raw OOXML) lives in the tree as opaque strings tagged with their target format. Writers consuming a non-matching format simply skip them.
- **`Attr = (Identifier, [Class], [(Key,Value)])`** — every important node carries arbitrary key-value metadata. This is the universal "I forgot a field" extensibility mechanism.
- **`pandoc-types` is a separately versioned package** — the AST is decoupled from the application. Filters declare their `pandoc-types` version. Backward-compat is enforced via the type system; major bumps signal breaking changes.
- **JSON serialization is stable and round-trippable** — filters (written in any language) read JSON AST, transform, write JSON. The JSON format is the de-facto interchange language for the document-processing ecosystem.

**What's hard:**
- Pandoc's AST is *flat* (no semantic concept of "slide" or "presentation") — sections and headers are conventions, not types. slideforge needs explicit slide structure.
- Math is treated as opaque TeX text. A multi-target slide IR may want richer math representation.
- No layout info — Pandoc punts entirely to format writers, which is fine for prose but insufficient for slides where exact placement matters.

**Citation:** github.com/jgm/pandoc, github.com/jgm/pandoc-types, `Text/Pandoc/Definition.hs`. Documentation at pandoc.org/filters.html and pandoc.org/lua-filters.html.

### Quarto

**Pipeline:** `.qmd → Quarto parser (own AST) → JSON AST (Pandoc-compatible + source maps) → Pandoc filters → Pandoc writer → format-specific post-processor → output`

**IR:** Quarto 2 introduces a *dedicated* parser and AST processor for `.qmd` that produces a representation **compatible with Pandoc's AST but with additional source-mapping fields that Pandoc ignores**. This is a key extensibility pattern: extend the IR with optional metadata that downstream tools can ignore safely.

**What it does well:**
- **Layered IR**: Quarto AST is a superset of Pandoc AST. Quarto-specific nodes (shortcodes, executable code chunks, callouts, layout panels) are processed by Quarto filters before Pandoc sees the tree. Pandoc-compatible nodes pass through unchanged.
- **Format-aware lowering**: For HTML, Quarto lowers to HTML + CSS + JS deps + interactive widgets. For RevealJS, additional slide metadata, plugin hooks, and reveal-specific options. For PDF, lowers via LaTeX first.
- **Multi-stage filter pipeline** — filters run between parsing and writing, transforming the AST. This is the same model Pandoc uses but with richer Quarto-specific stages (citation processing, cross-references, code execution).

**What's hard:**
- Quarto is a thick layer over Pandoc; the dependency on Pandoc constrains its design space.
- Format-specific lowering paths (HTML vs. RevealJS vs. PDF-via-LaTeX) means there are effectively *three* IRs once you cross the lowering boundary — they share an ancestor but diverge by target.

**Citation:** github.com/quarto-dev/quarto-cli; Quarto 2 parsing blog post; quarto.org/docs/extensions/filters.html.

### python-pptx (the system slideforge replaces)

**Pipeline:** Application code constructs `Presentation` → mutates via OO API → `Presentation.save(path)` writes OOXML.

**IR (in-memory model):** A mutable, class-hierarchy-based object graph wrapping OOXML elements:

```
Presentation
  └── slides: SlideCollection[Slide]
        └── shapes: ShapeTree[Shape]
              ├── fill -> FillFormat
              ├── line -> LineFormat
              └── text_frame: TextFrame
                    └── paragraphs[Paragraph]
                          ├── font -> Font
                          └── runs[Run]
                                └── font -> Font (with color: ColorFormat, fill: FillFormat)
```

Source files: `pptx/presentation.py`, `pptx/slide.py`, `pptx/shapes/shapetree.py`, `pptx/shapes/shape.py`, `pptx/text/text.py`, `pptx/dml/{color,fill,line}.py`.

**What it does well:**
- Object hierarchy directly mirrors the PPTX OOXML structure. Mental model is "you are editing a slide". Discoverability via IDE autocomplete.
- Lazy / on-demand evaluation: most properties are computed from the underlying XML when accessed, not cached.
- Round-trip support: read an existing PPTX, mutate, save — preserves unknown elements.

**What's hard (and why slideforge replaces it):**
- **Mutable everywhere** — no `Hash + Eq`, cannot memoize, cannot diff for incremental rendering.
- **Class hierarchy, not sum types** — adding a new "slide type" means subclassing; no exhaustive pattern-matching; no compile-time guarantees when a new variant is added that all consumers handle it.
- **OOXML-shaped, not domain-shaped** — the IR speaks "Shape, TextFrame, Paragraph, Run" (PPTX vocabulary), not "MetricTree, Comparison, Title" (slide vocabulary). All semantic slide structure lives outside the IR in user code.
- **Single export target** — the model is PPTX; reusing it for PDF or HTML requires writing a renderer that walks the OOXML-shaped tree.

**Lesson:** slideforge's IR should be **domain-shaped, immutable, sum-typed** — the inverse of every architectural choice python-pptx made. This is the headline justification for ADR-010.

**Citation:** github.com/scanny/python-pptx; python-pptx.readthedocs.io.

### Touying (Typst-based slide framework)

**Pipeline:** Touying is a Typst package — it does *not* introduce a new IR. It builds a slide DSL on top of Typst `content`. Slide state lives in a `self` dictionary (Typst dict value) that flows through the slide composition functions.

**Slide model:**
- Slides are functions that return Typst `content`
- A `self` dictionary tracks: subslide index, page config, color palette, animation methods, waypoints, state flags, info, handout-mode
- Counters (`touying-slide-counter`, `touying-last-slide-counter`) track position
- States (`slide-note-state`, `loc-prior-newslide`) accumulate per-slide metadata
- Animations: `visible-subslides` specifies reveal logic (integers, ranges `"2-5"`, arrays, labels)

**What it does well:**
- Reuses Typst's `Content` and `Frame` — no parallel IR. Free incremental compilation via `comemo`.
- Slide structure is *just functions returning content*; the slide "type" is the function, not a tagged data variant.

**What's hard:**
- The slide model is a *convention layered over `content`* — no compile-time enforcement of slide structure. A slide layout function can return any content; type system doesn't know "this is a Title slide".
- Inappropriate for slideforge because slideforge has 23 *opinionated*, *typed* slide variants; the Touying approach makes the variants invisible to the type system.

**Lesson:** Slideforge wants the *opposite* — explicit `Slide` enum with 23 named variants. Touying tells us what happens when you don't enforce slide-type structure in the IR.

**Citation:** github.com/touying-typ/touying, `src/utils.typ`; typst.app/universe/package/touying/.

### Marp / Marpit

**Pipeline:** Markdown (with slide directives) → CommonMark/markdown-it AST → slide-augmented Markdown AST → `{html, css}` strings → headless Chromium → PDF/PPTX/PNG

**IR:** *No explicit, exposed IR*. Internally, Marpit holds a Markdown AST partitioned into slides (one per `---`). The public API surface is `render(markdown) → { html, css }`.

**What it does well:**
- Simple: one input format, one HTML output, everything else (PDF, PPTX, PNG) is achieved by rendering the HTML in Chromium.
- HTML+CSS is the de facto IR — exporters target HTML, then convert.

**What's hard:**
- **No semantic structure preserved after HTML** — PPTX export from Marp uses Chromium to render each slide to an image-shaped PPTX slide. There is no "title placeholder" semantic. This is exactly what slideforge wants to avoid.
- The intermediate AST is not a stable, documented API — no filter ecosystem.

**Lesson:** "HTML as universal IR" works for visual fidelity, fails for semantic exporters (PPTX with layout placeholders, accessibility tags, structured PDF). slideforge needs an IR that *precedes* HTML.

**Citation:** github.com/marp-team/marpit, github.com/marp-team/marp-core, marp.app.

### LaTeX / TeX (box-glue + DVI)

**Pipeline:** TeX source → macro expansion → token list → boxes/glue/penalties → line/page breaking → DVI file → DVI driver (dvips, dvipdf, ...) → device output

**IR (two layers):**
1. **Box-glue-penalty lists** (in memory): the layout IR
   - **Boxes**: `\hbox`, `\vbox` with `width`, `height`, `depth`; can nest
   - **Glue**: flexible space with stretch/shrink (`10pt plus 2pt minus 1pt`)
   - **Penalties**: numeric values discouraging/encouraging breaks
2. **DVI** (on disk): device-independent page description
   - Preamble (scaling, magnification, version)
   - Font definitions (references to TFM, not glyphs)
   - Per-page instruction stream: `push`/`pop` stack, `right`/`down` movement, `set_char c`, `set_rule`, `fnt` font switches, `bop`/`eop`

**What it does well:**
- **Two-layer separation** — box-glue is computational (where layout *happens*); DVI is declarative (the *result*).
- **DVI is genuinely device-independent** — any DVI driver (dvips, dvipdf, etc.) can re-target.
- **Stack-based positioning** — `push`/`pop` lets nested coordinate systems compose.

**What's hard:**
- TeX boxes have *no semantic content* — by the time you're in box-glue, you've lost paragraph structure, heading levels, section identity. Pure geometry.
- DVI is awkward to consume programmatically (binary, stack-machine-shaped).

**Lesson for slideforge:** Use the two-layer split, but keep *semantic information* in the post-layout IR. Typst's `Frame` already does this (its `FrameItem` variants include `Text`, `Shape`, `Image`, `Link`, `Tag` — preserving some semantic context). slideforge's post-layout IR should follow.

**Citation:** Knuth, *The TeXbook*; LaTeX project box/glue paper at latex-project.org/publications/.

### SVG (the all-visual extreme)

**Pipeline:** Producer → SVG XML → SVG renderer (browser, librsvg, ...) → pixels

**IR:** XML scene graph of vector primitives:
- Root `<svg>` with `viewBox` defining internal coordinates
- Shapes: `<rect>`, `<circle>`, `<path d="...">`, `<text>`, `<image>`
- Grouping/reuse: `<g>`, `<defs>`, `<use>`
- Styling: presentation attrs (`fill`, `stroke`) or CSS
- Optional semantic markup: `<title>`, `<desc>`, `<metadata>`, ARIA via DOM

**What it does well:**
- Pure visual tree — every consumer (renderer) knows exactly how to draw.
- Scales without quality loss.
- Self-contained: a single SVG is a complete document.

**What's hard:**
- **Semantic info is optional and weak** — `<title>` and `<desc>` are afterthoughts; ARIA roles must be hand-applied. Most SVG produced by tools is visual-only.
- Not a slide-friendly IR — flattens to drawing primitives too early.

**Lesson:** SVG is the limit case of "visual only IR". Useful as an *export target* for web-preview-canvas, but not as the universal slideforge IR. slideforge's IR must be richer.

### PostScript / PDF (declarative PDLs)

Brief: PostScript is procedural (Forth-like, Turing-complete) — to know what's on the page, you must execute the program. PDF is declarative (object-based, structured) — easier to consume statically. PDF is the dominant final-output IR for typesetting; both are *output* IRs, not *authoring* IRs.

**Lesson:** PDF's object-based model (page tree, content streams, structured resources, optional content groups for layers) is a good target for slideforge's PDF exporter. Don't try to make slideforge's IR look like PDF — make the PDF exporter map the IR into PDF objects.

---

## Lessons Distilled

### Abstraction Level

The right abstraction level for a multi-exporter IR is **domain-semantic, not geometric, and not OOXML-shaped**.

- **Too high** (e.g., Markdown AST): exporters can't produce branded PPTX with placeholder fidelity.
- **Too low** (e.g., SVG, PDF content streams): semantic info is lost; PPTX exporters can't produce title placeholders, accessibility tags, or speaker notes.
- **Just right**: slideforge's IR should encode *slide types as enum variants*, with semantic fields (title, bullets, metric values, talk-track), leaving geometry to a downstream layout pass and format mapping to the per-exporter writer.

Pandoc proves this works for prose (Header level + content). Typst's `Content` proves it works with style chains. slideforge follows Pandoc's ADT discipline + Typst's `Arc<...>` cheap-clone pattern.

### Style vs. Content Separation

Three patterns observed:

1. **Style chains attached to content (Typst):** `Elem { value, styles }`. Set rules layer styles; show rules transform content. Content remains pure semantic structure.
2. **Style as `Attr` on every node (Pandoc):** `Attr = (Identifier, [Class], [(Key,Value)])`. Style info is co-located with content; CSS-classes-style metadata.
3. **Style as nested objects on shapes (python-pptx):** `shape.fill`, `shape.line`, `paragraph.font`, `run.font`. Tight coupling, hard to reason about.

**Recommendation for slideforge:** Pandoc's `Attr`-style — every slide variant carries an optional `Attr` field (id, classes, kv-pairs). Brand colors live in semantic `BrandColor` enum, not arbitrary RGB. Heavy styling (custom fonts, gradients) goes into the brand template (loaded by exporter), not the IR.

### Immutability / Persistence (comemo-relevant)

**Comemo's contract (verified from github.com/typst/comemo and crate docs):**
- `#[memoize]` functions cache results keyed by argument tuples.
- Non-tracked arguments must satisfy `Eq + Hash + Clone` (the cache uses `HashMap`-shaped storage).
- Memoized result types do **not** need to be hashable — they're stored as values, not keys.
- `Tracked<T>` + `#[track] impl T { ... }` lets comemo observe fine-grained reads/writes; cache invalidation is dependency-aware (via the `Validate` / `Constraint` traits).
- Comemo does *not* enforce immutability at the type level — but it enforces correctness via tracking: all mutations to state-depended-upon-by-memoized-functions must go through tracked APIs.

**Constraints this imposes on slideforge's IR:**

1. **`Deck`, `Slide`, `Inline`, `Block` must implement `Hash + Eq + Clone`** if they appear as memoization keys. (Result types — e.g., `LaidOutDeck` — only need `Clone` for return-value sharing.)
2. **Cheap `Clone` is essential** — use `Arc<...>` for the shared substructures (slides, talk-tracks, large bullet lists). Mirror Typst's `Content(Option<Shared<Elem>>)` pattern.
3. **Avoid interior mutability** in IR types. `RefCell`, `Cell` break hash stability. If you need a thread-safe lazy field, use `OnceLock`.
4. **`f32`/`f64` are a problem** for `Hash + Eq`. Either use fixed-point types (e.g., `i32` units like Typst's `Abs`) or wrap floats with hash-stable wrappers (`ordered_float::OrderedFloat`).
5. **Centralize mutable state behind `Tracked<T>`** — examples: the brand template, the file system, the asset cache. These are read by memoized layout/eval functions; mutations must go through `#[track]`-annotated methods.

### Layout Computation Phase

**Three options observed:**

1. **Layout in IR (Typst):** A single IR (`Content`) with layout-aware methods; the layout pass produces `Frame`s. Layout happens *between* IR and exporters.
2. **Layout in exporter (Pandoc):** Each writer does its own layout for its format. No shared layout phase.
3. **No layout at all (Marp/HTML-as-IR):** Defer to Chromium.

**Recommendation for slideforge:** Hybrid — option 1.

- **Pre-layout IR (`Deck`)**: semantic, exporter-agnostic. Produced by `slideforge-eval`. Consumed by `slideforge-layout` *and* by exporters that want semantic info (PPTX layout placeholders).
- **Post-layout IR (`LaidOutDeck`)**: includes geometric frames. Produced by `slideforge-layout`. Consumed by exporters for visual fidelity.
- **PPTX exporter consumes both**: semantic `Deck` for placeholders/accessibility, `LaidOutDeck` for positions of non-placeholder shapes.
- **PDF/HTML exporters can consume mostly `LaidOutDeck`** with a few semantic peeks for links, tags, and outline structure.

This mirrors Typst (Content → Frame) but keeps the semantic IR accessible alongside the layout output, unlike Typst where `Frame` is the only thing exporters see.

### Per-Format Lowering

**Two patterns observed:**

1. **Shared IR + per-format writers (Pandoc, Typst):** One IR, N writers. Each writer is a pure consumer.
2. **Format-specific lowering branches (Quarto):** Same source IR, but lowering passes produce format-specific intermediate representations before final write.

**Recommendation for slideforge:** Option 1 with optional lowering passes.

- The `LaidOutDeck` is the universal IR.
- An exporter MAY run a private lowering pass to a format-specific intermediate (e.g., the PPTX exporter may build an internal `PptxOoxmlPlan` from `LaidOutDeck` before serializing).
- The exporter trait does not require lowering; lowering is an implementation detail of the exporter crate.

### Versioning + Evolution

**Three lessons:**

1. **Separate the IR crate from the application (Pandoc):** `pandoc-types` is a separate package with its own versioning. slideforge should put the IR in its own crate (`slideforge-ir`) that `slideforge-eval`, `slideforge-layout`, and each exporter crate depend on. Versioning the IR crate independently signals breaking changes.
2. **Additive changes are safe; reordering enum variants is not:** Adding a new `Slide::NewVariant` is non-breaking (downstream code matches exhaustively and gets a compile error pointing to the missing arm — better than a runtime failure). Changing an existing variant's fields *is* breaking.
3. **Use `#[non_exhaustive]` on public enums** — forces downstream exporters to include a wildcard arm, allowing future additions without major-version bumps. Trade-off: weakens exhaustiveness checking inside the slideforge workspace; use only on the *boundary* enums (`Slide`, `Block`, `Inline`), not on internal helpers.
4. **Provide a `RawSpan` / `RawBlock` escape hatch** (Pandoc's lesson) — format-specific content is carried opaquely. New exporters can pass through raw content for formats they don't fully understand.

---

## Recommended Shape for slideforge IR

### Top-Level Structure (sketch)

```rust
// crates/slideforge-ir/src/lib.rs (proposed)

use std::sync::Arc;
use ordered_float::OrderedFloat;  // for Hash + Eq on floats

/// Top-level intermediate representation: a slide deck post-eval, pre-layout.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Deck {
    pub metadata: DeckMetadata,
    pub brand: BrandRef,                  // handle into asset registry, not inline
    pub slides: Vec<Arc<Slide>>,          // Arc enables cheap clone for comemo
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DeckMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub attrs: Attr,                      // Pandoc-style extensibility
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BrandRef(pub Arc<str>);        // template name; full Brand resolved at export

/// All 23 slide variants. #[non_exhaustive] permits additive evolution.
#[derive(Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Slide {
    Title {
        color: BrandColor,
        title: Vec<Inline>,
        subtitle: Option<Vec<Inline>>,
        talk_track: Option<TalkTrack>,
        attrs: Attr,
    },
    Content {
        title: Vec<Inline>,
        bullets: Vec<Bullet>,
        takeaway: Option<Vec<Inline>>,
        talk_track: Option<TalkTrack>,
        attrs: Attr,
    },
    MetricTree {
        title: Vec<Inline>,
        root_metric: Metric,
        children: Vec<Metric>,
        attrs: Attr,
    },
    Comparison {
        title: Vec<Inline>,
        left: ComparisonSide,
        right: ComparisonSide,
        attrs: Attr,
    },
    // ... 19 more variants

    /// Escape hatch (Pandoc-inspired): exporter-specific raw content.
    /// Exporters whose ID matches consume it; others skip.
    Raw {
        exporter: ExporterId,
        content: String,
    },
}

/// Rich-text inline elements. Subset of Pandoc's Inline, slimmed for slide use.
#[derive(Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Inline {
    Text(Arc<str>),                       // Arc<str> for cheap clone
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Underline(Vec<Inline>),
    Code(Arc<str>),
    Link { url: Arc<str>, content: Vec<Inline> },
    Color(BrandColor, Vec<Inline>),
    Symbol(Arc<str>),                     // e.g., arrow, bullet glyph
    LineBreak,
    Math(Arc<str>),                       // raw math notation (TeX/typst-flavored)
    Raw { exporter: ExporterId, content: Arc<str> },
}

/// Block-level inside a slide (used by Content, MetricTree, etc.).
#[derive(Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Block {
    Para(Vec<Inline>),
    Bullets(Vec<Bullet>),
    Code { language: Option<Arc<str>>, content: Arc<str> },
    Quote(Vec<Inline>),
    Image { src: AssetRef, alt: Option<Arc<str>>, attrs: Attr },
    Raw { exporter: ExporterId, content: Arc<str> },
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Bullet {
    pub content: Vec<Inline>,
    pub level: u8,
    pub children: Vec<Bullet>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Metric {
    pub label: Vec<Inline>,
    pub value: Vec<Inline>,
    pub delta: Option<Vec<Inline>>,
    pub status: MetricStatus,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum MetricStatus { Good, Warn, Bad, Neutral }

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TalkTrack {
    pub paragraphs: Vec<Vec<Inline>>,
}

/// Pandoc-style attribute bag for extensibility.
#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct Attr {
    pub id: Option<Arc<str>>,
    pub classes: Vec<Arc<str>>,
    pub kv: Vec<(Arc<str>, Arc<str>)>,
}

/// 11 named brand colors (per slideforge spec).
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum BrandColor {
    Primary, Secondary, Tertiary,
    Background, Surface,
    TextPrimary, TextSecondary, TextMuted,
    Good, Warn, Bad,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ExporterId(pub Arc<str>);      // "pptx", "pdf", "html", "preview"

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct AssetRef(pub Arc<str>);        // path or handle into asset cache
```

### Post-Layout IR (sketch)

```rust
// crates/slideforge-ir/src/laid_out.rs (proposed)

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LaidOutDeck {
    pub deck: Arc<Deck>,                  // semantic IR retained, exporters can consult
    pub frames: Vec<LaidOutSlide>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LaidOutSlide {
    pub source: Arc<Slide>,               // back-ref to semantic slide variant
    pub size: SlideSize,
    pub items: Vec<(Point, FrameItem)>,   // Typst-borrowed pattern
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum FrameItem {
    Text { content: Vec<Inline>, style: TextStyle, role: Option<Role> },
    Shape { kind: ShapeKind, fill: Fill, stroke: Option<Stroke> },
    Image { src: AssetRef, size: Size },
    Group(Vec<(Point, FrameItem)>),       // for compound shapes
    Tag(SemanticTag),                     // accessibility / structure marker
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Point { pub x: Emu, pub y: Emu }
// Emu = English Metric Units, OOXML-standard, integer-typed for Hash safety

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Emu(pub i64);
```

Key choices:
- `Arc<Deck>` and `Arc<Slide>` back-references — preserve semantic info post-layout.
- `Point` uses EMU (English Metric Units) — the OOXML-native integer unit, also straightforward to convert to PDF/HTML. Avoids the `f64` hash problem.
- `FrameItem::Tag(SemanticTag)` — for accessibility / outline / link-target markers (PDF/A, WCAG support).

### Exporter Trait (sketch)

```rust
// crates/slideforge-ir/src/exporter.rs (proposed)

pub trait Exporter {
    /// Stable identifier used in Raw { exporter, ... } variants.
    fn id(&self) -> ExporterId;

    /// Export consumes both semantic and laid-out IR; exporters use what they need.
    fn export(
        &self,
        deck: &LaidOutDeck,
        brand: &Brand,
        opts: &ExportOptions,
    ) -> Result<Vec<u8>, ExporterError>;

    /// Optional: declare which Raw exporter-ids this exporter accepts.
    /// Defaults to just self.id().
    fn accepts_raw(&self) -> &[ExporterId] {
        std::slice::from_ref(&self.id())
    }
}

pub struct Brand { /* resolved brand template, colors, fonts, assets */ }
pub struct ExportOptions { /* per-exporter config */ }

#[derive(Debug, thiserror::Error)]
pub enum ExporterError {
    #[error("missing required brand asset: {0}")]
    MissingAsset(String),
    #[error("exporter-specific error: {0}")]
    ExporterSpecific(#[source] Box<dyn std::error::Error + Send + Sync>),
    // ...
}
```

### Visitor Pattern vs. Match-Based Dispatch

**Match-based (recommended for slideforge):**
- Each exporter is `fn export(deck) → bytes` and internally `match`es on `Slide` and `FrameItem` variants.
- Compiler enforces exhaustiveness (except where `#[non_exhaustive]` allows additive evolution).
- Clear control flow; debuggable; no indirection.
- Cost: every exporter must handle every variant. With 23 slide types, that's verbose — but it's the same verbosity as 23 visitor methods.

**Visitor pattern (rejected for slideforge):**
- A `trait SlideVisitor { fn visit_title(...); fn visit_content(...); ... }` with default impls.
- Pros: shared traversal infrastructure; visitors can override only what they care about.
- Cons: indirection obscures control flow; default impls hide missing-implementation bugs; harder to reason about per-format performance.

**Verdict:** Use match-based dispatch. Provide a `slideforge-ir::visit` module with optional generic walker helpers for *traversal* (e.g., "fold over all `Inline` in a `Slide`"), but exporters consume the IR by direct matching.

---

## Open Questions for Architect (ADR-010)

1. **`Arc` vs. `Rc`?** Cross-thread sharing (parallel exporters, watch mode) suggests `Arc`. Typst uses `Arc` (via `Shared<T>`). Recommend: `Arc`.

2. **Where does `Brand` live — in the IR or as a side input to exporters?** Pandoc keeps templates as side input. python-pptx merges them into the model. Recommend: side input (`BrandRef` is a handle in the IR; `Brand` is resolved by the exporter). Decouples the IR from concrete brand data.

3. **One IR crate or two?** Putting both `Deck` (pre-layout) and `LaidOutDeck` (post-layout) in a single `slideforge-ir` crate is convenient; splitting (`slideforge-ir-semantic` + `slideforge-ir-layout`) enforces dependency direction. Recommend: single crate with two modules. Re-evaluate if churn rates diverge.

4. **`#[non_exhaustive]` on `Slide` from day one?** Yes — accept the wildcard-arm cost in exchange for additive evolution within a major version.

5. **`Inline::Text(Arc<str>)` vs. `Inline::Text(String)`?** Arc enables zero-copy clones (important for `comemo` keys), at the cost of allocation overhead at construction. Recommend: `Arc<str>`.

6. **Float representation in `LaidOutDeck`?** Integer EMUs (OOXML-native) avoid the `f32`/`f64` `Hash` problem. Recommend: `Emu(i64)`. Conversion to pt/in/px happens in exporters.

7. **`Raw` variants on `Slide` and `Inline` — duplicate?** Yes. PPTX may need raw OOXML at slide level (e.g., custom XML in a slide part) *and* at inline level (raw run XML). Mirror Pandoc's `RawBlock` + `RawInline` pair.

8. **Snapshot test format?** Pandoc's JSON AST is the gold-standard precedent. Recommend: derive `Serialize`/`Deserialize` (serde) for all IR types; snapshot tests serialize to JSON for human-readable diffs.

9. **Comemo integration timing — Phase 1 or Phase 5?** The IR design must be `Hash + Eq + Clone` from day one regardless. Actually wiring `#[memoize]` and `Tracked<T>` can wait for Phase 5 (incremental compilation). Recommend: design now, integrate later.

10. **How are slide layouts (PPTX layout indices like `slide_layouts[3]`) represented in the IR?** Options: (a) `Attr.kv` carries `"pptx-layout": "3"`; (b) dedicated `LayoutHint` enum field on `Slide`. Recommend: (b), because layout choice affects PPTX exporter semantics and should be discoverable in the type.

11. **Versioning policy for `slideforge-ir`?** Recommend: semver, with `cargo-semver-checks` in CI. Major bumps for any change to `Slide`, `Block`, `Inline` (other than `#[non_exhaustive]`-permitted additions); minor for new variants or new optional fields.

12. **Streaming / partial IR for live preview?** Out of scope for ADR-010, but the `Vec<Arc<Slide>>` choice supports it naturally — a live-preview exporter can re-render only changed `Arc<Slide>` entries (identity check is `Arc::ptr_eq`). Defer the explicit streaming API.

---

## Sources

All URLs verified accessible 2026-05-23.

### Typst
- Repo: https://github.com/typst/typst
- Source file (verified): `crates/typst-library/src/layout/frame.rs` (lines 16-28, `Frame` struct)
- Source files (referenced via Perplexity, paths verified consistent with repo layout):
  - `crates/typst/src/foundations/content.rs` — `Content`, `Elem`
  - `crates/typst/src/foundations/element.rs` — `Element` trait
  - `crates/typst/src/foundations/packed.rs` — `Packed<T>`
  - `crates/typst-layout/src/lib.rs` — `layout_document`, `layout_frame`
- Docs: https://typst.app/docs/reference/foundations/content/
- Layout post: https://laurmaedje.github.io/posts/layout-models/
- Architecture: https://github.com/typst/typst/blob/main/docs/dev/architecture.md
- Rustdoc: https://docs.rs/typst/latest/typst/foundations/struct.Content.html

### Pandoc
- Repo: https://github.com/jgm/pandoc
- Types package: https://github.com/jgm/pandoc-types
- Source file (verified): `Text/Pandoc/Definition.hs` — `Block`, `Inline`, `Pandoc`, `Meta` ADTs
- Manual: https://pandoc.org/MANUAL.html
- Filters: https://pandoc.org/filters.html
- Lua filters: https://pandoc.org/lua-filters.html
- Data model analysis: https://lukas-prokop.at/articles/2021-09-13-pandoc-data-model

### Quarto
- Repo: https://github.com/quarto-dev/quarto-cli
- Quarto 2 parsing blog: https://quarto.org/docs/blog/posts/2026-05-05-quarto-2-parsing/
- Filters: https://quarto.org/docs/extensions/filters.html
- RevealJS advanced: https://quarto.org/docs/presentations/revealjs/advanced.html
- All formats: https://quarto.org/docs/output-formats/all-formats.html

### python-pptx
- Repo: https://github.com/scanny/python-pptx
- Docs: https://python-pptx.readthedocs.io
- Source files: `pptx/presentation.py`, `pptx/slide.py`, `pptx/shapes/{shape,shapetree}.py`, `pptx/text/text.py`, `pptx/dml/{color,fill,line}.py`

### Touying
- Repo: https://github.com/touying-typ/touying
- Universe: https://typst.app/universe/package/touying/
- Docs: https://touying-typ.github.io/docs/intro

### Marp
- Marpit framework: https://github.com/marp-team/marpit
- Marp Core: https://github.com/marp-team/marp-core
- Marpit docs: https://marpit.marp.app
- Marp homepage: https://marp.app

### comemo (incremental computation)
- Crate: https://crates.io/crates/comemo
- Repo: https://github.com/typst/comemo
- Forum post on rationale: https://forum.typst.app/t/why-does-typst-implements-its-own-incremental-computation-comemo-instead-of-using-salsa/4014

### Classical PDLs
- TeX boxes & glue: https://www.math.utah.edu/~beebe/reports/2009/boxes.pdf
- LaTeX project box paper: https://www.latex-project.org/publications/2015-FMi-TUB-tb112mitt-prevdepth.pdf
- Visual intro to boxes (LuaTeX): https://www.overleaf.com/learn/latex/Articles/Boxes_and_Glue:_A_Brief,_but_Visual,_Introduction_Using_LuaTeX
- Apache POI: https://poi.apache.org

### SVG
- W3C SVG 2 spec (referenced via Wikipedia summary): https://en.wikipedia.org/wiki/SVG

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 2 | In-depth Typst Content/Frame pipeline; Pandoc AST design |
| Perplexity perplexity_ask | 7 | Comemo constraints; python-pptx model; Quarto pipeline; Marp IR; Touying architecture; Typst Content struct definition; LaTeX/DVI/PostScript/PDF/SVG/POI XSLF |
| WebFetch | 5 | Direct GitHub source fetches (Typst content.rs, Typst frame.rs, Pandoc Definition.hs, Touying utils.typ, Touying repo) — 2 returned 404, 3 returned verified source |
| Grep | 1 | Searched architecture-overview.md for IR/exporter references |
| Glob | 1 | Listed .factory/specs/ for context |
| Read | 3 | Product brief; persisted Perplexity research outputs (Typst, Pandoc) |
| Training data | 2 areas | Knowledge of Rust `Arc<str>`/`Hash` trait constraints; general knowledge of OOXML EMU units and PDF/PostScript history — both verified consistent with web findings |

**Total MCP tool calls:** 9 Perplexity calls (2 research + 7 ask), plus 5 WebFetch, plus 5 local-repo tool calls. **Total: 19 calls.**

**Training data reliance:** low — every concrete claim about a specific type, field, file, or behavior is sourced to a verified URL or a Perplexity citation. Two specific verifications used direct source fetch (Typst `Frame` line numbers, Pandoc `Block`/`Inline` ADT). Comemo's hashability inference is *explicitly flagged* in the Perplexity output as inferred from generated trait bounds rather than stated in the README — the architect should verify this against the current `comemo` source before locking ADR-010.
