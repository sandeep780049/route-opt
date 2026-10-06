"""Tour construction and improvement strategies.

Each strategy maps a ``Problem`` to a full tour (a permutation of the point
ids). Implementations are deliberately explicit about loop structure rather
than vectorized, so the same shape can be compared line-by-line with the
ports in other languages. All tie-breaking is deterministic: ties resolve to
the smallest id, except inside strategy-internal shuffles which consume the
seeded ``rng``.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import NamedTuple

import numpy as np

from routeopt.geometry import distance

Strategy = Callable[[np.ndarray, np.random.Generator], list[int]]


class Problem(NamedTuple):
    """A solver task: point coordinates plus strategy selection."""

    coords: np.ndarray  # shape (n, 2), float64
    strategy: str
    seed: int = 0


def nearest_neighbour(coords: np.ndarray, rng: np.random.Generator) -> list[int]:
    """Walk from a random start to the closest unvisited point each step."""
    n = len(coords)
    start = int(rng.integers(n))
    unvisited = set(range(n))
    unvisited.remove(start)
    tour = [start]
    current = start
    while unvisited:
        nxt = min(unvisited, key=lambda j: (distance(coords[current], coords[j]), j))
        tour.append(nxt)
        unvisited.remove(nxt)
        current = nxt
    return tour


def cheapest_insertion(coords: np.ndarray, rng: np.random.Generator) -> list[int]:
    """Start from a 2-point cycle, insert the point adding least length."""
    n = len(coords)
    menu = list(range(n))
    rng.shuffle(menu)
    tour = [menu[0], menu[1]]
    for cand in menu[2:]:
        best_pos, best_cost = -1, float("inf")
        for pos in range(len(tour)):
            u = tour[pos]
            v = tour[(pos + 1) % len(tour)]
            cost = (
                distance(coords[u], coords[cand])
                + distance(coords[cand], coords[v])
                - distance(coords[u], coords[v])
            )
            if cost < best_cost:
                best_pos, best_cost = pos, cost
        tour.insert(best_pos + 1, cand)
    return tour


def farthest_insertion(coords: np.ndarray, rng: np.random.Generator) -> list[int]:
    """Grow a cycle by always inserting the point farthest from the tour."""
    n = len(coords)
    pool = list(range(n))
    rng.shuffle(pool)
    tour = [pool[0], pool[1]]
    remaining = pool[2:]

    def min_dist_to_tour(p: int) -> float:
        return min(distance(coords[p], coords[t]) for t in tour)

    while remaining:
        far = max(remaining, key=min_dist_to_tour)
        best_pos, best_cost = -1, float("inf")
        for pos in range(len(tour)):
            u = tour[pos]
            v = tour[(pos + 1) % len(tour)]
            cost = (
                distance(coords[u], coords[far])
                + distance(coords[far], coords[v])
                - distance(coords[u], coords[v])
            )
            if cost < best_cost:
                best_pos, best_cost = pos, cost
        tour.insert(best_pos + 1, far)
        remaining.remove(far)
    return tour


def greedy(coords: np.ndarray, rng: np.random.Generator) -> list[int]:
    """Sort edges by length, join endpoints while avoiding subtours.

    The accepted edges form a Hamiltonian path; the tour is reconstructed by
    walking it end to end, starting at the degree-1 endpoint with the
    smallest id.
    """
    n = len(coords)
    if n < 2:
        return list(range(n))
    edges = sorted(
        (distance(coords[i], coords[j]), i, j)
        for i in range(n)
        for j in range(i + 1, n)
    )
    degree = [0] * n
    parent = list(range(n))
    accepted: list[tuple[int, int]] = []

    def find(x: int) -> int:
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for _, i, j in edges:
        if degree[i] < 2 and degree[j] < 2 and find(i) != find(j):
            parent[find(i)] = find(j)
            degree[i] += 1
            degree[j] += 1
            accepted.append((i, j))
            if len(accepted) == n - 1:
                break
    adj: dict[int, list[int]] = {}
    for u, v in accepted:
        adj.setdefault(u, []).append(v)
        adj.setdefault(v, []).append(u)
    start = min(x for x in range(n) if len(adj.get(x, [])) == 1)
    tour = [start]
    prev = -1
    while len(tour) < n:
        nxts = [w for w in adj[tour[-1]] if w != prev]
        prev = tour[-1]
        tour.append(nxts[0])
    return tour


def two_opt(coords: np.ndarray) -> list[int]:
    """Full 2-opt on the canonical clockwise seed tour until it is 2-optimal."""
    n = len(coords)
    tour = list(range(n))
    improved = True
    while improved:
        improved = False
        for i in range(n - 1):
            for j in range(i + 2, n):
                if i == 0 and j == n - 1:
                    continue
                u, v = coords[tour[i]], coords[tour[i + 1]]
                w, x = coords[tour[j]], coords[tour[(j + 1) % n]]
                before = distance(u, v) + distance(w, x)
                after = distance(u, w) + distance(v, x)
                if after < before:
                    tour[i + 1 : j + 1] = tour[i + 1 : j + 1][::-1]
                    improved = True
    return tour


def solve(problem: Problem) -> dict:
    """Run the requested strategy and return a contract answer object."""
    from routeopt import verify
    from routeopt.geometry import round_half_up, tour_length

    coords = problem.coords
    n = coords.shape[0]
    if n < 2:
        tour = list(range(n))
    else:
        rng = np.random.default_rng(problem.seed)
        table: dict[str, Strategy] = {
            "nearest_neighbour": nearest_neighbour,
            "greedy": greedy,
            "cheapest_insertion": cheapest_insertion,
            "farthest_insertion": farthest_insertion,
        }
        if problem.strategy == "two_opt":
            tour = two_opt(coords)
        elif problem.strategy in table:
            tour = table[problem.strategy](coords, rng)
        else:
            raise ValueError(f"unknown strategy: {problem.strategy!r}")

    length = round_half_up(tour_length(coords, tour))
    return {
        "solver": problem.strategy,
        "tour": tour,
        "length": length,
        "checks": verify.check_tour(coords, tour),
    }
