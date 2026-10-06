"""Geometry helpers shared by all strategies."""

from __future__ import annotations

import math
from collections.abc import Sequence

import numpy as np
from numpy.typing import ArrayLike

Point = tuple[float, float]


def distance(a: ArrayLike, b: ArrayLike) -> float:
    """Euclidean distance between two points.

    Accepts plain tuples/lists and numpy rows alike; both index coordinates
    positionally, which is exactly what the strategies rely on.
    """
    av = np.asarray(a, dtype=np.float64)
    bv = np.asarray(b, dtype=np.float64)
    return float(math.hypot(av[0] - bv[0], av[1] - bv[1]))


def tour_length(points: ArrayLike, tour: Sequence[int]) -> float:
    """Closed-tour length of ``tour`` over ``points``.

    The tour is treated as a cycle: the leg from the last visited point back
    to the first is included.
    """
    if not tour:
        return 0.0
    pts = np.asarray(points, dtype=np.float64)
    total = 0.0
    for i in range(len(tour)):
        a = pts[tour[i]]
        b = pts[tour[(i + 1) % len(tour)]]
        total += distance(a, b)
    return float(total)


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
