# Project Tray

System tray para abrir pastas de projetos no terminal, em editores e em agentes de IA.

## Instalação (Pacotes Prontos / Releases)

Baixe a versão mais recente na aba de **Releases** do repositório no formato adequado para o seu sistema:

### 1. Pacote Debian / Ubuntu (`.deb`)

Recomendado para **Ubuntu, Debian, Linux Mint, Pop!_OS** e derivados:

```bash
sudo apt install ./project-tray_*_amd64.deb
# ou: sudo dpkg -i ./project-tray_*_amd64.deb
```

> **Autostart**: O pacote instala o binário no sistema, adiciona o atalho no menu de aplicativos e configura automaticamente o início junto com a sessão (em `/etc/xdg/autostart/`).
>
> Para desinstalar:
> ```bash
> sudo apt remove project-tray
> ```

### 2. Pacote Universal (`.AppImage`)

Funciona em praticamente qualquer distribuição Linux moderna sem necessidade de instalação:

1. Baixe o arquivo `.AppImage` e dê permissão de execução:
   ```bash
   chmod +x project-tray-*-x86_64.AppImage
   ```
2. Execute diretamente:
   ```bash
   ./project-tray-*-x86_64.AppImage
   ```

> **Dica**: Se você usa [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher), ele se encarrega de mover o arquivo e criar os atalhos no menu do sistema.
>
> Para configurar autostart manualmente com o AppImage:
> ```bash
> mkdir -p ~/.config/autostart
> cat <<EOF > ~/.config/autostart/project-tray.desktop
> [Desktop Entry]
> Type=Application
> Name=Project Tray
> Exec=$(pwd)/project-tray-v0.1.0-x86_64.AppImage
> Icon=folder
> Terminal=false
> Categories=Development;
> X-GNOME-Autostart-enabled=true
> X-KDE-autostart-after=panel
> EOF
> ```

### 3. Pacote Compactado (`.tar.gz`)

Ideal para instalação rápida em qualquer distribuição através de script:

1. Extraia o pacote baixado:
   ```bash
   tar -xzvf project-tray-*-linux-x86_64.tar.gz
   cd project-tray-*-linux-x86_64
   ```

2. Instale para o seu usuário (sem necessidade de `sudo`):
   ```bash
   ./install.sh --user
   # ou: make install-user
   ```
   *(Instala o executável em `~/.local/bin` e configura o autostart em `~/.config/autostart`)*

   Ou instale no sistema inteiro (requer `sudo`):
   ```bash
   sudo ./install.sh --system
   # ou: sudo make install
   ```

3. Para desinstalar futuramente:
   ```bash
   ./uninstall.sh --user       # se instalou para o usuário
   # ou: sudo ./uninstall.sh --system # se instalou no sistema
   ```

---

## Compilação a partir do Código-Fonte

### Dependências de compilação (Debian/Ubuntu)

    sudo apt install build-essential pkg-config libgtk-4-dev libdbus-1-dev

Fedora: `gtk4-devel dbus-devel` · Arch: `gtk4 dbus`

### Compilar e Instalar (com Autostart)

O processo compila o executável via Cargo e registra o atalho no sistema e no autostart.

#### Instalação para o usuário atual (sem sudo)

```bash
make install-user
# ou: ./install.sh --user
```

#### Instalação no sistema (global / requer sudo)

```bash
sudo make install
# ou: sudo ./install.sh --system
```

#### Desinstalação

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
