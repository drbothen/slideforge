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
behavioral_contracts: [BC-5.02.001, BC-5.02.002]
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
| `Footnote(Vec<InlineNode>)` | Footnote superscript marker + run | `<sup>[{n}]</sup>` | `[^{n}]` |
| `Xref(Arc<str>)` | Plain text run (cross-ref link resolution is a future story) | `<a href="#{id}">{id}</a>` | `[{id}](#{id})` |
| `Superscript(Vec<InlineNode>)` | `<a:r><a:rPr baseline="30000"/><a:t>{content}</a:t></a:r>` | `<sup>{content}</sup>` | `^{content}^` |
| `Subscript(Vec<InlineNode>)` | `<a:r><a:rPr baseline="-25000"/><a:t>{content}</a:t></a:r>` | `<sub>{content}</sub>` | `~{content}~` |
| `Strikethrough(Vec<InlineNode>)` | `<a:r><a:rPr strike="sngStrike"/><a:t>{content}</a:t></a:r>` | `<del>{content}</del>` | `~~{content}~~` |
| `Highlight(Vec<InlineNode>)` | `<a:r><a:rPr highlight="yellow"/><a:t>{content}</a:t></a:r>` | `<mark>{content}</mark>` | `=={content}==` |

The 12 variant × 3 format matrix is complete — no `InlineError::UnsupportedNode` paths
for the `DefaultInlineFormat` implementation. Nested variants (`Bold(Vec<InlineNode>)`)
recursively render inner nodes and concatenate their output.

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

## Behavioral Contracts

| BC | Title | Covered ACs |
|----|-------|-------------|
| BC-5.02.001 | All 10 Plugin Trait Surfaces Implemented by Bundled Plugins via the Public Trait API | AC-001 through AC-004 |
| BC-5.02.002 | No Bundled Plugin Bypasses the Registered Trait Interface (Dog-Fooding Guarantee) | AC-005 through AC-007 |

## Acceptance Criteria

### AC-001: DefaultInlineFormat covers all 12 InlineNode variants for Ooxml
(traces to BC-5.02.001 postcondition 2 — "InlineFormat: all 11 inline formatting types" — note: actual count is 12 per BC-3.05.001 v1.3.4 and inline.rs AC-005)

`DefaultInlineFormat::render(node, InlineOutputFormat::Ooxml)` returns `Ok(String)`
for all 12 `InlineNode` variants. No variant returns `Err(InlineError::UnsupportedNode)`
for the OOXML format. The output for `Plain("hello")` is `<a:r><a:t>hello</a:t></a:r>`.
The output for `Bold(vec![Plain("hi")])` contains `<a:rPr b="1"/>` and `<a:t>hi</a:t>`.

Unit test: cover all 12 variants for Ooxml. Snapshot each output via `insta`.

### AC-002: DefaultInlineFormat covers all 12 InlineNode variants for Html
(traces to BC-5.02.001 postcondition 2)

`DefaultInlineFormat::render(node, InlineOutputFormat::Html)` returns `Ok(String)`
for all 12 `InlineNode` variants. The output for `Bold(vec![Plain("hi")])` is
`<strong>hi</strong>`. The output for `Code("fn foo()")` is `<code>fn foo()</code>`.
HTML special characters in `Plain` text are escaped (`&`, `<`, `>`, `"` → entities).

Unit test: cover all 12 variants for Html.

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

### AC-006: PPTX notes XML output is unchanged after the refactor
(traces to BC-5.02.002 postcondition 1 — cross-component via trait API)

All existing PPTX snapshot tests that cover note content pass unchanged after the
refactor. The rendered `<a:r>` markup produced via `DefaultInlineFormat::render(node,
Ooxml)` must be byte-for-byte identical to the markup previously produced by the
removed `serialize_inline_nodes_to_xml()` function (for the same input).

Verified by: existing snapshot tests in `slideforge-pptx` pass with `cargo insta test`.
No snapshot deltas are introduced by the refactor.

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
- [ ] Create `crates/slideforge-plugin-api/src/inline_formats/default_formatter.rs`:
  - `DefaultInlineFormat` struct (unit struct — stateless formatter)
  - `impl InlineFormat for DefaultInlineFormat`
  - `id()` returns `"default"`
  - `render(node, format)` — full `match (node, format)` over all 12 × 3 combinations
  - Recursive rendering for nested variants (`Bold`, `Italic`, `Highlight`, etc.):
    inner `Vec<InlineNode>` nodes are rendered recursively and their outputs concatenated
  - Math OOXML path: for `InlineNode::Math`, the OMML source is already in `MathNode.omml`
    (if available from STORY-029); otherwise emit a fallback `<a:t>{latex}</a:t>` with a
    `tracing::warn!` — do NOT panic
  - XML special-character escaping for OOXML text content (`<a:t>` content must escape
    `&`, `<`, `>` as XML entities)
  - HTML character escaping for the Html format
- [ ] Add `pub mod inline_formats;` to `crates/slideforge-plugin-api/src/lib.rs`
- [ ] Export `DefaultInlineFormat` from `lib.rs` public API
- [ ] Write unit tests (in `default_formatter.rs` `#[cfg(test)] mod tests`):
  - All 12 variants × Ooxml: snapshot via `insta`
  - All 12 variants × Html: snapshot via `insta`
  - All 12 variants × Markdown: snapshot via `insta`
  - Nested variant: `Bold(vec![Italic(vec![Plain("text")])])` → verify nesting works
  - HTML escaping: `Plain("a & b < c")` → Html: `"a &amp; b &lt; c"`
  - OOXML XML escaping: `Plain("a & b")` → Ooxml: `"<a:r><a:t>a &amp; b</a:t></a:r>"`

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
trait dispatch call. The key risk is that the existing `serialize_inline_nodes_to_xml()`
function took `hlink_urls: &[String]` as a context parameter for hyperlink relationship
IDs. The `DefaultInlineFormat::render()` method does not have access to relationship IDs
— the `Link` OOXML path will need to be handled differently:
- Either the `InlineFormat::render()` signature is extended to accept a relationship
  context (breaking change to the trait — evaluate against BC-5.02.001 invariant 2),
