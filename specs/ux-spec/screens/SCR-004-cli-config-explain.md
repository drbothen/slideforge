---
document_type: ux-spec-screen
screen_id: "SCR-004"
screen_name: "CLI: config explain"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: simple
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §3.1 CLI Command Surface (config)"
  - "interface-definitions.md §1.6"
  - "BC-5.04.001-003"
  - "q16-q25-decisions.md Q20 (config explain output example)"
---

# Screen: CLI config explain (SCR-004)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.

## Purpose and User Context

`slideforge config explain` shows the resolved configuration for a deck file with
full provenance: which value came from which config file at which cascade level.
This is the primary debugging tool for workspace configuration issues.

Canonical example from Q20 decision doc:
```
$ slideforge config explain incident-briefs/inc-2026-0320.sf
```

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Config key | `brand = "brands/incident-brand.toml"` | Key = value pairs |
| ELM-002 | Provenance | `# from: .sfconfig (incident-briefs/)` | Source annotation; dimmed |
| ELM-003 | Section header | `[build]`, `[brand]`, `[data]` | Groups related keys |
| ELM-004 | Override notice | `# CLI flag overrides this` | When CLI flag beats config |
| ELM-005 | Auto-applied notice | `# auto-applied from defaults/content.sf` | Q21 defaults/ directory |

---

## Output Format Specification

### Full Config Explain (no KEY argument)

```
Config for: incident-briefs/inc-2026-0320.sf

[brand]
  template = "brands/incident-brand.toml"   # from: .sfconfig (incident-briefs/)

[build]
  default_format = "pptx"                   # from: slideforge.toml (workspace)
  output_dir = "output/incidents/"          # from: .sfconfig (incident-briefs/)
  warn_only = false                         # from: slideforge.toml (workspace default)

[data]
  watch_interval_secs = 30                  # from: slideforge.toml (workspace default)

[set.severity_cards]
  color_high = "red"                        # from: .sfconfig (incident-briefs/)
  color_medium = "orange"                   # from: .sfconfig (incident-briefs/)
  color_low = "green"                       # from: .sfconfig (incident-briefs/)

[vars]
  department = "Security Operations"        # from: .sfconfig (incident-briefs/)
  classification = "CONFIDENTIAL"           # from: .sfconfig (incident-briefs/)

[variants.exec]
  exclude_tags = ["internal"]              # from: .sfconfig (incident-briefs/)

[packages]
  1898-slides = "v1.2.0"                   # from: sf.lock (workspace)

[defaults]
  # auto-applied from defaults/severity_cards.sf
  set.severity_cards.color_high = "red"
```

Color application:
- Section headers `[brand]` etc.: `color.stage.label`
- Key names: default white
- Values in quotes: `color.file.path` for paths; white for others
- `# from:` annotations: `color.timing` (dim)
- `# auto-applied from`: `color.hint`

### Single Key Explain

```
$ slideforge config explain brand.template

  brand.template = "brands/incident-brand.toml"

  Provenance:
    1. .sfconfig at incident-briefs/          (current — wins)
    2. slideforge.toml at workspace root      (overridden)
       value: "default-brand.toml"

  Cascade levels checked (highest first):
    CLI flag (--template)                     (not set)
    deck metadata (brand: ...)                (not set)
    .sfconfig (incident-briefs/)              → "brands/incident-brand.toml" ✓
    .sfconfig (parent dir)                    (not present)
    slideforge.toml workspace                 → "default-brand.toml"
    built-in default                          (none)
```

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | `slideforge config explain deck.sf` | Print full config with provenance | E-CFG-005 if no slideforge.toml found; E-CFG-006 if .sfconfig malformed |
| INT-002 | `slideforge config explain brand.template` | Print single key with cascade trace | E-CFG-005 if not in workspace |
| INT-003 | `--workspace-root <DIR>` | Resolve from specified root | E-CFG-005 if workspace.toml not found there |

---

## Accessibility

- All information in plain text; provenance comments use `# from:` prefix
- Color supplements only (dim for annotations, normal for values)
- Screen-reader safe: simple left-to-right line structure

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 60 chars | Provenance annotations on separate line |
| 60+ chars | Default right-aligned comment style |
