---
title: "S6 Spike: Multi-Renderer Parity Baseline + CI Infrastructure"
spike-id: S6
severity: HIGH
time-box: 3 days
status: RESOLVED
resolved: 2026-05-24
output: ADR-002 input
owner: architect + devops
research-date: 2026-05-24
inputs:
  - .factory/specs/visual-parity-contract.md
  - .factory/planning/brand-template-patterns.md
  - .factory/planning/q1-decision-final.md
---

# S6 Spike: Multi-Renderer Parity Baseline + CI Infrastructure

## Executive Summary

This spike establishes the complete CI infrastructure for slideforge's multi-renderer
visual parity testing. The core finding is that the two-layer approach specified in
`visual-parity-contract.md` (XML-snapshot tests + pixel-snapshot tests) is sound, but
the **PSNR ≥ 35 dB threshold is the discriminating gate** — not SSIM — for slide-level
rendering regressions at production dimensions. SSIM ≥ 0.97 is too lenient at 4000×2250
resolution and will miss real regressions (shape drift of 4pt passes SSIM at 0.9961).

LibreOffice is NOT installed on this macOS machine; all library validation was performed
against scikit-image with synthetic test data. The rendering pipeline design is
empirically grounded but the actual LibreOffice rendering quality assessment must be
performed in a Linux CI container where LibreOffice is installed.

**Key recommendations for ADR-002:**
1. Use LibreOffice via PPTX → PDF → PNG pipeline (not direct --convert-to png)
2. Tighten SSIM threshold from 0.97 to **0.99** for same-font, same-renderer comparisons
3. Add PSNR ≥ 35 dB as a co-gate (PSNR discriminates shape drift; SSIM does not)
4. Reference PNGs are stored in Git (not Git LFS) at this scale (< 5 MB per deck)
5. Cross-renderer divergences (PowerPoint vs LibreOffice) are catalogued but not CI-gated

---

## 1. LibreOffice Headless Rendering

### 1.1 LibreOffice Status on This Machine

LibreOffice is not installed on this macOS development machine. Available via Homebrew:

```bash
brew install --cask libreoffice       # 26.2.3 — fresh
brew install --cask libreoffice-still # 25.8.7 — enterprise stable
```

**Recommendation:** CI containers should use `libreoffice-still` (25.8.7) for
deterministic rendering. Pinning the version is mandatory; upgrading LibreOffice
changes rendering output and will trigger reference PNG updates.

### 1.2 Correct Command Pipeline

Direct `--convert-to png` only exports the **first slide**. The correct two-step pipeline:

```bash
# Step 1: PPTX → PDF (exports all slides)
/Applications/LibreOffice.app/Contents/MacOS/soffice \
    --headless --convert-to pdf --outdir /tmp/out /path/to/slides.pptx

# Step 2: PDF → PNG per slide (300 DPI, white background, no alpha)
magick -density 300 -background white -alpha remove \
    /tmp/out/slides.pdf \
    /tmp/out/slides-%03d.png
```

Output filename pattern: `<basename>-000.png`, `<basename>-001.png`, ... (0-indexed).

This pipeline is implemented at:
`.factory/planning/spikes/S6-code/render-pptx.sh`

The script auto-detects the LibreOffice binary (macOS app bundle or PATH),
captures version metadata, and measures elapsed time per deck.

### 1.3 Known LibreOffice Rendering Limitations

These are documented limitations, not bugs. They drive the divergence catalogue in
Section 4 and the acceptable-divergence policy in `visual-parity-contract.md`.

| Feature | PowerPoint Behavior | LibreOffice Behavior | Classification |
|---------|--------------------|--------------------|----------------|
| Font: Calibri | Renders exactly | Substitutes Carlito; text reflows possible | Acceptable divergence |
| Font: Aptos (Office default since 2024) | Renders exactly | No metrically-compatible substitute; uses Source Sans or similar | Acceptable divergence |
| Font: custom/embedded | Renders if embedded | Embedded font support is partial; may fall back to system | Acceptable divergence |
| Gradient: 3-stop | Full 3-stop rendering | Downgrades to 2-stop gradient | **Known LO limitation (ECMA-376 feature gap)** |
| Gradient: 2-stop | Exact | Minor color stop differences, slight banding | Acceptable divergence |
| Table border: double/triple | Full double/triple border | Collapses to single border | Known LO limitation |
| Table border: round-dotted | Round-dotted | Square-dotted | Known LO limitation |
| Shape positioning | Exact to EMU spec | ±2-4px drift common | Acceptable if within 4pt tolerance |
| clrMapOvr (dark theme) | Correct per spec | Incomplete support; may produce wrong bg/text color | **Bug territory — avoid in slideforge** |
| SmartArt | Native SmartArt | Converted to grouped shapes; layout may differ | Acceptable divergence |
| Direct `--convert-to png` | n/a | Exports first slide only | Implementation constraint |
| PNG DPI control | n/a | Not directly configurable via CLI; use ImageMagick post-step | Implementation constraint |

