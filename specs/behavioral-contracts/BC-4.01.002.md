---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-24T00:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: "[pending]"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: SS-TBD
capability: CAP-015
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

# BC-4.01.002: PPTX Output Passes Multi-Renderer Fidelity (SSIM≥0.99 AND PSNR≥35dB vs Reference)

## Description

PPTX output produced by slideforge must render correctly across all four target
renderers: PowerPoint, LibreOffice, Keynote, and Google Slides. The automated
CI gate uses LibreOffice Still (pinned version) as the reference renderer. Each
rendered slide must achieve SSIM ≥ 0.99 AND PSNR ≥ 35dB compared to the committed
reference PNG. This threshold was empirically validated in Spike S6 — the prior
0.97 SSIM threshold was shown to miss 4pt shape drift.

## Preconditions

1. A .pptx file has been produced by BC-4.01.001.
2. LibreOffice Still 25.8.7 (pinned) is installed in the CI environment.
3. Reference PNGs exist in `tests/fixtures/reference-pngs/` for the test fixture deck.
4. The PPTX uses no OOXML anti-patterns from the S6 bug list (no clrMapOvr, no 3-stop gradients, no double borders, no SmartArt).

## Postconditions

1. For each slide in the test fixture deck:
   - LibreOffice renders the PPTX to PNG at 300 DPI (via PDF intermediary).
   - SSIM of rendered PNG vs reference PNG ≥ 0.99.
   - PSNR of rendered PNG vs reference PNG ≥ 35dB.
2. Slide count in rendered output matches expected slide count exactly.
3. CI visual regression job passes.
4. Any regression (SSIM < 0.99 OR PSNR < 35dB) causes CI to fail and upload diff artifacts.

## Invariants

1. The SSIM threshold is ≥ 0.99 (not 0.97). (S6 empirical finding)
2. Both SSIM AND PSNR must pass — dual-metric gate. (S6 finding: SSIM alone misses gradient shift)
3. Reference PNG updates require human visual review + explicit commit message. AI agents do NOT update reference PNGs without human approval.
4. LibreOffice version is pinned — upgrades require reference PNG updates.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Font Calibri → Carlito substitution in LibreOffice (DIV-001) | Documented acceptable divergence; SSIM ≥ 0.90 cross-renderer (informational, not CI-gated) |
| EC-002 | Gradient rendering slight banding in LibreOffice (DIV-003) | Within SSIM ≥ 0.99 same-renderer tolerance; slideforge uses 2-stop gradients only |
| EC-003 | Shape position drift 2px (DIV-005) | Within ≤ 4pt (17px at 300 DPI) tolerance; SSIM ≥ 0.99 same-renderer |
| EC-004 | New slide type added without reference PNG update | CI fails (slide count mismatch); developer must generate and approve reference PNGs |
| EC-005 | PPTX output change from a code fix | CI fails; developer runs update-reference-pngs.sh, reviews, approves |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 31-type fixture deck rendered by LibreOffice | All slides: SSIM ≥ 0.99, PSNR ≥ 35dB | happy-path |
| Same fixture with known-good brand (no anti-patterns) | SSIM ≈ 1.0000 for deterministic elements (identical rendering) | happy-path |
| PPTX with 3-stop gradient (anti-pattern) | Blocked at generation time — BC-4.01.001 should not produce 3-stop gradients | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | CI visual regression gate: all slides pass SSIM ≥ 0.99 AND PSNR ≥ 35dB | GitHub Actions visual regression job |
| VP-TBD | No OOXML anti-patterns (BUG-001 through BUG-006) in output PPTX | PPTX structural test (parse XML, check for forbidden patterns) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 |
| Capability Anchor Justification | CAP-015 ("PPTX Export") per capabilities.md §CAP-015 — "no broken layouts, missing content, or corrupt files across four renderers" is the parity requirement stated in CAP-015 and DI-013 |
| L2 Domain Invariants | DI-013 (PPTX output must pass multi-renderer fidelity check) |
| Architecture Module | slideforge-pptx crate + CI visual regression job (filled by architect) |
| Stories | (filled by story-writer) |

## Related BCs

- BC-4.01.001 — depends on (this BC verifies the output of BC-4.01.001)
- NFR-007, NFR-008 — implements (SSIM and PSNR thresholds are the NFR targets)

## Architecture Anchors

- `architecture/export-architecture.md#visual-regression` — CI visual parity gate design (Spike S6)
- `architecture/visual-parity-contract.md` — threshold specification

## Story Anchor

(filled by story-writer)

## VP Anchors

(filled after VP creation)
