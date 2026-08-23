use crate::logging::LogEntry;
use crate::sorter::parquet_sorter::{export_to_parquet, init_database};
use crate::sorter::sorter;
use crate::tui::AppContext;
use crate::tui::screen::{Action, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use duckdb::Connection;
use glob;
use ratatui::widgets::{ListState, Paragraph};
use ratatui::{
    self, Frame, Terminal,
    layout::{Constraint, Direction},
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem},
};
use std::collections::HashSet;
use std::path;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;
use std::time::{Duration, Instant};
// you'll add these

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

pub struct AddDbScreen {
    files: Vec<path::PathBuf>,
    current_index: usize,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<LogEntry>,
    logo_height: u16,
    scroll_offset: u16,
    processed_files: HashSet<usize>,
    // Worker thread communication
    progress_rx: Option<Receiver<WorkerProgress>>,
    command_tx: Option<Sender<WorkerCommand>>,
    log_rx: Option<Receiver<LogEntry>>,
    is_processing: bool,
    worker_finished: bool,
}

impl AddDbScreen {
    pub fn new(import_dir: String, db_location: String) -> Self {
        let pattern = format!("{import_dir}/**/*");
        let files: Vec<path::PathBuf> = glob::glob(&pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

        let logo_height = LOGO.lines().count() as u16 + 2;

        // Log initial file count
        let mut logs = vec![];
        logs.push(LogEntry::info(format!(
            "Found {} files to process",
            files.len()
        )));

        let mut screen = Self {
            files,
            current_index: 0,
            progress_current: 0.0,
            progress_total: 0.0,
            logs,
            logo_height,
            scroll_offset: 0,
            processed_files: HashSet::new(),
            progress_rx: None,
            command_tx: None,
            log_rx: None,
            is_processing: false,
            worker_finished: false,
        };

        // Start the worker thread
        screen.start_worker(db_location);

        screen
    }

    /// Function used to start the worker as a new thread allowing the ui to keep updating
    /// Requires db_location
    fn start_worker(&mut self, db_location: String) {
        let (progress_tx, progress_rx) = channel::<WorkerProgress>();
        let (command_tx, command_rx) = channel::<WorkerCommand>();
        let (log_tx, log_rx) = channel::<LogEntry>();

        let files_clone = self.files.clone();

        self.logs.push(LogEntry::info("Starting processing..."));

        thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                process_files_worker(files_clone, db_location, progress_tx.clone(), command_rx, log_tx.clone());
            }));
            if let Err(e) = result {
                let _ = log_tx.send(LogEntry::error(format!("Worker thread panicked: {:?}", e)));
                let _ = progress_tx.send(WorkerProgress::WorkerFinished);
            }
        });

        self.progress_rx = Some(progress_rx);
        self.command_tx = Some(command_tx);
        self.log_rx = Some(log_rx);
        self.is_processing = true;
    }

    /// Function used to process progress updates
    fn process_update(&mut self) {
        if let Some(rx) = self.progress_rx.take() {
            let mut keep_rx = true;

            loop {
                match rx.try_recv() {
                    Ok(WorkerProgress::FileProgress(file_index, progress)) => {
                        if file_index == self.current_index {
                            self.progress_current = progress;
                        }
                    }
                    Ok(WorkerProgress::FileCompleted(file_index, hash, file_exist)) => {
                        self.processed_files.insert(file_index);

                        if file_index < self.files.len() {
                            self.logs.push(LogEntry::success(format!(
                                "Finished processing file {:?} with hash {}",
                                self.files[file_index].display(),
                                hash
                            )));

                            if !file_exist {
                                self.logs.push(LogEntry::info(format!(
                                    "New file - sorting content and adding {}",
                                    hash
                                )));
                            } else {
                                self.logs
                                    .push(LogEntry::error("File has already been imported"));
                            }
                        }

                        if !self.files.is_empty() {
                            self.progress_total =
                                self.processed_files.len() as f64 / self.files.len() as f64;
                        }

                        self.move_to_next_file(file_index);
                    }
                    Ok(WorkerProgress::SortCompleted(file_index, result)) => match result {
                        Ok(message) => {
                            if file_index < self.files.len() {
                                self.logs.push(LogEntry::success(format!(
                                    "Successfully sorted file {:?}: {:?}",
                                    self.files[file_index].display(),
                                    message
                                )));
                            }
                        }
                        Err(err) => {
                            if file_index < self.files.len() {
                                self.logs.push(LogEntry::error(format!(
                                    "Sort error for file {:?}: {:?}",
                                    self.files[file_index].display(),
                                    err
                                )));
                            }
                        }
                    },
                    Ok(WorkerProgress::WorkerFinished) => {
                        self.is_processing = false;
                        self.worker_finished = true;
                        self.logs.push(LogEntry::info("Worker thread finished"));
                        self.progress_total = 1.0;
                        self.logs
                            .push(LogEntry::success("=== PROCESSING COMPLETE ==="));
                        self.logs.push(LogEntry::info("Press 'q' to go back"));
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.is_processing = false;
                        // Only log this as an error if the worker never sent WorkerFinished —
                        // i.e. it disconnected unexpectedly (crash/panic) rather than finishing normally.
                        if !self.worker_finished {
                            self.logs
                                .push(LogEntry::error("Worker thread disconnected unexpectedly"));
                        }
                        keep_rx = false;
                        break;
                    }
                }
            }

            // Only keep the receiver around if the channel is still alive.
            if keep_rx {
                self.progress_rx = Some(rx);
            }
            // else: drop `rx` here, so process_update() has nothing to poll next time
            // and won't re-trigger the Disconnected branch every frame.
        }

        if let Some(rx) = self.log_rx.take() {
            while let Ok(log_entry) = rx.try_recv() {
                self.logs.push(log_entry);
            }
            self.log_rx = Some(rx);
        }
    }
    /// Function used to move to the next file to process
    fn move_to_next_file(&mut self, current_file_index: usize) {
        if current_file_index == self.current_index {
            // Find next unprocessed file
            let mut found = false;

            for i in self.current_index + 1..self.files.len() {
                if !self.processed_files.contains(&i) {
                    self.current_index = i;
                    self.progress_current = 0.0;
                    found = true;
                    break;
                }
            }

            if !found {
                // Check from beginning
                for i in 0..self.current_index {
                    if !self.processed_files.contains(&i) {
                        self.current_index = i;
                        self.progress_current = 0.0;
                        found = true;
                        break;
                    }
                }
            }

            if found {
                self.logs.push(LogEntry::info(format!(
                    "Moving to file {}/{}: {:?}",
                    self.current_index + 1,
                    self.files.len(),
                    self.files[self.current_index].display()
                )));
            }
        }
    }
    fn send_command(&self, command: WorkerCommand) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(command);
        }
    }
}

