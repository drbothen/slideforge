---
document_type: prd-supplement
supplement_type: error-taxonomy
level: L3
version: "2.32"
status: active
producer: product-owner
timestamp: 2026-06-11T00:00:00
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
| E-PAR-012 | broken | 1 | `E-PAR-012: unterminated {{ interpolation — missing }} to close the expression at <file>:<line>:<col>.` | CAP-001 |
| E-PAR-012-SHP | broken | 1 | `Unknown shape type '<keyword>' at <file>:<line>:<col>. Known types: [rect, ellipse, arrow, line, star, roundRect].` | CAP-023 |
| E-PAR-013 | broken | 1 | `Empty expression in {{ }} at <file>:<line>:<col>. An expression is required between {{ and }}.` | CAP-001 |
| E-PAR-014 | broken | 1 | `Unterminated math block at <file>:<line>:<col>. Missing closing $ (or $$).` | CAP-012 |
| E-PAR-015 | broken | 1 | `Invalid hex color '<value>' at <file>:<line>:<col>. Expected 6-digit hex (#RRGGBB). Short-form #RGB and alpha #RRGGBBAA are not supported.` | CAP-023 |
| E-PAR-016 | broken | 1 | `Shape gradient fill is not supported in v1.0 at <file>:<line>:<col>. Use a solid hex color or 'none'. Gradient fill is planned for a future release (STORY-072).` | CAP-023 |
| E-PAR-017 | broken | 1 | `Section sub-block key '<name>' is a reserved register name — use '<name>:' register syntax or choose a different key.` | BC-3.02.002 EC-006, CAP-001 |
| E-PAR-018 | broken | 1 | `section blocks must be top-level — found inside <context> block at <file>:<line>:<col>. Move the section: declaration to the top level of the .sf file.` | BC-3.02.002 EC-002, CAP-001 |
| E-PAR-019 | broken | 1 | `Unclosed inline-markup delimiter '<delim>' at <file>:<line>:<col>. Add a matching closing '<delim>' or escape the opening delimiter.` | BC-3.02.002, DIR-077-002 §5, CAP-001 |
| E-PAR-020 | broken | 1 | `Empty inline-markup span '<delim><delim>' at <file>:<line>:<col>. Remove the empty delimiter pair or add content between the delimiters.` | BC-3.02.002, DIR-077-002 §5, CAP-001 |
| E-PAR-021 | broken | 1 | `Inline-markup nesting depth exceeded at <file>:<line>:<col>: depth <N> exceeds maximum of <max>. Flatten or reduce nested inline markup.` | BC-3.02.002, DIR-077-002 §5, CAP-001, STORY-077 |
| E-PAR-022 | broken | 1 | `Link URL scheme not permitted at <file>:<line>:<col>: '<scheme>'. Allowed: http, https, mailto.` | BC-3.02.002, DIR-077-002 §5, CAP-001 |
| E-PAR-023 | broken | 1 | `E-PAR-023: section group name must be non-empty at <file>:<line>:<col>. Provide a quoted, non-empty name, e.g. section "Background":` | BC-4.01.003 EC-010, CAP-001 |
| E-PAR-024 | broken | 1 | `non-string list item at <file>:<line>:<col>. List items must be quoted string literals; got <type>. Wrap the value in quotes to use it as a string.` | BC-1.01.002, CAP-001, STORY-088 |

Note (E-PAR-023): `ParseError::EmptySectionGroupName` — emitted when the parser encounters a `section "":` slide-grouping block whose quoted-name token is an empty string. An empty section name is rejected at parse time (before IR or GUID derivation occurs), not at eval or validation time: the parser can determine emptiness immediately after consuming the quoted-string token. Severity: broken. Exit 1 in strict mode (consistent with all other E-PAR parse errors per BC-1.15.003 three-tier model: parse errors → exit 1; strict-mode validation errors → exit 2). Error is accumulated (parsing continues past this point to find additional errors). No `SectionGroupNode` is produced for the rejected block. The `<file>:<line>:<col>` span points at the opening `"` of the empty name token. The emitted message includes the `E-PAR-023:` code prefix (matching the emitter pattern used across all E-PAR codes that embed their code string). Variant name: `ParseError::EmptySectionGroupName`. Owner story: STORY-082. Traces to BC-4.01.003 EC-010, CAP-001.

Note (E-PAR-024): `ParseError::NonStringListItem` — emitted when the list-literal parser (`bullets: ["A", "B"]` or `@var x = ["A"]`) encounters a list item that is not a quoted string literal. The parser accepts only double-quoted string literals as list items in v1.0; unquoted values (bare words, integers, booleans, color names, etc.) are rejected at parse time. The `<type>` placeholder is substituted with a human-readable description of the actual token kind encountered — e.g., `"integer"`, `"boolean"`, `"bare word"`, `"color name"`. The `<file>:<line>:<col>` span points at the non-string token itself. Severity: broken. Exit 1 in strict mode (same exit code as all other E-PAR syntax errors). Error is accumulated (parsing continues past this item to find additional errors in the same list and file). No `FieldValue::List` item is produced for the rejected element; the overall list may still be partially constructed if other items are valid strings. Applies to both the inline `bullets: [...]` form and deck-level `@var ident = [...]` assignments. This code MUST be used instead of E-PAR-015 (which is reserved for invalid hex color values in shape parsing) — the two contexts are structurally and semantically distinct. Variant name suggestion: `ParseError::NonStringListItem`. Owner story: STORY-088. Traces to BC-1.01.002, CAP-001.

Note (E-PAR-017): Fires when a key inside a `section:` sub-block collides with a reserved register name (e.g., `notes`, `report`, `detail`). The parser rejects this at parse time to prevent silent shadowing of writing-register syntax. The `<name>` placeholder contains the colliding key name. Maps to STORY-078 (section block parser) and BC-3.02.002 EC-006 (section reserved-name collision). Owner story: STORY-078.

Note (E-PAR-018): The `<context>` placeholder is substituted with one of two values at the error site: `slide` (when the section block appears inside a slide body) or `@for/@if` (when the section block appears inside a control-flow body). The composite `@for/@if` value is the intended behavior — the shared `block_item` combinator that parses both `@for` and `@if` bodies cannot statically distinguish the two control-flow forms without splitting the combinator; reporting the composite accurately tells the user "you are inside a control-flow block" without requiring disproportionate parser restructuring. All contexts are prohibited — section blocks must appear at top-level only. Maps to BC-3.02.002 EC-002 and STORY-078 AC-005.

Note (E-PAR-012): Re-registered in v2.13. This code was previously marked RETIRED (indentation-level subsumed by E-PAR-001) in v1.1 (STORY-028 Pass-1 adjudication). However, the live production code in `crates/slideforge-syntax/src/parser/template.rs` (`unterminated_interpolation_msg()`) has always emitted the string `"E-PAR-012"` for unterminated `{{ }}` interpolation — users have been seeing this code in the wild. Per CLAUDE.md Source-of-Truth Precedence (spec wins, but when the spec itself is being authored or corrected, live code is the empirical ground truth for user-visible codes). The retirement note was incorrect: E-PAR-012 was never actually repurposed for indentation — E-PAR-001 subsumed the indentation-level diagnostic, but E-PAR-012 remained in the parser for unterminated `{{`. Re-registered here with its factual meaning. E-PAR-012-SHP (unknown shape type) retains its suffix notation as an informal disambiguation; no code change is required anywhere. Variant: `TemplateErrorKind::UnterminatedInterpolation` → emits `"E-PAR-012"`. No implementer action required — the code string in `unterminated_interpolation_msg()` is already correct.

Note (E-PAR-019): `ParseError::UnclosedInlineDelimiter` — emitted when the inline-markup parser encounters an opening delimiter (e.g., `**`, `__`, `` ` ``) inside a section sub-block field value with no matching closing delimiter before end-of-field or end-of-block. The span points at the opening delimiter character(s). Error is accumulated (does not halt further parsing in the block) and is strict-build-fatal (exit 1). The `<delim>` placeholder is substituted with the actual delimiter token (e.g., `**`, `__`, `` ` ``). Applicable to all inline-markup spans in section sub-block field values (`title:`, `detail:`, `report:`, etc.). Owner story: STORY-077. Traces to BC-3.02.002 and DIR-077-002 §5.

Note (E-PAR-020): `ParseError::EmptyInlineSpan` — emitted when the inline-markup parser encounters a delimiter pair with no content between them (e.g., `****`, `__`, ` `` `). The span points at the opening delimiter. Error is accumulated and strict-build-fatal (exit 1). The `<delim><delim>` placeholder in the message is substituted with the full empty-span token as it appears in source (e.g., `****`, `__`). Distinct from E-PAR-019 (unclosed delimiter, where the closing delimiter is simply absent) — E-PAR-020 fires when both delimiters are present but no content appears between them. Owner story: STORY-077. Traces to BC-3.02.002 and DIR-077-002 §5.

Note (E-PAR-021): `ParseError::InlineNestingDepthExceeded` — emitted when `scan_template_chunks` (in `crates/slideforge-syntax/src/parser/template.rs`) detects that the recursive inline-markup nesting depth has reached `MAX_INLINE_NESTING`. This fires at **parse time**, before any IR is produced, distinguishing it from E-LAY-005 (`LayoutError::InlineDepthExceeded`) which guards the SAME conceptual depth limit but at layout time on the already-constructed `Vec<InlineNode>` IR. The parse-stage guard is required to prevent adversarial inputs (e.g., thousands of alternating openers such as `^_^_^_...`) from exhausting the call stack via unbounded recursion in `scan_template_chunks`. When the depth cap is exceeded: (1) the error is accumulated (parsing does NOT halt — the error joins the accumulated diagnostic list), (2) the remainder of the field value beyond the cap depth is treated as literal text (no silent fallback — the literal treatment is in addition to the emitted diagnostic, not instead of it), and (3) the build exits with code 1 (strict-build-fatal) once all errors are reported, consistent with all other E-PAR errors. The `<N>` placeholder is the observed nesting depth at the point of rejection; `<max>` is the configured `MAX_INLINE_NESTING` constant. The `<file>:<line>:<col>` span points at the opening delimiter of the construct that pushed the depth over the cap. This variant is necessary for Phase-6 Kani bounded-model checking: without a finite depth cap enforced at parse time, the `scan_template_chunks` call graph is not provably terminating under adversarial input, which invalidates the Kani proof for the parser's error-accumulation property. Owner story: STORY-077. Traces to BC-3.02.002, DIR-077-002 §5, CAP-001.

Note (E-PAR-022): `ParseError::DisallowedLinkUrlScheme` — emitted by `scan_template_chunks` in `crates/slideforge-syntax/src/parser/template.rs` when the URL in a `[text](url)` link construct has a scheme that is not on the explicit allowlist. The span points at the opening `[` of the link. The error is accumulated (does not halt parsing — remaining field content continues to be parsed) and is strict-build-fatal (exit 1), consistent with all other E-PAR errors (routed through the same fatal path as E-PAR-019/020/021).

