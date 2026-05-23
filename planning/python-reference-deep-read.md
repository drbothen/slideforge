---
title: Python Reference Behavior Catalog
date: 2026-05-23
analyst: research-agent
source_files:
  - .factory/seed/reference/build-incident-brief.py
  - .factory/seed/reference/presentation-system.md
  - .factory/seed/reference/mss_metrics_leadership.py
  - .factory/seed/reference/_template.py
status: foundation-research
---

# Python Reference — Behavior Catalog

## Reading Strategy

1. Read `_template.py` (155 lines) first to anchor the data shape and observe the enumerated slide-type list (23 types organized into 8 categorical groups).
2. Read `build-incident-brief.py` (2216 lines) in two sequential pages: lines 1–1162 (constants, helpers, builders 1–11) and 1163–2216 (builders 12–23, dispatcher, validator, orchestrator, CLI). Took line-by-line notes of every `Inches(...)`, `Pt(...)`, `RGBColor(...)` literal.
3. Read `presentation-system.md` (1025 lines, 10 sections) to cross-check the prose contract against code reality.
4. Grep `mss_metrics_leadership.py` for `"type":` occurrences to verify which slide types appear in production use (25 slides, 11 distinct types).
5. Key helpers identified up front: `_parse_bullets` (217–229), `_add_takeaway` (1067–1074), `_takeaway_align` (1062–1064), `resolve_color` (85–89), `clear_all_placeholders` (94–99), `add_text_box` (104–117), `add_rich_text` (120–151), `add_rounded_rect` (193–199), `add_status_badge` (202–214), `add_table` (154–190).

---

## Common Infrastructure (read first)

### `resolve_color(color)` — lines 85–89
- Accepts either a string from `COLOR_MAP` (78–82) or a pre-built `RGBColor`.
- Unknown strings silently fall back to `TEAL` (`#81C6BD`). **This is a silent-default behavior** the Rust port must decide whether to inherit.

### `clear_all_placeholders(slide)` — lines 94–99
- Iterates `slide.placeholders` and removes their underlying XML elements (`sp.getparent().remove(sp)`).
- Called at the top of every non-`end` builder. Without this, the template's "Click to add text" prompts bleed through.

### `add_text_box(slide, left, top, width, height, text, font_size=BODY_SIZE, bold=False, color=DARK_GRAY, alignment=PP_ALIGN.LEFT)` — lines 104–117
- Always sets `word_wrap=True`, `font.name=FONT` ("Trebuchet MS"). Single-paragraph only.
- Default color is `DARK_GRAY` (`#383B3D`), not pure black.

### `add_rich_text(slide, left, top, width, height, paragraphs)` — lines 120–151
- Multi-paragraph with optional per-run formatting.
- Per-paragraph keys: `font_size`, `bold`, `color`, `alignment`, `space_after`, `space_before`, `runs`, `text`, `bullet`.
- When `bullet=True`, prepends `"  •  "` (U+2022 with two leading and two trailing spaces) to the paragraph text.
- Per-run keys: `text`, `font_size`, `bold`, `color`. Defaults inherit from paragraph.

### `_parse_bullets(bullets, default_size=BODY_SIZE)` — lines 217–229
**This is the load-bearing string-DSL.** Three input types:
1. `dict` → passed through unchanged.
2. `str` starting with `**` → strips ALL `*` characters (`b.strip("*")`), promotes to bold blue header at `HEADER_SIZE` (18pt), color `BLUE`, `space_before=6`, `space_after=2`, **no bullet glyph**.
3. Plain `str` → bullet glyph prepended, `font_size=default_size`, `space_after=3`.

### `add_rounded_rect(slide, left, top, width, height, fill_color)` — lines 193–199
- Solid fill, `line.fill.background()` (no border). Used for takeaway bar, highlight boxes, badges.

### `add_status_badge(slide, left, top, width, height, text, color)` — lines 202–214
- Wraps `add_rounded_rect` + white centered text at `BADGE_SIZE` (14pt), bold, font Trebuchet MS.
- Calls `resolve_color` on the color argument.

### `add_table(slide, left, top, width, rows_data, col_widths=None)` — lines 154–190
- Row height fixed at `Inches(0.35)`; total height = `0.35" × n_rows`.
- Row 0: blue fill (`#003766`), white text, bold.
- Even rows (r=2,4,...): light gray fill `RGBColor(0xF2,0xF2,0xF2)`.
- Odd rows (r=1,3,...): white fill.
- All cells: `SMALL_SIZE` (16pt), Trebuchet MS.

### `_takeaway_align(slide_data)` — lines 1062–1064
- Reads `slide_data["takeaway_align"]` ("left"/"center"/"right"), defaults to `PP_ALIGN.CENTER`. Unknown values also default to center via `dict.get(..., PP_ALIGN.CENTER)`.

### `_add_takeaway(slide, text, y=None, align=PP_ALIGN.CENTER, size=None)` — lines 1067–1074
- **Hard-coded layout:** bar at `(L=0.67", y=6.1")`, width `BW=11.8"`, height `0.5"`, fill `LIGHT_BLUE` (`#E8F0F8`).
- Text inset: x=`0.9"`, y=`bar_top+0.05"`, width `11.3"`, height `0.4"`.
- Default font size `TAKEAWAY_SIZE` (16pt), bold, color `BLUE`.
- `size` override allows per-slide custom font size via `slide_data["takeaway_size"]`.

