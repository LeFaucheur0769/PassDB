use crate::logging::LogEntry;
use crate::sorter::sorter;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use glob;
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

pub struct AddCombolist {
    files: Vec<path::PathBuf>,
    current_index: usize,
    progress_current: f64,
    progress_total: f64,
    logs: Vec<LogEntry>,
    logo_height: u16,
    scroll_offset: u16,
    processed_files: HashSet<usize>, // ADD THIS FIELD
}

impl AddCombolist {
    pub fn new(import_dir: String) -> Self {
        let pattern = format!("{import_dir}/**/*");
        let files: Vec<path::PathBuf> = glob::glob(&pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();
        let logo_height = 10; // Replace LOGO.lines().count() as u16 + 2 with actual value

        // Log initial file count
        let mut logs = vec![];
        logs.push(LogEntry::info(format!(
            "Found {} files to process",
            files.len()
        )));

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

    pub fn next_file(&mut self) {
        self.progress_current = 0.0;

        if self.current_index + 1 < self.files.len() {
            self.current_index += 1;
        }

        // Update total progress based on processed files
        if !self.files.is_empty() {
            self.progress_total = self.processed_files.len() as f64 / self.files.len() as f64;
        }

        if self.current_index < self.files.len() {
            self.logs.push(LogEntry::info(format!(
                "Processing file {}/{}: {:?}",
                self.current_index + 1,
                self.files.len(),
                self.files[self.current_index]
            )));
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = layout::Layout::default()
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

        frame.render_widget(process, chunks[0]);
        frame.render_widget(total_process, chunks[1]);
        frame.render_widget(logs_render, chunks[2]);
    }
}

pub fn add_combolist<B: Backend>(
    terminal: &mut Terminal<B>,
    db_location: String,
    import_dir: String,
) -> color_eyre::Result<&'static str> {
    let mut add_combo = AddCombolist::new(import_dir.to_string());

    add_combo
        .logs
        .push(LogEntry::info("Starting processing..."));

    // Channels for communication between threads
    let (progress_tx, progress_rx) = channel::<WorkerProgress>();
    let (command_tx, command_rx) = channel::<WorkerCommand>();

    // Launch worker thread
    let db_location_clone = db_location.clone();
    let files_clone = add_combo.files.clone();
    //let total_files = files_clone.len();

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
                    add_combo.progress_current = progress;
                }
            }
            Ok(WorkerProgress::FileCompleted(file_index, hash, file_exist)) => {
                add_combo.processed_files.insert(file_index);

                if file_index < add_combo.files.len() {
                    add_combo.logs.push(LogEntry::success(format!(
                        "Finished processing file {:?} with hash {}",
                        add_combo.files[file_index].display(),
                        hash
                    )));

                    if !file_exist {
                        add_combo.logs.push(LogEntry::info(format!(
                            "New file - sorting content and adding {}",
                            hash
                        )));
                    } else { add_combo.logs.push(LogEntry::error("File has already bee imported")) }
                }

                // Update total progress
                if !add_combo.files.is_empty() {
                    add_combo.progress_total =
                        add_combo.processed_files.len() as f64 / add_combo.files.len() as f64;
                } else {
                    add_combo.logs.push(LogEntry::error("No more files to process"));
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
                            add_combo.logs.push(LogEntry::info(format!(
                                "Moving to file {}/{}: {:?}",
                                next_index + 1,
                                add_combo.files.len(),
                                add_combo.files[next_index].display()
                            )));
                        }
                    } else {
                        // is_processing = false; Was causing an issue where the message was displayed before actually finishing
                        add_combo
                            .logs
                            .push(LogEntry::success("All files processed!"));
                    }
                }
            }
            Ok(WorkerProgress::SortCompleted(file_index, result)) => match result {
                Ok(message) => {
                    if file_index <= add_combo.files.len() {
                        add_combo.logs.push(LogEntry::success(format!(
                            "Successfully sorted file {:?}: {:?}",
                            add_combo.files[file_index].display(),
                            message
                        )));
                    }
                }
                Err(e) => {
                    if file_index <= add_combo.files.len() {
                        add_combo.logs.push(LogEntry::error(format!(
                            "Sort error for file {:?}: {:?}",
                            add_combo.files[file_index].display(),
                            e
                        )));
                    }
                }
            },
            Ok(WorkerProgress::WorkerFinished) => {
                is_processing = false;
                add_combo
                    .logs
                    .push(LogEntry::success("Worker thread finished"));
                add_combo.progress_total = 1.0;
            }
            Err(TryRecvError::Empty) => {} // No new progress updates
            Err(TryRecvError::Disconnected) => {
                add_combo
                    .logs
                    .push(LogEntry::error("Worker thread disconnected"));
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
                        add_combo.logs.push(LogEntry::info("Exiting..."));
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
                        add_combo.logs.push(LogEntry::info("Clearing logs"));
                        add_combo.logs.clear();
                        add_combo.scroll_offset = 0;
                    }
                    KeyCode::Char('p') => {
                        if is_processing {
                            let _ = command_tx.send(WorkerCommand::Pause);
                            add_combo.logs.push(LogEntry::info("Processing paused"));
                        } else {
                            let _ = command_tx.send(WorkerCommand::Resume);
                            add_combo.logs.push(LogEntry::info("Processing resumed"));
                        }
                    }
                    KeyCode::Char('s') => {
                        let _ = command_tx.send(WorkerCommand::SkipCurrentFile);
                        add_combo.logs.push(LogEntry::info("Skipping current file"));
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
                            add_combo
                                .logs
                                .push(LogEntry::success("All files processed!"));
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
            terminal
                .draw(|frame| add_combo.draw(frame))
                .map_err(|e| color_eyre::eyre::eyre!("Terminal error: {}", e))
                .unwrap();
            last_ui_update = now;
        }

        // Step 4: Check if we're done
        if !is_processing {
            // Small delay to let final messages come through
            thread::sleep(Duration::from_millis(500));
            add_combo
                .logs
                .push(LogEntry::success("=== PROCESSING COMPLETE ==="));
            add_combo.logs.push(LogEntry::info("Press 'q' to exit"));
            terminal
                .draw(|frame| add_combo.draw(frame))
                .map_err(|e| color_eyre::eyre::eyre!("Terminal error: {}", e))
                .unwrap();
            // Wait for user to press 'q' before exiting
            loop {
                if event::poll(Duration::from_millis(100))?
                    && let Event::Key(key) = event::read()?
                {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                            finished = true;
                            break;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            finished = true;
                            break;
                        }
                        _ => {}
                    }
                }
                terminal
                    .draw(|frame| add_combo.draw(frame))
                    .map_err(|e| color_eyre::eyre::eyre!("Terminal error: {}", e))
                    .unwrap();
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
        //log!("before should stop");
        if should_stop {
            break;
        }
        //log!("checking file {}", file_path.to_string_lossy());

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
        //log!("Processing hash of file {}", file_path.to_string_lossy());
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
                        //log!("Here starts the sorting process");
                        if !file_exist {
                            //log!("The file did not exist and is being sorted");
                            match sorter::Sort::new(file_path, &db_location) {
                                Ok(mut sorter) => {
                                    // println!("DEBUG: Starting to sort file {:?}", file_path);
                                    let result = sorter.sort_optimised_safe();
                                    //println!(
                                    // "DEBUG: Sort result for {:?}: {:?}",
                                    // file_path, result
                                    // );
                                    if let Err(e) = progress_tx
                                        .send(WorkerProgress::SortCompleted(file_index, result))
                                    {
                                        eprintln!("DEBUG: Failed to send sort completion: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "DEBUG: Failed to create sorter for {:?}: {}",
                                        file_path, e
                                    );
                                    // Send an error result anyway
                                    let _ = progress_tx.send(WorkerProgress::SortCompleted(
                                        file_index,
                                        Err(color_eyre::eyre::eyre!(
                                            "Failed to create sorter: {}",
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

        // Check stop condition again
        if should_stop {
            break;
        }
    }

    let _ = progress_tx.send(WorkerProgress::WorkerFinished);
}
