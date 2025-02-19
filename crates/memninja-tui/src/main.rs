use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};
use ratatui::{DefaultTerminal, Frame};
use std::rc::Rc;

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

enum AppMode {
    EditingPID,
}

struct App {
    pid_text: String,
    mode: AppMode,
}

impl App {
    pub fn new() -> Self {
        Self {
            pid_text: "".into(),
            mode: AppMode::EditingPID,
        }
    }

    fn render(&self, frame: &mut Frame) {
        let [main_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .areas(frame.area());

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .areas(main_area);

        let [top_left, top_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(top);

        let [pid_area, results_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
            .areas(top_left);

        let [pid_label_area, pid_box_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .areas(pid_area);

        let [pid_input_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .margin(1)
            .areas(pid_box_area);

        let main_app_block = Block::default()
            .title("MemNinja")
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(BorderType::Thick);
        frame.render_widget(main_app_block, main_area);

        let pid_label =
            Paragraph::new(vec![Line::from("PID:")]).style(Style::default().fg(Color::White));
        frame.render_widget(pid_label, pid_label_area);

        // PID input block
        let pid_input_block = Block::bordered();
        frame.render_widget(pid_input_block, pid_box_area);

        // The PID input (currently a text field)
        let pid_input =
            Paragraph::new(self.pid_text.as_str()).style(Style::default().fg(Color::Cyan));
        frame.render_widget(pid_input, pid_input_area);

        // The Results box
        let results_block = Block::bordered().title("Results");
        frame.render_widget(results_block, results_area);
    }

    fn handle_pid_input(&mut self, code: KeyCode) {
        if let KeyCode::Char(c) = code {
            if c.is_digit(10) {
                self.pid_text = format!("{}{}", self.pid_text, c);
            }
        } else if let KeyCode::Backspace = code {
            self.pid_text.truncate(self.pid_text.len() - 1);
        }
    }

    pub fn handle_input(&mut self, code: KeyCode) {
        match self.mode {
            AppMode::EditingPID => self.handle_pid_input(code),
        }
    }
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = Rc::new(App::new());
    loop {
        let render = |frame: &mut Frame| app.render(frame);
        terminal.draw(render)?;
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            if code == KeyCode::Esc {
                return Ok(());
            }
            Rc::get_mut(&mut app).unwrap().handle_input(code);
        }
    }
}
