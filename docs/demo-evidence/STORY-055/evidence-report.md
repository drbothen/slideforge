# Evidence Report — STORY-055

**Story:** STORY-055 — `slideforge build` command + miette error rendering  
**Feature branch:** `feature/STORY-055` (HEAD `fcdd1c55`)  
**Recorded:** 2026-06-08  
**Toolchain:** VHS 0.10.0 / slideforge CLI debug binary  
**Font:** FiraCode Nerd Font Mono (detected via `fc-list`)

---

## AC Coverage Map

| AC | Description | Success Evidence | Error Evidence | Exit Code Verified |
|----|-------------|-----------------|----------------|-------------------|
| AC-001 | Default build produces pptx + docx + pdf + html, exit 0 | AC-001-010-success-all-formats.gif/.webm | — | exit:0 shown |
| AC-002 | Parse error (tab indent) → exit 1, no output, miette span | — | AC-002-parse-error.gif/.webm | exit:1 shown |
| AC-003 | Eval error, strict mode → exit 2, no output | — | AC-003-eval-error-strict.gif/.webm | exit:2 shown |
| AC-004 | Eval error + --warn-only → exit 0, output written | AC-004-eval-warn-only.gif/.webm (second half) | AC-004-eval-warn-only.gif/.webm (first half, strict=exit2) | exit:0 and exit:2 shown |
| AC-005 | Export failure (unwritable dir) → exit 3, no partial output | — | AC-005-export-failure.gif/.webm | exit:3 shown |
| AC-006 | Multiple independent errors all reported in one run | — | AC-006-007-multi-error-source-order.gif/.webm | exit:1 shown, both E-PAR-002 lines visible |
| AC-007 | Errors printed in ascending source order (line 2 before line 3) | — | AC-006-007-multi-error-source-order.gif/.webm | Combined with AC-006 |
| AC-009 | --no-color → plain text, no ANSI codes | AC-009-no-color.gif/.webm | — | CLEAN-no-ANSI confirmed |
| AC-010 | --format pptx,html → only those two files written | AC-001-010-success-all-formats.gif/.webm (second scene) | — | exit:0 + ls shows pptx + html only |
| AC-011 | 6 tracing spans emitted per build (RUST_LOG=info) | AC-011-tracing-spans.gif/.webm | — | All 6 stage: lines visible |
| AC-012 | Exit code reflects earliest pipeline stage (parse=1 beats eval=2) | — | AC-012-exit-code-precedence.gif/.webm | exit:1 with both parse+eval errors in source |
| AC-015 | OTel feature gate (--otel-endpoint) | Not recorded — covered by unit tests (otel feature flag; no live endpoint required) | — | See test_BC_1_15_003_ac_015_otel_endpoint_flag_accepted_and_layer_constructed |

---

## Recording Artifacts

### AC-001 / AC-010 — Successful build: all formats + format selection

| File | Description |
|------|-------------|
| `AC-001-010-success-all-formats.gif` | VHS recording — scene 1: default build writes deck.pptx/.docx/.pdf/.html, exit 0; scene 2: --format pptx,html writes exactly two files |
| `AC-001-010-success-all-formats.webm` | Archival copy |
| `AC-001-010-success-all-formats.tape` | VHS script source |

**Acceptance criteria exercised:** AC-001 (all 4 formats, exit 0), AC-010 (format selection, exit 0)

---

### AC-002 — Parse error

| File | Description |
|------|-------------|
| `AC-002-parse-error.gif` | VHS recording — tab on line 2 → E-PAR-002 with file:line:col span, exit 1, dist/ absent |
| `AC-002-parse-error.webm` | Archival copy |
| `AC-002-parse-error.tape` | VHS script source |

**Acceptance criteria exercised:** AC-002 (parse error, exit 1, no output)

**Fixture:** `parse_error.sf` — `slideforge_version "1"\n\tlang "en-US"\n` (tab on line 2 = E-PAR-002)

---

### AC-003 — Eval error, strict mode

| File | Description |
|------|-------------|
| `AC-003-eval-error-strict.gif` | VHS recording — `{{ undefined_variable }}` in strict mode → E-EVL-001, exit 2, no output |
| `AC-003-eval-error-strict.webm` | Archival copy |
| `AC-003-eval-error-strict.tape` | VHS script source |

**Acceptance criteria exercised:** AC-003 (eval error, strict, exit 2, no output)

---

### AC-004 — Eval error + --warn-only

