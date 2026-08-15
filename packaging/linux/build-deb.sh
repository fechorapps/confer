#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/dist"
VERSION="${1:-1.0.0}"
ARCH="amd64"
PKG_NAME="confer-desktop_${VERSION}_${ARCH}"
BUILD_DIR="${OUTPUT_DIR}/${PKG_NAME}"

echo "========================================"
echo " Packaging Confer Desktop (.deb) v${VERSION}"
echo "========================================"

mkdir -p "${OUTPUT_DIR}"
rm -rf "${BUILD_DIR}"

# 1. Build release binary if not present
echo "==> Building optimized release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/client-desktop/Cargo.toml"

# 2. Setup directory tree
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"

# 3. Copy artifacts
cp "${ROOT_DIR}/client-desktop/target/release/confer-desktop" "${BUILD_DIR}/usr/bin/"
chmod 755 "${BUILD_DIR}/usr/bin/confer-desktop"

cp "${ROOT_DIR}/packaging/confer.desktop" "${BUILD_DIR}/usr/share/applications/"
cp "${ROOT_DIR}/packaging/icons/confer.svg" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/"

# 4. Generate DEBIAN/control
cat <<EOF > "${BUILD_DIR}/DEBIAN/control"
Package: confer-desktop
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: libasound2 (>= 1.0.16), libpulse0 (>= 0.99.1), libc6 (>= 2.31)
Maintainer: Fechor Apps <contact@fechorapps.com>
Description: Confer High-Performance Real-Time Video Collaboration Platform
 Native GPU immediate-mode video conferencing client with zero Electron overhead (<40MB RAM).
 Built with Rust, egui, and WebRTC zero-delay hardware capture.
Homepage: https://github.com/fechorapps/confer
EOF

# 5. Build .deb
echo "==> Generating Debian package..."
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${BUILD_DIR}" "${OUTPUT_DIR}/${PKG_NAME}.deb"
else
    # Universal POSIX deb builder (ar + tar)
    TEMP_DIR=$(mktemp -d)
    echo "2.0" > "${TEMP_DIR}/debian-binary"
    tar --numeric-owner --owner=0 --group=0 -czf "${TEMP_DIR}/control.tar.gz" -C "${BUILD_DIR}/DEBIAN" .
    tar --numeric-owner --owner=0 --group=0 -czf "${TEMP_DIR}/data.tar.gz" -C "${BUILD_DIR}" usr
    ar rcs "${OUTPUT_DIR}/${PKG_NAME}.deb" "${TEMP_DIR}/debian-binary" "${TEMP_DIR}/control.tar.gz" "${TEMP_DIR}/data.tar.gz"
    rm -rf "${TEMP_DIR}"
fi

echo "✓ Successfully built: ${OUTPUT_DIR}/${PKG_NAME}.deb"
ls -lh "${OUTPUT_DIR}/${PKG_NAME}.deb"
