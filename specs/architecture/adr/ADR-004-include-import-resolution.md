---
document_type: adr
adr_id: ADR-004
title: Include/import resolution semantics
status: accepted
date: 2026-05-24
spike_input: ~
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-004: Include/Import Resolution Semantics

## Context

The slideforge DSL supports two composition mechanisms: `@include` (inline-at-parse-time
from local files) and `@import` (package-level dependency resolution). They have distinct
semantics and must be resolved differently. Cycles must be detected and rejected (DI-007).

## Decision

**`@include`:** Resolved in `slideforge-syntax` at parse time. The referenced `.sf` file is
located relative to the including file's directory, read from disk (in the lexer phase,
before the chumsky parser), and inlined as token stream. Cycle detection uses a stack of
currently-open file paths; any attempt to include a path already on the stack is a
compile error (E-INC-001) with the full cycle path shown.

**`@import`:** Resolved in `slideforge-eval` after package installation. `@import` refers
to an installed package (from `sf.lock`). Resolution fails with a descriptive install hint
(E-IMP-001) if the package is not installed. Cycles in `@import` chains are detected at
package installation time, not at eval time.

## Consequences

**Positive:**
- Clean separation: `@include` is a file-composition mechanism; `@import` is a package
  dependency mechanism. They have different error codes and different recovery paths.
- Cycle detection for `@include` is O(depth) per include using a HashSet of canonical paths.
- Inline at parse time means `@include` files inherit the lexical scope of their inclusion point.

**Implementation notes:**
- `@include` resolution must use `std::fs::canonicalize()` to normalize paths before cycle
  comparison (handles `../relative/../../paths`).
- The span of an `@include` error points to the `@include` directive in the including file,
  plus a secondary label pointing to where the cycle begins.
- `@import` resolution must check `sf.lock` before disk; if lock says package is present
  but installation directory is missing, produce E-IMP-002 (package corrupted, re-install).
