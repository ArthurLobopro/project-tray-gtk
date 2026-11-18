//! Ícone e menu do system tray (StatusNotifierItem via ksni).

use crate::data::{Data, Project};
use crate::launcher;
use ksni::menu::{StandardItem, SubMenu};
use ksni::MenuItem;
use std::sync::Mutex;

/// Dados para a janela de edição de projeto.
#[derive(Clone, Debug)]
pub struct EditProjectInfo {
    pub category_index: Option<usize>,
    pub project_index: usize,
    pub project: Project,
    pub current_category: Option<String>,
    pub all_categories: Vec<String>,
}

/// Dados para a janela de renomeação de categoria.
#[derive(Clone, Debug)]
pub struct EditCategoryInfo {
    pub category_index: usize,
    pub current_name: String,
}

/// Mensagens do tray para a thread do GTK (que desenha as janelas).
pub enum Msg {
    AddProject(Vec<String>), // nomes das categorias existentes
    AddCategory,
    EditProject(EditProjectInfo),
    EditCategory(EditCategoryInfo),
}

pub struct AppTray {
    data: Mutex<Data>,
    tx: async_channel::Sender<Msg>,
}

impl AppTray {
    pub fn new(data: Data, tx: async_channel::Sender<Msg>) -> Self {
        Self {
            data: Mutex::new(data),
            tx,
        }
    }