**Allowlist (v1.0 DECISION):** `http`, `https`, `mailto`. Everything else is denied by default. Specifically denied: `javascript:`, `data:`, `vbscript:`, `file:`, and any unrecognized scheme.

**Relative/anchor link decision:** Relative path links (e.g., `page.sf`, `./assets/doc.pdf`) and anchor-only links (e.g., `#section`) are NOT allowed in v1.0. Rationale: (1) Internal cross-references are handled by `{{ ref("id") }}` / `InlineNode::Xref` — this is the canonical v1 mechanism for intra-deck navigation. (2) Relative paths have no defined resolution semantics across the five output formats (PPTX, DOCX, PDF, HTML, web preview): a relative path valid in HTML becomes meaningless in PPTX or PDF. (3) Anchor-only links (`#section`) are output-format-specific and have no portable semantics across the export pipeline. Deny-by-default ensures no scheme silently slips through (SEC-002 finding mandate). If a URL has no scheme at all (no `:` in the URL string), it is rejected as an unrecognized scheme.

**Scheme extraction:** The scheme is the substring before the first `:`. If no `:` is present in the URL, the URL has no scheme and is rejected. If the scheme is not one of `http`, `https`, `mailto` (case-insensitive comparison), E-PAR-022 is emitted.

**Message format:** `Link URL scheme not permitted at <file>:<line>:<col>: '<scheme>'. Allowed: http, https, mailto.` The `<scheme>` placeholder is the actual scheme string extracted from the URL (e.g., `javascript`, `data`, `file`). For scheme-less URLs, the `<scheme>` placeholder is `(none)`.

Variant name suggestion: `ParseError::DisallowedLinkUrlScheme`. Owner story: STORY-077 SEC-002. Traces to BC-3.02.002, DIR-077-002 §5, CAP-001.

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
| W-PAR-002 | cosmetic | 0 | `warning: [W-PAR-002] Duplicate section group name '<name>' at <file>:<line>:<col>. Both sections are emitted with the same GUID. Consider using distinct names.` | BC-4.01.003 EC-011, CAP-001 |

Note (W-PAR-001): Emitted when the section block parser encounters a sub-block key that is
neither a known layout key, a reserved register name (which triggers E-PAR-017), nor a
`<name>:` register block. Per DIR-077-001-A Ruling 2 (unknown section keys are ignored with
a warning, not rejected). No "Did you mean" suggestion is emitted — the parser does not
enumerate known keys in this message. Maps to BC-3.02.002 EC-005 / invariant 4
/ STORY-078 AC-004.

