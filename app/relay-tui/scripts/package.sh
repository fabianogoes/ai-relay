#!/usr/bin/env bash
# Builds one release target of relay-tui and packs it for the GitHub Release.
#
#   scripts/package.sh <target-triple>
#
# Writes dist/relay-tui-<version>-<target>.tar.gz and its .sha256. The release workflow and a developer on a Mac run this same script,
# so what CI publishes is what can be reproduced locally.
set -euo pipefail

target="${1:?usage: scripts/package.sh <target-triple>, e.g. aarch64-apple-darwin}"

cd "$(dirname "${BASH_SOURCE[0]}")/.."
version="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
name="relay-tui-${version}-${target}"

cargo build --release --locked --target "$target"

rm -rf "dist/${name}" "dist/${name}.tar.gz" "dist/${name}.tar.gz.sha256"
mkdir -p "dist/${name}"
cp "target/${target}/release/relay-tui" "dist/${name}/"
cp ../../LICENSE "dist/${name}/"
if [ -f README.md ]; then cp README.md "dist/${name}/"; fi

archive="${name}.tar.gz"
tar -C dist -czf "dist/${archive}" "${name}"

if command -v sha256sum > /dev/null 2>&1; then
  (cd dist && sha256sum "${archive}" > "${archive}.sha256")
else
  (cd dist && shasum -a 256 "${archive}" > "${archive}.sha256")
fi
rm -rf "dist/${name}"

echo "dist/${archive}"
