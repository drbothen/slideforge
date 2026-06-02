---
document_type: prd-supplement
supplement_type: error-taxonomy
level: L3
version: "2.10"
status: active
producer: product-owner
timestamp: 2026-06-01T00:00:00
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
| E-PAR-013 | broken | 1 | `Empty expression in {{ }} at <file>:<line>:<col>. An expression is required between {{ and }}.` | CAP-001 |
| E-PAR-014 | broken | 1 | `Unterminated math block at <file>:<line>:<col>. Missing closing $ (or $$).` | CAP-012 |
| E-PAR-015 | broken | 1 | `Invalid hex color '<value>' at <file>:<line>:<col>. Expected 6-digit hex (#RRGGBB). Short-form #RGB and alpha #RRGGBBAA are not supported.` | CAP-023 |
| E-PAR-016 | broken | 1 | `Shape gradient fill is not supported in v1.0 at <file>:<line>:<col>. Use a solid hex color or 'none'. Gradient fill is planned for a future release (STORY-072).` | CAP-023 |
| E-PAR-017 | broken | 1 | `Section sub-block key '<name>' is a reserved register name — use '<name>:' register syntax or choose a different key.` | BC-3.02.002 EC-006, CAP-001 |
| E-PAR-018 | broken | 1 | `section blocks must be top-level — found inside <context> block at <file>:<line>:<col>. Move the section: declaration to the top level of the .sf file.` | BC-3.02.002 EC-002, CAP-001 |
| E-PAR-019 | broken | 1 | `Unclosed inline-markup delimiter '<delim>' at <file>:<line>:<col>. Add a matching closing '<delim>' or escape the opening delimiter.` | BC-3.02.002, DIR-077-002 §5, CAP-001 |
| E-PAR-020 | broken | 1 | `Empty inline-markup span '<delim><delim>' at <file>:<line>:<col>. Remove the empty delimiter pair or add content between the delimiters.` | BC-3.02.002, DIR-077-002 §5, CAP-001 |

Note (E-PAR-017): Fires when a key inside a `section:` sub-block collides with a reserved register name (e.g., `notes`, `report`, `detail`). The parser rejects this at parse time to prevent silent shadowing of writing-register syntax. The `<name>` placeholder contains the colliding key name. Maps to STORY-078 (section block parser) and BC-3.02.002 EC-006 (section reserved-name collision). Owner story: STORY-078.

Note (E-PAR-018): The `<context>` placeholder is substituted with one of two values at the error site: `slide` (when the section block appears inside a slide body) or `@for/@if` (when the section block appears inside a control-flow body). The composite `@for/@if` value is the intended behavior — the shared `block_item` combinator that parses both `@for` and `@if` bodies cannot statically distinguish the two control-flow forms without splitting the combinator; reporting the composite accurately tells the user "you are inside a control-flow block" without requiring disproportionate parser restructuring. All contexts are prohibited — section blocks must appear at top-level only. Maps to BC-3.02.002 EC-002 and STORY-078 AC-005.

Note (E-PAR-019): `ParseError::UnclosedInlineDelimiter` — emitted when the inline-markup parser encounters an opening delimiter (e.g., `**`, `__`, `` ` ``) inside a section sub-block field value with no matching closing delimiter before end-of-field or end-of-block. The span points at the opening delimiter character(s). Error is accumulated (does not halt further parsing in the block) and is strict-build-fatal (exit 1). The `<delim>` placeholder is substituted with the actual delimiter token (e.g., `**`, `__`, `` ` ``). Applicable to all inline-markup spans in section sub-block field values (`title:`, `detail:`, `report:`, etc.). Owner story: STORY-077. Traces to BC-3.02.002 and DIR-077-002 §5.

Note (E-PAR-020): `ParseError::EmptyInlineSpan` — emitted when the inline-markup parser encounters a delimiter pair with no content between them (e.g., `****`, `__`, ` `` `). The span points at the opening delimiter. Error is accumulated and strict-build-fatal (exit 1). The `<delim><delim>` placeholder in the message is substituted with the full empty-span token as it appears in source (e.g., `****`, `__`). Distinct from E-PAR-019 (unclosed delimiter, where the closing delimiter is simply absent) — E-PAR-020 fires when both delimiters are present but no content appears between them. Owner story: STORY-077. Traces to BC-3.02.002 and DIR-077-002 §5.

