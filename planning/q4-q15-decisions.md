---
title: "Q4-Q15 Decisions — Tier 1 (Q4-Q7) + Tier 2 (Q8-Q15)"
date: 2026-05-24
status: LOCKED
decided_by: human
---

# Q4-Q15 Decisions

## Q4: Per-slide vs. deck-level template binding — LOCKED
**Decision:** Deck-level brand + brand_overlay per slide/section in v1.0. Multi-master (section-level full brand switch) in v2.
- One base brand per deck (or per variant)
- `brand_overlay:` block on slides or sections swaps logo, footer, confidentiality banner
- Under the hood: single master, placeholder content replacement (no master switching)
- Cross-renderer safe (single master works in PowerPoint, Keynote, Google Slides, LibreOffice)
- DOCX: per-section header/footer swap via sectPr
- Multi-master deferred to v2 (Google Slides degrades multi-master)
- Research: R12 (planning/template-binding-research.md)

Syntax:
```
metadata:
  brand "brands/1898.toml"

slide title:
  brand_overlay:
    logo "assets/acme-logo.png"
    footer "Prepared by 1898 & Co. for Acme Corp"
    confidentiality "CONFIDENTIAL"

section "Partner":
  brand_overlay:
    logo "assets/partner.png"
  slide content:
    title "Partner's Analysis"
```

## Q5: Output target declaration — LOCKED
**Decision:** CLI-driven. The .sf file does NOT declare output formats.
- `slideforge build deck.sf` → all formats
- `slideforge build deck.sf --format pptx,pdf` → specific formats
- Project defaults in `slideforge.toml`: `[build] default_formats = ["pptx", "docx"]`
- `outputs:` field reserved for v2

## Q6: Accessibility DSL surface — LOCKED
**Decision:** Accessibility is a compile-time contract.
- `alt "..."` REQUIRED on every visual element (images, charts, diagrams) — compile error if absent
- `decorative: true` opts out of alt requirement (emits empty alt + PDF Artifact tag)
- `lang "en-US"` required at deck level (defaults to "en" if absent)
- `label "..."` required on color-coded elements (severity_cards, status, progress_bar, weighted_composite) — compile error if color carries meaning without text label
- Validators enforce all above in strict mode (default)

## Q7: IR stability contract — RawBlock escape hatch — LOCKED
**Decision:** IR-internal raw only + structured shape DSL ships in v1.0. No raw XML exposure to users.
- `Raw { exporter: ExporterId, content: String }` exists in IR for plugin use only
- `raw` keyword reserved in DSL (parser rejects: "reserved for v2+")
- Structured `shape:` block ships in v1.0 for custom visual needs:
```
shape:
  type roundRect
  x 3in
  y 2in
  width 4in
  height 3in
  rotation 27deg
  fill gradient(brand.primary, brand.accent1)
  text "API Gateway"
  alt "Rounded rectangle representing API Gateway"
```
- shape DSL is validated, accessible, multi-format renderable
- Gated raw blocks (v2+) only if 3+ enterprise customers demonstrate need shape DSL can't cover
- Research: R13 (planning/raw-escape-hatch-research.md)

## Q8: Inline formatting surface — LOCKED
**Decision:** Core 7 + superscript + subscript + strikethrough + highlight in v1.0.

v1.0 inline formatting:
| Syntax | Effect |
|--------|--------|
| `**bold**` | Bold |
| `_italic_` | Italic |
| `` `code` `` | Code span |
| `[text](url)` | Hyperlink |
| `$math$` / `$$display$$` | Math (mode switch) |
| `{{ footnote("...") }}` | Footnote |
| `{{ ref("id") }}` / `{{ figref(n) }}` | Cross-reference |
| `^sup^` | Superscript |
| `~sub~` | Subscript |
| `~~del~~` | Strikethrough |
| `==highlight==` | Highlighted/marked text |

v2: `{{ color("red", "text") }}`, `{{ kbd("Ctrl+C") }}`, emoji shortcodes.
Never: underline (anti-pattern), inline font-size, inline font-face.

## Q9: Variable interpolation rules — LOCKED
**Decision:** {{ }} works everywhere except slide type keywords and brand paths. @include paths support interpolation.

