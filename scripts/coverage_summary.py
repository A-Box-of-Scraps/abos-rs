"""Print an llvm-cov JSON summary from stdin, with paths relative to the workspace."""

import argparse
import json
import os
import sys
from typing import TypedDict, cast

METRICS = ("regions", "functions", "lines")


class Metric(TypedDict):
    covered: int
    count: int
    percent: float


class FileCoverage(TypedDict):
    filename: str
    summary: dict[str, Metric]


class CoverageReport(TypedDict):
    files: list[FileCoverage]
    totals: dict[str, Metric]


class CoverageExport(TypedDict):
    data: list[CoverageReport]


def coverage_row(label: str, summary: dict[str, Metric]) -> list[str]:
    row = [label]
    for name in METRICS:
        metric = summary[name]
        percent = f"{metric['percent']:.1f}%" if metric["count"] else "-"
        row.append(f"({metric['covered']}/{metric['count']}) {percent}")
    return row


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--min-file-lines", type=int, required=True)
    min_file_lines = cast(int, parser.parse_args().min_file_lines)

    export = cast(CoverageExport, json.load(sys.stdin))
    report, = export["data"]
    rows = [["File", *(name.title() for name in METRICS)]]
    for entry in report["files"]:
        lines = entry["summary"]["lines"]
        if lines["covered"] * 100 >= min_file_lines * lines["count"]:
            continue
        rows.append(coverage_row(os.path.relpath(entry["filename"]), entry["summary"]))
    rows.append(coverage_row("TOTAL", report["totals"]))

    widths = [max(map(len, column)) for column in zip(*rows)]
    for label, *values in rows:
        cells = [label.ljust(widths[0])]
        cells.extend(value.rjust(width) for value, width in zip(values, widths[1:]))
        print("  ".join(cells))


if __name__ == "__main__":
    main()
