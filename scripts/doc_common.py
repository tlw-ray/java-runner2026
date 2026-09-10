"""Shared Markdown → HTML helpers for java-runner2026 documentation.

Source of truth: doc/guide_cn.md and doc/guide_en.md.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import markdown
from markdown.extensions.fenced_code import FencedCodeExtension
from markdown.extensions.tables import TableExtension
from markdown.extensions.toc import TocExtension

ROOT = Path(__file__).resolve().parents[1]
DOC_DIR = ROOT / "doc"
DIST_DOC_DIR = ROOT / "dist" / "doc"

MARKDOWN_EXTENSIONS = [
    FencedCodeExtension(),
    TableExtension(),
    TocExtension(baselevel=2),
    "nl2br",
]


@dataclass(frozen=True)
class GuideSpec:
    stem: str
    lang: str
    html_lang: str
    brand: str
    nav_home: str
    footer: str
    index_label: str
    pdf_label: str


GUIDES: tuple[GuideSpec, ...] = (
    GuideSpec(
        stem="guide_cn",
        lang="zh-CN",
        html_lang="zh-CN",
        brand="java-runner2026 使用文档",
        nav_home="文档首页",
        footer="java-runner2026 · 生成于 {generated}",
        index_label="使用与实现指南（中文）",
        pdf_label="guide_cn.pdf（中文 PDF）",
    ),
    GuideSpec(
        stem="guide_en",
        lang="en",
        html_lang="en",
        brand="java-runner2026 documentation",
        nav_home="Documentation home",
        footer="java-runner2026 · generated {generated}",
        index_label="User and Implementation Guide (English)",
        pdf_label="guide_en.pdf (English PDF)",
    ),
)


WEB_HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="{html_lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — java-runner2026</title>
  <style>
    :root {{
      --primary: #2e6be6;
      --text: #1f2937;
      --muted: #6b7280;
      --bg: #f8fafc;
      --card: #ffffff;
      --border: #e5e7eb;
      --code-bg: #0f172a;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Segoe UI", "Microsoft YaHei", sans-serif;
      color: var(--text);
      background: var(--bg);
      line-height: 1.65;
    }}
    header {{
      background: linear-gradient(135deg, #2e6be6, #1890ff);
      color: #fff;
      padding: 1.25rem 1.5rem;
    }}
    header a {{ color: #dbeafe; }}
    main {{
      max-width: 960px;
      margin: 0 auto;
      padding: 1.5rem;
    }}
    article {{
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 1.75rem 2rem;
      box-shadow: 0 1px 3px rgba(0,0,0,.06);
    }}
    h1,h2,h3 {{ line-height: 1.3; margin-top: 1.6em; }}
    h1 {{ margin-top: 0; font-size: 1.75rem; }}
    a {{ color: var(--primary); }}
    code {{
      background: #eef2ff;
      padding: .1em .35em;
      border-radius: 4px;
      font-size: .92em;
    }}
    pre {{
      background: var(--code-bg);
      color: #e2e8f0;
      padding: 1rem 1.1rem;
      border-radius: 8px;
      overflow-x: auto;
    }}
    pre code {{ background: none; padding: 0; color: inherit; }}
    table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
    th, td {{ border: 1px solid var(--border); padding: .55rem .75rem; text-align: left; }}
    th {{ background: #f1f5f9; }}
    hr {{ border: none; border-top: 1px solid var(--border); margin: 2rem 0; }}
    .mermaid {{
      background: #fff;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1rem;
      margin: 1rem 0;
      overflow-x: auto;
    }}
    footer {{
      text-align: center;
      color: var(--muted);
      font-size: .875rem;
      padding: 2rem 1rem 2.5rem;
    }}
    .nav {{ margin-top: .5rem; font-size: .95rem; }}
  </style>
</head>
<body>
  <header>
    <div><strong>{brand}</strong></div>
    <div class="nav"><a href="index.html">{nav_home}</a></div>
  </header>
  <main>
    <article>{body}</article>
  </main>
  <footer>{footer}</footer>
  <script type="module">
    import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
    mermaid.initialize({{ startOnLoad: true, theme: "neutral", securityLevel: "loose" }});
  </script>
</body>
</html>
"""

