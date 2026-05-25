---
document_type: gene-transfusion-assessment
level: L4
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
phase: 1b
inputs:
  - .factory/planning/python-reference-deep-read.md
  - .factory/seed/reference/build-incident-brief.py
  - .factory/seed/reference/presentation-system.md
  - .factory/seed/reference/mss_metrics_leadership.py
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/visual-parity-contract.md
  - .factory/specs/slide-types-catalog.md
input-hash: "[pending compute-input-hash]"
traces_to: .factory/specs/architecture/ARCH-INDEX.md
---

# Gene Transfusion Assessment: slideforge Python Reference

## Nature of This Transfusion

This is a behavioral transfusion, not a library transfusion. The Python reference
(`build-incident-brief.py`, 2216 lines) is not a library to be translated — it is an
authoritative source of visual behavior, layout constants, and design decisions that
the Rust implementation must selectively preserve, improve, or replace.

The assessment proceeds in three parts:

1. Behavior Inventory — what the Python reference does, organized by subsystem
2. Preserve / Improve / Replace / Drop matrix — per-behavior classification
3. Visual Parity Baseline — which reference outputs become holdout evaluation fixtures

The authoritative deep-read catalog is `.factory/planning/python-reference-deep-read.md`.
All R-NNN finding references below map to that document.

---

## Part 1: Behavior Inventory

### 1.1 Color Resolution

The reference defines 11 named colors in `COLOR_MAP` (lines 78–82), resolved by
`resolve_color()` (lines 85–89). Color names are strings; unknown strings silently fall
back to `TEAL` (`#81C6BD`). `RGBColor` objects may also be passed directly (bypassing
the map entirely).

**Internal-only colors** (not in the public map, hard-coded in builders): `#F2F2F2`,
`#F5F7FA`, `#FFF3ED`, `#E8F5E9`, `#E0E0E0`, `#336699`, `#BBCCDD`. These are scattered
across 15+ builders with no central registry.

### 1.2 Layout Constants

Slide dimensions: 13.33" × 7.5" (16:9 widescreen). Left margin `L=0.67"`, body width
`BW=11.8"`. Title band: `(0.67", 0.54")` — top of every content slide. Body start
`BT_SHORT=1.5"` (short layout, used by all 21 content builders). Takeaway bar: fixed at
`(0.67", 6.1")`, height `0.5"`, fill `#E8F0F8`.

Font-size ladder (named constants): TITLE_SIZE=24pt, SUBTITLE_SIZE=20pt, BODY_SIZE=18pt,
HEADER_SIZE=18pt, SMALL_SIZE=16pt, BADGE_SIZE=14pt, STAT_SIZE=44pt, STAT_LABEL_SIZE=16pt,
HIGHLIGHT_SIZE=20pt, TAKEAWAY_SIZE=16pt.

One anomaly: `build_title_slide` hard-codes `32pt` instead of using a constant (R1
finding 2 in deep-read). All other types use named constants.

### 1.3 Slide Type Implementations (23 types)

All 23 builder functions follow a consistent pattern:
`clear_all_placeholders` → position title text box → position body elements →
optionally position takeaway bar → inject `talk_track` into speaker notes.

Key behavioral properties per type are documented in detail in Part 2.

### 1.4 Bullet / Rich Text System

Two-layer system: `add_text_box` (single paragraph, no formatting control) and
`add_rich_text` (multi-paragraph with per-paragraph and per-run keys). Bullet glyphs are
embedded as `"  •  "` string prefixes by `add_rich_text` — not structural markup.

`_parse_bullets` is the load-bearing string-DSL: `"**header**"` prefix → bold blue
header; plain string → bullet item; dict → pass-through. This drives every bullets list
in the system.

### 1.5 Takeaway Bar

Present in 20 of 23 slide types (`table`, `end`, and `key_metrics`/alias are the
exceptions). Hard-coded geometry and color. Accepts `takeaway_align`
("left"/"center"/"right", default center) and `takeaway_size` (pt override). Neither
field is documented in the spec — they are undocumented escape hatches in the code.

When a takeaway is present, body height compresses from `5.0"` to `4.2"` (for
most types) to avoid overlap with the takeaway bar. This is handled per-builder with no
shared utility.

### 1.6 Table Primitive

`add_table` (lines 154–190): row height fixed at `0.35"`. Row 0 is blue header
(`#003766`), white bold text. Alternating fill: even rows `#F2F2F2`, odd rows white.
All cells at 16pt (SMALL_SIZE), Trebuchet MS. No border lines; fill-only rows.

