# Contributing to Imagenie

Imagenie is developed outcome-first: changes should improve agent task success,
image quality, execution efficiency, or interface reliability.

## Planning

- Roadmap items describe measurable outcomes, not implementation tasks.
- A milestone should be independently releasable and normally fit within one
  to three weeks.
- An issue should normally fit within one or two focused development days.
- New operators are deferred until the v0.1 exit criteria in `ROADMAP.md` pass.

Recommended labels:

- Area: `area/core`, `area/cli`, `area/model`, `area/desktop`, `area/release`.
- Type: `type/quality`, `type/perf`, `type/bug`, `type/docs`.
- Priority: `P0`, `P1`, `P2`.
- Gate: `needs-benchmark`, `breaking-change`, `blocked`.

## Pull requests

Keep branches short-lived and make each pull request prove one outcome. Before
requesting review, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
npm run build
```

Inference or model changes must attach a reproducible benchmark report. Agent
contract changes must update the JSON Schema, contract tests, documentation,
and compatibility notes together.

## Model changes

Record the model source, license, payload SHA-256, size, supported platforms,
quality comparison, and performance comparison. Do not merge a model whose
license is unclear or whose artifact cannot be reproduced and verified.

## Architectural decisions

Use `docs/decisions/0000-template.md` for choices that constrain multiple
components, introduce compatibility commitments, or select a model/runtime
strategy that will be expensive to reverse.
