"""Geometry helpers shared by all strategies."""

from __future__ import annotations

import math
from collections.abc import Sequence

Point = tuple[float, float]


def distance(a: Point, b: Point) -> float:
    """Euclidean distance between two points."""
    return math.hypot(a[0] - b[0], a[1] - b[1])


def tour_length(points: Sequence[Point], tour: Sequence[int]) -> float:
    """Closed-tour length of ``tour`` over ``points``.

    The tour is treated as a cycle: the leg from the last visited point back
    to the first is included.
    """
    if not tour:
        return 0.0
    total = 0.0
    for i in range(len(tour)):
        a = points[tour[i]]
        b = points[tour[(i + 1) % len(tour)]]
        total += distance(a, b)
    return total


def round_half_up(value: float) -> float:
    """Round to 2 decimals, half away from zero.

    Built-in ``round()`` uses banker's rounding, which makes ties disagree
    between languages; this mirrors the decimal half-up rule the contract
    requires.
    """
    scaled = value * 100.0
    if scaled >= 0:
        rounded = math.floor(scaled + 0.5)
    else:
        rounded = math.ceil(scaled - 0.5)
    return rounded / 100.0
