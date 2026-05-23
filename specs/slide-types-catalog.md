---
title: Slideforge Slide Types Catalog
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Slideforge Slide Types Catalog

> Source: §5 slide types table from PROJECT-SEED.md.
> Cross-reference: `./reference/presentation-system.md` is the authoritative visual spec for each type.
> Cross-reference: `./reference/build-incident-brief.py` is the authoritative behavioral reference.

---

## The 23 Slide Types

All 23 slide types from the Python reference implementation must be supported with visual parity.
The visual parity contract is defined in `visual-parity-contract.md`.

| Type | DSL Keyword | Visual Pattern |
|------|-------------|----------------|
| Section divider | `title` (with `color`) or `divider` | Full-bleed blue/purple slide |
| Bullet content | `content` | Title + bullets |
| Two columns | `two_column` | Two columns of bullets |
| Content + stat | `content_stat` | Blocks left, stat card right |
| Stat callout | `stat_callout` | 2-6 stat cards |
| Stats + summary | `stats_summary` | Effort top + zero-result band |
| Single highlight | `highlight` | Callout box + supporting bullets |
| Two boxes | `highlight_boxes` | Two stacked full-width bands |
| Split contrast | `split_contrast` | Two panels with mini-timelines |
| Card rows | `card_rows` | Two-column checklist |
| Severity cards | `severity_cards` | Risk register with badges |
| Numbered actions | `numbered_actions` | Numbered cards with categories |
| Vertical timeline | `vertical_timeline` | Cascade timeline cards |
| Horizontal timeline | `horizontal_timeline` | Time bar with alternating cards |
| Enhanced table | `enhanced_table` | Table with colored row borders |
| Plain table | `table` | Standard branded table |
| Status dashboard | `status` | Label + badge pairs |
| Progress bar | `progress_bar` | Horizontal bar with items |
| Metric tree | `metric_tree` | Root → categories → leaves |
| Formula | `formula` | Equation + term cards |
| Weighted composite | `weighted_composite` | Stacked bar + legend |
| End slide | `end` (with `color`) | Closing slide |
| Stat callout alias | `key_metrics` | Alias for `stat_callout` |

## Constraint: No Invented Types

Do NOT invent slide types. The 23 types above are the contract. New types require:
1. Human approval
2. Design review
3. Update to this catalog, the DSL spec, and the reference materials

## Reference Materials

For each slide type:
- **Visual spec:** `./reference/presentation-system.md` (Section 3 has per-type descriptions)
- **Behavioral reference:** `./reference/build-incident-brief.py` (search for `def build_X_slide`)
- **Real usage examples:** `./reference/mss_metrics_leadership.py` (25 slides, all major types)
- **Design rules:** `./reference/presentation-system.md` Section 5

## How to Use the Reference

The Python code is the authoritative behavior reference, but **do not literally port it line-by-line**. The Python implementation has accumulated stylistic and structural choices that don't translate well to Rust. Specifically:

- **Do** match the visual output of each slide type
- **Do** preserve the brand color vocabulary and font sizes
- **Do** keep the same slide types and their core semantic meaning
- **Don't** carry forward Python's mutable-dict patterns (use Rust enums + structs)
- **Don't** mirror the Python builder's procedural style (use the Typst-style pipeline)
- **Don't** preserve Python-specific quirks like the `_parse_bullets` helper's string-prefix-based formatting (`"**bold**"`); use proper inline-formatting syntax in the DSL
