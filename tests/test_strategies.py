"""Strategy-level tests: Hamiltonian output, determinism, known optimum."""

from __future__ import annotations

import numpy as np
import pytest

from routeopt.geometry import tour_length
from routeopt.tour import (
    Problem,
    cheapest_insertion,
    farthest_insertion,
    greedy,
    nearest_neighbour,
    solve,
    two_opt,
)

ALL_STRATEGIES = [
    "nearest_neighbour",
    "greedy",
    "cheapest_insertion",
    "farthest_insertion",
    "two_opt",
]


def _square() -> np.ndarray:
    return np.array([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], dtype=np.float64)


def _random_cloud(n: int, seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    return rng.random((n, 2)) * 100.0


@pytest.mark.parametrize("strategy", ALL_STRATEGIES)
def test_every_strategy_emits_hamiltonian_tour(strategy):
    coords = _random_cloud(12, seed=hash(strategy) % 1000)
    answer = solve(Problem(coords=coords, strategy=strategy, seed=7))
    tour = answer["tour"]
    assert sorted(tour) == list(range(12))
    assert answer["checks"]["visitedOnce"] is True
    assert answer["checks"]["closesCycle"] is True


def test_two_opt_matches_known_optimum_on_square():
    coords = _square()
    tour = two_opt(coords)
    assert tour_length(coords, tour) == pytest.approx(4.0)


def test_two_opt_crossing_removal():
    # five points where the id-order seed tour crosses itself
    coords = np.array(
        [
            [0.0, 0.0],
            [2.0, 0.0],  # id 1
            [1.0, 0.5],  # id 2, middle spike
            [0.0, 2.0],  # id 3
            [2.0, 2.0],  # id 4
        ],
        dtype=np.float64,
    )
    tour = two_opt(coords)
    length = tour_length(coords, tour)
    # optimum tours the five outer points and dips into the spike once:
    # 0 -> 3 -> 4 -> 2 -> 1 -> 0 = 2 + ~2.69 + ~2.06 + 1.0 + 1.0 < 9
    assert length < 9.0
    assert sorted(tour) == [0, 1, 2, 3, 4]


def test_greedy_is_deterministic_and_seed_insensitive():
    coords = _random_cloud(15, seed=3)
    a = greedy(coords, np.random.default_rng(1))
    b = greedy(coords, np.random.default_rng(999))
    assert a == b


def test_two_opt_deterministic():
    coords = _random_cloud(10, seed=8)
    assert two_opt(coords) == two_opt(coords)


@pytest.mark.parametrize("strategy", ["nearest_neighbour", "cheapest_insertion"])
def test_seeded_strategies_replay_identically(strategy):
    coords = _random_cloud(10, seed=5)
    r1 = solve(Problem(coords=coords, strategy=strategy, seed=42))
    r2 = solve(Problem(coords=coords, strategy=strategy, seed=42))
    assert r1["tour"] == r2["tour"]
    assert r1["length"] == r2["length"]


@pytest.mark.parametrize(
    "strategy", ["nearest_neighbour", "greedy", "cheapest_insertion", "farthest_insertion"]
)
def test_never_worse_than_twice_the_two_opt_length(strategy):
    # sanity ratio: raw construction is near-greedy, not catastrophically bad
    coords = _random_cloud(18, seed=11)
    construction = solve(Problem(coords=coords, strategy=strategy, seed=2))
    improved = solve(Problem(coords=coords, strategy="two_opt", seed=0))
    assert construction["length"] <= 2.0 * improved["length"]


def test_single_point_edge_case():
    coords = np.array([[3.0, 4.0]])
    answer = solve(Problem(coords=coords, strategy="nearest_neighbour", seed=0))
    assert answer["tour"] == [0]
    assert answer["length"] == 0.0


def test_two_point_edge_case():
    coords = np.array([[0.0, 0.0], [5.0, 0.0]])
    for strategy in ALL_STRATEGIES:
        answer = solve(Problem(coords=coords, strategy=strategy, seed=0))
        assert sorted(answer["tour"]) == [0, 1]
        assert answer["length"] == pytest.approx(10.0)


def test_unknown_strategy_raises():
    coords = _square()
    with pytest.raises(ValueError):
        solve(Problem(coords=coords, strategy="nope", seed=0))


def test_solution_length_matches_manual_calculation():
    coords = _random_cloud(8, seed=1)
    answer = solve(Problem(coords=coords, strategy="two_opt", seed=0))
    from routeopt.geometry import round_half_up

    recomputed = round_half_up(tour_length(coords, answer["tour"]))
    assert answer["length"] == recomputed
