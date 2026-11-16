# Project Tray

System tray para abrir pastas de projetos no terminal, em editores e em agentes de IA.

## Dependências (Debian/Ubuntu)

    sudo apt install build-essential pkg-config libgtk-4-dev libdbus-1-dev

Fedora: `gtk4-devel dbus-devel` · Arch: `gtk4 dbus`

## Executar

    cargo run --release

> **GNOME**: o shell não exibe ícones de tray por padrão. Instale a extensão
> "AppIndicator and KStatusNotifierItem Support". KDE, XFCE, Cinnamon e waybar funcionam direto.

## Dados

`~/.config/project-tray/data.json` (respeita `$XDG_CONFIG_HOME`):

- `categories`: lista de `{ name, projects: [{name, path}] }` (aparecem primeiro no menu)
- `projects`: projetos soltos
- `launchers`: como abrir cada projeto. `{path}` vira a pasta;
  `in_terminal: true` executa o comando dentro de um terminal na pasta.

Para adicionar outro editor/agente, edite `launchers` e use "Recarregar data.json" no menu:

    { "name": "Neovim", "group": "Editores", "command": "nvim", "in_terminal": true }

O terminal é detectado automaticamente (ou defina `$TERMINAL`).
