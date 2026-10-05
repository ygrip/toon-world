#!/usr/bin/env sh
set -eu

repo="ygrip/toon-world"
base_release_url="https://github.com/$repo/releases"
version="${TOON_WORLD_VERSION:-latest}"
install_dir="${TOON_WORLD_INSTALL_DIR:-$HOME/.local/bin}"

usage() {
  cat <<'EOF'
Usage: install.sh [--version VERSION] [--install-dir DIR]

Options:
  --version VERSION      Install a specific release, e.g. v0.6.0. Defaults to latest.
  --install-dir DIR      Install directory. Defaults to ~/.local/bin.

Environment:
  TOON_WORLD_VERSION
  TOON_WORLD_INSTALL_DIR
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || { echo "missing value for --version" >&2; exit 2; }
      version="$2"
      shift 2
      ;;
    --install-dir)
      [ "$#" -ge 2 ] || { echo "missing value for --install-dir" >&2; exit 2; }
      install_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$(uname -s)" in
  Darwin) os="macos" ;;
  Linux) os="linux" ;;
  *)
    echo "Unsupported OS: $(uname -s). Use install.ps1 on Windows." >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64) arch="x86_64" ;;
  arm64|aarch64) arch="aarch64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

asset="toon-world-$os-$arch"

case "$version" in
  latest|"") release_base="$base_release_url/latest/download" ;;
  v*) release_base="$base_release_url/download/$version" ;;
  *) release_base="$base_release_url/download/v$version" ;;
esac

if command -v curl >/dev/null 2>&1; then
  fetch() {
    curl -fL --retry 3 --retry-delay 1 --connect-timeout 10 "$1" -o "$2"
  }
elif command -v wget >/dev/null 2>&1; then
  fetch() {
    wget -q --tries=3 --timeout=10 "$1" -O "$2"
  }
else
  echo "curl or wget is required to install toon-world." >&2
  exit 1
fi

tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t toon-world)"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

echo "Detected $os $arch -> $asset"
echo "Downloading toon-world..."

fetch "$release_base/$asset" "$tmp_dir/$asset"
fetch "$release_base/SHA256SUMS" "$tmp_dir/SHA256SUMS"

expected="$(awk -v asset="$asset" '$2 == asset || $2 == "*" asset { print $1; exit }' "$tmp_dir/SHA256SUMS")"
if [ -z "$expected" ]; then
  echo "Checksum for $asset was not found in SHA256SUMS." >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$tmp_dir/$asset" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "$tmp_dir/$asset" | awk '{print $1}')"
else
  echo "sha256sum or shasum is required to verify the download." >&2
  exit 1
fi

if [ "$actual" != "$expected" ]; then
  echo "Checksum mismatch for $asset." >&2
  echo "Expected: $expected" >&2
  echo "Actual:   $actual" >&2
  exit 1
fi

mkdir -p "$install_dir"
tmp_target="$install_dir/.toon-world-install-$$"
cp "$tmp_dir/$asset" "$tmp_target"
chmod 755 "$tmp_target"
mv -f "$tmp_target" "$install_dir/toon-world"

echo "Installed toon-world to $install_dir/toon-world"

case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *) echo "Add $install_dir to PATH to run toon-world from any directory." ;;
esac
