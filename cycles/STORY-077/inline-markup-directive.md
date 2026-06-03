---
document_type: binding-directive
directive_id: DIR-077-002
issued_by: architect
issued_date: 2026-06-02
status: binding
targets: [test-writer, implementer, product-owner, story-writer]
story: STORY-077
bc: [BC-3.02.002]
supersedes: []
supplements: [DIR-077-001, DIR-077-001-A]
---

# Binding Directive DIR-077-002: Inline-Markup Parser Design for Section Sub-Block Field Values

## Status

BINDING. Governs the test-writer (Red Gate) and the implementer delivering STORY-077
AC-002 / BC-3.02.002 postcondition 8. DIR-077-001 and DIR-077-001-A established the
section-block parsing ownership split (STORY-078 = parser, STORY-077 = FieldValue::Inlines
upgrade + eval routing). This directive fills the gap neither prior directive addressed:
the INLINE MARKUP PARSER — the mechanism that converts raw text strings containing `**bold**`,
`` `code` ``, `[text](url)`, `{{ ref("id") }}`, etc. into `Vec<InlineNode>`.

This directive does NOT supersede DIR-077-001 or DIR-077-001-A. It supplements them with
binding decisions on all unresolved inline-markup questions.

---

## 1. Inline Syntax for v1 (Canonical Choices — Grounded in Q8 Decision)

The authoritative syntax table is from `q4-q15-decisions.md` Q8 (LOCKED decision). The
following is the canonical v1 inline markup syntax. Nothing here is invented — every
delimiter is copied verbatim from Q8.

| DSL Syntax | Effect | InlineNode Variant | Notes |
|---|---|---|---|
| `**text**` | Bold | `InlineNode::Bold(Vec<InlineNode>)` | CommonMark-compatible delimiter pair |
| `_text_` | Italic | `InlineNode::Italic(Vec<InlineNode>)` | Single-underscore pair; BOTH open and close `_` must be flanking (see Disambiguation Rule 2a); double-underscore is NOT supported in v1 |
| `` `text` `` | Code span | `InlineNode::Code(Arc<str>)` | Single-backtick pair; content is verbatim (no further parsing inside) |
| `[text](url)` | Hyperlink | `InlineNode::Link { text: Vec<InlineNode>, url: Arc<str> }` | CommonMark link syntax; URL scheme must be on the allowlist (http, https, mailto) — see E-PAR-022 |
| `$math$` / `$$math$$` | Math inline / display | `InlineNode::Math(MathNode)` | Already handled by `TemplateChunk::MathInline` / `MathDisplay`; see Section 3 |
| `{{ footnote("...") }}` | Footnote | `InlineNode::Footnote(Vec<InlineNode>)` | Built-in pseudo-function in {{ }} expression context |
| `{{ ref("id") }}` / `{{ figref(n) }}` | Cross-reference | `InlineNode::Xref(Arc<str>)` | Built-in pseudo-functions; id is the target identifier |
| `^text^` | Superscript | `InlineNode::Superscript(Vec<InlineNode>)` | Caret pair |
| `~text~` | Subscript | `InlineNode::Subscript(Vec<InlineNode>)` | Single-tilde pair |
| `~~text~~` | Strikethrough | `InlineNode::Strikethrough(Vec<InlineNode>)` | Double-tilde pair; must be checked before single-tilde |
| `==text==` | Highlighted | `InlineNode::Highlight(Vec<InlineNode>)` | Double-equals pair |
| (plain text) | Unstyled text | `InlineNode::Plain(Arc<str>)` | Any text not matching the above |

**Total: 11 markup forms in v1 (excluding plain fallback). The InlineNode enum has 12 variants
total (including Plain), all of which exist in `slideforge-types::InlineNode` as of this
writing.**

### Disambiguation Rules

1. `~~del~~` must be tried BEFORE `~sub~`. The parser consumes `~~` greedily when two
   consecutive tildes appear. A single `~` that is not the start of `~~` is subscript.

2. `**bold**` uses double-asterisk. Single-asterisk italic is NOT supported (only `_italic_`
   uses single delimiter). This avoids ambiguity with list bullets and arithmetic expressions.

