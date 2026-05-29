---
document_type: prd-supplement
supplement_type: error-taxonomy
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-05-29T00:00:00
phase: 1a
traces_to: .factory/specs/prd.md
primary_consumers: [implementer, test-writer]
---

# Error Taxonomy — slideforge v1.0

> Convention: E-<CAT>-<NNN> where CAT = 3-char subsystem abbreviation.
> Severity: broken (prevents output), degraded (partial output in warn-only), cosmetic (lint).
> Primary consumers: implementer, test-writer.

---

## Parse Errors (E-PAR)

Always fatal. Build halts with accumulated errors. No output produced.

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-PAR-001 | broken | 1 | `Unexpected indentation at <file>:<line>:<col>. Expected <N> spaces, found <M>.` | DI-018, CAP-001 |
| E-PAR-002 | broken | 1 | `Unclosed block starting at <file>:<line>:<col>. Missing dedent or closing keyword.` | CAP-001 |
| E-PAR-003 | broken | 1 | `Tab character at <file>:<line>:<col>. slideforge requires spaces for indentation.` | CAP-001 |
| E-PAR-004 | broken | 1 | `Include cycle detected: <path1> → <path2> → ... → <path1>` | DI-007, CAP-006 |
| E-PAR-005 | broken | 1 | `File not found: '<resolved-path>' (referenced at <file>:<line>:<col>)` | CAP-006 |
| E-PAR-006 | broken | 1 | `Reserved keyword '<word>' at <file>:<line>:<col>. '<word>' is reserved for <feature> (planned v2+). Did you mean '<suggestion>'?` | DI-021, CAP-001 |
| E-PAR-007 | broken | 1 | `Unknown slide type '<keyword>' at <file>:<line>:<col>. Did you mean '<closest-match>'?` | CAP-010 |
| E-PAR-008 | broken | 1 | `Variable name '<name>' collides with reserved keyword at <file>:<line>:<col>. Choose a different name.` | DI-021, CAP-001 |
| E-PAR-009 | broken | 1 | `'raw' keyword is not available in user .sf files at <file>:<line>:<col>. Use the shape: DSL instead.` | CAP-023 |
| E-PAR-010 | broken | 1 | `slideforge_version "<ver>" is not supported. This binary supports version "<supported>".` | CAP-028 |
| E-PAR-011 | broken | 1 | `Variant cycle detected: <var1> → <var2> → ... → <var1>` | DI-022, CAP-007 |
| ~~E-PAR-012~~ | ~~retired~~ | — | ~~Invalid indentation level at `<file>:<line>:<col>`. Expected multiple of `<N>` spaces.~~ RETIRED: indentation-level errors are subsumed by E-PAR-001 (which carries span and column info sufficient to diagnose this case). E-PAR-012 was reassigned in the STORY-028 Pass-1 adjudication (2026-05-28). | CAP-001 |
| E-PAR-012-SHP | broken | 1 | `Unknown shape type '<keyword>' at <file>:<line>:<col>. Known types: [rect, ellipse, arrow, line, star, roundRect].` | CAP-023 |
| E-PAR-013 | broken | 1 | `Invalid hex color '<value>' at <file>:<line>:<col>. Expected 6-digit hex (#RRGGBB). Short-form #RGB and alpha #RRGGBBAA are not supported.` | CAP-023 |
| E-PAR-014 | broken | 1 | `Shape gradient fill is not supported in v1.0 at <file>:<line>:<col>. Use a solid hex color or 'none'. Gradient fill is planned for a future release.` | CAP-023 |

---

## Evaluation Errors (E-EVL)

Fatal in strict mode (exit 2). In `--warn-only` mode: error-slide placeholder rendered
at the affected position; build continues.

