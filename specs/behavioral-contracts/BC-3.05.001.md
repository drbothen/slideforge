---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-024
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.05.001: All 11 Inline Format Types Render to Correct Output per Format

## Description

All 11 inline formatting types (bold, italic, code, link, math, footnote, xref,
superscript, subscript, strikethrough, highlight) are supported in slide text fields
and section body content. Each inline type maps to a distinct Rust `Inline` enum variant
and is rendered to the correct OOXML, HTML, or PDF representation per output format.
String-prefix-based formatting (e.g., `"**header**"`) is rejected — only structural
`Inline::*` variants are used.

## Preconditions

1. Text content contains inline formatting markup recognized by the DSL parser.
2. The declared inline type is one of the 11 registered types.
3. Output format is PPTX, DOCX, HTML, or PDF.

## Postconditions

Per output format:

**PPTX/DOCX:**
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
- Bold → `<strong>`, Italic → `<em>`, Code → `<code>`, Link → `<a href>`,
  Math → MathML `<math>`, Footnote → `<span role="note">`, Xref → `<a href="#">`,
  Super → `<sup>`, Sub → `<sub>`, Strike → `<del>`, Highlight → `<mark>`

**PDF:**
- Bold/Italic/Strike/Super/Sub → equivalent tagged text spans with PDF structure tags
- Code → monospace font span in tagged PDF
- Link → PDF annotation with URI action
- Math → vector path rendering per BC-1.10.003
- Highlight → PDF highlight annotation

## Invariants

1. All 11 inline types ship in v1.0 — no deferral. (CAP-024 priority: P1)
2. String-prefix-based bold (`"**text**"`) is an unrecognized pattern. The parser
   does not interpret markdown-style inline markup. This is a forbidden pattern
   per CLAUDE.md. (DI-004 implies values are not implicitly transformed)
3. Inline formatting within a math block (`$...$`) uses `@{var}` interpolation only;
   `{{ }}` text interpolation is disabled inside math mode. (BC-1.02.004)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Nested bold inside italic (`italic: bold: "text"`) | Both applied: PPTX `<a:rPr b="1" i="1">`; HTML `<em><strong>text</strong></em>` |
| EC-002 | Xref to a slide title that doesn't exist | E-EVL-001-class: "Xref target 'slide-title' not found in deck"; error accumulated |
| EC-003 | Highlight in PPTX with no highlight color in brand | Default to yellow highlight (#FFFF00); lint warning: "highlight color not declared in brand; using yellow" |
| EC-004 | Footnote in PPTX (no footnote feature in slides natively) | Footnote content moved to presenter notes with superscript reference number on slide |
| EC-005 | Code inline in .pptx (no semantic code type in OOXML) | Monospace font run; no semantic tagging (PPTX limitation documented in DSL reference) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `bullets: [bold: "Key finding"]` | PPTX: `<a:r><a:rPr b="1"/><a:t>Key finding</a:t></a:r>` | happy-path |
| `body: link: "See report" url: "https://example.com"` | PPTX: hlinkClick relationship; HTML: `<a href="https://example.com">See report</a>` | happy-path |
| `body: "**bold using markdown"` (forbidden pattern) | No bold applied; literal `**bold using markdown` rendered as plain text; lint warning | edge-case |
| `body: superscript: "2" (inside "CO₂")` | PPTX: `<a:rPr baseline="30000">`; HTML: `<sup>2</sup>` | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | All 11 inline variant types produce distinct non-empty XML in PPTX output | snapshot tests (one per inline type) |
| VP-TBD | Bold via `**markdown**` pattern does not trigger bold rendering | unit test (verify no b="1" in output for markdown syntax) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 ("Rich Inline Formatting") per capabilities.md §CAP-024 |
| Capability Anchor Justification | CAP-024 ("Rich Inline Formatting") per capabilities.md §CAP-024 — "All 11 inline types ship v1.0" is stated verbatim; this BC is the single contract covering all 11 types and their per-format rendering behavior |
| L2 Domain Invariants | DI-004 (no implicit type coercion — markdown-style bold is not applied) |
| Architecture Module | slideforge-eval crate — Inline enum; slideforge-pptx / slideforge-html / slideforge-pdf — inline renderers (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.10.003 — depends on (math inline type rendering delegated to math renderer per CAP-012)
- BC-1.02.001 — related to ({{ expr }} text interpolation is separate from inline formatting but can appear within formatted runs)

## Architecture Anchors

- `architecture/layout-subsystem.md#inline-format` — Inline enum definition and 11 variant types
- `architecture/system-overview.md#export` — per-format inline rendering

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
