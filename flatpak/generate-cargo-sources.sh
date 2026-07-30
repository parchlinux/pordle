#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v flatpak-cargo-generator &>/dev/null; then
    echo "Installing flatpak-cargo-generator..."
    pip install flatpak-cargo-generator
fi

echo "Generating cargo-sources.json from Cargo.lock..."
mkdir -p generated
flatpak-cargo-generator Cargo.lock -o generated/cargo-sources.json

echo "Done. cargo-sources.json is in generated/"
echo ""
echo "To build:"
echo "  flatpak-builder build-dir com.parchlinux.pordle.yml --force-clean"
echo "  flatpak-builder --user --install build-dir com.parchlinux.pordle.yml --force-clean"
echo "  flatpak run com.parchlinux.pordle"
