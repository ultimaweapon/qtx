use std::pin::Pin;
use std::process::ExitCode;

use qtx::mem::Strong;
use qtx::windows::MainWindow;
use qtx::{App, Runtime};

fn main() -> ExitCode {
    let mut rt = unsafe { Runtime::new() };

    rt.set_style("Fusion");

    rt.run(std::env::args(), run).unwrap()
}

async fn run(app: Pin<Strong<App>>) -> ExitCode {
    let main = MainWindow::new(&app);

    main.show();
    main.await;

    ExitCode::SUCCESS
}
