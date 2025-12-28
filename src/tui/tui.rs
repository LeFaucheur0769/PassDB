use crate::{search, sorter::sorter};
use clap::builder::Str;
use color_eyre::owo_colors::{
    OwoColorize,
    colors::{Yellow, xterm::DarkPurple},
};
use core::time;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use glob;
use ratatui::{
    self, Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    symbols::border,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    io::{self, Stdout},
    path,
    process::exit,
    result,
    thread::sleep,
    time::Duration,
};
use tui_input::{self, Input, backend::crossterm::EventHandler};

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
    import_dir: String, // import_location
    output_dir: String, // db_location
    to_sort_dir: String,
    export_dir: String, // export_results_location
) -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    //let _ = search(&mut terminal);
    let passdb = passdb_ui(&mut terminal)?;
    // Launch the right submenu for the right seletcted submenu
    let export = export_dir.clone();
    match passdb {
        "Add a combolist" => {
            //ratatui::restore();
            let _ = add_combolist(&mut terminal, output_dir, import_dir);
            //ratatui::init();
        }
        "Search a combolist" => {
            //terminal.clear()?;
            let search_combo =
                search_combolist_ui(&mut terminal, output_dir.clone(), export.clone());
            match search_combo.as_str() {
                "Print the output to the terminal" => {
                    let search =
                        search_input_email(&mut terminal, output_dir.clone(), export.clone());
                }
                "Save the output to a file" => {
                    println!("Print the output to the term");
                }
                "Exit" => println!("Exiting"),
                _ => eprintln!("Unknown option: {search_combo}"),
            }
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

fn search_combolist_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    db_dir: String,
    export_dir: String,
) -> String {
    let mut searchCombo = SearchCombolist::new(db_dir, export_dir);
    searchCombo.run(terminal);
    return searchCombo.selected_option;
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
    fn new(import_dir: String) -> Self {
        let partern = format!("{import_dir}/**/*");
        let files: Vec<path::PathBuf> = glob::glob(&partern)
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
fn add_combolist<B: Backend>(
    terminal: &mut Terminal<B>,
    db_location: String,
    import_dir: String,
) -> io::Result<&str> {
    let mut state = ListState::default();
    state.select(Some(0));

    let mut add_combo = AddCombolist::new(import_dir.to_string());
    let mut finished = false;

    while !finished {
        // process current file
        let current_file = &add_combo.files[add_combo.current_index];
        let mut hashfile =
            sorter::HashFile::new(current_file.to_str().unwrap(), db_location.as_str())?;
        let mut progress_current_file = 0.0;

        loop {
            if hashfile.update()? {
                progress_current_file = hashfile.progress();
                add_combo.progress_current = progress_current_file;
            } else {
                let hash = hashfile.finalize()?;
                add_combo.logs.push(format!(
                    "Finished processing file {:?} with hash {}",
                    current_file, hash
                ));
                break;
            }

            // draw UI
            terminal.draw(|frame| add_combo.draw(frame));

            // optional: handle quit key
            if event::poll(std::time::Duration::from_millis(10))?
                && let Event::Key(key) = event::read()?
            {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    finished = true;

                    break;
                }
            }
        }

        // move to next file
        if add_combo.current_index + 1 < add_combo.files.len() {
            add_combo.next_file();
        } else {
            finished = true; // last file finished
            sleep(Duration::from_millis(100));
        }
    }
    Ok("exit")
}

struct SearchCombolist {
    db_dir: String,
    export_dir: String,
    state: ListState,
    logo_height: u16,
    selected_option: String,
}

impl SearchCombolist {
    fn new(db_dir: String, export_dir: String) -> Self {
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

    fn update_state(&mut self, state: usize) {
        self.state.select(Some(state));
    }

    fn draw(&mut self, frame: &mut Frame) {
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

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<&str> {
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
                            if selected == "Exit" {
                                self.selected_option = selected.to_string();
                                break;
                            } else {
                                self.selected_option = selected.to_string();
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(self.selected_option.as_str())
    }
}

// Component used to show the input box where you can specify the email you want to search
// It also register the input.

#[derive(Debug, Default)]
struct SearchInputEmail {
    file_size: u64,
    bytes_read: u64,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<String>,
    input: Input,
    input_mode: InputMode,
    search_output: Vec<String>,
    db_dir: String,
    export_dir: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl SearchInputEmail {
    fn run(
        mut self,
        terminal: &mut ratatui::DefaultTerminal,
        db_dir: String,
        export_dir: String,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = crossterm::event::read()?;
            if let crossterm::event::Event::Key(key) = event {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.start_editing(),
                        KeyCode::Char('q') => return Ok(()), // exit
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            self.push_input(terminal, db_dir.clone(), export_dir.clone())
                        }
                        KeyCode::Esc => self.stop_editing(),
                        _ => {
                            self.input.handle_event(&event);
                        }
                    },
                }
            }
        }
    }

    fn start_editing(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    fn push_input(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        db_dir: String,
        export_dir: String,
    ) {
        //exit(10);
        terminal.clear();
        let mut search_output = SearchOutput::new();
        search_output.run(
            terminal,
            self.input.to_string(),
            db_dir.clone(),
            export_dir.clone(),
        );
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let input_area = Constraint::Length(3);
        let logs_area = Constraint::Min(1);
        let logo_height = LOGO.lines().count() as u16 + 2;
        let chunks = Layout::default()
            .constraints([Constraint::Length(logo_height), input_area, logs_area])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        self.render_logo(frame, chunks[0]);
        self.render(frame, chunks[1]);
    }

    fn render_logo(&mut self, frame: &mut Frame, area: Rect) {
        let logo = Paragraph::new(LOGO).block(Block::new().borders(Borders::NONE));

        frame.render_widget(logo, area);
    }

    fn searched_output(&mut self, frame: &mut Frame, area: Rect) {
        //let logo =
        //   Paragraph::new(self.search_email.clone()).block(Block::new().borders(Borders::NONE));

        //frame.render_widget(logo, area);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Color::Yellow.into(),
        };
        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));

        frame.render_widget(input, area);
        if self.input_mode == InputMode::Editing {
            // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
            // end of the input text and one line down from the border to the input line
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1))
        }
    }
}

// Function used to execute the Search component and return the email that
// has been written.
fn search_input_email(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    db_dir: String,
    export_dir: String,
) -> Result<(), std::io::Error> {
    let result = SearchInputEmail::default().run(terminal, db_dir, export_dir);
    return result;
}

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
        let result_area = Constraint::Min(0);
        //let logs_area = "";

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
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        //self.results =
        searcher.search()?;
        self.results = (&searcher.get_results()).to_vec();

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
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
