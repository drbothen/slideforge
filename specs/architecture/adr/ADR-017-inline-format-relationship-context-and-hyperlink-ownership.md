---
document_type: adr
adr_id: ADR-017
title: InlineFormat relationship context and hyperlink-run ownership boundary
status: accepted
date: 2026-06-05
accepted_date: 2026-06-05
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
context: >
  STORY-085 adversarial review surfaced two findings (F-001 [CRIT] and F-006 [MED])
  regarding the Link OOXML path in slideforge-pptx/src/notes_slide.rs. This ADR
  records the architectural analysis and selected resolution path. Option A was
  selected and authorized by the human on 2026-06-05.
human_gate_required: false
human_gate_resolved: >
  Human authorized Option A on 2026-06-05. HG-1 GRANTED: additive-defaulted trait
  methods are confirmed within the spirit of BC-5.02.001 invariant 2. HG-2 resolved:
  the human confirmed that "additive defaulted methods" are permitted; product-owner
  will clarify BC-5.02.001 invariant 2 prose to make this explicit. Options B and C
  are rejected. Implementation may proceed.
---

# ADR-017: InlineFormat Relationship Context and Hyperlink-Run Ownership Boundary

## Status

**ACCEPTED.** Human authorized Option A on 2026-06-05. HG-1 GRANTED: additive-defaulted
trait methods are confirmed within the spirit of BC-5.02.001 invariant 2. HG-2 resolved:
BC-5.02.001 invariant 2 prose will be clarified by the product-owner to explicitly
permit additive-defaulted methods. Options B and C are rejected with rationale recorded
below. Implementation proceeds via STORY-085 in-scope fix burst.

## Context

