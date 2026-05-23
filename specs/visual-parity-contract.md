---
title: Visual Parity Contract
status: BINDING
declared: 2026-05-23
---

# Visual Parity Contract

This document defines the exact contract for "visual parity" between slideforge output and the Python reference implementation, AND between slideforge synthesized output and ground-truth renderers.

## Reference Implementations

| Surface | Reference | Notes |
|---------|-----------|-------|
| Python parity (legacy → slideforge migration) | `.factory/seed/reference/build-incident-brief.py` rendered output | The Python tool is the ground truth for the existing 23 slide types as actually rendered by Office. |
| Multi-renderer parity (new synthesis output) | **Microsoft PowerPoint on Windows 11 (latest stable)** as the canonical renderer | PowerPoint defines "correct"; Keynote/Google Slides/LibreOffice are validated against PowerPoint, not the other way. |

## Tolerance Spec (BINDING)

The following tolerances govern ALL snapshot-test pass/fail decisions in CI:

| Property | Tolerance | Test Method |
|----------|-----------|-------------|
| Slide count | EXACT | `python-pptx`/`ooxmlsdk` parse + count |
| Slide type sequence | EXACT (same type, same order) | structural comparison |
| Slide title text | EXACT byte match | UTF-8 string equality |
| Bullet/body text | EXACT byte match | UTF-8 string equality |
| Brand colors (theme + accents) | EXACT RGB hex match | theme XML diff |
| Font family selection | EXACT match (Aptos = Aptos, not Calibri-substituted) | text run XML diff |
| Font sizes | EXACT match in points | text run XML diff |
| Positional layout (shape x/y/width/height) | ±4pt (≈ ±0.04 inch) | shape XML attribute diff |
| Image dimensions | ±2px | image XML attribute diff |
| Speaker notes content | EXACT byte match | notes XML diff |
| Page numbers, footer chrome | EXACT presence (text content may have date-stamp variance) | master layout XML diff |

## Cross-Renderer Divergence (DOCUMENTED, NOT GATED)

We do NOT block CI on divergence between PowerPoint and Keynote/Google Slides/LibreOffice. We DO maintain a divergence log at `.factory/visual-parity/divergence-log.md` documenting known acceptable divergences (e.g., LibreOffice falls back to Carlito font when Aptos is unavailable; this is recorded and not a failure).

## Test Surfaces

1. **XML-snapshot tests** — per-slide-type. The PPTX XML is canonicalized and committed as a fixture. Diff against fixture in CI = pass/fail.
2. **Pixel-snapshot tests** — full deck rendered via headless LibreOffice → PNG per slide → compared against committed PNG fixtures with `image-diff` tolerance band of `psnr ≥ 35dB` per slide (architect may tighten).
3. **Manual visual review** — at each phase gate, the human reviewer opens the latest test deck in Microsoft PowerPoint and confirms brand fidelity by eye. NOT replaced by automated tests; in addition to them.

## Acceptance Bar Override

If a downstream agent (architect, story-writer, implementer) believes a tolerance row is wrong, they MUST open an ADR proposing the change — they MAY NOT silently widen the bar in code.