impl Screen for AddDbScreen {
    fn handle_key(&mut self, key: KeyEvent, _ctx: &mut AppContext) -> Option<Action> {
        self.process_update();

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.is_processing {
                    self.send_command(WorkerCommand::Stop);
                    self.logs
                        .push(LogEntry::info("Stopping worker and exiting..."));
                    // Give the worker a moment to stop
                    thread::sleep(Duration::from_millis(100));
                }
                Some(Action::Pop)
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                None
            }
            KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                None
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                None
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                None
            }
            KeyCode::Char('c') => {
                self.logs.push(LogEntry::info("Clearing logs"));
                self.logs.clear();
                self.scroll_offset = 0;
                None
            }

            KeyCode::Char('p') => {
                if self.is_processing {
                    self.send_command(WorkerCommand::Pause);
                    self.logs.push(LogEntry::info("Processing paused"));
                } else {
                    self.send_command(WorkerCommand::Resume);
                    self.logs.push(LogEntry::info("Processing resumed"));
                }
                None
            }
            KeyCode::Char('s') => {
                self.send_command(WorkerCommand::SkipCurrentFile);
                self.logs.push(LogEntry::info("Skipped current file"));

                // Mark current file as processed and move to next
                let current = self.current_index;
                self.processed_files.insert(current);
                self.move_to_next_file(current);

                // Update total progress
                if !self.files.is_empty() {
                    self.progress_total =
                        self.processed_files.len() as f64 / self.files.len() as f64;
                }
                None
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Process any pending updates before drawing
        self.process_update();

        let area = frame.area();
        let chunks = layout::Layout::default()
            .constraints([
                Constraint::Length(self.logo_height),
                Constraint::Length(3), // Current file gauge
                Constraint::Length(3), // Total progress gauge
                Constraint::Min(1),    // Logs
            ])
            .direction(Direction::Vertical)
            .margin(1)
            .split(area);

        // Render logo
        let logo = Paragraph::new(LOGO).block(Block::new().borders(Borders::NONE));

        // Current file progress
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
        let visible_height = chunks[3].height as usize;
        let logs_start = self.scroll_offset as usize;

        let logs_vec: Vec<ListItem> = self
            .logs
            .iter()
            .enumerate()
            .skip(logs_start)
            .take(visible_height)
            .map(|(i, line)| {
                let log_entry = Line::styled(
                    format!("[{}] {}", i + 1, line.formatted_text()),
                    line.style(),
                );
                ListItem::new(log_entry)
            })
            .collect();

        let logs_render =
            List::new(logs_vec).block(Block::default().title("Logs").borders(Borders::ALL));



        frame.render_widget(logo, chunks[0]);
        frame.render_widget(process, chunks[1]);
        frame.render_widget(total_process, chunks[2]);
        frame.render_widget(logs_render, chunks[3]);
    }
}

