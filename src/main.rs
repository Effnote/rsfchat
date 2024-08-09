use miette::IntoDiagnostic;
use tui::Tui;

mod app;
mod connection;
mod tui;
mod widgets;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        tui::quit().unwrap();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
    if let Err(error) = run() {
        tui::quit().unwrap();
        println!("{:?}", error);
    }
}

fn run() -> miette::Result<()> {
    let tui = Tui::new().into_diagnostic()?;
    let runtime = tokio::runtime::Runtime::new().into_diagnostic()?;
    runtime.block_on(app::run(tui))?;
    Ok(())
}
