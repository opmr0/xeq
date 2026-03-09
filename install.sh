#!/bin/bash
set -e

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="aarch64"
else
    ARCH="x86_64"
fi

case "$OS" in
    linux)  BINARY="xeq-linux-x86_64" ;;
    darwin)
        if [ "$ARCH" = "aarch64" ]; then
            BINARY="xeq-macos-aarch64"
        else
            BINARY="xeq-macos-x86_64"
        fi
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Installing xeq for $OS ($ARCH)..."

LATEST=$(curl -s https://api.github.com/repos/opmr0/xeq/releases/latest | grep '"tag_name"' | head -1 | cut -d '"' -f 4)

if [ -z "$LATEST" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

echo "Downloading $LATEST..."
curl -L -o /tmp/xeq "https://github.com/opmr0/xeq/releases/download/$LATEST/$BINARY"
chmod +x /tmp/xeq

echo "Installing to /usr/local/bin (may require sudo)..."
sudo mv /tmp/xeq /usr/local/bin/xeq

echo ""
echo "xeq installed successfully!"
echo "Run 'xeq --help' to get started"
