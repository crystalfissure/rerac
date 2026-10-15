#!/usr/bin/env bash
# Package the unofficial Skybox Mod as a ready-to-install launcher build.
#   bash tools/package/skybox-mod.sh              build and package
#   bash tools/package/skybox-mod.sh --no-build   package existing release binaries
# Outputs dist/rerac-skybox-mod-<mod-version>-<os>-<arch>.zip and .zip.sha256.
# This bundles the modded runtime; it is not a DLL/plugin for an unmodified runtime.
set -euo pipefail

build=1
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    -h|--help) sed -n '2,6p' "$0"; exit 0 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
mod_version="0.1.0-preview.1"
version="$(awk '/^\[/ { in_pkg = ($0 == "[workspace.package]") } in_pkg && /^version = "/ { gsub(/^version = "|"$/, ""); print; exit }' Cargo.toml)"
case "$version" in
  ''|*[!a-zA-Z0-9.+-]*) echo "invalid workspace version: $version" >&2; exit 1 ;;
esac

exe=""
case "$(uname -s)" in
  Darwin) os=macos ;;
  Linux) os=linux ;;
  MINGW*|MSYS*|CYGWIN*) os=windows; exe=".exe" ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch=arm64 ;;
  x86_64|amd64) arch=x86_64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

# Retain the upstream binary, shader, manifest and --version-json checks.
if [ "$build" = 1 ]; then
  bash tools/package/package.sh
else
  bash tools/package/package.sh --no-build
fi
base="$repo/dist/rerac-$version-$os-$arch"
name="rerac-skybox-mod-$mod_version-$os-$arch"
dist="$repo/dist"
out="$dist/$name"
rm -rf "$out" "$dist/$name.zip" "$dist/$name.zip.sha256"
mkdir -p "$out"
cp -R "$base/." "$out/"
mv "$out/README.txt" "$out/RUNTIME_README.txt"
cp LICENSE "$out/LICENSE.txt"
cp docs/mods/skybox/RELEASE_NOTES.md "$out/RELEASE_NOTES.md"
sed -e "s/@MOD_VERSION@/$mod_version/g" -e "s/@RUNTIME_VERSION@/$version/g" \
  docs/mods/skybox/README.txt > "$out/README.txt"
if [ "$os" = windows ]; then
  cp tools/package/skybox/Play-Skybox-Mod.cmd tools/package/skybox/Start-Skybox-Mod.ps1 "$out/"
fi

# A symlink or other special file must not escape the audited payload list.
special="$(find "$out" ! -type d ! -type f -print)"
if [ -n "$special" ]; then
  echo "special files are not allowed in the mod package:" >&2
  echo "$special" >&2
  exit 1
fi
unexpected="$(cd "$out" && find . -type f \
  ! -path "./rerac$exe" ! -path "./rerac-extract$exe" \
  ! -path ./rerac-manifest.json ! -path ./README.txt \
  ! -path ./RUNTIME_README.txt ! -path ./LICENSE.txt ! -path ./RELEASE_NOTES.md \
  ! -path ./Play-Skybox-Mod.cmd ! -path ./Start-Skybox-Mod.ps1 \
  ! -path './assets/shaders/*.wgsl')"
if [ -n "$unexpected" ]; then
  echo "unexpected files in the mod package:" >&2
  echo "$unexpected" >&2
  exit 1
fi

if [ "$os" = windows ]; then
  sed -i 's/\r$//; s/$/\r/' "$out/README.txt" "$out/LICENSE.txt" "$out/Play-Skybox-Mod.cmd" "$out/Start-Skybox-Mod.ps1"
fi
case "$os" in
  macos) (cd "$dist" && ditto -c -k --keepParent "$name" "$name.zip") ;;
  windows) (cd "$dist" && "$(cygpath "$SYSTEMROOT")/System32/tar.exe" -a -c -f "$name.zip" "$name") ;;
  linux) (cd "$dist" && zip -qry "$name.zip" "$name") ;;
esac
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$dist" && sha256sum "$name.zip" > "$name.zip.sha256")
else
  (cd "$dist" && shasum -a 256 "$name.zip" > "$name.zip.sha256")
fi

echo "Skybox Mod $mod_version, runtime $version"
echo "Install this zip in ReRAC Launcher: $dist/$name.zip"
echo "Checksum: $dist/$name.zip.sha256"
