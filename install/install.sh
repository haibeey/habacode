#!/bin/bash

set -e

OS_TYPE=$(uname | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS_TYPE" in
    linux)
        case "$ARCH" in
            x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        INSTALL_DIR="/usr/bin"
        ;;
    darwin)
        case "$ARCH" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64)  TARGET="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        INSTALL_DIR="/usr/local/bin"
        ;;
    *)
        echo "Unsupported OS: $OS_TYPE"
        exit 1
        ;;
esac

ARCHIVE="habacode-${TARGET}.tar.gz"
URL="https://github.com/haibeey/habacode/releases/latest/download/${ARCHIVE}"

if command -v wget > /dev/null 2>&1; then
    wget -O "$ARCHIVE" "$URL"
elif command -v curl > /dev/null 2>&1; then
    curl -fsSL -o "$ARCHIVE" "$URL"
else
    echo "Neither wget nor curl found"
    exit 1
fi

tar -xzf "$ARCHIVE"
chmod +x "habacode-${TARGET}/habacode"

# Check if sudo is installed
if command -v sudo > /dev/null 2>&1; then
    USE_SUDO="sudo"
else
    USE_SUDO=""
fi

$USE_SUDO mv "habacode-${TARGET}/habacode" "$INSTALL_DIR/"

rm -rf "$ARCHIVE" "habacode-${TARGET}"
