use ratatui::{
    backend::CrosstermBackend,
    crossterm::{self, event::KeyboardEnhancementFlags, execute, terminal},
    Terminal,
};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self, std::io::Error> {
        terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            terminal::EnterAlternateScreen,
            crossterm::event::PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all())
        )?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Tui { terminal })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = quit();
    }
}

pub fn quit() -> Result<(), std::io::Error> {
    let result = terminal::disable_raw_mode();
    // Even if disable_raw_mode fails, we still want to try to continue
    execute!(
        std::io::stdout(),
        terminal::LeaveAlternateScreen,
        crossterm::event::PopKeyboardEnhancementFlags
    )
    .or(result)
}
