---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-056
title: "CLI: watch mode + notify integration + incremental rebuild"
epic: EPIC-15
wave: 5
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-cli
behavioral_contracts:
  - BC-5.05.001
  - BC-5.05.002
  - BC-5.05.003
  - BC-5.05.004
verification_properties: []
nfr_refs:
  - NFR-002
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
  - NFR-032
depends_on:
  - STORY-055
  - STORY-047
blocks: []
subsystems:
  - SS-18
target_module: slideforge-cli
---

# STORY-056: CLI: watch mode + notify integration + incremental rebuild

## Summary

Implement the `slideforge watch` subcommand in `slideforge-cli`. Watch mode monitors all
`.sf` source files (including `@include` targets) and file-based data sources for
changes, triggers re-evaluation within 200ms of OS file events (after 100ms debounce),
and pushes WebSocket update notifications to the preview server. HTTP data sources are
polled on a configurable interval (default 60s). The watch loop runs until Ctrl+C or
SIGTERM.

Key design constraints:

- Watch mode ALWAYS uses warn-only semantics. Validation errors produce error-slide
  placeholders, not build termination.
- Incremental rebuild: re-evaluation is triggered per-file-change; the evaluator is
  given the changed-file set so it can skip unchanged work (NFR-002: < 50ms incremental).
- HTTP data failures do NOT crash the watch process — they produce error-slide
  placeholders and the prior good deck state is retained.
- File watcher overflow (OS queue full) triggers an automatic full rebuild and a CLI
  warning.
- `r` keypress triggers a manual force-rebuild.

## Behavioral Contracts

| BC | Title | Postconditions Covered |
|----|-------|----------------------|
| BC-5.05.001 | watch polls .sf files and data sources; re-evaluates on change | All 5 postconditions; Invariants 1–3 |
| BC-5.05.002 | HTTP data source failure shows error-slide; does not crash | All 6 postconditions; Invariants 1–3 |
| BC-5.05.003 | Watch mode HTTP schema change surfaces undefined-var errors | All 5 postconditions; Invariants 1–3 |
| BC-5.05.004 | File watcher event loss produces CLI warning; force-rebuild available | All 5 postconditions; Invariants 1–3 |

## Acceptance Criteria

- [ ] **AC-001** — `slideforge watch deck.sf` starts the file watcher and web preview server.
  Modifying `deck.sf` triggers re-evaluation and a WebSocket update within 200ms of the
  OS file event (after debounce).
  (traces to BC-5.05.001 postcondition 1)

- [ ] **AC-002** — 10 rapid saves within 100ms are coalesced into exactly 1 re-evaluation
  (debounce). More precisely: any sequence of events within a 100ms window is treated as 1
  event; the evaluation fires after the window is quiet for 100ms.
  (traces to BC-5.05.001 postcondition 2)

- [ ] **AC-003** — HTTP data sources are polled every `http_poll_interval` seconds (default 60).
  A changed HTTP response triggers re-evaluation.
  (traces to BC-5.05.001 postcondition 3)

- [ ] **AC-004** — When re-evaluation completes, a WebSocket message is pushed to all connected
  preview clients per BC-4.03.004.
  (traces to BC-5.05.001 postcondition 4)

- [ ] **AC-005** — `slideforge watch` runs until Ctrl+C (SIGINT) or SIGTERM; it does not exit
  due to any recoverable error.
  (traces to BC-5.05.001 postcondition 5)

- [ ] **AC-006** — An HTTP data source that returns a 4xx/5xx response during polling causes
  E-DAT-001 to be emitted to terminal. A WebSocket error update is sent. Affected slides
  show error-slide placeholders. Non-affected slides display normally. The process continues.
  (traces to BC-5.05.002 postconditions 1–5)

- [ ] **AC-007** — An HTTP data source that recovers on the next poll triggers re-evaluation and
  the preview shows the correct content (error-slides replaced).
  (traces to BC-5.05.002 postcondition 6)

- [ ] **AC-008** — When an HTTP data source changes its response schema (a field is renamed),
  the next re-evaluation emits E-EVL-001 for each interpolation site that references the
  renamed field, with a hint listing currently-available fields.
  (traces to BC-5.05.003 postcondition 1, 2)

- [ ] **AC-009** — Non-affected slides (not referencing the renamed field) display normally with
  the new response data.
  (traces to BC-5.05.003 postcondition 4)

- [ ] **AC-010** — When the watcher detects an OS overflow event (e.g., `EventKind::Other` from
  notify indicating queue overflow), a warning is printed to stderr and a full rebuild is
  triggered immediately, bypassing debounce.
  (traces to BC-5.05.004 postcondition 1, 2)

