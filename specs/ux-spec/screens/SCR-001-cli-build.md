---
document_type: ux-spec-screen
screen_id: "SCR-001"
screen_name: "CLI: build"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §3.1 CLI Command Surface"
  - "PRD §3.2 Exit Code Semantics"
  - "PRD §3.3 JSON Output Schema"
  - "interface-definitions.md §1.2"
  - "BC-1.15.001"
  - "BC-1.15.002"
  - "BC-1.15.003"
---

# Screen: CLI build (SCR-001)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.

## Purpose and User Context

The `slideforge build` command is the primary user interaction with slideforge.
The user runs it in a terminal to compile one or more `.sf` source files into
output documents (PPTX, DOCX, PDF, HTML). This screen covers: the command
invocation surface, progress output during build, success output, and the
transition to error display (SCR-006) on failure.

Typical invocations:
- `slideforge build deck.sf` — build single file, default format (pptx)
- `slideforge build deck.sf --format pptx,pdf` — multiple formats
- `slideforge build deck.sf --variant exec` — build named variant
- `slideforge build --workspace` — build all workspace members
- `slideforge build deck.sf --json` — machine output for CI

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Command invocation | `slideforge build <SOURCE> [OPTIONS]` | Full syntax from interface-definitions.md §1.2 |
| ELM-002 | Stage progress line | `  Parsing   deck.sf` | Shown in --verbose mode only |
| ELM-003 | Stage progress line | `  Evaluating deck.sf` | Shown in --verbose mode only |
| ELM-004 | Stage progress line | `  Layout    deck.sf` | Shown in --verbose mode only |
| ELM-005 | Stage progress line | `  Exporting  deck.sf → dist/deck.pptx` | Shown in --verbose mode only |
| ELM-006 | Success summary | `Wrote 2 files to dist/ in 342ms` | Default mode (non-verbose) |
| ELM-007 | File output list | `  dist/deck.pptx (102 KB)` | One line per output file; default mode |
| ELM-008 | Warning summary | `1 warning` | Shown when warnings exist, exit 0 |
| ELM-009 | Warning detail | `warning[E-BRD-003]: Brand color slot 'acc3' inferred...` | Collapsible with --verbose |
| ELM-010 | JSON output | Full JSON object to stdout | When --json flag is used |
| ELM-011 | Timing breakdown | `parse: 45ms  eval: 12ms  layout: 89ms  export: 196ms` | --verbose mode only |

---

## Output Format Specification

### Default Mode (no flags, TTY present)

Build succeeds, no warnings:
```
Wrote 2 files to dist/ in 342ms
  dist/deck.pptx  (102 KB)
  dist/deck.docx   (89 KB)
```

Build succeeds with warnings (exit 0):
```
Wrote 1 file to dist/ in 285ms
  dist/deck.pptx  (102 KB)

1 warning:
  warning[E-BRD-003] brand.toml:0: Brand color slot 'acc3' inferred as #F59E0B
  (derived from acc1 #3B82F6). Review in brand.toml to confirm.
  = hint: run `slideforge extract-brand` to audit your brand configuration
```

Color application (token references from UX-INDEX.md terminal palette):
- Checkmark/success prefix: `color.success`
- File paths: `color.file.path`
- Warning prefix and code: `color.warning`
- Hint line: `color.hint`

### Verbose Mode (`--verbose`)

```
  Parsing    deck.sf                                    45ms
  Evaluating deck.sf                                    12ms
    @data kpis from https://api.acme.com/v1/kpis       (8ms, 2.3 KB)
  Layout     deck.sf                                    89ms
    25 slides, 3 section headers
  Exporting  deck.sf → dist/deck.pptx                  186ms
  Exporting  deck.sf → dist/deck.docx                  156ms

Wrote 2 files to dist/ in 488ms
  dist/deck.pptx  (102 KB)
  dist/deck.docx   (89 KB)
```

Color application:
- Stage labels (Parsing, Evaluating, etc.): `color.stage.label`
- Timing numbers: `color.timing`
- Data source URLs: `color.file.path`

### Quiet Mode (`--quiet`)

On success: no output (exit 0).
On error: errors printed to stderr only; no success lines.

### JSON Mode (`--json`)

All output goes to stdout as a single JSON object. No prose output.
Schema defined in interface-definitions.md §3 and PRD §3.3.

```json
{
  "$schema": "https://slideforge.dev/schema/build-result/v1.json",
  "status": "success",
  "version": "1.0.0",
  "source": "/absolute/path/to/deck.sf",
  "variant": null,
  "diagnostics": [],
  "outputs": [
    { "format": "pptx", "path": "/abs/dist/deck.pptx", "size_bytes": 104448 }
  ],
  "timing": {
    "total_ms": 342, "parse_ms": 45, "evaluate_ms": 12,
    "layout_ms": 89, "export_ms": 196
  }
}
```

### Non-TTY / CI (no --json flag, piped output)

When stdout is not a TTY and --json is not specified, output is identical to
default mode but without ANSI codes. Plain text, machine-readable line structure.

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | `slideforge build deck.sf` | Parse → Eval → Layout → Export; print success summary; exit 0 | On any error: transition to SCR-006 error display; exit 1-5 |
| INT-002 | `slideforge build deck.sf --warn-only` | Build proceeds even if validation errors exist; error-slide placeholders inserted; exit 0 if only warnings | Parse errors still fatal (exit 1) |
| INT-003 | `slideforge build deck.sf --json` | Emit JSON to stdout; exit 0 | Emit JSON with status "error" to stdout; exit 1-5 |
| INT-004 | `slideforge build --workspace` | Build all members; report per-member results; exit = highest error code | Failed members reported; successful members still produce output |
| INT-005 | `slideforge build deck.sf --variant exec` | Build with named variant; success output shows variant name | E-CFG-001 if variant not defined (exit 4) |
| INT-006 | SIGINT during build | Print "Build interrupted." to stderr; clean up partial output; exit 130 | N/A |

---

## Validation Rules (Flag Combinations)

| Combination | Behavior |
|-------------|---------|
| `<SOURCE>` + `--workspace` | E-CFG-002 (exit 64): mutually exclusive |
| `--warn-only` + `--strict-overflow` | Valid combination: overflow remains fatal; all other validation errors are warnings |
| `--quiet` + `--verbose` | E-CFG-004 (exit 64): contradictory |
| `--quiet` + `--json` | `--json` wins; no prose output |
| `--template` + `brand.template` in config | CLI flag wins |
| `--output` + `build.output_dir` in config | CLI flag wins |

---

## Accessibility

- **Terminal output**: All information carried in text and exit code; color supplements only
- **Error codes**: E-CAT-NNN format enables grep and scripting without color
- **Help text** (`slideforge build --help`): Plain text, no tables; screen-reader safe
- **JSON mode**: Structured machine output for CI tools and screen-reader-using developers
- **SIGINT handling**: Clean shutdown message ensures no silent hang

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 60 chars | File paths truncated at 40 chars with `...` suffix |
| 60-120 chars | Default layout |
| > 120 chars | Timing columns right-aligned in verbose mode |
| No TTY | Plain text, no box-drawing, no ANSI |
