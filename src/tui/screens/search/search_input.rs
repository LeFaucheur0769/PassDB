use crate::tui::screen::{Action, Screen};
use crate::tui::context::AppContext;
use crate::tui::screens::search::search_output::SearchOutputScreen;
use crossterm::event::KeyCode;
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Paragraph}, prelude::*};
use tui_input::{Input, backend::crossterm::EventHandler};

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

pub struct SearchInputScreen {
    input: Input,
    input_mode: InputMode,
    logo_height: u16,
    db_dir: String,
    export_dir: String,
}

impl SearchInputScreen {
    pub fn new(db_dir: String, export_dir: String) -> Self {
        Self {
            input: Input::default(),
            input_mode: InputMode::Normal,
            logo_height: LOGO.lines().count() as u16 + 2,
            db_dir,
            export_dir,
        }
    }
}

impl Screen for SearchInputScreen {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent, ctx: &mut AppContext) -> Option<Action> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                    None
                }
                KeyCode::Char('q') => Some(Action::Pop),
                _ => None,
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter => {
                    let query = self.input.value().to_string();
                    self.input_mode = InputMode::Normal;
                    Some(Action::Push(Box::new(SearchOutputScreen::new(
                        query,
                        self.db_dir.clone(),
                        self.export_dir.clone(),
                    ))))
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    None
                }
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    None
                }
            },
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(self.logo_height),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        let logo = Paragraph::new(LOGO)
            .block(Block::new().borders(Borders::NONE));
        frame.render_widget(logo, chunks[0]);

        let style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        };

        let input = Paragraph::new(self.input.value())
            .style(style)
            .block(Block::bordered().title("Input (press 'e' to edit, Enter to search)"));
        frame.render_widget(input, chunks[1]);

        if self.input_mode == InputMode::Editing {
            let x = self.input.visual_cursor() + 1;
            frame.set_cursor_position((chunks[1].x + x as u16, chunks[1].y + 1));
        }
    }
}