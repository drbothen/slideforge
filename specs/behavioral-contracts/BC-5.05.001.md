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

# BC-5.05.001: slideforge watch Polls .sf Files and Data Sources; Re-Evaluates on Change

## Description

`slideforge watch [source.sf]` starts the file watcher, monitors all .sf files
(including @include targets) and data source files for modifications, and triggers
a full re-evaluation when any change is detected. HTTP data sources are polled at
a configurable interval (default: 60s). The re-evaluation runs in warn-only mode
(never strict mode) so the live preview is never blocked by validation errors.

## Preconditions

1. `slideforge watch` is invoked with a valid or newly-created .sf source file path.
2. The filesystem supports inotify/FSEvents/ReadDirectoryChanges (all major platforms).
3. The web preview server from BC-4.03.004 is started alongside the watcher.

## Postconditions

1. File change events on any .sf file in the dependency graph trigger re-evaluation
   within 200ms of the OS event (after debounce).
2. Debounce period: rapid consecutive changes within 100ms are coalesced into a
   single evaluation.
3. HTTP data sources are polled every `[data].http_poll_interval` seconds (default: 60).
   A poll that returns a changed response triggers re-evaluation.
4. After each re-evaluation, the WebSocket update is pushed per BC-4.03.004.
5. `slideforge watch` continues running until Ctrl+C or SIGTERM.

## Invariants

1. Watch mode ALWAYS uses warn-only semantics — validation errors produce error-slide
   placeholders, not build termination. (DI-017 — strict mode produces no output; watch
   mode is explicitly warn-only)
2. Newly-added @include files are detected dynamically — the watcher updates its watch
   set after each re-evaluation.
3. The watcher does not poll .sf files (OS events only); HTTP sources are polled on a
   timer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | .sf file saved without change (touch) | File change event detected; re-evaluation triggered; preview refreshed (no change in output) |
| EC-002 | HTTP data source unavailable during poll | E-DAT-001/E-DAT-002 shows in error overlay; prior successful deck state remains visible (DEC-007) |
| EC-003 | New @include added to .sf file on save | On next re-evaluation, the new @include target is added to the watch set |
| EC-004 | 100 rapid saves in 1 second | Debounced to ≤ 10 evaluations (at most one per 100ms window) |
| EC-005 | .sf file deleted while watching | E-PAR-005 shown in error overlay; watch continues; prior state visible |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `touch deck.sf` (no content change) | Re-evaluation triggered; WebSocket update sent | happy-path |
| Edit deck.sf to add a slide | WebSocket update shows new slide within 500ms | happy-path |
| 10 rapid saves within 200ms | Exactly 1 re-evaluation triggered (debounce) | edge-case |
| HTTP source returns error during poll | Error overlay in preview; no crash; watch continues | edge-case (DEC-007) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Re-evaluation triggered within 200ms of OS file event | integration test: modify file, measure time-to-WebSocket-message |
| VP-TBD | Debounce: 10 rapid saves → 1 evaluation | integration test: write file 10× in 50ms; assert 1 WebSocket message |
| VP-TBD | No strict-mode exit during watch on validation error | integration test: introduce validation error; assert process still running |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 |
| Capability Anchor Justification | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 — "poll data sources and .sf files for changes, re-evaluate on change, and push incremental updates to the web preview via websocket" is verbatim from CAP-027 |
| L2 Domain Invariants | DI-017 (strict mode no-output; watch is always warn-only) |
| Architecture Module | slideforge-cli watch subcommand + file watcher (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.03.004 — depends on (WebSocket delivery of updates after re-evaluation)
- BC-5.05.002 — composes with (HTTP data failure in watch mode is handled by this and BC-5.05.002)
- BC-5.05.004 — composes with (file watcher event loss is a failure mode of this BC's watcher)
- BC-3.03.003 — depends on (warn-only mode error-slide rendering used during watch)

## Architecture Anchors

- `architecture/plugin-architecture.md#watch-mode` — file watcher + debounce + re-evaluation pipeline

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