### 1.7 Status Badges

`add_status_badge` wraps `add_rounded_rect` + centered white bold text at 14pt
(BADGE_SIZE). Used for severity labels, status indicators, timeline badges. Color
resolved via `resolve_color`.

### 1.8 Content Overflow Handling

The Python reference handles overflow exclusively through silent slicing:
- `events[:8]` in vertical_timeline
- `events[:12]` in enhanced_table (code says 12; spec says 10)
- `categories[:5]` in metric_tree
- `events[:6]` in horizontal_timeline and split_contrast
- `boxes[:2]` in highlight_boxes
- `top_stats[:4]` (implicit via `min(len, 4)`) in stats_summary

`card_rows` has NO overflow cap at all — content silently falls off the slide.

Bullet validation (`validate_slides`) WARNS on count > 6 and length > 85 chars but
never blocks the build.

### 1.9 Speaker Notes

Injected by the orchestrator (`build_presentation`, lines 2179–2183), not by individual
builders. All slides including `end` receive notes if `talk_track` is non-empty. Plain
string assignment to `notes_text_frame.text` — no formatting, no paragraph structure.

### 1.10 Template Layout System

11 PPTX template layout indices are used (`BLANK=0`, `BORDER_BLANK=1`,
`DIVIDER_PURPLE=4`, `DIVIDER_BLUE=5`, `SHORT_ONE=11`, `SHORT_TWO=12`, `LONG_ONE=13`,
`LONG_TWO=14`, `END_WHITE=18`, `END_PURPLE=19`, `END_BLUE=20`). The reference is
tightly coupled to a specific `1898` template file. The builder calls
`prs.slide_layouts[N]` by index — zero abstraction over the layout system.

`LAYOUT_LONG_ONE=13`, `LAYOUT_LONG_TWO=14`, `LAYOUT_BLANK=0`, `LAYOUT_BORDER_BLANK=1`
are defined but never used by any builder — dead code.

### 1.11 Font System

Single font throughout: Trebuchet MS. Set explicitly on every text run via `font.name`.
No fallback logic; LibreOffice substitutes Carlito when Trebuchet MS is absent. No
web-safe or system-font fallback chain.

### 1.12 Numeric / Auto-Sizing Behaviors

Several builders auto-size text or layout based on data volume:
- `stat_callout`: value font 40pt (n≤4) vs 28pt (5–6 stats)
- `content_stat` single-stat: 3-way font size switch based on string length
  (≤6 → 52pt; ≤14 → 36pt; >14 or contains `\n` → 28pt multi-line)
- `severity_cards`: card height auto-scales between `[0.7", 0.95"]` based on item count
- `vertical_timeline`: card height = `min((avail − gap*(n-1))/n, 0.7")`
- `enhanced_table`: row height = `min((avail − header − gaps)/n, 0.48")` then
  font downgrades from 13pt to 11pt if `row_h < 0.36"`
- `metric_tree`: enforces minimum 4 chips per column regardless of height available

### 1.13 Weighted Composite Bug

`build_weighted_composite_slide` normalizes segment widths by `weight/sum`, but
displays raw (un-normalized) weight values in the stacked bar inline labels AND in the
legend as `"{weight}% weight"`. If weights sum to 200, bars are half-width but labels
say 50/50/100. The spec does not mention normalization at all. R1 finding 15 in
deep-read documents this as a spec-code divergence.

### 1.14 Key Metrics Alias

`build_key_metrics_slide` is a pure runtime alias: it copies `slide_data`, renames
`metrics` key to `stats`, then delegates to `build_stat_callout_slide`. The alias
resolution happens at render time inside the builder, mutating a dict copy. The spec
does not document the copy-and-rename behavior.

---

## Part 2: Preserve / Improve / Replace / Drop Matrix

### Key: Classification Definitions

| Class | Meaning |
|-------|---------|
| PRESERVE | Replicate behavior exactly — it is correct and forms part of the visual contract |
| IMPROVE | Behavior exists but has a documented defect that slideforge corrects |
| REPLACE | Behavior is deliberately redesigned in slideforge (DSL-level or architectural) |
| DROP | Behavior is out of scope for slideforge v1.0 |

---

### 2.1 Color System