---

## Parse Warnings (W-PAR)

Non-fatal parse-time lint warnings. Build continues; output is produced. Exit code 0 unless
a separate fatal error is also present. Routed to `stderr` with a `warning:` prefix (not
`error:`). Accumulated alongside errors but do not increment the error counter for exit-code
calculation.

Convention: `W-<CAT>-<NNN>` mirrors the error namespace but signals non-fatal severity.

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| W-PAR-001 | cosmetic | 0 | `warning: [W-PAR-001] Unrecognized section sub-block key '<key>' at <file>:<line>:<col> — ignored` | BC-3.02.002 EC-005, CAP-001 |

Note (W-PAR-001): Emitted when the section block parser encounters a sub-block key that is
neither a known layout key, a reserved register name (which triggers E-PAR-017), nor a
`<name>:` register block. Per DIR-077-001-A Ruling 2 (unknown section keys are ignored with
a warning, not rejected). No "Did you mean" suggestion is emitted — the parser does not
enumerate known keys in this message. Maps to BC-3.02.002 EC-005 / invariant 4
/ STORY-078 AC-004.

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
| E-EVL-012 | broken | 2 | `figref() requires a figure-number argument at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002 |
| E-EVL-013 | broken | 2 | `ref() requires a non-empty id string at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002 |

Note (E-EVL-012): `EvalError::FigrefInvalidArg` — `figref()` was called with a missing or non-evaluable argument in an inline-markup context (section sub-block `detail:` or `report:` field). `figref(N)` requires exactly one argument that evaluates to a numeric or string figure number. If no argument is supplied, or the argument cannot be evaluated, no `InlineNode` is produced and this error is emitted. Distinct from E-EVL-011 (`UnsupportedBuiltinCall`), which covers `figref()` in non-inline expression contexts. Eval-stage error raised by `chunks_to_inline_nodes`. Allocated in STORY-077 worktree (DIR-077-002 / F-077-P3-001). Help text: "Use figref(N) where N is the figure number, e.g. figref(3) inside a {{ }} interpolation in a section detail:/report: field."

Note (E-EVL-013): `EvalError::InlineXrefEmptyId` — `ref("")` or `"" | ref` was given an empty id string in an inline-markup context. `ref("id")` requires a non-empty string id. The empty-string check occurs at eval time (after expression interpolation), not at parse time — hence the E-EVL prefix despite originating from DSL source. No `InlineNode::Xref` is produced; error accumulation continues. Allocated in STORY-077 worktree (DIR-077-002 / F-077-P3-001). Help text: "Provide a non-empty slide or figure id, e.g. ref(\"slide-1\") inside a {{ }} interpolation in a section detail:/report: field."

Note (E-EVL numbering gap — taxonomy debt): E-EVL-007 (`TooManySlides`), E-EVL-008 (`NotIterable`), E-EVL-009 (`LargeDeckWarning`), E-EVL-010 (`UnknownSectionType`), and E-EVL-011 (`UnsupportedBuiltinCall`) are allocated in `crates/slideforge-eval/src/error.rs` (visible in the STORY-077 worktree) but have not yet been formally registered in this taxonomy. E-EVL-012 and E-EVL-013 are placed at their canonical sequential positions per the code. Registration of E-EVL-007 through E-EVL-011 is a follow-on taxonomy-completeness task for the product-owner (not in scope for STORY-077 / DIR-077-002 / F-077-P3-001).

---

## Data Source Errors (E-DAT)

Fatal in strict mode (exit 2). In `--warn-only` mode: error-slide placeholder on all
slides that reference the failed data source.

