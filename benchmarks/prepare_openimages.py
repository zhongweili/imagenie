#!/usr/bin/env python3
"""Download curated Open Images assets and generate a checksum-pinned manifest."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import pathlib
import urllib.request
import zipfile
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
QUALITY = ROOT / "benchmarks" / "quality"
CACHE = QUALITY / ".cache"
ASSETS = QUALITY / "assets" / "open-images-v5"
METADATA_URL = (
    "https://storage.googleapis.com/openimages/2018_04/validation/"
    "validation-images-with-rotation.csv"
)
MASK_ARCHIVE = "https://storage.googleapis.com/openimages/v5/validation-masks/validation-masks-{shard}.zip"
IMAGE_MIRROR = "https://s3.amazonaws.com/open-images-dataset/validation/{image_id}.jpg"
EXPECTED_LICENSE = "https://creativecommons.org/licenses/by/2.0/"


def download(url: str, destination: pathlib.Path) -> pathlib.Path:
    if destination.is_file():
        return destination
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers={"User-Agent": "imagenie-quality-fixtures/1.0"})
    with urllib.request.urlopen(request) as response:
        destination.write_bytes(response.read())
    return destination


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def metadata() -> dict[str, dict[str, str]]:
    path = download(METADATA_URL, CACHE / "validation-images-with-rotation.csv")
    with path.open(newline="") as handle:
        return {row["ImageID"]: row for row in csv.DictReader(handle)}


def prepare_case(selection: dict[str, Any], rows: dict[str, dict[str, str]]) -> dict[str, Any]:
    image_id = selection["image_id"]
    row = rows.get(image_id)
    if row is None:
        raise ValueError(f"image {image_id} is missing from official validation metadata")
    if row["License"] != EXPECTED_LICENSE:
        raise ValueError(f"image {image_id} has unexpected license {row['License']!r}")
    for field in ("Author", "OriginalLandingURL"):
        if not row[field]:
            raise ValueError(f"image {image_id} is missing metadata field {field}")

    image_path = download(
        IMAGE_MIRROR.format(image_id=image_id),
        ASSETS / f"{image_id}.jpg",
    )
    mask_name = selection["mask_path"]
    if not mask_name.startswith(image_id) or pathlib.PurePath(mask_name).name != mask_name:
        raise ValueError(f"unsafe or mismatched mask path {mask_name!r}")
    archive_path = download(
        MASK_ARCHIVE.format(shard=image_id[0]),
        CACHE / f"validation-masks-{image_id[0]}.zip",
    )
    mask_path = ASSETS / mask_name
    if not mask_path.is_file():
        with zipfile.ZipFile(archive_path) as archive:
            try:
                mask_bytes = archive.read(mask_name)
            except KeyError as error:
                raise ValueError(f"mask {mask_name} is not in {archive_path.name}") from error
        mask_path.parent.mkdir(parents=True, exist_ok=True)
        mask_path.write_bytes(mask_bytes)

    return {
        "id": selection["id"],
        "operator": selection["operator"],
        "source": {
            "dataset_id": "open-images-v5",
            "image_id": image_id,
            "landing_url": row["OriginalLandingURL"],
            "author": row["Author"],
            "license": {"name": "CC-BY-2.0", "url": EXPECTED_LICENSE},
        },
        "input": {
            "path": str(image_path.relative_to(ROOT)),
            "sha256": sha256_file(image_path),
        },
        "reference": {
            "path": str(mask_path.relative_to(ROOT)),
            "sha256": sha256_file(mask_path),
            "annotation_method": (
                "Open Images V5 manually annotated validation instance mask; "
                + selection["review"]
            ),
        },
        "metrics": selection["metrics"],
        "prompt": selection["prompt"],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--selections",
        type=pathlib.Path,
        default=QUALITY / "open-images-selections.json",
    )
    parser.add_argument(
        "--output",
        type=pathlib.Path,
        default=QUALITY / "generated-manifest.json",
    )
    args = parser.parse_args()
    selections = json.loads(args.selections.read_text())
    manifest = json.loads((QUALITY / "manifest.json").read_text())
    rows = metadata()
    manifest["cases"] = [prepare_case(case, rows) for case in selections["cases"]]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(manifest, indent=2) + "\n")
    print(args.output.relative_to(ROOT) if args.output.is_relative_to(ROOT) else args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
