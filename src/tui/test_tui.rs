use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    self, Terminal,
    layout::{Constraint, Layout},
    prelude::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub fn test_fun() -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    test_tui(&mut terminal)?;
    ratatui::restore();
    println!("test");
    Ok(())
}

pub fn test_tui<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    let menu_test = ["test_sorting", "test_searching"];
    let mut state = ListState::default();
    state.select(Some(0));
    loop {
        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks = Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(0)])
                    .split(area);
                let items: Vec<ListItem> = menu_test
                    .iter()
                    .map(|m| ListItem::new(m.to_string()))
                    .collect();
                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("PassDB test menu"),
                    )
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol(">>");
                frame.render_stateful_widget(list, chunks[0], &mut state);
            })
            .map_err(|e| io::Error::other(format!("{e}")))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Down => {
                    let i = match state.selected() {
                        Some(i) if i + 1 < menu_test.len() => i + 1,
                        _ => 0, // wrap back to top
                    };
                    state.select(Some(i));
                }

                KeyCode::Up => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => menu_test.len() - 1, // wrap to bottom
                    };
                    state.select(Some(i));
                }

                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        let selected = menu_test[i];
                        if selected == "Exit" {
                            break;
                        } else {
                            return Ok(selected);
                        }
                    }
                }

                _ => {}
            }
        }
    }
    Ok("exit")
}
