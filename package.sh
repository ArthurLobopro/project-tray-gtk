#!/usr/bin/env bash
set -e

# Diretório raiz do projeto
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

# Obtém versão (via argumento, ou de Cargo.toml)
VERSION="$1"
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d '"' -f 2)
fi

echo "==> Preparando release para project-tray v${VERSION}..."

# 1. Compilação do binário
echo "==> Compilando o binário em modo release..."
cargo build --release

BIN_TARGET="target/release/project-tray"
if [ ! -f "$BIN_TARGET" ]; then
    echo "Erro: Binário $BIN_TARGET não encontrado!"
    exit 1
fi

DIST_DIR="$ROOT_DIR/dist"
mkdir -p "$DIST_DIR"

# 2. Gerar .tar.gz
echo "==> Criando pacote .tar.gz..."
TAR_DIR="project-tray-v${VERSION}-linux-x86_64"
rm -rf "$TAR_DIR"
mkdir -p "$TAR_DIR"

cp "$BIN_TARGET" "$TAR_DIR/"
cp project-tray.desktop "$TAR_DIR/"
cp Makefile "$TAR_DIR/"
cp install.sh "$TAR_DIR/"
cp uninstall.sh "$TAR_DIR/"
cp README.md "$TAR_DIR/"
if [ -d assets ]; then
    cp -r assets "$TAR_DIR/"
fi

TAR_FILE="$DIST_DIR/project-tray-v${VERSION}-linux-x86_64.tar.gz"
tar -czvf "$TAR_FILE" "$TAR_DIR"
rm -rf "$TAR_DIR"
echo "Pacote gerado: $TAR_FILE"

# 3. Gerar .deb
echo "==> Criando pacote .deb..."
if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_DIR="$ROOT_DIR/target/deb-package"
    rm -rf "$DEB_DIR"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/bin"
    mkdir -p "$DEB_DIR/usr/share/applications"
    mkdir -p "$DEB_DIR/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$DEB_DIR/etc/xdg/autostart"

    cat <<EOF > "$DEB_DIR/DEBIAN/control"
Package: project-tray
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Arthur Lobo
Depends: libgtk-4-1 (>= 4.10.0), libdbus-1-3
Description: System tray para gerenciar e abrir projetos
 System tray para abrir projetos no terminal, editores e agentes de IA com suporte a inicialização automática (autostart).
EOF

    install -m 755 "$BIN_TARGET" "$DEB_DIR/usr/bin/project-tray"
    install -m 644 project-tray.desktop "$DEB_DIR/usr/share/applications/project-tray.desktop"
    install -m 644 project-tray.desktop "$DEB_DIR/etc/xdg/autostart/project-tray.desktop"
    if [ -f assets/project-tray.svg ]; then
        install -m 644 assets/project-tray.svg "$DEB_DIR/usr/share/icons/hicolor/scalable/apps/project-tray.svg"
    fi
    if [ -f assets/project-tray.png ]; then
        mkdir -p "$DEB_DIR/usr/share/icons/hicolor/256x256/apps"
        install -m 644 assets/project-tray.png "$DEB_DIR/usr/share/icons/hicolor/256x256/apps/project-tray.png"
    fi

    DEB_FILE="$DIST_DIR/project-tray_${VERSION}_amd64.deb"
    dpkg-deb --build "$DEB_DIR" "$DEB_FILE"
    rm -rf "$DEB_DIR"
    echo "Pacote gerado: $DEB_FILE"
else
    echo "Aviso: 'dpkg-deb' não está disponível neste ambiente (será compilado no GitHub Actions)."
fi

# 4. Gerar .AppImage
echo "==> Criando pacote .AppImage..."
APPDIR="$ROOT_DIR/target/AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"

install -m 755 "$BIN_TARGET" "$APPDIR/usr/bin/project-tray"
install -m 644 project-tray.desktop "$APPDIR/usr/share/applications/project-tray.desktop"
install -m 644 project-tray.desktop "$APPDIR/project-tray.desktop"

if [ -f assets/project-tray.svg ]; then
    install -m 644 assets/project-tray.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/project-tray.svg"
    install -m 644 assets/project-tray.svg "$APPDIR/project-tray.svg"
fi

if [ -f assets/project-tray.png ]; then
    mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    install -m 644 assets/project-tray.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/project-tray.png"
    install -m 644 assets/project-tray.png "$APPDIR/project-tray.png"
    install -m 644 assets/project-tray.png "$APPDIR/.DirIcon"
elif [ -f assets/project-tray.svg ]; then
    ln -sf project-tray.svg "$APPDIR/.DirIcon"
fi

cat <<'EOF' > "$APPDIR/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}

export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

exec "${HERE}/usr/bin/project-tray" "$@"
EOF
chmod +x "$APPDIR/AppRun"

APPIMAGE_OUT="$DIST_DIR/project-tray-v${VERSION}-x86_64.AppImage"

APPIMAGETOOL="$ROOT_DIR/target/appimagetool"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Baixando appimagetool..."
    wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -O "$APPIMAGETOOL" || curl -sL https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -o "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
fi

APPIMAGETOOL_DIR="$ROOT_DIR/target/appimagetool-extracted"
if [ ! -d "$APPIMAGETOOL_DIR" ]; then
    echo "Extraindo appimagetool..."
    mkdir -p "$APPIMAGETOOL_DIR"
    (cd "$ROOT_DIR/target" && "$APPIMAGETOOL" --appimage-extract >/dev/null 2>&1 && mv squashfs-root/* "$APPIMAGETOOL_DIR/" && rm -rf squashfs-root)
fi

echo "Executando appimagetool..."
ARCH=x86_64 "$APPIMAGETOOL_DIR/AppRun" "$APPDIR" "$APPIMAGE_OUT"
rm -rf "$APPDIR"
echo "Pacote gerado: $APPIMAGE_OUT"

echo ""
echo "==> Todos os pacotes disponíveis em dist/:"
ls -lh "$DIST_DIR"
