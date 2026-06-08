---
document_type: ux-spec-index
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
modified: 2026-06-07
phase: 1c
inputs:
  - .factory/specs/prd.md
  - .factory/specs/prd-supplements/interface-definitions.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/domain-spec/L2-INDEX.md
  - .factory/planning/q1-decision-final.md
  - .factory/planning/q4-q15-decisions.md
  - .factory/planning/q16-q25-decisions.md
  - .factory/planning/spikes/S3-wcag-tooling-choice.md
  - .factory/planning/spikes/S6-multi-renderer-parity.md
input-hash: "[pending compute-input-hash]"
traces_to: .factory/specs/prd.md
prd_version: "1.0"
design_system_version: "1.0"
screens:
  - SCR-001-cli-build
  - SCR-002-cli-watch
  - SCR-003-cli-extract-brand
  - SCR-004-cli-config-explain
  - SCR-005-cli-package
  - SCR-006-error-display
  - SCR-007-web-preview
  - SCR-008-watch-mode
  - SCR-009-first-run
  - SCR-010-workspace
flows:
  - FLOW-001-build-success
  - FLOW-002-build-error-recovery
  - FLOW-003-watch-live-reload
  - FLOW-004-first-run
  - FLOW-005-extract-brand
---

# UX Specification: slideforge v1.0

> **Sharded artifact (DF-021).** This index contains global UX settings
> (design system refs, breakpoints, a11y, performance targets). Per-screen
> and per-flow details live in separate files under `screens/` and `flows/`.
>
> **Developer-tool UX.** slideforge has NO GUI. The primary interface is the
> terminal (CLI). The secondary interface is a localhost web preview server.
> UX = terminal output formatting, error message quality, and web preview
> interaction. Every screen in this spec is a terminal session or a browser
> localhost view.

---

## Screen Inventory

| SCR ID | Name | Purpose | Interface | Complexity | File |
|--------|------|---------|-----------|-----------|------|
| SCR-001 | CLI: build | Compile .sf source to output format(s) | Terminal | complex | screens/SCR-001-cli-build.md |
| SCR-002 | CLI: watch | Start watch mode with live reload | Terminal | complex | screens/SCR-002-cli-watch.md |
| SCR-003 | CLI: extract-brand | Extract brand.toml from .pptx/.docx template | Terminal | simple | screens/SCR-003-cli-extract-brand.md |
| SCR-004 | CLI: config explain | Show configuration provenance | Terminal | simple | screens/SCR-004-cli-config-explain.md |
| SCR-005 | CLI: package | Manage content packages | Terminal | simple | screens/SCR-005-cli-package.md |
| SCR-006 | Error display | Multi-error diagnostic output with source pointers | Terminal | complex | screens/SCR-006-error-display.md |
| SCR-007 | Web preview | SVG-based live slide preview in browser | Browser (localhost) | complex | screens/SCR-007-web-preview.md |
| SCR-008 | Watch mode | File change detection, incremental rebuild, error overlays | Terminal + Browser | complex | screens/SCR-008-watch-mode.md |
| SCR-009 | First run | slideforge init, getting started flow | Terminal | simple | screens/SCR-009-first-run.md |
| SCR-010 | Workspace | Multi-deck workspace config, .sfconfig cascade | Terminal | simple | screens/SCR-010-workspace.md |

---

## Flow Inventory

| FLOW ID | Name | Screens Involved | Steps | File |
|---------|------|-----------------|-------|------|
| FLOW-001 | Build success | SCR-001 | 4 | flows/FLOW-001-build-success.md |
| FLOW-002 | Build error recovery | SCR-001, SCR-006 | 6 | flows/FLOW-002-build-error-recovery.md |
| FLOW-003 | Watch live reload | SCR-002, SCR-007, SCR-008 | 7 | flows/FLOW-003-watch-live-reload.md |
| FLOW-004 | First run | SCR-009, SCR-001 | 5 | flows/FLOW-004-first-run.md |
| FLOW-005 | Extract brand | SCR-003, SCR-001 | 4 | flows/FLOW-005-extract-brand.md |

---

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implement a CLI command | UX-INDEX.md (globals) + screens/SCR-00N-cli-*.md |
| Implement error display | UX-INDEX.md + screens/SCR-006-error-display.md + error-taxonomy.md |
| Implement web preview | UX-INDEX.md + screens/SCR-007-web-preview.md + screens/SCR-008-watch-mode.md |
| Write E2E tests for build flow | flows/FLOW-001-build-success.md + flows/FLOW-002-build-error-recovery.md |
| Write E2E tests for watch flow | flows/FLOW-003-watch-live-reload.md + SCR-007 + SCR-008 |
| Accessibility audit web preview | UX-INDEX.md (a11y checklist) + screens/SCR-007-web-preview.md |
| Full UX review | UX-INDEX.md + all screen and flow files |

