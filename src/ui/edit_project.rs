//! Janela "Editar projeto": permite renomear, alterar pasta ou mudar de categoria.

use crate::data::Project;
use crate::ui::common::{
    base_window, buttons_row, error_label, expand_tilde, field_label, root_box, show_error,
};
use gtk::prelude::*;
use gtk::{gio, glib};

/// Janela "Editar projeto": permite renomear, alterar pasta ou mudar de categoria.
pub fn edit_project_window<F>(
    app: &gtk::Application,
    project: Project,
    current_category: Option<String>,
    categories: Vec<String>,
    on_save: F,
) where
    F: Fn(Option<String>, Project) -> Result<(), String> + 'static,
{
    let win = base_window(app, "Editar projeto");

    let path_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("/caminho/do/projeto")
        .text(&project.path)
        .build();
    let browse = gtk::Button::with_label("Escolher…");
    let path_row = gtk::Box::builder().spacing(6).build();
    path_row.append(&path_entry);
    path_row.append(&browse);

    let name_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("Nome do projeto")
        .text(&project.name)
        .activates_default(true)
        .build();

    let mut options = vec!["Sem categoria".to_string()];
    options.extend(categories);
    let refs: Vec<&str> = options.iter().map(String::as_str).collect();
    let dropdown = gtk::DropDown::from_strings(&refs);

    // Seleciona a categoria atual no dropdown
    let selected_index = match current_category.as_deref() {
        Some(cat) => options.iter().position(|o| o == cat).unwrap_or(0),
        None => 0,
    };
    dropdown.set_selected(selected_index as u32);

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

    browse.connect_clicked(glib::clone!(
        #[weak]
        win,
        #[weak]
        path_entry,
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
                    move |res| {
                        let Ok(file) = res else { return };
                        let Some(path) = file.path() else { return };
                        path_entry.set_text(&path.to_string_lossy());
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

            let name = name_entry.text().trim().to_string();
            if name.is_empty() {
                show_error(&error, "Informe um nome para o projeto.");
                return;
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
