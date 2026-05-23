---
title: Presentation System Reference
version: 1.0
created: 2026-03-30
owner: Joshua Magady
---

# Presentation System Reference

Complete reference for building 1898-branded incident brief presentations. This document is self-contained: everything you need to build, modify, or extend the system is here.

---

## Table of Contents

1. [Quick Start](#1-quick-start)
2. [Architecture](#2-architecture)
3. [Slide Types Reference](#3-slide-types-reference)
4. [Choosing the Right Slide Type](#4-choosing-the-right-slide-type)
5. [Design Rules](#5-design-rules)
6. [Data Format Reference](#6-data-format-reference)
7. [Shared Features](#7-shared-features)
8. [Content Validation](#8-content-validation)
9. [Adding New Slide Types](#9-adding-new-slide-types)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Quick Start

### Build an existing presentation

```bash
cd /path/to/incident_response
uv run python scripts/build-incident-brief.py scripts/incident_data/inc_2026_0320.py
```

### Create a new presentation

1. Copy the template: `cp scripts/incident_data/_template.py scripts/incident_data/inc_YYYY_NNNN.py`
2. Edit `METADATA` (incident ID, title, output path)
3. Edit `SLIDES` list (add slides using the types documented below)
4. Build: `uv run python scripts/build-incident-brief.py scripts/incident_data/inc_YYYY_NNNN.py`
5. Output: `.pptx` file at the path specified in `METADATA["output"]`

### Dependencies

```bash
uv add python-pptx lxml   # Already in pyproject.toml
```

---

## 2. Architecture

```
scripts/
├── build-incident-brief.py          # Builder engine (all slide types, validation, CLI)
└── incident_data/
    ├── _template.py                  # Copy this for each new incident
    └── inc_2026_0320.py             # Example: Trivy incident data

templates/1898-pptx/
└── 1898-Presentation-Template-V3.0-2026.pptx   # Brand template (do not modify)
```

### How it works

1. The builder loads the 1898 PowerPoint template
2. Removes any default slides from the template
3. Reads the `SLIDES` list from your data module
4. For each slide, dispatches to the correct builder function based on `"type"`
5. Adds speaker notes from `"talk_track"` if provided
6. Runs content validation and warns about potential issues
7. Saves the final `.pptx`

### Data module structure

Every data module must define two things:

```python
METADATA = {
    "incident_id": "INC-YYYY-NNNN",
    "title": "Incident Title",
    "date": "YYYY-MM-DD",
    "author": "Author Name",
    "output": Path("path/to/output.pptx"),
}

SLIDES = [
    {"type": "title", "title": "...", ...},
    {"type": "content", "title": "...", "bullets": [...], ...},
    # ...
]
```

---

## 3. Slide Types Reference

### 3.1 `title` — Section Divider

Blue or purple full-background slide for section breaks and the opening slide.

**When to use:** Opening slide, section dividers between major sections (Situation, Response, Lessons Learned).

**Data:**
```python
{
    "type": "title",
    "color": "blue",              # "blue" or "purple"
    "title": "Section Title",
    "subtitle": "Optional subtitle text",
    "talk_track": "...",
}
```

---

### 3.2 `content` — Title + Bullets

Standard content slide with a title and bullet points.

**When to use:** Simple lists of 4-6 items. Use sparingly; prefer more visual types when possible. Never put two `content` slides back to back.

**Data:**
```python
{
    "type": "content",
    "title": "Slide Title",
    "bullets": [
        "Regular bullet text",
        "**Bold blue header**",           # Starts with ** = bold blue header
        {"text": "Custom", "bold": True}, # Dict for full control
    ],
    "takeaway": "Optional key takeaway bar at bottom",
    "talk_track": "...",
}
```

**Limits:** Max 6 bullets. Max ~85 characters per bullet at 18pt.

---

### 3.3 `two_column` — Two Bullet Columns

Two side-by-side bullet columns with optional takeaway.

**When to use:** Comparing two related lists (rotation vs hardening, before vs after). Consider `card_rows` for a more polished look.

**Data:**
```python
{
    "type": "two_column",
    "title": "Slide Title",
    "left": ["**Header**", "Bullet 1", "Bullet 2"],
    "right": ["**Header**", "Bullet 1", "Bullet 2"],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

---

### 3.4 `content_stat` — Blocks + Stat Card

Content on the left (header/description blocks or bullets) with a stat card anchored on the right. The most versatile layout for combining narrative with a key number.

**When to use:** When a slide has narrative content plus one or more key metrics that deserve visual weight. Situation overviews, containment summaries.

**Data (blocks + single stat):**
```python
{
    "type": "content_stat",
    "title": "Slide Title",
    "blocks": [
        {"header": "Bold Title", "description": "Supporting text"},
        {"header": "Another Section", "description": "More detail"},
    ],
    "stat": {
        "value": "10,000+",
        "label": "organizations affected",
        "color": "blue",
    },
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

**Data (blocks + stacked stat rows):**
```python
{
    "type": "content_stat",
    "title": "Slide Title",
    "blocks": [...],
    "stat_rows": [
        {"value": "38min", "label": "to first disable"},
        {"value": "90min", "label": "all 18 repos disabled"},
        {"value": "5hr", "label": "C2 blocked on firewalls"},
        {"value": "\u2713", "label": "evidence archived pre-fix"},
    ],
    "stat_color": "blue",
    "stat_accent": "orange",       # Optional accent bar on stat card
    "takeaway": "...",
    "talk_track": "...",
}
```

**Also accepts `bullets` instead of `blocks`** for a simpler left side.

---

### 3.5 `stat_callout` — Stat Cards

2-6 stat cards displayed prominently. Three visual styles available.

**When to use:** Key metrics that need visual weight. Exposure numbers, investigation scope, timing stats.

**Styles:**

| Style | Look | Best For |
|-------|------|----------|
| `"cards"` (default) | Colored card backgrounds, white text, optional orange accent | 2-4 high-impact stats |
| `"light"` | White/light cards, colored left border, optional checkmark/x | Dashboard-style, 4-6 items |
| `"plain"` | Colored numbers on white, no card background | Minimal, when cards feel heavy |

**Data:**
```python
{
    "type": "stat_callout",
    "title": "Slide Title",
    "style": "cards",              # "cards", "light", or "plain"
    "accent": "orange",            # Slide-level accent for "cards" style (optional)
    "stats": [
        {
            "value": "26",
            "label": "Exfil processes",
            "sublabel": "against 0 baseline",  # Optional
            "color": "blue",
            "accent": "orange",     # Per-stat accent override (optional)
            "status": "pass",       # "light" style only: "pass" (checkmark) or "fail" (x)
            "icon": "check",        # "light" style only: "check" or "arrow" for header layout
        },
    ],
    "context": "Optional centered text below stats",
    "talk_track": "...",
}
```

**Limits:** Max 6 stats. 4 or fewer render in a single row. 5-6 render in a 3-column grid.

---

### 3.6 `stats_summary` — Stats Top + Summary Band Bottom

Investigation effort cards on top, zero-result summary band on bottom. Tells the story: "we looked at this much and found zero."

**When to use:** Verification results. Investigation scope (top) leading to clean findings (bottom).

**Data:**
```python
{
    "type": "stats_summary",
    "title": "What We Verified",
    "top_stats": [
        {"value": "1.4M", "label": "CloudTrail events", "sublabel": "Zero unauthorized API calls", "color": "blue"},
        {"value": "55", "label": "EKS nodes scanned", "sublabel": "Zero persistence artifacts", "color": "blue"},
    ],
    "top_accent": "green",         # Optional accent on top cards
    "summary": {
        "header": "Zero Indicators of Compromise",
        "color": "green",
        "bg_color": "light_green",  # Optional (auto-derived if omitted)
        "items": [
            {"value": "0", "label": "tpcp-docs repos", "sublabel": "No fallback exfiltration"},
        ],
    },
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

---

### 3.7 `highlight` — Callout Box + Supporting Bullets

A single key statement in a blue highlight box with supporting bullets underneath.

**When to use:** One critical finding or conclusion that deserves its own slide. "Why only 3 repos were hit." "What we believe."

**Data:**
```python
{
    "type": "highlight",
    "title": "Slide Title",
    "highlight": "The key statement that gets the blue box treatment.",
    "supporting": [
        "Supporting bullet 1",
        "Supporting bullet 2",
    ],
    "talk_track": "...",
}
```

---

### 3.8 `highlight_boxes` — Stacked Full-Width Bands

Two full-width horizontal bands, each with content on the left and a stat on the right. Supports icons, header counts, and accent colors.

**When to use:** Comparing two states or categories. "What Held vs What Worked." "Resolved vs In Progress." Two parallel stories on one slide.

**Data:**
```python
{
    "type": "highlight_boxes",
    "title": "Slide Title",
    "boxes": [
        {
            "header": "Resolved",
            "header_count": "4 of 6",          # Optional count above stat
            "items": ["Item 1", "Item 2"],
            "item_icon": "check",               # "check", "arrow", or "" (optional)
            "bg_color": "blue",                 # Box background
            "accent_color": "orange",           # Optional top accent line
            "text_color": "white",
            "stat_value": "4 / 6",
            "stat_label": "areas fully resolved",
            "stat_color": "green",
        },
        {
            "header": "In Progress",
            "items": ["Item 1"],
            "item_icon": "arrow",
            "bg_color": "light_blue",
            "accent_color": "orange",
            "text_color": "dark_gray",
            "stat_value": "2 weeks",
            "stat_label": "to full restoration",
            "stat_color": "orange",
        },
    ],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

---

### 3.9 `split_contrast` — Two Panels with Mini-Timelines

Two side-by-side panels, each with a header, mini-timeline events, and a stat. For contrasting two parallel narratives.

**When to use:** Our response vs attacker persistence. Before vs after. Speed vs patience. Two stories told simultaneously.

**Data:**
```python
{
    "type": "split_contrast",
    "title": "Slide Title",
    "left_panel": {
        "header": "Our Response",
        "bg_color": "blue",
        "text_color": "white",
        "accent_color": "orange",       # Optional top accent
        "dot_color": "green",
        "events": [
            {"date": "Mar 20 15:35", "text": "Detected and declared SEV-1"},
            {"date": "Mar 20 16:17", "text": "First repo disabled (38 min)"},
        ],
        "stat_value": "48h ahead",
        "stat_label": "of K8s payloads at other victims",
        "stat_color": "white",
    },
    "right_panel": {
        "header": "Attacker Persistence",
        "bg_color": "light_gray",
        "text_color": "dark_gray",
        "accent_color": "red",
        "dot_color": "red",
        "events": [
            {"date": "Mar 1", "text": "Initial breach of Aqua Security"},
            {"date": "Mar 4", "text": "Re-established access"},
        ],
        "stat_value": "18 days",
        "stat_label": "of quiet persistence",
        "stat_color": "red",
    },
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max 6 events per panel.

---

### 3.10 `card_rows` — Two-Column Checklist

Two columns of horizontal card rows with colored left borders. Clean checklist look.

**When to use:** Two parallel lists where each item is a completed action. Credential rotation + hardening. Security items + process items.

**Data:**
```python
{
    "type": "card_rows",
    "title": "Slide Title",
    "left_header": "Rotated (Day 1-2)",
    "left_color": "green",
    "left": ["Item 1", "Item 2", "Item 3"],
    "right_header": "Hardened (Day 3-4)",
    "right_color": "blue",
    "right": ["Item 1", "Item 2", "Item 3"],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

---

### 3.11 `severity_cards` — Risk Register with Badges

Full-width horizontal cards with severity left border and badge. Reads like a risk register or audit finding list.

**When to use:** Gaps/findings with severity ratings. Also works for status items with custom badges (CLEAN, RESOLVED, IN PROGRESS, INVESTIGATING).

**Data:**
```python
{
    "type": "severity_cards",
    "title": "Slide Title",
    "gaps": [
        {
            "header": "Docker Image Digest Pinning",
            "description": "Version tags repointed at registry level.",
            "severity": "HIGH",      # Badge text (any string)
            "color": "red",          # Badge + border color
        },
        {
            "header": "IR Documentation",
            "description": "Runbook now created.",
            "severity": "MEDIUM",
            "color": "orange",
        },
    ],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max ~6 items before cards get too thin.

---

### 3.12 `numbered_actions` — Color-Coded Numbered Cards

Numbered action cards in two columns with category color coding and legend.

**When to use:** Remaining action items. Prioritized to-do lists. Anything with numbered steps across categories.

**Data:**
```python
{
    "type": "numbered_actions",
    "title": "Remaining Actions",
    "columns": 2,                  # Optional. 1, 2 (default), or 3.
    "actions": [
        {"text": "Complete credential rotations", "category": "Security", "color": "orange"},
        {"text": "Deploy runner pod hardening", "category": "Security", "color": "orange"},
        {"text": "Route Docker pulls through ECR", "category": "Infrastructure", "color": "teal"},
        {"text": "Formalize IR process", "category": "Process", "color": "purple"},
    ],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

Legend is auto-derived from categories. Items are distributed column-major (fill column 1 fully, then column 2, etc.) so reading order remains 1 → 2 → 3 …

**Column choice:**
- `columns: 1` — full-width stack. Best for 3-5 short parallel statements where the parallel structure is the message.
- `columns: 2` — default. Best for 4-10 items.
- `columns: 3` — dense. Best for 9-12 items with short text.

---

### 3.13 `vertical_timeline` — Cascade Timeline Cards

Vertical timeline with a connecting line, colored dots, and event cards. Shows cascade/sequence relationships.

**When to use:** Attack campaign timelines. Multi-stage cascading events. Any sequence where order and progression matter.

**Data:**
```python
{
    "type": "vertical_timeline",
    "title": "Broader TeamPCP Campaign",
    "events": [
        {
            "date": "Mar 19",
            "target": "Trivy",
            "detail": "Actions, Docker Hub, GHCR, ECR",   # Optional
            "impact": "10,000+ organizations",
            "color": "blue",
            "badge": "NEAR-MISS",       # Optional badge text
            "badge_color": "orange",    # Optional badge color
        },
    ],
    "footnote": "Optional footnote text",
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max 8 events. Events with badges get a warm-tinted background.

---

### 3.14 `horizontal_timeline` — Alternating Above/Below

Horizontal timeline bar with event cards alternating above and below. Shows compressed timeframe spatially.

**When to use:** When the time compression is the story (8 days of attacks). Works best with 4-6 evenly distributed events. Avoid if events cluster on the same date.

**Data:** Same as `vertical_timeline`.

**Limits:** Max 6 events. Cards alternate above/below automatically.

---

### 3.15 `enhanced_table` — Colored Row Table with Badges

Table with colored row borders, alternating backgrounds, and optional per-row badges.

**When to use:** Tabular data that needs more visual structure than a plain table. Campaign summaries, IOC lists, comparison matrices.

**Data:** Same `events` format as timelines, plus optional `columns` and `col_widths`.

```python
{
    "type": "enhanced_table",
    "title": "Slide Title",
    "events": [...],               # Same format as vertical_timeline
    "columns": ["Date", "Target", "Impact"],  # Optional header override
    "col_widths": [Inches(1.5), Inches(5.5), Inches(4.8)],  # Optional
    "footnote": "...",
    "takeaway": "...",
    "talk_track": "...",
}
```

**Limits:** Max 10 rows.

---

### 3.16 `table` — Standard Data Table

Basic branded table with blue header row and alternating row colors.

**When to use:** Simple tabular data without the need for colored borders or badges.

**Data:**
```python
{
    "type": "table",
    "title": "Slide Title",
    "table": [
        ["Column 1", "Column 2", "Column 3"],  # Header row
        ["Data 1", "Data 2", "Data 3"],
    ],
    "col_widths": [Inches(3.0), Inches(5.0), Inches(3.8)],  # Optional
    "footnote": "Optional footnote",
    "talk_track": "...",
}
```

---

### 3.17 `status` — Status Dashboard

Label + colored badge pairs with optional summary stat cards below.

**When to use:** Quick status overview when the badge format is the clearest way to communicate state. Consider `highlight_boxes` for a more visual treatment of the same data.

**Data:**
```python
{
    "type": "status",
    "title": "Current Status",
    "statuses": [
        {"label": "Attacker Access", "value": "Contained: all credentials rotated", "color": "green"},
        {"label": "Current Sprint", "value": "Schedule impact: reduced capacity", "color": "orange"},
    ],
    "summary_stats": [                   # Optional stat cards below badges
        {"value": "4 of 6", "label": "areas resolved", "color": "green"},
        {"value": "2 weeks", "label": "to full restoration", "color": "orange"},
    ],
    "takeaway": "Optional takeaway bar",
    "talk_track": "...",
}
```

---

### 3.18 `progress_bar` — Horizontal Progress Bar

Progress bar with resolved items above and in-progress items below. Auto-calculates fill percentage.

**When to use:** When progress toward completion is the story and items are genuinely sequential tasks. Don't use for mixed findings/states that aren't a linear process.

**Data:**
```python
{
    "type": "progress_bar",
    "title": "Slide Title",
    "progress_label": "67% Resolved (4 of 6 areas)",
    "resolved": [
        {"label": "Attacker Access", "value": "Contained: all credentials rotated"},
    ],
    "in_progress": [
        {"label": "Current Sprint", "value": "Schedule impact: reduced capacity"},
    ],
    "takeaway": "...",
    "talk_track": "...",
}
```

---

### 3.19 `metric_tree` — Hierarchical Metric Tree

Root box at top with vertical connector to a horizontal busbar, then N colored category columns below with vertical drops, each containing a header card and a list of leaf metric chips with matching colored left borders.

**When to use:** Strategic metric trees, organizational rollups, any "root → categories → leaves" structure where the hierarchy itself is the message. Two-level only (root → N categories → leaves under each).

**Data:**
```python
{
    "type": "metric_tree",
    "title": "Executive Metric Tree",
    "root": {
        "label": "Retained Protected Revenue at Target Margin",
        "color": "blue",
    },
    "categories": [
        {
            "header": "Growth Health",
            "color": "blue",
            "items": ["ARR", "MRR", "Qualified Pipeline", "Win Rate", "..."],
        },
        # ... up to 5 categories
    ],
    "footnote": "Optional small-print caption shown above the takeaway bar",
    "takeaway": "Optional bottom takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max 5 category columns. Max 6-8 leaf chips per column (auto-capped based on available vertical space, which shrinks when a takeaway and/or footnote is present).

**Footnote/takeaway interaction:** The builder reserves a stacked bottom zone so the footnote sits cleanly above the takeaway bar with breathing room. Chips are auto-capped so they don't push into the footnote zone.

---

### 3.20 `formula` — Equation with Term Breakdown

Large central equation (stacked on two lines by default) with one definition card per multiplicand below. Color-codes each term to its definition card via a top accent stripe and matching term-name color.

**When to use:** When a multiplicative or weighted formula *is* the message. North Star formulas, scoring composites, ROI breakdowns.

**Data:**
```python
{
    "type": "formula",
    "title": "North Star Formula",
    "result": "Retained Protected Revenue at Target Margin",
    "operator": "×",                        # Optional. Multiplier symbol between terms. Default "×"
    "stack": True,                          # Optional. True (default): result on line 1, formula on line 2.
                                            # False: render inline on one line (may wrap if long).
    "accent": "orange",                     # Optional. Adds a colored divider line between equation and cards
    "terms": [
        {
            "name": "ARR",
            "definition": "Annual recurring revenue from retained MSS clients",
            "color": "blue",
        },
        {
            "name": "Gross Margin %",
            "definition": "Profitability of the service after delivery cost",
            "color": "teal",
        },
        {
            "name": "Service Health Score",
            "definition": "Composite of SLA, detection, onboarding, response, satisfaction, platform",
            "color": "purple",
        },
    ],
    "takeaway": "Optional bottom takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max 4 term cards comfortably (auto-fits via card width). Beyond 4 terms, card text gets cramped and formula equation likely wraps at 22pt even when stacked.

**Stack vs inline:**
- `stack=True` (default): result name on line 1, `= term × term × term` on line 2. Reads as a deliberate equation layout. Use this for any equation where the result name plus the formula would exceed one line at 22pt centered.
- `stack=False`: single-line `result = term × term × term`. Use only when the entire equation comfortably fits one line.

---

### 3.21 `weighted_composite` — Stacked Bar with Legend

Horizontal stacked bar segmented by weight percentages, with a legend below mapping each color to its component name and weight. Optional composite label above the bar and explanatory context below the legend.

**When to use:** When a composite score or weighted index is the message and you want to show "this is what 100% looks like." Service health scores, risk indexes, weighted KPIs.

**Data:**
```python
{
    "type": "weighted_composite",
    "title": "Service Health Score: What Makes Up 100%",
    "composite_label": "Service Health Score Components",   # Optional label above bar
    "components": [
        {"name": "SLA Performance", "weight": 25, "color": "blue"},
        {"name": "Detection Quality", "weight": 20, "color": "teal"},
        {"name": "Onboarding Health", "weight": 20, "color": "purple"},
        {"name": "Escalation Quality", "weight": 15, "color": "orange"},
        {"name": "Satisfaction / Renewal Risk", "weight": 10, "color": "green"},
        {"name": "Platform Reliability", "weight": 10, "color": "red"},
    ],
    "context": "Optional explanatory paragraph centered below the legend",
    "takeaway": "Optional bottom takeaway bar",
    "talk_track": "...",
}
```

**Limits:** Max 8 components. Weights should sum to 100; the builder normalizes if they don't.

**Legend layout (auto):**
- 1–4 components: one row, all components
- 5–6 components: 3 per row (so row 2 stays full and columns are wider, ~3.8" each, preventing label wrap)
- 7–8 components: 4 per row

**Inline percentage labels:** Segments wider than ~0.85" get a white `X%` label centered inside. Narrower segments rely on the legend.

---

### 3.22 `end` — Closing Slide

Blue, purple, or white closing slide. No content.

**Data:**
```python
{
    "type": "end",
    "color": "blue",    # "blue", "purple", or "white"
}
```

---

### 3.23 Aliases

| Type Name | Maps To | Notes |
|-----------|---------|-------|
| `key_metrics` | `stat_callout` | Accepts `metrics` key (mapped to `stats`). Backward compatibility. |

---

## 4. Choosing the Right Slide Type

### By content type

| You have... | Use | Why |
|-------------|-----|-----|
| 2-4 key numbers | `stat_callout` (cards style) | Big numbers with visual weight |
| 5-6 metrics in a grid | `stat_callout` (cards or light style) | Auto-grids into rows |
| Investigation effort + clean result | `stats_summary` | Effort on top, zero-result band below |
| Narrative content + one key number | `content_stat` | Blocks left, stat card right |
| Two parallel narratives with timelines | `split_contrast` | Side-by-side panels with mini-timelines |
| Two states to compare (done vs not done) | `highlight_boxes` | Stacked full-width bands |
| A cascade of sequential events | `vertical_timeline` | Timeline line with cascade cards |
| A compressed timeframe to show spatially | `horizontal_timeline` | Events along a time bar |
| Tabular data with visual hierarchy | `enhanced_table` | Colored borders and badges per row |
| Gaps/findings with severity ratings | `severity_cards` | Risk register with severity badges |
| Numbered action items by category | `numbered_actions` | Color-coded numbered cards |
| Two parallel checklists | `card_rows` | Horizontal cards with colored borders |
| One critical statement + support | `highlight` | Blue callout box + bullets below |
| Simple bullet list (last resort) | `content` | Only when nothing else fits |
| Quick status overview | `status` | Label + badge pairs |
| Progress toward completion | `progress_bar` | Only for genuinely sequential tasks |
| Root → categories → leaf metrics hierarchy | `metric_tree` | Strategic metric trees, org rollups |
| Multiplicative or weighted formula | `formula` | Equation up top, term-definition cards below |
| Composite score / weighted index | `weighted_composite` | Stacked bar showing parts of 100% |

### Avoiding visual monotony

- **Never put two `content` slides back to back.** The validator will warn.
- **Alternate visual patterns.** After a content_stat slide, use card_rows or stat_callout next, not another content_stat.
- **Use section dividers** (`title` type) to break the deck into logical sections.
- **Every data-heavy slide should have a takeaway bar** anchoring the bottom with the one sentence the audience should remember.

---

## 5. Design Rules

### Font Sizes (non-negotiable minimums)

| Element | Size | Constant |
|---------|------|----------|
| Slide titles | 24pt | `TITLE_SIZE` |
| Section headers within slides | 18pt | `HEADER_SIZE` |
| Body text / bullets | 18pt | `BODY_SIZE` |
| Table cells / small labels | 16pt | `SMALL_SIZE` |
| Badges / footnotes / sublabels | 14pt | `BADGE_SIZE` |
| Stat card numbers | 36-44pt | `STAT_SIZE` |
| Takeaway bar text | 16pt | `TAKEAWAY_SIZE` |

### Brand Colors

| Name | Hex | Usage |
|------|-----|-------|
| BLUE | #003766 | Primary brand. Titles, headers, card backgrounds |
| ORANGE | #FE6A37 | Accent lines, in-progress indicators |
| PURPLE | #54206F | Section dividers (alternative to blue) |
| GREEN | #339966 | Positive status, resolved, clean findings |
| RED | #CC3333 | Critical findings, attacker-related |
| TEAL | #00968B | Infrastructure category |
| DARK_GRAY | #383B3D | Body text |
| GRAY | #6F6F6F | Sublabels, footnotes |
| WHITE | #FFFFFF | Text on dark backgrounds |
| LIGHT_BLUE | #E8F0F8 | Takeaway bar background |
| LIGHT_GRAY | #B7BABA | Subdued text on dark backgrounds |

**Color strings in data:** Use the lowercase name: `"blue"`, `"orange"`, `"green"`, `"red"`, `"purple"`, `"teal"`, `"gray"`, `"white"`, `"light_blue"`, `"light_gray"`, `"dark_gray"`.

### Layout Constants

| Zone | Position | Size |
|------|----------|------|
| Left margin | 0.67" | -- |
| Title area | (0.67", 0.54") | 11.8" x 0.50" |
| Body start (short title) | (0.67", 1.5") | 11.8" x 5.0" |
| Takeaway bar | (0.67", 6.1") | 11.8" x 0.5" |

### Content Overflow Prevention

| Element | Max Characters | At Font Size | In Width |
|---------|---------------|-------------|----------|
| Full-width bullet | ~85 chars | 18pt | 11.8" |
| content_stat left block desc | ~70 chars | 14pt | 7.0" |
| Two-column bullet | ~50 chars | 18pt | 5.8" |
| Stat card label | ~20 chars | 14pt | per card width |

**Rule of thumb:** If a bullet wraps to a second line, it's too long. Shorten the text. Move detail to the talk track.

---

## 6. Data Format Reference

### Common fields (all slide types)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Slide type name |
| `title` | string | Yes (except `end`) | Slide title |
| `talk_track` | string | No | Speaker notes (natural speech, not script) |
| `takeaway` | string | No | Key takeaway bar text at bottom of slide |

### Bullet formatting (content, two_column)

| Format | Rendering |
|--------|-----------|
| `"Regular text"` | Bullet point with dot |
| `"**Bold text**"` | Bold blue section header (no bullet) |
| `{"text": "...", "bold": True, ...}` | Dict with full formatting control |

### Color values

Pass either a string name (`"blue"`) or an `RGBColor` object. String names are resolved via the color map.

---

## 7. Shared Features

### Takeaway Bar

Available on most slide types. Renders as a light blue rounded rectangle at the bottom of the slide with bold blue text. Use for the one sentence the audience should remember from this slide.

```python
"takeaway": "6 ecosystems compromised in 8 days from a single stolen credential."
```

### Talk Tracks (Speaker Notes)

Added to PowerPoint speaker notes. Visible in Presenter View. Follow the voice guidelines in `docs/voice-guidelines.md`.

```python
"talk_track": (
    "Here's what happened. On March 19th, a threat actor group called TeamPCP..."
)
```

### Event Badges

Available on timeline and table slide types. Any event can have a badge.

```python
{"date": "Mar 24", "target": "liteLLM", ..., "badge": "NEAR-MISS", "badge_color": "orange"}
```

Events with badges automatically get a warm-tinted background.

### Accent Bars

Available on stat_callout (cards style), content_stat, highlight_boxes, and split_contrast. Configurable at the slide level or per-item.

```python
"accent": "orange"       # Slide-level default
"accent": None           # Explicitly disable
```

---

## 8. Content Validation

The builder runs automatic validation before saving and warns about:

| Check | Threshold | Warning |
|-------|-----------|---------|
| Bullet length | >85 characters | "bullets[N] is X chars (max 85)" |
| Bullet count | >6 per content slide | "Consider splitting or using two_column" |
| Consecutive same type | Two `content` slides in a row | "Consider using stat_callout, highlight, or key_metrics for variety" |

Warnings are printed to stdout. They don't prevent the build.

---

## 9. Adding New Slide Types

### Step 1: Write the builder function

Add to `scripts/build-incident-brief.py` before `build_end_slide`:

```python
def build_my_new_slide(prs, slide_data):
    """Description of what this slide type does.

    field1: description
    field2: description
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    # Title
    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    # Your layout logic here...

    # Takeaway (optional)
    takeaway = slide_data.get("takeaway", "")
    if takeaway:
        _add_takeaway(slide, takeaway)

    return slide  # Must return slide for talk_track injection
```

### Step 2: Register in the dispatcher

```python
SLIDE_BUILDERS = {
    ...
    "my_new_type": build_my_new_slide,
    ...
}
```

### Step 3: Build a test deck

Create a standalone test script that builds a deck with your new type to verify rendering before using it in a real presentation.

### Step 4: Update this documentation

Add the new type to Section 3 (Slide Types Reference) and Section 4 (Choosing the Right Slide Type).

### Design principles for new types

- **Clear all placeholders** first. Template placeholders show "Click to add text" if not removed.
- **Return the slide object.** The orchestrator needs it to inject talk_track as speaker notes.
- **Scale dynamically.** Calculate card heights and spacing based on item count, don't hard-code for a specific number.
- **Set max item limits.** Document what happens beyond the limit and cap it in code.
- **Support takeaway bar.** Most slide types should accept an optional `takeaway` field.
- **Use `resolve_color()`** for all color values so string names work.
- **Use shared helpers:** `add_text_box`, `add_rich_text`, `add_rounded_rect`, `add_table`, `_add_takeaway`, `_parse_bullets`.

---

## 10. Troubleshooting

### "Click to add text" appears on slides

Placeholders weren't fully cleared. Ensure `clear_all_placeholders(slide)` is called at the top of every builder function.

### Text overflows / wraps

Shorten the text. Move detail to the talk track. Check the overflow limits in Section 5. If a content_stat block description wraps, it's too long for the 7" left column at 14pt.

### Two slides look the same

Use the validator warnings. Never put two `content` slides back to back. Alternate between visual types. Section 4 has the selection guide.

### Stat_callout accent not showing

Accent requires either a per-stat `"accent": "orange"` field or a slide-level `"accent": "orange"` field. If neither is present, no accent renders. Set to `None` explicitly to suppress.

### Talk track not appearing

Ensure the builder function returns the slide object. If it returns `None`, the orchestrator can't add speaker notes.

### Colors don't match

Use string names from the color map: `"blue"`, `"orange"`, `"green"`, `"red"`, `"purple"`, `"teal"`, `"gray"`, `"white"`, `"light_blue"`, `"light_gray"`, `"dark_gray"`. Custom colors require `RGBColor` objects.
