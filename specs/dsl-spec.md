---
title: Slideforge DSL Specification
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.1
created: 2026-05-23
updated: 2026-06-08
status: ACTIVE
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

# The inline list-literal form is also accepted for bullets:
slide content:
  title "Three Pillars"
  bullets: ["Detect", "Respond", "Recover"]   # same result as the dash-list form above

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

### Deck-Level Directives

Deck-level directives appear at the top level of a `.sf` file, outside any `slide` block. They are processed before any slide content is evaluated.

#### `vars:` block

The canonical way to declare multiple variables:

```
vars:
  company "Acme Corp"
  year 2026
  quarters ["Q1", "Q2", "Q3", "Q4"]
```

#### `@var` inline assignment

A single-variable shorthand for use in decks where a full `vars:` block would be verbose. Syntax:

```
@var <ident> = <value>
```

`<ident>` is an unquoted identifier. `<value>` may be any literal that is valid in a `vars:` block: a quoted string, a number, a boolean (`true`/`false`), or a list literal (`["A", "B"]`).

Examples:

```
@var company = "Acme Corp"
@var year = 2026
@var labels = ["Q1", "Q2", "Q3", "Q4"]
```

`@var` produces a single variable binding in the same scope as the `vars:` block. It can appear anywhere at the top level of a `.sf` file — before or after slide blocks. Multiple `@var` directives are allowed; later declarations for the same identifier overwrite earlier ones (last-wins scalar semantics per the merge rules in q16-q25-decisions.md).

**Keyword-collision rule:** If `<ident>` is a reserved slide-type keyword (e.g., `title`, `content`, `divider`), the parser emits **E-PAR-008** (`Variable name '<name>' collides with reserved keyword`) and rejects the declaration. The same rule applies to `vars:` block keys — `@var` is not a special bypass.

**ADR-007 context:** `@var` is a deck-level directive that belongs to ADR-007's directive list (alongside `@include`, `@for`, `@if`). It is NOT a control-flow statement — it cannot appear inside a slide body or inside an `@for`/`@if` block. It is strictly a top-level variable-binding directive.

### Inline List Literals

Lists in the DSL can be written in two equivalent forms. Both produce `FieldValue::List` in the IR.

#### Dash-list form (canonical for multi-item, multi-line lists)

```
bullets:
  - "Item one"
  - "Item two"
  - "Item three"
```

#### Inline list-literal form (shorthand for compact or data-driven lists)

```
bullets: ["Item one", "Item two", "Item three"]
```

The no-colon variant is also accepted by the parser:

```
bullets ["Item one", "Item two", "Item three"]
```

Both variants are semantically identical and produce the same `FieldValue::List` IR node.

**String-only constraint:** All items in a list literal must be **quoted string literals**. Unquoted values (bare words, integers, booleans, color names, or any other non-string token) are rejected at parse time with **E-PAR-024** (`non-string list item`). This applies to both `bullets: [...]` field values and `@var ident = [...]` deck-level assignments.

Examples of INVALID list items:

```
bullets: [Detect, Respond, Recover]   # ERROR E-PAR-024 — bare words, not quoted strings
bullets: [1, 2, 3]                    # ERROR E-PAR-024 — integers, not quoted strings
bullets: [true, false]                # ERROR E-PAR-024 — booleans, not quoted strings
```

**No implicit type coercion.** String values are stored as-is; `"NO"` stays the string `"NO"`, not the boolean `false`. `"1.10"` stays the string `"1.10"`, not the float `1.1`. This is a firm design decision (Q16 DSL decision; see q16-q25-decisions.md).

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

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-23 | product-owner | Initial extraction from PROJECT-SEED.md §5. Syntax goals, indented-block syntax sample, syntax features, reserved keywords, color vocabulary, slide type catalog reference, validation rules, error recovery, multi-file projects. |
| 1.1 | 2026-06-08 | product-owner | STORY-088 adversary Pass-1 OBS (process-gap): **Two DSL surfaces documented.** (1) `@var ident = value` deck-level inline variable assignment — shorthand for a single-variable binding, sibling to the `vars:` block. Documents syntax, semantics (last-wins overwrite, same scope as `vars:`), keyword-collision rule (E-PAR-008 on reserved-keyword ident), and ADR-007 placement (top-level directive only, not inside slide or control-flow bodies). (2) Inline list-literal form `bullets: ["A", "B", "C"]` (and no-colon `bullets [...]` parser variant) — documents both forms as producing `FieldValue::List`, the string-only constraint (E-PAR-024 on non-string items), no-implicit-type-coercion rule ("NO" stays "NO", "1.10" stays "1.10"), and equivalence to the canonical dash-list form. An inline code example added to the Syntax sample showing the `bullets: [...]` shorthand. No existing content removed or modified. Status updated from `SEED-EXTRACT` to `ACTIVE`. |
