use std::process::ExitCode;

use qtx::ffi::Strong;
use qtx::{App, Runtime};

fn main() -> ExitCode {
    let mut rt = unsafe { Runtime::new() };

    rt.set_style("Fusion");

    rt.run(std::env::args(), run).unwrap()
}

async fn run(_: Strong<App>) -> ExitCode {
    ExitCode::SUCCESS
}