PRINT_HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="{html_lang}">
<head>
  <meta charset="utf-8">
  <title>{title}</title>
  <style>
    @page {{ size: A4; margin: 18mm 16mm; }}
    body {{
      font-family: "Segoe UI", "Microsoft YaHei", Arial, sans-serif;
      color: #1f2937;
      line-height: 1.55;
      font-size: 11pt;
      max-width: 100%;
    }}
    h1 {{ font-size: 20pt; border-bottom: 2px solid #2e6be6; padding-bottom: .3em; }}
    h2 {{ font-size: 15pt; margin-top: 1.4em; color: #1e40af; page-break-after: avoid; }}
    h3 {{ font-size: 12.5pt; margin-top: 1.1em; page-break-after: avoid; }}
    pre, code {{ font-family: Consolas, "Courier New", monospace; font-size: 9pt; }}
    pre {{
      background: #f3f4f6;
      border: 1px solid #e5e7eb;
      padding: .6em .8em;
      overflow-x: auto;
      page-break-inside: avoid;
    }}
    code {{ background: #eef2ff; padding: .05em .25em; border-radius: 3px; }}
    table {{
      border-collapse: collapse;
      width: 100%;
      font-size: 10pt;
      margin: .8em 0;
      page-break-inside: avoid;
    }}
    th, td {{ border: 1px solid #d1d5db; padding: .35em .5em; text-align: left; }}
    th {{ background: #eff6ff; }}
    blockquote {{
      border-left: 4px solid #2e6be6;
      margin: 1em 0;
      padding: .2em .8em;
      color: #374151;
      background: #f8fafc;
    }}
    hr {{ border: none; border-top: 1px solid #e5e7eb; margin: 1.5em 0; }}
    .mermaid {{
      text-align: center;
      margin: 1em 0;
      page-break-inside: avoid;
    }}
    .meta {{ color: #6b7280; font-size: 9pt; margin-bottom: 1.5em; }}
  </style>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <script>
    mermaid.initialize({{ startOnLoad: true, theme: "neutral" }});
  </script>
</head>
<body>
  <p class="meta">{meta} · java-runner2026</p>
  {body}
</body>
</html>
"""

INDEX_TEMPLATE = """<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>java-runner2026 documentation</title>
  <style>
    body {{ font-family: "Segoe UI", "Microsoft YaHei", sans-serif; margin: 0; background: #f8fafc; color: #1f2937; }}
    header {{ background: linear-gradient(135deg, #2e6be6, #1890ff); color: #fff; padding: 2rem 1.5rem; }}
    main {{ max-width: 720px; margin: 0 auto; padding: 1.5rem; }}
    .card {{ background: #fff; border: 1px solid #e5e7eb; border-radius: 12px; padding: 1.5rem; margin-bottom: 1rem; }}
    h2 {{ margin-top: 0; font-size: 1.1rem; }}
    ul {{ padding-left: 1.25rem; }}
    a {{ color: #2e6be6; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    code {{ background: #eef2ff; padding: .1em .35em; border-radius: 4px; }}
    .muted {{ color: #6b7280; font-size: .9rem; }}
  </style>
</head>
<body>
  <header>
    <h1 style="margin:0 0 .5rem">java-runner2026</h1>
    <p style="margin:0;opacity:.9">Documentation index · generated {generated}</p>
  </header>
  <main>
    <div class="card">
      <h2>Guides / 指南</h2>
      <ul>
{guide_links}
      </ul>
      <p class="muted">Sources: <code>doc/guide_cn.md</code>, <code>doc/guide_en.md</code></p>
    </div>
  </main>
</body>
</html>
"""


def guide_md_path(spec: GuideSpec) -> Path:
    return DOC_DIR / f"{spec.stem}.md"


def guide_html_name(spec: GuideSpec) -> str:
    return f"{spec.stem}.html"


def guide_pdf_name(spec: GuideSpec) -> str:
    return f"{spec.stem}.pdf"


def read_title(md_text: str, fallback: str) -> str:
    match = re.search(r"^#\s+(.+)$", md_text, re.MULTILINE)
    return match.group(1).strip() if match else fallback


def preprocess_mermaid(text: str) -> str:
    def repl(match: re.Match[str]) -> str:
        body = match.group(1).strip()
        return f'<div class="mermaid">\n{body}\n</div>'

    return re.sub(r"```mermaid\s*\n(.*?)```", repl, text, flags=re.DOTALL)


def markdown_to_body(md_text: str) -> str:
    md_text = preprocess_mermaid(md_text)
    return markdown.markdown(md_text, extensions=MARKDOWN_EXTENSIONS)


def build_web_html(md_text: str, title: str, spec: GuideSpec, generated: str) -> str:
    body = markdown_to_body(md_text)
    return WEB_HTML_TEMPLATE.format(
        html_lang=spec.html_lang,
        title=title,
        brand=spec.brand,
        nav_home=spec.nav_home,
        footer=spec.footer.format(generated=generated),
        body=body,
    )


def build_print_html(md_text: str, title: str, spec: GuideSpec, generated: str) -> str:
    body = markdown_to_body(md_text)
    meta = (
        f"生成于 {generated}"
        if spec.lang.startswith("zh")
        else f"Generated {generated}"
    )
    return PRINT_HTML_TEMPLATE.format(
        html_lang=spec.html_lang,
        title=title,
        meta=meta,
        body=body,
    )


def build_index_html(generated: str) -> str:
    guide_links = []
    for spec in GUIDES:
        html_name = guide_html_name(spec)
        pdf_name = guide_pdf_name(spec)
        guide_links.append(
            f'        <li><a href="{html_name}">{spec.index_label}</a> · '
            f'<a href="{pdf_name}">{spec.pdf_label}</a></li>'
        )
    return INDEX_TEMPLATE.format(generated=generated, guide_links="\n".join(guide_links))


def load_guide(spec: GuideSpec) -> tuple[str, str]:
    path = guide_md_path(spec)
    if not path.is_file():
        raise FileNotFoundError(path)
    raw = path.read_text(encoding="utf-8")
    title = read_title(raw, spec.stem)
    return raw, title
