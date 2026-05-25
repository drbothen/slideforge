---
document_type: ux-spec-screen
screen_id: "SCR-010"
screen_name: "Workspace"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: simple
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §3.1 CLI Command Surface (--workspace flag)"
  - "interface-definitions.md §5.3 (--workspace behavior)"
  - "BC-5.04.001-003"
  - "q16-q25-decisions.md Q20 (workspace config)"
  - "q16-q25-decisions.md Q21 (defaults/ directory)"
---

# Screen: Workspace (SCR-010)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.

## Purpose and User Context

The workspace experience covers multi-deck projects where `slideforge.toml` declares
multiple `.sf` members. `slideforge build --workspace` builds all of them. The `.sfconfig`
cascade allows per-directory overrides (e.g., all incident briefs share a brand).

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Workspace header | Member count and workspace root | Shown at top of --workspace build |
| ELM-002 | Per-member build line | `[1/3] reports/q3-brief.sf` | One per member |
| ELM-003 | Per-member result | Success or error for each member | Follows per-member line |
| ELM-004 | Workspace summary | N built, M failed | Final summary line |
| ELM-005 | Config cascade table | From `config explain` output | Per SCR-004 |

---

## Output Format Specification

### Workspace Build (--workspace)

```
$ slideforge build --workspace

Building workspace (3 members):

  [1/3] reports/q3-brief.sf
        Wrote dist/q3-brief.pptx (102 KB) in 285ms

  [2/3] incident-briefs/inc-2026-0320.sf
        Wrote dist/inc-2026-0320.pptx (89 KB) in 231ms

  [3/3] briefs/exec-overview.sf

error[E-EVL-001]: Undefined variable '{{ kpi_target }}' at briefs/exec-overview.sf:47:9.
  --> briefs/exec-overview.sf:47:9
   |
47 |   subtitle "Target: {{ kpi_target }}"
   |             ^^^^^^^^^^^^^^^^^^^^^^^^ undefined
   |
   = hint: Declare in vars: block. Active scope: quarter, brand_color

Workspace build: 2 built, 1 failed (exit 2)
```

Color application:
- `[1/3]` counters: `color.timing`
- Member file paths: `color.file.path`
- "Wrote ..." success lines: `color.success`
- Error block: standard SCR-006 colors
- Final summary line: `color.error` if any failed; `color.success` if all pass

### Workspace: All Success

```
Workspace build: 3 built, 0 failed — dist/ (12 files, 1.2 MB total)
```

### E-CFG-002 (Source + --workspace together)

```
error[E-CFG-002]: Cannot specify both a source file and --workspace.
                  Use one or the other.
  = hint: slideforge build deck.sf            → single file
          slideforge build --workspace        → all workspace members
```

### .sfconfig Cascade Explanation (from `config explain`)

Per SCR-004 specification, the cascade explanation shows all levels:
- Slide-level field value (highest precedence)
- Deck-level vars / set
- .sfconfig (deepest directory first)
- slideforge.toml (workspace root)
- defaults/ auto-applied values (lowest, Q21)
- Built-in defaults

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | `slideforge build --workspace` | Build all members; per-member output; summary; exit = highest code | E-CFG-002 if SOURCE also given |
| INT-002 | `slideforge build --workspace --changed` | Build only members changed since last build | E-CFG-005 if not in workspace |
| INT-003 | `.sfconfig` in directory | Auto-applied to all .sf files in that directory and subdirectories (max 3 levels) | E-CFG-006 if .sfconfig malformed |
| INT-004 | `defaults/` directory present | `<slide_type>.sf` files auto-applied as set rules to workspace members (Q21) | Shown in `config explain` with `# auto-applied` annotation |

---

## Accessibility

Plain text output. Member counters `[1/3]` are readable by screen readers.
The summary line "N built, M failed" is a plain prose sentence — no table.

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 60 chars | Per-member success lines wrap; counter + path on first line, result indented |
| 60+ chars | Default layout with indented result |
