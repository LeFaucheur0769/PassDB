use crate::{tui::SearchCombolist, tui::add_combolist, tui::search_input};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    self, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

//const GAUGE1_COLOR: Color = tailwind::RED.c800;
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

pub fn tui(
    import_dir: String,
    output_dir: String,
    to_sort_dir: String,
    export_dir: String,
) -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    let passdb = passdb_ui(&mut terminal)?;
    let export = export_dir.clone();

    match passdb {
        "Add a combolist" => {
            add_combolist::add_combolist(&mut terminal, output_dir, import_dir)?;
        }
        "Search a combolist" => {
            let search_combo =
                search_combolist_ui(&mut terminal, output_dir.clone(), export.clone())?;
            match search_combo.as_str() {
                "Print the output to the terminal" => {
                    search_input::search_input_email(
                        &mut terminal,
                        output_dir.clone(),
                        export.clone(),
                    )?;
                }
                "Save the output to a file" => {
                    println!("Print the output to the term");
                }
                "Exit" => println!("Exiting"),
                other => eprintln!("Unknown option: {}", other),
            }
        }
        "Tools" => println!("Tools"),
        "Clean duplicates" => println!("Clean duplicates"),
        "Exit" => std::process::exit(0),
        other => eprintln!("Unknown option: {}", other),
    }

    ratatui::restore();
    Ok(())
}

fn passdb_ui<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    let menu_items = [
        "Add a combolist",
        "Search a combolist",
        "Tools",
        "Clean duplicates",
        "Exit",
    ];

    let mut state = ListState::default();
    state.select(Some(0));
    let logo_height = LOGO.lines().count() as u16 + 2;

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
                let logo_widget = Paragraph::new(LOGO)
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
            .map_err(|e| io::Error::other(format!("{e}")))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Down => {
                    let i = match state.selected() {
                        Some(i) if i + 1 < menu_items.len() => i + 1,
                        _ => 0, // wrap back to top
                    };
                    state.select(Some(i));
                }

                KeyCode::Up => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => menu_items.len() - 1, // wrap to bottom
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

fn search_combolist_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    db_dir: String,
    export_dir: String,
) -> color_eyre::Result<String> {
    let mut search_combo = SearchCombolist::new(db_dir, export_dir);
    search_combo.run(terminal)?;
    Ok(search_combo.selected_option().to_string())
}