- [ ] **AC-011** — The user can press `r` in the terminal to trigger a manual force-rebuild at
  any time. A confirmation message "Rebuilding..." is printed.
  (traces to BC-5.05.004 postcondition 4)

- [ ] **AC-012** — Watch mode ALWAYS uses warn-only semantics. A source with a validation error
  produces error-slide placeholders but does NOT terminate the watch process.
  (traces to BC-5.05.001 invariant 1)

- [ ] **AC-013** — A newly-added `@include` file is detected after the first re-evaluation that
  introduces it. The watcher adds the included file to its watch set.
  (traces to BC-5.05.001 invariant 2)

- [ ] **AC-014** — Incremental rebuild for a single `.sf` file change completes in < 50ms on
  the CI benchmark runner (NFR-002).
  (traces to NFR-002 — incremental rebuild < 50ms)

- [ ] **AC-015** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, `clippy::pedantic`
  clean.
  (traces to NFR-021, NFR-022, NFR-024)

## Tasks

1. Define `WatchArgs` in `src/cli.rs`:
   ```rust
   #[derive(Args)]
   pub struct WatchArgs {
       pub source: PathBuf,
       #[arg(long, default_value_t = 60)]
       pub http_poll_interval: u64,
       #[arg(long, default_value = "dist")]
       pub output_dir: PathBuf,
       #[arg(long, value_delimiter = ',', default_values_t = all_formats())]
       pub format: Vec<OutputFormat>,
       #[arg(long)]
       pub variant: Option<String>,
   }
   ```
2. Implement `run_watch(args: &WatchArgs, global: &GlobalFlags) -> ExitCode` in
   `src/commands/watch.rs`. The watch loop runs on the Tokio runtime:
   ```rust
   async fn watch_loop(ctx: WatchContext) {
       // 1. Start file watcher (notify 8.0)
       // 2. Start HTTP poll timer
       // 3. Start preview WebSocket server (delegates to slideforge-preview)
       // 4. Start keyboard input reader (for 'r' keypress)
       // 5. Main select! loop
       loop {
           tokio::select! {
               event = file_watcher_rx.recv() => handle_file_event(event, &ctx).await,
               _ = http_poll_timer.tick() => handle_http_poll(&ctx).await,
               _ = keyboard_rx.recv() => trigger_force_rebuild(&ctx).await,
               _ = shutdown_signal => break,
           }
       }
   }
   ```
3. Implement debounce via `tokio::time::sleep(Duration::from_millis(100))` with a reset
   on each new event within the window. Use a `DebouncedSender` wrapper over the file
   watcher channel.
4. Implement `handle_file_event()`:
   - On normal change event: update `changed_files` set; trigger debounced re-evaluation
   - On overflow event: print overflow warning to stderr; trigger immediate full rebuild
5. Implement `run_evaluation(ctx, changed_files, force_full: bool)`:
   - Construct `CompileOptions` with `warn_only: true` always
   - Pass `changed_files` to `slideforge::compile_incremental()` (or full compile if
     `force_full`)
   - Collect `DiagnosticSink`; render errors to terminal
   - Push WebSocket update to connected preview clients
   - If re-evaluation reveals new `@include` targets: add them to watcher
6. Implement HTTP poll loop: on each tick, re-fetch all HTTP data sources; compare hash
   of response body to cached hash; if changed, trigger re-evaluation with
   `changed_sources = {http_url}`.
7. Implement `keyboard_reader()` async task that reads stdin for `r` keypress and sends
   on a channel.
8. Implement shutdown handling: catch SIGINT/SIGTERM; drain the re-evaluation queue;
   exit cleanly.
9. Add `WatchState` struct tracking last-good deck, current error count, watch file set.
10. Write tests for all ACs.

## File List

- `crates/slideforge-cli/src/commands/watch.rs` — `run_watch()`, `WatchContext`,
  `WatchState`, `watch_loop()`, `handle_file_event()`, `handle_http_poll()`,
  `trigger_force_rebuild()`
- `crates/slideforge-cli/src/cli.rs` — add `WatchArgs` struct (extends existing file)
- `crates/slideforge-cli/src/debounce.rs` — `DebouncedSender<T>` utility
- `crates/slideforge-cli/src/keyboard.rs` — async keyboard reader
- `crates/slideforge-cli/Cargo.toml` — add: `tokio` (features: full), `notify =8.0`
- `crates/slideforge-cli/tests/watch_integration.rs` — integration tests

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~6 000 |
| BC-5.05.001, BC-5.05.002, BC-5.05.003, BC-5.05.004 | ~6 000 |
| STORY-055 (build command infrastructure, WatchArgs precursor) | ~3 000 |
| STORY-047 (preview server WebSocket API surface) | ~2 000 |
| Target source files to write | ~5 000 |
| Test files | ~3 000 |
| **Total** | **~25 000** |

