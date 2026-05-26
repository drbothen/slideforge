---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-008
title: "Parser: @include, variants:, set rules, aliases"
epic: EPIC-02
wave: 1
points: 8
priority: P0
tdd_mode: strict
status: draft
# BC status: BCs are present — story may transition to ready after PO review
crate: slideforge-syntax
behavioral_contracts:
  - BC-1.01.004
  - BC-1.06.001
  - BC-1.06.003
  - BC-1.07.001
  - BC-1.07.002
  - BC-1.07.003
  - BC-1.07.004
  - BC-1.07.005
  - BC-1.08.001
  - BC-1.08.002
  - BC-1.08.003
  - BC-1.09.001
  - BC-1.09.002
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
blocks:
  - STORY-011
  - STORY-013
subsystems:
  - SS-01
target_module: slideforge-syntax
---

# STORY-008: Parser: @include, variants:, set rules, aliases

## Summary

Extend `slideforge-syntax` with four remaining structural DSL features:

1. **`@include "path.sf"`** — inline a local `.sf` file at parse time, resolving
   the path relative to the including file. Variable paths (`@include "{{ var }}/x.sf"`)
   resolve the variable before file lookup; missing files emit E-PAR-005 with the
   resolved (not template) path.
2. **`variants:` block** — declare named audience variants with `include_tags` and
   `exclude_tags`. Parse the variant block into `VariantNode` with `vars:`, `include_tags`,
   `exclude_tags`, and optional `inherits` fields.
3. **`set` rules** — parse `set <type>: <field> <value>` deck-scope defaults into
   `SetRule` nodes. Support `{{ expr }}` and `brand.*` references in the value position.
4. **`alias <name> = <type>: ...`** — parse alias declarations into `AliasNode`,
   expanding them in the AST so no alias references remain in the final tree.

Evaluation semantics (variant filtering, set-rule precedence application, alias
resolution at evaluation scope) are implemented in `slideforge-eval` (STORY-011/013).
This story is purely syntactic.

## Behavioral Contracts

| BC | Title | Scope in this story |
|----|-------|---------------------|
| BC-1.01.004 | Resolve @include directives and detect cycles — fail-closed | IncludeResolver cycle detection; E-PAR-006 on include cycle |
| BC-1.06.001 | @include resolves local .sf file and inlines at parse time | Parse-time resolution; source span attribution |
| BC-1.06.003 | @include with variable path reports resolved path on error | E-PAR-005 content format |
| BC-1.07.001 | variants: block with include_tags/exclude_tags produces filtered deck | Parse: `VariantNode` structure |
| BC-1.07.002 | Variant vars: overrides deck-level vars per 11-level precedence chain | Parse: `vars:` subblock inside variant |
| BC-1.07.003 | Detect and reject cyclic variant inheritance graph | Parse-time cycle detection in `inherits` chains |
| BC-1.07.004 | Reject --variant flag referencing undefined variant | Parse: register all variant names |
| BC-1.07.005 | Variant that excludes all slides produces warning in warn-only mode | Parse: `empty_result_severity` propagated from variant block |
| BC-1.08.001 | set <type>: <field> <value> applies as default to all instances | Parse: `SetRule` node |
| BC-1.08.002 | set rules support {{ }} interpolation and brand.* references | Parse: `SetRuleValue::Template` and `SetRuleValue::BrandRef` |
| BC-1.08.003 | brand.* references in set rules are evaluated after brand loading | Parse: brand ref stored as `BrandRef(path)` — no evaluation at parse time |
| BC-1.09.001 | alias <name> = <type>: defines named preset that resolves at parse time | Parse + immediate expansion in AST |
| BC-1.09.002 | Alias cannot add new fields — only preset existing fields | Parse-time field name validation against base type's known fields |

## Acceptance Criteria

- [ ] **AC-001** — `@include "header.sf"` (where header.sf contains one `slide title:` block)
  inlines the slide at the `@include` position; the final `DeckNode` contains the slide
  as if it were declared inline.
  (traces to BC-1.06.001 postcondition 1, 3)

