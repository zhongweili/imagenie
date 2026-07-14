# ADR 0002: Operator-specific ONNX Runtime defaults

- Status: Accepted
- Date: 2026-07-14

## Context

The first M3 Max CPU baseline identified two dominant risks on a 1000x667
input: `rembg-high` peaked at 11.03 GiB RSS and `upscale` had a 19.22 second
warm p50. All model sessions used the same CPU configuration: four intra-op
threads with ONNX Runtime memory-pattern optimization enabled.

The benchmark runner now accepts execution provider, intra-op thread count, and
memory-pattern settings, so candidates can be measured without changing the
agent CLI contract.

## Decision

- Keep CPU as the production execution provider. CoreML experiments did not
  improve either target by the required 20% and may fall back for unsupported
  graph regions.
- Disable ONNX Runtime memory-pattern optimization for BiRefNet only.
- Select up to 12 logical CPUs for the convolution-heavy upscaler. Machines
  with fewer CPUs use the available count.
- Keep the existing CPU/four-thread/memory-pattern defaults for other models
  until operator-specific evidence justifies a change.

The CLI and desktop explicitly select these product defaults. The benchmark
binary can override them to preserve baseline/candidate comparisons.

## Evidence

Isolated release-mode experiments on the same M3 Max and input produced:

| Candidate | Baseline | Candidate result | Change |
| --- | ---: | ---: | ---: |
| BiRefNet peak RSS, memory pattern off | 11.03 GiB | 6.68 GiB | -39.5% |
| BiRefNet warm p50 | 3,714 ms | 3,780 ms | +1.8% |
| Upscale warm p50, 12 threads | 19,216 ms | 7,821 ms | -59.3% |
| Upscale peak RSS | 3.62 GiB | 3.61 GiB | effectively unchanged |

Eight upscale threads reached roughly 10.3 seconds; 16 threads regressed to
roughly 9.2 seconds, so 12 was the best tested setting. For both accepted
changes, baseline and candidate PNG files had identical SHA-256 hashes.

## Consequences

- `rembg --quality high` becomes viable on more 16 GB machines, though its
  roughly 6.7 GiB peak remains a release risk and needs further work.
- Upscale consumes more CPU cores per inference. Batch processing remains
  serialized at the model session, avoiding multiple simultaneous 12-thread
  inference runs in one process.
- CoreML remains available to the benchmark harness for future ONNX Runtime or
  model revisions, but is not presented as a supported product provider yet.
