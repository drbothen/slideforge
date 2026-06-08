---
document_type: behavioral-contract
level: L3
version: "1.2"
status: active
producer: product-owner
timestamp: 2026-06-08T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-014
lifecycle_status: active
introduced: v1.0.0
modified: [v1.1, v1.2]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-1.12.003: SVG Normalized via usvg Before PPTX Embedding (no foreignObject, absolute dims)

## Description

Before embedding a diagram SVG into any output format, it is passed through the `usvg`
normalization pipeline. usvg resolves `<use>` references, inlines CSS class-based styles
into presentation attributes, replaces unsupported SVG elements, and ensures all dimensions
are absolute pixel values (no percentage `%` units). This normalization step is required
because PPTX embedding is sensitive to SVG constructs that are legal in browsers but
cause rendering failures in Office applications.

## Preconditions

1. A valid SVG string has been produced by mermaid-rs-renderer (no E-EXP-008).
2. The `usvg` crate is compiled into the slideforge binary (no runtime dependency).

## Postconditions

1. The normalized SVG contains no `<foreignObject>` elements.
2. The normalized SVG contains no `<script>` elements.
3. The normalized SVG contains no CSS `@keyframes` or CSS class-based styles (all inlined to presentation attributes).
4. The normalized SVG root element has `width` and `height` attributes as absolute pixel values (no `%`, no `em`, no `rem`).
5. All `<use href="...">` references are resolved and inlined.
6. The normalized SVG is the form used for all format embeddings (PPTX, HTML, PDF).
7. If usvg fails to normalize the SVG (corrupted input), E-EXP-004 is emitted.
8. **SEC-002 (CWE-400, byte-size cap):** If the raw SVG input byte length exceeds `MAX_SVG_BYTES` (52,428,800 bytes / 50 MiB), `usvg_normalize` returns `Err(DiagramError::SvgNormalizationFailed)` with `cause` containing the actual byte count and the limit (`"SVG input too large: N bytes exceeds the 52428800-byte limit"`). This rejection MUST fire BEFORE any call to `usvg::Tree::from_str` — no heap allocation proportional to the SVG size is permitted. Error code: E-EXP-004.
9. **SEC-001 (CWE-674, nesting-depth cap):** If the parsed usvg `Tree` contains `<g>` group nesting deeper than `MAX_SVG_NESTING_DEPTH` (64 levels, where the root `Tree::root()` counts as depth 1), `usvg_normalize` returns `Err(DiagramError::SvgNormalizationFailed)` with `cause` containing the detected depth and the limit (`"SVG group nesting depth N exceeds the 64-level limit; possible DoS input"`). The depth scan MUST be iterative (stack-based DFS over `Group::children()` matching `usvg::Node::Group`), never recursive. This rejection fires AFTER `usvg::Tree::from_str` succeeds but BEFORE `tree.to_string()` is called. Error code: E-EXP-004.
10. SVG inputs that pass both guards (byte length ≤ `MAX_SVG_BYTES` AND nesting depth ≤ `MAX_SVG_NESTING_DEPTH`) are processed identically to the pre-guard implementation — the guards MUST NOT alter normalization output for legitimate SVG.

## Invariants

