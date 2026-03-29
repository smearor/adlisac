use crate::args::Args;
use adlisac_core::process::spawn_wayland_client;
use adlisac_gtk::compositor::AdlisacCompositor;
use adlisac_rotation::layout::RotatedBox;
use adlisac_rotation::rotation::SmearorRotation;
use clap::Parser;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4_layer_shell::Layer;
use gtk4_layer_shell::LayerShell;
use std::env;
use std::time::Duration;
use tracing::error;
use tracing::info;
use which::which;

pub mod args;

fn main() {
    // Force the OpenGL renderer for GSK to ensure better compatibility with textures.
    env::set_var("GSK_RENDERER", "gl");

    // Parse command line arguments.
    let command_line_arguments = Args::parse();

    let application = Application::builder().application_id("io.smearor.adlisac.wrapper").build();

    application.connect_activate(move |application_instance| {
        // Create the main application window.
        let main_window = ApplicationWindow::builder()
            .application(application_instance)
            .title("Adlisac Smart Desk Wrapper")
            .decorated(command_line_arguments.decorated)
            .build();

        // Enable Layer Shell support if needed (e.g., for desktop overlays).
        main_window.init_layer_shell();
        main_window.set_layer(Layer::Top);
        main_window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        main_window.set_namespace(Some("adlisac"));

        // Create the Adlisac Compositor widget.
        let adlisac_compositor = AdlisacCompositor::new();
        adlisac_compositor.set_hexpand(true);
        adlisac_compositor.set_vexpand(true);
        adlisac_compositor.set_size_request(command_line_arguments.width, command_line_arguments.height);

        // Create the rotation container.
        let rotation_container = RotatedBox::new(SmearorRotation::Deg(command_line_arguments.rotation));
        rotation_container.set_child(Some(&adlisac_compositor));
        rotation_container.set_hexpand(true);
        rotation_container.set_vexpand(true);

        // Add the rotation container to the window.
        main_window.set_child(Some(&rotation_container));

        // Prepare the client process arguments.
        let mut client_run_arguments = command_line_arguments.run_args.clone();

        // If no arguments provided, default to alacritty (terminal).
        if client_run_arguments.is_empty() {
            if let Ok(terminal_path) = which("alacritty") {
                client_run_arguments.push(terminal_path.into_os_string());
            } else if let Ok(term_path) = which("xterm") {
                client_run_arguments.push(term_path.into_os_string());
            }
        }

        // Delay the start of the client process slightly to ensure the compositor is ready.
        let compositor_socket_name = adlisac_compositor.socket_name();

        glib::timeout_add_local(Duration::from_millis(1000), move || {
            if !client_run_arguments.is_empty() {
                let executable_name = client_run_arguments[0].to_string_lossy().to_string();
                let remaining_arguments: Vec<String> = client_run_arguments.iter().skip(1).map(|arg| arg.to_string_lossy().to_string()).collect();

                match spawn_wayland_client(executable_name, remaining_arguments, compositor_socket_name.clone()) {
                    Ok(_) => info!("Client process started successfully on socket: {}", compositor_socket_name),
                    Err(spawn_error) => error!("Failed to spawn client: {:?}", spawn_error),
                }
            }
            glib::ControlFlow::Break
        });

        // Present the window.
        main_window.present();
    });

    // Run the GTK application.
    application.run_with_args::<&str>(&[]);
}
