---
title: "Q2 Decision — Custom / Composed Slide Types"
date: 2026-05-24
status: LOCKED
decided_by: human
---

# Q2 Decision: Custom / Composed Slide Types

## Decision Summary
v1.0 ships with A+B+C+D: 31 built-in slide types (23 seed + 8 new) + parametric aliases + reserved component syntax for v2 + composition via @include and sequential slides.

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

## Architect Note
The `diagram` slide type requires a Mermaid rendering engine in v1.0. This is delivered via the DiagramRenderer plugin surface (see Q3 decision). Spike S14 tracks this.
