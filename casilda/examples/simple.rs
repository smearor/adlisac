use casilda::Compositor;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use GtkWindowExt;

fn main() {
    let app = Application::builder()
        .application_id("io.smearor.casilda.simple")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Casilda Compositor Host")
            .default_width(1024)
            .default_height(768)
            .build();

        let compositor = Compositor::new("wayland-drift-0");
        let child = Some(&compositor);
        window.set_child(child);
        window.present();
    });

    app.run();
}