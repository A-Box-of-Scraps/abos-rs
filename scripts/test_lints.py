"""Run the lint workspace's Dylint UI tests and report their coverage."""

import argparse
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import cast


def coverage_env() -> dict[str, str]:
    # Dylint builds and loads the library in child processes. They must inherit
    # the same instrumentation and target directory as the test runner.
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(Path("target/dylint-coverage").resolve())
    exports = subprocess.check_output(["cargo", "llvm-cov", "show-env", "--sh"], env=env, encoding="utf-8")
    # Decode POSIX quoting on every OS without evaluating shell code.
    words = iter(shlex.split(exports))
    for command in words:
        assignment = next(words, "")
        key, separator, value = assignment.partition("=")
        if command != "export" or not separator or not key.isidentifier():
            raise ValueError(f"Unexpected cargo llvm-cov export: {command} {assignment}")
        env[key] = value
    return env


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--min-file-lines", required=True)
    _ = parser.add_argument("--output-path", required=True, type=Path)
    args, cargo_flags = parser.parse_known_args()
    threshold = cast(str, args.min_file_lines)
    report = cast(Path, args.output_path)
    env = coverage_env()

    tests = subprocess.run(
        ["cargo", "nextest", "run", *cargo_flags, "--cargo-quiet", "--no-tests=warn", "--no-fail-fast"], env=env, check=False
    )
    if tests.returncode:
        return tests.returncode

    # A threshold failure produces a useful report; a tool failure might not.
    report.unlink(missing_ok=True)
    coverage = subprocess.run(
        [
            "cargo", "llvm-cov", "report", "--json", "--summary-only", "--output-path", str(report),
            "--fail-under-file-lines", threshold, "--fail-under-lines", "80",
            "--fail-under-functions", "80", "--fail-under-regions", "80",
        ],
        env=env, check=False,
    )
    if report.is_file():
        with report.open("rb") as summary:
            _ = subprocess.run(
                [sys.executable, str(Path(__file__).with_name("coverage_summary.py")), "--min-file-lines", threshold],
                stdin=summary, check=True,
            )
    return coverage.returncode


if __name__ == "__main__":
    sys.exit(main())
