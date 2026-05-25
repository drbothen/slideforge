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
capability: CAP-012
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

# BC-1.10.003: Math Renders to OMML for PPTX/DOCX, MathML for HTML, Paths for PDF

## Description

A successfully parsed and validated math expression is rendered to a format-specific
representation by the MathRenderer plugin. The target format determines which rendering
backend is used: OMML (Office Math Markup Language) for PPTX and DOCX, MathML/KaTeX-HTML
for static HTML and web preview, and vector paths (via resvg) for PDF. A single math
AST node in the `LaidOutDeck` IR is the single source from which all format-specific
representations are derived.

## Preconditions

1. A math expression has been successfully parsed into a math AST node (no E-PAR or E-EXP-006 errors).
2. The MathRenderer plugin is compiled into the slideforge binary.
3. An output format is specified (pptx, docx, html, pdf, or preview).

## Postconditions

1. PPTX output: math is serialized as OMML (`<m:oMath>`) embedded in the slide's shape run. The OMML is valid per ECMA-376 Part 1 §22.
2. DOCX output: math is serialized as OMML embedded in the paragraph run (same as PPTX).
3. Static HTML output: math is rendered to MathML or KaTeX HTML `<math>` element accessible to screen readers.
4. Web preview: math is rendered via KaTeX to HTML+CSS for display in the browser.
5. PDF output: math is rendered to vector SVG paths via usvg+resvg and embedded as a path group (no text characters — full rasterization avoidance).
6. All five formats produce mathematically equivalent representations of the same expression.

## Invariants

1. The same math AST node is the source for all format renderings — no separate math source per format.
2. PPTX/DOCX OMML is produced from the math AST, not from a LaTeX string (ensures OOXML schema compliance).
3. PDF math paths are vector (not rasterized images) — required for PDF/UA-1 text extraction compliance where possible.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Display math (`$$...$$`) in PPTX | OMML display-mode equation block (centered, full-width) |
| EC-002 | Inline math inside a `bullets:` list item | OMML inline math run embedded within the bullet paragraph |
| EC-003 | Math expression with `@{var}` substitution (from BC-1.10.001) | Variable value spliced before rendering; rendered form uses resolved value |
| EC-004 | Math in a `notes` register | Math renders in PPTX speaker notes as OMML; HTML preview renders as KaTeX |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `$x^2 + y^2 = r^2$` built to PPTX | Output contains `<m:oMath>` with correct OMML; exit 0 | happy-path |
| `$$\sum_{i=0}^{n} i = \frac{n(n+1)}{2}$$` built to HTML | Output contains `<math>` or KaTeX HTML; exit 0 | happy-path |
| `$\alpha$` built to PDF | Vector path group in PDF (no rasterized image); exit 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | PPTX output contains <m:oMath> for each math expression | integration test: unzip PPTX, parse slide XML, verify OMML presence |
| VP-TBD | HTML output contains MathML or KaTeX HTML for each math expression | integration test: parse HTML, check for math element |
| VP-TBD | All 5 formats produce output with no E-EXP-006 for a supported expression | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 |
| Capability Anchor Justification | CAP-012 ("Math and LaTeX Rendering") per capabilities.md §CAP-012 — "Produce OMML for PPTX/DOCX, MathML/KaTeX for HTML, vector paths for PDF" is verbatim from CAP-012 |
| L2 Domain Invariants | DI-012 (single .sf source produces all formats consistently) |
| Architecture Module | slideforge-math crate (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-1.10.001 — depends on (this BC handles the rendering of math nodes produced by BC-1.10.001)
- BC-1.10.002 — depends on (this BC runs only when BC-1.10.002 has not produced an error)
- BC-4.01.001 — related to (PPTX exporter consumes OMML from this BC)

## Architecture Anchors

- `architecture/authoring-subsystem.md#math-mode` — MathRenderer plugin trait
- `architecture/pipeline.md` — math rendering stage in the export phase

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
