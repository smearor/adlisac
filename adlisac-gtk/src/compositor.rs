use crate::source::WaylandSource;
use adlisac_core::state::base::AdlisacState;
use gdk4::Key;
use gdk4::ModifierType;
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use smithay::backend::input::ButtonState;
use smithay::backend::input::KeyState;
use smithay::input::pointer::ButtonEvent;
use smithay::input::pointer::MotionEvent;
use smithay::reexports::wayland_server::Display;
use smithay::utils::SERIAL_COUNTER;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AdlisacCompositor {
        /// The Wayland display server managed by Smithay.
        pub(crate) display: RefCell<Option<Rc<RefCell<Display<AdlisacState>>>>>,
        /// The compositor state containing the windows and space.
        pub(crate) state: RefCell<Option<Rc<RefCell<AdlisacState>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AdlisacCompositor {
        const NAME: &'static str = "AdlisacCompositor";
        type Type = super::AdlisacCompositor;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for AdlisacCompositor {
        fn constructed(&self) {
            self.parent_constructed();
            let compositor_widget = self.obj();

            // Create a channel for thread-safe commit notifications
            let (commit_sender, commit_receiver) = mpsc::channel::<()>();

            // This callback is called when a client commits a new frame.
            // It sends a signal through the channel to avoid thread safety issues.
            let on_commit_callback = Box::new(move || {
                let _ = commit_sender.send(());
            });

            // Initialize the Wayland display.
            let mut display = Display::<AdlisacState>::new().expect("Failed to create Wayland display");

            // Initialize the Smithay compositor state.
            let state = AdlisacState::try_new(&mut display, Some(on_commit_callback)).expect("Failed to initialize AdlisacState");

            let display_rc = Rc::new(RefCell::new(display));
            let state_rc = Rc::new(RefCell::new(state));

            // Store the display and state in the widget's private data.
            *self.display.borrow_mut() = Some(display_rc.clone());
            *self.state.borrow_mut() = Some(state_rc.clone());

            // Integrate the Wayland event loop into the GLib main context.
            let wayland_source = WaylandSource::new(display_rc.clone(), state_rc.clone());
            wayland_source.attach(&glib::MainContext::default());

            // Set up a GLib source to listen for commit notifications
            let widget_weak = compositor_widget.downgrade();
            glib::spawn_future_local(async move {
                // Use a simple polling approach for now
                loop {
                    if let Ok(()) = commit_receiver.try_recv() {
                        if let Some(widget) = widget_weak.upgrade() {
                            widget.queue_draw();
                        }
                    }
                    // Small delay to prevent busy waiting
                    glib::timeout_future(std::time::Duration::from_millis(1)).await;
                }
            });

            tracing::info!("AdlisacCompositor widget constructed and Wayland display initialized.");

            // --- Input Handling ---

            // Pointer Motion
            let motion_controller = gtk4::EventControllerMotion::new();
            compositor_widget.add_controller(motion_controller.clone());

            // Convert Rc to Weak for better performance
            let state_weak = Rc::downgrade(&state_rc);
            let display_weak = Rc::downgrade(&display_rc);

            motion_controller.connect_motion(move |_controller, x, y| {
                if let (Some(state_rc), Some(display_rc)) = (state_weak.upgrade(), display_weak.upgrade()) {
                    let mut state_borrow = state_rc.borrow_mut();
                    let _display_handle = display_rc.borrow().handle();

                    if let Some(seat) = state_borrow.seats.first().cloned() {
                        if let Some(pointer_handle) = seat.get_pointer() {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u32;

                            // Determine which surface is under the pointer.
                            let under = state_borrow.space.element_under((x, y)).and_then(|(window, location)| {
                                window.toplevel().map(|toplevel| {
                                    let surface = toplevel.wl_surface();
                                    let location_f64 = (location.x as f64, location.y as f64).into();
                                    (surface.clone(), location_f64)
                                })
                            });

                            pointer_handle.motion(
                                &mut *state_borrow,
                                under,
                                &MotionEvent {
                                    location: (x, y).into(),
                                    serial,
                                    time,
                                },
                            );
                        }
                    }
                }
            });

            // Pointer Buttons (Click)
            let gesture_click = gtk4::GestureClick::new();
            gesture_click.set_button(0); // Capture all buttons

            // Use Weak references for button press handler
            let state_weak_press = Rc::downgrade(&state_rc);
            let display_weak_press = Rc::downgrade(&display_rc);

            gesture_click.connect_pressed(move |gesture, _n_press, _x, _y| {
                if let (Some(state_rc), Some(display_rc)) = (state_weak_press.upgrade(), display_weak_press.upgrade()) {
                    let mut state_borrow = state_rc.borrow_mut();
                    let _display_handle = display_rc.borrow().handle();

                    if let Some(seat) = state_borrow.seats.first().cloned() {
                        if let Some(pointer_handle) = seat.get_pointer() {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u32;
                            let gdk_button = gesture.current_button();

                            let wayland_button = match gdk_button {
                                1 => 0x110, // BTN_LEFT
                                2 => 0x112, // BTN_MIDDLE
                                3 => 0x111, // BTN_RIGHT
                                _ => 0x110,
                            };

                            pointer_handle.button(
                                &mut *state_borrow,
                                &ButtonEvent {
                                    button: wayland_button,
                                    state: ButtonState::Pressed,
                                    serial,
                                    time,
                                },
                            );
                        }
                    }
                }
            });

            // Use Weak references for button release handler
            let state_weak_release = Rc::downgrade(&state_rc);
            let display_weak_release = Rc::downgrade(&display_rc);

            gesture_click.connect_released(move |gesture, _n_press, _x, _y| {
                if let (Some(state_rc), Some(display_rc)) = (state_weak_release.upgrade(), display_weak_release.upgrade()) {
                    let mut state_borrow = state_rc.borrow_mut();
                    let _display_handle = display_rc.borrow().handle();

                    if let Some(seat) = state_borrow.seats.first().cloned() {
                        if let Some(pointer_handle) = seat.get_pointer() {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u32;
                            let gdk_button = gesture.current_button();

                            let wayland_button = match gdk_button {
                                1 => 0x110, // BTN_LEFT
                                2 => 0x112, // BTN_MIDDLE
                                3 => 0x111, // BTN_RIGHT
                                _ => 0x110,
                            };

                            pointer_handle.button(
                                &mut *state_borrow,
                                &ButtonEvent {
                                    button: wayland_button,
                                    state: ButtonState::Released,
                                    serial,
                                    time,
                                },
                            );
                        }
                    }
                }
            });

            compositor_widget.add_controller(gesture_click);

            // Keyboard Input
            let key_controller = gtk4::EventControllerKey::new();
            compositor_widget.add_controller(key_controller.clone());

            // Use Weak references for key press handler
            let state_weak_key_press = Rc::downgrade(&state_rc);
            let display_weak_key_press = Rc::downgrade(&display_rc);

            key_controller.connect_key_pressed(move |_controller: &gtk4::EventControllerKey, _keyval: Key, keycode: u32, _state: ModifierType| {
                if let (Some(state_rc), Some(display_rc)) = (state_weak_key_press.upgrade(), display_weak_key_press.upgrade()) {
                    let mut state_borrow = state_rc.borrow_mut();
                    let _display_handle = display_rc.borrow().handle();

                    if let Some(seat) = state_borrow.seats.first().cloned() {
                        if let Some(keyboard_handle) = seat.get_keyboard() {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u32;

                            keyboard_handle.input(
                                &mut *state_borrow,
                                keycode.into(),
                                KeyState::Pressed,
                                serial,
                                time,
                                |_, _, _| smithay::input::keyboard::FilterResult::Intercept(true), // filter function
                            );
                        }
                    }
                }
                glib::Propagation::Proceed
            });

            // Use Weak references for key release handler
            let state_weak_key_release = Rc::downgrade(&state_rc);
            let display_weak_key_release = Rc::downgrade(&display_rc);

            key_controller.connect_key_released(move |_controller: &gtk4::EventControllerKey, _keyval: Key, keycode: u32, _state: ModifierType| {
                if let (Some(state_rc), Some(display_rc)) = (state_weak_key_release.upgrade(), display_weak_key_release.upgrade()) {
                    let mut state_borrow = state_rc.borrow_mut();
                    let _display_handle = display_rc.borrow().handle();

                    if let Some(seat) = state_borrow.seats.first().cloned() {
                        if let Some(keyboard_handle) = seat.get_keyboard() {
                            let serial = SERIAL_COUNTER.next_serial();
                            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u32;

                            keyboard_handle.input(
                                &mut *state_borrow,
                                keycode.into(),
                                KeyState::Released,
                                serial,
                                time,
                                |_, _, _| smithay::input::keyboard::FilterResult::Intercept(true), // filter function
                            );
                        }
                    }
                }
                () // Return unit type for key release handler
            });
        }
    }

    impl WidgetImpl for AdlisacCompositor {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let state_ref = self.state.borrow();
            if let Some(state_rc) = state_ref.as_ref() {
                let state = state_rc.borrow();

                // Render the first window found in the space.
                if let Some((window, _location)) = state.get_windows().first() {
                    if let Some(texture) = crate::render_node::render_window_to_texture(window) {
                        snapshot.append_texture(&texture, &gtk4::graphene::Rect::new(0.0, 0.0, texture.width() as f32, texture.height() as f32));
                    }
                }
            }
        }

        fn measure(&self, orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let state_ref = self.state.borrow();
            if let Some(state_rc) = state_ref.as_ref() {
                let state = state_rc.borrow();

                if let Some((window, _location)) = state.get_windows().first() {
                    let geometry = window.geometry();
                    let size = if orientation == gtk4::Orientation::Horizontal {
                        geometry.size.w
                    } else {
                        geometry.size.h
                    };

                    return (size, size, -1, -1);
                }
            }

            (0, 0, -1, -1)
        }
    }
}

glib::wrapper! {
    pub struct AdlisacCompositor(ObjectSubclass<imp::AdlisacCompositor>)
        @extends gtk4::Widget, gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl AdlisacCompositor {
    /// Creates a new AdlisacCompositor widget.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Returns the name of the Wayland socket this compositor is listening on.
    pub fn socket_name(&self) -> String {
        self.imp()
            .state
            .borrow()
            .as_ref()
            .map(|state_rc| state_rc.borrow().socket_name.clone())
            .unwrap_or_else(|| std::ffi::OsString::from("default"))
            .to_string_lossy()
            .into_owned()
    }
}
