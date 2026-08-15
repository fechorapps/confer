#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/dist"
VERSION="${1:-1.0.0}"
ARCH="linux-x64"
BUNDLE_DIR="${OUTPUT_DIR}/confer-desktop-${VERSION}-${ARCH}"

echo "========================================"
echo " Packaging Confer Desktop Tarball v${VERSION}"
echo "========================================"

mkdir -p "${OUTPUT_DIR}"
rm -rf "${BUNDLE_DIR}"
mkdir -p "${BUNDLE_DIR}"

cargo build --release --manifest-path "${ROOT_DIR}/client-desktop/Cargo.toml"

cp "${ROOT_DIR}/client-desktop/target/release/confer-desktop" "${BUNDLE_DIR}/"
cp "${ROOT_DIR}/packaging/confer.desktop" "${BUNDLE_DIR}/"
cp "${ROOT_DIR}/packaging/icons/confer.svg" "${BUNDLE_DIR}/"
cp "${ROOT_DIR}/README.md" "${BUNDLE_DIR}/" 2>/dev/null || true

cat <<'EOF' > "${BUNDLE_DIR}/install.sh"
#!/usr/bin/env bash
set -euo pipefail

PREFIX="${1:-/usr/local}"
echo "Installing Confer Desktop to ${PREFIX}..."

mkdir -p "${PREFIX}/bin"
mkdir -p "${PREFIX}/share/applications"
mkdir -p "${PREFIX}/share/icons/hicolor/scalable/apps"

cp confer-desktop "${PREFIX}/bin/"
chmod 755 "${PREFIX}/bin/confer-desktop"

cp confer.desktop "${PREFIX}/share/applications/"
cp confer.svg "${PREFIX}/share/icons/hicolor/scalable/apps/"

echo "✓ Confer Desktop successfully installed!"
EOF
chmod +x "${BUNDLE_DIR}/install.sh"

tar -czvf "${OUTPUT_DIR}/confer-desktop-${VERSION}-${ARCH}.tar.gz" -C "${OUTPUT_DIR}" "confer-desktop-${VERSION}-${ARCH}"
echo "✓ Successfully built: ${OUTPUT_DIR}/confer-desktop-${VERSION}-${ARCH}.tar.gz"
