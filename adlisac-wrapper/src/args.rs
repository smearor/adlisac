use clap::Parser;
use std::ffi::OsString;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 0.0)]
    pub(crate) rotation: f32,

    #[arg(short = 'W', long, default_value_t = 1200)]
    pub(crate) width: i32,

    #[arg(short = 'H', long, default_value_t = 1200)]
    pub(crate) height: i32,

    #[arg(short = 'd', long, action)]
    pub(crate) decorated: bool,

    /// Runs the command in a shell
    #[arg(short = 's', long, action)]
    pub(crate) shell: bool,

    /// Runs the command in a shell
    #[arg(short = 'S', long, default_value = "/tmp/io.smearor.casilda.simple.sock")]
    pub(crate) socket: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) run_args: Vec<OsString>,
}