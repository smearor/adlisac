use crate::state::base::AdlisacState;
use smithay::delegate_xdg_shell;
use smithay::desktop::Window;
use smithay::utils::Logical;
use smithay::utils::Point;
use smithay::utils::Serial;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::shell::xdg::PopupSurface;
use smithay::wayland::shell::xdg::PositionerState;
use smithay::wayland::shell::xdg::ToplevelSurface;
use smithay::wayland::shell::xdg::XdgShellHandler;
use smithay::wayland::shell::xdg::XdgShellState;
use tracing::debug;
use wayland_server::protocol::wl_seat::WlSeat;

impl XdgShellHandler for AdlisacState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, toplevel_surface: ToplevelSurface) {
        let wayland_window = Window::new_wayland_window(toplevel_surface.clone());
        let window_position: Point<i32, Logical> = (0, 0).into();

        // Map the window into the compositor space.
        self.space.map_element(wayland_window, window_position, true);

        // Attempt to set keyboard focus to the new window.
        if let Some(seat_instance) = self.seats.first() {
            let keyboard_handle = seat_instance.get_keyboard();
            let surface_to_focus = toplevel_surface.wl_surface();
            let serial_number = SERIAL_COUNTER.next_serial();

            if let Some(keyboard) = keyboard_handle {
                // Set the keyboard focus to the surface of the new toplevel.
                keyboard.set_focus(self, Some(surface_to_focus.clone()), serial_number);
                debug!("Keyboard focus set to the new toplevel window.");
            }

            // Pointer and touch focus are typically updated during motion events,
            // but we ensure the seat knows about the new surface.
        }
    }

    fn new_popup(&mut self, popup_surface: PopupSurface, _positioner_state: PositionerState) {
        // Popups are often used for menus (e.g., in Firefox).
        // In this implementation, we log the request for now.
        debug!(?popup_surface, "New popup surface requested.");
    }

    fn grab(&mut self, _popup_surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        // A grab occurs when a popup wants to capture all input exclusively.
        debug!("Popup grab requested.");
    }

    fn reposition_request(&mut self, _popup_surface: PopupSurface, _positioner_state: PositionerState, _token: u32) {
        // Called when a popup wants to change its position (e.g., to stay on screen).
        debug!("Popup repositioning requested.");
    }

    fn toplevel_destroyed(&mut self, _toplevel_surface: ToplevelSurface) {
        // Clean up resources when a toplevel is destroyed.
        debug!("Toplevel window destroyed.");
    }
}

delegate_xdg_shell!(AdlisacState);
