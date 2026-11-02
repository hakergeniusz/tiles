#!/bin/sh
# Build a Debian .deb without dpkg-deb: a deb is just an ar archive holding
# a control and a data tarball (Debian format 2.0).
#
# Usage: run from the repository root after `cargo build --release`:
#   packaging/deb/build-deb.sh [output.deb]
set -eu

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)
[ -n "$VERSION" ] || { echo "cannot read version from Cargo.toml" >&2; exit 1; }
[ -x target/release/tiles ] || { echo "build first: cargo build --release" >&2; exit 1; }

STAGE="${TMPDIR:-/tmp}/tiles-deb.$$"
mkdir -p "$STAGE/DEBIAN" "$STAGE/usr/bin" \
         "$STAGE/usr/share/applications" \
         "$STAGE/usr/share/icons/hicolor/scalable/apps" \
         "$STAGE/usr/share/metainfo" \
         "$STAGE/usr/share/licenses/tiles"
cleanup() {
    # No rm(1) on this machine: use trash when available, otherwise leave
    # the staging dir in /tmp for the next reboot to clear.
    command -v trash >/dev/null 2>&1 && trash "$STAGE" >/dev/null 2>&1 || true
}
trap cleanup EXIT

cat > "$STAGE/DEBIAN/control" <<EOF
Package: tiles
Version: $VERSION
Architecture: amd64
Maintainer: hakergeniusz <hakergeniusz@users.noreply.github.com>
Depends: libgtk-4-1, libadwaita-1-0
Section: games
Priority: optional
Homepage: https://github.com/hakergeniusz/tiles
Description: The classic 2048 puzzle, built with GTK4 and libadwaita
 Tiles is the classic 2048 puzzle as a native Linux desktop app:
 animated moves, Light/Sepia/Dark color modes, mouse/touch swipes
 and a persisted best score.
EOF

install -m755 target/release/tiles "$STAGE/usr/bin/tiles"
install -m644 packaging/io.github.hakergeniusz.Tiles.desktop \
        "$STAGE/usr/share/applications/"
install -m644 resources/tiles.svg \
        "$STAGE/usr/share/icons/hicolor/scalable/apps/io.github.hakergeniusz.Tiles.svg"
install -m644 packaging/io.github.hakergeniusz.Tiles.metainfo.xml \
        "$STAGE/usr/share/metainfo/"
install -m644 LICENSE "$STAGE/usr/share/licenses/tiles/LICENSE"

tar --sort=name --owner=root:0 --group=root:0 \
    -czf "$STAGE/control.tar.gz" -C "$STAGE" DEBIAN
tar --sort=name --owner=root:0 --group=root:0 \
    -czf "$STAGE/data.tar.gz" -C "$STAGE" usr
printf '2.0\n' > "$STAGE/debian-binary"

OUT="${1:-tiles_${VERSION}_amd64.deb}"
ar rc "$OUT" "$STAGE/debian-binary" "$STAGE/control.tar.gz" "$STAGE/data.tar.gz"
echo "built $OUT"
