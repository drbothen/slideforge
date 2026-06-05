---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-085
title: "Bundled DefaultInlineFormat + PPTX OOXML Dog-Fooding Refactor"
epic: EPIC-21
wave: 4
points: 8
priority: P0
tdd_mode: strict
status: draft
behavioral_contracts: [BC-5.02.001, BC-5.02.002, BC-3.05.001]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023]
crate: slideforge-plugin-api
target_module: slideforge-plugin-api, slideforge-pptx
subsystems: [SS-14, SS-06]
depends_on:
  - STORY-002
  - STORY-028
  - STORY-037
  - STORY-038
blocks:
  - STORY-049
estimated_days: 3
---

# STORY-085: Bundled DefaultInlineFormat + PPTX OOXML Dog-Fooding Refactor

## Subsystem Anchor Justification

SS-14 (Plugin API) owns the `DefaultInlineFormat` implementation in
`slideforge-plugin-api/src/inline_formats/` — this mirrors the `SlideType` and
`SectionType` patterns per ADR-016 Decision 2. `slideforge-plugin-api` is the only
dependency-legal owner: `slideforge-types` (the cycle-free alternative) cannot implement
plugin traits because `slideforge-plugin-api` depends on `slideforge-types`, creating a
cycle if reversed.

SS-06 (PPTX Exporter) is also in scope because this story refactors
`slideforge-pptx/src/notes_slide.rs::serialize_inline_nodes_to_xml()` (the internal
function that generates `<a:r>` OOXML directly) to route through the registered
`InlineFormat` plugin trait — satisfying BC-5.02.002 postcondition 5 and EC-004.

A third crate — `slideforge-types` — is touched by the pass-5 fix (F-P5-001/OBS-P5-001)
to add the shared `display_text_is_empty` predicate (see Scope B cross-crate note below).
`slideforge-types` is the foundational leaf crate that both SS-14 (slideforge-plugin-api)
and SS-06 (slideforge-pptx) already depend on; adding a pure helper function there
introduces no new dependency edge and no cycle. No subsystem frontmatter change is
required — `slideforge-types` carries no SS designator of its own; it is a shared
infrastructure crate underlying all subsystems.

## Dependency Anchor Justifications

- Depends on STORY-002: STORY-002 delivered the `InlineFormat` trait and `InlineOutputFormat`
  enum in `slideforge-plugin-api/src/traits/inline_format.rs`. This story provides the
  concrete `DefaultInlineFormat` implementation of that trait.
- Depends on STORY-028 (Layout: shape: Block + Rich Inline Formatting): STORY-028 introduced
  the `FrameContent::InlineRun` path and `InlineNode`-carrying layout structures. The
  `DefaultInlineFormat::render()` must handle all 12 `InlineNode` variants, and the test
  vectors for layout-level inline content come from STORY-028's definitions.
- Depends on STORY-037 (PPTX Core Serialization): STORY-037 established the OOXML output
  structures and the serialization patterns in `slideforge-pptx`. The refactor in
  STORY-085 targets the inline serialization path within `slideforge-pptx/src/notes_slide.rs`
  delivered or extended by STORY-037/STORY-038.
- Depends on STORY-038 (PPTX Layout Compliance): STORY-038 finalized placeholder inheritance
  and slide XML patterns. The OOXML `<a:r>` run format must be consistent with the patterns
  STORY-038 established in run properties (`<a:rPr>`).
- Blocks STORY-049: STORY-049 calls `register_inline_format()` with the bundled
  `DefaultInlineFormat`. Without STORY-085, the `InlineFormat` surface has only stub
  implementations, and STORY-049's AC-001 ("all 10 surfaces registered") fails.

## Summary

Two scopes, one story (inseparable because the OOXML render path must exist in
`slideforge-plugin-api/src/inline_formats/` before `slideforge-pptx` can route to it):

### Scope A: DefaultInlineFormat (slideforge-plugin-api)

Per ADR-017 (Option A, human-approved), Scope A also includes adding a defaulted
`render_with_context` method to the `InlineFormat` trait along with the
`InlineRenderContext` struct. This resolves the hyperlink relationship-ID problem
identified in the Previous Story Intelligence section without a breaking trait change.