| Code | Severity | Exit (strict) | Message Format | Traces To |
|------|---------|--------------|---------------|-----------|
| E-EVL-001 | broken | 2 | `Undefined variable '{{ <name> }}' at <file>:<line>:<col>. Variables in scope: [<list>]` | DI-006, CAP-002 |
| E-EVL-002 | broken | 2 | `@{<name>} undefined in math context at <file>:<line>:<col>. Declare in vars: block.` | DI-006, CAP-012 |
| E-EVL-003 | broken | 2 | `Type error in expression at <file>:<line>:<col>: operator '<op>' expects <type>, got <actual>` | DI-004, CAP-002 |
| E-EVL-004 | broken | 2 | `Filter '<name>' not found at <file>:<line>:<col>. Available filters: [<list>]` | CAP-002 |
| ~~E-EVL-005~~ | ~~retired~~ | — | ~~Variant '<name>' not declared. Defined variants: [<list>]. Did you mean '<closest>'?~~ RETIRED: `--variant` is a CLI configuration flag, so undefined-variant errors are configuration errors. Use E-CFG-001 (exit 4) instead. See BC-1.07.004 and E-CFG-001. | DI-022, CAP-007 |
| E-EVL-006 | broken | 2 | `@for over non-collection value at <file>:<line>:<col>. '{{ <expr> }}' evaluated to <type>, expected list or map.` | CAP-004 |

---

## Data Source Errors (E-DAT)

Fatal in strict mode (exit 2). In `--warn-only` mode: error-slide placeholder on all
slides that reference the failed data source.

| Code | Severity | Exit (strict) | Message Format | Traces To |
|------|---------|--------------|---------------|-----------|
| E-DAT-001 | broken | 2 | `HTTP fetch failed: '<url>' returned HTTP <code>. Hint: use --offline to skip HTTP sources.` | CAP-003, FM-005 |
| E-DAT-002 | broken | 2 | `Network error fetching '<url>': <os-error>. Use --offline to skip HTTP sources.` | CAP-003, FM-005 |
| E-DAT-003 | broken | 2 | `Cannot parse response from '<url>' as <format>: <parse-error>` | CAP-003 |
| E-DAT-004 | broken | 2 | `File data source not found: '<path>' (referenced at <file>:<line>:<col>)` | CAP-003 |
| E-DAT-005 | broken | 2 | `Missing field '<field-path>' in data source '<name>' at <file>:<line>:<col>. Field does not exist in source data.` | DI-006, CAP-003 |
| E-DAT-006 | broken | 2 | `HTTP source '<url>' blocked by allowed_domains policy. Add domain to [data].allowed_domains in slideforge.toml.` | CAP-003, R-011 |
| E-DAT-007 | broken | 2 | `XLSX header row at '<path>' has empty cell at column <idx> (0-indexed). All header cells must be non-empty strings. Do not use blank column headers; remove unused columns or name all headers.` | BC-1.03.006 EC-007, DI-004 |
| E-DAT-008 | broken | 2 | `XLSX header cell at column <idx> in '<path>' has type <calamine-type> (value: <repr>). Header cells must be String-typed. Use a string label as the column header.` | BC-1.03.006 EC-008, DI-004 |
| E-DAT-009 | broken | 2 | `XLSX datetime cell at <col>:<row> in '<path>' has invalid ISO 8601 value '<value>'. Expected format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS±HH:MM` | BC-1.03.006 EC-009, DI-004 |
| E-DAT-010 | broken | 2 | `XLSX numeric cell at <col>:<row> in '<path>' has non-finite value (<NaN\|Infinity>). Non-finite floats are not representable in slideforge values.` | BC-1.03.006 EC-012 |
| E-DAT-011 | broken | 2 | `'<path>' has .<ext> extension but is not a valid XLSX archive (ZIP magic bytes not found). File may be corrupted or misnamed.` | BC-1.03.006 EC-013 |
| E-DAT-012 | broken | 2 | `TEXT column '<column_name>' at row <row_idx> in '<path>' contains invalid UTF-8 bytes. SQLite TEXT values must be valid UTF-8.` | BC-1.03.007 EC-008, DI-004 |
| E-DAT-013 | broken | 2 | `'<path>' has .<ext> extension but is not a valid SQLite database (SQLite file header not found). File may be corrupted or misnamed.` | BC-1.03.007 EC-009 |
| E-DAT-014 | broken | 2 | `Unsupported extension for SQLite data source: '<ext>'. Accepted extensions: .db, .sqlite, .sqlite3` | BC-1.03.007 EC-010 |

