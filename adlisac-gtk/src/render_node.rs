use gtk4::gdk;
use gtk4::glib::Bytes;
use gtk4::prelude::Cast;
use smithay::desktop::Window;
use smithay::wayland::compositor::{BufferAssignment, SurfaceAttributes, SurfaceData};
use smithay::wayland::shm::with_buffer_contents;
use smithay::reexports::wayland_server::Resource;

/// Attempts to render a Smithay window's current buffer into a GDK texture.
///
/// This function currently supports shared memory (SHM) buffers and converts them
/// into GDK memory textures suitable for GTK4 rendering.
pub fn render_window_to_texture(
    target_window: &Window,
) -> Option<gdk::Texture> {
    let wayland_surface = target_window.toplevel()?.wl_surface();

    // Access the surface data to find the currently attached buffer.
    let surface_data = wayland_surface.data::<SurfaceData>().unwrap();
    
    // Get the current buffer from the surface
    // Use the correct pattern from Smithay Anvil example
    let assigned_buffer = surface_data
        .cached_state
        .get::<SurfaceAttributes>()
        .current()
        .buffer
        .as_ref()
        .map(|assignment| match assignment {
            BufferAssignment::NewBuffer(buffer) => BufferAssignment::NewBuffer(buffer.clone()),
            BufferAssignment::Removed => BufferAssignment::Removed,
        })
        .unwrap_or(BufferAssignment::Removed);

    if let BufferAssignment::NewBuffer(buffer_handle) = assigned_buffer {
        // Attempt to read the buffer contents as shared memory.
        with_buffer_contents(&buffer_handle, |memory_pointer, data_length, buffer_metadata| {
            let buffer_width = buffer_metadata.width;
            let buffer_height = buffer_metadata.height;
            let buffer_stride = buffer_metadata.stride;

            // Create a safe slice from raw memory and copy it into a GLib Bytes object.
            // Safety: with_buffer_contents ensures that memory pointer is valid during this closure.
            let pixel_slice = unsafe { std::slice::from_raw_parts(memory_pointer, data_length) };
            let pixel_bytes = Bytes::from_owned(pixel_slice);

            // Wayland SHM (ARGB8888) is typically represented as BGRA in GDK (Little Endian).
            let gdk_memory_format = gdk::MemoryFormat::B8g8r8a8;

            // Construct a GDK texture from the copied pixel data.
            let gdk_texture = gdk::MemoryTexture::new(
                buffer_width,
                buffer_height,
                gdk_memory_format,
                &pixel_bytes,
                buffer_stride as usize,
            );

            Some(gdk_texture.upcast_ref::<gdk::Texture>().clone())
        }).ok()?
    } else {
        None
    }
}
