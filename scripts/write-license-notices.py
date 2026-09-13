"""Bundle dependency license notices from the exact locked Cargo sources."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=ROOT
))
members = set(metadata["workspace_members"])
packages = sorted(
    (package for package in metadata["packages"] if package["id"] not in members),
    key=lambda package: (package["name"], package["version"]),
)
sections = {}
lines = [
    "PecoFence — third-party Rust dependencies",
    "Includes runtime and build/test dependencies from Cargo.lock.",
    "WebView2 Loader has its own LICENSE-WebView2Loader.txt.",
    "",
]
for package in packages:
    label = f'{package["name"]} {package["version"]}'
    lines.append(f'{label}: {package.get("license") or "see upstream license"}')
    directory = Path(package["manifest_path"]).parent
    files = [
        path for path in directory.iterdir()
        if path.is_file() and path.name.lower().startswith(("license", "copying", "notice"))
    ]
    if package.get("license_file"):
        files.append(directory / package["license_file"])
    for path in sorted(set(files)):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace").strip()
        if not text:
            continue
        digest = hashlib.sha256(text.encode()).hexdigest()
        entry = sections.setdefault(digest, {"names": [], "text": text})
        entry["names"].append(f"{label} / {path.name}")
for entry in sections.values():
    lines.extend(("", "=" * 72, "\n".join(entry["names"]), "=" * 72, entry["text"]))
Path(sys.argv[1]).write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f'Bundled {len(sections)} license texts for {len(packages)} dependencies')
