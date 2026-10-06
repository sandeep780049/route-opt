"""Command line interface: read a scenario JSON file, emit the answer JSON."""

from __future__ import annotations

import argparse
import json
import sys
from typing import TextIO

import numpy as np

from routeopt.tour import Problem, solve


def run_scenario(scenario: dict) -> dict:
    """Build a Problem from a scenario dict and solve it."""
    rows = scenario["points"]
    coords = np.array([[p["x"], p["y"]] for p in rows], dtype=np.float64)
    problem = Problem(
        coords=coords,
        strategy=scenario["strategy"],
        seed=int(scenario.get("seed", 0)),
    )
    return solve(problem)


def main(argv: list[str] | None = None) -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(prog="routeopt")
    parser.add_argument("scenario", help="path to a scenario JSON file")
    parser.add_argument("--pretty", action="store_true", help="indent the output")
    args = parser.parse_args(argv)

    with open(args.scenario, encoding="utf-8") as fh:
        scenario = json.load(fh)

    try:
        answer = run_scenario(scenario)
    except ValueError as exc:
        print(f"routeopt: {exc}", file=sys.stderr)
        return 2

    out: TextIO = sys.stdout
    json.dump(answer, out, indent=2 if args.pretty else None)
    out.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
