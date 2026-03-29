use smithay::wayland::compositor::CompositorClientState;
use smithay::reexports::wayland_server::backend::ClientData;
use smithay::reexports::wayland_server::backend::ClientId;
use smithay::reexports::wayland_server::backend::DisconnectReason;

/// Data associated with a Wayland client that connects to the Adlisac compositor.
/// One instance of this state exists per client.
#[derive(Default)]
pub struct ClientState {
    /// The compositor state associated with this specific client.
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(
        &self,
        _client_identifier: ClientId,
    ) {
        // Initialization logic for a new client connection.
    }

    fn disconnected(
        &self,
        _client_identifier: ClientId,
        _disconnect_reason: DisconnectReason,
    ) {
        // Cleanup logic for a client that has disconnected.
    }
}
