/**
 * Scenario loading and contract checking utilities.
 *
 * A scenario file declares a point cloud, a solver strategy and optional
 * seed; the expected answer is embedded for the verifier to compare against
 * the port implementations.
 */

export interface ScenarioPoint {
  readonly id: number;
  readonly x: number;
  readonly y: number;
}

export interface ExpectedAnswer {
  readonly tour: readonly number[];
  readonly length: number;
}

export interface Scenario {
  readonly name: string;
  readonly strategy: string;
  readonly seed: number;
  readonly points: readonly ScenarioPoint[];
  readonly expect: ExpectedAnswer;
}

export interface ContractChecks {
  readonly visitedOnce: boolean;
  readonly closesCycle: boolean;
  readonly finiteLength: boolean;
}

/** Distance identical to the Python/Rust ports (`math.hypot`). */
export function distance(a: ScenarioPoint, b: ScenarioPoint): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.hypot(dx, dy);
}

/** Closed-tour length over the scenario points, following id order. */
export function tourLength(
  points: readonly ScenarioPoint[],
  tour: readonly number[],
): number {
  if (tour.length === 0) return 0;
  const byId = new Map(points.map((p) => [p.id, p]));
  let total = 0;
  for (let i = 0; i < tour.length; i++) {
    const cur = byId.get(tour[i]!);
    const nxt = byId.get(tour[(i + 1) % tour.length]!);
    if (cur === undefined || nxt === undefined) {
      throw new Error(`tour references unknown id at index ${i}`);
    }
    total += distance(cur, nxt);
  }
  return total;
}

/** Round half away from zero to 2 decimals (contract rounding). */
export function roundHalfUp(value: number): number {
  const scaled = value * 100;
  const rounded = scaled >= 0 ? Math.floor(scaled + 0.5) : Math.ceil(scaled - 0.5);
  return rounded / 100;
}

/** Contract booleans, mirroring `routeopt/verify.py`. */
export function checkTour(
  points: readonly ScenarioPoint[],
  tour: readonly number[],
): ContractChecks {
  const ids = points.map((p) => p.id);
  const visitedOnce =
    tour.length === ids.length && new Set(tour).size === ids.length;
  const closesCycle =
    visitedOnce && [...tour].sort((a, b) => a - b).every((v, i) => v === ids.sort((a, b) => a - b)[i]);
  const finiteLength = points.every(
    (p) => Number.isFinite(p.x) && Number.isFinite(p.y),
  );
  return { visitedOnce, closesCycle, finiteLength };
}

/** Parse a scenario JSON document, validating the essential fields. */
export function parseScenario(name: string, raw: unknown): Scenario {
  const obj = raw as Record<string, unknown>;
  const pointsRaw = obj["points"];
  if (!Array.isArray(pointsRaw)) {
    throw new Error(`${name}: "points" must be an array`);
  }
  const points: ScenarioPoint[] = pointsRaw.map((p, i) => {
    const rec = p as Record<string, unknown>;
    return {
      id: Number(rec["id"] ?? i),
      x: Number(rec["x"]),
      y: Number(rec["y"]),
    };
  });
  const expectRaw = (obj["expect"] ?? {}) as Record<string, unknown>;
  const tourRaw = Array.isArray(expectRaw["tour"]) ? expectRaw["tour"] : [];
  return {
    name,
    strategy: typeof obj["strategy"] === "string" ? obj["strategy"] : "nearest_neighbour",
    seed: Number(obj["seed"] ?? 0),
    points,
    expect: {
      tour: tourRaw.map(Number),
      length: Number(expectRaw["length"] ?? 0),
    },
  };
}
