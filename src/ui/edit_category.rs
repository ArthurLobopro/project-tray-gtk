//! Janela "Renomear categoria".

use crate::ui::common::{
    base_window, buttons_row, error_label, field_label, root_box, show_error,
};
use gtk::glib;
use gtk::prelude::*;

/// Janela "Renomear categoria".
pub fn edit_category_window<F>(app: &gtk::Application, current_name: String, on_save: F)
where
    F: Fn(String) -> Result<(), String> + 'static,
{
    let win = base_window(app, "Renomear categoria");

    let name_entry = gtk::Entry::builder()
        .hexpand(true)
        .placeholder_text("Nome da categoria")
        .text(&current_name)
        .activates_default(true)
        .build();

    let grid = gtk::Grid::builder().column_spacing(12).build();
    grid.attach(&field_label("Nome"), 0, 0, 1, 1);
    grid.attach(&name_entry, 1, 0, 1, 1);

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
