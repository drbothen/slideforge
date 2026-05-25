#!/usr/bin/env bash
# S6 Spike: LibreOffice headless PPTX → PNG rendering script
#
# Usage:
#   ./render-pptx.sh <input.pptx> <output-dir>
#
# Requires: LibreOffice 24.x or 26.x installed
#   macOS:  /Applications/LibreOffice.app/Contents/MacOS/soffice
#   Linux:  soffice (or /usr/lib/libreoffice/program/soffice)
#
# Strategy: PPTX → PDF via LibreOffice, then PDF → PNG via ImageMagick
# (LibreOffice --convert-to png only exports the FIRST slide; the PDF
#  intermediary extracts all slides.)
#
# Output filename pattern: <base>-<NNN>.png  (zero-padded 3 digits, 0-indexed)
# e.g., sample-000.png, sample-001.png, sample-002.png

set -euo pipefail

INPUT="${1:?Usage: render-pptx.sh <input.pptx> <output-dir>}"
OUTDIR="${2:?Usage: render-pptx.sh <input.pptx> <output-dir>}"

# Detect LibreOffice binary (macOS app bundle first, then PATH)
if [[ -x "/Applications/LibreOffice.app/Contents/MacOS/soffice" ]]; then
    SOFFICE="/Applications/LibreOffice.app/Contents/MacOS/soffice"
elif command -v soffice &>/dev/null; then
    SOFFICE="soffice"
else
    echo "ERROR: LibreOffice not found." >&2
    echo "  macOS: brew install --cask libreoffice" >&2
    echo "  Linux: apt-get install libreoffice" >&2
    exit 1
fi

# Detect ImageMagick (magick v7 or convert v6)
if command -v magick &>/dev/null; then
    MAGICK="magick"
elif command -v convert &>/dev/null; then
    MAGICK="convert"
else
    echo "ERROR: ImageMagick not found." >&2
    echo "  macOS: brew install imagemagick" >&2
    exit 1
fi

mkdir -p "$OUTDIR"

BASENAME="$(basename "$INPUT" .pptx)"
PDF_PATH="${OUTDIR}/${BASENAME}.pdf"
START_NS=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")

echo "--- slideforge S6 render-pptx.sh ---"
echo "Input:     $INPUT"
echo "Output:    $OUTDIR"
echo "LibreOffice: $SOFFICE"

# Capture LibreOffice version for metadata
LO_VERSION=$("$SOFFICE" --version 2>&1 | head -1)
echo "Version:   $LO_VERSION"

# Step 1: PPTX → PDF via LibreOffice headless
echo ""
echo "[1/3] Converting PPTX → PDF via LibreOffice..."
"$SOFFICE" --headless --convert-to pdf --outdir "$OUTDIR" "$INPUT" 2>&1 | sed 's/^/  [lo] /'

if [[ ! -f "$PDF_PATH" ]]; then
    echo "ERROR: PDF not produced at $PDF_PATH" >&2
    exit 1
fi

echo "  => $PDF_PATH"

# Step 2: PDF → PNG via ImageMagick
# -density 300: 300 DPI (standard for print-quality slide comparison)
# -background white: flatten transparency against white
# -alpha remove: discard alpha channel (PNG CI artifacts are cleaner)
# -compress LZW: lossless PNG
# Output: <basename>-<NNN>.png (zero-padded 3 digits)
echo ""
echo "[2/3] Converting PDF → PNG (300 DPI) via ImageMagick..."
"$MAGICK" -density 300 -background white -alpha remove \
    "$PDF_PATH" \
    "${OUTDIR}/${BASENAME}-%03d.png" 2>&1 | sed 's/^/  [im] /'

# Step 3: Collect metadata
END_NS=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")
ELAPSED_MS=$(( (END_NS - START_NS) / 1000000 ))

SLIDE_COUNT=$(ls "${OUTDIR}/${BASENAME}"-[0-9]*.png 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "[3/3] Metadata"
echo "  Slides rendered: $SLIDE_COUNT"
echo "  Elapsed:         ${ELAPSED_MS}ms"
echo "  LO version:      $LO_VERSION"
echo "  Output files:"
ls -lh "${OUTDIR}/${BASENAME}"-[0-9]*.png 2>/dev/null | awk '{print "    " $9 " (" $5 ")"}' || true
echo ""
echo "Done. PNGs written to: $OUTDIR"