    fn persist(&self) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .save()
            .map_err(|e| format!("Falha ao salvar data.json: {e}"))
    }

    // Chamados pela janela GTK via `handle.update(...)`.
    pub fn add_project(&mut self, category: Option<&str>, project: Project) -> Result<(), String> {
        self.data.lock().unwrap().add_project(category, project)?;
        self.persist()
    }

    pub fn edit_project(
        &mut self,
        from_category: Option<usize>,
        from_index: usize,
        target_category: Option<&str>,
        project: Project,
    ) -> Result<(), String> {
        self.data
            .lock()
            .unwrap()
            .edit_project(from_category, from_index, target_category, project)?;
        self.persist()
    }

    pub fn add_category(&mut self, name: &str) -> Result<(), String> {
        self.data.lock().unwrap().add_category(name)?;
        self.persist()
    }

    pub fn rename_category(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        self.data.lock().unwrap().rename_category(index, new_name)?;
        self.persist()
    }

    fn remove_project(&mut self, category: Option<usize>, index: usize) {
        self.data.lock().unwrap().remove_project(category, index);
        let _ = self.persist();
    }

    fn remove_category(&mut self, index: usize) {
        self.data.lock().unwrap().remove_category(index);
        let _ = self.persist();
    }

    /// Submenu de um projeto: terminal / editores / agentes de IA / editar / remover.
    fn project_item(
        &self,
        data: &Data,
        project: &Project,
        category: Option<usize>,
        index: usize,
        available_launchers: &[&crate::data::Launcher],
        all_categories: &[String],
    ) -> MenuItem<Self> {
        let mut items: Vec<MenuItem<Self>> = Vec::new();
        let mut last_group: Option<&str> = None;

        for l in available_launchers {
            if last_group.is_some() && last_group != Some(l.group.as_str()) {
                items.push(MenuItem::Separator);
            }
            last_group = Some(l.group.as_str());

            let (launcher, path) = ((*l).clone(), project.path.clone());
            let icon_name = launcher::resolve_icon(l);
            items.push(
                StandardItem {
                    label: l.name.clone(),
                    icon_name,
                    activate: Box::new(move |_| launcher::open(&launcher, &path)),
                    ..Default::default()
                }
                .into(),
            );
        }

        if items.is_empty() {
            items.push(
                StandardItem {
                    label: "(nenhum aplicativo instalado)".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);

        let edit_info = EditProjectInfo {
            category_index: category,
            project_index: index,
            project: project.clone(),
            current_category: category
                .and_then(|ci| data.categories.get(ci))
                .map(|c| c.name.clone()),
            all_categories: all_categories.to_vec(),
        };

        items.push(
            StandardItem {
                label: "Editar projeto…".into(),
                icon_name: "document-edit-symbolic".into(),
                activate: Box::new(move |t: &mut Self| {
                    let _ = t.tx.send_blocking(Msg::EditProject(edit_info.clone()));
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(
            StandardItem {
                label: "Remover projeto".into(),
                icon_name: "edit-delete-symbolic".into(),
                activate: Box::new(move |t: &mut Self| t.remove_project(category, index)),
                ..Default::default()
            }
            .into(),
        );

        SubMenu {
            label: project.name.clone(),
            icon_name: "folder".into(),
            submenu: items,
            ..Default::default()
        }
        .into()
    }
}

impl ksni::Tray for AppTray {
    fn id(&self) -> String {
        "project-tray".into()
    }

    fn title(&self) -> String {
        "Project Tray".into()
    }

    fn icon_name(&self) -> String {
        "folder".into()
    }

    fn item_is_menu(&self) -> bool {
        true
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        // Recarrega o json sempre que o menu da tray for aberto/gerado
        let mut data_guard = self.data.lock().unwrap();
        *data_guard = Data::load();
        let data = &*data_guard;

        let mut menu: Vec<MenuItem<Self>> = Vec::new();
        let available_launchers: Vec<&crate::data::Launcher> = data
            .launchers
            .iter()
            .filter(|l| launcher::is_available(l))
            .collect();
        let all_categories = data.category_names();

        // 1) categorias primeiro
        for (ci, cat) in data.categories.iter().enumerate() {
            let mut sub: Vec<MenuItem<Self>> = cat
                .projects
                .iter()
                .enumerate()
                .map(|(pi, p)| {
                    self.project_item(data, p, Some(ci), pi, &available_launchers, &all_categories)
                })
                .collect();

            if sub.is_empty() {
                sub.push(
                    StandardItem {
                        label: "(categoria vazia)".into(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into(),
                );
            }
            sub.push(MenuItem::Separator);

            let cat_name = cat.name.clone();
            sub.push(
                StandardItem {
                    label: "Renomear categoria…".into(),
                    icon_name: "document-edit-symbolic".into(),
                    activate: Box::new(move |t: &mut Self| {
                        let _ = t.tx.send_blocking(Msg::EditCategory(EditCategoryInfo {
                            category_index: ci,
                            current_name: cat_name.clone(),
                        }));
                    }),
                    ..Default::default()
                }
                .into(),
            );

            sub.push(
                StandardItem {
                    label: "Remover categoria".into(),
                    icon_name: "edit-delete-symbolic".into(),
                    activate: Box::new(move |t: &mut Self| t.remove_category(ci)),
                    ..Default::default()
                }
                .into(),
            );

            menu.push(
                SubMenu {
                    label: cat.name.clone(),
                    icon_name: "folder-open".into(),
                    submenu: sub,
                    ..Default::default()
                }
                .into(),
            );
        }

        // 2) projetos soltos
        if !data.categories.is_empty() && !data.projects.is_empty() {
            menu.push(MenuItem::Separator);
        }
        for (pi, p) in data.projects.iter().enumerate() {
            menu.push(self.project_item(data, p, None, pi, &available_launchers, &all_categories));
        }

        if data.categories.is_empty() && data.projects.is_empty() {
            menu.push(
                StandardItem {
                    label: "Nenhum projeto cadastrado".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        // 3) ações
        menu.push(MenuItem::Separator);
        menu.push(
            StandardItem {
                label: "Adicionar projeto…".into(),
                icon_name: "list-add-symbolic".into(),
                activate: Box::new(|t: &mut Self| {
                    let cats = t.data.lock().unwrap().category_names();
                    let _ = t.tx.send_blocking(Msg::AddProject(cats));
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(
            StandardItem {
                label: "Nova categoria…".into(),
                icon_name: "folder-new-symbolic".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send_blocking(Msg::AddCategory);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(MenuItem::Separator);
        menu.push(
            StandardItem {
                label: "Sair".into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        );

        menu
    }
}
