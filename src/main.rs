mod data;
mod launcher;
mod tray;
mod ui;

use gtk::prelude::*;
use gtk::{gio, glib};
use tray::{AppTray, Msg};

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("dev.projecttray.App")
        .build();

    // Instância única: se já estiver rodando, esta sai.
    if app.register(gio::Cancellable::NONE).is_ok() && app.is_remote() {
        return glib::ExitCode::SUCCESS;
    }

    // O tray roda em uma thread própria (D-Bus); o GTK desenha as janelas.
    let (tx, rx) = async_channel::unbounded::<Msg>();
    let service = ksni::TrayService::new(AppTray::new(data::Data::load(), tx));
    let handle = service.handle();
    service.spawn();

    app.connect_startup(move |app| {
        // Sem janelas abertas o app continuaria encerrando; `hold` mantém vivo.
        std::mem::forget(app.hold());

        let app = app.clone();
        let handle = handle.clone();
        let rx = rx.clone();

        glib::spawn_future_local(async move {
            while let Ok(msg) = rx.recv().await {
                let handle = handle.clone();
                match msg {
                    Msg::AddProject(categories) => {
                        ui::add_project_window(&app, categories, move |cat, project| {
                            handle.update(|t| t.add_project(cat.as_deref(), project))
                        });
                    }
                    Msg::AddCategory => {
                        ui::add_category_window(&app, move |name| {
                            handle.update(|t| t.add_category(&name))
                        });
                    }
                }
            }
        });
    });

    app.connect_activate(|_| {});
    app.run()
}
