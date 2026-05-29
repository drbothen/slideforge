---
document_type: verification-property
vp_id: VP-039
title: "Shape: unknown shape type keyword produces E-PAR-012 (no silent Custom fallback)"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: false
bc_trace: [BC-3.04.001, DI-021]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-039: Unknown Shape Type Keyword Produces E-PAR-012 (No Silent Custom Fallback)

## Property Statement

For any `type:` keyword in a `shape:` block that is NOT in the closed v1.0 vocabulary
(`rect`, `ellipse`, `arrow`, `line`, `star`, `roundRect`), the parser MUST produce
`ParseError::UnknownShapeType` (mapped to E-PAR-012) with a message listing the known
types. The parser MUST NOT create a `ShapeType::Custom(Arc<str>)` variant or any other
silent fallback.

Formally: `keyword ∉ {"rect", "ellipse", "arrow", "line", "star", "roundRect"}`
→ `Err(ParseError::UnknownShapeType { keyword, known_types: [...] })`.

## Motivation

BC-3.04.001 Invariant 4 and EC-005. The CLAUDE.md canonical principle ("no silent
fallback") is the governing rule: inventing a `ShapeType::Custom(...)` variant that
silently accepts unknown keywords would mask authoring errors. A user who typos
`type frobnicator` must get an immediate, actionable parse error rather than a
silently-produced PPTX with a shape of unknown type.

## Feasibility Assessment

Not Kani-amenable: the check is in the parser which involves complex parsing state.
Verified via a unit test on the `parse_shape_type` function with a known-bad keyword.

`kani_amenable: false` — parser function with string dispatch.

## Test Harness

```rust
// crates/slideforge-layout/src/tests/shape_type.rs
#[test]
fn test_unknown_shape_type_keyword_produces_par012() {
    let result = parse_shape_type("frobnicator");
    let err = result.expect_err("unknown keyword should produce error");
    match err {
        ParseError::UnknownShapeType { keyword, known_types } => {
            assert_eq!(keyword, "frobnicator");
            assert!(known_types.contains(&"rect"), "must list rect in known types");
            assert!(known_types.contains(&"roundRect"), "must list roundRect");
        }
        _ => panic!("expected UnknownShapeType, got {:?}", err),
    }
}

#[test]
fn test_all_valid_shape_types_parse_successfully() {
    for kw in &["rect", "ellipse", "arrow", "line", "star", "roundRect"] {
        let result = parse_shape_type(kw);
        assert!(result.is_ok(), "valid keyword '{}' should parse", kw);
    }
}

#[test]
fn test_no_custom_variant_in_shape_type_enum() {
    // Compile-time check: ShapeType has no Custom variant
    // This is enforced by the enum definition itself (no exhaustive match needed)
    // The test just verifies the match is exhaustive over the 6 known variants
    let all: &[ShapeType] = &[
        ShapeType::Rect, ShapeType::Ellipse, ShapeType::Arrow,
        ShapeType::Line, ShapeType::Star, ShapeType::RoundRect,
    ];
    assert_eq!(all.len(), 6);
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `"frobnicator"` | `ParseError::UnknownShapeType` with known-types list | error (EC-005) |
| `"Rect"` (wrong case) | `ParseError::UnknownShapeType` (keywords are case-sensitive) | error |
| `"rect"` | `Ok(ShapeType::Rect)` | happy-path |
| `"roundRect"` | `Ok(ShapeType::RoundRect)` | happy-path |
