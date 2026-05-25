---
document_type: ux-spec-screen
screen_id: "SCR-002"
screen_name: "CLI: watch"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §2.5 BC-5.05.001-005"
  - "interface-definitions.md §1.3"
  - "q1-decision-final.md §1 (Data Binding — watch mode polls HTTP)"
  - "q16-q25-decisions.md Q17 (watch mode always warn-only)"
---

# Screen: CLI watch (SCR-002)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.
> Watch mode is a long-running process. All output rules from SCR-001 apply
> plus the watch-specific banner and event log described here.

## Purpose and User Context

`slideforge watch deck.sf` starts a long-running process that:
1. Compiles the deck immediately on launch
2. Starts an axum HTTP + WebSocket server
3. Watches all .sf files and file-based data sources for changes
4. Re-runs the full pipeline on any change (incremental: < 50ms target, NFR-002)
5. Pushes delta to WebSocket clients (web preview auto-updates)

Watch mode always uses warn-only mode (per Q17). Parse errors are still fatal and
shown in the terminal, but the web preview shows error-slide placeholders rather
than stopping.

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Launch banner | Watch mode header with URL | Shown once on start |
| ELM-002 | Initial build result | Success or error summary | Same format as SCR-001 |
| ELM-003 | File change event | `[watch] deck.sf changed — rebuilding...` | On each file change |
| ELM-004 | Rebuild result | Success line or error block | Per SCR-001 / SCR-006 |
| ELM-005 | HTTP poll event | `[watch] kpis data refreshed — rebuilding...` | HTTP data source refresh |
| ELM-006 | Interrupt message | `[watch] Stopping. Bye.` | On SIGINT (exit 130) |
| ELM-007 | Port conflict error | `error[E-CFG-007]: Port 3000 is already in use` | If port occupied |

---

## Output Format Specification

### Launch Banner

```
slideforge watch — deck.sf

  Preview: http://127.0.0.1:3000
  Watching: deck.sf, shared/header.sf, data/metrics.json
  HTTP data sources polled every 30s

  Parsed deck.sf in 45ms — 25 slides
  Wrote preview in 89ms

  Waiting for changes... (Ctrl+C to stop)
```

Color application:
- `slideforge watch —` header: `color.stage.label`
- `http://127.0.0.1:3000`: `color.file.path` (underlined, clickable in modern terminals)
- Watched files list: `color.file.path`
- `Waiting for changes...`: `color.timing` (dim)
- `(Ctrl+C to stop)`: `color.timing` (dim)

### File Change Event

```
[watch] deck.sf changed (12:34:56) — rebuilding...
  Rebuilt in 23ms — 25 slides (no change to slide count)

[watch] shared/header.sf changed (12:35:12) — rebuilding...

error[E-PAR-001]: Unexpected indentation at shared/header.sf:8:3.
  --> shared/header.sf:8:3
   |
 8 |    title "Q3 Review"
   |    ^ expected 2 spaces, found 3
   |
   = hint: slideforge requires 2-space indentation multiples.

[watch] Build failed. Waiting for changes... (Ctrl+C to stop)
```

Color application:
- `[watch]` prefix: `color.timing` (dim)
- Timestamp in parentheses: `color.timing`
- Rebuild success line: `color.success`
- Error block: standard SCR-006 colors
- `[watch] Build failed.` line: `color.error`

### HTTP Data Source Poll

```
[watch] kpis refreshed from https://api.acme.com/v1/kpis (2.3 KB) — rebuilding...
  Rebuilt in 18ms — 25 slides
```

### SIGINT Shutdown

```
^C
[watch] Stopping. Bye.
```

Clean exit. Exit code 130.

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | Launch `slideforge watch deck.sf` | Banner prints; initial build runs; server starts on port 3000 | Port conflict: E-CFG-007; source not found: E-PAR-005 |
| INT-002 | .sf file saved | Rebuild triggered; result printed; WebSocket push to browser | Parse error: shown in terminal; error overlay in web preview |
| INT-003 | File-based data source updated | Rebuild triggered (same path as INT-002) | Data parse failure: E-DAT-003 warning; placeholder in preview |
| INT-004 | HTTP data poll (every 30s default) | If data changed: rebuild; if unchanged: silent | HTTP fail: E-DAT-001 warning; previous data kept; explicit notice |
| INT-005 | Ctrl+C (SIGINT) | Shutdown message; exit 130 | N/A |
| INT-006 | `--port` conflict | E-CFG-007 printed; exit 4 | N/A |
| INT-007 | Browser opens `http://127.0.0.1:3000` | Web preview served (SCR-007) | 404 if watch not running |

---

## Watch List Display

On launch, the terminal prints every watched path. The list must be accurate —
if a file is watched, it appears here. If not watched (e.g., HTTP source polling
is separate), it shows with a different label.

```
  Watching: deck.sf
            shared/header.sf
            data/metrics.json
  Polling:  https://api.acme.com/v1/kpis (every 30s)
```

---

## Accessibility

- Timestamp on each event line enables log review without color
- Error code in every error (machine-filterable with grep)
- Exit code 130 for SIGINT (scripting-safe)
- URL displayed as plain text (works in all terminals; modern terminals auto-link)

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 60 chars | Watched file list wrapped; URL on separate line |
| 60-120 chars | Default layout |
| No TTY | Banner still prints; ANSI stripped; useful for log capture |
