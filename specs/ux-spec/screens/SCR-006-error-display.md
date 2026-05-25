---
document_type: ux-spec-screen
screen_id: "SCR-006"
screen_name: "Error Display"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §5 Error Taxonomy"
  - "error-taxonomy.md (all E-xxx-NNN entries)"
  - "PRD §3.2 Exit Code Semantics"
  - "PRD §3.3 JSON Output Schema"
  - "BC-1.15.001"
  - "BC-1.15.002"
  - "BC-1.15.003"
  - "q16-q25-decisions.md Q23"
---

# Screen: Error Display (SCR-006)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.
> This screen is the most critical UX surface in slideforge. Errors are
> the primary user interaction. Every format below must be implemented exactly.

## Purpose and User Context

Error display is shown whenever `slideforge build` or `slideforge watch` encounters
any diagnostic condition — parse errors, validation errors, lint warnings. The user
sees this in the terminal (stderr for errors, stdout for the success line). In watch
mode, errors also appear as error-slide placeholders in the web preview.

This screen specifies:
1. Single error format (all categories)
2. Multi-error accumulation display
3. Warn-only / error-slide placeholder format
4. JSON error output format
5. Watch mode error overlay in terminal

Key constraint from Q23: ALL errors in the parse and evaluation phases are
accumulated before reporting. The user sees all errors in one pass, not one at a time.

---

## Elements

| ID | Type | Label | Required |
|----|------|-------|---------|
| ELM-001 | Error header | `error[E-CAT-NNN]: message text` | All errors |
| ELM-002 | File/line/col locator | ` --> deck.sf:42:7` | All errors with span |
| ELM-003 | Source context line | `   42 |   slide content:` | All errors with span |
| ELM-004 | Error caret line | `      |         ^^^^^^^ error indicator` | All errors with span |
| ELM-005 | Hint line | `   = hint: Add alt "..." or mark decorative: true` | When hint available |
| ELM-006 | Error count footer | `error: aborting due to 3 previous errors` | Multi-error |
| ELM-007 | Warning header | `warning[E-CAT-NNN]: message text` | Warnings |
| ELM-008 | Error-slide placeholder | Red slide with error text in web preview | Watch + warn-only mode |
| ELM-009 | JSON diagnostic object | See JSON schema | When --json flag active |

---

## Error Format Specification

### Single Fatal Error (canonical format, miette/ariadne style)

```
error[E-PAR-001]: Unexpected indentation at deck.sf:42:7.
                  Expected 4 spaces, found 6.
  --> deck.sf:42:7
   |
41 | slide content:
42 |       title "Hello"
   |       ^ expected 4 spaces here (found 6)
   |
   = hint: slideforge uses 2-space indentation multiples.
           Did you mix spaces and tabs? Run: cat -A deck.sf | grep -n $'\t'
```

Color application:
- `error[E-PAR-001]:` prefix: `color.error` + `color.error.code`
- `--> deck.sf:42:7`: `color.file.path`
- Source context lines `41 |`, `42 |`: `color.source.context`
- The offending token (`      `): `color.source.highlight`
- The `^` caret line: `color.source.caret`
- `= hint:` and hint text: `color.hint`

### Single Warning (non-fatal)

```
warning[E-BRD-003]: Brand color slot 'acc3' inferred as #F59E0B
                    (derived from acc1 #3B82F6). Review in brand.toml to confirm.
  --> brand.toml (slot: acc3)
   |
   = hint: run `slideforge extract-brand template.pptx` to audit all
           color slots from the source template.
```

Color application:
- `warning[E-BRD-003]:` prefix: `color.warning`
- Hint: `color.hint`

### Multi-Error Accumulation (per Q23, BC-1.15.002)

All accumulated errors displayed sequentially, each in the single-error format.
A count footer appears after all errors:

```
error[E-PAR-001]: Unexpected indentation at deck.sf:42:7.
  --> deck.sf:42:7
   |
42 |       title "Hello"
   |       ^ expected 4 spaces here

error[E-A11-001]: Missing alt text on image 'logo.png' at deck.sf:55:3.
                  Add alt "..." or mark decorative: true.
  --> deck.sf:55:3
   |
55 |   image "logo.png"
   |   ^^^^^^^^^^^^^^^^ missing alt text
   |
   = hint: Add `alt "Company logo showing an abstract blue triangle"` on
           the next line, or add `decorative: true` if the image is purely ornamental.

error[E-A11-002]: Missing label on color-coded element 'severity_cards' at deck.sf:71:1.
  --> deck.sf:71:1
   |
71 | slide severity_cards:
   | ^^^^^^^^^^^^^^^^^^^^ color carries meaning here — add label "..."
   |
   = hint: Color alone must not convey severity. Add label "CRITICAL: {{ title }}"
           so screen readers and colorblind users get the same information.

error: aborting due to 3 previous errors
  For more information about each error, use the error code:
  E-PAR-001 https://slideforge.dev/errors/E-PAR-001
  E-A11-001 https://slideforge.dev/errors/E-A11-001
  E-A11-002 https://slideforge.dev/errors/E-A11-002
```

