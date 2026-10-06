"""Generate the scenario corpus under examples/.

Each scenario embeds the reference answer computed by the Python port, so the
TypeScript verifier can grade every other implementation against it.

Usage:
    python scripts/gen_corpus.py [--out examples] [--force]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "core"))

from routeopt.tour import Problem, solve


def _grid(n_side: int, spread: float) -> list[dict]:
    pts = []
    k = 0
    for i in range(n_side):
        for j in range(n_side):
            pts.append({"id": k, "x": round(i * spread, 3), "y": round(j * spread, 3)})
            k += 1
    return pts


def _ring(n: int, radius: float, cx: float = 50.0, cy: float = 50.0) -> list[dict]:
    import math

    pts = []
    for k in range(n):
        theta = 2 * math.pi * k / n
        pts.append(
            {
                "id": k,
                "x": round(cx + radius * math.cos(theta), 3),
                "y": round(cy + radius * math.sin(theta), 3),
            }
        )
    return pts


def _scatter(n: int, seed: int, lo: float = 0.0, hi: float = 100.0) -> list[dict]:
    import random

    rng = random.Random(seed)
    return [
        {
            "id": k,
            "x": round(rng.uniform(lo, hi), 3),
            "y": round(rng.uniform(lo, hi), 3),
        }
        for k in range(n)
    ]


def scenario_files() -> dict[str, dict]:
    """Corpus definitions: name -> scenario document without `expect`."""
    return {
        "unit-square-4": {
            "name": "unit-square-4",
            "strategy": "two_opt",
            "seed": 0,
            "points": [
                {"id": 0, "x": 0.0, "y": 0.0},
                {"id": 1, "x": 1.0, "y": 0.0},
                {"id": 2, "x": 1.0, "y": 1.0},
                {"id": 3, "x": 0.0, "y": 1.0},
            ],
        },
        "grid-9-greedy": {
            "name": "grid-9-greedy",
            "strategy": "greedy",
            "seed": 11,
            "points": _grid(3, 7.5),
        },
        "grid-16-nn": {
            "name": "grid-16-nn",
            "strategy": "nearest_neighbour",
            "seed": 3,
            "points": _grid(4, 12.0),
        },
        "ring-12-insertion": {
            "name": "ring-12-insertion",
            "strategy": "cheapest_insertion",
            "seed": 21,
            "points": _ring(12, 30.0),
        },
        "scatter-18-farthest": {
            "name": "scatter-18-farthest",
            "strategy": "farthest_insertion",
            "seed": 5,
            "points": _scatter(18, seed=202),
        },
        "scatter-25-twoopt": {
            "name": "scatter-25-twoopt",
            "strategy": "two_opt",
            "seed": 9,
            "points": _scatter(25, seed=77),
        },
        "collinear-8": {
            "name": "collinear-8",
            "strategy": "two_opt",
            "seed": 1,
            "points": [{"id": k, "x": float(k), "y": 2.5} for k in range(8)],
        },
        "single-point": {
            "name": "single-point",
            "strategy": "nearest_neighbour",
            "seed": 0,
            "points": [{"id": 0, "x": 4.2, "y": -1.7}],
        },
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="gen_corpus")
    parser.add_argument("--out", default="examples", help="output directory")
    parser.add_argument("--force", action="store_true", help="overwrite existing files")
    args = parser.parse_args(argv)

    out_dir = REPO_ROOT / args.out
    out_dir.mkdir(parents=True, exist_ok=True)

    written = 0
    for name, doc in scenario_files().items():
        target = out_dir / f"{name}.json"
        if target.exists() and not args.force:
            print(f"skip (exists): {target}")
            continue
        coords_rows = doc["points"]
        import numpy as np

        coords = np.array([[p["x"], p["y"]] for p in coords_rows], dtype=np.float64)
        answer = solve(
            Problem(coords=coords, strategy=doc["strategy"], seed=doc["seed"])
        )
        payload = dict(doc)
        payload["expect"] = {
            "tour": answer["tour"],
            "length": answer["length"],
        }
        target.write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"wrote: {target}")
        written += 1
    print(f"done ({written} written)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
