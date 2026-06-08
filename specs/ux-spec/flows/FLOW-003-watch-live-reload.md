---
document_type: ux-spec-flow
flow_id: "FLOW-003"
flow_name: "Watch Live Reload"
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
modified: 2026-06-07
phase: 1c
traces_to: UX-INDEX.md
screens_involved:
  - SCR-002
  - SCR-007
  - SCR-008
prd_requirements:
  - "PRD §2.5 BC-5.05.001-005"
  - "PRD §4 NFR-002 (< 50ms incremental rebuild — DEFERRED to v1.x; see nfr-catalog v1.3)"
  - "PRD §4 NFR-001 (< 500ms cold build — v1.0 watch rebuild path governed by this)"
  - "interface-definitions.md §1.3"
  - "q16-q25-decisions.md Q17 (watch always warn-only)"
  - "q1-decision-final.md §1 (HTTP data polling)"
---

# Flow: Watch Live Reload (FLOW-003)

> **Sharded UX flow (DF-021).** Navigate via `UX-INDEX.md`.

## Summary

The primary authoring feedback loop. User starts watch mode, opens the browser
preview, edits source files, and sees the slide deck update in near-real-time.
This flow covers one full edit-rebuild-reload cycle.

---

## Flow Steps (Happy Path)

| Step | Actor | Interface | Action | Output |
|------|-------|-----------|--------|--------|
| 1 | User | Terminal | `slideforge watch deck.sf` | Launch banner (SCR-002 ELM-001) |
| 2 | System | Terminal | Initial build runs (all stages) | Success summary; "Waiting for changes..." |
| 3 | System | Browser | HTTP server ready; user opens preview URL | Web preview renders (SCR-007) |
| 4 | System | Browser | WebSocket connects | Green dot; "Live" status (SCR-007 ELM-003) |
| 5 | User | Editor | Saves deck.sf | (no output yet) |
| 6 | System | Terminal | File change detected | `[watch] deck.sf changed (HH:MM:SS) — rebuilding...` |
| 7 | System | Browser | `build_started` WS message | Loading overlay on changed slides |
| 8 | System | System | Full re-evaluation rebuild (NFR-002 incremental target deferred to v1.x; v1.0 performs full rebuild) | (internal) |
| 9 | System | Terminal | Rebuild success | `Rebuilt in 18ms — 25 slides` |
| 10 | System | Browser | `slide_delta` WS message | Only changed slide SVGs swap in-place |
| 11 | System | Browser | Overlays removed | Slides show updated content |
| 12 | User | — | Continues editing | Cycle repeats from Step 5 |

---

## Flow Steps (Error During Watch)

| Step | Actor | Interface | Action | Output |
|------|-------|-----------|--------|--------|
| 1-5 | As above | — | — | — |
| 6 | System | Terminal | File change detected | Rebuild line |
| 7 | System | Browser | `build_started` WS message | Loading overlay |
| 8 | System | System | Rebuild hits parse error | (internal) |
| 9 | System | Terminal | `[watch] Build failed.` + error block (SCR-006) | Error diagnostic with file:line:col |
| 10 | System | Browser | `build_error` WS message | Error overlay on all slides; orange dot |
| 11 | User | Editor | Reads error in terminal; fixes source; saves | — |
| 12 | System | — | New rebuild triggered (Step 6 again) | Success or next error |

---

## HTTP Data Source Polling Sub-Flow

| Step | Actor | Interface | Action | Output |
|------|-------|-----------|--------|--------|
| A | System | Background | HTTP poll interval expires (default: 30s) | — |
| B | System | System | Fetch all HTTP data sources | — |
| C1 (data changed) | System | Terminal | `[watch] kpis refreshed — rebuilding...` | Rebuild triggered |
| C2 (data unchanged) | System | — | Silent — no rebuild | (nothing) |
| D (fetch fails) | System | Terminal | `warning[E-DAT-001]: HTTP fetch failed — keeping previous data` | Warning; no rebuild |

---

## Performance Gate

> **NFR-002 deferral (approved 2026-06-07):** The < 50ms incremental rebuild target
> (NFR-002, comemo-based) is DEFERRED to v1.x. v1.0 watch mode performs a full
> re-evaluation on every change; perceived latency is governed by NFR-001 (< 500ms
> cold build). The live-reload UX (file watch, WebSocket push, browser update)
> ships in v1.0 — only the sub-50ms performance commitment is deferred.

The step 8→10 path (file save → browser update) for v1.0 is bounded by the full
rebuild path (NFR-001 < 500ms). The CI E2E budget below uses a 500ms assertion.
When comemo incremental lands in v1.x, this gate tightens to < 200ms total:
- File system event detection: < 10ms
- Incremental pipeline: < 50ms (NFR-002, v1.x target)
- WebSocket message: < 1ms
- Browser DOM swap: < 16ms

---

## E2E Test Required

Yes. Integration test:
1. Start `slideforge watch` on a test deck
2. Assert browser preview loads and green dot appears
3. Modify a field in the deck source file
4. Assert browser DOM updates within 500ms (CI budget; governed by NFR-001 full rebuild in v1.0)
5. Assert terminal shows rebuild line with "ms" timing
6. Introduce an error into the source file
7. Assert browser shows error overlay
8. Fix the error
9. Assert browser returns to normal slide display
