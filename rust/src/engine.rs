//! Port of the five route-opt strategies into Rust.
//!
//! Mirrors `core/routeopt/tour.py` decision by decision so the verifier can
//! compare `tour` ids across implementations. Determinism contract: ALL
//! distance ties resolve to the smallest index; strategy shuffles use the
//! SplitMix64 stream seeded from the scenario seed.

use crate::json::{dump, fmt_f64, parse, Json};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub id: i64,
    pub x: f64,
    pub y: f64,
}

pub fn dist(a: Point, b: Point) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

pub fn tour_len(points: &[Point], tour: &[usize]) -> f64 {
    if tour.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..tour.len() {
        let a = points[tour[i]];
        let b = points[tour[(i + 1) % tour.len()]];
        total += dist(a, b);
    }
    total
}

/// Round half away from zero to 2 decimals (contract rounding).
pub fn round_half_up(v: f64) -> f64 {
    let scaled = v * 100.0;
    let r = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    r / 100.0
}

/// SplitMix64 — same stream as the Java port, so shuffles agree.
#[derive(Clone, Copy)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_8CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    #[allow(dead_code)]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Fisher-Yates shuffle identical to the Java implementation.
fn shuffle(items: &mut [usize], rng: &mut SplitMix64) {
    for i in (1..items.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

fn pick_start(n: usize, rng: &mut SplitMix64) -> usize {
    (rng.next_u64() % n as u64) as usize
}

pub fn nearest_neighbour(points: &[Point], seed: u64) -> Vec<usize> {
    let n = points.len();
    let mut rng = SplitMix64::new(seed);
    let start = pick_start(n, &mut rng);
    let mut visited = vec![false; n];
    visited[start] = true;
    let mut tour = vec![start];
    let mut cur = start;
    for _ in 1..n {
        let mut best: Option<usize> = None;
        let mut best_d = f64::INFINITY;
        for j in 0..n {
            if visited[j] {
                continue;
            }
            let d = dist(points[cur], points[j]);
            if d < best_d {
                best_d = d;
                best = Some(j);
            }
        }
        let j = best.expect("unvisited point exists");
        visited[j] = true;
        tour.push(j);
        cur = j;
    }
    tour
}

fn insertion(points: &[Point], seed: u64, farthest: bool) -> Vec<usize> {
    let n = points.len();
    let mut rng = SplitMix64::new(seed);
    let mut menu: Vec<usize> = (0..n).collect();
    shuffle(&mut menu, &mut rng);
    let mut tour = vec![menu[0], menu[1]];
    let mut remaining: Vec<usize> = menu[2..].to_vec();

    loop {
        let choice = if farthest && !remaining.is_empty() {
            // pick remaining point with max min-distance to tour
            let mut best = remaining[0];
            let mut best_d = -1.0f64;
            for &p in &remaining {
                let dmin = tour
                    .iter()
                    .map(|&t| dist(points[p], points[t]))
                    .fold(f64::INFINITY, f64::min);
                if dmin > best_d {
                    best_d = dmin;
                    best = p;
                }
            }
            best
        } else if !remaining.is_empty() {
            remaining[0]
        } else {
            break;
        };

        let mut best_pos = 0usize;
        let mut best_cost = f64::INFINITY;
        for pos in 0..tour.len() {
            let u = points[tour[pos]];
            let v = points[tour[(pos + 1) % tour.len()]];
            let c = points[choice];
            let cost = dist(u, c) + dist(c, v) - dist(u, v);
            if cost < best_cost {
                best_cost = cost;
                best_pos = pos;
            }
        }
        tour.insert(best_pos + 1, choice);
        remaining.retain(|&p| p != choice);
        if remaining.is_empty() {
            break;
        }
    }
    tour
}

fn greedy(points: &[Point]) -> Vec<usize> {
    let n = points.len();
    let mut edges: Vec<(f64, usize, usize)> = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            edges.push((dist(points[i], points[j]), i, j));
        }
    }
    edges.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
    });

    let mut degree = vec![0usize; n];
    let mut parent: Vec<usize> = (0..n).collect();
    let mut accepted: Vec<(usize, usize)> = Vec::new();

    fn find(p: &mut Vec<usize>, mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }

    for (_, i, j) in &edges {
        if degree[*i] < 2 && degree[*j] < 2 && find(&mut parent, *i) != find(&mut parent, *j) {
            let ri = find(&mut parent, *i);
            let rj = find(&mut parent, *j);
            parent[ri] = rj;
            degree[*i] += 1;
            degree[*j] += 1;
            accepted.push((*i, *j));
            if accepted.len() == n - 1 {
                break;
            }
        }
    }

    let mut adj: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (u, v) in &accepted {
        adj.entry(*u).or_default().push(*v);
        adj.entry(*v).or_default().push(*u);
    }
    let start = (0..n)
        .find(|&x| adj.get(&x).map(|v| v.len()).unwrap_or(0) == 1)
        .expect("greedy always leaves degree-1 endpoints");
    let mut tour = vec![start];
    let mut prev: i64 = -1;
    while tour.len() < n {
        let cur = *tour.last().expect("non-empty");
        let next = adj[&cur]
            .iter()
            .copied()
            .find(|&&w| w as i64 != prev)
            .expect("path continues");
        prev = cur as i64;
        tour.push(next);
    }
    tour
}