| Code | Severity | Exit (strict) | Message Format | Traces To |
|------|---------|--------------|---------------|-----------|
| E-DAT-001 | broken | 2 | `HTTP fetch failed: '<url>' returned HTTP <status>. Hint: use --offline to skip HTTP sources.` | CAP-003, FM-005 |
| E-DAT-002 | broken | 2 | `Network error fetching '<url>': <os-error>. Use --offline to skip HTTP sources.` | CAP-003, FM-005 |
| E-DAT-003 | broken | 2 | `Cannot parse response from '<url>' as <format>: <parse-error>` | CAP-003 |
| E-DAT-004 | broken | 2 | `File data source not found: '<path>' (referenced at <file>:<line>:<col>)` | CAP-003 |
| E-DAT-005 | broken | 2 | `Missing field '<field-path>' in data source '<name>' at <file>:<line>:<col>. Field does not exist in source data.` | DI-006, CAP-003 |
| E-DAT-006 | broken | 2 | `HTTP source '<url>' blocked by allowed_domains policy. Add domain to [data].allowed_domains in slideforge.toml.` — SSRF sub-case via `DataError::SsrfBlocked`. Also used for body-size cap and similar policy rejections via `DataError::PolicyRejected` (display: `"[E-DAT-006] data policy rejected '<uri>': <message> (at <span>)"`). E-DAT-006 covers all DataSource-policy rejections; route to `SsrfBlocked` for domain blocks, `PolicyRejected` for body-cap and other policy rejections. | CAP-003, R-011 |
| E-DAT-007 | broken | 2 | `XLSX header row at '<path>' has empty cell at column <idx> (0-indexed). All header cells must be non-empty strings. Do not use blank column headers; remove unused columns or name all headers.` | BC-1.03.006 EC-007, DI-004 |
| E-DAT-008 | broken | 2 | `XLSX header cell at column <idx> in '<path>' has type <calamine-type> (value: <repr>). Header cells must be String-typed. Use a string label as the column header.` | BC-1.03.006 EC-008, DI-004 |
| E-DAT-009 | broken | 2 | `XLSX datetime cell at <col>:<row> in '<path>' has invalid ISO 8601 value '<value>'. Expected format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS±HH:MM` | BC-1.03.006 EC-009, DI-004 |
| E-DAT-010 | broken | 2 | `XLSX numeric cell at <col>:<row> in '<path>' has non-finite value (<NaN\|Infinity>). Non-finite floats are not representable in slideforge values.` | BC-1.03.006 EC-012 |
| E-DAT-011 | broken | 2 | `'<path>' has .<ext> extension but is not a valid XLSX archive (ZIP magic bytes not found). File may be corrupted or misnamed.` | BC-1.03.006 EC-013 |
| E-DAT-012 | broken | 2 | `TEXT column '<column_name>' at row <row_idx> in '<path>' contains invalid UTF-8 bytes. SQLite TEXT values must be valid UTF-8.` | BC-1.03.007 EC-008, DI-004 |
| E-DAT-013 | broken | 2 | `'<path>' has .<ext> extension but is not a valid SQLite database (SQLite file header not found). File may be corrupted or misnamed.` | BC-1.03.007 EC-009 |
| ~~E-DAT-014~~ | ~~retired~~ | — | ~~`Unsupported extension for SQLite data source: '<ext>'. Accepted extensions: .db, .sqlite, .sqlite3`~~ Retired in v1.8 — subsumed by E-DAT-003 at the dispatcher boundary (Pass-13 fix). Code constant retained in `slideforge-data` for SemVer compat only. See changelog v1.8. | BC-1.03.007 EC-010 (now E-DAT-003) |
| E-DAT-015 | broken | 2 | `[E-DAT-015] data source error for '<uri>': <plugin-message>` — unspecified error from a third-party DataSource plugin whose message does not embed a [E-DAT-NNN] bracket code. Plugin authors: embed [E-DAT-NNN] in error messages for precise routing. | CAP-003, BC-1.03.004 |

---

## Layout Errors (E-LAY)

Canvas overflow is a warning by default. Set `[build].strict_overflow = true` in
slideforge.toml to promote to a blocking error.

