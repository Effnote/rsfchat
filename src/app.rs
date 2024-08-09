use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use miette::IntoDiagnostic;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Wrap},
};
use tokio_stream::StreamExt;

use crate::tui::Tui;
use crate::widgets::TextArea;

pub struct App {}

impl App {
    fn new() -> Self {
        App {}
    }
}

pub async fn run(mut tui: Tui) -> miette::Result<()> {
    let app = App::new();
    let mut event_stream = crossterm::event::EventStream::new();
    let mut textarea = TextArea::new("");
    tui.terminal.clear().into_diagnostic()?;
    while let Some(ref event) = event_stream.try_next().await.into_diagnostic()? {
        textarea.event(event);
        tui.terminal
            .draw(|frame| {
                let layout: [Rect; 4] = Layout::vertical([
                    Constraint::Percentage(10),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(40),
                ])
                .areas(frame.size());
                frame.render_widget(
                    Paragraph::new(format!("{:?}", event)).wrap(Wrap { trim: true }),
                    layout[0],
                );
                frame.render_widget(
                    Paragraph::new(format!(
                        "{:?}",
                        textarea.wrapped_text(layout[3].width as usize)
                    ))
                    .wrap(Wrap { trim: false }),
                    layout[1],
                );
                frame.render_widget(
                    Paragraph::new(format!("{:?}", textarea.text(layout[3])))
                        .wrap(Wrap { trim: true }),
                    layout[2],
                );
                frame.render_widget(&textarea, layout[3]);
            })
            .into_diagnostic()?;
        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: _,
            })
        ) {
            break;
        }
    }
    Ok(())
}
