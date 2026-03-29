use crate::state::base::AdlisacState;
use smithay::delegate_seat;
use smithay::input::Seat;
use smithay::input::SeatHandler;
use smithay::input::SeatState;
use smithay::input::pointer::CursorImageStatus;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use tracing::trace;

impl SeatHandler for AdlisacState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(
        &mut self,
        _seat_instance: &Seat<Self>,
        _focused_surface: Option<&WlSurface>,
    ) {
        // Triggered when focus transitions between surfaces.
        trace!("Input focus changed.");
    }

    fn cursor_image(
        &mut self,
        _seat_instance: &Seat<Self>,
        _cursor_image_status: CursorImageStatus,
    ) {
        // Sets the appearance of the mouse cursor (e.g., arrow, hand).
        trace!("Cursor image update requested.");
    }
}

delegate_seat!(AdlisacState);
