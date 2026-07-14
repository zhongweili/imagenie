#!/usr/bin/env python3
"""Validate quality fixture provenance, licenses, paths, and checksums."""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import pathlib
import re
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
SHA256 = re.compile(r"^[a-f0-9]{64}$")
OPERATORS = {"rembg", "upscale", "restore", "segment", "erase", "segment_erase"}
METRICS = {
    "rembg": {"iou", "boundary_f_score"},
    "upscale": {"psnr", "ssim", "lpips"},
    "restore": {"identity_similarity", "face_quality"},
    "segment": {"prompt_iou", "success_rate"},
    "erase": {"boundary_artifact_score", "pairwise_preference"},
    "segment_erase": {"task_success_rate", "total_latency_ms"},
}


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate(manifest: dict[str, Any], root: pathlib.Path, require_ready: bool = False) -> list[str]:
    errors: list[str] = []
    if manifest.get("schema_version") != "1.0":
        errors.append("schema_version must be 1.0")
    status = manifest.get("gate_status")
    if status not in {"blocked", "ready"}:
        errors.append("gate_status must be blocked or ready")
    if status == "blocked" and not manifest.get("gate_blockers"):
        errors.append("blocked manifest must explain at least one gate blocker")
    if require_ready and status != "ready":
        errors.append("quality gate is not ready")

    sources = manifest.get("approved_sources", [])
    source_ids = {source.get("id") for source in sources if isinstance(source, dict)}
    if None in source_ids:
        errors.append("every approved source needs an id")
    for source in sources:
        if not isinstance(source, dict):
            errors.append("approved_sources entries must be objects")
            continue
        if source.get("per_asset_verification_required") is not True:
            errors.append(f"source {source.get('id', '<unknown>')} must require per-asset verification")
        validate_https(source.get("homepage"), f"source {source.get('id')} homepage", errors)
        validate_license(source.get("annotation_license"), f"source {source.get('id')} annotation", errors)
        validate_license(source.get("expected_image_license"), f"source {source.get('id')} image", errors)

    cases = manifest.get("cases", [])
    if not isinstance(cases, list):
        return errors + ["cases must be an array"]
    seen_ids: set[str] = set()
    counts: collections.Counter[str] = collections.Counter()
    for index, case in enumerate(cases):
        label = f"case[{index}]"
        if not isinstance(case, dict):
            errors.append(f"{label} must be an object")
            continue
        case_id = case.get("id")
        if not isinstance(case_id, str) or not case_id:
            errors.append(f"{label} needs a non-empty id")
        elif case_id in seen_ids:
            errors.append(f"duplicate case id: {case_id}")
        else:
            seen_ids.add(case_id)
            label = case_id

        operator = case.get("operator")
        if operator not in OPERATORS:
            errors.append(f"{label}: unknown operator {operator!r}")
        else:
            counts[operator] += 1
            metrics = case.get("metrics", [])
            if not metrics:
                errors.append(f"{label}: metrics must not be empty")
            unknown = set(metrics) - METRICS[operator]
            if unknown:
                errors.append(f"{label}: unsupported metrics {sorted(unknown)}")

        source = case.get("source", {})
        if source.get("dataset_id") not in source_ids:
            errors.append(f"{label}: source dataset_id is not approved")
        for field in ("image_id", "author"):
            if not source.get(field):
                errors.append(f"{label}: source.{field} is required")
        validate_https(source.get("landing_url"), f"{label}: source landing_url", errors)
        validate_license(source.get("license"), f"{label}: source", errors)

        validate_asset(case.get("input"), root, f"{label}: input", errors)
        validate_asset(
            case.get("reference"),
            root,
            f"{label}: reference",
            errors,
            require_annotation=True,
        )
        if operator in {"segment", "segment_erase"} and not isinstance(case.get("prompt"), dict):
            errors.append(f"{label}: prompt is required for {operator}")

    if status == "ready":
        if manifest.get("gate_blockers"):
            errors.append("ready manifest must have no gate_blockers")
        minimums = manifest.get("minimum_cases", {})
        for operator in OPERATORS:
            minimum = minimums.get(operator)
            if not isinstance(minimum, int) or minimum < 1:
                errors.append(f"minimum_cases.{operator} must be a positive integer")
            elif counts[operator] < minimum:
                errors.append(f"{operator}: has {counts[operator]} cases, requires {minimum}")
    return errors


def validate_https(value: Any, label: str, errors: list[str]) -> None:
    if not isinstance(value, str) or not value.startswith("https://"):
        errors.append(f"{label} must be an https URL")


def validate_license(value: Any, label: str, errors: list[str]) -> None:
    if not isinstance(value, dict) or not value.get("name"):
        errors.append(f"{label} license name is required")
        return
    validate_https(value.get("url"), f"{label} license URL", errors)


def validate_asset(
    value: Any,
    root: pathlib.Path,
    label: str,
    errors: list[str],
    require_annotation: bool = False,
) -> None:
    if not isinstance(value, dict):
        errors.append(f"{label} must be an object")
        return
    relative = value.get("path")
    expected = value.get("sha256")
    if not isinstance(relative, str) or not relative:
        errors.append(f"{label}.path is required")
        return
    if not isinstance(expected, str) or not SHA256.fullmatch(expected):
        errors.append(f"{label}.sha256 must be 64 lowercase hex characters")
        return
    if require_annotation and not value.get("annotation_method"):
        errors.append(f"{label}.annotation_method is required")
    path = (root / relative).resolve()
    try:
        path.relative_to(root.resolve())
    except ValueError:
        errors.append(f"{label}.path escapes the repository root")
        return
    if not path.is_file():
        errors.append(f"{label}.path does not exist: {relative}")
    elif sha256_file(path) != expected:
        errors.append(f"{label}.sha256 does not match {relative}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", nargs="?", default="benchmarks/quality/manifest.json")
    parser.add_argument("--require-ready", action="store_true")
    args = parser.parse_args()
    path = pathlib.Path(args.manifest)
    if not path.is_absolute():
        path = ROOT / path
    errors = validate(json.loads(path.read_text()), ROOT, args.require_ready)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"valid: {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
