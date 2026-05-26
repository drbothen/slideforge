---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-002
title: "Plugin Trait API (all 10 surfaces)"
epic: EPIC-01
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-plugin-api
subsystems: [SS-14]
target_module: slideforge-plugin-api
behavioral_contracts: [BC-5.02.001, BC-5.02.002]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-001]
blocks:
  - STORY-003
  - STORY-049
  - STORY-050
estimated_days: 2
---

# STORY-002: Plugin Trait API (all 10 surfaces)

## Summary

Define all 10 plugin trait surfaces in the `slideforge-plugin-api` crate. This crate
declares the public extension contracts that every bundled plugin implements and that
future external plugins will implement. The crate has zero production dependencies on
other workspace crates (it imports types from `slideforge-types` only). All 10 traits
are `Send + Sync`. This story satisfies BC-5.02.001 (all 10 surfaces defined and
compilable) and BC-5.02.002 (the dog-fooding guarantee is enforced at the trait level).

The `PluginRegistry` struct is also defined here with registration methods for each
surface. Actual bundled plugin implementations come in later stories; this story only
defines the traits and the registry container.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| `crates/slideforge-plugin-api/src/` files to write | ~5,000 |
| Test files | ~2,000 |
| Cargo.toml | ~400 |
| **Total** | **~10,900** |

Agent context budget: 200k tokens. This story is ~5.5% of budget — well within limit.

## Behavioral Contracts

| BC ID | Title | Clauses Covered |
|-------|-------|----------------|
| BC-5.02.001 | All 10 plugin trait surfaces implemented by bundled plugins via the public trait API | Preconditions 1-3; Postconditions 1-4; Invariants 1-3 |
| BC-5.02.002 | No bundled plugin bypasses the registered trait interface (dog-fooding guarantee) | Preconditions 1-2; Postconditions 1-4; Invariants 1-3 |

## Acceptance Criteria

- [ ] **AC-001:** `DataSource` trait is defined with methods `id(&self) -> &str` and `load(&self, uri: &str, opts: &DataSourceOptions) -> Result<Value, DataSourceError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-002:** `Exporter` trait is defined with methods `id(&self) -> &str`, `extension(&self) -> &str`, and `export(&self, deck: &Deck, laid_out: &LaidOutDeck, brand: &Brand, opts: &ExportOptions) -> Result<Vec<u8>, ExportError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-003:** `ChartRenderer` trait is defined with methods `id(&self) -> &str` and `render(&self, spec: &ChartSpec, brand: &Brand) -> Result<Vec<u8>, ChartError>` returning SVG bytes — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-004:** `DiagramRenderer` trait is defined with methods `id(&self) -> &str` and `render(&self, source: &str, opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError>` returning SVG bytes — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-005:** `Validator` trait is defined with methods `id(&self) -> &str` and `validate(&self, deck: &Deck, opts: &ValidatorOptions) -> Vec<Diagnostic>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-006:** `MathRenderer` trait is defined with methods `id(&self) -> &str` and `render(&self, node: &MathNode, format: MathOutputFormat) -> Result<Vec<u8>, MathError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-007:** `BrandProvider` trait is defined with methods `id(&self) -> &str` and `load(&self, source: &BrandSource) -> Result<Brand, BrandError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-008:** `SlideType` trait is defined with methods `id(&self) -> &str`, `required_fields(&self) -> &[FieldDef]`, `optional_fields(&self) -> &[FieldDef]`, `layout_name(&self) -> &str`, and `lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-009:** `SectionType` trait is defined with methods `id(&self) -> &str` and `generate(&self, slides: &[Slide]) -> Vec<SectionBlock>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-010:** `InlineFormat` trait is defined with methods `id(&self) -> &str` and `render(&self, node: &InlineNode, format: InlineOutputFormat) -> Result<String, InlineError>` — trait is `Send + Sync`. (traces to BC-5.02.001 postcondition 1)
- [ ] **AC-011:** `PluginRegistry` struct is defined with 10 registration methods (`register_data_source`, `register_exporter`, `register_chart_renderer`, `register_diagram_renderer`, `register_validator`, `register_math_renderer`, `register_brand_provider`, `register_slide_type`, `register_section_type`, `register_inline_format`) and corresponding lookup methods. (traces to BC-5.02.001 postcondition 3)
- [ ] **AC-012:** All error types (`DataSourceError`, `ExportError`, `ChartError`, `DiagramError`, `MathError`, `BrandError`, `LayoutError`, `InlineError`) are defined as `thiserror`-derived enums with at least one variant each. (traces to BC-5.02.001 postcondition 2)
- [ ] **AC-013:** A minimal test plugin implementing `DataSource` that imports ONLY `slideforge-plugin-api` and `slideforge-types` (not any other workspace crate) compiles successfully. This is verified by a test in `tests/dog_food_test.rs`. (traces to BC-5.02.002 postcondition 3)
- [ ] **AC-014:** `PluginRegistry` is `Send + Sync` — verified by `fn assert_send_sync<T: Send + Sync>() {}` compile-time assertion in `lib.rs`. (traces to BC-5.02.001 invariant 3)
- [ ] **AC-015:** Exactly 10 plugin surfaces. Confirmed by a unit test that calls each of the 10 registration methods on a fresh `PluginRegistry` and asserts registry state. (traces to BC-5.02.001 invariant 1)
- [ ] **AC-016:** `#![forbid(unsafe_code)]` present on crate root (NFR-024).
- [ ] **AC-017:** `#![warn(missing_docs)]` present; all public items documented (NFR-023).
- [ ] **AC-018:** `cargo clippy -p slideforge-plugin-api -- -D warnings` produces zero warnings (NFR-022).
- [ ] **AC-019:** All production deps in `Cargo.toml` use `=` version pinning (NFR-025). Only `slideforge-types` (workspace path dep) and `thiserror = "=2.0.18"` are needed.

