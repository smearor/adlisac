use std::env;
use casilda::Compositor;
use glib::SpawnFlags;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use gtk4_layer_shell::LayerShell;
use GtkWindowExt;
use which::which;
use layout::RotatedBox;
use rotation::SmearorRotation;
use crate::args::Args;
use crate::layer::SmearorLayer;

pub mod layer;
pub mod layout;
pub mod rotation;
pub mod args;

// WORK IN PROGRESS

fn main() {
    env::set_var("GSK_RENDERER", "gl");
    let args = Args::parse();
    let run_args = args.run_args.clone();
    let rotation = args.rotation;
    let width = args.width;
    let height = args.height;
    let socket = args.socket.clone();

    let app = Application::builder()
        .application_id("io.smearor.casilda.simple")
        .build();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("io.smearor.casilda.simple")
            .decorated(args.decorated)
            .opacity(1.0)
            // .default_width(1200)
            // .default_height(1200)
            .build();
        window.init_layer_shell();
        window.set_layer(SmearorLayer::Top.into());

        let compositor = Compositor::builder()
            .socket(&socket)
            .build();
        compositor.set_hexpand(true);
        compositor.set_vexpand(true);
        compositor.set_opacity(1.0);
        compositor.set_overflow(gtk4::Overflow::Visible);
        compositor.set_widget_name("compositor");
        // compositor.set_size_request(1200, 1200);
        compositor.set_size_request(width, height);
        // compositor.connect_destroy(|compositor| {
        //     println!("destroy compositor");
        // });
        // compositor.connect_closure("toplevel-added",
        // false,
        // move |args| {
        //     let toplevel = args[1].get::<casilda::Toplevel>().unwrap();
        //     println!("Toplevel: {:?}", toplevel);
        //     toplevel.set_maximized(true);
        //     None
        //
        // });

        let provider = gtk4::CssProvider::new();
        provider.load_from_data("
            window {
                background-color: transparent;
                border: 1px solid red;
            }
            #rotated_container {
                border: 1px solid blue;
            }
            #compositor {
                background-color: transparent;
                border: 1px solid green;
            }
        ");

        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // let rotated_container = RotatedBox::new(SmearorRotation::Deg180);
        let rotated_container = RotatedBox::new(SmearorRotation::Deg(rotation));
        // rotated_container.set_size_request(1280, 1024);
        rotated_container.set_child(Some(&compositor));
        rotated_container.set_hexpand(true);
        rotated_container.set_vexpand(true);
        // rotated_container.set_rotation(SmearorRotation::Deg(args.rotation));
        // rotated_container.set_size_request(1200, 1200);
        rotated_container.set_widget_name("rotated_container");

        window.set_child(Some(&rotated_container));
        // window.set_child(Some(&compositor));
        // window.set_opacity(0.9);

        let mut run_args = run_args.clone();
        if args.shell {
            match which("sh") {
                Ok(sh_path) => {
                    run_args.insert(0, sh_path.as_os_str().into());
                    run_args.insert(1, "-c".into());
                }
                Err(e) => {
                    eprintln!("Couldn't find sh: {}", e);
                }
            }
        }

        let socket = socket.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {

            // compositor.set_opacity(0.9);
            // compositor.set_property("transform", 2i32);

            let mut env_vars: Vec<OsString> = env::vars_os()
                .filter(|(k, v)| { !k.to_string_lossy().to_uppercase().starts_with("WAYLAND_DISPLAY") })
                .map(|(k, v)| {
                    let mut res = k;
                    res.push("=");
                    res.push(v);
                    res
                })
                .collect();
            env_vars.push(OsString::from(format!("WAYLAND_DISPLAY={socket}")));
            env_vars.push(OsString::from("WINIT_UNIX_BACKEND=wayland"));
            env_vars.push(OsString::from("GDK_BACKEND=wayland"));
            env_vars.push(OsString::from("WLR_NO_HARDWARE_CURSORS=1"));

            println!("{:?}", env_vars);

            // let argv0 = run_args[0].clone();
            // let run_args_1 = run_args.clone();
            // println!("{:?}", argv0);

            let argv = if !run_args.is_empty() {
                let argv0 = run_args[0].clone();
                let bin_path = match which(argv0.clone()) {
                    Ok(path) => {
                        path
                    }
                    Err(e) => {
                        eprintln!("Couldn't find executable '{:?}': {}", argv0, e);
                        PathBuf::from(argv0)
                    }
                };
                let mut run_args = run_args.clone();
                run_args.remove(0);
                run_args.insert(0, bin_path.as_os_str().into());
                run_args
            } else {
                let bin_path = match which("alacritty") {
                    Ok(path) => {
                        path
                    }
                    Err(e) => {
                        eprintln!("Couldn't find alacritty executable: {}", e);
                        PathBuf::from("alacritty")
                    }
                };
                let bin_path = bin_path.as_os_str().into();
                vec![
                    bin_path,
                    OsString::from("--option"), OsString::from("window.decorations=\"None\""),
                    OsString::from("--option"), OsString::from("window.startup_mode=\"Maximized\""),
                    OsString::from("--option"), OsString::from("window.padding.x=0"),
                    OsString::from("--option"), OsString::from("window.padding.y=0"),
                    OsString::from("--option"), OsString::from("window.opacity=0.5"),
                ]
            };

            match compositor.spawn_async(
                None,
                argv,
                // vec![
                //     // OsString::from("/usr/bin/gnome-calculator"),
                //     OsString::from("/home/aschaeffer/.cargo/bin/alacritty"),
                //     OsString::from("--option"),
                //     OsString::from("window.decorations=\"None\""),
                //     OsString::from("--option"),
                //     // OsString::from("window.startup_mode=\"Fullscreen\""),
                //     OsString::from("window.startup_mode=\"Maximized\""),
                //     OsString::from("--option"),
                //     OsString::from("window.position.x=0"),
                //     OsString::from("--option"),
                //     OsString::from("window.position.y=0"),
                //     OsString::from("--option"),
                //     OsString::from("window.padding.x=0"),
                //     OsString::from("--option"),
                //     OsString::from("window.padding.y=0"),
                //     OsString::from("--option"),
                //     OsString::from("window.opacity=0.5"),
                //     OsString::from("--option"),
                //     OsString::from("font.size=14"),
                // ],
                env_vars,
                // vec![
                //     OsString::from("GDK_BACKEND=wayland"),
                //     OsString::from(format!("WAYLAND_DISPLAY={socket_name}")),
                //     OsString::from("WINIT_UNIX_BACKEND=wayland"),
                //     OsString::from("XKB_DEFAULT_LAYOUT=de"),
                // ],
                SpawnFlags::DEFAULT
            ) {
                Ok(pid) => println!("pid {}", pid.0),
                Err(e) => eprintln!("error: {e}"),
            }
            glib::ControlFlow::Break
        });

        let rotation_state = std::rc::Rc::new(std::cell::Cell::new(rotation));
        glib::timeout_add_local(Duration::from_millis(16), move || {
            let current_angle = rotation_state.get();
            let next_angle = (current_angle + 0.1) % 360.0;
            rotation_state.set(next_angle);
            rotated_container.set_rotation(SmearorRotation::Deg(next_angle));
            glib::ControlFlow::Continue
        });

        window.present();
    });

    app.run_with_args::<&str>(&[]);
    // app.run();
}