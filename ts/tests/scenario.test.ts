import { describe, expect, it } from "vitest";

import {
  checkTour,
  distance,
  parseScenario,
  roundHalfUp,
  tourLength,
  type ScenarioPoint,
} from "../src/scenario.js";

const unit: ScenarioPoint[] = [
  { id: 0, x: 0, y: 0 },
  { id: 1, x: 1, y: 0 },
  { id: 2, x: 1, y: 1 },
  { id: 3, x: 0, y: 1 },
];

describe("geometry contract", () => {
  it("computes the 3-4-5 distance", () => {
    const a: ScenarioPoint = { id: 0, x: 0, y: 0 };
    const b: ScenarioPoint = { id: 1, x: 3, y: 4 };
    expect(distance(a, b)).toBeCloseTo(5, 12);
  });

  it("closes the cycle when measuring a full loop", () => {
    // unit square looped 0 -> 1 -> 2 -> 3 -> 0 is exactly 4.0
    expect(tourLength(unit, [0, 1, 2, 3])).toBeCloseTo(4, 12);
    // open path 0 -> 1 -> 2 -> 3 is 3.0, so close must add the last leg
    expect(tourLength(unit, [0, 1, 2, 3])).toBeGreaterThan(3);
  });

  it("rounds halves away from zero like the other ports", () => {
    expect(roundHalfUp(2.675)).toBe(2.68);
    expect(roundHalfUp(0.125)).toBe(0.13);
    expect(roundHalfUp(-0.125)).toBe(-0.13);
  });
});

describe("contract checks", () => {
  it("accepts a Hamiltonian tour", () => {
    expect(checkTour(unit, [0, 1, 2, 3])).toEqual({
      visitedOnce: true,
      closesCycle: true,
      finiteLength: true,
    });
  });

  it("rejects tours with repeats or omissions", () => {
    const checks = checkTour(unit, [0, 1, 2]);
    expect(checks.visitedOnce).toBe(false);
    expect(checks.closesCycle).toBe(false);
    expect(checks.finiteLength).toBe(true);
  });

  it("flags non-finite coordinates", () => {
    const bad: ScenarioPoint[] = [...unit, { id: 4, x: Number.NaN, y: 0 }];
    expect(checkTour(bad, [0, 1, 2, 3, 4]).finiteLength).toBe(false);
  });
});

describe("scenario parsing", () => {
  it("reads a full document with expectations", () => {
    const doc = {
      strategy: "greedy",
      seed: 5,
      points: [
        { id: 0, x: 0, y: 0 },
        { id: 1, x: 2, y: 0 },
      ],
      expect: { tour: [0, 1], length: 5 },
    };
    const sc = parseScenario("two-point", doc);
    expect(sc.strategy).toBe("greedy");
    expect(sc.seed).toBe(5);
    expect(sc.points).toHaveLength(2);
    expect(sc.expect.tour).toEqual([0, 1]);
  });

  it("rejects documents without points", () => {
    expect(() => parseScenario("broken", {})).toThrow(/points/);
  });
});