---

## Layout Errors (E-LAY)

Canvas overflow is a warning by default. Set `[build].strict_overflow = true` in
slideforge.toml to promote to a blocking error.

| Code | Severity | Exit (strict-overflow) | Message Format | Traces To |
|------|---------|----------------------|---------------|-----------|
| E-LAY-001 | degraded | 2 (if strict-overflow) | `CanvasOverflow: slide '<title>' field '<field>' overflows by ~<N> EMU (~<M>pt). Consider reducing content or font size.` | CAP-022, DEC-013 |
| E-LAY-002 | broken | 2 | `Zero-slide deck: no slide blocks found in '<file>'. A deck must contain at least one slide.` | CAP-022, DEC-011 |
| E-LAY-003 | degraded | 2 (if strict-overflow) | `Chart data is empty for slide '<title>'. Rendering error-slide placeholder.` | CAP-013, DEC-014 |
| E-LAY-004 | broken | 2 | `Shape at slide <slide_index> (<file>:<line>:<col>) has no alt text and is not marked decorative: true. Add alt "..." or decorative: true.` | BC-3.04.001 EC-001, DI-001, CAP-023 |
| E-LAY-005 | broken | 2 | `Inline nesting depth exceeded at slide <slide_index>: depth <depth> exceeds maximum of 64. Flatten the inline tree.` | BC-3.05.001 EC-006, CAP-024 |
| E-LAY-006 | broken | 2 | `Arithmetic overflow computing EMU for shape position at slide <source_slide_index> (<file>:<line>:<col>). Value <value> in <unit> exceeds i64 range after conversion. Use a value ≤ 9,007,199,254 inches (approximately 9.0 × 10⁹ in).` | BC-3.04.001 EC-014, EC-015, CAP-023 |

Note (E-LAY-004): This is the layout-layer defensive check for missing alt text on
shapes. `LayoutError::MissingAlt` maps to E-LAY-004 in the CLI diagnostic renderer.
The variant carries a `span: SourceSpan` field (file/line/col). The primary enforcement
is E-A11-001 in the validation stage; E-LAY-004 fires only if the validation stage was
bypassed (internal invariant violation).

Note (E-LAY-004 field naming): The canonical field name for slide index in
`LayoutError::MissingAlt` is `slide_index` (not `source_slide_index`). The module
doc-comment in `slideforge-layout/src/error.rs` describes the intent to use
`source_slide_index` consistently across variants, but the STORY-028 variants
(`MissingAlt`, `MissingRiskCardField`, `MalformedSeverityCards`,
`UnresolvedSeverityCards`) use `slide_index`. **The canonical field name going forward
is `source_slide_index`** per the module header declaration (which is the source of
truth per CLAUDE.md precedence rule — the header doc is earlier and more authoritative
than individual variant choices). Data-engineer must rename `slide_index` → `source_slide_index`
on `MissingAlt`, `MissingRiskCardField`, `MalformedSeverityCards`, and
`UnresolvedSeverityCards` in a follow-up fix burst anchored to STORY-028.

