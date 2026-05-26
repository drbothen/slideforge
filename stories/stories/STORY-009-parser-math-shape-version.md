---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-009
title: "Parser: math delimiters, shape:, DSL versioning, diagnostics"
epic: EPIC-02
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-syntax
behavioral_contracts:
  - BC-1.01.005
  - BC-1.01.006
  - BC-1.09.001
  - BC-1.13.001
  - BC-3.04.002
verification_properties: []
nfr_refs:
  - NFR-021
  - NFR-022
  - NFR-023
  - NFR-024
depends_on:
  - STORY-005
  - STORY-006
  - STORY-007
  - STORY-008
blocks:
  - STORY-029
subsystems:
  - SS-01
target_module: slideforge-syntax
---

# STORY-009: Parser: math delimiters, shape:, DSL versioning, diagnostics

## Summary

Complete the `slideforge-syntax` parser with the remaining four surface-area concerns:

1. **DSL versioning gate** — `slideforge_version "N"` must be the first token sequence
   parsed; forward-incompatible versions are rejected immediately and fatally.
2. **Math mode delimiters** — `$...$` (inline) and `$$...$$` (display) toggle math mode
   inside field values. Inside math mode, `@{var}` is the interpolation syntax (not
   `{{ }}`). The lexer (STORY-005) already produces `MathStart`/`MathEnd` tokens; this
   story integrates them into field value parsing.
3. **`shape:` block** — parse a `shape:` block declaring a custom shape with `type`,
   `position`, `fill`, `text`, and `alt` fields into a `ShapeNode`.
4. **Reserved keyword enforcement** — `raw`, `@fn`, `@mixin`, `component`, `extends`,
   and ~25 more reserved keywords produce E-PAR-006 at any DSL position. Variable names
   in `vars:` blocks that collide with slide type keywords produce E-PAR-008.

This story depends on all four preceding parser stories and delivers the final
correctness gate before the parser is considered complete for Phase 1.

## Behavioral Contracts

| BC | Title | Scope in this story |
|----|-------|---------------------|
| BC-1.01.005 | Reject reserved keywords with named-feature error message | E-PAR-006 production for all ~30 reserved keywords |
| BC-1.01.006 | Reject variable names colliding with slide type keywords | E-PAR-008 from `vars:` keyword collision check |
| BC-1.09.001 | alias <name> = <type>: defines named preset that resolves at parse time | AC-013: alias name collision with reserved keyword → E-PAR-006 |
| BC-1.13.001 | slideforge_version "1" required; reject forward-incompatible | Version gate before all other parsing |
| BC-3.04.002 | raw keyword in user .sf files is rejected at parse time | E-PAR-009 for all `raw*` variants |

## Acceptance Criteria

- [ ] **AC-001** — A source file with `slideforge_version "1"` at the top parses normally;
  the version declaration appears in `DeckNode.version` as `Some(Spanned("1"))`.
  (traces to BC-1.13.001 postcondition 1)

- [ ] **AC-002** — A source file with `slideforge_version "2"` immediately emits E-PAR-010:
  "Version 2 not supported. This binary supports version 1."; exit 1; no further
  parsing occurs.
  (traces to BC-1.13.001 postcondition 3, invariant 2 — fail-fast)

- [ ] **AC-003** — A source file missing `slideforge_version` emits E-PAR-010 with hint:
  'Add `slideforge_version "1"` to deck metadata.' (traces to BC-1.13.001 postcondition 2)

- [ ] **AC-004** — `slideforge_version "1.2"` (minor version suffix) is accepted; major
  version "1" matches. (traces to BC-1.13.001 invariant 3)

- [ ] **AC-005** — `slideforge_version "2"` with `--warn-only` flag still fails with exit 1
  (forward-incompatible version is always fatal, not demoted by `--warn-only`).
  (traces to BC-1.13.001 postcondition 5)

- [ ] **AC-006** — `title "$x^2$"` (inline math) parses the field value as
  `FieldValue::Template([TemplateChunk::MathInline("x^2")])`.
  (traces to BC-1.01.005 — math mode is parsed without using reserved keywords)

