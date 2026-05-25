---
document_type: domain-spec-section
level: L2
section: "failure-modes"
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs:
  - .factory/planning/domain-research.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q16-q25-decisions.md
  - .factory/planning/dsl-competitor-analysis.md
input-hash: "[pending]"
traces_to: L2-INDEX.md
---

# Failure Modes

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> FM-NNN describes runtime or build-time failure modes grouped by subsystem.
> Each entry includes: trigger, observable behavior, recovery strategy.

---

## Authoring Subsystem Failures

### FM-001: Parse Error — Unrecoverable

**Trigger:** Indentation inconsistency, unclosed block, or lexer token that cannot be
classified.
**Observable:** Build halts with accumulated parse errors. No AST produced.
**Recovery:** User fixes all reported parse errors. Each error has file:line:col + hint.
Watch mode: continues showing last valid state; new error-slide placeholder on the affected
slide.

---

### FM-002: Include Resolution Failure

**Trigger:** `@include "path.sf"` where the path does not exist after variable resolution.
**Observable:** Compile error naming the unresolved path and the source line that referenced it.
**Recovery:** User corrects the path or creates the missing file.

---

### FM-003: Import Package Not Found

**Trigger:** `@import "pkg/item"` where `pkg` is not in sf.lock.
**Observable:** Compile error with package name and suggested install command.
**Recovery:** `slideforge package install <repo>` then rebuild.

---

### FM-004: Undefined Variable Reference

**Trigger:** `{{ var }}` or `@{var}` references a name not in any scope level.
**Observable:** Compile error with scope path (showing active vars blocks at each level)
and the reference's source span.
**Recovery:** Declare the variable in an appropriate scope, or fix the field name typo.

---

### FM-005: Data Source Fetch Failure (Build Time)

**Trigger:** `@data x from "https://..."` where the URL is unreachable, returns non-200,
or the response cannot be parsed as the declared format.
**Observable:** Compile error with source URL, HTTP status code (if any), and format
parse error details.
**Recovery:** Fix URL or data format. Use `--offline` flag if network is unavailable.

---

## Layout Subsystem Failures

### FM-006: Canvas Overflow

**Trigger:** Content (bullets, table rows, text) exceeds the slide canvas after layout.
**Observable:** `CanvasOverflow` validation warning with the slide, element, and estimated
overflow amount in EMU. Not a blocking error by default (user may intentionally have
dense content with small font).
**Recovery in strict mode:** Reduce content, decrease font size, split across multiple slides.
**Recovery in watch mode:** Warning overlay on the slide; build continues.

---

### FM-007: Brand File Not Found or Unreadable

**Trigger:** `brand "path/to/brand.toml"` or `--template brand.pptx` where the file
does not exist or is not a valid brand file.
**Observable:** Compile error with the resolved path and attempted parse error.
**Recovery:** Fix the brand file path or ensure the brand file exists.

---

### FM-008: Brand Synthesis Failure — Incomplete Palette

**Trigger:** brand.toml missing required color tokens; brand synthesizer cannot infer
all 12 OOXML scheme slots.
**Observable:** Validation warning listing inferred color slots with their derived values.
Build continues with synthesized palette.
**Recovery:** Add explicit color tokens to brand.toml for the flagged slots.

---

### FM-009: Font Not Available on Build Host

**Trigger:** Brand declares `fonts.heading = "Aptos Display"` but the font is not
installed on the build machine (e.g., in CI on Linux).
**Observable:** Warning: "Font 'Aptos Display' not available; using Liberation Sans
(panose: [...]). Text metrics may differ." Layout proceeds with fallback font.
**Recovery:** Install the required fonts on the build host, or declare a fallback in
brand.toml.

---

## Export Subsystem Failures

### FM-010: PPTX Serialization Error

**Trigger:** ooxmlsdk produces invalid XML (schema validation error, duplicate element,
invalid relationship reference).
**Observable:** Export error with the OOXML element path and schema violation detail.
**Recovery:** This is a slideforge bug. User should report with the .sf source. Workaround:
disable the specific slide feature (shapes, math) to isolate the offending element.

---

### FM-011: PDF Export — Chrome Headless Unavailable

**Trigger:** Chrome/Chromium binary not available on the build host when PDF export
is requested.
**Observable:** Export error: "PDF export requires Chrome. Set SLIDEFORGE_CHROME_PATH
or install Chrome/Chromium."
**Recovery:** Install Chrome, or set `SLIDEFORGE_CHROME_PATH` environment variable.

---

### FM-012: Math Rendering Failure

**Trigger:** LaTeX expression that pulldown-latex cannot parse or that produces invalid
MathML.
**Observable:** Compile error with the LaTeX source span, parse error message, and a hint
toward the unsupported command.
**Recovery:** Fix the LaTeX expression or use a supported subset (document of supported
commands published in DSL reference).

---

### FM-013: Chart Rendering Failure

**Trigger:** plotters panics or returns an error for a chart specification (e.g., invalid
axis range, incompatible data types).
**Observable:** Export error with chart slide title and plotters error detail.
**Recovery:** Fix the chart data or chart specification (type mismatch, empty data, invalid
range).

---

### FM-014: Diagram Rendering Failure

**Trigger:** DiagramRenderer plugin (mermaid) produces an error for the provided source.
**Observable:** Compile error with the diagram slide title and mermaid error detail (line
within source block).
**Recovery:** Fix the Mermaid source syntax.

---

### FM-015: Output File Write Failure

**Trigger:** The output path is unwritable (permissions, disk full, path does not exist).
**Observable:** Export error with the target path and OS error.
**Recovery:** Fix output directory permissions or path; ensure disk space.

---

## Watch Mode Failures

### FM-016: File Watcher Stops Delivering Events

**Trigger:** OS file watching limit exceeded (e.g., `inotify` limit on Linux) or the
watcher process crashes.
**Observable:** CLI warning: "File watcher may have missed events. Press any key to
force rebuild."
**Recovery:** Increase OS inotify limit (`sysctl fs.inotify.max_user_watches`); restart
watch mode.

---

### FM-017: Web Preview WebSocket Connection Drop

**Trigger:** Network issue or browser tab closed while watch mode is running.
**Observable:** Browser shows stale preview; reconnects automatically when the tab regains
focus (preview server reconnects on page reload or WebSocket reconnect event).
**Recovery:** Automatic — no user action needed. Preview server continues running; client
reconnects on next page load.

---

## Plugin / Registry Failures

### FM-018: Plugin Trait API Violation Detected at Runtime

**Trigger:** A bundled plugin calls into another plugin's internal API or bypasses the
registered trait interface (violation of DI-008).
**Observable:** This is a slideforge bug, not a user error. Detected via test failure or
panic in the plugin boundary.
**Recovery:** Fix the plugin to use only the declared trait API. If the API is insufficient,
fix the API first. The "dog-fooding guarantee" means this failure also indicates an API
correctness gap that must be fixed before any external plugin could work.