Note (W-PAR-002): `ParseWarning::DuplicateSectionGroupName` — emitted when two or more `section "Name":` slide-grouping blocks in the same file share an identical quoted name. Severity: cosmetic. Exit 0 (warning does not block the build). Behavior: ALL sections with the duplicate name are emitted to the IR and included in `LaidOutDeck.slide_sections`; each is assigned the same deterministic GUID (because GUIDs are derived from section name via `sha2` hash — identical names yield identical GUIDs, per BC-4.01.003 invariant 3). No crash; no section is dropped. The warning is accumulated alongside parse errors but does NOT increment the error counter for exit-code calculation. Routed to `stderr` with a `warning:` prefix. The `<name>` placeholder is the duplicated quoted name as it appears in source; the `<file>:<line>:<col>` span points at the second (and subsequent) occurrence(s). This is a parse-time detection: the parser maintains a `seen_names: HashSet<Arc<str>>` while consuming slide-grouping blocks in order and emits W-PAR-002 on each collision. Variant name suggestion: `ParseWarning::DuplicateSectionGroupName`. Owner story: STORY-082. Traces to BC-4.01.003 EC-011, CAP-001.

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
| E-EVL-007 | broken | 2 | `slide count <count> exceeds the configured maximum of <max> at <file>:<line>:<col>` | CAP-004 |
| E-EVL-008 | broken | 2 | `cannot iterate over '<value_type>' value at <file>:<line>:<col>: @for requires a list` | CAP-004 |
| E-EVL-009 | degraded | 0 | `Large iteration: @for over <count> items may produce a very large deck. Consider filtering data at source.` | CAP-004 |
| E-EVL-010 | broken | 2 | `Unknown section type '<name>'. Known types: [<known_types>]` | BC-3.02.002, CAP-001 |
| E-EVL-011 | broken | 2 | `built-in call '<func>(...)' cannot be used in expression context at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002 |
| E-EVL-012 | broken | 2 | `figref() requires a figure-number argument at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002 |
| E-EVL-013 | broken | 2 | `ref() requires a non-empty id string at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002 |
| E-EVL-014 | broken | 2 | `footnote() requires a text argument at <file>:<line>:<col>` | CAP-002, CAP-024, BC-3.02.002, DIR-077-002 §5, STORY-077 |

Note (E-EVL-012): `EvalError::FigrefInvalidArg` — `figref()` was called with a missing or non-evaluable argument in an inline-markup context (section sub-block `detail:` or `report:` field). `figref(N)` requires exactly one argument that evaluates to a numeric or string figure number. If no argument is supplied, or the argument cannot be evaluated, no `InlineNode` is produced and this error is emitted. Distinct from E-EVL-011 (`UnsupportedBuiltinCall`), which covers `figref()` in non-inline expression contexts. Eval-stage error raised by `chunks_to_inline_nodes`. Allocated in STORY-077 worktree (DIR-077-002 / F-077-P3-001). Help text: "Use figref(N) where N is the figure number, e.g. figref(3) inside a {{ }} interpolation in a section detail:/report: field."

Note (E-EVL-013): `EvalError::InlineXrefEmptyId` — emitted when `ref()` produces no usable cross-reference id string in an inline-markup context (section sub-block `detail:` or `report:` field). Three conditions all yield E-EVL-013, matching the figref precedent (one code covers no-arg, bad-arg, and eval-to-empty): (1) `ref()` — zero arguments supplied, no id string to evaluate; (2) `ref("")` / `"" | ref` — argument present but evaluates to an empty string; (3) `ref(expr)` — argument present but `expr` evaluates to empty (e.g., a variable holding `""`). All three conditions are detected at eval time (after expression interpolation), not at parse time — hence the E-EVL prefix. No `InlineNode::Xref` is produced in any of the three cases; error accumulation continues. Allocated in STORY-077 worktree (DIR-077-002 / F-077-P3-001); scope broadened to cover zero-arg and eval-to-empty in STORY-077 adversary pass-9 (F-077-P9-001). Help text: "Provide a non-empty slide or figure id, e.g. ref(\"slide-1\") inside a {{ }} interpolation in a section detail:/report: field."

Note (E-EVL-014): `EvalError::FootnoteInvalidArg` — emitted when `footnote()` is called with a missing or unusable text argument in an inline-markup context (section sub-block `detail:` or `report:` field). Two conditions yield E-EVL-014, mirroring the `FigrefInvalidArg` pattern: (1) `footnote()` — zero arguments supplied, no text to emit; (2) `footnote(expr)` — argument present but `expr` evaluates to empty or non-string text (e.g., `footnote("")`). Both conditions are detected at eval time by `chunks_to_inline_nodes` in `slideforge-eval`. No `InlineNode::Footnote` is produced in either case; error accumulation continues (eval does NOT halt). Eval-stage error (`broken`, exit 2 in strict mode). Variant name: `EvalError::FootnoteInvalidArg`. Registered in STORY-077 adversary pass-9 (F-077-P9-001); E-EVL-014 confirmed free in taxonomy (E-EVL-001 through E-EVL-013 fully accounted for) and confirmed absent from `crates/slideforge-eval/src/error.rs` at time of registration. Traces to CAP-002, CAP-024, BC-3.02.002, DIR-077-002 §5, STORY-077. Help text: "Provide a non-empty text string, e.g. footnote(\"your note text\") inside a {{ }} interpolation in a section detail:/report: field."

Note (E-EVL-007): `EvalError::TooManySlides { count, max, span }` — emitted when the `@for` loop (or the overall deck) would produce more slides than the configured `EvalConfig::max_total_slides` hard cap. Only produced when `max_total_slides` is `Some(n)` and the limit is exceeded; not produced when the limit is `None`. Severity: broken (fatal in strict mode). Exit 2 in strict mode; in `--warn-only` mode: error-slide placeholder rendered. Help text: "Reduce the collection size, or increase max_total_slides in EvalConfig." Registered in v2.13 (was allocated in `crates/slideforge-eval/src/error.rs` but not formally registered in taxonomy — taxonomy debt note removed). Traces to CAP-004.

Note (E-EVL-008): `EvalError::NotIterable { value_type, span }` — emitted when the expression in `@for x in <expr>` evaluates to a value that is not iterable. Per BC-2.04.001: `List` and `Map` values are iterable; scalars (`Int`, `Float`, `Bool`, `Str`) and `Null` are rejected with this error. For maps, each entry is bound as a two-field map `{ key, value }` inside the loop body. Severity: broken (fatal in strict mode). Exit 2 in strict mode. Help text: "Wrap the value in a list literal or ensure the variable holds a list." Registered in v2.13. Traces to CAP-004.

Note (E-EVL-009): `EvalError::LargeDeckWarning { count, threshold, span }` — a **warning**, not an error. Emitted when the generated slide count exceeds `EvalConfig::large_deck_warn_threshold`. Evaluation continues normally. Severity: degraded. Exit 0 (warning only). Distinguished from `TooManySlides` (E-EVL-007, which is a hard error) by the soft message and zero exit. In the table above, severity is `degraded` and exit is `0` — the degraded classification reflects that output IS produced but the deck is larger than expected. Help text: "Consider splitting the deck into multiple files, or increase large_deck_warn_threshold in EvalConfig." Registered in v2.13. Traces to CAP-004.

Note (E-EVL-010): `EvalError::UnknownSectionType { name, known_types, span }` — emitted when an unrecognised section type name is encountered during evaluation (per BC-3.02.002 invariant 3 / DIR-077-001-A Ruling 3). The parser stores the section type name verbatim in `SectionNode.kind`; the evaluator (`eval_section_nodes`) validates it against the full `SectionType` plugin registry (built-ins per `CANONICAL_MANUAL_SECTION_TYPES` in `slideforge-types`: `executive_summary`, `risk_register`, `methodology`, `scope`, `approval`, `appendix`, `glossary` — plus any plugin-registered types). An unrecognised name is a FATAL eval error. Severity: broken. Exit 2 in strict mode. Help text: "Check the section type name for typos, or register a custom SectionType plugin." Registered in v2.13. Traces to BC-3.02.002, CAP-001.

Note (E-EVL-011): `EvalError::UnsupportedBuiltinCall { func, span }` — emitted when a built-in pseudo-function call (`ref`, `footnote`, `figref`) is used in a context where it cannot be evaluated to a general `Value`. `Expr::Call` nodes are recognised in inline-markup context by `chunks_to_inline_nodes` (which converts them to `InlineNode::Xref` / `Footnote`). When `eval_expr` encounters a `Call` node in any other context (e.g., a `vars:` binding, an `@if` condition), it emits this error — there is no runtime value to return for a cross-reference function (Q1 decision: no user-callable functions in v1). Distinct from E-EVL-012 (`FigrefInvalidArg`) and E-EVL-013 (`InlineXrefEmptyId`), which cover malformed arguments to these functions in inline-markup context. Help text: "Built-in functions ref(), footnote(), and figref() are only valid inside {{ }} interpolations in section detail:/report: fields." Registered in v2.13. Traces to CAP-002, CAP-024, BC-3.02.002.

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
| E-LAY-003 | broken | 2 (strict, default) / 0 (--warn-only) | `[E-LAY-003] Chart data is empty for slide '<title>'. Rendering error-slide placeholder.` | CAP-013, DEC-014, BC-1.11.002, STORY-098, F-098-P1-003 |
| E-LAY-004 | broken | 2 | `Shape at slide <source_slide_index> (<file>:<line>:<col>) has no alt text and is not marked decorative: true. Add alt "..." or decorative: true.` | BC-3.04.001 EC-001, DI-001, CAP-023 |
| E-LAY-005 | broken | 2 | `Inline nesting depth exceeded at slide <source_slide_index>: depth <depth> exceeds maximum of 64. Flatten the inline tree.` | BC-3.05.001 EC-006, CAP-024 |
| E-LAY-006 | broken | 2 | `Arithmetic overflow computing EMU for shape position at slide <source_slide_index> (<file>:<line>:<col>). Value <value> in <unit> exceeds i64 range after conversion. Use a value ≤ 9,007,199,254 inches (approximately 9.0 × 10⁹ in).` | BC-3.04.001 EC-014, EC-015, CAP-023 |
| E-LAY-007 | broken | 2 | `Bullet list structural nesting depth exceeds the limit (64) at slide <source_slide_index>. Reduce bullet list nesting depth.` | BC-3.05.001, CAP-024 |
| E-LAY-008 | broken | 2 | `[E-LAY-008] Slide '<slide_type>' at <file>:<line>:<col> has no content region for 'bullets'. Slide type '<slide_type>' defines no Body or Generic Empty region. Use a slide type with a body region (e.g. 'content', 'detail', 'bullets_only') or remove the 'bullets:' field.` | CAP-022, STORY-094, F-094-P2-003 |

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

Note (E-LAY-003): **Reclassified v2.31 (F-098-P1-003 + F-098-P1-007 adjudication).** E-LAY-003 is
`broken`, NOT `degraded`. It is NOT gated on `[build].strict_overflow` — it is gated on `--warn-only`.
The `strict-overflow` gate applies ONLY to canvas-overflow conditions (E-LAY-001). Empty or missing
chart data is a data-integrity failure, not an overflow condition.

**Trigger conditions (both trigger E-LAY-003):**
1. A `slide chart:` block has a `data:` binding that evaluates to an empty collection (zero rows,
   empty list) — the empty-evaluating-binding case (BC-1.11.002 original scope).
2. A `slide chart:` block has NO `data:` field at all (missing data source) — the missing-data case
   (F-098-P1-007 adjudication). The REND-010 defect (chart with no data renders silently empty)
   is in this case. Both are equally broken: the chart cannot render meaningful content.

**Mode semantics:**
- Strict mode (default): E-LAY-003 is `broken`; build exits 2; no output file is written.
- `--warn-only` mode: E-LAY-003 emitted as warning; error-slide placeholder rendered at the chart
  slide's position; build continues; exit 0.

**Layout-preview note:** The `chart.rs` doc comment that permits data-less charts for "layout previews
and @data runtime references" without error is INCORRECT as of v2.31. Layout previews MUST use
`--warn-only` to suppress E-LAY-003. The implementer (STORY-098) must correct the `chart.rs` doc
comment to remove or update this allowance.

**ChartRenderer guard:** The ChartRenderer plugin is never called with empty or absent data — the
validator intercepts at Stage 5 (pre-layout). Traces to BC-1.11.002, DI-017, DI-018.

Note (E-LAY-007): `LayoutError::BulletDepthExceeded { source_slide_index: usize, depth: usize }`
maps to E-LAY-007. This error guards the STRUCTURAL bullet-nesting depth (i.e., the depth of the
`BulletItem.children` chain (the number of nested `children[..]` levels) in the authored .sf file).
It is DISTINCT from
E-LAY-005 (`InlineDepthExceeded`), which guards the INLINE tree depth within the `Vec<InlineNode>`
content of a single `BulletItem`. A document can have bullets nested many structural levels
deep (triggering E-LAY-007) while each individual bullet's inline content is shallow (no E-LAY-005),
and vice versa. The same 64-level cap applies to both surfaces (BC-3.05.001 invariant 4).
Allocated in STORY-073 adversary pass 2 (MED-2 finding). Owner story: STORY-073.

Note (E-LAY-008): `LayoutError::BulletsOnContentlessSlideType { slide_type: Arc<str>, source_slide_index: usize, span: SourceSpan }` maps to E-LAY-008.

**Classification: User-authoring error.** This is NOT an internal layout invariant breach (which would be `LayoutError::InvalidBoundingBox` or similar). E-LAY-008 fires when the layout pass determines that the authored `bullets:` field cannot be placed because the slide type's region map defines no region with a Body or Generic Empty role. Examples of content-less slide types: `title`, `closing`, `section_break`, `blank`. Slide types that DO have a content region include `content`, `detail`, `bullets_only`, and most body-content types.

**Trigger condition:** The layout pass iterates each slide's region map looking for a candidate region to receive bullet content. If no region carries a Body role or Generic Empty role, and the slide has a populated `bullets:` field, this error fires instead of producing a sentinel or garbage bounding box. The sentinel bbox path (`LayoutError::InvalidBoundingBox`) remains reserved for INTERNAL invariant violations where a region exists but its geometry is degenerate — it is never the correct path for an author-visible content-placement failure.

**Source span:** The `span: SourceSpan` field points at the first character of the `bullets:` keyword in the authored .sf file (file, line, col). This is the same span convention used by E-LAY-004 (`MissingAlt`) and E-LAY-006 (`ArithmeticOverflow`) — the span travels from the IR through the layout pass and is rendered by miette.

**Strict mode vs. warn-only:** In strict mode (default): broken, exit 2, no output written. In `--warn-only` mode: error-slide placeholder rendered at the affected slide position; build continues; exit 0 (unless other fatal errors are also present).

**Error accumulation:** E-LAY-008 is accumulated alongside other layout diagnostics (DI-018). If multiple slides have this problem, all instances are reported in a single build pass before the exit-code gate fires.

**Correction hint (embedded in message):** "Use a slide type with a body region (e.g. `content`, `detail`, `bullets_only`) or remove the `bullets:` field." Implementers must include this hint verbatim in the emitted message so miette can render it in the `help:` line. The `[E-LAY-008]` bracket code prefix in the message is required (consistent with E-PAR-023, W-A11-002, and other codes that self-identify in their message string).

**Distinction from E-VAL-101/W-VAL-103:** E-VAL-101 fires when a required field is ABSENT (Stage 5, validate_fields). W-VAL-103 fires when an UNKNOWN field is present (Stage 5, validate_fields). E-LAY-008 fires at LAYOUT TIME (Stage 6) when the field is present and known — `bullets:` is a valid field for many slide types — but the SLIDE TYPE has no region to receive the content. This is a placement-time conflict, not a schema validation failure. These three errors are complementary and non-overlapping.

**Variant name:** `LayoutError::BulletsOnContentlessSlideType`. Allocated in STORY-094 F-094-P2-003 fix burst (2026-06-11). Traces to CAP-022.

---

## Export Errors (E-EXP)

Always fatal (exit 3). Partial output is never written to disk (all-or-nothing).

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| E-EXP-001 | broken | 3 | `PPTX serialization error at OOXML element '<element-path>': <detail>` | CAP-015, FM-010 |
| E-EXP-002 | broken | 3 | `DOCX serialization error: <detail>` | CAP-016 |
| E-EXP-003 | broken | 3 | `PDF export error: <detail>. Verify pdf-writer + krilla stack.` Includes `PdfExportError::InvalidXmpTitle` (SEC-050-001) — see note below. | CAP-017, FM-011 |
| E-EXP-004 | broken | 3 | `SVG normalization failed for diagram '<title>': <usvg-error>` | CAP-014 |
| E-EXP-005 | broken | 3 | `Chart render failed for slide '<title>': <plotters-error>` | CAP-013, FM-013 |
| E-EXP-006 | broken | 3 | `Math render failed for expression at <file>:<line>:<col>: <detail>. See supported LaTeX subset in DSL reference.` | CAP-012, FM-012 |
| E-EXP-007 | broken | 3 | `Cannot write output to '<path>': <os-error>. Check directory exists and is writable.` | FM-015 |
| E-EXP-008 | broken | 3 | `Diagram render failed for slide '<title>': <mermaid-error> at source line <N>` | CAP-014, FM-014 |

Note (E-EXP-004, `DiagramError::SvgNormalizationFailed`, SEC-001/SEC-002): E-EXP-004 is emitted by TWO production call sites — both `slideforge-diagrams` and `slideforge-pdf`. The error code is shared because both sites represent the same logical failure class (SVG normalization failure before export). The `cause` field in the respective error variants carries distinguishing detail. The three sub-cases and their call sites are:

(1) **Malformed SVG (usvg parse failure)** — both `crates/slideforge-diagrams/src/normalize.rs` (`DiagramError::SvgNormalizationFailed`) and `crates/slideforge-pdf/src/svg_embed.rs` (`PdfExportError::SvgEmbed`) emit E-EXP-004 when `usvg::Tree::from_str` returns an error. Cause: the usvg error message.

(2) **SEC-002 (CWE-400, byte-size cap exceeded, 50 MiB limit)** — `crates/slideforge-diagrams/src/normalize.rs` and `crates/slideforge-pdf/src/svg_embed.rs` both check `svg_str.len() > MAX_SVG_BYTES` (52,428,800) BEFORE calling `usvg::Tree::from_str`. Cause: `"SVG input too large: N bytes exceeds the 52428800-byte limit"`. The guard must fire before parse to prevent resource exhaustion.

(3) **SEC-001 (CWE-674, nesting-depth cap exceeded, 64-level limit)** — `crates/slideforge-diagrams/src/normalize.rs` and `crates/slideforge-pdf/src/svg_embed.rs` both scan the parsed `usvg::Tree` for `<g>` group nesting depth > `MAX_SVG_NESTING_DEPTH` (64). The two crates differ in their traversal approach: `slideforge-diagrams` uses an **iterative stack-based DFS** (`Vec<(&Group, usize)>` heap stack; `Group::children()` matching `usvg::Node::Group`; root counts as depth 1) because BC-1.12.003 inv-6 explicitly requires that the depth scan not be recursive. `slideforge-pdf` uses a **bounded recursive scan** (`render_group` calls itself as `render_group(g, surface, depth + 1)?`) — this is safe because the depth cap fires before any overflow risk, but the implementation is recursive, not iterative. The two crates share the same depth threshold (`MAX_SVG_NESTING_DEPTH = 64`) but their depth cause strings differ by a trailing context suffix: `slideforge-diagrams` emits `"SVG group nesting depth N exceeds the 64-level limit; possible DoS input"` (no suffix); `slideforge-pdf` emits `"SVG group nesting depth N exceeds the 64-level limit; possible DoS input — aborting SVG embed"` (trailing `" — aborting SVG embed"`). Each crate's wording matches its own binding spec; neither is wrong.

**Constant parity requirement:** `MAX_SVG_BYTES` and `MAX_SVG_NESTING_DEPTH` MUST be numerically equal across both crates at all times. A change to one requires a simultaneous change to the other.

**E-EXP-008 distinction:** E-EXP-008 (`Diagram render failed`) covers mermaid-rs-renderer failures; E-EXP-004 covers `usvg` normalization failures. These are sequential stages — E-EXP-008 is upstream of E-EXP-004. If mermaid-rs-renderer fails (E-EXP-008), the SVG normalization step is never reached.

**STORY-043 / STORY-079:** SEC-001/SEC-002 were first introduced in `slideforge-pdf` (STORY-043) and applied to `slideforge-diagrams` (STORY-079) for defense-in-depth. Both crates share the same constant values (`MAX_SVG_BYTES = 52,428,800`, `MAX_SVG_NESTING_DEPTH = 64`) and the same SIZE cause string verbatim (`"SVG input too large: N bytes exceeds the 52428800-byte limit"`). The DEPTH cause string differs: `slideforge-pdf` appends `" — aborting SVG embed"` to the shared base; `slideforge-diagrams` omits the suffix. See sub-case (3) above.

Note (E-EXP-003, `PdfExportError::InvalidXmpTitle`, SEC-050-001): `PdfExportError::InvalidXmpTitle { title, code_point }` is a PDF-crate-internal variant raised by `validate_title_for_xmp` when the deck title contains an XML-1.0-illegal control character (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F, U+FFFE, U+FFFF) that cannot be safely embedded in the XMP metadata stream (CWE-116). At the CLI diagnostic layer this variant surfaces as an E-EXP-003 message (`"PDF export error: PDF XMP metadata error: deck title contains XML-1.0-illegal control character U+XXXX — title: \"...\""`) — no separate taxonomy code is allocated. This mirrors the treatment of the analogous PPTX variant `PptxError::InvalidLanguageTag` (SEC-039-001, `validate_lang_for_xml`), which similarly routes through E-EXP-001 without a dedicated code. Both variants are security guards (CWE-116) that fire pre-serialization; both are broken-severity, exit 3. No Rust variant change is required to achieve this taxonomy consistency — the thiserror `Display` text of `InvalidXmpTitle` already provides a precise user-facing message.

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
| E-A11-001 | broken | 2 | `Missing alt text on <element-type> '<identifier>' at <file>:<line>:<col>. Add alt "..." or decorative: true.` | DI-001, CAP-020 |
| E-A11-002 | broken | 2 | `Missing label on color-coded element '<type>' '<identifier>' at <file>:<line>:<col>. Color alone must not convey meaning. Add label "...".` | DI-002, CAP-020 |
| E-A11-003 | cosmetic | 0 | `Missing lang declaration in deck metadata. Defaulting to "en". Screen readers may mispronounce non-English content. Add lang "en-US" (or appropriate BCP-47 tag).` | DI-003, CAP-020 |
| E-A11-004 | broken | 2 | `WCAG contrast ratio insufficient for text '<excerpt>' at <file>:<line>:<col>: <ratio>:1 (required 4.5:1 for normal text, 3:1 for large text). Consider using a higher-contrast color combination.` | CAP-022 |

Note (E-A11-001 trigger — ADR-018 + ADR-019): `E-A11-001` is emitted by `AltTextValidator`
during the **post-layout validation pass** (Stage 6b per ADR-018, human-authorized 2026-06-05),
NOT the pre-layout validation pass (Stage 5). This is because `ContentBlock::Chart`,
`ContentBlock::Image`, and `ContentBlock::Diagram` only exist in `LaidOutDeck.slides[*].frames`
(created by `layout::run`) and are never present in the pre-layout `Deck` (which always has
`slides[*].blocks == vec![]` after eval, per `slideforge-eval/src/for_eval.rs:342`).
`AltTextValidator.validate()` (Stage 5 dispatch) is a no-op stub — this is correct, not a bug.
Stage 6b diagnostics are accumulated into the same combined diagnostic list as Stage 5 diagnostics,
and the strict-mode gate fires once on the combined list.

**Precise trigger condition (ADR-019 Decision 5.3):** E-A11-001 fires when
`FrameContent::Chart { alt: AltText::Unspecified }`, `FrameContent::Image { alt: AltText::Unspecified }`,
or `FrameContent::Diagram { alt: AltText::Unspecified }` is found in `validate_post_layout`.
`AltText::Unspecified` is the third variant of the `AltText` enum (added in ADR-019 Decision 4)
representing the pipeline placeholder state: "no author alt-text data was threaded into this frame."
It is produced by:
- `regions.rs` structural placeholders (all five sites changed from `AltText::Decorative` to
  `AltText::Unspecified` in ADR-019 Decision 5.1), and
- `thread_media_alt_into_frames` fallback when the upstream `ContentBlock` has `alt = None`
  (ADR-019 Decision 5.2).

`AltText::Decorative` — produced when the author explicitly writes `decorative: true` —
does **NOT** trigger E-A11-001. `AltText::Decorative` is a valid author opt-out;
`validate_post_layout` treats it as passing. Before ADR-019, `regions.rs` misused
`AltText::Decorative` as a structural placeholder, causing false-positive E-A11-001 on
decorative elements. That bug is fixed: the three-variant enum makes the discrimination unambiguous.

**Dual-remedy message:** The message format `"... Add alt \"...\" or decorative: true."` instructs the
author on both remediation paths: (1) add `alt "description"` for visual content that needs a
description, or (2) add `decorative: true` for purely decorative content. Both remedies work
because the threading pass (Stage 2b, ADR-019) reads both `fields["alt"]` and `fields["decorative"]`
and sets the `ContentBlock.alt` field accordingly — which propagates to `AltText::Provided` or
`AltText::Decorative` on the frame, neither of which triggers E-A11-001.

A deck with a missing `alt` on a chart (no `alt "..."`, no `decorative: true`) still produces
`Err(BuildError::ValidationFailed)` (exit 2, no output) — the guarantee is upheld.
Implementer citation: this note supersedes any spec or comment that says E-A11-001 is
checked "before layout" — the authoritative sources are ADR-018 Decision 3 and ADR-019 Decision 5.3.

---

## Accessibility Warnings (W-A11)

Non-fatal accessibility lint warnings. Output is produced; exit code is 0. Routed to
`stderr` with a `warning:` prefix. Emitted via `tracing::warn!` with structured `code`
field — they are tracing events, not diagnostic-accumulator entries.

| Code | Severity | Exit | Message Format | Traces To |
|------|---------|------|---------------|-----------|
| W-A11-002 | cosmetic | 0 | Emitted in two contexts: (1) **Stage-2b `resolve_alt` (charts/images/diagrams):** `"warning: [W-A11-002] slide has both decorative: true and a non-empty alt; decorative takes precedence in Stage 2b (chart/image/diagram). Consider removing the alt field or removing decorative: true."` (2) **Shape DSL path (`slideforge-validate`):** `"warning: [W-A11-002] shape has both alt and decorative: true; alt takes precedence. Consider removing one."` | DI-001, BC-1.16.001 PC-12, BC-3.04.001 Inv-11 |

Note (W-A11-002): This warning fires when an element declares both `decorative: true` and
a non-empty `alt "..."`, flagging a likely authoring mistake where the author's accessibility
intent is ambiguous. The resolution direction differs by domain:

- **Stage-2b `thread_fields_to_blocks::resolve_alt` (charts, images, diagrams):** Decorative
  wins. `resolve_alt` returns `AltText::Decorative` and emits W-A11-002 via
  `tracing::warn!(code = "W-A11-002", ...)` at resolution time. The `alt` string is discarded.
  This warning is emitted by Stage-2b itself — NOT by `AltTextValidator::validate()`, which is
  Shape-only per ADR-018 v1.2 Decision-3 and cannot reach charts/images/diagrams pre-layout.
  Governed by BC-1.16.001 PC-12 (decorative-first).

- **Shape DSL path (`layout_shapes` → `ShapeFrame`, `slideforge-validate` validator):** Alt
  wins. The `ShapeFrame.alt` carries `AltText::Provided(s)` at layout resolution time;
  `ShapeSpec.decorative` is preserved in the IR (not mutated). W-A11-002 is emitted by
  `slideforge-validate` after layout to prompt the author to clean up the ambiguity, but
  the alt text MUST NOT be suppressed. Governed by BC-3.04.001 Invariant 11 (alt-first,
  shape DSL path only).

W-A11-001 is a DEPRECATED code (was used in BC-5.01.002 §3 prior to v1.2). All references
should use W-A11-002. W-A11-001 is not registered here — it is retired.

Pre-registration check (2026-06-06): W-A11-002 was referenced in BC-3.04.001 v1.5.1+,
BC-1.16.001 EC-004, and the STORY-086 pass-5 adjudication (F-086-P5-MED-002), but had
no taxonomy row prior to this entry. No collision with existing codes — the W-A11 warning
namespace was previously empty in this taxonomy.

---

## Validation Errors (E-VAL)

Fatal in strict mode (exit 2). In `--warn-only` mode: error is reported as a warning;
output may still be produced.

Formal-registration note (2026-06-07, v2.20): E-VAL-101, E-VAL-102, and W-VAL-103 are
now formally registered below. These codes have been informally in use in
`crates/slideforge-plugin-api/src/slide_types/registry.rs` (`validate_fields`) since
prior to v2.18; this version resolves the debt noted in v2.18 ("will be formally
registered in a future spec burst"). No implementer action required — the code strings
in `registry.rs` are already correct and match the message formats documented here
exactly (confirmed by reading the `validate_fields` function in
`crates/slideforge-plugin-api/src/slide_types/registry.rs` at time of
registration). The 1xx grouping (101 absent, 102 empty, 103 unknown, 104 type-mismatch)
is hereby formally established as the `validate_fields` schema family.

| Code | Severity | Exit (strict) | Message Format | Traces To |
|------|---------|--------------|---------------|-----------|
| E-VAL-011 | broken | 2 | `<slide_type> <field_path> must be <constraint>; got <value>.` (See message variants in BC-1.17.002 PC3, BC-1.17.003 inv 6/7) | BC-1.17.002 PC3, BC-1.17.003 inv 6/7 |
| E-VAL-012 | broken | 2 | Three message variants depending on traversal kind: (1) `Image path '<path>' escapes the source root — contains a path-traversal segment ('..'). Image paths must be relative and contained within the source file's directory tree.` (2) `Image path '<path>' escapes the source root — path is absolute (starts with '/' or '\'). Image paths must be relative and contained within the source file's directory tree.` (3) `Image path '<path>' escapes the source root — uses a Windows drive letter prefix. Image paths must be relative and contained within the source file's directory tree.` All three variants include `at <file>:<line>:<col>` span suffix (from `spec.span`). | BC-1.16.001 EC-012, CAP-022, CWE-22 |
| E-VAL-101 | broken | 2 | `Required field '<name>' missing on <type> slide. Required fields for '<type>': [<list>].` | BC-1.18.001 PC (absent-field precondition), DI-018, CAP-022 |
| E-VAL-102 | broken | 2 | `Required field '<name>' is empty on <type> slide. Required fields for '<type>': [<list>].` | BC-1.18.001 PC (empty-field precondition), DI-018, CAP-022 |
| W-VAL-103 | cosmetic | 0 | `Unknown field '<key>' for slide type '<type>'. Known fields: [<list>].` | BC-1.18.001 (unknown-field branch), CAP-022 |
| E-VAL-104 | broken | 2 | Two message branches under this single code (same pattern as E-BRD-001/E-BRD-002 multi-variant): (T1 type-mismatch) `Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>. See the DSL reference for valid field types.` — (T2 OneOf violation) `Field '<name>' on <type> slide has disallowed value "<val>": allowed values are [<list>].` Source span carried in `Diagnostic.span`, rendered by miette as `file:line:col` (consistent with E-VAL-101/102 convention — not embedded literally in the message string). | BC-1.18.001, DI-018, DI-004, CAP-022 |

