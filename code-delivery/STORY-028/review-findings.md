---
document_type: review-findings
story_id: STORY-028
pr_number: 34
---

# Review Findings — STORY-028 PR #34

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|-------|-----------|---------|
| 1 (security) | 3 LOW | 0 | N/A (informational) | 0 | PASS |
| 1 (pr-review) | 1 LOW | 0 | 0 | 0 | APPROVE |

**PR-merge gate: CLEAN (0 CRIT/HIGH/MED)**

## Security Review Findings (Cycle 1)

| ID | Severity | Category | Description | Resolution |
|----|----------|----------|-------------|------------|
| SEC-1 | LOW (CWE-682) | informational | `from_inches` division by 1_000 not `checked_div` | Divisor is compile-time constant; no div-by-zero possible. No fix required. |
| SEC-2 | LOW (CWE-400) | informational | Recursive `check_inline_node` depth bound uses counter not stack depth | `MAX_INLINE_DEPTH=64` adequate; VP-045 Kani proof pending Phase 6. No fix required. |
| SEC-3 | LOW (CWE-912) | informational | `build_fill_spec` and `parse_hex_color` are `pub` under `#[cfg(test)]` | Test-helper visibility; not in production binary. No fix required. |

## PR Review Findings (Cycle 1)

| ID | Severity | Category | File | Line | Description | Route | Resolution |
|----|----------|----------|------|------|-------------|-------|------------|
| PRR-1 | LOW | code-quality | `shapes.rs` | 104 | `from_em(any, 0)` returns `Some(Emu(0))` silently — no guard, no test for `em_in_emu=0` | suggestion | Non-blocking; inline comment posted. Fix deferred per production-grade principle — `debug_assert!` + doc update acceptable in STORY-074 (brand-em-sizing) which owns em_in_emu wiring. |

## Triage Notes

All findings are LOW/informational. Zero blocking findings. PR-merge gate is CLEAN.

The `from_em(em_in_emu=0)` finding (PRR-1) is PLAUSIBLE but not reachable in current production code (layout.rs hardcodes `DEFAULT_EM_IN_EMU=457_200`). The correct fix is to add a `debug_assert!(em_in_emu > 0)` and update the function contract doc. This is tracked as a follow-up in STORY-074 (brand-em-sizing), which will wire the actual `BrandFonts.font_size_emu` field and is the correct scope for establishing the full em_in_emu precondition contract.
