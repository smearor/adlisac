use crate::error::AdlisacError;
use crate::error::AdlisacResult;
use smithay::desktop::Space;
use smithay::desktop::Window;
use smithay::input::Seat;
use smithay::input::SeatState;
use smithay::reexports::wayland_server::Display;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::ListeningSocket;
use smithay::utils::Logical;
use smithay::utils::Point;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
use std::ffi::OsString;

/// Global state of the Adlisac compositor.
pub struct AdlisacState {
    /// Handle to the Wayland display.
    pub display_handle: DisplayHandle,
    /// The desktop space where windows are mapped.
    pub space: Space<Window>,
    /// State for the Wayland compositor protocol.
    pub compositor_state: CompositorState,
    /// State for the XDG shell protocol (standard windows).
    pub xdg_shell_state: XdgShellState,
    /// State for the Layer Shell protocol (overlays/panels).
    pub layer_shell_state: WlrLayerShellState,
    /// State for shared memory buffer handling.
    pub shm_state: ShmState,
    /// State for input seats.
    pub seat_state: SeatState<Self>,
    /// List of active input seats.
    pub seats: Vec<Seat<Self>>,
    /// The name of the Wayland socket.
    pub socket_name: OsString,
    /// The listening socket for new client connections.
    pub listening_socket: Option<ListeningSocket>,
    /// Callback triggered when a client commits a new frame.
    pub on_commit_callback: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

impl AdlisacState {
    /// Attempts to initialize a new AdlisacState.
    ///
    /// This method sets up the Wayland protocols, input seats, and the listening socket.
    pub fn try_new(wayland_display: &mut Display<Self>, on_commit_callback: Option<Box<dyn Fn() + Send + Sync + 'static>>) -> AdlisacResult<Self> {
        let display_handle = wayland_display.handle();

        // Initialize Wayland protocol states.
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, Vec::new());

        // Initialize the default input seat.
        let mut seat_state = SeatState::new();
        let mut input_seats = Vec::new();

        let mut default_seat = seat_state.new_wl_seat(&display_handle, "seat-0");
        default_seat.add_pointer();

        // Add a keyboard to the seat with default repeat rates.
        default_seat
            .add_keyboard(Default::default(), 200, 25)
            .map_err(|error| AdlisacError::InternalSmithay(format!("Failed to add keyboard to seat: {}", error)))?;

        default_seat.add_touch();
        input_seats.push(default_seat);

        let desktop_space = Space::default();

        // Bind a new Wayland listening socket automatically.
        let socket_listener = ListeningSocket::bind_auto("wayland-", 0..100).map_err(AdlisacError::SocketCreationFailed)?;

        // Retrieve the socket name for client connections.
        let wayland_socket_name = socket_listener
            .socket_name()
            .ok_or_else(|| AdlisacError::ProtocolInitError("Failed to retrieve the Wayland socket name".to_string()))?
            .to_os_string();

        Ok(Self {
            display_handle,
            space: desktop_space,
            compositor_state,
            xdg_shell_state,
            layer_shell_state,
            shm_state,
            seat_state,
            seats: input_seats,
            socket_name: wayland_socket_name,
            listening_socket: Some(socket_listener),
            on_commit_callback,
        })
    }

    /// Returns a list of all mapped windows and their logical locations.
    pub fn get_windows(&self) -> Vec<(Window, Point<i32, Logical>)> {
        self.space
            .elements()
            .map(|window| {
                let window_location = self.space.element_location(window).unwrap_or_default();
                (window.clone(), window_location)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::AdlisacState;
    use crate::error::AdlisacResult;
    use smithay::reexports::wayland_server::Display;

    #[test]
    fn test_adlisac_state_initialization() -> AdlisacResult<()> {
        let mut wayland_display = Display::<AdlisacState>::new().map_err(|_| crate::error::AdlisacError::DisplayInitFailed)?;
        let adlisac_state = AdlisacState::try_new(&mut wayland_display, None)?;
        assert!(!adlisac_state.socket_name.is_empty());
        assert!(!adlisac_state.seats.is_empty());
        assert!(adlisac_state.listening_socket.is_some());
        Ok(())
    }
}