Note (E-VAL-101): `validate_fields` formal registration. Emitted by `validate_fields` in
`crates/slideforge-plugin-api/src/slide_types/registry.rs` when a required field (declared
in `SlideType::required_fields()`) is absent from `Slide.fields` entirely (i.e., the key
is not present in the fields map). Severity: broken. Exit 2 in strict mode. The `<name>`
placeholder is the absent field name; `<type>` is the slide type ID; `<list>` is the
comma-separated list of all required field names for that type (built once before the loop:
`required_list_str = required_list.join(", ")`). The diagnostic also carries a `hint` field:
`"add '<name>: <value>' to this slide block. Required fields for '<type>': [<list>]."`.
Error accumulation per DI-018 — `validate_fields` continues past absent-field errors to
find all issues.

**Pre-registration collision check (2026-06-07):** E-VAL-101 confirmed not present in
error-taxonomy.md prior to v2.20 (only E-VAL-011 and E-VAL-012 were formally registered;
E-VAL-101 was debt-noted since v2.18). Confirmed present at the E-VAL-101 emission site in `validate_fields`
(`crates/slideforge-plugin-api/src/slide_types/registry.rs`) with `code: Arc::from("E-VAL-101")`. No implementer action required.

Note (E-VAL-102): `validate_fields` formal registration. Emitted by `validate_fields`
when a required field is present as `FieldValue::Literal(Value::Str(s))` with `s.is_empty()`.
The field exists in the map but its value is an empty string — which `validate_fields`
rejects for required fields. Severity: broken. Exit 2 in strict mode. Placeholders:
`<name>` = field name; `<type>` = slide type ID; `<list>` = all required field names.
Hint: `"provide a non-empty value for '<name>'. Required fields for '<type>': [<list>]."`.
Note: a required field that is `Value::Str("  ")` (whitespace-only) does NOT trigger
E-VAL-102 — the empty-check is `s.is_empty()`, not `s.trim().is_empty()`. Authors must
provide at minimum a non-empty string (whitespace is accepted by E-VAL-102, though layout
may still reject whitespace-only title/body values at its own stage).

