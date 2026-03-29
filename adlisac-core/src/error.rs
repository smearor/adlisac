use miette::Diagnostic;
use thiserror::Error;
use smithay::reexports::wayland_server::BindError;
use std::io::Error as IoError;

#[derive(Error, Debug, Diagnostic)]
pub enum AdlisacError {
    #[error("Failed to initialize wayland display handle")]
    #[diagnostic(code(adlisac::display_init_failed))]
    DisplayInitFailed,

    #[error("Failed to create wayland socket: {0}")]
    #[diagnostic(code(adlisac::socket_creation_failed), help("Please check if XDG_RUNTIME_DIR has been set and is writeable."))]
    SocketCreationFailed(#[from] BindError),

    #[error("Failed to initialize wayland protocol: {0}")]
    #[diagnostic(code(adlisac::protocol_init_failed))]
    ProtocolInitError(String),

    #[error("Internal smithay error: {0}")]
    #[diagnostic(code(adlisac::internal_smithay_error))]
    InternalSmithay(String),
}

#[derive(Error, Debug, Diagnostic)]
pub enum BridgeError {
    #[error("Failed to dispatch wayland event")]
    DispatchFailed(#[source] IoError),

    #[error("Failed to flush wayland display")]
    FlushFailed(#[source] IoError),
}

pub type AdlisacResult<T> = Result<T, AdlisacError>;