- [ ] **AC-002** — Source spans for nodes originating from an included file cite the included
  file's `file_id` and path, not the entry file's `file_id`.
  (traces to BC-1.06.001 postcondition 2)

- [ ] **AC-003** — `@include "{{ client }}/template.sf"` with `client="acme"` and the file
  `acme/template.sf` absent emits E-PAR-005 with message `File not found: 'acme/template.sf'`
  (the resolved path), not `File not found: '{{ client }}/template.sf'`.
  (traces to BC-1.06.003 postcondition 1, invariant 2)

- [ ] **AC-004** — A `variants:` block with two named variants (`exec`, `internal`), each
  declaring `include_tags` and/or `exclude_tags`, parses into a `VariantsBlock` containing
  two `VariantNode` entries with their tag lists captured.
  (traces to BC-1.07.001 postcondition 1)

- [ ] **AC-005** — A variant declaring `vars: { color: "red" }` inside its block parses the
  variant's `vars` subblock and stores it in `VariantNode.vars`.
  (traces to BC-1.07.002 precondition 1)

- [ ] **AC-006** — A variant with `inherits: "exec"` that creates a cycle (exec inherits
  internal, internal inherits exec) produces E-VAR-001: "Cyclic variant inheritance
  detected: exec → internal → exec"; no `VariantsBlock` is produced.
  (traces to BC-1.07.003 postcondition — reject cyclic graph)

- [ ] **AC-007** — The parser registers all declared variant names in `DeckNode.variant_names:
  Vec<String>`. The `--variant` flag validation (rejecting undefined names) can be done
  by comparing the CLI arg against this list at build time.
  (traces to BC-1.07.004 precondition)

- [ ] **AC-008** — `set content: footer "Confidential"` parses to `SetRule { slide_type:
  "content", field: "footer", value: SetRuleValue::Template([TemplateChunk::Literal(
  "Confidential")]) }`.
  (traces to BC-1.08.001 postcondition 1)

- [ ] **AC-009** — `set content: footer "{{ brand.footer }}"` parses the value as
  `SetRuleValue::Template([TemplateChunk::Expr(Expr::FieldAccess { base: Ident("brand"),
  field: "footer" })])`, with the `brand.*` reference stored as-is (not evaluated).
  (traces to BC-1.08.002, BC-1.08.003 invariant 1 — brand refs are not evaluated at
  parse time)

- [ ] **AC-010** — `set unknown_type: footer "x"` produces E-PAR-007: "Unknown slide type
  'unknown_type'. Did you mean 'content'? (valid types: title, content, ...)".
  (traces to BC-1.08.001 edge case EC-001)

- [ ] **AC-011** — `alias exec_title = title:` + `footer "Internal"` defines an `AliasNode`
  that is fully expanded: a subsequent `slide exec_title: title "Hello"` is represented
  in the AST as `slide title:` with `footer = "Internal"` preset and `title = "Hello"`.
  (traces to BC-1.09.001 postcondition 2, 5)

- [ ] **AC-012** — `alias exec_title = title:` + `new_field "value"` produces E-PAR-011:
  "'new_field' is not a valid field of slide type 'title'. Aliases can only preset
  existing fields." (traces to BC-1.09.002 invariant 2)

- [ ] **AC-013** — `alias title = title:` (alias name collides with built-in type) produces
  E-PAR-006 (reserved keyword / built-in type collision).
  (traces to BC-1.09.001 edge case EC-001)

- [ ] **AC-014** — A static `@include` cycle (file A includes file B, which includes file A)
  produces E-PAR-006 with the cycle path shown (e.g., `Circular @include: a.sf → b.sf → a.sf`).
  The parse fails fast on cycle detection; no infinite loop or stack overflow occurs.
  (traces to BC-1.01.004 postcondition 1 — fail-closed on include cycle)