2a. **`_italic_` — CommonMark-style bilateral flanking (AMENDED 2026-06-03, supersedes
    original "open-side left-flanking only" wording; authorized by OBS-P24-A human decision).**

    A `_` character acts as an ITALIC OPENER only when it is a **left-flanking delimiter**:
    the byte immediately BEFORE the `_` must NOT be an ASCII alphanumeric character or `_`
    (word-character). This is the existing open-side guard already implemented in
    `scan_template_chunks` (see the `prev_is_word` check in the `_` branch).

    A `_` character acts as an ITALIC CLOSER only when it is a **right-flanking delimiter**:
    the byte immediately AFTER the `_` must NOT be an ASCII alphanumeric character or `_`
    (word-character). This is the NEW close-side guard added by this amendment.

    Both guards must pass for the `_` to open or close an italic span. A `_` that fails
    either guard is treated as a literal character.

    **Precise close-side flanking condition for the implementer:**
    When `scan_template_chunks` is scanning inside an italic span (i.e., with
    `close_on = Some("_")`), the close-on check at the top of the loop currently fires
    whenever `s[pos..].starts_with("_")`. After this amendment, before returning the
    italic close, the implementer must additionally check: is the byte at `pos + 1`
    (i.e., the byte immediately following the closing `_`) an ASCII alphanumeric or `_`?
    If YES — word-internal `_` — do NOT treat it as the italic closer; advance `pos` by 1
    and continue scanning (treating the `_` as literal). If NO — non-word character or end
    of string — treat it as the italic closer as before.

    Condition in code terms:
    ```
    let next_is_word = s.as_bytes().get(pos + 1)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
    if next_is_word {
        // Word-internal `_` — not an italic closer; advance as literal.
        pos += 1;
        continue;
    }
    // Passes right-flanking guard — treat as italic closer.
    ```

    This check is applied inside the `close_on = Some("_")` branch in
    `scan_template_chunks`, which is also the location where the `close_on` match
    fires for ALL recursive italic scans. The existing `prev_is_word` check on the
    OPEN side is unchanged.

    **Example of the new expected parse:**

    Input: `_apply file_path here_`

    - `_` at position 0: prev char = none (start of string) → NOT word-char → left-flanking
      check PASSES → open italic.
    - Recursive scan of `apply file_path here_` with `close_on = Some("_")`:
      - Scans `apply file` as literal.
      - Reaches `_` at `file_path` (the `_` between `file` and `path`):
        - Right-flanking check: next byte is `p` (alphanumeric) → `next_is_word = true`
        → SKIP (word-internal `_`; advance as literal).
      - Continues scanning `path here`.
      - Reaches `_` at end: next byte = none (end of string) → `next_is_word = false`
        → right-flanking check PASSES → close italic.
    - Result: `Italic([Plain("apply file_path here")])` — ONE italic node containing the
      full text with the internal underscore preserved as a literal character.

    **Contrast with pre-amendment behavior (now superseded):**
    The pre-amendment parser had NO close-side guard, so `_apply file_path here_` would
    close italic at the word-internal `_` in `file_path`, producing
    `Italic([Plain("apply file")])` followed by `Plain("path here_")` — incorrect.

3. `` `code` `` — content between single-backtick delimiters is verbatim. No further inline
   markup is processed inside a code span (same rule as CommonMark). `{{ }}` interpolation
   does NOT fire inside a code span.

4. `[text](url)` — the `text` portion IS further processed for nested inline markup
   (e.g., `[**bold link**](url)` is valid). The `url` portion is verbatim text. The
   URL scheme must be on the v1 allowlist (`http`, `https`, `mailto`) — see E-PAR-022 in
   the error taxonomy. Relative paths, anchor links, and unrecognized schemes are rejected
   at parse time.

5. `{{ ref("id") }}` and `{{ figref(n) }}` are EXPRESSION-form cross-references, not
   standalone delimiters. They are recognized as special `{{ expr }}` forms during the
   `{{ }}` interpolation pass. The eval stage converts `Expr::Call { func: "ref", arg: "id" }`
   to `InlineNode::Xref(Arc::from("id"))`. The parser does NOT need a special case for `ref`
   — the expression parser handles it as a function call.

6. `{{ footnote("...") }}` follows the same pattern as `ref`: recognized as an expression-form
   call during interpolation; the eval stage converts it to `InlineNode::Footnote(...)`.

7. `$...$` and `$$...$$` are already parsed by `template_value()` as `TemplateChunk::MathInline`
   and `TemplateChunk::MathDisplay`. The inline-to-InlineNode conversion (Section 3) maps these
   to `InlineNode::Math(MathNode { latex, display: false/true })`.

**The Forbidden Pattern is absolute:** Bold is `InlineNode::Bold(Vec<InlineNode>)` — never a
prefix string like `"**header**"` passed as `InlineNode::Plain`. CLAUDE.md explicitly lists
string-prefix bold as a forbidden pattern (R1 finding).

---

## 2. Structural Model — InlineNode Composition

`InlineNode` in `slideforge-types` is already the correct target type. It is recursive:
`Bold(Vec<InlineNode>)` — bold content may contain italic, xref, etc. This is correct and
intentional. The inline markup parser must produce exactly this type — no intermediate
representation.