Context budget: 25 000 / 200 000 ≈ 12.5% — within limit.

## Test Strategy

**Unit tests** (`src/commands/watch.rs #[cfg(test)]`):

- `test_debounce_coalesces_events()`: send 10 events within 50ms; assert debounce fires
  exactly 1 evaluation.
- `test_debounce_two_windows()`: send 2 events separated by 150ms; assert 2 evaluations.
- `test_overflow_triggers_full_rebuild()`: inject synthetic overflow event; assert
  `force_full = true` passed to `run_evaluation`.
- `test_warn_only_always_set()`: check `CompileOptions::warn_only == true` regardless of
  `global.warn_only` value.

**Integration tests** (`tests/watch_integration.rs`):

- `test_file_change_triggers_rebuild()`: start watch in background; write to deck.sf;
  assert re-evaluation triggered within 500ms (generous for CI).
- `test_http_failure_no_crash()`: mock HTTP server returns 500; assert watch process still
  alive; assert error overlay WebSocket message received.
- `test_http_recovery()`: mock HTTP returns 500 then 200 on next poll; assert recovery
  re-evaluation triggers and sends non-error WebSocket message.
- `test_schema_change_evl001()`: mock HTTP response schema change; assert E-EVL-001 in
  terminal output and WebSocket error message.
- `test_force_rebuild_r_keypress()`: send `r` to stdin; assert rebuild triggered.
- `test_new_include_added_to_watcher()`: deck.sf is modified to add `@include sub.sf`;
  assert watcher picks up `sub.sf` for monitoring.

**Benchmark** (`benches/build_bench.rs` — Criterion, also used by STORY-059):

- `bench_incremental_rebuild_single_file()`: modify 1 field in a 25-slide fixture;
  measure re-evaluation time; assert < 50ms per NFR-002.

## Dependencies

- **Depends on:** STORY-055 (provides `Cli`, `GlobalFlags`, `OutputFormat`, `run_build()`
  infrastructure, exit code logic — watch builds on top of it)