### Talk-track formatting — lines 2179–2183
- Single assignment to `slide.notes_slide.notes_text_frame.text = talk_track`. **No structured formatting; plain string assignment.**
- No prefix, no boilerplate. Empty/missing talk_track skips notes injection entirely.

---

## Per-Slide-Type Behavior Catalog (23 entries)

### `build_title_slide` (lines 234–247)
- DSL keyword: `title`
- Inputs: `color` ("blue"|"purple", default "blue"), `title`, `subtitle` (optional), `talk_track`.
- Behaviors:
  - Layout index `LAYOUT_DIVIDER_BLUE=5` for "blue" else `LAYOUT_DIVIDER_PURPLE=4`. Any non-"blue" string maps to purple (silent fallthrough).
  - Title: `(0.67", 2.0")`, size `12.0" × 1.5"`, **32pt** (hard-coded, not `TITLE_SIZE`), bold, WHITE, LEFT align.
  - Subtitle: `(0.67", 3.5")`, `12.0" × 1.0"`, `SUBTITLE_SIZE=20pt`, color `LIGHT_GRAY`, default LEFT align.
  - Subtitle skipped entirely if empty string.
- Quirks: Hard-codes 32pt instead of using a named constant. Empty subtitle leaves vertical gap.

### `build_content_slide` (lines 250–273)
- DSL keyword: `content`
- Inputs: `title`, `bullets` (list), `takeaway` (optional), `takeaway_align`, `takeaway_size`, `talk_track`.
- Behaviors:
  - Layout `LAYOUT_SHORT_ONE=11`.
  - Title at `(L, 0.54")`, `BW × 0.50"`, `TITLE_SIZE=24pt`, bold, BLUE.
  - Body area: top `BT_SHORT=1.5"`, width `BW=11.8"`, height `Inches(4.2)` if takeaway else `BH_SHORT=5.0"`. **Takeaway compresses body height.**
  - Bullets routed through `_parse_bullets`.
  - Takeaway via `_add_takeaway` if present.

### `build_two_column_slide` (lines 276–299)
- DSL keyword: `two_column`
- Inputs: `title`, `left` (list), `right` (list), `takeaway`, `takeaway_align`, `takeaway_size`.
- Behaviors:
  - Layout `LAYOUT_SHORT_TWO=12`.
  - Column width `5.8"` each. Left at `L=0.67"`, right at `6.86"`. Gap: `6.86 - (0.67 + 5.8) = 0.39"`.
  - Body height: `4.2"` if takeaway, else `5.0"`.

### `build_table_slide` (lines 302–321)
- DSL keyword: `table`
- Inputs: `title`, `table` (rows incl. header), `col_widths` (optional list of EMU), `footnote`, `talk_track`.
- Behaviors:
  - Table at `(L, BT_SHORT=1.5")`, width `BW=11.8"`.
  - Footnote top = `BT_SHORT + 0.35" × n_rows + 0.2"`, height `0.5"`, `BADGE_SIZE=14pt`, color GRAY.
- **No takeaway support** — distinct from most other types.

### `build_status_slide` (lines 324–378)
- DSL keyword: `status`
- Inputs: `title`, `statuses` (list of `{label,value,color}`), `summary_stats` (optional list of `{value,label,color}`), `takeaway`.
- Behaviors:
  - Per-row badge: label at `(L, y)` width `5.0"` BODY_SIZE bold DARK_GRAY; badge at `(6.0", y)` width `5.5"` height `0.38"`, fill = resolved color, text WHITE bold BADGE_SIZE centered.
  - Vertical spacing: `0.48"` if summary_stats or takeaway present, else `0.55"`. Starts at `BT_SHORT=1.5"`.
  - Summary stat cards: gap `0.4"`, width auto = `(BW − total_gap)/n`, height `1.1"`, top = last_y + `0.2"`. Fill = stat color. Value at 30pt bold WHITE centered; label at 13pt WHITE centered (note: **13pt, not BADGE_SIZE**).

### `build_stat_callout_slide` (lines 381–537)
- DSL keyword: `stat_callout` (alias: `key_metrics` accepts `metrics` key)
- Inputs: `title`, `stats` (2–6), `style` ("cards"|"light"|"plain"), `accent` (slide-level or per-stat), `context`.
- Behaviors:
  - Grid: n≤4 → `cols=n`; 5–6 → `cols=3`; 7+ → `cols=4`. Rows = ceil(n/cols).
  - Card height `2.6"`, gap `0.3"`. Accent bar height `0.06"`.
  - Available vertical area: `4.0"` if context else `4.8"`; cards vertically centered in this area.
  - Value font: 40pt if n≤4 else 28pt.
  - **"cards" style:** colored card fill, white text. Optional top accent bar (`s["accent"]` or slide-level `accent`); `accent=None` explicitly disables. Value 40pt/28pt centered WHITE; label `BADGE_SIZE` bold WHITE centered; sublabel 11pt LIGHT_GRAY.
  - **"light" style:** background `RGBColor(0xF5,0xF7,0xFA)`, colored left stripe `0.06"`. If `icon` set ("check"=U+2713, "arrow"=U+25B6, else literal char), renders header layout with icon + label + value below at 13pt; else standard centered layout. Optional `status` ("pass"→U+2713 GREEN, "fail"→U+2717 RED) in top-right at 20pt.
  - **"plain" style:** no background, value `STAT_SIZE=44pt` colored, label `STAT_LABEL_SIZE=16pt` DARK_GRAY, sublabel 12pt GRAY.
  - Context: top = cards_bottom + `0.3"`, width `BW`, `SMALL_SIZE=16pt`, color GRAY, CENTER.

