"""Incident brief data template.

Copy this file and rename to inc_YYYY_NNNN.py for each new incident.
Fill in METADATA and SLIDES with the incident-specific content.

Usage:
    uv run python scripts/build-incident-brief.py scripts/incident_data/inc_YYYY_NNNN.py

Full reference: docs/presentation-system.md
Working example: scripts/incident_data/inc_2026_0320.py

Slide types available (23):
    Structural
        title              - Section divider (blue or purple full-bleed)
        end                - Closing slide (blue, purple, or white)

    Narrative + bullets
        content            - Title + bullets (last resort; never two back-to-back)
        two_column         - Two parallel bullet columns
        highlight          - One critical statement in blue callout box + support
        content_stat       - Narrative blocks left + stat card right (most versatile)

    Metrics
        stat_callout       - 2-6 stat cards (styles: cards, light, plain)
        stats_summary      - Investigation effort cards + zero-result band
        weighted_composite - Stacked bar with weight legend (composite scores, indexes)
        key_metrics        - Alias of stat_callout (backward compat)

    Comparison / parallel narratives
        highlight_boxes    - Two stacked full-width bands (e.g. resolved vs in-progress)
        split_contrast     - Two side-by-side panels with mini-timelines
        card_rows          - Two-column checklist with colored left borders

    Hierarchy / formulas
        metric_tree        - Root box → category columns → leaf metric chips
        formula            - Equation (stacked by default) + term-definition cards

    Risk / actions / status
        severity_cards     - Risk register: full-width cards with severity badges
        numbered_actions   - Color-coded numbered action cards with legend
        status             - Label + colored badge pairs
        progress_bar       - Horizontal progress bar (sequential work only)

    Timelines / tables
        vertical_timeline    - Cascade timeline with connecting line and event cards
        horizontal_timeline  - Timeline bar with cards alternating above/below
        enhanced_table       - Table with colored row borders and optional badges
        table                - Plain branded table with blue header row

Common slide fields:
    talk_track   - Speaker notes (natural speech, not script). Voice rules in docs/voice-guidelines.md
    takeaway     - One-sentence key takeaway rendered in light-blue bar at bottom

Bullet formatting (content, two_column):
    "Regular bullet text"           - Standard bullet
    "**Bold header**"               - Bold blue header (no bullet)
    {"text": "custom", ...}         - Dict with full formatting control

Color names (use lowercase strings in data):
    "blue", "orange", "purple", "green", "red", "teal",
    "gray", "white", "light_blue", "light_gray", "dark_gray"

Hard constraints (validator will warn):
    - Max 6 bullets per content slide
    - Max ~85 characters per bullet at 18pt (wrap = too long; trim or move to talk track)
    - Never two `content` slides back to back
    - Fonts: titles 24pt, body 18pt, badges 14pt, stat numbers 36-44pt
"""

from pathlib import Path
from pptx.util import Inches

METADATA = {
    "incident_id": "INC-YYYY-NNNN",
    "title": "Incident Title",
    "date": "YYYY-MM-DD",
    "author": "Author Name",
    "output": Path(__file__).parent.parent.parent
              / "incidents" / "YYYY-MM-DD_INC-YYYY-NNNN_threat-type"
              / "INC-YYYY-NNNN-Executive-Brief.pptx",
}

SLIDES = [
    # ── Title Slide ─────────────────────────────────────────────
    {
        "type": "title",
        "color": "blue",  # or "purple"
        "title": "Incident Title",
        "subtitle": "INC-YYYY-NNNN  |  SEV-N  |  CVE-YYYY-NNNNN  |  Date",
    },

    # ── Situation ───────────────────────────────────────────────
    {
        "type": "content",
        "title": "Situation",
        "bullets": [
            "What happened",
            "Who was affected",
            "Scope of impact",
        ],
    },

    # ── Two Column Example ──────────────────────────────────────
    {
        "type": "two_column",
        "title": "Two Column Slide",
        "left": [
            "**Left Header**",
            "Left bullet 1",
            "Left bullet 2",
        ],
        "right": [
            "**Right Header**",
            "Right bullet 1",
            "Right bullet 2",
        ],
    },

    # ── Table Example ───────────────────────────────────────────
    {
        "type": "table",
        "title": "Table Slide",
        "table": [
            ["Column 1", "Column 2", "Column 3"],  # Header row
            ["Data 1", "Data 2", "Data 3"],
        ],
        "col_widths": [Inches(3.0), Inches(5.0), Inches(3.8)],
        "footnote": "Optional footnote text",
    },

    # ── Status Example ──────────────────────────────────────────
    {
        "type": "status",
        "title": "Current Status",
        "statuses": [
            {"label": "Area 1", "value": "Status text", "color": "green"},
            {"label": "Area 2", "value": "Status text", "color": "orange"},
            {"label": "Area 3", "value": "Status text", "color": "red"},
        ],
    },

    # ── Section Divider ─────────────────────────────────────────
    {
        "type": "title",
        "color": "blue",
        "title": "Section Title",
        "subtitle": "Optional subtitle",
    },

    # ── End Slide ───────────────────────────────────────────────
    {
        "type": "end",
        "color": "blue",  # or "purple" or "white"
    },
]
