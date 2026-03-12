#!/usr/bin/env python3
"""Collects normalized cross-platform release assets from Tauri build output."""

from __future__ import annotations

import argparse
import shutil
import sys
import tarfile
import zipfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--platform", choices=["linux", "macos", "windows"], required=True)
    parser.add_argument("--arch", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output-dir", required=True)
    return parser.parse_args()


def bundle_roots(target: str) -> list[Path]:
    return [Path("target") / target / "release" / "bundle", Path("target") / "release" / "bundle"]


def find_first(patterns: list[str], roots: list[Path]) -> Path | None:
    for root in roots:
        if not root.exists():
            continue
        for pattern in patterns:
            matches = sorted(root.glob(pattern))
            if matches:
                return matches[0]
    return None


def require_copy(roots: list[Path], patterns: list[str], destination: Path) -> None:
    source = find_first(patterns, roots)
    if source is None:
        raise FileNotFoundError(f"Could not find any asset matching {patterns}")
    shutil.copy2(source, destination)


def package_cli_binary(platform: str, arch: str, target: str, output_dir: Path) -> None:
    binary_name = "haptic-harmony-simulation.exe" if platform == "windows" else "haptic-harmony-simulation"
    candidates = [Path("target") / target / "release" / binary_name, Path("target") / "release" / binary_name]
    binary = next((candidate for candidate in candidates if candidate.exists()), None)
    if binary is None:
        raise FileNotFoundError(f"Could not find CLI binary at any of: {candidates}")

    if platform == "windows":
        archive_path = output_dir / f"haptic-harmony-simulation-cli-{platform}-{arch}.zip"
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            archive.write(binary, arcname=binary.name)
    else:
        archive_path = output_dir / f"haptic-harmony-simulation-cli-{platform}-{arch}.tar.gz"
        with tarfile.open(archive_path, "w:gz") as archive:
            archive.add(binary, arcname=binary.name)


def main() -> int:
    args = parse_args()
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    roots = bundle_roots(args.target)

    if args.platform == "macos":
        require_copy(roots, ["dmg/*.dmg", "macos/*.dmg"], output_dir / f"haptic-harmony-simulation-macos-{args.arch}.dmg")
    elif args.platform == "windows":
        require_copy(roots, ["nsis/*.exe"], output_dir / f"haptic-harmony-simulation-windows-{args.arch}.exe")
        require_copy(roots, ["msi/*.msi", "wix/*.msi"], output_dir / f"haptic-harmony-simulation-windows-{args.arch}.msi")
    else:
        require_copy(roots, ["appimage/*.AppImage", "appimage/*.appimage"], output_dir / f"haptic-harmony-simulation-linux-{args.arch}.AppImage")
        require_copy(roots, ["deb/*.deb"], output_dir / f"haptic-harmony-simulation-linux-{args.arch}.deb")
        rpm = find_first(["rpm/*.rpm"], roots)
        if rpm is not None:
            shutil.copy2(rpm, output_dir / f"haptic-harmony-simulation-linux-{args.arch}.rpm")

    package_cli_binary(args.platform, args.arch, args.target, output_dir)
    print(f"Collected release assets into {output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())