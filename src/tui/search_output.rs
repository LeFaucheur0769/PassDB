use crate::search;
use crossterm::event::KeyCode;
use ratatui::{
    self, Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use std::io;

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

// Component that is used to render the current state of the search and the output
pub struct SearchOutput {
    logo_hight: u16,
    results: Vec<String>,
    scroll_offset: u16,
}

impl SearchOutput {
    pub fn new() -> Self {
        SearchOutput {
            logo_hight: LOGO.lines().count() as u16 + 2,
            results: vec![],
            scroll_offset: 0,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let logo_area = Constraint::Length(self.logo_hight);

        let logo = Paragraph::new(LOGO)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .border_set(symbols::border::ROUNDED)
                    .borders(Borders::ALL),
            )
            .ratio(1.0);

        // Main chunks used for the ui
        let chunks = Layout::default()
            .constraints([logo_area, Constraint::Min(0)])
            .direction(Direction::Vertical)
            .split(area);

        // Chunks used for the gauge
        let gauge_chunk = Layout::default()
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .direction(Direction::Vertical)
            .split(chunks[1]); // Spliting the already split area by using chunks

        // Chunk used for the results
        let visible_height = gauge_chunk[1].height as usize;
        let items: Vec<ListItem> = self
            .results
            .iter()
            .skip(self.scroll_offset as usize)
            .take(visible_height)
            .map(|line| ListItem::new(line.as_str()))
            .collect();
        let results = List::new(items).block(
            Block::default()
                .border_set(symbols::border::DOUBLE)
                .borders(Borders::ALL),
        );

        frame.render_widget(logo, chunks[0]);
        frame.render_widget(gauge, gauge_chunk[0]);
        frame.render_widget(results, gauge_chunk[1]);
    }

    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        email_to_search: String,
        db_dir: String,
        export_dir: String,
    ) -> io::Result<()> {
        let mut searcher =
            search::search::Searcher::new(db_dir, export_dir, email_to_search.clone())
                .map_err(io::Error::other)?;
        //self.results =
        searcher.search()?;
        self.results = (searcher.get_results()).to_vec();

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|e| io::Error::other(format!("{e}")))?;
            let event = crossterm::event::read()?;
            if let crossterm::event::Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down => {
                        // only scroll if there are more lines than the display area
                        if self.scroll_offset < self.results.len() as u16 - 1 {
                            self.scroll_offset += 1;
                        }
                    }
                    KeyCode::Up => {
                        if self.scroll_offset > 0 {
                            self.scroll_offset -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
