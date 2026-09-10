"""Build HTML and PDF documentation from doc/guide_cn.md and doc/guide_en.md.

Outputs under dist/doc/ (HTML, PDF, index.html only — doc/ keeps Markdown sources).

Usage:
  python scripts/build_docs.py
  python scripts/build_docs.py --html-only
  python scripts/build_docs.py --pdf-only
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
_SCRIPTS = ROOT / "scripts"
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))

from doc_common import (  # noqa: E402
    DIST_DOC_DIR,
    GUIDES,
    build_index_html,
    build_print_html,
    build_web_html,
    guide_html_name,
    guide_pdf_name,
    load_guide,
)


def find_browser() -> Path:
    candidates = [
        Path(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
        Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        Path(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
    ]
    for path in candidates:
        if path.is_file():
            return path
    raise SystemExit("未找到 Microsoft Edge 或 Chrome，无法生成 PDF。")


def html_to_pdf(html_path: Path, pdf_path: Path, browser: Path) -> None:
    url = html_path.resolve().as_uri()
    cmd = [
        str(browser),
        "--headless=new",
        "--disable-gpu",
        "--no-first-run",
        "--no-default-browser-check",
        f"--print-to-pdf={pdf_path.resolve()}",
        "--run-all-compositor-stages-before-draw",
        "--virtual-time-budget=20000",
        url,
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0 or not pdf_path.is_file():
        raise SystemExit(
            f"PDF 生成失败 ({pdf_path.name}):\n{result.stderr or result.stdout}"
        )


def build_guides_html(out_dir: Path, generated: str) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    for spec in GUIDES:
        raw, title = load_guide(spec)
        html = build_web_html(raw, title, spec, generated)
        out_path = out_dir / guide_html_name(spec)
        out_path.write_text(html, encoding="utf-8")
        print(f"Wrote {out_path.relative_to(ROOT)}")


def write_index_html(out_dir: Path, generated: str) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    index_path = out_dir / "index.html"
    index_path.write_text(build_index_html(generated), encoding="utf-8")
    print(f"Wrote {index_path.relative_to(ROOT)}")


def build_html(out_dir: Path, generated: str) -> None:
    build_guides_html(out_dir, generated)
    write_index_html(out_dir, generated)


def build_pdf(out_dir: Path, generated: str) -> None:
    browser = find_browser()
    out_dir.mkdir(parents=True, exist_ok=True)
    for spec in GUIDES:
        raw, title = load_guide(spec)
        html = build_print_html(raw, title, spec, generated)
        pdf_path = out_dir / guide_pdf_name(spec)
        with tempfile.TemporaryDirectory() as tmp:
            html_path = Path(tmp) / f"{spec.stem}.html"
            html_path.write_text(html, encoding="utf-8")
            html_to_pdf(html_path, pdf_path, browser)
        print(f"Wrote {pdf_path.relative_to(ROOT)} ({pdf_path.stat().st_size // 1024} KB)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build java-runner2026 documentation")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--html-only", action="store_true", help="Generate HTML only")
    group.add_argument("--pdf-only", action="store_true", help="Generate PDF only")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=DIST_DOC_DIR,
        help="Output directory (default: dist/doc/)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    generated = datetime.now().strftime("%Y-%m-%d %H:%M")
    out_dir = args.out_dir.resolve()

    if args.pdf_only:
        build_pdf(out_dir, generated)
    elif args.html_only:
        build_html(out_dir, generated)
    else:
        build_html(out_dir, generated)
        build_pdf(out_dir, generated)


if __name__ == "__main__":
    main()
