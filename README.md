# Project Tray

System tray para abrir pastas de projetos no terminal, em editores e em agentes de IA.

## Dependências (Debian/Ubuntu)

    sudo apt install build-essential pkg-config libgtk-4-dev libdbus-1-dev

Fedora: `gtk4-devel dbus-devel` · Arch: `gtk4 dbus`

## Compilação e Instalação (com Autostart)

O processo de instalação compila o executável e registra o aplicativo tanto no menu do sistema quanto na inicialização automática (XDG Autostart).

### Instalação para o usuário atual (sem sudo)

```bash
make install-user
# ou: ./install.sh --user
```

Isso instala:
- Binário em `~/.local/bin/project-tray`
- Entrada de aplicativo em `~/.local/share/applications/project-tray.desktop`
- Inicialização automática em `~/.config/autostart/project-tray.desktop`

### Instalação no sistema (global / requer sudo)

```bash
sudo make install
# ou: sudo ./install.sh --system
```

Isso instala:
- Binário em `/usr/local/bin/project-tray`
- Entrada de aplicativo em `/usr/local/share/applications/project-tray.desktop`
- Inicialização automática em `/etc/xdg/autostart/project-tray.desktop`

### Desinstalação

```bash
make uninstall-user     # Para desinstalar do usuário
# ou: sudo make uninstall # Para desinstalar do sistema
```

## Empacotamento e Releases

### Gerar pacotes localmente (.deb, .AppImage, .tar.gz)

```bash
make package
# ou: ./package.sh [versão]
```

Os arquivos serão gerados no diretório `dist/`.

### Releases automáticas (GitHub Actions)

O workflow `.github/workflows/release.yml` é disparado automaticamente ao criar uma tag de versão:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Ele compila o aplicativo no Ubuntu e anexa automaticamente os pacotes `.deb`, `.AppImage` e `.tar.gz` à nova Release no GitHub. Também pode ser disparado manualmente via **Actions** na interface do repositório (com suporte a informar a versão desejada).

## Executar em desenvolvimento

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
