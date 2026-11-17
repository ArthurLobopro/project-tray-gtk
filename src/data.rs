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

    pub fn rename_category(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err("Informe um nome para a categoria.".into());
        }
        if index >= self.categories.len() {
            return Err("Categoria não encontrada.".into());
        }
        if self
            .categories
            .iter()
            .enumerate()
            .any(|(i, c)| i != index && c.name.eq_ignore_ascii_case(new_name))
        {
            return Err("Já existe uma categoria com esse nome.".into());
        }
        self.categories[index].name = new_name.to_string();
        Ok(())
    }

    pub fn edit_project(
        &mut self,
        from_category: Option<usize>,
        from_index: usize,
        target_category: Option<&str>,
        new_project: Project,
    ) -> Result<(), String> {
        let new_name = new_project.name.trim();
        if new_name.is_empty() {
            return Err("O nome do projeto não pode ser vazio.".into());
        }
        let new_path = new_project.path.trim();
        if new_path.is_empty() {
            return Err("O caminho do projeto não pode ser vazio.".into());
        }

        let current_cat_name = from_category
            .and_then(|ci| self.categories.get(ci))
            .map(|c| c.name.as_str());

        let same_category = match (current_cat_name, target_category) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        if same_category {
            let list = match from_category {
                None => &mut self.projects,
                Some(ci) => match self.categories.get_mut(ci) {
                    Some(c) => &mut c.projects,
                    None => return Err("Categoria não encontrada.".into()),
                },
            };
            if from_index >= list.len() {
                return Err("Projeto não encontrado.".into());
            }
            if list[from_index].path != new_path
                && list
                    .iter()
                    .enumerate()
                    .any(|(i, p)| i != from_index && p.path == new_path)
            {
                return Err("Essa pasta já está cadastrada neste local.".into());
            }
            list[from_index].name = new_name.to_string();
            list[from_index].path = new_path.to_string();
        } else {
            // Verifica se a pasta já existe no destino
            let dest_exists = match target_category {
                None => self.projects.iter().any(|p| p.path == new_path),
                Some(name) => match self.categories.iter().find(|c| c.name == name) {
                    Some(c) => c.projects.iter().any(|p| p.path == new_path),
                    None => return Err(format!("Categoria \"{name}\" não encontrada.")),
                },
            };
            if dest_exists {
                return Err("Essa pasta já está cadastrada na categoria de destino.".into());
            }

            // Remove da origem
            let removed = match from_category {
                None => {
                    if from_index < self.projects.len() {
                        Some(self.projects.remove(from_index))
                    } else {
                        None
                    }
                }
                Some(ci) => {
                    if let Some(c) = self.categories.get_mut(ci) {
                        if from_index < c.projects.len() {
                            Some(c.projects.remove(from_index))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            if removed.is_none() {
                return Err("Projeto não encontrado na origem.".into());
            }

            // Insere no destino
            let dest_list = match target_category {
                None => &mut self.projects,
                Some(name) => {
                    &mut self
                        .categories
                        .iter_mut()
                        .find(|c| c.name == name)
                        .unwrap()
                        .projects
                }
            };
            dest_list.push(Project {
                name: new_name.to_string(),
                path: new_path.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_category() {
        let mut data = Data::default();
        data.add_category("Backend").unwrap();
        assert!(data.rename_category(0, "Backend API").is_ok());
        assert_eq!(data.categories[0].name, "Backend API");

        // Nome vazio deve falhar
        assert!(data.rename_category(0, "").is_err());
    }

    #[test]
    fn test_edit_project_rename_and_move_category() {
        let mut data = Data::default();
        data.add_category("Trabalho").unwrap();
        data.add_category("Pessoal").unwrap();

        data.add_project(
            Some("Trabalho"),
            Project {
                name: "App 1".into(),
                path: "/home/user/app1".into(),
            },
        )
        .unwrap();

        // 1. Renomear mantendo categoria
        data.edit_project(
            Some(0),
            0,
            Some("Trabalho"),
            Project {
                name: "App Renomeado".into(),
                path: "/home/user/app1".into(),
            },
        )
        .unwrap();
        assert_eq!(data.categories[0].projects[0].name, "App Renomeado");

        // 2. Mover para categoria Pessoal
        data.edit_project(
            Some(0),
            0,
            Some("Pessoal"),
            Project {
                name: "App Pessoal".into(),
                path: "/home/user/app1".into(),
            },
        )
        .unwrap();
        assert!(data.categories[0].projects.is_empty());
        assert_eq!(data.categories[1].projects.len(), 1);
        assert_eq!(data.categories[1].projects[0].name, "App Pessoal");

        // 3. Mover para Sem categoria (None)
        data.edit_project(
            Some(1),
            0,
            None,
            Project {
                name: "App Solto".into(),
                path: "/home/user/app1".into(),
            },
        )
        .unwrap();
        assert!(data.categories[1].projects.is_empty());
        assert_eq!(data.projects.len(), 1);
        assert_eq!(data.projects[0].name, "App Solto");

        // 4. Mover de Sem categoria para Trabalho
        data.edit_project(
            None,
            0,
            Some("Trabalho"),
            Project {
                name: "App de volta ao Trabalho".into(),
                path: "/home/user/app1".into(),
            },
        )
        .unwrap();
        assert!(data.projects.is_empty());
        assert_eq!(data.categories[0].projects.len(), 1);
    }
}
