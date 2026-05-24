---
title: "Q2 Decision — Custom / Composed Slide Types"
date: 2026-05-24
status: LOCKED
decided_by: human
---

# Q2 Decision: Custom / Composed Slide Types

## Decision Summary
v1.0 ships with A+B+C+D: 31 built-in slide types (23 seed + 8 new) + parametric aliases + reserved component syntax for v2 + composition via @include and sequential slides.

## Research Context

### R3 — DSL Competitor User-Pain (key finding)
Users do NOT ask for "let me define new types." They ask for "please add columns/grids as a first-class built-in type." Slidev's custom layout system is underused, poorly documented, and creates ecosystem fragmentation across projects. Marp users LEAVE because of Marp's limited type set — but the solution they want is MORE BUILT-IN TYPES, not an extension API.

**Design principle:** Every type added to the core is better than every type left to user configuration.

### R7 — Composition/Mixins Prior Art (key verdict)
"The 23 opinionated slide types ARE the component library. User-defined components would dilute the opinionated stance that is the product's primary value." R7 studied 15 composition ecosystems and found that user-defined component systems create fragmentation (Company A's `incident-card` ≠ Company B's) without proportional benefit for a domain-specific DSL.

### R1 — Python Reference Deep-Read
The Python reference's 23 slide types cover 11 distinct visual patterns in a real 25-slide leadership deck. The remaining 12 types are available but unused — the type vocabulary is already generous. No evidence of users needing custom types in the Python tool's history.

### R10 — PowerPoint Element Taxonomy (gap analysis)
R10 identified slide types the seed DOESN'T have that users would want: toc, agenda, chart, diagram, quote, grid, bio, team. These should be BUILT-IN types, not left to user definition. This directly informed the 8 additions.

### Fragmentation Risk
Slidev's custom layout system has created ecosystem fragmentation — every project has its own layouts, none are portable. slideforge's value is the SHARED VOCABULARY: when you say `slide severity_cards:`, every slideforge user knows exactly what that looks like. User-defined types would break this shared understanding.

## Layer A: 31 Built-in Slide Types (v1.0)

### Original 23 (from seed)
title, content, two_column, content_stat, stat_callout, stats_summary, highlight, highlight_boxes, split_contrast, card_rows, severity_cards, numbered_actions, vertical_timeline, horizontal_timeline, enhanced_table, table, status, progress_bar, metric_tree, formula, weighted_composite, end, key_metrics (alias for stat_callout)

### 8 New Types (added during Q2 discussion)
| Type | Description | Complexity |
|------|-------------|------------|
| chart | Full-slide data chart from @data. SVG via plotters. Bar, line, pie, scatter, area, histogram, stacked. | Medium |
| toc | Auto-generated table of contents. Reads all slide titles, renders as navigation index. | Low |
| agenda | Meeting agenda with items, times, owners. Structured list with columns. | Low |
| quote | Full-slide testimonial/pull-quote. Styled quote text + attribution + optional photo. | Low |
| grid | Multi-column/multi-row flexible layout. User defines cells with arbitrary content. | High |
| bio | Speaker/team member profile. Photo + name + title + bio text. | Medium |
| diagram | Full-slide diagram rendering. Mermaid syntax rendered as SVG. Ships in v1.0 via DiagramRenderer plugin. | High |
| team | Team roster grid. Multiple bio cards in a grid layout. | Medium |

## Layer B: Parametric Aliases (v1.0)

Syntax:
```
alias incident_card = severity_cards:
  set:
    color_high red
    color_medium orange
    color_low green
    label_format "{{ severity | upper }}: {{ title }}"
```

- An alias is a NAMED preset — same underlying type, custom name + defaults
- Can't add NEW fields — only preset existing ones
- Resolved at parse time (alias → underlying type + merged set-rule defaults)
- One new keyword: `alias`
- Precedence: slide-level > alias set > deck-level set > defaults/ dir > brand template

Aliases also work for document section types:
```
alias compliance_methodology = methodology:
  set:
    heading "Compliance Assessment Methodology"
```

## What Set Rules + Aliases Cover (80%)

Most "custom type" requests are actually requests for customized DEFAULTS on existing types:

| User want | Solution in v1.0 |
|-----------|-----------------|
| "Our severity cards always use red/orange/green" | `set severity_cards: color_high red` |
| "We call them 'incident cards' not 'severity cards'" | `alias incident_card = severity_cards:` |
| "Our stat callouts always use blue accent" | `set stat_callout: accent blue` |
| "Every deck starts with our standard disclaimer" | `@include "shared/disclaimer.sf"` |
| "Different color scheme per client" | `variants:` with per-variant `vars:` |

## What Set Rules DON'T Cover (20% — v2 territory)

