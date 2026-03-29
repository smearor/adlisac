use std::process::{Command, Child, Stdio};
use crate::error::AdlisacResult;
use crate::error::AdlisacError;

/// Spawns a new Wayland client process connected to the specified Wayland socket.
///
/// This function sets the `WAYLAND_DISPLAY` environment variable so that the
/// child process knows which compositor to connect to.
pub fn spawn_wayland_client(
    executable_name: String,
    arguments: Vec<String>,
    wayland_socket_name: String,
) -> AdlisacResult<Child> {
    // Construct the command to execute.
    let mut command_to_execute = Command::new(executable_name);

    // Set the target Wayland display for the client application.
    command_to_execute.env("WAYLAND_DISPLAY", wayland_socket_name);

    // Add all command-line arguments to the process.
    command_to_execute.args(arguments);

    // Inherit the parent's standard input/output/error streams.
    command_to_execute.stdin(Stdio::inherit());
    command_to_execute.stdout(Stdio::inherit());
    command_to_execute.stderr(Stdio::inherit());

    // Spawn the child process and handle potential execution errors.
    let child_process = command_to_execute.spawn()
        .map_err(|io_error| AdlisacError::ProtocolInitError(format!("Failed to spawn child process: {}", io_error)))?;

    tracing::info!("Successfully spawned client process with PID: {}", child_process.id());

    Ok(child_process)
}
