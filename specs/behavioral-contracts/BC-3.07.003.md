---
document_type: behavioral-contract
level: L3
version: "1.0"
status: active
producer: product-owner
timestamp: 2026-06-11T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-18
capability: CAP-030
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.07.003: CLI Input Path Normalized Before Brand and Config Discovery

## Description

When the user invokes `slideforge build <path>` with a bare filename (e.g.,
`slideforge build deck.sf`) or any relative path, `Path::parent()` returns
`Some(Path::new(""))` for bare filenames — an empty string, not a valid directory.
The CLI MUST normalize the input path before using it as the base for brand
resolution, `.sfconfig` discovery, or any file I/O that requires a parent directory.
Diagnostics MUST NEVER cite an empty path — doing so obscures the real failure
from the user.

This contract closes REND-009 (demo-deep-review-2026-06-11): "brand I/O error for
''" — the empty string was cited as the brand discovery base directory.

## Preconditions

1. The user has invoked `slideforge build <path>` where `<path>` is a valid file path
   string (may be bare filename, relative, or absolute).
2. The CLI has received the path as a `String` or `OsString` argument from clap.

## Postconditions

1. **Path normalization occurs before any I/O:** Before the CLI passes the source path
   to brand resolution, `.sfconfig` discovery, or any system call that requires the
   parent directory, the path is normalized via the following rule:

   ```
   normalize_parent(path):
     let parent = path.parent()
     if parent == Some(Path::new("")) || parent == None:
       return Path::new(".")
     else:
       return parent.unwrap()
   ```

   Concrete behavior by example:
   - `"deck.sf"` → parent is `""` → normalized to `"."`
   - `"./deck.sf"` → parent is `"."` → unchanged, returns `"."`
   - `"subdir/deck.sf"` → parent is `"subdir"` → unchanged, returns `"subdir"`
   - `"/abs/path/deck.sf"` → parent is `"/abs/path"` → unchanged, returns `"/abs/path"`

2. **No diagnostic ever cites an empty path:** Any diagnostic emitted by the brand
   resolution module, the `.sfconfig` loader, or the CLI itself MUST use the
   normalized parent path (i.e., `"."` rather than `""`) when constructing error
   messages. An empty string in a diagnostic message is a contract violation.

3. **Brand discovery succeeds for bare-filename invocations:** When the normalized
   parent is `"."`, brand discovery searches the current working directory (same as
   if the user had written `"./deck.sf"`). This matches user expectation: running
   `slideforge build deck.sf` from the directory containing `deck.sf` succeeds
   (assuming all other preconditions are met).

4. **Regression test:** A unit test for `normalize_parent` covers all four cases
   listed in postcondition 1 and is part of the `slideforge-cli` test suite.

## Invariants

1. The normalization logic is a pure function with no side effects: given the same
   input path string, it always returns the same normalized parent path. (DI-009)

2. The normalization is applied ONCE at the CLI entry point (before any crate
   receives the path). No downstream crate (brand, config, layout, export) performs
   its own bare-filename fix — the CLI is the single normalization site. This prevents
   drift between crates.

3. Normalized parent paths are valid directory handles on all supported platforms
   (macOS arm64/x86_64, Linux x86_64/arm64, Windows x86_64). `Path::new(".")` is
   valid on all three.

4. This normalization does NOT change the source file path itself (the path passed
   to the parser). Only the PARENT (used for brand/config discovery) is normalized.
   The parser receives the original path as given by the user for error span
   purposes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `slideforge build deck.sf` (bare filename, CWD contains deck) | Brand searched in `.`; build succeeds; no "I/O error for ''" emitted |
| EC-002 | `slideforge build ./deck.sf` (explicit dot-relative) | Parent is already `"."` after normalization; behavior identical to EC-001 |
| EC-003 | `slideforge build /abs/path/deck.sf` (absolute) | Parent is `/abs/path`; brand searched there |
| EC-004 | `slideforge build subdir/deck.sf` | Parent is `subdir`; brand searched in `subdir` |
| EC-005 | `slideforge build deck.sf` but deck.sf not found | E-PAR-005 ("File not found: './deck.sf'") — note the normalized `./` prefix in the message; no "error for ''" |
| EC-006 | Brand file missing from normalized parent dir | E-BRD-001 citing the normalized path (e.g., `"."` or `"./brand.pptx"`), NOT citing `""` |
| EC-007 | Windows path `deck.sf` | `Path::parent()` returns `Some(Path::new(""))` on Windows too; normalized to `"."` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `normalize_parent("deck.sf")` | `Path::new(".")` | bare-filename |
| `normalize_parent("./deck.sf")` | `Path::new(".")` | dot-relative |
| `normalize_parent("/abs/path/deck.sf")` | `Path::new("/abs/path")` | absolute |
| `normalize_parent("subdir/deck.sf")` | `Path::new("subdir")` | relative-with-dir |
| CLI invocation `build "deck.sf"` from dir containing file and default brand | build succeeds; exit 0; no "I/O error for ''" | integration |
| CLI invocation `build "deck.sf"` brand file missing | E-BRD-001 message cites `"."` not `""`; exit 4 | brand-missing |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | normalize_parent returns "." for bare filename and "./<bare>" inputs | unit test (four cases in postcondition 1) |
| VP-TBD | No E-BRD-001 or similar diagnostic cites empty path "" for bare-filename invocation | integration test: build deck.sf in temp dir; grep stderr for I/O error for ""; assert none |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 |
| Capability Anchor Justification | CAP-030 ("Diagnostic Reporting with Source Spans") per capabilities.md §CAP-030 — "Emit all errors and warnings with file:line:col spans and correction hints rendered via miette/ariadne." An empty path in a diagnostic violates the core requirement that spans cite valid, interpretable locations. This BC ensures path normalization removes the root cause of empty-path diagnostics. |
| L2 Domain Invariants | DI-018 (error accumulation — diagnostic quality invariant; no unintelligible location) |
| REND Source | REND-009 (demo-deep-review-2026-06-11) — "brand I/O error for ''" — Path::parent() of bare filename returns "" not "." |
| Architecture Modules | SS-18 (CLI Orchestrator — slideforge-cli; brand resolution module) |
| Stories | STORY-101 |

## Related BCs

- BC-1.15.001 — composes with (all errors carry valid file spans; this BC ensures the CLI path is valid before it reaches the span machinery)
- BC-4.01.001 — upstream of (PPTX build requires brand resolution to succeed; brand resolution requires a valid parent path)
- BC-3.07.002 — composes with (image path normalization for media embedding uses the same normalized parent as the brand resolution path)

## Architecture Anchors

- `architecture/module-decomposition.md` — SS-18 CLI Orchestrator; brand resolution entry point

## Story Anchor

STORY-101

## VP Anchors

(filled after VP creation)