- [ ] **AC-015** — `#![forbid(unsafe_code)]`, zero `.unwrap()` in non-test code,
  `clippy::pedantic` clean. (traces to NFR-021, NFR-022, NFR-024)

- [ ] **AC-016** — All new public items have rustdoc; `cargo doc --no-deps` produces 0 warnings.
  (traces to NFR-023)

## Tasks

1. Implement `IncludeResolver` in `include.rs`:
   - Input: `@include "path"` node, current file's directory, `SourceMap`.
   - Action: read the included file, add it to `SourceMap`, parse it recursively, inline
     the resulting `DeckNode.items` at the include position.
   - Error: if path is not found, emit E-PAR-005 with resolved path.
   - Variable path: if path contains `{{ expr }}`, evaluate the expression with the
     current `vars:` scope (vars available at top of the including file); then do file
     lookup.
   - Cycle detection: maintain a `visited: HashSet<PathBuf>` across the recursive call
     stack; if the same canonical path appears twice, emit E-PAR-006 (include cycle).
     Note: deep cycle detection (across eval cycles) is an EVAL concern (BC-1.06.002
     is covered by STORY-013). Parse-time only detects static cycles.
2. Add `IncludeNode { path: Spanned<TemplateValue> }` to `ast.rs`. After resolution,
   `IncludeNode` is replaced by its inlined items; it does not appear in the final
   `DeckNode`. Add an intermediate `PreResolvedDeckNode` type if needed.
3. Implement `variants_block()` parser in `parser/variants.rs`:
   - `"variants:" INDENT (variant_decl)+ DEDENT`
   - `variant_decl ::= IDENT ":" INDENT (include_tags | exclude_tags | vars_block | inherits_decl)* DEDENT`
   - `include_tags ::= "include_tags:" "[" (IDENT ("," IDENT)*)? "]"`
   - `exclude_tags ::= "exclude_tags:" "[" (IDENT ("," IDENT)*)? "]"`
   - `inherits_decl ::= "inherits:" IDENT`
4. Implement cycle detection for `inherits` chains in `variants_block()`:
   - Build an adjacency map `name → inherits_name` during parse.
   - Run DFS; on back-edge, emit E-VAR-001 with the cycle path.
5. Extend `set_rule()` parser (already in STORY-006) to support `SetRuleValue::BrandRef`
   for `brand.*` field access expressions. Add `SetRuleValue` enum:
   `Template(Vec<TemplateChunk>)` | `BrandRef(Vec<String>)`.
6. Implement `alias_decl()` parser in `parser/alias.rs`:
   - `"alias" IDENT "=" IDENT ":" INDENT (field_line)+ DEDENT`
   - Register alias name in a `AliasRegistry` (local to the parse call).
   - Validate that each field in the alias body is a known field of the base type
     (consult `KNOWN_SLIDE_FIELDS` const map — hardcoded for all 31 types).
   - Emit E-PAR-011 for unknown fields; E-PAR-006 for alias name collision with built-in.
   - Expand the alias immediately: any `slide <alias_name>:` encountered after the
     declaration is rewritten to `slide <base_type>:` with preset fields merged.
7. Add `KNOWN_SLIDE_FIELDS: phf::Map<&str, &[&str]>` compile-time map for all 31 slide
   types → their known field names.
8. Write snapshot tests for: `@include` inlining, variant block, set rule with brand
   ref, alias expansion.
9. Write unit tests for all ACs.

## File List

- `crates/slideforge-syntax/src/include.rs` — `IncludeResolver`, `resolve_includes()` (new)
- `crates/slideforge-syntax/src/ast.rs` — updated: `VariantsBlock`, `VariantNode`,
  `AliasNode`, `SetRuleValue` added; `IncludeNode` (intermediate, pre-resolution)
