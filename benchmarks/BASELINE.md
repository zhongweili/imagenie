# v0.1 Performance Baseline

The first reproducible performance baseline was recorded on 2026-07-14 from
commit `686a2db` on an Apple M3 Max (16 logical CPUs, 64 GB RAM), macOS/aarch64,
using ONNX Runtime's CPU execution provider.

Source reports:

- [Human-readable report](reports/local-development-20260714T070544Z.md)
- [Machine-readable report](reports/local-development-20260714T070544Z.json)

The inputs are local, gitignored development images without recorded
redistribution licenses. The numbers are valid as a performance baseline for
this machine; output quality was not evaluated and this is not a release gate.

## Findings

| Operator | Warm p50 | Peak RSS | Decision |
| --- | ---: | ---: | --- |
| rembg | 975 ms | 1.72 GiB | Keep as the default; wait for quality fixtures before tuning. |
| rembg-high | 3,714 ms | 11.03 GiB | Highest memory risk; profile first. |
| upscale | 19,216 ms | 3.62 GiB | Highest latency risk; profile second. |
| restore | 1,169 ms | 1.33 GiB | Accept as the provisional baseline. |
| segment | 28 ms | 1.75 GiB | Embedding cache is effective; cold inference was 1,028 ms. |
| click-erase | 1,979 ms | 2.49 GiB | Optimize after the two larger risks. |

The SAM result is the strongest architectural signal: retaining the model and
input embedding reduces repeated point segmentation by roughly 97%. The same
measure-before-changing approach should be used for tiling, input resolution,
and execution-provider work.

## Next measured milestone

Phase 3A is complete when all of the following are true:

1. A redistributable fixture manifest records source, license, checksum,
   annotation method, and expected metric for every quality case.
2. The first quality set contains at least 10 images each for rembg, upscale,
   restore, and erase, plus at least 20 annotated prompts for segment and the
   segment-to-erase composite.
3. Profiling identifies the allocation and compute hot spots for `rembg-high`
   and `upscale`, including a CPU baseline and at least one viable Apple
   Silicon execution-provider or tiling candidate.
4. The first optimization candidate achieves at least a 20% warm-latency or
   memory reduction without regressing a primary quality metric by more than
   2%.

Initial candidate targets on this exact input and environment are:

- `rembg-high`: warm p50 at or below 2,971 ms; peak RSS below 8 GiB, with a
  longer-term 16 GB-machine suitability target below 6 GiB.
- `upscale`: warm p50 at or below 15,373 ms and peak RSS at or below 2.90 GiB.
- `click-erase`: warm p50 at or below 1,583 ms after the two primary risks are
  addressed.

Every performance PR must include the baseline and candidate reports from the
same commit state, input checksums, model checksums, and environment. A faster
result does not merge if quality is unmeasured or regresses beyond the gate.
