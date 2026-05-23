#!/usr/bin/env python3
"""Build 1898-branded incident brief presentation from case data.

Generic incident brief builder that reads structured data and produces
a branded PowerPoint presentation using the 1898 template.

Usage:
    uv run python scripts/build-incident-brief.py <incident-data-module>

The incident data module must define a SLIDES list and METADATA dict.
See scripts/incident_data/ for examples.
"""

import importlib
import sys
from pathlib import Path

from pptx import Presentation
from pptx.util import Inches, Pt, Emu
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.enum.shapes import MSO_SHAPE

# ── 1898 Brand ──────────────────────────────────────────────────────
ORANGE = RGBColor(0xFE, 0x6A, 0x37)
BLUE = RGBColor(0x00, 0x37, 0x66)
PURPLE = RGBColor(0x54, 0x20, 0x6F)
DARK_GRAY = RGBColor(0x38, 0x3B, 0x3D)
GRAY = RGBColor(0x6F, 0x6F, 0x6F)
TEAL = RGBColor(0x81, 0xC6, 0xBD)
WHITE = RGBColor(0xFF, 0xFF, 0xFF)
LIGHT_GRAY = RGBColor(0xB7, 0xBA, 0xBA)
RED = RGBColor(0xCC, 0x33, 0x33)
GREEN = RGBColor(0x33, 0x99, 0x66)
LIGHT_BLUE = RGBColor(0xE8, 0xF0, 0xF8)
FONT = "Trebuchet MS"

# ── Layout content zones ────────────────────────────────────────────
L = Inches(0.67)
BT_SHORT = Inches(1.5)
BT_LONG = Inches(2.1)
BW = Inches(11.8)
BH_SHORT = Inches(5.0)
BH_LONG = Inches(4.5)

# Font sizes (minimum for projected presentations)
TITLE_SIZE = 24
SUBTITLE_SIZE = 20
BODY_SIZE = 18          # Body text minimum per voice guidelines
HEADER_SIZE = 18        # Section headers within slides
SMALL_SIZE = 16
BADGE_SIZE = 14
STAT_SIZE = 44          # Large stat numbers
STAT_LABEL_SIZE = 16    # Stat labels
HIGHLIGHT_SIZE = 20     # Highlight/callout text
TAKEAWAY_SIZE = 16      # Key takeaway at bottom

# Content validation
MAX_BULLET_CHARS = 85   # Max chars per bullet at BODY_SIZE before overflow warning
MAX_BULLETS = 6         # Max bullets per single-column slide

# Template path
TEMPLATE = Path(__file__).parent.parent / "templates" / "1898-pptx" / "1898-Presentation-Template-V3.0-2026.pptx"

# ── Layout indices (from template inspection) ───────────────────────
LAYOUT_BLANK = 0
LAYOUT_BORDER_BLANK = 1
LAYOUT_DIVIDER_PURPLE = 4
LAYOUT_DIVIDER_BLUE = 5
LAYOUT_SHORT_ONE = 11
LAYOUT_SHORT_TWO = 12
LAYOUT_LONG_ONE = 13
LAYOUT_LONG_TWO = 14
LAYOUT_END_WHITE = 18
LAYOUT_END_PURPLE = 19
LAYOUT_END_BLUE = 20

COLOR_MAP = {"green": GREEN, "red": RED, "orange": ORANGE,
             "blue": BLUE, "teal": TEAL, "gray": GRAY,
             "purple": PURPLE, "white": WHITE,
             "light_blue": LIGHT_BLUE, "light_gray": LIGHT_GRAY,
             "dark_gray": DARK_GRAY}


def resolve_color(color):
    """Resolve a color string or RGBColor to RGBColor."""
    if isinstance(color, str):
        return COLOR_MAP.get(color, TEAL)
    return color


# ── Placeholder management ──────────────────────────────────────────

def clear_all_placeholders(slide):
    """Remove ALL placeholder XML elements so default text doesn't show."""
    to_remove = list(slide.placeholders)
    for ph in to_remove:
        sp = ph._element
        sp.getparent().remove(sp)


# ── Primitive drawing helpers ───────────────────────────────────────

def add_text_box(slide, left, top, width, height, text, font_size=BODY_SIZE,
                 bold=False, color=DARK_GRAY, alignment=PP_ALIGN.LEFT):
    """Add a simple text box to a slide."""
    txBox = slide.shapes.add_textbox(left, top, width, height)
    tf = txBox.text_frame
    tf.word_wrap = True
    p = tf.paragraphs[0]
    p.text = text
    p.font.size = Pt(font_size)
    p.font.bold = bold
    p.font.color.rgb = color
    p.font.name = FONT
    p.alignment = alignment
    return txBox


def add_rich_text(slide, left, top, width, height, paragraphs):
    """Add a text box with mixed formatting."""
    txBox = slide.shapes.add_textbox(left, top, width, height)
    tf = txBox.text_frame
    tf.word_wrap = True

    for i, pdata in enumerate(paragraphs):
        p = tf.paragraphs[0] if i == 0 else tf.add_paragraph()
        p.font.name = FONT
        p.font.size = Pt(pdata.get("font_size", BODY_SIZE))
        p.font.bold = pdata.get("bold", False)
        p.font.color.rgb = pdata.get("color", DARK_GRAY)
        p.alignment = pdata.get("alignment", PP_ALIGN.LEFT)
        if pdata.get("space_after"):
            p.space_after = Pt(pdata["space_after"])
        if pdata.get("space_before"):
            p.space_before = Pt(pdata["space_before"])

        if "runs" in pdata:
            for rd in pdata["runs"]:
                run = p.add_run()
                run.text = rd["text"]
                run.font.name = FONT
                run.font.size = Pt(rd.get("font_size", pdata.get("font_size", BODY_SIZE)))
                run.font.bold = rd.get("bold", pdata.get("bold", False))
                run.font.color.rgb = rd.get("color", pdata.get("color", DARK_GRAY))
        else:
            text = pdata.get("text", "")
            if pdata.get("bullet"):
                text = f"  \u2022  {text}"
            p.text = text
    return txBox


def add_table(slide, left, top, width, rows_data, col_widths=None):
    """Add a branded table to a slide."""
    n_rows = len(rows_data)
    n_cols = len(rows_data[0]) if rows_data else 0
    if n_rows == 0 or n_cols == 0:
        return None

    row_height = Inches(0.35)
    table_height = row_height * n_rows
    shape = slide.shapes.add_table(n_rows, n_cols, left, top, width, table_height)
    table = shape.table

    if col_widths:
        for i, w in enumerate(col_widths):
            table.columns[i].width = w

    for r, row in enumerate(rows_data):
        for c, cell_text in enumerate(row):
            cell = table.cell(r, c)
            cell.text = str(cell_text)
            para = cell.text_frame.paragraphs[0]
            para.font.name = FONT
            para.font.size = Pt(SMALL_SIZE)
            para.font.color.rgb = WHITE if r == 0 else DARK_GRAY
            para.font.bold = r == 0

            if r == 0:
                cell.fill.solid()
                cell.fill.fore_color.rgb = BLUE
            elif r % 2 == 0:
                cell.fill.solid()
                cell.fill.fore_color.rgb = RGBColor(0xF2, 0xF2, 0xF2)
            else:
                cell.fill.solid()
                cell.fill.fore_color.rgb = WHITE

    return shape


def add_rounded_rect(slide, left, top, width, height, fill_color):
    """Add a rounded rectangle shape."""
    shape = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, left, top, width, height)
    shape.fill.solid()
    shape.fill.fore_color.rgb = fill_color
    shape.line.fill.background()
    return shape


def add_status_badge(slide, left, top, width, height, text, color):
    """Add a colored status badge with text."""
    shape = add_rounded_rect(slide, left, top, width, height, resolve_color(color))
    tf = shape.text_frame
    tf.word_wrap = True
    p = tf.paragraphs[0]
    p.text = text
    p.font.size = Pt(BADGE_SIZE)
    p.font.bold = True
    p.font.color.rgb = WHITE
    p.font.name = FONT
    p.alignment = PP_ALIGN.CENTER
    return shape


def _parse_bullets(bullets, default_size=BODY_SIZE):
    """Convert bullet list to paragraph data. Shared by content and column builders."""
    paras = []
    for b in bullets:
        if isinstance(b, dict):
            paras.append(b)
        elif isinstance(b, str) and b.startswith("**"):
            text = b.strip("*")
            paras.append({"text": text, "bold": True, "font_size": HEADER_SIZE,
                          "color": BLUE, "space_before": 6, "space_after": 2})
        else:
            paras.append({"text": b, "bullet": True, "font_size": default_size, "space_after": 3})
    return paras


# ── Slide builders ──────────────────────────────────────────────────

def build_title_slide(prs, slide_data):
    """Section divider slide (blue or purple background)."""
    layout_idx = LAYOUT_DIVIDER_BLUE if slide_data.get("color", "blue") == "blue" else LAYOUT_DIVIDER_PURPLE
    slide = prs.slides.add_slide(prs.slide_layouts[layout_idx])
    clear_all_placeholders(slide)

    add_text_box(slide, Inches(0.67), Inches(2.0), Inches(12.0), Inches(1.5),
                 slide_data.get("title", ""), font_size=32, bold=True,
                 color=WHITE, alignment=PP_ALIGN.LEFT)
    subtitle = slide_data.get("subtitle", "")
    if subtitle:
        add_text_box(slide, Inches(0.67), Inches(3.5), Inches(12.0), Inches(1.0),
                     subtitle, font_size=SUBTITLE_SIZE, color=LIGHT_GRAY)
    return slide


