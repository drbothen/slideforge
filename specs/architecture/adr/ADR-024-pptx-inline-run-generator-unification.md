---
document_type: adr
adr_id: ADR-024
title: PPTX inline-run generator unification — single recursive engine for body and notes
status: accepted
date: 2026-06-09
approved: 2026-06-09
producer: architect
deciders: [architect, human]
story: STORY-081
traces_to: ARCH-INDEX.md
supersedes: null
superseded_by: null
resolves: FU-PPTX-DUAL-RUN-GENERATOR
---

# ADR-024 — PPTX Inline-Run Generator Unification

## Status

**ACCEPTED.** Human authorized scope expansion on 2026-06-09 within STORY-081. The
implementation (write no code; TDD execution follows this ADR) proceeds according to
the Migration Plan below.

---

## Context

### The Dual-Generator Problem

PPTX inline-OOXML run generation has, since STORY-085, been driven by two independent
implementations that both consume `&[InlineNode]` and both produce OOXML `<a:r>` run
strings:

**Body path** — `slideforge-pptx/src/slide_serializer.rs`
Processes `FrameContent::Body` and `FrameContent::TextRun` slide frames. Currently
calls `extract_body_text` / `extract_inline_text`, which discard all inline markup
and produce plain text strings passed to `build_shape`. This means body frames do
not yet emit rich OOXML runs — inline markup is extracted to plain text for shape
construction in the current STORY-081 implementation scope. The rich-OOXML body
path is introduced by STORY-081 via a new `build_rich_shape` / `inline_runs_to_ooxml`
helper that uses typed `ooxmlsdk` `Run` / `RunProperties` builders for schema-correct
child ordering.

**Notes path** — `slideforge-pptx/src/notes_slide.rs`
Processes `Register::Notes` `InlineNode` slices. Uses `dispatch_inline_nodes_to_ooxml`
which iterates a flat node slice and calls `InlineFormat::render_with_context` once
per top-level node. For `Link` nodes at the top level this correctly threads the
pre-registered `hyperlink_rid` from `InlineRenderContext`. For all other nodes it
delegates to `render()` → `render_ooxml_accumulate`, which is the raw-string
accumulate path inside `DefaultInlineFormat`.

### Three Divergence Findings

These three findings were produced by adversarial review of STORY-081:

**ADV-P11-HIGH-001** (Pass 11, remediated):
The notes path emitted `highlight="yellow"` as an ATTRIBUTE on `<a:rPr>`, while the
body path used the `<a:highlight><a:srgbClr val="FFFF00"/>` CHILD element (the
schema-correct form). Root cause: `DefaultInlineFormat::emit_run` (used by the notes
path) and the typed ooxmlsdk body builder were independent implementations that
diverged on the highlight representation. Remediated by fixing `emit_run`, but the
architectural risk — any future inline-form change must be applied to both paths in
lockstep — remained.

**F-P15-HIGH-001** (Pass 15, remediated):
The body path orphaned an External relationship for a `Link` nested inside another
`Link`'s display text (`Link{ url:U1, text:[Link{url:U2}] }`). The body path
collected both U1 and the inner U2 as relationship candidates, then the inner arm
emitted `hyperlink_rid: None` (no rId lookup hit), resulting in one External rel
with zero `<a:hlinkClick>` runs. The notes path avoided this because it flattens
all link display text to plain text (never recurses into Link display children).
The asymmetry was a direct consequence of the dual-generator design.

**F-P16-M1** (Pass 16, the finding that triggered this ADR):
The notes path silently drops bold, italic, and other formatting inside a hyperlink's
display text. `[click **here**](https://example.com)` in notes OOXML renders as plain
`click here` with no `<a:rPr b="1"/>` on the bold span. The body path and HTML path
preserve the formatting. The root cause is structural: `dispatch_inline_nodes_to_ooxml`
iterates the entry slice node-by-node and calls `render_with_context` once per top-level
node. A `Link` node's `text` children are not iterated at the dispatch level — they are
passed as a single `Vec<InlineNode>` into `DefaultInlineFormat::render_with_context`,
which flattens them via `extract_plain_text_depth_limited`. For non-Link nodes the
same single dispatch call works correctly because `render_ooxml_accumulate` recurses
into children with `RunProps` accumulation. But the `Link + Ooxml + Some(rid)` arm in
`render_with_context` bypasses `render_ooxml_accumulate` entirely and calls
`extract_plain_text_depth_limited` — which is intentionally a plain-text extractor,
discarding all formatting structure. The result: `**here**` → `here`.

This is a BC-3.02.002 PC8 violation ("all output formats") confirmed by the human as
a defect requiring a fix in STORY-081 scope.

### The Structural Root Cause

