#!/usr/bin/env bash
# S6 Spike: CI visual regression gate script
#
# This script is the single entry point for CI visual regression testing.
# It:
#   1. Renders the generated .pptx via LibreOffice → PNG
#   2. Runs the visual-diff.py comparison against committed reference PNGs
#   3. Uploads diff artifacts on failure
#   4. Exits 0 (pass) or 1 (regression detected)
#
# Environment variables (set in CI via workflow env: block):
#   PPTX_PATH          Path to the .pptx file to render (required)
#   REFERENCE_DIR      Path to committed reference PNG fixtures (required)
#   DIFF_ARTIFACT_DIR  Where to write diff images for upload (default: /tmp/slide-diffs)
#   SSIM_THRESHOLD     SSIM pass threshold (default: 0.97, override with 0.95 for noisy fonts)
#   PSNR_THRESHOLD_DB  PSNR warn threshold (default: 35.0)
#   STRICT_PSNR        Set to "true" to fail on PSNR warnings (default: false)
#
# Example GitHub Actions step:
#   - name: Visual regression test
#     env:
#       PPTX_PATH: tests/fixtures/test-fixture.pptx
#       REFERENCE_DIR: tests/fixtures/reference-pngs
#     run: bash .factory/planning/spikes/S6-code/ci-visual-gate.sh

set -euo pipefail

PPTX_PATH="${PPTX_PATH:?Set PPTX_PATH to the .pptx file to render}"
REFERENCE_DIR="${REFERENCE_DIR:?Set REFERENCE_DIR to the reference PNG directory}"
DIFF_ARTIFACT_DIR="${DIFF_ARTIFACT_DIR:-/tmp/slide-diffs}"
SSIM_THRESHOLD="${SSIM_THRESHOLD:-0.97}"
PSNR_THRESHOLD_DB="${PSNR_THRESHOLD_DB:-35.0}"
STRICT_PSNR="${STRICT_PSNR:-false}"

RENDER_DIR="/tmp/slideforge-rendered-$(date +%s)"
JSON_REPORT="${DIFF_ARTIFACT_DIR}/visual-diff-report.json"

# Locate this script's directory to find sibling scripts
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "slideforge CI Visual Regression Gate"
echo "=========================================="
echo "Input PPTX:    $PPTX_PATH"
echo "Reference dir: $REFERENCE_DIR"
echo "Render dir:    $RENDER_DIR"
echo "Diffs dir:     $DIFF_ARTIFACT_DIR"
echo "SSIM threshold: $SSIM_THRESHOLD"
echo "PSNR threshold: $PSNR_THRESHOLD_DB dB"
echo ""

# Step 1: Render PPTX → PNG
echo "[1/3] Rendering PPTX to PNG..."
bash "${SCRIPT_DIR}/render-pptx.sh" "$PPTX_PATH" "$RENDER_DIR"

# Step 2: Run visual diff
echo ""
echo "[2/3] Running visual diff..."
mkdir -p "$DIFF_ARTIFACT_DIR"

STRICT_FLAG=""
if [[ "$STRICT_PSNR" == "true" ]]; then
    STRICT_FLAG="--strict"
fi

export SSIM_THRESHOLD PSNR_THRESHOLD_DB

DIFF_EXIT=0
python3 "${SCRIPT_DIR}/visual-diff.py" \
    "$REFERENCE_DIR" \
    "$RENDER_DIR" \
    --out "$DIFF_ARTIFACT_DIR" \
    --json "$JSON_REPORT" \
    $STRICT_FLAG \
    || DIFF_EXIT=$?

# Step 3: Report
echo ""
echo "[3/3] Summary"
if [[ -f "$JSON_REPORT" ]]; then
    # Extract summary without jq dependency
    python3 - <<EOF
import json
with open("$JSON_REPORT") as f:
    r = json.load(f)
s = r["summary"]
print(f"  Pass: {s['pass']}  Warn: {s['warn']}  Fail: {s['fail']}  Total: {s['total']}")
if s["fail"] > 0:
    print("")
    print("  Failing slides:")
    for slide in r["slides"]:
        if slide["status"] == "FAIL":
            print(f"    {slide['slide']}: SSIM={slide['ssim_val']:.4f} PSNR={slide['psnr_val']:.1f}dB diff={slide['pixel_diff_pct']:.2f}%")
EOF
fi

if [[ $DIFF_EXIT -ne 0 ]]; then
    echo ""
    echo "VISUAL REGRESSION DETECTED"
    echo "Diff images: $DIFF_ARTIFACT_DIR"
    echo ""
    echo "To investigate:"
    echo "  1. Open diff images in $DIFF_ARTIFACT_DIR"
    echo "  2. Identify whether the regression is a real change or a rendering env difference"
    echo "  3. If intentional: update reference PNGs with:"
    echo "     cp ${RENDER_DIR}/*.png ${REFERENCE_DIR}/"
    echo "     git add ${REFERENCE_DIR}/*.png"
    echo "     git commit -m 'test(visual): update reference PNGs for <change description>'"
    echo ""
    exit 1
fi

echo ""
echo "Visual regression gate: PASSED"
exit 0