### `build_highlight_slide` (lines 540–583)
- DSL keyword: `highlight`
- Inputs: `title`, `highlight` (string), `supporting` (list), `takeaway`.
- Behaviors:
  - **Conditional layout:** if takeaway present: box top `1.5"`/h `1.8"`, supporting top `3.6"`/h `2.3"`. Otherwise box top `2.0"`/h `2.0"`, supporting top `4.4"`/h `2.2"`.
  - Highlight box: `(0.9", box_top)`, width `11.4"`, fill LIGHT_BLUE rounded.
  - Highlight text: inside box with `0.3"` top inset and `0.6"` height reduction, `HIGHLIGHT_SIZE=20pt`, bold, BLUE, LEFT.

### `build_key_metrics_slide` (lines 586–591)
- DSL keyword: `key_metrics`
- **Pure alias** for `build_stat_callout_slide`. If `metrics` key present and `stats` absent, copies dict and renames `metrics`→`stats`. Uses `dict(slide_data)` to avoid mutating caller.

### `build_content_stat_slide` (lines 594–731)
- DSL keyword: `content_stat`
- Inputs: `title`, `blocks` (list of `{header,description}`) OR `bullets`, `stat` ({value,label,color}) OR `stat_rows` (list), `stat_color`, `stat_accent`, `takeaway`.
- Behaviors:
  - Left column: width `7.0"`, starts at `L`. Right column at `x=8.2"`, width `4.2"`.
  - Body height `4.2"` if takeaway else `5.0"`.
  - Blocks: paragraphs alternate (header BODY_SIZE bold BLUE space_after 2; description SMALL_SIZE DARK_GRAY space_after 10).
  - Stat rows: card top `1.6"`, height `4.2"`. Optional accent bar `0.06"`. Value 28pt bold WHITE right-aligned in 1.7" col; label BADGE_SIZE WHITE left-aligned. **Value 32pt if it's a U+2713 checkmark.** Dividers between rows at `RGBColor(0x33,0x66,0x99)`, height `0.03"`.
  - Single stat: card top `2.0"`, height `3.2"`, rounded rectangle. **Auto-sizes value font based on content:**
    - Has `\n` OR `len > 14` → 28pt, multi-line, vertically centered as value+label unit with `0.25"` gap.
    - `len > 6` → 36pt, value top `0.45"`, label top `1.85"`.
    - Else → 52pt, value top `0.4"`, label top `1.8"`.
  - Label: `STAT_LABEL_SIZE=16pt`, WHITE, centered.

### `build_card_rows_slide` (lines 734–802)
- DSL keyword: `card_rows`
- Inputs: `title`, `left`/`right` (list of strings or `{text}` dicts), `left_header`/`right_header`, `left_color`/`right_color`, `takeaway`.
- Behaviors:
  - Card BG `RGBColor(0xF5,0xF7,0xFA)`. Left column at `L`, right at `6.67"`. Column width `5.8"`.
  - Header (if present): `(col_x, 1.4")`, `0.4"` tall, `HEADER_SIZE=18pt` bold BLUE.
  - Card row: height `0.65"`, vertical gap `0.15"`, starts at `1.9"`. Left stripe width `0.06"`.
  - Text inset: `0.2"` left, `0.12"` top, `BADGE_SIZE=14pt`, DARK_GRAY.
  - **No row-cap enforcement** — long lists overflow silently below the slide.

### `build_severity_cards_slide` (lines 805–886)
- DSL keyword: `severity_cards`
- Inputs: `title`, `gaps` (list of `{header,description,severity,color}`), `takeaway`.
- Behaviors:
  - Bottom limit: `5.95"` if takeaway else `6.5"`. Available height = `bottom_limit − 1.4"`.
  - Card height auto-scales: `(available_h − 0.1" × (n-1)) / n`, clamped to `[0.7", 0.95"]`.
  - **Proportional text positioning:** header_top = h*0.08; header_h = h*0.28; desc_top = h*0.40; desc_h = h*0.38.
  - Header at `(0.9", y+header_top)`, width `8.5"`, SMALL_SIZE bold BLUE.
  - Description at `(0.9", y+desc_top)`, 13pt GRAY (not BADGE_SIZE).
  - Severity badge at `(10.7", y + (h - 0.32")/2)`, width `1.5"`, height `0.32"`, 12pt bold WHITE centered. Practical max ~6 items.