| Context | {{ }} works? |
|---------|-------------|
| String fields (title, bullets, notes, report, alt, label) | ✅ |
| Numeric fields (columns, width) | ✅ (coerced to number) |
| @data source paths | ✅ |
| @include paths | ✅ (vars resolved before includes) |
| color: fields | ✅ (must resolve to valid color name) |
| Brand/template paths | ❌ (loaded before evaluation) |
| Slide type keywords | ❌ (structural, not dynamic) |
| Nested {{ {{ }} }} | ❌ (use dot/bracket: {{ items[0].name }}) |

Architect note: @include path interpolation creates evaluation order dependency — vars must be resolved before includes.

## Q10: set rules — LOCKED
**Decision:** set is universal — works on all slide types (31), all section types, supports {{ }} interpolation, references brand.* values.

Syntax:
```
set chart:
  color brand.primary
  legend true

set severity_cards:
  color_high brand.danger
  label_format "{{ severity | upper }}: {{ title }}"

set methodology:
  framework "NIST CSF v2.0"
```

## Q11: Variants — LOCKED
**Decision:** Full multiple inheritance in v1.0.

- `inherits: [parent1, parent2, ...]` for variant composition
- Merge order: deck vars → parents left-to-right (later overrides earlier) → variant's own vars
- Merge semantics (from Q1): last-wins scalars, replace lists, deep-merge maps
- No diamond problem (flat namespace, no nested inheritance chains)
- Per-variant brand: and brand_overlay: supported

Syntax:
```
variants:
  exec:
    exclude_tags: [technical]
    vars: { detail_level: "summary" }
  external:
    exclude_tags: [internal]
    vars: { redact_financials: true }
  na:
    vars: { currency: "USD" }
  exec-external-na:
    inherits: [exec, external, na]
    vars:
      custom_footer "NA Executive"
```

## Q12: @include semantics depth — LOCKED
**Decision:** Include EVERYTHING — complete slides, set rules, vars, aliases, AND fragment includes (content inside slide fields).

- Top-level includes: slides, set, vars, aliases
- Fragment includes: @include inside notes:, report:, detail:, or any field block
- Cycle detection required
- Resolution: relative to source file → --include-path
- {{ }} interpolation in paths (Q9)
- Hot-reload: included file changes trigger re-parse

Syntax:
```
# Top-level:
@include "shared/disclaimer-slide.sf"
@include "defaults/severity-colors.sf"

# Fragment (inside a slide field):
slide severity_cards:
  notes:
    @include "fragments/standard-notes.sf"
  report:
    @include "fragments/methodology.sf"
```

Architect note: cross-file source-span tracking required. Parser needs file-aware span type.

## Q13: Block comments / doc-comments — LOCKED
**Decision:** Three comment styles, each with distinct purpose.

| Style | Syntax | Purpose | Flows to output? |
|-------|--------|---------|-----------------|
| Line comment | `# text` | Annotations, notes to self | ❌ |
| Block comment | `### ... ###` | Hide/disable slides during iteration | ❌ |
| Doc-comment | `#! text` | Documentation → LSP hover, slideforge doc | ✅ To docs/LSP only |

Doc-comments attach to the nearest subsequent AST node.

## Q14: Conditional rendering — LOCKED
**Decision:** @if / @elif / @else at ALL scopes + inline {{ if x: a else: b }} expressions.

Scopes:
- Slide-level (include/exclude entire slides)
- Element-level (inside @for loops)
- Field-level (inline expression)
- Section-level (conditional document sections)

Syntax:
```
@if severity == "critical":
  color red
@elif severity == "high":
  color orange
@else:
  color green
```

## Q15: Asset references — LOCKED
**Decision:** Both inline paths + optional assets: registry.

- Inline relative paths always work
- Optional `assets:` block for named references (DRY)
- `{{ assets.name }}` for referencing named assets
- Resolution: relative to .sf file

Syntax:
```
assets:
  logo "assets/1898-logo.png"
  team "assets/team-2026.jpg"

slide title:
  logo {{ assets.logo }}

slide bio:
  photo "assets/jmagady.jpg"    # inline also works
```