### Nesting Rules

These rules are BINDING for both parser and for test-writer Red Gate tests:

| Container | May contain | May NOT contain |
|---|---|---|
| `Bold(children)` | Plain, Italic, Code, Link, Math, Xref, Superscript, Subscript, Strikethrough, Highlight, Footnote | Another Bold (unnested — a closing `**` ends the bold span) |
| `Italic(children)` | Plain, Bold, Code, Link, Math, Xref, Superscript, Subscript, Strikethrough, Highlight, Footnote | Another Italic at the same level |
| `Code(str)` | (verbatim content only — no InlineNode children) | Any inline markup |
| `Link { text, url }` | Any InlineNode in `text`; plain str in `url` | Link inside Link |
| `Footnote(children)` | Plain, Bold, Italic, Code, Link, Math, Xref, Superscript, Subscript | Another Footnote |
| `Superscript`, `Subscript`, `Strikethrough`, `Highlight` | Plain, Bold, Italic, Code | Further nesting of same type |

**Adjacent spans:** Two adjacent bold spans (`**a** **b**`) produce TWO separate `Bold` nodes.
The parser does not merge adjacent runs of the same type.

**Empty spans:** `**` immediately followed by `**` (empty bold) is a parse error (E-PAR-NNN;
error code to be assigned in the error taxonomy). Similarly for all other delimiter pairs.
Error accumulation applies — empty-span error is non-fatal; surrounding text continues.

**Unclosed spans:** An unclosed `**` at the end of the text block is a parse error (E-PAR-NNN).
The error carries a span pointing to the opening `**`. Error accumulation applies — the
unclosed content is treated as Plain text and parsing continues.

---

## 3. WHERE Parsing Happens — The Architectural Decision

### The Crate Dependency Constraint

`slideforge-syntax` does NOT depend on `slideforge-types` (verified in `Cargo.toml`).
`slideforge-types::InlineNode` is not accessible from within `slideforge-syntax`. Therefore:

**`slideforge-syntax` CANNOT directly produce `Vec<InlineNode>`.** Adding a dependency
`slideforge-syntax → slideforge-types` would introduce a circular dependency risk (types
is the foundation layer with zero deps; syntax depends on chumsky only). This coupling must
NOT be introduced.

### Decision: Two-Phase Architecture — Parse-Time Markup Representation + Eval-Time InlineNode Production

The inline markup parser runs in TWO phases:

**Phase 1 — Parse time (`slideforge-syntax`): `template_value()` extended with inline markup chunks.**

Extend `TemplateChunk` in `slideforge-syntax/src/template.rs` with new variants that
represent inline markup structure without referencing `slideforge-types`:

```rust
pub enum TemplateChunk {
    // EXISTING (unchanged):
    Literal(String),
    Expr(Expr),
    MathInline(String),
    MathDisplay(String),
    MathInterp(Expr),

    // NEW — inline markup (added by STORY-077 parser extension work):
    Bold(Vec<TemplateChunk>),
    Italic(Vec<TemplateChunk>),
    Code(String),          // verbatim; no inner chunks
    Link {
        text: Vec<TemplateChunk>,
        url: String,
    },
    Superscript(Vec<TemplateChunk>),
    Subscript(Vec<TemplateChunk>),
    Strikethrough(Vec<TemplateChunk>),
    Highlight(Vec<TemplateChunk>),
    // Note: Footnote and Xref are NOT represented as TemplateChunk variants.
    // They arrive via {{ footnote("...") }} / {{ ref("id") }} as TemplateChunk::Expr
    // (function-call expressions), which the eval stage converts to InlineNode::Footnote
    // and InlineNode::Xref. No new TemplateChunk variant is required for those two.
}
```

These new `TemplateChunk` variants mirror `InlineNode` structurally but use only
`TemplateChunk` children (self-referential, no cross-crate type). This preserves the
`#[derive(Debug, Clone, PartialEq, Eq, Hash)]` requirement on `TemplateChunk` (required
for comemo Hash compatibility per ADR-013).

**Phase 2 — Eval time (`slideforge-eval`): TemplateChunk sequence → Vec<InlineNode>.**

