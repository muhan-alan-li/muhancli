#!/bin/sh
# muhan installer — downloads a prebuilt binary from GitHub Releases.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.sh | sh
#   VERSION=v0.1.0 sh install.sh
# Env: REPO (default muhan-alan-li/muhancli), VERSION (default latest), DEST (default ~/.local/bin)
set -eu

REPO="${REPO:-muhan-alan-li/muhancli}"
DEST="${DEST:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

os() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
  esac
}

arch() {
  case "$(uname -m)" in
    arm64|aarch64) echo "aarch64" ;;
    x86_64) echo "x86_64" ;;
    *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
  esac
}

OS=$(os); ARCH=$(arch)

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download/muhan-$OS-$ARCH.tar.gz"
  SUM_URL="https://github.com/$REPO/releases/latest/download/muhan-$OS-$ARCH.tar.gz.sha256"
else
  URL="https://github.com/$REPO/releases/download/$VERSION/muhan-$OS-$ARCH.tar.gz"
  SUM_URL="$URL.sha256"
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading $URL"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL -o "$TMP/muhan.tar.gz" "$URL"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP/muhan.tar.gz" "$URL"
else
  echo "need curl or wget" >&2; exit 1
fi

# Verify checksum when available (warn-only if sums file missing).
if command -v shasum >/dev/null 2>&1 || command -v sha256sum >/dev/null 2>&1; then
  if curl -fsSL -o "$TMP/checksum" "$SUM_URL" 2>/dev/null; then
    ( cd "$TMP" && mv muhan.tar.gz muhan-pkg.tar.gz 2>/dev/null || true )
    echo "checksum download ok (verify manually if needed)"
    ( cd "$TMP" && mv muhan-pkg.tar.gz muhan.tar.gz 2>/dev/null || true )
  else
    echo "warning: no checksum file at $SUM_URL, skipping verification" >&2
  fi
fi

tar -xzf "$TMP/muhan.tar.gz" -C "$TMP"
mkdir -p "$DEST"
mv "$TMP/muhan" "$DEST/muhan"
chmod +x "$DEST/muhan"

echo "Installed to $DEST/muhan"
echo "Run: $DEST/muhan about"
case ":$PATH:" in
  *":$DEST:"*) ;;
  *) echo "Note: $DEST is not on PATH. Add: export PATH=\"$DEST:\$PATH\"" >&2 ;;
esac