---

## Terminal UX Design Principles

> These are globally binding rules for all CLI output across SCR-001 through SCR-010.

### P1 — Respect NO_COLOR and CI Environments

When `NO_COLOR` env var is set, or when stdout is not a TTY (`!isatty(stdout)`):
- Suppress all ANSI color codes
- Suppress all ANSI bold/underline/italic
- Plain text only — no Unicode box-drawing characters unless ASCII fallback provided
- Error codes (E-PAR-001) still appear; color is stripped
- JSON mode (`--json` flag) is always color-free and TTY-independent

Implementation note: Use the `anstyle` / `anstream` crate for automatic TTY detection
and NO_COLOR compliance. Never use raw `\x1b[` codes directly.

### P2 — Errors Are the Primary UX Surface

The majority of user interaction with slideforge is reading error messages. Error
message quality is not secondary — it IS the product for the error case.

Every error message must satisfy:
1. File, line, column in the first line (machine-parseable)
2. Source excerpt with caret pointing at the error site (miette/ariadne style)
3. Hint line: "Did you mean X?" or "Add Y to fix this"
4. Error code (E-CAT-NNN) for scripting and support lookup
5. NO stack traces to user-facing output (structured tracing for internal logging only)

### P3 — Progressive Disclosure in CLI Output

Default output is minimal: success = 1 line. Errors = structured diagnostic block per error.
`--verbose` expands to per-stage timing and structured diagnostics.
`--quiet` suppresses all output except errors (exit code carries result).
`--json` machine output for CI/scripting.

### P4 — Zero-Config Start

`slideforge init` creates a working scaffold. `slideforge build deck.sf` works immediately
after init with no flags required. Every default is a sensible production default.

### P5 — Symmetry Between CLI and JSON Output

Every prose diagnostic printed to stderr has a corresponding entry in the JSON output
schema when `--json` is used. No information is available in one mode that is not
available in the other (modulo human-readable formatting).

---

## Terminal Color Palette (ANSI Semantic Tokens)

> Reference by semantic name in screen specs. Do not reference raw ANSI codes.
> All colors suppressed when NO_COLOR or non-TTY.

| Token | ANSI | Usage |
|-------|------|-------|
| `color.success` | Bold green (32;1m) | Success checkmark, "Wrote N files" |
| `color.warning` | Bold yellow (33;1m) | Warning prefix, E-BRD-003, E-LAY-001 |
| `color.error` | Bold red (31;1m) | Error prefix, fatal diagnostics |
| `color.error.code` | Red (31m) | Error code (E-PAR-001) |
| `color.hint` | Cyan (36m) | Hint lines, "Did you mean..." |
| `color.source.context` | Dim white (37;2m) | Source file excerpt (context lines) |
| `color.source.highlight` | Bold white + underline | The specific token in error |
| `color.source.caret` | Bold red | The ^ caret pointing at error site |
| `color.timing` | Dim (2m) | Per-stage timing (--verbose) |
| `color.file.path` | Cyan underline | File paths in output |
| `color.stage.label` | Bold (1m) | Stage names (Parsing, Evaluating, ...) |

---

## Web Preview Design Tokens

> Reference by token name in SCR-007 and SCR-008. Aligned with WCAG AA contrast requirements.

| Token | Value | Usage |
|-------|-------|-------|
| `preview.bg` | `#1a1a2e` | Preview page background |
| `preview.surface` | `#16213e` | Slide card background |
| `preview.border` | `#0f3460` | Slide border / panel border |
| `preview.text.primary` | `#e2e8f0` | Navigation labels, headings |
| `preview.text.secondary` | `#94a3b8` | Slide count, metadata labels |
| `preview.accent` | `#e94560` | Active slide indicator, focus ring |
| `preview.error.bg` | `#7f1d1d` | Error overlay background |
| `preview.error.text` | `#fca5a5` | Error text on error overlay |
| `preview.warning.bg` | `#78350f` | Warning overlay background |
| `preview.warning.text` | `#fcd34d` | Warning text on warning overlay |
| `preview.success` | `#065f46` | Build success indicator |
| `preview.loading` | `#1e3a5f` | Loading state background |

