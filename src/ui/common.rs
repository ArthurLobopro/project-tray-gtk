//! Widgets e helpers compartilhados para a interface gráfica GTK 4.

use gtk::prelude::*;

pub fn base_window(app: &gtk::Application, title: &str) -> gtk::Window {
    gtk::Window::builder()
        .application(app)
        .title(title)
        .default_width(480)
        .resizable(false)
        .build()
}

pub fn root_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(14)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build()
}

pub fn field_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .halign(gtk::Align::End)
        .build()
}

pub fn error_label() -> gtk::Label {
    gtk::Label::builder()
        .css_classes(["error"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .visible(false)
        .build()
}

pub fn show_error(label: &gtk::Label, msg: &str) {
    label.set_text(msg);
    label.set_visible(true);
}

pub fn buttons_row(cancel: &gtk::Button, save: &gtk::Button) -> gtk::Box {
    let row = gtk::Box::builder()
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    row.append(cancel);
    row.append(save);
    row
}

pub fn expand_tilde(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    p.to_string()
}