| Code | Severity | Exit (strict-overflow) | Message Format | Traces To |
|------|---------|----------------------|---------------|-----------|
| E-LAY-001 | degraded | 2 (if strict-overflow) | `CanvasOverflow: slide '<title>' field '<field>' overflows by ~<N> EMU (~<M>pt). Consider reducing content or font size.` | CAP-022, DEC-013 |
| E-LAY-002 | broken | 2 | `Zero-slide deck: no slide blocks found in '<file>'. A deck must contain at least one slide.` | CAP-022, DEC-011 |
| E-LAY-003 | degraded | 2 (if strict-overflow) | `Chart data is empty for slide '<title>'. Rendering error-slide placeholder.` | CAP-013, DEC-014 |
| E-LAY-004 | broken | 2 | `Shape at slide <source_slide_index> (<file>:<line>:<col>) has no alt text and is not marked decorative: true. Add alt "..." or decorative: true.` | BC-3.04.001 EC-001, DI-001, CAP-023 |
| E-LAY-005 | broken | 2 | `Inline nesting depth exceeded at slide <source_slide_index>: depth <depth> exceeds maximum of 64. Flatten the inline tree.` | BC-3.05.001 EC-006, CAP-024 |
| E-LAY-006 | broken | 2 | `Arithmetic overflow computing EMU for shape position at slide <source_slide_index> (<file>:<line>:<col>). Value <value> in <unit> exceeds i64 range after conversion. Use a value ≤ 9,007,199,254 inches (approximately 9.0 × 10⁹ in).` | BC-3.04.001 EC-014, EC-015, CAP-023 |
| E-LAY-007 | broken | 2 | `Bullet list structural nesting depth exceeds the limit (64) at slide <source_slide_index>. Reduce bullet list nesting depth.` | BC-3.05.001, CAP-024 |

Note (E-LAY-004): This is the layout-layer defensive check for missing alt text on
shapes. `LayoutError::MissingAlt` maps to E-LAY-004 in the CLI diagnostic renderer.
The variant carries a `span: SourceSpan` field (file/line/col). The primary enforcement
is E-A11-001 in the validation stage; E-LAY-004 fires only if the validation stage was
bypassed (internal invariant violation).

Note (E-LAY-004 field naming): The variant fields on `LayoutError::MissingAlt`,
`MissingRiskCardField`, `MalformedSeverityCards`, and `UnresolvedSeverityCards` were
renamed from `slide_index` to `source_slide_index` in STORY-028 pass-1 (data-engineer
schema burst, commit 9ea373a8) and the spec supplements were aligned in STORY-028
pass-7 sweep (commit e52eb3d8). The canonical field name is `source_slide_index`
throughout all `LayoutError` variants that identify a specific slide. See
`interface-definitions.md §8` for the full module naming contract.

Note (E-LAY-006): `LayoutError::ArithmeticOverflow { source_slide_index: usize, span: SourceSpan }`
maps to E-LAY-006. This error is produced when `ShapeUnit::from_inches` or `ShapeUnit::from_em`
detects i64 overflow during the milliunit-to-EMU multiplication (the `checked_mul` path).
Silent saturation via `saturating_mul` is FORBIDDEN — it would produce garbage coordinates
in the IR. The `ArithmeticOverflow` variant MUST have a load-bearing constructor path (it
is NOT dead code); any implementation that marks it `#[allow(dead_code)]` or documents it
as "future strict-mode validator" is in direct violation of BC-3.04.001 invariant 8 and
CLAUDE.md Rule 3 (no AI-added tech-debt register entries without explicit human direction).
Adjudicated in adversary pass 2 on STORY-028, item M (2026-05-29).

