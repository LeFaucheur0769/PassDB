use crate::{sorter::sorter, tui};
use glob;
use ratatui::{
    self,
    Frame,
    Terminal,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    prelude::*,
    //    style::palette::tailwind,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};
use std::{io, path};

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

struct AddCombolist {
    files: Vec<path::PathBuf>,
    current_index: usize,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<String>,
    logo_height: u16,
}

impl AddCombolist {
    fn new() -> Self {
        let files: Vec<path::PathBuf> = glob::glob("/home/grimreaper/Desktop/DEV/**/*.md")
            .expect("Failed to read glob patern")
            .filter_map(Result::ok)
            .collect();
        let logo_height = LOGO.lines().count() as u16 + 2;
        AddCombolist {
            files,
            current_index: 0,
            progress_current: 0.0,
            progress_total: 0.0,
            logs: vec![],
            logo_height,
        }
    }

    fn next_file(&mut self) {
        self.progress_current = 0.0;
        if self.current_index == 280 {
            self.current_index = 280;
        } else {
            self.current_index += 1;
        }
        if self.current_index as f64 / self.files.len() as f64 == 1.0 {
            self.progress_total = 1.0;
        } else {
            self.progress_total = self.current_index as f64 / self.files.len() as f64;
        }
        self.logs.push(format!(
            "Processing file {}/{}: {:?}",
            self.current_index,
            self.files.len(),
            self.files[self.current_index - 1]
        ));
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let mut chunks = Layout::default()
            .constraints([Constraint::Min(1), Constraint::Min(1)])
            .direction(layout::Direction::Vertical)
            .margin(1)
            .split(area);
        let file_name = self
            .files
            .get(self.current_index)
            .and_then(|p| p.to_str())
            .unwrap_or("Invalid UTF-8");

        if area.height > self.logo_height + 15 {
            chunks = Layout::default()
                .constraints([
                    Constraint::Length(self.logo_height),
                    Constraint::Min(0),
                    Constraint::Min(0),
                ])
                .direction(layout::Direction::Vertical)
                .margin(1)
                .split(area);
        }
        // The gague bare of the current file
        let process = Gauge::default()
            .block(Block::default().title(file_name).borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.progress_current);

        // The total process
        let total_process = Gauge::default()
            .block(
                Block::default()
                    .title("Total processed files")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(self.progress_total)
            .label(format!("{}/{}", self.current_index, self.files.len()));

        // The logs of what is beeing processed //
        let logs = Paragraph::new(self.logs.join("\n"))
            .block(Block::default().borders(Borders::ALL))
            .scroll((self.logs.iter().count() as u16, 5))
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(process, chunks[0]);
        frame.render_widget(total_process, chunks[1]);
        frame.render_widget(logs, chunks[2]);
    }
}

// crate gauges that fills while the hashes are being processed
// create a second gauge that fills the more files are processed
fn add_combolist<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<&str> {
    let mut state = ListState::default();
    state.select(Some(0));

    let mut add_combo = AddCombolist::new();
    for i in add_combo.files.clone() {
        let mut hashfile = sorter::HashFile::new(i.to_str().unwrap())?;
        let path_str = i.to_str().unwrap();
        let mut progress_files_processed = 0.0;
        loop {
            if hashfile.update()? {
                progress_files_processed = hashfile.progress();
            } else {
                let hash = hashfile.finalize()?;
                add_combo.progress_current = progress_files_processed;
                add_combo.logs.push(format!(
                    "Finished processing file {path_str} with hash {hash}"
                ));
                break;
            }
            add_combo.progress_current = progress_files_processed;
            {
                terminal
                    .draw(|frame| add_combo.draw(frame))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
            }
            // check for quit or manual control
            if event::poll(std::time::Duration::from_millis(10))?
                && let Event::Key(key) = event::read()?
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok("quit"),
                    _ => {}
                }
            }
        }
        if add_combo.current_index + 1 < add_combo.files.len() {
            add_combo.next_file();
        }
    }
    Ok("exit")
}
