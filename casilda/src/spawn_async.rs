use std::ptr;
use glib::translate::{from_glib, from_glib_full, IntoGlib, ToGlibPtr};
use crate::Compositor;

impl Compositor {
    pub fn spawn_async(
        &self,
        working_directory: Option<std::path::PathBuf>,
        argv: Vec<std::ffi::OsString>,
        envp: Vec<std::ffi::OsString>,
        flags: glib::SpawnFlags,
    ) -> Result<glib::Pid, glib::Error> {
        unsafe {
            let mut pid = std::mem::MaybeUninit::uninit();
            let mut error = ptr::null_mut();
            let is_success = ffi::casilda_compositor_spawn_async(
                self.to_glib_none().0,
                working_directory.to_glib_none().0,
                argv.to_glib_none().0,
                envp.to_glib_none().0,
                flags.into_glib(),
                None,
                ptr::null_mut(),
                pid.as_mut_ptr(),
                &mut error,
            );

            if is_success == glib::ffi::GTRUE {
                Ok(from_glib(pid.assume_init()))
            } else {
                Err(from_glib_full(error))
            }
        }
    }
}