### Warn-Only Mode Error (E-EVL, E-DAT, E-A11 in --warn-only)

In warn-only mode, some errors that would be fatal in strict mode become warnings.
An error-slide placeholder is inserted in the output at the affected position.

Terminal output (identical format to warning):
```
warning[E-EVL-001]: Undefined variable '{{ client_name }}' at deck.sf:38:9.
                    Variables in scope: [quarter, brand_color]
  --> deck.sf:38:9
   |
38 |   title "{{ client_name }} Report"
   |          ^^^^^^^^^^^^^^^ undefined
   |
   = hint: Declare in vars: block at deck level, or check for typo.
           Active scope: quarter, brand_color
   = note: In --warn-only mode, an error-slide placeholder was inserted.
```

### Watch Mode Error Banner (Terminal)

When watch mode encounters a build error after a file change, display:
```
[watch] deck.sf changed — rebuilding...

error[E-PAR-003]: Tab character at deck.sf:12:1. slideforge requires spaces.
  --> deck.sf:12:1
   |
12 |	  title "Hello"
   | ^ tab character (replace with spaces)
   |
   = hint: Use `sed -i 's/\t/  /g' deck.sf` to convert tabs to 2 spaces.

[watch] Build failed. Watching for changes... (Ctrl+C to stop)
```

Color application:
- `[watch]` prefix: `color.timing` (dim, non-intrusive)
- Error block: standard error colors

### Error-Slide Placeholder (Web Preview, Warn-Only Mode)

When `--warn-only` is active and an error occurs at a specific slide, the web
preview renders an error-slide placeholder at that position. Defined in SCR-007
and SCR-008. Terminal output uses the warning format above; the web preview
shows a red-background SVG slide.

Error-slide content in web preview:
- Background: `preview.error.bg` (#7f1d1d)
- Large centered text: "Build Warning" in `preview.error.text`
- Error code: `E-CAT-NNN`
- Short message (40 chars max, truncated): first line of error message
- Sub-text: "Fix in [filename] and save to refresh"

---

## JSON Error Output (--json flag)

Each diagnostic maps to a JSON diagnostic object per interface-definitions.md §3:

```json
{
  "status": "error",
  "diagnostics": [
    {
      "severity": "parse_error",
      "code": "E-PAR-001",
      "category": "PAR",
      "message": "Unexpected indentation at deck.sf:42:7. Expected 4 spaces, found 6.",
      "file": "/abs/path/deck.sf",
      "line": 42,
      "col": 7,
      "length": 6,
      "hint": "slideforge uses 2-space indentation multiples."
    }
  ],
  "outputs": [],
  "timing": {
    "total_ms": 45, "parse_ms": 45, "evaluate_ms": 0,
    "layout_ms": 0, "export_ms": 0
  }
}
```

---

## Error Categories Reference

| Category | Abbreviation | Fatal? | Accumulates? |
|---------|-------------|--------|-------------|
| Parse | PAR | Always | Yes — all parse errors before aborting |
| Evaluation | EVL | Strict: yes; Warn-only: placeholder | Yes |
| Data source | DAT | Strict: yes; Warn-only: placeholder | Yes |
| Layout | LAY | Overflow: warning by default | Yes |
| Export | EXP | Always | No (first export error aborts) |
| Brand | BRD | Missing brand: yes; inferred slot: warning | Yes |
| Package | PKG | Missing/mismatch: yes | Yes |
| Configuration | CFG | Always | No (config checked first) |
| Accessibility | A11 | Strict: yes; Warn-only: warning | Yes |

---

## Accessibility

- All error output goes to stderr (errors never lost in stdout pipe)
- Error codes (E-CAT-NNN) enable grep filtering without color
- Plain text fallback when NO_COLOR or non-TTY (all ANSI stripped)
- Source line numbers in every error (machine-parseable `file:line:col` format)
- Hint text is readable prose — no jargon, no "see docs" without a URL
- `error: aborting due to N previous errors` footer states the total count plainly

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 60 chars | Source context truncated; `^` caret may wrap — still correct |
| 60-120 chars | Default layout |
| > 120 chars | No change — errors are already compact |
| No TTY | All ANSI stripped; `-->` and `|` characters preserved (ASCII art structure readable as plain text) |