- [ ] **AC-007** — `body "$$\\sum_{i=0}^{n} i$$"` (display math) parses as
  `TemplateChunk::MathDisplay("\\\\sum_{i=0}^{n} i")`.
  (traces to BC-1.01.005 — same; display math delimiter handled)

- [ ] **AC-008** — Inside a math block, `@{var}` parses as `MathInterp(Expr::Ident("var"))`;
  `{{ var }}` inside `$...$` parses as `TemplateChunk::Literal("{{ var }}")` (math mode
  disables text interpolation).
  (traces to BC-1.01.005 — mode-based parsing prevents keyword collision inside math)

- [ ] **AC-009** — `shape:` block with `type "rect"`, `position "10% 20%"`, `fill "#FF0000"`,
  `text "Label"`, `alt "A red rectangle"` parses to a `ShapeNode` with all five fields.
  (traces to BC-3.04.002 — `raw` is the forbidden alternative; `shape:` is the correct one)

- [ ] **AC-010** — `slide content: raw pptx: <p:sp/>` emits E-PAR-009:
  `'raw' keyword is not available in user .sf files at <file>:<line>:<col>. Use the shape: DSL instead.`
  (traces to BC-3.04.002 postcondition 1, invariant 1)

- [ ] **AC-011** — `raw` as a variable name (`vars: { raw: "data" }`) emits E-PAR-008
  (variable name collision with reserved keyword), NOT E-PAR-009.
  (traces to BC-3.04.002 edge case EC-002 — `raw` in vars: uses BC-1.01.006 code path)

- [ ] **AC-012** — `@fn compute(x):` emits E-PAR-006 naming `'@fn'` and the planned feature
  "user-defined functions (planned v2+)".
  (traces to BC-1.01.005 postcondition 1, invariant 2)

- [ ] **AC-013** — `vars: { chart: "my-data.json" }` emits E-PAR-008:
  `Variable name 'chart' collides with reserved keyword at <file>:<line>:<col>. Choose a different name.`
  (traces to BC-1.01.006 postcondition 1 — DEC-010)

- [ ] **AC-014** — `vars: { chart_data: "x" }` (suffix, not exact match) produces no error.
  (traces to BC-1.01.006 invariant — only exact matches rejected)

