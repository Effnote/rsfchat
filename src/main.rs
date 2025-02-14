use app::App;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::error::TryRecvError;

mod app;
mod io;
mod widgets;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
    let terminal = ratatui::init();
    let run_result = run(terminal);
    ratatui::restore();
    if let Err(error) = run_result {
        eprintln!("{:?}", error);
    }
}

fn run(mut terminal: ratatui::DefaultTerminal) -> miette::Result<()> {
    let (connection, mut event_stream) = io::start()?;
    let mut app = App::new(connection);
    while !app.should_quit {
        app.draw(&mut terminal).unwrap();
        let timeout = Instant::now() + Duration::from_millis(500);
        let Some(event) = event_stream.blocking_recv() else {
            break;
        };
        app.event(event).unwrap();
        // Keep processing until there are no more events in the queue, or the timeout has been hit.
        // The timeout makes sure that we don't get stuck for too long without redrawing.
        loop {
            match event_stream.try_recv() {
                Ok(event) => app.event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => miette::bail!("Event stream disconnected."),
            }?;
            if Instant::now() > timeout {
                break;
            }
        }
    }
    Ok(())
}
