use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
};

#[derive(Clone)]
pub struct LogEntry {
    pub text: String,
    pub is_error: bool,
    pub line_number: Option<usize>,
}

pub struct CleanFile {}

impl CleanFile {
    pub fn new() {}

    pub fn clean_file() {
        let path = "";
        let mut logs: Vec<LogEntry> = vec![];
        let file = OpenOptions::new().read(true).open(path).unwrap();
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            match line {
                Ok(line) => {
                    todo!()
                }
                Err(e) => logs.push(LogEntry {
                    text: format!("Error at line {i} : {e}"),
                    is_error: true,
                    line_number: Some(i + 1),
                }),
            }
        }
    }
}

pub struct CleanLine {}

impl CleanLine {
    pub fn new(line: &str) {
        todo!()
    }
}