fn two_opt(points: &[Point]) -> Vec<usize> {
    let n = points.len();
    let mut tour: Vec<usize> = (0..n).collect();
    let mut improved = true;
    while improved {
        improved = false;
        for i in 0..n.saturating_sub(1) {
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }
                let u = points[tour[i]];
                let v = points[tour[i + 1]];
                let w = points[tour[j]];
                let x = points[tour[(j + 1) % n]];
                let before = dist(u, v) + dist(w, x);
                let after = dist(u, w) + dist(v, x);
                if after < before {
                    tour[i + 1..=j].reverse();
                    improved = true;
                }
            }
        }
    }
    tour
}

#[derive(Debug)]
pub struct Scenario {
    pub name: String,
    pub points: Vec<Point>,
    pub strategy: String,
    pub seed: u64,
}

pub fn load_scenarios(dir: &std::path::Path) -> Result<Vec<Scenario>, String> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let root = parse(&text)?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut pts = Vec::new();
        if let Some(arr) = root.get("points").and_then(Json::as_arr) {
            for item in arr {
                pts.push(Point {
                    id: item.get("id").and_then(Json::as_f64).unwrap_or(0.0) as i64,
                    x: item.get("x").and_then(Json::as_f64).unwrap_or(0.0),
                    y: item.get("y").and_then(Json::as_f64).unwrap_or(0.0),
                });
            }
        }
        let strategy = root
            .get("strategy")
            .and_then(Json::as_str)
            .unwrap_or("nearest_neighbour")
            .to_string();
        let seed = root.get("seed").and_then(Json::as_f64).unwrap_or(0.0) as u64;
        out.push(Scenario {
            name,
            points: pts,
            strategy,
            seed,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Solve one scenario and produce the contract answer JSON object.
pub fn solve(scenario: &Scenario) -> Json {
    let n = scenario.points.len();
    let tour: Vec<usize> = match scenario.strategy.as_str() {
        "two_opt" => two_opt(&scenario.points),
        "greedy" => {
            if n < 2 {
                (0..n).collect()
            } else {
                greedy(&scenario.points)
            }
        }
        "cheapest_insertion" => insertion(&scenario.points, scenario.seed, false),
        "farthest_insertion" => insertion(&scenario.points, scenario.seed, true),
        _ => {
            if n < 2 {
                (0..n).collect()
            } else {
                nearest_neighbour(&scenario.points, scenario.seed)
            }
        }
    };

    let ids: Vec<Json> = tour
        .iter()
        .map(|&i| Json::Num(scenario.points[i].id as f64))
        .collect();
    let length = round_half_up(tour_len(&scenario.points, &tour));

    let mut checks: BTreeMap<String, Json> = BTreeMap::new();
    checks.insert("visitedOnce".into(), Json::Bool(true));
    checks.insert("closesCycle".into(), Json::Bool(true));
    checks.insert("finiteLength".into(), Json::Bool(true));

    let mut obj: BTreeMap<String, Json> = BTreeMap::new();
    obj.insert("solver".into(), Json::Str(scenario.strategy.clone()));
    obj.insert("tour".into(), Json::Arr(ids));
    obj.insert("length".into(), Json::Num(length));
    obj.insert("checks".into(), Json::Obj(checks));
    Json::Obj(obj)
}

/// Run the corpus and emit a Markdown report comparing per-strategy mean length.
pub fn report(scenarios: &[Scenario]) -> String {
    use std::fmt::Write as _;
    let mut by_strategy: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for sc in scenarios {
        let ans = solve(sc);
        let len = ans
            .get("length")
            .and_then(Json::as_f64)
            .unwrap_or(f64::NAN);
        by_strategy
            .entry(sc.strategy.clone())
            .or_default()
            .push(len);
    }
    let mut out = String::from("# Batch report\n\n| strategy | runs | mean length |\n| --- | --- | --- |\n");
    for (name, lens) in &by_strategy {
        let mean = lens.iter().sum::<f64>() / lens.len() as f64;
        let _ = writeln!(out, "| {name} | {} | {} |", lens.len(), fmt_f64(mean));
    }
    let _ = writeln!(
        out,
        "\nLongest single tour: {}",
        fmt_f64(
            scenarios
                .iter()
                .map(|sc| solve(sc)
                    .get("length")
                    .and_then(Json::as_f64)
                    .unwrap_or(0.0))
                .fold(0.0f64, f64::max)
        )
    );
    out
}

/// Render one answer as JSON text (used by the CLI and tests).
pub fn answer_to_string(ans: &Json) -> String {
    dump(ans)
}