```rust
pub struct InlineRenderContext<'a> {
    pub hyperlink_rid: Option<&'a str>,
}

// Default method added to InlineFormat trait (additive, non-breaking):
fn render_with_context(
    &self,
    node: &InlineNode,
    format: InlineOutputFormat,
    ctx: &InlineRenderContext<'_>,
) -> Result<String, InlineError> {
    self.render(node, format) // default: ignores context
}
```

`DefaultInlineFormat` overrides `render_with_context` specifically for the
`Link + Ooxml + Some(rid)` case: when `ctx.hyperlink_rid` is `Some(rid)`, it
emits the full `<a:r><a:rPr><a:hlinkClick r:id="{rid}"/></a:rPr><a:t>{text}</a:t></a:r>`
run. When `ctx.hyperlink_rid` is `None`, it falls back to display-text as a plain
run and emits `tracing::warn!("Hyperlink relationship not registered for OOXML link
to {url}")`. See EC-002 for the fallback behavior.

BC-5.02.001 v1.4 permits additive-defaulted methods (invariant 2 updated accordingly).
BC-5.02.002 v1.4 confirms hyperlink runs route through `render_with_context` — no
carve-out. Reference: ADR-017.

Create `slideforge-plugin-api/src/inline_formats/` module containing `DefaultInlineFormat`,
which implements `InlineFormat` for ALL 12 `InlineNode` variants × 3 output formats:

| Variant | Ooxml | Html | Markdown |
|---------|-------|------|----------|
| `Plain(Arc<str>)` | `<a:r><a:t>{text}</a:t></a:r>` | `{text}` (escaped) | `{text}` |
| `Bold(Vec<InlineNode>)` | `<a:r><a:rPr b="1"/><a:t>{content}</a:t></a:r>` | `<strong>{content}</strong>` | `**{content}**` |
| `Italic(Vec<InlineNode>)` | `<a:r><a:rPr i="1"/><a:t>{content}</a:t></a:r>` | `<em>{content}</em>` | `*{content}*` |
| `Code(Arc<str>)` | `<a:r><a:rPr ...monospace/><a:t>{text}</a:t></a:r>` | `<code>{text}</code>` | `` `{text}` `` |
| `Link { text, url }` | `<a:r>` with hyperlink relationship r:id + rendered text | `<a href="{url}">{text}</a>` | `[{text}]({url})` |
| `Math(MathNode)` | OMML passthrough (emit `<a14:m>` or `<m:oMath>` wrapper) | `<span class="math">{latex}</span>` | `${latex}$` or `$${latex}$$` |
| `Footnote(Vec<InlineNode>)` | Inline body content rendered; numbered marker (`[^{n}]` / `<sup>[{n}]</sup>`) is DEFERRED — see note below | `<sup>[inline body]</sup>` (no numbering) | `[inline body]` (no numbering) |
| `Xref(Arc<str>)` | Plain text run (cross-ref link resolution is a future story) | `<a href="#{id}">{id}</a>` | `[{id}](#{id})` |
| `Superscript(Vec<InlineNode>)` | `<a:r><a:rPr baseline="30000"/><a:t>{content}</a:t></a:r>` | `<sup>{content}</sup>` | `^{content}^` |
| `Subscript(Vec<InlineNode>)` | `<a:r><a:rPr baseline="-25000"/><a:t>{content}</a:t></a:r>` | `<sub>{content}</sub>` | `~{content}~` |
| `Strikethrough(Vec<InlineNode>)` | `<a:r><a:rPr strike="sngStrike"/><a:t>{content}</a:t></a:r>` | `<del>{content}</del>` | `~~{content}~~` |
| `Highlight(Vec<InlineNode>)` | `<a:r><a:rPr highlight="yellow"/><a:t>{content}</a:t></a:r>` | `<mark>{content}</mark>` | `=={content}==` |

The 12 variant × 3 format matrix is complete — no `InlineError::UnsupportedNode` paths
for the `DefaultInlineFormat` implementation. Nested variants (`Bold(Vec<InlineNode>)`)
recursively render inner nodes and concatenate their output.

