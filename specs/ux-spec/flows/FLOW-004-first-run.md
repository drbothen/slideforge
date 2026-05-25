---
document_type: ux-spec-flow
flow_id: "FLOW-004"
flow_name: "First Run"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
traces_to: UX-INDEX.md
screens_involved:
  - SCR-009
  - SCR-001
prd_requirements:
  - "PRD §1.4 Target Users"
  - "UX Principle P4 (Zero-config start)"
  - "BC-1.15.001"
---

# Flow: First Run (FLOW-004)

> **Sharded UX flow (DF-021).** Navigate via `UX-INDEX.md`.

## Summary

The complete new user journey from binary installation to a working first deck.
Target time: under 2 minutes. No flags required. No config editing required to
get a working PPTX output.

---

## Flow Steps

| Step | Actor | Action | Terminal Output | Exit |
|------|-------|--------|----------------|------|
| 1 | User | `slideforge --version` | `slideforge 1.0.0` | 0 |
| 2 | User | `slideforge init` (in new directory) | Scaffold files created; file list; next-steps | 0 |
| 3 | User | *(optional)* Open deck.sf in editor; read scaffold | — | — |
| 4 | User | `slideforge build deck.sf` | `Wrote 1 file to dist/ in 285ms` + path | 0 |
| 5 | User | Open `dist/deck.pptx` in PowerPoint / Keynote | 3-slide deck renders correctly | — |

---

## Success Condition

After step 4:
- Exit code 0
- `dist/deck.pptx` exists and is > 0 bytes
- Success line printed

After step 5:
- PPTX opens in Office / Keynote without errors
- 3 slides visible: Title, Content, End

---

## Scaffolded Deck Guarantees (SCR-009)

The generated `deck.sf` must be immediately buildable with NO modifications. This
means the scaffold must:
- Have `slideforge_version "1"` declared
- Have `lang "en-US"` (no E-A11-003 warning)
- Have a `brand "brand.toml"` reference and a valid scaffolded `brand.toml`
- Have 3 valid slides (title, content, end types) with no undefined variables
- Use only the default brand colors (no custom palette required)

If a freshly-scaffolded project produces any error on first build, that is a P0
defect in the scaffold implementation.

---

## E2E Test Required

Yes. The "scaffold round-trip" test:
1. `slideforge init` in a temp directory
2. `slideforge build deck.sf` — must exit 0
3. Verify `dist/deck.pptx` exists and size > 0
4. Verify zero errors in stderr

This test must run in CI on every PR.
