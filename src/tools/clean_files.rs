use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
};

use crate::logging::LogEntry;

pub struct CleanFile {}

impl CleanFile {
    pub fn new() {}

    pub fn clean_file() {
        let path = ""; // TODO
        let mut logs: Vec<LogEntry> = vec![]; // Vec used to store the logs
        let file = OpenOptions::new().read(true).open(path).unwrap(); 
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            match line {
                Ok(line) => {
                    todo!()
                }
                Err(e) => logs.push(LogEntry::error(format!("Error at line {i} : {e}"))),
            }
        }
        logs.push(LogEntry::success(format!("Finished cleaning {}", path)));
    }
}

pub struct CleanLine {}

impl CleanLine {
    pub fn new(line: &str) {
        // todo!()
    }
}