| Use case | Why set/aliases can't solve it | v2 solution |
|----------|-------------------------------|-------------|
| Compound slides (stats on top + timeline below in ONE slide) | `set` customizes one type; can't combine two types into one slide canvas | `component exec_dashboard: layout: split_horizontal` |
| Domain-specific vocabulary with NEW fields | `alias` can rename and preset, but can't add fields that don't exist on the underlying type | `component soc_alert extends severity_cards: add field: analyst_notes` |
| Cross-org sharing of slide patterns | No package/registry mechanism in v1.0 | Library/package model with installable slide kits |

## Layer C: Reserved Component Syntax (v2)

Reserved keywords (parser recognizes, rejects with "planned for v2"):
- `component` — user-defined compound slide types
- `extends` — type inheritance
- `inherits` — type inheritance (alternate keyword)
- `alias` ships in v1.0 (Layer B)

v2 intended syntax (normative for forward-compatibility):
```
component exec_dashboard:
  layout: split_horizontal
  top: stat_callout
  bottom: horizontal_timeline
  fields:
    kpi_data: list
    timeline_data: list
```

## Layer D: Composition via @include + Sequential Slides (v1.0)

Already locked in Q1. Users compose complex layouts as slide sequences:
```
@include "shared/kpi-header.sf"
slide incident_card:
  title "Current Incidents"
  card: ...
@include "shared/timeline-footer.sf"
```

## Progressive Skill-Level Model

| User level | What they use | Complexity |
|-----------|---------------|------------|
| PM writing a quick deck | `slide content:`, `slide title:` — raw built-in types | Zero |
| Analyst customizing defaults | `set severity_cards: color_high red` — deck-level presets | Low |
| Team standardizing vocabulary | `alias incident_card = severity_cards:` — named company patterns | Low |
| Power user composing layouts | `@include` + sequential slides — multi-slide compositions | Medium |
| v2 power user (future) | `component exec_dashboard:` — compound types with layout composition | High |

## DSL Syntax Proposals for the 8 New Types

### slide chart:
Full-slide data chart rendered via ChartRenderer plugin (plotters in v1.0).
```
slide chart:
  title "ARR Trend — 12 Months"
  type bar                          # bar, line, pie, scatter, area, histogram, stacked
  data {{ kpis.monthly }}           # data source from @data
  x "month"                        # x-axis field
  y "arr"                           # y-axis field
  color brand.primary               # brand color reference
  alt "Bar chart showing ARR trend over 12 months, growing from $1.2M to $2.4M"
  notes """
    Point out the Q3 dip — it's the churn event from the Acme loss.
  """
  report """
    Annual recurring revenue grew from $1.2M to $2.4M over the
    trailing twelve months, representing 100% year-over-year growth.
    A temporary decline in Q3 2025 (visible in the chart) corresponds
    to the Acme Corp contract loss, which was subsequently recovered.
  """
```
Note: This is DISTINCT from inline `{{ chart.bar(...) }}` inside report/detail blocks. `slide chart:` is a FULL-SLIDE chart. `{{ chart.bar() }}` is an inline chart inside prose.

### slide toc:
Auto-generated table of contents. Reads all slide titles from the deck.
```
slide toc:
  title "Agenda"
  style numbered                    # numbered, bulleted, or grid
  exclude_types: [title, end, toc]  # don't include these in the TOC
  color blue
```
Auto-populates from slide titles. No manual bullet authoring needed.

### slide agenda:
Meeting agenda with structured items.
```
slide agenda:
  title "Meeting Agenda"
  item:
    time "10:00 - 10:15"
    topic "Opening & Context Setting"
    owner "J. Magady"
    status done                     # done, active, upcoming
  item:
    time "10:15 - 10:45"
    topic "Incident Review"
    owner "K. Chen"
    status active
  item:
    time "10:45 - 11:00"
    topic "Action Items & Next Steps"
    owner "All"
    status upcoming
```

### slide quote:
Full-slide testimonial or pull-quote.
```
slide quote:
  text "The detection-to-containment time went from 4 hours to 23 minutes. That's not incremental improvement — that's a paradigm shift."
  attribution "Sarah Chen, CISO"
  role "Acme Corp"
  photo "assets/sarah-chen.jpg"     # optional
  color blue
  alt "Quote from Sarah Chen, CISO of Acme Corp"
```

### slide grid:
Flexible multi-column/multi-row layout. R3's #1 differentiator vs. Marp/Slidev.
```
slide grid:
  title "Comparison Matrix"
  columns 3
  rows 2
  cell [1,1]:
    heading "Option A"
    content "Low cost, limited features"
    color blue
  cell [1,2]:
    heading "Option B"
    content "Medium cost, full features"
    color green
  cell [1,3]:
    heading "Option C"
    content "High cost, enterprise features"
    color purple
  cell [2,1]:
    stat: value "$$12K" label "Annual"
  cell [2,2]:
    stat: value "$28K" label "Annual"
  cell [2,3]:
    stat: value "$45K" label "Annual"
```
The layout engine divides the slide canvas into a grid and renders each cell as a mini-content area. Cells can contain text, stats, bullets, or images.