| # | Behavior | Class | Justification | BC / ADR |
|---|----------|-------|---------------|----------|
| C-01 | 11 named color vocabulary (`blue`, `orange`, `purple`, `dark_gray`, `gray`, `teal`, `white`, `light_gray`, `red`, `green`, `light_blue`) | PRESERVE | These are the visual brand contract for all 23 slide types; all output snapshots depend on them | BC-2.01.002 |
| C-02 | Internal-only colors (`#F5F7FA`, `#FFF3ED`, `#E8F5E9`, `#E0E0E0`, `#F2F2F2`, `#336699`, `#BBCCDD`) | PRESERVE | Required for visual parity; must be named constants in `slideforge-types` design-token table | BC-4.01.002 |
| C-03 | Unknown color string silently falls back to TEAL | IMPROVE | BC-1.02.003 (no silent coercion), CLAUDE.md Forbidden Patterns entry, R1 finding 3 in deep-read — must be compile error in slideforge | BC-1.02.003, BC-3.03.002 |
| C-04 | Non-`"blue"` string in `build_title_slide` silently becomes purple | IMPROVE | R1 finding 4 in deep-read — enum-typed `color` field; invalid value is a parse error with correction hint | BC-3.01.002, BC-1.15.001 |
| C-05 | Unknown `end` color silently becomes blue | IMPROVE | Same pattern as C-04 — enum-typed field | BC-3.01.002, BC-1.15.001 |
| C-06 | Brand colors applied per slide type through hardcoded builder logic | REPLACE | slideforge uses brand.toml + `brand.*` references in `set` rules; slide-type defaults read from brand token table, not hard-coded | BC-2.01.002, BC-1.08.003 |
| C-07 | `RGBColor` objects accepted directly in builders | DROP | Users of the slideforge DSL use named colors or hex literals; raw object injection is internal Python API only |  |

### 2.2 Layout Constants

| # | Behavior | Class | Justification | BC / ADR |
|---|----------|-------|---------------|----------|
| L-01 | Slide dimensions: 13.33" × 7.5" widescreen | PRESERVE | Visual parity contract binding; all snapshot fixtures use this geometry | BC-4.01.002 |
| L-02 | Left margin `L=0.67"`, body width `BW=11.8"` | PRESERVE | Required for positional snapshot tolerances (±4pt) | BC-4.01.002 |
| L-03 | Title band position `(0.67", 0.54")` × `(11.8", 0.5")` | PRESERVE | Present on all 21 content slide types | BC-4.01.002 |
| L-04 | Body start `BT_SHORT=1.5"` | PRESERVE | Required for layout fidelity across all short layouts | BC-4.01.002 |
| L-05 | Takeaway bar at `(0.67", 6.1")`, height `0.5"`, fill `#E8F0F8` | PRESERVE | Hard-coded geometry of the takeaway bar is part of the visual contract | BC-4.01.002 |
| L-06 | Body compression `5.0" → 4.2"` when takeaway present | PRESERVE | Prevents takeaway overlap; must be encoded in the layout engine as a named layout variant | SS-05 |
| L-07 | Font-size ladder (10 named sizes) | PRESERVE | All sizes are part of the snapshot contract; stored as design tokens in `slideforge-types` | BC-4.01.002 |
| L-08 | `build_title_slide` uses hard-coded 32pt | IMPROVE | R1 finding 2 in deep-read — add `DIVIDER_TITLE_SIZE=32` to design-token table; no magic numbers | ADR-013 |
| L-09 | Dead layout constants (`BT_LONG`, `BH_LONG`, `LAYOUT_LONG_*`, `LAYOUT_BLANK`, `LAYOUT_BORDER_BLANK`) | DROP | R1 finding 15 in deep-read; no builders use them; do not port until long-layout types are added | |

### 2.3 Per-Slide-Type Layout and Visual Behavior

