#!/usr/bin/env bash
set -e

MODE="user"
if [ "$EUID" -eq 0 ]; then
    MODE="system"
fi

for arg in "$@"; do
    case "$arg" in
        --user)
            MODE="user"
            ;;
        --system)
            MODE="system"
            ;;
        --help|-h)
            echo "Uso: ./uninstall.sh [--user | --system]"
            echo "  --user    Remove a instalação do usuário atual"
            echo "  --system  Remove a instalação do sistema (requer sudo)"
            exit 0
            ;;
        *)
            echo "Opção desconhecida: $arg"
            echo "Uso: ./uninstall.sh [--user | --system]"
            exit 1
            ;;
    esac
done

BIN_NAME="project-tray"
DESKTOP_FILE="project-tray.desktop"

if [ "$MODE" = "system" ]; then
    PREFIX="${PREFIX:-/usr/local}"
    BIN_DIR="$PREFIX/bin"
    APP_DIR="$PREFIX/share/applications"
    AUTOSTART_DIR="/etc/xdg/autostart"
    ICON_DIR="$PREFIX/share/icons/hicolor/scalable/apps"

    echo "==> Removendo do sistema..."
    rm -f "$BIN_DIR/$BIN_NAME"
    rm -f "$APP_DIR/$DESKTOP_FILE"
    rm -f "$AUTOSTART_DIR/$DESKTOP_FILE"
    rm -f "$ICON_DIR/project-tray.svg"
else
    BIN_DIR="${HOME}/.local/bin"
    APP_DIR="${HOME}/.local/share/applications"
    AUTOSTART_DIR="${HOME}/.config/autostart"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"

    echo "==> Removendo do usuário ($USER)..."
    rm -f "$BIN_DIR/$BIN_NAME"
    rm -f "$APP_DIR/$DESKTOP_FILE"
    rm -f "$AUTOSTART_DIR/$DESKTOP_FILE"
    rm -f "$ICON_DIR/project-tray.svg"
fi

echo "Desinstalação concluída!"