### slide bio:
Speaker or team member profile.
```
slide bio:
  name "Joshua Magady"
  title "Director, Security Operations"
  photo "assets/jmagady.jpg"
  alt "Professional headshot of Joshua Magady"
  bio """
    15 years in cybersecurity. Led SOC operations for Fortune 500
    clients. Author of the 1898 & Co. incident response framework.
  """
  contact:
    email "josh.magady@1898.com"
    linkedin "linkedin.com/in/jmagady"
  color blue
```

### slide diagram:
Full-slide diagram from Mermaid syntax, rendered via DiagramRenderer plugin.
```
slide diagram:
  title "Incident Response Flow"
  lang mermaid                      # mermaid, graphviz (v2), d2 (v2)
  source """
    graph TD
      A[Detection] -->|23 min| B[Triage]
      B -->|15 min| C[Containment]
      C -->|4 hrs| D[Eradication]
      D -->|24 hrs| E[Recovery]
      E --> F[Lessons Learned]
  """
  alt "Flowchart showing 5-phase incident response process with timing"
  color blue
```

### slide team:
Team roster — multiple bio cards in a grid.
```
slide team:
  title "Incident Response Team"
  columns 3                         # 2, 3, or 4 columns
  member:
    name "J. Magady"
    title "Incident Commander"
    photo "assets/jmagady.jpg"
    alt "Joshua Magady headshot"
  member:
    name "K. Chen"
    title "Lead Analyst"
    photo "assets/kchen.jpg"
    alt "Karen Chen headshot"
  member:
    name "R. Patel"
    title "Forensics Lead"
    photo "assets/rpatel.jpg"
    alt "Raj Patel headshot"
```

## New Slide Types → Document Section Mapping

How each new type renders in .docx output:

| Slide type | .pptx rendering | .docx rendering |
|-----------|----------------|----------------|
| chart | Full-slide SVG chart | Inline figure with caption + alt text |
| toc | Navigation index slide | Becomes the actual document TOC (auto-generated) |
| agenda | Visual agenda with status badges | Meeting agenda table |
| quote | Full-bleed quote with attribution | Block quote with attribution |
| grid | Visual grid with cells | Comparison table or multi-column section |
| bio | Photo + text profile | Author/contributor bio paragraph with inline photo |
| diagram | Full-slide Mermaid SVG | Inline figure with caption + alt text |
| team | Grid of profile cards | Contributors table or bio list |

## Chart Slide Type vs. Inline Chart Function

slideforge has TWO chart surfaces:

| Surface | Syntax | Where it renders | Use case |
|---------|--------|-----------------|----------|
| `slide chart:` | Dedicated slide block | Full slide canvas | "This slide IS a chart" |
| `{{ chart.bar(...) }}` | Inline function in report/detail | Inside prose/document | "This paragraph includes a chart" |

Both use the same ChartRenderer plugin (plotters). Both produce SVG. The difference is rendering context: full-slide vs. inline figure.

## Architect Notes

### Mermaid Rendering Engine (Spike S14)
The `diagram` slide type requires a Mermaid rendering engine in v1.0. This is delivered via the DiagramRenderer plugin surface (see Q3 decision). Options for the architect to evaluate:

| Option | Approach | Single-binary? | Quality | Complexity |
|--------|----------|---------------|---------|------------|
| A | mermaid-rs (pure Rust parser + SVG renderer) | Yes | Unknown — maturity TBD | Medium |
| B | Bundled mermaid WASM (compile mermaid.js to WASM, embed) | Yes | High (same as mermaid.js) | High |
| C | mermaid CLI (Node.js) as optional external tool | No (requires Node) | High | Low |
| D | Headless browser (Playwright/Chrome) rendering | No (requires browser) | High | Medium |

Spike S14 must resolve this. The single-binary constraint from Q1 (R3: "no runtime dependencies") strongly favors Option A or B.

### Grid Layout Engine
The `grid` slide type requires the layout engine to:
1. Divide the slide canvas into an N×M grid
2. Render each cell as an independent content area
3. Handle cell spanning (future: `cell [1, 1:2]:` spans two columns)
4. Respect brand margins and padding

This is the highest-complexity new type. The architect should design the grid layout as a general-purpose sub-layout that other types can reuse (e.g., `team` uses grid internally).

### Chart Slide vs. Chart Function Unification
Both `slide chart:` and `{{ chart.bar(...) }}` must go through the same ChartRenderer plugin. The architect should ensure the ChartSpec IR type serves both contexts. The difference is only in the rendering target (full slide vs. inline figure).
