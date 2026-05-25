---
document_type: verification-property
vp_id: VP-008
title: Alt diagnostic invariant — image with empty alt always produces diagnostic
module: slideforge-validate
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-5.01.001, DI-001]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-008: Alt Diagnostic Invariant

## Property Statement

For any `Deck` containing a `VisualElement` where `alt_text == AltText::Text(s)` and
`s.is_empty()`, the `AltTextValidator::validate(&deck, &brand)` function MUST return
a `Vec<Diagnostic>` containing at least one entry of kind `DiagnosticKind::MissingAlt`.

Formally: `∀ deck: Deck where deck.contains_empty_alt() → validate(deck).any(is_missing_alt)`.

## Motivation

VP-002 covers the parse-time alt check (syntax layer). VP-008 covers the validation-layer
alt check: after eval, when the full Deck IR is available. The two checks are independent —
a template variable resolving to an empty string at eval time would pass the parse-time
check but must fail the validate-time check. Both must hold independently.

## Feasibility Assessment

Feasible. `AltTextValidator::validate()` is a pure function: `&Deck + &Brand → Vec<Diagnostic>`.
Kani can model-check this with a `Deck` containing a single image with `AltText::Text("")`.
The function has no I/O, no loops over unbounded data (the slide/element structure is bounded
in the Kani model), and the check is a simple string-empty comparison.

## Proof Harness Skeleton

```rust
// crates/slideforge-validate/src/proofs/alt_invariant.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn empty_alt_text_produces_diagnostic() {
        let deck = Deck::with_single_image_empty_alt();
        let brand = Brand::default();
        let diagnostics = AltTextValidator.validate(&deck, &brand);
        let has_missing_alt = diagnostics.iter().any(|d| {
            matches!(d.kind, DiagnosticKind::MissingAlt)
        });
        kani::assert!(has_missing_alt);
    }

    #[kani::proof]
    fn decorative_image_produces_no_missing_alt() {
        let deck = Deck::with_single_image_decorative();
        let brand = Brand::default();
        let diagnostics = AltTextValidator.validate(&deck, &brand);
        let has_missing_alt = diagnostics.iter().any(|d| {
            matches!(d.kind, DiagnosticKind::MissingAlt)
        });
        kani::assert!(!has_missing_alt);
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit test: `test_image_empty_alt_fails_validation` and
`test_decorative_image_passes_validation` in `slideforge-validate/src/tests/alt.rs`.