Note (E-LAY-007): `LayoutError::BulletDepthExceeded { source_slide_index: usize, depth: usize }`
maps to E-LAY-007. This error guards the STRUCTURAL bullet-nesting depth (i.e., the depth of the
`BulletItem.children` chain (the number of nested `children[..]` levels) in the authored .sf file).
It is DISTINCT from
E-LAY-005 (`InlineDepthExceeded`), which guards the INLINE tree depth within the `Vec<InlineNode>`
content of a single `BulletItem`. A document can have bullets nested many structural levels
deep (triggering E-LAY-007) while each individual bullet's inline content is shallow (no E-LAY-005),
and vice versa. The same 64-level cap applies to both surfaces (BC-3.05.001 invariant 4).
Allocated in STORY-073 adversary pass 2 (MED-2 finding). Owner story: STORY-073.

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
| E-BRD-001 | broken | 4 | Shared by three `BrandError` variants: (1) `FileNotFound` — `Brand file not found: '<resolved-path>'. Check the template path in slideforge.toml or --template flag.` (2) `LogoRequired` — `E-BRD-001: A logo path is required but was empty or absent. Provide a non-empty logo path — either the brand.toml [logo] 'path' (brand synthesis) or the brand_overlay: logo value (per-slide overlay).` (3) `TomlReadError` — `Cannot read brand.toml at '<path>': <reason>.` The `LogoRequired` subcase is user-reachable via per-slide `brand_overlay: logo ""` (empty string) or an absent logo key; made context-neutral in STORY-025. | CAP-018, FM-007 |
| E-BRD-002 | broken | 4 | Shared by two `BrandError` variants: (1) `ParseError` — `Cannot parse brand template '<path>': <detail>. File may be corrupted or not a valid PPTX/DOCX.` (2) `TomlParseError` — `Cannot parse brand.toml at '<path>': <detail>.` | CAP-018, FM-007 |
| E-BRD-003 | cosmetic | 0 | `Brand color slot '<slot>' inferred as #<hex> (derived from <source-color>). Review in brand.toml to confirm.` | DI-015, CAP-018, DEC-016 |
| E-BRD-004 | cosmetic | 0 | `Font '<font-name>' not available on this build host. Using '<fallback>' (panose: [<class>]). Text metrics may differ.` | CAP-018, FM-009 |
| E-BRD-005 | cosmetic | 0 | `Invalid hex color value '<value>' in brand.toml slot '<slot>'. Use 6-digit uppercase hex RGB (e.g. #3B82F6; case insensitive — uppercase or lowercase accepted).` | CAP-018, FM-007 |
| E-BRD-006 | broken | 4 | `<path>: brand.toml already exists. Use --force to overwrite.` | CAP-018 |
| E-BRD-007 | broken | 4 | `Logo path '<logo_path>' escapes the brand root directory '<brand_dir>'. The logo file must be inside (or beneath) the brand root directory.` Applies to any user-supplied logo path: master brand logo (in brand synthesis / brand.toml), and per-slide `brand_overlay: logo` paths. The canonical path check is performed after file-existence (E-BRD-001 fires first for non-existent paths). Maps to `BrandError::LogoOutsideBrandDir`. | CAP-018, CAP-019, FM-007 |

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

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-24 | product-owner | Initial creation — parse errors (E-PAR-001 through E-PAR-011), evaluation errors (E-EVL-001 through E-EVL-006), layout errors (E-LAY-001 through E-LAY-003), export errors (E-EXP-001 through E-EXP-007), brand errors (E-BRD-001 through E-BRD-005), package errors (E-PKG-001 through E-PKG-004), configuration errors (E-CFG-001 through E-CFG-008), accessibility errors (E-A11-001 through E-A11-004) |
| 1.1 | 2026-05-25 | product-owner | STORY-028 Pass-1 adjudication: E-PAR-012 retired (indentation-level subsumed by E-PAR-001); E-PAR-012-SHP added for unknown shape type; E-PAR-013 added for invalid hex color; E-PAR-014 added for gradient not supported in v1.0; E-LAY-004 added for MissingAlt; E-LAY-005 added for inline nesting depth exceeded |
| 1.2 | 2026-05-28 | product-owner | STORY-028 pass-5 sweep: E-LAY-006 added for ArithmeticOverflow; E-EXP-008 added for diagram render failure; E-DAT-006 through E-DAT-014 added for HTTP SSRF, XLSX header, SQLite data source errors; E-CFG-005 through E-CFG-008 added |
| 1.3 | 2026-05-29 | product-owner | Pass-7 sweep: E-LAY-004 and E-LAY-006 notes updated — source_slide_index canonical field name documented; E-BRD-007 added for logo path escaping brand directory; E-BRD-005 semantics note updated (original missing-slot fatal retired; reused for invalid hex in brand.toml) |
| 1.4 | 2026-05-29 | product-owner | Pass-8 sweep: E-LAY-006 note updated with ArithmeticOverflow dead-code prohibition rule per adversary pass 2 item M adjudication |
| 1.5 | 2026-05-29 | product-owner | Pass-9 sweep (F-P9-HIGH-002): E-PAR-013 (hex color invalid) and E-PAR-014 (gradient unsupported) renamed to E-PAR-015 and E-PAR-016 respectively to resolve namespace collision with parser template codes (E-PAR-013 = empty {{ }}, E-PAR-014 = unterminated math block, both pre-existing in slideforge-syntax/src/parser/template.rs). E-PAR-013 and E-PAR-014 now document their actual parser meaning. Implementer handoff: shape parsing code in STORY-028 worktree currently references E-PAR-013 only in doc comments (not in emitted string messages) — implementer must use E-PAR-015/E-PAR-016 codes in all emitted error messages for shape parsing. |
| 1.6 | 2026-05-30 | implementer | STORY-021 Pass-2 adversarial fix burst: E-DAT-015 added for unspecified data-source error (catch-all from third-party plugins without [E-DAT-NNN] bracket codes). E-DAT-015 replaces the incorrect E-DAT-004 routing that mis-categorized arbitrary plugin errors as file-not-found. DataError::AuthFailed variant added for correct AuthError mapping (no "Use --offline" hint). |
| 1.7 | 2026-05-30 | implementer | STORY-021 Pass-3 + Pass-4 adversarial fix bursts. Pass-3: `DataError::PolicyRejected` variant added (carries E-DAT-006 for body-cap and similar policy rejections, distinct from SSRF-domain blocks which use `SsrfBlocked`). Display format: `"[E-DAT-006] data policy rejected '<uri>': <message> (at <span>)"`. E-DAT-006 row expanded to document both semantic sub-cases (SSRF via `SsrfBlocked`; body-cap and policy via `PolicyRejected`). Pass-4: `E_DAT_006_POLICY` alias removed (was documentary only, carried no load-bearing semantic distinction — dispatcher now uses `E_DAT_006` directly at the single `PolicyRejected` construction site). Dispatcher E-DAT-004 routing arm updated to distinguish `FileNotFound` from `IoError` by inspecting the label prefix following the bracket code (F-P4-HIGH-001 fix): `"file not found: "` → `FileNotFound`; all other labels (`"I/O error reading '"`, `"failed to open file '"`, etc.) → `IoError`. |
| 1.8 | 2026-05-30 | implementer | STORY-021 Pass-14 adversarial fix burst. (1) E-DAT-014 retired — subsumed by E-DAT-003 at the dispatcher boundary (Pass-13 fix). SQLite extension errors now route through `DataError::UnsupportedFormat` (E-DAT-003). The `E_DAT_014` constant is retained in `slideforge-data` for SemVer compat only; no production path produces a `DataError` whose `.code()` returns `"E-DAT-014"`. E-DAT-014 row marked retired. (2) E-DAT-001 Display updated to spec-mandated format: `"[E-DAT-001] HTTP fetch failed: '<url>' returned HTTP <status>. Hint: use --offline to skip HTTP sources. (at <span>)"` — adds URL, "HTTP fetch failed" label, and `--offline` discovery hint. (3) E-DAT-002 Display updated to spec-mandated format: `"[E-DAT-002] network error fetching '<url>': <cause>. Use --offline to skip HTTP sources. (at <span>)"` — adds URL and `--offline` discovery hint. `AuthFailed` Display unchanged (no `--offline` hint, per Pass-5 F-P5-LOW-008). (4) `extract_http_status` in dispatcher.rs updated to use `rfind("[E-DAT-")` anchoring to prevent stray `]` in third-party plugin messages from misplacing the search region (F-P14-LOW-001 fix). |
| 1.9 | 2026-05-30 | product-owner | STORY-024 adversarial fix (spec-parity gap): E-BRD-006 added for `brand extract` output-exists guard. The Brand Errors table previously jumped from E-BRD-005 to E-BRD-007; E-BRD-006 was referenced by STORY-024 AC-002 and BC-2.01.003 EC-001 but had no taxonomy row. Traces to CAP-018. Implementer note: `BrandError::OutputExists { path: Arc<str> }` and constant `E_BRD_006` must be added to `crates/slideforge-brand/src/error.rs` as part of STORY-024 — neither exists in the codebase at this writing. |
| 2.0 | 2026-05-30 | product-owner | STORY-025 adversary finding F-025-001: E-BRD-007 description widened from synthesis-only to all user-supplied logo paths (master brand logo AND per-slide `brand_overlay: logo` paths). Message template updated from "brand.toml directory" to "brand root directory" for generality. CAP-019 added to Traces To column. Behavior at canonical path check ordering (E-BRD-001 fires first for non-existent paths) and `BrandError::LogoOutsideBrandDir` mapping made explicit. Exit code 4 and severity unchanged. |
| 2.1 | 2026-05-30 | product-owner | STORY-025 fix burst taxonomy-completeness gap: E-BRD-001 row expanded to document all three shared `BrandError` variants — `FileNotFound`, `LogoRequired`, and `TomlReadError`. The `LogoRequired` subcase was previously undocumented in the taxonomy; it is user-reachable via per-slide `brand_overlay: logo ""` (empty string) or absent logo key. The context-neutral `LogoRequired` message (`"E-BRD-001: A logo path is required but was empty or absent. Provide a non-empty logo path — either the brand.toml [logo] 'path' (brand synthesis) or the brand_overlay: logo value (per-slide overlay)."`) matches the message emitted by `crates/slideforge-brand/src/error.rs` after the STORY-025 context-neutral refactor. E-BRD-002 row similarly expanded to document its two shared variants (`ParseError` and `TomlParseError`). Code, severity (broken), and exit code (4) unchanged for both. |
| 2.2 | 2026-05-30 | product-owner | Variant-name correction: E-BRD-001 third variant corrected from `BrandReadError` → `TomlReadError`; E-BRD-002 second variant corrected from `BrandTomlParseError` → `TomlParseError`. Names now match the actual `BrandError` enum variants in `crates/slideforge-brand/src/error.rs` exactly. No semantic change — severity, exit code, message format, and Traces To columns are unchanged. |
| 2.3 | 2026-05-31 | product-owner | STORY-076 taxonomy decision note: srgbClr transform detection (BC-2.01.001 EC-006) does NOT introduce a new E-BRD-NNN error code. The `tracing::warn!` emitted when a srgbClr element has lumMod/lumOff/tint/shade children is an observability log event (structured field output to the tracing subscriber), not a user-facing diagnostic with an E-BRD code. The inline TOML comment written by BrandExtractor (`# derived via tint/shade; may not match exact color`) is the user-visible signal. This is consistent with the existing schemeClr transform treatment (also no error code — just a TOML comment). No new rows added to the Brand Errors table. |
| 2.4 | 2026-06-01 | product-owner | STORY-078 adversary pass 3 CRIT-A (E-PAR-015/016 collision) + CRIT-B (undocumented W-PAR-001). CRIT-A resolution: STORY-078 section-block parser code had reused E-PAR-015 and E-PAR-016 — those codes are already assigned to shape parsing (hex color and gradient respectively, per v1.5 changelog). Allocated fresh codes E-PAR-017 (section reserved-name collision, maps to BC-3.02.002 EC-006) and E-PAR-018 (section block inside slide/@for/@if body, maps to BC-3.02.002 EC-002 + STORY-078 AC-005). CRIT-B resolution: created new W-PAR (Parse Warnings) namespace — first entry W-PAR-001 (unrecognized section sub-block key, non-fatal, maps to BC-3.02.002 EC-005 + DIR-077-001-A Ruling 2 + STORY-078 AC-004). Existing E-PAR-015 (hex color) and E-PAR-016 (gradient) are UNCHANGED. |
| 2.5 | 2026-06-01 | product-owner | STORY-073 adversary pass 2 MED-2 — allocated E-LAY-007 for `BulletDepthExceeded` structural depth guard. E-LAY-007 covers `LayoutError::BulletDepthExceeded { source_slide_index, depth }` — the structural bullet-nesting depth cap (64 levels) distinct from E-LAY-005 (inline tree depth on `Vec<InlineNode>`). Traces to BC-3.05.001 and CAP-024. Owner story: STORY-073. |
| 2.6 | 2026-06-01 | product-owner | STORY-078 adversary pass 4 OBS-1 — W-PAR-001 message format corrected. Removed the "Did you mean one of: [<known-keys>]?" clause from the documented message: the parser does not emit a known-key suggestion list, so the clause was a code↔taxonomy divergence. The `at <file>:<line>:<col>` location token is retained (consistent with E-PAR namespace convention). W-PAR-001 note updated to state explicitly that no "Did you mean" suggestion is emitted. No other rows touched. |
| 2.7 | 2026-06-01 | product-owner | STORY-073 adversary pass 4 MED-1 — corrected E-LAY-007 explanatory note. Replaced the incorrect field reference "`BulletItem.depth` field which represents nested bullet levels" with "depth of the `BulletItem.children` chain (the number of nested `children[..]` levels)" — `BulletItem` has no `depth` field; structural depth is the children-chain length. The E-LAY-007 message-format row is unchanged. |
| 2.8 | 2026-06-01 | product-owner | STORY-078 adversary pass 6 MED-1 — E-PAR-018 context reconciled to `slide` / `@for/@if` composite. The Note previously claimed three emittable values (`slide`, `@for`, `@if`/`@elif`/`@else`); the parser architecturally emits only two: `slide` (slide bodies) and `@for/@if` (control-flow bodies via the shared `block_item` combinator). The composite is the intended behavior — splitting the shared combinator to distinguish `@for` from `@if` would be disproportionate for the marginal UX gain. Note rewritten to document the two actual values and explain the composite as designed, not a limitation. Message-format row (`found inside <context> block`) is unchanged. No other rows touched. |
| 2.9 | 2026-06-02 | product-owner | STORY-077 inline-markup parser (DIR-077-002 / F-077-P3-001): E-EVL-012 (`FigrefInvalidArg`) and E-EVL-013 (`InlineXrefEmptyId`) registered. E-EVL-012: `figref()` called with missing or non-evaluable argument in inline-markup section sub-block context — no figure-number `InlineNode` produced. E-EVL-013: `ref("")` / `"" | ref` given empty cross-reference id string in inline-markup context — no `InlineNode::Xref` produced. Both are eval-stage errors (`broken`, exit 2), produced by `chunks_to_inline_nodes` in `slideforge-eval`. Codes E-EVL-012/013 confirmed free — no collision with existing E-EVL-001 through E-EVL-011. Note appended documenting the taxonomy debt for E-EVL-007 through E-EVL-011 (allocated in code but not yet registered). E-PAR-015 (hex color invalid) and E-PAR-016 (gradient unsupported) confirmed already registered since v1.5 — no new E-PAR rows added for inline markup parse errors (those remain E-PAR-NNN placeholders in DIR-077-002 §5 pending a separate registration task). |
| 2.10 | 2026-06-02 | product-owner | STORY-077 inline-markup parser — E-PAR-019 and E-PAR-020 registered for parse-stage inline-markup errors in section sub-block field values. E-PAR-019 (`UnclosedInlineDelimiter`): opening delimiter (e.g., `**`, `__`, `` ` ``) with no matching close before end-of-field; span on opening delimiter; accumulated, strict-build-fatal (exit 1). E-PAR-020 (`EmptyInlineSpan`): delimiter pair with no content between them (e.g., `****`, `__`); span on opening delimiter; accumulated, strict-build-fatal (exit 1). E-PAR-017/018 confirmed assigned (STORY-078 section-block parser, v2.4 changelog) — E-PAR-019/020 confirmed free prior to this registration. E-PAR-015/016 collision that prompted the STORY-078 allocation (v2.4) served as precedent for this pre-registration check. Traces to BC-3.02.002, DIR-077-002 §5, STORY-077. |