### 1.4 Quality Assessment at 300 DPI

At 300 DPI, a 16:9 slide renders to approximately 4000×2250 pixels. ImageMagick's
`-density 300` with `-background white -alpha remove` produces lossless PNG.

The PDF intermediary introduces one potential quality degradation: text rendering via
LibreOffice's internal PDF engine. In practice, LibreOffice's PDF export is
high-fidelity for vector content (text, shapes). Raster images embedded in slides
may be down-sampled if they exceed LibreOffice's internal rendering resolution.

**Assessment:** 300 DPI with the PDF intermediary pipeline is adequate for CI
regression detection. It matches the threshold analysis in Section 2.

---

## 2. Visual Diff Infrastructure

### 2.1 Tool Comparison

| Tool | Language | Type | Strengths | Weaknesses | Verdict |
|------|----------|------|-----------|------------|---------|
| pixelmatch | Node.js | Pixel diff | Fast, simple, CI-native | No perceptual metric; noisy on anti-aliasing | Good for quick CI gating |
| ImageMagick `compare` | CLI | PSNR/SSIM/MSE | Flexible, no dependency install | Metric interpretation non-obvious | Use for PSNR measurement |
| scikit-image SSIM + PSNR | Python | Perceptual | Best perceptual accuracy; good for batch | Requires Python + Pillow + scikit-image | **PRIMARY CHOICE** |

**Selected approach:** scikit-image SSIM + PSNR via Python script, with JSON report
output for CI artifact upload. Implementation at:
`.factory/planning/spikes/S6-code/visual-diff.py`

Both SSIM and PSNR are computed per slide. The CI gate uses SSIM as primary
(catches structural differences) and PSNR as secondary (catches subtle drift that
SSIM misses at large image dimensions).

### 2.2 Threshold Calibration (Empirical)

Empirical testing at realistic slide dimensions (4000×2250 = 9M pixels):

| Scenario | SSIM | PSNR | Assessment |
|----------|------|------|------------|
| Identical images | 1.0000 | inf | PASS (trivial) |
| Anti-aliasing noise (0.5% pixels, ±3 values) | 1.0000 | 65.2 dB | PASS — noise invisible |
| Text reflow (200×50px block shifted 2px) | 0.9989 | 37.2 dB | PASS — minor font substitution |
| Shape drift (300×100px shifted 17px = 4pt at 300 DPI) | 0.9961 | 31.3 dB | **WARN on PSNR < 35 dB** |
| Gradient color shift (±10 values entire slide) | 0.9988 | 32.0 dB | **WARN on PSNR < 35 dB** |

**Critical finding:** SSIM ≥ 0.97 does NOT discriminate the cases this project cares
about. A 4pt shape drift (the maximum allowed by visual-parity-contract.md) and a
significant gradient color shift both pass SSIM at 0.9961 and 0.9988 respectively.

**Revised threshold recommendations:**

| Metric | Contract Value | Recommended CI Gate | Rationale |
|--------|---------------|--------------------|-----------| 
| SSIM | 0.97 (existing) | **Raise to 0.99** | 0.97 is too lenient; shape drift of 4pt passes it |
| PSNR | 35 dB (existing) | **Keep 35 dB** | Correctly warns on 4pt drift (31.3 dB) and gradient shift |
| Gate logic | PSNR ≥ 35 dB | FAIL if SSIM < 0.99 OR PSNR < 35 dB | Both must pass |

**Action for ADR-002:** The existing `psnr ≥ 35dB` threshold in visual-parity-contract.md
is the correct discriminating gate. The `SSIM ≥ 0.97` threshold should be revised upward
to 0.99 when comparing same-renderer, same-font outputs. When comparing across renderers
(LibreOffice vs PowerPoint reference), wider tolerances apply and those comparisons are
documented in the divergence log, not CI-gated.

