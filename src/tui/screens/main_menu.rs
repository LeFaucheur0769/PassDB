use crate::tui::context::AppContext;
use crate::tui::screen::{Action, Screen};
use crate::tui::screens::add_db::AddDbScreen;
use crate::tui::screens::search::search_db::SearchDbScreen;
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

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
pub struct MainMenuScreen {
    state: ListState,
    logo_height: u16,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            logo_height: LOGO.lines().count() as u16 + 2,
        }
    }
}

impl Screen for MainMenuScreen {
    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        ctx: &mut AppContext,
    ) -> Option<Action> {
        let menu_items = [
            "Add a combolist",
            "Search a combolist",
            "Tools",
            "Clean duplicates",
            "Exit",
        ];

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            KeyCode::Down => {
                let i = match self.state.selected() {
                    Some(i) if i + 1 < menu_items.len() => i + 1,
                    _ => 0,
                };
                self.state.select(Some(i));
                None
            }
            KeyCode::Up => {
                let i = match self.state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => menu_items.len() - 1,
                };
                self.state.select(Some(i));
                None
            }
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    match menu_items[i] {
                        "Add a combolist" => Some(Action::Push(Box::new(AddDbScreen::new(
                            ctx.import_dir.clone(),
                            ctx.db_location.clone(),
                        )))),
                        "Search a combolist" => Some(Action::Push(Box::new(SearchDbScreen::new(
                            ctx.db_location.clone(),
                            ctx.export_dir.clone(),
                        )))),
                        "Exit" => Some(Action::Quit),
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
        let menu_items = [
            "Add a combolist",
            "Search a combolist",
            "Tools",
            "Clean duplicates",
            "Exit",
        ];

        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(self.logo_height), Constraint::Min(0)])
            .split(area);

        let logo_widget = Paragraph::new(LOGO)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(logo_widget, chunks[0]);

        let items: Vec<ListItem> = menu_items
            .iter()
            .map(|m| ListItem::new(m.to_string()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("PassDB Menu"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.state);
    }
}
