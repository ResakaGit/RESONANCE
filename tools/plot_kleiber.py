#!/usr/bin/env python3
"""Plot Kleiber's law from `demos kleiber --csv` output (matplotlib version).

Bonus alternative to the built-in SVG export (`demos kleiber --svg out.svg`),
for users who already have matplotlib installed.

Usage:
    cargo run --release --bin demos -- kleiber --n 256 --csv kleiber.csv
    python3 tools/plot_kleiber.py kleiber.csv --out kleiber.png

The resulting PNG is publication-quality (300 DPI by default).
"""

from __future__ import annotations

import argparse
import csv
import math
import sys
from pathlib import Path


def main() -> int:
    p = argparse.ArgumentParser(description="Plot Kleiber's law from a demos CSV.")
    p.add_argument("csv", type=Path, help="CSV produced by `demos kleiber --csv`.")
    p.add_argument("--out", type=Path, default=Path("kleiber.png"),
                   help="Output PNG path (default: kleiber.png).")
    p.add_argument("--dpi", type=int, default=300)
    p.add_argument("--axiomatic", type=float, default=0.75,
                   help="KLEIBER_EXPONENT for reference line (default: 0.75).")
    args = p.parse_args()

    try:
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError as e:
        print(f"error: needs matplotlib + numpy ({e})", file=sys.stderr)
        return 1

    # Load (mass, metabolic_rate) pairs from CSV.
    masses, rates = [], []
    with args.csv.open() as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                m = float(row["mass"])
                b = float(row["metabolic_rate"])
                if m > 0 and b > 0 and math.isfinite(m) and math.isfinite(b):
                    masses.append(m)
                    rates.append(b)
            except (KeyError, ValueError):
                continue

    if not masses:
        print("error: no valid (mass, metabolic_rate) rows in CSV", file=sys.stderr)
        return 2

    masses = np.array(masses)
    rates = np.array(rates)
    log_m = np.log(masses)
    log_b = np.log(rates)

    # Linear regression in log-log space (Kleiber's law form).
    slope, intercept = np.polyfit(log_m, log_b, 1)

    # Plot.
    fig, ax = plt.subplots(figsize=(8, 6))
    ax.scatter(masses, rates, s=18, alpha=0.55, c="#0066cc",
               edgecolors="none", label=f"n={len(masses)} samples")

    # Regression line.
    x_line = np.linspace(masses.min(), masses.max(), 200)
    y_line = np.exp(intercept) * x_line ** slope
    ax.plot(x_line, y_line, "r--", linewidth=1.8,
            label=f"fit: B ∝ M^{slope:.4f}")

    # Axiomatic reference (anchor through fitted intercept).
    y_ax = np.exp(intercept) * x_line ** args.axiomatic
    ax.plot(x_line, y_ax, "g:", linewidth=1.2, alpha=0.7,
            label=f"axiomatic: B ∝ M^{args.axiomatic}")

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("body mass (arbitrary units)")
    ax.set_ylabel("metabolic rate (arbitrary units)")
    ax.set_title(
        "Kleiber's law verification — log-log regression\n"
        f"fitted slope {slope:.6f} vs axiomatic {args.axiomatic} "
        f"(error {abs(slope - args.axiomatic):.2e})"
    )
    ax.grid(True, which="both", linestyle="--", alpha=0.3)
    ax.legend(loc="best")
    fig.tight_layout()
    fig.savefig(args.out, dpi=args.dpi)
    print(f"wrote {args.out} ({args.dpi} DPI, slope={slope:.6f})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