STORY-085 refactors `slideforge-pptx` notes inline serialization to dog-food the
`InlineFormat` trait (BC-5.02.002 — "no bundled plugin bypasses the registered
trait interface"). The implementation routes most inline nodes through
`DefaultInlineFormat::render(node, InlineOutputFormat::Ooxml)`, but the **Link**
node OOXML path is hand-constructed in the exporter because it needs a hyperlink
relationship id (rId) that is exporter/package-owned state.

Adversarial review of the STORY-085 deliverable surfaced two findings:

**F-001 [CRIT]:** BC-5.02.002 postcondition 5 and EC-004 forbid ANY `<a:r>` run
construction in `slideforge-pptx` production code outside the single trait-dispatch
call site. The Link arm is a second construction site — a dog-fooding bypass. The
canonical AC-005 grep vector (`grep -r "a:r\|a:rPr\|serialize_inline"`) must return
zero matches outside the dispatch wrapper.

**F-006 [MED]:** The exporter uses `let formatter = DefaultInlineFormat;` (direct
construction) rather than resolving the registered `"default"` InlineFormat via the
`PluginRegistry`. BC-5.02.002 postcondition 5 says rendering goes "through the
registry" — a third-party override of the `"default"` inline format would not take
effect in notes rendering.

The root tension: `InlineFormat::render(&self, node: &InlineNode, format:
InlineOutputFormat) -> Result<String, InlineError>` has no mechanism to request
or receive an rId from the exporter's package-relationship manager. A hyperlink
`<a:hlinkClick r:id="rIdN"/>` in OOXML is a two-part artifact: the `<a:rPr>` run
property XML (formatting-layer) AND a `.rels` relationship entry (package-layer).
The trait currently only covers the formatting-layer.

## Analysis

### Question 1: Is the Link `<a:r>` construction a genuine BC-5.02.002 violation?

**Yes, it is a genuine violation** — with an important architectural distinction that
clarifies the correct resolution path.

BC-5.02.002 postcondition 5 states: "No production code path in `slideforge-pptx`
performs `<a:r>` run construction outside of the `InlineFormat` trait dispatch."
EC-004 makes explicit that the grep-zero test vector is the architectural invariant.
The Link arm constructs `<a:r>...<a:hlinkClick>...</a:r>` directly in `notes_slide.rs`
— this is literally a second `<a:r>` construction site outside the trait dispatch.

However, the structural reality is that an OOXML hyperlink run is a composite of two
distinct concerns:

1. **Formatting-layer:** `<a:r><a:rPr><a:hlinkClick r:id="rIdN"/></a:rPr><a:t>display
   text</a:t></a:r>` — this is the output the InlineFormat trait is meant to produce.

2. **Package-layer:** The `.rels` entry (`TargetMode="External"`, relationship type,
   URL) — this is exporter/package state that the `InlineFormat` trait has no
   mechanism to touch.

The `.rels` management IS legitimately exporter-owned. It is not formatting: it is
package relationship bookkeeping that requires the ZIP part assembly context. No
reasonable InlineFormat plugin design would own `.rels` file construction — that is
an OOXML package concern, not an inline formatting concern.

The contract violation is real, but it is precisely located: the trait currently
cannot express "produce a hyperlink run given a pre-registered rId". That gap is
fixable without making the trait own `.rels` plumbing. The `.rels` registration
stays in the exporter. The `<a:r>` construction moves into the trait. The bridge is
a context parameter: the exporter registers the relationship first, then passes the
resulting rId into the formatting call.

**Contract-faithful reading:** The violation is genuine. The correct fix is architectural
(close the API gap), not a carve-out.

### Question 2: What is the minimal production-grade fix?

Three options evaluated:

---

#### Option A: Extend InlineFormat trait with a defaulted `render_with_context` method

Add a second method to the `InlineFormat` trait with a default implementation:

```rust
/// Context provided by an exporter for inline nodes that require
/// package-level resources (e.g., hyperlink relationship IDs).
///
/// Exporters that manage OOXML relationships populate `hyperlink_rid`
/// before calling `render_with_context`. Exporters that do not manage
/// relationships (HTML, Markdown) leave the context at default.
pub struct InlineRenderContext<'a> {
    /// The relationship ID (`rId3`, `rId4`, …) pre-registered by the
    /// exporter for the current `Link` node's URL. `None` if the
    /// exporter does not support relationship-bearing hyperlinks.
    pub hyperlink_rid: Option<&'a str>,
}

impl Default for InlineRenderContext<'_> {
    fn default() -> Self {
        Self { hyperlink_rid: None }
    }
}

pub trait InlineFormat: Send + Sync {
    fn id(&self) -> &str;
    fn render(&self, node: &InlineNode, format: InlineOutputFormat)
        -> Result<String, InlineError>;

    /// Render `node` with additional exporter-provided context.
    ///
    /// The default implementation ignores the context and delegates to
    /// `render()`. Override this method to handle context-bearing cases
    /// (e.g., hyperlink rIds in OOXML).
    ///
    /// This is a defaulted method — existing implementations of
    /// `InlineFormat` do not need to change. They will use the default,
    /// which falls back to `render()` and emits display text (with a
    /// `tracing::warn!` if an rId was expected but not used).
    fn render_with_context(
        &self,
        node: &InlineNode,
        format: InlineOutputFormat,
        context: &InlineRenderContext<'_>,
    ) -> Result<String, InlineError> {
        let _ = context; // default ignores context
        self.render(node, format)
    }
}
```

The exporter flow becomes:
1. Exporter's relationship manager registers the URL → receives `rId`.
2. Exporter constructs `InlineRenderContext { hyperlink_rid: Some(&rid) }`.
3. Exporter calls `formatter.render_with_context(node, Ooxml, &ctx)`.
4. `DefaultInlineFormat::render_with_context` sees the rId and emits the
   full `<a:r><a:rPr><a:hlinkClick r:id="rIdN"/></a:rPr><a:t>text</a:t></a:r>`.
5. No `<a:r>` construction exists outside `DefaultInlineFormat`.

**Impact on BC-5.02.001 invariant 2** ("trait definitions in slideforge-plugin-api
are the contracts — no alternate unstable API exists"):

Adding a *defaulted* method to a trait is a non-breaking additive change. Existing
implementors continue to compile without changes — they inherit the default. The
trait signature is stable in the sense that no existing `impl InlineFormat for Foo`
block needs a new required method. This does NOT violate invariant 2 in spirit —
the concern of invariant 2 is that no secret/internal bypass API exists alongside
the trait, not that the trait can never gain new defaulted methods.

However, invariant 2 is phrased broadly ("trait definitions are the contracts").
A strict reading could interpret "trait definitions" as frozen. This ambiguity
requires the human to confirm that adding a defaulted method is within the spirit
of the BC. This is the human gate for Option A.

**Impact on existing InlineFormat implementors:** Zero mandatory changes. All
existing implementations compile unchanged and use the default behavior.

**Impact on registry:** None. The method dispatches through the same `dyn InlineFormat`
trait object.

**F-006 fix interaction:** Option A and the F-006 fix are independent and both
necessary. F-006 is fixed by threading the `PluginRegistry`-resolved `InlineFormat`
into the notes serializer rather than direct-constructing `DefaultInlineFormat`.
Option A is the mechanism that makes the resolved formatter capable of handling rIds.
The two fixes combine cleanly: resolve via registry, then call `render_with_context`.

---

#### Option B: BC carve-out — relationship-bearing runs are exporter-owned

Document that `<a:hlinkClick>` run construction is legitimately exporter-owned
because it requires package-layer context, and amend BC-5.02.002 to add an explicit
exception: "The grep-zero test vector is scoped to exclude runs bearing
`<a:hlinkClick>` — hyperlink relationship construction is exporter-owned by
architectural necessity."

**Assessment:** This is a BC amendment — human-only by CLAUDE.md precedence rules.
More importantly, it is architecturally weaker than Option A. It normalizes a bypass
instead of closing the API gap. A third-party `InlineFormat` implementor wishing to
customize hyperlink rendering (e.g., adding URL shortening, UTM tracking, or
accessibility attributes to hyperlink runs) would still have no mechanism to do so
— they could not override the exporter's hand-coded hyperlink construction. Option A
preserves the override surface. Option B abandons it.

Option B is the correct fallback ONLY if Option A proves infeasible after human
review (e.g., the trait signature change is rejected). It should not be the primary
recommendation.

---

#### Option C: Two-phase render — trait returns a `RenderFragment` enum

Instead of adding a context parameter, the trait's return type becomes a `RenderFragment`
that can signal "needs rId" before emitting the final string:

```rust
pub enum RenderFragment {
    Complete(String),
    NeedsHyperlinkRid { url: String, display_text: String },
}
```

The exporter inspects the fragment, registers the relationship if needed, then
resolves the fragment to a final string.

**Assessment:** This is more complex than Option A, requires a breaking return-type
change to `render()` (affecting ALL existing implementations), and provides no
practical benefit over Option A's context-parameter design. Option A's defaulted
method is strictly simpler: the context is opt-in, the default is correct for
non-OOXML formats, and no existing impls break. Option C is rejected.

---

### Question 3: F-006 registry-routing fix

F-006 is independently correct and in-scope-feasible for STORY-085. The fix is:

1. The `PptxExporter` (or `NotesSlideSerializer`) must accept the registered
   `InlineFormat` from the `PluginRegistry` rather than constructing
   `DefaultInlineFormat` directly.
2. The exporter already receives a `PluginRegistry` reference (or can be threaded
   one) — this is the `default_registry()` pattern from `plugin-architecture.md`.
3. The call becomes: `registry.lookup_inline_format("default")?.render_with_context(...)`.

This fix does NOT require a BC amendment and does NOT require human authorization.
It is a production-grade correctness fix that the implementer can apply in-scope.

The F-006 fix must be paired with the Option A trait extension — without the context
parameter, routing through the registry still cannot deliver rIds to the formatter.
The two are jointly necessary.

## Decision

**SELECTED: Option A.** Authorized by human on 2026-06-05.

Rationale:
1. Option A closes the actual API gap rather than normalizing the bypass.
2. The defaulted-method extension is non-breaking to existing implementations.
3. It preserves the third-party override surface (alignment with the plugin-first principle).
4. The `.rels` relationship management stays correctly exporter-owned — Option A only
   bridges the rId from the package layer into the formatting layer, not the other
   way around.
5. Option B requires a BC amendment (human-only) and is architecturally weaker — it
   normalizes a bypass and permanently removes the third-party hyperlink override
   surface. **Rejected.**
6. Option C requires a breaking return-type change to `render()` affecting all existing
   implementations, with no compensating benefit over Option A's simpler context-parameter
   design. **Rejected.**

**HG-1 — GRANTED (2026-06-05):** Adding `render_with_context` defaulted method +
`InlineRenderContext` to the `InlineFormat` trait is authorized. Additive-defaulted
methods are within the spirit of BC-5.02.001 invariant 2.

**HG-2 — RESOLVED (2026-06-05):** "Additive defaulted methods" are permitted. The
product-owner will clarify BC-5.02.001 invariant 2 prose to state explicitly: "Additive
defaulted methods may be added to trait definitions in slideforge-plugin-api without
violating this invariant, provided no existing implementors are required to change."

**Items now implementer-executable in-scope (no further authorization required):**

| Item | Nature |
|------|--------|
| `InlineRenderContext` struct + `render_with_context` defaulted method in `slideforge-plugin-api/src/traits/inline_format.rs` | Authorized by HG-1 above |
| `DefaultInlineFormat::render_with_context` — Link + Ooxml arm using supplied rId | Bundled plugin impl; within BC-5.02.001/BC-5.02.002 scope |
| F-006: Thread the registry-resolved `InlineFormat` into notes serializer instead of direct `DefaultInlineFormat` construction | Production-grade correctness fix; no BC amendment needed |
| Updating the STORY-085 AC-005 grep test to account for the new dispatch pattern | Test alignment to the agreed architecture |

## Consequences

### Option A — Selected and Implemented

1. `slideforge-plugin-api/src/traits/inline_format.rs` gains `InlineRenderContext` struct
   and `render_with_context` defaulted method.
2. `DefaultInlineFormat::render_with_context` handles the `Link + Ooxml` case using the
   supplied `hyperlink_rid`. The existing `render()` continues to emit display-text
   fallback with `tracing::warn!` when called without a context (HTML, Markdown, or
   OOXML without pre-registered rId).
3. `slideforge-pptx/src/notes_slide.rs` removes all `<a:r>` construction; the Link arm
   calls `formatter.render_with_context(node, Ooxml, &ctx)` where `ctx` carries the
   pre-registered rId. The relationship registration (`hlink_urls` + `build_rels()`) stays
   in the exporter — it is not touched by the trait.
4. F-006 is resolved simultaneously: `formatter` is the registry-resolved instance, not
   a directly-constructed `DefaultInlineFormat`.
5. The AC-005 grep vector confirms zero `<a:r>` / `<a:rPr>` construction in
   `slideforge-pptx/src/` (all construction is in `slideforge-plugin-api/src/inline_formats/`).
6. No existing `InlineFormat` implementors (current or future third-party) are broken.
7. BC-5.02.001 invariant 2 prose will be clarified by the product-owner to explicitly
   permit additive-defaulted methods (follow-up action; implementation does not wait on it).

### Option B — Rejected

Option B (BC carve-out for `<a:hlinkClick>` runs) is rejected because it normalizes a
bypass rather than closing the API gap, and permanently removes third-party override
capability for hyperlink run rendering in PPTX. Not reconsidered without new information.

### Option C — Rejected

Option C (two-phase `RenderFragment` return type) is rejected because it requires a
breaking return-type change to `render()` affecting all existing implementations, with
no benefit over Option A's simpler additive-defaulted design. Not reconsidered without
new information.

## References

- BC-5.02.001 v1.3 — invariant 2: "The trait definitions in slideforge-plugin-api are
  the contracts — no alternate unstable API exists."
- BC-5.02.002 v1.3 — postcondition 5, EC-004, canonical grep test vector.
- ADR-006 — Plugin-first architecture.
- ADR-016 — Registry builder + surface ownership (confirmed InlineFormat impls in
  slideforge-plugin-api; STORY-085 owns the refactor).
- STORY-085 "Previous Story Intelligence" section — named the two-path choice explicitly:
  "extend trait signature vs URL-as-text fallback."
- OOXML spec §12.3.6 — hyperlink relationship type `hyperlink` with `TargetMode="External"`.
