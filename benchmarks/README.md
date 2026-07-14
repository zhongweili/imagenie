# Imagenie Benchmarks

Benchmarks gate model selection and performance work. They are not ad-hoc demo
scripts.

## Required measurements

Every report records cold latency, warm latency, peak RSS, batch throughput,
input dimensions, execution provider, hardware, Imagenie commit, and model
artifact SHA-256.

Task-specific quality gates:

| Operator | Primary metrics | Required qualitative review |
| --- | --- | --- |
| rembg | IoU, boundary F-score | hair, transparent and soft edges |
| upscale | PSNR, SSIM, LPIPS | texture and oversharpening artifacts |
| restore | identity similarity, face quality | non-face region preservation |
| segment | prompt IoU, success rate | empty and tiny mask failures |
| erase | boundary artifact score | pairwise realism preference |

Composite `segment -> erase` cases additionally track one-call task success and
total latency.

## Workflow

1. Add or update cases in `manifest.json`.
2. Run the pinned baseline and candidate with the same environment.
3. Store generated artifacts outside git; commit only the manifest and summary
   report.
4. Link the report from any PR that changes inference or model selection.

Real datasets and redistribute-restricted images must not be committed without
an explicit license record.

## Running the local performance baseline

The benchmark binary runs one operator in an isolated process and records model
load time, cold latency, warm samples, peak RSS, environment details, the git
revision, input SHA-256, and model artifact SHA-256. The suite runner builds the
release binary once and aggregates the case reports:

```sh
python3 benchmarks/run_suite.py benchmarks/suites/local-development.json
```

Reports are written to `benchmarks/reports/` as JSON and Markdown. Pass
`--output-prefix /path/to/name` to choose another location, or `--skip-build`
when the release binary is already current.

`local-development.json` intentionally references gitignored images under
`experiments/images/`. Those inputs have no recorded redistribution license, so
the suite is a reproducible performance baseline on the current workstation but
**is not a quality or release gate**. A quality gate may only be enabled after
each case has a committed source, license, annotation method, and expected
metric.
