#!/usr/bin/env bash
# check-pdf-deps.sh — CI guard against browser-based and C-library PDF deps.
#
# BC-4.03.002 AC-002 / AC-007 / AC-009
#
# Usage:
#   scripts/check-pdf-deps.sh           # Checks workspace Cargo.lock
#   scripts/check-pdf-deps.sh --help    # Show help
#
# Exit codes:
#   0 — All checks passed; no forbidden deps found.
#   1 — One or more forbidden deps found; CI should fail.
#
# Run this from the workspace root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CARGO_LOCK="${WORKSPACE_ROOT}/Cargo.lock"

if [[ "${1:-}" == "--help" ]]; then
    echo "Usage: scripts/check-pdf-deps.sh"
    echo ""
    echo "Checks that Cargo.lock contains no browser-based or C-library PDF deps."
    echo "BC-4.03.002 compliance check."
    exit 0
fi

if [[ ! -f "${CARGO_LOCK}" ]]; then
    echo "ERROR: Cargo.lock not found at ${CARGO_LOCK}"
    echo "Run 'cargo build --workspace' first to generate Cargo.lock."
    exit 1
fi

echo "==> Checking Cargo.lock for forbidden PDF dependencies..."
echo "    Cargo.lock: ${CARGO_LOCK}"
echo ""

FOUND_FORBIDDEN=0

# Forbidden browser-based PDF crates (BC-4.03.002 invariant 1)
BROWSER_FORBIDDEN=(
    "chromium"
    "headless-chrome"
    "puppeteer-rs"
    "wkhtmltopdf"
)

# Forbidden C-library FFI PDF bindings (BC-4.03.002 invariant 4)
FFI_FORBIDDEN=(
    "libharu"
    "cairo-rs"
    "pango-sys"
    "freetype-sys"
    "harfbuzz-sys"
)

echo "[Browser-based PDF deps] — all must be ABSENT:"
for crate in "${BROWSER_FORBIDDEN[@]}"; do
    if grep -q "name = \"${crate}\"" "${CARGO_LOCK}"; then
        echo "  FAIL: Found forbidden browser-based PDF dep: ${crate}"
        FOUND_FORBIDDEN=1
    else
        echo "  OK:   ${crate} is absent"
    fi
done

echo ""
echo "[C-library FFI PDF deps] — all must be ABSENT:"
for crate in "${FFI_FORBIDDEN[@]}"; do
    if grep -q "name = \"${crate}\"" "${CARGO_LOCK}"; then
        echo "  FAIL: Found forbidden C-library FFI dep: ${crate}"
        FOUND_FORBIDDEN=1
    else
        echo "  OK:   ${crate} is absent"
    fi
done

echo ""
echo "[Required pure-Rust PDF engine] — must be PRESENT:"
if grep -q "name = \"krilla\"" "${CARGO_LOCK}"; then
    echo "  OK:   krilla is present"
else
    echo "  WARN: krilla not yet in Cargo.lock (run 'cargo build -p slideforge-pdf' first)"
    # Not a hard failure — Cargo.lock may not be populated yet before first build
fi

echo ""
echo "[No-subprocess source check] — AC-008 / scope-directive Decision 3:"
echo "  Assert zero std::process / Command::new / process::Command usage in"
echo "  crates/slideforge-pdf/src/ (source-level enforcement of AC-008)."
echo ""

PDF_SRC_DIR="${WORKSPACE_ROOT}/crates/slideforge-pdf/src"
if [[ ! -d "${PDF_SRC_DIR}" ]]; then
    echo "  WARN: crates/slideforge-pdf/src not found — skipping source check"
else
    # Grep for actual use-site patterns (non-comment lines only).
    # Use perl-regex look-ahead to skip lines whose trimmed content starts
    # with `//` (Rust comment lines, including doc comments `///`).
    # The output of grep has format "file:lineno:content"; we filter on the
    # content portion (after the second colon).
    SUBPROCESS_HITS=$(grep -rn "std::process\|Command::new\|process::Command" \
        "${PDF_SRC_DIR}/" 2>/dev/null \
        | awk -F: '{ rest=$3; for(i=4;i<=NF;i++) rest=rest":"$i; gsub(/^[ \t]*/,"",rest); if (substr(rest,1,2) != "//") print }' \
        || true)
    if [[ -n "${SUBPROCESS_HITS}" ]]; then
        echo "${SUBPROCESS_HITS}"
        echo "  FAIL: subprocess usage found in crates/slideforge-pdf/src/"
        echo "        BC-4.03.002 AC-008 requires zero std::process in slideforge-pdf."
        FOUND_FORBIDDEN=1
    else
        echo "  OK:   no std::process / Command::new / process::Command in slideforge-pdf/src/"
    fi
fi

echo ""
if [[ "${FOUND_FORBIDDEN}" -eq 1 ]]; then
    echo "FAIL: Forbidden PDF dependencies or subprocess usage found."
    echo "      BC-4.03.002 requires a pure-Rust PDF stack (krilla + pdf-writer)."
    echo "      Remove the forbidden dependencies from slideforge-pdf/Cargo.toml."
    exit 1
else
    echo "PASS: No forbidden PDF dependencies or subprocess usage found."
    exit 0
fi
