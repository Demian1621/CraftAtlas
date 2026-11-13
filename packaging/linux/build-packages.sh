#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Usage: build-packages.sh <version_without_v> [bin_path] [out_dir]}"
BIN_PATH="${2:-target/release/minecraft-companion}"
OUT_DIR="${3:-dist/linux}"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "ERROR: invalid version string: $VERSION" >&2
  exit 1
fi

if [[ ! -f "$BIN_PATH" ]]; then
  echo "ERROR: binary not found at $BIN_PATH" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ICON_SRC="$REPO_ROOT/assets/custom_app_icon.png"
DESKTOP_SRC="$SCRIPT_DIR/minecraft-companion.desktop"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

# Normalize icon to 256x256 PNG (crisp nearest scaling keeps pixel silhouette sharp)
ICON_256="$WORK_DIR/minecraft-companion.png"
if command -v convert >/dev/null 2>&1; then
  convert "$ICON_SRC" -filter Point -resize 256x256 "$ICON_256"
else
  cp "$ICON_SRC" "$ICON_256"
fi

# 1. Portable .tar.gz (Arch / Manjaro / Gentoo / Generic Linux)
TAR_STAGE="$WORK_DIR/tar/minecraft-companion-v${VERSION}-linux-x86_64"
mkdir -p "$TAR_STAGE"
cp "$BIN_PATH" "$TAR_STAGE/minecraft-companion"
chmod +x "$TAR_STAGE/minecraft-companion"
cp "$DESKTOP_SRC" "$TAR_STAGE/minecraft-companion.desktop"
cp "$ICON_256" "$TAR_STAGE/minecraft-companion.png"
cat > "$TAR_STAGE/install-user.sh" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/256x256/apps"
install -m 755 "$HERE/minecraft-companion" "$HOME/.local/bin/minecraft-companion"
install -m 644 "$HERE/minecraft-companion.png" "$HOME/.local/share/icons/hicolor/256x256/apps/minecraft-companion.png"
install -m 644 "$HERE/minecraft-companion.desktop" "$HOME/.local/share/applications/minecraft-companion.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" || true
echo "Installed to ~/.local/bin/minecraft-companion"
EOF
chmod +x "$TAR_STAGE/install-user.sh"
tar -C "$WORK_DIR/tar" -czf "$OUT_DIR/minecraft-companion-v${VERSION}-linux-x86_64.tar.gz" "minecraft-companion-v${VERSION}-linux-x86_64"

# 2. Debian / Ubuntu / Mint / Pop!_OS (.deb)
DEB_STAGE="$WORK_DIR/deb"
mkdir -p "$DEB_STAGE/DEBIAN" \
         "$DEB_STAGE/usr/bin" \
         "$DEB_STAGE/usr/share/applications" \
         "$DEB_STAGE/usr/share/icons/hicolor/256x256/apps"
install -m 755 "$BIN_PATH" "$DEB_STAGE/usr/bin/minecraft-companion"
install -m 644 "$DESKTOP_SRC" "$DEB_STAGE/usr/share/applications/minecraft-companion.desktop"
install -m 644 "$ICON_256" "$DEB_STAGE/usr/share/icons/hicolor/256x256/apps/minecraft-companion.png"

cat > "$DEB_STAGE/DEBIAN/control" << EOF
Package: minecraft-companion
Version: ${VERSION}
Section: games
Priority: optional
Architecture: amd64
Maintainer: Demian1621 <Demian1621@users.noreply.github.com>
Depends: libc6, libstdc++6, libasound2t64 | libasound2, libx11-6, libxcb1, libxkbcommon0, libxkbcommon-x11-0, libgl1 | libegl1
Homepage: https://github.com/Demian1621/minecraft-companion
Description: Technical desktop companion for Minecraft Java Edition
 Offline biome and structure seed map explorer powered by cubiomes, anvil
 god-gear optimizer, JEI/EMI recipe tree, potion alchemy guide, and
 curated technical farm blueprints.
EOF
dpkg-deb --build --root-owner-group "$DEB_STAGE" "$OUT_DIR/minecraft-companion_${VERSION}_amd64.deb"

# 3. Fedora / RHEL / openSUSE (.rpm)
if command -v rpmbuild >/dev/null 2>&1; then
  RPM_TOP="$WORK_DIR/rpmbuild"
  mkdir -p "$RPM_TOP"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
  cp "$BIN_PATH" "$RPM_TOP/SOURCES/minecraft-companion"
  cp "$DESKTOP_SRC" "$RPM_TOP/SOURCES/minecraft-companion.desktop"
  cp "$ICON_256" "$RPM_TOP/SOURCES/minecraft-companion.png"

  cat > "$RPM_TOP/SPECS/minecraft-companion.spec" << EOF
%global debug_package %{nil}
Name:           minecraft-companion
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Technical desktop companion for Minecraft Java Edition
License:        MIT
URL:            https://github.com/Demian1621/minecraft-companion
Requires:       alsa-lib, libxkbcommon, libX11, libxcb, mesa-libGL

%description
Offline biome and structure seed map explorer powered by cubiomes, anvil
god-gear optimizer, JEI/EMI recipe tree, potion alchemy guide, and
curated technical farm blueprints.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps
install -m 755 %{_sourcedir}/minecraft-companion %{buildroot}/usr/bin/minecraft-companion
install -m 644 %{_sourcedir}/minecraft-companion.desktop %{buildroot}/usr/share/applications/minecraft-companion.desktop
install -m 644 %{_sourcedir}/minecraft-companion.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/minecraft-companion.png

%files
/usr/bin/minecraft-companion
/usr/share/applications/minecraft-companion.desktop
/usr/share/icons/hicolor/256x256/apps/minecraft-companion.png
EOF

  rpmbuild --define "_topdir $RPM_TOP" -bb "$RPM_TOP/SPECS/minecraft-companion.spec"
  find "$RPM_TOP/RPMS" -name '*.rpm' -exec cp {} "$OUT_DIR/minecraft-companion-${VERSION}-1.x86_64.rpm" \;
fi

# 4. Universal Linux AppImage (runs across Ubuntu, Fedora, Arch, SteamOS, openSUSE, etc.)
APPDIR="$WORK_DIR/AppDir"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$APPDIR/usr/share/applications"
install -m 755 "$BIN_PATH" "$APPDIR/usr/bin/minecraft-companion"
install -m 644 "$DESKTOP_SRC" "$APPDIR/minecraft-companion.desktop"
install -m 644 "$DESKTOP_SRC" "$APPDIR/usr/share/applications/minecraft-companion.desktop"
install -m 644 "$ICON_256" "$APPDIR/minecraft-companion.png"
install -m 644 "$ICON_256" "$APPDIR/.DirIcon"
install -m 644 "$ICON_256" "$APPDIR/usr/share/icons/hicolor/256x256/apps/minecraft-companion.png"

cat > "$APPDIR/AppRun" << 'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "$HERE/usr/bin/minecraft-companion" "$@"
EOF
chmod +x "$APPDIR/AppRun"

APPIMAGETOOL="$WORK_DIR/appimagetool.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage"
APPIMAGETOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 -o "$APPIMAGETOOL" "$APPIMAGETOOL_URL"
printf '%s  %s\n' "$APPIMAGETOOL_SHA256" "$APPIMAGETOOL" | sha256sum -c -
chmod +x "$APPIMAGETOOL"
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGETOOL" \
  "$APPDIR" "$OUT_DIR/minecraft-companion-v${VERSION}-linux-x86_64.AppImage"

ls -lh "$OUT_DIR"
