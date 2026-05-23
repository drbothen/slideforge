---
title: Slideforge DSL Specification
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Slideforge DSL Specification

> Source: §5 (DSL Specification) + reserved keywords + color vocabulary + validation rules
> from PROJECT-SEED.md. Sample DSL files are in sample-dsl-snippets.md.
> Slide type catalog (the 23-type table) is in slide-types-catalog.md.

---

## Section 5: DSL Specification

### Syntax goals

1. **Readable to non-developers.** A program manager should be able to author a slide file without learning Rust.
2. **Composable.** Common patterns (colors, accents, takeaways) work the same across slide types.
3. **Editor-friendly.** Grammar amenable to syntax highlighting, autocomplete, jump-to-definition.
4. **Error-tolerant.** Typos and incomplete sections produce useful errors AND a partial AST.
5. **No mandatory escape hatches into Rust.** The DSL is sufficient by itself.

### Proposed syntax: indented blocks with explicit keywords

Indentation-significant (Python-like / YAML-like), with explicit slide-type keywords. Strings are double-quoted. Lists use repeated keys or bracketed shorthand.

```
metadata:
  incident_id "INC-2026-0320"
  title "Trivy Supply Chain Compromise"
  date "2026-03-19"
  author "Joshua Magady"
  output "incidents/.../INC-2026-0320-Brief.pptx"
  template "templates/1898-pptx/1898-Presentation-Template-V3.0-2026.pptx"

# A title slide
slide title:
  color blue
  title "Trivy Supply Chain Compromise"
  subtitle "INC-2026-0320  |  SEV-1  |  March 19-23, 2026"
  talk_track """
    Here's what happened. On March 19th, TeamPCP compromised the
    Trivy scanner used widely in CI/CD pipelines...
  """

# A content slide
slide content:
  title "Five Things Every MSSP Metric System Must Answer"
  bullets:
    - "Can we sell it?"
    - "Can we onboard it?"
    - bold "Can we detect accurately?"        # inline formatting
    - "Can we operate within SLA?"
    - "Can we do all this profitably?"
  takeaway "If our metrics can't answer all five, we're measuring the wrong things."

# A metric_tree slide (complex nested structure)
slide metric_tree:
  title "Executive Metric Tree"
  root:
    label "Retained Protected Revenue at Target Margin"
    color blue
  category:
    header "Growth Health"
    color blue
    items:
      - "ARR"
      - "MRR"
      - "Qualified Pipeline"
      - "Win Rate"
      - "Avg Deal Size"
      - "Profitable Bookings"
  category:
    header "Retention Health"
    color green
    items:
      - "Logo Retention"
      - "Net Revenue Retention"
      # ...
  footnote "Each leaf rolls up to one dimension."
  takeaway "Four dimensions. One North Star. No single metric carries the business."

# Section divider
slide divider:
  color purple
  title "Executive Health Dimensions"
  subtitle "Four rollups beneath the North Star"

# End slide
slide end:
  color blue
```

### Syntax features

- **Indentation-significant** for block structure (chumsky supports semantic indentation natively)
- **Triple-quoted strings** for multi-line content (talk tracks especially)
- **Repeated keyword blocks** for lists of complex items (multiple `category:` blocks)
- **Bracketed shorthand** for simple lists (`["one", "two", "three"]`)
- **Inline modifiers** (`bold "text"`, `italic "text"`)
- **Color names** as bareword keywords (no quotes needed): `blue`, `orange`, `purple`, `green`, `red`, `teal`, `gray`, `white`, `light_blue`, `light_gray`, `dark_gray`
- **Comments** with `#`
- **No imports** — single-file or multi-file slide projects use a `slideforge.toml` config

### File extension

`.sf` for slide files. `.sf.toml` for project config.

### Reserved keywords

```
metadata, slide, takeaway, talk_track, footnote,
title, content, two_column, content_stat, stat_callout,
stats_summary, highlight, highlight_boxes, split_contrast,
card_rows, severity_cards, numbered_actions, vertical_timeline,
horizontal_timeline, enhanced_table, table, status, progress_bar,
metric_tree, formula, weighted_composite, end, key_metrics,
divider,
true, false, none,
bold, italic, code,
```

### Color vocabulary

Brand colors are named identifiers, not hex codes (hex is reserved for advanced use):

```
blue, orange, purple, green, red, teal, dark_gray, gray, white,
light_blue, light_gray
```

The actual hex values are defined in the brand configuration, not in user DSL code. This is critical: users say "color blue," not "color #003766." The DSL stays brand-portable.

### Slide types (all 23 from Python reference)

Each slide type from the Python implementation must be supported with visual parity. The full slide type catalog and field reference is in:
- `./reference/presentation-system.md` (authoritative for visual behavior)
- `slide-types-catalog.md` (this repo's catalog of the 23 types)

For each slide type, the agent factory should:

1. Read the Python builder function in `./reference/build-incident-brief.py` (e.g., `build_metric_tree_slide`)
2. Understand the visual output (consult `./reference/mss_metrics_leadership.py` for real usage examples)
3. Define the DSL syntax for that slide type (consistent with the patterns above)
4. Implement the IR representation
5. Implement the PPTX renderer

### Validation rules

Same as the Python implementation:

- **Bullet length** > 85 characters at 18pt → warning
- **Bullet count** > 6 in a `content` slide → warning
- **Consecutive `content` slides** → warning
- **Cross-slide consistency** (counts in summary slides match content) → warning

All warnings include source spans pointing to the exact line/column in the DSL.

### Error recovery semantics

See ADR-009 (proposed) for the error recovery contract: what partial AST is produced on parse failure, whether the evaluator runs on a partial AST, and whether layout/export is attempted on a partial IR. This must be decided before Phase 1 parser design.

### Multi-file projects

`@include "path.sf"` directives are supported (resolved by Q3 in decisions-applied.md).
See ADR-004 for `@include` resolution semantics (relative vs. absolute paths, cycle detection, source-span propagation across files).
