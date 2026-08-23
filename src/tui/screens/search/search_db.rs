use crate::tui::screen::{Action, Screen};
use crate::tui::context::AppContext;
use crate::tui::screens::search::search_input::SearchInputScreen;
use crossterm::event::KeyCode;
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, prelude::*};

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

pub struct SearchDbScreen {
    state: ListState,
    logo_height: u16,
    db_dir: String,
    export_dir: String,
}

impl SearchDbScreen {
    pub fn new(db_dir: String, export_dir: String) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            logo_height: LOGO.lines().count() as u16 + 2,
            db_dir,
            export_dir,
        }
    }
}

impl Screen for SearchDbScreen {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent, ctx: &mut AppContext) -> Option<Action> {
        let options = [
            "Print the output to the terminal",
            "Save the output to a file",
            "Exit",
        ];

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Pop),
            KeyCode::Down => {
                let i = match self.state.selected() {
                    Some(i) if i + 1 < options.len() => i + 1,
                    _ => 0,
                };
                self.state.select(Some(i));
                None
            }
            KeyCode::Up => {
                let i = match self.state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => options.len() - 1,
                };
                self.state.select(Some(i));
                None
            }
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    match options[i] {
                        "Print the output to the terminal" => {
                            Some(Action::Push(Box::new(SearchInputScreen::new(
                                self.db_dir.clone(),
                                self.export_dir.clone(),
                            ))))
                        }
                        "Save the output to a file" => {
                            // TODO: Implement file output
                            None
                        }
                        "Exit" => Some(Action::Pop),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let options = [
            "Print the output to the terminal",
            "Save the output to a file",
            "Exit",
        ];

        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(self.logo_height), Constraint::Min(1)])
            .split(area);

        let logo = Paragraph::new(LOGO)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().borders(Borders::NONE));

        let items: Vec<ListItem> = options
            .iter()
            .map(|m| ListItem::new(m.to_string()))
            .collect();

        let menu = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(logo, chunks[0]);
        frame.render_stateful_widget(menu, chunks[1], &mut self.state);
    }
}