All three findings share a single architectural root: two generators for the same
input domain. The body path uses typed ooxmlsdk builders with `RunProps` accumulation
and node-by-node recursive descent. The notes path uses a flat top-level dispatch
loop that calls into `render_with_context` once per top-level node. The `render_with_context`
signature takes a single `InlineNode`, so it can only see one node at a time. For
formatting wrappers like `Bold`, this is fine because `render_ooxml_accumulate` recurses
into children. For `Link`, it fails because the `Link + Ooxml + Some(rid)` arm calls
`extract_plain_text_depth_limited` instead of recursively rendering children with
`RunProps` accumulation.

The `render_with_context` API contract (ADR-017 Option A) was designed to solve the
hyperlink-rId threading problem for the notes path. It succeeded for top-level plain
links. It did not anticipate the need to thread the rId through nested formatting
wrappers while simultaneously preserving their `RunProps` accumulation context. That
requires iterating the display text children with the full `render_ooxml_accumulate`
machinery, not flattening them.

### Human Decision (Binding)

The human authorized unification of the two PPTX inline-run generators inside
STORY-081 on 2026-06-09. The decision is binding and recorded here as the permanent
architectural rationale. The plug-in-first dog-fooding guarantee (BC-5.02.002 / ADR-006)
prohibits bundled plugins from bypassing the `InlineFormat` trait API — having the
notes path use a different internal mechanism from the body path is exactly that smell.

---

## Decision

### Chosen Representation: Typed RunSpec (neutral intermediate)

The unified engine produces a `Vec<OoxmlRun>` rather than either (a) typed ooxmlsdk
`Run` objects or (b) raw-string OOXML fragments.

`OoxmlRun` is a new neutral typed struct in `slideforge-plugin-api` (alongside
`DefaultInlineFormat`, or in a dedicated `ooxml_run` submodule):

