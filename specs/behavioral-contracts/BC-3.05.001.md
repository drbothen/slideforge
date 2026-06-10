---
document_type: behavioral-contract
level: L3
version: "1.4.2"
status: active
producer: product-owner
timestamp: 2026-06-10T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-05
capability: CAP-024
lifecycle_status: active
introduced: v1.0.0
modified:
  - "v1.2 — adversary pass 1 adjudication: variant count corrected to 12, payload shapes corrected to Vec<InlineNode> for structured variants, inline depth bound added (max 64), math xref validation boundary codified"
  - "v1.3 — story spec AC-005 enum example corrected to match production types"
  - "v1.3.1 — VP propagation burst: assigned VP-043 through VP-047 to all VP-TBD entries"
  - "v1.3.2 — adversary pass 2 adjudications S/O: LayoutError::Multiple smart constructor invariants codified; XrefTargetNotFound warnings must reach LaidOutDeck.warnings (not silently dropped)"
  - "v1.3.3 — pass-7 drift fix (F-P7-HIGH-005): slide_index → source_slide_index in Invariant 4, Inline Depth Bound section, EC-002, EC-006, and Canonical Test Vectors per AC-BC-A9 canonical field name"
  - "v1.3.4 — F-P25-MED-001: Architecture Module corrected — InlineNode enum lives in slideforge-types (not slideforge-eval); inline validation pass (run_inline_validation) lives in slideforge-layout"
  - "v1.3.5 — adversary pass 2 OBS-2 (STORY-073): subsystem corrected from SS-TBD to SS-05 (Layout Engine); BC governs layout-stage validation and STORY-073 anchors subsystems: [SS-05]"
  - "v1.3.6 — STORY-085 adversary F-004 [HIGH]: tighten HTML postcondition to require HTML-escaping of ALL interpolated values — attribute values (href/url/Xref id) AND text content (Math latex source) — not just Plain text. Added EC-009. Escaping rule: &, <, >, \" must be escaped in all HTML output positions (both attribute and content contexts)."
  - "v1.4.0 — STORY-081 re-anchor (human ruling 2026-06-09): BC-3.05.001 is now the primary anchor for slide-level inline markup rendering. Added: (1) explicit slide-level field scope list (title/subtitle/body/bullets/caption/description) with AC-006 dual-title shadow invariant; (2) per-exporter rendering matrix (5 surfaces: PPTX body, PPTX notes, DOCX, HTML, PDF) with unified-engine detail per ADR-024 — body via ooxml_run_to_ooxmlsdk, notes via serialize_ooxml_run, both call render_inline_nodes_to_runs; (3) corrected PPTX Highlight postcondition from attribute form to child-element form (<a:highlight><a:srgbClr val=\"FFFF00\"/></a:highlight>) per ADV-P11-HIGH-001 fix; (4) corrected hyperlink reference-set invariant (replacing false count-equality claim) per ADR-024 INV-4; (5) rId namespace isolation (slide vs notes rels parts); (6) nested/wrapped link behavior (F-040-P3-001); (7) safe-URL-scheme and empty-display-text guards; (8) Math PPTX behavior (tracing::warn + degraded plain run); (9) EC-007 interpolation-stays-Plain; (10) ADR-024, ADR-017, BC-5.02.002 cross-references. STORY-081 added to Stories traceability."
  - "v1.4.1 — F-P17-002 Math/HTML deferral codification: PC-4 Math clause corrected from aspirational 'Math → MathML <math>' to actual v1.0 behavior: Math → <code class=\"math\">{HTML-escaped LaTeX source}</code> as a degraded accessible-text fallback. Full MathML rendering explicitly deferred to STORY-045 (cited). EC-010 and canonical test vector updated to reference <code class=\"math\"> element (not <math>) as the v1.0 HTML output. No other PC-4 form changed — Footnote → <span role=\"note\"> confirmed correct (F-P17-002 Footnote half resolved by implementation in feature/STORY-081 at e5b1e92e)."
  - "v1.4.2 — F-P24-MED-001 full-form accuracy reconciliation (2026-06-10): corrected all per-variant per-surface postconditions to match actual v1.0 implementation confirmed by code audit. (1) PC-1 Footnote: was aspirational 'presenter note annotation registered; superscript reference number emitted'; corrected to actual: inner body content rendered inline, numbered-marker mechanism DEFERRED to STORY-085 F-010. (2) PC-1 Xref: was aspirational 'internal hyperlink to slide index or heading via rId'; corrected to actual: plain text run rendered identically to Plain (ooxml_runs.rs:340-345, same match arm). (3) PC-2 Footnote and Xref: corrected to match actual (same engine as PC-1). (4) PC-3 Math: was aspirational '<m:oMath> block'; corrected to actual: plain text run of latex source (document_body.rs:564). (5) PC-3 Footnote: was aspirational '<w:footnote>/<w:endnote>'; corrected to actual: collect_plain_text(children) as plain run (document_body.rs:568). (6) PC-3 Xref: was aspirational 'internal hyperlink to heading or figure'; corrected to actual: plain text run of target string (document_body.rs:571). (7) PC-5 Math: was aspirational 'vector path rendering per BC-1.10.003'; corrected to actual: SKIPPED — no spans produced (slide_pdf.rs:294-296). (8) PC-5 Footnote: was aspirational 'footnote reference number / page reference in tagged PDF'; corrected to actual: inner body content rendered inline as regular text spans with inherited font face (slide_pdf.rs:282-287). (9) PC-5 Xref: was aspirational 'page reference in tagged PDF'; corrected to actual: plain text span rendered identically to Plain (slide_pdf.rs:238-244, same match arm). (10) OBS-P24-004 fix: InlineDepthExceeded struct shape corrected from 2-field { source_slide_index, depth: 65 } to 3-field { source_slide_index, depth, max } per production error.rs:226-233; error code corrected from missing to E-LAY-005 (error.rs doc comment line 218)."
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
Xref, Superscript, Subscript, Strikethrough, Highlight) are supported in slide-level
field values (`title`, `subtitle`, `body`, `bullets`, `caption`, `description`) and in
section body content. Each inline type maps to a distinct Rust `InlineNode` enum variant
and is rendered to the correct native representation across all five output surfaces: PPTX
slide body, PPTX notes, DOCX, HTML, and PDF. PPTX slide body and PPTX notes share a
single unified inline-run engine (`render_inline_nodes_to_runs` in `slideforge-plugin-api`,
per ADR-024). String-prefix-based formatting (e.g., `"**header**"`) is rejected —
only structural `InlineNode::*` variants are used. Nesting is bounded to a maximum
depth of 64 to prevent DoS.

