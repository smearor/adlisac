use crate::state::base::AdlisacState;
use smithay::delegate_shm;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::shm::ShmHandler;
use smithay::wayland::shm::ShmState;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;

impl ShmHandler for AdlisacState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for AdlisacState {
    fn buffer_destroyed(
        &mut self,
        _destroyed_buffer: &WlBuffer,
    ) {
        // Resources can be cleaned up here when a buffer is destroyed.
    }
}

delegate_shm!(AdlisacState);
