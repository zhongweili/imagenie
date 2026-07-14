#!/usr/bin/env python3
"""Run isolated Imagenie benchmark cases and aggregate machine-readable reports."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import subprocess
import sys
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
BENCH_BINARY = ROOT / "target" / "release" / "imagenie-bench"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("suite", type=pathlib.Path)
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--output-prefix", type=pathlib.Path)
    return parser.parse_args()


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def case_command(case: dict[str, Any]) -> list[str]:
    command = [
        str(BENCH_BINARY),
        "--operator",
        case["operator"],
        "--input",
        case["input"],
        "--warmups",
        str(case.get("warmups", 1)),
        "--iterations",
        str(case.get("iterations", 3)),
    ]
    for option in ("point", "mask", "dilate"):
        if option in case:
            command.extend([f"--{option}", str(case[option])])
    return command


def render_markdown(report: dict[str, Any]) -> str:
    dataset = report["dataset"]
    lines = [
        f"# Benchmark: {report['suite_id']}",
        "",
        f"Generated: `{report['generated_at']}`",
        "",
        f"Dataset status: `{dataset['status']}`; quality gate: `{str(dataset['quality_gate']).lower()}`.",
        "",
        "This report measures performance only. Output quality was not evaluated.",
        "",
        "| Case | Operator | Load ms | Cold ms | Warm p50 ms | Warm p95 ms | Peak RSS MiB |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for item in report["cases"]:
        bench = item["report"]
        measurements = bench["measurements"]
        stats = measurements["warm_stats"]
        rss = measurements["peak_rss_bytes"]
        lines.append(
            "| {id} | {operator} | {load:.1f} | {cold:.1f} | {p50} | {p95} | {rss} |".format(
                id=item["id"],
                operator=bench["case"]["operator"],
                load=measurements["model_load_ms"],
                cold=measurements["cold_ms"],
                p50=format_number(stats["p50_ms"]),
                p95=format_number(stats["p95_ms"]),
                rss=format_number(rss / 1024 / 1024 if rss is not None else None),
            )
        )
    first = report["cases"][0]["report"] if report["cases"] else None
    if first:
        environment = first["environment"]
        lines.extend(
            [
                "",
                "## Environment",
                "",
                f"- OS/architecture: `{environment['os']}/{environment['arch']}`",
                f"- CPU: `{environment.get('cpu_model') or 'unknown'}`",
                f"- Logical CPUs: `{environment['logical_cpus']}`",
                f"- Execution provider: `{environment['execution_provider']}`",
                f"- Git commit: `{first['git'].get('commit') or 'unknown'}`",
                f"- Dirty worktree during run: `{str(first['git'].get('dirty')).lower()}`",
            ]
        )
    return "\n".join(lines) + "\n"


def format_number(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.1f}"


def main() -> int:
    args = parse_args()
    suite_path = args.suite if args.suite.is_absolute() else ROOT / args.suite
    suite = json.loads(suite_path.read_text())
    if not args.skip_build:
        print("Building imagenie-bench (release)...", file=sys.stderr)
        subprocess.run(
            ["cargo", "build", "--release", "-p", "imagenie-bench"],
            cwd=ROOT,
            check=True,
        )

    case_reports = []
    for index, case in enumerate(suite["cases"], start=1):
        input_path = ROOT / case["input"]
        if not input_path.is_file():
            raise SystemExit(f"missing input for {case['id']}: {input_path}")
        print(f"[{index}/{len(suite['cases'])}] {case['id']}", file=sys.stderr)
        completed = run(case_command(case))
        case_reports.append(
            {
                "id": case["id"],
                "report": json.loads(completed.stdout),
                "stderr": completed.stderr.strip() or None,
            }
        )

    generated_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()
    aggregate = {
        "schema_version": "1.0",
        "suite_id": suite["id"],
        "description": suite.get("description"),
        "generated_at": generated_at,
        "dataset": suite["dataset"],
        "cases": case_reports,
    }
    timestamp = generated_at.replace(":", "").replace("+00:00", "Z").replace("-", "")
    prefix = args.output_prefix or ROOT / "benchmarks" / "reports" / f"{suite['id']}-{timestamp}"
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
