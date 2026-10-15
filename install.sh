#!/bin/sh
# Install a prebuilt release without sudo. Works with sh and bash.
# Configuration: VERSION, DEST, REPO, NO_MODIFY_PATH=1.
set -eu

fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

REPO="${REPO:-muhan-alan-li/muhancli}"
VERSION="${VERSION:-latest}"
DEST="${DEST:-$HOME/.local/bin}"
case "$DEST" in
    /*) ;;
    *) fail 'DEST must be an absolute path.' ;;
esac
case "$(uname -s)" in
    Darwin) OS=macos ;;
    Linux) OS=linux ;;
    *) fail 'Supported systems: Linux and macOS. On Windows, use install.ps1 in PowerShell.' ;;
esac
case "$(uname -m)" in
    arm64|aarch64) ARCH=aarch64 ;;
    x86_64|amd64) ARCH=x86_64 ;;
    *) fail 'Supported architectures: x86_64 and ARM64.' ;;
esac

if command -v curl >/dev/null 2>&1; then
    download() { curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    download() { wget --https-only --tries=3 --timeout=30 -qO "$2" "$1"; }
else
    fail 'Install curl or wget, then try again.'
fi
if command -v sha256sum >/dev/null 2>&1; then
    checksum() { sha256sum "$1"; }
elif command -v shasum >/dev/null 2>&1; then
    checksum() { shasum -a 256 "$1"; }
else
    fail 'A SHA-256 tool is required (sha256sum or shasum).'
fi

TMP=$(mktemp -d)
STAGED=
cleanup() {
    rm -rf "$TMP"
    if [ -n "$STAGED" ]; then rm -f "$STAGED"; fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Resolve latest once, so the archive and checksum always come from one version.
if [ "$VERSION" = latest ]; then
    download "https://api.github.com/repos/$REPO/releases/latest" "$TMP/release.json" ||
        fail 'Could not find the latest release. Check the repository releases page or set VERSION=vX.Y.Z.'
    VERSION=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$TMP/release.json" | head -n 1)
fi
case "$VERSION" in
    v[0-9]*) ;;
    *) fail 'VERSION must be a release tag such as v0.1.0.' ;;
esac
case "$VERSION" in *[!A-Za-z0-9._-]*) fail 'Invalid release tag.' ;; esac

ASSET="muhan-$OS-$ARCH.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
printf 'Installing muhan %s for %s/%s…\n' "$VERSION" "$OS" "$ARCH"
download "$URL" "$TMP/$ASSET" || fail "Could not download $ASSET for $VERSION. Check that the release supports your platform."
download "$URL.sha256" "$TMP/expected.sha256" || fail 'Could not download the required checksum. Nothing was installed.'
WANT=$(awk 'NR == 1 {print $1}' "$TMP/expected.sha256")
[ "${#WANT}" -eq 64 ] || fail 'Invalid SHA-256 checksum file.'
case "$WANT" in *[!0-9a-fA-F]*) fail 'Invalid SHA-256 checksum file.' ;; esac
WANT=$(printf '%s' "$WANT" | tr 'A-F' 'a-f')
GOT=$(checksum "$TMP/$ASSET" | awk '{print $1}')
[ "$WANT" = "$GOT" ] || fail 'Checksum mismatch. Nothing was installed.'

# Extract only the executable, not arbitrary paths from the archive.
tar -xzf "$TMP/$ASSET" -C "$TMP" muhan
[ -f "$TMP/muhan" ] && [ ! -L "$TMP/muhan" ] || fail 'Release archive is missing a regular muhan executable.'
chmod +x "$TMP/muhan"
"$TMP/muhan" --version || fail 'The downloaded binary cannot run on this system.'
mkdir -p "$DEST"
STAGED=$(mktemp "$DEST/.muhan.XXXXXX")
cp "$TMP/muhan" "$STAGED"
chmod 755 "$STAGED"
mv -f "$STAGED" "$DEST/muhan"
STAGED=

# A piped installer cannot change its parent shell's PATH. Configure future shells
# and show the command needed to use muhan immediately in the current shell.
QUOTED_DEST=$(printf '%s' "$DEST" | sed "s/'/'\\\\''/g")
PATH_LINE="export PATH='$QUOTED_DEST':\$PATH"
add_to_profile() {
    if [ -f "$1" ] && grep -Fqx "$PATH_LINE" "$1"; then return; fi
    if printf '\n# muhan portfolio CLI\n%s\n' "$PATH_LINE" >> "$1"; then
        printf 'Added PATH setup to %s\n' "$1"
    else
        printf 'Could not update %s; add this manually: %s\n' "$1" "$PATH_LINE" >&2
    fi
}
case ":$PATH:" in
    *":$DEST:"*) ;;
    *)
        if [ "${NO_MODIFY_PATH:-0}" != 1 ]; then
            case "${SHELL:-}" in
                */zsh) add_to_profile "${ZDOTDIR:-$HOME}/.zshrc" ;;
                */bash)
                    add_to_profile "$HOME/.bashrc"
                    if [ -f "$HOME/.bash_profile" ]; then
                        add_to_profile "$HOME/.bash_profile"
                    elif [ -f "$HOME/.bash_login" ]; then
                        add_to_profile "$HOME/.bash_login"
                    else
                        add_to_profile "$HOME/.profile"
                    fi
                    ;;
                */fish) printf 'For fish, run: fish_add_path "%s"\n' "$DEST" ;;
                *) add_to_profile "$HOME/.profile" ;;
            esac
        fi
        printf '\nTo use muhan in this terminal, run:\n  %s\n' "$PATH_LINE"
        ;;
esac
printf '\nInstalled to %s/muhan\nExplore with: muhan about\nMore commands: muhan --help\n' "$DEST"
