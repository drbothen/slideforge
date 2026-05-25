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

For any `Deck` containing a `VisualElement` (image, chart, diagram) where the `alt`
field is absent AND `decorative: true` is absent, calling `validate(&deck, &brand)`
MUST return a non-empty `Vec<Diagnostic>` containing at least one entry of kind
`ValidationErrorKind::MissingAlt`.

Formally: `∀ deck: Deck where deck.contains_visual_element_without_alt()`,
`validate(&deck, &brand).any(|e| e.kind == ValidationErrorKind::MissingAlt)`.

## Motivation

BC-5.01.001 requires that missing alt text is a compile error, not a warning.
DI-001 (domain invariant) makes this an unconditional business rule. The property
must hold regardless of the rest of the document's validity — a valid-looking slide
with a missing alt is still rejected.

## Feasibility Assessment

Feasible. The `slideforge-validate` crate's `validate(&deck, &brand)` is a pure
function over the `Deck` IR — no I/O, no parser invocation. It inspects each
`VisualElement`'s `alt_text` field, which is a discriminated union
`AltText::Text(s) | AltText::Decorative`, and emits `ValidationErrorKind::MissingAlt`
diagnostics without short-circuiting. Kani can model-check this over a bounded
document structure (≤ 10 slides, ≤ 5 visual elements per slide). The S3 spike
confirmed 5/5 validation tests pass with this structural pattern.

## Proof Harness Skeleton

```rust
// crates/slideforge-validate/src/proofs/alt_enforcement.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(20)]
    fn missing_alt_produces_validation_error() {
        // Construct a minimal Deck with one image lacking alt text (no decorative flag)
        let deck = Deck::with_single_image_no_alt();
        let brand = Brand::default();
        let diagnostics = validate(&deck, &brand);
        // Image without alt or decorative MUST produce a MissingAlt validation error
        let has_missing_alt = diagnostics.iter().any(|e| {
            matches!(e.kind, ValidationErrorKind::MissingAlt)
        });
        kani::assert!(has_missing_alt);
    }

    #[kani::proof]
    fn decorative_image_passes_validation() {
        // A decorative image (decorative: true) must NOT produce a MissingAlt error
        let deck = Deck::with_single_image_decorative();
        let brand = Brand::default();
        let diagnostics = validate(&deck, &brand);
        let has_missing_alt = diagnostics.iter().any(|e| {
            matches!(e.kind, ValidationErrorKind::MissingAlt)
        });
        kani::assert!(!has_missing_alt);
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit test in `slideforge-validate/src/validator.rs`:
`test_image_without_alt_is_error`. Confirmed pattern from S3 spike.