**Footnote numbering deferral note.** v1.0 `DefaultInlineFormat` renders the
`Footnote(Vec<InlineNode>)` variant by rendering its inner body content inline — it
does NOT emit a numbered superscript marker (`[^{n}]` / `<sup>[n]</sup>`) because no
footnote-numbering mechanism exists yet. Numbered-marker footnote references (where a
sequential counter assigns `{n}` and the marker links to a footnote list) are
intentionally deferred to the same future story that implements cross-reference
resolution — currently noted in the `Xref` row as "cross-ref link resolution is a
future story". Until that story is scheduled and delivered, `Footnote` renders its body
content without a marker. This is a known limitation, not a bug. The absence of a
marker is explicitly logged via `tracing::debug!("Footnote marker numbering deferred")`.

**Xref cross-ref resolution note.** The `Xref(Arc<str>)` row emits `<a href="#{id}">` in
HTML/Ooxml modes. Full cross-reference link resolution (verifying the target anchor
exists, resolving across slides) is a future story. The current implementation produces
the `href` attribute mechanically from the `id` string.

### Scope B: PPTX OOXML Dog-Fooding Refactor (slideforge-pptx)

Refactor `slideforge-pptx/src/notes_slide.rs::serialize_inline_nodes_to_xml()` — the
internal function that directly constructs `<a:r>` OOXML without going through the
`InlineFormat` trait — to instead dispatch through the registered `InlineFormat`
implementation via `InlineOutputFormat::Ooxml`.

After the refactor:
- The `grep -r "a:r\|a:rPr\|a:t" crates/slideforge-pptx/src/` command returns zero
  matches OUTSIDE the single `InlineFormat` dispatch call site and the `DefaultInlineFormat`
  implementation in `slideforge-plugin-api`
- `slideforge-pptx`'s `serialize_inline_nodes_to_xml` function is removed or made a thin
  wrapper that calls `inline_format_registry.lookup_inline_format("default")?.render(node,
  InlineOutputFormat::Ooxml)`

This satisfies BC-5.02.002 postcondition 5 and the architectural grep-zero test vector.

**Cross-crate shared predicate (F-P5-001 / OBS-P5-001).** The orphan-External-relationship
invariant — rId registration count == `hlinkClick` emission count — is enforced by a single
shared predicate `display_text_is_empty(&[InlineNode]) -> bool` placed in
`slideforge-types/src/inline.rs`. Both the pptx exporter (rId registration guard in
`notes_slide.rs`) and the plugin-api formatter (`hlinkClick` emission in
`default_formatter.rs`) call this one function. Placing the predicate in any other location
would require either duplicating the logic across crates or creating a dependency inversion
that violates the DAG; `slideforge-types` is the only dependency-legal leaf crate reachable
by both consumers without introducing a new edge. This was introduced during the pass-5 fix
burst to close F-P5-001/OBS-P5-001.

**Registry routing note (F-006).** The notes serializer in `notes_slide.rs` is
registry-ready — it accepts `&dyn InlineFormat` at the call site. Full `PluginRegistry`
resolution (where the registry looks up the registered `"default"` `InlineFormat` by
name at the PPTX exporter call site) is wired in STORY-049 (Plugin Registry Assembly).
Until STORY-049 is delivered, STORY-085 passes the bundled `DefaultInlineFormat`
directly (as a concrete `&DefaultInlineFormat` instance, not a registry lookup). This is
not a shortcut — STORY-049 already blocks on STORY-085 (see `blocks` frontmatter), so
registry resolution arrives in the immediately following story.

## Behavioral Contracts

| BC | Version | Title | Covered ACs | Notes |
|----|---------|-------|-------------|-------|
| BC-5.02.001 | v1.4 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-004 | v1.4: invariant 2 now permits additive-defaulted methods; `render_with_context` addition is conformant |
| BC-5.02.002 | v1.4 | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-005 through AC-007 | v1.4: registration-vs-construction split clarified; hyperlink runs route through `render_with_context`; no carve-out for Link+OOXML path |
| BC-3.05.001 | v1.3.6 | HTML/OOXML Escaping of All Interpolated Values | AC-002 | v1.3.6: ALL interpolated values must be escaped (`& < > "`) — attribute positions (Link.url, Xref.id in href) AND text positions (Math.latex), not only Plain text; covers EC-009/EC-010 |

## Acceptance Criteria

### AC-001: DefaultInlineFormat covers all 12 InlineNode variants for Ooxml
(traces to BC-5.02.001 postcondition 2 — "InlineFormat: all 11 inline formatting types" — note: actual count is 12 per BC-3.05.001 v1.3.4 and inline.rs AC-005)

