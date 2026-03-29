use adlisac_core::state::base::AdlisacState;
use crate::source::WaylandSource;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use glib::Object;
use smithay::reexports::wayland_server::Display;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

// Smithay Input Imports
use smithay::backend::input::{ButtonState, KeyState};
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::utils::SERIAL_COUNTER;

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

            // Create a weak reference to the widget to avoid reference cycles in the callback.
            let widget_weak_reference = compositor_widget.downgrade();

            // This callback is called when a client commits a new frame.
            // It notifies the GTK widget to redraw.
            let on_commit_callback = Box::new(move || {
                if let Some(widget) = widget_weak_reference.upgrade() {
                    // Queue a redraw on the GTK main thread.
                    widget.queue_draw();
                }
            });

            // Initialize the Wayland display.
            let mut display = Display::<AdlisacState>::new().expect("Failed to create Wayland display");

            // Initialize the Smithay compositor state.
            let state = AdlisacState::try_new(&mut display, Some(on_commit_callback))
                .expect("Failed to initialize AdlisacState");

            let display_rc = Rc::new(RefCell::new(display));
            let state_rc = Rc::new(RefCell::new(state));

            // Store the display and state in the widget's private data.
            *self.display.borrow_mut() = Some(display_rc.clone());
            *self.state.borrow_mut() = Some(state_rc.clone());

            // Integrate the Wayland event loop into the GLib main context.
            let wayland_source = WaylandSource::new(display_rc.clone(), state_rc.clone());
            wayland_source.attach(&glib::MainContext::default());

            tracing::info!("AdlisacCompositor widget constructed and Wayland display initialized.");

            // --- Input Handling ---

            // Pointer Motion
            let motion_controller = gtk4::EventControllerMotion::new();
            compositor_widget.add_controller(&motion_controller);
            motion_controller.connect_motion(glib::clone!(@strong state_rc, @strong display_rc => move |_controller, x, y| {
                let mut state_borrow = state_rc.borrow_mut();
                let display_handle = display_rc.borrow().handle();

                if let Some(seat) = state_borrow.seats.first().cloned() {
                    if let Some(pointer_handle) = seat.get_pointer() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Instant::now();

                        // Determine which surface is under the pointer.
                        let (under, _surface_location) = state_borrow.space
                            .element_under((x, y))
                            .map(|(window, location)| (Some(window.toplevel().wl_surface().clone()), location))
                            .unwrap_or((None, (0, 0).into()));

                        pointer_handle.motion(
                            &mut *state_borrow,
                            &display_handle,
                            &under,
                            MotionEvent {
                                location: (x, y).into(),
                                serial,
                                time: time.as_nanos() as u32,
                            },
                        );
                    }
                }
            }));

            // Pointer Buttons (Click)
            let gesture_click = gtk4::GestureClick::new();
            gesture_click.set_button(0); // Capture all buttons
            compositor_widget.add_controller(&gesture_click);

            gesture_click.connect_pressed(glib::clone!(@strong state_rc, @strong display_rc => move |gesture, _n_press, _x, _y| {
                let mut state_borrow = state_rc.borrow_mut();
                let display_handle = display_rc.borrow().handle();

                if let Some(seat) = state_borrow.seats.first().cloned() {
                    if let Some(pointer_handle) = seat.get_pointer() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Instant::now();
                        let gdk_button = gesture.current_button();

                        let wayland_button = match gdk_button {
                            1 => 0x110, // BTN_LEFT
                            2 => 0x112, // BTN_MIDDLE
                            3 => 0x111, // BTN_RIGHT
                            _ => 0x110,
                        };

                        pointer_handle.button(
                            &mut *state_borrow,
                            &display_handle,
                            ButtonEvent {
                                button: wayland_button,
                                state: ButtonState::Pressed,
                                serial,
                                time: time.as_nanos() as u32,
                            },
                        );
                    }
                }
            }));

            gesture_click.connect_released(glib::clone!(@strong state_rc, @strong display_rc => move |gesture, _n_press, _x, _y| {
                let mut state_borrow = state_rc.borrow_mut();
                let display_handle = display_rc.borrow().handle();

                if let Some(seat) = state_borrow.seats.first().cloned() {
                    if let Some(pointer_handle) = seat.get_pointer() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Instant::now();
                        let gdk_button = gesture.current_button();

                        let wayland_button = match gdk_button {
                            1 => 0x110, // BTN_LEFT
                            2 => 0x112, // BTN_MIDDLE
                            3 => 0x111, // BTN_RIGHT
                            _ => 0x110,
                        };

                        pointer_handle.button(
                            &mut *state_borrow,
                            &display_handle,
                            ButtonEvent {
                                button: wayland_button,
                                state: ButtonState::Released,
                                serial,
                                time: time.as_nanos() as u32,
                            },
                        );
                    }
                }
            }));

            // Keyboard Input
            let key_controller = gtk4::EventControllerKey::new();
            compositor_widget.add_controller(&key_controller);

            key_controller.connect_key_pressed(glib::clone!(@strong state_rc, @strong display_rc => move |_controller, _keyval, keycode, _state| {
                let mut state_borrow = state_rc.borrow_mut();
                let display_handle = display_rc.borrow().handle();

                if let Some(seat) = state_borrow.seats.first().cloned() {
                    if let Some(keyboard_handle) = seat.get_keyboard() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Instant::now();

                        keyboard_handle.input(
                            &mut *state_borrow,
                            &display_handle,
                            keycode,
                            KeyState::Pressed,
                            serial,
                            time.as_nanos() as u32,
                            "evdev",
                        );
                    }
                }
                glib::Propagation::Proceed
            }));

            key_controller.connect_key_released(glib::clone!(@strong state_rc, @strong display_rc => move |_controller, _keyval, keycode, _state| {
                let mut state_borrow = state_rc.borrow_mut();
                let display_handle = display_rc.borrow().handle();

                if let Some(seat) = state_borrow.seats.first().cloned() {
                    if let Some(keyboard_handle) = seat.get_keyboard() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = Instant::now();

                        keyboard_handle.input(
                            &mut *state_borrow,
                            &display_handle,
                            keycode,
                            KeyState::Released,
                            serial,
                            time.as_nanos() as u32,
                            "evdev",
                        );
                    }
                }
                glib::Propagation::Proceed
            }));
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
                        snapshot.append_texture(
                            &texture,
                            &gtk4::graphene::Rect::new(
                                0.0,
                                0.0,
                                texture.width() as f32,
                                texture.height() as f32,
                            ),
                        );
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
        @extends gtk4::Widget;
}

impl AdlisacCompositor {
    /// Creates a new AdlisacCompositor widget.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Returns the name of the Wayland socket this compositor is listening on.
    pub fn socket_name(&self) -> String {
        self.imp().state.borrow().as_ref()
            .map(|state_rc| state_rc.borrow().socket_name.clone())
            .unwrap_or_default()
    }
}
