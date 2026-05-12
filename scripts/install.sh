#!/usr/bin/env sh
# install.sh — Download and install todotxt-tui (TUI) and/or todotxt (CLI) for Linux/macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.sh | sh
#   curl -fsSL ... | sh -s -- --cli    # CLI only
#   curl -fsSL ... | sh -s -- --both   # TUI + CLI
#   # or clone the repo and run directly:
#   sh scripts/install.sh [--tui|--cli|--both]
#   sh scripts/install.sh --local       # install from local build
#
# Options:
#   --tui    Install todotxt-tui only (default)
#   --cli    Install todotxt only
#   --both   Install both todotxt-tui and todotxt
#   --local  Copy locally-built binaries from BUILD_DIR instead of downloading
#
# Environment overrides:
#   INSTALL_DIR   Installation directory (default: /usr/local/bin, or ~/.local/bin)
#   RELEASE_TAG   Specific release tag (default: latest)
#   BUILD_DIR     Local build output directory for --local (default: ./target/release)

set -e

REPO="radioflyer28/todotxt.rs"

# ── Parse arguments ───────────────────────────────────────────────────────────

INSTALL_TUI=1
INSTALL_CLI=0
LOCAL=0

while [ $# -gt 0 ]; do
    case "$1" in
        --tui)   INSTALL_TUI=1; INSTALL_CLI=0 ;;
        --cli)   INSTALL_TUI=0; INSTALL_CLI=1 ;;
        --both)  INSTALL_TUI=1; INSTALL_CLI=1 ;;
        --local) LOCAL=1 ;;
        *) echo "Unknown option: $1. Use --tui (default), --cli, --both, or --local." >&2; exit 1 ;;
    esac
    shift
done

# ── Detect OS and architecture (skipped for --local) ─────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$LOCAL" = "0" ]; then
    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) SUFFIX="linux-x86_64" ;;
                *)
                    echo "Unsupported Linux architecture: $ARCH" >&2
                    echo "Pre-built binaries are available for x86_64 only." >&2
                    echo "Build from source: https://github.com/${REPO}#10-building-from-source" >&2
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            # macOS ships universal binaries covering both arm64 and x86_64.
            SUFFIX="macos-universal"
            ;;
        *)
            echo "Unsupported OS: $OS" >&2
            echo "For Windows, use: irm https://raw.githubusercontent.com/${REPO}/master/scripts/install.ps1 | iex" >&2
            exit 1
            ;;
    esac
fi

# ── Resolve install directory ─────────────────────────────────────────────────

if [ -n "$INSTALL_DIR" ]; then
    DEST_DIR="$INSTALL_DIR"
elif [ -w "/usr/local/bin" ]; then
    DEST_DIR="/usr/local/bin"
else
    DEST_DIR="$HOME/.local/bin"
    mkdir -p "$DEST_DIR"
fi

# ── Download helper ───────────────────────────────────────────────────────────

download_binary() {
    _asset="$1"
    _dest="$2"

    if [ -n "$RELEASE_TAG" ]; then
        _url="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${_asset}"
    else
        _url="https://github.com/${REPO}/releases/latest/download/${_asset}"
    fi

    echo "Downloading ${_asset} ..."
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL --progress-bar -o "$_dest" "$_url"
    elif command -v wget > /dev/null 2>&1; then
        wget -q --show-progress -O "$_dest" "$_url"
    else
        echo "Error: curl or wget is required." >&2
        exit 1
    fi

    chmod +x "$_dest"
    echo "Installed: $_dest"
    echo "Version:   $($_dest --version 2>/dev/null || echo 'unknown')"
    echo ""
}

# ── Local copy helper ─────────────────────────────────────────────────────────

copy_local_binary() {
    _name="$1"
    _dest="$2"
    _build_dir="${BUILD_DIR:-./target/release}"
    _src="${_build_dir}/${_name}"

    if [ ! -f "$_src" ]; then
        echo "Error: local binary not found: $_src" >&2
        echo "Build first with: cargo build --release" >&2
        exit 1
    fi

    echo "Copying ${_src} ..."
    cp "$_src" "$_dest"
    chmod +x "$_dest"
    echo "Installed: $_dest"
    echo "Version:   $($_dest --version 2>/dev/null || echo 'unknown')"
    echo ""
}

# ── Install requested binaries ────────────────────────────────────────────────

if [ "$INSTALL_TUI" = "1" ]; then
    if [ "$LOCAL" = "1" ]; then
        copy_local_binary "todotxt-tui" "$DEST_DIR/todotxt-tui"
    else
        download_binary "todotxt-tui-${SUFFIX}" "$DEST_DIR/todotxt-tui"
    fi
fi

if [ "$INSTALL_CLI" = "1" ]; then
    if [ "$LOCAL" = "1" ]; then
        copy_local_binary "todotxt" "$DEST_DIR/todotxt"
    else
        download_binary "todotxt-${SUFFIX}" "$DEST_DIR/todotxt"
    fi
fi

# ── PATH check ────────────────────────────────────────────────────────────────

case ":$PATH:" in
    *":$DEST_DIR:"*) ;;
    *)
        echo "WARNING: $DEST_DIR is not on your PATH."
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:$DEST_DIR\""
        echo ""
        ;;
esac

# ── Alias suggestion ──────────────────────────────────────────────────────────

echo "Tip: add short aliases to your shell profile:"
echo ""
if [ "$INSTALL_TUI" = "1" ]; then
    echo "  alias todo='todotxt-tui'"
fi
if [ "$INSTALL_CLI" = "1" ]; then
    echo "  alias td='todotxt'"
fi
echo ""
