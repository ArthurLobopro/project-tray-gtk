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
            echo "Uso: ./install.sh [--user | --system]"
            echo "  --user    Instala para o usuário atual (~/.local/bin e ~/.config/autostart)"
            echo "  --system  Instala no sistema (/usr/local/bin e /etc/xdg/autostart, requer sudo)"
            exit 0
            ;;
        *)
            echo "Opção desconhecida: $arg"
            echo "Uso: ./install.sh [--user | --system]"
            exit 1
            ;;
    esac
done

echo "==> Compilando o project-tray em modo release..."
cargo build --release

BIN_NAME="project-tray"
TARGET_BIN="target/release/$BIN_NAME"
DESKTOP_FILE="project-tray.desktop"

if [ ! -f "$TARGET_BIN" ]; then
    echo "Erro: Binário não encontrado em $TARGET_BIN"
    exit 1
fi

if [ "$MODE" = "system" ]; then
    PREFIX="${PREFIX:-/usr/local}"
    BIN_DIR="$PREFIX/bin"
    APP_DIR="$PREFIX/share/applications"
    AUTOSTART_DIR="/etc/xdg/autostart"

    echo "==> Instalando no sistema (PREFIX=$PREFIX)..."
    install -d "$BIN_DIR"
    install -m 755 "$TARGET_BIN" "$BIN_DIR/$BIN_NAME"
    install -d "$APP_DIR"
    install -m 644 "$DESKTOP_FILE" "$APP_DIR/$DESKTOP_FILE"
    install -d "$AUTOSTART_DIR"
    install -m 644 "$DESKTOP_FILE" "$AUTOSTART_DIR/$DESKTOP_FILE"
else
    BIN_DIR="${HOME}/.local/bin"
    APP_DIR="${HOME}/.local/share/applications"
    AUTOSTART_DIR="${HOME}/.config/autostart"

    echo "==> Instalando para o usuário ($USER)..."
    install -d "$BIN_DIR"
    install -m 755 "$TARGET_BIN" "$BIN_DIR/$BIN_NAME"
    install -d "$APP_DIR"
    install -m 644 "$DESKTOP_FILE" "$APP_DIR/$DESKTOP_FILE"
    install -d "$AUTOSTART_DIR"
    install -m 644 "$DESKTOP_FILE" "$AUTOSTART_DIR/$DESKTOP_FILE"

    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$BIN_DIR"; then
        echo ""
        echo "Aviso: '$BIN_DIR' parece não estar no seu PATH."
        echo "Certifique-se de adicionar '$BIN_DIR' ao PATH no seu ~/.bashrc ou ~/.zshrc."
    fi
fi

echo ""
echo "Instalação concluída com sucesso!"
echo "- Binário: $BIN_DIR/$BIN_NAME"
echo "- Entrada no menu: $APP_DIR/$DESKTOP_FILE"
echo "- Autostart configurado: $AUTOSTART_DIR/$DESKTOP_FILE"
