use std::pin::Pin;
use std::process::ExitCode;

use qtx::mem::Strong;
use qtx::windows::MainWindow;
use qtx::{App, Runtime};

fn main() -> ExitCode {
    let mut rt = unsafe { Runtime::new() };

    rt.set_style("Fusion");

    match rt.run(std::env::args(), run).unwrap() {
        Some(v) => v,              // `run` run to completion.
        None => ExitCode::FAILURE, // Qt's event loop exit before `run` finished.
    }
}

async fn run(app: Pin<Strong<App>>) -> ExitCode {
    let main = MainWindow::new(&app);

    main.set_title("Qtx");
    main.show();
    main.await;

    ExitCode::SUCCESS
}
