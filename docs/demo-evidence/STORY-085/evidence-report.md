# STORY-085 Demo Evidence Report

**Story:** STORY-085 — Bundled DefaultInlineFormat + PPTX OOXML Dog-Fooding Refactor
**Crates:** `slideforge-plugin-api`, `slideforge-pptx`, `slideforge-types`
**Branch:** `feature/STORY-085`
**Recorded:** 2026-06-05
**Recording tool:** VHS (terminal capture of `cargo run --example` + `cargo nextest` + `cargo clippy`)

---

## Artifacts

| File | Format | Covered ACs |
|------|--------|-------------|
| `AC-001-007-inline-formats.gif` | GIF (PR embed) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |
| `AC-001-007-inline-formats.webm` | WEBM (archival) | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |
| `AC-001-007-inline-formats.tape` | VHS script | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |

**Example binary:** `crates/slideforge-plugin-api/examples/story_085_inline_formats.rs`
**Run command:** `cargo run --example story_085_inline_formats -p slideforge-plugin-api -q`

---

## Coverage Map

### AC-001 — DefaultInlineFormat covers all 12 InlineNode variants for OOXML

**Demonstrated in recording at:** Section 1 — demo example binary output (AC-001 block)
**Evidence:** The `story_085_inline_formats` example instantiates all 12 `InlineNode`
variants and calls `render(node, InlineOutputFormat::Ooxml)` on each. The recording
shows all 12 producing `Ok` results with correct `<a:r>` OOXML markup:
- `Plain("hello")` → `<a:r><a:t>hello</a:t></a:r>`
- `Bold([Plain("hi")])` → `<a:r><a:rPr b="1"/><a:t>hi</a:t></a:r>`
- `Italic` → `<a:rPr i="1"/>`, `Code` → `<a:rPr><a:latin typeface="Courier New"/>`, etc.
- All 12 variants show `OK` status; final line shows `PASS 12/12 variants returned Ok for OOXML`

**Result:** `PASS`

---

### AC-002 — DefaultInlineFormat covers all 12 InlineNode variants for HTML with full escaping

**Demonstrated in recording at:** Section 1 — demo example binary output (AC-002 block)
**Evidence (12 variants):** All 12 variants return `Ok` HTML strings with correct wrapper tags.
The recording shows `PASS 12/12 variants returned Ok for HTML + 4/4 escaping checks`.

**Evidence (BC-3.05.001 v1.3.6 escaping checks):** 4 dedicated escaping assertions shown:
- `Plain("a & b < c")` → HTML: `a &amp; b &lt; c`
- `Link { text: "R&D", url: "https://ex.com/a&b" }` → HTML:
  `<a href="https://ex.com/a&amp;b">R&amp;D</a>` (URL attribute + text both escaped)
- `Xref("sec<1>")` → HTML: `<a href="#sec&lt;1&gt;">sec&lt;1&gt;</a>` (id in href + text)
- `Math(latex: "x & y")` → HTML: `<span class="math">x &amp; y</span>` (latex text)

**Result:** `PASS`

---

### AC-003 — DefaultInlineFormat covers all 12 InlineNode variants for Markdown

**Demonstrated in recording at:** Section 1 — demo example binary output (AC-003 block)
**Evidence:** All 12 variants return `Ok` with correct Markdown syntax:
- `Italic([Plain("em")])` → `*em*`
- `Strikethrough([Plain("del")])` → `~~del~~`
- `Bold` → `**hi**`, `Code` → `` `fn foo()` ``, `Math` → `$x^2$`, `Link` → `[click here](https://example.com)`
- `Superscript` → `^2^`, `Subscript` → `~n~`, `Highlight` → `==marked==`
- `Footnote` → `[footnote body]` (no numbered `^` marker, per spec deferral)
- `Xref` → `[slide-3](#slide-3)`
- Recording shows `PASS 12/12 variants returned Ok for Markdown`

**Result:** `PASS`

---

### AC-004 — DefaultInlineFormat compiles using only slideforge-plugin-api public API

**Demonstrated in recording at:** Section 2 — module boundary fitness-function test
**Evidence (fitness test):** `cargo nextest run -p slideforge-plugin-api -E 'test(module_boundary)'`
runs the `inline_formats_module_boundary` fitness-function test that scans
`inline_formats/*.rs` source files for forbidden `use` imports of exporter crates,
eval/syntax/layout crates, and other out-of-bounds dependencies. Recording shows `PASS 1 test passed`.

