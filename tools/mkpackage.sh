#!/bin/bash
# Construit le paquet Arch c-man : tarball des sources + makepkg.
set -euo pipefail

cd "$(dirname "$0")/.."
PKGDIR=package
TARBALL="$PKGDIR/c-man-2.0.0.tar.gz"

echo "== tarball des sources =="
tar czf "$TARBALL" \
  --transform "s,^,c-man-2.0.0/," \
  Cargo.toml Cargo.lock build.rs src content README.md LICENSE

echo "== makepkg =="
cd "$PKGDIR"
makepkg -f "$@"

echo
echo "Paquet construit : $(ls -t c-man-*.pkg.tar.* | head -1)"
echo "Installation système : sudo pacman -U $(ls -t c-man-*.pkg.tar.* | head -1)"
