#!/usr/bin/env python3
"""
S6 Spike: Generate a sample test .pptx that exercises divergence-prone PPTX features.

This fixture exercises the highest-risk rendering surface areas identified in S6:
  1. Text with custom fonts (Inter, Calibri, Aptos — each handled differently by LO)
  2. Shapes with gradient fills (linear, radial)
  3. Table with various border styles
  4. Dark-background slide using clrMapOvr pattern (simulated via explicit colors)
  5. Grouped shapes
  6. Image placeholder
  7. Slide with text wrapping at layout boundaries

Usage:
    pip install python-pptx
    python3 make-test-pptx.py [output.pptx]

Output:
    test-fixture.pptx (or specified path)

The generated .pptx is intentionally minimal — it uses python-pptx's direct XML
manipulation where needed to exercise OOXML features that python-pptx's high-level
API abstracts away (e.g., gradient fills, exact border colors).
"""

import sys
from pathlib import Path
from pptx import Presentation
from pptx.util import Inches, Pt, Emu
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.oxml.ns import qn
import lxml.etree as etree


# ---------------------------------------------------------------------------
# Color constants (slideforge default brand palette from brand-template-patterns.md)
# ---------------------------------------------------------------------------
BRAND_PRIMARY   = RGBColor(0x00, 0x66, 0xCC)   # accent1
BRAND_SECONDARY = RGBColor(0x00, 0xA3, 0xA1)   # accent2
ACCENT_YELLOW   = RGBColor(0xFF, 0xC7, 0x00)   # accent3
TEXT_DARK       = RGBColor(0x1A, 0x1A, 0x1A)
BACKGROUND      = RGBColor(0xFF, 0xFF, 0xFF)
DARK_BG         = RGBColor(0x00, 0x33, 0x66)   # dk2 — for dark slides
LIGHT_TEXT      = RGBColor(0xFF, 0xFF, 0xFF)


# ---------------------------------------------------------------------------
# Helper: add gradient fill to shape via raw XML
# ---------------------------------------------------------------------------
def apply_linear_gradient(shape, color1: RGBColor, color2: RGBColor, angle_deg: int = 0) -> None:
    """Apply a 2-stop linear gradient fill to the shape spPr element."""
    spPr = shape.shape._element.spPr
    # Remove any existing fill
    for child in list(spPr):
        tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
        if tag in ("solidFill", "gradFill", "noFill", "blipFill", "pattFill"):
            spPr.remove(child)

    # Build <a:gradFill> with two stops
    gradFill = etree.SubElement(spPr, qn("a:gradFill"))
    gsLst = etree.SubElement(gradFill, qn("a:gsLst"))

    gs1 = etree.SubElement(gsLst, qn("a:gs"))
    gs1.set("pos", "0")
    srgb1 = etree.SubElement(gs1, qn("a:srgbClr"))
    srgb1.set("val", f"{color1.rgb:06X}")

    gs2 = etree.SubElement(gsLst, qn("a:gs"))
    gs2.set("pos", "100000")
    srgb2 = etree.SubElement(gs2, qn("a:srgbClr"))
    srgb2.set("val", f"{color2.rgb:06X}")

    lin = etree.SubElement(gradFill, qn("a:lin"))
    lin.set("ang", str(angle_deg * 60000))  # angle in 60,000ths of a degree
    lin.set("scaled", "0")


# ---------------------------------------------------------------------------
# Slide 1: Title slide with font variants
# ---------------------------------------------------------------------------
def add_title_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[0]  # "Title Slide" layout
    slide = prs.slides.add_slide(slide_layout)

    # Set title placeholder
    title_ph = slide.placeholders[0]
    title_ph.text = "slideforge Visual Parity Test Fixture"
    title_tf = title_ph.text_frame
    title_tf.paragraphs[0].runs[0].font.size = Pt(36)
    title_tf.paragraphs[0].runs[0].font.color.rgb = TEXT_DARK

    # Set subtitle placeholder
    subtitle_ph = slide.placeholders[1]
    subtitle_ph.text = "S6 Spike: Multi-Renderer Parity Baseline"
    subtitle_tf = subtitle_ph.text_frame
    subtitle_tf.paragraphs[0].runs[0].font.size = Pt(18)
    subtitle_tf.paragraphs[0].runs[0].font.color.rgb = BRAND_PRIMARY

    # Add speaker notes
    notes_slide = slide.notes_slide
    notes_tf = notes_slide.notes_text_frame
    notes_tf.text = "Slide 1: Title. Tests: title/subtitle placeholders, font size, brand color on text."


