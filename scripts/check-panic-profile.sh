#!/usr/bin/env bash
# check-panic-profile.sh — CI guard for the panic=unwind safety perimeter.
#
# BC-5.02.001 EC-003 / STORY-049 AC-008 / Architecture Compliance Rule 4
#
# `std::panic::catch_unwind` — used at the plugin dispatch boundary to isolate
# third-party plugin panics — is a no-op when `panic = "abort"` is active in a
# shipped profile.  If a future PR silently flips [profile.release] to
# panic=abort, EC-003 becomes FALSE in shipped builds with NO test failure
# (tests run under the dev profile which has no panic=abort override).
#
# This script enforces the invariant at the Cargo.toml source level so CI
# catches the regression before any binary is built or deployed.
#
# Usage:
#   scripts/check-panic-profile.sh           # Checks workspace Cargo.toml
#   scripts/check-panic-profile.sh --help    # Show help
#
# Exit codes:
#   0 — All checks passed; catch_unwind safety perimeter is intact.
#   1 — Violation found; CI must fail.
#
# Run this from the workspace root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CARGO_TOML="${WORKSPACE_ROOT}/Cargo.toml"

if [[ "${1:-}" == "--help" ]]; then
    echo "Usage: scripts/check-panic-profile.sh"
    echo ""
    echo "Verifies that shipped Cargo profiles (release, dist, bench) declare"
    echo "panic = \"unwind\" and contain no panic = \"abort\"."
    echo "BC-5.02.001 EC-003 / STORY-049 AC-008 compliance check."
    exit 0
fi

if [[ ! -f "${CARGO_TOML}" ]]; then
    echo "ERROR: Cargo.toml not found at ${CARGO_TOML}" >&2
    echo "Run this script from the workspace root, or ensure Cargo.toml exists." >&2
    exit 1
fi

echo "==> Checking Cargo.toml for panic=unwind safety perimeter (BC-5.02.001 EC-003)..."
echo "    Cargo.toml: ${CARGO_TOML}"
echo ""

VIOLATIONS=0
PROFILES_VALIDATED=0

# ---------------------------------------------------------------------------
# Section-aware parser helpers
#
# Strategy: scan Cargo.toml line by line, tracking the current [profile.NAME]
# section.  Only attribute lines that belong to the target section are tested,
# preventing false matches from commented-out lines or unrelated profile
# sections.
#
# Rules:
#   1. A line matching /^\s*#/ is a comment — skip it.
#   2. A line matching /^\[profile\.([a-z_]+)\]/ opens a new section.
#   3. A line matching /^\[/ (but not a profile header) closes any open
#      profile section (some other table started).
#   4. Within a profile section, /^\s*panic\s*=\s*"([^"]+)"/ is the value.
# ---------------------------------------------------------------------------

