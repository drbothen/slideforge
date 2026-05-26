---
document_type: story
traces_to: .factory/stories/STORY-INDEX.md
story_id: STORY-004
title: "Value System + EMU Types"
epic: EPIC-01
wave: 1
points: 5
priority: P0
tdd_mode: strict
status: draft
crate: slideforge-types
subsystems: [SS-15]
target_module: slideforge-types
behavioral_contracts: [BC-1.02.003]
verification_properties: []
nfr_refs: [NFR-021, NFR-022, NFR-023, NFR-024, NFR-025]
depends_on: [STORY-001]
blocks:
  - STORY-011
  - STORY-012
  - STORY-014
estimated_days: 2
---

# STORY-004: Value System + EMU Types

## Summary

Expand and harden the `Value` enum and `Emu` type defined as skeletons in STORY-001.
This story adds the compile-time enforcement that no implicit coercion paths exist
between `Value` variants — satisfying BC-1.02.003. It also completes the EMU arithmetic
system with the 11-level precedence enum (`MergePrecedence`) and the `TypeKind`
descriptor used in type-error messages (E-EVL-003). The `Value` type system is the
foundation of the no-coercion invariant (DI-004): the Rust type system itself is used
to enforce that you cannot call `Value::Bool` on an integer without going through an
explicit conversion function.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,000 |
| `src/value.rs` expanded implementation | ~3,000 |
| `src/emu.rs` completed implementation | ~2,000 |
| `src/precedence.rs` new module | ~1,000 |
| `src/type_kind.rs` new module | ~800 |
| Unit tests | ~2,500 |
| **Total** | **~12,300** |

Agent context budget: 200k tokens. This story is ~6.2% of budget — well within limit.

## Behavioral Contracts

| BC ID | Title | Clauses Covered |
|-------|-------|----------------|
| BC-1.02.003 | No implicit type coercion — values are never silently converted | Preconditions 1-2; Postconditions 1-5; Invariants 1-3 |

## Acceptance Criteria

