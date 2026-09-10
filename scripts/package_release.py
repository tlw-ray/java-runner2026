"""Assemble an out-of-the-box release directory for java-runner.

Output layout (dist/java-runner/):
  java-runner.exe
  java-runner.toml
  config/java-runner.toml
  doc/*.html + *.pdf + index.html   (from doc/guide_*.md; no README)
  查看文档.bat / 更新java-runner.bat
"""

from __future__ import annotations

import shutil
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
_SCRIPTS = ROOT / "scripts"
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))

from build_docs import build_guides_html, build_pdf  # noqa: E402
from doc_common import (  # noqa: E402
    DIST_DOC_DIR,
    GUIDES,
    guide_html_name,
    guide_pdf_name,
)

CONFIG_SRC = ROOT / "config"
DIST_ROOT = ROOT / "dist" / "java-runner"

RELEASE_INDEX_TEMPLATE = """<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>java-runner 发布包说明</title>
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
    .steps li {{ margin: .4rem 0; }}
    .muted {{ color: #6b7280; font-size: .9rem; }}
  </style>
</head>
<body>
  <header>
    <h1 style="margin:0 0 .5rem">java-runner 发布包</h1>
    <p style="margin:0;opacity:.9">Windows JVM 扫描与 Java 程序启动器 · v{version}</p>
  </header>
  <main>
    <div class="card">
      <h2>快速开始</h2>
      <ol class="steps">
        <li>确保本机已安装 Java（JRE/JDK）</li>
        <li>双击 <code>java-runner.exe</code> 运行（程序会自动切换 UTF-8 并按 exe 所在目录加载配置）</li>
        <li>按提示选择 JVM，程序将执行 <code>java-runner.toml</code> 中配置的启动命令</li>
        <li>修改启动参数：编辑根目录 <code>java-runner.toml</code> 后重新运行</li>
      </ol>
      <p class="muted">界面语言随 Windows 区域自动选择（zh* 中文，否则英文）；可用 <code>JAVA_RUNNER_LANG</code> 覆盖。</p>
    </div>
    <div class="card">
      <h2>目录说明</h2>
      <ul>
        <li><code>java-runner.exe</code> — 主程序</li>
        <li><code>java-runner.toml</code> — 启动配置（推荐编辑此文件）</li>
        <li><code>config/</code> — 配置备份目录</li>
        <li><code>doc/</code> — HTML / PDF 使用文档（中/英）</li>
      </ul>
    </div>
    <div class="card">
      <h2>详细文档</h2>
      <ul>
{doc_links}
      </ul>
    </div>
    <div class="card">
      <h2>常用命令</h2>
      <pre><code>java-runner.exe --scan-only
java-runner.exe --select 1
java-runner.exe --json
java-runner.exe --config java-runner.toml
java-runner.exe --no-pause</code></pre>
    </div>
  </main>
</body>
</html>
"""


def read_version() -> str:
    cargo = ROOT / "Cargo.toml"
    for line in cargo.read_text(encoding="utf-8").splitlines():
        if line.strip().startswith("version"):
            return line.split("=", 1)[1].strip().strip('"')
    return "0.0.0"


def find_exe() -> Path:
    candidates = [
        ROOT / "release" / "java-runner.exe",
        ROOT / "target" / "release" / "java-runner.exe",
        ROOT / "java-runner.exe",
    ]
    for path in candidates:
        if path.is_file():
            return path
    raise SystemExit(
        "未找到 java-runner.exe。请先执行 cargo build --release，或将 exe 放入 release/ 目录。"
    )


def dist_doc_cache_ready() -> bool:
    if not DIST_DOC_DIR.is_dir():
        return False
    return all(
        (DIST_DOC_DIR / guide_html_name(spec)).is_file()
        and (DIST_DOC_DIR / guide_pdf_name(spec)).is_file()
        for spec in GUIDES
    )


def build_release_docs(out_doc: Path, generated: str) -> None:
    out_doc.mkdir(parents=True, exist_ok=True)
    if dist_doc_cache_ready():
        for spec in GUIDES:
            for name in (guide_html_name(spec), guide_pdf_name(spec)):
                shutil.copy2(DIST_DOC_DIR / name, out_doc / name)
        print("  doc/ (reused dist/doc cache)")
    else:
        build_guides_html(out_doc, generated)
        build_pdf(out_doc, generated)

    html_links = []
    pdf_links = []
    for spec in GUIDES:
        html_links.append(
            f'        <li><a href="{guide_html_name(spec)}">{spec.index_label}</a></li>'
        )
        pdf_links.append(
            f'        <li><a href="{guide_pdf_name(spec)}">{spec.pdf_label}</a></li>'
        )

    doc_links = "\n".join(html_links + pdf_links)
    release_index = RELEASE_INDEX_TEMPLATE.format(version=read_version(), doc_links=doc_links)
    (out_doc / "index.html").write_text(release_index, encoding="utf-8")
    print("  doc/index.html")


def write_bat_files(dist: Path) -> None:
    (dist / "查看文档.bat").write_text(
        "@echo off\r\n"
        "cd /d \"%~dp0\"\r\n"
        "start \"\" \"doc\\index.html\"\r\n",
        encoding="utf-8",
    )
    (dist / "更新java-runner.bat").write_text(
        "@echo off\r\n"
        "chcp 65001 >nul\r\n"
        "cd /d \"%~dp0\"\r\n"
        "echo 正在结束 java-runner.exe ...\r\n"
        "taskkill /F /IM java-runner.exe >nul 2>&1\r\n"
        "timeout /t 1 /nobreak >nul\r\n"
        "if exist \"java-runner.new.exe\" (\r\n"
        "  move /Y \"java-runner.new.exe\" \"java-runner.exe\"\r\n"
        "  echo 已更新 java-runner.exe\r\n"
        ") else (\r\n"
        "  echo 未找到 java-runner.new.exe\r\n"
        "  echo 请将新版本 exe 命名为 java-runner.new.exe 放在本目录后重新运行此脚本\r\n"
        ")\r\n"
        "pause\r\n",
        encoding="utf-8",
    )
    print("  查看文档.bat")
    print("  更新java-runner.bat")


def package(dist: Path) -> None:
    generated = datetime.now().strftime("%Y-%m-%d %H:%M")
    exe_src = find_exe()

    if dist.exists():
        shutil.rmtree(dist)
    dist.mkdir(parents=True)

    print(f"Packaging -> {dist}")
    print(f"  exe: {exe_src.relative_to(ROOT)}")

    shutil.copy2(exe_src, dist / "java-runner.exe")
    print("  java-runner.exe")

    config_file = CONFIG_SRC / "java-runner.toml"
    if not config_file.is_file():
        raise SystemExit(f"缺少配置文件: {config_file}")

    shutil.copy2(config_file, dist / "java-runner.toml")
    (dist / "config").mkdir(exist_ok=True)
    shutil.copy2(config_file, dist / "config" / "java-runner.toml")
    print("  java-runner.toml")
    print("  config/java-runner.toml")

    build_release_docs(dist / "doc", generated)
    write_bat_files(dist)

    print(f"\n完成。发布目录: {dist}")
    print("用户解压/复制整个目录即可使用。")


def main() -> None:
    out = DIST_ROOT
    if len(sys.argv) > 1:
        out = Path(sys.argv[1]).resolve()
    package(out)


if __name__ == "__main__":
    main()