- **Depends on:** STORY-047 (preview server WebSocket API — watch pushes updates to the
  preview server's broadcast channel)
- **Blocks:** (nothing in EPIC-15 — this is the last complex story)

## Dependency Anchor Justifications

- SS-18 owns this story's scope because SS-18 is the CLI Orchestrator subsystem owning
  `slideforge-cli` per ARCH-INDEX Subsystem Registry. File watching and pipeline
  re-triggering is lifecycle I/O — the CLI's domain.
- STORY-056 depends on STORY-055 because watch mode extends the CLI struct defined there
  (`Command::Watch(WatchArgs)`) and reuses `run_build()` infrastructure for each
  re-evaluation cycle.
- STORY-056 depends on STORY-047 because watch mode pushes WebSocket update messages to
  the preview server's broadcast channel — that API is defined in the preview server story.

## Architecture Compliance Rules

1. `slideforge-cli` is an **effectful shell** — watch mode performs file I/O, network I/O,
   and terminal I/O. No pure-core logic belongs here.
2. Watch mode uses Tokio's async runtime. The Tokio runtime MUST NOT be initialized more
   than once per process. Initialize in `main()` via `#[tokio::main]`.
3. The `notify` watcher runs in a separate OS thread (non-async); it communicates with
   the Tokio runtime via an `mpsc::channel`. Do NOT block the Tokio executor.
4. HTTP polling MUST NOT use the same `reqwest` client instance concurrently from
   multiple tasks — each poll is sequential (poll, wait, poll). Use a single-threaded
   poll task per HTTP source.
5. `WatchState.last_good_deck` holds the last successfully compiled deck; it is
   NEVER cleared on error, only replaced on success.
6. Force-rebuild queue: if a rebuild is in progress when `r` is pressed, the force-rebuild
   is queued and fires immediately after the current rebuild completes. No concurrent
   evaluations.

**Forbidden dependencies for watch.rs:**
- Must NOT import `chumsky` — parsing is SS-01's domain.
- Must NOT import `ooxmlsdk`, `pdf-writer`, `krilla` — exporting is SS-06/07/08/09.
- Must NOT import `slideforge-eval` directly — all pipeline calls go through
  `slideforge::compile_incremental()` (root crate).

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `notify` | `=8.0` | File system event watching (inotify/FSEvents/ReadDirectoryChanges) |
| `tokio` | `=1.44` | Async runtime; `tokio::select!`, timers, channels |
| `clap` | `=4.5` | `WatchArgs` derive |
| `tracing` | `=0.1` | Span instrumentation for watch events |
| `reqwest` | `=0.12` | HTTP poll client (also used in slideforge-data; CLI uses it for polling) |

Note: `reqwest` is already in the dependency graph for `slideforge-data`. Import it in
`slideforge-cli` with `=0.12` pinning.

## File Structure Requirements

```
crates/slideforge-cli/
  src/
    commands/
      watch.rs              # run_watch(), WatchContext, WatchState, watch_loop()
    debounce.rs             # DebouncedSender<T>
    keyboard.rs             # async keyboard reader task
  tests/
    watch_integration.rs    # integration tests
```

## Previous Story Intelligence

From STORY-055: the `Cli` struct and `GlobalFlags` are already defined. `WatchArgs` must
be added to the `Command::Watch(WatchArgs)` arm without modifying the existing arms.

Watch mode always forces `warn_only: true` in `CompileOptions` regardless of what the
user passed in `GlobalFlags.warn_only`. This is a contract from BC-5.05.001 invariant 1
and must be enforced in `run_watch()` before constructing `CompileOptions`.

Key from BC-5.05.002: the `last_good_deck` in `WatchState` must be cloned and held even
after an error. It is the state shown in the browser until recovery. This means
`Deck` (or `LaidOutDeck`) must implement `Clone` — verify this is enforced in STORY-001
(ir-core-types) before implementing STORY-056.

## Implementation Notes

### Debounce algorithm

```rust
struct DebouncedSender<T> {
    tx: mpsc::UnboundedSender<()>,
    pending: Arc<Mutex<Option<T>>>,
}

impl<T: Send + 'static> DebouncedSender<T> {
    fn send(&self, item: T) {
        *self.pending.lock().unwrap() = Some(item);
        // Reset the debounce timer by sending a "something changed" signal.
        // The receiver loop waits 100ms after the LAST signal before firing.
        let _ = self.tx.send(());
    }
}

// Receiver loop:
async fn debounce_loop(mut rx: mpsc::UnboundedReceiver<()>, fire_tx: mpsc::Sender<()>) {
    while rx.recv().await.is_some() {
        loop {
            match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                Err(_) => break, // timeout: 100ms quiet → fire
                Ok(None) => return, // channel closed
                Ok(Some(_)) => continue, // reset timer
            }
        }
        let _ = fire_tx.send(()).await;
    }
}
```

### Notify watcher event handling

`notify 8.0` provides `EventKind` variants. Map them as follows:
- `EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)` → file changed event
- `EventKind::Other` with data indicating overflow → overflow event (triggers full rebuild)

Note: notify 8.0 raised MSRV to 1.77 and added `RecommendedCache` for automatic file ID
caching on Windows/macOS. The `recommended_watcher(handler)` API still exists as a
convenience function returning the platform `RecommendedWatcher`. On macOS, FSEvents
coalesces events automatically; no special handling needed. On Linux, `IN_Q_OVERFLOW`
arrives as a synthetic event from inotify — the notify crate surfaces this as a
watcher-level error via the `Error` type passed to the event handler. Register an error
check in the `EventHandler` closure that detects overflow conditions.

### HTTP polling task

```rust
async fn http_poll_task(
    sources: Vec<HttpDataSource>,
    interval: Duration,
    rebuild_tx: mpsc::Sender<RebuildTrigger>,
    mut cancel: CancellationToken,
) {
    let mut cache: HashMap<String, u64> = HashMap::new(); // url → hash of last body
    let mut ticker = tokio::time::interval(interval);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                for source in &sources {
                    match fetch_with_timeout(&source.url, Duration::from_secs(30)).await {
                        Ok(body) => {
                            let hash = compute_hash(&body);
                            if cache.get(&source.url) != Some(&hash) {
                                cache.insert(source.url.clone(), hash);
                                let _ = rebuild_tx.send(RebuildTrigger::HttpChange(source.url.clone())).await;
                            }
                        }
                        Err(e) => emit_http_error(e, source),
                    }
                }
            }
            _ = cancel.cancelled() => break,
        }
    }
}
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `.sf` file deleted while watching | E-PAR-005 in terminal (file not found); watch continues; prior state visible |
| EC-002 | `r` pressed while rebuild in progress | Queued: force-rebuild fires immediately after current rebuild completes |
| EC-003 | HTTP source timeout (30s) | Treated as E-DAT-002 (network error); error-slide placeholder; watch continues |
| EC-004 | Multiple HTTP sources; only one fails | Only slides referencing failing source show error-slide; others normal |
| EC-005 | 100 rapid saves in 1 second | Debounced to ≤ 10 evaluations; no more than 1 per 100ms window |
| EC-006 | Ctrl+C during a rebuild | Current rebuild completes; then exit cleanly |
| EC-007 | `@include` target deleted after being watched | E-PAR-005 for the include; error-slide; watch continues; watcher removes path |
