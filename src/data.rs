//! Modelo de dados e persistência em ~/.config/project-tray/data.json

use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

pub const APP_DIR: &str = "project-tray";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub projects: Vec<Project>,
}

/// Uma forma de abrir um projeto. `{path}` no comando é trocado pela pasta.
/// Edite o JSON para adicionar seus próprios editores/agentes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Launcher {
    pub name: String,
    pub group: String,
    /// Vazio + `in_terminal: true` = apenas abre um terminal na pasta.
    pub command: String,
    #[serde(default)]
    pub in_terminal: bool,
    /// Nome do ícone no tema FreeDesktop (ex: "vscode", "zed", "system-file-manager")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    /// Categorias aparecem primeiro no menu.
    #[serde(default)]
    pub categories: Vec<Category>,
    /// Projetos soltos (sem categoria).
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default = "default_launchers")]
    pub launchers: Vec<Launcher>,
}

fn launcher(name: &str, group: &str, command: &str, in_terminal: bool, icon: Option<&str>) -> Launcher {
    Launcher {
        name: name.into(),
        group: group.into(),
        command: command.into(),
        in_terminal,
        icon: icon.map(String::from),
    }
}

pub fn default_launchers() -> Vec<Launcher> {
    vec![
        launcher("Abrir no terminal", "Terminal", "", true, Some("utilities-terminal")),
        launcher("VS Code", "Editores", "code {path}", false, Some("vscode")),
        launcher("Cursor", "Editores", "cursor {path}", false, Some("cursor")),
        launcher("Zed", "Editores", "zed {path}", false, Some("zed")),
        launcher("Claude Code", "Agentes de IA", "claude", true, Some("utilities-terminal")),
        launcher("Gemini CLI", "Agentes de IA", "gemini", true, Some("utilities-terminal")),
        launcher("Codex", "Agentes de IA", "codex", true, Some("utilities-terminal")),
        launcher("Gerenciador de arquivos", "Arquivos", "xdg-open {path}", false, Some("system-file-manager")),
    ]
}

impl Default for Data {
    fn default() -> Self {
        Self {
            categories: vec![],
            projects: vec![],
            launchers: default_launchers(),
        }
    }
}

impl Data {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_DIR)
            .join("data.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("data.json inválido ({e}); salvando backup em data.json.bak");
                    let _ = fs::rename(&path, path.with_extension("json.bak"));
                    Self::fresh()
                }
            },
            Err(_) => Self::fresh(),
        }
    }

    fn fresh() -> Self {
        let data = Self::default();
        if let Err(e) = data.save() {
            eprintln!("não foi possível criar data.json: {e}");
        }
        data
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::config_path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // escrita atômica: grava em arquivo temporário e renomeia
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(&tmp, json)?;
        fs::rename(tmp, path)
    }

    pub fn category_names(&self) -> Vec<String> {
        self.categories.iter().map(|c| c.name.clone()).collect()
    }

    pub fn add_category(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Informe um nome para a categoria.".into());
        }
        if self
            .categories
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(name))
        {
            return Err("Já existe uma categoria com esse nome.".into());
        }
        self.categories.push(Category {
            name: name.into(),
            projects: vec![],
        });
        Ok(())
    }

    pub fn add_project(&mut self, category: Option<&str>, project: Project) -> Result<(), String> {
        let list = match category {
            None => &mut self.projects,
            Some(name) => match self.categories.iter_mut().find(|c| c.name == name) {
                Some(c) => &mut c.projects,
                None => return Err(format!("Categoria \"{name}\" não encontrada.")),
            },
        };
        if list.iter().any(|p| p.path == project.path) {
            return Err("Essa pasta já está cadastrada neste local.".into());
        }
        list.push(project);
        Ok(())
    }

    pub fn remove_project(&mut self, category: Option<usize>, index: usize) {
        let list = match category {
            None => Some(&mut self.projects),
            Some(i) => self.categories.get_mut(i).map(|c| &mut c.projects),
        };
        if let Some(list) = list {
            if index < list.len() {
                list.remove(index);
            }
        }
    }

    pub fn remove_category(&mut self, index: usize) {
        if index < self.categories.len() {
            self.categories.remove(index);
        }
    }
}
