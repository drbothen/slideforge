---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-001
title: "IR Core Types (Deck, Slide, Value, ContentBlock)"
epic: EPIC-01
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-types
subsystems: [SS-15]
target_module: slideforge-types
behavioral_contracts: []
# BC status: pending PO authorship — STORY-001 is a pure-type definition story.
# The behavioral contracts that TEST these types are in STORY-002 (plugin traits),
# STORY-004 (Value system), and downstream eval/validate stories.
# Types defined here are the substrate; the contracts live on behaviors that USE them.
# No BC is directly attached here, but this story is a prerequisite for all BCs
# in the system. See dependency_justification below.
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: []
blocks:
  - STORY-002
  - STORY-003
  - STORY-004
  - STORY-005
  - STORY-006
  - STORY-007
  - STORY-008
  - STORY-009
  - STORY-010
estimated_days: 2
---

# STORY-001: IR Core Types (Deck, Slide, Value, ContentBlock)

## Summary

Define all shared IR type definitions in the `slideforge-types` crate. This crate
is the foundation leaf of the workspace dependency graph — every other crate depends
on it directly or transitively. The types defined here are `Deck`, `Slide`,
`ContentBlock`, `Block`, `InlineNode`, `Value`, `Register`, `MathNode`, `FieldValue`,
and the `Emu` newtype. All types implement `Hash + Eq + Clone`. String fields use
`Arc<str>`. The crate has zero production dependencies on other workspace crates.

This story delivers type definitions only. Behavioral validation of the types (e.g.,
required-field enforcement, no-coercion) is implemented in STORY-003 and STORY-004
respectively, which depend on these types.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,000 |
| `crates/slideforge-types/src/` files to write | ~4,000 |
| Test file | ~2,000 |
| Cargo.toml | ~500 |
| **Total** | **~9,500** |

Agent context budget: 200k tokens. This story is ~4.75% of budget — well within limit.

## Acceptance Criteria

- [ ] **AC-001:** `Deck` struct compiles with fields: `slides: Vec<Slide>`, `vars: IndexMap<Arc<str>, Value>`, `metadata: DeckMetadata`, and `registers: Vec<Register>` — and implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-002:** `Slide` struct compiles with fields: `slide_type: Arc<str>`, `fields: IndexMap<Arc<str>, FieldValue>`, `blocks: Vec<Block>`, `register: Option<Register>`, `tags: Vec<Arc<str>>`, `source_span: SourceSpan>` — and implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-003:** `Value` enum has exactly the variants: `Str(Arc<str>)`, `Int(i64)`, `Float(ordered_float::OrderedFloat<f64>)`, `Bool(bool)`, `List(Vec<Value>)`, `Map(IndexMap<Arc<str>, Value>)`, `Null` — and implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-004:** `Emu` is a newtype `struct Emu(pub i64)` with associated constants `EMU_PER_INCH: i64 = 914_400` and `EMU_PER_POINT: i64 = 12_700`, and methods `Emu::from_inches(f64) -> Emu` and `Emu::from_points(f64) -> Emu` and arithmetic ops `Add`, `Sub`, `Mul<i64>` — and implements `Hash + Eq + Clone + Copy + Debug`.
- [ ] **AC-005:** `ContentBlock` enum has variants covering all slide content structures: `Text(TextBlock)`, `Bullets(Vec<BulletItem>)`, `Chart(ChartSpec)`, `Diagram(DiagramSpec)`, `Shape(ShapeSpec)`, `Math(MathNode)`, `Image(ImageSpec)`, `Table(TableSpec)` — and implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-006:** `MathNode` struct has fields `latex: Arc<str>` and `display: bool` (display math `$$...$$` vs inline `$...$`), implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-007:** `Register` enum has exactly three variants: `Notes`, `Report`, `Detail` — and implements `Hash + Eq + Clone + Copy + Debug + PartialOrd + Ord`.
- [ ] **AC-008:** `InlineNode` enum has exactly 11 variants: `Plain(Arc<str>)`, `Bold(Vec<InlineNode>)`, `Italic(Vec<InlineNode>)`, `Code(Arc<str>)`, `Link { text: Vec<InlineNode>, url: Arc<str> }`, `Math(MathNode)`, `Footnote(Vec<InlineNode>)`, `Xref(Arc<str>)`, `Superscript(Vec<InlineNode>)`, `Subscript(Vec<InlineNode>)`, `Strikethrough(Vec<InlineNode>)`, `Highlight(Vec<InlineNode>)` — and implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-009:** `SourceSpan` struct has fields `file: Arc<str>`, `line: u32`, `col: u32`, `byte_offset: usize` — implements `Hash + Eq + Clone + Debug + Default`.
- [ ] **AC-010:** `DeckMetadata` struct has fields `title: Option<Arc<str>>`, `slideforge_version: Arc<str>`, `lang: Option<Arc<str>>`, `author: Option<Arc<str>>` — implements `Hash + Eq + Clone + Debug`.
- [ ] **AC-011:** `#![forbid(unsafe_code)]` attribute is present on the crate root (NFR-024).
- [ ] **AC-012:** `#![warn(missing_docs)]` attribute is present; all public items have rustdoc (NFR-023).
- [ ] **AC-013:** `cargo clippy --all-targets -- -D warnings` produces zero warnings for this crate (NFR-022).
- [ ] **AC-014:** All production deps in `Cargo.toml` use `=` version pinning (NFR-025). Only `thiserror = "=2.0.18"`, `ordered-float = "=4.6.0"`, `indexmap = "=2.7.1"`, `arc-swap = "=1.7.1"` are needed.
- [ ] **AC-015:** `cargo test -p slideforge-types` passes. Tests confirm: `Value` variants do not implement `From`/`Into` conversions between each other (no coercion path exists at the type level).
- [ ] **AC-016:** The crate has zero workspace crate dependencies — confirmed by `Cargo.toml` `[dependencies]` section.

