#!/usr/bin/env python3
"""Visual regression harness.

    visual.py render  --out DIR [--only NAME ...]
    visual.py compare --baseline DIR --actual DIR --out DIR [--allow-changes]
"""

from __future__ import annotations

import argparse
import base64
import html
import json
import os
import re
import shutil
import subprocess
import sys
import time
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
CASES_FILE = HERE / "cases.toml"
RENDER_TIMEOUT_SECS = 300
LOG_TAIL_LINES = 40


def load_cases(only: list[str] | None = None) -> list[dict]:
    data = tomllib.loads(CASES_FILE.read_text())
    defaults = data.get("defaults", {})
    cases, seen = [], set()
    for raw in data.get("case", []):
        case = {**defaults, **raw}
        name = case.get("name")
        if not name or not re.fullmatch(r"[A-Za-z0-9_-]+", name):
            sys.exit(f"cases.toml: invalid case name {name!r}")
        if name in seen:
            sys.exit(f"cases.toml: duplicate case {name!r}")
        if ("rust" in case) == ("python" in case):
            sys.exit(f"cases.toml: case {name!r} must set exactly one of `rust` or `python`")
        seen.add(name)
        cases.append(case)
    if only:
        unknown = set(only) - seen
        if unknown:
            sys.exit(f"unknown case(s): {', '.join(sorted(unknown))}")
        cases = [c for c in cases if c["name"] in only]
    return cases


def run(cmd: list[str], cwd: Path, env: dict | None = None) -> None:
    print(f"$ {' '.join(cmd)}  (in {cwd})", flush=True)
    subprocess.run(cmd, cwd=cwd, env=env, check=True)


def git_head(root: Path) -> str | None:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=root, capture_output=True, text=True, check=True
        )
        return out.stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None


def adapter_from_log(log: str) -> str | None:
    match = re.search(r"AdapterInfo \{[^}]*\}", log)
    if not match:
        return None
    fields = dict(re.findall(r'(\w+): ("[^"]*"|[^,}]+)', match.group(0)))
    name, backend, driver = (fields.get(k, "").strip().strip('"') for k in ("name", "backend", "driver_info"))
    return f"{name} ({backend}{', ' + driver if driver else ''})"