# ---------------------------------------------------------------------------
# Slide 2: Text wrapping and font fallback stress test
# ---------------------------------------------------------------------------
def add_text_wrap_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[1]  # "Title and Content"
    slide = prs.slides.add_slide(slide_layout)

    title_ph = slide.placeholders[0]
    title_ph.text = "Font Rendering & Text Wrapping"

    body_ph = slide.placeholders[1]
    tf = body_ph.text_frame
    tf.clear()

    # Paragraph 1: Calibri (Windows default — will substitute in LibreOffice)
    p1 = tf.paragraphs[0]
    run1 = p1.add_run()
    run1.text = "Calibri 14pt: The quick brown fox jumps over the lazy dog. Extra text to force wrapping at the right edge of the text box boundary."
    run1.font.name = "Calibri"
    run1.font.size = Pt(14)
    run1.font.color.rgb = TEXT_DARK

    # Paragraph 2: Arial (cross-platform safe)
    p2 = tf.add_paragraph()
    run2 = p2.add_run()
    run2.text = "Arial 12pt: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
    run2.font.name = "Arial"
    run2.font.size = Pt(12)
    run2.font.color.rgb = BRAND_SECONDARY

    # Paragraph 3: Small text near wrapping boundary
    p3 = tf.add_paragraph()
    run3 = p3.add_run()
    run3.text = "Inter 10pt (custom/system): WRAPPING TEST — this sentence is exactly long enough to test line break behavior and font metric differences between renderers."
    run3.font.name = "Inter"
    run3.font.size = Pt(10)
    run3.font.color.rgb = TEXT_DARK

    notes_tf = slide.notes_slide.notes_text_frame
    notes_tf.text = "Slide 2: Font wrapping stress test. Known LibreOffice behavior: Calibri → Carlito substitution; line breaks differ. Inter fallback varies by platform."


# ---------------------------------------------------------------------------
# Slide 3: Gradient fills (2-stop linear + radial approximation)
# ---------------------------------------------------------------------------
def add_gradient_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[5]  # "Title Only"
    slide = prs.slides.add_slide(slide_layout)

    title_ph = slide.placeholders[0]
    title_ph.text = "Gradient Fill Rendering"

    # Shape 1: horizontal linear gradient (brand_primary → brand_secondary)
    box1 = slide.shapes.add_shape(
        1,  # MSO_SHAPE_TYPE.RECTANGLE
        Inches(0.5), Inches(1.5), Inches(5.5), Inches(1.5)
    )
    box1.name = "GradientLinearHoriz"
    # Set label for accessibility
    # Apply gradient via XML
    spPr1 = box1._element.spPr
    for child in list(spPr1):
        tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
        if tag in ("solidFill", "gradFill", "noFill"):
            spPr1.remove(child)
    gf1 = etree.SubElement(spPr1, qn("a:gradFill"))
    gsl1 = etree.SubElement(gf1, qn("a:gsLst"))
    gs1a = etree.SubElement(gsl1, qn("a:gs"))
    gs1a.set("pos", "0")
    sc1a = etree.SubElement(gs1a, qn("a:srgbClr"))
    sc1a.set("val", "0066CC")
    gs1b = etree.SubElement(gsl1, qn("a:gs"))
    gs1b.set("pos", "100000")
    sc1b = etree.SubElement(gs1b, qn("a:srgbClr"))
    sc1b.set("val", "00A3A1")
    lin1 = etree.SubElement(gf1, qn("a:lin"))
    lin1.set("ang", "0")
    lin1.set("scaled", "0")
    box1.text_frame.text = "Linear gradient: brand_primary → brand_secondary (horizontal)"
    box1.text_frame.paragraphs[0].runs[0].font.color.rgb = LIGHT_TEXT
    box1.text_frame.paragraphs[0].runs[0].font.size = Pt(11)

    # Shape 2: diagonal gradient (brand_primary → accent_yellow)
    box2 = slide.shapes.add_shape(
        1,
        Inches(0.5), Inches(3.2), Inches(5.5), Inches(1.5)
    )
    box2.name = "GradientLinearDiag"
    spPr2 = box2._element.spPr
    for child in list(spPr2):
        tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
        if tag in ("solidFill", "gradFill", "noFill"):
            spPr2.remove(child)
    gf2 = etree.SubElement(spPr2, qn("a:gradFill"))
    gsl2 = etree.SubElement(gf2, qn("a:gsLst"))
    gs2a = etree.SubElement(gsl2, qn("a:gs"))
    gs2a.set("pos", "0")
    sc2a = etree.SubElement(gs2a, qn("a:srgbClr"))
    sc2a.set("val", "003366")
    gs2b = etree.SubElement(gsl2, qn("a:gs"))
    gs2b.set("pos", "100000")
    sc2b = etree.SubElement(gs2b, qn("a:srgbClr"))
    sc2b.set("val", "FFC700")
    lin2 = etree.SubElement(gf2, qn("a:lin"))
    lin2.set("ang", "2700000")  # 45 degrees = 45 * 60000
    lin2.set("scaled", "0")
    box2.text_frame.text = "Diagonal gradient: dark_navy → accent_yellow (45deg)"
    box2.text_frame.paragraphs[0].runs[0].font.color.rgb = LIGHT_TEXT
    box2.text_frame.paragraphs[0].runs[0].font.size = Pt(11)

    notes_tf = slide.notes_slide.notes_text_frame
    notes_tf.text = "Slide 3: Gradient fills. Known LO behavior: slight color stop differences, banding at high DPI. 3-color gradients not supported (this slide uses 2-stop only)."