- [ ] **AC-001:** `Value` enum has exactly 7 variants as defined in STORY-001 (`Str`, `Int`, `Float`, `Bool`, `List`, `Map`, `Null`). No `From<bool> for Value`, `From<i64> for Value`, or `From<f64> for Value` implementations exist. Verified by a compile-fail doctest that attempts `let _: Value = true;`. (traces to BC-1.02.003 precondition 1 and invariant 1)
- [ ] **AC-002:** `Value::Str(Arc::from("NO"))` interpolated through the system remains `Arc::from("NO")` — it is never coerced to `Value::Bool(false)`. Verified by unit test asserting `Value::Str(Arc::from("NO")) != Value::Bool(false)`. (traces to BC-1.02.003 postcondition 2)
- [ ] **AC-003:** `Value::Str(Arc::from("1.10"))` remains exactly `Arc::from("1.10")` — not truncated to `Arc::from("1.1")`. Verified by unit test asserting the exact string content. (traces to BC-1.02.003 postcondition 3)
- [ ] **AC-004:** `TypeKind` enum is defined with variants: `Str`, `Int`, `Float`, `Bool`, `List`, `Map`, `Null` — plus a `Display` impl producing human-readable names (`"string"`, `"integer"`, `"float"`, `"boolean"`, `"list"`, `"map"`, `"null"`). (traces to BC-1.02.003 invariant 3 — type errors report actual and expected types)
- [ ] **AC-005:** `Value::type_kind(&self) -> TypeKind` method exists and returns the correct `TypeKind` for each variant. Unit test verifies all 7 variants. (traces to BC-1.02.003 invariant 3)
- [ ] **AC-006:** `Value::is_truthy(&self) -> bool` method exists for `@if` condition evaluation. Returns: `Bool(b) => b`, `Int(n) => n != 0`, `Float(f) => *f != 0.0`, `Str(s) => !s.is_empty()`, `List(l) => !l.is_empty()`, `Map(m) => !m.is_empty()`, `Null => false`. This method does NOT coerce — it is an explicit truthiness check (evaluator calls it explicitly, never implicitly). Documented as such in rustdoc. (traces to BC-1.02.003 invariant 1 — no implicit coercion, explicit conversion allowed)
- [ ] **AC-007:** `MergePrecedence` enum is defined with exactly 11 variants in precedence order (lowest to highest): `Default`, `SlideTypeDefault`, `SetRule`, `DataSource`, `DeckVar`, `IncludeVar`, `VariantVar`, `CliFlag`, `PerSlideOverride`, `BrandOverlay`, `Internal`. Implements `PartialOrd + Ord + Hash + Eq + Clone + Copy + Debug`. (traces to BC-1.02.003 invariant 1 — precedence chain is the merge mechanism, not coercion)
- [ ] **AC-008:** `Emu::from_inches(1.0)` == `Emu(914_400)`. Verified by unit test. (traces to architecture/ir-design.md EMU System section)
- [ ] **AC-009:** `Emu::from_points(1.0)` == `Emu(12_700)`. Verified by unit test. (traces to architecture/ir-design.md)
- [ ] **AC-010:** `Emu` implements `Add<Emu>`, `Sub<Emu>`, `Mul<i64>`, `Div<i64>`, `Neg`. All operations return `Emu`. Unit tests verify arithmetic correctness and that standard 16:9 slide dimensions compute correctly: `Emu::from_inches(10.0) == Emu(9_144_000)` (width), `Emu(5_143_500)` (height). (traces to architecture/ir-design.md)
- [ ] **AC-011:** `Emu::to_points(self) -> f64` and `Emu::to_inches(self) -> f64` exist for export boundaries. Verified by unit test roundtrip. (traces to architecture/ir-design.md §Coordinate Mapping at Export Boundaries)
- [ ] **AC-012:** `Value::as_str(&self) -> Option<&str>` and `Value::as_int(&self) -> Option<i64>` and `Value::as_float(&self) -> Option<f64>` and `Value::as_bool(&self) -> Option<bool>` accessor methods exist and return `None` for mismatched types (NOT coercion — just pattern matching). (traces to BC-1.02.003 invariant 1 — explicit extraction, not coercion)
- [ ] **AC-013:** `cargo clippy -p slideforge-types -- -D warnings` produces zero new warnings from this story's additions (NFR-022).
- [ ] **AC-014:** All public items added in this story have rustdoc comments (NFR-023). Specifically: `TypeKind`, `MergePrecedence`, `Value::type_kind`, `Value::is_truthy`, `Value::as_str`, `Value::as_int`, `Emu::from_inches`, `Emu::from_points`, `Emu::to_points`.

## Previous Story Intelligence

STORY-001 created the skeleton `Value` enum and `Emu` newtype. This story EXPANDS those
definitions — it does not replace them. Before implementing, read `src/value.rs` and
`src/emu.rs` from STORY-001 to understand what already exists. Do not duplicate definitions.

Key watch items from STORY-001 design:
- `Value::Float` uses `ordered_float::OrderedFloat<f64>`, not raw `f64`.
- `Value::Map` uses `IndexMap<Arc<str>, Value>`, not `HashMap`.
- `Arc<str>` is the string type throughout.

## Architecture Compliance Rules

Sourced from `architecture/crate-architecture.md`, `architecture/ir-design.md`, and `architecture/system-overview.md`:

1. **No coercion impls (BC-1.02.003 invariant 1, DI-004):** Do NOT add `From<bool> for Value`, `From<i64> for Value`, `From<f64> for Value`, `From<String> for Value`, or any other implicit conversion. The only value constructors are the enum variants themselves.
2. **Pure Core (SS-15):** This is all pure type code. No I/O, no async, no network.
3. **`OrderedFloat` for floats (ir-design.md):** `Value::Float` continues to use `ordered_float::OrderedFloat<f64>`. `Value::as_float()` returns `Option<f64>` (unwrapping the `OrderedFloat` wrapper at the API boundary).
4. **EMU as `i64` everywhere (ADR-013):** `Emu` remains a newtype over `i64`. Float conversion happens ONLY in `from_inches`/`from_points` (at the DSL boundary). All internal arithmetic uses integer `Emu` operations.
5. **Forbidden dependencies:** This code lives in `slideforge-types`. MUST NOT import from `slideforge-eval`, `slideforge-syntax`, or any effectful crate.