`DefaultInlineFormat::render(node, InlineOutputFormat::Ooxml)` returns `Ok(String)`
for all 12 `InlineNode` variants. No variant returns `Err(InlineError::UnsupportedNode)`
for the OOXML format. The output for `Plain("hello")` is `<a:r><a:t>hello</a:t></a:r>`.
The output for `Bold(vec![Plain("hi")])` contains `<a:rPr b="1"/>` and `<a:t>hi</a:t>`.

Unit test: cover all 12 variants for Ooxml. Snapshot each output via `insta`.

### AC-002: DefaultInlineFormat covers all 12 InlineNode variants for Html with full escaping
(traces to BC-5.02.001 postcondition 2; traces to BC-3.05.001 v1.3.6 postcondition 1 — all interpolated values escaped)

`DefaultInlineFormat::render(node, InlineOutputFormat::Html)` returns `Ok(String)`
for all 12 `InlineNode` variants. The output for `Bold(vec![Plain("hi")])` is
`<strong>hi</strong>`. The output for `Code("fn foo()")` is `<code>fn foo()</code>`.

Per BC-3.05.001 v1.3.6, escaping applies to ALL interpolated positions — not only
`Plain` text content:
- **Text positions:** `Plain` text, `Code` text, `Math.latex` — escape `&`, `<`, `>`, `"`
- **Attribute positions:** `Link.url` (in `href="..."`), `Xref.id` (in `href="#{id}"`) —
  escape `&`, `<`, `>`, `"` per EC-009/EC-010

Specific assertions:
- `Plain("a & b < c")` → Html: `"a &amp; b &lt; c"`
- `Link { text: "R&D", url: "https://ex.com/a&b" }` → Html:
  `<a href="https://ex.com/a&amp;b">R&amp;D</a>`
- `Xref("sec<1>")` → Html: `<a href="#sec&lt;1&gt;">sec&lt;1&gt;</a>`
- `Math(latex: "x & y")` → Html: `<span class="math">x &amp; y</span>`

Unit test: cover all 12 variants for Html; add dedicated escaping tests for url, id,
and latex attribute/text positions per BC-3.05.001 v1.3.6.

### AC-003: DefaultInlineFormat covers all 12 InlineNode variants for Markdown
(traces to BC-5.02.001 postcondition 2)

`DefaultInlineFormat::render(node, InlineOutputFormat::Markdown)` returns `Ok(String)`
for all 12 variants. The output for `Italic(vec![Plain("em")])` is `*em*`. The output
for `Strikethrough(vec![Plain("del")])` is `~~del~~`.

Unit test: cover all 12 variants for Markdown.

### AC-004: DefaultInlineFormat compiles using only slideforge-plugin-api public API
(traces to BC-5.02.001 postcondition 3 — no private API imports)

`DefaultInlineFormat` imports only from `slideforge_plugin_api::traits::inline_format::{InlineFormat, InlineError, InlineOutputFormat}` and `slideforge_types::InlineNode`. No imports
from any private or `pub(crate)` symbol. Verified by: `cargo build -p slideforge-plugin-api`
passes with zero new clippy warnings; `cargo tree -p slideforge-plugin-api` shows no new
crate deps.

### AC-005: slideforge-pptx no longer directly constructs <a:r> OOXML outside dispatch site
(traces to BC-5.02.002 postcondition 5 and EC-004)

After the refactor, the command:
```
grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/
```
returns zero matches EXCEPT for:
- The single call site in `slideforge-pptx` where inline nodes are dispatched to
  `InlineFormat::render(node, InlineOutputFormat::Ooxml)`
- Any OOXML assembly in the `InlineOutputFormat::Ooxml` path of `DefaultInlineFormat`
  (which lives in `slideforge-plugin-api`, not `slideforge-pptx`)

The internal function `serialize_inline_nodes_to_xml()` in `notes_slide.rs` is
removed. Its callers are updated to use the registered `InlineFormat`.

### AC-006: PPTX notes XML output is correct after the refactor
(traces to BC-5.02.002 postcondition 1 — cross-component via trait API)

All existing PPTX snapshot tests that cover note content pass after the refactor,
with the following fidelity expectations derived from the variant matrix above:

