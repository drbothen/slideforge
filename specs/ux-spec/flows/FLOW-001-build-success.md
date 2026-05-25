---
document_type: ux-spec-flow
flow_id: "FLOW-001"
flow_name: "Build Success"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
traces_to: UX-INDEX.md
screens_involved:
  - SCR-001
prd_requirements:
  - "PRD §3.1 CLI Command Surface"
  - "PRD §3.2 Exit Code Semantics"
  - "BC-1.15.001"
---

# Flow: Build Success (FLOW-001)

> **Sharded UX flow (DF-021).** Navigate via `UX-INDEX.md`.

## Summary

The happy path for `slideforge build`. User runs the command; all pipeline stages
complete without error; output files are written to disk; success summary printed;
exit code 0.

---

## Flow Steps

| Step | Actor | Action | Terminal Output | Exit |
|------|-------|--------|----------------|------|
| 1 | User | `slideforge build deck.sf` | (no output yet) | — |
| 2 | System | Parse `.sf` file(s) | (silent in default; per-stage line in --verbose) | — |
| 3 | System | Evaluate variables, data sources | (silent in default) | — |
| 4 | System | Layout all slides | (silent in default) | — |
| 5 | System | Export to requested formats | (silent in default) | — |
| 6 | System | Print success summary | `Wrote 1 file to dist/ in 342ms` + file list | 0 |

---

## Success Condition

All of:
- Exit code 0
- At least one output file written to `--output` directory
- Success summary line printed (unless `--quiet`)
- JSON result with `"status": "success"` if `--json`

---

## Variations

| Variation | Step Difference | Output Difference |
|-----------|----------------|-------------------|
| `--verbose` | Steps 2-5 print per-stage timing lines | More output; same exit code |
| `--quiet` | Steps 2-6 produce no terminal output | Silent; exit 0 only |
| `--json` | All steps produce JSON output to stdout | JSON with `"status": "success"` |
| `--workspace` | Steps 2-5 repeated per workspace member | Per-member lines + summary |
| With warnings | Same steps; warnings printed before success line | Exit 0; warning count shown |

---

## E2E Test Required

Yes. The canonical test case: build `tests/fixtures/canonical-deck.sf --format pptx`
and assert exit 0, output file exists, file size > 0, success line matches pattern.
