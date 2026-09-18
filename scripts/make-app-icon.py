"""Write crates/app/assets/pecofence.ico (the executables' embedded icon).

Uses the variant E renderer from make-msix-assets.py (white mark on the indigo plate,
same style as the Square44x44Logo target-size icons) at every size Explorer, the
taskbar and Alt+Tab ask for. Pillow stores each frame PNG-compressed.

    python scripts/make-app-icon.py [output .ico]
"""

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_OUT = HERE.parent / "crates" / "app" / "assets" / "pecofence.ico"
SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256)


def load_renderer():
    spec = importlib.util.spec_from_file_location("make_msix_assets", HERE / "make-msix-assets.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_OUT
    renderer = load_renderer()
    frames = [renderer.render_target(size) for size in SIZES]
    out.parent.mkdir(parents=True, exist_ok=True)
    # The largest frame is the base image; append_images supplies the others so that
    # Pillow does not just downscale the 256 px rendering.
    frames[-1].save(out, format="ICO", sizes=[(s, s) for s in SIZES], append_images=frames[:-1])
    print(f"wrote {out} ({out.stat().st_size} bytes, sizes {', '.join(map(str, SIZES))})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
