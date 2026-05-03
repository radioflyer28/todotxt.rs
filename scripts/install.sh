#!/usr/bin/env sh
# install.sh — Download and install todotxt-tui for Linux or macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.sh | sh
#   # or clone the repo and run:
#   sh scripts/install.sh
#
# Environment overrides:
#   INSTALL_DIR   Installation directory (default: /usr/local/bin, or ~/bin if not writable)
#   RELEASE_TAG   Specific release tag to install (default: latest)

set -e

REPO="radioflyer28/todotxt.rs"
BINARY_NAME="todotxt-tui"

# ── Detect OS and architecture ────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) ASSET="${BINARY_NAME}-linux-x86_64" ;;
      *)
        echo "Unsupported Linux architecture: $ARCH" >&2
        echo "Pre-built binaries are available for x86_64 only." >&2
        echo "Build from source: https://github.com/${REPO}#10-building-from-source" >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    # macOS ships a universal binary covering both arm64 and x86_64.
    ASSET="${BINARY_NAME}-macos-universal"
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    echo "For Windows, use: irm https://raw.githubusercontent.com/${REPO}/master/scripts/install.ps1 | iex" >&2
    exit 1
    ;;
esac

# ── Resolve install directory ─────────────────────────────────────────────────

if [ -n "$INSTALL_DIR" ]; then
  DEST_DIR="$INSTALL_DIR"
elif [ -w "/usr/local/bin" ]; then
  DEST_DIR="/usr/local/bin"
else
  DEST_DIR="$HOME/.local/bin"
  mkdir -p "$DEST_DIR"
fi

DEST="$DEST_DIR/$BINARY_NAME"

# ── Resolve download URL ──────────────────────────────────────────────────────

if [ -n "$RELEASE_TAG" ]; then
  URL="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
fi

# ── Download ──────────────────────────────────────────────────────────────────

echo "Downloading ${ASSET} ..."

if command -v curl > /dev/null 2>&1; then
  curl -fsSL --progress-bar -o "$DEST" "$URL"
elif command -v wget > /dev/null 2>&1; then
  wget -q --show-progress -O "$DEST" "$URL"
else
  echo "Error: curl or wget is required." >&2
  exit 1
fi

chmod +x "$DEST"

echo ""
echo "Installed: $DEST"
echo "Version:   $($DEST --version 2>/dev/null || echo 'unknown')"

# ── PATH check ────────────────────────────────────────────────────────────────

case ":$PATH:" in
  *":$DEST_DIR:"*) ;;
  *)
    echo ""
    echo "WARNING: $DEST_DIR is not on your PATH."
    echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "  export PATH=\"\$PATH:$DEST_DIR\""
    ;;
esac

# ── Alias suggestion ──────────────────────────────────────────────────────────

echo ""
echo "Tip: add a short alias to your shell profile:"
echo ""
echo "  alias todo='todotxt-tui'"
echo ""
echo "Then just run: todo"
