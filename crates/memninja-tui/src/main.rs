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
use widgets::{input_box::InputBox, EnumSelect, EnumSelectState};

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(terminal);
    ratatui::restore();
    result
}

#[derive(PartialEq, Eq)]
enum AppMode {
    EditingPID,
    EditingScanValue,
    None,
}

struct App<'a> {
    should_exit: bool,
    pid_input: InputBox<'a>,
    mode: AppMode,
    core_ctl: CoreController,
    scan_state: ScanState<'a>,
}

struct ScanState<'a> {
    scan_type: EnumSelectState<ScanType>,
    mem_type: EnumSelectState<MemType>,
    scan_value: InputBox<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            should_exit: false,
            pid_input: InputBox::new()
                .title("Process ID")
                .title_bottom("<p>")
                .title_bottom("Focus")
                .title_bottom("<a>")
                .title_bottom("Attach")
                .title_bottom("<d>")
                .title_bottom("Detach"),
            mode: AppMode::None,
            core_ctl: CoreController::default(),
            scan_state: ScanState {
                scan_type: EnumSelectState::new(),
                mem_type: EnumSelectState::new(),
                scan_value: InputBox::new()
                    .title("Scan Value")
                    .title_bottom("</>")
                    .title_bottom("Focus")
                    .title_bottom(Line::from("<Enter>").right_aligned())
                    .title_bottom(Line::from("Perform Scan").right_aligned()),
            },
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.core_ctl.start()?;
        Ok(while !self.should_exit {
            terminal.draw(|frame| self.render(frame))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_input(key_event);
            }
        })
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

        let pid_input = self.pid_input.clone().title(
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
        );
        frame.render_widget(&pid_input, pid_area);

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
        frame.render_widget(&self.scan_state.scan_value, scan_value_area);

        // Cheats area
        frame.render_widget(Block::bordered().title("Cheats"), bottom);
    }

    fn update_focus_colors(&mut self) {
        self.pid_input = if self.mode == AppMode::EditingPID {
            self.pid_input.clone().box_fg(Color::Cyan)
        } else {
            self.pid_input.clone().box_fg(Color::White)
        };

        self.scan_state.scan_value = if self.mode == AppMode::EditingScanValue {
            self.scan_state.scan_value.clone().box_fg(Color::Cyan)
        } else {
            self.scan_state.scan_value.clone().box_fg(Color::White)
        };
    }

    fn handle_global_input(&mut self, event: KeyEvent) {
        if let KeyCode::Char(c) = event.code {
            match c {
                '/' => self.mode = AppMode::EditingScanValue,
                'p' => self.mode = AppMode::EditingPID,
                'q' => self.should_exit = true,
                _ => {}
            };
            self.update_focus_colors();
        }
    }

    fn handle_pid_input(&mut self, event: KeyEvent) {
        if let KeyCode::Char(c) = event.code {
            match c {
                'a' => {
                    if let Ok(pid) = u32::from_str_radix(&self.pid_input.text, 10) {
                        let _ = self
                            .core_ctl
                            .send_command(CoreCommand::Attach(AttachTarget::Process(pid)));
                        return;
                    }
                }
                'd' => {
                    let _ = self.core_ctl.send_command(CoreCommand::Detach);
                    return;
                }
                _ => {}
            };
        }
        self.pid_input
            .handle_input(event, |s| u32::from_str_radix(s, 10).is_ok());
    }

    fn handle_scan_value_input(&mut self, event: KeyEvent) {
        self.scan_state.scan_value.handle_input(event, |_| true);
    }

    pub fn handle_input(&mut self, event: KeyEvent) {
        // We can always exit an any focus by hitting esc
        if self.mode != AppMode::None {
            if KeyCode::Esc == event.code {
                self.mode = AppMode::None;
                self.update_focus_colors();
                return;
            }
        }
        match self.mode {
            AppMode::EditingPID => self.handle_pid_input(event),
            AppMode::EditingScanValue => self.handle_scan_value_input(event),
            AppMode::None => {
                self.handle_global_input(event);
            }
        }
    }
}