1. usvg normalization is applied to EVERY diagram SVG before format embedding — no bypass path.
2. Normalization is applied after mermaid-rs-renderer produces the SVG and before any format-specific exporter receives it.
3. The normalized SVG faithfully represents the same visual diagram as the input SVG.
4. **Guard ordering invariant:** the byte-size guard (SEC-002) MUST precede the nesting-depth guard (SEC-001) MUST precede the `font_db()` call. The font database load (50–300ms, `OnceLock`) must never be paid for an input that will be rejected. An oversized input MUST never reach the allocation-heavy `usvg::Tree::from_str` parse.
5. **Constant parity invariant:** `MAX_SVG_BYTES` (52,428,800) and `MAX_SVG_NESTING_DEPTH` (64) in `slideforge-diagrams/src/normalize.rs` MUST equal the identically-named constants in `slideforge-pdf/src/svg_embed.rs` at all times. If one is changed, both must be changed in the same commit. This prevents split-brain security posture between the two SVG processing paths.
6. **Iterative traversal invariant:** the nesting-depth scan MUST NOT use Rust recursion for tree traversal — use an explicit stack of `(&Group, usize)` tuples. A recursive depth counter would introduce the same stack-exhaustion vector (CWE-674) it is designed to prevent.
7. **Tracing invariant:** both rejection paths MUST emit a `tracing::warn!` event with structured fields (`bytes` for size-cap, depth field for nesting-cap) before returning `Err(...)`. This is required for security observability (NFR-016).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SVG contains `<use href="#symbol">` cross-references | Resolved and inlined by usvg; no dangling references in output |
| EC-002 | SVG uses `width="100%"` (percentage dimensions) | Normalized to absolute px computed from viewBox + container |
| EC-003 | SVG contains `<foreignObject>` (rare but technically valid Mermaid output) | foreignObject removed or replaced by usvg normalization |
| EC-004 | usvg normalization fails (malformed SVG from renderer) | E-EXP-004 emitted; exit 3; error names diagram slide title |
| EC-005 | SVG byte length exactly equals `MAX_SVG_BYTES` (52,428,800 bytes) | PASS — boundary is exclusive (`>` rejects, `==` is allowed) |
| EC-006 | SVG byte length is `MAX_SVG_BYTES + 1` (52,428,801 bytes) | FAIL — `Err(SvgNormalizationFailed)` with E-EXP-004; cause contains actual count and limit |
| EC-007 | SVG group nesting depth exactly equals `MAX_SVG_NESTING_DEPTH` (64 levels) | PASS — boundary is exclusive (`>` rejects, `==` is allowed) |
| EC-008 | SVG group nesting depth is `MAX_SVG_NESTING_DEPTH + 1` (65 levels) | FAIL — `Err(SvgNormalizationFailed)` with E-EXP-004; cause contains detected depth and limit |
| EC-009 | Valid real-world Mermaid flowchart SVG within both limits | PASS — `Ok(NormalizedDiagramSvg)`; no behavioral change from pre-guard implementation |
| EC-010 | SVG with existing malformed content (usvg parse failure), within byte/depth limits | Unchanged — existing `map_err` path fires E-EXP-004 for usvg parse failure; guards do not interact |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| flowchart SVG with CSS class styles (`.nodeLabel { color: red }`) | Normalized: style inlined as presentation attribute; no `<style>` element | happy-path |
| SVG with percentage width | Normalized: width/height in absolute px | happy-path |
| Malformed SVG (from mocked renderer) | E-EXP-004; exit 3 | error |
| SVG of `MAX_SVG_BYTES + 1` bytes (minimal valid SVG + whitespace padding to 52,428,801 bytes) | `Err(SvgNormalizationFailed)` with cause containing "52428800"; no usvg parse attempted | SEC-002 |
| SVG with `MAX_SVG_NESTING_DEPTH + 1` (65) nested `<g>` elements | `Err(SvgNormalizationFailed)` with cause containing detected depth and "64-level limit" | SEC-001 |
| Valid Mermaid flowchart SVG well within both limits (real `simple_geometry_svg()` fixture) | `Ok(NormalizedDiagramSvg)`; normalization output unchanged from pre-guard | regression |

**SEC-002 test construction note:** To avoid allocating a full 50 MiB buffer in the test, use a minimal SVG with whitespace padding:
```
let prefix = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">";
let suffix = b"</svg>";
let padding = MAX_SVG_BYTES + 1 - prefix.len() - suffix.len();
let svg = format!("{}{}{}", str::from_utf8(prefix).unwrap(), " ".repeat(padding), str::from_utf8(suffix).unwrap());
```
The resulting SVG is valid XML but oversized; the size guard fires before any expensive parse.