- `crates/slideforge-syntax/src/parser/variants.rs` — `variants_block()`, cycle detect (new)
- `crates/slideforge-syntax/src/parser/alias.rs` — `alias_decl()`, `AliasRegistry` (new)
- `crates/slideforge-syntax/src/known_fields.rs` — `KNOWN_SLIDE_FIELDS` phf map (new)
- `crates/slideforge-syntax/src/parser/deck.rs` — updated: calls `variants_block()`,
  `alias_decl()`, dispatches `@include` to `IncludeResolver`
- `crates/slideforge-syntax/src/parser/mod.rs` — declare new submodules
- `crates/slideforge-syntax/tests/fixtures/include_header.sf` — included file
- `crates/slideforge-syntax/tests/fixtures/deck_with_include.sf` — entry file
- `crates/slideforge-syntax/tests/fixtures/variants_block.sf`
- `crates/slideforge-syntax/tests/fixtures/set_rule_brand.sf`
- `crates/slideforge-syntax/tests/fixtures/alias_decl.sf`

## Token Budget Estimate

| Item | Approx tokens |
|------|--------------|
| This story spec | ~5 000 |
| BC files (13 BCs) | ~9 500 |
| STORY-006 source (ast.rs, parser/deck.rs) | ~3 000 |
| STORY-007 source (control_flow.rs) | ~2 000 |
| Target source files to write | ~7 000 |
| Test files | ~4 000 |
| **Total** | **~30 000** |

Context budget: 30 000 / 200 000 ≈ 15% — within limit.

## Test Strategy

**Unit tests** (`#[cfg(test)] mod tests` inside each new module):

- `test_include_inlines_slide()`: parse deck with `@include`; assert included slide
  appears in `DeckNode.items` at the correct position.
- `test_include_span_attribution()`: slide from included file has `span.file_id` matching
  the included file, not the entry file.
- `test_include_variable_path_resolved_in_error()`: `@include "{{ client }}/x.sf"` with
  `client="acme"`, file absent → E-PAR-005 message contains `acme/x.sf`.
- `test_variants_two_variants()`: parse `variants:` with `exec` and `internal`; assert
  `DeckNode.variants.len() == 2`.
- `test_variant_includes_tags()`: `exec` variant with `include_tags: ["exec"]`; assert
  `VariantNode.include_tags == ["exec"]`.
- `test_variant_cycle_rejected()`: `exec` inherits `internal`, `internal` inherits
  `exec`; assert E-VAR-001 with cycle path.
- `test_variant_names_registered()`: parse deck with 3 variants; assert
  `DeckNode.variant_names` contains all 3 names.
- `test_set_rule_literal_value()`: `set content: footer "Conf"` → `SetRule` with
  `SetRuleValue::Template([Literal("Conf")])`.
- `test_set_rule_brand_ref()`: `set content: footer "{{ brand.footer }}"` → `SetRule`
  with template containing `FieldAccess { base: Ident("brand"), field: "footer" }`.
- `test_set_rule_unknown_type()`: `set unknown_type: footer "x"` → E-PAR-007.
- `test_alias_expansion()`: `alias exec_title = title:` + `footer "Internal"` followed
  by `slide exec_title: title "Hello"`; assert final AST contains `slide title:` with
  both `footer` preset and `title` explicit.
- `test_alias_unknown_field_rejected()`: alias presets `new_field` on `title` type →
  E-PAR-011.
- `test_alias_name_collides_with_builtin()`: `alias title = title:` → E-PAR-006.

**Snapshot tests**:

- `test_snapshot_include_inlining()`: entry deck with `@include`; snapshot merged AST.
- `test_snapshot_variants_block()`: variants fixture; snapshot `VariantsBlock`.
- `test_snapshot_set_rule_brand()`: set rule with brand ref; snapshot `SetRule`.
- `test_snapshot_alias_expanded()`: alias declaration + usage; snapshot expanded `SlideNode`.

## Dependencies

- **Depends on:** STORY-005 (lexer: `@include`, `variants`, `alias`, `set` as keyword
  tokens; string literal tokens for paths)
