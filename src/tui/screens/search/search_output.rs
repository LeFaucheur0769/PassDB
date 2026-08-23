use crate::tui::screen::{Action, Screen};
use crate::tui::context::AppContext;
use crate::search;
use crossterm::event::KeyCode;
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Gauge, List, ListItem, Paragraph}, prelude::*};

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
pub struct SearchOutputScreen {
    logo_height: u16,
    results: Vec<String>,
    scroll_offset: u16,
}

impl SearchOutputScreen {
    pub fn new(email_to_search: String, db_dir: String, export_dir: String) -> Self {
        let mut screen = Self {
            logo_height: LOGO.lines().count() as u16 + 2,
            results: vec![],
            scroll_offset: 0,
        };

        // Perform the search immediately
        if let Ok(mut searcher) = search::search::Searcher::new(db_dir, export_dir, email_to_search) {
            searcher.search();
            screen.results = searcher.get_results().to_vec();
        }

        screen
    }
}

impl Screen for SearchOutputScreen {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent, ctx: &mut AppContext) -> Option<Action> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Pop),
            KeyCode::Down => {
                if self.scroll_offset < self.results.len() as u16 - 1 {
                    self.scroll_offset += 1;
                }
                None
            }
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                None
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(self.logo_height),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .direction(Direction::Vertical)
            .split(area);

        let logo = Paragraph::new(LOGO)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(logo, chunks[0]);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .block(Block::default()
                .border_set(symbols::border::ROUNDED)
                .borders(Borders::ALL))
            .ratio(1.0);
        frame.render_widget(gauge, chunks[1]);

        let visible_height = chunks[2].height as usize;
        let items: Vec<ListItem> = self.results
            .iter()
            .skip(self.scroll_offset as usize)
            .take(visible_height)
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let results = List::new(items)
            .block(Block::default()
                .border_set(symbols::border::DOUBLE)
                .borders(Borders::ALL)
                .title("Search Results"));
        frame.render_widget(results, chunks[2]);
    }
}