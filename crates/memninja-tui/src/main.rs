mod memninja_core;
mod widgets;

use memninja_core::{
    types::{AttachTarget, MemType, ScanType},
    CoreCommand, CoreController,
};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use ratatui::{DefaultTerminal, Frame};
use widgets::{EnumSelect, EnumSelectState};

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

enum AppMode {
    EditingPID,
}

struct App {
    pid_text: String,
    mode: AppMode,
    core_ctl: CoreController,
    scan_state: ScanState,
}

struct ScanState {
    scan_type: EnumSelectState<ScanType>,
    mem_type: EnumSelectState<MemType>,
    scan_value: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            pid_text: String::new(),
            mode: AppMode::EditingPID,
            core_ctl: CoreController::default(),
            scan_state: ScanState {
                scan_type: EnumSelectState::new(),
                mem_type: EnumSelectState::new(),
                scan_value: String::new(),
            },
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.core_ctl.start()?;
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if code == KeyCode::Esc {
                    return Ok(());
                }
                self.handle_input(code);
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let is_attached = self.core_ctl.check_attached();
        let [main_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .areas(frame.area());

        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(2)
            .areas(main_area);

        let [top_left, top_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(top);

        let [pid_area, results_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
            .areas(top_left);

        let main_app_block = Block::default()
            .title("MemNinja")
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(BorderType::Thick);
        frame.render_widget(main_app_block, main_area);

        let pid_input = Paragraph::new(self.pid_text.as_str())
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::bordered()
                    .title("Process ID")
                    .title_bottom("<a>")
                    .title_bottom("Attach")
                    .title_bottom("<d>")
                    .title_bottom("Detach")
                    .title(
                        Line::from(if is_attached {
                            "Attached"
                        } else {
                            "Not attached"
                        })
                        .style(Style::default().fg(if is_attached {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        }))
                        .right_aligned(),
                    ),
            )
            .centered();
        frame.render_widget(pid_input, pid_area);

        let results_block = Block::bordered().title("Results");
        frame.render_widget(results_block, results_area);

        // Scanner
        frame.render_widget(Block::bordered().title("Scanner"), top_right);
        let [scan_options_area, scan_value_area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .margin(1)
            .areas(top_right);

        // Scan and value type
        let [scan_type_area, mem_type_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .areas(scan_options_area);

        let scan_type = EnumSelect::<ScanType>::new("Scan Type");
        frame.render_stateful_widget(scan_type, scan_type_area, &mut self.scan_state.scan_type);
        let mem_type = EnumSelect::<MemType>::new("Value Type");
        frame.render_stateful_widget(mem_type, mem_type_area, &mut self.scan_state.mem_type);

        // Scan value filter
        let scan_value = Paragraph::new(self.scan_state.scan_value.clone()).block(
            Block::bordered()
                .title("Scan Value")
                .title_bottom(Line::from("<Enter>").right_aligned())
                .title_bottom(Line::from("Perform Scan").right_aligned()),
        );
        frame.render_widget(scan_value, scan_value_area);

        // Cheats area
        frame.render_widget(Block::bordered().title("Cheats"), bottom);
    }

    fn handle_pid_input(&mut self, code: KeyCode) {
        if let KeyCode::Char(c) = code {
            if c.is_digit(10) {
                let new_pid_text = format!("{}{}", self.pid_text, c);
                if let Ok(new_pid) = u32::from_str_radix(&new_pid_text, 10) {
                    self.pid_text = new_pid_text;
                }
            }
            match c {
                'a' => {
                    if let Ok(pid) = u32::from_str_radix(&self.pid_text, 10) {
                        let _ = self
                            .core_ctl
                            .send_command(CoreCommand::Attach(AttachTarget::Process(pid)));
                    }
                }
                'd' => {
                    let _ = self.core_ctl.send_command(CoreCommand::Detach);
                }
                _ => {}
            };
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
