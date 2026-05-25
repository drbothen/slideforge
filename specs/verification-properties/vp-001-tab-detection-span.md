---
document_type: verification-property
vp_id: VP-001
title: Tab detection byte span accuracy
module: slideforge-syntax
tool: Kani
phase: P6
priority: P0
status: draft
bc_trace: [BC-1.01.003, DI-018]
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-001: Tab detection byte span accuracy

## Property Statement

For any input string containing a tab character in an indentation position,
the lexer MUST produce a `LexError::TabInIndent` with a `SimpleSpan` whose
`start` and `end` byte offsets identify the exact byte position of the tab.

Formally: `∀ input: &str, ∀ tab_pos: usize where input[tab_pos] == '\t' and tab_pos is indentation`,
`lex(input).errors.any(|e| matches!(e.kind, TabInIndent) && e.span.start == tab_pos)`.

## Motivation

BC-1.01.003 requires tab indentation to produce a hard error with file:line:col span.
Incorrect spans send users to the wrong location in their editor. This is a high-frequency
user-facing correctness property — the first thing every slideforge user will encounter
when coming from YAML or Python habits.

## Feasibility Assessment

Feasible. The hand-written indentation lexer tracks byte offsets explicitly for every
character scanned. The `tab_pos` is the byte index when the lexer encounters `\t` while
scanning leading whitespace. Kani bounded-model checking over strings of length ≤ 256
bytes is O(length) and within Kani's model-checking budget.

## Proof Harness Skeleton

```rust
// crates/slideforge-syntax/src/proofs/tab_detection.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    #[kani::unwind(300)]
    fn tab_span_accuracy() {
        // Create an input with a single tab at a known indentation position
        let tab_pos: usize = kani::any();
        kani::assume(tab_pos < 200);

        // Build: spaces up to tab_pos, then a tab, then identifier text
        let mut input = String::new();
        for _ in 0..tab_pos { input.push(' '); }
        input.push('\t');
        input.push_str("field: value");

        let result = lex(&input);
        let tab_error = result.errors.iter().find(|e| {
            matches!(e.kind, LexErrorKind::TabInIndent)
        });
        kani::assert!(tab_error.is_some());
        let err = tab_error.unwrap();
        kani::assert!(err.span.start == tab_pos);
        kani::assert!(err.span.end == tab_pos + 1);
    }
}
```

## Test Coverage (before Phase 6)

Concrete unit test in `slideforge-syntax/src/lexer.rs`:
`test_tab_in_indent_produces_error` — confirmed passing in S4 spike.
