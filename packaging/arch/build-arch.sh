#!/usr/bin/env bash
# Builds the pacman package from the current working tree, inside an Arch container.
# The actual recipe is makepkg-inner.sh, shared with the GitHub workflow.
set -euo pipefail
cd "$(dirname "$0")/../.."
mkdir -p packaging/arch/out

docker run --rm -v "$PWD:/repo:ro" -v "$PWD/packaging/arch/out:/out" \
  -e VERSION="${VERSION:-0.1.0}" -e OUT=/out archlinux:latest bash -c '
set -e
pacman -Syu --noconfirm --needed base-devel rust nodejs pnpm webkit2gtk-4.1 gtk3 librsvg openssl >/dev/null
cd /repo && bash packaging/arch/makepkg-inner.sh
chown "$(stat -c %u:%g /out)" /out/*.pkg.tar.zst
'
