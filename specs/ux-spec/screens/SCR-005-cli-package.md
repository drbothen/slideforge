---
document_type: ux-spec-screen
screen_id: "SCR-005"
screen_name: "CLI: package"
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-05-24T00:00:00
phase: 1c
complexity: simple
traces_to: UX-INDEX.md
prd_requirements:
  - "PRD §3.1 CLI Command Surface (package)"
  - "interface-definitions.md §1.4"
  - "BC-5.03.001-003"
  - "q16-q25-decisions.md Q19 (package model)"
  - "error-taxonomy.md E-PKG-001 through E-PKG-004"
---

# Screen: CLI package (SCR-005)

> **Sharded UX screen (DF-021).** Navigate via `UX-INDEX.md`.

## Purpose and User Context

`slideforge package` manages content packages (install, list, remove, verify).
Packages are git-based: `slideforge package install github.com/org/slides`.
The lockfile (`sf.lock`) records pinned commit SHAs and SHA-256 checksums.

---

## Elements

| ID | Type | Label | Notes |
|----|------|-------|-------|
| ELM-001 | Install progress | `Fetching github.com/org/slides...` | With spinner if TTY |
| ELM-002 | Install success | `Installed 1898-slides v1.2.0` | With SHA-256 preview |
| ELM-003 | Lock update notice | `Updated sf.lock` | Always shown on install |
| ELM-004 | Package list row | `1898-slides  v1.2.0  git+https://github.com/1898/slides` | From list subcommand |
| ELM-005 | Verify result | `1898-slides v1.2.0  OK  (sha256: e3b0c44...)` | From verify subcommand |
| ELM-006 | Verify failure | `error[E-PKG-003]: Checksum mismatch` | With expected vs actual |

---

## Output Format Specification

### Install Success

```
Fetching https://github.com/1898/slides...
  Resolved v1.2.0 (commit abc123de)
  Verified sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

Installed 1898-slides v1.2.0
Updated sf.lock

  @import "1898-slides/title-slide"   — ready to use
  @import "1898-slides/content"       — ready to use
```

Color application:
- `Installed ...` line: `color.success`
- `@import` examples: `color.file.path`
- Commit SHA: `color.timing` (dim)

### Package List

```
$ slideforge package list

Installed packages (from sf.lock):

  NAME           VERSION  SOURCE
  1898-slides    v1.2.0   git+https://github.com/1898/slides?tag=v1.2.0
  mssp-common    v1.0.0   git+https://github.com/mssp-tools/common?tag=v1.0.0

2 packages installed. Run `slideforge package verify` to check checksums.
```

### Verify All

```
$ slideforge package verify

  1898-slides  v1.2.0  sha256: e3b0c44...  OK
  mssp-common  v1.0.0  sha256: 789def0...  OK

All 2 packages verified.
```

### E-PKG-003 Checksum Mismatch

```
error[E-PKG-003]: Package 'mssp-common' SHA-256 checksum mismatch.
  Expected: sha256:789def012345678...
  Got:      sha256:aabbccdd012345...
  = hint: The package contents changed since you locked it. This may indicate
          a supply-chain issue. Run: slideforge package install mssp-common --version 1.0.0
          to reinstall from the original tag, or audit the package manually.
```
Exit code 5.

---

## Interactions

| ID | Trigger | Success Path | Error Path |
|----|---------|-------------|------------|
| INT-001 | `package install <repo>` | Fetch, pin, verify, write sf.lock | E-PKG-002 if unreachable; E-PKG-001 if manifest missing |
| INT-002 | `package list` | Print installed packages table | E-CFG-005 if no sf.lock |
| INT-003 | `package verify` | Check all checksums; OK all pass | E-PKG-003 if mismatch; exit 5 |

---

## Accessibility

All output plain text. Checksums shown as full hex strings (not truncated in final output;
truncated only in list display for readability).

## Responsive Adaptations (Terminal Width)

| Width | Adaptation |
|-------|-----------|
| < 80 chars | List table wraps SOURCE column to new line |
| 80+ chars | Default tabular layout |
