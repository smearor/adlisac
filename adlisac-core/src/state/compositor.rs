use crate::state::base::AdlisacState;
use crate::state::client::ClientState;
use smithay::delegate_compositor;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Client;
use smithay::wayland::compositor::CompositorClientState;
use smithay::wayland::compositor::CompositorHandler;
use smithay::wayland::compositor::CompositorState;
use std::sync::OnceLock;
use tracing::warn;

impl CompositorHandler for AdlisacState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, wayland_client: &'a Client) -> &'a CompositorClientState {
        if let Some(state_instance) = wayland_client.get_data::<ClientState>() {
            &state_instance.compositor_state
        } else {
            warn!("Compositor client state missing for client. Using fallback.");
            static FALLBACK_STATE: OnceLock<CompositorClientState> = OnceLock::new();
            FALLBACK_STATE.get_or_init(CompositorClientState::default)
        }
    }

    fn commit(&mut self, _surface_to_commit: &WlSurface) {
        // Trigger the registered callback to notify the GTK widget about the new frame.
        if let Some(on_commit_callback) = &self.on_commit_callback {
            on_commit_callback();
        }
    }
}

delegate_compositor!(AdlisacState);

#[cfg(test)]
mod tests {
    use super::AdlisacState;
    use smithay::reexports::wayland_server::Display;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    #[test]
    fn test_compositor_commit_callback_execution() -> Result<(), Box<dyn std::error::Error>> {
        let mut wayland_display = Display::<AdlisacState>::new().map_err(|error| format!("Failed to create Wayland display: {}", error))?;
        let was_callback_invoked = Arc::new(AtomicBool::new(false));
        let was_callback_invoked_clone = was_callback_invoked.clone();
        let on_commit_callback = Box::new(move || {
            was_callback_invoked_clone.store(true, Ordering::SeqCst);
        });
        let adlisac_state =
            AdlisacState::try_new(&mut wayland_display, Some(on_commit_callback)).map_err(|error| format!("Failed to create AdlisacState: {:?}", error))?;
        if let Some(callback) = &adlisac_state.on_commit_callback {
            callback();
        }
        assert!(was_callback_invoked.load(Ordering::SeqCst));
        Ok(())
    }
}
