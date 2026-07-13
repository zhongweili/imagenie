"""Validate LaMa inpainting: erase an object given a rectangle mask.

Usage: uv run test_lama.py <image> <x1,y1,x2,y2 in relative 0-1 coords>
"""

import sys
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
from PIL import Image, ImageDraw

ROOT = Path(__file__).parent
MODEL = ROOT / "models/candidates/lama_fp32.onnx"
OUT = ROOT / "outputs/erase"
OUT.mkdir(parents=True, exist_ok=True)

SIZE = 512


def run(image_path: str, rect: tuple[float, float, float, float]) -> None:
    img = Image.open(image_path).convert("RGB")
    resized = img.resize((SIZE, SIZE), Image.LANCZOS)

    mask = Image.new("L", (SIZE, SIZE), 0)
    draw = ImageDraw.Draw(mask)
    x1, y1, x2, y2 = (int(v * SIZE) for v in rect)
    draw.rectangle([x1, y1, x2, y2], fill=255)

    img_arr = np.asarray(resized, dtype=np.float32).transpose(2, 0, 1)[None] / 255.0
    mask_arr = np.asarray(mask, dtype=np.float32)[None, None] / 255.0
    mask_arr = (mask_arr > 0.5).astype(np.float32)

    sess = ort.InferenceSession(str(MODEL), providers=["CPUExecutionProvider"])
    t0 = time.perf_counter()
    (output,) = sess.run(None, {"image": img_arr, "mask": mask_arr})
    dt = time.perf_counter() - t0

    out_arr = output[0].transpose(1, 2, 0)
    # Carve export outputs 0-255 range already; clamp and check
    if out_arr.max() <= 1.5:
        out_arr = out_arr * 255.0
    result = Image.fromarray(np.clip(out_arr, 0, 255).astype(np.uint8))

    stem = Path(image_path).stem
    side = Image.new("RGB", (SIZE * 3, SIZE))
    marked = resized.copy()
    ImageDraw.Draw(marked).rectangle([x1, y1, x2, y2], outline=(255, 0, 0), width=3)
    side.paste(marked, (0, 0))
    side.paste(Image.merge("RGB", [mask] * 3), (SIZE, 0))
    side.paste(result, (SIZE * 2, 0))
    side.save(OUT / f"{stem}.lama.png")
    print(f"{stem}: {dt * 1000:.0f}ms, output range [{output.min():.2f}, {output.max():.2f}]")


if __name__ == "__main__":
    image = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "images/car-1.jpg")
    coords = sys.argv[2] if len(sys.argv) > 2 else "0.25,0.4,0.75,0.85"
    run(image, tuple(float(v) for v in coords.split(",")))