/// Worker thread function
fn process_files_worker(
    files: Vec<path::PathBuf>,
    db_location: String,
    progress_tx: Sender<WorkerProgress>,
    command_rx: Receiver<WorkerCommand>,
    log_tx: Sender<LogEntry>,
) {
    let mut paused = false;
    let mut should_stop = false;
    let mut skip_current = false;

    // Open the persistent database
    let db_path = format!("{}/sorted/data.db", db_location);
    let conn = match crate::sorter::parquet_sorter::init_database(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            let _ = progress_tx.send(WorkerProgress::WorkerFinished);
            return;
        }
    };

    // Process each file
    for (file_index, file_path) in files.iter().enumerate() {
        if should_stop {
            break;
        }

        skip_current = false;

        // Handle commands
        loop {
            match command_rx.try_recv() {
                Ok(WorkerCommand::Stop) => {
                    let _ = progress_tx.send(WorkerProgress::WorkerFinished);
                    return;
                }
                Ok(WorkerCommand::Pause) => {
                    paused = true;
                }
                Ok(WorkerCommand::Resume) => {
                    paused = false;
                }
                Ok(WorkerCommand::SkipCurrentFile) => {
                    skip_current = true;
                    break;
                }
                Err(_) => break,
            }
        }

        if skip_current {
            continue;
        }

        // Hash the file
        if let Ok(mut hashfile) = sorter::HashFile::new(file_path, &db_location) {
            let mut file_processed = false;

            while !file_processed && !should_stop && !skip_current {
                // Handle commands
                loop {
                    match command_rx.try_recv() {
                        Ok(WorkerCommand::Stop) => {
                            should_stop = true;
                            break;
                        }
                        Ok(WorkerCommand::Pause) => {
                            paused = true;
                        }
                        Ok(WorkerCommand::Resume) => {
                            paused = false;
                        }
                        Ok(WorkerCommand::SkipCurrentFile) => {
                            skip_current = true;
                            file_processed = true;
                            break;
                        }
                        Err(_) => break,
                    }
                }
                if should_stop || skip_current {
                    break;
                }

                if paused {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                // Update hashing
                if hashfile.update().unwrap_or(false) {
                    let progress = hashfile.progress();
                    let _ = progress_tx.send(WorkerProgress::FileProgress(file_index, progress));
                    thread::sleep(Duration::from_millis(100));
                } else {
                    // Hashing complete
                    if let Ok((hash, file_exist)) = hashfile.finalize() {
                        let _ = progress_tx.send(WorkerProgress::FileCompleted(
                            file_index,
                            hash.clone(),
                            file_exist,
                        ));

                        if !file_exist {
                            match sorter::Sort::new(file_path, &db_location) {
                                Ok(mut sorter) => {
                                    let result = sorter.sort_into_db(&conn, &log_tx);
                                    let _ = progress_tx
                                        .send(WorkerProgress::SortCompleted(file_index, result));
                                }
                                Err(e) => {
                                    let _ = progress_tx.send(WorkerProgress::SortCompleted(
                                        file_index,
                                        Err(color_eyre::eyre::eyre!(
                                            "Failed to create sorter : {}",
                                            e
                                        )),
                                    ));
                                }
                            }
                        }
                    }
                    file_processed = true;
                }
            }
        }
        if should_stop {
            break;
        }
    }

    let _ = progress_tx.send(WorkerProgress::WorkerFinished);
}
