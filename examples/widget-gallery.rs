use qtx::mem::Strong;
use qtx::widgets::Tab;
use qtx::windows::MainWindow;
use qtx::{App, Runtime};
use std::ops::Deref;
use std::pin::Pin;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut rt = unsafe { Runtime::new() };

    rt.set_style("Fusion");

    match rt.run(std::env::args(), run).unwrap() {
        Some(v) => v,              // `run` run to completion.
        None => ExitCode::FAILURE, // Qt's event loop exit before `run` finished.
    }
}

async fn run(app: Pin<Strong<App>>) -> ExitCode {
    // Construct main window.
    let main = MainWindow::new(&app);

    main.set_title("Qtx");

    // Build main tab.
    let tab = Tab::new(main.deref());

    main.set_central_widget(tab.deref());

    // Run.
    main.show();
    main.await;

    ExitCode::SUCCESS
}
