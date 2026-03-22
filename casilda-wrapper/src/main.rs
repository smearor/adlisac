use casilda::Compositor;
use glib::SpawnFlags;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::ffi::OsString;
use std::time::Duration;
use GtkWindowExt;
use layout::RotatedBox;
use rotation::SmearorRotation;

pub mod layout;
pub mod rotation;

// WORK IN PROGRESS

fn main() {
    let app = Application::builder()
        .application_id("io.smearor.casilda.simple")
        .build();
    app.connect_activate(|app| {
        let socket_name = String::from("/tmp/io.smearor.casilda.simple.1.sock");
        let window = ApplicationWindow::builder()
            .application(app)
            .title("io.smearor.casilda.simple")
            // .decorated(false)
            .opacity(1.0)
            .default_width(1280)
            .default_height(1024)
            .build();

        let compositor = Compositor::builder()
            .socket(&socket_name)
            .build();
        compositor.set_hexpand(true);
        compositor.set_vexpand(true);
        compositor.set_opacity(1.0);
        compositor.set_overflow(gtk4::Overflow::Visible);
        compositor.set_widget_name("compositor");
        compositor.set_size_request(1280, 1024);

        let provider = gtk4::CssProvider::new();
        provider.load_from_data("
            window {
                background-color: transparent;
            }
            #compositor {
                background-color: transparent;
            }
        ");

        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // let rotated_container = RotatedBox::new(SmearorRotation::Deg180);
        let rotated_container = RotatedBox::new(SmearorRotation::Deg(45.0));
        // rotated_container.set_size_request(1280, 1024);
        rotated_container.set_child(Some(&compositor));
        rotated_container.set_hexpand(true);
        rotated_container.set_vexpand(true);
        rotated_container.set_rotation(SmearorRotation::Deg(90.0));

        window.set_child(Some(&rotated_container));
        // window.set_child(Some(&compositor));
        // window.set_opacity(0.9);

        glib::timeout_add_local(std::time::Duration::from_millis(500), move || {

            // compositor.set_opacity(0.9);
            // compositor.set_property("transform", 2i32);

            match compositor.spawn_async(
                None,
                vec![
                    OsString::from("/usr/bin/gnome-calculator"),
                    // OsString::from("/usr/bin/alacritty"),
                    // OsString::from("--option"),
                    // OsString::from("window.decorations=\"None\""),
                    // OsString::from("--option"),
                    // OsString::from("window.startup_mode=\"Fullscreen\""),
                    // OsString::from("--option"),
                    // OsString::from("window.position.x=0"),
                    // OsString::from("--option"),
                    // OsString::from("window.position.y=0"),
                    // OsString::from("--option"),
                    // OsString::from("window.padding.x=0"),
                    // OsString::from("--option"),
                    // OsString::from("window.padding.y=0"),
                    // OsString::from("--option"),
                    // OsString::from("window.opacity=0.5"),
                    // OsString::from("--option"),
                    // OsString::from("font.size=14"),
                ],
                vec![
                    OsString::from("GDK_BACKEND=wayland"),
                    OsString::from(format!("WAYLAND_DISPLAY={socket_name}")),
                    OsString::from("WINIT_UNIX_BACKEND=wayland"),
                ],
                SpawnFlags::DEFAULT
            ) {
                Ok(pid) => println!("pid {}", pid.0),
                Err(e) => eprintln!("error: {e}"),
            }
            glib::ControlFlow::Break
        });

        let rotation_state = std::rc::Rc::new(std::cell::Cell::new(0.0));
        glib::timeout_add_local(Duration::from_millis(16), move || {
            let current_angle = rotation_state.get();
            let next_angle = (current_angle + 0.1) % 360.0;
            rotation_state.set(next_angle);
            rotated_container.set_rotation(SmearorRotation::Deg(next_angle));
            glib::ControlFlow::Continue
        });

        window.present();
    });

    app.run();
}