## Preconditions

1. Text content contains inline formatting recognized by the DSL parser.
2. The declared inline type is one of the 12 registered types.
3. Output format is PPTX (slide body), PPTX (notes), DOCX, HTML, or PDF.
4. Inline nesting depth does not exceed 64.
5. For slide-level fields: the evaluated `FieldValue::Inlines` has been produced by
   `eval_slide_node` (STORY-081 eval stage) from the parsed `TemplateChunk` sequence.
   For `title` fields carrying inline markup, see the Slide-Level Title Constraint section.

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

Per output surface. All five surfaces must honor the contract (STORY-081 and ADR-024
established the unified engine and cross-surface correctness obligations).

### PC-1: PPTX Slide Body

The PPTX slide body path calls `render_inline_nodes_to_runs(nodes, hlink_resolver)`
(in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`) then converts the
resulting `Vec<OoxmlRun>` to typed ooxmlsdk `Run` objects via `ooxml_run_to_ooxmlsdk`.
Native OOXML mapping per variant:

- Plain → `<a:t>text</a:t>` in a plain run (no `<a:rPr>` or attributes set)
- Bold → `<a:rPr b="1"/>` before `<a:t>`
- Italic → `<a:rPr i="1"/>` before `<a:t>`
- Code → `<a:rPr><a:latin typeface="Courier New"/></a:rPr>` (monospace font run)
- Superscript → `<a:rPr baseline="30000"/>` (30% above baseline)
- Subscript → `<a:rPr baseline="-25000"/>` (25% below baseline)
- Strikethrough → `<a:rPr strike="sngStrike"/>`
- **Highlight → `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` as a CHILD
  ELEMENT of `<a:rPr>`, NOT an attribute.** The form `<a:rPr highlight="yellow">` is
  a schema violation (ADV-P11-HIGH-001). Brand highlight color replaces `FFFF00` if
  declared; defaults to yellow (#FFFF00) if not (EC-003).
- Link → `<a:hlinkClick r:id="rIdN"/>` child element in `<a:rPr>` on all display-text
  leaf runs; the URL is pre-registered as an External relationship in `slide{N}.xml.rels`
  by the body path before calling `render_inline_nodes_to_runs`. See Hyperlink Invariants.
- Math → `tracing::warn!` emitted; a plain-text `OoxmlRun` with the LaTeX source as
  text is produced (degraded run). Math block rendering is delegated to BC-1.10.003;
  `InlineNode::Math` in a slide body field is a degraded path in v1.0.
- Footnote → inner body content rendered inline as a plain run (children are recursed
  with unchanged `RunProps`). NO presenter-note annotation is registered and NO
  superscript reference number is emitted on the slide run. Numbered-marker mechanism
  **DEFERRED to STORY-085 F-010**. (`tracing::debug!("Footnote marker numbering deferred")`
  is emitted; see `ooxml_runs.rs:430-442`.)
- Xref → plain text run rendered identically to `Plain` (the production code matches
  `InlineNode::Plain(text) | InlineNode::Xref(text)` in a single arm at
  `ooxml_runs.rs:340-345`). No internal hyperlink or rId is registered in v1.0.
  Cross-reference resolution to slide headings is **DEFERRED to STORY-085**.

### PC-2: PPTX Notes

The PPTX notes path also calls `render_inline_nodes_to_runs(nodes, hlink_resolver)`,
then serializes the resulting `Vec<OoxmlRun>` to raw-string XML via `serialize_ooxml_run`
(same module in `slideforge-plugin-api`). The `hlink_resolver` closure is backed by the
notes-slide-scoped URL list (`notesSlide{N}.xml.rels`) — a separate rId namespace from
the slide body. Native OOXML mapping is **identical to PC-1** (both consumers share one
engine; the `OoxmlRun` intermediate is the same struct, serialized differently only in the
final conversion step). This was hardened across the 16-pass STORY-081 adversarial cascade:

- All 8 OOXML inline forms (Bold, Italic, Code, Highlight, Superscript, Subscript,
  Strikethrough, Link) produce the SAME schema-correct `<a:rPr>` structure in notes as
  in the slide body — no degraded form, no attribute-vs-element divergence.
- Formatting INSIDE a hyperlink's display text is preserved: `[click **here**](url)` in
  notes OOXML renders the "here" run with `<a:rPr b="1"><a:hlinkClick r:id="rId3"/></a:rPr>`
  (Fix for F-P16-M1 — the pre-ADR-024 notes path silently dropped bold in this case).
- **Footnote** (notes surface): same as PC-1 — inner body content rendered inline as a
  plain run; no presenter-note annotation; no superscript reference number. Both body
  and notes share the same engine (`render_inline_nodes_to_runs`), so the STORY-085
  F-010 deferral applies equally to both surfaces.
- **Xref** (notes surface): same as PC-1 — plain text run identical to `Plain`; no rId
  registered in v1.0.

### PC-3: DOCX

- Plain → `<w:r>` with `<w:t>` (no run properties)
- Bold → `<w:b/>` in `<w:rPr>`
- Italic → `<w:i/>` in `<w:rPr>`
- Code → `RunFonts { ascii: "Courier New", high_ansi: "Courier New" }` in `<w:rPr>`
  (monospace font applied via `RunFonts`; no `<w:rStyle>` element — see
  `document_body.rs:449-461`)
- Superscript → `<w:vertAlign w:val="superscript"/>` in `<w:rPr>`
- Subscript → `<w:vertAlign w:val="subscript"/>` in `<w:rPr>`
- Strikethrough → `<w:strike/>` in `<w:rPr>` (ooxmlsdk `Strike::default()`)
- Highlight → `<w:highlight w:val="yellow"/>` in `<w:rPr>` (ooxmlsdk typed builder,
  `HighlightColorValues::Yellow`)
- Link → `<w:hyperlink r:id="...">` wrapping the display-text runs (runs receive the
  `"Hyperlink"` character style; formatting inside link display text is preserved)
- Math → plain text run of the LaTeX source string with no run properties (degraded
  fallback; `document_body.rs:564-566`). Full OMML `<m:oMath>` rendering is
  **DEFERRED to STORY-045**.
- Footnote → `collect_plain_text(children)` emitted as a plain `<w:r>` with no run
  properties (degraded fallback; `document_body.rs:567-569`). Structured
  `<w:footnote>`/`<w:endnote>` rendering is **DEFERRED to STORY-085 F-010**.
- Xref → plain text run of the target identifier string with no run properties (degraded
  fallback; `document_body.rs:570-573`). Internal hyperlink to heading is
  **DEFERRED to STORY-085**.

### PC-4: HTML

- Plain → bare text node, Bold → `<strong>`, Italic → `<em>`, Code → `<code>`,
  Link → `<a href="...">`, **Math → `<code class="math">{HTML-escaped LaTeX source}</code>`
  (v1.0 degraded accessible-text fallback; full MathML `<math>` rendering DEFERRED to
  STORY-045)**, Footnote → `<span role="note">`,
  Xref → `<a href="#...">`, Superscript → `<sup>`, Subscript → `<sub>`,
  Strikethrough → `<del>`, Highlight → `<mark>`
- **HTML escaping is mandatory for ALL interpolated values in HTML output — not only
  `Plain` text.** The characters `&`, `<`, `>`, and `"` MUST be escaped to their HTML
  entity equivalents in every position they appear, regardless of which `InlineNode`
  variant they originate from:
  - **Attribute values** (href, data-* attributes, etc.): The `url` field of `Link`
    nodes and the `id` field of `Xref` nodes MUST be HTML-attribute-escaped before
    being written into `href="..."` or equivalent attribute positions. A raw `url`
    containing `"` would break attribute quoting; a `url` containing `<` would break
    HTML structure.
  - **Text content** (rendered inside element tags): The `latex` source string carried
    by `Math` nodes, the inner text of `Plain`, `Code`, and all other leaf nodes MUST
    be HTML-content-escaped before being written between tags. An unescaped `<` in
    LaTeX source would be interpreted as a tag open.
  - Failure to escape ANY of these positions is a security defect (XSS injection via
    crafted slide content) and a correctness defect (malformed HTML). `DefaultInlineFormat`
    MUST escape all positions. Third-party `InlineFormat` implementors are strongly
    advised to do the same — the trait contract documents this requirement.

### PC-5: PDF

PDF inline rendering via krilla `=0.6.0` (pinned in `slideforge-pdf/Cargo.toml`). Font
faces are resolved from `ResolvedFontSet` via `resolve_font_set(brand, override_path)`
using `fontdb =0.23.0` metadata-aware lookup (ADR-023). Fallback: `tracing::warn!` then
regular face (never silent wrong-face).

- Bold → `ResolvedFontSet.bold` face (distinct `Font` instance; no `set_bold()` API in krilla 0.6.0)
- Italic → `ResolvedFontSet.italic` face
- Code → `ResolvedFontSet.mono` face (`FontFaceKind::Mono`; `slide_pdf.rs:245-250`)
- Superscript → `ResolvedFontSet.regular` face at `font_size * SUPER_SUB_SCALE` (0.583)
  with `baseline_y - (font_size * SUPER_RISE_FRACTION)` (0.333) raised in Y-down space
  (`SUPER_OFFSET_UNITS` signal = +333 in `KrillaTextSpan.y_offset_units`;
  `exporter.rs` `SUPER_SUB_SCALE`/`SUPER_RISE_FRACTION` constants)
- Subscript → `ResolvedFontSet.regular` face at `font_size * SUPER_SUB_SCALE` (0.583)
  with `baseline_y + (font_size * SUB_DROP_FRACTION)` (0.333) lowered in Y-down space
  (`SUB_OFFSET_UNITS` signal = −333)
- Strikethrough → inner body content rendered inline as regular text spans with the
  inherited font face (`slide_pdf.rs:282-287`). In v1.0, no strike-line path is drawn
  over the run. Native strike decoration is **DEFERRED to STORY-085**. This is
  CONSISTENT (same top-level and nested behavior — both produce plain spans).
- Highlight → inner body content rendered inline as regular text spans with the
  inherited font face (`slide_pdf.rs:282-287`). In v1.0, no filled-rectangle background
  is drawn behind the run. Highlight background rendering is **DEFERRED to STORY-085**.
  This is CONSISTENT (same top-level and nested behavior).
- Link → display-text content is recursed and collected as regular text spans with the
  inherited font face (`slide_pdf.rs:289-293`). In v1.0, no URL annotation / rectangle
  overlay is added. PDF link annotation rendering is **DEFERRED to STORY-085**.
- Plain → regular text span with inherited font face (PDF/UA structure tag applied by
  `tag_engine`)
- Math → **SKIPPED** — `InlineNode::Math(_) => {}` produces NO `KrillaTextSpan`
  entries (`slide_pdf.rs:294-297`). Math content is absent from PDF output in v1.0.
  Full math rendering via `BC-1.10.003` is **DEFERRED to STORY-009 / STORY-045**.
- Footnote → inner body content rendered inline as regular text spans with the
  inherited font face (handled in same arm as `Strikethrough`/`Highlight` at
  `slide_pdf.rs:282-287`). No reference number is emitted and no footnote annotation
  is registered. Structured footnote support is **DEFERRED to STORY-085 F-010**.
- Xref → plain text span rendered identically to `Plain` (same match arm as `Plain`
  at `slide_pdf.rs:238-244`). No page-reference annotation or cross-reference
  resolution in v1.0. Full cross-reference support is **DEFERRED to STORY-085**.

**PDF v1.0 scope boundary (actual):** Bold/Italic/Code/Super/Sub are fully rendered
with correct font face and baseline positioning. All other formatting variants
(Strikethrough, Highlight, Link, Footnote, Xref, Math) produce plain text spans only
in v1.0 — no geometric annotations, no URL overlays, no reference numbers, no math
paths. This is the defined v1.0 PDF boundary per STORY-081 AC-004. These are not
violations; full rendering for these variants is tracked in STORY-085 (Footnote, Xref,
Strikethrough, Highlight, Link) and STORY-009/STORY-045 (Math).

## Slide-Level Field Scope

Inline markup renders in the following slide-level field values (established by STORY-081):

| Field | Inline markup supported? | Notes |
|-------|-------------------------|-------|
| `title` | Special — see constraint below | AC-006 dual-title behavior applies |
| `subtitle` | Yes | Full inline markup; all exporters |
| `body` | Yes | Full inline markup; all exporters |
| `bullets` (each list item) | Yes | Full inline markup; all exporters |
| `caption` | Yes | Full inline markup; all exporters |
| `description` | Yes | Full inline markup; all exporters |

### Slide-Level Title Constraint (AC-006 Dual-Title Shadow)

PPTX title placeholder fields (`<p:ph type="title"/>`) do not reliably support mixed
inline formatting across all renderers (PowerPoint, Keynote, Google Slides). The layout
engine enforces:

1. If a `title` field is parsed with inline markup, the eval stage emits a
   `LayoutWarning::InlineMarkupInTitle { slide_title, stripped_text }` warning.
2. PPTX output receives a plain-text title run (`<a:t>stripped_text</a:t>` — no
   `<a:rPr>` formatting on the title placeholder).
3. For DOCX, PDF, and HTML output, a `title_inlines: Vec<InlineNode>` shadow field is
   produced by the eval stage and used by non-PPTX exporters to render the title with
   full inline structure.
4. In **strict mode** (default), `LayoutWarning::InlineMarkupInTitle` is promoted to a
   fatal error (build exits 1). In `--warn-only` mode, it is a warning and output is
   produced with the PPTX-plain / non-PPTX-inline split.

This is NOT a missing feature — it is the defined contract. Authors must either rewrite
the title without markup, or accept the PPTX plain-text degradation under `--warn-only`.

## Hyperlink Invariants (PPTX — Both Body and Notes)

These invariants apply to `InlineNode::Link` nodes rendered to PPTX (both body and
notes surfaces). They were established and hardened by the STORY-081 adversarial cascade
(passes 1-16) and codified in ADR-024 INV-4 through INV-7.

### HI-1: Corrected Reference-Set Invariant (Replaces False Count-Equality)

Every `<a:hlinkClick>` run references a registered External relationship, AND every
registered External relationship is referenced by at least one `<a:hlinkClick>` run
(no orphan rel, no dangling rId).

**IMPORTANT:** The count-equality form (`external_rel_count == hlinkclick_count`) is
FALSE for multi-leaf link display text and MUST NOT be used as a test assertion or
invariant. A single External relationship can back N `<a:hlinkClick>` runs when the
display text tree has multiple leaf nodes (e.g., `[click **here** now](url)` produces
three runs — "click ", "here", " now" — each with `hyperlink_rid = Some(rId3)`, backed
by one External rel). The reference-set form stated above is the correct invariant.

### HI-2: rId Namespace Isolation

The slide-body path and the notes-slide path each use their own rId namespace:
- Slide body: `slide{N}.xml.rels` (rIds allocated by `slide_serializer.rs`)
- Notes: `notesSlide{N}.xml.rels` (rIds allocated by `notes_slide.rs`)

No rId leaks between slide and notes namespaces. `render_inline_nodes_to_runs` is
namespace-agnostic — it calls the caller-provided `hlink_resolver` closure and stores
whatever rId string it returns. The consumer constructs the closure from its own
namespace. No cross-part rId leakage is possible by construction (ADR-024 INV-10).

### HI-3: Nested-Link-in-Display-Text No-Double-Register (F-040-P3-001)

A `Link` node whose display text children contain another `Link` node does NOT
double-register an External relationship. The outer link's URL is registered once.
The inner `Link`'s URL is not registered; its display text is rendered recursively
and the resulting runs inherit the outer link's rId (ADR-024 INV-5 + INV-6).

### HI-4: Safe-Scheme and Empty-Display-Text Guards

A `Link` node whose URL has an unsafe scheme (not `https://`, `http://`, `mailto://`,
or other explicitly permitted schemes) produces a plain-text run with no `<a:hlinkClick>`
and no External relationship registered. A `Link` node with empty display text likewise
produces no orphan relationship. These guards are applied by the `hlink_resolver`
closure (caller's responsibility to map only safe-scheme URLs to rIds).

### HI-5: Formatting Preserved Inside Link Display Text (F-P16-M1 Fix)

Bold, italic, and other formatting variants wrapping a link, or inside a link's display
text, are preserved in ALL PPTX output (body AND notes). `[click **here**](url)` renders
with `<a:rPr b="1"><a:hlinkClick r:id="rIdN"/></a:rPr>` on the "here" run. The
pre-ADR-024 notes path silently dropped formatting in this case (F-P16-M1); ADR-024
unified engine eliminates the bug class by construction.

## Invariants

1. All 12 inline types ship in v1.0 — no deferral. (CAP-024 priority: P1)
2. String-prefix-based bold (`"**text**"`) is an unrecognized pattern. The parser
   does not interpret markdown-style inline markup. This is a forbidden pattern
   per CLAUDE.md. (DI-004 implies values are not implicitly transformed)
3. Inline formatting within a math block (`$...$`) uses `@{var}` interpolation only;
   `{{ }}` text interpolation is disabled inside math mode. (BC-1.02.004)
4. Nesting depth is bounded at 64 levels. Inline trees deeper than 64 produce
   `LayoutError::InlineDepthExceeded { source_slide_index, depth: 65, max: 64 }`
   (error code **E-LAY-005** per `slideforge-layout/src/error.rs` doc comment). This
   is a 3-field struct — `source_slide_index`, `depth` (the actual depth detected,
   ≥ 65), and `max` (the allowed maximum, 64). This is a hard error (not a warning)
   to prevent stack overflow on export.
5. `InlineNode::Xref` validation traverses ONLY top-level inline sequences; it does
   NOT recursively validate xrefs inside `MathNode` content. Math is a separate
   validation surface. (v1.0 scope boundary — see Math Xref Boundary section below.)
6. The layout stage preserves `InlineNode` sequences verbatim in `FrameContent::TextRun`.
   Exporters translate per-format; the layout stage does NOT produce format-specific
   markup.
7. **XrefTargetNotFound warnings must reach LaidOutDeck.warnings (Item O).** The layout
   runner (`layout::run`) MUST NOT silently drop `LayoutWarning::XrefTargetNotFound`
   events produced by `run_inline_validation`. They MUST be accumulated in
   `LaidOutDeck.warnings: Vec<LayoutWarning>` and returned to the caller. A layout
   runner that drops warnings without propagating them violates this BC and BC-3.04.001
   postcondition 7. (CLAUDE.md Rule 4: AI-built defects are the AI's responsibility to fix)
8. **LayoutError::Multiple smart constructor invariants (Item S).** The constructor
   `LayoutError::multiple(errors: Vec<Self>) -> Self` enforces these invariants at
   all call sites:
   - `errors` MUST be non-empty. An empty vec triggers `debug_assert!` (panics in
     debug builds; undefined behavior in release would be worse — the assert makes
     the bug visible). An empty `Multiple` is a logic error in the accumulation loop.
   - Nested `Multiple` variants MUST be flattened. `multiple(vec![Multiple { inner }])`
     MUST produce `Multiple { inner: flattened_vec }`, NOT
     `Multiple { inner: vec![Multiple { inner }] }`. Nested `Multiple` values are
     unreachable from correct callers but the constructor defends against misuse.
   See interface-definitions.md §9 for the full constructor contract.
9. **Unified engine is the sole PPTX inline-run generator (ADR-024 INV-1 / BC-5.02.002).**
   `render_inline_nodes_to_runs` (in `slideforge-plugin-api/src/inline_formats/ooxml_runs.rs`)
   is the SINGLE function that maps `&[InlineNode]` to OOXML run structures for both the
   slide-body path and the notes-slide path. No other function in `slideforge-pptx`
   constructs `<a:r>` or `<a:rPr>` markup. The AC-005 grep-zero test vector (`grep -r
   "a:r\|a:rPr\|serialize_inline" crates/slideforge-pptx/src/` excluding the two
   conversion functions) MUST return zero matches.
10. **A resolved variable that looks like markup stays Plain (interpolation EC-007).**
    `{{ x }}` resolving to a string containing `**bold**` or other markup delimiters
    produces `InlineNode::Plain(Arc::from("**bold**"))`. The resolved string is NOT
    re-parsed for inline markup. This applies to ALL slide-level content fields.
    (DIR-077-002 §3 rule: inline markup is parsed from DSL source, not dynamically
    resolved values.)

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
`LayoutError::InlineDepthExceeded { source_slide_index, depth: 65, max: 64 }` (or
the actual exceeded depth in the `depth` field; `max` is always 64). Error code:
**E-LAY-005** (per `slideforge-layout/src/error.rs` §`InlineDepthExceeded` doc
comment). This is a hard error — output is NOT produced for the affected slide.

Rationale: unbounded recursion in inline tree traversal during export (PPTX XML
generation, PDF span building, HTML generation) can exhaust the stack on real-world
inputs. 64 levels is a conservative bound that no legitimate document approaches.

Canonical test vector: a tree of 65 nested `Bold(vec![Bold(vec![...])])` nodes
must produce `LayoutError::InlineDepthExceeded`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Nested bold inside italic (`Italic(vec![Bold(vec![Plain("text")])])`) | Both applied: PPTX body `<a:rPr b="1" i="1">`; PPTX notes same via unified engine; HTML `<em><strong>text</strong></em>` |
| EC-002 | Xref to a slide title that doesn't exist | `LayoutWarning::XrefTargetNotFound { target, source_slide_index }` accumulated; not fatal |
| EC-003 | Highlight in PPTX with no highlight color in brand | Default to yellow highlight (#FFFF00) via `<a:highlight><a:srgbClr val="FFFF00"/></a:highlight>` child element; lint warning: "highlight color not declared in brand; using yellow" |
| EC-004 | Footnote in PPTX (no structured footnote feature in slides) | Inner body content rendered inline as a plain `OoxmlRun` (children recursed with unchanged `RunProps`). No presenter-note annotation registered; no superscript reference number emitted on the slide run. `tracing::debug!("Footnote marker numbering deferred")` emitted. Numbered-marker mechanism DEFERRED to STORY-085 F-010. |
| EC-005 | Code inline in .pptx (no semantic code type in OOXML) | Monospace font run via `<a:latin typeface="Courier New"/>`; no semantic tagging (PPTX limitation documented in DSL reference) |
| EC-006 | Inline nesting at depth 65 | `LayoutError::InlineDepthExceeded { source_slide_index, depth: 65, max: 64 }` (E-LAY-005); hard error; 3-field struct |
| EC-007 | `Xref("slide-title")` inside a `MathNode` | NOT validated by xref pass; math is a separate validation surface |
| EC-008 | `"**bold using markdown"` (forbidden pattern) | No bold applied; literal `**bold using markdown` rendered as Plain text; lint warning |
| EC-009 | `Link { text: vec![Plain("click")], url: "https://evil.com?a=1&b=<script>" }` rendered to HTML | `href` attribute value is HTML-attribute-escaped: `href="https://evil.com?a=1&amp;b=&lt;script&gt;"`. The unescaped raw URL is NEVER written directly into attribute position. Similarly, `Xref("my\"id")` produces `href="#my&quot;id"` — not `href="#my"id"`. |
| EC-010 | `Math(MathNode { latex: "<b>not bold</b>" })` rendered to HTML | LaTeX source is HTML-content-escaped before being written as the text content of `<code class="math">`: `<code class="math">&lt;b&gt;not bold&lt;/b&gt;</code>`. An unescaped `<` would break the HTML parse tree. v1.0 renders `<code class="math">` (STORY-045 deferred MathML). |
| EC-011 | `title: "**Bold Title**"` containing inline markup | Eval stage: `LayoutWarning::InlineMarkupInTitle` emitted; `FieldValue::Str("Bold Title")` for PPTX title placeholder; `title_inlines: vec![InlineNode::Bold([InlineNode::Plain("Bold Title")])]` shadow for DOCX/PDF/HTML. In strict mode: fatal error. In `--warn-only`: warning only. |
| EC-012 | `Link { text: vec![Plain("click "), Bold([Plain("here")]), Plain(" now")], url: "https://example.com" }` in PPTX notes | THREE runs produced (one per leaf), each with `hyperlink_rid = Some("rId3")`; the "here" run has `bold=true` AND `hyperlink_rid=Some("rId3")`; ONE External rel registered. Count-equality test (`ext_rel_count == hlinkclick_count`) would be `1 != 3` — use reference-set invariant HI-1 instead. |
| EC-013 | `Link { url: "javascript:evil()", text: vec![Plain("x")] }` in PPTX | Unsafe URL scheme: no External relationship registered; no `<a:hlinkClick>` emitted; display text "x" rendered as plain run; `tracing::warn!` emitted. |
| EC-014 | `Link { url: "https://example.com", text: vec![Link { url: "https://inner.com", text: vec![Plain("inner")] }] }` in PPTX | Outer URL registered once; inner URL NOT registered (F-040-P3-001 / HI-3). Inner display text runs inherit the outer rId. No orphan rel, no double-register. |
| EC-015 | `{{ x }}` where `x = "**literal**"` in slide bullet | `InlineNode::Plain(Arc::from("**literal**"))` — NOT re-parsed as Bold. The resolved string is treated as plain text (Invariant 10 / DIR-077-002 §3). |
| EC-016 | `InlineNode::Math` in PPTX slide body (inline math in a bullet) | `tracing::warn!` emitted; a plain-text OoxmlRun with the LaTeX source as text is produced (degraded path). Full math rendering via BC-1.10.003 is the correct path; this EC documents the defined degraded behavior for v1.0 InlineNode::Math in slide body context. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Bold(vec![Plain("Key finding")])` in PPTX slide body | `<a:r><a:rPr b="1"/><a:t>Key finding</a:t></a:r>` | happy-path |
| `Bold(vec![Plain("Key finding")])` in PPTX notes | `<a:r><a:rPr b="1"/><a:t>Key finding</a:t></a:r>` — identical to body via unified engine | happy-path (notes parity) |
| `Highlight(vec![Plain("text")])` in PPTX | `<a:r><a:rPr><a:highlight><a:srgbClr val="FFFF00"/></a:highlight></a:rPr><a:t>text</a:t></a:r>` — child element, NOT `highlight="yellow"` attribute | highlight-child-element |
| `Link { text: vec![Plain("See report")], url: "https://example.com" }` | PPTX body: hlinkClick relationship in slide{N}.xml.rels, runs with `hyperlink_rid`; PPTX notes: hlinkClick in notesSlide{N}.xml.rels, same run structure; HTML: `<a href="https://example.com">See report</a>` | happy-path |
| `Link { text: vec![Plain("click "), Bold([Plain("here")]), Plain(" now")], url: "https://example.com" }` in PPTX notes | 3 runs, each with `hyperlink_rid="rId3"`; "here" run has `b="1"` AND `hyperlink_rid`; 1 External rel registered | notes-formatting-in-link |
| `"**bold using markdown"` (forbidden pattern) | No bold; literal text rendered; lint warning issued | edge-case |
| `Superscript(vec![Plain("2")])` (inside "CO₂") | PPTX: `<a:rPr baseline="30000">`; HTML: `<sup>2</sup>` | happy-path |
| 65-deep nested `Bold(vec![Bold(vec![...])])` | `LayoutError::InlineDepthExceeded { source_slide_index: 0, depth: 65, max: 64 }` (E-LAY-005) | depth-bound |
| `Xref("nonexistent-slide")` in top-level text | `LayoutWarning::XrefTargetNotFound { target: "nonexistent-slide", source_slide_index: 0 }` | warning |
| All 12 variants present in one `Vec<InlineNode>` | All 12 variants survive layout pass unchanged; `FrameContent::TextRun` preserves all | exhaustive |
| `Link { text: vec![Plain("x")], url: "https://a.com?q=1&lang=<en>" }` → HTML | `<a href="https://a.com?q=1&amp;lang=&lt;en&gt;">x</a>` — `&` and `<` escaped in attribute | escaping-attribute |
| `Math(MathNode { latex: "a < b & c > d" })` → HTML | v1.0: `<code class="math">a &lt; b &amp; c &gt; d</code>` — LaTeX source HTML-escaped as text content of `<code class="math">`. (Full MathML deferred to STORY-045.) | escaping-content |
| `Xref("slide\"with-quote")` → HTML | `<a href="#slide&quot;with-quote">` — `"` escaped in attribute value | escaping-attribute |
| slide with `title: "**Bold**"` → PPTX (strict mode) | `LayoutError` (promoted from `InlineMarkupInTitle` warning); build exits 1; no PPTX output | title-constraint-strict |
| slide with `title: "**Bold**"` → DOCX (--warn-only) | `LayoutWarning::InlineMarkupInTitle` warning; DOCX title rendered as bold via `title_inlines` shadow field | title-constraint-warn-only |
| `{{ x }}` where `x = "**markup**"` in bullet | `InlineNode::Plain(Arc::from("**markup**"))` — no re-parsing; literal asterisks in output | interpolation-stays-plain |

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
| Capability Anchor Justification | CAP-024 ("Rich Inline Formatting") per capabilities.md §CAP-024 — "All 12 inline types ship v1.0" is stated in this BC; this BC is the single contract covering all 12 types and their per-format rendering behavior, including the slide-level field scope established by STORY-081 |
| L2 Domain Invariants | DI-004 (no implicit type coercion — markdown-style bold is not applied; resolved variables containing markup stay plain), DI-018 (error accumulation — depth exceeded is a hard error not a silent truncation) |
| Architecture Module | `slideforge-types` crate — `InlineNode` enum; `slideforge-layout` crate — inline validation pass (`run_inline_validation`); `slideforge-plugin-api` crate — unified PPTX engine (`render_inline_nodes_to_runs`, `OoxmlRun`, `serialize_ooxml_run`) per ADR-024; `slideforge-eval` crate — `eval_slide_node` (slide-level field conversion, STORY-081); `slideforge-pptx` — body path (`ooxml_run_to_ooxmlsdk`) + notes path (`serialize_ooxml_run` call site); `slideforge-docx` / `slideforge-html` / `slideforge-pdf` — per-format inline renderers |
| Stories | STORY-028, STORY-081 |

## Related BCs

- BC-1.10.003 — depends on (math inline type rendering delegated to math renderer per CAP-012)
- BC-1.02.001 — related to ({{ expr }} text interpolation is separate from inline formatting but can appear within formatted runs)
- BC-5.02.002 — composes with (dog-fooding guarantee: the unified PPTX engine in `slideforge-plugin-api` satisfies BC-5.02.002's requirement that no bundled plugin bypasses the registered trait interface; ADR-024 unified engine is the mechanism that closes this gap for both body and notes paths)
- BC-3.02.002 — related to (slide-level inline markup for section sub-block fields was previously deferred to a follow-up story per BC-3.02.002 PC8 scope boundary; STORY-081 closes that gap; this BC is now the anchor for slide-level inline markup rendering across all exporters)

## Architecture Anchors

- `architecture/crate-architecture.md` — InlineNode enum definition and 12 variant types
- `architecture/system-overview.md` — per-format inline rendering
- `architecture/adr/ADR-024-pptx-inline-run-generator-unification.md` — unified engine design: `render_inline_nodes_to_runs` + `OoxmlRun` + `serialize_ooxml_run` in `slideforge-plugin-api`; body path via `ooxml_run_to_ooxmlsdk`; notes path via `serialize_ooxml_run`; INV-1 through INV-10 (hyperlink correctness, namespace isolation, nested-link no-double-register, formatting preservation in link display text)
- `architecture/adr/ADR-017-inline-format-relationship-context-and-hyperlink-ownership.md` — defines `InlineRenderContext` + `render_with_context` (Option A, accepted); ADR-024 supersedes its usage as the PPTX notes dispatch mechanism but both ADRs coexist; ADR-017 governs the trait extension; ADR-024 governs the PPTX serializer call strategy

## Story Anchor

STORY-028, STORY-081

## VP Anchors

VP-043, VP-044, VP-045, VP-046, VP-047
