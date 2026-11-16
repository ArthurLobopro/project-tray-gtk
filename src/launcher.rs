//! Abre uma pasta de projeto no terminal, editor ou agente de IA.

use crate::data::Launcher;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn open(launcher: &Launcher, path: &str) {
    // Divide o template antes de substituir {path}, para suportar espaços na pasta.
    let mut args: Vec<String> = launcher
        .command
        .split_whitespace()
        .map(|token| token.replace("{path}", path))
        .collect();

    let cmd = if launcher.in_terminal {
        if let Some(first) = args.first_mut() {
            if let Some(resolved) = resolve_binary(first) {
                *first = resolved;
            }
        }
        terminal_command(path, &args)
    } else if let Some((prog, rest)) = args.split_first() {
        let prog = resolve_binary(prog).unwrap_or_else(|| prog.to_string());
        let mut c = Command::new(&prog);
        c.args(rest);
        Some(c)
    } else {
        None
    };

    match cmd {
        Some(c) => spawn_detached(c, path),
        None => eprintln!("nada para executar em \"{}\"", launcher.name),
    }
}

fn spawn_detached(mut cmd: Command, cwd: &str) {
    cmd.current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0); // desacopla do tray

    match cmd.spawn() {
        // evita processos zumbis
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => eprintln!("falha ao executar {:?}: {e}", cmd.get_program()),
    }
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
        .unwrap_or(false)
}

pub fn in_path(bin: &str) -> bool {
    if bin.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return is_executable(&home.join(&bin[2..]));
        }
    }
    if bin.contains('/') {
        return is_executable(Path::new(bin));
    }
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(bin)))
        })
        .unwrap_or(false)
}

fn terminal_candidates() -> impl Iterator<Item = String> {
    let env_term = std::env::var("TERMINAL").ok().filter(|t| !t.is_empty());
    let defaults = [
        "kitty",
        "alacritty",
        "wezterm",
        "foot",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "ptyxis",
        "tilix",
        "x-terminal-emulator",
        "xterm",
    ]
    .into_iter()
    .map(String::from);

    env_term.into_iter().chain(defaults)
}

pub fn has_terminal() -> bool {
    terminal_candidates().any(|t| in_path(&t))
}

fn detect_terminal() -> Option<String> {
    terminal_candidates().find(|t| in_path(t))
}

fn binary_alternatives(bin: &str) -> &'static [&'static str] {
    match bin {
        "zed" | "zeditor" => &["zed", "zeditor"],
        "code" | "codium" | "code-oss" => &["code", "codium", "code-oss"],
        "cursor" | "cursor-editor" | "cursor-bin" => &["cursor", "cursor-editor", "cursor-bin"],
        _ => &[],
    }
}

/// Procura o binário informado ou suas variantes conhecidas no PATH.
pub fn resolve_binary(bin: &str) -> Option<String> {
    if in_path(bin) {
        return Some(bin.to_string());
    }
    for alt in binary_alternatives(bin) {
        if in_path(alt) {
            return Some(alt.to_string());
        }
    }
    None
}

/// Verifica se o executável do launcher está presente e pronto para uso no sistema.
pub fn is_available(launcher: &Launcher) -> bool {
    if launcher.in_terminal && !has_terminal() {
        return false;
    }

    let trimmed = launcher.command.trim();
    if trimmed.is_empty() {
        // Sem comando explícito: só é válido se abrir no terminal
        return launcher.in_terminal;
    }

    let Some(prog) = trimmed.split_whitespace().next() else {
        return false;
    };

    resolve_binary(prog).is_some()
}

/// Monta o comando do emulador de terminal disponível. `args` vazio = só abre o shell.
fn terminal_command(path: &str, args: &[String]) -> Option<Command> {
    let term = detect_terminal()?;
    let joined = args.join(" ");
    let mut c = Command::new(&term);

    let term_name = Path::new(&term)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&term);

    match term_name {
        "gnome-terminal" => {
            c.arg(format!("--working-directory={path}"));
            if !args.is_empty() {
                c.arg("--").args(args);
            }
        }
        "konsole" => {
            c.args(["--workdir", path]);
            if !args.is_empty() {
                c.arg("-e").args(args);
            }
        }
        "kitty" => {
            c.args(["--directory", path]).args(args);
        }
        "alacritty" => {
            c.args(["--working-directory", path]);
            if !args.is_empty() {
                c.arg("-e").args(args);
            }
        }
        "xfce4-terminal" => {
            c.arg(format!("--working-directory={path}"));
            if !args.is_empty() {
                c.arg("-x").args(args);
            }
        }
        "wezterm" => {
            c.args(["start", "--cwd", path]);
            if !args.is_empty() {
                c.arg("--").args(args);
            }
        }
        "foot" => {
            c.arg(format!("--working-directory={path}")).args(args);
        }
        "ptyxis" => {
            c.args(["--new-window", "-d", path]);
            if !args.is_empty() {
                c.args(["-x", &joined]);
            }
        }
        "tilix" => {
            c.args(["-w", path]);
            if !args.is_empty() {
                c.args(["-e", &joined]);
            }
        }
        // xterm, x-terminal-emulator, $TERMINAL desconhecido: usa o cwd do processo
        _ => {
            if !args.is_empty() {
                c.arg("-e").args(args);
            }
        }
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_existent_binary_is_not_available() {
        let l = Launcher {
            name: "Fake".into(),
            group: "Editores".into(),
            command: "non_existent_binary_123456 {path}".into(),
            in_terminal: false,
        };
        assert!(!is_available(&l));
    }

    #[test]
    fn test_existing_system_binary_is_available() {
        let l = Launcher {
            name: "Shell".into(),
            group: "Sistema".into(),
            command: "sh".into(),
            in_terminal: false,
        };
        assert!(is_available(&l));
    }

    #[test]
    fn test_zed_resolves_to_zeditor_if_installed() {
        if in_path("zeditor") {
            let l = Launcher {
                name: "Zed".into(),
                group: "Editores".into(),
                command: "zed {path}".into(),
                in_terminal: false,
            };
            assert!(is_available(&l));
            assert_eq!(resolve_binary("zed"), Some("zeditor".into()));
        }
    }
}
