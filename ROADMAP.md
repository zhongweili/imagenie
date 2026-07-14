# Imagenie Roadmap

Imagenie is a local image-processing execution engine for agents. The project
optimizes for output quality, predictable performance, stable machine
interfaces, and privacy by default.

## v0.1 — Reliable agent CLI

Exit criteria:

- The five operators (`upscale`, `rembg`, `restore`, `segment`, and `erase`)
  expose a versioned JSONL contract.
- At least 95% of the representative agent task suite completes in one call;
  at least 99% of valid invocations return parseable machine output without a
  panic.
- Every operator has contract tests, quality fixtures, and cold/warm latency
  and peak-memory baselines on the reference machines.
- macOS, Linux, and Windows binaries are published with verified model
  artifacts and documented installation instructions.

## v0.2 — Quality and performance

Exit criteria:

- Model changes are gated by task-specific quality reports and license review.
- Default models do not regress more than 2% on primary quality metrics.
- Targeted operators improve warm latency or batch throughput by at least 20%
  over the v0.1 baseline without a quality regression.
- Hardware execution-provider choices are documented and selected from
  reproducible benchmarks.

## v0.3 — Agent ecosystem

Exit criteria:

- A thin MCP adapter maps the stable CLI schemas without duplicating inference
  logic.
- Published examples cover shell agents, Codex, and Claude Code.
- Agent task success and recovery behavior are evaluated end to end.

## Current sequence

1. Freeze the v0.1 Agent Contract and evaluation harness.
2. Converge CLI and desktop processing through one `imagenie-core` API.
3. Establish and improve model quality and performance baselines.
4. Add cargo-dist, release automation, Homebrew, and agent-first documentation.
5. Build ecosystem adapters only after the CLI contract is stable.

New operators are out of scope until v0.1 exits.