### `build_highlight_boxes_slide` (lines 889–966)
- DSL keyword: `highlight_boxes`
- Inputs: `title`, `boxes` (max 2, `{header,header_count,items,item_icon,bg_color,accent_color,text_color,stat_value,stat_label,stat_color}`), `takeaway`.
- Behaviors:
  - Box height `2.1"`, gap `0.2"`, starts at `1.4"`. Width `BW`. **Hard-capped to first 2 boxes via `boxes[:2]`.**
  - Item icons: "check"=U+2713, "arrow"=U+25B6, else "". Icon + double-space prepended to each item.
  - Stat label color computed: if `bg==BLUE` → `RGBColor(0xBB,0xCC,0xDD)` else GRAY.
  - Stat value 36pt bold; stat label 13pt. Items rendered via `add_rich_text` with `BADGE_SIZE` color=text_color.
  - `header_count` shifts stat down by `0.1"`.

### `build_numbered_actions_slide` (lines 969–1056)
- DSL keyword: `numbered_actions`
- Inputs: `title`, `actions` (list of `{text,category,color}`), `columns` (1–3, default 2, **clamped via `max(1, min(.., 3))`**), `legend` (auto-derived if absent), `takeaway`.
- Behaviors:
  - Card height `0.55"`, gap `0.12"`. Column gap `0.17"` if multi-col else 0. Cards start at `1.5"`.
  - Numbered circle: `(x+0.1", y+0.085")`, size `0.38"`, fill = action color, white 13pt bold text centered.
  - **Column-major distribution:** col 0 fills first (`per_col = ceil(n/cols)`), then col 1, etc. Numbering is global (1..n) in data order.
  - Legend at `y=5.2"`. Auto-derived from unique categories in data order. Each entry: 0.18" colored dot + 12pt GRAY label, `1.8"` spacing.

### `build_progress_bar_slide` (lines 1077–1175)
- DSL keyword: `progress_bar`
- Inputs: `title`, `resolved`, `in_progress` (both lists of `{label,value}`), `progress_label`, `takeaway`.
- Behaviors:
  - Hard-coded colors: `LIGHT_GREEN_BG=#E8F5E9`, `LIGHT_ORANGE_BG=#FFF3ED`, `BAR_BG=#E0E0E0`.
  - Bar at `(L, 3.4")`, width `BW`, height `0.22"`. Progress fill = `len(resolved)/total`. Filled portion GREEN, remainder ORANGE.
  - Progress label above bar at `bar_y − 0.35"`, BADGE_SIZE bold DARK_GRAY centered.
  - Resolved cards above (`y=1.5"`, height `1.2"`). In-progress cards below (`y=3.85"`, height `0.95"`).
  - In-progress cards centered in `BW × 0.7` to avoid sparse layout. Gap `0.35"` (vs `0.25"` for resolved).
  - Resolved label 12pt bold GREEN centered, value 11pt DARK_GRAY centered. In-progress label 13pt bold ORANGE.

### `build_stats_summary_slide` (lines 1178–1284)
- DSL keyword: `stats_summary`
- Inputs: `title`, `top_stats` (1–4, **capped via `min(len, 4)`**), `summary` ({header,color,bg_color,items}), `top_accent`, `takeaway`.
- Behaviors:
  - Top stat cards at `y=1.5"`, height `2.0"`, gap `0.3"`. Width auto. Value 36pt WHITE; label `BADGE_SIZE` WHITE; sublabel 11pt `RGBColor(0xBB,0xCC,0xDD)`.
  - Summary band at `y=3.8"`, height `1.6"`. BG color auto = `LIGHT_GREEN_BG=#E8F5E9` (hard-coded default regardless of summary color).
  - Band has top accent `0.06"` at summary color, then header at `(0.9", y+0.15")` `HEADER_SIZE=18pt` bold summary-color.
  - Summary items in a row, evenly spaced. Inline `"{value}  {label}"` (two-space sep) at BADGE_SIZE bold DARK_GRAY; sublabel below at 11pt GRAY.

### `build_split_contrast_slide` (lines 1287–1377)
- DSL keyword: `split_contrast`
- Inputs: `title`, `left_panel`, `right_panel` (each `{header,bg_color,text_color,accent_color,dot_color,events,stat_value,stat_label,stat_color}`), `takeaway`.
- Behaviors:
  - Each panel: width `5.8"`, height `4.2"`, top `1.4"`. Left at `L`, right at `6.67"`.
  - **Max 6 events per panel** (`min(len(events), 6)`).
  - Events: dot `0.14"`, date 10pt bold (text_color), description 11pt (text_color). Vertical step `0.35"`.
  - Stat at `panel_y + panel_h − 1.4"`. Value 36pt bold centered (stat_color); label 11pt centered (text_color).

