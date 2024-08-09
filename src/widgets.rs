use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Text,
    widgets::WidgetRef,
};
use unicode_segmentation::UnicodeSegmentation;

pub struct TextArea {
    pub text: String,
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> TextArea {
        let text = text.into();
        TextArea { text }
    }

    pub fn event(&mut self, event: &crossterm::event::Event) {
        match event {
            &crossterm::event::Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                state: _,
            }) => {
                match code {
                    KeyCode::Backspace => {
                        if modifiers == KeyModifiers::NONE {
                            self.delete_char();
                        } else if modifiers == KeyModifiers::CONTROL {
                            self.delete_word();
                        }
                    }
                    KeyCode::Enter => {
                        self.text.push('\n');
                    }
                    KeyCode::Left => {}
                    KeyCode::Right => {}
                    KeyCode::Up => {}
                    KeyCode::Down => {}
                    KeyCode::Home => {}
                    KeyCode::End => {}
                    KeyCode::PageUp => {}
                    KeyCode::PageDown => {}
                    KeyCode::Tab => {}
                    KeyCode::BackTab => {}
                    KeyCode::Delete => {}
                    KeyCode::Insert => {}
                    KeyCode::Char(c) => {
                        if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT {
                            self.text.push(c);
                        }
                    }
                    _ => {}
                };
            }
            crossterm::event::Event::Mouse(_) => {}
            crossterm::event::Event::Paste(_) => {}
            crossterm::event::Event::Resize(_, _) => {}
            _ => {}
        }
    }

    fn delete_char(&mut self) {
        self.text.pop();
    }

    fn delete_word(&mut self) {
        let Some((boundary, _)) = self.text.unicode_word_indices().last() else {
            return;
        };
        self.text.truncate(boundary);
    }

    pub fn wrapped_text(&self, width: usize) -> String {
        let mut text = self.text.clone();
        // Dummy character representing the cursor, so trailing whitespace doesn't get trimmed
        text.push('_');
        let mut wrapped_text = textwrap::fill(&text, width);
        // Remove dummy character
        wrapped_text.pop();
        wrapped_text
    }

    pub fn text(&self, area: Rect) -> Text<'static> {
        let wrapped_text = self.wrapped_text(area.width as usize);
        let trailing_newline = wrapped_text.ends_with('\n');
        let mut text = Text::raw(wrapped_text);
        // Text::raw trims the last trailing newline, so we have to add it back in
        if trailing_newline {
            text.push_line("");
        }
        if text.lines.last().map(|line| line.width()).unwrap_or(0) < area.width as usize {
            text.push_span("_".on_yellow());
        } else {
            text.push_line("_".on_yellow());
        }
        text.bg(Color::Indexed(24))
    }
}

impl WidgetRef for TextArea {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.text(area).render_ref(area, buf);
    }
}