## Previous Story Intelligence

N/A — first story in epic. This is the foundation leaf node of the entire workspace.

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md` and `architecture/ir-design.md`:

1. **Pure Core classification (SS-15):** `slideforge-types` is a Pure Core crate. It MUST NOT introduce any I/O, filesystem access, network, or async code. All types are value types.
2. **`Hash + Eq + Clone` on all types (ADR-005):** Required for future `comemo` incremental compilation and for Kani bounded-model checking (Phase 6). Adding these traits retroactively is API-breaking — they must be present from initial declaration.
3. **`Arc<str>` for all string fields (ir-design.md):** Do not use `String`. `Arc<str>` makes cloning cheap and enables correct `Hash + Eq` on interned strings.
4. **`ordered_float::OrderedFloat<f64>` for floats:** Plain `f64` does not implement `Hash + Eq`. Use `OrderedFloat` from the `ordered-float` crate.
5. **Zero workspace deps:** `slideforge-types` and `slideforge-plugin-api` are leaves — they have zero workspace crate dependencies. Violation would create a circular dependency.
6. **EMU as `i64`:** Float-to-EMU conversion happens ONLY at the DSL parse / layout boundary. Internal computation MUST use `i64` EMU values (ADR-013).
7. **Forbidden dependencies:** `slideforge-types` MUST NOT depend on: `slideforge-plugin-api`, `slideforge-eval`, `slideforge-syntax`, `slideforge-validate`, `slideforge-layout`, or any effectful crate. Build will fail if such a dep appears.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `thiserror` | `=2.0.18` | Error derives for crate-level error enum |
| `ordered-float` | `=4.6.0` | `OrderedFloat<f64>` for hash-safe float in `Value` |
| `indexmap` | `=2.7.1` | `IndexMap` for field ordering (preserves insertion order; needed for PPTX layout and test determinism) |
| `arc-swap` | `=1.7.1` | If needed for interning; optional |

Dev dependencies (no version pinning required):
- `serde` with `derive` feature (for snapshot tests in downstream crates; do NOT add serde to production deps in this crate unless needed)

## File Structure Requirements

Files to create:

```
crates/slideforge-types/
├── Cargo.toml                     # crate manifest; zero workspace deps
├── src/
│   ├── lib.rs                     # crate root; #![forbid(unsafe_code)]; pub use all modules
│   ├── deck.rs                    # Deck, DeckMetadata structs
│   ├── slide.rs                   # Slide, FieldValue structs
│   ├── value.rs                   # Value enum (no coercion impls)
│   ├── block.rs                   # ContentBlock, Block, TextBlock, BulletItem enums/structs
│   ├── inline.rs                  # InlineNode enum (11 variants)
│   ├── math.rs                    # MathNode struct
│   ├── emu.rs                     # Emu newtype + arithmetic
│   ├── register.rs                # Register enum (Notes, Report, Detail)
│   ├── span.rs                    # SourceSpan struct
│   └── specs.rs                   # ChartSpec, DiagramSpec, ShapeSpec, ImageSpec, TableSpec
```

Files NOT to modify: None (crate is newly created in this story).

## Tasks

1. **Create `crates/slideforge-types/Cargo.toml`** with correct metadata, `[lib]`, edition 2024, and pinned production deps (`thiserror =2.0.18`, `ordered-float =4.6.0`, `indexmap =2.7.1`). (15 min)
2. **Write `src/lib.rs`** with `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, `#![deny(clippy::pedantic)]` and pub module declarations. (10 min)
3. **Write `src/span.rs`** — `SourceSpan` struct with `Default`. (10 min)
4. **Write `src/register.rs`** — `Register` enum with 3 variants. (10 min)
5. **Write `src/value.rs`** — `Value` enum with 7 variants using `OrderedFloat` for floats and `IndexMap<Arc<str>, Value>` for Map. Include a `#[cfg(test)] mod tests` that asserts no `From<bool> for Value` impl exists (compile-fail test style via doc-test). (30 min)
6. **Write `src/emu.rs`** — `Emu(i64)` newtype with constants and conversion methods and `Add/Sub/Mul<i64>` impls. (20 min)
7. **Write `src/math.rs`** — `MathNode { latex: Arc<str>, display: bool }`. (5 min)
8. **Write `src/inline.rs`** — `InlineNode` enum with exactly 11 variants (matching AC-008). (20 min)
9. **Write `src/specs.rs`** — `ChartSpec`, `DiagramSpec`, `ShapeSpec`, `ImageSpec`, `TableSpec` structs with placeholder fields sufficient for STORY-003 to build on. (30 min)
10. **Write `src/block.rs`** — `ContentBlock` enum (8 variants), `TextBlock`, `BulletItem`. (20 min)
11. **Write `src/slide.rs`** — `Slide`, `FieldValue`. (20 min)
12. **Write `src/deck.rs`** — `Deck`, `DeckMetadata`. (20 min)
13. **Run `cargo clippy -p slideforge-types -- -D warnings`** and fix all warnings. (15 min)
14. **Run `cargo test -p slideforge-types`** and confirm pass. (5 min)
15. **Run `RUSTDOCFLAGS="-D warnings" cargo doc -p slideforge-types --no-deps`** and fix any missing-docs warnings. (15 min)

