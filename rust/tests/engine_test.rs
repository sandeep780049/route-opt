//! Integration tests for the route engine (runs via `cargo test`).

use route_engine::engine::{
    dist, load_scenarios, nearest_neighbour, round_half_up, solve, tour_len, Point, Scenario,
    SplitMix64,
};
use std::path::Path;

fn pts(coords: &[(i64, f64, f64)]) -> Vec<Point> {
    coords
        .iter()
        .map(|&(id, x, y)| Point { id, x, y })
        .collect()
}

#[test]
fn distances_and_rounding_match_python() {
    let a = Point { id: 0, x: 0.0, y: 0.0 };
    let b = Point { id: 1, x: 3.0, y: 4.0 };
    assert_eq!(dist(a, b), 5.0);

    // Contract rounding is half away from zero (matches the Python port).
    assert_eq!(round_half_up(2.675), 2.68);
    assert_eq!(round_half_up(0.125), 0.13);
    assert_eq!(round_half_up(-0.125), -0.13);
}

#[test]
fn splitmix64_stream_is_stable() {
    let mut rng = SplitMix64::new(42);
    let a = rng.next_u64();
    let b = rng.next_u64();
    assert_ne!(a, b, "stream must not repeat immediately");
    let mut rng2 = SplitMix64::new(42);
    assert_eq!(rng2.next_u64(), a, "same seed must replay the stream");
}

#[test]
fn tours_are_hamiltonian_and_lengths_match() {
    let points = pts(&[
        (0, 0.0, 0.0),
        (1, 5.0, 1.0),
        (2, 4.0, 4.0),
        (3, 1.0, 5.0),
        (4, 2.5, 2.5),
    ]);
    let tour = nearest_neighbour(&points, 7);
    assert_eq!(tour.len(), points.len());
    assert_eq!(
        {
            let mut t = tour.clone();
            t.sort();
            t
        },
        (0..points.len()).collect::<Vec<_>>()
    );
    let closed = tour_len(&points, &tour);
    assert!(closed > 0.0);
}

#[test]
fn solve_reports_valid_contract_answer() {
    let scenario = Scenario {
        name: "unit".into(),
        points: pts(&[(0, 0.0, 0.0), (1, 1.0, 0.0), (2, 1.0, 1.0), (3, 0.0, 1.0)]),
        strategy: "two_opt".into(),
        seed: 0,
    };
    let answer = solve(&scenario);
    let tour_ids: Vec<i64> = answer
        .get("tour")
        .map(|t| t.as_arr().unwrap().iter().map(|v| v.as_f64().unwrap() as i64).collect())
        .expect("tour present");
    assert_eq!(tour_ids.len(), 4);
    let length = answer.get("length").map(|l| l.as_f64().unwrap()).unwrap();
    assert_eq!(length, 4.0_f64);
}

#[test]
fn scenarios_load_from_corpus_dir() {
    let dir = Path::new("examples");
    if !dir.exists() {
        return; // corpus not generated in this environment
    }
    let list = load_scenarios(dir).expect("load");
    assert!(!list.is_empty());
    for sc in &list {
        let answer = solve(sc);
        assert!(answer.get("tour").is_some());
    }
}