- **Plain / Bold / Italic:** The rendered `<a:r>` markup produced via
  `DefaultInlineFormat::render(node, Ooxml)` is byte-for-byte identical to the markup
  previously produced by the removed `serialize_inline_nodes_to_xml()` function. No
  regressions are tolerable for these three variants.
- **Code / Footnote / Superscript / Subscript / Strikethrough / Highlight:** The new
  output is deliberately richer than the legacy function's output. The legacy
  `serialize_inline_nodes_to_xml()` flattened these variants to plain or plain bold/italic
  runs; `DefaultInlineFormat` instead emits correct dedicated run properties
  (`Code` → `<a:latin typeface="Courier New"/>`, `Superscript` → `baseline="30000"`,
  `Subscript` → `baseline="-25000"`, `Strikethrough` → `strike="sngStrike"`,
  `Highlight` → `highlight="yellow"`). This is an intentional fidelity improvement,
  not a regression. The variant matrix in Scope A is the authority for expected output.

Verified by: existing snapshot tests in `slideforge-pptx` pass with `cargo insta test`.
Snapshot deltas for the six enriched variants above are expected and must be reviewed and
accepted; no unexpected deltas outside those six variants may be introduced by the refactor.

### AC-007: No production catch_unwind added; existing tests unaffected
(traces to BC-5.02.002 invariant 3 — verified continuously in CI)

The refactor in `slideforge-pptx` does not add any new `catch_unwind`, `panic!`,
`unwrap()`, or `expect()` calls on `Result` paths in production code. All `Result`
propagation uses `?` or `map_err`. Clippy pedantic passes on `slideforge-pptx`.

## Tasks

### Scope A — DefaultInlineFormat

- [ ] Create `crates/slideforge-plugin-api/src/inline_formats/mod.rs`:
  - Declare submodule `default_formatter`
  - Re-export `DefaultInlineFormat` as public
- [ ] Add `InlineRenderContext` struct to `crates/slideforge-plugin-api/src/traits/inline_format.rs`
  and add the defaulted `render_with_context(&self, node, format, ctx: &InlineRenderContext)`
  method to the `InlineFormat` trait (per ADR-017 Option A); default impl delegates to
  `self.render(node, format)` — non-breaking for all existing implementors
- [ ] Add `tracing = { workspace = true }` to `crates/slideforge-plugin-api/Cargo.toml`
  (required for EC-001/EC-002 `tracing::warn!` fallback paths; workspace-managed, pinned)
- [ ] Create `crates/slideforge-plugin-api/src/inline_formats/default_formatter.rs`:
  - `DefaultInlineFormat` struct (unit struct — stateless formatter)
  - `impl InlineFormat for DefaultInlineFormat`
  - `id()` returns `"default"`
  - `render(node, format)` — full `match (node, format)` over all 12 × 3 combinations
  - `render_with_context(node, format, ctx)` — override for `Link + Ooxml + Some(rid)`:
    emit `<a:r><a:rPr><a:hlinkClick r:id="{rid}"/></a:rPr><a:t>{text}</a:t></a:r>`;
    for `Link + Ooxml + None`, fall back to display-text plain run with `tracing::warn!`
  - Recursive rendering for nested variants (`Bold`, `Italic`, `Highlight`, etc.):
    inner `Vec<InlineNode>` nodes are rendered recursively and their outputs concatenated
  - Math OOXML path: for `InlineNode::Math`, the `MathNode` struct has no pre-rendered
    OMML field (`MathNode` exposes only `latex`, `display`, and `span` in v1.0 — no
    `omml` field exists). Always emit the fallback
    `<a:r><a:t>{latex}</a:t></a:r>` with `tracing::warn!("Math OMML rendering not
    pre-computed for OOXML")` — do NOT panic. OMML pre-rendering is future work;
    do NOT reference a `MathNode.omml` field that does not exist.
  - XML special-character escaping for OOXML text content (`<a:t>` content must escape
    `&`, `<`, `>` as XML entities)
  - HTML character escaping for ALL interpolated positions per BC-3.05.001 v1.3.6:
    text content AND attribute positions (`Link.url`, `Xref.id`, `Math.latex`)
  - Footnote: render inner body content inline; emit
    `tracing::debug!("Footnote marker numbering deferred")` — no numeric marker
