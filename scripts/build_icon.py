"""Build java-runner2026.ico from pre-rendered icons in 素材/.

Expected source files (RGBA, exact pixel size):
  icon_16x16.png, icon_32x32.png, icon_64x64.png,
  icon_128x128.png, icon_256x256.png

Intermediate ICO sizes (24, 48) are derived from the nearest native tier.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
SOURCE_DIR = ROOT / "素材"
ASSETS = ROOT / "assets"
MASTER = ASSETS / "java-runner2026-icon.png"
OUT_ICO = ASSETS / "java-runner2026.ico"
OUT_DIR = ASSETS / "icon-sizes"

ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)
NATIVE_SIZES = (16, 32, 64, 128, 256)


def native_source(size: int) -> Path:
    return SOURCE_DIR / f"icon_{size}x{size}.png"


def load_native(size: int) -> Image.Image:
    path = native_source(size)
    if not path.is_file():
        raise SystemExit(f"Source icon not found: {path}")
    img = Image.open(path).convert("RGBA")
    if img.size != (size, size):
        raise SystemExit(f"{path.name} must be {size}x{size}, got {img.size[0]}x{img.size[1]}")
    return img


def sharpen(img: Image.Image) -> Image.Image:
    return img.filter(ImageFilter.UnsharpMask(radius=0.9, percent=120, threshold=2))


def render_icon(size: int, natives: dict[int, Image.Image]) -> Image.Image:
    if size in natives:
        return natives[size].copy()

    if size == 24:
        return natives[32].resize((24, 24), Image.Resampling.LANCZOS)
    if size == 48:
        return natives[64].resize((48, 48), Image.Resampling.LANCZOS)

    raise ValueError(f"No render path for {size}px")


def save_ico(images: dict[int, Image.Image], path: Path) -> None:
    ordered_sizes = sorted((s for s in ICO_SIZES if s in images), reverse=True)
    ordered = [images[s].convert("RGBA") for s in ordered_sizes]
    size_list = [(img.width, img.height) for img in ordered]
    ordered[0].save(path, format="ICO", sizes=size_list, append_images=ordered[1:])


def main() -> None:
    ASSETS.mkdir(parents=True, exist_ok=True)
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    print(f"Source: {SOURCE_DIR}")

    natives: dict[int, Image.Image] = {size: load_native(size) for size in NATIVE_SIZES}

    images: dict[int, Image.Image] = {}
    for size in ICO_SIZES:
        out_path = OUT_DIR / f"icon_{size}.png"
        if size in natives:
            shutil.copy2(native_source(size), out_path)
            icon = Image.open(out_path).convert("RGBA")
            tag = "素材"
        else:
            icon = render_icon(size, natives)
            icon.save(out_path, format="PNG")
            tag = "derived"
        images[size] = icon
        print(f"  {size}x{size}  [{tag}]")

    sharpen(natives[256].resize((512, 512), Image.Resampling.LANCZOS)).save(MASTER, format="PNG")
    save_ico(images, OUT_ICO)
    print(f"Wrote {OUT_ICO} ({OUT_ICO.stat().st_size // 1024} KB)")
    print(f"Wrote {MASTER}")


if __name__ == "__main__":
    main()