## Library and Framework Requirements

| Library | Pinned Version | Usage |
|---------|---------------|-------|
| `ordered-float` | `=4.6.0` | Already in crate from STORY-001; `OrderedFloat<f64>` for `Value::Float` |
| `indexmap` | `=2.7.1` | Already in crate from STORY-001; `IndexMap` for `Value::Map` |
| `thiserror` | `=2.0.18` | Already in crate |

No new production dependencies are required for this story.

## File Structure Requirements

Files to CREATE (new modules):

```
crates/slideforge-types/src/
├── type_kind.rs                     # TypeKind enum + Display impl
└── precedence.rs                    # MergePrecedence enum (11 levels)
```

Files to MODIFY (expand STORY-001 stubs):

```
crates/slideforge-types/src/
├── value.rs                         # Add: type_kind(), is_truthy(), as_str/int/float/bool()
├── emu.rs                           # Add: Div<i64>, Neg, to_points(), to_inches()
└── lib.rs                           # Add: pub mod type_kind; pub mod precedence;
```

## Tasks

1. **Create `src/type_kind.rs`** — `TypeKind` enum with 7 variants and `Display` impl. (15 min)
2. **Create `src/precedence.rs`** — `MergePrecedence` enum with 11 variants in precedence order, `PartialOrd`/`Ord` derived. (15 min)
3. **Expand `src/value.rs`:**
   - Add `type_kind(&self) -> TypeKind` method.
   - Add `is_truthy(&self) -> bool` method.
   - Add `as_str`, `as_int`, `as_float`, `as_bool` accessor methods.
   - Add explicit rustdoc to each method explaining "this is NOT coercion."
   - Add compile-fail test (doctest) verifying no implicit `From` impls. (45 min)
4. **Expand `src/emu.rs`:**
   - Add `Div<i64>` impl.
   - Add `Neg` impl.
   - Add `to_points(self) -> f64` method.
   - Add `to_inches(self) -> f64` method.
   - Add `SLIDE_WIDTH` and `SLIDE_HEIGHT` constants if not already in STORY-001. (20 min)
5. **Update `src/lib.rs`** — add `pub mod type_kind;` and `pub mod precedence;`. (5 min)
6. **Write unit tests** in each new module's `#[cfg(test)]` block (see Test Strategy). (30 min)
7. **Run `cargo test -p slideforge-types`** — confirm all tests pass. (5 min)
8. **Run `cargo clippy -p slideforge-types -- -D warnings`** — fix any warnings. (10 min)
9. **Run `RUSTDOCFLAGS="-D warnings" cargo doc -p slideforge-types --no-deps`** — fix any missing-docs warnings. (10 min)

## Test Strategy

### Unit tests (`src/value.rs` `#[cfg(test)]`)

