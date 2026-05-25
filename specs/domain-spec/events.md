---
document_type: domain-spec-section
level: L2
section: "events"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/domain-research.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q16-q25-decisions.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Domain Events

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.

## Processing Stages (Build-Time Events)

The slideforge build pipeline is event-driven in the sense that each stage completion
triggers the next. Domain events here describe state transitions in the four-stage pipeline
(Parse → Evaluate → Layout → Export) and in watch mode.

---

## Stage Events

### EVT-001: SourceFileLoaded

**Trigger:** User invokes `slideforge build <file>` or watch mode detects file change.
**Precondition:** Source file exists and is readable.
**Outcome:** Raw source bytes available for parsing.
**Consumers:** Parser (slideforge-syntax).
**Error path:** File not found → diagnostic `SourceNotFound` with path; build terminates.

---

### EVT-002: ParseComplete

**Trigger:** Parser finishes consuming the primary .sf file and all `@include` dependencies.
**Precondition:** All `@include`/`@import` paths are resolvable; no include cycles.
**Outcome:** A typed AST (or a set of accumulated parse errors if partial).
**Consumers:** Evaluator (slideforge-eval).
**Error path:** Parse errors → accumulated diagnostics; evaluation does not proceed in strict mode.

Note: Parse errors do not prevent error accumulation — the parser continues after each
error using chumsky's error recovery to find additional issues in the same pass.

---

### EVT-003: DataFetchComplete

**Trigger:** All `@data` directives resolved during evaluation.
**Precondition:** Data sources reachable (local paths exist; HTTP endpoints respond).
**Outcome:** All named data bindings available in evaluation scope.
**Consumers:** Evaluator (variable resolution, @for materialization, @if evaluation).
**Error path:** Unreachable source → `DataSourceError` with source name and URI; build fails.

---

### EVT-004: EvaluationComplete

**Trigger:** Evaluator finishes resolving all variables, materializing all `@for` iterations,
evaluating all `@if` conditionals, and expanding all aliases.
**Precondition:** ParseComplete; DataFetchComplete; no undefined variable references.
**Outcome:** `Deck` IR — the semantic, pre-layout intermediate representation. All dynamic
content is resolved to static values. No more `{{ }}` expressions in the IR.
**Consumers:** Validator plugins; Layout engine.
**Error path:** Undefined variable → `UndefinedVariable` with scope path and span; validation errors → accumulate.

---

### EVT-005: ValidationComplete

**Trigger:** All registered `Validator` plugins finish processing the `Deck` IR.
**Precondition:** EvaluationComplete.
**Outcome:** `Vec<Diagnostic>` — a set of validation diagnostics (errors and warnings).
**Consumers:** CLI reporter; watch-mode error overlay; Layout engine (only proceeds if no errors in strict mode).
**Error path:** Validation errors in strict mode → no output produced; all diagnostics reported.

---

### EVT-006: LayoutComplete

**Trigger:** Layout engine finishes computing geometric positioning for all slides.
**Precondition:** ValidationComplete (no blocking errors); Brand fully loaded.
**Outcome:** `LaidOutDeck` IR — all slides have positioned frames, text flow applied,
overflow warnings generated.
**Consumers:** All Exporter plugins (may run in parallel).
**Error path:** Overflow detected → `CanvasOverflow` warning (not blocking in warn-only; blocking in strict).

---

### EVT-007: ExportComplete

**Trigger:** A specific Exporter plugin finishes producing its output format.
**Precondition:** LayoutComplete; Brand assets available.
**Outcome:** Output bytes written to the configured output path (one per format).
**Consumers:** CLI (reports success/timing); web preview server (pushes via websocket).
**Error path:** Write failure → `ExportWriteError`; other exports may still proceed.

---

## Watch Mode Events

### EVT-008: SourceChanged

**Trigger:** File watcher (notify crate) detects a change in a .sf file or an `@include`
dependency.
**Consumers:** Pipeline re-run from ParseComplete (or earlier if data sources also changed).
**Note:** In v1.0, watch mode triggers a full re-build. Incremental re-evaluation via comemo
is planned for v1.x.

---

### EVT-009: DataSourceChanged

**Trigger:** File watcher detects a change in a local data file referenced by `@data`, OR
the polling interval elapses for an HTTP data source.
**Consumers:** Pipeline re-run from DataFetchComplete (re-fetch and re-evaluate).

---

### EVT-010: ValidationFailed (Watch Mode)

**Trigger:** ValidationComplete produces errors during a watch-mode build cycle.
**Outcome:** Error-slide placeholders rendered in the web preview (red slides showing error
text); other valid slides continue to render.
**Consumers:** Web preview server; CLI diagnostic output.
**Note:** Watch mode runs in warn-only severity — validation errors produce error placeholders
but do NOT abort the preview. This is the "error-slide placeholder" feature (Q17, Q23).

---

### EVT-011: PreviewPushComplete

**Trigger:** After each successful (or warn-only) export in watch mode, the web preview
server pushes the updated SVG canvas to all connected websocket clients.
**Consumers:** Browser clients showing the live preview.

---

## Brand Events

### EVT-012: BrandLoaded

**Trigger:** BrandProvider plugin loads brand configuration from .pptx template,
.docx template, or brand.toml file.
**Precondition:** Brand file path resolved; file readable.
**Outcome:** `Brand` struct available for Layout and Export.
**Error path:** Brand file not found → `BrandNotFound`; invalid template → `BrandParseError`.
**Note:** Brand is loaded once per build, before layout begins. Re-loaded on brand file change
in watch mode (triggers re-layout of all slides).

---

## Event Causality Chain (Normal Build)

```
SourceFileLoaded
  → ParseComplete
    → DataFetchComplete (parallel for all @data sources)
      → EvaluationComplete
        → ValidationComplete
          ← BrandLoaded (parallel with evaluation)
          → LayoutComplete
            → ExportComplete (one per format, run in parallel)
```
