CARGO ?= cargo
TARGET = release
BIN_NAME = project-tray
TARGET_BIN = target/$(TARGET)/$(BIN_NAME)
DESKTOP_FILE = project-tray.desktop

# Configuração de instalação do sistema (padrão: /usr/local)
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
SYSCONFDIR ?= /etc

# Configuração de instalação do usuário local (~/.local e ~/.config)
USER_PREFIX ?= $(HOME)/.local
USER_BINDIR ?= $(USER_PREFIX)/bin
USER_DATADIR ?= $(USER_PREFIX)/share
USER_AUTOSTARTDIR ?= $(HOME)/.config/autostart

.PHONY: all build check test clean install uninstall install-user uninstall-user

all: build

build:
	$(CARGO) build --release

check:
	$(CARGO) check

test:
	$(CARGO) test

clean:
	$(CARGO) clean

# Instalação no sistema (requer sudo/root)
install: build
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 "$(TARGET_BIN)" "$(DESTDIR)$(BINDIR)/$(BIN_NAME)"
	install -d "$(DESTDIR)$(DATADIR)/applications"
	install -m 644 "$(DESKTOP_FILE)" "$(DESTDIR)$(DATADIR)/applications/$(DESKTOP_FILE)"
	install -d "$(DESTDIR)$(SYSCONFDIR)/xdg/autostart"
	install -m 644 "$(DESKTOP_FILE)" "$(DESTDIR)$(SYSCONFDIR)/xdg/autostart/$(DESKTOP_FILE)"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/$(BIN_NAME)"
	rm -f "$(DESTDIR)$(DATADIR)/applications/$(DESKTOP_FILE)"
	rm -f "$(DESTDIR)$(SYSCONFDIR)/xdg/autostart/$(DESKTOP_FILE)"

# Instalação para o usuário atual (sem sudo)
install-user: build
	install -d "$(USER_BINDIR)"
	install -m 755 "$(TARGET_BIN)" "$(USER_BINDIR)/$(BIN_NAME)"
	install -d "$(USER_DATADIR)/applications"
	install -m 644 "$(DESKTOP_FILE)" "$(USER_DATADIR)/applications/$(DESKTOP_FILE)"
	install -d "$(USER_AUTOSTARTDIR)"
	install -m 644 "$(DESKTOP_FILE)" "$(USER_AUTOSTARTDIR)/$(DESKTOP_FILE)"

uninstall-user:
	rm -f "$(USER_BINDIR)/$(BIN_NAME)"
	rm -f "$(USER_DATADIR)/applications/$(DESKTOP_FILE)"
	rm -f "$(USER_AUTOSTARTDIR)/$(DESKTOP_FILE)"
