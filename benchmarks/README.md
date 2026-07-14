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