def cmd_render(args: argparse.Namespace) -> int:
    root = REPO
    out = Path(args.out).resolve()
    logs = out / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    cases = load_cases(args.only)

    rust = [c for c in cases if "rust" in c]
    python = [c for c in cases if "python" in c]
    py_dir = root / "crates" / "processing_pyo3"

    if rust and not args.skip_build:
        examples = [arg for c in rust for arg in ("--example", c["rust"])]
        run(["cargo", "build", "--release", *examples], cwd=root)
    if python and not args.skip_build:
        run(["uv", "run", "maturin", "develop", "--release"], cwd=py_dir)

    manifest = {"commit": git_head(root), "adapter": None, "cases": {}}
    for case in cases:
        name = case["name"]
        png = out / f"{name}.png"
        png.unlink(missing_ok=True)
        env = {
            **os.environ,
            "PROCESSING_CI_SCREENSHOT": str(png),
            "PROCESSING_CI_FRAME": str(case["frame"]),
        }
        env.setdefault("PROCESSING_ASSET_ROOT", str(root / "assets"))
        if "rust" in case:
            cmd, cwd = ["cargo", "run", "--quiet", "--release", "--example", case["rust"]], root
        else:
            cmd, cwd = ["uv", "run", "python", f"examples/{case['python']}"], py_dir

        print(f"--- {name}", flush=True)
        started = time.monotonic()
        try:
            proc = subprocess.run(
                cmd,
                cwd=cwd,
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                errors="replace",
                timeout=RENDER_TIMEOUT_SECS,
            )
            log, code = proc.stdout, proc.returncode
        except subprocess.TimeoutExpired as e:
            log = (e.stdout or b"").decode(errors="replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
            log += f"\n[visual.py] timed out after {RENDER_TIMEOUT_SECS}s\n"
            code = None
        elapsed = time.monotonic() - started
        (logs / f"{name}.log").write_text(log)

        if code == 0 and png.exists():
            status = "ok"
        elif code == 0:
            status = "no-capture"
        else:
            status = "timeout" if code is None else f"exit {code}"
        manifest["adapter"] = manifest["adapter"] or adapter_from_log(log)
        manifest["cases"][name] = {"status": status, "seconds": round(elapsed, 1)}
        print(f"    {status} in {elapsed:.1f}s", flush=True)
        if status != "ok":
            print("\n".join(log.splitlines()[-LOG_TAIL_LINES:]), flush=True)

    (out / "manifest.json").write_text(json.dumps(manifest, indent=2))
    failed = [n for n, c in manifest["cases"].items() if c["status"] != "ok"]
    if failed:
        print(f"failed to render: {', '.join(failed)}", file=sys.stderr)
    # Render failures are recorded in the manifest and surfaced by `compare`.
    return 0


def odiff(exe: str, base: Path, actual: Path, diff: Path, threshold: float) -> tuple[str, float]:
    """Returns (outcome, differing percent) where outcome is match, pixels or layout."""
    proc = subprocess.run(
        [
            exe, str(base), str(actual), str(diff),
            "--antialiasing", "--fail-on-layout", "--parsable-stdout", "--diff-mask",
            f"--threshold={threshold}",
        ],
        capture_output=True,
        text=True,
    )
    stdout = proc.stdout.strip()
    if proc.returncode == 0:
        return "match", 0.0
    if proc.returncode == 21:
        return "layout", 100.0
    if proc.returncode == 22:
        _count, percent = stdout.split(";")
        return "pixels", float(percent)
    raise RuntimeError(f"odiff exited {proc.returncode}: {stdout} {proc.stderr.strip()}")


def read_manifest(directory: Path) -> dict:
    path = directory / "manifest.json"
    return json.loads(path.read_text()) if path.exists() else {"cases": {}}


def cmd_compare(args: argparse.Namespace) -> int:
    baseline, actual, out = Path(args.baseline), Path(args.actual), Path(args.out)
    diffs = out / "diff"
    diffs.mkdir(parents=True, exist_ok=True)
    exe = args.odiff or shutil.which("odiff")
    if not exe:
        sys.exit("odiff not found; install it with `npm install -g odiff-bin` or pass --odiff")

    base_manifest, actual_manifest = read_manifest(baseline), read_manifest(actual)
    results = []
    for case in load_cases():
        name = case["name"]
        base_png, actual_png, diff_png = (
            baseline / f"{name}.png",
            actual / f"{name}.png",
            diffs / f"{name}.png",
        )
        render_status = actual_manifest["cases"].get(name, {}).get("status", "not rendered")
        result = {
            "name": name,
            "source": case.get("rust") or case.get("python"),
            "kind": "rust" if "rust" in case else "python",
            "max_diff_percent": case["max_diff_percent"],
            "diff_percent": None,
        }
        if not actual_png.exists():
            result.update(status="error", detail=render_status)
        elif not base_png.exists():
            result.update(status="new", detail="no baseline on main")
        else:
            outcome, percent = odiff(exe, base_png, actual_png, diff_png, case["threshold"])
            result["diff_percent"] = percent
            if outcome == "layout":
                result.update(status="changed", detail="image size changed")
            elif percent > case["max_diff_percent"]:
                result.update(status="changed", detail=f"{percent:g}% of pixels differ")
            else:
                result.update(status="pass", detail="identical" if outcome == "match" else f"{percent:g}% (within tolerance)")
        results.append(result)

    report = {
        "baseline_commit": base_manifest.get("commit"),
        "actual_commit": actual_manifest.get("commit"),
        "adapter": actual_manifest.get("adapter"),
        "allow_changes": args.allow_changes,
        "results": results,
    }
    (out / "results.json").write_text(json.dumps(report, indent=2))
    (out / "summary.md").write_text(render_summary(report))
    (out / "report.html").write_text(render_html(report, baseline, actual, diffs, actual / "logs"))

    counts = {s: sum(r["status"] == s for r in results) for s in ("pass", "changed", "new", "error")}
    print(" ".join(f"{k}={v}" for k, v in counts.items()))
    if counts["error"]:
        return 1
    if counts["changed"] and not args.allow_changes:
        return 1
    return 0


STATUS_ICON = {"pass": "✅", "changed": "❌", "new": "🆕", "error": "💥"}


def render_summary(report: dict) -> str:
    results = report["results"]
    changed = [r for r in results if r["status"] == "changed"]
    notable = [r for r in results if r["status"] != "pass"]
    total = len(results)
    if not notable:
        headline = f"**Visual regression: no changes** across {total} cases"
    else:
        parts = []
        for status, label in (("changed", "changed"), ("error", "failed to render"), ("new", "new")):
            n = sum(r["status"] == status for r in results)
            if n:
                parts.append(f"{n} {label}")
        headline = f"**Visual regression: {', '.join(parts)}** of {total} cases"
    lines = [headline, ""]
    if notable:
        lines += ["| case | status | detail |", "|---|---|---|"]
        for r in notable:
            lines.append(f"| `{r['name']}` | {STATUS_ICON[r['status']]} {r['status']} | {r['detail']} |")
        lines.append("")
    if changed:
        if report["allow_changes"]:
            lines.append("Changes accepted by the `deliberate-rendering-change` label.")
        else:
            lines.append("If these changes are intentional, add the `deliberate-rendering-change` label.")
        lines.append("")
    base = (report.get("baseline_commit") or "unknown")[:10]
    lines.append(f"<sub>baseline `{base}` · adapter `{report.get('adapter') or 'unknown'}`</sub>")
    return "\n".join(lines) + "\n"


def data_uri(path: Path) -> str | None:
    if not path.exists():
        return None
    return "data:image/png;base64," + base64.b64encode(path.read_bytes()).decode()


def figure(label: str, path: Path, css_class: str = "") -> str:
    uri = data_uri(path)
    body = f'<img src="{uri}" alt="{label}">' if uri else '<div class="missing">none</div>'
    return f'<figure class="{css_class}">{body}<figcaption>{label}</figcaption></figure>'


def render_html(report: dict, baseline: Path, actual: Path, diffs: Path, logs: Path) -> str:
    order = {"error": 0, "changed": 1, "new": 2, "pass": 3}
    sections = []
    for r in sorted(report["results"], key=lambda r: (order[r["status"]], r["name"])):
        name, status = r["name"], r["status"]
        title = (
            f'<h2><span class="badge {status}">{status}</span> {html.escape(name)} '
            f'<small>{html.escape(r["kind"])}: {html.escape(r["source"])} · '
            f'{html.escape(r["detail"])}</small></h2>'
        )
        if status == "pass":
            sections.append(f'<section class="pass">{title}</section>')
            continue
        figures = ""
        if status == "changed":
            figures = (
                figure("baseline (main)", baseline / f"{name}.png")
                + figure("this PR", actual / f"{name}.png")
                + figure("changed pixels", diffs / f"{name}.png", "mask")
            )
        elif status == "new":
            figures = figure("this PR", actual / f"{name}.png")
        log = ""
        if status == "error":
            log_path = logs / f"{name}.log"
            tail = "\n".join(log_path.read_text().splitlines()[-LOG_TAIL_LINES:]) if log_path.exists() else ""
            log = f"<pre>{html.escape(tail or 'no log captured')}</pre>"
        sections.append(f'<section>{title}<div class="figures">{figures}</div>{log}</section>')

    meta = (
        f"baseline {html.escape(str(report.get('baseline_commit')))} · "
        f"PR {html.escape(str(report.get('actual_commit')))} · "
        f"adapter {html.escape(str(report.get('adapter')))}"
    )
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Visual regression report</title>
<style>
:root {{ --bg: #fff; --fg: #1f2328; --muted: #656d76; --border: #d0d7de; --panel: #f6f8fa;
  --pass: #1a7f37; --changed: #cf222e; --new: #0969da; --error: #8250df; }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg: #0d1117; --fg: #e6edf3; --muted: #8d96a0;
  --border: #30363d; --panel: #161b22; --pass: #3fb950; --changed: #f85149; --new: #4493f8; --error: #ab7df8; }} }}
body {{ margin: 0 auto; max-width: 1600px; padding: 16px; background: var(--bg); color: var(--fg);
  font: 14px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
header p {{ color: var(--muted); margin-top: 0; }}
section {{ border-top: 1px solid var(--border); padding: 12px 0; }}
section.pass h2 {{ font-size: 14px; margin: 0; }}
h2 {{ font-size: 16px; margin: 0 0 8px; }}
h2 small {{ color: var(--muted); font-weight: normal; }}
.badge {{ display: inline-block; min-width: 64px; text-align: center; border-radius: 12px; padding: 0 8px;
  color: #fff; font-size: 12px; font-weight: 600; }}
.badge.pass {{ background: var(--pass); }} .badge.changed {{ background: var(--changed); }}
.badge.new {{ background: var(--new); }} .badge.error {{ background: var(--error); }}
.figures {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 8px; }}
figure {{ margin: 0; background: var(--panel); border: 1px solid var(--border); border-radius: 6px; padding: 6px; }}
figure img {{ display: block; width: 100%; height: auto; image-rendering: pixelated; }}
figure.mask img {{ background: #fff; outline: 1px solid var(--border); }}
figcaption {{ color: var(--muted); font-size: 12px; margin-top: 4px; }}
.missing {{ color: var(--muted); padding: 24px; text-align: center; }}
pre {{ background: var(--panel); border: 1px solid var(--border); border-radius: 6px; padding: 8px;
  overflow-x: auto; font-size: 12px; }}
</style>
</head>
<body>
<header>
<h1>Visual regression report</h1>
<p>{meta}</p>
</header>
{"".join(sections)}
</body>
</html>
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)

    render = sub.add_parser("render", help="render every case to PNG")
    render.add_argument("--out", required=True)
    render.add_argument("--only", nargs="+", metavar="NAME")
    render.add_argument("--skip-build", action="store_true", help="reuse existing builds")
    render.set_defaults(func=cmd_render)

    compare = sub.add_parser("compare", help="diff rendered cases against a baseline")
    compare.add_argument("--baseline", required=True)
    compare.add_argument("--actual", required=True)
    compare.add_argument("--out", required=True)
    compare.add_argument("--odiff", help="path to the odiff binary")
    compare.add_argument("--allow-changes", action="store_true", help="report changes without failing")
    compare.set_defaults(func=cmd_compare)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