### `build_vertical_timeline_slide` (lines 1380–1486)
- DSL keyword: `vertical_timeline`
- Inputs: `title`, `events` (max 8, capped via `events[:8]`), `footnote`, `takeaway`.
- Behaviors:
  - Available height `3.8"` if footnote/takeaway else `4.5"`. Card height = min((avail − gap*(n-1))/n, 0.7"). Gap `0.08"`.
  - Timeline x = `1.5"`; card x = `2.1"`, card width `10.0"`; dot size `0.18"`. Start y `1.35"`.
  - Vertical line: GRAY rectangle from first dot center to last dot center, `0.03"` wide.
  - Card BG: `RGBColor(0xFF,0xF3,0xED)` (NEARMISS_BG) if `badge` set, else `RGBColor(0xF5,0xF7,0xFA)`.
  - Date label: 11pt bold DARK_GRAY right-aligned in 1.15" gutter on the left.
  - Target: 13pt bold BLUE. Detail (optional): 10pt GRAY below target.
  - Impact: 12pt DARK_GRAY at `card_x+4.8"`, width `4.2"` (or `3.0"` if badge present).
  - Badge: rounded rect, `1.2" × 0.28"`, 9pt bold WHITE centered. Default badge_color "orange".

### `build_horizontal_timeline_slide` (lines 1489–1602)
- DSL keyword: `horizontal_timeline`
- Inputs: same as vertical_timeline. **Max 6 events** (`events[:6]`).
- Behaviors:
  - Bar at `(0.9", 3.3")`, width `11.4"`, height `0.05"`. Dot size `0.22"`.
  - Card width = `min(11.4/n − 0.1, 2.0)`. Card height `1.5"`.
  - **Alternation:** even index (0, 2, 4) → above bar; odd → below. Each card has a vertical connector to the bar.
  - Card has top accent `0.05"` in event color. Date 10pt bold GRAY centered, target 12pt bold BLUE centered, impact 10pt DARK_GRAY centered. Badge at card bottom, 9pt bold WHITE.

### `build_enhanced_table_slide` (lines 1605–1722)
- DSL keyword: `enhanced_table`
- Inputs: `title`, `events` (max 12, capped via `events[:12]`), `columns` (default `["Date","Target","Impact"]`), `col_widths` (default `[1.5", 5.5", 4.8"]`), `footnote`, `takeaway`.
- Behaviors:
  - Header row at `y=1.5"`, height `0.42"`, fill BLUE. Column headers BADGE_SIZE bold WHITE.
  - Row height = `min((avail − header − gaps)/n, 0.48")`. Gap `0.04"`.
  - **Auto-scale row text:** if `row_h < 0.36"` → 11pt with `0.05"` top offset; else 13pt with `0.08"` top offset.
  - Row BG: NEARMISS_BG if badge, else `#F5F7FA` (even i) or WHITE (odd i). Left stripe `0.06"` in event color.
  - Target column: if `ev["detail"]` present, target becomes `"{target} ({detail})"` (parenthesized).
  - Impact column: width reduced by `1.3"` if badge present.
  - Badge at right edge: `1.2" × 0.28"`, 9pt bold WHITE.
  - Doc says max 10 rows; code caps at 12.

### `build_metric_tree_slide` (lines 1725–1846)
- DSL keyword: `metric_tree`
- Inputs: `title`, `root` ({label,color}), `categories` (max 5, `categories[:5]`), `footnote`, `takeaway`.
- Behaviors:
  - Root: rounded rect, centered, width `6.5"`, height `0.7"`, top `1.3"`. Header text `HEADER_SIZE` WHITE centered.
  - Connector from root to busbar at `y=2.25"`, `0.03"` wide GRAY.
  - Category columns: gap `0.18"`, width auto. Each has a drop line from busbar to header at `y=2.45"`. Header card rounded rect, height `0.45"`, BADGE_SIZE bold WHITE centered.
  - Leaf chips: height `0.32"`, gap `0.06"`, light BG `#F5F7FA`, left stripe `0.05"` in category color, 11pt DARK_GRAY.
  - **Footnote/takeaway stacking logic** (1795–1806):
    - Both: footnote at `5.7"`, chip bottom limit `5.55"`.
    - Takeaway only: chip limit `5.95"`.
    - Footnote only: footnote at `6.3"`, chip limit `6.15"`.
    - Neither: chip limit `6.5"`.
  - Max chips per column = `max(int(available_h/(chip_h+gap)), 4)` — **forces minimum 4 even if cramped**.

### `build_formula_slide` (lines 1849–1962)
- DSL keyword: `formula`
- Inputs: `title`, `result` (string), `terms` (list of `{name,definition,color}`), `operator` (default "×"), `stack` (bool, default True), `accent`, `takeaway`.
- Behaviors:
  - Equation built as runs at 22pt bold. Result colored BLUE; terms colored per `term["color"]`; operators DARK_GRAY.
  - **Stack mode (default):** two paragraphs, both centered. Line 1 = result; line 2 = `= term × term × term` (with `"  "` double-space padding around operators).
  - **Inline mode:** single line `result  = term × term × term`. May wrap if long.
  - Equation y/height: `(1.55", 1.5")` stacked vs `(1.7", 1.4")` inline.
  - Optional accent line at `cards_y − 0.25"` (cards at `y=3.3"`), `2.0"` wide centered, height `0.04"`.
  - Term cards: width auto, height `2.0"`, BG `#F5F7FA`, top stripe `0.06"` in term color. Name `HEADER_SIZE` bold colored centered; definition BADGE_SIZE DARK_GRAY centered.

### `build_weighted_composite_slide` (lines 1965–2068)
- DSL keyword: `weighted_composite`
- Inputs: `title`, `components` (list of `{name,weight,color}`), `composite_label`, `context`, `takeaway`.
- Behaviors:
  - **Normalizes weights** by dividing by sum (`total = sum(weights) or 1`).
  - Composite label above bar at `bar_y − 0.45"`, HEADER_SIZE bold BLUE centered.
  - Stacked bar at `(L, 2.1")`, width `BW`, height `0.9"`. Segment width = `BW × (weight/total)` truncated to int.
  - **Inline % label** only if segment width ≥ `0.85"`. HEADER_SIZE bold WHITE centered, formatted as `"{weight}%"` (uses raw weight, not normalized).
  - Legend layout: n≤4 → 1 row of n; 5–6 → 3 per row; 7+ → 4 per row.
  - Legend col height `0.65"`. Last row centered if not full (`row_start_x` shifted).
  - Each legend entry: 0.2" dot + name BADGE_SIZE bold DARK_GRAY + weight `"{weight}% weight"` 11pt GRAY beneath.
  - Context paragraph below legend (if present): inset `1.0"` each side, centered BADGE_SIZE DARK_GRAY.

### `build_end_slide` (lines 2071–2078)
- DSL keyword: `end`
- Inputs: `color` ("blue"|"purple"|"white", default "blue").
- Behaviors:
  - Layout map: blue→20, purple→19, white→18. Unknown values default to blue. Clears all placeholders. **No text, no title, no content.**

---

## Validation Rules (from `validate_slides`, lines 2112–2153)

1. **Bullet length (lines 2123–2130):** For each `bullets`/`left`/`right` item that is a plain string OR dict with `text`, warn if `len > MAX_BULLET_CHARS=85` AND `not text.startswith("**")` (bold headers exempt). Warning includes slide number, type, key, index, length, and first 50 chars.
2. **Bullet count (lines 2132–2137):** Warn if `len(bullets) > MAX_BULLETS=6` AND `slide_type == "content"`. Suggests `two_column`.
3. **Consecutive bullet slides (lines 2139–2143):** Warn if current AND previous slide_type are both `"content"`. Suggests `stat_callout`, `highlight`, or `key_metrics`.
4. **Output (lines 2147–2151):** Warnings printed to stdout with `⚠` prefix. **Never blocks the build.**
5. **No validation for:** unknown slide types (lines 2174–2185, just a stdout warning in orchestrator), empty required fields, color name typos, max item counts (those are silently capped via slicing).

---

## Color Vocabulary (hex resolution, lines 25–35, 78–82)

| Name | Hex | Used For |
|------|-----|----------|
| `blue` | `#003766` | Primary brand. Titles, headers, default card BG, takeaway text |
| `orange` | `#FE6A37` | Accent lines, in-progress, near-miss |
| `purple` | `#54206F` | Section dividers (alt), accent |
| `dark_gray` | `#383B3D` | Body text default |
| `gray` | `#6F6F6F` | Sublabels, footnotes, timeline lines |
| `teal` | `#81C6BD` | Infrastructure category, fallback for unknown color names |
| `white` | `#FFFFFF` | Text on dark backgrounds |
| `light_gray` | `#B7BABA` | Subdued text, sublabel on dark backgrounds |
| `red` | `#CC3333` | Critical findings, attacker indicators |
| `green` | `#339966` | Positive status, resolved |
| `light_blue` | `#E8F0F8` | Takeaway bar BG, highlight box BG |

**Internal-only colors** (not in `COLOR_MAP`, hard-coded in builders):
- `#F2F2F2` (table even rows), `#F5F7FA` (card BG default), `#FFF3ED` (NEARMISS_BG), `#E8F5E9` (LIGHT_GREEN_BG), `#FFF3ED` (LIGHT_ORANGE_BG), `#E0E0E0` (BAR_BG), `#336699` (DIV_BLUE), `#BBCCDD` (sublabel on blue BG).

---

## Layout Constants (lines 39–60)

- **Slide dimensions:** Inherited from 1898 template (13.33" × 7.5" widescreen, 16:9).
- **Default left margin (L):** `0.67"`.
- **Body width (BW):** `11.8"` (`L + BW = 12.47"` → right margin `0.86"`).
- **Title band:** `(0.67", 0.54")` × `(11.8", 0.5")` — title bar at top.
- **Body start (short):** `BT_SHORT = 1.5"`. (Body start long `BT_LONG = 2.1"` defined but not used by short layouts.)
- **Body heights:** `BH_SHORT = 5.0"` (no takeaway), `BH_LONG = 4.5"` (defined but not used). When takeaway present, body compressed to `4.2"` in most builders.
- **Takeaway bar:** `(0.67", 6.1")` × `(11.8", 0.5")` fill `LIGHT_BLUE`. Text inset `(0.9", 6.15")` × `(11.3", 0.4")`.
- **Footer band:** Reserved by template; not drawn by builder code.

### Font sizes
| Constant | Value |
|---|---|
| `TITLE_SIZE` | 24 |
| `SUBTITLE_SIZE` | 20 |
| `BODY_SIZE` | 18 |
| `HEADER_SIZE` | 18 |
| `SMALL_SIZE` | 16 |
| `BADGE_SIZE` | 14 |
| `STAT_SIZE` | 44 |
| `STAT_LABEL_SIZE` | 16 |
| `HIGHLIGHT_SIZE` | 20 |
| `TAKEAWAY_SIZE` | 16 |

### Layout indices (PPTX template, line 65–76)
`BLANK=0, BORDER_BLANK=1, DIVIDER_PURPLE=4, DIVIDER_BLUE=5, SHORT_ONE=11, SHORT_TWO=12, LONG_ONE=13, LONG_TWO=14, END_WHITE=18, END_PURPLE=19, END_BLUE=20`.

### Talk-track speaker-notes formatting
- Plain string assignment to `notes_slide.notes_text_frame.text` in orchestrator (lines 2179–2183).
- No structured formatting, no prefix, no auto-wrap markers. Speaker notes are raw freeform text.

---

## Behaviors NOT Captured in presentation-system.md

1. **`resolve_color` silent fallback to TEAL** for unknown strings (line 88) — the spec lists 11 valid names but doesn't mention what happens on typo. Tested by typoing "blu": you get teal cards.
2. **`build_title_slide` uses hard-coded 32pt** instead of any named constant (line 242). Spec says titles are 24pt; section dividers are 32pt — only the code reveals this.
3. **`_parse_bullets` strips ALL asterisks** via `b.strip("*")`, not just the leading/trailing pair (line 224). `"**bold ** mid**"` becomes `"bold ** mid"` — wait, no: `str.strip("*")` only removes leading/trailing. But mid-string `*` is preserved. Verify: `"**hello**".strip("*")` → `"hello"` ✓.
4. **`build_horizontal_timeline_slide` alternates by index parity (i%2==0)** — even indices above, odd below. Spec says "alternating" but doesn't define which starts above.
5. **`build_enhanced_table` caps at 12 rows** in code (`events[:12]`) but spec says max 10.
6. **`build_stat_callout` 7+ stats clamps cols to 4** (line 409) but spec says max 6.
7. **`build_metric_tree` chip min-cap of 4** regardless of available height (line 1809) — chips will overflow if 4 truly don't fit.
8. **`build_content_stat` single-stat font size is a 3-way auto switch** at len ≤6, ≤14, >14 OR has `\n` (lines 695–718). Spec says "auto-size value based on content" without giving the exact thresholds.
9. **`build_status` summary stat cards use 13pt for label**, not BADGE_SIZE (line 372).
10. **`stats_summary` summary band BG auto-defaults to LIGHT_GREEN_BG** (`#E8F5E9`) regardless of `summary["color"]` (line 1252). Spec implies bg derives from color; reality is hard-coded green tint.
11. **`build_highlight_boxes` caps to 2 boxes** via `boxes[:2]` (line 910). Spec says "two stacked" but doesn't say what happens with 3+.
12. **`takeaway_align` and `takeaway_size` keys** accepted on every takeaway-supporting slide; never documented in the spec.
13. **`enhanced_table` detail-merge into target column** as `"{target} ({detail})"` (line 1692). Spec doesn't mention `detail` parameter for tables (only for vertical_timeline).
14. **`build_progress_bar` recolors text** to GREEN for resolved label and ORANGE for in-progress label — those colors are not configurable.
15. **`build_weighted_composite` percentage label uses RAW weight**, not normalized weight (line 2013). If weights sum to 200, segments are halved but the label still says original. The legend also shows raw `f"{weight}% weight"`.
16. **`build_metric_tree` cap of 5 categories** via `categories[:5]` (line 1743) — spec says "max 5" but the cap is silent.
17. **`build_horizontal_timeline` connector lines are different** for above (from card_bottom going down) vs below (from bar_bottom going down) — both 0.15" tall but at different anchor points (lines 1547–1556).
18. **Talk-track injection happens in orchestrator, not builders** (lines 2179–2183), so every slide type (including `end`) supports `talk_track` even when not documented.

---

## Quirks the Rust Implementation Should NOT Carry Forward

1. **String-prefix bold notation `"**header**"`** (line 223): magic prefix that mutates rendering. Use a dedicated DSL element (e.g., a `header:` block or distinct `Header` IR variant).
2. **Dict-mutation/aliasing in `build_key_metrics_slide`** (lines 588–590): copies dict, renames key. Aliases should be resolved at parse time into a canonical IR, not at render time.
3. **Silent fallback to TEAL** for unknown colors (line 88): the Rust implementation should error or warn loudly. Typos are bugs.
4. **Silent fallback to PURPLE** in `build_title_slide` for any non-"blue" string (line 236): same problem — `"orange"` silently becomes purple.
5. **Silent fallback to BLUE end-slide layout** for unknown end color (line 2074): same.
6. **Silent slicing/capping** (`events[:8]`, `boxes[:2]`, `top_stats[:n_top]`, `categories[:5]`, `events[:6]`, `events[:12]`): the IR should validate ranges at compile time; truncation is a bug.
7. **Hard-coded font size 32pt in title slide** (line 242): magic numbers scattered through builders. Rust should have a single design-tokens constant table.
8. **Inconsistent caps:** spec says 6/10 but code does 7/12 in some places (stat_callout, enhanced_table). The IR should have one source of truth.
9. **Mutation of `slide_data` via `dict(slide_data)` then `pop`** in key_metrics alias: messy. Aliases resolved at lex/parse stage.
10. **The two-space prefix `"  •  "` on bullets** built into `add_rich_text` via string concatenation (line 149): bullets should be a structural marker, not embedded characters.
11. **Hard-coded color recoloring** (e.g., progress_bar labels forced to GREEN/ORANGE, status summary stat cards at 13pt not BADGE_SIZE): visual contract should be expressible in tokens, not buried in builders.
12. **`takeaway_align`/`takeaway_size` undocumented escape hatches** — either lift to first-class IR fields or remove. Don't keep undocumented behavior.
13. **Plain `str.strip("*")`** for bold-header detection — fragile parsing. Use a proper lexer.
14. **`build_end_slide` returns a slide that has zero content** but is still considered a "slide" for talk_track injection. End-slide-with-talk-track is a weird edge case the spec doesn't address.
15. **Unused constants `BT_LONG`, `BH_LONG`, `LAYOUT_LONG_ONE`, `LAYOUT_LONG_TWO`, `LAYOUT_BLANK`, `LAYOUT_BORDER_BLANK`** (lines 40–73) — dead code that suggests future plans never realized. Don't port until needed.
16. **Per-builder color constants defined locally** (`CARD_BG_DEFAULT`, `NEARMISS_BG`, `LIGHT_GREEN_BG`, etc. each declared inside each builder) — should be global tokens.

---

## Open Questions for Architect

1. **Should "unknown color" be an error or a silent fallback?** Recommend error in the Rust IR; spec doesn't specify but production data is presumably typo-free.
2. **What is the canonical max for `enhanced_table` and `stat_callout`** — code says 12/unlimited, spec says 10/6. Pick one and enforce.
3. **`takeaway_align` and `takeaway_size`** — promote to first-class spec or drop?
4. **`build_end_slide` + talk_track** — is a content-less slide with speaker notes a real use case?
5. **Layout index numbers (4, 5, 11, 12, 18, 19, 20)** are template-specific. Should slideforge reproduce the exact template, ship its own, or be template-agnostic?
6. **Talk-track formatting:** Python is plain text. Should slideforge support markdown, paragraphs, structured cues (e.g., "[pause]" markers)?
7. **`weighted_composite` normalization with raw-label display** — bug or feature? If user gives weights `[50, 50, 50]`, segments are normalized to 33% each but labels say 50%. Recommend: error if `|sum - 100| > 1`.
8. **`_parse_bullets`** — should the Rust DSL keep ANY string-prefix notation, or require all formatting to be structural?
9. **Card-row column overflow** (`build_card_rows_slide` has no row cap): silent overflow off the slide or compile error?
10. **Are font sizes (24/20/18/16/14/etc.) truly "minimum" or "exact"?** Spec calls them "minimum for projected presentations" but code applies them exactly. Rust should pick one model.
11. **Should the validator block or warn?** Python only warns. Visual-contract regressions might warrant errors in Rust.
12. **"detail" parameter scope** — only documented for `vertical_timeline` but `enhanced_table` builder uses it too. Promote to standard event field?

---

## File Inventory Confirmation

- **`build-incident-brief.py`:** 2216 lines, 23 `build_X_slide` functions, 1 dispatcher (`SLIDE_BUILDERS`), 1 validator (`validate_slides`), 1 orchestrator (`build_presentation`), 1 CLI entry, 10 primitive/helper functions (`resolve_color`, `clear_all_placeholders`, `add_text_box`, `add_rich_text`, `add_table`, `add_rounded_rect`, `add_status_badge`, `_parse_bullets`, `_takeaway_align`, `_add_takeaway`).
- **`presentation-system.md`:** 1025 lines, 10 top-level sections (Quick Start, Architecture, Slide Types Reference [23 subsections 3.1–3.23], Choosing the Right Slide Type, Design Rules, Data Format Reference, Shared Features, Content Validation, Adding New Slide Types, Troubleshooting). 235 `^##` headings counted.
- **`mss_metrics_leadership.py`:** 25 slides, 11 distinct slide types in use: `title` (5x), `numbered_actions` (1x), `highlight` (3x), `formula` (1x), `weighted_composite` (1x), `enhanced_table` (5x), `metric_tree` (2x), `content_stat` (5x), `severity_cards` (1x), `end` (1x). Notable absences from this real-world deck: `content`, `two_column`, `table`, `status`, `stat_callout`, `stats_summary`, `highlight_boxes`, `split_contrast`, `progress_bar`, `card_rows`, `vertical_timeline`, `horizontal_timeline`, `key_metrics`. Suggests the production "leadership-grade" deck leans heavily on `enhanced_table` and `content_stat` over plain bullets.
- **`_template.py`:** 155 lines, demonstrates 6 slide types (title, content, two_column, table, status, end) with minimal viable data shapes.
