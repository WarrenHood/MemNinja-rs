pub mod input_box;

use ratatui::widgets::{block::Title, Block, Paragraph, StatefulWidget, Widget};
use std::marker::PhantomData;
use strum::IntoEnumIterator;

pub struct EnumSelect<'a, T> {
    _marker: PhantomData<T>,
    pub block: Block<'a>,
}

impl<'a, T> EnumSelect<'a, T> {
    pub fn new(title: &'a str) -> Self {
        Self {
            _marker: PhantomData,
            block: Block::bordered().title(title),
        }
    }
}

pub struct EnumSelectState<T: IntoEnumIterator> {
    choices: Vec<T>,
    index: usize,
}

impl<T> EnumSelectState<T>
where
    T: IntoEnumIterator + Clone,
{
    pub fn new() -> Self {
        Self {
            choices: T::iter().collect(),
            index: 0,
        }
    }

    pub fn select_next(&mut self) {
        if !self.choices.is_empty() {
            self.index = (self.index + 1) % self.choices.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.choices.is_empty() {
            self.index = (self.index + self.choices.len() - 1) % self.choices.len();
        }
    }

    pub fn get_value(&self) -> T {
        self.choices[self.index].clone()
    }
}

impl<'a, T> StatefulWidget for EnumSelect<'a, T>
where
    T: IntoEnumIterator + ToString,
{
    type State = EnumSelectState<T>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let text = state
            .choices
            .get(state.index)
            .map_or_else(|| "INVALID OPTION".to_string(), |choice| choice.to_string());

        let paragraph = Paragraph::new(text).block(self.block);
        Widget::render(paragraph, area, buf);
    }
}
