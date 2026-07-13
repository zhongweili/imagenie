"""Full click-to-erase pipeline: SAM 2.1 point prompt -> mask -> dilate -> LaMa inpaint.

Usage: uv run test_segment_erase.py <image> <x,y in relative 0-1 coords>
"""

import sys
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
from PIL import Image, ImageDraw

ROOT = Path(__file__).parent
SAM_DIR = ROOT / "models/candidates/sam2.1_tiny"
LAMA = ROOT / "models/candidates/lama_fp32.onnx"
OUT = ROOT / "outputs/segment_erase"
OUT.mkdir(parents=True, exist_ok=True)

SAM_SIZE = 1024
LAMA_SIZE = 512


def sam_segment(img: Image.Image, point: tuple[float, float]) -> tuple[np.ndarray, float, dict]:
    timings = {}
    resized = img.resize((SAM_SIZE, SAM_SIZE), Image.LANCZOS)
    arr = np.asarray(resized, dtype=np.float32) / 255.0
    arr = (arr - np.array([0.485, 0.456, 0.406], dtype=np.float32)) / np.array(
        [0.229, 0.224, 0.225], dtype=np.float32
    )
    tensor = arr.transpose(2, 0, 1)[None]

    encoder = ort.InferenceSession(str(SAM_DIR / "sam2.1_hiera_tiny.encoder.onnx"), providers=["CPUExecutionProvider"])
    t0 = time.perf_counter()
    embed_hi0, embed_hi1, image_embed = encoder.run(None, {"image": tensor})
    timings["encode_ms"] = (time.perf_counter() - t0) * 1000

    decoder = ort.InferenceSession(str(SAM_DIR / "sam2.1_hiera_tiny.decoder.onnx"), providers=["CPUExecutionProvider"])
    coords = np.array([[[point[0] * SAM_SIZE, point[1] * SAM_SIZE]]], dtype=np.float32)
    labels = np.array([[1]], dtype=np.float32)
    t0 = time.perf_counter()
    masks, iou = decoder.run(
        None,
        {
            "image_embed": image_embed,
            "high_res_feats_0": embed_hi0,
            "high_res_feats_1": embed_hi1,
            "point_coords": coords,
            "point_labels": labels,
            "mask_input": np.zeros((1, 1, 256, 256), dtype=np.float32),
            "has_mask_input": np.zeros((1,), dtype=np.float32),
        },
    )
    timings["decode_ms"] = (time.perf_counter() - t0) * 1000

    best = int(np.argmax(iou[0]))
    mask = (masks[0, best] > 0).astype(np.uint8)
    return mask, float(iou[0, best]), timings


def dilate(mask: np.ndarray, r: int) -> np.ndarray:
    """Poor-man's dilation via PIL MaxFilter."""
    from PIL import ImageFilter

    m = Image.fromarray(mask * 255)
    return np.asarray(m.filter(ImageFilter.MaxFilter(2 * r + 1))) > 127


def lama_erase(img: Image.Image, mask: np.ndarray) -> tuple[Image.Image, float]:
    resized = img.resize((LAMA_SIZE, LAMA_SIZE), Image.LANCZOS)
    mask_img = Image.fromarray((mask * 255).astype(np.uint8)).resize((LAMA_SIZE, LAMA_SIZE), Image.NEAREST)

    img_arr = np.asarray(resized, dtype=np.float32).transpose(2, 0, 1)[None] / 255.0
    mask_arr = (np.asarray(mask_img, dtype=np.float32)[None, None] / 255.0 > 0.5).astype(np.float32)

    sess = ort.InferenceSession(str(LAMA), providers=["CPUExecutionProvider"])
    t0 = time.perf_counter()
    (output,) = sess.run(None, {"image": img_arr, "mask": mask_arr})
    dt = (time.perf_counter() - t0) * 1000
    return Image.fromarray(np.clip(output[0].transpose(1, 2, 0), 0, 255).astype(np.uint8)), dt


def run(image_path: str, point: tuple[float, float]) -> None:
    img = Image.open(image_path).convert("RGB")
    stem = Path(image_path).stem

    mask, iou, timings = sam_segment(img, point)
    print(f"SAM: encode {timings['encode_ms']:.0f}ms, decode {timings['decode_ms']:.0f}ms, best iou {iou:.3f}, mask px {mask.sum()}")

    mask_dilated = dilate(mask.astype(np.uint8), r=12).astype(np.uint8)
    result, lama_ms = lama_erase(img, mask_dilated)
    print(f"LaMa: {lama_ms:.0f}ms")

    s = LAMA_SIZE
    panel = Image.new("RGB", (s * 3, s))
    marked = img.resize((s, s), Image.LANCZOS)
    d = ImageDraw.Draw(marked)
    px, py = point[0] * s, point[1] * s
    d.ellipse([px - 6, py - 6, px + 6, py + 6], fill=(255, 0, 0))
    panel.paste(marked, (0, 0))
    mask_vis = Image.fromarray((mask_dilated * 255).astype(np.uint8)).resize((s, s), Image.NEAREST)
    panel.paste(Image.merge("RGB", [mask_vis] * 3), (s, 0))
    panel.paste(result, (s * 2, 0))
    panel.save(OUT / f"{stem}.click_erase.png")
    result.save(OUT / f"{stem}.result.png")


if __name__ == "__main__":
    image = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "images/car-1.jpg")
    coords = sys.argv[2] if len(sys.argv) > 2 else "0.8,0.5"
    run(image, tuple(float(v) for v in coords.split(",")))