Note (E-LAY-006): `LayoutError::ArithmeticOverflow { source_slide_index: usize, span: SourceSpan }`
maps to E-LAY-006. This error is produced when `ShapeUnit::from_inches` or `ShapeUnit::from_em`
detects i64 overflow during the milliunit-to-EMU multiplication (the `checked_mul` path).
Silent saturation via `saturating_mul` is FORBIDDEN — it would produce garbage coordinates
in the IR. The `ArithmeticOverflow` variant MUST have a load-bearing constructor path (it
is NOT dead code); any implementation that marks it `#[allow(dead_code)]` or documents it
as "future strict-mode validator" is in direct violation of BC-3.04.001 invariant 8 and
CLAUDE.md Rule 3 (no AI-added tech-debt register entries without explicit human direction).
Adjudicated in adversary pass 2 on STORY-028, item M (2026-05-29).

---

## Export Errors (E-EXP)

Always fatal (exit 3). Partial output is never written to disk (all-or-nothing).

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-EXP-001 | broken | 3 | `PPTX serialization error at OOXML element '<element-path>': <detail>` | CAP-015, FM-010 |
| E-EXP-002 | broken | 3 | `DOCX serialization error: <detail>` | CAP-016 |
| E-EXP-003 | broken | 3 | `PDF export error: <detail>. Verify pdf-writer + krilla stack.` | CAP-017, FM-011 |
| E-EXP-004 | broken | 3 | `SVG normalization failed for diagram '<title>': <usvg-error>` | CAP-014 |
| E-EXP-005 | broken | 3 | `Chart render failed for slide '<title>': <plotters-error>` | CAP-013, FM-013 |
| E-EXP-006 | broken | 3 | `Math render failed for expression at <file>:<line>:<col>: <detail>. See supported LaTeX subset in DSL reference.` | CAP-012, FM-012 |
| E-EXP-007 | broken | 3 | `Cannot write output to '<path>': <os-error>. Check directory exists and is writable.` | FM-015 |
| E-EXP-008 | broken | 3 | `Diagram render failed for slide '<title>': <mermaid-error> at source line <N>` | CAP-014, FM-014 |

---

## Brand Errors (E-BRD)

Fatal for missing brand (exit 4). Warnings for inferred color slots (build continues).

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-BRD-001 | broken | 4 | `Brand file not found: '<resolved-path>'. Check the template path in slideforge.toml or --template flag.` | CAP-018, FM-007 |
| E-BRD-002 | broken | 4 | `Cannot parse brand template '<path>': <detail>. File may be corrupted or not a valid PPTX/DOCX.` | CAP-018, FM-007 |
| E-BRD-003 | cosmetic | 0 | `Brand color slot '<slot>' inferred as #<hex> (derived from <source-color>). Review in brand.toml to confirm.` | DI-015, CAP-018, DEC-016 |
| E-BRD-004 | cosmetic | 0 | `Font '<font-name>' not available on this build host. Using '<fallback>' (panose: [<class>]). Text metrics may differ.` | CAP-018, FM-009 |
| E-BRD-005 | cosmetic | 0 | `Invalid hex color value '<value>' in brand.toml slot '<slot>'. Use 6-digit uppercase hex RGB (e.g. #3B82F6; case insensitive — uppercase or lowercase accepted).` | CAP-018, FM-007 |
| E-BRD-007 | broken | 4 | `Logo path '<logo_path>' escapes the brand.toml directory '<brand_dir>'. The logo file must be inside (or beneath) the brand.toml directory.` | CAP-018, FM-007 |

Note (E-BRD-005): The original semantic ("missing required color slot — fatal") was retired when the brand synthesis algorithm (BC-2.01.004) was designed to always infer missing slots (no execution path produces a fatal missing-slot error). The code number was subsequently reused for invalid hex color validation, added in the Pass-11 fix burst. The revised semantic is documented here; BC-2.01.004 and BC-2.01.005 describe the inference algorithm that makes the original missing-slot fatal error unreachable.

---

## Package Errors (E-PKG)

