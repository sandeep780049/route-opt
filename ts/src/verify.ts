/**
 * Cross-language contract verifier.
 *
 * Usage: tsx src/verify.ts [--python <cmd>] [--rust <bin>] [--cpp <bin>] [--corpus <dir>]
 *
 * Rules (see docs/contract.md):
 *  - every port must emit a valid Hamiltonian cycle whose stated `length`
 *    equals the length recomputed from its tour ids;
 *  - deterministic strategies (`two_opt`, `greedy`) must reproduce the
 *    embedded `expect` answer exactly (same tour ids, same length);
 *  - seeded strategies must agree across the SplitMix64 ports (rust, cpp, ts,
 *    java); Python uses its own numpy stream and is validated for correctness
 *    only.
 */

import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import {
  checkTour,
  parseScenario,
  roundHalfUp,
  tourLength,
  type Scenario,
} from "./scenario.js";

interface PortSpec {
  readonly name: string;
  readonly command: string;
  readonly args: readonly string[];
}

interface PortAnswer {
  readonly tour: number[];
  readonly length: number;
}

const DETERMINISTIC = new Set(["two_opt", "greedy"]);

function parseArgs(argv: readonly string[]): Map<string, string> {
  const opts = new Map<string, string>();
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg?.startsWith("--") && i + 1 < argv.length) {
      opts.set(arg.slice(2), argv[i + 1] ?? "");
      i++;
    }
  }
  return opts;
}

function listCorpus(corpusDir: string): Scenario[] {
  const entries = readdirSync(corpusDir).filter((f) => f.endsWith(".json")).sort();
  return entries.map((file) => {
    const raw = JSON.parse(readFileSync(join(corpusDir, file), "utf8")) as unknown;
    return parseScenario(file.replace(/\.json$/, ""), raw);
  });
}

function runPort(port: PortSpec, scenarioPath: string): string {
  const proc = spawnSync(port.command, [...port.args, scenarioPath], {
    encoding: "utf8",
    timeout: 60_000,
  });
  if (proc.status !== 0 || proc.error) {
    throw new Error(`port ${port.name} failed: ${String(proc.stderr).slice(0, 400)}`);
  }
  return proc.stdout;
}

function extractAnswer(stdout: string): PortAnswer {
  const start = stdout.indexOf("{");
  const end = stdout.lastIndexOf("}");
  if (start === -1 || end <= start) {
    throw new Error(`no JSON object in output: ${stdout.slice(0, 120)}`);
  }
  const parsed = JSON.parse(stdout.slice(start, end + 1)) as {
    tour?: unknown;
    length?: unknown;
  };
  if (!Array.isArray(parsed.tour)) {
    throw new Error("answer has no tour array");
  }
  return {
    tour: parsed.tour.map(Number),
    length: Number(parsed.length ?? NaN),
  };
}

function toursEqual(a: readonly number[], b: readonly number[]): boolean {
  return a.length === b.length && a.every((v, i) => v === b[i]);
}

function validAnswer(scenario: Scenario, answer: PortAnswer): string | null {
  const checks = checkTour(scenario.points, answer.tour);
  if (!checks.visitedOnce || !checks.closesCycle) {
    return "not a Hamiltonian cycle";
  }
  const recomputed = roundHalfUp(tourLength(scenario.points, answer.tour));
  if (Math.abs(recomputed - answer.length) >= 1e-9) {
    return `length ${answer.length} != recomputed ${recomputed}`;
  }
  return null;
}

function main(): number {
  const opts = parseArgs(process.argv.slice(2));
  const corpusDir = opts.get("corpus") ?? "examples";
  const scenarios = listCorpus(corpusDir);
  if (scenarios.length === 0) {
    console.error(`no scenarios in ${corpusDir}`);
    return 2;
  }

  const ports: PortSpec[] = [];
  const pythonBin = opts.get("python");
  if (pythonBin) {
    ports.push({
      name: "python",
      command: pythonBin,
      args: ["-m", "routeopt.cli"],
    });
  }
  const cppBin = opts.get("cpp");
  if (cppBin) {
    ports.push({ name: "cpp", command: cppBin, args: [] });
  }
  const rustBin = opts.get("rust");
  if (rustBin) {
    ports.push({ name: "rust", command: rustBin, args: [] });
  }
  if (ports.length === 0) {
    console.error("no ports requested (pass --python and/or --cpp/--rust)");
    return 2;
  }

  const rows: string[] = [];
  let failures = 0;
  const fail = (scenario: string, port: string, detail: string) => {
    failures++;
    rows.push(`FAIL  ${scenario.padEnd(24)}  ${port.padEnd(8)}  ${detail}`);
  };
  const pass = (scenario: string, port: string, detail: string) => {
    rows.push(`PASS  ${scenario.padEnd(24)}  ${port.padEnd(8)}  ${detail}`);
  };

  for (const scenario of scenarios) {
    const scenarioPath = join(corpusDir, `${scenario.name}.json`);

    // collect answers, reporting hard failures (port crashed / bad JSON)
    const answers = new Map<string, PortAnswer>();
    for (const port of ports) {
      try {
        answers.set(port.name, extractAnswer(runPort(port, scenarioPath)));
      } catch (err) {
        fail(scenario.name, port.name, err instanceof Error ? err.message : String(err));
      }
    }

    // rule 1: every answer is a valid, self-consistent Hamiltonian cycle
    for (const port of ports) {
      const answer = answers.get(port.name);
      if (!answer) continue;
      const problem = validAnswer(scenario, answer);
      if (problem) {
        fail(scenario.name, port.name, problem);
        continue;
      }

      // rule 2: deterministic strategies must reproduce the embedded answer
      if (DETERMINISTIC.has(scenario.strategy)) {
        const tourEq = toursEqual(answer.tour, scenario.expect.tour);
        const lenEq = Math.abs(answer.length - scenario.expect.length) < 1e-9;
        if (tourEq && lenEq) {
          pass(scenario.name, port.name, `matches expectation (${answer.length})`);
        } else {
          fail(
            scenario.name,
            port.name,
            `tour/length diverge from expectation (${scenario.expect.length})`,
          );
        }
        continue;
      }

      // rule 3 (final verdict below): seeded strategies must agree across the
      // SplitMix64 ports; python is validated above but uses its own stream
      if (port.name === "python") {
        pass(scenario.name, port.name, `reference stream, len=${answer.length}`);
      } else {
        pass(scenario.name, port.name, `len=${answer.length}`);
      }
    }

    // rule 3: cross-port agreement for seeded strategies (non-python ports)
    if (!DETERMINISTIC.has(scenario.strategy)) {
      const splitMixPorts = ports.filter(
        (p) => p.name !== "python" && answers.has(p.name),
      );
      if (splitMixPorts.length >= 2) {
        const [anchor, ...others] = splitMixPorts;
        const anchorAnswer = answers.get(anchor!.name)!;
        for (const p of others!) {
          const ans = answers.get(p!.name)!;
          if (
            !toursEqual(ans.tour, anchorAnswer.tour) ||
            Math.abs(ans.length - anchorAnswer.length) >= 1e-9
          ) {
            fail(
              scenario.name,
              p!.name,
              `disagrees with ${anchor!.name} (len ${ans.length} vs ${anchorAnswer.length})`,
            );
          }
        }
      }
    }
  }

  console.log(rows.join("\n"));
  console.log(
    failures === 0
      ? `\nall ${scenarios.length} scenarios agreed across ${ports.length} port(s)`
      : `\n${failures} check(s) failed`,
  );
  return failures === 0 ? 0 : 1;
}

process.exit(main());
