use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{block::Title, Block, Paragraph, Widget},
};

#[derive(Clone)]
pub struct InputBox<'a> {
    pub text: String,
    block: Block<'a>,
}

impl<'a> InputBox<'a> {
    pub fn new() -> InputBox<'a> {
        Self {
            text: String::new(),
            block: Block::bordered(),
        }
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Title<'a>>,
    {
        self.block = self.block.title(title);
        self
    }

    pub fn title_bottom<T: Into<Line<'a>>>(mut self, title: T) -> Self {
        self.block = self.block.title_bottom(title);
        self
    }

    pub fn box_fg(mut self, color: Color) -> Self {
        self.block = self.block.style(Style::default().fg(color));
        self
    }

    pub fn box_bg(mut self, color: Color) -> Self {
        self.block = self.block.style(Style::default().bg(color));
        self
    }

    pub fn handle_input(&mut self, event: KeyEvent, validator: impl Fn(&str) -> bool) {
        if let KeyCode::Char(c) = event.code {
            let new_text = format!(
                "{}{}",
                self.text,
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            );
            if validator(&new_text) {
                self.text = new_text;
            }
        } else if let KeyCode::Backspace = event.code {
            if self.text.len() > 0 {
                self.text.truncate(self.text.len() - 1);
            }
        }
    }
}

impl<'a> Widget for &InputBox<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let input = Paragraph::new(self.text.clone()).block(self.block.clone());
        Widget::render(input, area, buf);
    }
}