- [ ] Add `pub mod inline_formats;` to `crates/slideforge-plugin-api/src/lib.rs`
- [ ] Export `DefaultInlineFormat` and `InlineRenderContext` from `lib.rs` public API
- [ ] Write unit tests (in `default_formatter.rs` `#[cfg(test)] mod tests`):
  - All 12 variants × Ooxml: snapshot via `insta`
  - All 12 variants × Html: snapshot via `insta`
  - All 12 variants × Markdown: snapshot via `insta`
  - Nested variant: `Bold(vec![Italic(vec![Plain("text")])])` → verify nesting works
  - HTML escaping (text position): `Plain("a & b < c")` → Html: `"a &amp; b &lt; c"`
  - HTML escaping (attribute position — EC-009): `Link { text: "R&D", url: "https://ex.com/a&b" }`
    → Html: `<a href="https://ex.com/a&amp;b">R&amp;D</a>`
  - HTML escaping (attribute position — EC-010): `Xref("sec<1>")` → Html:
    `<a href="#sec&lt;1&gt;">sec&lt;1&gt;</a>`
  - HTML escaping (text position — EC-010): `Math(latex: "x & y")` → Html:
    `<span class="math">x &amp; y</span>`
  - OOXML XML escaping: `Plain("a & b")` → Ooxml: `"<a:r><a:t>a &amp; b</a:t></a:r>"`
  - `render_with_context` with `Some(rid)`: `Link { text: "click", url: "..." }` + Ooxml
    → `<a:r><a:rPr><a:hlinkClick r:id="rId1"/></a:rPr><a:t>click</a:t></a:r>`
  - `render_with_context` with `None`: emits plain text run + `tracing::warn!` fired

### Scope B — PPTX Refactor

