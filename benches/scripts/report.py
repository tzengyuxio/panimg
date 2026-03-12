#!/usr/bin/env python3
"""Aggregate hyperfine benchmark results into a Markdown report."""

import json
import os
import platform
import subprocess
import sys
from pathlib import Path

RESULTS_DIR = Path(__file__).resolve().parent.parent / "results"


def get_system_info() -> dict:
    """Collect system information for the report header."""
    info = {
        "os": f"{platform.system()} {platform.release()}",
        "arch": platform.machine(),
        "cpu": "unknown",
        "ram": "unknown",
    }

    if platform.system() == "Darwin":
        try:
            info["cpu"] = (
                subprocess.check_output(
                    ["sysctl", "-n", "machdep.cpu.brand_string"], text=True
                ).strip()
            )
            mem_bytes = int(
                subprocess.check_output(
                    ["sysctl", "-n", "hw.memsize"], text=True
                ).strip()
            )
            info["ram"] = f"{mem_bytes // (1024**3)} GB"
        except (subprocess.CalledProcessError, ValueError):
            pass
    elif platform.system() == "Linux":
        try:
            with open("/proc/cpuinfo") as f:
                for line in f:
                    if line.startswith("model name"):
                        info["cpu"] = line.split(":", 1)[1].strip()
                        break
            with open("/proc/meminfo") as f:
                for line in f:
                    if line.startswith("MemTotal"):
                        kb = int(line.split()[1])
                        info["ram"] = f"{kb // (1024**2)} GB"
                        break
        except (OSError, ValueError):
            pass

    return info


def format_time(seconds: float) -> str:
    """Format seconds into a human-readable string."""
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.1f} µs"
    elif seconds < 1.0:
        return f"{seconds * 1_000:.1f} ms"
    else:
        return f"{seconds:.2f} s"


def load_hyperfine_results(json_path: Path) -> list[dict]:
    """Load and parse a hyperfine JSON result file."""
    with open(json_path) as f:
        data = json.load(f)
    return data.get("results", [])


def results_to_table(results: list[dict]) -> str:
    """Convert hyperfine results to a Markdown table."""
    if not results:
        return "(no results)\n"

    lines = [
        "| Command | Mean | Min | Max | Relative |",
        "|---------|------|-----|-----|----------|",
    ]

    # Sort by mean time
    sorted_results = sorted(results, key=lambda r: r["mean"])
    fastest = sorted_results[0]["mean"]

    for r in sorted_results:
        name = r["command"]
        mean = format_time(r["mean"])
        stddev = format_time(r["stddev"])
        min_t = format_time(r["min"])
        max_t = format_time(r["max"])
        relative = r["mean"] / fastest if fastest > 0 else 1.0
        rel_str = "1.00x" if relative == 1.0 else f"{relative:.2f}x"
        lines.append(f"| {name} | {mean} ± {stddev} | {min_t} | {max_t} | {rel_str} |")

    return "\n".join(lines) + "\n"


def read_text_report(path: Path) -> str:
    """Read a plain text report file."""
    if path.exists():
        return path.read_text()
    return ""


def main():
    if not RESULTS_DIR.exists():
        print(f"ERROR: Results directory not found: {RESULTS_DIR}", file=sys.stderr)
        sys.exit(1)

    json_files = sorted(RESULTS_DIR.glob("*.json"))
    if not json_files:
        print("WARNING: No JSON result files found. Run benchmarks first.", file=sys.stderr)

    sys_info = get_system_info()

    report_lines = [
        "# panimg Benchmark Report",
        "",
        "## Environment",
        "",
        f"- **OS**: {sys_info['os']}",
        f"- **CPU**: {sys_info['cpu']}",
        f"- **RAM**: {sys_info['ram']}",
        f"- **Architecture**: {sys_info['arch']}",
        "",
    ]

    # Process each JSON result file
    for json_file in json_files:
        bench_name = json_file.stem.replace("_", " ").title()
        results = load_hyperfine_results(json_file)

        report_lines.append(f"## {bench_name}")
        report_lines.append("")
        report_lines.append(results_to_table(results))
        report_lines.append("")

    # Include memory report if available
    mem_report = RESULTS_DIR / "memory_report.txt"
    if mem_report.exists():
        report_lines.append("## Memory Usage")
        report_lines.append("")
        report_lines.append("```")
        report_lines.append(read_text_report(mem_report))
        report_lines.append("```")
        report_lines.append("")

    # Include quality report if available
    quality_report = RESULTS_DIR / "quality_report.txt"
    if quality_report.exists():
        report_lines.append("## Output Quality")
        report_lines.append("")
        report_lines.append(read_text_report(quality_report))
        report_lines.append("")

    report_path = RESULTS_DIR / "REPORT.md"
    report_content = "\n".join(report_lines)
    report_path.write_text(report_content)
    print(f"Report written to: {report_path}")


if __name__ == "__main__":
    main()