**Evidence (binary compilation):** The `story_085_inline_formats.rs` example binary
imports only `slideforge_plugin_api::{DefaultInlineFormat, InlineFormat, InlineOutputFormat}` and
`slideforge_types::{InlineNode, MathNode, SourceSpan}`. It compiled with zero warnings,
proving no private API paths are required.

**Result:** `PASS`

---

### AC-005 — slideforge-pptx no longer directly constructs `<a:r>` OOXML outside dispatch site

**Demonstrated in recording at:** Section 3 — grep-zero architectural test
**Evidence:** `cargo nextest run -p slideforge-pptx -E 'test(bc_5_02_002_ac005)'` runs
`test_bc_5_02_002_ac005_grep_zero_ar_rpr_serialize_inline_outside_dispatch`, which
internally performs the canonical grep audit: `grep -r "a:r|a:rPr|serialize_inline" crates/slideforge-pptx/src/`
and asserts zero matches outside the single registered dispatch site. Recording shows `PASS 1 test passed`.

The internal `serialize_inline_nodes_to_xml()` function has been removed from `notes_slide.rs`.
All inline rendering now routes through `DefaultInlineFormat::render(node, InlineOutputFormat::Ooxml)`.

**Result:** `PASS`

---

### AC-006 — PPTX notes XML output is correct after the refactor

**Demonstrated in recording at:** Section 4 — PPTX snapshot stability tests
**Evidence:** `cargo nextest run -p slideforge-pptx -E 'test(bc_5_02_002_ac006)'` runs
three pinned AC-006 tests:
- `test_bc_5_02_002_ac006_notes_xml_output_pinned_plain_text` — Plain text run byte identity
- `test_bc_5_02_002_ac006_notes_xml_output_pinned_bold_italic` — Bold + Italic run byte identity
- `test_bc_5_02_002_ac006_notes_xml_output_pinned_xml_escape` — XML escaping in `<a:t>` content

Recording shows `PASS 3/3 tests passed` with no snapshot deltas. The refactored
dispatch path via `DefaultInlineFormat` is byte-for-byte identical to the removed
`serialize_inline_nodes_to_xml()` for Plain, Bold, and Italic nodes.

**Result:** `PASS`

---

### AC-007 — No production catch_unwind added; existing tests unaffected

**Demonstrated in recording at:** Section 5 — clippy pedantic + unwrap_used gate
**Evidence:** `cargo clippy -p slideforge-plugin-api -p slideforge-pptx --all-targets -- -D warnings -D clippy::pedantic -D clippy::unwrap_used` completes with `Finished` and zero warnings or errors.

The separate `test_bc_5_02_002_ac007_no_catch_unwind_in_pptx_src` test (also passing,
shown in the full nextest run) explicitly scans `crates/slideforge-pptx/src/` source files
for `catch_unwind` strings and asserts zero matches.

All `Result` propagation in the refactored `notes_slide.rs` uses `?` and `map_err` — no
new `.unwrap()` or `catch_unwind` calls were introduced.

**Result:** `PASS`

---

## CI Gate Confirmation

All CI gates ran clean in the worktree before recording:

| Gate | Command | Result |
|------|---------|--------|
| Tests (plugin-api) | `cargo nextest run -p slideforge-plugin-api --no-fail-fast` | CLEAN (265/265 PASS) |
| Tests (pptx) | `cargo nextest run -p slideforge-pptx --no-fail-fast` | CLEAN |
| Clippy | `cargo clippy -p slideforge-plugin-api -p slideforge-pptx --all-targets -- -D warnings -D clippy::pedantic -D clippy::unwrap_used` | CLEAN |
| Module boundary | `cargo nextest run -p slideforge-plugin-api -E 'test(module_boundary)'` | CLEAN |
| Grep-zero | `cargo nextest run -p slideforge-pptx -E 'test(bc_5_02_002_ac005)'` | CLEAN |

---

## Bonus — Variant × Format Matrix Sample

The recording shows the representative row from the full matrix:

| Node | OOXML | HTML | Markdown |
|------|-------|------|----------|
| `Bold([Plain("hi")])` | `<a:r><a:rPr b="1"/><a:t>hi</a:t></a:r>` | `<strong>hi</strong>` | `**hi**` |

This directly validates the spec matrix in STORY-085 Scope A, confirming the
canonical expected output per format.
