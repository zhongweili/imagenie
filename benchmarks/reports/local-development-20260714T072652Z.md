# Benchmark: local-development

Generated: `2026-07-14T07:26:52+00:00`

Dataset status: `local_only_unverified_license`; quality gate: `false`.

This report measures performance only. Output quality was not evaluated.

| Case | Operator | Load ms | Cold ms | Warm p50 ms | Warm p95 ms | Peak RSS MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| rembg-fast-animal | rembg | 91.0 | 1023.2 | 993.7 | 994.2 | 1706.9 |
| rembg-high-animal | rembg-high | 993.5 | 4246.2 | 3762.1 | 3768.4 | 6823.4 |
| upscale-animal | upscale | 76.9 | 7841.0 | 7840.9 | 8065.8 | 3701.1 |
| restore-face | restore | 180.2 | 1247.6 | 1213.3 | 1218.2 | 1446.3 |
| segment-animal | segment | 571.1 | 1029.3 | 25.6 | 26.2 | 1698.6 |
| click-erase-animal | click-erase | 2803.8 | 2973.0 | 1993.1 | 2032.1 | 2354.2 |

## Environment

- OS/architecture: `macos/aarch64`
- CPU: `Apple M3 Max`
- Logical CPUs: `16`
- Execution provider: `cpu`
- Git commit: `275e0e7c9dd1b82bae78325487eca0acb44b227f`
- Dirty worktree during run: `false`
