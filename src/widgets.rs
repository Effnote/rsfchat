use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    text::Text,
    widgets::{
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        StatefulWidgetRef, Widget, WidgetRef,
    },
};
use ratatui_macros::{horizontal, vertical};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Copy, Clone)]
pub struct TextArea {
    background: Color,
    foreground: Color,
    cursor: Color,
}

pub struct TextAreaState {
    text: String,
    scrollbar_state: ScrollbarState,
}

impl TextArea {
    pub fn new() -> TextArea {
        TextArea {
            background: Color::Indexed(18),
            foreground: Color::White,
            cursor: Color::LightYellow,
        }
    }
}

impl TextAreaState {
    pub fn new() -> Self {
        TextAreaState {
            text: String::new(),
            scrollbar_state: ScrollbarState::new(0),
        }
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
            crossterm::event::Event::Paste(data) => {
                self.paste(data);
            }
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

    fn paste(&mut self, data: &str) {
        self.text.push_str(data);
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
}

impl StatefulWidgetRef for TextArea {
    type State = TextAreaState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let word_count = state.text.unicode_words().count();
        let byte_count = state.text.len();
        let wrapped_text = state.wrapped_text(area.width as usize);
        let trailing_newline = wrapped_text.ends_with('\n');
        let mut text = Text::raw(wrapped_text);
        state.scrollbar_state = state
            .scrollbar_state
            .content_length(text.lines.len())
            .position(text.lines.len());
        // Text::raw trims the last trailing newline, so we have to add it back in
        if trailing_newline {
            text.push_line("");
        }
        if text.lines.last().map(|line| line.width()).unwrap_or(0) < area.width as usize {
            text.push_span("_".on_yellow());
        } else {
            text.push_line("_".on_yellow());
        }
        let [area, status_area] = vertical![*=1, ==1].areas(area);
        let [text_area, scrollbar_area] = horizontal![*=1, ==1].areas(area);
        let number_of_lines = text.lines.len();
        let scroll = number_of_lines.saturating_sub(text_area.height as usize);
        Paragraph::new(text)
            .scroll((scroll as u16, 0))
            .bg(self.background)
            .render_ref(text_area, buf);
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            scrollbar_area,
            buf,
            &mut state.scrollbar_state,
        );
        Text::raw(format!(
            "Words: {}    Bytes: {} / 50000    Lines: {}",
            word_count, byte_count, number_of_lines
        ))
        .render(status_area, buf);
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}