| File | Description |
|------|-------------|
| `AC-004-eval-warn-only.gif` | VHS recording — scene 1: strict mode → exit 2, no output; scene 2: --warn-only → exit 0, eval_error.pptx written |
| `AC-004-eval-warn-only.webm` | Archival copy |
| `AC-004-eval-warn-only.tape` | VHS script source |

**Acceptance criteria exercised:** AC-004 (warn-only, exit 0, output produced) — also shows contrast with AC-003 strict path

---

### AC-005 — Export failure

| File | Description |
|------|-------------|
| `AC-005-export-failure.gif` | VHS recording — output dir `/nonexist-root-xyz/dist` is unwritable → exit 3, no partial output |
| `AC-005-export-failure.webm` | Archival copy |
| `AC-005-export-failure.tape` | VHS script source |

**Acceptance criteria exercised:** AC-005 (export failure, exit 3, no partial output)

---

### AC-006 / AC-007 — Multiple errors, source order

| File | Description |
|------|-------------|
| `AC-006-007-multi-error-source-order.gif` | VHS recording — two tabs at lines 2 and 3 → both E-PAR-002 entries shown in source order (line 2 first), exit 1 |
| `AC-006-007-multi-error-source-order.webm` | Archival copy |
| `AC-006-007-multi-error-source-order.tape` | VHS script source |

**Acceptance criteria exercised:** AC-006 (all errors reported), AC-007 (source-order: line 2 before line 3)

---

### AC-009 — --no-color plain text

| File | Description |
|------|-------------|
| `AC-009-no-color.gif` | VHS recording — scene 1: --no-color error output (plain text, no ANSI); scene 2: pipe through cat -v confirms zero ESC bytes |
| `AC-009-no-color.webm` | Archival copy |
| `AC-009-no-color.tape` | VHS script source |

**Acceptance criteria exercised:** AC-009 (--no-color, no ANSI escape codes in output)

---

### AC-011 — Tracing spans

| File | Description |
|------|-------------|
| `AC-011-tracing-spans.gif` | VHS recording — RUST_LOG=info shows all 6 pipeline stage spans: parse, evaluate, brand, validate, layout, export |
| `AC-011-tracing-spans.webm` | Archival copy |
| `AC-011-tracing-spans.tape` | VHS script source |

**Acceptance criteria exercised:** AC-011 (6 tracing spans emitted per build)

---

### AC-012 — Exit code precedence

| File | Description |
|------|-------------|
| `AC-012-exit-code-precedence.gif` | VHS recording — source has tab (E-PAR-002) on line 2 and `{{ undef }}` (E-EVL-001) on line 4 → exit 1 (parse wins over eval=2) |
| `AC-012-exit-code-precedence.webm` | Archival copy |
| `AC-012-exit-code-precedence.tape` | VHS script source |

**Acceptance criteria exercised:** AC-012 (parse error exit code takes precedence over eval error)

---

## Non-Recorded ACs (unit test coverage)

| AC | Reason not recorded |
|----|---------------------|
| AC-015 (OTel feature gate) | Requires `otel` Cargo feature compiled in; not compiled in debug binary. Unit test `test_BC_1_15_003_ac_015_otel_endpoint_flag_accepted_and_layer_constructed` in `crates/slideforge-cli/tests/build_integration.rs` provides coverage via `#[cfg(feature = "otel")]`. |

---

## Recording Notes

- **VHS version:** 0.10.0 — does not support absolute paths in `Output` directives; relative paths required (tapes invoked from evidence directory).
- **VHS parser limitation:** `Wait+Line /pattern/` fails for most shell prompt patterns in this version; replaced with `Sleep` timing for command completion.
- **Fixture pre-creation:** All `.sf` fixture files and `brand.toml`/`logo.png` were pre-created at `/tmp/sf-fixtures/` before VHS invocation. This avoids embedded newline/escape-character issues in VHS `Type` strings.
- **PATH injection:** `slideforge` binary injected via `export PATH="...target/debug:$PATH"` in the shell that invokes `vhs`, not inside the tape (VHS inherits the invoking shell's environment).
- **Font:** `FiraCode Nerd Font Mono` (detected via `fc-list`; priority list: JetBrains Mono > FiraCode Nerd Font Mono > Menlo).
- **No harness/example code added:** No new production or test code was written for this recording pass. Demo fixtures are ephemeral `/tmp/` files; no source modifications.

---

## Canonical Gate Status

No source code was added or modified. The gate question is N/A for demo-recorder output.

The integration test suite (`cargo nextest run -p slideforge-cli`) passes cleanly on HEAD `fcdd1c55` per the implementer's pre-commit gate run (cited in the story delivery record).
