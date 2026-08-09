use crate::tui::SearchOutput;
use crossterm::event::KeyCode;
use ratatui::{
    self, Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::Stdout;
use std::io::{self};
use tui_input::{self, Input, backend::crossterm::EventHandler};

const LOGO: &str = r#"
                                                     
 ██▓███   ▄▄▄        ██████   ██████ ▓█████▄  ▄▄▄▄   
▓██░  ██▒▒████▄    ▒██    ▒ ▒██    ▒ ▒██▀ ██▌▓█████▄ 
▓██░ ██▓▒▒██  ▀█▄  ░ ▓██▄   ░ ▓██▄   ░██   █▌▒██▒ ▄██
▒██▄█▓▒ ▒░██▄▄▄▄██   ▒   ██▒  ▒   ██▒░▓█▄   ▌▒██░█▀  
▒██▒ ░  ░ ▓█   ▓██▒▒██████▒▒▒██████▒▒░▒████▓ ░▓█  ▀█▓
▒▓▒░ ░  ░ ▒▒   ▓▒█░▒ ▒▓▒ ▒ ░▒ ▒▓▒ ▒ ░ ▒▒▓  ▒ ░▒▓███▀▒
░▒ ░       ▒   ▒▒ ░░ ░▒  ░ ░░ ░▒  ░ ░ ░ ▒  ▒ ▒░▒   ░ 
░░         ░   ▒   ░  ░  ░  ░  ░  ░   ░ ░  ░  ░    ░ 
               ░  ░      ░        ░     ░     ░      

        Welcome to PassDB - By GrimReaper        
"#;

pub fn search_input_email(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    db_dir: String,
    export_dir: String,
) -> Result<(), std::io::Error> {
    SearchInputEmail::default().run(terminal, db_dir, export_dir)
}

#[derive(Debug, Default)]
pub struct SearchInputEmail {
    file_size: u64,
    bytes_read: u64,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<String>,
    input: Input,
    input_mode: InputMode,
    search_output: Vec<String>,
    db_dir: String,
    export_dir: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl SearchInputEmail {
    pub fn run(
        mut self,
        terminal: &mut ratatui::DefaultTerminal,
        db_dir: String,
        export_dir: String,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = crossterm::event::read()?;
            if let crossterm::event::Event::Key(key) = event {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.start_editing(),
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => self
                            .push_input(terminal, db_dir.clone(), export_dir.clone())
                            .map_err(|e| io::Error::other(format!("{e}")))?,
                        KeyCode::Esc => self.stop_editing(),
                        _ => {
                            self.input.handle_event(&event);
                        }
                    },
                }
            }
        }
    }

    fn start_editing(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    fn push_input(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        db_dir: String,
        export_dir: String,
    ) -> color_eyre::Result<()> {
        //exit(10);
        terminal
            .clear()
            .map_err(|e| io::Error::other(format!("{e}")))?;
        let mut search_output = SearchOutput::new();
        search_output
            .run(
                terminal,
                self.input.to_string(),
                db_dir.clone(),
                export_dir.clone(),
            )
            .map_err(|e| std::io::Error::other(format!("{e}")))?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let input_area = Constraint::Length(3);
        let logs_area = Constraint::Min(1);
        let logo_height = LOGO.lines().count() as u16 + 2;
        let chunks = Layout::default()
            .constraints([Constraint::Length(logo_height), input_area, logs_area])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        self.render_logo(frame, chunks[0]);
        self.render(frame, chunks[1]);
    }

    fn render_logo(&mut self, frame: &mut Frame, area: Rect) {
        let logo = Paragraph::new(LOGO).block(Block::new().borders(Borders::NONE));

        frame.render_widget(logo, area);
    }

    fn searched_output(&mut self, _frame: &mut Frame, _area: Rect) {
        //let logo =
        //   Paragraph::new(self.search_email.clone()).block(Block::new().borders(Borders::NONE));

        //frame.render_widget(logo, area);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Color::Yellow.into(),
        };
        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));

        frame.render_widget(input, area);
        if self.input_mode == InputMode::Editing {
            // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
            // end of the input text and one line down from the border to the input line
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1))
        }
    }
}