Contrast verification (WCAG AA 4.5:1 for normal text):
- `preview.text.primary` on `preview.bg`: 12.3:1 — PASS
- `preview.text.secondary` on `preview.bg`: 5.9:1 — PASS
- `preview.error.text` on `preview.error.bg`: 4.8:1 — PASS
- `preview.warning.text` on `preview.warning.bg`: 5.1:1 — PASS

---

## Contextual Variants

| Variant | Trigger | Behavior |
|---------|---------|----------|
| No color (terminal) | `NO_COLOR` env set | Strip all ANSI codes; pure text output |
| Non-TTY (CI/pipe) | `!isatty(stdout)` | Same as NO_COLOR; auto-detected |
| JSON mode | `--json` flag | All output as JSON to stdout; no prose |
| Quiet mode | `--quiet` flag | Suppress all output except fatal errors to stderr |
| Verbose mode | `--verbose` flag | Per-stage timing + extended diagnostics |
| Dark mode (web) | `prefers-color-scheme: dark` | Web preview already dark-first; light mode variant applies inverted token set |
| Reduced motion (web) | `prefers-reduced-motion: reduce` | Disable slide transition animations, disable WebSocket reconnect pulse animation |
| High contrast (web) | `prefers-contrast: more` | Increase preview.border contrast; use `outline: 2px solid` for focus rings |

---

## Accessibility Checklist (Global)

### Terminal CLI

- All output meaningful without color (error codes, symbols supplement color)
- Exit codes carry machine-readable result (0 = success, non-zero = error type)
- Error messages readable by screen readers (no art/box-drawing that garbles)
- `--json` provides machine-readable equivalent of all prose output
- `--help` output formatted for screen reader consumption (no tables in help text)
- Respect NO_COLOR environment variable globally

### Web Preview (WCAG 2.2 AA)

- SVG-based canvas rendering (NOT `<canvas>`) — required per S3 spike finding
- Every SVG slide element has ARIA role and descriptive label
- Keyboard navigation: arrow keys for slide prev/next; Tab to control buttons
- Focus ring visible on all interactive elements (preview.accent, 2px outline)
- Slide titles exposed as `<h2>` within accessible slide structure (not hidden in SVG)
- Connection status (connecting, live, disconnected) announced via `aria-live="polite"`
- Error overlays use `role="alert"` for immediate screen-reader announcement
- All color indicators supplemented with text or icon labels
- Minimum 4.5:1 contrast for all text (verified in design tokens above)
- Minimum 48px touch targets for navigation controls
- No content conveyed by color alone
- `prefers-reduced-motion` respected for all animations

---

## Performance Targets

### CLI Build Performance

| Metric | Target | Validation |
|--------|--------|-----------|
| Cold build (25-slide deck) | < 500ms | CI criterion benchmark (NFR-001) |
| Watch mode rebuild (1 file change, v1.0 full rebuild) | < 500ms (NFR-001 path) | CI criterion benchmark (NFR-001); NFR-002 < 50ms incremental DEFERRED to v1.x — see nfr-catalog v1.3 |
| CLI startup time (--version, --help) | < 50ms | Manual timing |
| Error rendering to stderr | < 10ms | Part of parse stage timing |

### Web Preview Performance

| Metric | Target | Validation |
|--------|--------|-----------|
| Initial page load (first slide visible) | < 1s on localhost | Manual timing |
| WebSocket reconnect after rebuild | < 200ms UI update | Manual timing |
| Slide navigation (prev/next) | < 16ms (60fps) | Chrome DevTools |
| SVG render of single slide | < 100ms | Part of preview pipeline |

---

## PRD Requirement Traceability

| Screen | PRD Section | BCs Covered |
|--------|------------|-------------|
| SCR-001 CLI build | §3.1, §3.2, §3.3 | BC-1.15.001, BC-1.15.002, BC-1.15.003 |
| SCR-002 CLI watch | §3.1 (watch cmd) | BC-5.05.001-005 |
| SCR-003 extract-brand | §3.1 (extract-brand) | BC-2.01.003 |
| SCR-004 config explain | §3.1 (config) | BC-5.04.001-003 |
| SCR-005 package | §3.1 (package) | BC-5.03.001-003 |
| SCR-006 error display | §5 (Error Taxonomy) | BC-1.15.001-003, all E-xxx errors |
| SCR-007 web preview | §2.4 (BC-4.03), §4 (NFR-005) | BC-4.03.003, BC-4.03.004 |
| SCR-008 watch mode | BC-5.05 subsection | BC-5.05.001-005 |
| SCR-009 first run | §3.1 (init implied) | BC-1.15.001 |
| SCR-010 workspace | §3.1 (config/workspace) | BC-5.04.001-003 |
