use gtk4::gdk;
use gtk4::glib::Bytes;
use smithay::desktop::Window;
use smithay::wayland::compositor::BufferAssignment;
use smithay::wayland::compositor::SurfaceAttributes;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shm::with_buffer_contents;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// Attempts to render a Smithay window's current buffer into a GDK texture.
///
/// This function currently supports shared memory (SHM) buffers and converts them
/// into GDK memory textures suitable for GTK4 rendering.
pub fn render_window_to_texture(
    target_window: &Window,
) -> Option<gdk::Texture> {
    let wayland_surface = target_window.toplevel().wl_surface();

    // Access the surface state to find the currently attached buffer.
    with_states(wayland_surface, |surface_states| {
        let surface_attributes = surface_states.cached_state.current::<SurfaceAttributes>();

        // Identify the buffer assigned to the surface.
        let assigned_buffer = match &surface_attributes.buffer {
            Some(BufferAssignment::NewBuffer(buffer_handle)) => Some(buffer_handle),
            Some(BufferAssignment::Attached(buffer_handle)) => Some(buffer_handle),
            _ => None,
        }?;

        // Attempt to read the buffer contents as shared memory.
        with_buffer_contents(assigned_buffer, |memory_pointer, data_length, buffer_metadata| {
            let buffer_width = buffer_metadata.width;
            let buffer_height = buffer_metadata.height;
            let buffer_stride = buffer_metadata.stride;

            // Create a safe slice from the raw memory and copy it into a GLib Bytes object.
            // Safety: with_buffer_contents ensures the memory pointer is valid during this closure.
            let pixel_slice = unsafe { std::slice::from_raw_parts(memory_pointer, data_length) };
            let pixel_bytes = Bytes::copy(pixel_slice);

            // Wayland SHM (ARGB8888) is typically represented as BGRA in GDK (Little Endian).
            let gdk_memory_format = gdk::MemoryFormat::B8g8r8a8;

            // Construct the GDK texture from the copied pixel data.
            let gdk_texture = gdk::MemoryTexture::new(
                buffer_width,
                buffer_height,
                gdk_memory_format,
                &pixel_bytes,
                buffer_stride as usize,
            );

            Some(gdk_texture.upcast::<gdk::Texture>())
        }).ok().flatten()
    })
}