Fatal for missing/mismatched package (exit 5).

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-PKG-001 | broken | 5 | `Package '<name>' not found in sf.lock. Run: slideforge package install <repo>` | DI-019, CAP-025, DEC-019 |
| E-PKG-002 | broken | 5 | `Cannot fetch package '<url>': <network-error>. Use --offline to skip network access.` | CAP-025 |
| E-PKG-003 | broken | 5 | `Package '<name>' SHA-256 checksum mismatch. Expected: <expected>, got: <actual>. Run: slideforge package verify` | CAP-025, R-012 |
| E-PKG-004 | degraded | 0 | `sf.lock not committed or missing for project with dependencies. Run: slideforge package lock. Builds may not be reproducible.` | DI-019, CAP-025 |

---

## Configuration Errors (E-CFG)

Fatal (exit 4).

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-CFG-001 | broken | 4 | `Variant '<name>' not defined in deck. Defined variants: [<list>]. Did you mean '<closest>'?` | DI-022, CAP-007, DEC-020 |
| E-CFG-002 | broken | 64 | `Cannot specify both a source file and --workspace. Use one or the other.` | CAP-026 |
| E-CFG-003 | ~~retired~~ | — | ~~Flags --warn-only and --strict-overflow are contradictory~~. RETIRED: these flags compose correctly (see interface-definitions.md §5.1). No error is emitted. | — |
| E-CFG-004 | broken | 64 | `Flags --quiet and --verbose are contradictory.` | CAP-030 |
| E-CFG-005 | broken | 4 | `slideforge.toml [workspace] not found in '<dir>' or any parent directory.` | CAP-026 |
| E-CFG-006 | broken | 4 | `.sfconfig at '<path>' is malformed: <detail>` | CAP-026 |
| E-CFG-007 | broken | 64 | `Package list/remove/verify subcommand requires a project with slideforge.toml. None found in '<dir>' or any parent.` | CAP-025 |
| E-CFG-008 | broken | 4 | `slideforge.toml already exists in '<dir>'. Run with --force to overwrite (CAUTION: destructive), or remove slideforge.toml first.` | CAP-026 |

---

## Accessibility Errors (E-A11)

Fatal in strict mode (exit 2). Warning in `--warn-only` mode (but output still produced).

| Code | Severity | Exit (strict) | Message Format | Traces To |
|------|---------|--------------|---------------|-----------|
| E-A11-001 | broken | 2 | `Missing alt text on <element-type> '<identifier>' at <file>:<line>:<col>. Add alt "..." or mark decorative: true.` | DI-001, CAP-020 |
| E-A11-002 | broken | 2 | `Missing label on color-coded element '<type>' '<identifier>' at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".` | DI-002, CAP-020 |
| E-A11-003 | cosmetic | 0 | `Missing lang declaration in deck metadata. Defaulting to "en". Screen readers may mispronounce non-English content. Add lang "en-US" (or appropriate BCP-47 tag).` | DI-003, CAP-020 |
| E-A11-004 | broken | 2 | `WCAG contrast ratio insufficient for text '<excerpt>' at <file>:<line>:<col>: <ratio>:1 (required 4.5:1 for normal text, 3:1 for large text). Consider using a higher-contrast color combination.` | CAP-022 |

---

## Diagnostic Severity Definitions

| Severity | Meaning | Strict Mode | Warn-Only Mode |
|---------|---------|------------|---------------|
| `broken` | Prevents output production | Fatal (non-zero exit) | Some errors become warnings (E-EVL, E-DAT, E-A11) |
| `degraded` | Output produced but potentially incorrect | Fatal if --strict-overflow | Output + placeholder/warning |
| `cosmetic` | Style or advisory warning; no functional impact | Warning (exit 0) | Warning (exit 0) |

---

## Error Accumulation Policy

Per DI-018 and BC-1.15.002:
- All errors in the parse and evaluation phases are accumulated before reporting.
- The parser continues after a recoverable error to find additional errors.
- A single build run reports ALL parse errors, not just the first one.
- Exception: if a parse error prevents further parsing (unclosed block, complete
  indentation failure), accumulation stops at that point and reports all errors found so far.
- Exit code is the HIGHEST severity code encountered across all errors.
