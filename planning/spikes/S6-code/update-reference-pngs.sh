#!/usr/bin/env bash
# S6 Spike: Reference PNG update workflow
#
# Renders the current test-fixture.pptx and replaces the committed reference PNGs.
# Run this when you intentionally change the PPTX output and want to "bless" new references.
#
# Usage:
#   ./update-reference-pngs.sh [pptx-path] [reference-dir]
#
# Defaults:
#   pptx-path:     tests/fixtures/test-fixture.pptx
#   reference-dir: tests/fixtures/reference-pngs/
#
# Workflow (matches CI pipeline decision in S6 spike):
#   1. Render PPTX → PNG via LibreOffice
#   2. Show visual diff vs current references (if any exist)
#   3. Copy new PNGs to reference dir
#   4. Remind operator to commit the change with a meaningful message

set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
PPTX_PATH="${1:-${REPO_ROOT}/tests/fixtures/test-fixture.pptx}"
REFERENCE_DIR="${2:-${REPO_ROOT}/tests/fixtures/reference-pngs}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RENDER_DIR="/tmp/slideforge-ref-update-$(date +%s)"

echo "=========================================="
echo "slideforge Reference PNG Update Workflow"
echo "=========================================="
echo "PPTX:          $PPTX_PATH"
echo "Reference dir: $REFERENCE_DIR"
echo "Render dir:    $RENDER_DIR"
echo ""

# Step 1: Render
echo "[1/4] Rendering PPTX → PNG..."
bash "${SCRIPT_DIR}/render-pptx.sh" "$PPTX_PATH" "$RENDER_DIR"

# Step 2: Diff vs existing references (informational only — not gating)
if [[ -d "$REFERENCE_DIR" ]] && ls "$REFERENCE_DIR"/*.png &>/dev/null 2>&1; then
    echo ""
    echo "[2/4] Comparing new renders vs existing references (informational)..."
    python3 "${SCRIPT_DIR}/visual-diff.py" \
        "$REFERENCE_DIR" \
        "$RENDER_DIR" \
        --out "/tmp/slideforge-update-diffs" \
        || true  # Don't fail — this is informational before we update
else
    echo ""
    echo "[2/4] No existing references to compare against (first-time setup)."
fi

# Step 3: Copy new PNGs to reference dir
echo ""
echo "[3/4] Copying new PNGs to reference directory..."
mkdir -p "$REFERENCE_DIR"
cp "$RENDER_DIR"/*.png "$REFERENCE_DIR/"
echo "  Copied:"
ls "$REFERENCE_DIR"/*.png | while read -r f; do echo "    $f"; done

# Step 4: Remind to commit
echo ""
echo "[4/4] Next steps"
echo "  These reference PNGs are now in your working tree."
echo "  Review the changes (open PNGs, check diffs) then commit:"
echo ""
echo "    git add ${REFERENCE_DIR}/*.png"
echo "    git commit -m 'test(visual): update reference PNGs — <describe why>'"
echo ""
echo "  Reference PNG update decisions are HUMAN-APPROVED changes."
echo "  They record the new 'blessed' expected output. The commit message"
echo "  must explain WHY the expected output changed."
echo ""
echo "  If this is a first-time setup (no prior references):"
echo "    git add ${REFERENCE_DIR}/"
echo "    git commit -m 'test(visual): add initial reference PNGs for visual regression'"
