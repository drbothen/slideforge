---
document_type: adr
adr_id: ADR-002
title: Visual parity testing with LibreOffice headless
status: accepted
date: 2026-05-24
spike_input: S6-multi-renderer-parity.md
traces_to: ARCH-INDEX.md
supersedes: ~
---

# ADR-002: Visual Parity Testing with LibreOffice Headless

## Context

slideforge must enforce multi-renderer visual parity (DI-013, NFR-007/008). CI needs an
automated way to detect regressions in PPTX rendering output. Spike S6 established that
SSIM ≥ 0.97 is too lenient — shape drift of 4pt passes SSIM at 0.9961 but fails PSNR at
31.3 dB. Both metrics are required.

## Decision

Visual regression CI uses LibreOffice Still (pinned version) as the automated renderer,
with a PPTX → PDF → PNG pipeline. Gate: SSIM ≥ 0.99 AND PSNR ≥ 35 dB per slide.
Reference PNGs stored in Git (not LFS). Human approval required for reference updates.

## Consequences

**Render pipeline:**
1. `cargo test` generates `tests/fixtures/test-fixture.pptx`
2. LibreOffice headless: PPTX → PDF → PNG at 300 DPI via ImageMagick
3. `visual-diff.py`: SSIM + PSNR per slide vs committed reference PNGs (scikit-image)
4. CI fails if any slide: SSIM < 0.99 OR PSNR < 35 dB

**CI gate thresholds (amending visual-parity-contract.md):**
- Same-renderer (LibreOffice CI): SSIM ≥ 0.99 AND PSNR ≥ 35 dB
- Cross-renderer (informational only): SSIM ≥ 0.90 per slide

**Renderer scope:**
- LibreOffice Still 25.8.7 (pinned): automated CI, every PR
- Microsoft PowerPoint, Keynote: manual phase gate validation
- Google Slides: view-only compatibility, spot-check only

**OOXML anti-patterns avoided (from S6 §4.2):**
- BUG-001: No `clrMapOvr` alone for dark slides (use + explicit white runs per brand-architecture.md)
- BUG-002: No 3-stop gradients (use 2-stop only in v1.0)
- BUG-003/004: No double/round-dotted table borders

**Reference PNG update policy:**
- Human review + explicit commit message required. AI agents do not update without approval.
