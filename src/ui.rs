//! Janelas GTK 4: cadastro de projeto e de categoria.

use crate::data::Project;
use gtk::prelude::*;
use gtk::{gio, glib};

fn base_window(app: &gtk::Application, title: &str) -> gtk::Window {
    gtk::Window::builder()
        .application(app)
        .title(title)
        .default_width(480)
        .resizable(false)
        .build()
}

fn root_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(14)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build()
}

fn field_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .halign(gtk::Align::End)
        .build()
}

fn error_label() -> gtk::Label {
    gtk::Label::builder()
        .css_classes(["error"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .visible(false)
        .build()
}

fn show_error(label: &gtk::Label, msg: &str) {
    label.set_text(msg);
    label.set_visible(true);
}

fn buttons_row(cancel: &gtk::Button, save: &gtk::Button) -> gtk::Box {
    let row = gtk::Box::builder()
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    row.append(cancel);
    row.append(save);
    row
}

fn expand_tilde(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    p.to_string()
}

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

/// Janela "Nova categoria": apenas o nome.
pub fn add_category_window<F>(app: &gtk::Application, on_save: F)
where
    F: Fn(String) -> Result<(), String> + 'static,
{
    let win = base_window(app, "Nova categoria");

    let name_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("Nome da categoria")
        .activates_default(true)
        .build();

    let grid = gtk::Grid::builder().column_spacing(12).build();
    grid.attach(&field_label("Nome"), 0, 0, 1, 1);
    grid.attach(&name_entry, 1, 0, 1, 1);

    let error = error_label();
    let cancel = gtk::Button::with_label("Cancelar");
    let save = gtk::Button::builder()
        .label("Criar")
        .css_classes(["suggested-action"])
        .build();

    let root = root_box();
    root.append(&grid);
    root.append(&error);
    root.append(&buttons_row(&cancel, &save));
    win.set_child(Some(&root));
    win.set_default_widget(Some(&save));

    cancel.connect_clicked(glib::clone!(
        #[weak]
        win,
        move |_| win.close()
    ));

    save.connect_clicked(glib::clone!(
        #[weak]
        win,
        #[weak]
        name_entry,
        #[weak]
        error,
        move |_| match on_save(name_entry.text().trim().to_string()) {
            Ok(()) => win.close(),
            Err(e) => show_error(&error, &e),
        }
    ));

    win.present();
}
