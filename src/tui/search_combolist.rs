

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    self, Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph },
};
use std::{
    io::{self},
    
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


pub struct SearchCombolist {
    db_dir: String,
    export_dir: String,
    state: ListState,
    logo_height: u16,
    selected_option: String,
}

impl SearchCombolist {
    pub fn new(db_dir: String, export_dir: String) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        SearchCombolist {
            db_dir,
            export_dir,
            state,
            logo_height: LOGO.lines().count() as u16 + 2,
            selected_option: "Exit".to_string(),
        }
    }

    pub fn update_state(&mut self, state: usize) {
        self.state.select(Some(state));
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let options = [
            "Print the output to the terminal",
            "Save the output to a file",
            "Exit",
        ];

        //self.state.select(Some(0));
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
            .highlight_style(style::Style::default().fg(ratatui::style::Color::Yellow));
        frame.render_widget(logo, chunks[0]);
        frame.render_stateful_widget(menu, chunks[1], &mut self.state);
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<&str> {
        let options = [
            "Print the output to the terminal",
            "Save the output to a file",
            "Exit",
        ];

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

            if event::poll(std::time::Duration::from_millis(10))?
                && let Event::Key(key) = event::read()?
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down => {
                        let i = match self.state.selected() {
                            Some(i) if i + 1 < options.len() => i + 1,
                            _ => 0,
                        };
                        self.update_state(i);
                    }
                    KeyCode::Up => {
                        let i = match self.state.selected() {
                            Some(i) if i > 0 => i - 1,
                            _ => options.len() - 1,
                        };
                        self.update_state(i);
                    }
                    KeyCode::Enter => {
                        if let Some(i) = self.state.selected() {
                            let selected = options[i];
                            // if selected == "Exit" {
                            self.selected_option = selected.to_string();
                            break;
                            // }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(self.selected_option.as_str())
    }

        pub fn selected_option(&self) -> &str {
        &self.selected_option
    }
}
