# Benchmark: local-development

Generated: `2026-07-14T07:05:44+00:00`

Dataset status: `local_only_unverified_license`; quality gate: `false`.

This report measures performance only. Output quality was not evaluated.

| Case | Operator | Load ms | Cold ms | Warm p50 ms | Warm p95 ms | Peak RSS MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| rembg-fast-animal | rembg | 79.8 | 1058.1 | 975.4 | 990.2 | 1757.4 |
| rembg-high-animal | rembg-high | 1359.6 | 4628.2 | 3713.8 | 3820.1 | 11295.5 |
| upscale-animal | upscale | 109.9 | 19552.8 | 19216.2 | 19975.2 | 3707.0 |
| restore-face | restore | 178.5 | 1250.8 | 1168.8 | 1204.6 | 1363.3 |
| segment-animal | segment | 652.4 | 1027.8 | 27.5 | 28.4 | 1792.5 |
| click-erase-animal | click-erase | 3163.4 | 2983.4 | 1978.5 | 2010.5 | 2554.7 |

## Environment

- OS/architecture: `macos/aarch64`
- CPU: `Apple M3 Max`
- Logical CPUs: `16`
- Execution provider: `cpu`
- Git commit: `686a2db06c66f62d376b3622ceeff2b6b2eca961`
- Dirty worktree during run: `false`
