---
document_type: ux-spec-screen
screen_id: "SCR-003"
screen_name: "CLI: extract-brand"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: simple
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §3.1 CLI Command Surface (extract-brand)"
  - "interface-definitions.md §1.5"
  - "BC-2.01.003 (brand extraction from .pptx/.docx)"
  - "error-taxonomy.md E-BRD-001, E-BRD-002, E-BRD-003, E-BRD-004"
---

# Screen: CLI extract-brand (SCR-003)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.

## Purpose and User Context

`slideforge extract-brand` reads a `.pptx` or `.docx` template file, extracts
the OOXML theme (12 color slots, typography, logo references), and writes a
`brand.toml` file. This is the entry point for users migrating from an existing
Office template to slideforge.

Typical invocation: `slideforge extract-brand corporate-template.pptx`

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Progress line | `Extracting brand from corporate-template.pptx...` | Dim; shown briefly |
| ELM-002 | Success summary | `Wrote brand.toml` | With file path |
| ELM-003 | Slot summary | Color slot table | 12 slots with hex values |
| ELM-004 | Warning: inferred slot | `warning[E-BRD-003]: Slot 'acc5' inferred as #8B5CF6` | Per inferred slot |
| ELM-005 | Warning: missing font | `warning[E-BRD-004]: Font 'Calibri' unavailable` | If font not on host |
| ELM-006 | Next-step hint | Review prompt | Always shown on success |

---

## Output Format Specification

### Success Output

```
Extracting brand from corporate-template.pptx...

Wrote brand.toml

  Color slots extracted:
    dk1      #1F2937  (text-dark)
    lt1      #FFFFFF  (background)
    dk2      #374151
    lt2      #F9FAFB
    acc1     #3B82F6  (primary)
    acc2     #10B981
    acc3     #F59E0B
    acc4     #EF4444  (danger)
    acc5     #8B5CF6  (inferred from acc1)
    acc6     #06B6D4
    hlink    #2563EB
    folHlink #7C3AED

  Typography:
    heading: Inter 28pt
    body:    Inter 14pt
    code:    (not detected — no code placeholder in template)

1 warning:
  warning[E-BRD-003]: Color slot 'acc5' inferred as #8B5CF6
  (derived from acc1 #3B82F6 with hue-shift). Review and adjust in brand.toml.

Next steps:
  1. Open brand.toml and verify the extracted values
  2. Add to slideforge.toml: [brand] template = "brand.toml"
  3. Run: slideforge build deck.sf
```

Color application:
- Slot names (dk1, acc1...): `color.stage.label`
- Hex values: `color.file.path`
- `(inferred from ...)` annotations: `color.timing`
- Warning block: standard SCR-006 warning format
- Next steps: `color.hint`

### No-Warnings Success

If all 12 color slots are directly extracted (no inference):
```
Wrote brand.toml — 12 color slots, 2 fonts

Next steps:
  1. Open brand.toml and verify the extracted values
  2. Add to slideforge.toml: [brand] template = "brand.toml"
  3. Run: slideforge build deck.sf
```

### Error: Template Not Found

```
error[E-BRD-001]: Brand file not found: 'corporate-template.pptx'.
                  Check the path — file does not exist at that location.
  = hint: Use an absolute path or ensure you are in the correct directory.
          Example: slideforge extract-brand /Users/you/templates/brand.pptx
```
Exit code 4.

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | `slideforge extract-brand template.pptx` | Extract 12 slots + typography; write brand.toml; print slot table | E-BRD-001 if file not found; E-BRD-002 if file corrupt |
| INT-002 | `--output custom-brand.toml` | Write to specified path instead | E-EXP-007 if path not writable |
| INT-003 | Template has inferred slots | Write brand.toml with inferred values; print E-BRD-003 warnings; exit 0 | N/A (warnings are non-fatal) |

---

## Accessibility

- All output is plain text with color supplementing
- Slot names and hex values both printed (color-independent)
- Next-steps section is numbered prose — readable by screen readers
- Exit code 0 on success (with warnings), non-zero only on error

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 50 chars | Slot table wraps; hex on next line |
| 50+ chars | Default two-column slot table |
