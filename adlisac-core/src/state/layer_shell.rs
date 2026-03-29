use crate::state::base::AdlisacState;
use smithay::delegate_layer_shell;
use smithay::wayland::shell::wlr_layer::Layer;
use smithay::wayland::shell::wlr_layer::LayerSurface;
use smithay::wayland::shell::wlr_layer::WlrLayerShellHandler;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use tracing::info;

impl WlrLayerShellHandler for AdlisacState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        layer_surface: LayerSurface,
        _output_target: Option<WlOutput>,
        _surface_layer: Layer,
        _namespace_identifier: String,
    ) {
        // Management of layer elements such as taskbars or background images.
        info!(?layer_surface, "New layer surface detected.");
    }
}

delegate_layer_shell!(AdlisacState);