- Or the `Link` variant in OOXML mode emits a simplified hyperlink run without a
  relationship (emitting the URL as plain text with a `tracing::warn!` that relationship-based
  hyperlinks require the exporter to call a separate relationship-registration API).

The implementer must choose the production-grade path: if the trait signature extension
is the correct fix, create a follow-up story or expand scope (human-authorized). If the
simplified URL-as-text fallback is acceptable for v1.0 with a warning, document it.
**Do not silently drop link information** — the OOXML output must either have a working
hyperlink relationship or explicitly log that the relationship was not registered.

## Architecture Compliance Rules

1. **`DefaultInlineFormat` in slideforge-plugin-api only.** No OOXML generation code
   in the slideforge-plugin-api module may be moved into `slideforge-pptx`.
2. **Dog-fooding enforced (BC-5.02.002).** After the refactor, `slideforge-pptx` produces
   zero `<a:r>` markup outside the single dispatch call site. The grep-zero test vector
   is the architectural invariant check.
3. **Snapshot test stability.** The refactor is output-preserving. If any snapshot delta
   is introduced, the implementer must investigate whether the `DefaultInlineFormat` output
   differs from the removed private function — this is a correctness regression, not a
   "snapshot update" scenario.
4. **`#![forbid(unsafe_code)]`** in force for both crates; no unsafe additions.
5. **`clippy::pedantic` clean** on both `slideforge-plugin-api` and `slideforge-pptx`
   after all changes.
6. **No new catch_unwind** in production code. The `InlineFormat::render()` returns
   `Result<String, InlineError>` — propagate errors properly.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `slideforge-types` | workspace (already dep) | `InlineNode`, `MathNode` types |
| `slideforge-plugin-api` | workspace | `InlineFormat` trait, `InlineOutputFormat`, `InlineError` |
| `insta` | workspace (already dep in slideforge-pptx) | Snapshot tests for render output |
| `ooxmlsdk` | `=0.6.1` | OOXML types reference (no new dep — already in slideforge-pptx) |

No new library dependencies required for either crate.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/slideforge-plugin-api/src/inline_formats/mod.rs` | Create | Module root, re-exports |
| `crates/slideforge-plugin-api/src/inline_formats/default_formatter.rs` | Create | DefaultInlineFormat impl (12 variants × 3 formats) |
| `crates/slideforge-plugin-api/src/lib.rs` | Modify | Add `pub mod inline_formats;` + re-export |
| `crates/slideforge-pptx/src/notes_slide.rs` | Modify | Remove serialize_inline_nodes_to_xml; add trait dispatch |

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|-----------------|
| This story spec | ~3,000 |
| BC-5.02.001 (postcondition 2-3) | ~800 |
| BC-5.02.002 (postconditions 1, 5; EC-004) | ~1,000 |
| ADR-016 Decisions 2+3 | ~600 |
| `InlineFormat` trait + `InlineOutputFormat` + `InlineError` (inline_format.rs) | ~2,000 |
| `InlineNode` enum all 12 variants (inline.rs) | ~1,500 |
| `notes_slide.rs` serialize_inline_nodes_to_xml (lines 232–311) | ~1,500 |
| Test and snapshot files to write | ~2,000 |
| **Total** | **~12,400** |

Context budget: ~12% of a 100k-token context window. Within limit for a 8-point story.

## Test Strategy

- **Unit tests in `default_formatter.rs`**: Full 12 × 3 matrix via `insta` snapshots.
  Nesting tests for recursive variants. Escaping tests for HTML and OOXML text content.
- **Snapshot regression**: Run `cargo insta test -p slideforge-pptx` before and after
  the refactor. Zero new snapshots should appear (the rendered output should be identical).
  If new snapshots appear, investigate the diff before accepting.
- **Grep architectural test**: `grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/`
  must return only the dispatch call site after the refactor. Add this as an explicit
  test that runs in CI (via `#[test]` that shells out to grep or via a dedicated audit
  test that scans the source tree).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `InlineNode::Math` with OOXML path — `MathNode` has no pre-rendered OMML | Log `tracing::warn!("Math OMML rendering not pre-computed for OOXML")` and emit fallback `<a:r><a:t>{latex}</a:t></a:r>`; do NOT panic |
| EC-002 | `InlineNode::Link` in OOXML — relationship ID not available in render context | Emit display text as plain run with `tracing::warn!("Hyperlink relationship not registered for OOXML link to {url}")` — do NOT silently drop the text |
| EC-003 | `InlineNode::Plain` with `&` or `<` in OOXML mode | XML-escape to `&amp;` / `&lt;` before wrapping in `<a:t>` |
| EC-004 | Recursive nesting depth > 64 (BC-3.05.001 invariant — max 64 depth) | Return `Err(InlineError::RenderError { node_kind, message: "inline nesting depth exceeds maximum of 64" })` — do NOT stack overflow |

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

Build-time enforcement: any import from a forbidden crate produces a compilation error
because the forbidden crates are not in the respective `Cargo.toml` dep lists.