A new pure function `chunks_to_inline_nodes(chunks: &[TemplateChunk], env: &Env, sink: &mut DiagnosticSink) -> Vec<InlineNode>` in `slideforge-eval` performs the conversion:
- `TemplateChunk::Literal(s)` → `InlineNode::Plain(Arc::from(s))`
- `TemplateChunk::Bold(children)` → `InlineNode::Bold(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Italic(children)` → `InlineNode::Italic(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Code(s)` → `InlineNode::Code(Arc::from(s))`
- `TemplateChunk::Link { text, url }` → `InlineNode::Link { text: chunks_to_inline_nodes(text, ...), url: Arc::from(url) }`
- `TemplateChunk::MathInline(latex)` → `InlineNode::Math(MathNode { latex: Arc::from(latex), display: false, span: ... })`
- `TemplateChunk::MathDisplay(latex)` → `InlineNode::Math(MathNode { latex: Arc::from(latex), display: true, span: ... })`
- `TemplateChunk::Superscript(children)` → `InlineNode::Superscript(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Subscript(children)` → `InlineNode::Subscript(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Strikethrough(children)` → `InlineNode::Strikethrough(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Highlight(children)` → `InlineNode::Highlight(chunks_to_inline_nodes(children, ...))`
- `TemplateChunk::Expr(Expr::Call { func: "ref", args: [Expr::Str(id)] })` → `InlineNode::Xref(Arc::from(id))`
- `TemplateChunk::Expr(Expr::Call { func: "figref", args: [Expr::Num(n)] })` → `InlineNode::Xref(Arc::from(format!("fig-{n}")))`
- `TemplateChunk::Expr(Expr::Call { func: "footnote", args: [Expr::Str(text)] })` → `InlineNode::Footnote(vec![InlineNode::Plain(Arc::from(text))])`
- `TemplateChunk::Expr(other)` → evaluate `other` to string via `eval_expr_to_string`, wrap as `InlineNode::Plain`
- `TemplateChunk::MathInterp(expr)` → evaluate `expr` to string, produce `InlineNode::Plain(Arc::from(s))` (math interp is already inside MathInline/MathDisplay and does not need a separate InlineNode)

The output of `chunks_to_inline_nodes` is stored as `slideforge_types::FieldValue::Inlines(nodes)`
on `SectionBlock.body[key]`.

### Integration Point for STORY-077 (section sub-block detail:/report:)

When STORY-078 has parsed a section block, the `FieldNode` for a `detail:` or `report:`
sub-block holds a `Spanned<slideforge_syntax::FieldValue::Template(Vec<TemplateChunk>)>`.
At the point where STORY-077 produces `RegisteredContent` for the section:

1. Extract the `Vec<TemplateChunk>` from the `FieldValue::Template`.
2. Call `chunks_to_inline_nodes(chunks, env, sink)` to produce `Vec<InlineNode>`.
3. Store as `slideforge_types::FieldValue::Inlines(nodes)` on `SectionBlock.body[key]`.
4. `extract_section_register_content` reads `FieldValue::Inlines` and produces the
   `RegisteredContent { register, content: nodes }` entry.

This is the minimum correct implementation. There is NO intermediate representation; the
conversion is a single function call at eval time.

### {{ }} Interpolation Composes With Inline Markup

The `template_value()` combinator already handles `{{ expr }}`. The inline markup parsing
extension sits at the SAME level as `{{ }}` processing — both are post-lexer, scanning
the raw string content. Composition is natural:

`**{{ client_name }} is bold**` → `TemplateChunk::Bold([TemplateChunk::Expr(Expr::Ident("client_name")), TemplateChunk::Literal(" is bold")])`

At eval time: `InlineNode::Bold([InlineNode::Plain(Arc::from("Acme is bold"))])` after
evaluating `client_name = "Acme"`. The `{{ }}` expression is resolved INSIDE the Bold children.

`{{ client_name }}` that evaluates to a string containing `**bold**` is NOT further
processed for inline markup. The resolved string is treated as Plain text. Inline markup
is parsed from DSL source, not from dynamically resolved values. This rule prevents
security and correctness issues from user-data driving markup parsing.

### Math Mode Disables Text Interpolation (Unchanged)

`$...$` and `$$...$$` are already handled by `template_value()`. Inside a math region:
- `{{ var }}` is treated as literal LaTeX text (existing rule, unchanged).
- `@{var}` is math-mode interpolation (existing rule, unchanged).
- `**bold**` inside a math region is treated as literal LaTeX text. Math mode disables all
  text-mode inline markup.
This rule requires no implementation change — `template_value()` already treats the content
between `$` delimiters as verbatim (except for `@{var}`).

---

## 4. STORY-077 Scope Boundary — Section Sub-Block vs. Slide-Level

### In Scope for STORY-077

The inline markup parser (Phase 1 + Phase 2) is required to deliver AC-002. The scope is:

- `TemplateChunk` new variants (Bold, Italic, Code, Link, Superscript, Subscript,
  Strikethrough, Highlight) added to `slideforge-syntax/src/template.rs`.
- The inline markup scanning pass added to `template_value()` in
  `slideforge-syntax/src/parser/template.rs`. This combinator is already called by the
  STORY-078 section-block parser for sub-block values. Adding inline markup scanning here
  makes the parser produce the correct `TemplateChunk` sequence for section sub-blocks.