- **Depends on:** STORY-006 (base AST: `DeckNode`, `SetRule`, `FieldValue`, `Span`)
- **Depends on:** STORY-007 (template value parsing for `{{ var }}` in include paths
  and set rule values; `TemplateChunk` type)
- **Blocks:** STORY-011 (evaluator processes the variant and set-rule nodes produced
  here), STORY-013 (`@include` cycle detection at eval time uses the include graph
  built by the parse-time resolver)

## Dependency Anchor Justifications

- SS-01 owns this story's scope because SS-01 is the DSL Parser subsystem and
  `slideforge-syntax` is its sole crate per ARCH-INDEX Subsystem Registry.
- STORY-008 depends on STORY-007 because variable paths in `@include "{{ var }}/x.sf"`
  use `template_value()` from STORY-007; set rule values with `{{ }}` also require it.
- STORY-008 blocks STORY-013 because the eval-stage `@include` cycle detection (BC-1.06.002)
  requires the include graph that the parse-stage resolver builds.

## Architecture Compliance Rules

1. `slideforge-syntax` is **pure core** (SS-01). `IncludeResolver` performs file I/O
   (reading `.sf` files), which is effectful. This is acceptable because the parse stage
   is permitted to perform file reads (the file system is treated as an input, not an
   output). No network I/O or side effects beyond file reads.
2. `IncludeResolver` must receive a `file_loader: &dyn Fn(&Path) -> io::Result<String>`
   callback rather than calling `std::fs::read_to_string` directly — this enables test
   isolation via a mock loader.
3. `KNOWN_SLIDE_FIELDS` must be a `phf::Map` (perfect hash, compile-time) — no runtime
   HashMap. Add `phf` as a build-time dependency in `Cargo.toml`.
4. All new node types: `Hash + Eq + Clone + Debug`.
5. `AliasRegistry` is a parse-local structure (not stored in `DeckNode`); aliases are
   expanded eagerly and do not appear in the final AST.

**Forbidden dependencies:** same as STORY-006/007. Additionally:
- `std::fs` direct calls in `IncludeResolver` are forbidden; use the injected `file_loader`
  callback. Direct `std::fs` calls are allowed in `parser/mod.rs` `parse_file()` entry
  point only.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `chumsky` | `=0.10.1` | Extend parser combinators |
| `thiserror` | `=2.0.18` | Error derives |
| `miette` | `=7.6.0` | `Diagnostic` trait |
| `phf` | `=0.11.3` | `KNOWN_SLIDE_FIELDS` compile-time map (with `features = ["macros"]`) |

Note: `phf` is a build-time code-generation dependency. Add to `Cargo.toml`:
```toml
[dependencies]
phf = { version = "=0.11.3", features = ["macros"] }
```
This is a production dependency. Pin version: `phf = "=0.11.3"` (latest patch in 0.11.x; version 0.11.0 does NOT exist on crates.io — 0.11.2 is the first in this series).

## File Structure Requirements

```
crates/slideforge-syntax/
  src/
    include.rs            # IncludeResolver; file_loader callback pattern
    known_fields.rs       # KNOWN_SLIDE_FIELDS phf::Map<&str, &[&str]>
    parser/
      variants.rs         # variants_block() combinator
      alias.rs            # alias_decl(), AliasRegistry
      deck.rs             # updated: orchestrates all new parsers
    ast.rs                # updated: VariantsBlock, VariantNode, AliasNode, SetRuleValue
  tests/
    fixtures/
      include_header.sf   # single slide, used as includable file
      deck_with_include.sf
      variants_block.sf
      set_rule_brand.sf
      alias_decl.sf
```

## Previous Story Intelligence

From STORY-006/007: map all span origins through `SourceMap.file_id` at node construction
time (not post-hoc). For `IncludeResolver`, the included file must be registered with a
new `file_id` in `SourceMap` before its content is parsed, so that all node spans created
during included-file parsing carry the correct `file_id` automatically.

## Implementation Notes

