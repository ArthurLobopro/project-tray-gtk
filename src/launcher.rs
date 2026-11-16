//! Abre uma pasta de projeto no terminal, editor ou agente de IA.

use crate::data::Launcher;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn open(launcher: &Launcher, path: &str) {
    // Divide o template antes de substituir {path}, para suportar espaços na pasta.
    let args: Vec<String> = launcher
        .command
        .split_whitespace()
        .map(|token| token.replace("{path}", path))
        .collect();

    let cmd = if launcher.in_terminal {
        terminal_command(path, &args)
    } else if let Some((prog, rest)) = args.split_first() {
        let mut c = Command::new(prog);
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

fn in_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| Path::new(&dir).join(bin).is_file()))
        .unwrap_or(false)
}

/// Monta o comando do emulador de terminal disponível. `args` vazio = só abre o shell.
fn terminal_command(path: &str, args: &[String]) -> Option<Command> {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(t) = std::env::var("TERMINAL") {
        if !t.is_empty() {
            candidates.push(t);
        }
    }
    for t in [
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
    ] {
        candidates.push(t.into());
    }

    let term = candidates.into_iter().find(|t| in_path(t))?;
    let joined = args.join(" ");
    let mut c = Command::new(&term);

    match term.as_str() {
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
