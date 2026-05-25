---
document_type: ux-spec-flow
flow_id: "FLOW-002"
flow_name: "Build Error Recovery"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
traces_to: UX-INDEX.md
screens_involved:
  - SCR-001
  - SCR-006
prd_requirements:
  - "PRD §5 Error Taxonomy"
  - "PRD §3.2 Exit Code Semantics"
  - "BC-1.15.002 (error accumulation)"
  - "BC-1.15.003"
  - "q16-q25-decisions.md Q23 (error recovery)"
---

# Flow: Build Error Recovery (FLOW-002)

> **Sharded UX flow (DF-021).** Navigate via `UX-INDEX.md`.

## Summary

The primary error-correction loop. User runs build, encounters errors, reads error
output, fixes the source file, re-runs build. This flow covers the diagnostic UX
from first failure through resolution.

---

## Flow Steps (Fatal Error Case)

| Step | Actor | Action | Terminal Output | Exit |
|------|-------|--------|----------------|------|
| 1 | User | `slideforge build deck.sf` | (no output yet) | — |
| 2 | System | Parse `.sf` file — encounters errors | (accumulating, not printing yet) | — |
| 3 | System | Continue parsing to accumulate ALL errors (Q23) | (accumulating) | — |
| 4 | System | Print ALL accumulated errors (SCR-006 format) | Multi-error block per SCR-006 | — |
| 5 | System | Print error count footer | `error: aborting due to N previous errors` | 1 |
| 6 | User | Read errors; identify file, line, col from each | — | — |
| 7 | User | Open deck.sf in editor; fix all reported errors | — | — |
| 8 | User | Re-run `slideforge build deck.sf` | Success (FLOW-001) or new error set | 0 or 1 |

---

## Flow Steps (Warn-Only Recovery with --warn-only)

| Step | Actor | Action | Terminal Output | Exit |
|------|-------|--------|----------------|------|
| 1 | User | `slideforge build deck.sf --warn-only` | (no output yet) | — |
| 2-4 | System | Parse + eval + layout; validation errors become warnings | Warnings printed per SCR-006 | — |
| 5 | System | Export output WITH error-slide placeholders | Wrote N files... + warning count | 0 |
| 6 | User | Open output PPTX; sees error-slide placeholders at affected positions | — | — |
| 7 | User | Fix issues in source; rebuild without --warn-only | Clean build (FLOW-001) | 0 |

---

## Error Categories and Recovery Hints

| Error Category | Recovery Hint (shown in SCR-006) | Fix Action |
|---------------|----------------------------------|-----------|
| E-PAR-xxx | Line/col + source excerpt; suggestion | Edit deck.sf at reported location |
| E-EVL-001 | List of in-scope variables | Add var to `vars:` block |
| E-A11-001 | `Add alt "..." or decorative: true` | Add alt text to image/chart/diagram |
| E-A11-002 | `Add label "..."` | Add label to color-coded element |
| E-BRD-001 | Check path; use absolute path | Fix brand template path |
| E-DAT-001 | `Use --offline to skip HTTP sources` | Fix URL or use --offline |
| E-CFG-001 | List of defined variants | Fix variant name or add it |

---

## Variations

| Variation | Behavior |
|-----------|---------|
| Multiple error categories in one file | All accumulated; printed in order of occurrence |
| Error in @included file | File path in error shows included file, not entry file |
| Error after successful build (regression) | Same flow; user expects this was previously working |
| CI environment (--json) | JSON output with `"status": "error"`; same exit codes |

---

## E2E Test Required

Yes. Test cases:
1. Build a fixture with a known E-PAR-001 error; assert exit 1, stderr contains "E-PAR-001", file:line:col present
2. Build a fixture with multiple errors; assert all error codes appear in stderr
3. Build with --warn-only on a file with E-A11-001; assert exit 0, output file written