## Test Strategy

### Unit tests (in `#[cfg(test)] mod tests` in each module)

- `value.rs` — verify `Value::Str(arc)` roundtrips through `Clone + Eq`. Verify `Value::Int(1).eq(&Value::Bool(true))` is `false` (no coercion at type level). Verify `HashMap<Value, ()>` compiles and inserts (confirms `Hash`). Verify `Value::Float(OrderedFloat(1.1))` is hashable.
- `emu.rs` — verify `Emu::from_inches(1.0) == Emu(914_400)`. Verify `Emu::from_points(1.0) == Emu(12_700)`. Verify integer arithmetic: `Emu(100) + Emu(200) == Emu(300)`. Verify standard 16:9 slide dimensions: `Emu::from_inches(10.0) == Emu(9_144_000)` (width), `Emu::from_inches(7.5).0` (not exactly — actual slide height is 5,143,500 EMU which is 5.625 inches — verify this constant).
- `register.rs` — verify `Register::Notes < Register::Report < Register::Detail` (for sort ordering).
- `inline.rs` — verify `InlineNode` can be placed in a `Vec` (clone-ability).
- `deck.rs` — verify `Deck::default()` or a simple constructed `Deck` compiles with all required fields.

### Integration tests

None in this crate — types are exercised by downstream crates.

### Snapshot tests

None in this story — snapshot tests for rendered output come with the exporter stories.

## Dependencies

**Depends on:** None (this is Wave 1, no prerequisites).

**Blocks:**
- STORY-002 (plugin-trait-api): needs `Deck`, `LaidOutDeck`, `Brand`, `Slide`, `Value` types to write trait signatures.
- STORY-003 (slide-type-impls): needs `Slide`, `FieldValue`, `ContentBlock`, `SourceSpan` to implement `SlideType` trait.
- STORY-004 (value-system-emu): expands on `Value` and `Emu` behavior, needs this story's type definitions as base.
- STORY-005 through STORY-010 (parser stories): lexer/parser output is typed AST nodes that reference `Value`, `SourceSpan`, `Register`.

Dependency justification: STORY-002 depends on STORY-001 because the plugin trait signatures in `slideforge-plugin-api` reference `Deck`, `LaidOutDeck`, `Brand`, `Slide`, and `Value` types that live in `slideforge-types`. Without the types, the trait signatures cannot compile.

## Implementation Notes