- `chunks_to_inline_nodes` function added to `slideforge-eval`.
- The conversion called at section-sub-block evaluation time (detail:/report:) producing
  `slideforge_types::FieldValue::Inlines`.
- Unit tests for all 11 markup forms in section body context.
- Snapshot tests for section sub-block parse output.

### Slide-Level Inline Markup — OUT OF SCOPE FOR STORY-077

Slide-level fields (e.g., `title`, `bullets`, `body` on slide content blocks) currently
produce `TemplateChunk::Literal` for markup-containing strings. They are NOT upgraded to
`FieldValue::Inlines` in STORY-077.

**Rationale for deferral:** The `template_value()` change (Phase 1) affects ALL call sites,
including slide-level fields. Once `TemplateChunk::Bold` etc. exist, the parser will
correctly produce them for slide field values too. However, the EVAL stage conversion of
slide fields to `FieldValue::Inlines` requires changes to `eval_slide_node` and the `Slide`
field representation. That change is significantly larger (every slide field consumer
must handle `FieldValue::Inlines`), touches the layout engine and all five exporters, and is
not required to satisfy AC-002 (which is specifically about section sub-block field values).

**The boundary is cleanly consistent:** The `template_value()` parser change is shared code
and ships in STORY-077. The EVAL-stage conversion (`chunks_to_inline_nodes`) is called
only for section sub-block values in STORY-077. Slide-level conversion is deferred.

**This leaves a temporary inconsistency:** After STORY-077, slide fields that contain `**bold**`
will parse to `TemplateChunk::Bold` (correct) but the eval stage will not convert them to
`InlineNode::Bold` — they will be evaluated through the existing `eval_field_value_to_value`
path which produces `Value::Str` (treating the `TemplateChunk::Bold` content as flat text).
This is documented below as the BC/story amendment that must be flagged.

### Follow-Up Story — Slide-Level Inline Markup (Required Before v1.0)

A follow-up story is MANDATORY before v1.0 to close the inconsistency. It covers:

1. Eval-stage conversion of slide field values that are `FieldValue::Template` with inline
   markup chunks → `slideforge_types::FieldValue::Inlines`.
2. Layout engine updates to consume `FieldValue::Inlines` for fields where inline structure
   matters (bullets, body text, title for DOCX/PDF).
3. All exporter updates (PPTX, DOCX, PDF, HTML) to render `InlineNode::Bold` etc. as
   structural formatting (OOXML `<a:rPr b="1"/>`, HTML `<strong>`, PDF bold run).
4. PPTX-specific note: PPTX title fields are single-run; only bullet/body text can carry
   inline formatting in the OOXML model. The layout engine must enforce this constraint.

