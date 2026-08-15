#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/dist"
VERSION="${1:-1.0.0}"

echo "========================================"
echo " Packaging Confer Backend Server v${VERSION}"
echo "========================================"

mkdir -p "${OUTPUT_DIR}"

# 1. Publish Linux x64 Self-Contained Single-File Executable
echo "==> Publishing .NET 10 Server for Linux x64..."
dotnet publish "${ROOT_DIR}/src/api/Api.csproj" \
    -c Release \
    -r linux-x64 \
    --self-contained true \
    -p:PublishSingleFile=true \
    -p:IncludeNativeLibrariesForSelfExtract=true \
    -p:PublishTrimmed=false \
    -o "${OUTPUT_DIR}/confer-server-linux-x64"

tar -czvf "${OUTPUT_DIR}/confer-server-${VERSION}-linux-x64.tar.gz" -C "${OUTPUT_DIR}" "confer-server-linux-x64"
echo "✓ Built: ${OUTPUT_DIR}/confer-server-${VERSION}-linux-x64.tar.gz"

# 2. Publish Windows x64 Self-Contained Single-File Executable
echo "==> Publishing .NET 10 Server for Windows x64..."
dotnet publish "${ROOT_DIR}/src/api/Api.csproj" \
    -c Release \
    -r win-x64 \
    --self-contained true \
    -p:PublishSingleFile=true \
    -p:IncludeNativeLibrariesForSelfExtract=true \
    -p:PublishTrimmed=false \
    -o "${OUTPUT_DIR}/confer-server-win-x64"

tar -czvf "${OUTPUT_DIR}/confer-server-${VERSION}-win-x64.tar.gz" -C "${OUTPUT_DIR}" "confer-server-win-x64"
echo "✓ Built: ${OUTPUT_DIR}/confer-server-${VERSION}-win-x64.tar.gz"
