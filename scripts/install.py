#!/usr/bin/env python3
"""Install the latest toon-world standalone binary for this OS and architecture."""

from __future__ import annotations

import argparse
import hashlib
import os
import platform
import shutil
import stat
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path
from typing import Optional, Tuple

REPOSITORY = "ygrip/toon-world"
BASE_RELEASE_URL = f"https://github.com/{REPOSITORY}/releases"


def detect_asset() -> Tuple[str, str]:
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "darwin":
        os_name = "macos"
    elif system == "linux":
        os_name = "linux"
    elif system == "windows":
        os_name = "windows"
    else:
        raise SystemExit(f"Unsupported operating system: {platform.system()}")

    if machine in {"x86_64", "amd64"}:
        arch = "x86_64"
    elif machine in {"aarch64", "arm64"}:
        arch = "aarch64"
    else:
        raise SystemExit(f"Unsupported architecture: {platform.machine()}")

    suffix = ".exe" if os_name == "windows" else ""
    return f"toon-world-{os_name}-{arch}{suffix}", suffix


def release_base(version: Optional[str]) -> str:
    if not version or version == "latest":
        return f"{BASE_RELEASE_URL}/latest/download"
    normalized = version if version.startswith("v") else f"v{version}"
    return f"{BASE_RELEASE_URL}/download/{normalized}"


def download(url: str, destination: Path) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": "toon-world-installer"})
    try:
        with urllib.request.urlopen(request) as response, destination.open("wb") as output:
            shutil.copyfileobj(response, output)
    except urllib.error.HTTPError as error:
        raise SystemExit(f"Download failed ({error.code}): {url}") from error
    except urllib.error.URLError as error:
        raise SystemExit(f"Download failed: {error.reason}") from error


def expected_checksum(checksums: str, asset: str) -> str:
    for line in checksums.splitlines():
        parts = line.strip().split()
        if len(parts) >= 2 and parts[-1].lstrip("*") == asset:
            return parts[0].lower()
    raise SystemExit(f"Checksum for {asset} was not found in SHA256SUMS")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def default_install_dir() -> Path:
    configured = os.environ.get("TOON_WORLD_INSTALL_DIR")
    if configured:
        return Path(configured).expanduser()
    return Path.home() / ".local" / "bin"


def path_contains(directory: Path) -> bool:
    target = os.path.normcase(os.path.abspath(str(directory)))
    return any(
        os.path.normcase(os.path.abspath(part)) == target
        for part in os.environ.get("PATH", "").split(os.pathsep)
        if part
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Install the toon-world standalone binary for this OS and architecture."
    )
    parser.add_argument(
        "--version",
        default=os.environ.get("TOON_WORLD_VERSION", "latest"),
        help="Release version to install, e.g. v0.6.0. Defaults to latest.",
    )
    parser.add_argument(
        "--install-dir",
        type=Path,
        default=default_install_dir(),
        help="Destination directory. Defaults to ~/.local/bin or TOON_WORLD_INSTALL_DIR.",
    )
    args = parser.parse_args()

    asset, suffix = detect_asset()
    base = release_base(args.version)
    install_dir = args.install_dir.expanduser().resolve()
    destination = install_dir / f"toon-world{suffix}"

    print(f"Detected {platform.system()} {platform.machine()} -> {asset}")
    print(f"Installing to {destination}")

    with tempfile.TemporaryDirectory(prefix="toon-world-install-") as temp:
        temp_dir = Path(temp)
        binary = temp_dir / asset
        checksum_file = temp_dir / "SHA256SUMS"

        download(f"{base}/{asset}", binary)
        download(f"{base}/SHA256SUMS", checksum_file)

        expected = expected_checksum(checksum_file.read_text(encoding="utf-8"), asset)
        actual = sha256(binary)
        if actual != expected:
            raise SystemExit(
                f"Checksum mismatch for {asset}: expected {expected}, got {actual}"
            )

        install_dir.mkdir(parents=True, exist_ok=True)
        temporary_destination = install_dir / f".toon-world-install{suffix}"
        shutil.copyfile(binary, temporary_destination)

        if platform.system().lower() != "windows":
            mode = temporary_destination.stat().st_mode
            temporary_destination.chmod(
                mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
            )

        os.replace(temporary_destination, destination)

    print(f"Installed toon-world to {destination}")
    if not path_contains(install_dir):
        print(f"Add {install_dir} to PATH to run toon-world from any directory.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
