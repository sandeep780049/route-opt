# route_engine — Rust port tests

Run with `cargo test` from this directory. These tests mirror the Python suite:

- distance helper agrees with the closed-form 3-4-5 triangle,
- contract rounding is half-away-from-zero in both 2.675 and the negative tie,
- SplitMix64 streams are deterministic per seed,
- every strategy emits a Hamiltonian cycle over the input ids,
- two_opt on the unit square degenerates to length 4.0.
