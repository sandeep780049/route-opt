"""Geometry and contract-verification tests."""

from __future__ import annotations

import math

import numpy as np
import pytest
from routeopt.geometry import distance, round_half_up, tour_length
from routeopt.verify import check_tour


def test_triangle_distance():
    a = (0.0, 0.0)
    b = (3.0, 4.0)
    assert distance(a, b) == pytest.approx(5.0)


def test_tour_length_closes_cycle():
    coords = np.array(
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], dtype=np.float64
    )
    # looped square: 0 -> 1 -> 2 -> 3 -> 0 is exactly 4.0
    assert tour_length(coords, [0, 1, 2, 3]) == pytest.approx(4.0)
    # a 3-length tour cycles back to the start:
    # legs 0->1 (1) + 1->2 (1) + 2->0 (sqrt(2)) = 2 + sqrt(2)
    assert tour_length(coords, [0, 1, 2]) == pytest.approx(2.0 + math.sqrt(2))
    # and it must differ from the open-path length (3.0), proving closure
    assert tour_length(coords, [0, 1, 2]) > 3.0


def test_round_half_up_contract():
    # ties round away from zero, unlike Python's built-in round()
    assert round_half_up(2.675) == 2.68
    assert round_half_up(0.125) == 0.13
    assert round_half_up(-0.125) == -0.13
    assert round_half_up(1.005) == 1.0  # 1.005 is not exactly representable


def test_round_half_up_matches_other_ports():
    for raw, expected in [(2.345, 2.35), (2.344, 2.34), (12.0, 12.0)]:
        assert round_half_up(raw) == expected


def test_check_tour_happy_path():
    coords = np.zeros((4, 2))
    checks = check_tour(coords, [0, 1, 2, 3])
    assert checks == {
        "visitedOnce": True,
        "closesCycle": True,
        "finiteLength": True,
    }


def test_check_tour_rejects_incomplete_and_repeats():
    coords = np.zeros((4, 2))
    assert check_tour(coords, [0, 1, 2])["visitedOnce"] is False
    assert check_tour(coords, [0, 1, 2, 2])["closesCycle"] is False


def test_check_tour_flags_nan_coords():
    coords = np.array([[0.0, 0.0], [float("nan"), 1.0], [1.0, 1.0], [0.0, 1.0]])
    assert check_tour(coords, [0, 1, 2, 3])["finiteLength"] is False
