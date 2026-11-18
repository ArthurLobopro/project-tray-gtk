//! Janela "Adicionar projeto": pasta + nome (+ categoria opcional).

use crate::data::Project;
use crate::ui::common::{
    base_window, buttons_row, error_label, expand_tilde, field_label, root_box, show_error,
};
use gtk::prelude::*;
use gtk::{gio, glib};

/// Janela "Adicionar projeto": pasta + nome (+ categoria opcional).
pub fn add_project_window<F>(app: &gtk::Application, categories: Vec<String>, on_save: F)
where
    F: Fn(Option<String>, Project) -> Result<(), String> + 'static,
{
    let win = base_window(app, "Adicionar projeto");

    let path_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("/caminho/do/projeto")
        .build();
    let browse = gtk::Button::with_label("Escolher…");
    let path_row = gtk::Box::builder().spacing(6).build();
    path_row.append(&path_entry);
    path_row.append(&browse);

    let name_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("Nome do projeto (padrão: nome da pasta)")
        .activates_default(true)
        .build();

    let mut options = vec!["Sem categoria".to_string()];
    options.extend(categories);
    let refs: Vec<&str> = options.iter().map(String::as_str).collect();
    let dropdown = gtk::DropDown::from_strings(&refs);

    let grid = gtk::Grid::builder().row_spacing(10).column_spacing(12).build();
    grid.attach(&field_label("Pasta"), 0, 0, 1, 1);
    grid.attach(&path_row, 1, 0, 1, 1);
    grid.attach(&field_label("Nome"), 0, 1, 1, 1);
    grid.attach(&name_entry, 1, 1, 1, 1);
    grid.attach(&field_label("Categoria"), 0, 2, 1, 1);
    grid.attach(&dropdown, 1, 2, 1, 1);

    let error = error_label();
    let cancel = gtk::Button::with_label("Cancelar");
    let save = gtk::Button::builder()
        .label("Salvar")
        .css_classes(["suggested-action"])
        .build();

    let root = root_box();
    root.append(&grid);
    root.append(&error);
    root.append(&buttons_row(&cancel, &save));
    win.set_child(Some(&root));
    win.set_default_widget(Some(&save));

    // Seletor de pasta nativo
    browse.connect_clicked(glib::clone!(
        #[weak]
        win,
        #[weak]
        path_entry,
        #[weak]
        name_entry,
        move |_| {
            let dialog = gtk::FileDialog::builder()
                .title("Escolha a pasta do projeto")
                .modal(true)
                .build();
            dialog.select_folder(
                Some(&win),
                gio::Cancellable::NONE,
                glib::clone!(
                    #[weak]
                    path_entry,
                    #[weak]
                    name_entry,
                    move |res| {
                        let Ok(file) = res else { return };
                        let Some(path) = file.path() else { return };
                        path_entry.set_text(&path.to_string_lossy());
                        if name_entry.text().trim().is_empty() {
                            if let Some(n) = path.file_name() {
                                name_entry.set_text(&n.to_string_lossy());
                            }
                        }
                    }
                ),
            );
        }
    ));

    cancel.connect_clicked(glib::clone!(
        #[weak]
        win,
        move |_| win.close()
    ));

    save.connect_clicked(glib::clone!(
        #[weak]
        win,
        #[weak]
        path_entry,
        #[weak]
        name_entry,
        #[weak]
        dropdown,
        #[weak]
        error,
        move |_| {
            let path = expand_tilde(path_entry.text().trim());
            let dir = std::path::Path::new(&path);
            if path.is_empty() || !dir.is_dir() {
                show_error(&error, "Escolha uma pasta existente.");
                return;
            }

            let mut name = name_entry.text().trim().to_string();
            if name.is_empty() {
                name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
            }

            let category = match dropdown.selected() {
                0 => None,
                i => options.get(i as usize).cloned(),
            };

            match on_save(category, Project { name, path }) {
                Ok(()) => win.close(),
                Err(e) => show_error(&error, &e),
            }
        }
    ));

    win.present();
}
