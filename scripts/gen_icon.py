#!/usr/bin/env python3
"""Generate macOS .icns icon from text-based "S" design.

Uses Pillow to render text with system fonts, producing:
  - icon.icns (macOS app bundle)
  - icon_embedded.png (128x128, for binary embedding)
  - icon_preview.png (512x512, for visual inspection)
"""
import os
import sys
import subprocess
import tempfile

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    subprocess.check_call([sys.executable, "-m", "pip", "install", "Pillow"])
    from PIL import Image, ImageDraw, ImageFont

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))


def draw_icon(size: int) -> Image.Image:
    """Draw green squircle with white bold "S" matching UI toolbar style."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 10% padding per Apple HIG
    margin = int(size * 0.10)
    corner = int(size * 0.22)

    # Green squircle background
    draw.rounded_rectangle(
        [margin, margin, size - margin - 1, size - margin - 1],
        radius=corner,
        fill=(0, 180, 120, 255),
    )

    # White bold "S" — try system fonts in order
    font_size = int(size * 0.62)
    font = None
    for name in [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Supplemental/Helvetica Bold.ttf",
        "/Library/Fonts/Arial Bold.ttf",
        "/System/Library/Fonts/SFNSMono.ttf",
    ]:
        try:
            font = ImageFont.truetype(name, font_size)
            break
        except (OSError, IOError):
            continue

    if font is None:
        font = ImageFont.load_default()

    # Center the "S" text
    bbox = draw.textbbox((0, 0), "S", font=font)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    x = (size - tw) / 2 - bbox[0]
    y = (size - th) / 2 - bbox[1]
    draw.text((x, y), "S", fill=(255, 255, 255, 255), font=font)

    return img


def generate_icns(output_path: str):
    """Generate .icns file using macOS iconutil."""
    with tempfile.TemporaryDirectory() as tmpdir:
        iconset = os.path.join(tmpdir, "SerialRUN.iconset")
        os.makedirs(iconset)

        sizes = {
            "icon_16x16.png": 16,
            "icon_16x16@2x.png": 32,
            "icon_32x32.png": 32,
            "icon_32x32@2x.png": 64,
            "icon_128x128.png": 128,
            "icon_128x128@2x.png": 256,
            "icon_256x256.png": 256,
            "icon_256x256@2x.png": 512,
            "icon_512x512.png": 512,
            "icon_512x512@2x.png": 1024,
        }

        for name, sz in sizes.items():
            img = draw_icon(sz)
            img.save(os.path.join(iconset, name))

        subprocess.run(
            ["iconutil", "-c", "icns", iconset, "-o", output_path],
            check=True,
        )
        print(f"Generated: {output_path}")


def generate_embedded(dst_dir: str):
    """Generate 128x128 PNG for binary embedding."""
    img = draw_icon(128)
    dst = os.path.join(dst_dir, "icon_embedded.png")
    img.save(dst)
    print(f"Generated: {dst}")


def generate_preview(dst_dir: str):
    """Generate 512x512 PNG for visual inspection."""
    img = draw_icon(512)
    dst = os.path.join(dst_dir, "icon_preview.png")
    img.save(dst)
    print(f"Generated: {dst}")


if __name__ == "__main__":
    project_root = os.path.dirname(SCRIPT_DIR)
    gui_src = os.path.join(project_root, "crates", "serialrun-gui")
    app_resources = os.path.join(
        project_root, "target", "release",
        "SerialRUN.app", "Contents", "Resources"
    )

    args = sys.argv[1:]
    if not args:
        os.makedirs(app_resources, exist_ok=True)
        generate_icns(os.path.join(app_resources, "icon.icns"))
        generate_embedded(gui_src)
        generate_preview(SCRIPT_DIR)
    elif args[0] == "--icns":
        os.makedirs(os.path.dirname(args[1]) or ".", exist_ok=True)
        generate_icns(args[1])
    elif args[0] == "--embedded":
        generate_embedded(gui_src)
    elif args[0] == "--preview":
        generate_preview(SCRIPT_DIR)
    else:
        os.makedirs(os.path.dirname(args[0]) or ".", exist_ok=True)
        generate_icns(args[0])
