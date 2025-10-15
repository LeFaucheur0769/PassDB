use color_eyre::Result;
use ratatui::{
    self, Terminal,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

pub fn tui() -> color_eyre::Result<(String)> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let passdb = passdb_ui(&mut terminal)?;
    ratatui::restore();
    Ok(passdb.to_string())
}

fn passdb_ui<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    let menu_items = vec![
        "Add a combolist",
        "Search a combolist",
        "Tools",
        "Clean duplicates",
        "Exit",
    ];

    let logo = r#"
                                                     
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

    let mut state = ListState::default();
    state.select(Some(0));
    let logo_height = logo.lines().count() as u16 + 2;

    loop {
        terminal
            .draw(|frame| {
                // defining the windows size and defining the chunks used by the different part of
                // the tui
                let area = frame.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
                    .split(area);

                // Render the logo
                let logo_widget = Paragraph::new(logo)
                    .alignment(ratatui::layout::Alignment::Left)
                    .block(Block::default().borders(Borders::NONE));
                frame.render_widget(logo_widget, chunks[0]);

                // Render the menu
                let items: Vec<ListItem> = menu_items
                    .iter()
                    .map(|m| ListItem::new(m.to_string()))
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("PassDB Menu"))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(list, chunks[1], &mut state);
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down => {
                    let i = match state.selected() {
                        Some(i) if i < menu_items.len() => i + 1,
                        _ => menu_items.len() + 1,
                    };
                    state.select(Some(i));
                }
                KeyCode::Up => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => menu_items.len() - 1,
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        let selected = menu_items[i];
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
