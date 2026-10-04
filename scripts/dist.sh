#!/usr/bin/env bash
# Gathers every release artifact into dist/, building first unless SKIP_BUILD=1.
# The Arch package needs Docker and a long first run, so it only rebuilds with WITH_ARCH=1.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ "${SKIP_BUILD:-0}" != "1" ]; then
  pnpm tauri build
fi
if [ "${WITH_ARCH:-0}" = "1" ]; then
  bash packaging/arch/build-arch.sh
fi

rm -rf dist && mkdir dist
bundle=src-tauri/target/release/bundle
cp "$bundle"/deb/*.deb "$bundle"/rpm/*.rpm "$bundle"/appimage/*.AppImage dist/ 2>/dev/null || true
find packaging/arch/out -maxdepth 1 -name '*.pkg.tar.zst' ! -name '*-debug-*' \
  -exec cp {} dist/ \; 2>/dev/null || true

echo "dist/ holds:"
ls -lh dist | awk 'NR>1 {print "  " $9 " (" $5 ")"}'
