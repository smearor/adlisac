#![cfg_attr(docsrs, feature(doc_cfg))]

#[allow(unused_imports)]
#[macro_use]
extern crate gtk4;

#[allow(unused_imports)]
#[macro_use]
extern crate glib;


/// No-op.
macro_rules! skip_assert_initialized {
    () => {};
}

/// Asserts that this is the main thread and either `gdk::init` or `gtk::init` has been called.
macro_rules! assert_initialized_main_thread {
    () => {
        if !::gtk4::is_initialized_main_thread() {
            if ::gtk4::is_initialized() {
                panic!("GTK may only be used from the main thread.");
            } else {
                panic!("GTK has not been initialized. Call `gtk::init` first.");
            }
        }
    };
}


use ffi;
pub use auto::*;
mod auto;
pub mod spawn_async;

pub mod functions {
    // pub use super::auto::functions::*;
}


