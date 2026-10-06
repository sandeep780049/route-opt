# Release checklist

- [ ] `python -m pytest tests -q` green (30 tests)
- [ ] `cargo test --manifest-path rust/Cargo.toml` green (5 tests)
- [ ] `g++ -std=c++14 -I cpp/include -o dist/clonecheck_test cpp/test/points_test.cpp && ./dist/clonecheck_test` exits 0
- [ ] `mvn -B test --file java/pom.xml` green (4 tests, CI)
- [ ] `npm --prefix ts install && npm --prefix ts test && npm --prefix ts run typecheck` green (7 tests)
- [ ] `python scripts/gen_corpus.py --force` regenerates corpus; `git diff examples` shows only expected churn
- [ ] all language CHANGELOGs updated, if any
- [ ] version bumps in `core/pyproject.toml`, `rust/Cargo.toml`, `ts/package.json`, `java/pom.xml` kept in lockstep
