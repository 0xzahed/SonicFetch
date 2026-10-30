#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
VERSION=$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)
ARCH=$(dpkg --print-architecture)
OUT_DIR=${1:-"$ROOT/dist"}

if [ -z "$VERSION" ]; then
    echo "Could not read workspace version from Cargo.toml" >&2
    exit 1
fi

cargo build --manifest-path "$ROOT/Cargo.toml" --release -p sonicfetch-desktop --bin sonicfetch

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT HUP INT TERM
PKG="$WORK/sonicfetch"
mkdir -p "$PKG/usr/bin" "$PKG/usr/share/doc/sonicfetch"
install -m 0755 "$ROOT/target/release/sonicfetch" "$PKG/usr/bin/sonicfetch"
install -m 0644 "$ROOT/LICENSE" "$PKG/usr/share/doc/sonicfetch/copyright"

cat > "$PKG/DEBIAN/control" <<EOF
Package: sonicfetch
Version: $VERSION
Section: net
Priority: optional
Architecture: $ARCH
Depends: libc6, libsqlite3-0, ca-certificates
Maintainer: SonicFetch Contributors
Description: secure local HTTP and HTTPS download manager
 Command line download manager with a persistent queue, pause/resume,
 retry, and safe temporary-file finalization.
EOF

mkdir -p "$OUT_DIR"
dpkg-deb --root-owner-group --build "$PKG" "$OUT_DIR/sonicfetch_${VERSION}_${ARCH}.deb"
echo "Package created: $OUT_DIR/sonicfetch_${VERSION}_${ARCH}.deb"
