# route-opt

A small, self-contained framework for studying multi-strategy route
optimization on point sets. The same solver contract is implemented
independently in Python 3, Rust, C++ and TypeScript, cross-checked by a
shared scenario corpus, with a Java plane-sweep utility. Modules live in
their own directories:

| Language | Directory | Role |
| --- | --- | --- |
| Python 3 | `core/` | Reference implementation: all five strategies, CLI, contract helpers. |
| Rust | `rust/` | `route_engine` binary: batch runner + per-strategy stats over the corpus. |
| C++ | `cpp/` | Point parsing/path-length utility + CLI used by non-Python tooling. |
| TypeScript | `ts/` | Contract verifier: loads scenario JSON, runs port binaries, reports PASS/FAIL matrix. |
| Java | `java/` | Plane-sweep closest-pair diagnostic for corpus sanity checks. |

Scenarios are built from classical construction heuristics (nearest
neighbour, greedy edge insertion, cheapest/farthest insertion) plus the
Lin–Kernighan-style 2-opt improvement pass, exercised on generated point
clouds rather than TSPLIB instances.

## Contract

Every solver accepts:

- a point set serialized as a JSON array of objects
  `{ "id": int, "x": float, "y": float }`,
- a strategy name (`"nearest_neighbour"`, `"greedy"`, `"cheapest_insertion"`,
  `"farthest_insertion"`, `"two_opt"`),
- an optional random seed for shuffle/tie-breaking determinism.

Every solver emits:

```json
{
  "solver": "greedy",
  "tour": [3, 1, 4, 0, 2],
  "length": 17.83,
  "checks": { "visitedOnce": true, "closesCycle": true, "finiteLength": true }
}
```

`tour` is a permutation of input ids, `length` is the closed-tour length in
Euclidean distance rounded half-up to 2 decimals so that implementations in
different languages agree bit-for-bit at the JSON level, and `checks` records
verification booleans computed by the solver (and re-derived by the TS
verifier, so a port cannot assert its own booleans to pass).

The full spec — including the rounding rule and determinism guarantees — is
in [`docs/contract.md`](docs/contract.md).

## Running locally

```bash
# Python reference + tests
python -m pip install -e core
pytest tests/ -q

# Rust engine
cargo test --manifest-path rust/Cargo.toml

# C++ utility
g++ -std=c++14 -I cpp/include -o dist/clonecheck cpp/src/points_cli.cpp

# Java plane-sweep diagnostics
mvn -B test --file java/pom.xml

# TypeScript verifier
npm --prefix ts install && npm --prefix ts test

# Cross-language verification pass
python scripts/gen_corpus.py --out examples --force
npm --prefix ts run verify -- --python python --corpus examples
```

Or run everything at once: `make help` lists the same steps one per target.

## Status

Experimental. Roadmap:

1. Grow the corpus — larger point clouds, degenerate collinear/duplicate
   sets, seeds that force tie-breaks.
2. Port `two_opt` into Rust/C++/Java for real cross-language performance
   benchmarking (currently only Python and Rust run it over the corpus).
3. Wire the C++/Java ports into `ts/src/verify.ts` as first-class citizens,
   so all four languages appear in the PASS/FAIL matrix.

If you want a similar deep dive on real TSPLIB instances, see
[or-tools routing examples](https://developers.google.com/optimization/routing).

## License

MIT — see [LICENSE](LICENSE).
