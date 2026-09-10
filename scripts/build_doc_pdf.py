"""Build PDF files from doc/guide_*.md into dist/doc/.

Thin wrapper around build_docs.py --pdf-only (kept for README compatibility).
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    script = ROOT / "scripts" / "build_docs.py"
    result = subprocess.run([sys.executable, str(script), "--pdf-only"], check=False)
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
