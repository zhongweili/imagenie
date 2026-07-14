#!/usr/bin/env python3
"""Evaluate checksum-pinned quality fixtures with the Rust benchmark binary."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import subprocess
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "release" / "imagenie-bench"


def command(case: dict[str, Any]) -> list[str]:
    result = [
        str(BINARY),
        "--operator",
        case["operator"].replace("_", "-"),
        "--input",
        case["input"]["path"],
        "--reference",
        case["reference"]["path"],
        "--warmups",
        "0",
        "--iterations",
        "1",
    ]
    if "prompt" in case:
        result.extend(["--point", f"{case['prompt']['x']},{case['prompt']['y']}"])
    return result


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Quality fixture smoke report",
        "",
        f"Generated: `{report['generated_at']}`",
        "",
        f"Gate status: `{report['gate_status']}`. This report does not make the incomplete fixture set release-ready.",
        "",
        "| Case | Operator | IoU | Author | License |",
        "| --- | --- | ---: | --- | --- |",
    ]
    for item in report["cases"]:
        fixture = item["fixture"]
        measured = item["report"]
        source = fixture["source"]
        iou = measured["result"]["quality_metrics"]["iou"]
        lines.append(
            f"| {fixture['id']} | {fixture['operator']} | {iou:.4f} | "
            f"[{source['author']}]({source['landing_url']}) | "
            f"[{source['license']['name']}]({source['license']['url']}) |"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "manifest",
        nargs="?",
        type=pathlib.Path,
        default=ROOT / "benchmarks" / "quality" / "generated-manifest.json",
    )
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--output-prefix", type=pathlib.Path)
    args = parser.parse_args()
    manifest_path = args.manifest if args.manifest.is_absolute() else ROOT / args.manifest
    manifest = json.loads(manifest_path.read_text())
    if not args.skip_build:
        subprocess.run(["cargo", "build", "--release", "-p", "imagenie-bench"], cwd=ROOT, check=True)

    reports = []
    for fixture in manifest["cases"]:
        if fixture["operator"] not in {"rembg", "segment"}:
            raise SystemExit(f"unsupported quality operator: {fixture['operator']}")
        print(f"evaluating {fixture['id']}", file=sys.stderr)
        completed = subprocess.run(
            command(fixture),
            cwd=ROOT,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        measured = json.loads(completed.stdout)
        if measured["case"]["input_sha256"] != fixture["input"]["sha256"]:
            raise SystemExit(f"input checksum drift for {fixture['id']}")
        if measured["case"]["reference_sha256"] != fixture["reference"]["sha256"]:
            raise SystemExit(f"reference checksum drift for {fixture['id']}")
        reports.append({"fixture": fixture, "report": measured})

    now = dt.datetime.now(dt.timezone.utc)
    aggregate = {
        "schema_version": "1.0",
        "generated_at": now.replace(microsecond=0).isoformat(),
        "gate_status": manifest["gate_status"],
        "cases": reports,
    }
    prefix = args.output_prefix or (
        ROOT / "benchmarks" / "reports" / f"quality-smoke-{now.strftime('%Y%m%dT%H%M%SZ')}"
    )
    prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = prefix.with_suffix(".json")
    markdown_path = prefix.with_suffix(".md")
    json_path.write_text(json.dumps(aggregate, indent=2) + "\n")
    markdown_path.write_text(render_markdown(aggregate))
    print(json_path.relative_to(ROOT) if json_path.is_relative_to(ROOT) else json_path)
    print(markdown_path.relative_to(ROOT) if markdown_path.is_relative_to(ROOT) else markdown_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