```rust
#[test]
fn str_no_stays_str() {
    let v = Value::Str(Arc::from("NO"));
    assert_ne!(v, Value::Bool(false));
    assert_eq!(v.type_kind(), TypeKind::Str);
}

#[test]
fn float_precision_preserved() {
    // "1.10" must not become "1.1" (BC-1.02.003 postcondition 3)
    let s = "1.10";
    let v = Value::Str(Arc::from(s));
    assert_eq!(v.as_str().unwrap(), "1.10");
}

#[test]
fn is_truthy_variants() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(Value::Int(1).is_truthy());
    assert!(!Value::Int(0).is_truthy());
    assert!(Value::Str(Arc::from("x")).is_truthy());
    assert!(!Value::Str(Arc::from("")).is_truthy());
    assert!(!Value::Null.is_truthy());
}

#[test]
fn as_bool_does_not_coerce() {
    // Value::Str("true") should NOT become Some(true)
    let v = Value::Str(Arc::from("true"));
    assert_eq!(v.as_bool(), None);  // None, not Some(true)
    // Value::Int(1) should NOT become Some(true)
    let v = Value::Int(1);
    assert_eq!(v.as_bool(), None);  // None, not Some(true)
}

#[test]
fn type_kind_all_variants() {
    let cases = [
        (Value::Str(Arc::from("")), TypeKind::Str),
        (Value::Int(0), TypeKind::Int),
        (Value::Float(ordered_float::OrderedFloat(0.0)), TypeKind::Float),
        (Value::Bool(false), TypeKind::Bool),
        (Value::List(vec![]), TypeKind::List),
        (Value::Map(indexmap::IndexMap::new()), TypeKind::Map),
        (Value::Null, TypeKind::Null),
    ];
    for (v, expected) in cases {
        assert_eq!(v.type_kind(), expected, "type_kind mismatch for {:?}", v);
    }
}
```

### Unit tests (`src/emu.rs` `#[cfg(test)]`)

```rust
#[test]
fn from_inches_one_inch() {
    assert_eq!(Emu::from_inches(1.0), Emu(914_400));
}

#[test]
fn from_points_one_point() {
    assert_eq!(Emu::from_points(1.0), Emu(12_700));
}

#[test]
fn slide_dimensions() {
    // Standard 16:9: 10in × 5.625in
    assert_eq!(SLIDE_WIDTH, Emu(9_144_000));
    assert_eq!(SLIDE_HEIGHT, Emu(5_143_500));
}

#[test]
fn emu_arithmetic() {
    assert_eq!(Emu(100) + Emu(200), Emu(300));
    assert_eq!(Emu(300) - Emu(100), Emu(200));
    assert_eq!(Emu(100) * 3, Emu(300));
    assert_eq!(Emu(300) / 3, Emu(100));
    assert_eq!(-Emu(100), Emu(-100));
}

#[test]
fn to_points_roundtrip() {
    let emu = Emu::from_points(72.0);  // 72pt = 1 inch
    let back = emu.to_points();
    assert!((back - 72.0).abs() < 0.001);
}
```

### Unit tests (`src/precedence.rs` `#[cfg(test)]`)

```rust
#[test]
fn precedence_ordering() {
    assert!(MergePrecedence::Default < MergePrecedence::SlideTypeDefault);
    assert!(MergePrecedence::SlideTypeDefault < MergePrecedence::SetRule);
    assert!(MergePrecedence::CliFlag < MergePrecedence::PerSlideOverride);
    assert!(MergePrecedence::PerSlideOverride < MergePrecedence::BrandOverlay);
    assert!(MergePrecedence::BrandOverlay < MergePrecedence::Internal);
}
```

### Snapshot tests

None in this crate.

### Compile-fail test (BC-1.02.003 invariant 1)

Include in `src/value.rs` as a doctest:

```rust
/// Demonstrates that Value does NOT support implicit conversion from bool.
///
/// ```compile_fail
/// use slideforge_types::Value;
/// let _: Value = true; // should not compile
/// ```
```

This is the strongest possible verification of the no-coercion invariant — the Rust
compiler itself enforces it.

## Dependencies

**Depends on:** STORY-001 (IR core types, specifically `Value` and `Emu` skeletons).

Dependency justification: STORY-004 expands the `Value` and `Emu` types defined in STORY-001. Without STORY-001's initial definitions, there is nothing to expand.

**Blocks:**
- STORY-011 (expression evaluator core): the evaluator emits E-EVL-003 type errors using `TypeKind` to name actual and expected types. `Value::type_kind()` and `Value::is_truthy()` are required by the evaluator.
- STORY-012 (variable scoping): scoping implementation uses `Value::type_kind()` in error messages.
- STORY-014 (type system — no-coercion story): that story tests the evaluator's behavior; this story provides the type-level enforcement.

## Implementation Notes

### Explicit Conversion vs. Coercion

The distinction between allowed explicit conversion and forbidden coercion:

| Pattern | Allowed? | Reason |
|---------|---------|--------|
| `Value::as_str()` returning `None` for non-strings | Yes | Explicit check; caller handles `None` |
| `Value::as_bool()` returning `None` for `Value::Int(1)` | Yes | No coercion — `Int(1)` is not `Bool(true)` |
| `let b: bool = value.as_bool().unwrap_or(false)` | Yes | Caller handles explicitly; no implicit coercion |
| `From<bool> for Value` | FORBIDDEN | Would allow `let v: Value = true;` — implicit coercion |
| `Value::Bool(true) == Value::Int(1)` | Returns `false` | They are different variants; `Eq` does not coerce |
| Pipe filter `| bool` in an expression | Yes (future) | Explicit operator; implemented in evaluator |

### `is_truthy` Rustdoc Comment

```rust
/// Evaluates the truthiness of a value for use in `@if` condition evaluation.
///
/// This method does NOT perform type coercion. It is an explicit truthiness
/// check that is only called by the evaluator when processing an `@if`
/// condition. The evaluator explicitly calls `.is_truthy()` — it is never
/// called implicitly during arithmetic or comparison.
///
/// Per BC-1.02.003: a string value like `"NO"` evaluates to truthy (non-empty string),
/// NOT to false. Use `@if var == false` to compare a boolean, not `@if var`.
pub fn is_truthy(&self) -> bool { ... }
```

### `MergePrecedence` — 11 Levels (from q1-decision-final.md §merge-semantics)

The 11-level chain from lowest to highest precedence:
1. `Default` — hardcoded defaults in slide type definitions
2. `SlideTypeDefault` — defaults declared via `default:` in `optional_fields()`
3. `SetRule` — `set <type>: <field> <value>` rules
4. `DataSource` — values from `@data` bindings
5. `DeckVar` — `vars:` block in deck metadata
6. `IncludeVar` — vars passed through `@include`
7. `VariantVar` — `variants: { vars: ... }` overrides
8. `CliFlag` — `--variant` and other CLI overrides
9. `PerSlideOverride` — explicit field values on individual slides
10. `BrandOverlay` — `brand_overlay:` per-slide overrides
11. `Internal` — system-internal values (e.g., generated slide numbers)

### `TypeKind::Display` Format

```rust
impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Str => write!(f, "string"),
            TypeKind::Int => write!(f, "integer"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Bool => write!(f, "boolean"),
            TypeKind::List => write!(f, "list"),
            TypeKind::Map => write!(f, "map"),
            TypeKind::Null => write!(f, "null"),
        }
    }
}
```

E-EVL-003 error message format (used by evaluator in STORY-011):
`Type mismatch: operator '*' expects numeric, got string. Hint: use | int or | float filter.`

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `Value::Float(OrderedFloat(f64::NAN)).is_truthy()` | `OrderedFloat::nan() != 0.0` evaluates to `true` — NaN is truthy (consistent with OrderedFloat behavior) |
| EC-002 | `Value::Str(Arc::from("")).is_truthy()` | `false` — empty string is falsy |
| EC-003 | `Value::List(vec![Value::Null]).is_truthy()` | `true` — non-empty list even if it contains only `Null` |
| EC-004 | `Emu::from_inches(0.0)` | `Emu(0)` — valid, used for zero-offset positioning |
| EC-005 | `Emu` overflow (`Emu(i64::MAX) + Emu(1)`) | Debug: panic (overflow check). Release: wraps. Acceptable — slide coordinates never approach `i64::MAX`. Kani proves bounds in STORY-066. |
| EC-006 | `MergePrecedence::Internal` used in user-facing DSL | Not possible — `Internal` is only set by system code, not by any DSL construct. |
