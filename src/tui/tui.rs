use crate::{search, sorter::sorter};
use color_eyre::owo_colors::colors::{Yellow, xterm::DarkPurple};
use crossterm::event::{self, Event, KeyCode};
use glob;
use ratatui::{
    self, Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    io::{self, Stdout},
    path,
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
            add_combolist(&mut terminal, output_dir, import_dir)?;
        }
        "Search a combolist" => {
            let search_combo =
                search_combolist_ui(&mut terminal, output_dir.clone(), export.clone())?;
            match search_combo.as_str() {
                "Print the output to the terminal" => {
                    let _search =
                        search_input_email(&mut terminal, output_dir.clone(), export.clone())?;
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
) -> color_eyre::Result<String> {
    let mut search_combo = SearchCombolist::new(db_dir, export_dir);
    search_combo.run(terminal)?;
    Ok(search_combo.selected_option)
}
#[derive(Debug)]
enum WorkerProgress {
    FileProgress(usize, f64),                         // (file_index, progress)
    FileCompleted(usize, String, bool),               // (file_index, hash, file_exist)
    SortCompleted(usize, color_eyre::Result<String>), // (file_index, result)
    WorkerFinished,
}

#[derive(Debug)]
enum WorkerCommand {
    Stop,
    Pause,
    Resume,
    SkipCurrentFile,
}

struct AddCombolist {
    files: Vec<path::PathBuf>,
    current_index: usize,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<String>,
    logo_height: u16,
    scroll_offset: u16,
    processed_files: HashSet<usize>, // ADD THIS FIELD
}

impl AddCombolist {
    fn new(import_dir: String) -> Self {
        let pattern = format!("{import_dir}/**/*");
        let files: Vec<path::PathBuf> = glob::glob(&pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();
        let logo_height = 10; // Replace LOGO.lines().count() as u16 + 2 with actual value

        // Log initial file count
        let mut logs = vec![];
        logs.push(format!("Found {} files to process", files.len()));

        AddCombolist {
            files,
            current_index: 0,
            progress_current: 0.0,
            progress_total: 0.0,
            logs,
            logo_height,
            scroll_offset: 0,
            processed_files: HashSet::new(), // Initialize the new field
        }
    }

    fn next_file(&mut self) {
        self.progress_current = 0.0;

        if self.current_index + 1 < self.files.len() {
            self.current_index += 1;
        }

        // Update total progress based on processed files
        if !self.files.is_empty() {
            self.progress_total = self.processed_files.len() as f64 / self.files.len() as f64;
        }

        if self.current_index < self.files.len() {
            self.logs.push(format!(
                "Processing file {}/{}: {:?}",
                self.current_index + 1,
                self.files.len(),
                self.files[self.current_index]
            ));
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let mut chunks = layout::Layout::default()
            .constraints([
                Constraint::Length(3), // Current file gauge
                Constraint::Length(3), // Total progress gauge
                Constraint::Min(1),    // Logs
            ])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        let file_name = self
            .files
            .get(self.current_index)
            .and_then(|p| p.file_name())
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("No file selected");

        // Current file progress gauge
        let process = Gauge::default()
            .block(
                Block::default()
                    .title(format!("Current: {}", file_name))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.progress_current)
            .label(format!("{:.1}%", self.progress_current * 100.0));

        // Total progress gauge
        let total_files = self.files.len();
        let processed_count = self.processed_files.len();
        let total_process = Gauge::default()
            .block(
                Block::default()
                    .title(format!(
                        "Total Progress: {}/{} files",
                        processed_count, total_files
                    ))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(self.progress_total)
            .label(format!("{:.1}%", self.progress_total * 100.0));

        // Logs
        let visible_height = chunks[2].height as usize;
        let logs_start = self.scroll_offset as usize;
        let logs_end = logs_start + visible_height;
        let logs_vec: Vec<ListItem> = self
            .logs
            .iter()
            .enumerate()
            .skip(logs_start)
            .take(visible_height)
            .map(|(i, line)| ListItem::new(format!("[{}] {}", i + 1, line)))
            .collect();

        let logs_render =
            List::new(logs_vec).block(Block::default().title("Logs").borders(Borders::ALL));

        frame.render_widget(process, chunks[0]);
        frame.render_widget(total_process, chunks[1]);
        frame.render_widget(logs_render, chunks[2]);
    }
}

fn add_combolist<B: Backend>(
    terminal: &mut Terminal<B>,
    db_location: String,
    import_dir: String,
) -> color_eyre::Result<&'static str> {
    let mut add_combo = AddCombolist::new(import_dir.to_string());
    add_combo.logs.push("Starting processing...".to_string());

    // Channels for communication between threads
    let (progress_tx, progress_rx) = channel::<WorkerProgress>();
    let (command_tx, command_rx) = channel::<WorkerCommand>();

    // Launch worker thread
    let db_location_clone = db_location.clone();
    let files_clone = add_combo.files.clone();
    let total_files = files_clone.len();

    thread::spawn(move || {
        process_files_worker(files_clone, db_location_clone, progress_tx, command_rx);
    });

    let mut finished = false;
    let mut is_processing = true;
    let mut last_ui_update = Instant::now();
    let ui_update_interval = Duration::from_millis(16); // ~60 FPS

    while !finished {
        // Step 1: Check for worker progress updates
        match progress_rx.try_recv() {
            Ok(WorkerProgress::FileProgress(file_index, progress)) => {
                if file_index == add_combo.current_index {
                    add_combo.progress_current = progress as f64;
                }
            }
            Ok(WorkerProgress::FileCompleted(file_index, hash, file_exist)) => {
                add_combo.processed_files.insert(file_index);

                if file_index < add_combo.files.len() {
                    add_combo.logs.push(format!(
                        "Finished processing file {:?} with hash {}",
                        add_combo.files[file_index].display(),
                        hash
                    ));

                    if !file_exist {
                        add_combo
                            .logs
                            .push(format!("New file - sorting content and adding {}", hash));
                    }
                }

                // Update total progress
                if !add_combo.files.is_empty() {
                    add_combo.progress_total =
                        add_combo.processed_files.len() as f64 / add_combo.files.len() as f64;
                }

                // If this is the current file, move to next unprocessed one
                if file_index == add_combo.current_index {
                    // Find next unprocessed file
                    let mut next_index = add_combo.current_index;
                    let mut found = false;

                    for i in add_combo.current_index..add_combo.files.len() {
                        if !add_combo.processed_files.contains(&i) {
                            next_index = i;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        // Check from beginning
                        for i in 0..add_combo.current_index {
                            if !add_combo.processed_files.contains(&i) {
                                next_index = i;
                                found = true;
                                break;
                            }
                        }
                    }

                    if found {
                        add_combo.current_index = next_index;
                        add_combo.progress_current = 0.0;
                        if next_index < add_combo.files.len() {
                            add_combo.logs.push(format!(
                                "Moving to file {}/{}: {:?}",
                                next_index + 1,
                                add_combo.files.len(),
                                add_combo.files[next_index].display()
                            ));
                        }
                    } else {
                        is_processing = false;
                        add_combo.logs.push("All files processed!".to_string());
                    }
                }
            }
            Ok(WorkerProgress::SortCompleted(file_index, result)) => match result {
                Ok(message) => {
                    if file_index < add_combo.files.len() {
                        add_combo.logs.push(format!(
                            "Successfully sorted file {:?}: {:?}",
                            add_combo.files[file_index].display(),
                            message
                        ));
                    }
                }
                Err(e) => {
                    if file_index < add_combo.files.len() {
                        add_combo.logs.push(format!(
                            "Sort error for file {:?}: {:?}",
                            add_combo.files[file_index].display(),
                            e
                        ));
                    }
                }
            },
            Ok(WorkerProgress::WorkerFinished) => {
                is_processing = false;
                add_combo.logs.push("Worker thread finished".to_string());
                add_combo.progress_total = 1.0;
            }
            Err(TryRecvError::Empty) => {} // No new progress updates
            Err(TryRecvError::Disconnected) => {
                add_combo
                    .logs
                    .push("Worker thread disconnected".to_string());
                is_processing = false;
            }
        }

        // Step 2: Handle input (non-blocking)
        while event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Send stop command to worker
                        let _ = command_tx.send(WorkerCommand::Stop);
                        finished = true;
                        add_combo.logs.push("Exiting...".to_string());
                    }
                    KeyCode::Up => {
                        add_combo.scroll_offset = add_combo.scroll_offset.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        add_combo.scroll_offset = add_combo.scroll_offset.saturating_add(1);
                    }
                    KeyCode::PageUp => {
                        add_combo.scroll_offset = add_combo.scroll_offset.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        add_combo.scroll_offset = add_combo.scroll_offset.saturating_add(10);
                    }
                    KeyCode::Char('c') => {
                        add_combo.logs.push("Clearing logs".to_string());
                        add_combo.logs.clear();
                        add_combo.scroll_offset = 0;
                    }
                    KeyCode::Char('p') => {
                        if is_processing {
                            let _ = command_tx.send(WorkerCommand::Pause);
                            add_combo.logs.push("Processing paused".to_string());
                        } else {
                            let _ = command_tx.send(WorkerCommand::Resume);
                            add_combo.logs.push("Processing resumed".to_string());
                        }
                    }
                    KeyCode::Char('s') => {
                        let _ = command_tx.send(WorkerCommand::SkipCurrentFile);
                        add_combo.logs.push("Skipping current file".to_string());
                        add_combo.processed_files.insert(add_combo.current_index);

                        // Find next unprocessed file
                        let mut found = false;
                        for i in add_combo.current_index + 1..add_combo.files.len() {
                            if !add_combo.processed_files.contains(&i) {
                                add_combo.current_index = i;
                                add_combo.progress_current = 0.0;
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            is_processing = false;
                            add_combo.logs.push("No more files to process".to_string());
                        }

                        // Update total progress
                        if !add_combo.files.is_empty() {
                            add_combo.progress_total = add_combo.processed_files.len() as f64
                                / add_combo.files.len() as f64;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Step 3: Redraw UI at a controlled rate
        let now = Instant::now();
        if now.duration_since(last_ui_update) >= ui_update_interval {
            terminal.draw(|frame| add_combo.draw(frame));
            last_ui_update = now;
        }

        // Step 4: Check if we're done
        if !is_processing && add_combo.processed_files.len() >= add_combo.files.len() {
            // Small delay to let final messages come through
            thread::sleep(Duration::from_millis(500));
            add_combo
                .logs
                .push("=== PROCESSING COMPLETE ===".to_string());
            add_combo.logs.push("Press 'q' to exit".to_string());
            terminal.draw(|frame| add_combo.draw(frame));

            // Wait for user to press 'q' before exiting
            loop {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                                finished = true;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                terminal.draw(|frame| add_combo.draw(frame));
            }
        }

        // Small sleep to prevent busy waiting
        thread::sleep(Duration::from_millis(1));
    }

    Ok("exit")
}

// Worker thread function
fn process_files_worker(
    files: Vec<path::PathBuf>,
    db_location: String,
    progress_tx: Sender<WorkerProgress>,
    command_rx: Receiver<WorkerCommand>,
) {
    let mut paused = false;
    let mut should_stop = false;
    let mut skip_current = false;

    for (file_index, file_path) in files.iter().enumerate() {
        if should_stop {
            break;
        }

        // Reset skip flag for new file
        skip_current = false;

        // Check for commands before starting a new file
        loop {
            match command_rx.try_recv() {
                Ok(WorkerCommand::Stop) => {
                    let _ = progress_tx.send(WorkerProgress::WorkerFinished);
                    return;
                }
                Ok(WorkerCommand::Pause) => paused = true,
                Ok(WorkerCommand::Resume) => paused = false,
                Ok(WorkerCommand::SkipCurrentFile) => {
                    skip_current = true;
                    break;
                }
                Err(_) => break, // No more commands
            }
        }

        if skip_current {
            continue; // Skip this file
        }

        // Process current file
        if let Ok(mut hashfile) = sorter::HashFile::new(file_path, &db_location) {
            let mut file_processed = false;

            while !file_processed && !should_stop && !skip_current {
                // Check for commands
                loop {
                    match command_rx.try_recv() {
                        Ok(WorkerCommand::Stop) => {
                            should_stop = true;
                            break;
                        }
                        Ok(WorkerCommand::Pause) => paused = true,
                        Ok(WorkerCommand::Resume) => paused = false,
                        Ok(WorkerCommand::SkipCurrentFile) => {
                            skip_current = true;
                            file_processed = true;
                            break;
                        }
                        Err(_) => break, // No more commands
                    }
                }

                if should_stop || skip_current {
                    break;
                }

                if paused {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                // Update hash progress
                if hashfile.update().unwrap_or(false) {
                    let progress = hashfile.progress();
                    let _ = progress_tx.send(WorkerProgress::FileProgress(file_index, progress));
                    thread::sleep(Duration::from_millis(10)); // Prevent tight loop
                } else {
                    // File hash completed
                    if let Ok((hash, file_exist)) = hashfile.finalize() {
                        let _ = progress_tx.send(WorkerProgress::FileCompleted(
                            file_index,
                            hash.clone(),
                            file_exist,
                        ));

                        // If it's a new file, sort it
                        if !file_exist {
                            sorter::Sort::new(file_path, &db_location).ok().and_then(
                                |mut sorter| {
                                    let result = sorter.sort_optimised();
                                    progress_tx
                                        .send(WorkerProgress::SortCompleted(file_index, result))
                                        .ok()
                                },
                            );
                        }
                    }
                    file_processed = true;
                }
            }
        }

        // Check stop condition again
        if should_stop {
            break;
        }
    }

    let _ = progress_tx.send(WorkerProgress::WorkerFinished);
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