- [ ] **AC-015** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code, and
  `clippy::pedantic` clean. (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-016** — All new public items have rustdoc; `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

## Tasks

1. Implement the **version gate** at the start of the `deck()` parser:
   - Parse `slideforge_version STRING` as the first production before any other.
   - If the token sequence is missing, emit E-PAR-010 (with hint) and continue (version
     warning is non-fatal in some modes per BC-1.13.001 postcondition 2 — treat missing
     as a warning; treat wrong version as fatal).
   - If version string major ≠ "1", emit E-PAR-010 and STOP parsing (return immediately
     with the error; do not attempt to parse the rest of the file).
   - Store `DeckNode.version: Option<Spanned<String>>`.
2. Add `TemplateChunk::MathInline(String)` and `TemplateChunk::MathDisplay(String)` and
   `TemplateChunk::MathInterp(Expr)` variants to `template.rs`.
3. Implement math mode parsing in `template_value()`:
   - On `Token::MathStart` (single `$`): switch to math mode; collect raw chars until
     `Token::MathEnd`; parse `@{expr}` inside math mode as `MathInterp`.
   - On `Token::MathStartDisplay` (double `$$`): collect until `Token::MathEndDisplay`.
   - `{{ }}` inside math mode: treated as literal text (not interpolation).
4. Define `ShapeNode` in `ast.rs`:
   ```rust
   pub struct ShapeNode {
       pub shape_type: Option<Spanned<String>>,  // "rect", "circle", "arrow", etc.
       pub position: Option<Spanned<String>>,
       pub fill: Option<Spanned<String>>,
       pub text: Option<Spanned<TemplateValue>>,
       pub alt: Spanned<String>,  // required — compile error if absent (STORY-015)
   }
   ```
5. Implement `shape_block()` parser combinator in `parser/shape.rs`:
   - `"shape:" INDENT (shape_field)* DEDENT`
   - `shape_field ::= ("type" | "position" | "fill" | "text" | "alt") value NEWLINE`
   - Unrecognized field names emit E-PAR-002 (unexpected field) but continue parsing.
   - `alt` absence is not an error at parse time — it's a VALIDATION concern (STORY-015).
6. Implement the **reserved keyword table** in `keywords.rs`:
   - `RESERVED_KEYWORDS: phf::Map<&str, &str>` mapping keyword → planned feature description.
   - Keywords: `raw`, `raw_pptx`, `raw_html`, `raw_xml`, `raw_docx`, `@fn`, `@mixin`,
     `@while`, `component`, `extends`, `import_as`, `macro`, `template`, `override`,
     `abstract`, `interface`, `module`, `namespace`, `type` (non-alias use), `enum`,
     `struct`, `impl`, `trait`, `use`, `match`, `let`, `mut`, `ref`, `const`.
   - The `raw*` variants emit E-PAR-009 specifically; all others emit E-PAR-006.
7. Implement **variable name collision check** in `parser/deck.rs` when parsing `vars:`
   blocks: after collecting all key names, check each against the union of:
   - All 31 slide type keywords.
   - All directive keywords (`@for`, `@if`, `@data`, `@include`, etc.).
   - All structural keywords (`vars`, `set`, `alias`, `variants`, `section`).
   - All reserved keywords from the table above.
   Emit E-PAR-008 per collision; continue parsing (accumulated).
8. Write unit tests for all ACs. Write snapshot tests for: math inline, math display,
   shape block, reserved keyword error, variable name collision.

## File List

- `crates/slideforge-syntax/src/keywords.rs` — `RESERVED_KEYWORDS` phf map; reserved
  keyword classification functions (new)
- `crates/slideforge-syntax/src/ast.rs` — updated: `ShapeNode`, `TemplateChunk::Math*`
  variants, `DeckNode.version`
- `crates/slideforge-syntax/src/template.rs` — updated: math mode parsing integration
- `crates/slideforge-syntax/src/parser/shape.rs` — `shape_block()` combinator (new)
- `crates/slideforge-syntax/src/parser/deck.rs` — updated: version gate at entry,
  variable name collision check in `vars:` parser, `shape_block()` call in `slide_block`
- `crates/slideforge-syntax/src/parser/mod.rs` — declare `shape` module
- `crates/slideforge-syntax/tests/fixtures/version_gate.sf`
- `crates/slideforge-syntax/tests/fixtures/math_inline.sf`
- `crates/slideforge-syntax/tests/fixtures/shape_block.sf`
- `crates/slideforge-syntax/tests/fixtures/reserved_keyword.sf`
- `crates/slideforge-syntax/tests/fixtures/var_name_collision.sf`

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~4 000 |
| BC files (5 BCs) | ~3 500 |
| STORY-006/007/008 source (ast.rs, template.rs, parser/deck.rs) | ~5 000 |
| Target source files to write | ~4 500 |
| Test files | ~3 000 |
| **Total** | **~20 000** |

Context budget: 20 000 / 200 000 ≈ 10% — within limit.

## Test Strategy

**Unit tests** (in corresponding `#[cfg(test)]` blocks):

- `test_version_1_accepted()`: parse with `slideforge_version "1"` → no error.
- `test_version_2_rejected_fail_fast()`: parse with `slideforge_version "2"` → E-PAR-010;
  no other nodes in result.
- `test_version_missing_emits_warning()`: parse with no version → E-PAR-010 hint variant.
- `test_version_minor_suffix_accepted()`: parse `slideforge_version "1.2"` → accepted.
- `test_version_fatal_in_warn_only()`: parse with version "2" + `--warn-only` mode flag;
  assert result is still `Err` (version error is non-demotable).
- `test_math_inline_parsed()`: `title "$x^2$"` → `[MathInline("x^2")]`.
- `test_math_display_parsed()`: `body "$$E = mc^2$$"` → `[MathDisplay("E = mc^2")]`.
- `test_math_interp_in_math_mode()`: `title "$@{x} + 1$"` → `[MathInline(""), MathInterp(Ident("x")), MathInline(" + 1")]` (or equivalent split representation).
- `test_template_interp_ignored_in_math_mode()`: `title "${{ x }}$"` → `[MathInline("{{ x }}")]` (literal, not interpolated).
- `test_shape_block_all_fields()`: parse shape block with all 5 fields; assert `ShapeNode`.
- `test_shape_block_missing_optional_field()`: shape without `fill`; `fill: None`.
- `test_raw_pptx_rejected()`: `slide content: raw pptx: <p/>` → E-PAR-009.
- `test_raw_as_var_name()`: `vars: { raw: "x" }` → E-PAR-008 (not E-PAR-009).
- `test_at_fn_rejected()`: `@fn compute(x):` → E-PAR-006 naming `@fn`.
- `test_var_name_chart_rejected()`: `vars: { chart: "data" }` → E-PAR-008.
- `test_var_name_chart_data_accepted()`: `vars: { chart_data: "data" }` → no error.
- `test_comment_raw_word_ignored()`: `# raw OOXML here` as a comment → no error.

**Snapshot tests**:

- `test_snapshot_version_gate()`: version fixture; snapshot.
- `test_snapshot_math_inline()`: math inline fixture; snapshot.
- `test_snapshot_shape_block()`: shape block fixture; snapshot.

## Dependencies

- **Depends on:** STORY-005 (lexer: `MathStart`, `MathEnd`, `MathStartDisplay`,
  `MathEndDisplay`, `MathInterp` token variants; `Token::ReservedKeyword` variant)
- **Depends on:** STORY-006 (base parser infrastructure, `DeckNode`, `Span`, error types)
- **Depends on:** STORY-007 (`template_value()` parser for math block integration)
- **Depends on:** STORY-008 (`KNOWN_SLIDE_FIELDS` phf map for shape field validation;
  `AliasRegistry` for alias-name-collision check)
- **Blocks:** STORY-029 (math rendering — consumes `MathInline`/`MathDisplay` AST nodes)

## Dependency Anchor Justifications

- SS-01 owns this story's scope because SS-01 is the DSL Parser subsystem owning
  `slideforge-syntax` per ARCH-INDEX Subsystem Registry.
- STORY-009 depends on STORY-008 because the reserved keyword table and collision check
  for variable names must cross-reference `KNOWN_SLIDE_FIELDS` (built in STORY-008) and
  the alias registry (also STORY-008) to detect alias-name-as-variable collisions.
- STORY-009 blocks STORY-029 because the math renderer (EPIC-10) consumes `MathInline`
  and `MathDisplay` AST node variants defined here.

## Architecture Compliance Rules

1. `slideforge-syntax` is **pure core** (SS-01). No file I/O beyond what STORY-008
   introduced in `IncludeResolver`. No network calls. No runtime allocation of the
   keyword map (use `phf`).
2. `RESERVED_KEYWORDS` must be `phf::Map` — same requirement as `KNOWN_SLIDE_FIELDS`
   from STORY-008.
3. Version check must run before ANY other parser combinator is dispatched — this is
   enforced architecturally by making `deck()` start with `version_gate()` before any
   other alternative.
4. Math mode switching is a LEXER responsibility (STORY-005). The parser consumes
   `MathStart`/`MathEnd` tokens — it does not re-scan raw characters. Do not re-implement
   lexer logic in the parser.
5. `ShapeNode` must NOT contain a `raw_xml: Option<String>` field even as a stub —
   this would violate BC-3.04.002 invariant 3 at the type level.

**Forbidden dependencies:** same as STORY-006/007/008.
Additionally: the `shape_block()` parser must NOT call any function from
`slideforge-layout` or `slideforge-pptx` — shape layout is an export-time concern.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `chumsky` | `=0.10.1` | Version gate combinator, math mode token consumption |
| `thiserror` | `=2.0.18` | Error derives for E-PAR-009, E-PAR-010 |
| `miette` | `=7.6.0` | `Diagnostic` impls |
| `phf` | `=0.11.3` | `RESERVED_KEYWORDS` compile-time map |

## File Structure Requirements

```
crates/slideforge-syntax/
  src/
    keywords.rs           # RESERVED_KEYWORDS phf::Map + classify_keyword()
    parser/
      shape.rs            # shape_block() combinator
      deck.rs             # updated: version_gate() at entry, keyword collision check
    ast.rs                # updated: ShapeNode, MathInline/Display/Interp TemplateChunk variants
    template.rs           # updated: math mode integration
```

## Previous Story Intelligence

From STORY-006: the version gate must be implemented as the first `and_then()` call in
the `deck()` combinator chain, not as a pre-check outside chumsky. This ensures error
spans from the version gate reference the correct file position.

From STORY-007/008: math mode adds a new `TemplateChunk` variant to the already-extended
AST from STORY-007. Ensure the existing snapshot tests for `TemplateChunk` are updated
to accommodate the new variants (the insta `update` flag will regenerate; verify the
diff manually before committing).

## Implementation Notes

### Version gate implementation

```rust
fn version_gate<'src>() -> impl Parser<'src, &'src [Token], Option<Spanned<String>>, Rich<Token>> {
    just(Token::Ident("slideforge_version".into()))
        .then(select! { Token::Str(v) => v })
        .map_with(|(_, ver), meta| {
            let major = ver.split('.').next().unwrap_or(&ver);
            if major != "1" {
                // fatal: return Err immediately
                todo!("emit E-PAR-010 and abort")
            }
            Some(Spanned(ver, meta.span()))
        })
        .or_not()   // missing version → None + E-PAR-010 warning
}
```

Note: "fail fast" for wrong version means the `deck()` combinator chain should NOT
attempt error recovery on E-PAR-010 for wrong version — it must propagate as a fatal
error that short-circuits the entire parse.

### Reserved keyword table (partial)

| Keyword | Planned Feature | Error code |
|---------|----------------|-----------|
| `raw`, `raw pptx:`, `raw html:`, `raw xml:`, `raw docx:` | IR-internal only | E-PAR-009 |
| `@fn` | User-defined functions (v2+) | E-PAR-006 |
| `@mixin` | Style mixins (v2+) | E-PAR-006 |
| `@while` | Loop (intentionally absent; non-termination risk) | E-PAR-006 |
| `component` | Reusable components (v2+) | E-PAR-006 |
| `extends` | Type extension (v2+); hint: "Use alias instead." | E-PAR-006 |
| `@yield` | Generator-style (v3+) | E-PAR-006 |
| `macro` | Macro system (v2+) | E-PAR-006 |
| `abstract`, `interface`, `namespace` | Reserved; unused | E-PAR-006 |

### Math mode field value structure

```
"Hello $x^2$ world @{n} items"
→ [
    Literal("Hello "),
    MathInline("x^2"),
    Literal(" world "),
    Expr(Ident("n")),    ← text-mode interpolation resumes after MathEnd
    Literal(" items")
  ]
```

`{{ }}` inside `$...$` math block → `MathInline` chunk containing literal `{{ ... }}`
text. No interpolation. This prevents reserved DSL syntax from appearing inside math
expressions, per the two-interpolation-syntax design (ADR-009).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge_version "0"` | E-PAR-010 with migrate hint; fatal |
| EC-002 | `slideforge_version` appears after first slide block | E-PAR-010: "version declaration must appear in deck metadata at top of file"; fatal |
| EC-003 | `$` without closing `$` | E-PAR-004: "unclosed math delimiter"; remainder treated as literal |
| EC-004 | `$$` without closing `$$` | E-PAR-004: "unclosed display math delimiter" |
| EC-005 | `shape:` block with `raw` field name | E-PAR-009 for the `raw` keyword; shape block parsing continues |
| EC-006 | Comment `# raw pptx: ...` | No error — comments are stripped by the lexer before keyword checking |
| EC-007 | `vars: { title: "My Deck", chart: "data" }` (two collisions) | 2 E-PAR-008 diagnostics accumulated |