### LaidOutDeck — Define Skeleton Here

`LaidOutDeck` is the geometric IR (output of layout engine, consumed by exporters). Define a skeleton here even though it is fully populated in STORY-026 (layout engine). Exporters depend on `LaidOutDeck` at the type level from Wave 1 onward.

Minimum fields for `LaidOutDeck`:
```rust
pub struct LaidOutDeck {
    pub slides: Vec<LaidOutSlide>,
    pub metadata: DeckMetadata,
}

pub struct LaidOutSlide {
    pub elements: Vec<LaidOutElement>,
    pub slide_number: u32,
    pub source_slide: Arc<Slide>,   // back-ref to semantic Slide
}

pub struct LaidOutElement {
    pub x: Emu,
    pub y: Emu,
    pub width: Emu,
    pub height: Emu,
    pub semantic_role: SemanticRole,
    pub content: LaidOutContent,
}

pub enum SemanticRole {
    Heading { level: u8 },
    Paragraph,
    Figure,
    Table,
    Decorative,
}
```

### Brand — Define Skeleton Here

`Brand` is consumed by exporters and the layout engine. Define minimum:
```rust
pub struct Brand {
    pub name: Arc<str>,
    pub palette: BrandPalette,
    pub fonts: BrandFonts,
    pub layouts: Vec<LayoutDefinition>,
}
```

Full fields are fleshed out in STORY-022/023 (brand loading/synthesis). The skeleton must compile without those stories.

### No `From`/`Into` Coercions

Do NOT implement `From<bool> for Value`, `From<i64> for Value`, etc. These impls would create implicit coercion paths that violate BC-1.02.003 (no implicit type coercion). Constructors are explicit: `Value::Bool(true)`, `Value::Int(42)`. If ergonomic construction is desired for tests, use a test-only helper — never in production code.

### Ordering Consideration for `IndexMap`

Use `indexmap::IndexMap` (not `std::collections::HashMap`) for all map-typed fields in IR types. This preserves insertion order, which is required for:
1. Deterministic PPTX field ordering (OOXML element ordering is schema-significant)
2. Snapshot test stability
3. CLI JSON output stability

### Canvas Size Constants

Define these in `emu.rs` or a `canvas.rs` module:
```rust
pub const SLIDE_WIDTH: Emu = Emu(9_144_000);   // 10 inches at 16:9
pub const SLIDE_HEIGHT: Emu = Emu(5_143_500);  // 5.625 inches at 16:9
```

### `FieldValue` Design

`FieldValue` is the parsed value of a slide field before expression evaluation. It wraps either a literal `Value` or an expression string to be evaluated:
```rust
pub enum FieldValue {
    Literal(Value),
    Expr(Arc<str>),      // raw expression text, e.g., "{{ count + 1 }}"
    Interpolated(Vec<StringPart>),  // mixed literal + expr parts
}

pub enum StringPart {
    Literal(Arc<str>),
    Expr(Arc<str>),
}
```

### Error Type

The crate does not need a rich error enum — types should be infallible to construct. Add a minimal `TypeError` enum with `thiserror` derive for future use in the evaluator:
```rust
#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("type mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: Arc<str>, actual: Arc<str> },
}
```

### Workspace thiserror Version — First Implementation Task

The workspace `Cargo.toml` currently declares `thiserror = "1"`. This MUST be updated to `thiserror = "2"` (resolved pin `=2.0.18`) as the first implementation task, since ooxmlsdk 0.6.1 and all story specs require thiserror 2.x. A mismatched workspace declaration will cause Cargo to pull two conflicting major versions, breaking the unified dependency graph. Update `[workspace.dependencies]` before writing any crate-level `Cargo.toml`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `Value::Float` with NaN | `OrderedFloat` maps NaN to a canonical representation — `Hash` works; `Eq` is consistent. `OrderedFloat::nan() == OrderedFloat::nan()` is `true` (unlike raw `f64`). |
| EC-002 | `InlineNode` deeply nested (bold inside italic inside bold) | `Vec<InlineNode>` is heap-allocated; no stack overflow risk for reasonable nesting depths. |
| EC-003 | `Arc<str>` empty string | `Arc::<str>::from("")` is valid; used for optional fields with empty default. |
| EC-004 | `Emu` overflow in arithmetic | `i64` arithmetic — overflow panics in debug, wraps in release. Kani proofs in STORY-066 will prove bounds. Accept debug panic for now; document in rustdoc. |
| EC-005 | `IndexMap` with 1000+ entries | Performance characteristic. No special handling needed in type definitions. |