def build_content_slide(prs, slide_data):
    """Standard content slide: title + bullets.

    Optional: "takeaway" key adds a bold key-takeaway line at bottom.
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    bullets = slide_data.get("bullets", [])
    takeaway = slide_data.get("takeaway", "")

    # Adjust body height if takeaway present
    body_h = Inches(4.2) if takeaway else BH_SHORT

    if bullets:
        add_rich_text(slide, L, BT_SHORT, BW, body_h, _parse_bullets(bullets))

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_two_column_slide(prs, slide_data):
    """Two-column content slide with optional takeaway."""
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_TWO])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    takeaway = slide_data.get("takeaway", "")
    col_h = Inches(4.2) if takeaway else BH_SHORT
    col_w = Inches(5.8)

    left_col = slide_data.get("left", [])
    if left_col:
        add_rich_text(slide, L, BT_SHORT, col_w, col_h, _parse_bullets(left_col))

    right_col = slide_data.get("right", [])
    if right_col:
        add_rich_text(slide, Inches(6.86), BT_SHORT, col_w, col_h, _parse_bullets(right_col))

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_table_slide(prs, slide_data):
    """Slide with title, table, and optional footnote."""
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    table_data = slide_data.get("table", [])
    if table_data:
        add_table(slide, L, BT_SHORT, BW, table_data, slide_data.get("col_widths"))

    footnote = slide_data.get("footnote", "")
    if footnote:
        n_rows = len(table_data)
        footnote_top = BT_SHORT + Inches(0.35) * n_rows + Inches(0.2)
        add_text_box(slide, L, footnote_top, BW, Inches(0.5),
                     footnote, font_size=BADGE_SIZE, color=GRAY)

    return slide


def build_status_slide(prs, slide_data):
    """Status dashboard with colored badges.

    Optional enhancements:
    - summary_stats: list of {"value": "4 of 6", "label": "areas resolved", "color": "green"}
      Renders stat cards below the badges.
    - takeaway: bottom bar text
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    statuses = slide_data.get("statuses", [])
    summary_stats = slide_data.get("summary_stats", [])
    takeaway = slide_data.get("takeaway", "")

    badge_w = Inches(5.5)
    badge_h = Inches(0.38)
    spacing = Inches(0.48) if (summary_stats or takeaway) else Inches(0.55)
    y = BT_SHORT

    for s in statuses:
        add_text_box(slide, L, y, Inches(5.0), badge_h,
                     s.get("label", ""), font_size=BODY_SIZE, bold=True, color=DARK_GRAY)
        add_status_badge(slide, Inches(6.0), y, badge_w, badge_h,
                         s.get("value", ""), s.get("color", TEAL))
        y += spacing

    # Optional summary stats below badges
    if summary_stats:
        n_stats = len(summary_stats)
        stat_gap = Inches(0.4)
        total_stat_gap = stat_gap * (n_stats - 1)
        stat_w = (BW - total_stat_gap) / n_stats
        stat_h = Inches(1.1)
        stat_y = y + Inches(0.2)

        for i, ss in enumerate(summary_stats):
            sx = L + (stat_w + stat_gap) * i
            color = resolve_color(ss.get("color", "blue"))
            shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, sx, stat_y, stat_w, stat_h)
            shape.fill.solid(); shape.fill.fore_color.rgb = color; shape.line.fill.background()
            add_text_box(slide, sx, stat_y + Inches(0.1), stat_w, Inches(0.55),
                         str(ss.get("value", "")), font_size=30, bold=True,
                         color=WHITE, alignment=PP_ALIGN.CENTER)
            add_text_box(slide, sx + Inches(0.1), stat_y + Inches(0.65), stat_w - Inches(0.2), Inches(0.35),
                         ss.get("label", ""), font_size=13, color=WHITE,
                         alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_stat_callout_slide(prs, slide_data):
    """Stat callout slide: 2-6 stats displayed prominently.

    stats: list of {"value": "26", "label": "Exfiltration processes",
                    "sublabel": "against 0 baseline", "color": "blue"}
    style: "cards" (colored card backgrounds, default) or "plain" (colored numbers, no background)
    context: optional centered text below the stats
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    stats = slide_data.get("stats", [])
    n = len(stats)
    if n == 0:
        return slide

    style = slide_data.get("style", "cards")  # "cards" (blue + orange accent), "light" (white + left border), "plain" (no background)
    context = slide_data.get("context", "")

    # Grid layout
    if n <= 4:
        cols = n
    elif n <= 6:
        cols = 3
    else:
        cols = 4
    rows = (n + cols - 1) // cols

    gap = Inches(0.3)
    total_gap = gap * (cols - 1)
    card_w = (BW - total_gap) / cols
    card_h = Inches(2.6)
    accent_h = Inches(0.06)
    border_w = Inches(0.06)

    total_cards_h = card_h * rows + gap * (rows - 1)
    available_h = Inches(4.0) if context else Inches(4.8)
    start_y = BT_SHORT + (available_h - total_cards_h) / 2

    for idx, s in enumerate(stats):
        r = idx // cols
        c = idx % cols
        x = L + (card_w + gap) * c
        y = start_y + (card_h + gap) * r
        color = resolve_color(s.get("color", "blue"))
        pad = Inches(0.1)
        inner_w = card_w - Inches(0.2)
        val_size = 40 if n <= 4 else 28

        if style == "light":
            # Light card with colored left border stripe
            CARD_BG_L = RGBColor(0xF5, 0xF7, 0xFA)
            shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, y, card_w, card_h)
            shape.fill.solid()
            shape.fill.fore_color.rgb = CARD_BG_L
            shape.line.fill.background()
            # Left border stripe
            stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, y, border_w, card_h)
            stripe.fill.solid()
            stripe.fill.fore_color.rgb = color
            stripe.line.fill.background()
            # Icon + label header (if icon provided) or just value
            icon = s.get("icon", "")  # "check", "arrow", or unicode char
            if icon:
                # Icon + label as a header row at top
                icon_char = "\u2713" if icon == "check" else "\u25B6" if icon == "arrow" else icon
                icon_color_r = resolve_color(s.get("icon_color", color))
                add_text_box(slide, x + Inches(0.15), y + Inches(0.15), Inches(0.35), Inches(0.35),
                             icon_char, font_size=18, bold=True, color=icon_color_r)
                add_text_box(slide, x + Inches(0.5), y + Inches(0.15), inner_w - Inches(0.4), Inches(0.35),
                             s.get("label", ""), font_size=SMALL_SIZE, bold=True, color=BLUE)
                # Value as body text below
                add_text_box(slide, x + Inches(0.15), y + Inches(0.6), inner_w, Inches(0.8),
                             str(s.get("value", "")), font_size=13, color=DARK_GRAY)
                # Sublabel
                sublabel = s.get("sublabel", "")
                if sublabel:
                    add_text_box(slide, x + Inches(0.15), y + Inches(1.1), inner_w, Inches(0.4),
                                 sublabel, font_size=11, color=GRAY)
            else:
                # Standard light layout: big value centered
                add_text_box(slide, x + Inches(0.15), y + Inches(0.3), inner_w, Inches(1.0),
                             str(s.get("value", "")), font_size=val_size, bold=True,
                             color=color, alignment=PP_ALIGN.CENTER)
                add_text_box(slide, x + pad, y + Inches(1.3), inner_w, Inches(0.6),
                             s.get("label", ""), font_size=BADGE_SIZE, bold=True,
                             color=DARK_GRAY, alignment=PP_ALIGN.CENTER)
                sublabel = s.get("sublabel", "")
                if sublabel:
                    add_text_box(slide, x + pad, y + Inches(1.85), inner_w, Inches(0.4),
                                 sublabel, font_size=11, bold=False,
                                 color=GRAY, alignment=PP_ALIGN.CENTER)

            # Status indicator: checkmark or x in top-right corner
            status = s.get("status", "")
            if status:
                indicator = "\u2713" if status == "pass" else "\u2717"
                ind_color = GREEN if status == "pass" else RED
                add_text_box(slide, x + card_w - Inches(0.5), y + Inches(0.1),
                             Inches(0.4), Inches(0.4),
                             indicator, font_size=20, bold=True, color=ind_color,
                             alignment=PP_ALIGN.RIGHT)

        elif style == "plain":
            # No background, colored numbers
            add_text_box(slide, x, y + Inches(0.1), card_w, Inches(1.0),
                         str(s.get("value", "")), font_size=STAT_SIZE, bold=True,
                         color=color, alignment=PP_ALIGN.CENTER)
            add_text_box(slide, x, y + Inches(1.1), card_w, Inches(0.5),
                         s.get("label", ""), font_size=STAT_LABEL_SIZE, bold=False,
                         color=DARK_GRAY, alignment=PP_ALIGN.CENTER)
            sublabel = s.get("sublabel", "")
            if sublabel:
                add_text_box(slide, x, y + Inches(1.55), card_w, Inches(0.4),
                             sublabel, font_size=12, bold=False,
                             color=GRAY, alignment=PP_ALIGN.CENTER)

        else:
            # Default "cards" style: colored card with optional accent top
            # Accent bar: per-stat "accent" color, slide-level "accent" default, or None to disable
            accent_color = s.get("accent", slide_data.get("accent", None))
            if accent_color is not None:
                accent_color = resolve_color(accent_color)
                acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, y, card_w, accent_h)
                acc.fill.solid()
                acc.fill.fore_color.rgb = accent_color
                acc.line.fill.background()
            # Card body (offset by accent if present)
            body_offset = accent_h if accent_color is not None else 0
            body = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, y + body_offset, card_w, card_h - body_offset)
            body.fill.solid()
            body.fill.fore_color.rgb = color
            body.line.fill.background()
            # Value
            add_text_box(slide, x + pad, y + Inches(0.35), inner_w, Inches(1.0),
                         str(s.get("value", "")), font_size=val_size, bold=True,
                         color=WHITE, alignment=PP_ALIGN.CENTER)
            # Label
            add_text_box(slide, x + pad, y + Inches(1.3), inner_w, Inches(0.6),
                         s.get("label", ""), font_size=BADGE_SIZE, bold=True,
                         color=WHITE, alignment=PP_ALIGN.CENTER)
            # Sublabel
            sublabel = s.get("sublabel", "")
            if sublabel:
                add_text_box(slide, x + pad, y + Inches(1.85), inner_w, Inches(0.4),
                             sublabel, font_size=11, bold=False,
                             color=LIGHT_GRAY, alignment=PP_ALIGN.CENTER)

    if context:
        ctx_top = start_y + total_cards_h + Inches(0.3)
        add_text_box(slide, L, ctx_top, BW, Inches(0.8),
                     context, font_size=SMALL_SIZE, color=GRAY, alignment=PP_ALIGN.CENTER)

    return slide


def build_highlight_slide(prs, slide_data):
    """Highlight/callout slide: a single key statement with optional supporting text.

    Use for the most important takeaway that deserves its own slide.
    Supports optional takeaway bar at the bottom — when present, the layout
    compresses to give the takeaway room without crowding the supporting bullets.
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    takeaway = slide_data.get("takeaway", "")

    # Layout zones: compress when takeaway is present so all three (highlight box,
    # supporting bullets, takeaway) get proper breathing room. Otherwise use the
    # original spacious layout.
    if takeaway:
        box_top = Inches(1.5)
        box_h = Inches(1.8)
        sup_top = Inches(3.6)
        sup_h = Inches(2.3)
    else:
        box_top = Inches(2.0)
        box_h = Inches(2.0)
        sup_top = Inches(4.4)
        sup_h = Inches(2.2)

    highlight = slide_data.get("highlight", "")
    if highlight:
        add_rounded_rect(slide, Inches(0.9), box_top, Inches(11.4), box_h, LIGHT_BLUE)
        add_text_box(slide, Inches(1.2), box_top + Inches(0.3), Inches(10.8), box_h - Inches(0.6),
                     highlight, font_size=HIGHLIGHT_SIZE, bold=True, color=BLUE,
                     alignment=PP_ALIGN.LEFT)

    supporting = slide_data.get("supporting", [])
    if supporting:
        add_rich_text(slide, L, sup_top, BW, sup_h, _parse_bullets(supporting))

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_key_metrics_slide(prs, slide_data):
    """Alias for stat_callout. Accepts 'metrics' or 'stats' key."""
    if "metrics" in slide_data and "stats" not in slide_data:
        slide_data = dict(slide_data)
        slide_data["stats"] = slide_data.pop("metrics")
    return build_stat_callout_slide(prs, slide_data)


def build_content_stat_slide(prs, slide_data):
    """Content slide with info blocks on the left and a stat anchor on the right.

    blocks: list of {"header": "Bold Title", "description": "Supporting text"}
    bullets: list of bullet strings (alternative to blocks, uses standard bullets)
    stat: {"value": "10,000+", "label": "organizations affected", "color": "blue"}
    takeaway: optional bottom bar
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    takeaway = slide_data.get("takeaway", "")
    body_h = Inches(4.2) if takeaway else BH_SHORT
    left_w = Inches(7.0)

    # Left side: blocks (header + description) or bullets
    blocks = slide_data.get("blocks", [])
    bullets = slide_data.get("bullets", [])

    if blocks:
        paras = []
        for b in blocks:
            paras.append({"text": b.get("header", ""), "bold": True,
                          "font_size": BODY_SIZE, "color": BLUE, "space_after": 2})
            paras.append({"text": b.get("description", ""),
                          "font_size": SMALL_SIZE, "color": DARK_GRAY, "space_after": 10})
        add_rich_text(slide, L, BT_SHORT, left_w, body_h, paras)
    elif bullets:
        add_rich_text(slide, L, BT_SHORT, left_w, body_h, _parse_bullets(bullets))

    # Right side: stat anchor (single stat or stacked rows)
    stat = slide_data.get("stat", {})
    stat_rows = slide_data.get("stat_rows", [])
    stat_x = Inches(8.2)
    stat_w = Inches(4.2)
    DIV_BLUE = RGBColor(0x33, 0x66, 0x99)

    if stat_rows:
        # Stacked rows with dividers: list of {"value": "38min", "label": "to first disable"}
        color = resolve_color(slide_data.get("stat_color", "blue"))
        accent_color = slide_data.get("stat_accent", None)

        card_top = Inches(1.6)
        card_h = Inches(4.2)
        accent_h = Inches(0.06)

        # Optional accent bar
        if accent_color is not None:
            accent_rgb = resolve_color(accent_color)
            acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, stat_x, card_top, stat_w, accent_h)
            acc.fill.solid(); acc.fill.fore_color.rgb = accent_rgb; acc.line.fill.background()
            body_offset = accent_h
        else:
            body_offset = 0

        # Card body
        body = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, stat_x, card_top + body_offset,
                                       stat_w, card_h - body_offset)
        body.fill.solid(); body.fill.fore_color.rgb = color; body.line.fill.background()

        n_rows = len(stat_rows)
        row_h = (card_h - body_offset) / n_rows

        for i, sr in enumerate(stat_rows):
            iy = card_top + body_offset + row_h * i
            # Value (right-aligned)
            val = str(sr.get("value", ""))
            val_size = 28 if val != "\u2713" else 32
            add_text_box(slide, stat_x + Inches(0.15), iy + Inches(0.2),
                         Inches(1.7), Inches(0.7),
                         val, font_size=val_size, bold=True,
                         color=WHITE, alignment=PP_ALIGN.RIGHT)
            # Label (left-aligned, with gap)
            add_text_box(slide, stat_x + Inches(2.0), iy + Inches(0.25),
                         Inches(2.0), Inches(0.6),
                         sr.get("label", ""), font_size=BADGE_SIZE, bold=False,
                         color=WHITE, alignment=PP_ALIGN.LEFT)
            # Divider (except after last)
            if i < n_rows - 1:
                div_y = iy + row_h - Inches(0.01)
                div = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE,
                    stat_x + Inches(0.25), div_y, stat_w - Inches(0.5), Inches(0.03))
                div.fill.solid(); div.fill.fore_color.rgb = DIV_BLUE; div.line.fill.background()

    elif stat:
        # Single stat: big value + label. Auto-size value based on content:
        #   short numbers/symbols → 52pt (e.g., "26", "100%", "10K+")
        #   medium text         → 32pt (e.g., "67% Resolved")
        #   long or multi-line  → 28pt, multi-line allowed, label shifted down
        color = resolve_color(stat.get("color", "blue"))
        card_top = Inches(2.0)
        card_h = Inches(3.2)
        add_rounded_rect(slide, stat_x, card_top, stat_w, card_h, color)

        val = str(stat.get("value", ""))
        has_newline = "\n" in val
        total_len = len(val.replace("\n", " "))

        if has_newline or total_len > 14:
            # Long/multi-line value: vertically center value+label as a unit so
            # the card doesn't read hollow in the middle.
            val_font = 28
            val_lines = val.count("\n") + 1
            val_text_h = Inches(0.55 * val_lines)
            label_text_h = Inches(0.4)
            gap_h = Inches(0.25)
            total_content_h = val_text_h + gap_h + label_text_h
            top_margin = (card_h - total_content_h) / 2
            val_top = card_top + top_margin
            val_h = val_text_h + Inches(0.1)
            label_top = val_top + val_text_h + gap_h
        elif total_len > 6:
            val_font = 36
            val_top = card_top + Inches(0.45)
            val_h = Inches(1.3)
            label_top = card_top + Inches(1.85)
        else:
            # Original: short numbers/symbols at 52pt
            val_font = 52
            val_top = card_top + Inches(0.4)
            val_h = Inches(1.4)
            label_top = card_top + Inches(1.8)

        add_text_box(slide, stat_x, val_top, stat_w, val_h,
                     val, font_size=val_font, bold=True,
                     color=WHITE, alignment=PP_ALIGN.CENTER)

        add_text_box(slide, stat_x + Inches(0.2), label_top, stat_w - Inches(0.4), Inches(1.0),
                     stat.get("label", ""), font_size=STAT_LABEL_SIZE, bold=False,
                     color=WHITE, alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_card_rows_slide(prs, slide_data):
    """Two-column checklist with horizontal card rows.

    left/right: list of strings (card items)
    left_header/right_header: column header text
    left_color/right_color: left border color per column (default blue)
    takeaway: optional bottom bar
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    takeaway = slide_data.get("takeaway", "")
    CARD_BG = RGBColor(0xF5, 0xF7, 0xFA)

    left_x = L
    right_x = Inches(6.67)
    col_w = Inches(5.8)
    card_h = Inches(0.65)
    row_gap = Inches(0.15)
    border_w = Inches(0.06)

    # Left column
    left_header = slide_data.get("left_header", "")
    left_color = resolve_color(slide_data.get("left_color", "blue"))
    left_items = slide_data.get("left", [])

    if left_header:
        add_text_box(slide, left_x, Inches(1.4), col_w, Inches(0.4),
                     left_header, font_size=HEADER_SIZE, bold=True, color=BLUE)

    for i, item in enumerate(left_items):
        text = item if isinstance(item, str) else item.get("text", "")
        y = Inches(1.9) + (card_h + row_gap) * i
        # Card background
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, left_x, y, col_w, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = CARD_BG; shape.line.fill.background()
        # Left border stripe
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, left_x, y, border_w, card_h)
        stripe.fill.solid(); stripe.fill.fore_color.rgb = left_color; stripe.line.fill.background()
        # Text
        add_text_box(slide, left_x + Inches(0.2), y + Inches(0.12), col_w - Inches(0.35), Inches(0.4),
                     text, font_size=BADGE_SIZE, color=DARK_GRAY)

    # Right column
    right_header = slide_data.get("right_header", "")
    right_color = resolve_color(slide_data.get("right_color", "blue"))
    right_items = slide_data.get("right", [])

    if right_header:
        add_text_box(slide, right_x, Inches(1.4), col_w, Inches(0.4),
                     right_header, font_size=HEADER_SIZE, bold=True, color=BLUE)

    for i, item in enumerate(right_items):
        text = item if isinstance(item, str) else item.get("text", "")
        y = Inches(1.9) + (card_h + row_gap) * i
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, right_x, y, col_w, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = CARD_BG; shape.line.fill.background()
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, right_x, y, border_w, card_h)
        stripe.fill.solid(); stripe.fill.fore_color.rgb = right_color; stripe.line.fill.background()
        add_text_box(slide, right_x + Inches(0.2), y + Inches(0.12), col_w - Inches(0.35), Inches(0.4),
                     text, font_size=BADGE_SIZE, color=DARK_GRAY)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_severity_cards_slide(prs, slide_data):
    """Full-width severity cards with colored borders and severity badges.

    gaps: list of {"header": "...", "description": "...", "severity": "HIGH", "color": "red"}
    takeaway: optional bottom bar
    Card heights auto-scale based on item count and whether takeaway is present.
    Practical max ~6 items; beyond that cards get too thin for the description text.
    """
    CARD_BG = RGBColor(0xF5, 0xF7, 0xFA)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    gaps = slide_data.get("gaps", [])
    takeaway = slide_data.get("takeaway", "")
    n = len(gaps)

    gap_h = Inches(0.1)
    start_y = Inches(1.4)
    border_w = Inches(0.06)

    # Reserve bottom space for takeaway (brand-standard y=6.1, h=0.5)
    bottom_limit = Inches(5.95) if takeaway else Inches(6.5)
    available_h = bottom_limit - start_y

    # Auto-scale card height with sensible bounds
    if n > 0:
        card_h = (available_h - gap_h * (n - 1)) / n
        max_h = Inches(0.95)
        min_h = Inches(0.7)
        if card_h > max_h:
            card_h = max_h
        elif card_h < min_h:
            card_h = min_h
    else:
        card_h = Inches(0.9)

    # Scale text positions relative to card height so text fits as cards shrink.
    # Proportions: 8% top pad / 28% header / 4% gap / 38% desc / 22% bottom pad
    header_top = card_h * 0.08
    header_h = card_h * 0.28
    desc_top = card_h * 0.40
    desc_h = card_h * 0.38

    for i, g in enumerate(gaps):
        y = start_y + (card_h + gap_h) * i
        color = resolve_color(g.get("color", "red"))

        # Card background
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, BW, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = CARD_BG; shape.line.fill.background()

        # Left severity border
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, border_w, card_h)
        stripe.fill.solid(); stripe.fill.fore_color.rgb = color; stripe.line.fill.background()

        # Header
        add_text_box(slide, Inches(0.9), y + header_top, Inches(8.5), header_h,
                     g.get("header", ""), font_size=SMALL_SIZE, bold=True, color=BLUE)

        # Description
        add_text_box(slide, Inches(0.9), y + desc_top, Inches(8.5), desc_h,
                     g.get("description", ""), font_size=13, color=GRAY)

        # Severity badge (vertically centered in card)
        sev = g.get("severity", "")
        if sev:
            badge_w = Inches(1.5)
            badge_h = Inches(0.32)
            badge_x = Inches(10.7)
            badge_y = y + (card_h - badge_h) / 2
            add_rounded_rect(slide, badge_x, badge_y, badge_w, badge_h, color)
            add_text_box(slide, badge_x, badge_y + Inches(0.02), badge_w, badge_h - Inches(0.04),
                         sev, font_size=12, bold=True, color=WHITE, alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_highlight_boxes_slide(prs, slide_data):
    """Two stacked full-width highlight boxes, each with items on left and stat on right.

    boxes: list of 2 dicts, each with:
        header, items (list of strings), stat_value, stat_label,
        bg_color, accent_color (optional top accent), text_color, stat_color
    takeaway: optional bottom bar
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    boxes = slide_data.get("boxes", [])
    takeaway = slide_data.get("takeaway", "")

    box_h = Inches(2.1)
    box_gap = Inches(0.2)
    start_y = Inches(1.4)

    for idx, box in enumerate(boxes[:2]):
        y = start_y + (box_h + box_gap) * idx
        bg = resolve_color(box.get("bg_color", "blue"))
        text_color = resolve_color(box.get("text_color", "white"))
        stat_text_color = resolve_color(box.get("stat_color", "white"))

        # Box background
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, BW, box_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = bg; shape.line.fill.background()

        # Optional top accent
        accent = box.get("accent_color")
        if accent:
            acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, BW, Inches(0.06))
            acc.fill.solid(); acc.fill.fore_color.rgb = resolve_color(accent); acc.line.fill.background()

        # Header (with optional count on the right, inside the stat area)
        add_text_box(slide, Inches(0.9), y + Inches(0.15), Inches(5.5), Inches(0.4),
                     box.get("header", ""), font_size=SUBTITLE_SIZE, bold=True, color=text_color)
        header_count = box.get("header_count", "")
        if header_count:
            # Position count as small label above the stat value
            count_color = resolve_color(box.get("stat_color", text_color))
            add_text_box(slide, Inches(7.5), y + Inches(0.15), Inches(4.5), Inches(0.3),
                         header_count, font_size=13, bold=True, color=count_color,
                         alignment=PP_ALIGN.CENTER)

        # Items (with optional icon per item)
        items = box.get("items", [])
        item_icon = box.get("item_icon", "")  # "check", "arrow", or "" for none
        paras = []
        for item in items:
            text = item if isinstance(item, str) else item.get("text", "")
            if item_icon:
                icon = "\u2713" if item_icon == "check" else "\u25B6" if item_icon == "arrow" else ""
                text = f"{icon}  {text}"
            paras.append({"text": text, "font_size": BADGE_SIZE, "color": text_color, "space_after": 4})
        if paras:
            add_rich_text(slide, Inches(0.9), y + Inches(0.55), Inches(6.3), Inches(1.4), paras)

        # Stat on right (shift down if header_count is present)
        stat_val = box.get("stat_value", "")
        stat_lbl = box.get("stat_label", "")
        if stat_val:
            stat_offset = Inches(0.1) if header_count else 0
            lbl_color = RGBColor(0xBB, 0xCC, 0xDD) if bg == BLUE else GRAY
            add_text_box(slide, Inches(7.5), y + Inches(0.35) + stat_offset, Inches(4.5), Inches(0.8),
                         stat_val, font_size=36, bold=True, color=stat_text_color,
                         alignment=PP_ALIGN.CENTER)
            add_text_box(slide, Inches(7.5), y + Inches(1.15) + stat_offset, Inches(4.5), Inches(0.5),
                         stat_lbl, font_size=13, bold=False, color=lbl_color,
                         alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_numbered_actions_slide(prs, slide_data):
    """Numbered action cards in 1, 2, or 3 columns with color-coded categories.

    actions: list of {"text": "...", "category": "Security", "color": "orange"}
    columns: 1 (full-width stack), 2 (default), or 3 (dense). Items are distributed
        column-major: column 1 fills first, then column 2, etc. Numbering is global
        (1 .. n) following the data order.
    legend: list of {"label": "Security", "color": "orange"} (auto-derived if omitted)
    takeaway: optional bottom bar
    """
    CARD_BG = RGBColor(0xF5, 0xF7, 0xFA)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    actions = slide_data.get("actions", [])
    takeaway = slide_data.get("takeaway", "")
    n = len(actions)
    if n == 0:
        return slide

    columns = max(1, min(slide_data.get("columns", 2), 3))
    col_gap_x = Inches(0.17) if columns > 1 else Inches(0)
    card_w = (BW - col_gap_x * (columns - 1)) / columns
    card_h = Inches(0.55)
    gap = Inches(0.12)
    num_sz = Inches(0.38)

    # Distribute items column-major (fill col 0 first, then col 1, etc.)
    per_col = (n + columns - 1) // columns  # ceiling division
    col_lists = []
    for c in range(columns):
        start = c * per_col
        end = min(start + per_col, n)
        col_lists.append(actions[start:end])

    def draw_action(x, y, num, text, color):
        # Card background
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, y, card_w, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = CARD_BG; shape.line.fill.background()
        # Numbered circle
        circ = slide.shapes.add_shape(MSO_SHAPE.OVAL, x + Inches(0.1), y + Inches(0.085), num_sz, num_sz)
        circ.fill.solid(); circ.fill.fore_color.rgb = color; circ.line.fill.background()
        tf = circ.text_frame; p = tf.paragraphs[0]
        p.text = str(num); p.font.size = Pt(13); p.font.bold = True
        p.font.color.rgb = WHITE; p.font.name = FONT; p.alignment = PP_ALIGN.CENTER
        # Text
        add_text_box(slide, x + Inches(0.58), y + Inches(0.1), card_w - Inches(0.7), Inches(0.4),
                     text, font_size=BADGE_SIZE, color=DARK_GRAY)

    for c in range(columns):
        col_x = L + (card_w + col_gap_x) * c
        for i, a in enumerate(col_lists[c]):
            global_idx = c * per_col + i
            color = resolve_color(a.get("color", "blue"))
            y = Inches(1.5) + (card_h + gap) * i
            draw_action(col_x, y, global_idx + 1, a.get("text", ""), color)

    # Legend
    legend = slide_data.get("legend", [])
    if not legend:
        # Auto-derive from actions
        seen = {}
        for a in actions:
            cat = a.get("category", "")
            if cat and cat not in seen:
                seen[cat] = resolve_color(a.get("color", "blue"))
        legend = [{"label": k, "color": v} for k, v in seen.items()]

    if legend:
        legend_y = Inches(5.2)
        lx = L
        for item in legend:
            color = resolve_color(item.get("color", "blue"))
            dot = slide.shapes.add_shape(MSO_SHAPE.OVAL, lx, legend_y + Inches(0.05),
                                          Inches(0.18), Inches(0.18))
            dot.fill.solid(); dot.fill.fore_color.rgb = color; dot.line.fill.background()
            add_text_box(slide, lx + Inches(0.25), legend_y, Inches(1.5), Inches(0.3),
                         item.get("label", ""), font_size=12, color=GRAY)
            lx += Inches(1.8)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


_ALIGN_MAP = {"left": PP_ALIGN.LEFT, "center": PP_ALIGN.CENTER, "right": PP_ALIGN.RIGHT}


def _takeaway_align(slide_data):
    """Return alignment from slide_data['takeaway_align'], defaulting to CENTER."""
    return _ALIGN_MAP.get(slide_data.get("takeaway_align", "center"), PP_ALIGN.CENTER)


def _add_takeaway(slide, text, y=None, align=PP_ALIGN.CENTER, size=None):
    """Shared helper: add a takeaway bar at the bottom."""
    font_sz = size or TAKEAWAY_SIZE
    bar_top = y or Inches(6.1)
    add_rounded_rect(slide, L, bar_top, BW, Inches(0.5), LIGHT_BLUE)
    add_text_box(slide, Inches(0.9), bar_top + Inches(0.05), Inches(11.3), Inches(0.4),
                 text, font_size=font_sz, bold=True, color=BLUE,
                 alignment=align)


def build_progress_bar_slide(prs, slide_data):
    """Horizontal progress bar with resolved items above and in-progress items below.

    resolved: list of {"label": "...", "value": "..."}
    in_progress: list of {"label": "...", "value": "..."}
    progress_label: text above bar (e.g., "67% Resolved (4 of 6 areas)")
    takeaway: optional bottom bar
    """
    LIGHT_GREEN_BG = RGBColor(0xE8, 0xF5, 0xE9)
    LIGHT_ORANGE_BG = RGBColor(0xFF, 0xF3, 0xED)
    BAR_BG = RGBColor(0xE0, 0xE0, 0xE0)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    resolved = slide_data.get("resolved", [])
    in_progress = slide_data.get("in_progress", [])
    takeaway = slide_data.get("takeaway", "")
    progress_label = slide_data.get("progress_label", "")

    total = len(resolved) + len(in_progress)
    pct = len(resolved) / total if total > 0 else 0

    # Progress bar in the middle
    bar_y = Inches(3.4)
    bar_h = Inches(0.22)

    # Label above bar
    if progress_label:
        add_text_box(slide, L, bar_y - Inches(0.35), BW, Inches(0.3),
                     progress_label, font_size=BADGE_SIZE, bold=True, color=DARK_GRAY,
                     alignment=PP_ALIGN.CENTER)

    # Gray background bar
    bar_bg = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, bar_y, BW, bar_h)
    bar_bg.fill.solid(); bar_bg.fill.fore_color.rgb = BAR_BG; bar_bg.line.fill.background()

    # Green fill
    fill_w = int(BW * pct)
    if fill_w > 0:
        bar_fill = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, bar_y, fill_w, bar_h)
        bar_fill.fill.solid(); bar_fill.fill.fore_color.rgb = GREEN; bar_fill.line.fill.background()

    # Orange remaining
    if fill_w < BW:
        bar_rem = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L + fill_w, bar_y, BW - fill_w, bar_h)
        bar_rem.fill.solid(); bar_rem.fill.fore_color.rgb = ORANGE; bar_rem.line.fill.background()

    # Resolved items above the bar
    n_res = len(resolved)
    if n_res > 0:
        res_gap = Inches(0.25)
        res_w = (BW - res_gap * (n_res - 1)) / n_res
        res_h = Inches(1.2)
        res_y = Inches(1.5)

        for i, item in enumerate(resolved):
            rx = L + (res_w + res_gap) * i
            shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, rx, res_y, res_w, res_h)
            shape.fill.solid(); shape.fill.fore_color.rgb = LIGHT_GREEN_BG; shape.line.fill.background()
            acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, rx, res_y, res_w, Inches(0.05))
            acc.fill.solid(); acc.fill.fore_color.rgb = GREEN; acc.line.fill.background()
            add_text_box(slide, rx + Inches(0.1), res_y + Inches(0.15), res_w - Inches(0.2), Inches(0.25),
                         item.get("label", ""), font_size=12, bold=True, color=GREEN,
                         alignment=PP_ALIGN.CENTER)
            add_text_box(slide, rx + Inches(0.1), res_y + Inches(0.4), res_w - Inches(0.2), Inches(0.7),
                         item.get("value", ""), font_size=11, color=DARK_GRAY,
                         alignment=PP_ALIGN.CENTER)

    # In-progress items below the bar
    n_prog = len(in_progress)
    if n_prog > 0:
        prog_gap = Inches(0.35)
        prog_total_w = BW * 0.7  # Don't span full width for 1-2 items
        prog_w = (prog_total_w - prog_gap * (n_prog - 1)) / n_prog
        prog_start_x = L + (BW - prog_total_w) / 2  # Center
        prog_y = Inches(3.85)
        prog_h = Inches(0.95)

        for i, item in enumerate(in_progress):
            px = prog_start_x + (prog_w + prog_gap) * i
            shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, px, prog_y, prog_w, prog_h)
            shape.fill.solid(); shape.fill.fore_color.rgb = LIGHT_ORANGE_BG; shape.line.fill.background()
            acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, px, prog_y, prog_w, Inches(0.05))
            acc.fill.solid(); acc.fill.fore_color.rgb = ORANGE; acc.line.fill.background()
            add_text_box(slide, px + Inches(0.1), prog_y + Inches(0.12), prog_w - Inches(0.2), Inches(0.25),
                         item.get("label", ""), font_size=13, bold=True, color=ORANGE,
                         alignment=PP_ALIGN.CENTER)
            add_text_box(slide, px + Inches(0.1), prog_y + Inches(0.4), prog_w - Inches(0.2), Inches(0.4),
                         item.get("value", ""), font_size=11, color=DARK_GRAY,
                         alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_stats_summary_slide(prs, slide_data):
    """Stats cards on top + summary band on bottom. Shows effort → result.

    top_stats: list of {"value": "1.4M", "label": "CloudTrail events",
                        "sublabel": "Zero unauthorized API calls", "color": "blue"}
    summary: {"header": "Zero Indicators of Compromise", "color": "green",
              "items": [{"value": "0", "label": "tpcp-docs repos", "sublabel": "No fallback exfil"}]}
    takeaway: optional bottom bar
    """
    LIGHT_GREEN_BG = RGBColor(0xE8, 0xF5, 0xE9)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    top_stats = slide_data.get("top_stats", [])
    summary = slide_data.get("summary", {})
    takeaway = slide_data.get("takeaway", "")

    # Top: stat cards (1-4 items)
    n_top = min(len(top_stats), 4)
    if n_top > 0:
        gap_t = Inches(0.3)
        total_gap = gap_t * (n_top - 1)
        card_w = (BW - total_gap) / n_top
        card_h = Inches(2.0)
        card_y = Inches(1.5)
        accent_h = Inches(0.06)

        for i, s in enumerate(top_stats[:n_top]):
            x = L + (card_w + gap_t) * i
            color = resolve_color(s.get("color", "blue"))

            # Accent top
            acc_color = s.get("accent", slide_data.get("top_accent", None))
            if acc_color:
                acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, card_y, card_w, accent_h)
                acc.fill.solid(); acc.fill.fore_color.rgb = resolve_color(acc_color); acc.line.fill.background()

            # Card body
            body_off = accent_h if acc_color else 0
            body = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, x, card_y + body_off,
                                           card_w, card_h - body_off)
            body.fill.solid(); body.fill.fore_color.rgb = color; body.line.fill.background()

            # Value
            add_text_box(slide, x, card_y + Inches(0.2), card_w, Inches(0.7),
                         str(s.get("value", "")), font_size=36, bold=True,
                         color=WHITE, alignment=PP_ALIGN.CENTER)
            # Label
            add_text_box(slide, x + Inches(0.1), card_y + Inches(0.9), card_w - Inches(0.2), Inches(0.4),
                         s.get("label", ""), font_size=BADGE_SIZE, bold=True,
                         color=WHITE, alignment=PP_ALIGN.CENTER)
            # Sublabel
            sublabel = s.get("sublabel", "")
            if sublabel:
                add_text_box(slide, x + Inches(0.1), card_y + Inches(1.3), card_w - Inches(0.2), Inches(0.4),
                             sublabel, font_size=11, bold=False,
                             color=RGBColor(0xBB, 0xCC, 0xDD), alignment=PP_ALIGN.CENTER)

    # Bottom: summary band
    if summary:
        sum_color = resolve_color(summary.get("color", "green"))
        band_y = Inches(3.8)
        band_h = Inches(1.6)

        # Light background
        bg_color = summary.get("bg_color")
        if bg_color:
            bg_rgb = resolve_color(bg_color)
        else:
            # Auto light version
            bg_rgb = LIGHT_GREEN_BG
        band = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, band_y, BW, band_h)
        band.fill.solid(); band.fill.fore_color.rgb = bg_rgb; band.line.fill.background()

        # Top accent line
        acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, band_y, BW, Inches(0.06))
        acc.fill.solid(); acc.fill.fore_color.rgb = sum_color; acc.line.fill.background()

        # Header
        add_text_box(slide, Inches(0.9), band_y + Inches(0.15), Inches(11.3), Inches(0.35),
                     summary.get("header", ""), font_size=HEADER_SIZE, bold=True, color=sum_color)

        # Summary items in a row
        sum_items = summary.get("items", [])
        n_items = len(sum_items)
        if n_items > 0:
            item_w = BW / n_items
            for j, si in enumerate(sum_items):
                ix = Inches(0.9) + item_w * j
                # Value + label on one line
                val_text = f"{si.get('value', '')}  {si.get('label', '')}"
                add_text_box(slide, ix, band_y + Inches(0.55), item_w, Inches(0.35),
                             val_text, font_size=BADGE_SIZE, bold=True, color=DARK_GRAY)
                # Sublabel
                sub = si.get("sublabel", "")
                if sub:
                    add_text_box(slide, ix, band_y + Inches(0.9), item_w, Inches(0.35),
                                 sub, font_size=11, color=GRAY)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_split_contrast_slide(prs, slide_data):
    """Split contrast slide: two side-by-side panels with mini-timelines and stats.

    left_panel: {"header": "...", "bg_color": "blue", "text_color": "white",
                 "accent_color": "orange" (optional top accent),
                 "dot_color": "green",
                 "events": [{"date": "Mar 20 15:35", "text": "Detected SEV-1"}],
                 "stat_value": "48h ahead", "stat_label": "of K8s payloads at other victims",
                 "stat_color": "white"}
    right_panel: same structure
    takeaway: optional bottom bar
    """
    CARD_BG_R = RGBColor(0xF5, 0xF7, 0xFA)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    takeaway = slide_data.get("takeaway", "")

    half_w = Inches(5.8)
    panel_h = Inches(4.2)
    panel_y = Inches(1.4)
    left_x = L
    right_x = Inches(6.67)
    accent_h = Inches(0.06)

    for panel_data, px in [(slide_data.get("left_panel", {}), left_x),
                            (slide_data.get("right_panel", {}), right_x)]:
        if not panel_data:
            continue

        bg = resolve_color(panel_data.get("bg_color", "blue"))
        text_col = resolve_color(panel_data.get("text_color", "white"))
        dot_col = resolve_color(panel_data.get("dot_color", "green"))
        stat_col = resolve_color(panel_data.get("stat_color", "white"))

        # Panel background
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, px, panel_y, half_w, panel_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = bg; shape.line.fill.background()

        # Optional top accent
        acc_color = panel_data.get("accent_color")
        if acc_color:
            acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, px, panel_y, half_w, accent_h)
            acc.fill.solid(); acc.fill.fore_color.rgb = resolve_color(acc_color); acc.line.fill.background()

        # Header
        add_text_box(slide, px + Inches(0.25), panel_y + Inches(0.2), half_w - Inches(0.5), Inches(0.4),
                     panel_data.get("header", ""), font_size=SUBTITLE_SIZE, bold=True, color=text_col)

        # Mini-timeline events
        events = panel_data.get("events", [])
        ey = panel_y + Inches(0.7)
        dot_sz = Inches(0.14)
        max_events = min(len(events), 6)

        for j in range(max_events):
            ev = events[j]
            # Dot
            dot = slide.shapes.add_shape(MSO_SHAPE.OVAL,
                px + Inches(0.3), ey + Inches(0.04), dot_sz, dot_sz)
            dot.fill.solid(); dot.fill.fore_color.rgb = dot_col; dot.line.fill.background()

            # Date
            add_text_box(slide, px + Inches(0.55), ey, Inches(1.5), Inches(0.22),
                         ev.get("date", ""), font_size=10, bold=True, color=text_col)

            # Description
            add_text_box(slide, px + Inches(2.1), ey, Inches(3.4), Inches(0.22),
                         ev.get("text", ""), font_size=11, color=text_col)

            ey += Inches(0.35)

        # Stat value (big, bottom of panel)
        stat_y = panel_y + panel_h - Inches(1.4)
        add_text_box(slide, px, stat_y, half_w, Inches(0.8),
                     panel_data.get("stat_value", ""), font_size=36, bold=True,
                     color=stat_col, alignment=PP_ALIGN.CENTER)

        # Stat label
        add_text_box(slide, px + Inches(0.2), stat_y + Inches(0.75), half_w - Inches(0.4), Inches(0.4),
                     panel_data.get("stat_label", ""), font_size=11, color=text_col,
                     alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_vertical_timeline_slide(prs, slide_data):
    """Vertical timeline with cascade cards. Max 8 items.

    events: list of {"date": "Mar 19", "target": "Trivy", "detail": "Actions, Docker Hub",
                     "impact": "10,000+ orgs", "color": "blue", "badge": "NEAR-MISS" (optional)}
    footnote: optional text below cards
    takeaway: optional bottom bar
    """
    CARD_BG_DEFAULT = RGBColor(0xF5, 0xF7, 0xFA)
    NEARMISS_BG = RGBColor(0xFF, 0xF3, 0xED)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    events = slide_data.get("events", [])[:8]
    footnote = slide_data.get("footnote", "")
    takeaway = slide_data.get("takeaway", "")
    n = len(events)
    if n == 0:
        return slide

    # Scaling: calculate card height based on item count
    available_h = Inches(3.8) if (footnote or takeaway) else Inches(4.5)
    gap_h = Inches(0.08)
    total_gap = gap_h * (n - 1)
    card_h = min((available_h - total_gap) / n, Inches(0.7))

    timeline_x = Inches(1.5)
    card_x = Inches(2.1)
    card_w = Inches(10.0)
    dot_sz = Inches(0.18)
    start_y = Inches(1.35)

    # Vertical timeline line (first dot center to last dot center)
    first_dot_cy = start_y + card_h / 2
    last_dot_cy = start_y + (card_h + gap_h) * (n - 1) + card_h / 2
    line = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE,
        timeline_x - Inches(0.015), first_dot_cy, Inches(0.03), last_dot_cy - first_dot_cy)
    line.fill.solid(); line.fill.fore_color.rgb = GRAY; line.line.fill.background()

    for i, ev in enumerate(events):
        y = start_y + (card_h + gap_h) * i
        color = resolve_color(ev.get("color", "blue"))
        has_badge = bool(ev.get("badge", ""))

        # Timeline dot
        dot_y = y + (card_h - dot_sz) / 2
        dot = slide.shapes.add_shape(MSO_SHAPE.OVAL,
            timeline_x - dot_sz / 2, dot_y, dot_sz, dot_sz)
        dot.fill.solid(); dot.fill.fore_color.rgb = color; dot.line.fill.background()

        # Calculate vertical centering for text block within card
        detail = ev.get("detail", "")
        text_block_h = Inches(0.48) if detail else Inches(0.25)
        text_top = y + (card_h - text_block_h) / 2

        # Date label (vertically centered with card)
        add_text_box(slide, Inches(0.15), y + (card_h - Inches(0.3)) / 2, Inches(1.15), Inches(0.3),
                     ev.get("date", ""), font_size=11, bold=True, color=DARK_GRAY,
                     alignment=PP_ALIGN.RIGHT)

        # Card background
        card_bg = NEARMISS_BG if has_badge else CARD_BG_DEFAULT
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, card_x, y, card_w, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = card_bg; shape.line.fill.background()

        # Left color border
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, card_x, y, Inches(0.06), card_h)
        stripe.fill.solid(); stripe.fill.fore_color.rgb = color; stripe.line.fill.background()

        # Target (bold)
        add_text_box(slide, card_x + Inches(0.18), text_top, Inches(4.0), Inches(0.25),
                     ev.get("target", ""), font_size=13, bold=True, color=BLUE)

        # Detail (smaller, below target)
        if detail:
            add_text_box(slide, card_x + Inches(0.18), text_top + Inches(0.23), Inches(4.0), Inches(0.22),
                         detail, font_size=10, color=GRAY)

        # Impact (right side, vertically centered)
        impact_w = Inches(4.2) if not has_badge else Inches(3.0)
        add_text_box(slide, card_x + Inches(4.8), y + (card_h - Inches(0.3)) / 2, impact_w, Inches(0.3),
                     ev.get("impact", ""), font_size=12, color=DARK_GRAY)

        # Badge
        if has_badge:
            badge_x = card_x + Inches(8.3)
            badge_y = y + (card_h - Inches(0.28)) / 2
            add_rounded_rect(slide, badge_x, badge_y, Inches(1.2), Inches(0.28),
                             resolve_color(ev.get("badge_color", "orange")))
            add_text_box(slide, badge_x, badge_y + Inches(0.01), Inches(1.2), Inches(0.24),
                         ev["badge"], font_size=9, bold=True, color=WHITE,
                         alignment=PP_ALIGN.CENTER)

    # Footnote
    end_y = start_y + (card_h + gap_h) * n
    if footnote:
        add_text_box(slide, L, end_y + Inches(0.1), BW, Inches(0.3),
                     footnote, font_size=12, color=GRAY)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_horizontal_timeline_slide(prs, slide_data):
    """Horizontal timeline with alternating cards above/below. Max 6 items.

    events: list of {"date": "Mar 19", "target": "Trivy",
                     "impact": "10,000+ orgs", "color": "blue", "badge": "NEAR-MISS" (optional)}
    footnote: optional text
    takeaway: optional bottom bar
    """
    CARD_BG_DEFAULT = RGBColor(0xF5, 0xF7, 0xFA)
    NEARMISS_BG = RGBColor(0xFF, 0xF3, 0xED)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    events = slide_data.get("events", [])[:6]
    footnote = slide_data.get("footnote", "")
    takeaway = slide_data.get("takeaway", "")
    n = len(events)
    if n == 0:
        return slide

    # Layout constants
    bar_y = Inches(3.3)
    bar_h = Inches(0.05)
    bar_left = Inches(0.9)
    bar_w = Inches(11.4)
    dot_sz = Inches(0.22)

    # Timeline bar
    line = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, bar_left, bar_y, bar_w, bar_h)
    line.fill.solid(); line.fill.fore_color.rgb = GRAY; line.line.fill.background()

    # Even spacing regardless of actual dates
    card_w = min(Inches(11.4 / n - 0.1), Inches(2.0))
    spacing = bar_w / n

    for i, ev in enumerate(events):
        color = resolve_color(ev.get("color", "blue"))
        has_badge = bool(ev.get("badge", ""))

        # Center X for this event
        cx = bar_left + spacing * i + spacing / 2

        # Dot on timeline
        dot = slide.shapes.add_shape(MSO_SHAPE.OVAL,
            cx - dot_sz / 2, bar_y - dot_sz / 2 + bar_h / 2, dot_sz, dot_sz)
        dot.fill.solid(); dot.fill.fore_color.rgb = color; dot.line.fill.background()

        # Alternate above/below
        card_h = Inches(1.5)
        card_left = cx - card_w / 2

        if i % 2 == 0:
            # Above
            card_top = bar_y - Inches(0.15) - card_h
            # Connector
            conn = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE,
                cx - Inches(0.01), card_top + card_h, Inches(0.02), Inches(0.15))
            conn.fill.solid(); conn.fill.fore_color.rgb = GRAY; conn.line.fill.background()
        else:
            # Below
            card_top = bar_y + bar_h + Inches(0.15)
            conn = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE,
                cx - Inches(0.01), bar_y + bar_h, Inches(0.02), Inches(0.15))
            conn.fill.solid(); conn.fill.fore_color.rgb = GRAY; conn.line.fill.background()

        # Card
        card_bg = NEARMISS_BG if has_badge else CARD_BG_DEFAULT
        shape = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, card_left, card_top, card_w, card_h)
        shape.fill.solid(); shape.fill.fore_color.rgb = card_bg; shape.line.fill.background()

        # Color top accent
        acc = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, card_left, card_top, card_w, Inches(0.05))
        acc.fill.solid(); acc.fill.fore_color.rgb = color; acc.line.fill.background()

        # Date
        add_text_box(slide, card_left, card_top + Inches(0.1), card_w, Inches(0.2),
                     ev.get("date", ""), font_size=10, bold=True, color=GRAY,
                     alignment=PP_ALIGN.CENTER)

        # Target
        add_text_box(slide, card_left + Inches(0.05), card_top + Inches(0.3),
                     card_w - Inches(0.1), Inches(0.3),
                     ev.get("target", ""), font_size=12, bold=True, color=BLUE,
                     alignment=PP_ALIGN.CENTER)

        # Impact
        add_text_box(slide, card_left + Inches(0.05), card_top + Inches(0.6),
                     card_w - Inches(0.1), Inches(0.7),
                     ev.get("impact", ""), font_size=10, color=DARK_GRAY,
                     alignment=PP_ALIGN.CENTER)

        # Badge
        if has_badge:
            badge_y = card_top + card_h - Inches(0.35)
            badge_w = min(card_w - Inches(0.2), Inches(1.2))
            badge_x = card_left + (card_w - badge_w) / 2
            add_rounded_rect(slide, badge_x, badge_y, badge_w, Inches(0.25),
                             resolve_color(ev.get("badge_color", "orange")))
            add_text_box(slide, badge_x, badge_y + Inches(0.01), badge_w, Inches(0.22),
                         ev["badge"], font_size=9, bold=True, color=WHITE,
                         alignment=PP_ALIGN.CENTER)

    if footnote:
        add_text_box(slide, L, Inches(5.3), BW, Inches(0.3),
                     footnote, font_size=12, color=GRAY)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_enhanced_table_slide(prs, slide_data):
    """Enhanced table with colored row borders and optional badges. Practical max ~12 rows.

    events: list of {"date": "Mar 19", "target": "Trivy (Actions, Docker Hub)",
                     "impact": "10,000+ orgs", "color": "blue", "badge": "NEAR-MISS" (optional)}
    columns: list of column header strings (default: ["Date", "Target", "Impact"])
    col_widths: list of Inches (optional)
    footnote: optional text
    takeaway: optional bottom bar

    Row text auto-scales (13pt → 11pt) when row height shrinks below 0.4"
    to prevent text overflow into adjacent rows.
    """
    CARD_BG_DEFAULT = RGBColor(0xF5, 0xF7, 0xFA)
    NEARMISS_BG = RGBColor(0xFF, 0xF3, 0xED)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    events = slide_data.get("events", [])[:12]
    columns = slide_data.get("columns", ["Date", "Target", "Impact"])
    footnote = slide_data.get("footnote", "")
    takeaway = slide_data.get("takeaway", "")
    n = len(events)
    if n == 0:
        return slide

    # Scaling
    header_h = Inches(0.42)
    row_gap = Inches(0.04)
    available_h = Inches(3.6) if (footnote or takeaway) else Inches(4.5)
    row_h = min((available_h - header_h - row_gap * n) / n, Inches(0.48))

    # Auto-scale row text size and positioning to fit tighter rows
    if row_h < Inches(0.36):
        row_font = 11
        text_top_offset = Inches(0.05)
        text_h = Inches(0.24)
    else:
        row_font = 13
        text_top_offset = Inches(0.08)
        text_h = Inches(0.3)

    table_top = Inches(1.5)
    border_w = Inches(0.06)

    # Column widths
    col_widths_in = slide_data.get("col_widths", [Inches(1.5), Inches(5.5), Inches(4.8)])
    col_starts = [L + Inches(0.1)]  # After border
    for j in range(1, len(col_widths_in)):
        col_starts.append(col_starts[j-1] + col_widths_in[j-1])

    # Header row
    header = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, table_top, BW, header_h)
    header.fill.solid(); header.fill.fore_color.rgb = BLUE; header.line.fill.background()
    for j, col_name in enumerate(columns):
        add_text_box(slide, col_starts[j], table_top + Inches(0.06),
                     col_widths_in[j], Inches(0.3),
                     col_name, font_size=BADGE_SIZE, bold=True, color=WHITE)

    # Data rows
    for i, ev in enumerate(events):
        y = table_top + header_h + row_gap + (row_h + row_gap) * i
        color = resolve_color(ev.get("color", "blue"))
        has_badge = bool(ev.get("badge", ""))

        # Row background
        row_bg = NEARMISS_BG if has_badge else (CARD_BG_DEFAULT if i % 2 == 0 else WHITE)
        row = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, BW, row_h)
        row.fill.solid(); row.fill.fore_color.rgb = row_bg; row.line.fill.background()

        # Left color border
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, L, y, border_w, row_h)
        stripe.fill.solid(); stripe.fill.fore_color.rgb = color; stripe.line.fill.background()

        # Date
        add_text_box(slide, col_starts[0], y + text_top_offset,
                     col_widths_in[0], text_h,
                     ev.get("date", ""), font_size=row_font, bold=True, color=DARK_GRAY)

        # Target
        target_text = ev.get("target", "")
        detail = ev.get("detail", "")
        if detail:
            target_text = f"{target_text} ({detail})"
        add_text_box(slide, col_starts[1], y + text_top_offset,
                     col_widths_in[1], text_h,
                     target_text, font_size=row_font, color=DARK_GRAY)

        # Impact
        impact_w = col_widths_in[2] - (Inches(1.3) if has_badge else 0)
        add_text_box(slide, col_starts[2], y + text_top_offset,
                     impact_w, text_h,
                     ev.get("impact", ""), font_size=row_font, color=DARK_GRAY)

        # Badge
        if has_badge:
            badge_x = L + BW - Inches(1.3)
            badge_y = y + (row_h - Inches(0.28)) / 2
            add_rounded_rect(slide, badge_x, badge_y, Inches(1.2), Inches(0.28),
                             resolve_color(ev.get("badge_color", "orange")))
            add_text_box(slide, badge_x, badge_y + Inches(0.01), Inches(1.2), Inches(0.24),
                         ev["badge"], font_size=9, bold=True, color=WHITE,
                         alignment=PP_ALIGN.CENTER)

    # Footnote
    end_y = table_top + header_h + row_gap + (row_h + row_gap) * n
    if footnote:
        add_text_box(slide, L, end_y + Inches(0.1), BW, Inches(0.3),
                     footnote, font_size=12, color=GRAY)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_metric_tree_slide(prs, slide_data):
    """Hierarchical tree: root box at top, N category columns below with connecting busbar.

    root: {"label": "Root metric name", "color": "blue"}
    categories: list of
        {"header": "Growth Health", "color": "blue", "items": ["ARR", "MRR", ...]}
    footnote: optional text below tree
    takeaway: optional bottom bar
    """
    LIGHT_CARD_BG = RGBColor(0xF5, 0xF7, 0xFA)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    root = slide_data.get("root", {})
    categories = slide_data.get("categories", [])[:5]
    footnote = slide_data.get("footnote", "")
    takeaway = slide_data.get("takeaway", "")
    n = len(categories)
    if n == 0:
        return slide

    root_color = resolve_color(root.get("color", "blue"))
    root_label = root.get("label", "")

    # Root box: centered, ~6" wide
    root_w = Inches(6.5)
    root_h = Inches(0.7)
    root_x = L + (BW - root_w) / 2
    root_y = Inches(1.3)
    root_shape = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, root_x, root_y, root_w, root_h)
    root_shape.fill.solid(); root_shape.fill.fore_color.rgb = root_color; root_shape.line.fill.background()
    add_text_box(slide, root_x, root_y + Inches(0.15), root_w, Inches(0.45),
                 root_label, font_size=HEADER_SIZE, bold=True, color=WHITE,
                 alignment=PP_ALIGN.CENTER)

    # Vertical connector from root to busbar
    busbar_y = Inches(2.25)
    conn_x = L + BW / 2 - Inches(0.015)
    conn = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, conn_x, root_y + root_h,
                                   Inches(0.03), busbar_y - (root_y + root_h))
    conn.fill.solid(); conn.fill.fore_color.rgb = GRAY; conn.line.fill.background()

    # Compute column positions
    col_gap = Inches(0.18)
    total_gap = col_gap * (n - 1)
    col_w = (BW - total_gap) / n
    col_xs = [L + (col_w + col_gap) * i for i in range(n)]
    col_centers = [cx + col_w / 2 for cx in col_xs]

    # Horizontal busbar connecting columns (from first to last column center)
    bus_left = col_centers[0]
    bus_right = col_centers[-1]
    bus = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, bus_left, busbar_y,
                                  bus_right - bus_left, Inches(0.03))
    bus.fill.solid(); bus.fill.fore_color.rgb = GRAY; bus.line.fill.background()

    # Category headers + leaf chips
    header_y = Inches(2.45)
    header_h = Inches(0.45)
    chip_h = Inches(0.32)
    chip_gap = Inches(0.06)
    chip_start_y = header_y + header_h + Inches(0.12)

    # Reserve bottom zone for footnote (above takeaway) and/or takeaway bar.
    # Takeaway lives at y=6.1, h=0.5 → ends at 6.6.
    # Footnote needs to sit ABOVE the takeaway with breathing room.
    if takeaway and footnote:
        footnote_y = Inches(5.7)        # text ends ~6.0, takeaway starts 6.1
        chip_bottom_limit = Inches(5.55)
    elif takeaway:
        footnote_y = None
        chip_bottom_limit = Inches(5.95)
    elif footnote:
        footnote_y = Inches(6.3)
        chip_bottom_limit = Inches(6.15)
    else:
        footnote_y = None
        chip_bottom_limit = Inches(6.5)

    available_chip_h = chip_bottom_limit - chip_start_y
    max_chips = max(int(available_chip_h / (chip_h + chip_gap)), 4)

    for i, cat in enumerate(categories):
        cx = col_xs[i]
        color = resolve_color(cat.get("color", "blue"))
        # Drop line from busbar to category header
        drop_x = col_centers[i] - Inches(0.015)
        drop = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, drop_x, busbar_y,
                                       Inches(0.03), header_y - busbar_y)
        drop.fill.solid(); drop.fill.fore_color.rgb = GRAY; drop.line.fill.background()

        # Category header card
        hdr = slide.shapes.add_shape(MSO_SHAPE.ROUNDED_RECTANGLE, cx, header_y, col_w, header_h)
        hdr.fill.solid(); hdr.fill.fore_color.rgb = color; hdr.line.fill.background()
        add_text_box(slide, cx, header_y + Inches(0.08), col_w, Inches(0.3),
                     cat.get("header", ""), font_size=BADGE_SIZE, bold=True,
                     color=WHITE, alignment=PP_ALIGN.CENTER)

        # Leaf chips: light bg with colored left border
        items = cat.get("items", [])[:max_chips]
        for j, item in enumerate(items):
            cy = chip_start_y + (chip_h + chip_gap) * j
            chip = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, cx, cy, col_w, chip_h)
            chip.fill.solid(); chip.fill.fore_color.rgb = LIGHT_CARD_BG; chip.line.fill.background()
            stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, cx, cy, Inches(0.05), chip_h)
            stripe.fill.solid(); stripe.fill.fore_color.rgb = color; stripe.line.fill.background()
            add_text_box(slide, cx + Inches(0.15), cy + Inches(0.04), col_w - Inches(0.2), Inches(0.25),
                         item, font_size=11, color=DARK_GRAY)

    if footnote and footnote_y is not None:
        add_text_box(slide, L, footnote_y, BW, Inches(0.3),
                     footnote, font_size=BADGE_SIZE, color=GRAY,
                     alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_formula_slide(prs, slide_data):
    """Equation slide: large central formula + term-definition cards below.

    result: "Retained Protected Revenue at Target Margin"
    terms: list of {"name": "ARR", "definition": "Annual recurring revenue from retained MSS clients",
                    "color": "blue"}
    operator: multiplier symbol between terms (default "×")
    stack: if True (default), equation renders on two lines (result, then = term × term × term).
        If False, equation renders inline on one line and will wrap if too long.
    accent: optional accent color for top divider line above terms
    takeaway: optional bottom bar
    """
    LIGHT_CARD_BG = RGBColor(0xF5, 0xF7, 0xFA)

    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    result = slide_data.get("result", "")
    terms = slide_data.get("terms", [])
    operator = slide_data.get("operator", "×")
    stack = slide_data.get("stack", True)
    accent = slide_data.get("accent")
    takeaway = slide_data.get("takeaway", "")

    n = len(terms)
    if n == 0:
        return slide

    # Build the "= term × term × term" run list (shared by stacked and inline)
    formula_runs = [{"text": "=  ", "bold": True, "color": DARK_GRAY, "font_size": 22}]
    for i, t in enumerate(terms):
        if i > 0:
            formula_runs.append({"text": f"  {operator}  ", "bold": True,
                                 "color": DARK_GRAY, "font_size": 22})
        formula_runs.append({
            "text": t.get("name", ""),
            "bold": True,
            "color": resolve_color(t.get("color", "blue")),
            "font_size": 22,
        })

    eq_y = Inches(1.55) if stack else Inches(1.7)
    eq_h = Inches(1.5) if stack else Inches(1.4)

    if stack:
        # Two paragraphs: result on line 1, formula on line 2
        paragraphs = [
            {
                "alignment": PP_ALIGN.CENTER,
                "font_size": 22,
                "space_after": 8,
                "runs": [{"text": result, "bold": True, "color": BLUE, "font_size": 22}],
            },
            {
                "alignment": PP_ALIGN.CENTER,
                "font_size": 22,
                "runs": formula_runs,
            },
        ]
        add_rich_text(slide, L, eq_y, BW, eq_h, paragraphs)
    else:
        # Single line: result = term × term × term
        inline_runs = [{"text": result, "bold": True, "color": BLUE, "font_size": 22}]
        inline_runs.append({"text": "  ", "bold": True, "color": DARK_GRAY, "font_size": 22})
        inline_runs.extend(formula_runs)
        add_rich_text(slide, L, eq_y, BW, eq_h, [{
            "alignment": PP_ALIGN.CENTER,
            "font_size": 22,
            "runs": inline_runs,
        }])

    # Optional accent line between equation and term cards
    cards_y = Inches(3.3)
    if accent:
        acc_color = resolve_color(accent)
        line_y = cards_y - Inches(0.25)
        line_w = Inches(2.0)
        line_x = L + (BW - line_w) / 2
        line = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, line_x, line_y, line_w, Inches(0.04))
        line.fill.solid(); line.fill.fore_color.rgb = acc_color; line.line.fill.background()

    # Term-definition cards in a row
    gap = Inches(0.25)
    total_gap = gap * (n - 1)
    card_w = (BW - total_gap) / n
    card_h = Inches(2.0)

    for i, t in enumerate(terms):
        cx = L + (card_w + gap) * i
        color = resolve_color(t.get("color", "blue"))

        # Card body
        card = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, cx, cards_y, card_w, card_h)
        card.fill.solid(); card.fill.fore_color.rgb = LIGHT_CARD_BG; card.line.fill.background()
        # Top accent stripe
        stripe = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, cx, cards_y, card_w, Inches(0.06))
        stripe.fill.solid(); stripe.fill.fore_color.rgb = color; stripe.line.fill.background()

        # Term name
        add_text_box(slide, cx + Inches(0.15), cards_y + Inches(0.22), card_w - Inches(0.3), Inches(0.4),
                     t.get("name", ""), font_size=HEADER_SIZE, bold=True, color=color,
                     alignment=PP_ALIGN.CENTER)
        # Definition
        add_text_box(slide, cx + Inches(0.2), cards_y + Inches(0.7), card_w - Inches(0.4), Inches(1.2),
                     t.get("definition", ""), font_size=BADGE_SIZE, color=DARK_GRAY,
                     alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_weighted_composite_slide(prs, slide_data):
    """Horizontal stacked bar with weighted segments + legend below.

    composite_label: optional label above the bar (e.g., "Service Health Score = 100%")
    components: list of {"name": "SLA Performance", "weight": 25, "color": "blue"}
        Weights should sum to ~100 but builder will normalize if not.
    context: optional paragraph of explanatory text below the legend
    takeaway: optional bottom bar
    """
    slide = prs.slides.add_slide(prs.slide_layouts[LAYOUT_SHORT_ONE])
    clear_all_placeholders(slide)

    add_text_box(slide, L, Inches(0.54), BW, Inches(0.50),
                 slide_data.get("title", ""), font_size=TITLE_SIZE, bold=True, color=BLUE)

    components = slide_data.get("components", [])
    composite_label = slide_data.get("composite_label", "")
    context = slide_data.get("context", "")
    takeaway = slide_data.get("takeaway", "")

    n = len(components)
    if n == 0:
        return slide

    # Normalize weights
    total = sum(c.get("weight", 0) for c in components) or 1

    # Composite label above bar
    bar_y = Inches(2.1)
    if composite_label:
        add_text_box(slide, L, bar_y - Inches(0.45), BW, Inches(0.35),
                     composite_label, font_size=HEADER_SIZE, bold=True, color=BLUE,
                     alignment=PP_ALIGN.CENTER)

    # Stacked bar
    bar_h = Inches(0.9)
    cursor_x = L
    for c in components:
        weight = c.get("weight", 0)
        seg_w = int(BW * (weight / total))
        color = resolve_color(c.get("color", "blue"))

        seg = slide.shapes.add_shape(MSO_SHAPE.RECTANGLE, cursor_x, bar_y, seg_w, bar_h)
        seg.fill.solid(); seg.fill.fore_color.rgb = color; seg.line.fill.background()

        # Inline percentage label inside segment (only if segment is wide enough)
        if seg_w >= Inches(0.85):
            add_text_box(slide, cursor_x, bar_y + Inches(0.28), seg_w, Inches(0.4),
                         f"{weight}%", font_size=HEADER_SIZE, bold=True, color=WHITE,
                         alignment=PP_ALIGN.CENTER)
        cursor_x += seg_w

    # Legend below: chip + name + weight
    legend_y = Inches(3.4)
    # Wider columns for 5-6 items so long labels don't wrap and row 2 isn't sparse
    if n <= 4:
        per_row = n
    elif n <= 6:
        per_row = 3
    else:
        per_row = 4
    n_rows = (n + per_row - 1) // per_row
    legend_row_h = Inches(0.65)
    legend_col_gap = Inches(0.2)
    legend_col_w = (BW - legend_col_gap * (per_row - 1)) / per_row

    for idx, c in enumerate(components):
        row = idx // per_row
        col = idx % per_row
        # If last row has fewer items, center them
        row_items = per_row if (row < n_rows - 1) else (n - row * per_row)
        row_total_w = legend_col_w * row_items + legend_col_gap * (row_items - 1)
        row_start_x = L + (BW - row_total_w) / 2 if row_items < per_row else L

        lx = row_start_x + (legend_col_w + legend_col_gap) * col
        ly = legend_y + legend_row_h * row
        color = resolve_color(c.get("color", "blue"))

        # Color dot
        dot = slide.shapes.add_shape(MSO_SHAPE.OVAL, lx, ly + Inches(0.1),
                                      Inches(0.2), Inches(0.2))
        dot.fill.solid(); dot.fill.fore_color.rgb = color; dot.line.fill.background()
        # Name (allow up to 2-line wrap as a safety net)
        add_text_box(slide, lx + Inches(0.3), ly + Inches(0.04),
                     legend_col_w - Inches(0.3), Inches(0.36),
                     f"{c.get('name', '')}", font_size=BADGE_SIZE, bold=True, color=DARK_GRAY)
        # Weight (placed below name with enough headroom for a wrapped name)
        add_text_box(slide, lx + Inches(0.3), ly + Inches(0.4),
                     legend_col_w - Inches(0.3), Inches(0.22),
                     f"{c.get('weight', 0)}% weight", font_size=11, color=GRAY)

    # Context paragraph below legend
    if context:
        ctx_y = legend_y + legend_row_h * n_rows + Inches(0.25)
        ctx_h = (Inches(6.0) if takeaway else Inches(6.5)) - ctx_y
        if ctx_h > Inches(0.3):
            add_text_box(slide, L + Inches(1.0), ctx_y, BW - Inches(2.0), ctx_h,
                         context, font_size=BADGE_SIZE, color=DARK_GRAY,
                         alignment=PP_ALIGN.CENTER)

    if takeaway:
        _add_takeaway(slide, takeaway, align=_takeaway_align(slide_data), size=slide_data.get("takeaway_size"))

    return slide


def build_end_slide(prs, slide_data):
    """Closing slide."""
    color = slide_data.get("color", "blue")
    layout_idx = {"white": LAYOUT_END_WHITE, "purple": LAYOUT_END_PURPLE,
                  "blue": LAYOUT_END_BLUE}.get(color, LAYOUT_END_BLUE)
    slide = prs.slides.add_slide(prs.slide_layouts[layout_idx])
    clear_all_placeholders(slide)
    return slide


# ── Slide type dispatcher ───────────────────────────────────────────

SLIDE_BUILDERS = {
    "title": build_title_slide,
    "content": build_content_slide,
    "two_column": build_two_column_slide,
    "table": build_table_slide,
    "status": build_status_slide,
    "stat_callout": build_stat_callout_slide,
    "highlight": build_highlight_slide,
    "key_metrics": build_key_metrics_slide,
    "content_stat": build_content_stat_slide,
    "card_rows": build_card_rows_slide,
    "severity_cards": build_severity_cards_slide,
    "highlight_boxes": build_highlight_boxes_slide,
    "numbered_actions": build_numbered_actions_slide,
    "progress_bar": build_progress_bar_slide,
    "stats_summary": build_stats_summary_slide,
    "split_contrast": build_split_contrast_slide,
    "vertical_timeline": build_vertical_timeline_slide,
    "horizontal_timeline": build_horizontal_timeline_slide,
    "enhanced_table": build_enhanced_table_slide,
    "metric_tree": build_metric_tree_slide,
    "formula": build_formula_slide,
    "weighted_composite": build_weighted_composite_slide,
    "end": build_end_slide,
}


# ── Content validation ──────────────────────────────────────────────

def validate_slides(slides_data):
    """Validate slide content and warn about potential issues."""
    warnings = []
    prev_type = None

    for i, sd in enumerate(slides_data):
        slide_num = i + 1
        slide_type = sd.get("type", "content")
        title = sd.get("title", "untitled")

        # Check bullet length
        for key in ("bullets", "left", "right"):
            items = sd.get(key, [])
            for j, b in enumerate(items):
                text = b if isinstance(b, str) else b.get("text", "")
                if len(text) > MAX_BULLET_CHARS and not text.startswith("**"):
                    warnings.append(
                        f"  Slide {slide_num} ({title}): {key}[{j}] is {len(text)} chars "
                        f"(max {MAX_BULLET_CHARS}): \"{text[:50]}...\"")

        # Check bullet count
        bullets = sd.get("bullets", [])
        if len(bullets) > MAX_BULLETS and slide_type == "content":
            warnings.append(
                f"  Slide {slide_num} ({title}): {len(bullets)} bullets (max {MAX_BULLETS}). "
                f"Consider splitting into two slides or using two_column.")

        # Check consecutive bullet slides
        if slide_type == "content" and prev_type == "content":
            warnings.append(
                f"  Slides {slide_num-1}-{slide_num}: consecutive bullet slides. "
                f"Consider using stat_callout, highlight, or key_metrics for variety.")

        prev_type = slide_type

    if warnings:
        print(f"\n⚠  Content warnings ({len(warnings)}):")
        for w in warnings:
            print(w)
        print()

    return warnings


# ── Build orchestrator ──────────────────────────────────────────────

def build_presentation(slides_data, metadata, output_path):
    """Build a complete presentation from structured slide data."""
    prs = Presentation(str(TEMPLATE))

    # Remove any default slides from template
    while len(prs.slides) > 0:
        sldId = prs.slides._sldIdLst[0]
        rId = sldId.get("r:id") or sldId.get("{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id")
        if rId:
            prs.part.drop_rel(rId)
        prs.slides._sldIdLst.remove(sldId)

    # Validate content
    validate_slides(slides_data)

    for slide_data in slides_data:
        slide_type = slide_data.get("type", "content")
        builder = SLIDE_BUILDERS.get(slide_type)
        if builder:
            slide = builder(prs, slide_data)
            # Add speaker notes (talk track) if provided
            talk_track = slide_data.get("talk_track", "")
            if talk_track and slide is not None:
                notes_slide = slide.notes_slide
                notes_tf = notes_slide.notes_text_frame
                notes_tf.text = talk_track
        else:
            print(f"Warning: Unknown slide type '{slide_type}', skipping")

    prs.save(str(output_path))
    print(f"Presentation saved: {output_path}")
    print(f"  Slides: {len(prs.slides)}")
    return output_path


# ── CLI entry point ─────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: uv run python scripts/build-incident-brief.py <data_module>")
        print("  Example: uv run python scripts/build-incident-brief.py scripts.incident_data.inc_2026_0320")
        sys.exit(1)

    module_path = sys.argv[1]

    if module_path.endswith(".py"):
        spec = importlib.util.spec_from_file_location("data", module_path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
    else:
        mod = importlib.import_module(module_path)

    if not hasattr(mod, "SLIDES") or not hasattr(mod, "METADATA"):
        print(f"Error: {module_path} must define SLIDES and METADATA")
        sys.exit(1)

    output = Path(mod.METADATA.get("output", "output.pptx"))
    build_presentation(mod.SLIDES, mod.METADATA, output)
