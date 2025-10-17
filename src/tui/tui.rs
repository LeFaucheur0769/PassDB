use crate::sorter::sorter;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::KeyEvent;
use glob;
use ratatui::{
    self, Terminal,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::palette::tailwind,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};
use std::{fs, io, path, time::Duration};

const GAUGE1_COLOR: Color = tailwind::RED.c800;
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

pub fn tui() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let passdb = passdb_ui(&mut terminal)?;
    // Launch the right submenu for the right seletcted submenu
    match passdb {
        "Add a combolist" => {
            //ratatui::restore();
            let _ = add_combolist(&mut terminal);
            //ratatui::init();
        }
        "Search a combolist" => {
            //terminal.clear()?;
            let _ = search_combolist_ui(&mut terminal);
        }
        "Tools" => println!("Tools"),
        "Clean duplicates" => println!("Clean duplicates"),
        "Exit" => std::process::exit(0),
        _ => eprintln!("Unknown option: {passdb}"),
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
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;

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

fn search_combolist_ui<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    // Add the combolist to the database ui

    let menu_combo = [
        "Print the output to the terminal",
        "Save the output to a file",
        "Exit",
    ];

    // Set the other variables

    let mut state = ListState::default();
    state.select(Some(0));
    let logo_height = LOGO.lines().count() as u16 + 2;

    loop {
        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
                    .margin(1)
                    .split(area);

                // Render the logo
                let logo = Paragraph::new(LOGO)
                    .alignment(ratatui::layout::Alignment::Left)
                    .block(Block::default().borders(Borders::NONE));
                frame.render_widget(logo, chunks[0]);

                // Render the menu
                let items: Vec<ListItem> = menu_combo
                    .iter()
                    .map(|m| ListItem::new(m.to_string()))
                    .collect();

                let menu = List::new(items)
                    .block(ratatui::widgets::Block::default().borders(Borders::ALL))
                    .highlight_style(
                        ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                    )
                    .highlight_symbol(">> ");

                frame.render_stateful_widget(menu, chunks[1], &mut state);
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down => {
                    let i = match state.selected() {
                        Some(i) if i + 1 < menu_combo.len() => i + 1,
                        _ => 0, // wrap back to top
                    };
                    state.select(Some(i));
                }
                KeyCode::Up => {
                    let i = match state.selected() {
                        Some(i) if i > 0 => i - 1,
                        _ => menu_combo.len() - 1, // wrap to bottom
                    };
                    state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = state.selected() {
                        let selected = menu_combo[i];
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

// crate gauges that fills while the hashes are being processed
// create a second gauge that fills the more files are processed
fn add_combolist<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    let mut state = ListState::default();
    state.select(Some(0));
    let logo_height = LOGO.lines().count() as u16 + 2;
    // let directory = fs::read_dir("tmp").unwrap();
    //let old_files: Vec<path::PathBuf> = vec![path::PathBuf::from("tmp"),path::PathBuf::from("test2"),path::PathBuf::from("test3"),];
    let files: Vec<path::PathBuf> = glob::glob("/home/grimreaper/Desktop/DEV/**/*.md")
        .expect("Failed to read glob patern")
        .filter_map(Result::ok)
        .collect();
    let mut progress_files_processed = 0.0;
    for (i, file_name) in files.iter().enumerate() {
        if i > 30 {
            break; // just when testing to not stuck the app
        }
        loop {
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let mut chunks = Layout::default()
                        .constraints([Constraint::Min(1)])
                        .direction(layout::Direction::Vertical)
                        .margin(1)
                        .split(area);
                    if area.height > logo_height + 15 {
                        chunks = Layout::default()
                            .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
                            .direction(layout::Direction::Vertical)
                            .margin(1)
                            .split(area);
                    }

                    let process = Gauge::default()
                        .block(
                            Block::default()
                                .title(file_name.to_str().unwrap_or("Invalid UTF-8"))
                                .borders(Borders::ALL),
                        )
                        .gauge_style(Style::default().fg(Color::Green))
                        .ratio(progress_files_processed);
                    let total_process = Gauge::default()
                        .block(
                            Block::default()
                                .title("Total processed files")
                                .borders(Borders::ALL),
                        )
                        .gauge_style(Style::default().fg(Color::Blue))
                        .ratio(i as f64 / files.iter().count() as f64)
                        .label(format!("{}/{}", i, files.iter().count()));

                    frame.render_widget(process, chunks[0]);
                    frame.render_widget(total_process, chunks[1]);
                })
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    // Debug to manipulate the gauges
                    KeyCode::Right => {
                        if progress_files_processed < 0.99 {
                            progress_files_processed += 0.01;
                        } else {
                            progress_files_processed = 1.0;
                        }
                    }
                    KeyCode::Left => {
                        if progress_files_processed > 0.01 {
                            progress_files_processed -= 0.01;
                        } else {
                            progress_files_processed = 0.0;
                        }
                    }

                    _ => {}
                }
            }

            // go to the next file if gauge or ui if the gauge is full
            if progress_files_processed == 1.0 {
                progress_files_processed = 0.0;
                break;
            }
        }
    }
    Ok("exit")
}

fn update(
    number_of_files: &mut u64,
    current_file_number: &mut u64,
    current_file_bytes_percent: &mut f64,
) {
    let progress_files_processed = *number_of_files / *current_file_number;
    let current_file_percent = current_file_bytes_percent;
}
