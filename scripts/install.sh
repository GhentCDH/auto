#!/bin/sh
# Install (or update) the `auto` CLI from the GitHub releases.
#
#   curl -fsSL https://raw.githubusercontent.com/GhentCDH/auto/main/scripts/install.sh | sh
#   sh install.sh v1.15.0        # a specific release (default: latest)
#
# Environment:
#   AUTO_INSTALL_DIR   where to put the binary (default: ~/.local/bin)
#   AUTO_DOWNLOAD_URL  base URL of the release assets (default: GitHub)
set -eu

REPO="GhentCDH/auto"
VERSION="${1:-latest}"
INSTALL_DIR="${AUTO_INSTALL_DIR:-$HOME/.local/bin}"

fail() {
    echo "error: $*" >&2
    exit 1
}

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64 | Linux-amd64) target=x86_64-unknown-linux-musl ;;
    Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-musl ;;
    Darwin-arm64) target=aarch64-apple-darwin ;;
    Darwin-x86_64) target=x86_64-apple-darwin ;;
    *) fail "no prebuilt auto for $(uname -s) $(uname -m); build it from source:
  cargo install --git https://github.com/$REPO auto-cli" ;;
esac

if [ -n "${AUTO_DOWNLOAD_URL:-}" ]; then
    base="$AUTO_DOWNLOAD_URL"
elif [ "$VERSION" = latest ]; then
    base="https://github.com/$REPO/releases/latest/download"
else
    base="https://github.com/$REPO/releases/download/v${VERSION#v}"
fi

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$1" -O "$2"
    else
        fail "curl or wget is needed"
    fi
}

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

asset="auto-$target.tar.gz"
echo "downloading $asset ($VERSION)"
download "$base/$asset" "$tmp/$asset" ||
    fail "could not download $base/$asset (CLI binaries are published from v1.15.0 on)"
download "$base/$asset.sha256" "$tmp/$asset.sha256" ||
    fail "could not download the checksum $base/$asset.sha256"

expected=$(cut -d' ' -f1 "$tmp/$asset.sha256")
actual=$(sha256 "$tmp/$asset")
[ "$expected" = "$actual" ] || fail "checksum mismatch for $asset"

tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$INSTALL_DIR"
# Write next to the target and rename, so a running `auto` is not overwritten
# in place.
cp "$tmp/auto" "$INSTALL_DIR/.auto.new"
chmod 755 "$INSTALL_DIR/.auto.new"
mv -f "$INSTALL_DIR/.auto.new" "$INSTALL_DIR/auto"
echo "installed $("$INSTALL_DIR/auto" --version) to $INSTALL_DIR/auto"

found=$(command -v auto 2>/dev/null || true)
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        if [ -n "$found" ] && [ "$found" != "$INSTALL_DIR/auto" ]; then
            echo "warning: \`auto\` on your PATH is $found, which comes before $INSTALL_DIR/auto"
        fi
        ;;
    *) echo "note: $INSTALL_DIR is not on your PATH; add it, for example in ~/.profile:
  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

echo "first install? sign in with: auto login --url <your auto server>"
