# Model Validation Results

Environment: macOS Apple Silicon, onnxruntime CPU EP, 1024x1024 input.

## Current models (models-v0.1.0, inspected 2026-07-13)

| File | Architecture (inferred) | Input | Notes |
|---|---|---|---|
| background_removal.onnx (84MB) | U2Net/ISNet family (stage1-6 + side outputs, 7 outputs) | fixed 1x3x1024x1024 f32 | opset 15 |
| face_restoration.onnx (324MB) | GFPGAN family (conv_body_down/up, StyleGAN-like Gemm blocks) | fixed 1x3x512x512 f32 | opset 11 |
| image_upscaling.onnx (34MB) | Real-ESRGAN family (RRDB: dense Conv+LeakyReLU+Concat) | dynamic NCHW f16 | opset 10, 2x |

## Background removal: current vs BiRefNet-general-lite

Candidate: `BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx` (214MB, from rembg releases, MIT).
Preprocessing: ImageNet mean/std, output is logits → sigmoid (min-max normalization produces garbage).

| Metric | current (ISNet) | BiRefNet-lite |
|---|---|---|
| Avg latency (CPU, 8 imgs) | 523ms | 2908ms (~5.6x slower) |
| Model size | 84MB | 214MB |
| Edge quality | good silhouette | finer: correctly detects see-through gaps between hair strands (girl-2 ear region) that current model fills as solid |

**Verdict**: BiRefNet-lite is a real quality upgrade on fine structures but 5.6x slower on CPU.
Recommendation: ship both — current model as fast default, BiRefNet as `--quality high`.
TODO: re-benchmark with CoreML EP before finalizing; test birefnet-general (full, swin_v1 large) if quality gap justifies.

## Erase (LaMa) — new operator

Candidate: `Carve/LaMa-ONNX lama_fp32.onnx` (~200MB, Apache 2.0).
IO: `image` f32 [b,3,512,512] + `mask` f32 [b,1,512,512] (binary, 1=erase) → output already in 0-255 range (no denormalization needed).

| Metric | Value |
|---|---|
| Latency (CPU, 512x512) | ~1270ms |
| Erase quality | person fully removed from car-1; road/trees inpainted naturally, no visible seams at thumbnail scale |

**Verdict**: works out of the box, quality good enough for v1 `erase` operator.
Notes for engineering:
- Model is fixed at 512x512 (dynamic batch only) — need tile/resize strategy for larger images; check IOPaint's crop-around-mask approach for full-res workflow.
- Mask must be binary (threshold 0.5); soft masks untested.
- 17k nodes (FFC/Einsum heavy) — verify ort (Rust) supports Einsum before committing.

## Segment (SAM 2.1) + click-to-erase pipeline — new operator

Candidate: `vietanhdev/segment-anything-2.1-onnx-models` sam2.1_hiera_tiny (encoder 104MB + decoder 16MB, Apache 2.0).
Pipeline: point prompt → SAM mask (best of 3 by iou) → dilate r=12 → LaMa inpaint.

| Metric | Value |
|---|---|
| SAM encode (CPU, 1024x1024) | ~585ms (once per image) |
| SAM decode (per click) | ~12ms |
| Full click-to-erase | ~1.8s end-to-end |
| Quality | lion (iou 0.96): precise mask, clean erase. Person (iou 0.55): worked with dilation |

**Verdict**: click-to-erase pipeline works end-to-end on CPU, this is the v1 killer feature.
Notes for engineering:
- Decoder is 12ms — cache image embedding, support multiple clicks/refinement cheaply.
- Point accuracy matters (missed click → tiny garbage mask); agent callers should use `iou_predictions` + mask pixel count as confidence check, expose both in --json output.
- Mask dilation before LaMa is required (avoids halo/leftover edges).
- Encoder emits a benign shape-merge warning in ort; verify on Rust ort too.