| # | Type | Behavior | Class | Justification | BC |
|---|------|----------|-------|---------------|----|
| T-01 | `title` | Blue/purple variant from `color` field; title 32pt bold white; subtitle 20pt light_gray; subtitle omitted when empty string | PRESERVE | Visual contract for section dividers; all 5 uses in production deck | BC-3.01.001, BC-4.01.002 |
| T-02 | `content` | Title BLUE 24pt; bullets via `_parse_bullets`; body height compression when takeaway | PRESERVE | Core workhorse type | BC-3.01.001 |
| T-03 | `two_column` | Column widths 5.8" each; gap 0.39"; body height compression | PRESERVE | Visual contract | BC-3.01.001 |
| T-04 | `table` | Row height 0.35"; blue header row; alternating fill; 16pt Trebuchet; footnote support; no takeaway | PRESERVE | Table visual contract | BC-3.01.001 |
| T-05 | `status` | Badge per status item; summary stat cards below; vertical spacing adaptive to summary presence | PRESERVE | Production use in leadership deck | BC-3.01.001 |
| T-06 | `stat_callout` | Grid layout (n≤4→cols=n; 5–6→cols=3; 7+→cols=4); card height 2.6"; three styles (cards/light/plain); auto-sized value font (40pt/28pt) | PRESERVE | One of the most visually complex types; 0 of 23 layouts is free to drop | BC-3.01.001 |
| T-07 | `highlight` | Conditional layout (box/supporting geometry changes when takeaway present); LIGHT_BLUE rounded rect | PRESERVE | 3 uses in production deck | BC-3.01.001 |
| T-08 | `key_metrics` | Alias for `stat_callout`; accepts `metrics` key | PRESERVE (as DSL alias) | Rename/alias resolution moves to parse time via `alias` syntax; no runtime dict mutation | BC-1.09.001 |
| T-09 | `content_stat` | Left 7.0" column (blocks/bullets); right 4.2" column (stat card or stat rows); 3-way auto-size for single stat | PRESERVE | 5 of 25 slides in production deck; most complex layout type | BC-3.01.001 |
| T-10 | `card_rows` | Two-column card rows; 0.65" height; left stripe; header optional; background `#F5F7FA` | PRESERVE | Card visual pattern used in multiple types | BC-3.01.001 |
| T-11 | `severity_cards` | Auto-scaled card height `[0.7", 0.95"]`; proportional text positioning; severity badge right-aligned; 13pt description (not BADGE_SIZE) | PRESERVE (including 13pt anomaly) | 13pt is part of the visual contract; the inconsistency with BADGE_SIZE must be documented in design tokens, not "fixed" | BC-3.01.001, BC-4.01.002 |
| T-12 | `highlight_boxes` | Two stacked boxes; 2.1" height; optional stat value/label; item icons (check/arrow/literal) | PRESERVE | Visual pattern for dual-band comparisons | BC-3.01.001 |
| T-13 | `numbered_actions` | Numbered circle (0.38"); column-major distribution; legend auto-derived from categories; 1–3 column layout | PRESERVE | Action plan visual pattern | BC-3.01.001 |
| T-14 | `progress_bar` | Hard-coded GREEN/ORANGE bar fill; progress label above bar; resolved/in-progress card layouts above and below | PRESERVE (colors non-configurable) | Progress bar colors are part of the semantic visual contract (green=done, orange=in-progress) | BC-3.01.001 |
| T-15 | `stats_summary` | Top stat cards 36pt; summary band with bg=`#E8F5E9` (hard-coded regardless of summary color); inline `"{value}  {label}"` | IMPROVE | R1 finding 10 in deep-read — `stats_summary` band BG is intended to derive from `summary["color"]` per the spec prose but the code always uses `#E8F5E9`. This is a known code-spec divergence. slideforge behavior: band BG is a derived desaturated tint of `summary.color`; if not provided, defaults to `#E8F5E9` | BC-3.01.001 |
| T-16 | `split_contrast` | Dual panel 5.8"×4.2"; max 6 events; dot + date + description; stat at panel bottom | PRESERVE | Visual parity required | BC-3.01.001 |
| T-17 | `vertical_timeline` | Card height auto-scaled; NEARMISS_BG when badge; timeline gutter with vertical line; date right-aligned; badge optional | PRESERVE | 0 uses in production deck but required by visual contract | BC-3.01.001 |
| T-18 | `horizontal_timeline` | 6-event cap; alternating above/below bar by index parity (even=above); connector lines; card accent | PRESERVE (including even-first alternation) | Visual contract is defined by even-index-above behavior | BC-3.01.001 |
| T-19 | `enhanced_table` | Column-striped rows with left stripe; badge integration; footnote; row font auto-downgrade (13→11pt if `row_h < 0.36"`) | PRESERVE | 5 of 25 slides in production deck | BC-3.01.001 |
| T-20 | `metric_tree` | Root → busbar → category headers → leaf chips; footnote/takeaway stacking geometry; minimum 4 chips per column forced | IMPROVE | R1 finding (metric_tree chip min-cap of 4 can cause overflow). slideforge: min-chip-count must not force overflow past the slide boundary. Replace implicit clamp with explicit `CanvasOverflow` warning from the validator (BC-3.03.001) | BC-3.03.001, BC-3.01.001 |
| T-21 | `formula` | Equation runs with operator characters; stack mode (default) vs inline mode; term cards with stripe accent | PRESERVE | Unique type for structural equations | BC-3.01.001 |
| T-22 | `weighted_composite` | Normalized segment widths; raw-weight labels in legend and inline bar | IMPROVE | R1 finding 15 (weights display bug): if `|sum_weights - 100| > 1`, slideforge emits a validation error. If weights sum to exactly 100 (or nearly so), display the provided values. If they are not percentages, normalize and display normalized values. No silent misleading label. | BC-3.01.002, BC-3.03.002 |
| T-23 | `end` | Three color variants (blue/purple/white); no content; placeholder cleared | PRESERVE | Closing slide visual contract | BC-3.01.001 |

### 2.4 Bullet and Rich Text Formatting

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| B-01 | Two-paragraph model (header paragraph + body paragraph with bullet glyph) | PRESERVE | Visual output contract; header paragraphs render as 18pt bold blue; body as 18pt dark_gray | BC-3.05.001, BC-4.01.002 |
| B-02 | Bullet glyph `•` with leading/trailing space padding | PRESERVE (as visual output) | The rendered glyph and spacing are part of the visual contract | BC-4.01.002 |
| B-03 | `"**header**"` string prefix triggers bold-header rendering | REPLACE | CLAUDE.md Forbidden Patterns entry, R1 finding 1 and 6 in deep-read. slideforge uses structural `header:` block or `Inline::Bold` variant in the DSL; string-prefix magic is eliminated at the lexer | BC-3.05.001 |
| B-04 | Per-run formatting in rich text (font_size, bold, color, alignment, space_after) | PRESERVE | The per-paragraph and per-run model maps to the DSL's inline formatting and `runs:` block | BC-3.05.001 |
| B-05 | Bullet list warnings (count >6, length >85 chars) | IMPROVE | Python only warns; slideforge emits `CanvasOverflow` warning (count > cap) in strict mode produces error, in warn-only mode continues | BC-3.03.001, BC-3.03.002, BC-3.03.003 |
| B-06 | `str.strip("*")` bold detection — all leading/trailing asterisks stripped | DROP | Eliminated by B-03 REPLACE; no string-prefix parsing in slideforge | BC-1.15.001 |

### 2.5 Takeaway Bar

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| TK-01 | Takeaway bar geometry, fill, text positioning | PRESERVE | Part of visual contract on 20 of 23 types | BC-4.01.002 |
| TK-02 | `takeaway_align` (left/center/right, default center) | PRESERVE (promoted to first-class) | R1 finding 12 in deep-read: was undocumented escape hatch; slideforge promotes to a documented `align:` field on the `takeaway:` block | BC-3.01.001 |
| TK-03 | `takeaway_size` (pt override) | PRESERVE (promoted to first-class) | Same reasoning as TK-02; slideforge promotes to documented `font_size:` field | BC-3.01.001 |
| TK-04 | Body height compresses `5.0" → 4.2"` when takeaway present | PRESERVE | Layout engine encodes two named height variants: `BodyHeight::Full` and `BodyHeight::Compressed` | SS-05 |

### 2.6 Content Overflow and Validation

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| OV-01 | Silent slicing of all list fields (`events[:8]`, `boxes[:2]`, etc.) | IMPROVE | R1 finding 6, 7, 11 in deep-read. slideforge: all list fields have declared maximums in the `SlideType` trait. Exceeding the maximum is a `CanvasOverflow` validation warning in warn-only mode and a compile error in strict mode | BC-3.03.001, BC-3.03.002, BC-3.03.003 |
| OV-02 | `card_rows` has no overflow cap — content silently falls off slide | IMPROVE | Specific case of OV-01. slideforge validator applies the same overflow detection; no silent runoff | BC-3.03.001 |
| OV-03 | Python validator only emits warnings, never blocks build | REPLACE | slideforge strict mode (default) blocks on validation errors per BC-3.03.002 | BC-3.03.002 |
| OV-04 | Unknown slide type emits stdout warning and skips builder | REPLACE | slideforge: unknown slide type keyword is a compile error with a suggestion (Levenshtein-nearest known type) per BC-3.01.003 | BC-3.01.003, BC-1.15.001 |

### 2.7 Speaker Notes / Talk Track

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| NK-01 | Talk track text injected into PPTX speaker notes | PRESERVE | Part of the visual/structural contract per BC-4.01.003 | BC-4.01.003 |
| NK-02 | Notes are plain text, unformatted | REPLACE | slideforge supports the `notes:` writing register which may include DSL inline formatting; the register maps to structured paragraph elements in the notes XML | BC-1.14.001 |
| NK-03 | `end` slide supports talk_track | PRESERVE | The end slide may carry presenter notes even with no visual content | BC-1.14.001 |
| NK-04 | Notes injected by orchestrator, not individual builders | REPLACE | slideforge: notes register content is part of the slide IR (`Slide.notes`); injection happens at the IR construction stage, not at PPTX serialization time | SS-05, SS-06 |

### 2.8 Brand / Template System

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| BR-01 | Presentation built from a pre-existing `.pptx` template file | REPLACE | slideforge generates all layouts from `brand.toml` via brand synthesizer; no dependency on a specific external template file. Python was tightly coupled to `1898 template`. | BC-2.01.002, BC-2.01.005, ADR-010 |
| BR-02 | Single Trebuchet MS font for all elements | IMPROVE | slideforge uses the font declared in `brand.toml`; Trebuchet MS is the default but is user-configurable. Build host font unavailability produces a warning with fallback per BC-2.01.006 | BC-2.01.006 |
| BR-03 | No fallback font chain — LibreOffice silently substitutes Carlito | IMPROVE | slideforge explicitly documents Carlito as the LibreOffice fallback in the divergence log; this is a known cross-renderer divergence, not a silent failure | BC-4.01.002, visual-parity-contract.md |
| BR-04 | Layout indices are template-specific integers (4, 5, 11, 12, 18, 19, 20) | REPLACE | slideforge brand synthesizer generates the 31 layouts (11 standard + 20 custom) per BC-2.01.005; layout selection in the IR uses named layout identifiers, not template-index integers | BC-2.01.005, BC-4.01.005 |
| BR-05 | `clear_all_placeholders` called on every slide to remove template chrome | REPLACE | slideforge synthesizes slides from scratch; there are no template-placeholder bleeds to clear. The OOXML serializer does not inherit live template placeholder content | BC-4.01.001 |

### 2.9 Key Metrics Alias

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| KM-01 | `key_metrics` keyword accepted, maps to `stat_callout` visual rendering | PRESERVE | The `key_metrics` DSL keyword must continue to work; it is the user-facing name for the type | BC-1.09.001 |
| KM-02 | Alias resolution via `dict(slide_data)` copy + key rename at render time | REPLACE | R1 finding 9 in deep-read. slideforge resolves the alias at parse time; the IR only ever sees the canonical `StatCallout` node. No runtime dict mutation | BC-1.09.001, BC-1.09.002 |

### 2.10 Auto-Sizing Behaviors

| # | Behavior | Class | Justification | BC |
|---|----------|-------|---------------|----|
| AS-01 | `stat_callout` value font: 40pt (n≤4) / 28pt (5–6) | PRESERVE | Part of visual contract; layout engine encodes this as a named size-tier constant | BC-4.01.002 |
| AS-02 | `content_stat` single-stat 3-way font switch (≤6→52pt, ≤14→36pt, >14/`\n`→28pt) | PRESERVE | R1 finding 8 in deep-read documents the exact thresholds; these become design-token-backed constants in the layout engine | BC-4.01.002 |
| AS-03 | `severity_cards` card-height auto-scale `[0.7", 0.95"]` | PRESERVE | Required for visual parity with variable-length content | BC-4.01.002 |
| AS-04 | `enhanced_table` row font downgrade (13→11pt when `row_h < 0.36"`) | PRESERVE | Height threshold and font step are part of the visual contract; must be exact | BC-4.01.002 |
| AS-05 | `metric_tree` forces minimum 4 chips per column (can overflow) | IMPROVE | Replace with CanvasOverflow warning: layout engine clips at the slide boundary and reports overflow. Minimum 4 is NOT enforced when it causes overflow | BC-3.03.001 |

### 2.11 Edge-Case Behaviors to Carry Forward

These specific behaviors from R1 deep-read are not bugs — they are features that must
be documented and implemented exactly:

| # | Behavior | Class | Justification |
|---|----------|-------|---------------|
| E-01 | `horizontal_timeline` even-index-above, odd-index-below alternation | PRESERVE | Even=above is the visual contract; must be documented in layout engine as `TimelineAlternation::EvenAbove` |
| E-02 | `split_contrast` max 6 events per panel — excess is a validation error in slideforge | IMPROVE | Python silently slices; slideforge emits CanvasOverflow |
| E-03 | `enhanced_table` detail field folds into target column as `"{target} ({detail})"` | PRESERVE | R1 finding 13 in deep-read; this is a documented feature even if undocumented in the spec prose; slideforge promotes `detail:` to a first-class field on the event struct |
| E-04 | `weighted_composite` center-short legend rows (last row x-shifted if incomplete) | PRESERVE | Visual polish detail; must be replicated in layout math |
| E-05 | `numbered_actions` column-major distribution with global sequential numbering | PRESERVE | The column-major fill pattern is the visual contract; row-major would produce different output |
| E-06 | `stats_summary` summary items inline as `"{value}  {label}"` (two-space separator) | PRESERVE | Exact string construction including the two-space separator is part of the visual contract |

---

## Part 3: Visual Parity Baseline

These reference outputs become the canonical holdout evaluation fixtures. The standard
is: slideforge PPTX rendered in Microsoft PowerPoint on Windows 11 must be
visually indistinguishable from the Python reference output rendered in the same
environment, within the tolerances defined in `visual-parity-contract.md`.

### 3.1 Primary Fixture: mss_metrics_leadership.py

The file `.factory/seed/reference/mss_metrics_leadership.py` generates a 25-slide
production leadership deck using 11 of the 23 slide types. This deck is the primary
visual parity fixture because it represents real-world usage at production quality.

**Fixture target:** Run the Python reference against the 1898 template, capture the
resulting `.pptx`, render each slide to PNG in PowerPoint (Windows 11), commit 25
reference PNGs as snapshot fixtures.

**Coverage:** `title` (5×), `numbered_actions` (1×), `highlight` (3×), `formula` (1×),
`weighted_composite` (1×), `enhanced_table` (5×), `metric_tree` (2×),
`content_stat` (5×), `severity_cards` (1×), `end` (1×).

### 3.2 Secondary Fixtures: Per-Type Synthetic Decks

For the 12 types NOT covered by the production deck, construct minimal synthetic test
decks using `_template.py` and the parameter examples in `presentation-system.md`:

| Type | Coverage Gap | Fixture Priority |
|------|-------------|-----------------|
| `content` | Not in production deck | HIGH — most common type for basic usage |
| `two_column` | Not in production deck | HIGH — second most common expected type |
| `table` | Not in production deck | HIGH — distinct visual pattern |
| `status` | Not in production deck | HIGH — badge pattern used by many |
| `stat_callout` | Not in production deck | HIGH — cards style/light style/plain style all need separate fixtures |
| `stats_summary` | Not in production deck | MEDIUM |
| `highlight_boxes` | Not in production deck | MEDIUM |
| `split_contrast` | Not in production deck | MEDIUM |
| `card_rows` | Not in production deck | MEDIUM |
| `progress_bar` | Not in production deck | MEDIUM |
| `vertical_timeline` | Not in production deck | MEDIUM |
| `horizontal_timeline` | Not in production deck | MEDIUM |

### 3.3 Edge-Case Fixture Sets

These fixture sets validate the auto-sizing and overflow boundaries:

| Set | Slides | What It Tests |
|-----|--------|---------------|
| `stat_callout-2` | 1 slide | n=2 stats — 2 cards centered |
| `stat_callout-4` | 1 slide | n=4 stats — 4 cards single row, 40pt font |
| `stat_callout-5` | 1 slide | n=5 stats — 3+2 layout, 28pt font |
| `stat_callout-6` | 1 slide | n=6 stats — 3+3 layout, 28pt font |
| `content_stat-short` | 1 slide | single stat, value ≤6 chars → 52pt |
| `content_stat-medium` | 1 slide | single stat, value 7–14 chars → 36pt |
| `content_stat-long` | 1 slide | single stat, value >14 chars or `\n` → 28pt |
| `enhanced_table-small` | 1 slide | ≤8 rows — normal font (13pt) |
| `enhanced_table-large` | 1 slide | ≥11 rows — font downgrade (11pt) |
| `severity_cards-2` | 1 slide | 2 items — large card height |
| `severity_cards-6` | 1 slide | 6 items — small card height |
| `metric_tree-deep` | 1 slide | 5 categories × 4+ chips — tests chip overflow detection |
| `takeaway-align-variants` | 3 slides | left/center/right takeaway alignment on `content` type |
| `title-blue-purple` | 2 slides | blue and purple divider variants |
| `end-three-variants` | 3 slides | blue/purple/white end slides |

### 3.4 Fixture Generation Protocol

1. Run the Python reference to generate `.pptx` files from the fixture slide data
2. Open each `.pptx` in Microsoft PowerPoint on Windows 11; export each slide as PNG
   at 1920×1080 (144dpi)
3. Commit reference PNGs to `.factory/visual-parity/reference-fixtures/`
4. These PNGs become the ground truth for `cargo test --features visual-parity` in
   the slideforge CI pipeline
5. Failure threshold: PSNR ≥ 35dB per slide (defined in `visual-parity-contract.md`)

---

## Part 4: Modules Without Transfusion Candidates

The following slideforge modules were assessed for gene transfusion from the Python
reference and found to have no viable candidate — they must be implemented from scratch.

| Module | Reason |
|--------|--------|
| `slideforge-syntax` (DSL parser) | Python has no DSL parser — data is Python dicts/lists. The chumsky-based `.sf` parser is a new invention with no reference. |
| `slideforge-eval` (expression evaluator) | Python has no expression evaluator — all values are Python expressions evaluated by the interpreter. slideforge's `{{ expr }}` engine has no reference equivalent. |
| `slideforge-validate` (compile-time validator) | Python validation is 42 lines of warn-only checks. slideforge's full type-checked validation subsystem is architecturally new. |
| `slideforge-brand` (brand synthesizer) | Python is hard-coupled to a specific `.pptx` template file. The `brand.toml → 31-layout OOXML` synthesizer has no reference. |
| `slideforge-data` (data sources) | Python reads a dict literal from the same file. JSON/CSV/YAML/HTTP data binding has no reference. |
| `slideforge-charts` (chart renderer) | Python has no chart rendering capability. |
| `slideforge-math` (math renderer) | Python has no math rendering capability. |
| `slideforge-diagrams` (diagram renderer) | Python has no diagram rendering capability. |
| `slideforge-package` (package management) | Python has no package management. |
| `slideforge-config` (workspace configuration) | Python has no workspace configuration. |
| `slideforge-preview` (web preview) | Python has no preview server. |
| `slideforge-pdf` (PDF export) | Python has no PDF export. |
| `slideforge-docx` (DOCX export) | Python has no DOCX export. |
| `slideforge-html` (HTML export) | Python has no HTML export. |

### Transfusion Scope: Layout Engine + PPTX Serializer

The Python reference provides behavioral transfusion material — not code transfusion —
for two modules:

**`slideforge-layout`** (SS-05): The layout engine must reproduce the exact EMU
coordinates, size relationships, and adaptive-sizing behaviors cataloged in Part 2.
The Python constants table (layout constants, font sizes, color hex values) translates
directly into the design-token constant table in `slideforge-types`. Estimated effort
saving: 3–5 story points of empirical layout calibration work (deriving the correct
numbers from scratch vs reading them from the reference).

**`slideforge-pptx`** (SS-06): The PPTX serializer for the 23 slide types must
reproduce XML that renders identically to the Python reference output. The reference
provides the exact visual target for each type, validated by the fixture set. Estimated
effort saving: 5–8 story points of visual debugging work that would otherwise be
required to hit the PSNR ≥ 35dB target without a reference.

---

## Summary

| Metric | Value |
|--------|-------|
| Python reference files assessed | 4 (build-incident-brief.py 2216L, presentation-system.md 1024L, mss_metrics_leadership.py 740L, _template.py 155L) |
| Behaviors inventoried | 75 (across 11 behavioral categories) |
| PRESERVE | 42 |
| IMPROVE | 17 |
| REPLACE | 11 |
| DROP | 5 |
| Modules with code transfusion candidates | 0 |
| Modules with behavioral transfusion value | 2 (slideforge-layout, slideforge-pptx) |
| Estimated story-point saving from behavioral reference | 8–13 SP (visual calibration + fixture baseline) |
| Primary holdout fixture source | mss_metrics_leadership.py (25-slide production deck) |
| Secondary fixture count (per-type synthetic) | 12 types + 15 edge-case sets |
| R1 findings consumed | 16 of 18 (findings 17–18 are informational geometry notes) |

---

## Binding Decisions

These decisions are LOCKED and must not be reversed without an ADR:

1. **No code from `build-incident-brief.py` is ported to Rust.** The Python code is
   behavioral reference only. The OOXML output is the deliverable, not the Python
   implementation pattern.

2. **All 23 slide types must reach visual parity.** The 12 types absent from the
   production deck get synthetic fixtures. None are dropped from v1.0.

3. **All IMPROVE items are implemented in v1.0.** The improvements are not optional
   polish — they fix documented defects (silent fallbacks, overflow bugs, spec-code
   divergences). Deferring them would violate the production-grade default.

4. **The `_parse_bullets` string-prefix convention (`"**header**"`) is eliminated**
   at the DSL lexer. No user-facing `.sf` syntax preserves string-prefix formatting
   conventions.

5. **`weighted_composite` normalization validation is a compile error** when
   `|sum(weights) - 100| > 1`. Misleading weight labels are not tolerated in
   production output.
