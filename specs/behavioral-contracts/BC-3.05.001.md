---
document_type: behavioral-contract
level: L3
version: "1.3.1"
status: active
producer: product-owner
timestamp: 2026-05-28T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-024
lifecycle_status: active
introduced: v1.0.0
modified: ["v1.2 — adversary pass 1 adjudication: variant count corrected to 12, payload shapes corrected to Vec<InlineNode> for structured variants, inline depth bound added (max 64), math xref validation boundary codified", "v1.3 — story spec AC-005 enum example corrected to match production types", "v1.3.1 — VP propagation burst: assigned VP-043 through VP-047 to all VP-TBD entries"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.05.001: All 12 Inline Format Types Render to Correct Output per Format

## Description

All 12 inline formatting types (Plain, Bold, Italic, Code, Link, Math, Footnote,
Xref, Superscript, Subscript, Strikethrough, Highlight) are supported in slide text
fields and section body content. Each inline type maps to a distinct Rust `InlineNode`
enum variant and is rendered to the correct OOXML, HTML, or PDF representation per
output format. String-prefix-based formatting (e.g., `"**header**"`) is rejected —
only structural `InlineNode::*` variants are used. Nesting is bounded to a maximum
depth of 64 to prevent DoS.

## Preconditions

1. Text content contains inline formatting recognized by the DSL parser.
2. The declared inline type is one of the 12 registered types.
3. Output format is PPTX, DOCX, HTML, or PDF.
4. Inline nesting depth does not exceed 64.

## InlineNode Variant Schema (Authoritative)

The production enum in `slideforge-types/src/inline.rs` has exactly **12 variants**.
The count in earlier BC versions ("11") was an off-by-one error — the original list
named 12 items (Plain, Bold, Italic, Code, Link, Math, Footnote, Xref, Superscript,
Subscript, Strikethrough, Highlight). **12 is correct.** This is the authoritative
count per the production code and the Q8 decision (q4-q15-decisions.md line 75-93).

Payload shapes are production-accurate:

```rust
pub enum InlineNode {
    // Leaf nodes (terminal — no nested InlineNodes)
    Plain(Arc<str>),           // plain text
    Code(Arc<str>),            // monospace, no further inline processing
    Xref(Arc<str>),            // cross-ref target identifier
    Math(MathNode),            // LaTeX source; rendered by slideforge-math

    // Container nodes (nested content — supports inline composition)
    Bold(Vec<InlineNode>),
    Italic(Vec<InlineNode>),
    Footnote(Vec<InlineNode>),
    Superscript(Vec<InlineNode>),
    Subscript(Vec<InlineNode>),
    Strikethrough(Vec<InlineNode>),
    Highlight(Vec<InlineNode>),

    // Structured node (named fields)
    Link {
        text: Vec<InlineNode>,   // display content, can be styled
        url: Arc<str>,           // target URL
    },
}
```

All variants derive `Debug + Clone + PartialEq + Eq + Hash` (DI-010, CLAUDE.md).

Note: the story spec lines 126-139 (AC-005) showed `Arc<str>` payloads on Bold,
Italic, etc. — those were incorrect. The production enum uses `Vec<InlineNode>` for
container variants to enable nesting (e.g., bold inside italic). **BC is the
source of truth for contract semantics (CLAUDE.md precedence rule 1).**

## Postconditions

Per output format:

**PPTX/DOCX:**
- Plain → `<a:t>text</a:t>` in a plain run
- Bold → `<a:rPr b="1">`
- Italic → `<a:rPr i="1">`
- Code → `<a:rPr>` with monospace font run
- Link → `<a:hlinkClick r:id="...">` relationship in PPTX; `<w:hyperlink>` in DOCX
- Math → OMML `<m:oMath>` block (per BC-1.10.003)
- Footnote → endnote or footnote in DOCX; presenter note annotation in PPTX
- Xref → internal hyperlink to slide index or heading
- Superscript → `<a:rPr baseline="30000">` (30% above baseline)
- Subscript → `<a:rPr baseline="-25000">` (25% below baseline)
- Strikethrough → `<a:rPr strike="sngStrike">`
- Highlight → `<a:rPr highlight="yellow">` (or brand accent color)

**HTML:**
- Plain → bare text node, Bold → `<strong>`, Italic → `<em>`, Code → `<code>`,
  Link → `<a href>`, Math → MathML `<math>`, Footnote → `<span role="note">`,
  Xref → `<a href="#">`, Superscript → `<sup>`, Subscript → `<sub>`,
  Strikethrough → `<del>`, Highlight → `<mark>`

**PDF:**
- Bold/Italic/Strike/Super/Sub → equivalent tagged text spans with PDF structure tags
- Code → monospace font span in tagged PDF
- Plain → tagged text span (PDF/UA structure)
- Link → PDF annotation with URI action
- Math → vector path rendering per BC-1.10.003
- Highlight → PDF highlight annotation

## Invariants

1. All 12 inline types ship in v1.0 — no deferral. (CAP-024 priority: P1)
2. String-prefix-based bold (`"**text**"`) is an unrecognized pattern. The parser
   does not interpret markdown-style inline markup. This is a forbidden pattern
   per CLAUDE.md. (DI-004 implies values are not implicitly transformed)
3. Inline formatting within a math block (`$...$`) uses `@{var}` interpolation only;
   `{{ }}` text interpolation is disabled inside math mode. (BC-1.02.004)
4. Nesting depth is bounded at 64 levels. Inline trees deeper than 64 produce
   `LayoutError::InlineDepthExceeded { slide_index, depth: 65 }`. This is a
   hard error (not a warning) to prevent stack overflow on export.
5. `InlineNode::Xref` validation traverses ONLY top-level inline sequences; it does
   NOT recursively validate xrefs inside `MathNode` content. Math is a separate
   validation surface. (v1.0 scope boundary — see Math Xref Boundary section below.)
6. The layout stage preserves `InlineNode` sequences verbatim in `FrameContent::TextRun`.
   Exporters translate per-format; the layout stage does NOT produce format-specific
   markup.

## Math Xref Validation Boundary

`InlineNode::Xref` references in top-level text sequences are validated against
slide titles during the layout pass (AC-007). However, xref expressions embedded
within `MathNode` contents are NOT validated in v1.0.

Rationale: math rendering (BC-1.10.003) is a separate validation surface with its
own LaTeX error semantics. Traversing into `MathNode` for xref validation would
require the layout engine to parse LaTeX ASTs — out of scope for v1.0. Future
maintainers may add this in a dedicated story anchored to BC-1.10.003.

This boundary is explicit and intentional, not an oversight.

## Inline Depth Bound

Maximum nesting depth: **64 levels**.

At depth 65+, `layout_shapes` (or the inline validation pass) returns
`LayoutError::InlineDepthExceeded { slide_index, depth: 65 }` (or the actual
exceeded depth). This is a hard error — output is NOT produced for the affected slide.

Rationale: unbounded recursion in inline tree traversal during export (PPTX XML
generation, PDF span building, HTML generation) can exhaust the stack on real-world
inputs. 64 levels is a conservative bound that no legitimate document approaches.

Canonical test vector: a tree of 65 nested `Bold(vec![Bold(vec![...])])` nodes
must produce `LayoutError::InlineDepthExceeded`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Nested bold inside italic (`Italic(vec![Bold(vec![Plain("text")])])`) | Both applied: PPTX `<a:rPr b="1" i="1">`; HTML `<em><strong>text</strong></em>` |
| EC-002 | Xref to a slide title that doesn't exist | `LayoutWarning::XrefTargetNotFound { target, slide_index }` accumulated; not fatal |
| EC-003 | Highlight in PPTX with no highlight color in brand | Default to yellow highlight (#FFFF00); lint warning: "highlight color not declared in brand; using yellow" |
| EC-004 | Footnote in PPTX (no footnote feature in slides natively) | Footnote content moved to presenter notes with superscript reference number on slide |
| EC-005 | Code inline in .pptx (no semantic code type in OOXML) | Monospace font run; no semantic tagging (PPTX limitation documented in DSL reference) |
| EC-006 | Inline nesting at depth 65 | `LayoutError::InlineDepthExceeded { slide_index, depth: 65 }`; hard error |
| EC-007 | `Xref("slide-title")` inside a `MathNode` | NOT validated by xref pass; math is a separate validation surface |
| EC-008 | `"**bold using markdown"` (forbidden pattern) | No bold applied; literal `**bold using markdown` rendered as Plain text; lint warning |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Bold(vec![Plain("Key finding")])` | PPTX: `<a:r><a:rPr b="1"/><a:t>Key finding</a:t></a:r>` | happy-path |
| `Link { text: vec![Plain("See report")], url: "https://example.com" }` | PPTX: hlinkClick relationship; HTML: `<a href="https://example.com">See report</a>` | happy-path |
| `"**bold using markdown"` (forbidden pattern) | No bold; literal text rendered; lint warning issued | edge-case |
| `Superscript(vec![Plain("2")])` (inside "CO₂") | PPTX: `<a:rPr baseline="30000">`; HTML: `<sup>2</sup>` | happy-path |
| 65-deep nested `Bold(vec![Bold(vec![...])])` | `LayoutError::InlineDepthExceeded { slide_index: 0, depth: 65 }` | depth-bound |
| `Xref("nonexistent-slide")` in top-level text | `LayoutWarning::XrefTargetNotFound { target: "nonexistent-slide", slide_index: 0 }` | warning |
| All 12 variants present in one `Vec<InlineNode>` | All 12 variants survive layout pass unchanged; `FrameContent::TextRun` preserves all | exhaustive |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-043 | All 12 inline variant types produce distinct non-empty XML in PPTX output | snapshot tests (one per inline type) |
| VP-044 | Bold via markdown pattern does not trigger `b="1"` in output | unit test |
| VP-045 | Inline tree at depth 65 produces InlineDepthExceeded error | unit test + Kani (pure depth-count function) |
| VP-046 | Xref inside MathNode is NOT flagged by xref validation pass | unit test |
| VP-047 | All 12 variants survive layout pass in FrameContent::TextRun | unit test (exhaustive variant coverage) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 ("Rich Inline Formatting") per capabilities.md §CAP-024 |
| Capability Anchor Justification | CAP-024 ("Rich Inline Formatting") per capabilities.md §CAP-024 — "All 12 inline types ship v1.0" is stated in this BC; this BC is the single contract covering all 12 types and their per-format rendering behavior |
| L2 Domain Invariants | DI-004 (no implicit type coercion — markdown-style bold is not applied), DI-018 (error accumulation — depth exceeded is a hard error not a silent truncation) |
| Architecture Module | slideforge-eval crate — InlineNode enum; slideforge-pptx / slideforge-html / slideforge-pdf — inline renderers |
| Stories | STORY-028 |

## Related BCs

- BC-1.10.003 — depends on (math inline type rendering delegated to math renderer per CAP-012)
- BC-1.02.001 — related to ({{ expr }} text interpolation is separate from inline formatting but can appear within formatted runs)

## Architecture Anchors

- `architecture/crate-architecture.md` — InlineNode enum definition and 12 variant types
- `architecture/system-overview.md` — per-format inline rendering

## Story Anchor

STORY-028

## VP Anchors

VP-043, VP-044, VP-045, VP-046, VP-047
