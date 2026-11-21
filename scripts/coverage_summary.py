"""Print an llvm-cov JSON summary from stdin, with paths relative to the workspace."""

import argparse
import json
import os
import sys


METRICS = ("regions", "functions", "lines")


def coverage_row(label, summary):
    row = [label]
    for name in METRICS:
        metric = summary[name]
        percent = f"{metric['percent']:.1f}%" if metric["count"] else "-"
        row.append(f"({metric['covered']}/{metric['count']}) {percent}")
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--min-file-lines", type=int, required=True)
    args = parser.parse_args()

    report, = json.load(sys.stdin)["data"]
    rows = [["File", *(name.title() for name in METRICS)]]
    for entry in report["files"]:
        lines = entry["summary"]["lines"]
        if lines["covered"] * 100 >= args.min_file_lines * lines["count"]:
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
