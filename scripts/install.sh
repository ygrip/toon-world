#!/usr/bin/env sh
set -eu

case "$(uname -s)" in
  Darwin|Linux) ;;
  *) echo "toon-world installer supports macOS and Linux only; use install.ps1 on Windows." >&2; exit 1 ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  if ! command -v curl >/dev/null 2>&1; then
    echo "cargo is required. Install Rust from https://rustup.rs/." >&2
    exit 1
  fi
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

cargo build --release
install_dir=${TOON_WORLD_INSTALL_DIR:-"$HOME/.local/bin"}
mkdir -p "$install_dir"
install -m 755 target/release/toon-world "$install_dir/toon-world"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) echo "Installed to ${install_dir}. Add it to PATH to run toon-world from any directory." ;;
esac