## Previous Story Intelligence

STORY-001 defines the core types (`Deck`, `Slide`, `Value`, `Brand`, `LaidOutDeck`,
`SourceSpan`, `MathNode`, `ChartSpec`, `DiagramSpec`, `InlineNode`, etc.) that are
referenced in the trait signatures of this story. Implementer must verify that all
types referenced in AC-001 through AC-010 exist in `slideforge-types` before writing
trait definitions. If a type is missing, add it to `slideforge-types` in this story
(the split between the two stories is soft — the constraint is that no workspace dep
cycles are introduced).

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md` and `architecture/plugin-architecture.md`:

1. **Zero workspace deps except `slideforge-types` (crate-architecture.md):** `slideforge-plugin-api` is a leaf. Its only allowed workspace dep is `slideforge-types`. Any import from `slideforge-eval`, `slideforge-syntax`, or any effectful crate is a build violation.
2. **All 10 traits are `Send + Sync` (plugin-architecture.md):** The plugin registry is constructed once and used from multiple threads in the CLI. Traits that are not `Send + Sync` will fail to register.
3. **Static dispatch in registry (plugin-architecture.md §Registry):** The `PluginRegistry` stores `Box<dyn Trait>` for each surface — dynamic dispatch is correct here. Each surface stores a `Vec<Box<dyn Trait>>` to allow multiple implementations (e.g., multiple exporters).
4. **Dog-fooding guarantee (BC-5.02.002 + DI-008):** The traits must be self-contained enough that a bundled plugin crate can implement them using only `slideforge-plugin-api` + `slideforge-types`. No "escape hatch" trait methods that require internal crate knowledge.
5. **Canonical trait signatures supersede drafts (CLAUDE.md precedence rule):** The signatures in `interface-definitions.md §6` supersede any earlier draft signatures. Use the signatures in AC-001 through AC-010 of this story (which are sourced from that document). Do not invent alternate signatures.
6. **Forbidden dependencies:** `slideforge-plugin-api` MUST NOT depend on: `slideforge-eval`, `slideforge-syntax`, `slideforge-validate`, `slideforge-layout`, `slideforge-pptx`, or any effectful workspace crate. Build fails if such a dep appears.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `slideforge-types` | workspace path | All IR types in trait signatures |
| `thiserror` | `=2.0.18` | Error type derives for all plugin error enums |

Dev-only (no pinning required):
- None beyond what `slideforge-types` already pulls in.

## File Structure Requirements

Files to create:

```
crates/slideforge-plugin-api/
├── Cargo.toml                      # crate manifest; deps: slideforge-types (path), thiserror =2.0.18
├── src/
│   ├── lib.rs                      # crate root; re-exports all traits + error types + registry
│   ├── traits/
│   │   ├── mod.rs                  # pub use all trait modules
│   │   ├── data_source.rs          # DataSource trait + DataSourceOptions + DataSourceError
│   │   ├── exporter.rs             # Exporter trait + ExportOptions + ExportError
│   │   ├── chart_renderer.rs       # ChartRenderer trait + ChartError
│   │   ├── diagram_renderer.rs     # DiagramRenderer trait + DiagramOptions + DiagramError
│   │   ├── validator.rs            # Validator trait + ValidatorOptions + Diagnostic
│   │   ├── math_renderer.rs        # MathRenderer trait + MathOutputFormat + MathError
│   │   ├── brand_provider.rs       # BrandProvider trait + BrandSource + BrandError
│   │   ├── slide_type.rs           # SlideType trait + FieldDef + Canvas + LayoutError
│   │   ├── section_type.rs         # SectionType trait + SectionBlock
│   │   └── inline_format.rs        # InlineFormat trait + InlineOutputFormat + InlineError
│   └── registry.rs                 # PluginRegistry struct + 10 register/lookup methods
└── tests/
    └── dog_food_test.rs            # Minimal test plugin that uses only public API