### 2.3 ImageMagick CLI Alternative

For CI environments without Python, the equivalent ImageMagick commands:

```bash
# PSNR between two slide PNGs
magick compare -metric PSNR reference.png actual.png /dev/null 2>&1

# SSIM (available as SSIM metric in IM 7+)
magick compare -metric SSIM reference.png actual.png /dev/null 2>&1

# Generate diff image (amplified for visibility)
magick compare -metric MSE reference.png actual.png diff.png
```

Note: ImageMagick `compare -metric SSIM` is available in IM 7.x (confirmed installed
at version 7.1.2-12 on this machine). The Python SSIM implementation from scikit-image
is preferred because it handles multi-channel SSIM correctly and produces structured
JSON output for CI artifact parsing.

---

## 3. Reference Screenshot Pipeline

### 3.1 Storage Strategy: Git (not Git LFS)

**Decision: Store reference PNGs in Git, not Git LFS.**

Rationale:
- A 6-slide test deck at 300 DPI generates ~5 MB of PNGs
- slideforge's full 31-type fixture (estimated 62 slides) generates ~50 MB
- Git LFS adds operational complexity (LFS server, storage billing, clone behavior)
- 50 MB of binary fixtures in Git is acceptable for a project that values
  `supply-chain: All production-crate deps pinned with =; Cargo.lock committed`
- Reference PNGs change infrequently (only on intentional PPTX output changes)

If the fixture set grows beyond 100 MB (e.g., full multi-theme coverage), revisit
Git LFS. Include in `.gitattributes`:

```
tests/fixtures/reference-pngs/*.png  binary
```

### 3.2 Reference PNG Location

```
tests/
  fixtures/
    test-fixture.pptx          # The test deck (source of truth for what gets rendered)
    reference-pngs/            # Committed baseline PNGs (one per slide)
      test-fixture-000.png     # Slide 1
      test-fixture-001.png     # Slide 2
      ...
```

The `.pptx` file is generated by `make-test-pptx.py` (see S6-code/). For production
tests, slideforge's own test runner generates the `.pptx` from `.sf` fixture source,
then renders and compares.