**SEC-001 test construction note:** Build a string with 65 levels of `<g>` nesting, then close all tags. The parsed `usvg::Tree` will have `Group` depth 65 (root at 1 plus 64 nested `<g>` groups). The test MUST inspect the `cause` field to distinguish the depth-cap path from the size-cap or usvg-parse-failure path.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Normalized SVG has no foreignObject, no script, no % dimensions | unit test: parse normalized SVG, check for forbidden elements/attributes |
| VP-TBD | usvg failure produces E-EXP-004 (not a panic) | unit test with malformed SVG input |
| VP-TBD | All <use> references resolved (no dangling href) | unit test: parse normalized SVG, verify no <use> elements remain |
| VP-TBD | SVG exceeding 50 MiB is rejected with E-EXP-004 before usvg::Tree::from_str is called (SEC-002, CWE-400) | unit test: `test_sec_dos_svg_oversize_rejected` — construct SVG of MAX_SVG_BYTES+1 bytes; verify SvgNormalizationFailed returned with cause containing limit |
| VP-TBD | SVG with group nesting depth > 64 is rejected with E-EXP-004 after parse, before re-serialization (SEC-001, CWE-674) | unit test: `test_sec_dos_svg_deep_nesting_rejected` — construct SVG with 65 nested `<g>` elements; verify SvgNormalizationFailed returned with depth-cap cause |
| VP-TBD | Depth traversal is iterative (not recursive) — provably bounded stack depth | code review + Kani proof (Phase 6): iterative DFS stack depth bounded by MAX_SVG_NESTING_DEPTH |
| VP-TBD | Valid SVG within both limits passes through unaffected (regression) | unit test: `test_sec_dos_svg_valid_input_unaffected` — real-world Mermaid fixture returns Ok(NormalizedDiagramSvg) |
| VP-TBD | NFR-016 tracing obligation: both rejection paths emit tracing::warn! with structured fields | unit test + log capture: verify warn event with correct fields on size-cap and depth-cap paths |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 |
| Capability Anchor Justification | CAP-014 ("Diagram Rendering (Mermaid)") per capabilities.md §CAP-014 — "The output SVG is PPTX-safe by default: no foreignObject, no script, no percentage dimensions" is the normalization requirement; the DoS guards are security preconditions for that normalization to complete safely |
| L2 Domain Invariants | DI-013 (PPTX output must pass multi-renderer fidelity check) |
| Security Invariants | SEC-001 (CWE-674 — nesting-depth cap, MAX_SVG_NESTING_DEPTH=64); SEC-002 (CWE-400 — byte-size cap, MAX_SVG_BYTES=52428800) |
| NFR References | NFR-016 (security observability / tracing), NFR-021 (no .unwrap() in non-test code), NFR-022 (zero panics), NFR-023 (no unsafe code / #![forbid(unsafe_code)]), NFR-024 (safe Rust only), NFR-025 (no new production dependencies) |
| Architecture Module | slideforge-diagrams crate — `usvg_normalize` function in `crates/slideforge-diagrams/src/normalize.rs` |
| Sibling Implementation | slideforge-pdf / `crates/slideforge-pdf/src/svg_embed.rs` — identical constants (SEC-001/SEC-002 pattern established in STORY-043) |
| Stories | STORY-079 (STORY-034 dep: established `usvg_normalize` and `RawDiagramSvg`; STORY-079 adds byte-size + nesting-depth guards) |

## Related BCs

- BC-1.12.001 — composes with (normalization is a step in BC-1.12.001's rendering pipeline)
- BC-4.01.001 — depends on (PPTX exporter receives normalized SVG from this BC)

## Architecture Anchors

- `architecture/system-overview.md` — usvg normalization pipeline and mermaid-rs-renderer + usvg integration (Spike S14)
- `architecture/module-decomposition.md` — SS-11 (Diagrams) boundary; DoS guard placement before `font_db()` call

## Story Anchor

STORY-034 (established `usvg_normalize` and `RawDiagramSvg`); STORY-079 (adds SEC-001/SEC-002 guards)

## VP Anchors

(filled after VP creation)

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-05-24 | product-owner | Initial creation — usvg normalization postconditions (foreignObject, script, CSS, absolute dims, use-resolution, E-EXP-004 on failure). |
| 1.1 | 2026-05-24 | product-owner | (status: draft — postconditions pending STORY-079 security invariant review). |
| 1.2 | 2026-06-08 | product-owner | STORY-079 PO review complete. Added postconditions 8–10 (SEC-002 byte-size cap, SEC-001 nesting-depth cap, regression invariant); invariants 4–7 (guard ordering, constant parity, iterative traversal, tracing); EC-005 through EC-010 (boundary conditions); SEC-001/SEC-002 canonical test vectors with construction notes; five new VPs. Traceability expanded with SEC-001/SEC-002, NFR-016/021–025, sibling slideforge-pdf reference, STORY-079. Status promoted from draft to active. |
