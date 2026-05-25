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

# BC-5.05.002: HTTP Data Source Failure in Watch Mode Shows Error-Slide; Does Not Crash

## Description

When an HTTP data source (`@data name from "https://..."`) fails during watch mode
polling (network error, 4xx/5xx response, timeout), the watch process emits E-DAT-001
or E-DAT-002, renders error-slide placeholders in the web preview for affected slides,
and continues running. The prior successfully-evaluated deck state remains visible in
the browser until the data source recovers. This covers DEC-007.

## Preconditions

1. `slideforge watch` is running.
2. At least one `@data` directive references an HTTP/HTTPS URL.
3. The HTTP data source returns an error response or is unreachable.

## Postconditions

1. E-DAT-001 or E-DAT-002 is emitted in the terminal output.
2. The WebSocket sends an error update: `{type: "error", errors: [E-DAT-xxx], slides: [...
   error-slide placeholders for affected slides ...]}`.
3. Affected slides show error-slide placeholders in the web preview (per BC-3.03.003).
4. Non-affected slides remain visible with their last-good data.
5. The `slideforge watch` process is still running and continues polling.
6. When the HTTP source becomes available again, the next successful poll triggers a
   re-evaluation and the preview recovers automatically.

## Invariants

1. Watch mode NEVER exits due to a transient data source error. The process continues
   until Ctrl+C.
2. The last-good deck state is preserved in memory for display. It is never discarded
   on error (only replaced by a successful re-evaluation).
3. Error-slide placeholders are the ONLY output on affected slides — no partial/corrupt
   slide content is shown.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | HTTP source returns 404 | E-DAT-001 (HTTP 404); error-slide for affected slides; watch continues |
| EC-002 | HTTP source unreachable (DNS failure) | E-DAT-002 (network error); error-slide; watch continues |
| EC-003 | HTTP source recovers on next poll | Next successful poll triggers re-evaluation; error-slide replaced with real content |
| EC-004 | HTTP source timeout (30s) | E-DAT-002 (timeout treated as network error); error-slide; watch continues |
| EC-005 | Multiple HTTP sources; only one fails | Only affected slides show error-slide; slides using other sources display normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HTTP source mock returns 500 | E-DAT-001; error-slide for affected slides; process still running | error (DEC-007) |
| HTTP source recovers after 3 failed polls | Successful poll triggers re-evaluation; preview shows real content | happy-path (recovery) |
| Two HTTP sources; one fails | Slides from working source: normal; slides from failed source: error-slide | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Process still alive after HTTP failure | integration test: inject HTTP error; assert process PID unchanged |
| VP-TBD | Error-slide placeholder shown for affected slides | integration test: parse WebSocket message; check slide type |
| VP-TBD | Recovery: successful poll replaces error-slide with real content | integration test: mock server recovery; assert WebSocket reload message |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 |
| Capability Anchor Justification | CAP-027 ("Watch Mode with Live Data Refresh") per capabilities.md §CAP-027 — "Render error-slide placeholders in warn-only mode during watch" is verbatim from CAP-027; this BC specifies the HTTP failure sub-case |
| L2 Domain Invariants | DI-017 (strict mode no-output; watch mode is warn-only — errors produce placeholders, not termination) |
| Architecture Module | slideforge-cli watch + data source error handling (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-5.05.001 — composes with (watch mode pipeline; this BC covers the HTTP error sub-case)
- BC-3.03.003 — depends on (warn-only error-slide placeholder rendering)
- BC-4.03.004 — depends on (WebSocket delivers the error state to the browser)

## Architecture Anchors

- `architecture/cross-cutting.md#watch-mode` — HTTP data source failure handling

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