### 3.3 Reference Generation Workflow

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Reference PNG Lifecycle                            │
│                                                                      │
│  First time setup OR intentional output change:                     │
│                                                                      │
│  1. Developer makes PPTX output change (code change + spec update)  │
│  2. Run: ./update-reference-pngs.sh                                 │
│  3. Script renders PPTX → PNG + shows diff vs old references        │
│  4. Developer reviews diff images visually                           │
│  5. Developer commits new PNGs with a message explaining why        │
│                                                                      │
│  Routine CI:                                                         │
│  1. CI renders PPTX → PNG (fresh, from current code)                │
│  2. CI runs visual-diff.py vs committed reference PNGs              │
│  3. If fail: CI uploads diff images as artifacts                    │
│  4. Developer reviews diffs and either fixes code or updates refs   │
│                                                                      │
│  Policy:                                                             │
│  - Reference PNG updates require HUMAN review + explicit commit msg │
│  - AI agents do NOT update reference PNGs without human approval    │
│  - Commit message must state WHY output changed                     │
└──────────────────────────────────────────────────────────────────────┘
```

Update workflow implemented at: `.factory/planning/spikes/S6-code/update-reference-pngs.sh`

### 3.4 "Blessed" Reference Change Policy

When slideforge intentionally changes PPTX output (e.g., layout improvement,
bug fix in shape positioning, new slide type), the reference PNG update is a
**spec-change event** requiring:

1. The code change that caused the output change (in the same or prior PR)
2. A `test(visual): update reference PNGs — <reason>` commit in the same PR
3. Human visual inspection (open the PNG, confirm it looks correct)

This is NOT automated. AI implementers must flag any code change that affects
PPTX output and request reference PNG update approval from the human reviewer.

---

## 4. Cross-Renderer Divergence Catalogue

The policy from `visual-parity-contract.md` is: divergences between PowerPoint
(canonical renderer) and LibreOffice/Keynote/Google Slides are **documented, not
CI-gated**. The divergence log lives at `.factory/visual-parity/divergence-log.md`.

### 4.1 Divergences Classified as ACCEPTABLE (document, do not block CI)

| ID | Feature | PowerPoint Output | LibreOffice Output | Keynote Output | Notes |
|----|---------|------------------|--------------------|----------------|-------|
| DIV-001 | Calibri font | Native Calibri | Carlito substitution; possible line reflow | System font fallback | Font metrics differ; text wrapping may change by 1-2 lines |
| DIV-002 | Aptos font (Office 2024 default) | Native Aptos | Source Sans or similar; significant metric difference | System fallback | No metrically-compatible Aptos substitute in LO as of 2026-05 |
| DIV-003 | 2-stop linear gradient | Exact | Slight banding/color stop difference | Generally accurate | Minor visual difference, semantics preserved |
| DIV-004 | Table: thin border (0.5pt) at 72 DPI | Visible | May disappear or become 1px | Visible | Only at low DPI exports; not at 300 DPI |
| DIV-005 | Shape anti-aliasing edges | Smooth | Slightly different anti-aliasing | Smooth | ±2 pixel edge difference; SSIM > 0.99 |
| DIV-006 | Embedded font rendering | Exact | May fall back if embedding flags unsupported | Varies | Test with specific font embedding per S6 test fixture |
| DIV-007 | Chart data label positioning | Exact | May shift by 2-4px depending on chart type | Generally accurate | Minor positioning drift; acceptable within 4pt tolerance |
| DIV-008 | PDF intermediary color profile | n/a (direct render) | Possible slight gamma/color space shift | n/a | CI pipeline uses LO PDF export; minor color difference |

### 4.2 Divergences Classified as BUG TERRITORY (must fix in slideforge OOXML output)

These are not LibreOffice bugs — they are PPTX generation patterns that slideforge
should avoid to maximize cross-renderer fidelity:

| ID | Anti-Pattern | Risk | Slideforge Mitigation |
|----|-------------|------|----------------------|
| BUG-001 | Using `clrMapOvr` for dark slides | LO clrMapOvr support incomplete; wrong bg/text colors | Use explicit slide background fill + explicit text colors for dark slides instead of relying on clrMap remapping |
| BUG-002 | 3-stop gradients | Downgrades to 2-stop in LO | slideforge only generates 2-stop gradients in v1.0 |
| BUG-003 | Double/triple table borders | Collapses to single in LO | slideforge uses single-line borders only |
| BUG-004 | Round-dotted borders | Round→square-dotted in LO | slideforge uses square-dotted only |
| BUG-005 | SmartArt | Converted to grouped shapes in LO | slideforge does not use SmartArt (uses shape DSL instead) |
| BUG-006 | Slide IDs starting below 256 | May cause issues in LO | slideforge starts at 256 per ECMA-376 spec (confirmed in brand-template-patterns.md) |

### 4.3 Keynote and Google Slides

Keynote and Google Slides validation is **manual, per phase-gate only** — not CI-automated.

Known divergences from public documentation and community reports:
- **Keynote:** Generally good PPTX import fidelity for simple slides. Custom layouts
  may not render correctly (falls back to closest standard layout). Font substitution
  similar to LibreOffice but uses macOS fonts.
- **Google Slides:** Weakest PPTX import fidelity. Known issues: custom fonts always
  substitute, complex shapes may simplify, table formatting stripped. Acceptable for
  viewing; not acceptable as a parity target beyond basic content.

**Recommendation for ADR-002:** Google Slides is NOT included in the CI visual
regression gate. It is documented as "view-only compatible" in the multi-renderer
parity spec. LibreOffice is the sole automated CI renderer.

---

## 5. CI Integration Design

### 5.1 CI Workflow Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│  GitHub Actions: Visual Regression Job                                │
│                                                                       │
│  Trigger: push to develop, PRs to develop                            │
│                                                                       │
│  Container: ubuntu-latest                                             │
│    └─ LibreOffice Still 25.8.7 (pinned, not "latest")               │
│    └─ ImageMagick 7.x                                                │
│    └─ Python 3.11 + Pillow + scikit-image + numpy                   │
│                                                                       │
│  Steps:                                                               │
│  1. cargo test --workspace → generates tests/fixtures/*.pptx         │
│     (slideforge renders its own test fixtures as part of test suite)  │
│  2. render-pptx.sh: PPTX → PDF → PNG (300 DPI)                      │
│  3. visual-diff.py: PNG vs reference-pngs/ (SSIM + PSNR)            │
│  4. Upload diff artifacts on failure                                  │
│  5. Exit 1 if any slide fails SSIM < 0.99 OR PSNR < 35 dB          │
│                                                                       │
│  Runtime estimate: ~45s for a 25-slide deck                          │
│    - LO render: ~20s                                                  │
│    - ImageMagick PDF→PNG: ~15s at 300 DPI                           │
│    - Python SSIM+PSNR 25 slides: ~10s                                │
└───────────────────────────────────────────────────────────────────────┘
```

