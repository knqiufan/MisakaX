"""Trace PNG bolts to filled SVG paths with rounded joins."""

from __future__ import annotations

from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from skimage.measure import approximate_polygon, find_contours
from skimage.morphology import binary_closing, disk

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src" / "assets" / "brand" / "misakax-logo.png"
CLEAN = ROOT / "src" / "assets" / "brand" / "misakax-logo-clean.png"
OUT_SVG = ROOT / "src" / "assets" / "brand" / "misakax-logo.svg"

CREAM = (254, 245, 234, 255)
CREAM_HEX = "#FEF5EA"
BOLT_HEX = "#363635"
RADIUS_PX = 440.0
VIEW = 128


def load_master() -> Image.Image:
    """Prefer clean RGBA; rebuild from raster if needed."""
    if CLEAN.exists():
        img = Image.open(CLEAN).convert("RGBA")
        if img.getpixel((0, 0))[3] == 0:
            return img
    # Rebuild from whatever is in SRC
    src = Image.open(SRC).convert("RGBA")
    # If already clean, keep
    if src.getpixel((0, 0))[3] == 0 and src.getpixel((512, 512))[0] > 240:
        src.save(CLEAN)
        return src
    return rebuild_from_rgb()


def rebuild_from_rgb() -> Image.Image:
    im = Image.open(SRC).convert("RGB")
    w, h = im.size
    cx, cy = w / 2, h / 2
    base = im.convert("RGBA")
    mask = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(mask)
    draw.ellipse(
        [cx - RADIUS_PX, cy - RADIUS_PX, cx + RADIUS_PX, cy + RADIUS_PX],
        fill=255,
    )
    mask = mask.filter(ImageFilter.GaussianBlur(1.5))
    bp = base.load()
    mp = mask.load()
    for y in range(h):
        for x in range(w):
            a = mp[x, y]
            if a == 0:
                bp[x, y] = (0, 0, 0, 0)
                continue
            r, g, b, _ = bp[x, y]
            is_bolt = r < 100 and g < 100 and b < 100
            is_cream = r > 230 and g > 220 and b > 200 and (r - b) > 5
            if not is_bolt and not is_cream:
                bp[x, y] = (CREAM[0], CREAM[1], CREAM[2], a)
            else:
                bp[x, y] = (r, g, b, a)
    base.save(CLEAN)
    base.save(SRC)
    return base


def contour_to_path(contour: np.ndarray, scale: float, tolerance: float) -> str:
    approx = approximate_polygon(contour, tolerance=tolerance)
    parts: list[str] = []
    for i, (y, x) in enumerate(approx):
        cmd = "M" if i == 0 else "L"
        parts.append(f"{cmd}{x * scale:.2f} {y * scale:.2f}")
    parts.append("Z")
    return " ".join(parts)


def main() -> None:
    img = load_master()
    # Ensure SRC is the clean transparent master
    img.save(SRC)
    img.save(CLEAN)

    arr = np.array(img)
    bolt = (
        (arr[:, :, 0] < 100)
        & (arr[:, :, 1] < 100)
        & (arr[:, :, 2] < 100)
        & (arr[:, :, 3] > 128)
    )
    bolt = binary_closing(bolt, disk(2))
    scale = VIEW / float(bolt.shape[1])
    radius = RADIUS_PX * scale

    contours = sorted(
        find_contours(bolt.astype(float), 0.5),
        key=len,
        reverse=True,
    )[:2]
    # Sort left then right by centroid x
    contours = sorted(contours, key=lambda c: float(c[:, 1].mean()))
    paths = [contour_to_path(c, scale, tolerance=1.6) for c in contours]

    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {VIEW} {VIEW}" fill="none">\n'
        f'  <circle cx="64" cy="64" r="{radius:.2f}" fill="{CREAM_HEX}"/>\n'
        f'  <path fill="{BOLT_HEX}" stroke="{BOLT_HEX}" stroke-width="2.5" '
        f'stroke-linejoin="round" stroke-linecap="round" d="{paths[0]}"/>\n'
        f'  <path fill="{BOLT_HEX}" stroke="{BOLT_HEX}" stroke-width="2.5" '
        f'stroke-linejoin="round" stroke-linecap="round" d="{paths[1]}"/>\n'
        f"</svg>\n"
    )
    OUT_SVG.write_text(svg, encoding="utf-8")
    print(f"wrote {OUT_SVG}")
    print(f"points {[p.count('L') + 1 for p in paths]} r={radius:.2f}")


if __name__ == "__main__":
    main()