```rust
/// A single OOXML text run, ready for serialization into `<a:r>` markup.
///
/// Represents the complete run-property state at a single leaf in the inline
/// node tree plus the text content. All eight OOXML run-property axes are
/// captured independently — they are not mutually exclusive and may be combined
/// (e.g., bold + italic + strikethrough in a single run).
///
/// `OoxmlRun` is the canonical intermediate representation shared by the slide-body
/// and notes-slide PPTX serializers. Neither serializer constructs `<a:r>` markup
/// directly; both call `render_inline_nodes_to_runs` and then serialize the
/// resulting `Vec<OoxmlRun>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OoxmlRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub baseline: Option<i32>,
    pub strike: bool,
    pub highlight: bool,
    pub code_font: bool,
    pub hyperlink_rid: Option<String>,
}
```

`OoxmlRun` satisfies `Hash + Eq + Clone` (ADR-013 / comemo compatibility).

### Rationale for RunSpec over Raw String or Typed ooxmlsdk

**Over raw-string OOXML:** The raw-string path requires manually maintaining child
element ordering for `<a:rPr>`. This is the mechanism that caused ADV-P11-HIGH-001
(the highlight attribute vs element divergence). `OoxmlRun`'s serializer is the
SINGLE place that converts properties to XML — child ordering is structurally
guaranteed by one function, not two.

**Over typed ooxmlsdk `Run` directly:** The `ooxmlsdk` typed builders are a
`slideforge-pptx` dependency, not a `slideforge-plugin-api` dependency. Moving
`OoxmlRun` to `slideforge-plugin-api` (where `DefaultInlineFormat` lives) avoids
adding an `ooxmlsdk` dependency to the plugin-api crate. The body path in
`slideforge-pptx` can convert `Vec<OoxmlRun>` to typed ooxmlsdk `Run` objects
at the point of consumption (one small conversion function), preserving ooxmlsdk's
schema-correct child ordering guarantee for the body path. The notes path, which
uses raw-string XML, serializes `OoxmlRun` to string directly — but via the SAME
serialization function, so the child ordering is identical for both consumers.

**Alternative considered — typed ooxmlsdk in plugin-api:**
Adding `ooxmlsdk` as a dependency of `slideforge-plugin-api` would expose the
plugin surface to an OOXML-specific type, coupling all future `InlineFormat`
implementors to the ooxmlsdk version. `OoxmlRun` is a plain struct with no
external crate dependencies and is stable under version changes. Rejected.

### The Unified Engine Function

A new function `render_inline_nodes_to_runs` is added to
`slideforge-plugin-api/src/inline_formats/ooxml_runs.rs` (new module):

```rust
/// Render a sequence of `InlineNode`s to a flat list of typed OOXML runs.
///
/// This is the SINGLE inline-run generator for all PPTX serializers.
/// Both the slide-body path and the notes-slide path call this function.
///
/// # Parameters
///
/// - `nodes`: the top-level inline node slice to render
/// - `hlink_resolver`: a callback that maps a URL to a pre-registered
///   relationship ID, or `None` if the URL has not been registered or
///   is not safe-scheme. The resolver is called once per `Link` node
///   encountered during recursive descent (both top-level and inside
///   formatting wrappers). The notes path and body path provide
///   different resolver closures backed by their respective `hlink_urls`
///   / `hlink_map` data structures.
///
/// # Returns
///
/// A `Vec<OoxmlRun>` in document order. Empty runs (where `text` is
/// empty after plain-text extraction) are omitted.
///
/// # Errors
///
/// Returns `InlineError::RenderError` if inline nesting depth exceeds 64.
pub fn render_inline_nodes_to_runs(
    nodes: &[InlineNode],
    hlink_resolver: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<OoxmlRun>, InlineError>
```

The implementation is a recursive-descent traversal that threads `RunProps`
accumulation exactly as `render_ooxml_accumulate` does today. The key addition
is the `hlink_resolver` closure that supplies an `rId` when a `Link` node is
encountered during the traversal — whether the `Link` is at the top level or
nested inside `Bold`, `Italic`, `Strikethrough`, or any other formatting wrapper.

When a `Link` is encountered:
1. Call `hlink_resolver(url)` to obtain `Option<String>` rId.
2. If `Some(rid)`: render the link's display text children recursively using
   the SAME `render_inline_nodes_to_runs` call (not `extract_plain_text_depth_limited`),
   accumulating the current `RunProps` into each resulting run, then SET
   `hyperlink_rid = Some(rid)` on each produced run.
3. If `None` (unsafe scheme, or nested-link-in-link-display-text via the F-040-P3-001
   invariant): render display text recursively (same `RunProps` accumulation),
   emit runs WITHOUT `hyperlink_rid`, log `tracing::warn!` once for the link node.

This design resolves F-P16-M1 completely: `Bold([Link{...}])` is now rendered by
entering Bold (setting `bold=true` in `RunProps`), then entering the Link (resolving
rId via closure), then rendering Link's display text children with `bold=true`
accumulated — producing `OoxmlRun { text: "...", bold: true, hyperlink_rid: Some(rid) }`.

### How Body and Notes Each Call the Engine

**Body path** (`slideforge-pptx/src/slide_serializer.rs`):

```rust
// Build hlink_map from the slide's registered hyperlinks.
// (hlink_map: HashMap<String /* url */, String /* rId */> built before this call)
let runs = render_inline_nodes_to_runs(nodes, &|url| {
    hlink_map.get(url).cloned()
})?;
// Convert Vec<OoxmlRun> to Vec<ooxmlsdk::schemas::a::Run> for typed body builder.
let ooxml_runs: Vec<Run> = runs.iter().map(ooxml_run_to_ooxmlsdk).collect();
```

**Notes path** (`slideforge-pptx/src/notes_slide.rs`):

```rust
// unique_hlinks: Vec<String> already collected, rId = "rId{idx+3}".
let runs = render_inline_nodes_to_runs(nodes, &|url| {
    unique_hlinks.iter().position(|u| u == url)
        .map(|idx| format!("rId{}", idx + 3))
})?;
// Serialize Vec<OoxmlRun> to raw-string XML (notes path uses string-based XML).
for run in &runs {
    out.push_str(&serialize_ooxml_run(run));
}
```

`serialize_ooxml_run` is a pure function in the new `ooxml_runs` module. It is the
SINGLE site that converts `OoxmlRun` properties to `<a:rPr>` attribute/child XML.
Child ordering is structurally guaranteed here by the function's structure: bold
attribute → italic → baseline → strike → highlight → code_font (`<a:latin>`) →
hyperlink (`<a:hlinkClick r:id="..."/>`). Both body and notes use the same child
ordering because both ultimately derive from `OoxmlRun`.

### Relationship to ADR-017

ADR-017 Option A added `render_with_context` to the `InlineFormat` trait to allow
the notes path to receive a pre-registered rId for top-level `Link` nodes. ADR-024
supersedes the notes-path usage of `render_with_context` by replacing the per-node
dispatch loop with a single call to `render_inline_nodes_to_runs`. The
`render_with_context` method and `InlineRenderContext` struct remain in the trait
and API surface — they are not removed — because:

1. They may be used by future `InlineFormat` implementors for custom behavior on
   individual nodes with context.
2. Removing them would be a breaking change to the trait surface.
3. The body path may still call `render_with_context` for other contexts (e.g.,
   future HTML-with-link-rel use cases).

However, within `slideforge-pptx`, neither the body path nor the notes path calls
`render_with_context` for OOXML inline-run generation after this ADR is implemented.
Both paths call `render_inline_nodes_to_runs` instead. The `// AC-005-DISPATCH-SITE`
comment in `notes_slide.rs` is removed (there is no longer a per-node dispatch loop).

The BC-5.02.002 dog-fooding guarantee is preserved and strengthened: all `<a:r>` run
markup construction in `slideforge-pptx` flows through `render_inline_nodes_to_runs`
→ `serialize_ooxml_run`, which both live in `slideforge-plugin-api`. The grep-zero
AC-005 test vector remains valid.

### rId Namespace Isolation

The `hlink_resolver` closure is constructed at the call site by the consumer
(`slide_serializer.rs` or `notes_slide.rs`) from its own namespace. The body path
resolver queries `slide{N}.xml.rels`-scoped rIds; the notes path resolver queries
`notesSlide{N}.xml.rels`-scoped rIds. `render_inline_nodes_to_runs` is namespace-
agnostic — it calls the provided closure and stores whatever string it returns. No
rId leak between slide and notes namespaces is possible by construction.

---

## Invariants Preserved

The following invariants are explicitly verified by the migration and must hold
after unification:

**INV-1 — One engine, two consumers.**
`render_inline_nodes_to_runs` is the sole function that maps `&[InlineNode]` to
OOXML run structures. No other function in `slideforge-pptx` or `slideforge-plugin-api`
produces `<a:r>` markup for the slide-body or notes-slide paths. The AC-005 grep
vector (`grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/`) MUST
return zero matches outside of `serialize_ooxml_run` (in `slideforge-plugin-api`)
and the two call sites in `slide_serializer.rs` and `notes_slide.rs` that convert
`Vec<OoxmlRun>` to output format.

**INV-2 — Schema-correct child ordering.**
`serialize_ooxml_run` is the single site for `<a:rPr>` construction. Attribute and
child-element ordering is structurally determined by the function's source order, not
by any caller. The ordering matches the DrawingML CT_TextCharacterProperties schema:
attributes (`b`, `i`, `baseline`, `strike`, `highlight`) precede child elements
(`<a:latin>` for code, `<a:hlinkClick>` for hyperlinks).

The body path converts `OoxmlRun` to typed ooxmlsdk `Run` objects via `ooxml_run_to_ooxmlsdk`.
That conversion function sets typed fields on `RunProperties`, delegating child
ordering enforcement to ooxmlsdk. This provides an additional schema-validation layer
for the body path without requiring the notes path to use ooxmlsdk.

**INV-3 — All 8 inline forms produce identical body/notes output.**
Bold, Italic, Code, Link (with rId), Highlight, Superscript, Subscript, Strikethrough,
and all legal nesting combinations produce bit-identical `OoxmlRun` sequences when
called from the body path and from the notes path with the same input tree and the
same rId mapping. This replaces the `test_obs_p15_001` cross-path equivalence test
(see Migration Plan step 7).

**INV-4 — Hyperlink correctness (no orphan rels, no dangling rIds).**
For every registered safe-scheme non-empty `Link` node encountered in
`render_inline_nodes_to_runs`:
- If `hlink_resolver` returns `Some(rid)` → exactly one run in the output has
  `hyperlink_rid = Some(rid)` per plain-text leaf in the display text.
- If `hlink_resolver` returns `None` → no run has `hyperlink_rid = Some(rid)`;
  display text is rendered as plain runs with inherited `RunProps`.

The corrected hyperlink invariant (replacing the erroneous `external_rel_count == hlinkclick_count`):

> **Every `<a:hlinkClick>` run references a registered External relationship, AND
> every registered External relationship is referenced by at least one `<a:hlinkClick>`
> run (no orphan rel, no dangling rId).**

The count-equality form (`external_rel_count == hlinkclick_count`) is FALSE for
multi-leaf link display text: a single External rel can back N `<a:hlinkClick>` runs
when the display text tree has multiple leaf nodes (e.g., `[click **here** now](url)`
produces three runs, each with `hyperlink_rid = Some(rid)`, backed by one External
rel). The correct invariant is the reference-set form stated above: all-rels-referenced
AND all-clicks-backed.

**INV-5 — F-040-P3-001 no-double-register invariant.**
`render_inline_nodes_to_runs` does NOT call `hlink_resolver` for `Link` nodes that
appear INSIDE another `Link`'s display text children. If the traversal enters a
`Link` node at the top level and then encounters a child `Link` while recursing into
its display text, the inner `Link` is treated as a plain-text source node (its URL
is ignored for rId purposes, its display text is rendered recursively). This prevents
double-registration and is consistent with the existing `collect_hyperlink_urls`
behavior in the notes path.

**INV-6 — Pass-15 inherit-on-None behavior preserved.**
For the body path: a `Link` node whose display text contains a child `Link` emits
runs with the OUTER link's `hyperlink_rid` (from `hlink_resolver(outer_url)`) on all
display-text leaf runs. The inner `Link`'s URL is not registered (INV-5) and its
`hlink_resolver` call is not made — instead, the inner `Link`'s display text is
flattened into runs that inherit the outer `Link`'s `hyperlink_rid`. This matches the
Pass-15 remediation behavior (`let inherited_rid = rid.or_else(...)`).

**INV-7 — Safe-scheme and empty-display-text guards preserved.**
`render_inline_nodes_to_runs` does not perform URL scheme validation — that is the
caller's responsibility. The resolver closure provided by each consumer must already
map only safe-scheme URLs to rIds (returning `None` for unsafe-scheme URLs). The
depth-limited guard (64 levels, INV-4 in ADR-017) applies to the recursive descent
inside `render_inline_nodes_to_runs` at the same depth limit.

**INV-8 — Math node behavior preserved.**
`InlineNode::Math` produces a `tracing::warn!` and a plain-text `OoxmlRun` with the
LaTeX source as text. This matches the existing EC-001 behavior on both paths.

**INV-9 — Hash/Eq/Clone determinism (ADR-013).**
`OoxmlRun` derives `Hash + Eq + Clone`. The order of runs in `Vec<OoxmlRun>` is
deterministic (document order from left-to-right depth-first traversal). Integer
EMU coordinates and the integer `baseline` field (`i32`) are unaffected.

**INV-10 — Notes rId namespace isolation.**
The notes path resolver closure captures `unique_hlinks` (the notes-slide-scoped URL
list) and computes `rId{idx+3}`. The body path resolver captures its slide-scoped
`hlink_map`. These are caller-owned closures; `render_inline_nodes_to_runs` never
touches rId allocation. No namespace leakage is possible.

---

## Migration Plan

The implementer follows these steps in order. Tests MUST remain green (or newly red
then green) at each step per TDD discipline.

**Step 1 — Add `OoxmlRun` struct and `serialize_ooxml_run` to `slideforge-plugin-api`.**

Create `crates/slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`. Define
`OoxmlRun` struct (as above) and `serialize_ooxml_run(run: &OoxmlRun) -> String`.
Re-export from `slideforge-plugin-api/src/inline_formats/mod.rs`. No existing code
is touched. Write unit tests in the new module:
- All properties off → `<a:r><a:t>{text}</a:t></a:r>`
- `bold=true` → `<a:rPr b="1"/>` before `<a:t>`
- `italic=true` → `<a:rPr i="1"/>`
- `bold=true, italic=true` → `<a:rPr b="1" i="1"/>`
- `baseline=Some(30000)` → `<a:rPr baseline="30000"/>`
- `strike=true` → `<a:rPr strike="sngStrike"/>`
- `highlight=true` → `<a:rPr highlight="yellow"><a:srgbClr val="FFFF00"/></a:rPr>`
  (child element form, NOT attribute — lessons from ADV-P11-HIGH-001)
- `code_font=true` → `<a:rPr><a:latin typeface="Courier New"/></a:rPr>`
- `hyperlink_rid=Some("rId3")` → `<a:rPr><a:hlinkClick r:id="rId3"/></a:rPr>`
- Combined: bold + hyperlink_rid → `<a:rPr b="1"><a:hlinkClick r:id="rId3"/></a:rPr>`
- Text XML-escaped
- Empty `text` → empty string returned (no run emitted)

Verify: `cargo nextest run -p slideforge-plugin-api --no-fail-fast` green.

**Step 2 — Add `render_inline_nodes_to_runs` to `slideforge-plugin-api`.**

Add `render_inline_nodes_to_runs` to `ooxml_runs.rs` (as specified in Decision
section). Write unit tests covering:
- `Plain("hello")` → one `OoxmlRun { text: "hello", ..default }`
- `Bold([Plain("hi")])` → `OoxmlRun { text: "hi", bold: true, .. }`
- `Bold([Italic([Plain("x")])])` → `OoxmlRun { text: "x", bold: true, italic: true, .. }`
- `Link` with resolver returning `Some("rId3")` → `OoxmlRun { text: "click", hyperlink_rid: Some("rId3"), .. }`
- `Link` with resolver returning `None` → `OoxmlRun { text: "click", hyperlink_rid: None, .. }`
- `Bold([Link{text:[Plain("here")]}])` with resolver returning `Some("rId3")` →
  `OoxmlRun { text: "here", bold: true, hyperlink_rid: Some("rId3"), .. }`
  (the F-P16-M1 regression test)
- `Link{url:U1, text:[Link{url:U2, text:[Plain("inner")]}]}` → inner link's url is
  NOT passed to resolver; inner runs inherit outer rId (INV-5 + INV-6)
- Math node → `OoxmlRun { text: latex_source, .. }` with `tracing::warn!`
- Nesting depth > 64 → `InlineError::RenderError`
- All 8 inline forms × nesting × combined property cases

Verify: all new tests green.

**Step 3 — Retarget the notes path to `render_inline_nodes_to_runs`.**

Modify `notes_slide.rs`:
- Remove `dispatch_inline_nodes_to_ooxml` function entirely.
- In `build_xml`, replace the per-entry call to `dispatch_inline_nodes_to_ooxml`
  with a call to `render_inline_nodes_to_runs` with a resolver closure over
  `unique_hlinks`.
- Serialize `Vec<OoxmlRun>` to string using `serialize_ooxml_run` for each run.
- Remove the `inline_format: &dyn InlineFormat` parameter from `build_xml` and
  `NotesSlideSerializer::build` (the unified engine replaces the trait dispatch).
  Update `build` callers accordingly.
- Remove the `// AC-005-DISPATCH-SITE` comment (it no longer applies to this path).
- Update the module-level doc comment to reflect the new architecture.

All existing notes tests MUST remain green. The following tests change semantics:
- Tests asserting `external_rel_count == hlinkclick_count` for multi-leaf link
  display text may need updating to the reference-set invariant form (see INV-4).
- `test_f085_p6_001_bold_wrapping_link_no_orphan_rel`: currently asserts notes drops
  the link inside Bold (no hlinkClick). After step 3, this MUST be updated — the
  unified engine NOW preserves the hyperlink on the bold run. The new assertion is:
  one External rel, one hlinkClick, run has `b="1"` and `hlinkClick`. This test was
  a guard for the OLD (defective) behavior; it must be inverted to guard the correct
  behavior (BC-3.02.002 PC8 compliance).

Add a new regression test: `test_f_p16_m1_bold_in_link_display_text_formatting_preserved_notes`.

Verify: `cargo nextest run -p slideforge-pptx --no-fail-fast` green.

**Step 4 — Retarget the body path to `render_inline_nodes_to_runs`.**

Modify `slide_serializer.rs`:
- Introduce `ooxml_run_to_ooxmlsdk(run: &OoxmlRun) -> ooxmlsdk::schemas::a::Run`
  as a private function. This converts `OoxmlRun` fields to typed ooxmlsdk
  `Run` / `RunProperties` builders for schema-correct child ordering.
- Replace any internal inline-run construction (the STORY-081 body rich-run path)
  with a call to `render_inline_nodes_to_runs` with a resolver closure over the
  slide's `hlink_map`.
- The body path still pre-registers hyperlink URLs into `slide{N}.xml.rels` before
  calling `render_inline_nodes_to_runs` (relationship registration is exporter-owned
  per ADR-017; the unified engine only produces runs, not rels).

All existing body-path tests MUST remain green.

Verify: `cargo nextest run -p slideforge-pptx --no-fail-fast` green.

**Step 5 — Delete dead code.**

- Remove `collect_hyperlink_urls` and `deduplicate_preserve_order` from
  `notes_slide.rs` if they are no longer called (URL collection for the notes
  path is still needed for rels building; verify whether `collect_hyperlink_urls`
  can be replaced by a simpler visitor over `render_inline_nodes_to_runs` output,
  or retained as a pre-pass). If retained, verify it remains in sync with the
  unified engine's resolver logic.
- Remove the `inline_format` parameter from all call sites and tests that passed
  `&DefaultInlineFormat` to `NotesSlideSerializer::build`.
- The `render_with_context` method on `DefaultInlineFormat` is RETAINED (per the
  Decision rationale above). Do not remove it.

Verify: `cargo nextest run -p slideforge-pptx -p slideforge-plugin-api --no-fail-fast` green.

**Step 6 — Add the F-P16-M1 regression test to the notes test suite.**

Write `test_f_p16_m1_bold_in_link_display_text_formatting_preserved_notes` in
`notes_tests.rs`. Assert:
- Input: `[click **here** now](https://example.com)` represented as
  `Link{ text: [Plain("click "), Bold([Plain("here")]), Plain(" now")], url: "https://example.com" }`
- Notes XML contains three `<a:hlinkClick>` (one per leaf run, all backed by rId3).
- The `<a:r>` run for "here" contains BOTH `b="1"` and references `rId3`.
- One External rel is registered.

FAILS before step 3; PASSES after step 3.

**Step 7 — Replace or retire `test_obs_p15_001` cross-path equivalence test.**

The Pass-15 cross-path equivalence test (`test_obs_p15_001_body_notes_cross_path_rel_click_parity`)
was added to make future dual-generator divergence visible. After unification, the
body and notes paths share a single engine — structural divergence via a second
implementation path is impossible by construction. The test is therefore retired
(deleted or replaced with a comment explaining why it is no longer needed).

Replace it with a structural invariant test that verifies `render_inline_nodes_to_runs`
produces identical output for body and notes call sites given identical inputs and
resolver behavior. This is simpler and more direct than a cross-path black-box comparison.

**Step 8 — Run the full workspace gate.**

`just check` (fmt + clippy + nextest + doctests). Verify CLEAN.
Confirm: `grep -r "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/` returns
zero matches outside of the two conversion functions (`ooxml_run_to_ooxmlsdk` in
`slide_serializer.rs` and `serialize_ooxml_run` calls in `notes_slide.rs`).

---

## Consequences

### Positive

1. **FU-PPTX-DUAL-RUN-GENERATOR is resolved by construction.** Body and notes cannot
   diverge on inline-form rendering because they share one engine. Any future inline-form
   addition (e.g., Underline, Color, Size) requires one change in `render_inline_nodes_to_runs`
   and propagates to both paths automatically.

2. **F-P16-M1 is fixed.** Bold, italic, and other formatting inside a hyperlink's display
   text is preserved in notes output. `[click **here**](url)` in notes now renders with a
   bold `<a:rPr b="1"><a:hlinkClick r:id="rId3"/></a:rPr>` on the "here" run.

3. **BC-3.02.002 PC8 compliance restored.** "All output formats" now includes notes PPTX
   for inline markup within hyperlink display text.

4. **Schema-correct child ordering guaranteed for both paths.** `serialize_ooxml_run` is
   the single site for `<a:rPr>` construction on the notes path. `ooxml_run_to_ooxmlsdk`
   delegates to ooxmlsdk on the body path. No manual child ordering maintenance required.

5. **Simplified notes path.** `dispatch_inline_nodes_to_ooxml` (a complex per-node loop
   with context threading) is replaced by a single function call. The notes path no longer
   receives an `&dyn InlineFormat` parameter.

6. **AC-005 grep-zero invariant strengthened.** There is now a single call to
   `serialize_ooxml_run` in the notes serialization path — no per-node `render_with_context`
   loop. The grep-zero test is easier to audit.

### Negative / Trade-offs

1. **`render_with_context` is no longer the dispatch mechanism for PPTX notes.**
   The method remains in the trait for forward compatibility but is not used in the PPTX
   notes path after this ADR. A future `InlineFormat` implementor that overrides
   `render_with_context` will not have that override exercised by the notes path.
   This is acceptable: the notes PPTX path is not a format-plugin extension point — it
   is a PPTX package serialization concern owned by `slideforge-pptx`.

2. **`OoxmlRun` is a new public type in `slideforge-plugin-api`.**
   It is part of the public API surface of the plugin crate. Future changes to `OoxmlRun`
   fields affect all consumers. The struct is kept minimal (8 fields) and well-documented.
   It does not depend on any external crate.

3. **URL collection for notes rels pre-pass may need adjustment.**
   `collect_hyperlink_urls` performs a pre-pass over the node tree to collect URLs for
   rels registration before the render pass. The unified engine's recursive Link traversal
   mirrors this but for render output. The implementer must verify that the URL collection
   pre-pass remains consistent with what `render_inline_nodes_to_runs` will visit (same
   top-level-only registration rule, INV-5). If they diverge, orphan rels re-appear.
   The recommended approach: derive URL collection from the resolver closure's calls —
   log which URLs the resolver is queried for and pre-register those. Alternatively, retain
   `collect_hyperlink_urls` with the F-085-P6-001 fix (top-level only) and verify it is
   still in sync with the engine's Link traversal behavior. The implementer MUST add a
   regression test verifying zero orphan rels after the migration.

### Design Risks for the Implementer

**Risk 1 — Nested-link-in-display-text resolver interaction (INV-5 + INV-6).**
`render_inline_nodes_to_runs` must NOT call `hlink_resolver` for a `Link` encountered
inside another `Link`'s display-text children (the F-040-P3-001 rule). The recursive
descent must track whether it is inside a `Link`'s display-text traversal and suppress
resolver calls in that context. Implement via a `bool inside_link_display_text`
parameter in the internal recursive helper. When `true`, `Link` nodes are rendered
as plain-text runs (display text recursed with same `RunProps`, no resolver call, no
`hyperlink_rid` assigned).

**Risk 2 — Multi-leaf link display text and rel count.**
A single `Link` with rich display text (`[click **here** now](url)`) will produce
three `OoxmlRun` entries, each with `hyperlink_rid = Some("rId3")`. The rels pre-pass
registers `url` → `rId3` ONCE. The serialized notes XML will have three `<a:hlinkClick>`
runs all referencing `rId3`. This is schema-valid per OOXML — multiple runs can reference
the same relationship. However, the count-equality test (`external_rel_count == hlinkclick_count`)
will now be FALSE (1 != 3). Any test asserting count equality MUST be updated to the
reference-set invariant (INV-4). The implementer MUST audit all existing count-equality
assertions in `notes_tests.rs` before declaring migration complete.

**Risk 3 — `highlight` child-element ordering.**
`<a:highlight>` is a child element of `<a:rPr>`, not an attribute. It must appear AFTER
any attribute-form properties (`b`, `i`, `baseline`, `strike`) and is a SIBLING of
`<a:latin>` and `<a:hlinkClick>`. In `serialize_ooxml_run`, the ordering is:
attributes first, then child elements in this order: `<a:latin>` (code), then
`<a:highlight>` (highlight), then `<a:hlinkClick>` (hyperlink). Do NOT emit
`highlight="yellow"` as an attribute — that was the ADV-P11-HIGH-001 defect.
The `OoxmlRun.highlight` field MUST serialize via the child element form only.

**Risk 4 — `inline_format` parameter removal from `NotesSlideSerializer::build`.**
`build` currently takes `inline_format: &dyn InlineFormat`. After step 3, this
parameter is removed. All call sites in `lib.rs` and tests must be updated. The
implementer MUST search for all call sites before removing the parameter.

---

## Alternatives Considered

### Alternative A — Retain two generators, add parity test

Keep `dispatch_inline_nodes_to_ooxml` and the body rich-run path separate. Add a
comprehensive cross-path equivalence test that asserts identical OOXML output for
all 8 inline forms × nesting combinations on both paths.

**Rejected.** The three adversary findings (ADV-P11-HIGH-001, F-P15-HIGH-001,
F-P16-M1) demonstrate that parity tests add detection cost but not prevention.
ADV-P11-HIGH-001 survived several passes without detection precisely because the
test suite exercised each path in isolation. Structural unification eliminates the
class of bugs — detection tests cannot. The `test_obs_p15_001` parity test added
after Pass-15 would have caught F-P16-M1 only if it had tested the formatting-inside-
link axis, which it did not. Writing exhaustive parity tests for all N×M combinations
of node types × nesting depths is maintenance overhead that grows with every new
inline form. One engine does not require this overhead.

### Alternative B — Move body path to raw-string; both paths use `render_ooxml_accumulate`

Have the body path call `DefaultInlineFormat::render()` / `render_with_context()` per
node (same as the current notes path) and concatenate raw strings, abandoning typed
ooxmlsdk builders for inline run markup.

**Rejected.** The ooxmlsdk typed path on the body path provides a schema-validation
layer (structured field setting instead of manual attribute concatenation). Abandoning
it reduces correctness guarantees. The correct direction is to bring the notes path up
to the typed-intermediate level, not to bring the body path down to the raw-string level.

### Alternative C — Extend `render_with_context` to accept `RunProps` + recursion closure

Keep `render_with_context` as the dispatch mechanism but extend `InlineRenderContext`
with a `run_props: RunProps` field and a recursive callback. `DefaultInlineFormat::render_with_context`
would then recurse through display-text children using the inherited props.

**Rejected.** This embeds OOXML-specific run-property accumulation semantics into
the `InlineFormat` trait API, which is output-format-agnostic. `RunProps` is an OOXML
concept; HTML and Markdown formatters have no use for it. Polluting the trait with
OOXML-specific state violates the plugin-first principle. `render_inline_nodes_to_runs`
belongs in `slideforge-plugin-api` alongside `DefaultInlineFormat` but outside the
`InlineFormat` trait — it is a utility for OOXML producers, not a trait contract.

---

## References

- BC-3.02.002 v1.5 — PC8: "InlineNode variants … rendering guarantee through the
  InlineFormat plugin surface … all output formats."
- BC-5.02.002 v1.4 — postcondition 5, EC-004 grep-zero vector, dog-fooding guarantee.
- ADR-006 — Plugin-first architecture; dog-fooding guarantee.
- ADR-013 — Integer EMU coordinates; `Hash + Eq + Clone` requirement.
- ADR-017 — `render_with_context` + `InlineRenderContext` (Option A, accepted).
  ADR-024 does not supersede ADR-017 — both coexist. ADR-017 governs the trait
  extension; ADR-024 governs the PPTX serializer-level call strategy.
- OBS-P11-001 (Pass 11) — first identification of FU-PPTX-DUAL-RUN-GENERATOR.
- OBS-P15-001 (Pass 15) — third recurrence; recommended architectural unification.
- ECMA-376 §21.1.2.3 (DrawingML CT_TextCharacterProperties) — `<a:rPr>` schema.
- OOXML spec §12.3.6 — hyperlink relationship type with `TargetMode="External"`.

---

## Decision Log

| Date | Author | Note |
|------|--------|------|
| 2026-06-09 | architect | ADR-024 produced in response to F-P16-M1 (Pass 16) and OBS-P15-001 (Pass 15) / FU-PPTX-DUAL-RUN-GENERATOR. Human authorized scope expansion within STORY-081. |
| 2026-06-09 | human | Approved. Implementation proceeds under TDD via the Migration Plan above. |