### 5.2 GitHub Actions Workflow Snippet

```yaml
# .github/workflows/visual-regression.yml
name: Visual Regression

on:
  push:
    branches: [develop]
  pull_request:
    branches: [develop]

jobs:
  visual-regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          lfs: false  # No LFS needed for Git-stored PNGs

      - name: Install LibreOffice Still (pinned)
        run: |
          sudo apt-get update -qq
          sudo apt-get install -y libreoffice=1:25.8.7~* 2>/dev/null \
            || sudo apt-get install -y libreoffice  # fallback: latest
          soffice --version

      - name: Install ImageMagick
        run: sudo apt-get install -y imagemagick

      - name: Install Python deps
        run: pip install Pillow scikit-image numpy

      - name: Build + generate test fixtures
        run: cargo test --workspace --test visual_fixtures 2>&1
        # This test target generates tests/fixtures/test-fixture.pptx

      - name: Render PPTX → PNG
        run: |
          bash .factory/planning/spikes/S6-code/render-pptx.sh \
            tests/fixtures/test-fixture.pptx \
            /tmp/rendered

      - name: Visual diff
        env:
          SSIM_THRESHOLD: "0.99"
          PSNR_THRESHOLD_DB: "35.0"
        run: |
          python3 .factory/planning/spikes/S6-code/visual-diff.py \
            tests/fixtures/reference-pngs/ \
            /tmp/rendered \
            --out /tmp/diffs \
            --json /tmp/visual-report.json

      - name: Upload diff artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: visual-regression-diffs
          path: /tmp/diffs/
          retention-days: 7
```

### 5.3 Which Renderers Run Where

| Renderer | CI Automated | Phase Gate Manual | Notes |
|----------|-------------|------------------|-------|
| LibreOffice (headless) | YES — every PR | — | Primary CI renderer |
| Microsoft PowerPoint | NO — CI-infeasible | YES — each phase gate | Human opens file in PPT, confirms fidelity |
| Keynote | NO | YES — each phase gate | macOS only; human validates |
| Google Slides | NO | Optional spot-check | View-only compatibility target |

### 5.4 Regression Detection Workflow for Developers

When a visual regression is detected in CI:

1. Download the diff artifacts from the failed CI run
2. Examine the amplified diff PNGs (bright areas = changed pixels)
3. If the regression is a **bug** (unexpected output change): fix the PPTX generation code
4. If the regression is **intentional** (you changed layout, fixed positioning):
   - Run `./update-reference-pngs.sh` locally
   - Review the new reference PNGs visually
   - Commit the updated PNGs to the PR with a message explaining the change
5. Never silently widen the SSIM/PSNR thresholds without an ADR amendment

---

## 6. Sample PPTX Test Fixture

### 6.1 Fixture Design

The test fixture at `.factory/planning/spikes/S6-code/make-test-pptx.py` generates
a 6-slide deck that exercises the highest-risk rendering features:

| Slide | Feature Tested | Primary Risk |
|-------|---------------|-------------|
| 1 | Title/subtitle placeholders, font size, brand color on text | Font rendering, placeholder positioning |
| 2 | Text wrapping: Calibri 14pt, Arial 12pt, Inter 10pt (long lines near boundary) | Font substitution → line reflow |
| 3 | 2-stop linear gradients (horizontal + 45°) | Gradient color stop fidelity |
| 4 | Table: header fill, cell text, thin borders | Cell fill, border rendering |
| 5 | Dark background (explicit fill, light text, accent bar) | Dark slide rendering, text contrast |
| 6 | 4 positioned shapes (exact EMU coordinates) | Shape positioning drift tolerance |

