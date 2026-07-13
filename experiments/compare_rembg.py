"""Compare current background removal model (U2Net/ISNet family) vs BiRefNet-lite.

Outputs cutouts side by side into outputs/rembg/ and prints per-image timing.
"""

import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
from PIL import Image

ROOT = Path(__file__).parent
CURRENT = ROOT / "models/current/background_removal.onnx"
BIREFNET = ROOT / "models/candidates/birefnet-general-lite.onnx"
IMAGES = sorted((ROOT / "images").glob("*.jpg"))
OUT = ROOT / "outputs/rembg"
OUT.mkdir(parents=True, exist_ok=True)

SIZE = 1024


def preprocess(img: Image.Image, mean: tuple, std: tuple) -> np.ndarray:
    resized = img.convert("RGB").resize((SIZE, SIZE), Image.LANCZOS)
    arr = np.asarray(resized, dtype=np.float32) / 255.0
    arr = (arr - np.array(mean, dtype=np.float32)) / np.array(std, dtype=np.float32)
    return arr.transpose(2, 0, 1)[None]


def mask_to_cutout(img: Image.Image, mask: np.ndarray, sigmoid: bool) -> Image.Image:
    mask = np.squeeze(mask)
    if sigmoid:
        mask = 1.0 / (1.0 + np.exp(-mask))
    else:
        mn, mx = mask.min(), mask.max()
        mask = (mask - mn) / (mx - mn + 1e-8)
    alpha = Image.fromarray((mask * 255).astype(np.uint8)).resize(img.size, Image.LANCZOS)
    out = img.convert("RGBA")
    out.putalpha(alpha)
    return out


def run(name: str, model_path: Path, mean: tuple, std: tuple, sigmoid: bool = False) -> None:
    sess = ort.InferenceSession(str(model_path), providers=["CPUExecutionProvider"])
    input_name = sess.get_inputs()[0].name
    total = 0.0
    for img_path in IMAGES:
        img = Image.open(img_path)
        tensor = preprocess(img, mean, std)
        t0 = time.perf_counter()
        outputs = sess.run(None, {input_name: tensor})
        dt = time.perf_counter() - t0
        total += dt
        cutout = mask_to_cutout(img, outputs[0], sigmoid)
        cutout.save(OUT / f"{img_path.stem}.{name}.png")
        print(f"{name} {img_path.name}: {dt * 1000:.0f}ms")
    print(f"{name} avg: {total / len(IMAGES) * 1000:.0f}ms\n")


if __name__ == "__main__":
    # Current model: ISNet-style normalization (mean 0.5, std 1.0)
    run("current", CURRENT, (0.5, 0.5, 0.5), (1.0, 1.0, 1.0))
    # BiRefNet: ImageNet normalization, output is logits
    run("birefnet", BIREFNET, (0.485, 0.456, 0.406), (0.229, 0.224, 0.225), sigmoid=True)