- [ ] In `crates/slideforge-pptx/src/notes_slide.rs`:
  - Remove `serialize_inline_nodes_to_xml()` and `serialize_inline_nodes_inner()` helper
  - Add a parameter or context struct that carries a reference to the registered
    `InlineFormat` plugin (from the `PluginRegistry` passed into the exporter)
  - Replace all call sites of the removed functions with:
    `inline_format.render(node, InlineOutputFormat::Ooxml).unwrap_or_else(|_| String::new())`
    (or proper `?` propagation if the exporter's error type can carry `InlineError`)
- [ ] Verify zero remaining `a:r` / `a:rPr` construction in `slideforge-pptx/src/` (outside
  the new dispatch call site) by running the canonical grep test vector
- [ ] Run `cargo insta test -p slideforge-pptx` to confirm snapshot stability
- [ ] Run `cargo clippy -p slideforge-pptx --all-targets -- -D warnings` to confirm clean

## Previous Story Intelligence

This story builds on patterns from STORY-037 (PPTX Core Serialization) and STORY-028
(shape: Block + Rich Inline Formatting). Key lesson from those stories: OOXML run markup
for plain text must always wrap `<a:t>` inside `<a:r>`, even for unstyled runs. The
`<a:r>` element is mandatory — naked `<a:t>` is not valid OOXML.

The refactor in Scope B follows the pattern of replacing a local private function with a
trait dispatch call. The previous open question about the `Link` OOXML hyperlink path
(whether to extend the trait signature or fall back to display-text with a warning) has
been resolved by ADR-017 Option A (human-approved): the `InlineFormat` trait receives a
defaulted `render_with_context` method plus the `InlineRenderContext { hyperlink_rid:
Option<&str> }` struct. This is non-breaking (default impl delegates to `render`) and
production-grade (the exporter can supply the relationship ID when available). See Scope
A summary above and the Tasks section for implementation details.

**Do not silently drop link information** — the `None` path must emit `tracing::warn!`
and the display text as a plain run (EC-002).

## Architecture Compliance Rules

1. **`DefaultInlineFormat` in slideforge-plugin-api only.** No OOXML generation code
   in the slideforge-plugin-api module may be moved into `slideforge-pptx`.
2. **`InlineRenderContext` and `render_with_context` in the trait file.** Per ADR-017,
   these additions go in `crates/slideforge-plugin-api/src/traits/inline_format.rs`
   alongside `InlineFormat`. They are part of the public plugin API surface, not
   internal to `default_formatter.rs`.
3. **Dog-fooding enforced (BC-5.02.002).** After the refactor, `slideforge-pptx` produces
   zero `<a:r>` markup outside the single dispatch call site. The grep-zero test vector
   is the architectural invariant check.
4. **Snapshot test stability.** The refactor is output-preserving. If any snapshot delta
   is introduced, the implementer must investigate whether the `DefaultInlineFormat` output
   differs from the removed private function — this is a correctness regression, not a
   "snapshot update" scenario.
5. **`#![forbid(unsafe_code)]`** in force for both crates; no unsafe additions.
6. **`clippy::pedantic` clean** on both `slideforge-plugin-api` and `slideforge-pptx`
   after all changes.
7. **No new catch_unwind** in production code. The `InlineFormat::render()` returns
   `Result<String, InlineError>` — propagate errors properly.

## Library & Framework Requirements

| Library | Version | Purpose | Crate |
|---------|---------|---------|-------|
| `slideforge-types` | workspace (already dep) | `InlineNode`, `MathNode` types | slideforge-plugin-api |
| `slideforge-plugin-api` | workspace | `InlineFormat` trait, `InlineOutputFormat`, `InlineError` | slideforge-pptx |
| `insta` | workspace (already dep in slideforge-pptx) | Snapshot tests for render output | both (dev-dep) |
| `tracing-test` | `=0.2.5` (dev-dep) | Assert tracing log events fire in unit tests (footnote-deferral debug, Math/Link warn fallbacks) — load-bearing per TD-VSDD-059 | slideforge-plugin-api |
| `ooxmlsdk` | `=0.6.1` | OOXML types reference (no new dep — already in slideforge-pptx) | slideforge-pptx |
| `tracing` | `{ workspace = true }` | `tracing::warn!` for EC-001 (Math OMML missing) and EC-002 (Link RID not available) fallback paths — mandated by EC-001/EC-002 | slideforge-plugin-api |

`tracing` is a new dependency for `slideforge-plugin-api` (workspace-managed, pinned).
It is required because EC-001 and EC-002 mandate non-silent fallback via `tracing::warn!`
on the Math OMML and Link RID missing code paths. Do not use `eprintln!` or `println!`
as substitutes — the conventions section explicitly forbids `println!` in library crates.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-plugin-api/src/traits/inline_format.rs` | Modify | Add `InlineRenderContext` struct + defaulted `render_with_context` method to `InlineFormat` trait (ADR-017) |
| `crates/slideforge-plugin-api/Cargo.toml` | Modify | Add `tracing = { workspace = true }` dependency |
| `crates/slideforge-plugin-api/src/inline_formats/mod.rs` | Create | Module root, re-exports |
| `crates/slideforge-plugin-api/src/inline_formats/default_formatter.rs` | Create | DefaultInlineFormat impl (12 variants × 3 formats + render_with_context override) |
| `crates/slideforge-plugin-api/src/lib.rs` | Modify | Add `pub mod inline_formats;` + re-export `DefaultInlineFormat`, `InlineRenderContext` |
| `crates/slideforge-pptx/src/notes_slide.rs` | Modify | Remove serialize_inline_nodes_to_xml; add trait dispatch via `DefaultInlineFormat` |
| `crates/slideforge-types/src/inline.rs` | Modify | Add shared `pub fn display_text_is_empty(nodes: &[InlineNode]) -> bool` — single source of truth for Link display-text emptiness, used by both the pptx exporter (rId registration guard) and the plugin-api formatter (hlinkClick emission), preventing the F-P5-001 orphan-relationship drift (OBS-P5-001) |
| `crates/slideforge-plugin-api/tests/module_boundary_test.rs` | Create | Fitness-function test scanning `inline_formats/*.rs` for forbidden `use` imports |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~4,000 |
| BC-5.02.001 v1.4 (postcondition 2-3, invariant 2) | ~800 |
| BC-5.02.002 v1.4 (postconditions 1, 5; EC-004) | ~1,000 |
| BC-3.05.001 v1.3.6 (all interpolated positions — EC-009/EC-010) | ~600 |
| ADR-016 Decisions 2+3 | ~600 |
| ADR-017 Option A (render_with_context + InlineRenderContext) | ~400 |
| `InlineFormat` trait + `InlineRenderContext` + `InlineOutputFormat` + `InlineError` (inline_format.rs) | ~2,200 |
| `InlineNode` enum all 12 variants (inline.rs) | ~1,500 |
| `notes_slide.rs` serialize_inline_nodes_to_xml (lines 232–311) | ~1,500 |
| Test and snapshot files to write | ~2,500 |
| **Total** | **~15,100** |

Context budget: ~15% of a 100k-token context window. Within limit for an 8-point story.

## Test Strategy

- **Unit tests in `default_formatter.rs`**: Full 12 × 3 matrix via `insta` snapshots.
  Nesting tests for recursive variants. Escaping tests for HTML and OOXML text content,
  including attribute-position escaping for `Link.url`, `Xref.id`, and `Math.latex`
  per BC-3.05.001 v1.3.6.
- **Snapshot regression**: Run `cargo insta test -p slideforge-pptx` before and after
  the refactor. Zero new snapshots should appear (the rendered output should be identical).
  If new snapshots appear, investigate the diff before accepting.
- **Grep architectural test**: `grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/`
  must return only the dispatch call site after the refactor. Add this as an explicit
  test that runs in CI (via `#[test]` that shells out to grep or via a dedicated audit
  test that scans the source tree).
- **VP-053 formal backing note**: The concrete escaping unit tests in `default_formatter.rs`
  (verifying `& < > "` are escaped in all interpolated positions) are formally backed by
  VP-053 (inline-html-escape-all-interpolated). VP-053's exhaustive proptest harness —
  which exercises arbitrary Unicode input across all 12 × 3 render paths — is implemented
  in Phase 6 formal hardening. The `verification_properties` frontmatter remains `[]`
  because the VP-053 proptest implementation is Phase 6 work; the concrete unit tests
  delivered by this story provide deterministic spot-coverage of the same property.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `InlineNode::Math` with OOXML path — `MathNode` has no pre-rendered OMML | Log `tracing::warn!("Math OMML rendering not pre-computed for OOXML")` and emit fallback `<a:r><a:t>{latex}</a:t></a:r>`; do NOT panic |
| EC-002 | `InlineNode::Link` in OOXML — relationship ID not available in render context | Emit display text as plain run with `tracing::warn!("Hyperlink relationship not registered for OOXML link to {url}")` — do NOT silently drop the text |
| EC-003 | `InlineNode::Plain` with `&` or `<` in OOXML mode | XML-escape to `&amp;` / `&lt;` before wrapping in `<a:t>` |
| EC-004 | Recursive nesting depth > 64 (BC-3.05.001 invariant — max 64 depth) | Return `Err(InlineError::RenderError { node_kind, message: "inline nesting depth exceeds maximum of 64" })` — do NOT stack overflow |
| EC-009 | `Link.url` contains `&` or `<` in HTML attribute position (BC-3.05.001 v1.3.6) | Escape to `&amp;` / `&lt;` inside the `href="..."` attribute value; e.g., `href="https://ex.com/a&amp;b"` |
| EC-010 | `Xref.id` or `Math.latex` contains `&`, `<`, `>`, or `"` in HTML output | Escape ALL four characters in attribute positions (`href="#{id}"`) and text positions (`<span class="math">{latex}</span>`); do NOT pass raw values through |

## Forbidden Dependencies

`slideforge-plugin-api/src/inline_formats/` MUST NOT:
- Depend on `slideforge-pptx`, `slideforge-docx`, `slideforge-pdf`, `slideforge-html`
- Depend on `slideforge-eval`, `slideforge-syntax`, `slideforge-layout`
- Depend on any crate not already in `slideforge-plugin-api/Cargo.toml`

`slideforge-pptx/src/notes_slide.rs` refactored code MUST NOT:
- Call private / `pub(crate)` functions in `slideforge-plugin-api` — only the public
  `InlineFormat::render()` trait method
- Introduce new `catch_unwind` in the dispatch path
- Add `slideforge-eval`, `slideforge-syntax`, or `slideforge-layout` as dependencies

Build-time enforcement: enforced by
`crates/slideforge-plugin-api/tests/module_boundary_test.rs` — a fitness-function test
that scans `inline_formats/*.rs` for forbidden `use` imports and fails the test suite if
any are found. Note: `slideforge-layout` is already linked by `slideforge-plugin-api`, so
the crate-not-in-Cargo.toml guard does NOT apply to it — the fitness-function test is the
authoritative enforcement mechanism for all six forbidden crates listed above.
