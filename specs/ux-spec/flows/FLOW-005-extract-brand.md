---
document_type: ux-spec-flow
flow_id: "FLOW-005"
flow_name: "Extract Brand"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
traces_to: UX-INDEX.md
screens_involved:
  - SCR-003
  - SCR-001
prd_requirements:
  - "PRD §3.1 CLI Command Surface (extract-brand)"
  - "BC-2.01.003"
  - "error-taxonomy.md E-BRD-001 through E-BRD-004"
---

# Flow: Extract Brand (FLOW-005)

> **Sharded UX flow (DF-021).** Navigate via `UX-INDEX.md`.

## Summary

The migration path for users with an existing `.pptx` brand template. User has
a corporate PowerPoint template and wants to use it as the brand source for
slideforge. This flow covers extraction and the first build with the extracted brand.

---

## Flow Steps

| Step | Actor | Action | Terminal Output | Exit |
|------|-------|--------|----------------|------|
| 1 | User | `slideforge extract-brand corporate.pptx` | Extraction progress + slot table | 0 (warnings possible) |
| 2 | User | Open `brand.toml` in editor; review extracted values | — | — |
| 3 | User | Add `[brand] template = "brand.toml"` to `slideforge.toml` | — | — |
| 4 | User | `slideforge build deck.sf` | Build with brand applied; success summary | 0 |
| 5 | User | Open output PPTX; verify brand colors and typography match template | — | — |

---

## Success Condition

After step 1:
- `brand.toml` exists with all 12 color slots populated
- Any inferred slots shown as E-BRD-003 warnings (exit 0)

After step 4:
- PPTX uses brand colors from extracted `brand.toml`
- No E-BRD-001 errors (brand file found)

---

## Error Paths

| Error | When | Recovery |
|-------|------|---------|
| E-BRD-001 | Template file path wrong | Fix path; re-run |
| E-BRD-002 | Template file corrupted or not PPTX | Use a different template |
| E-BRD-003 (warning) | Some color slots could not be directly extracted | Review inferred values in brand.toml; adjust manually |
| E-BRD-004 (warning) | Brand font not available on this host | Install font or specify fallback in brand.toml |

---

## E2E Test Required

Yes:
1. `slideforge extract-brand tests/fixtures/sample-brand-template.pptx`
2. Assert exit 0 and `brand.toml` written
3. Assert `brand.toml` contains all 12 color slot keys
4. `slideforge build tests/fixtures/branded-deck.sf` with the extracted brand
5. Assert exit 0 and PPTX output written
