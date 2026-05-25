---
document_type: architecture-section
section: verification-architecture
version: "1.0"
status: approved
producer: architect
timestamp: 2026-05-24T00:00:00
traces_to: ARCH-INDEX.md
---

# Verification Architecture

## Proof Strategy by Category

### Must Prove (Kani, Phase 6)

Security boundaries, arithmetic invariants, and safety-critical state properties
in pure-core crates.

| Module | Property | Kani Proof |
|--------|----------|-----------|
| slideforge-syntax | Tab detection produces error with byte-accurate span | VP-001 |
| slideforge-validate | Alt-missing produces error before layout (in validation stage) | VP-002 |
| slideforge-syntax | @for over bounded collection always terminates | VP-003 |
| slideforge-eval | No implicit coercion: "1.10" string stays string | VP-004 |
| slideforge-eval | Integer arithmetic in {{ expr }} cannot overflow given i64 bounds | VP-005 |
| slideforge-pdf | EMU-to-PDF coordinate mapping: correct Y-axis flip, no overflow | VP-006 |
| slideforge-validate | WCAG contrast formula: correct luminance linearization (0.04045 threshold) | VP-007 |
| slideforge-validate | Alt enforcement: image with AltText::Text("") always produces diagnostic | VP-008 |

### Should Prove (proptest, Phase 6)

Core algorithms with complex invariants where exhaustive proof is infeasible but
property-based testing provides strong guarantees.

| Module | Property | Tool |
|--------|----------|------|
| slideforge-syntax | Parse of any valid .sf source produces non-empty AST | proptest |
| slideforge-eval | Variable scoping: outer @for vars visible in inner scope | proptest |
| slideforge-layout | LaidOutDeck always has same slide count as Deck | proptest |
| slideforge-brand | 12-slot palette round-trip: synthesize → extract → same colors | proptest |
| slideforge-pptx | Every synthesized PPTX is a valid ZIP with [Content_Types].xml | proptest |

### Test Sufficient (integration + snapshot)

UI logic, CLI behavior, non-critical rendering paths.

| Module | Coverage Method |
|--------|----------------|
| slideforge-cli | Integration tests: CLI command matrix |
| slideforge-preview | axe-core/playwright + Playwright E2E |
| slideforge-diagrams | Snapshot tests: SVG output per diagram type |
| slideforge-charts | Snapshot tests: SVG output per chart type |

## P0 Verification Properties (Phase 6 blocking)

Must pass before v1.0 release (formal-verifier gate):
- VP-001: Tab detection byte span accuracy
- VP-002: Alt-missing produces error before layout (in validation stage)
- VP-003: @for termination
- VP-004: No implicit coercion
- VP-006: PDF coordinate mapping
- VP-007: WCAG contrast formula
- VP-008: Alt diagnostic invariant

## P1 Verification Properties (Phase 6 stretch goals)

- VP-005: Integer arithmetic bounds in eval
- proptest suites for parser, eval, layout, brand round-trip

## Tooling (ADR-011, Feasibility Notes)

| Tool | Purpose | Platform | Phase |
|------|---------|----------|-------|
| Kani | Formal model checking — pure functions | Linux/macOS only | 6 |
| cargo-fuzz | Fuzzing — parser and eval inputs | Linux CI | 6 |
| cargo-mutants | Mutation testing — kill-rate budget | All | 6 |
| proptest | Property-based tests — algebraic properties | All | 3+ |
| @axe-core/playwright | WCAG AA web surface | CI (Node.js in test) | 3+ |
| veraPDF | PDF/UA-1 compliance | CI (Docker sidecar) | 4+ |
| custom OOXML linter | PPTX accessibility rules A1-A4+ | All | 3+ |

## Kani Proof Skeleton (VP-006 example)

```rust
// crates/slideforge-pdf/src/proofs/coordinate_mapping.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(10)]
    fn y_axis_flip_no_overflow() {
        let ir_y: i64 = kani::any();
        let elem_h: i64 = kani::any();
        // Constrain to valid slide dimensions
        kani::assume(ir_y >= 0 && ir_y <= SLIDE_HEIGHT_EMU);
        kani::assume(elem_h >= 0 && elem_h <= SLIDE_HEIGHT_EMU);
        kani::assume(ir_y + elem_h <= SLIDE_HEIGHT_EMU);
        // The Y-axis flip must not overflow i64 or produce negative values
        let result = ir_y_to_pdf_y(Emu(ir_y), Emu(elem_h));
        kani::assert(result >= 0.0);
        kani::assert(result <= SLIDE_HEIGHT_PT);
    }
}
```

Proof harness skeletons for VP-001 through VP-008 live in
`crates/<crate>/src/proofs/<vp-name>.rs`. They are not compiled in normal builds
(gated by `#[cfg(kani)]`).
