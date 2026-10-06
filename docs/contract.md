# Contract specification

This document defines what every language implementation of `route-opt` must
agree on. It is short by design: the points of reference are the test suites
in each module, which pin the same scenarios.

## Scenario file

```json
{
  "name": "unit-square-4",
  "strategy": "two_opt",
  "seed": 0,
  "points": [
    { "id": 0, "x": 0.0, "y": 0.0 },
    { "id": 1, "x": 1.0, "y": 0.0 },
    { "id": 2, "x": 1.0, "y": 1.0 },
    { "id": 3, "x": 0.0, "y": 1.0 }
  ],
  "expect": { "tour": [0, 1, 2, 3], "length": 4.0 }
}
```

Fields:

| field      | required | notes                                        |
| ---------- | -------- | -------------------------------------------- |
| `name`     | no       | equals the file stem when loaded from disk    |
| `strategy` | no       | defaults to `nearest_neighbour`               |
| `seed`     | no       | integer; drives construction-strategy shuffles|
| `points`   | yes      | at least 1 point; coordinates finite          |
| `expect`   | no       | embedded reference answer for the verifier    |

Answer:

```json
{
  "solver": "two_opt",
  "tour": [0, 1, 2, 3],
  "length": 4.0,
  "checks": { "visitedOnce": true, "closesCycle": true, "finiteLength": true }
}
```

## Rounding rule

`length` = closed tour length, rounded **half away from zero** to 2 decimal
places. The tie behavior matters: values like `2.675` or `0.125` land exactly
on a midpoint at 2 decimals in decimal arithmetic, and implementations must
not disagree because of float representation. Python, Rust, TypeScript, C++
and Java ports all implement the same rule (see `tests` in each module).

## Determinism

- `two_opt` and `greedy` never consume randomness. Given the same points,
  every port must return the same tour.
- Construction strategies (`nearest_neighbour`, `cheapest_insertion`,
  `farthest_insertion`) shuffle their candidate ordering with the seeded
  SplitMix64 stream in Rust/Java/C++/TS and `numpy.random.default_rng` in
  Python. Shuffles must therefore agree across ports for the same `seed`,
  but the Python port is treated as the reference whenever its RNG stream
  differs.

## Verification booleans

| name           | meaning                                        |
| -------------- | ---------------------------------------------- |
| `visitedOnce`  | tour length equals point count, no id repeats  |
| `closesCycle`  | tour ids equal the point id set                |
| `finiteLength` | all coordinates finite (rejects NaN/Inf input) |

## Adding a port

1. Read `docs/contract.md` and the corresponding test module in Python.
2. Implement, keeping loop shapes close to the Rust port (easiest to diff).
3. Add a module-level test at the same coverage level as `tests/` here.
4. Extend `scripts/gen_corpus.py` if the port covers a new strategy.