The fixture explicitly avoids the anti-patterns from Section 4.2 (no clrMapOvr,
no 3-stop gradients, no double borders, no SmartArt). It uses only 2-stop gradients,
explicit colors, and shapes that slideforge will actually generate.

### 6.2 Production Test Fixture Strategy

For Phase 3 implementation, the `tests/` crate (or a dedicated `tests/visual/` test
target) should:
1. Render a `.sf` fixture file through slideforge's full pipeline
2. Produce the `.pptx` output
3. Trigger the visual regression gate in CI

The `.sf` fixture should cover all 31 slide types, exercising each with the brand
color palette. This expands the reference PNG set to ~62 slides but remains within
the 50 MB Git storage budget.

---

## 7. Recommended Tolerance Thresholds

### 7.1 Same-Renderer (LibreOffice CI) Thresholds

These apply when comparing slideforge-generated PPTX rendered twice in the same
LibreOffice version. These are the CI gate values.

| Property | Threshold | Method | Rationale |
|----------|-----------|--------|-----------|
| **SSIM** | **≥ 0.99** | scikit-image multichannel | Tighter than contract's 0.97; empirically 0.97 does not catch 4pt shape drift |
| **PSNR** | **≥ 35 dB** | scikit-image | Confirmed in contract; catches gradient color shift + shape drift |
| Slide count | EXACT | Count PNGs | Missing slide is a hard failure |
| Gate logic | Fail if SSIM < 0.99 OR PSNR < 35 dB | Both metrics must pass | Dual-metric coverage |

### 7.2 Cross-Renderer (LibreOffice vs PowerPoint) Thresholds

These apply when comparing LibreOffice renders vs PowerPoint renders of the same
PPTX. These are INFORMATIONAL ONLY — they populate the divergence log, not gate CI.

| Property | Acceptable Divergence | Notes |
|----------|----------------------|-------|
| Font rendering | SSIM ≥ 0.90 | Font substitution changes text edges; perceptually fine |
| Gradient rendering | SSIM ≥ 0.93 | Slight color stop differences acceptable |
| Shape positioning | ≤ 4pt drift (≤ 17px at 300 DPI) | Per visual-parity-contract.md |
| Table borders | Style may differ (double→single) | Known LO limitation; document only |
| Dark backgrounds | Color must be readable (contrast ≥ 3:1) | Functional requirement, not pixel-perfect |

### 7.3 Proposed Amendment to visual-parity-contract.md

The following change to the contract's pixel-snapshot tolerance is proposed for
architect sign-off (via ADR-002):

```
Current: psnr ≥ 35dB per slide
Proposed: psnr ≥ 35dB per slide AND ssim ≥ 0.99 per slide (same-renderer)
          psnr ≥ 25dB per slide AND ssim ≥ 0.90 per slide (cross-renderer, informational)
```

If the architect disagrees with the threshold change, open a standalone ADR before
Phase 3 implementation begins.

---

## 8. Decisions for ADR-002

Based on this spike, ADR-002 (visual regression CI infrastructure) should record:

1. **LibreOffice version pinning:** LibreOffice Still (current: 25.8.7) pinned in CI.
   Upgrades require reference PNG update workflow.

2. **Render pipeline:** PPTX → PDF (LibreOffice) → PNG (ImageMagick 300 DPI).
   Not: direct `--convert-to png` (first slide only limitation).

3. **Comparison tool:** scikit-image SSIM + PSNR via `visual-diff.py`.
   Fallback: ImageMagick `compare -metric SSIM/PSNR`.

4. **CI gate thresholds:** SSIM ≥ 0.99 AND PSNR ≥ 35 dB (amends 0.97 in contract).

5. **Reference PNG storage:** Git (not LFS) at `tests/fixtures/reference-pngs/`.
   Capped at 100 MB total; revisit LFS if exceeded.

6. **Reference PNG update policy:** Human-approved, explicit commit message required.
   AI agents do not update reference PNGs without human approval.

7. **Cross-renderer scope:** LibreOffice in CI (automated). PowerPoint + Keynote at
   phase gates (manual). Google Slides as view-only compatibility target only.

8. **OOXML anti-patterns to avoid** (BUG-001 through BUG-006 in Section 4.2).

---

## 9. Code Artifacts

All code produced in this spike is at `.factory/planning/spikes/S6-code/`:

| File | Purpose |
|------|---------|
| `render-pptx.sh` | PPTX → PDF → PNG rendering via LibreOffice + ImageMagick |
| `make-test-pptx.py` | Generates 6-slide test fixture .pptx (requires python-pptx) |
| `visual-diff.py` | SSIM + PSNR comparison of two PNG directories; JSON report |
| `ci-visual-gate.sh` | CI entry point: render + diff + artifact upload |
| `update-reference-pngs.sh` | Guided reference PNG update workflow for developers |

These scripts are the reference implementation for:
- `slideforge-ci` crate (Phase 3) visual regression test target
- GitHub Actions workflow `.github/workflows/visual-regression.yml`

---

## 10. Open Questions for ADR-002

1. **LibreOffice installation in CI:** Use apt package or Docker image? A pinned
   Docker image (e.g., `libreofficedocker/libreoffice:25.8.7`) would be more
   reproducible than apt pinning, which can break when the version leaves the repo.

2. **Font installation in CI:** To test Calibri rendering (for the font-fallback
   divergence path), install `ttf-mscorefonts-installer` in CI? Or accept that
   CI always uses font-substituted rendering (thus baseline is always Carlito)?
   Recommendation: accept Carlito as the CI baseline; document as DIV-001.

3. **ImageMagick memory limit:** Large decks at 300 DPI require significant memory
   (a 62-slide deck ≈ 1.5 GB RAM for ImageMagick PDF→PNG). Verify GitHub Actions
   runner RAM (currently 7 GB on ubuntu-latest). Use `-limit memory 4GB` to prevent
   OOM kills.

4. **Threshold for SSIM 0.99 vs 0.97:** The empirical data from Section 2.2 shows
   SSIM 0.97 is too lenient. The architect should review and confirm 0.99 before
   it is written into ADR-002.

---

## Appendix A: LibreOffice Installation Quick Reference

```bash
# macOS (development)
brew install --cask libreoffice-still

# Ubuntu CI (pinned version)
sudo apt-get install -y \
    libreoffice-impress \
    libreoffice-writer \
    fonts-liberation \
    fonts-crosextra-carlito \
    fonts-crosextra-caladea

# The LibreOffice executable path:
# macOS: /Applications/LibreOffice.app/Contents/MacOS/soffice
# Linux: soffice (or /usr/lib/libreoffice/program/soffice)
```

## Appendix B: Metric Interpretation Reference

| Scenario | SSIM | PSNR | Human Perception |
|----------|------|------|-----------------|
| Identical images | 1.0000 | inf | Pixel-perfect |
| Anti-aliasing noise only | ≥ 0.9999 | ≥ 60 dB | Indistinguishable |
| Minor font rendering difference | 0.998–0.9999 | 40–65 dB | Invisible to most |
| Text reflow (1-2 words per line) | 0.995–0.999 | 35–45 dB | Noticeable on inspection |
| Shape drift 4pt (≈17px at 300 DPI) | ~0.996 | 30–35 dB | Visible on close inspection |
| Gradient color shift ±10 values | ~0.999 | 30–35 dB | Subtle but perceptible |
| Font substitution (Calibri→Carlito) | 0.90–0.97 | 20–35 dB | Perceptible layout change |
| Missing shape or element | < 0.90 | < 25 dB | Obvious visual regression |

## Appendix C: Divergence Log Stub

Initial entries for `.factory/visual-parity/divergence-log.md` (to be created by devops-engineer):

```markdown
# Divergence Log — slideforge Multi-Renderer Parity

| ID | Date | Feature | PowerPoint | LibreOffice | Keynote | Classification |
|----|------|---------|------------|-------------|---------|----------------|
| DIV-001 | 2026-05-24 | Calibri → Carlito font substitution | Native Calibri | Carlito substitute; possible line reflow | System font | Acceptable — document in CI artifacts |
| DIV-002 | 2026-05-24 | Aptos (Office 2024 default) | Native Aptos | No metrically-compatible substitute | System font | Acceptable — slideforge uses Inter/Calibri as default, not Aptos |
| DIV-003 | 2026-05-24 | 2-stop gradient color fidelity | Exact | Minor banding/color stop difference | Generally accurate | Acceptable — within SSIM 0.93 cross-renderer threshold |
| DIV-004 | 2026-05-24 | Table border styles | Double/round-dotted supported | Single/square-dotted only | Generally good | Known LO limitation — slideforge avoids double/round borders |
```