This story should be created by the story-writer and added to Wave 4 or 5. It is a
BLOCKING prerequisite for the v1.0 Quality Bar (spec convergence: "bold renders as bold
in all output formats, not just DOCX/PDF section bodies").

---

## 5. Error Handling — Malformed Inline Markup

Per the error accumulation convention (CLAUDE.md, Q23 LOCKED): the parser NEVER fails on
first error. Malformed markup accumulates an error and parsing continues.

### Rules (Binding)

| Malformed Input | Error Class | Severity | Recovery |
|---|---|---|---|
| Unclosed `**bold` (no closing `**`) | E-PAR-NNN (inline-bold-unclosed) | Fatal | Treat content through end of string as Bold, emit error with span at opening `**`; parsing continues |
| Empty span `****` | E-PAR-NNN (inline-empty-span) | Fatal | Skip the empty span, emit error with span at `****` location; parsing continues |
| Unclosed `_italic`, `` `code ``, `[link` | Same class as unclosed bold | Fatal | Same recovery pattern as unclosed bold |
| `{{ ref("") }}` (empty xref id) | E-PAR-NNN (inline-xref-empty-id) | Fatal | Emit error; produce no InlineNode for this expression; parsing continues |
| Unknown `{{ func("...") }}` call | (evaluated by eval stage — not a parse-time error) | Eval fatal | See eval error taxonomy |

**Span requirement:** Every inline markup error MUST carry a `Span` pointing to the
opening delimiter of the malformed construct. The error message MUST include a correction
hint (per CLAUDE.md error handling conventions). Example:

```
E-PAR-XXX: unclosed bold delimiter `**` at line 3, column 12.
  Hint: add a closing `**` after the bold text, e.g., `**bold text**`.
```

**Graceful vs. fatal:** All malformed inline markup errors are FATAL in strict mode (the
default) and WARNINGS in `--warn-only` mode. This aligns with the project-wide three-tier
severity model (Q19 LOCKED). The partial AST with the error sentinel is still produced —
error accumulation proceeds.

---

## 6. Plugin-Surface Alignment — InlineFormat Trait (Surface #10)

### Current State

`InlineFormat` trait (surface #10 from q3-decision-final.md) is defined in
`slideforge-plugin-api/src/traits/inline_format.rs`. It governs OUTPUT formatting — converting
an `InlineNode` to OOXML, HTML, or Markdown string. It is the RENDERING surface.

The built-in formatter is "built-in plugins ARE the test suite." The `InlineFormat` trait
is already defined and wired.

### What This Directive Adds

This directive adds an INLINE PARSER — a parse-time capability that produces `InlineNode`
from DSL source text. This is a DIFFERENT concern from `InlineFormat` (which goes the
other direction: InlineNode → output string).

**Decision: The v1 inline markup parser does NOT go through a plugin trait.**

Rationale:
1. The Q8 decision explicitly locks the syntax table. There is no extensibility requirement
   for inline markup syntax in v1.0. The "v2: `{{ color("red", "text") }}`" note in Q8 is
   the only future extension — and it uses `{{ }}` expression syntax, not a new delimiter.
2. The parser is classified as "core engine — fixed grammar, not extensible" in q3-decision-final.md.
   The inline markup syntax is part of the grammar, not part of the plugin surface.
3. Adding a plugin trait for inline markup PARSING would require the plugin system to be
   active at parse time, which violates the pure-core parser boundary.

**Future v2 path:** When `{{ color("red", "text") }}` is added in v2, it arrives as an
expression-form `{{ expr }}` that the existing `{{ }}` interpolation machinery handles.
The `chunks_to_inline_nodes` eval function is extended with a new case for the `color`
call. No new plugin trait is needed for this pattern.

**The `InlineFormat` trait (surface #10) is not modified by STORY-077.** The built-in
`InlineFormat` implementation already handles all 12 `InlineNode` variants for Ooxml, Html,
and Markdown output. STORY-077 creates the `InlineNode` instances that flow into this
existing rendering path.

---

## 7. Files Changed by STORY-077 (Inline Markup Work)

| File | Change | Reason |
|---|---|---|
| `crates/slideforge-syntax/src/template.rs` | Add Bold, Italic, Code, Link, Superscript, Subscript, Strikethrough, Highlight variants to `TemplateChunk` | Phase 1: parse-time markup representation |
| `crates/slideforge-syntax/src/parser/template.rs` | Extend `template_value()` to recognize and parse inline markup delimiters; produce new `TemplateChunk` variants | Phase 1: the inline markup scanning pass |
| `crates/slideforge-eval/src/register_routing.rs` | Add `chunks_to_inline_nodes()` function; call it when converting section sub-block detail:/report: values | Phase 2: eval-time conversion to InlineNode |
| `crates/slideforge-eval/src/eval.rs` | (Minimal: only if `eval_section_nodes` needs to call the new function directly) | Wiring if needed |

No changes required to:
- `slideforge-types` (InlineNode already exists with the correct 12 variants)
- `slideforge-plugin-api` (InlineFormat trait unchanged)
- Any exporter crate (exporter changes deferred to follow-up story)

---

## 8. Test-Writer (Red Gate) Requirements

The test-writer MUST write failing tests that drive ALL of the following before
the implementer begins. These tests fail until the implementation is complete.

### Required Red Gate Tests in `crates/slideforge-syntax/`

**Template chunk parsing (new TemplateChunk variants):**

1. `test_bold_chunk_produced` — `"**hello**"` → `vec![TemplateChunk::Bold([TemplateChunk::Literal("hello")])]`
2. `test_italic_chunk_produced` — `"_hello_"` → `vec![TemplateChunk::Italic([TemplateChunk::Literal("hello")])]`
3. `test_code_chunk_produced` — `` "`fn foo()`" `` → `vec![TemplateChunk::Code("fn foo()")]`
4. `test_link_chunk_produced` — `"[click here](https://example.com)"` → `vec![TemplateChunk::Link { text: [TemplateChunk::Literal("click here")], url: "https://example.com" }]`
5. `test_superscript_chunk_produced` — `"^2^"` → `vec![TemplateChunk::Superscript([TemplateChunk::Literal("2")])]`
6. `test_subscript_chunk_produced` — `"~n~"` → `vec![TemplateChunk::Subscript([TemplateChunk::Literal("n")])]`
7. `test_strikethrough_before_subscript` — `"~~del~~ ~sub~"` → `vec![TemplateChunk::Strikethrough([...]), TemplateChunk::Literal(" "), TemplateChunk::Subscript([...])]`
8. `test_highlight_chunk_produced` — `"==highlight me=="` → `vec![TemplateChunk::Highlight([TemplateChunk::Literal("highlight me")])]`
9. `test_bold_with_interpolation` — `"**{{ client }}**"` → `vec![TemplateChunk::Bold([TemplateChunk::Expr(Expr::Ident("client"))])]`
10. `test_nested_bold_italic` — `"**_bold italic_**"` → `vec![TemplateChunk::Bold([TemplateChunk::Italic([TemplateChunk::Literal("bold italic")])])]`
11. `test_code_span_no_inner_markup` — `` "`**not bold**`" `` → `vec![TemplateChunk::Code("**not bold**")]` (verbatim)
12. `test_math_mode_no_inline_markup` — `"$**not bold**$"` → `vec![TemplateChunk::MathInline("**not bold**")]` (verbatim)
13. `test_unclosed_bold_error_accumulated` — `"**unclosed"` → parse error accumulated; result contains Bold or Literal sentinel; error count == 1 with span at `**`
14. `test_empty_bold_error_accumulated` — `"****"` → parse error accumulated; error count == 1
15. `test_template_chunk_bold_derives_hash_eq_clone_debug` — comemo compatibility (Hash + Eq + Clone + Debug on TemplateChunk::Bold)

### Required Red Gate Tests in `crates/slideforge-eval/`

**chunks_to_inline_nodes conversion:**

16. `test_bold_chunk_to_inline_node` — `TemplateChunk::Bold([TemplateChunk::Literal("hi")])` → `InlineNode::Bold([InlineNode::Plain(Arc::from("hi"))])`
17. `test_italic_chunk_to_inline_node` — as above for Italic
18. `test_code_chunk_to_inline_node` — `TemplateChunk::Code("fn x() {}")` → `InlineNode::Code(Arc::from("fn x() {}"))`
19. `test_link_chunk_to_inline_node` — Link with text vec and url
20. `test_math_inline_chunk_to_inline_node` — `TemplateChunk::MathInline("x^2")` → `InlineNode::Math(MathNode { latex: "x^2", display: false, ... })`
21. `test_math_display_chunk_to_inline_node` — `TemplateChunk::MathDisplay(...)` → `InlineNode::Math(MathNode { display: true, ... })`
22. `test_expr_ref_call_to_xref` — `TemplateChunk::Expr(Expr::Call { func: "ref", args: [Expr::Str("slide-1")] })` → `InlineNode::Xref(Arc::from("slide-1"))`
23. `test_expr_footnote_call_to_footnote_node` — `TemplateChunk::Expr(Expr::Call { func: "footnote", args: [Expr::Str("see appendix")] })` → `InlineNode::Footnote([InlineNode::Plain(Arc::from("see appendix"))])`

**Section sub-block end-to-end (AC-002 / BC-3.02.002 PC8):**

24. `test_ac002_bold_in_section_detail_produces_inlines` — Full pipeline: parse source text `section methodology:\n  detail:\n    **Bold claim.**` → `SectionBlock.body["detail"]` is `slideforge_types::FieldValue::Inlines(vec![InlineNode::Bold([InlineNode::Plain(Arc::from("Bold claim."))])])`
25. `test_ac002_xref_in_section_detail_produces_xref_node` — parse `section methodology:\n  detail:\n    See {{ ref("slide-1") }}.` → `SectionBlock.body["detail"]` contains `InlineNode::Xref(Arc::from("slide-1"))`
26. `test_ac002_plain_text_not_literal_asterisks` — parse `section methodology:\n  detail:\n    **Bold** text.` → `RegisteredContent.content` contains `InlineNode::Bold(...)` and `InlineNode::Plain(Arc::from(" text."))`, NOT a single `InlineNode::Plain(Arc::from("**Bold** text."))`
27. `test_ac002_interpolation_with_bold_context` — `**{{ client }}**` with `client = "Acme"` → `InlineNode::Bold([InlineNode::Plain(Arc::from("Acme"))])`
28. `test_ac002_report_sub_block_produces_inlines` — same as test 24 but for `report:` key

These tests are the Red Gate. All must fail before the implementer starts. All must pass at
end of TDD cycle, along with all pre-existing passing tests.

---

## 9. Flagged BC/Story Amendments (Do NOT Edit — Flagged for product-owner and story-writer)

The following documents require amendments as a direct consequence of this directive.
This directive does NOT edit them. The product-owner and story-writer must apply these
amendments in their own agent bursts.

### 9.1 BC-3.02.002 — Required PC8 Wording Clarification

**Current PC8:** "...bold text within a section sub-block renders as bold in DOCX/PDF
output (not as literal asterisks)."

**Required addition:** PC8 should explicitly state the parse-time representation:
"`detail:` and `report:` sub-block field values are stored as `FieldValue::Inlines`
(not `Value::Str`) in `SectionBlock.body`. The inline markup forms `**bold**`, `_italic_`,
`` `code` ``, `[text](url)`, `^sup^`, `~sub~`, `~~del~~`, `==highlight==`, `{{ ref("id") }}`,
`{{ footnote("...") }}` are recognized within sub-block content and produce structural
`InlineNode` variants. Plain strings without markup are represented as `InlineNode::Plain`."

**Amendment type:** Clarification / implementation detail. No behavioral change. Bump BC
version from 1.4 to 1.5.

### 9.2 STORY-077 — Task List Extension

**Current task list** does not explicitly call out:
- Adding new `TemplateChunk` variants to `slideforge-syntax/src/template.rs`
- Adding the inline markup scanning pass to `parser/template.rs`

These are required to deliver AC-002. The story-writer must add these as explicit tasks.

### 9.3 Follow-Up Story — Slide-Level Inline Markup (New Story Required)

**story-writer must create a new story** (proposed STORY-NNN, to be assigned) covering:
- Eval-stage conversion of slide field values from `FieldValue::Template` (with inline markup
  `TemplateChunk` variants) to `slideforge_types::FieldValue::Inlines` where the field type
  semantically carries inline content (bullets, body text).
- Layout + all-exporter updates to render `InlineNode::Bold` etc. as structural formatting.
- This story should be added to Wave 4 (if capacity allows) or Wave 5.
- It BLOCKS v1.0 release — the Quality Bar requires bold to render as bold in ALL output
  formats, not only in DOCX/PDF section bodies.

**Traceability:** This story traces to BC-3.02.002 PC8 (the "observable consequence" clause)
and to the Quality Bar row "Implementation: zero `.unwrap()` outside tests; `clippy::pedantic`
clean" (which implicitly requires correct structural rendering everywhere, not just one code path).

---

## 10. Changelog

| Date | Author | Change |
|------|--------|--------|
| 2026-06-02 | architect | Initial issue — §1 through §9 (inline markup syntax, two-phase architecture, error handling, plugin surface, Red Gate requirements, BC/story amendments) |
| 2026-06-03 | product-owner | **§1 AMENDMENT — italic `_` bilateral flanking (OBS-P24-A, human-authorized):** Disambiguation Rule 2a added. The prior wording specified only an open-side left-flanking guard for `_`. The amended rule adds a symmetric close-side right-flanking guard: a `_` closes italic only when the byte immediately following it is NOT an ASCII alphanumeric or `_`. This prevents `_apply file_path here_` from closing italic at the word-internal `_` in `file_path`. The amendment is implementable by adding a `next_is_word` guard in the `close_on = Some("_")` branch of `scan_template_chunks`, mirroring the existing `prev_is_word` open-side guard. See Rule 2a for the precise condition and expected parse. Note entry row in `_` italic syntax table updated to cite bilateral flanking. The original "no flanking requirement" wording is superseded and removed. |
| 2026-06-03 | product-owner | **§1 NOTE — E-PAR-022 link URL scheme allowlist cross-reference added:** The `[text](url)` hyperlink row in §1 syntax table and Disambiguation Rule 4 updated to reference E-PAR-022 (`DisallowedLinkUrlScheme`). Relative/anchor link v1 decision documented in Rule 4 and E-PAR-022 Note (in error taxonomy). |

---

## 11. Architect Self-Audit (DIR-077-002)

- [x] Did I rationalize any decision with "MVP," "for now," or "good enough"?
  NO. The deferral of SLIDE-LEVEL inline markup is a scope boundary, not an MVP shortcut.
  The boundary is explicitly flagged as requiring a follow-up story that BLOCKS v1.0. The
  current scope delivers AC-002 completely correctly; the deferral is in a SEPARATE feature.
- [x] Did I add a tech-debt-register entry without all three required conditions?
  NO. The follow-up story is flagged with (a) explicit human authorization context (the
  production-grade default principle requires it before v1.0), (b) a concrete future
  dependency (slide-level rendering), and (c) a specific wave/story anchor.
- [x] Did I leave any "pending architect review" TODO?
  NO. All six questions from the task prompt are answered with binding decisions.
- [x] Did I find a bug in another AI's output and surface it as an advisory?
  The crate-dependency gap (slideforge-syntax cannot import slideforge-types::InlineNode)
  is an architectural constraint I RESOLVED by specifying the two-phase model, not surfaced
  as a question.
- [x] Did I default to the cheapest mechanism?
  NO. The two-phase model (TemplateChunk variants + eval-time conversion) is the
  architecturally correct path. Adding a dependency to slideforge-syntax would be cheaper
  in code but would violate the foundation-layer constraint.
