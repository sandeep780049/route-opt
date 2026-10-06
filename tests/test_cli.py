"""End-to-end CLI tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from routeopt.cli import main, run_scenario

SCENARIO = {
    "name": "square",
    "strategy": "two_opt",
    "seed": 0,
    "points": [
        {"id": 0, "x": 0.0, "y": 0.0},
        {"id": 1, "x": 1.0, "y": 0.0},
        {"id": 2, "x": 1.0, "y": 1.0},
        {"id": 3, "x": 0.0, "y": 1.0},
    ],
}


def test_run_scenario_returns_contract_answer():
    answer = run_scenario(SCENARIO)
    assert answer["solver"] == "two_opt"
    assert sorted(answer["tour"]) == [0, 1, 2, 3]
    assert answer["length"] == 4.0
    assert answer["checks"]["closesCycle"] is True


def test_cli_roundtrip(tmp_path: Path, capsys):
    scenario_path = tmp_path / "square.json"
    scenario_path.write_text(json.dumps(SCENARIO), encoding="utf-8")
    rc = main([str(scenario_path)])
    assert rc == 0
    out = capsys.readouterr().out
    answer = json.loads(out)
    assert answer["length"] == 4.0


def test_cli_rejects_unknown_strategy(tmp_path: Path, capsys):
    scenario_path = tmp_path / "bad.json"
    scenario_path.write_text(
        json.dumps({**SCENARIO, "strategy": "magic"}), encoding="utf-8"
    )
    rc = main([str(scenario_path)])
    assert rc == 2
    assert "unknown strategy" in capsys.readouterr().err


def test_corpus_scenarios_parse_and_solve():
    import sys

    corpus = Path(__file__).resolve().parent.parent / "examples"
    if not corpus.exists():
        pytest.skip("corpus not generated")
    dumped = sys.modules.get("routeopt.tour")
    assert dumped is not None  # reference module imported
    from routeopt.cli import run_scenario as _rs

    for scenario_file in sorted(corpus.glob("*.json")):
        scenario = json.loads(scenario_file.read_text(encoding="utf-8"))
        answer = _rs(scenario)
        assert answer["checks"]["visitedOnce"] is True