```

## Tasks

1. **Create `Cargo.toml`** with correct metadata, `[lib]`, edition 2024, and production deps (`slideforge-types` as path dep, `thiserror = "=2.0.18"`). (10 min)
2. **Write `src/traits/data_source.rs`** — `DataSource` trait, `DataSourceOptions`, `DataSourceError`. (15 min)
3. **Write `src/traits/exporter.rs`** — `Exporter` trait, `ExportOptions`, `ExportError`. (15 min)
4. **Write `src/traits/chart_renderer.rs`** — `ChartRenderer` trait, `ChartError`. (10 min)
5. **Write `src/traits/diagram_renderer.rs`** — `DiagramRenderer` trait, `DiagramOptions`, `DiagramError`. (10 min)
6. **Write `src/traits/validator.rs`** — `Validator` trait, `ValidatorOptions`, `Diagnostic`. (15 min)
7. **Write `src/traits/math_renderer.rs`** — `MathRenderer` trait, `MathOutputFormat` enum (`Omml`, `MathMl`, `Pdf`), `MathError`. (10 min)
8. **Write `src/traits/brand_provider.rs`** — `BrandProvider` trait, `BrandSource` enum (`PptxFile(Arc<str>)`, `TomlFile(Arc<str>)`), `BrandError`. (10 min)
9. **Write `src/traits/slide_type.rs`** — `SlideType` trait, `FieldDef` struct, `Canvas` struct (width/height in EMU), `LayoutError`. (20 min)
10. **Write `src/traits/section_type.rs`** — `SectionType` trait, `SectionBlock` struct. (10 min)
11. **Write `src/traits/inline_format.rs`** — `InlineFormat` trait, `InlineOutputFormat` enum (`Ooxml`, `Html`, `Markdown`), `InlineError`. (10 min)
12. **Write `src/traits/mod.rs`** — pub use all trait modules. (5 min)
13. **Write `src/registry.rs`** — `PluginRegistry` struct with `Vec<Box<dyn Trait>>` per surface + 10 `register_*` + 10 `lookup_*` methods. Add compile-time `Send + Sync` assertion. (30 min)
14. **Write `src/lib.rs`** — `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, pub use, `fn assert_registry_send_sync()` compile-time assertion. (10 min)
15. **Write `tests/dog_food_test.rs`** — minimal struct implementing `DataSource` using only public API; confirm it registers and looks up correctly. (20 min)
16. **Run `cargo clippy -p slideforge-plugin-api -- -D warnings`** and fix all. (15 min)
17. **Run `cargo test -p slideforge-plugin-api`** and confirm pass. (5 min)

## Test Strategy

### Unit tests (inline)

