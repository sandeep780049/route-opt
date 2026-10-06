"""Contract checks: what makes a tour acceptable."""

from __future__ import annotations

from typing import Any

import numpy as np


def check_tour(coords: np.ndarray, tour: list[int]) -> dict[str, Any]:
    """Return the standard verification booleans for a candidate tour.

    ``visitedOnce``  - the tour has length n and no id repeats
    ``closesCycle``  - the id set exactly equals the input id set
    ``finiteLength`` - every leg is finite (guards NaN coordinate input)
    """
    n = coords.shape[0]
    visited_once = len(tour) == n and len(set(tour)) == n
    closes_cycle = visited_once and set(tour) == set(range(n))
    finite = bool(np.isfinite(coords).all())
    return {
        "visitedOnce": bool(visited_once),
        "closesCycle": bool(closes_cycle),
        "finiteLength": finite,
    }
