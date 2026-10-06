.PHONY: help python rust cpp java ts verify corpus clean

help:
	@echo "make corpus   - regenerate examples/ expected answers from the Python port"
	@echo "make python   - run the Python test suite"
	@echo "make rust     - cargo test + release build"
	@echo "make cpp      - build the points utility"
	@echo "make java     - maven test (requires local maven)"
	@echo "make ts       - vitest + typecheck"
	@echo "make verify   - cross-language verification pass"

corpus:
	python scripts/gen_corpus.py --force

python:
	python -m pytest tests -q

rust:
	cargo test --manifest-path rust/Cargo.toml

cpp:
	@mkdir -p dist
	g++ -std=c++14 -I cpp/include -o dist/clonecheck cpp/src/*.cpp

java:
	mvn -B test --file java/pom.xml

ts:
	npm --prefix ts install
	npm --prefix ts test

verify: corpus
	python -m pip install -e core
	npm --prefix ts install
	npx --prefix ts tsx ts/src/verify.ts --python python --corpus examples

clean:
	rm -rf dist .pytest_cache .mypy_cache .ruff_cache rust/target ts/node_modules core/build core/*.egg-info
