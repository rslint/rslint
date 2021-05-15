// Typescript script to run and compare the benchmarks for eslint and rslint.

import {
  bench,
  BenchmarkTimer,
  runBenchmarks,
} from "https://deno.land/std@0.96.0/testing/bench.ts";

// number of times that each project should linted to prevent variance.
const RUN_COUNT = 1;

// A list of projects that should be linted.
const PROJECTS = ["projects/oak", "projects/engine262"];

PROJECTS.forEach((project) => {
  bench({
    name: `rslint ${project}`,
    runs: RUN_COUNT,
    async func(b: BenchmarkTimer): Promise<void> {
      b.start();
      const proc = Deno.run({
        cmd: ["../target/release/rslint", project],
        cwd: "./benchmarks",
        stdout: "null",
        stderr: "null",
      });

      await proc.status();
      b.stop();
    },
  });

  bench({
    name: `eslint ${project}`,
    runs: RUN_COUNT,
    async func(b: BenchmarkTimer): Promise<void> {
      b.start();
      const proc = Deno.run({
        cmd: [
          "npm",
          "run",
          "eslint",
          "--",
          "--no-eslintrc",
          "--ext",
          ".js",
          "--ext",
          ".ts",
          project,
        ],
        cwd: "./benchmarks",
        stdout: "null",
        stderr: "null",
      });

      await proc.status();
      b.stop();
    },
  });
});

if (Deno.args.includes("--json")) {
  const data = await runBenchmarks({ silent: true });
  console.log(JSON.stringify(data.results));
} else {
  const data = await runBenchmarks();
  console.log(data);
}