- `registry.rs` — register one dummy impl per surface, confirm `lookup_*` returns it. Confirm second registration appends (not replaces).
- `registry.rs` — confirm registering to an uninitialized surface does not panic.
- Each trait module — compile-time `fn assert_send_sync<T: Send + Sync>()` tests using each trait object `dyn Trait`.

### Integration test (`tests/dog_food_test.rs`)

```rust
// This test verifies BC-5.02.002: a plugin can be written using only the public API.
use slideforge_plugin_api::{DataSource, DataSourceOptions, DataSourceError, PluginRegistry};
use slideforge_types::Value;

struct TestDataSource;

impl DataSource for TestDataSource {
    fn id(&self) -> &str { "test" }
    fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
        Ok(Value::Null)
    }
}

#[test]
fn dog_food_data_source() {
    let mut registry = PluginRegistry::new();
    registry.register_data_source(Box::new(TestDataSource));
    let plugin = registry.lookup_data_source("test").expect("registered plugin not found");
    assert_eq!(plugin.id(), "test");
}
```

### Snapshot tests

None in this crate.

## Dependencies

**Depends on:** STORY-001 (IR core types — all trait signatures reference `Deck`, `Slide`, `LaidOutDeck`, `Brand`, `Value`, `MathNode`, `ChartSpec`, `DiagramSpec`, `InlineNode`, `LaidOutSlide`, `SourceSpan`).

Dependency justification: STORY-002 depends on STORY-001 because all 10 trait signatures reference IR types (`Deck`, `LaidOutDeck`, `Brand`, etc.) that live in `slideforge-types`. The trait file cannot compile without those type definitions.

**Blocks:**
- STORY-003 (slide-type-impls): needs `SlideType` trait to implement 31 types.
- STORY-049 (plugin-registry-assembly): needs all 10 trait definitions to assemble the root registry.
- STORY-050 (e2e integration tests): needs all traits to write integration test fixtures.

## Implementation Notes

### `FieldDef` for `SlideType`

`FieldDef` describes a field slot for a slide type. Minimum definition:
```rust
pub struct FieldDef {
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub required: bool,
    pub default_value: Option<Value>,
}
```

### `Canvas` for `SlideType::lay_out`

```rust
pub struct Canvas {
    pub width: Emu,
    pub height: Emu,
}

impl Default for Canvas {
    fn default() -> Self {
        Self { width: SLIDE_WIDTH, height: SLIDE_HEIGHT }
    }
}
```

### `Diagnostic` for `Validator`

```rust
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Arc<str>,        // e.g., "E-PAR-003", "E-EVL-003"
    pub message: Arc<str>,
    pub span: SourceSpan,
    pub hint: Option<Arc<str>>,
}

pub enum DiagnosticSeverity { Error, Warning, Info }
```

### Registry Lookup Semantics

- `lookup_*_by_id(id: &str) -> Option<&dyn Trait>` — find by trait `id()`.
- `lookup_slide_type(keyword: &str) -> Option<&dyn SlideType>` — find slide type by its keyword (which matches `SlideType::id()`).
- All lookups return `Option<&dyn Trait>` (not `Result`) — callers handle "not found."

### No Dynamic Plugin Loading

All v1.0 plugins are statically compiled (plugin-architecture.md §v1.0 Plugin Loading).
Do not add WASM loading, `libloading`, or dynamic dispatch infrastructure. The `PluginRegistry`
stores `Box<dyn Trait>` which is sufficient for static dispatch to plugin implementations.

### `PluginRegistry` is NOT `Clone`

`Box<dyn Trait>` trait objects are not `Clone` by default. `PluginRegistry` should be
wrapped in `Arc<PluginRegistry>` by callers who need shared ownership. The registry
itself is constructed once at process start and then only borrowed immutably.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two plugins registered with same `id()` for same surface | Both are stored; `lookup_by_id` returns the first match (insertion order from `IndexMap`). Second registration does not overwrite. |
| EC-002 | `lookup_slide_type` called with unknown keyword | Returns `None`; caller emits E-PAR-007. |
| EC-003 | `PluginRegistry::new()` before any registrations | All lookup methods return `None`; no panic. |
| EC-004 | `DataSource::load` returns `Err(DataSourceError)` | Error propagates to evaluator; evaluator wraps in its own error type. |
