#!/usr/bin/env bash
set -euo pipefail

VERSION=${1:?Usage: ./release.sh <version>}
TAG="v$VERSION"
TAP_DIR="$(dirname "$0")/../homebrew-hnr"

# Bump Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
cargo build --release 2>&1 | tail -2

# Commit, tag, push hnr
git add Cargo.toml Cargo.lock
git commit -m "chore: bump to $VERSION"
git tag "$TAG"
git push
git push origin "$TAG"

# Publish to crates.io
cargo publish

# Wait for crates.io to index
echo "Waiting for crates.io to index $TAG..."
sleep 20

# Compute SHA256 of GitHub tarball
SHA=$(curl -sL "https://github.com/prasanthj/hnr/archive/refs/tags/${TAG}.tar.gz" | shasum -a 256 | awk '{print $1}')
echo "SHA256: $SHA"

# Update homebrew tap
TAP_FORMULA="$TAP_DIR/Formula/hnr.rb"
sed -i '' "s|refs/tags/v[0-9.]*\.tar\.gz|refs/tags/${TAG}.tar.gz|" "$TAP_FORMULA"
sed -i '' "s/sha256 \".*\"/sha256 \"$SHA\"/" "$TAP_FORMULA"

cd "$TAP_DIR"
git add Formula/hnr.rb
git commit -m "hnr $VERSION"
git push

echo "Released hnr $VERSION to crates.io and homebrew tap."
