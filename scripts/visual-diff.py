#!/usr/bin/env python3
"""
visual-diff.py — Slide visual regression checker.

Compares generated PNG slides against committed reference PNGs using SSIM
and PSNR metrics. Enforces NFR-007 (SSIM >= 0.99) and NFR-008 (PSNR >= 35dB).

Usage:
    python3 scripts/visual-diff.py \\
        --generated /tmp/visual-regression/png \\
        --reference tests/fixtures/reference-pngs \\
        --ssim-threshold 0.99 \\
        --psnr-threshold 35.0 \\
        --fail-on-violation

Exit codes:
    0 — all slides pass thresholds
    1 — one or more slides failed a threshold
    2 — invocation error (missing dirs, missing deps, etc.)

Dependencies (installed in CI via pip):
    scikit-image >= 0.21
    numpy >= 1.24
"""

import argparse
import os
import sys
from pathlib import Path


def check_imports() -> None:
    """Fail early with a helpful message if dependencies are not installed."""
    try:
        import numpy  # noqa: F401
        from skimage import io, metrics  # noqa: F401
    except ImportError as exc:
        print(
            f"Missing dependency: {exc}\n"
            "Install with: pip3 install scikit-image numpy",
            file=sys.stderr,
        )
        sys.exit(2)


def compute_psnr(ref_arr, gen_arr) -> float:
    """Compute PSNR between two uint8 numpy arrays (higher is better)."""
    import numpy as np

    mse = float(np.mean((ref_arr.astype(float) - gen_arr.astype(float)) ** 2))
    if mse == 0:
        return float("inf")
    return 10.0 * np.log10((255.0 ** 2) / mse)


def compare_slides(
    generated_dir: Path,
    reference_dir: Path,
    ssim_threshold: float,
    psnr_threshold: float,
    fail_on_violation: bool,
) -> int:
    """
    Compare all PNG files in generated_dir against corresponding files in
    reference_dir.

    Returns 0 if all pass, 1 if any fail (when fail_on_violation is True).
    """
    from skimage import io, metrics

    if not generated_dir.exists():
        print(f"ERROR: generated dir does not exist: {generated_dir}", file=sys.stderr)
        return 2
    if not reference_dir.exists():
        print(f"ERROR: reference dir does not exist: {reference_dir}", file=sys.stderr)
        return 2

    generated_pngs = sorted(generated_dir.glob("*.png"))
    if not generated_pngs:
        print(f"ERROR: no PNG files found in {generated_dir}", file=sys.stderr)
        return 2

    violations = []
    results = []

    for gen_path in generated_pngs:
        ref_path = reference_dir / gen_path.name
        if not ref_path.exists():
            msg = f"MISSING reference PNG for {gen_path.name} — expected at {ref_path}"
            print(f"  WARN: {msg}")
            violations.append(msg)
            continue

        gen_img = io.imread(str(gen_path))
        ref_img = io.imread(str(ref_path))

        if gen_img.shape != ref_img.shape:
            msg = (
                f"SHAPE MISMATCH {gen_path.name}: "
                f"generated {gen_img.shape} vs reference {ref_img.shape}"
            )
            print(f"  FAIL: {msg}")
            violations.append(msg)
            continue

        # Compute SSIM (channel_axis=-1 for RGB/RGBA images)
        channel_axis = -1 if gen_img.ndim == 3 else None
        ssim_score = metrics.structural_similarity(
            ref_img,
            gen_img,
            channel_axis=channel_axis,
            data_range=255,
        )
        psnr_score = compute_psnr(ref_img, gen_img)

        ssim_pass = ssim_score >= ssim_threshold
        psnr_pass = psnr_score >= psnr_threshold or psnr_score == float("inf")
        slide_pass = ssim_pass and psnr_pass

        status = "PASS" if slide_pass else "FAIL"
        ssim_flag = "" if ssim_pass else f" [SSIM BELOW {ssim_threshold}]"
        psnr_flag = "" if psnr_pass else f" [PSNR BELOW {psnr_threshold}dB]"
        line = (
            f"  {status}: {gen_path.name} "
            f"SSIM={ssim_score:.4f}{ssim_flag} "
            f"PSNR={psnr_score:.1f}dB{psnr_flag}"
        )
        print(line)
        results.append(line)

        if not slide_pass:
            violations.append(
                f"{gen_path.name}: SSIM={ssim_score:.4f} (need >={ssim_threshold}), "
                f"PSNR={psnr_score:.1f}dB (need >={psnr_threshold}dB)"
            )

    print()
    print(f"Results: {len(generated_pngs)} slides compared, {len(violations)} violations.")

    if violations:
        print("\nViolations:")
        for v in violations:
            print(f"  - {v}")
        if fail_on_violation:
            return 1

    return 0


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare generated slide PNGs against reference PNGs.",
    )
    parser.add_argument(
        "--generated",
        required=True,
        type=Path,
        help="Directory containing generated PNG files.",
    )
    parser.add_argument(
        "--reference",
        required=True,
        type=Path,
        help="Directory containing reference (ground-truth) PNG files.",
    )
    parser.add_argument(
        "--ssim-threshold",
        type=float,
        default=0.99,
        help="Minimum acceptable SSIM score per slide (NFR-007, default: 0.99).",
    )
    parser.add_argument(
        "--psnr-threshold",
        type=float,
        default=35.0,
        help="Minimum acceptable PSNR in dB per slide (NFR-008, default: 35.0).",
    )
    parser.add_argument(
        "--fail-on-violation",
        action="store_true",
        help="Exit with code 1 if any slide fails a threshold.",
    )
    args = parser.parse_args()

    check_imports()

    print(
        f"Visual regression check\n"
        f"  Generated: {args.generated}\n"
        f"  Reference: {args.reference}\n"
        f"  SSIM threshold: >= {args.ssim_threshold}\n"
        f"  PSNR threshold: >= {args.psnr_threshold} dB\n"
    )

    exit_code = compare_slides(
        generated_dir=args.generated,
        reference_dir=args.reference,
        ssim_threshold=args.ssim_threshold,
        psnr_threshold=args.psnr_threshold,
        fail_on_violation=args.fail_on_violation,
    )
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
