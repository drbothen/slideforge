#!/usr/bin/env python3
"""
S6 Spike: Visual diff tool for PPTX slide PNG comparison.

Compares two directories of slide PNGs (reference vs actual) using:
  1. SSIM (Structural Similarity Index) — primary perceptual metric
  2. PSNR (Peak Signal-to-Noise Ratio) — secondary quality metric
  3. Pixel diff count — quick sanity check

Usage:
    python3 visual-diff.py <reference-dir> <actual-dir> [--out diff-dir] [--strict]

Exit codes:
    0 — all slides pass threshold
    1 — one or more slides fail threshold
    2 — setup error (missing files, mismatched slide counts)

Thresholds (from S6 spike research; may be tuned via env vars):
    SSIM_THRESHOLD    default: 0.97   (fail if SSIM < this)
    PSNR_THRESHOLD_DB default: 35.0   (warn if PSNR < this; fail in --strict)

Requirements:
    pip install Pillow scikit-image numpy
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import NamedTuple

import numpy as np
from PIL import Image
from skimage.metrics import peak_signal_noise_ratio as psnr
from skimage.metrics import structural_similarity as ssim


# ---- Thresholds (overridable via env) ----------------------------------

SSIM_THRESHOLD = float(os.environ.get("SSIM_THRESHOLD", "0.97"))
PSNR_THRESHOLD_DB = float(os.environ.get("PSNR_THRESHOLD_DB", "35.0"))


# ---- Result type -------------------------------------------------------

class SlideResult(NamedTuple):
    slide: str        # filename
    ssim_val: float
    psnr_val: float   # float('inf') if images identical
    pixel_diff_count: int
    pixel_diff_pct: float
    ssim_pass: bool
    psnr_warn: bool
    width: int
    height: int
    status: str       # PASS | WARN | FAIL


# ---- Core comparison ---------------------------------------------------

def load_as_rgb_array(path: Path) -> np.ndarray:
    """Load PNG as H×W×3 uint8 array, stripping alpha."""
    img = Image.open(path).convert("RGB")
    return np.array(img)


def compare_slides(ref_path: Path, act_path: Path) -> SlideResult:
    ref_arr = load_as_rgb_array(ref_path)
    act_arr = load_as_rgb_array(act_path)

    # If dimensions differ, resize actual to reference size for comparison.
    # A dimension mismatch itself is a finding; document it.
    if ref_arr.shape != act_arr.shape:
        from PIL import Image as PILImage
        act_img = PILImage.open(act_path).convert("RGB").resize(
            (ref_arr.shape[1], ref_arr.shape[0]), PILImage.LANCZOS
        )
        act_arr = np.array(act_img)

    h, w = ref_arr.shape[:2]

    # SSIM: compare over luminance (single channel) for speed; multichannel=True
    # gives per-channel values — use channel_axis=-1 for skimage >= 0.19
    ssim_val = float(ssim(ref_arr, act_arr, channel_axis=-1, data_range=255))

    # PSNR
    psnr_val = float(psnr(ref_arr, act_arr, data_range=255))

    # Raw pixel diff: count pixels where any channel differs by > 4/255
    diff = np.abs(ref_arr.astype(np.int16) - act_arr.astype(np.int16))
    pixel_diff_count = int(np.sum(np.any(diff > 4, axis=-1)))
    pixel_diff_pct = pixel_diff_count / (h * w) * 100.0

    ssim_pass = ssim_val >= SSIM_THRESHOLD
    psnr_warn = psnr_val < PSNR_THRESHOLD_DB

    if not ssim_pass:
        status = "FAIL"
    elif psnr_warn:
        status = "WARN"
    else:
        status = "PASS"

    return SlideResult(
        slide=ref_path.name,
        ssim_val=ssim_val,
        psnr_val=psnr_val,
        pixel_diff_count=pixel_diff_count,
        pixel_diff_pct=pixel_diff_pct,
        ssim_pass=ssim_pass,
        psnr_warn=psnr_warn,
        width=w,
        height=h,
        status=status,
    )


def write_diff_image(ref_path: Path, act_path: Path, out_path: Path) -> None:
    """Write an amplified diff PNG to out_path for visual inspection."""
    ref = load_as_rgb_array(ref_path)
    act_arr = load_as_rgb_array(act_path)
    if ref.shape != act_arr.shape:
        from PIL import Image as PILImage
        act_img = PILImage.open(act_path).convert("RGB").resize(
            (ref.shape[1], ref.shape[0]), PILImage.LANCZOS
        )
        act_arr = np.array(act_img)
    diff = np.abs(ref.astype(np.int16) - act_arr.astype(np.int16)).astype(np.uint8)
    # Amplify by 8x so subtle differences become visible
    amplified = np.clip(diff.astype(np.uint16) * 8, 0, 255).astype(np.uint8)
    Image.fromarray(amplified).save(out_path, format="PNG")


# ---- CLI ---------------------------------------------------------------

def collect_slide_pngs(directory: Path) -> list[Path]:
    """Return sorted list of *.png files in directory."""
    return sorted(directory.glob("*.png"))


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Compare reference vs actual slide PNGs for visual regression."
    )
    parser.add_argument("reference", help="Directory of reference (blessed) PNGs")
    parser.add_argument("actual", help="Directory of actual (newly rendered) PNGs")
    parser.add_argument("--out", help="Directory to write diff images", default=None)
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail on PSNR warnings in addition to SSIM failures",
    )
    parser.add_argument(
        "--json", dest="json_out", default=None,
        help="Write JSON report to this path",
    )
    args = parser.parse_args()

    ref_dir = Path(args.reference)
    act_dir = Path(args.actual)
    diff_dir = Path(args.out) if args.out else None

    if not ref_dir.is_dir():
        print(f"ERROR: reference directory not found: {ref_dir}", file=sys.stderr)
        return 2
    if not act_dir.is_dir():
        print(f"ERROR: actual directory not found: {act_dir}", file=sys.stderr)
        return 2

    if diff_dir:
        diff_dir.mkdir(parents=True, exist_ok=True)

    ref_pngs = collect_slide_pngs(ref_dir)
    act_pngs = collect_slide_pngs(act_dir)

    if len(ref_pngs) == 0:
        print(f"ERROR: no PNG files found in reference directory: {ref_dir}", file=sys.stderr)
        return 2

    if len(ref_pngs) != len(act_pngs):
        print(
            f"ERROR: slide count mismatch — reference has {len(ref_pngs)}, actual has {len(act_pngs)}",
            file=sys.stderr,
        )
        return 2

    print(f"Comparing {len(ref_pngs)} slide(s)...")
    print(f"  SSIM threshold:  >= {SSIM_THRESHOLD}")
    print(f"  PSNR threshold:  >= {PSNR_THRESHOLD_DB} dB")
    print(f"  Strict mode:     {'yes' if args.strict else 'no'}")
    print()

    results: list[SlideResult] = []
    for ref_path, act_path in zip(ref_pngs, act_pngs):
        result = compare_slides(ref_path, act_path)
        results.append(result)

        status_icon = {"PASS": ".", "WARN": "W", "FAIL": "F"}[result.status]
        psnr_str = f"{result.psnr_val:6.1f} dB" if result.psnr_val != float("inf") else "    inf"
        print(
            f"  [{status_icon}] {result.slide:<40} "
            f"SSIM={result.ssim_val:.4f}  PSNR={psnr_str}  "
            f"diff={result.pixel_diff_pct:.2f}%"
        )

        if diff_dir and result.status != "PASS":
            diff_path = diff_dir / f"diff-{result.slide}"
            write_diff_image(ref_path, act_path, diff_path)

    # Summary
    passes = sum(1 for r in results if r.status == "PASS")
    warns = sum(1 for r in results if r.status == "WARN")
    fails = sum(1 for r in results if r.status == "FAIL")

    print()
    print(f"Results: {passes} pass, {warns} warn, {fails} fail  (of {len(results)} slides)")

    if diff_dir:
        print(f"Diff images written to: {diff_dir}")

    # JSON report
    if args.json_out:
        report = {
            "thresholds": {
                "ssim": SSIM_THRESHOLD,
                "psnr_db": PSNR_THRESHOLD_DB,
            },
            "summary": {"pass": passes, "warn": warns, "fail": fails, "total": len(results)},
            "slides": [r._asdict() for r in results],
        }
        with open(args.json_out, "w") as f:
            json.dump(report, f, indent=2)
        print(f"JSON report: {args.json_out}")

    # Exit code
    hard_fails = fails + (warns if args.strict else 0)
    return 1 if hard_fails > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
