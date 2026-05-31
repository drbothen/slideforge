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
echo "  NOTE (SEC-003): The authoritative load-bearing check for this invariant"
echo "  is the Rust integration test 'no_forbidden_deps' (tests/no_forbidden_deps.rs)."
echo "  This shell script is a coarse defense-in-depth layer that catches obvious"
echo "  violations early in CI. The Rust test is the source of truth."
echo ""

PDF_SRC_DIR="${WORKSPACE_ROOT}/crates/slideforge-pdf/src"
if [[ ! -d "${PDF_SRC_DIR}" ]]; then
    echo "  FAIL: crates/slideforge-pdf/src not found — cannot run no-subprocess source check" >&2
    echo "        Rename or relocation of the crate src/ directory must be reflected here." >&2
    exit 1
else
    # Positive-coverage guard: count .rs files to be scanned.
    # The crate has at least 6 source files; fewer than 5 means the directory
    # is unexpectedly sparse and the check would be a no-op pass.
    RS_FILE_COUNT=$(find "${PDF_SRC_DIR}" -name "*.rs" | wc -l | tr -d ' ')
    if [[ "${RS_FILE_COUNT}" -lt 5 ]]; then
        echo "  FAIL: only ${RS_FILE_COUNT} .rs file(s) found in crates/slideforge-pdf/src/" >&2
        echo "        Expected at least 5. The source check would be a no-op — failing closed." >&2
        exit 1
    fi
    echo "  INFO: scanning ${RS_FILE_COUNT} .rs file(s) in crates/slideforge-pdf/src/"

    # Grep for actual use-site patterns (non-comment lines only).
    # SEC-003: Strip BOTH line comments (`//`) and block comment lines (`/*`).
    # The authoritative check is the Rust integration test; this is defense-in-depth.
    #
    # Filter logic (awk):
    #   - Split on ':' to separate file, line number, and content.
    #   - Reconstruct the content portion (fields 3..NF joined with ':').
    #   - Strip leading whitespace from content.
    #   - Skip lines whose trimmed content starts with '//' (line comments,
    #     including '///' doc comments).
    #   - Skip lines whose trimmed content starts with '/*' (block comment lines).
    #   - Only print lines that survive both filters — these are real use sites.
    SUBPROCESS_HITS=$(grep -rn "std::process\|Command::new\|process::Command" \
        "${PDF_SRC_DIR}/" 2>/dev/null \
        | awk -F: '{
              rest=$3;
              for(i=4;i<=NF;i++) rest=rest":"$i;
              gsub(/^[ \t]*/,"",rest);
              if (substr(rest,1,2) != "//" && substr(rest,1,2) != "/*") print
          }' \
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