**Pre-registration collision check (2026-06-07):** E-VAL-102 confirmed not present in
error-taxonomy.md prior to v2.20. Confirmed present at the E-VAL-102 emission site in `validate_fields`
(`crates/slideforge-plugin-api/src/slide_types/registry.rs`) with `code: Arc::from("E-VAL-102")`. No implementer action required.

Note (W-VAL-103): `validate_fields` formal registration. Emitted by `validate_fields`
when a field key in `Slide.fields` is not in the union of `required_fields()` and
`optional_fields()` for the slide's registered type. The `<key>` placeholder is the
unrecognized field name; `<type>` is the slide type ID; `<list>` is the sorted
comma-separated known-field list (built once before the unknown-field loop:
`known_list.sort_unstable(); known_list.join(", ")`).
The diagnostic also carries a `hint` field: `"Valid fields for '<type>' are: <known_list_str>"`.
Routed to `stderr` with a `warning:` prefix. Distinct from E-VAL-104 (type mismatch on a
KNOWN field) — W-VAL-103 fires when the field is UNKNOWN to the slide type entirely.

**Route A: Context-Sensitive Severity (v2.29, REND-005/STORY-098).**
W-VAL-103 has TWO severity contexts — the message format is unchanged but the severity
used at accumulation time differs:

1. **Content-drop sub-case — broken/exit-2 in strict mode:**
   When the unknown field key is `"shape"` or `"body"` AND the slide type's
   `known_fields()` does NOT include that key (i.e., the field is unknown AND would
   cause authored user content to be silently dropped), W-VAL-103 is promoted to
   `broken` severity in strict mode. In strict mode, `validate_fields` adds this
   diagnostic to the error accumulator with `broken` severity. The build exits 2 and
   produces no output (per BC-3.03.002 invariant 4). In `--warn-only` mode, the
   severity is `cosmetic` (exit 0) and the field is dropped with a warning.
   Rationale: CLAUDE.md production-grade default principle — silent dropping of authored
   content in the default build mode is a build failure, not a lint. "No silent fallback"
   (CLAUDE.md Rule 1) and DI-017 (all-or-nothing in strict mode) both mandate this.
   **Critical boundary (PO adjudication F-098-ADJ-BODY-CONTENT, v2.32):** If the
   field key IS in the slide type's `known_fields()`, it is NOT an unknown field and
   W-VAL-103 is not emitted at all — regardless of whether the key is `"body"` or
   `"shape"`. Concretely: `body` on the `content` slide type does not trigger W-VAL-103
   because `content`'s `known_fields()` includes `"body"`. `body` on `quote`, `title`,
   `stat_callout`, and other types that do NOT declare `body` still triggers exit-2.

2. **Non-content sub-case — cosmetic/exit-0 (unchanged):**
   When the unknown field key is any key OTHER than `"shape"` or `"body"` (i.e., a
   metadata annotation, a future-reserved keyword, or any field that has no content
   impact on the rendered output), W-VAL-103 remains cosmetic and does NOT increment
   the error counter. Behavior is unchanged from v2.20.

**No new error code E-VAL-105 is introduced.** Route A was chosen over Route B
(new E-VAL-105 for content-drop sub-case) because: (a) the two contexts differ only in
the field-key set, not in the message format or user-visible semantics, making a new code
redundant; (b) the content-drop check is a two-key set (`{"shape", "body"}`) and does not
require a new code prefix; (c) adding a new code would require updating all test fixtures
and consuming code without providing additional user value.

