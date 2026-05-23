---
title: Slideforge Success Metrics
source: "extracted from PROJECT-SEED.md 2026-05-23"
parent: product-brief.md
version: 1.0
created: 2026-05-23
status: SEED-EXTRACT
---

# Slideforge Success Metrics

> Source: §10 (Success Metrics) from PROJECT-SEED.md. Verbatim extraction.
> NOTE: The visual parity criterion (#1) has been superseded by the binding definition
> in `visual-parity-contract.md`. The "indistinguishable" language is replaced by the
> precise tolerance spec in that document.

---

## Section 10: Success Metrics

The project is "done with v1.0" when:

1. **Visual parity**: The reference deck (`mss_metrics_leadership.py` ported to `.sf`) produces a `.pptx` indistinguishable from the Python tool's output to a reviewer who didn't know it was rebuilt.
   > NOTE: "indistinguishable" is defined precisely in `visual-parity-contract.md`. That document supersedes this wording.

2. **Performance**: A 25-slide deck builds in < 500ms cold, < 50ms incremental.

3. **Distribution**: A user can `brew install slideforge` and run a build without installing Rust, Python, or anything else.

4. **Documentation**: A developer with no prior context can author a working `.sf` file using only `docs/dsl-reference.md`.
   > Suggested validation method: a usability test with one internal developer who has not seen the DSL; success = produces a syntactically valid 3-slide `.sf` file within 30 minutes.

5. **Editor support**: At least one editor (VS Code) has syntax highlighting for `.sf` files. Bonus: a basic LSP for autocomplete.
   > NOTE: "Bonus" is not a decision. This criterion must be promoted to a v1.0 gate with a measurable threshold (e.g., "autocomplete for slide type keywords in VS Code extension") or formally marked v1.x deferred. Product-owner must resolve this in the PRD.

## Quality Bar

The v1.0 release is also gated on ALL rows in the Quality Bar table in `STATE.md`. The 5 criteria above are in addition to, not instead of, the Quality Bar.
