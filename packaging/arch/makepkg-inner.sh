#!/usr/bin/env bash
# Runs INSIDE an Arch system (container or CI) with the repository as the working
# directory. Both the local Docker build and the GitHub workflow call this, so the
# makepkg recipe lives in exactly one place.
#   VERSION  package version (default 0.1.0)
#   OUT      where the finished .pkg.tar.zst lands (default packaging/arch/out)
set -euo pipefail
VERSION="${VERSION:-0.1.0}"
OUT="${OUT:-packaging/arch/out}"

pkgdir=/home/builder/pkg
id builder &>/dev/null || useradd -m builder
mkdir -p "$pkgdir" "$OUT"

tar --exclude=./.git --exclude=./packaging --exclude=./node_modules \
    --exclude=./src-tauri/target --exclude=./build --exclude=./.svelte-kit --exclude=./dist \
    --transform "s,^\./,viewust-$VERSION/," -czf "$pkgdir/viewust-$VERSION.tar.gz" .
cp packaging/arch/PKGBUILD.local "$pkgdir/PKGBUILD"
cp packaging/arch/viewust.desktop "$pkgdir/"
sed -i "s/^pkgver=.*/pkgver=$VERSION/" "$pkgdir/PKGBUILD"
chown -R builder "$pkgdir"

su builder -c "cd $pkgdir && MAKEFLAGS=-j\$(nproc) makepkg -f --skipchecksums"
cp "$pkgdir"/viewust-[0-9]*.pkg.tar.zst "$OUT/"