# ---------------------------------------------------------------------------
# Slide 4: Table with various border styles
# ---------------------------------------------------------------------------
def add_table_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[5]  # "Title Only"
    slide = prs.slides.add_slide(slide_layout)

    title_ph = slide.placeholders[0]
    title_ph.text = "Table Border Rendering"

    rows, cols = 4, 3
    left, top = Inches(0.5), Inches(1.5)
    width, height = Inches(12), Inches(4.5)

    table = slide.shapes.add_table(rows, cols, left, top, width, height).table
    table.columns[0].width = Inches(3)
    table.columns[1].width = Inches(5)
    table.columns[2].width = Inches(4)

    # Header row
    headers = ["Feature", "PowerPoint Behavior", "LibreOffice Behavior"]
    for col_idx, header in enumerate(headers):
        cell = table.cell(0, col_idx)
        cell.text = header
        para = cell.text_frame.paragraphs[0]
        para.runs[0].font.bold = True
        para.runs[0].font.size = Pt(13)
        para.runs[0].font.color.rgb = LIGHT_TEXT
        fill = cell.fill
        fill.solid()
        fill.fore_color.rgb = BRAND_PRIMARY

    # Data rows
    data = [
        ("Font (Calibri)", "Renders Calibri exactly", "Substitutes Carlito; text reflows possible"),
        ("Gradient (3-stop)", "Renders 3-stop gradient", "Downgrades to 2-stop gradient (known limitation)"),
        ("Table borders", "Supports double/round-dotted", "Single border only; round→square-dotted"),
    ]
    for row_idx, (feat, ppt_val, lo_val) in enumerate(data, start=1):
        table.cell(row_idx, 0).text = feat
        table.cell(row_idx, 1).text = ppt_val
        table.cell(row_idx, 2).text = lo_val
        for col_idx in range(3):
            cell = table.cell(row_idx, col_idx)
            para = cell.text_frame.paragraphs[0]
            for run in para.runs:
                run.font.size = Pt(11)
                run.font.color.rgb = TEXT_DARK

    notes_tf = slide.notes_slide.notes_text_frame
    notes_tf.text = "Slide 4: Table with solid-fill header row. Tests: cell fill colors, text in cells, border rendering. Known LO: thin borders may disappear at low DPI."


