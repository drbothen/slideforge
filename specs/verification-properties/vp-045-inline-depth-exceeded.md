---
document_type: verification-property
vp_id: VP-045
title: "Inline: tree at depth 65 produces InlineDepthExceeded error"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-3.05.001, DI-018]
anchored_to_bc: BC-3.05.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-045: Inline Tree at Depth 65 Produces InlineDepthExceeded Error

## Property Statement

For any `InlineNode` tree with nesting depth >= 65, the layout-stage inline validator
MUST return `Err(LayoutError::InlineDepthExceeded { slide_index, depth: 65 })` (or
the actual exceeded depth). A tree at depth 64 MUST NOT trigger this error.

Formally:
- `inline_depth(tree) >= 65 → Err(LayoutError::InlineDepthExceeded { depth: actual })`
- `inline_depth(tree) <= 64 → Ok(())` (depth check passes)

The depth bound is exactly 64 (trees of depth 1-64 are accepted; depth 65+ is rejected).

## Motivation

BC-3.05.001 Invariant 4 and EC-006. The inline tree depth bound prevents stack overflow
during recursive tree traversal in all three export paths (PPTX XML generation, HTML
generation, PDF span building). 64 levels is a conservative bound that no legitimate
document will approach. DI-018 requires this to be a hard error, not a silent truncation.

## Feasibility Assessment

Kani-amenable: the depth-counting function `inline_tree_depth(node: &InlineNode) -> usize`
is a pure recursive function with a bounded termination condition. Kani can verify:
1. A tree of depth exactly 64 passes the check.
2. A tree of depth 65 fails.
The unwind bound needs to cover the recursion depth (65).

`kani_amenable: true` — pure recursive depth measurement.

## Proof Harness Skeleton

```rust
// crates/slideforge-layout/src/proofs/inline_depth.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    fn make_nested_bold(depth: usize) -> InlineNode {
        if depth == 0 {
            InlineNode::Plain(Arc::from("leaf"))
        } else {
            InlineNode::Bold(vec![make_nested_bold(depth - 1)])
        }
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn depth_64_is_accepted() {
        let tree = make_nested_bold(63); // 64 levels total (0..=63)
        let depth = inline_tree_depth(&tree);
        kani::assert!(depth == 64);
        kani::assert!(check_inline_depth(&tree).is_ok());
    }

    #[kani::proof]
    #[kani::unwind(70)]
    fn depth_65_is_rejected() {
        let tree = make_nested_bold(64); // 65 levels total (0..=64)
        let result = check_inline_depth(&tree);
        kani::assert!(matches!(result, Err(LayoutError::InlineDepthExceeded { depth: 65, .. })));
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-layout/src/tests/inline_depth.rs
fn make_nested_bold(depth: usize) -> InlineNode {
    if depth == 0 { InlineNode::Plain(Arc::from("x")) }
    else { InlineNode::Bold(vec![make_nested_bold(depth - 1)]) }
}

#[test]
fn test_depth_64_accepted() {
    let tree = make_nested_bold(63); // 64 total levels
    assert!(check_inline_depth(0, &tree).is_ok());
}

#[test]
fn test_depth_65_rejected() {
    // BC-3.05.001 canonical test vector: 65-deep nested Bold nodes
    let tree = make_nested_bold(64); // 65 total levels
    let err = check_inline_depth(0, &tree).expect_err("depth 65 must be rejected");
    assert!(matches!(err, LayoutError::InlineDepthExceeded { slide_index: 0, depth: 65, .. }));
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `Bold(Bold(... 64 levels ...))` | `Ok(())` — depth 64 accepted | boundary |
| `Bold(Bold(... 65 levels ...))` | `LayoutError::InlineDepthExceeded { depth: 65 }` | depth-bound (EC-006) |
| `Plain("text")` (depth 1) | `Ok(())` | happy-path |