**Implementer action (STORY-098, updated per F-098-ADJ-BODY-CONTENT):** In `validate_fields`
in `crates/slideforge-plugin-api/src/slide_types/registry.rs`, when a W-VAL-103 diagnostic
is about to be accumulated (i.e., when a field key is not in the slide type's known fields),
check whether `key` is `"shape"` or `"body"`. If yes AND the build mode is `strict`,
accumulate as `broken` severity (which causes the output gate in `slideforge-cli` to exit 2).
If `--warn-only`, accumulate as `cosmetic` regardless of key. All other unknown-field keys:
accumulate as `cosmetic` always. **Note:** By definition, if the field reached this path it
is already unknown (not in `known_fields()`), so the `content` type with `body` declared in
its `known_fields()` will never reach this code path for that field — the gate fires only
for genuinely unknown fields.

**Pre-registration collision check (2026-06-07):** W-VAL-103 confirmed not present in
error-taxonomy.md prior to v2.20 (the W-VAL warning namespace was empty). Confirmed
present at the W-VAL-103 emission site in `validate_fields`
(`crates/slideforge-plugin-api/src/slide_types/registry.rs`) with `code: Arc::from("W-VAL-103")`.
v2.29: content-drop severity promotion added per BC-3.03.002 v1.2 Invariant 4 (STORY-098).
v2.32: PO binding adjudication F-098-ADJ-BODY-CONTENT (2026-06-11) — clarified that the
  content-drop gate fires only when the field is UNKNOWN (not in known_fields()). `body`
  on `content` slide type is VALID (content declares body in known_fields()); it never
  reaches the W-VAL-103 accumulation path. BC-3.03.002 updated to v1.3. See changelog v2.32.

Note (E-VAL-104): NEW — schema-driven field-value type validation. Emitted by the new
type-check arm in `validate_fields` (to be added by STORY-089) when a field's runtime
`Value` variant does not match its `FieldDef.expected_type` annotation. Two message
branches share this single code (consistent with E-BRD-001/E-BRD-002 multi-variant pattern
and E-VAL-012 three-variant pattern):

**T1 — Type-mismatch (wrong Value variant):**
`Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>. See the DSL reference for valid field types.`
- `<name>`: field name from `FieldDef.name`
- `<type>`: slide type ID from `slide_type.id()`
- `<expected>`: human-readable display name from `FieldType::display_name()` — e.g., `"integer"` for `FieldType::Int`, `"boolean"` for `FieldType::Bool`, `"string"` for `FieldType::Str`, `"list"` for `FieldType::List`, `"map"` for `FieldType::Map`, `"float"` for `FieldType::Float`, `"string"` for `FieldType::OneOf` (the expected base type is Str)
- `<actual>`: `value_type_name(value)` — e.g., `"string"`, `"integer"`, `"boolean"`, `"list"`, `"map"`, `"float"`

**T2 — OneOf violation (`FieldType::OneOf` — value is Str but not in allowlist):**
`Field '<name>' on <type> slide has disallowed value "<val>": allowed values are [<list>].`
- `<val>`: the disallowed string value
- `<list>`: comma-separated allowlist from `FieldType::OneOf(variants)` — e.g., `bar, line, pie, scatter, area, stacked-bar, stacked-area` for `chart_type`

**Source span convention (M1 alignment, STORY-089 adversary):** Neither T1 nor T2 embeds a literal `(at <file>:<line>:<col>)` suffix in the message string. The source location is carried structurally in `Diagnostic.span` (the slide's `source_span`) and rendered by miette in the terminal output as `file:line:col`. This is the established convention for all `validate_fields` codes — E-VAL-101 and E-VAL-102 follow the identical pattern; there is no `(at ...)` suffix in their message strings either. E-VAL-104's templates previously deviated from this convention; this note records the alignment.

Note: T2 is only reached when the value is `Value::Str`. If the value on an `OneOf`-typed
field is a non-Str variant (e.g., `Value::Int`), T1 fires (type mismatch — expected string,
got integer). The `OneOf` allowlist check is subordinate to the variant check.

Severity: broken. Exit 2 in strict mode. In `--warn-only` mode: E-VAL-104 is reported as
a warning; error-slide placeholder rendered for the affected slide; build continues.

Error accumulation per DI-018: `validate_fields` accumulates ALL E-VAL-104 diagnostics for
a slide before returning. A slide with three mistyped annotated fields produces three
E-VAL-104 diagnostics; all three are reported in a single build pass.

The `type_matches(value: &Value, expected: &FieldType) -> bool` pure helper function
implements the matching logic. Fields annotated `None` or `Some(FieldType::Any)` are
unconditionally skipped (no E-VAL-104 emitted). `FieldValue::Inlines(_)` fields are also
skipped for this arm.

**Pre-registration collision check (2026-06-07):** E-VAL-104 confirmed free in
error-taxonomy.md (the E-VAL namespace had E-VAL-011, E-VAL-012, E-VAL-101, E-VAL-102,
W-VAL-103 — no E-VAL-104). Confirmed absent from `grep -rE "E-VAL-104" crates/**/*.rs`:
zero matches in codebase at time of registration. The 1xx grouping convention (101=absent,
102=empty, 103=unknown, 104=type-mismatch) is formally established by this version.

**New BC:** BC-1.18.001 (registered 2026-06-07) defines the full behavioral contract for
E-VAL-104, including preconditions, postconditions, invariants, edge cases, and canonical
test vectors. STORY-089 owns implementation.

Note (E-VAL-011): Numeric field out-of-range violation for color-coded slide types. Emitted
by `ValueRangeValidator` at Stage 5 (pre-layout) in `slideforge::registry::register_bundled_plugins`.
Reads `Slide.fields` directly (Stage-2b independence guaranteed — reads `FieldValue::Literal` only).
Strict-mode fatal: `DiagnosticSeverity::Error` → `BuildError::ValidationFailed`. Validator ID:
`"value-range"`.

**Message variants:**

- `progress_bar` value out of range (BC-1.17.002 PC3):
  - Type error or absent: `progress_bar value must be an integer; got <actual_type> or absent.`
  - Out of range: `progress_bar value must be between 0 and 100; got <N>.`
- `weighted_composite` component weight non-positive (BC-1.17.003 inv 6):
  `weighted_composite components[<idx>].weight must be positive; got <actual>.`
- `weighted_composite` component score out of range (BC-1.17.003 inv 7):
  `weighted_composite components[<idx>].score must be between 0 and 100; got <N>.`

**Error accumulation:** `ValueRangeValidator` iterates ALL components and collects ALL
E-VAL-011 diagnostics before returning (DI-018). It does NOT bail on the first failure.

**Additional message variant — empty components (BC-1.17.003, F-087-P2-001):**

> Note: E-VAL-011 is also emitted when `weighted_composite.components` is present but
> empty (`[]`). Message: `"weighted_composite requires at least one component; got empty list."`.
> This is a structural value contract violation, not a type error. `Value::List([])` is a
> valid DSL value that passes Stage 2a evaluation but violates BC-1.17.003 postcondition 3.
> The empty-list check is performed in `ValueRangeValidator::validate_weighted_composite_components`
> before the per-component iteration loop.

Allocated per architect adjudication F-087-P1-001 (STORY-087 pass 1, 2026-06-06).
Empty-components variant added per architect adjudication F-087-P2-001 (STORY-087 pass 2, 2026-06-06).

Note (E-VAL-012): `ValidatorDiagnostic` emitted by `ImagePathValidator` (Validator surface #5, validator ID `"image-path"`) during the **pre-layout validation pass** (Stage 5) in `slideforge-validate`. The validator operates on the semantic `Deck` IR (specifically on `ContentBlock::Image(ImageSpec { path, .. })` entries in `Slide.blocks`) after `thread_fields_to_blocks` (Stage 2b, ADR-019) has populated `Slide.blocks` from resolved field values.

**Containment rule (CWE-22):** An `ImageSpec.path` value is rejected if it meets ANY of the following conditions:
1. Contains a path-traversal segment: the string `".."` appears as a complete path component (i.e., the path string contains `".."` separated by `/` or `\`, or starts with `".."`).
2. Is absolute: starts with `/` (Unix-style) or `\` (Windows UNC-style).
3. Uses a Windows drive letter prefix: matches the pattern `<letter>:` at the start (e.g., `C:\`, `D:/`).

**Stage justification:** E-VAL-012 is allocated to the E-VAL namespace (Validator surface, pre-layout, exit 2) rather than E-PAR (parse stage) because `ImageSpec.path` is populated by Stage 2b (`thread_fields_to_blocks`) after expression evaluation — the raw DSL field is `src:` (user-written keyword), which may be an interpolated `{{ expr }}` resolving to an attacker-controlled path at eval time. A parse-stage guard (`E-PAR-NNN`) would fire on the literal token before interpolation, which is insufficient when the path is expression-computed. The validation stage (Stage 5) sees the post-eval path value, making E-VAL-012 the correct stage for this containment check. This mirrors the precedent established by E-PAR-021 vs. E-LAY-005 (same conceptual depth limit, two guards at the correct stages).

**No disk I/O:** The validator checks the path string only (syntactic containment check). It does NOT open the file, resolve symlinks, or perform any filesystem access. Symlink escape (a deeper containment concern) is deferred to the export stage, which resolves to a canonical path before opening. This is the pre-emptive string-level guard only.

**Severity:** broken. Exit 2 in strict mode. In `--warn-only` mode: error-slide placeholder rendered for the affected slide; build continues.

**Message format (full, with span):**
- Traversal segment: `Image path '<path>' escapes the source root — contains a path-traversal segment ('..'). Image paths must be relative and contained within the source file's directory tree. (at <file>:<line>:<col>)`
- Absolute path: `Image path '<path>' escapes the source root — path is absolute (starts with '/' or '\'). Image paths must be relative and contained within the source file's directory tree. (at <file>:<line>:<col>)`
- Drive letter: `Image path '<path>' escapes the source root — uses a Windows drive letter prefix. Image paths must be relative and contained within the source file's directory tree. (at <file>:<line>:<col>)`

**Variant name:** `ValidatorDiagnostic` with `code: Arc::from("E-VAL-012")`. The three traversal-kind sub-cases are distinguished in the message text, not as separate Rust variants (same approach as E-BRD-001 and E-BRD-002, which share a single code across multiple `BrandError` variants).

**Traceability:** SEC-001, BC-1.16.001 EC-012 (image path traversal / containment), CAP-022 ("Compile-Time Content Validation") per capabilities.md §CAP-022 — the `ImagePathValidator` is a compile-time content validator rejecting paths that violate the source-root containment invariant.

**Pre-registration collision check (2026-06-07):** E-VAL-012 confirmed free — not present in error-taxonomy.md prior to this entry (the E-VAL namespace contained only E-VAL-011 plus the informally-used codes E-VAL-101/102/W-VAL-103), and confirmed absent from `grep -rE "E-VAL-012" crates/**/*.rs` (no output — zero matches in the codebase at time of registration).

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
| 2.11 | 2026-06-02 | product-owner | STORY-077 adversary pass-7 finding F-077-P7-002 — allocated E-PAR-021 (`ParseError::InlineNestingDepthExceeded`) for the parse-stage inline-markup nesting-depth bound in `scan_template_chunks`. Adjudication: E-LAY-005 (`LayoutError::InlineDepthExceeded`) was considered for reuse but rejected — it is stage-specific to layout (fires during `Vec<InlineNode>` traversal, propagates as fatal `Err`, exit 2) and does not accumulate; E-PAR-021 fires at parse time (inside `scan_template_chunks` recursion), accumulates non-fatally for further parsing, and exits 1. Stage-accurate allocation is the established pattern per E-PAR-015/016 vs shape-code collision history. Pre-registration collision check: E-PAR-021 confirmed free in taxonomy and in grep of all `crates/**/*.rs`. Message format: `Inline-markup nesting depth exceeded at <file>:<line>:<col>: depth <N> exceeds maximum of <max>. Flatten or reduce nested inline markup.` Traces to BC-3.02.002, DIR-077-002 §5, CAP-001, STORY-077. |
| 2.12 | 2026-06-02 | product-owner | STORY-077 adversary pass-9 finding F-077-P9-001 — (1) E-EVL-013 Note broadened: scope expanded from `ref("")`/`"" | ref` (empty-string only) to cover all three no-usable-id conditions: `ref()` zero-arg, `ref("")` empty-string arg, and `ref(expr)` where expr evaluates to empty. All three yield E-EVL-013; no new code allocated (mirrors figref precedent). (2) E-EVL-014 (`EvalError::FootnoteInvalidArg`) registered — new code for `footnote()` called with zero arguments or an argument that evaluates to empty/unusable text in inline-markup section sub-block context; no `InlineNode::Footnote` produced; eval-stage, broken, exit 2, error accumulation continues. E-EVL-014 confirmed free: absent from taxonomy (E-EVL-001 through E-EVL-013 fully accounted for) and from grep of `crates/slideforge-eval/src/error.rs`. Traces to CAP-002, CAP-024, BC-3.02.002, DIR-077-002 §5, STORY-077. |
| 2.13 | 2026-06-03 | product-owner | STORY-077 follow-up burst (human-authorized 2026-06-03): (1) **E-PAR-022 registered** (`DisallowedLinkUrlScheme`) — parse-stage error for `[text](url)` links with disallowed URL schemes. Allowlist: `http`, `https`, `mailto`. All other schemes (including `javascript:`, `data:`, `vbscript:`, `file:`) and scheme-less URLs are rejected. Relative/anchor links (`#section`, `page.sf`) are NOT a v1 use case — internal cross-references use `{{ ref("id") }}`. Accumulated (same fatal path as E-PAR-019/020/021), exit 1. SEC-002 mandated. (2) **E-PAR-012 re-registered** (active) — re-registered for unterminated `{{ }}` interpolation. The v1.1 retirement was incorrect: `unterminated_interpolation_msg()` in `template.rs` has always emitted `"E-PAR-012"` and continues to do so. No implementer code change required. (3) **E-EVL-007 through E-EVL-011 registered** — taxonomy debt note removed. E-EVL-007 (`TooManySlides`, broken, exit 2), E-EVL-008 (`NotIterable`, broken, exit 2), E-EVL-009 (`LargeDeckWarning`, degraded, exit 0 — warning only), E-EVL-010 (`UnknownSectionType`, broken, exit 2), E-EVL-011 (`UnsupportedBuiltinCall`, broken, exit 2). All five were allocated in `crates/slideforge-eval/src/error.rs`; now formally registered. No code changes required (doc-only). |
| 2.14 | 2026-06-05 | product-owner | STORY-050 Gap-2 / ADR-018 (human-authorized 2026-06-05): **E-A11-001 pipeline-stage note added.** E-A11-001 is emitted during the post-layout validation pass (Stage 6b, `validate_post_layout(&LaidOutDeck)`) per ADR-018, not the pre-layout pass (Stage 5). Root cause confirmed: `Deck.slides[*].blocks == vec![]` after eval (`for_eval.rs:342`); `FrameContent::Chart/Image/Diagram` only exist in `LaidOutDeck` post-layout. The error code, severity (broken), and exit code (2) are **unchanged** — only the pipeline stage where enforcement fires has changed. Note appended to E-A11-001 documenting this for implementers. This note supersedes any existing spec text or code comment stating E-A11-001 is checked "before layout." BC-5.02.001 v1.5 and BC-5.01.001 v1.2 updated in same burst. |
| 2.16 | 2026-06-05 | product-owner | ADR-019 (Stage 2b, human-authorized 2026-06-05): **E-A11-001 trigger condition precisely documented.** E-A11-001 fires on `AltText::Unspecified` frames (NOT on `AltText::Decorative` frames). `AltText::Unspecified` is the new third enum variant (ADR-019 Decision 4) meaning "structural pipeline placeholder; no author alt-text data threaded." `AltText::Decorative` (author wrote `decorative: true`) is a valid opt-out — `validate_post_layout` now distinguishes these two cases unambiguously. Five `regions.rs` structural placeholder sites changed from `Decorative` to `Unspecified` (ADR-019 Decision 5.1); three `thread_media_alt_into_frames` fallback sites changed from `Decorative` to `Unspecified` (ADR-019 Decision 5.2); three `validate_post_layout` match arms updated accordingly (ADR-019 Decision 5.3). Dual-remedy message note added: "Add alt \"...\" or decorative: true" — both remedies work because Stage 2b (ADR-019) reads both fields and sets ContentBlock.alt → propagates to AltText::Provided or AltText::Decorative on frame → neither triggers E-A11-001. Error code, severity (broken), and exit code (2) are UNCHANGED — only the triggering variant name is narrowed from any-non-Provided to specifically Unspecified. BC-5.01.001 v1.3 and BC-5.02.001 v1.6 updated in same burst. |
| 2.15 | 2026-06-05 | product-owner | STORY-050 PR #61 security review (SEC-050-001) taxonomy consistency: **E-EXP-003 note added** for `PdfExportError::InvalidXmpTitle`. Confirmed that the analogous PPTX variant `PptxError::InvalidLanguageTag` (SEC-039-001) has no dedicated taxonomy code — it routes through E-EXP-001 as a variant-only pattern. Per consistency-mirror rule: `InvalidXmpTitle` likewise receives no dedicated code; it routes through E-EXP-003 as a variant-only pattern. E-EXP-003 table row updated to cite the variant; explanatory note added documenting the routing, the CWE-116 security context, the parallel with SEC-039-001, and confirming no Rust variant change is required. No new E-EXP-NNN code allocated. |
| 2.17 | 2026-06-06 | product-owner | STORY-086 pass-5 adjudication (F-086-P5-MED-002): **W-A11-002 registered** — new "Accessibility Warnings (W-A11)" section added. W-A11-002 was referenced in BC-3.04.001 v1.5.1+, BC-1.16.001 EC-004, and STORY-086 adjudication but had no taxonomy entry (the W-A11 warning namespace was empty prior to this version). Pre-registration collision check confirmed W-A11-002 free. Semantics: fires when both `decorative: true` and a non-empty `alt "..."` are set on the same element, flagging a likely authoring mistake. Two-context warning: (1) Stage-2b `resolve_alt` for charts/images/diagrams — decorative wins, W-A11-002 emitted via `tracing::warn!(code = "W-A11-002")` at resolution time; (2) shape DSL `slideforge-validate` — alt wins (BC-3.04.001 Inv-11), W-A11-002 emitted post-layout. W-A11-001 (deprecated predecessor) is NOT registered — it is retired. Taxonomy version 2.16 → 2.17. Note: changelog row 2.15 appears after 2.16 in the table due to authoring order — this is a documentation sequencing artifact, not a retcon; both v2.15 and v2.16 were produced on the same date (2026-06-05) in separate bursts. |
| 2.18 | 2026-06-06 | product-owner | STORY-087 pass-1 adjudication (F-087-P1-001): **E-VAL-011 registered** — new "Validation Errors (E-VAL)" section added. E-VAL-011 is allocated for numeric field out-of-range violations emitted by `ValueRangeValidator` at Stage 5 (pre-layout). Covers `progress_bar` value out of [0, 100] (BC-1.17.002 PC3) and `weighted_composite` component `weight` non-positive / `score` out of [0, 100] (BC-1.17.003 inv 6/7). Severity: broken. Exit 2 (strict-mode fatal). Error accumulation per DI-018: all component errors collected before returning. Pre-registration collision check: E-VAL-011 confirmed free — no existing E-VAL-NNN entries in taxonomy prior to this version. The informally-used codes E-VAL-101, E-VAL-102, W-VAL-103 in `registry.rs::validate_fields` remain unregistered (future burst). |
| 2.18-addendum | 2026-06-06 | product-owner | STORY-087 pass-2 adjudication (F-087-P2-001): **E-VAL-011 addendum** — empty-components message variant added to existing E-VAL-011 Note. No new code allocated (same error code, same severity/exit). `weighted_composite.components: []` (empty list) is now an explicitly documented E-VAL-011 trigger with exact message `"weighted_composite requires at least one component; got empty list."`. Implementation directive: `ValueRangeValidator::validate_weighted_composite_components` must add an early-return arm for `Value::List(list) if list.is_empty()` that emits the diagnostic before the per-component loop. No taxonomy version bump per adjudication directive. |
| 2.20 | 2026-06-07 | product-owner | STORY-089 Wave-5 slot-1 spec burst (human-authorized 2026-06-07): **Four codes registered — E-VAL-101, E-VAL-102, W-VAL-103 (formal registrations, no implementer action), E-VAL-104 (new).** (1) **E-VAL-101 formal registration:** `validate_fields` required-field-absent code; confirmed present at the E-VAL-101 emission site in `validate_fields` (`crates/slideforge-plugin-api/src/slide_types/registry.rs`); message `Required field '<name>' missing on <type> slide. Required fields for '<type>': [<list>].`; broken/exit 2; DI-018 accumulation. (2) **E-VAL-102 formal registration:** `validate_fields` required-field-empty-string code; confirmed at the E-VAL-102 emission site in `validate_fields` (`crates/slideforge-plugin-api/src/slide_types/registry.rs`); message `Required field '<name>' is empty on <type> slide. Required fields for '<type>': [<list>].`; broken/exit 2. (3) **W-VAL-103 formal registration:** `validate_fields` unknown-field warning code; confirmed at the W-VAL-103 emission site in `validate_fields` (`crates/slideforge-plugin-api/src/slide_types/registry.rs`); message `Unknown field '<key>' for slide type '<type>'. Known fields: [<list>].`; cosmetic/exit 0; first W-VAL entry in namespace. (4) **E-VAL-104 NEW:** schema-driven field-value type validation. Two message branches: T1 type-mismatch — `Field '<name>' on <type> slide has wrong type: expected <expected>, got <actual>. See the DSL reference for valid field types. (at <file>:<line>:<col>)`; T2 OneOf violation — `Field '<name>' on <type> slide has disallowed value "<val>": allowed values are [<list>]. (at <file>:<line>:<col>)`. Broken/exit 2. Multi-variant single-code pattern per E-BRD-001 precedent. **Collision check:** E-VAL-104 confirmed free in taxonomy (only E-VAL-011, E-VAL-012, E-VAL-101/102, W-VAL-103 existed); confirmed absent from `grep -rE "E-VAL-104" crates/**/*.rs` (zero matches). 1xx grouping formally established: 101=absent, 102=empty, 103=unknown, 104=type-mismatch. New BC-1.18.001 defines the full behavioral contract. STORY-089 owns implementation. Produces no output (no code changes to crates/ — spec-only burst). |
| 2.19 | 2026-06-07 | product-owner | SEC-001 image-path-traversal containment (Wave-4 follow-up, human-authorized 2026-06-07): **E-VAL-012 registered** — new "Validation Errors (E-VAL)" row for `ImagePathValidator` (Validator surface #5, validator ID `"image-path"`). E-VAL-012 guards `ImageSpec.path` values that traverse outside the source-file root: (1) contains `".."` path-traversal segment, (2) is absolute (`/` or `\`), or (3) uses a Windows drive letter prefix (`C:\`, etc.). CWE-22. **Stage justification:** E-VAL (Validator/pre-layout, exit 2) is correct over E-PAR (parse/exit 1) because `ImageSpec.path` is post-eval — `src:` may be an interpolated `{{ expr }}`; a parse-stage guard would not catch expression-computed paths. Precedent: E-PAR-021 vs. E-LAY-005 (dual-stage guards for the same conceptual limit at correct stages). **Family justification:** E-VAL (not a new E-IMG family) because the existing E-VAL namespace covers all `Validator`-surface compile-time content diagnostics; no precedent for per-feature subsystem codes (E-BRD, E-DAT, E-PKG are subsystem families, not feature families). Three message variants distinguished in message text only (not separate codes): traversal-segment, absolute-path, drive-letter. Severity: broken. Exit: 2 (strict mode). `--warn-only`: error-slide placeholder, build continues. Traces to BC-1.16.001 EC-012, CAP-022. **Collision check (2026-06-07):** E-VAL-012 confirmed free in taxonomy (only E-VAL-011, E-VAL-101, E-VAL-102, W-VAL-103 existed) and confirmed absent from `grep -rE "E-VAL-012" crates/**/*.rs` (zero matches). BC-1.16.001 updated with EC-012 in same burst. |
| 2.21 | 2026-06-07 | product-owner | STORY-089 adversary findings M1 + M2 (spec-only corrections, no implementer action). **M1 (E-VAL-104 message template alignment):** Removed the trailing ` (at <file>:<line>:<col>)` literal suffix from all E-VAL-104 message templates — table row (both T1 and T2 branches), Note (E-VAL-104) T1 block, Note (E-VAL-104) T2 block. The source location is carried structurally in `Diagnostic.span` (the slide's `source_span`) and rendered by miette as `file:line:col` — it is NOT embedded literally in the message string. This is the established convention for all `validate_fields` codes: E-VAL-101 message `Required field '<name>' missing on <type> slide. Required fields for '<type>': [<list>].` has no span suffix; E-VAL-102 likewise. E-VAL-104 was the inconsistent outlier. A clarifying note has been added to the Note (E-VAL-104) T2 section documenting this convention. BC-1.18.001 v1.2 updated in same burst (PC-2, PC-3, EC-001, EC-003, and canonical test vectors all de-suffixed). **M2 (chart.data optional-at-schema record):** Added Note (BC-1.18.001 Invariant 8) recording that `chart.data` is OPTIONAL at field-schema level — `validate_fields` does NOT emit E-VAL-101 for absent `chart.data`. Data presence is enforced by `ChartRenderer` at render time (export stage), not at Stage-5 schema validation. This reclassification (required → optional) was applied in STORY-089 wiring and is tested in the STORY-089 test suite. No taxonomy table changes for M2 (it is a BC-level behavioral note, not a new error code). |
| 2.24 | 2026-06-07 | product-owner | STORY-082 spec reconciliation burst: **E-PAR-023 and W-PAR-002 allocated** for slide-grouping section edge cases EC-010 and EC-011. (1) **E-PAR-023 (`ParseError::EmptySectionGroupName`)** — broken, exit 1 (corrected from original allocation which incorrectly stated exit 2; see v2.28 correction). Fires when `section "":` declares an empty quoted section-group name. Error accumulated; no `SectionGroupNode` produced; span on opening `"`. Message: `E-PAR-023: section group name must be non-empty at <file>:<line>:<col>. Provide a quoted, non-empty name, e.g. section "Background":`. Pre-registration check: E-PAR-023 confirmed free in taxonomy and from `grep -r "E-PAR-023" crates/` (no matches). (2) **W-PAR-002 (`ParseWarning::DuplicateSectionGroupName`)** — cosmetic, exit 0. Fires when two or more `section "Name":` blocks in the same file share an identical quoted name. Both sections emitted; same GUID (deterministic by sha2 name hash); no crash. Message: `warning: [W-PAR-002] Duplicate section group name '<name>' at <file>:<line>:<col>. Both sections are emitted with the same GUID. Consider using distinct names.` Pre-registration check: W-PAR-002 confirmed free — W-PAR namespace had only W-PAR-001 prior to this version; grep of `crates/` confirms zero matches. BC-4.01.003 updated to v1.3 in same burst (EC-010, EC-011, invariant 5 added). |
| 2.23 | 2026-06-07 | product-owner | STORY-089 adversary O-1 (spec-only, no implementer action): **De-brittled source citations in E-VAL-101, E-VAL-102, W-VAL-103 Notes and in the v2.20 changelog entry.** Six brittle `registry.rs:<line-number>` references replaced with stable construct references — each now names the function (`validate_fields`) and the file path (`crates/slideforge-plugin-api/src/slide_types/registry.rs`) without a line number, which will not drift when STORY-089 or future stories insert or remove lines inside `validate_fields`. No error codes, message formats, severities, or behaviors changed. |
| 2.22 | 2026-06-07 | product-owner | STORY-089 adversary LOW fixes (spec-only, no implementer action). **(1) E-VAL-104 Note — `value_type_name()` function name corrected:** The `<actual>` placeholder description in Note (E-VAL-104) T1 block previously cited `Value::type_name()` (an associated method). The actual production code in `slideforge-plugin-api` uses the free function `value_type_name(value)`. Corrected to `value_type_name(value)`. No code change required — implementer was already using the correct function name; this was a spec/note divergence only. **(2) BC-1.18.001 v1.3 — annotated-site count corrected from 9 to 8:** BC-1.18.001 is bumped to v1.3 in the same burst (three textual occurrences corrected: frontmatter v1.1 reason string, PC-9 body, Traceability Slide Types Affected table). Per-component `weight: Float` on weighted_composite is nested Map validation (T3 deferred, ADR-020 Decision 6) — not a top-level FieldDef site. The correct count is 8 annotated FieldDef sites across 8 slide types. |
| 2.25 | 2026-06-08 | product-owner | STORY-079 PO review complete — **Note (E-EXP-004) added** documenting the two production call sites sharing E-EXP-004 (`slideforge-diagrams` and `slideforge-pdf`) and the three sub-cases: (1) malformed SVG (usvg parse failure), (2) SEC-002 byte-size cap exceeded (CWE-400, MAX_SVG_BYTES=52,428,800), (3) SEC-001 nesting-depth cap exceeded (CWE-674, MAX_SVG_NESTING_DEPTH=64). Constant parity requirement documented. E-EXP-008 / E-EXP-004 stage distinction clarified. STORY-043 / STORY-079 provenance noted. No new error code allocated — E-EXP-004 coverage is confirmed sufficient for all three sub-cases per STORY-079 Design Note. BC-1.12.003 updated to v1.2 in the same burst. |
| 2.27 | 2026-06-08 | product-owner | STORY-088 adversary Pass-1 CRIT-1: **E-PAR-024 allocated** (`ParseError::NonStringListItem`) — broken, exit 1. Fires when the list-literal parser (`bullets: [...]` inline form or `@var ident = [...]` deck-level assignment) encounters a list item that is not a quoted string literal. Message: `non-string list item at <file>:<line>:<col>. List items must be quoted string literals; got <type>. Wrap the value in quotes to use it as a string.` The `<type>` placeholder names the actual token kind (e.g., `"integer"`, `"boolean"`, `"bare word"`). Error accumulated; parsing continues past the rejected item. This code supersedes the incorrect reuse of E-PAR-015 (hex color invalid) in the STORY-088 implementation — implementer must update `deck.rs` and `control_flow.rs` to emit `E-PAR-024`. Pre-registration collision check: E-PAR-024 confirmed free — highest existing E-PAR code was E-PAR-023 (STORY-082, v2.24). No other codes in taxonomy reference `"E-PAR-024"`. Traces to BC-1.01.002, CAP-001, STORY-088. |
| 2.26 | 2026-06-08 | product-owner | STORY-079 adversary Pass-1 OBS-1/OBS-2 — **E-EXP-004 Note corrected for accurate slideforge-pdf sibling description.** Two inaccuracies in the v2.25 Note introduced false parity between `slideforge-diagrams` and `slideforge-pdf`. **OBS-1 (implementation method):** The Note falsely claimed both call sites use "iterative stack-based DFS". Corrected: `slideforge-diagrams` is iterative (heap-allocated `Vec<(&Group, usize)>` stack; required by BC-1.12.003 inv-6); `slideforge-pdf` is recursive (`render_group` calls `render_group(g, surface, depth + 1)?`) — safe (depth-capped at 64, no overflow risk) but not iterative. **OBS-2 (cause string wording):** The Note falsely implied the depth cause strings are byte-identical. Corrected: both crates emit the same SIZE cause string verbatim; the DEPTH cause string differs by a trailing suffix — `slideforge-pdf` emits `"… possible DoS input — aborting SVG embed"` while `slideforge-diagrams` emits `"… possible DoS input"` (no suffix). The STORY-043/STORY-079 provenance paragraph updated to accurately state that only the constants and size cause string are shared verbatim. No error codes, constants, or severities changed. |
| 2.28 | 2026-06-08 | product-owner | Pass-5 IMP-1 spec correction: **E-PAR-023 exit code corrected 2→1.** E-PAR-023 (`ParseError::EmptySectionGroupName`) is a parse-stage error. Per BC-1.15.003 three-tier exit-code model (parse errors → exit 1; strict-mode validation/eval errors → exit 2), and consistent with every other E-PAR row in the taxonomy, E-PAR-023 must exit 1. The v2.24 allocation incorrectly stated exit 2. Corrected in: (1) E-PAR table row Exit column 2→1; (2) Note (E-PAR-023) exit rationale rewritten; (3) v2.24 changelog row annotated with correction reference. OBS-1 also fixed: message format in the E-PAR-023 table row updated to include the `E-PAR-023:` code self-prefix (aligning with the emitter convention used across E-PAR codes; the Note now documents this prefix requirement). BC-4.01.003 updated to v1.4 in the same burst (PC-7 and EC-010 exit 2→1; changelog entry added). |
| 2.29 | 2026-06-11 | product-owner | STORY-094 rendering-fix wave (REND-005, W-VAL-103 Route A): **W-VAL-103 content-drop severity promotion documented.** W-VAL-103 (`validate_fields` unknown-field warning) gains a context-sensitive severity path: when the unknown field key is `"shape"` or `"body"` on a slide type that does not support it (causing authored content to be silently dropped), W-VAL-103 is promoted from cosmetic to broken in strict mode (exit 2, no output). Non-content unknown fields remain cosmetic/exit-0 (unchanged). This aligns with the CLAUDE.md production-grade default (no silent dropping of authored content in strict mode) and DI-017 (all-or-nothing). Implementer action: STORY-098 (content-drop severity promotion in `validate_fields` for `"shape"` and `"body"` keys in strict mode). No new taxonomy code allocated. |
| 2.30 | 2026-06-11 | product-owner | STORY-094 rendering-fix wave, F-094-P2-003: **E-LAY-008 allocated** (`LayoutError::BulletsOnContentlessSlideType`) — user-authoring error fired at layout time when a `bullets:` field targets a slide type whose region map defines no Body or Generic Empty region (e.g., `title`, `closing`, `section_break`, `blank`). Severity: broken. Exit: 2 (strict mode). Source span required: points at the `bullets:` keyword in the authored .sf file. Message template: `[E-LAY-008] Slide '<slide_type>' at <file>:<line>:<col> has no content region for 'bullets'. Slide type '<slide_type>' defines no Body or Generic Empty region. Use a slide type with a body region (e.g. 'content', 'detail', 'bullets_only') or remove the 'bullets:' field.` Correction hint embedded in message. Distinction from `LayoutError::InvalidBoundingBox` (internal invariant breach) and E-VAL-101/W-VAL-103 (schema validation, Stage 5) documented in Note. Pre-registration collision check: E-LAY-008 confirmed free — highest existing E-LAY code was E-LAY-007 (v2.5). Traces to CAP-022. |
| 2.31 | 2026-06-11 | product-owner | STORY-098 adversary Pass-1 F-098-P1-003 (HIGH) + F-098-P1-007 (OBS adjudication): **E-LAY-003 reclassified `degraded` → `broken`; exit gate changed `strict-overflow` → `--warn-only`; trigger scope widened to cover missing `data:` field.** (1) Severity corrected from `degraded` to `broken` — BC-1.11.002 postcondition 2 specifies blocking error on empty data in default strict mode; DiagnosticSeverity::Error in STORY-098 implementation; the degraded/strict-overflow classification was incorrect and contradicted both the BC and the code. (2) Exit gate corrected: E-LAY-003 is NOT controlled by `[build].strict_overflow`; it is controlled by `--warn-only`. `strict-overflow` applies exclusively to canvas overflow (E-LAY-001). (3) Trigger scope widened (F-098-P1-007 adjudication, PO decision BINDING): a `slide chart:` block with NO `data:` field (missing data source) NOW triggers E-LAY-003 in addition to a `data:` binding that evaluates empty. Both conditions are equally broken — the chart cannot render meaningful content and previously rendered silently empty (the REND-010 defect class). The layout-preview allowance in `chart.rs` docs is INCORRECT and must be corrected by the implementer; layout previews must use `--warn-only`. Message template updated to add `[E-LAY-003]` self-prefix. Explanatory Note (E-LAY-003) added after E-LAY table. BC-1.11.002 updated to v1.2 in same burst (precondition 1 widened, description updated, new EC-005, version bump). |
| 2.32 | 2026-06-11 | product-owner | PO binding adjudication F-098-ADJ-BODY-CONTENT (STORY-098 three-way spec conflict): **W-VAL-103 content-drop scope clarified — `body` on `content` slide type is NOT a content-drop case.** The W-VAL-103 content-drop promotion (v2.29) fires only when the field key is genuinely UNKNOWN (not in the slide type's `known_fields()`). The `content` slide type now declares `body` in its `known_fields()` and `ContentSlideType::optional` — `body` on `content` is schema-valid and renders as prose in the content area (PPTX body placeholder / DOCX Normal). It never reaches the W-VAL-103 accumulation path. `body` on other types that do NOT declare `body` (e.g., `quote`, `title`, `stat_callout`) still triggers W-VAL-103 broken/exit-2 in strict mode (unchanged). W-VAL-103 Note section updated with "Critical boundary" paragraph. BC-3.03.002 updated to v1.3 (EC-007 reversed, Invariant 4 and postcondition 3 clarified). STORY-098 story spec updated to v1.2 (AC-002 and AC-003 revised). |
