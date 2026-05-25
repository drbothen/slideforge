---
document_type: verification-property
vp_id: VP-002
title: Alt-missing produces error before layout (in validation stage)
module: slideforge-validate
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-5.01.001, DI-001]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-002: Alt-Missing Produces Error Before Layout (in Validation Stage)

## Property Statement

For any parsed `VisualElement` (image, chart, diagram) where the `alt` field is absent
AND `decorative: true` is absent, the parse result's error set must be non-empty and
contain at least one diagnostic of kind `MissingAlt`.

Formally: `∀ ast: Document where ast.contains_visual_element_without_alt()`,
`parse_output.errors.any(|e| e.kind == MissingAlt)`.

## Motivation

BC-5.01.001 requires that missing alt text is a compile error, not a warning.
DI-001 (domain invariant) makes this an unconditional business rule. The property
must hold regardless of the rest of the document's validity — a valid-looking slide
with a missing alt is still rejected.

## Feasibility Assessment

Feasible. The chumsky `validate()` mechanism emits non-terminal errors without stopping
the parse (confirmed in S3 spike: 5/5 tests pass). The property is a boolean check over
the AST node's `alt_text` field, which is a discriminated union `AltText::Text(s) | AltText::Decorative`.
Kani can model-check this over a bounded document structure (≤ 10 slides, ≤ 5 visual elements per slide).

## Proof Harness Skeleton

```rust
// crates/slideforge-validate/src/proofs/alt_enforcement.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(20)]
    fn missing_alt_produces_error() {
        // Construct a minimal document with one image lacking alt text
        let source = "slide content:\n  image: photo.png\n";
        let result = parse(source);
        // Image without alt or decorative MUST produce a MissingAlt error
        let has_missing_alt = result.errors.iter().any(|e| {
            matches!(e.kind, ParseErrorKind::MissingAlt)
        });
        kani::assert!(has_missing_alt);
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit test in `slideforge-validate/src/validator.rs`:
`test_image_without_alt_is_error`. Confirmed pattern from S3 spike.