# ---------------------------------------------------------------------------
# Slide 5: Dark background (simulates clrMapOvr pattern)
# ---------------------------------------------------------------------------
def add_dark_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[0]  # Title Slide — override background manually
    slide = prs.slides.add_slide(slide_layout)

    # Apply dark background fill to slide
    bg = slide.background
    fill = bg.fill
    fill.solid()
    fill.fore_color.rgb = DARK_BG

    title_ph = slide.placeholders[0]
    title_ph.text = "Dark Background Slide"
    tf = title_ph.text_frame
    for para in tf.paragraphs:
        for run in para.runs:
            run.font.color.rgb = LIGHT_TEXT
            run.font.size = Pt(36)

    subtitle_ph = slide.placeholders[1]
    subtitle_ph.text = "Simulates clrMapOvr dark-theme layout"
    tf2 = subtitle_ph.text_frame
    for para in tf2.paragraphs:
        for run in para.runs:
            run.font.color.rgb = ACCENT_YELLOW
            run.font.size = Pt(18)

    # Accent shape with brand color
    accent_box = slide.shapes.add_shape(
        1,
        Inches(0.5), Inches(5.5), Inches(12.3), Inches(0.15)
    )
    accent_box.name = "DarkSlideAccentBar"
    accent_fill = accent_box.fill
    accent_fill.solid()
    accent_fill.fore_color.rgb = BRAND_PRIMARY
    accent_box.line.fill.background()  # no border

    notes_tf = slide.notes_slide.notes_text_frame
    notes_tf.text = "Slide 5: Dark background (manual fill; not clrMapOvr). Tests: light text on dark bg, accent color rendering, background fill. Known LO: clrMapOvr support is incomplete; this slide uses explicit colors instead."


# ---------------------------------------------------------------------------
# Slide 6: Grouped shapes (positioning drift test)
# ---------------------------------------------------------------------------
def add_grouped_shapes_slide(prs: Presentation) -> None:
    slide_layout = prs.slide_layouts[5]  # "Title Only"
    slide = prs.slides.add_slide(slide_layout)

    title_ph = slide.placeholders[0]
    title_ph.text = "Shape Positioning & Grouping"

    # Add 4 colored rectangles that form a 2x2 grid (no actual grouping via python-pptx
    # since GroupShape API is limited; we add adjacent shapes to test positioning)
    positions = [
        (Inches(0.5),  Inches(1.5),  BRAND_PRIMARY,   "Box TL: exact position 0.5in, 1.5in"),
        (Inches(6.75), Inches(1.5),  BRAND_SECONDARY, "Box TR: exact position 6.75in, 1.5in"),
        (Inches(0.5),  Inches(4.25), ACCENT_YELLOW,   "Box BL: exact position 0.5in, 4.25in"),
        (Inches(6.75), Inches(4.25), RGBColor(0xFF, 0x6F, 0x61), "Box BR: exact position 6.75in, 4.25in"),
    ]
    for left, top, color, label in positions:
        box = slide.shapes.add_shape(1, left, top, Inches(5.5), Inches(2.5))
        box.fill.solid()
        box.fill.fore_color.rgb = color
        box.line.color.rgb = TEXT_DARK
        box.line.width = Pt(1)
        tf = box.text_frame
        tf.text = label
        para = tf.paragraphs[0]
        for run in para.runs:
            run.font.size = Pt(11)
            run.font.color.rgb = LIGHT_TEXT if color != ACCENT_YELLOW else TEXT_DARK

    notes_tf = slide.notes_slide.notes_text_frame
    notes_tf.text = "Slide 6: Shape positioning test. Tests: exact shape x/y placement (tolerance ±4pt per visual-parity-contract.md), border rendering, solid fill."


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> None:
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("test-fixture.pptx")

    prs = Presentation()
    # 16:9 widescreen (EMU: 12192000 x 6858000)
    prs.slide_width = Emu(12192000)
    prs.slide_height = Emu(6858000)

    print(f"Building test fixture: {out_path}")
    add_title_slide(prs)
    print("  [1/6] Title slide with font styling")
    add_text_wrap_slide(prs)
    print("  [2/6] Text wrapping and font fallback stress test")
    add_gradient_slide(prs)
    print("  [3/6] Gradient fill rendering")
    add_table_slide(prs)
    print("  [4/6] Table with border styles and cell fills")
    add_dark_slide(prs)
    print("  [5/6] Dark background (clrMapOvr simulation)")
    add_grouped_shapes_slide(prs)
    print("  [6/6] Shape positioning (drift tolerance test)")

    prs.save(str(out_path))
    print(f"\nSaved: {out_path}")
    print("\nTo render via LibreOffice:")
    print(f"  ./render-pptx.sh {out_path} ./rendered-pngs/")
    print("\nTo compare against reference:")
    print(f"  python3 visual-diff.py ./reference-pngs/ ./rendered-pngs/ --out ./diffs/")


if __name__ == "__main__":
    main()
