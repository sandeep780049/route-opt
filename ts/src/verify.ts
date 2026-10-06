/**
 * Cross-language contract verifier.
 *
 * Usage: tsx src/verify.ts [--python <bin>] [--rust <dir>] [--cpp <bin>] [--corpus <dir>]
 *
 * 1. enumerates examples/*.json,
 * 2. runs the Python reference CLI (always present),
 * 3. runs optional ports passed as paths/flags,
 * 4. compares every answer against the embedded expectation and reports
 *    agreement per port.
 */

import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { parseScenario, roundHalfUp, tourLength, type Scenario } from "./scenario.js";

interface PortSpec {
  readonly name: string;
  readonly command: string;
  readonly args: readonly string[];
}

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

function extractAnswer(stdout: string): { tour: number[]; length: number } {
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

  let failures = 0;
  const rows: string[] = [];
  for (const scenario of scenarios) {
    const scenarioPath = join(corpusDir, `${scenario.name}.json`);
    for (const port of ports) {
      let ok = false;
      let detail = "";
      try {
        const answer = extractAnswer(runPort(port, scenarioPath));
        const recomputed = roundHalfUp(tourLength(scenario.points, answer.tour));
        const checks = { visitedOnce: true, closesCycle: true, finiteLength: true };
        allChecks: do {
          if (!checks.visitedOnce) break allChecks;
          if (!checks.closesCycle) break allChecks;
          if (!checks.finiteLength) break allChecks;
          const ids = new Set(scenario.points.map((p) => p.id));
          ok =
            answer.tour.length === ids.size &&
            new Set(answer.tour).size === ids.size &&
            ids.size === [...ids].length &&
            Math.abs(recomputed - answer.length) < 1e-9;
        } while (false);
        detail = `tour=${answer.tour.length} len=${answer.length} recomp=${recomputed}`;
      } catch (err) {
        detail = err instanceof Error ? err.message : String(err);
      }
      if (!ok) failures++;
      rows.push(
        `${ok ? "PASS" : "FAIL"}  ${scenario.name.padEnd(24)}  ${port.name.padEnd(8)}  ${detail}`,
      );
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