# get_profile_panic <profile_name> <cargo_toml>
# Prints the panic value found in [profile.<name>], or "" if absent.
# Skips commented-out lines.
get_profile_panic() {
    local profile="$1"
    local toml="$2"
    local in_section=0
    local value=""

    while IFS= read -r line; do
        # Skip pure comment lines (leading optional whitespace then #)
        if [[ "${line}" =~ ^[[:space:]]*# ]]; then
            continue
        fi

        # Detect opening of target profile section
        if [[ "${line}" =~ ^\[profile\."${profile}"\] ]] || \
           [[ "${line}" =~ ^\[profile\.${profile}\] ]]; then
            in_section=1
            continue
        fi

        # Detect opening of ANY other section — ends our target section
        if [[ "${line}" =~ ^\[ ]]; then
            in_section=0
            continue
        fi

        if [[ "${in_section}" -eq 1 ]]; then
            # Match: panic = "value"  (with optional spaces around =)
            if [[ "${line}" =~ ^[[:space:]]*panic[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
                value="${BASH_REMATCH[1]}"
            fi
        fi
    done < "${toml}"

    echo "${value}"
}

# ---------------------------------------------------------------------------
# RULE 1: [profile.release] must explicitly declare panic = "unwind"
# ---------------------------------------------------------------------------
echo "[profile.release] — must declare panic = \"unwind\" explicitly:"
RELEASE_PANIC="$(get_profile_panic "release" "${CARGO_TOML}")"
if [[ "${RELEASE_PANIC}" == "unwind" ]]; then
    echo "  OK:   panic = \"unwind\" found in [profile.release]"
    PROFILES_VALIDATED=$(( PROFILES_VALIDATED + 1 ))
elif [[ -z "${RELEASE_PANIC}" ]]; then
    echo "  FAIL: [profile.release] does not declare a panic strategy." >&2
    echo "        BC-5.02.001 EC-003 requires explicit panic = \"unwind\"." >&2
    VIOLATIONS=$(( VIOLATIONS + 1 ))
else
    echo "  FAIL: [profile.release] declares panic = \"${RELEASE_PANIC}\" (expected \"unwind\")." >&2
    echo "        catch_unwind is a no-op under panic = \"abort\"; EC-003 would be FALSE." >&2
    VIOLATIONS=$(( VIOLATIONS + 1 ))
fi

echo ""

# ---------------------------------------------------------------------------
# RULE 2: [profile.dist] must explicitly declare panic = "unwind"
# (explicit — NOT relying on inherits = "release"; belt-and-suspenders)
# ---------------------------------------------------------------------------
echo "[profile.dist] — must declare panic = \"unwind\" explicitly:"
DIST_PANIC="$(get_profile_panic "dist" "${CARGO_TOML}")"
if [[ "${DIST_PANIC}" == "unwind" ]]; then
    echo "  OK:   panic = \"unwind\" found in [profile.dist]"
    PROFILES_VALIDATED=$(( PROFILES_VALIDATED + 1 ))
elif [[ -z "${DIST_PANIC}" ]]; then
    echo "  FAIL: [profile.dist] does not explicitly declare panic = \"unwind\"." >&2
    echo "        Rule 4 requires the explicit declaration — not relying on inheritance." >&2
    VIOLATIONS=$(( VIOLATIONS + 1 ))
else
    echo "  FAIL: [profile.dist] declares panic = \"${DIST_PANIC}\" (expected \"unwind\")." >&2
    VIOLATIONS=$(( VIOLATIONS + 1 ))
fi

echo ""

# ---------------------------------------------------------------------------
# RULE 3: No shipped profile (release, dist, bench) may contain panic = "abort"
# ---------------------------------------------------------------------------
echo "[abort scan] — no shipped profile may contain panic = \"abort\":"
for PROFILE in release dist bench; do
    PANIC_VAL="$(get_profile_panic "${PROFILE}" "${CARGO_TOML}")"
    if [[ "${PANIC_VAL}" == "abort" ]]; then
        echo "  FAIL: [profile.${PROFILE}] contains panic = \"abort\"." >&2
        echo "        This silently breaks catch_unwind (EC-003) in shipped builds." >&2
        VIOLATIONS=$(( VIOLATIONS + 1 ))
    else
        echo "  OK:   [profile.${PROFILE}] does not contain panic = \"abort\""
    fi
done

echo ""

# ---------------------------------------------------------------------------
# POSITIVE COVERAGE: confirm PROFILES_VALIDATED reaches expected minimum
# ---------------------------------------------------------------------------
EXPECTED_VALIDATED=2   # release + dist
if [[ "${PROFILES_VALIDATED}" -lt "${EXPECTED_VALIDATED}" ]]; then
    echo "FAIL: positive-coverage check: validated ${PROFILES_VALIDATED} profiles," \
         "expected at least ${EXPECTED_VALIDATED}." >&2
    echo "      If Cargo.toml was restructured, update EXPECTED_VALIDATED in this script." >&2
    VIOLATIONS=$(( VIOLATIONS + 1 ))
fi

# ---------------------------------------------------------------------------
# Final verdict
# ---------------------------------------------------------------------------
if [[ "${VIOLATIONS}" -gt 0 ]]; then
    echo "FAIL: panic=unwind safety perimeter check failed (${VIOLATIONS} violation(s))."
    echo "      BC-5.02.001 EC-003 requires panic = \"unwind\" in all shipped profiles."
    echo "      See Cargo.toml [profile.release] and [profile.dist]."
    exit 1
else
    echo "check-panic-profile: validated ${PROFILES_VALIDATED} shipped profiles as panic=unwind"
    echo "PASS: panic=unwind safety perimeter intact. catch_unwind will function correctly in shipped builds."
    exit 0
fi
