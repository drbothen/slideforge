---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-027
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-5.05.004: File Watcher Event Loss Produces CLI Warning; Force-Rebuild Available

## Description

OS file watcher backends (inotify, FSEvents, ReadDirectoryChanges) can drop events
under high filesystem activity or when the watch queue overflows. When slideforge
detects a watcher overflow or event loss (e.g., via `IN_Q_OVERFLOW` on Linux), it
emits a CLI warning and automatically triggers a full rebuild to recover consistency.
Users can also trigger a manual force-rebuild at any time with a keypress in the
terminal.

## Preconditions

1. `slideforge watch` is running with a file watcher active.
2. An OS-reported watcher overflow or event loss condition is detected (platform-specific
   signal from the watcher library — e.g., notify crate overflow event).

## Postconditions

1. A warning is printed to stderr: `Warning: file watcher event overflow detected.
   Triggering full rebuild to recover consistency.`
2. A full re-evaluation is triggered immediately (bypassing debounce).
3. After the full rebuild, watch resumes normal event-driven operation.
4. The user can manually trigger a force-rebuild at any time by pressing `r` in the
   terminal while `slideforge watch` is active.
5. No files are written and no process is restarted — only re-evaluation is performed.

## Invariants

1. The force-rebuild is always a full re-evaluation from the current .sf file state
   on disk — not a diff-based incremental update.
2. The watcher resumes after force-rebuild; it does NOT exit or restart the watcher
   process.
3. Event loss detection depends on the underlying OS/library signal — slideforge does
   not implement its own event delivery tracking.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `IN_Q_OVERFLOW` on Linux (inotify queue full) | Warning emitted; full rebuild triggered; watch resumes |
| EC-002 | User presses `r` to force-rebuild | Full re-evaluation triggered; WebSocket update sent; confirmation message shown |
| EC-003 | Force-rebuild while previous rebuild is still in progress | Queued: force-rebuild starts immediately after current rebuild completes |
| EC-004 | No event loss ever occurs during session | No warning; watcher operates normally; `r` still works |
| EC-005 | macOS FSEvents: burst of 1000 file events | FSEvents coalesces events; notify crate delivers them as a batch; no overflow; normal behavior |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Simulated `IN_Q_OVERFLOW` event from notify crate | Warning on stderr; immediate full rebuild; WebSocket update | happy-path |
| User presses `r` during watch | Full rebuild triggered; "Rebuilding..." printed; WebSocket update | happy-path |
| `r` pressed while rebuild in progress | Rebuild queued; no duplicate concurrent evaluation | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Overflow event triggers full rebuild and warning | integration test: inject synthetic overflow event |
| VP-TBD | `r` keypress triggers full rebuild | integration test: send 'r' to stdin while watch is running |
| VP-TBD | No concurrent rebuilds (queue serialize) | integration test: rapid `r` presses; assert serial evaluation |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 |
| Capability Anchor Justification | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 — file watcher reliability and recovery mechanisms are implied by the live data refresh requirement |
| L2 Domain Invariants | DI-017 (strict mode no-output; watch is always warn-only — force-rebuild also uses warn-only) |
| Architecture Module | slideforge-cli watch + notify crate integration (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.05.001 — composes with (this BC is the watcher failure sub-case of the watch mode pipeline)
- BC-4.03.004 — depends on (WebSocket delivers force-rebuild result to the browser)

## Architecture Anchors

- `architecture/cross-cutting.md#watch-mode` — file watcher overflow handling and force-rebuild

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