### Architecture Note: Pure Core Boundary

`slideforge-syntax` is classified as "Pure core with effectful file-loading boundary." The
`IncludeResolver`'s `file_loader` callback is the single I/O boundary. All Kani proofs
(VP-001, VP-003, VP-009, VP-014) target properties WITHIN the pure core, never crossing
the file-loading boundary. The `file_loader: &dyn Fn(&Path) -> io::Result<String>` injection
pattern is mandatory precisely because Kani cannot model-check effectful I/O — the pure core
functions (cycle detection, AST inlining, span attribution) must be separately testable
without filesystem access.

### `@include` resolution algorithm

```
fn resolve_include(
    path_template: &TemplateValue,
    current_vars: &VarsScope,
    current_dir: &Path,
    source_map: &mut SourceMap,
    visited: &mut HashSet<PathBuf>,
    file_loader: &dyn Fn(&Path) -> io::Result<String>,
) -> Result<Vec<BlockItem>, Vec<SyntaxError>> {
    // 1. Evaluate path_template with current_vars (string only; no side effects)
    let resolved_path = current_dir.join(evaluate_template_string(path_template, current_vars));
    let canonical = resolved_path.canonicalize().map_err(|_| E-PAR-005(resolved_path))?;
    // 2. Cycle check
    if visited.contains(&canonical) {
        return Err(vec![E-PAR-006_include_cycle(canonical)]);
    }
    visited.insert(canonical.clone());
    // 3. Load + parse
    let src = file_loader(&canonical)?;
    let file_id = source_map.add_file(canonical.clone(), src.clone());
    let included_ast = parse_with_source_map(&src, file_id, source_map, visited, file_loader)?;
    visited.remove(&canonical);
    Ok(included_ast.items)
}
```

### Variant cycle detection algorithm

Build adjacency list from `inherits` declarations; run DFS with color marking (white/gray/black).
Gray → gray edge = cycle. Report cycle path by unwinding the DFS stack.

### Alias expansion

The `AliasRegistry` is a `HashMap<String, (String, Vec<FieldNode>)>` (alias name → base
type name + preset fields). After parsing an `alias` declaration, add to registry.
When `slide <alias_name>:` is encountered, look up the registry, merge preset fields with
any explicit fields (slide-level wins), and emit a `SlideNode` with the base type.

### Error codes introduced in this story

| Code | Message pattern | When |
|------|----------------|------|
| E-PAR-005 | `File not found: '<resolved_path>'` | `@include` file missing |
| E-PAR-007 | `Unknown slide type '<name>'. Did you mean '<suggestion>'?` | `set` rule with unknown type |
| E-PAR-011 | `'<field>' is not a valid field of slide type '<type>'. Aliases can only preset existing fields.` | alias body with unknown field |
| E-VAR-001 | `Cyclic variant inheritance detected: <a> → <b> → ... → <a>` | variant `inherits` cycle |

Note: E-PAR-006 (reserved keyword / name collision) is reused from BC-1.01.005 — do
NOT introduce a new code for alias-name-collision-with-builtin.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `@include "missing.sf"` (static path, file absent) | E-PAR-005 with literal path; accumulated |
| EC-002 | `@include "{{ undefined_var }}/x.sf"` | E-EVL-001 for undefined var; E-PAR-005 NOT emitted (variable error takes precedence) |
| EC-003 | `@include` in an included file (nested includes) | Resolved recursively; same cycle detection applies |
| EC-004 | `variants:` block with zero variants | E-PAR-002: "variants: block must declare at least one variant" |
| EC-005 | `set content: footer "x"` appears twice (same type+field) | Both `SetRule` nodes stored; last-wins is an EVAL concern (STORY-011) |
| EC-006 | Alias of an alias (transitive aliasing, e.g. `alias b = a:`, `alias a = title:`) | E-PAR-011: "alias cannot reference another alias; use the base type directly" — transitive aliasing is not supported in v1.0 |
