//! Ícone e menu do system tray (StatusNotifierItem via ksni).

use crate::data::{Data, Project};
use crate::launcher;
use ksni::menu::{StandardItem, SubMenu};
use ksni::MenuItem;

/// Mensagens do tray para a thread do GTK (que desenha as janelas).
pub enum Msg {
    AddProject(Vec<String>), // nomes das categorias existentes
    AddCategory,
}

pub struct AppTray {
    data: Data,
    tx: async_channel::Sender<Msg>,
}

impl AppTray {
    pub fn new(data: Data, tx: async_channel::Sender<Msg>) -> Self {
        Self { data, tx }
    }

    fn persist(&self) -> Result<(), String> {
        self.data
            .save()
            .map_err(|e| format!("Falha ao salvar data.json: {e}"))
    }

    // Chamados pela janela GTK via `handle.update(...)`.
    pub fn add_project(&mut self, category: Option<&str>, project: Project) -> Result<(), String> {
        self.data.add_project(category, project)?;
        self.persist()
    }

    pub fn add_category(&mut self, name: &str) -> Result<(), String> {
        self.data.add_category(name)?;
        self.persist()
    }

    fn remove_project(&mut self, category: Option<usize>, index: usize) {
        self.data.remove_project(category, index);
        let _ = self.persist();
    }

    fn remove_category(&mut self, index: usize) {
        self.data.remove_category(index);
        let _ = self.persist();
    }

    /// Submenu de um projeto: terminal / editores / agentes de IA / remover.
    fn project_item(
        &self,
        project: &Project,
        category: Option<usize>,
        index: usize,
        available_launchers: &[&crate::data::Launcher],
    ) -> MenuItem<Self> {
        let mut items: Vec<MenuItem<Self>> = Vec::new();
        let mut last_group: Option<&str> = None;

        for l in available_launchers {
            if last_group.is_some() && last_group != Some(l.group.as_str()) {
                items.push(MenuItem::Separator);
            }
            last_group = Some(l.group.as_str());

            let (launcher, path) = ((*l).clone(), project.path.clone());
            items.push(
                StandardItem {
                    label: l.name.clone(),
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
        items.push(
            StandardItem {
                label: "Remover projeto".into(),
                icon_name: "list-remove-symbolic".into(),
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

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut menu: Vec<MenuItem<Self>> = Vec::new();
        let available_launchers: Vec<&crate::data::Launcher> = self
            .data
            .launchers
            .iter()
            .filter(|l| launcher::is_available(l))
            .collect();

        // 1) categorias primeiro
        for (ci, cat) in self.data.categories.iter().enumerate() {
            let mut sub: Vec<MenuItem<Self>> = cat
                .projects
                .iter()
                .enumerate()
                .map(|(pi, p)| self.project_item(p, Some(ci), pi, &available_launchers))
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
            sub.push(
                StandardItem {
                    label: "Remover categoria".into(),
                    icon_name: "list-remove-symbolic".into(),
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
        if !self.data.categories.is_empty() && !self.data.projects.is_empty() {
            menu.push(MenuItem::Separator);
        }
        for (pi, p) in self.data.projects.iter().enumerate() {
            menu.push(self.project_item(p, None, pi, &available_launchers));
        }

        if self.data.categories.is_empty() && self.data.projects.is_empty() {
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
                    let _ = t.tx.send_blocking(Msg::AddProject(t.data.category_names()));
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
        menu.push(
            StandardItem {
                label: "Recarregar data.json".into(),
                icon_name: "view-refresh-symbolic".into(),
                activate: Box::new(|t: &mut Self| t.data = Data::load()),
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
